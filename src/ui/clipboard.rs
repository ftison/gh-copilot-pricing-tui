use crate::data::LlmCostEntry;

/// Builds a TSV (tab-separated values) representation of the full pricing table,
/// including headers and one row per entry.
///
/// Price columns come through as plain numeric strings without a `$` symbol so
/// that spreadsheets can treat them as numbers.
#[must_use]
pub fn build_tsv(entries: &[LlmCostEntry]) -> String {
    let mut lines: Vec<String> = Vec::with_capacity(entries.len() + 1);

    lines.push(
        LlmCostEntry::headers()
            .into_iter()
            .map(cleanup_cell)
            .collect::<Vec<_>>()
            .join("\t"),
    );

    for entry in entries {
        lines.push(
            entry
                .row()
                .into_iter()
                .map(strip_dollar)
                .map(cleanup_cell)
                .collect::<Vec<_>>()
                .join("\t"),
        );
    }

    lines.join("\n")
}

/// Removes a leading `$` from a cell so numeric prices stay numeric in the
/// clipboard. Cells without a leading `$`, including "N/A", are left unchanged.
fn strip_dollar(value: impl AsRef<str>) -> String {
    let trimmed = value.as_ref().trim();
    trimmed
        .strip_prefix('$')
        .map_or_else(|| trimmed.to_owned(), std::borrow::ToOwned::to_owned)
}

/// Sanitizes a cell so embedded tabs or newlines do not break the TSV layout.
fn cleanup_cell(value: impl AsRef<str>) -> String {
    value.as_ref().replace(['\t', '\n'], " ")
}

/// Copies the given text into the system clipboard.
pub fn copy_to_clipboard(text: &str) -> Result<(), arboard::Error> {
    let mut clipboard = arboard::Clipboard::new()?;
    clipboard.set_text(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Price, ReleaseStatus, Tier};

    fn sample_entry() -> Result<LlmCostEntry, crate::error::GhLlmCostError> {
        Ok(LlmCostEntry {
            provider: "OpenAI".to_owned(),
            model: "GPT-5 mini".to_owned(),
            release_status: ReleaseStatus::Ga,
            category: "Lightweight".to_owned(),
            tier: Tier::Default,
            threshold: "Not applicable".to_owned(),
            input: Price::parse("$0.25")?,
            cached_input: Price::parse("$0.025")?,
            cache_write: Price::parse("Not applicable")?,
            output: Price::parse("$2.00")?,
        })
    }

    #[test]
    fn tsv_includes_headers_and_rows() -> Result<(), crate::error::GhLlmCostError> {
        let entries = vec![sample_entry()?];
        let tsv = build_tsv(&entries);

        let lines: Vec<&str> = tsv.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines[0],
            "Provider\tModel\tRelease status\tCategory\tTier\tThreshold\tInput\tCached input\tCache write\tOutput"
        );
        assert!(lines[1].starts_with("OpenAI\tGPT-5 mini\tGA"));
        Ok(())
    }

    #[test]
    fn tsv_escapes_tabs_and_newlines() -> Result<(), crate::error::GhLlmCostError> {
        let mut entry = sample_entry()?;
        entry.provider = "Open\tAI".to_owned();
        entry.model = "GPT\n5".to_owned();

        let tsv = build_tsv(&[entry]);
        let row = tsv.lines().nth(1).unwrap_or_default();
        let cells: Vec<&str> = row.split('\t').collect();

        assert_eq!(cells[0], "Open AI");
        assert_eq!(cells[1], "GPT 5");
        Ok(())
    }

    #[test]
    fn tsv_strips_dollar_prefix() -> Result<(), crate::error::GhLlmCostError> {
        let entries = vec![sample_entry()?];
        let tsv = build_tsv(&entries);
        let row = tsv.lines().nth(1).unwrap_or_default();
        let cells: Vec<&str> = row.split('\t').collect();

        assert_eq!(cells[6], "0.25");
        assert_eq!(cells[7], "0.03");
        assert_eq!(cells[8], "N/A");
        assert_eq!(cells[9], "2.00");
        Ok(())
    }
}
