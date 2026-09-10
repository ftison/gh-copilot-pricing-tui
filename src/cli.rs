use crate::data::DecimalSeparator;
use crate::error::GhLlmCostError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CliConfig {
    pub decimal_separator: DecimalSeparator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliAction {
    Run(CliConfig),
    Help(String),
    Version(String),
}

/// Parses command-line arguments.
///
/// # Errors
///
/// Returns an error if an unknown option is encountered, an option argument is missing,
/// or a decimal separator is invalid.
pub fn parse_args<I>(args: I) -> Result<CliAction, GhLlmCostError>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    // Skip binary name if present.
    let _bin = args.next();

    let mut decimal_separator = DecimalSeparator::default();

    while let Some(arg) = args.next() {
        if arg == "-h" || arg == "--help" {
            return Ok(CliAction::Help(help_message()));
        }
        if arg == "-V" || arg == "--version" {
            return Ok(CliAction::Version(format!(
                "{} {}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            )));
        }
        if arg == "-d" || arg == "--decimal-separator" {
            let val = args.next().ok_or_else(|| {
                GhLlmCostError::Cli(format!("Option '{arg}' requires an argument"))
            })?;
            decimal_separator = val.parse()?;
        } else if let Some(val) = arg.strip_prefix("--decimal-separator=") {
            decimal_separator = val.parse()?;
        } else if let Some(val) = arg.strip_prefix("-d=") {
            decimal_separator = val.parse()?;
        } else {
            return Err(GhLlmCostError::Cli(format!("Unknown option: {arg}")));
        }
    }

    Ok(CliAction::Run(CliConfig { decimal_separator }))
}

#[must_use]
pub fn help_message() -> String {
    format!(
        "Usage: {} [OPTIONS]\n\n\
        Options:\n  \
          -d, --decimal-separator <CHAR>  Decimal separator for numeric values ('.' or ',', default: '.')\n  \
          -h, --help                      Print help\n  \
          -V, --version                   Print version",
        env!("CARGO_PKG_NAME")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_default_args() -> Result<(), GhLlmCostError> {
        let action = parse_args(vec!["gh_copilot_pricing_tui".to_owned()])?;
        assert_eq!(
            action,
            CliAction::Run(CliConfig {
                decimal_separator: DecimalSeparator::Dot
            })
        );
        Ok(())
    }

    #[test]
    fn parse_decimal_separator_long() -> Result<(), GhLlmCostError> {
        let action = parse_args(vec![
            "gh_copilot_pricing_tui".to_owned(),
            "--decimal-separator".to_owned(),
            ",".to_owned(),
        ])?;
        assert_eq!(
            action,
            CliAction::Run(CliConfig {
                decimal_separator: DecimalSeparator::Comma
            })
        );
        Ok(())
    }

    #[test]
    fn parse_decimal_separator_equals() -> Result<(), GhLlmCostError> {
        let action = parse_args(vec![
            "gh_copilot_pricing_tui".to_owned(),
            "--decimal-separator=,".to_owned(),
        ])?;
        assert_eq!(
            action,
            CliAction::Run(CliConfig {
                decimal_separator: DecimalSeparator::Comma
            })
        );
        Ok(())
    }

    #[test]
    fn parse_decimal_separator_short() -> Result<(), GhLlmCostError> {
        let action = parse_args(vec![
            "gh_copilot_pricing_tui".to_owned(),
            "-d".to_owned(),
            ".".to_owned(),
        ])?;
        assert_eq!(
            action,
            CliAction::Run(CliConfig {
                decimal_separator: DecimalSeparator::Dot
            })
        );
        Ok(())
    }

    #[test]
    fn parse_decimal_separator_invalid() {
        let result = parse_args(vec![
            "gh_copilot_pricing_tui".to_owned(),
            "-d".to_owned(),
            ";".to_owned(),
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_decimal_separator_missing_value() {
        let result = parse_args(vec!["gh_copilot_pricing_tui".to_owned(), "-d".to_owned()]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_unknown_option() {
        let result = parse_args(vec![
            "gh_copilot_pricing_tui".to_owned(),
            "--foo".to_owned(),
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_help_flag() -> Result<(), GhLlmCostError> {
        let action = parse_args(vec![
            "gh_copilot_pricing_tui".to_owned(),
            "--help".to_owned(),
        ])?;
        assert!(matches!(action, CliAction::Help(_)));
        Ok(())
    }
}
