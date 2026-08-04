use thiserror::Error;

#[derive(Debug, Error)]
pub enum GhLlmCostError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("Failed to parse pricing table: {0}")]
    Parse(String),

    #[error("No pricing data found in the fetched documentation")]
    NoData,
}
