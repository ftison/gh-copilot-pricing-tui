use crate::data::LlmCostEntry;

pub struct AppState {
    entries: Vec<LlmCostEntry>,
    selected: usize,
}

impl AppState {
    #[must_use]
    pub fn new(entries: Vec<LlmCostEntry>) -> Self {
        Self {
            entries,
            selected: 0,
        }
    }

    #[must_use]
    pub fn selected(&self) -> usize {
        self.selected.min(self.entries.len().saturating_sub(1))
    }

    #[must_use]
    pub fn visible_entries(&self) -> &[LlmCostEntry] {
        &self.entries
    }

    pub fn next(&mut self) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn page_down(&mut self) {
        let viewport_height = 20; // Safe default before dynamic sizing.
        let max = self.entries.len().saturating_sub(1);
        self.selected = (self.selected + viewport_height).min(max);
    }

    pub fn page_up(&mut self) {
        let viewport_height = 20;
        self.selected = self.selected.saturating_sub(viewport_height);
    }

    pub fn go_to_top(&mut self) {
        self.selected = 0;
    }

    pub fn go_to_bottom(&mut self) {
        if !self.entries.is_empty() {
            self.selected = self.entries.len() - 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_entries(count: usize) -> Result<Vec<LlmCostEntry>, crate::error::GhLlmCostError> {
        let base = LlmCostEntry {
            provider: "Test".to_owned(),
            model: "Model".to_owned(),
            release_status: crate::data::ReleaseStatus::Ga,
            category: "Cat".to_owned(),
            tier: crate::data::Tier::Default,
            threshold: "N/A".to_owned(),
            input: crate::data::Price::parse("$1.00")?,
            cached_input: crate::data::Price::parse("$0.10")?,
            cache_write: crate::data::Price::parse("Not applicable")?,
            output: crate::data::Price::parse("$2.00")?,
        };
        Ok(vec![base; count])
    }

    #[test]
    fn empty_state_stays_at_zero() {
        let state = AppState::new(vec![]);
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn navigation_bounds() -> Result<(), crate::error::GhLlmCostError> {
        let mut state = AppState::new(dummy_entries(3)?);
        assert_eq!(state.selected(), 0);
        state.previous();
        assert_eq!(state.selected(), 0);
        state.go_to_bottom();
        assert_eq!(state.selected(), 2);
        state.next();
        assert_eq!(state.selected(), 2);
        Ok(())
    }
}
