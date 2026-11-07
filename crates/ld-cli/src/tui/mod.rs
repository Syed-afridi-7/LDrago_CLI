//! Interactive Terminal User Interface (TUI) module for LDrago CLI.

pub mod app;
pub mod ui;

pub use app::{ActivePane, AppState};
pub use ui::render;

use ratatui::backend::Backend;
use ratatui::Terminal;
use std::io;

/// Render an interactive frame to the given terminal instance.
pub fn render_to_terminal<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &AppState,
) -> io::Result<()> {
    terminal.draw(|frame| {
        ui::render(frame, app);
    })?;
    Ok(())
}
