use std::collections::HashMap;

use crate::error::GhLlmCostError;

pub mod fetch;
pub mod parse;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tier {
    Default,
    LongContext,
    NotApplicable,
}

impl Tier {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Tier::Default => "Default",
            Tier::LongContext => "Long context",
            Tier::NotApplicable => "N/A",
        }
    }
}

impl TryFrom<&str> for Tier {
    type Error = GhLlmCostError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "Default" => Ok(Tier::Default),
            "Long context" => Ok(Tier::LongContext),
            "Not applicable" => Ok(Tier::NotApplicable),
            other => Err(GhLlmCostError::Parse(format!("Unknown tier: {other}"))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReleaseStatus {
    Ga,
    PublicPreview,
    Preview,
}

impl ReleaseStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            ReleaseStatus::Ga => "GA",
            ReleaseStatus::PublicPreview => "Public preview",
            ReleaseStatus::Preview => "Preview",
        }
    }
}

impl TryFrom<&str> for ReleaseStatus {
    type Error = GhLlmCostError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "GA" => Ok(ReleaseStatus::Ga),
            "Public preview" => Ok(ReleaseStatus::PublicPreview),
            "Preview" => Ok(ReleaseStatus::Preview),
            other => Err(GhLlmCostError::Parse(format!(
                "Unknown release status: {other}"
            ))),
        }
    }
}

/// Represents a price in USD per 1 million tokens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Price(Option<f64>);

impl Price {
    /// Parses a price string like "$1.75" or "Not applicable".
    ///
    /// # Errors
    ///
    /// Returns an error if the string is neither a valid dollar amount nor a
    /// recognized "not applicable" sentinel.
    pub fn parse(value: &str) -> Result<Self, GhLlmCostError> {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("Not applicable") {
            return Ok(Self(None));
        }

        let without_dollar = trimmed
            .strip_prefix('$')
            .ok_or_else(|| GhLlmCostError::Parse(format!("Missing '$' in price: {trimmed}")))?;

        let parsed = without_dollar
            .replace(',', "")
            .parse::<f64>()
            .map_err(|e| GhLlmCostError::Parse(format!("Invalid price '{trimmed}': {e}")))?;

        if parsed.is_sign_negative() {
            return Err(GhLlmCostError::Parse(format!(
                "Negative prices are not allowed: {trimmed}"
            )));
        }

        Ok(Self(Some(parsed)))
    }

    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        self.0
    }

    #[must_use]
    pub fn display(&self) -> String {
        match self.0 {
            None => "N/A".to_owned(),
            Some(v) => format!("${v:.2}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LlmCostEntry {
    pub provider: String,
    pub model: String,
    pub release_status: ReleaseStatus,
    pub category: String,
    pub tier: Tier,
    pub threshold: String,
    pub input: Price,
    pub cached_input: Price,
    pub cache_write: Price,
    pub output: Price,
}

impl LlmCostEntry {
    #[must_use]
    pub fn headers() -> Vec<&'static str> {
        vec![
            "Provider",
            "Model",
            "Release status",
            "Category",
            "Tier",
            "Threshold",
            "Input",
            "Cached input",
            "Cache write",
            "Output",
        ]
    }

    #[must_use]
    pub fn row(&self) -> Vec<String> {
        vec![
            self.provider.clone(),
            self.model.clone(),
            self.release_status.as_str().to_owned(),
            self.category.clone(),
            self.tier.as_str().to_owned(),
            self.threshold.clone(),
            self.input.display(),
            self.cached_input.display(),
            self.cache_write.display(),
            self.output.display(),
        ]
    }
}

#[derive(Debug, Clone, Default)]
pub struct PricingData {
    pub entries: Vec<LlmCostEntry>,
}

impl PricingData {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns the index of each entry grouped by provider.
    #[must_use]
    pub fn provider_boundaries(&self) -> HashMap<String, Vec<usize>> {
        let mut map: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, entry) in self.entries.iter().enumerate() {
            map.entry(entry.provider.clone()).or_default().push(idx);
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_parses_dollar_amount() -> Result<(), GhLlmCostError> {
        let price = Price::parse("$1.75")?;
        assert!((price.as_f64().unwrap_or(f64::NAN) - 1.75).abs() < f64::EPSILON);
        Ok(())
    }

    #[test]
    fn price_parses_not_applicable() -> Result<(), GhLlmCostError> {
        let price = Price::parse("Not applicable")?;
        assert!(price.as_f64().is_none());
        Ok(())
    }

    #[test]
    fn price_rejects_negative() {
        let result = Price::parse("$-1.00");
        assert!(result.is_err());
    }

    #[test]
    fn tier_parsing() -> Result<(), GhLlmCostError> {
        assert_eq!(Tier::try_from("Default")?, Tier::Default);
        assert_eq!(Tier::try_from("Long context")?, Tier::LongContext);
        assert_eq!(Tier::try_from("Not applicable")?, Tier::NotApplicable);
        Ok(())
    }

    #[test]
    fn release_status_parsing() -> Result<(), GhLlmCostError> {
        assert_eq!(
            ReleaseStatus::try_from("Public preview")?,
            ReleaseStatus::PublicPreview
        );
        Ok(())
    }
}
