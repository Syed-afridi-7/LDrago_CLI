//! Interactive streaming REPL and TUI command entry points for LDrago.

use anyhow::Result;
use ld_models::ModelRouter;

use crate::config::AppConfig;
use crate::repl::prompt::run_interactive_repl;

/// Execute the interactive streaming REPL session.
pub async fn execute_repl(router: &ModelRouter, config: &AppConfig) -> Result<()> {
    run_interactive_repl(router, config).await
}

/// Execute the interactive LDrago TUI / REPL session.
///
/// Routes directly to the natural terminal streaming agent REPL.
pub async fn execute_tui(router: &ModelRouter, config: &AppConfig) -> Result<()> {
    execute_repl(router, config).await
}
