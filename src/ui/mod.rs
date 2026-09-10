use crate::data::{DecimalSeparator, LlmCostEntry};

pub mod clipboard;

/// Builds a TSV representation of the full pricing table including headers.
#[must_use]
pub fn build_table_tsv(entries: &[LlmCostEntry], separator: DecimalSeparator) -> String {
    clipboard::build_tsv(entries, separator)
}

/// Copies the given text to the system clipboard.
///
/// # Errors
///
/// Returns an error if the system clipboard cannot be accessed or written.
pub fn copy_to_clipboard(text: &str) -> Result<(), arboard::Error> {
    clipboard::copy_to_clipboard(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_table_has_only_header_row() {
        let tsv = build_table_tsv(&[], DecimalSeparator::Dot);
        assert_eq!(tsv.lines().count(), 1);
    }
}
