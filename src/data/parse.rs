use std::collections::HashMap;

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::data::{LlmCostEntry, Price, PricingData, ReleaseStatus, Tier};
use crate::error::GhLlmCostError;

/// Parses the GitHub Markdown documentation and extracts pricing tables.
///
/// # Errors
///
/// Returns an error if a table cannot be parsed or contains unexpected data.
pub fn parse_documentation(markdown: &str) -> Result<PricingData, GhLlmCostError> {
    let mut entries = Vec::new();
    let mut current_provider = String::new();

    let mut in_h3 = false;
    let mut _in_table = false;
    let mut in_table_head = false;
    let mut in_table_row = false;
    let mut in_table_cell = false;

    let mut header_map: HashMap<String, usize> = HashMap::new();
    let mut current_headers: Vec<String> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut current_cell: String = String::new();

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    for event in Parser::new_ext(markdown, options) {
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H3,
                ..
            }) => {
                in_h3 = true;
                current_provider.clear();
            }
            Event::Text(text) | Event::Code(text) if in_h3 => {
                current_provider.push_str(&text);
            }
            Event::End(TagEnd::Heading(HeadingLevel::H3)) => {
                in_h3 = false;
            }
            Event::Start(Tag::Table { .. }) => {
                _in_table = true;
                header_map.clear();
                current_headers.clear();
                current_row.clear();
                current_cell.clear();
            }
            Event::End(TagEnd::Table) => {
                _in_table = false;
                header_map.clear();
                current_headers.clear();
                current_row.clear();
                current_cell.clear();
            }
            Event::Start(Tag::TableHead) => {
                in_table_head = true;
                current_headers.clear();
            }
            Event::End(TagEnd::TableHead) => {
                in_table_head = false;
                for (idx, header) in current_headers.iter().enumerate() {
                    header_map.insert(normalize_header(header), idx);
                }
            }
            Event::Start(Tag::TableRow) => {
                in_table_row = true;
                current_row.clear();
                current_cell.clear();
            }
            Event::End(TagEnd::TableRow) => {
                in_table_row = false;
                if !current_row.is_empty()
                    && let Some(entry) = build_entry(&current_provider, &current_row, &header_map)?
                {
                    entries.push(entry);
                }
                current_row.clear();
                current_cell.clear();
            }
            Event::Start(Tag::TableCell) => {
                in_table_cell = true;
                current_cell.clear();
            }
            Event::End(TagEnd::TableCell) => {
                in_table_cell = false;
                let value = current_cell.trim().to_owned();
                if in_table_head {
                    current_headers.push(value);
                } else if in_table_row {
                    current_row.push(value);
                }
                current_cell.clear();
            }
            Event::Text(text) | Event::Code(text) if in_table_cell => {
                current_cell.push_str(&text);
            }
            Event::SoftBreak | Event::HardBreak if in_table_cell => {
                current_cell.push(' ');
            }
            Event::Start(Tag::Emphasis) | Event::End(TagEnd::Emphasis) => {
                // Markdown emphasis markers (asterisks) appear in some header
                // text on the GitHub page. They are ignored so the raw text is
                // preserved.
            }
            _ => {}
        }
    }

    if entries.is_empty() {
        return Err(GhLlmCostError::NoData);
    }

    Ok(PricingData { entries })
}

fn normalize_header(header: &str) -> String {
    header
        .trim()
        .to_lowercase()
        .replace(' ', "_")
        .replace(['(', ')'], "")
}

fn build_entry(
    provider: &str,
    row: &[String],
    header_map: &HashMap<String, usize>,
) -> Result<Option<LlmCostEntry>, GhLlmCostError> {
    // Skip separator-only rows produced by some markdown tables.
    if row.iter().all(|cell| {
        cell.chars()
            .all(|c| c == '-' || c == '|' || c.is_whitespace())
    }) {
        return Ok(None);
    }

    let provider = provider.trim().to_owned();
    if provider.is_empty() {
        return Err(GhLlmCostError::Parse(
            "Pricing entry found without a provider heading".to_owned(),
        ));
    }

    let get = |name: &str| -> Result<String, GhLlmCostError> {
        let idx = header_map
            .get(name)
            .copied()
            .ok_or_else(|| GhLlmCostError::Parse(format!("Missing column: {name}")))?;
        row.get(idx)
            .map(|s| s.trim().to_owned())
            .ok_or_else(|| GhLlmCostError::Parse(format!("Row too short, missing {name}")))
    };

    // Some providers do not have a cache write column.
    let cache_write = if header_map.contains_key("cache_write") {
        Price::parse(&get("cache_write")?)?
    } else {
        Price::parse("Not applicable")?
    };

    // Tier and threshold are also optional depending on the provider.
    let tier = if header_map.contains_key("tier") {
        Tier::try_from(get("tier")?.as_str())?
    } else {
        Tier::NotApplicable
    };

    let threshold = if header_map.contains_key("threshold_input_tokens") {
        get("threshold_input_tokens")?
    } else {
        "N/A".to_owned()
    };

    let entry = LlmCostEntry {
        provider,
        model: get("model")?,
        release_status: ReleaseStatus::try_from(get("release_status")?.as_str())?,
        category: get("category")?,
        tier,
        threshold,
        input: Price::parse(&get("input")?)?,
        cached_input: Price::parse(&get("cached_input")?)?,
        cache_write,
        output: Price::parse(&get("output")?)?,
    };

    Ok(Some(entry))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
### OpenAI

| Model      | Release status | Category | Tier         | Threshold (input tokens) |  Input | Cached input | Cache write | Output |
| ---------- | -------------- | -------- | ------------ | ------------------------ | -----: | -----------: | ----------: | -----: |
| GPT-5 mini | GA             | Lightweight | Default   | Not applicable           |  $0.25 |       $0.025 | Not applicable |  $2.00 |

### Moonshot AI

| Model          | Release status | Category  | Input | Cached input | Output |
| -------------- | -------------- | --------- | ----: | -----------: | -----: |
| Kimi K2.7 Code | GA             | Versatile | $0.95 |        $0.19 |  $4.00 |
"#;

    #[test]
    fn parse_sample_documentation() {
        let data = parse_documentation(SAMPLE).unwrap();
        assert_eq!(data.len(), 2);

        let openai = data
            .entries
            .iter()
            .find(|e| e.provider == "OpenAI")
            .unwrap();
        assert_eq!(openai.model, "GPT-5 mini");
        assert_eq!(openai.tier, Tier::Default);
        assert!((openai.input.as_f64().unwrap() - 0.25).abs() < f64::EPSILON);
        assert!(openai.cache_write.as_f64().is_none());

        let moonshot = data
            .entries
            .iter()
            .find(|e| e.provider == "Moonshot AI")
            .unwrap();
        assert_eq!(moonshot.model, "Kimi K2.7 Code");
        assert_eq!(moonshot.tier, Tier::NotApplicable);
    }

    #[test]
    fn normalize_header_strips_punctuation_and_spaces() {
        assert_eq!(
            normalize_header("Threshold (input tokens)"),
            "threshold_input_tokens"
        );
        assert_eq!(normalize_header("Cached input"), "cached_input");
    }
}
