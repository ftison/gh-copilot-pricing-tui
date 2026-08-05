use std::io;

use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEventKind};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};
use ratatui::{Frame, Terminal};

use crate::data::LlmCostEntry;
use crate::error::GhLlmCostError;
use crate::ui::{build_table_tsv, copy_to_clipboard};

pub mod state;

pub use state::AppState;

/// Runs the interactive TUI until the user quits.
///
/// # Errors
///
/// Returns an error if the terminal cannot be initialized, restored, or if an
/// I/O error occurs while reading events.
pub async fn run_tui(entries: Vec<LlmCostEntry>) -> Result<(), GhLlmCostError> {
    let mut terminal = setup_terminal()?;
    let mut app_state = AppState::new(entries);

    let result = run_loop(&mut terminal, &mut app_state).await;

    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, GhLlmCostError> {
    crossterm::terminal::enable_raw_mode()
        .map_err(|e| GhLlmCostError::Terminal(format!("Failed to enable raw mode: {e}")))?;
    let mut stdout = io::stdout();
    crossterm::execute!(&mut stdout, crossterm::terminal::EnterAlternateScreen)
        .map_err(|e| GhLlmCostError::Terminal(format!("Failed to enter alternate screen: {e}")))?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
        .map_err(|e| GhLlmCostError::Terminal(format!("Failed to create terminal: {e}")))
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), GhLlmCostError> {
    crossterm::terminal::disable_raw_mode()
        .map_err(|e| GhLlmCostError::Terminal(format!("Failed to disable raw mode: {e}")))?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )
    .map_err(|e| GhLlmCostError::Terminal(format!("Failed to leave alternate screen: {e}")))?;
    terminal
        .show_cursor()
        .map_err(|e| GhLlmCostError::Terminal(format!("Failed to show cursor: {e}")))?;
    Ok(())
}

async fn run_loop<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &mut AppState,
) -> Result<(), GhLlmCostError> {
    let mut redraw = true;

    loop {
        if redraw {
            terminal
                .draw(|frame| draw(frame, state))
                .map_err(|e| GhLlmCostError::Terminal(format!("Failed to draw: {e}")))?;
            redraw = false;
        }

        if event::poll(std::time::Duration::from_millis(100)).map_err(GhLlmCostError::Io)?
            && let CrosstermEvent::Key(key) = event::read().map_err(GhLlmCostError::Io)?
        {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Down | KeyCode::Char('j') => {
                    state.next();
                    redraw = true;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    state.previous();
                    redraw = true;
                }
                KeyCode::PageDown => {
                    state.page_down();
                    redraw = true;
                }
                KeyCode::PageUp => {
                    state.page_up();
                    redraw = true;
                }
                KeyCode::Home => {
                    state.go_to_top();
                    redraw = true;
                }
                KeyCode::End => {
                    state.go_to_bottom();
                    redraw = true;
                }
                KeyCode::Char('c') => {
                    let tsv = build_table_tsv(state.entries());
                    match copy_to_clipboard(&tsv) {
                        Ok(()) => state.set_status(format!(
                            "Copied {} rows to clipboard",
                            state.entries().len()
                        )),
                        Err(e) => state.set_status(format!("Failed to copy: {e}")),
                    }
                    redraw = true;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn draw(frame: &mut Frame, state: &mut AppState) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(area);

    let header_cells: Vec<Cell> = LlmCostEntry::headers()
        .into_iter()
        .map(|title| {
            Cell::from(title).style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        })
        .collect();
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = state
        .visible_entries()
        .iter()
        .map(|entry| {
            let cells: Vec<Cell> = entry.row().into_iter().map(Cell::from).collect();
            Row::new(cells).height(1)
        })
        .collect();

    let widths: Vec<Constraint> = vec![
        Constraint::Length(14),
        Constraint::Length(28),
        Constraint::Length(16),
        Constraint::Length(12),
        Constraint::Length(14),
        Constraint::Length(26),
        Constraint::Length(10),
        Constraint::Length(14),
        Constraint::Length(12),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" GitHub Copilot LLM Pricing ")
                .title_style(Style::default().bold()),
        )
        .row_highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("> ");

    let mut table_state = TableState::default();
    table_state.select(Some(state.selected()));
    frame.render_stateful_widget(table, chunks[0], &mut table_state);

    let help_text = if let Some(status) = state.take_status() {
        format!(
            "{status} | q/Esc: quit | ↑/k ↓/j: move | PgUp/PgDown: page | Home/End: top/bottom | c: copy"
        )
    } else {
        "q/Esc: quit | ↑/k ↓/j: move | PgUp/PgDown: page | Home/End: top/bottom | c: copy"
            .to_owned()
    };
    let help =
        Paragraph::new(help_text).block(Block::default().borders(Borders::ALL).title(" Help "));
    frame.render_widget(help, chunks[1]);
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
    fn app_state_navigation() -> Result<(), crate::error::GhLlmCostError> {
        let entries = vec![sample_entry()?; 5];
        let mut state = AppState::new(entries);

        assert_eq!(state.selected(), 0);
        state.next();
        assert_eq!(state.selected(), 1);
        state.previous();
        assert_eq!(state.selected(), 0);
        state.go_to_bottom();
        assert_eq!(state.selected(), 4);
        state.next();
        assert_eq!(state.selected(), 4);
        Ok(())
    }

    #[test]
    fn headers_match_row_length() -> Result<(), crate::error::GhLlmCostError> {
        let entry = sample_entry()?;
        assert_eq!(LlmCostEntry::headers().len(), entry.row().len());
        Ok(())
    }
}
