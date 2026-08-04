use crate::error::GhLlmCostError;

const URL: &str =
    "https://docs.github.com/fr/copilot/reference/copilot-billing/models-and-pricing.md";

/// Fetches the GitHub Copilot pricing documentation as Markdown.
///
/// # Errors
///
/// Returns an error if the HTTP request fails or the body cannot be read.
pub async fn fetch_documentation() -> Result<String, GhLlmCostError> {
    let client = reqwest::Client::builder()
        .user_agent("gh-llm-cost-table/0.1.0")
        .build()?;

    let response = client.get(URL).send().await?;
    if !response.status().is_success() {
        return Err(GhLlmCostError::Terminal(format!(
            "GitHub returned HTTP {}",
            response.status()
        )));
    }

    let body = response.text().await?;
    Ok(body)
}

#[cfg(test)]
mod tests {
    // We deliberately avoid network calls in unit tests.
    use super::URL;

    #[test]
    fn url_is_valid() {
        assert!(URL.parse::<reqwest::Url>().is_ok());
    }
}
