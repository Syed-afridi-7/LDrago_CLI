//! Claude Code-style interactive streaming prompt and REPL runner.

use anyhow::Result;
use ld_models::ModelRouter;
use ld_schemas::BrainType;
use std::env;
use std::io::{self, BufRead, Write};

use crate::agent::dispatcher::ToolDispatcher;
use crate::agent::loop_engine::AgentLoop;
use crate::config::AppConfig;
use super::commands::{parse_repl_command, ReplCommand};
use super::ReplSession;

/// Run the interactive REPL loop using custom reader and writer streams.
///
/// Enables natural terminal streaming output as well as deterministic in-memory testing.
pub async fn run_repl_loop<R: BufRead, W: Write>(
    router: &ModelRouter,
    dispatcher: &ToolDispatcher,
    mut reader: R,
    mut writer: W,
) -> Result<()> {
    let mut session = ReplSession::new();
    let agent = AgentLoop::new(router, dispatcher);

    // Print clean 1-line technical banner once
    writeln!(writer, "\n{}", super::banner::session_banner_text(session.active_brain))?;
    writeln!(writer, "Type your prompt, or use /model, /status, /help, /exit\n")?;
    writer.flush()?;

    let mut line = String::new();

    while session.running {
        write!(writer, "{}", session.prompt_prefix())?;
        writer.flush()?;

        line.clear();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            // EOF reached
            break;
        }

        let trimmed = line.trim();
        match parse_repl_command(trimmed) {
            ReplCommand::Exit => {
                writeln!(writer, "Exiting LDrago.")?;
                session.running = false;
            }
            ReplCommand::Clear => {
                write!(writer, "\x1b[2J\x1b[1;1H")?;
                writeln!(writer, "\n{}", super::banner::session_banner_text(session.active_brain))?;
                writer.flush()?;
            }
            ReplCommand::Help => {
                writeln!(writer, "\nCommands:")?;
                writeln!(writer, "  /model [coding|design|general]  - View or switch active brain")?;
                writeln!(writer, "  /status                        - Technical health & workspace status")?;
                writeln!(writer, "  /clear                         - Clear terminal screen")?;
                writeln!(writer, "  /exit                          - Exit session")?;
                writeln!(writer, "  [prompt]                       - Ask the local brain or give a coding task\n")?;
            }
            ReplCommand::Status => {
                let model_str = match session.active_brain {
                    BrainType::Coding => "Coding (Local Transformer)",
                    BrainType::Design => "Design (Local SSM Mamba)",
                    BrainType::General => "General (Ollama)",
                };
                writeln!(writer, "\n  Active Brain:    {model_str}")?;
                writeln!(writer, "  Workspace:       {}", dispatcher.workspace_root.display())?;
                writeln!(writer, "  Sandbox:         Contained (Zero-Trust Policy Active)\n")?;
            }
            ReplCommand::Model(target) => {
                if let Some(name) = target {
                    if session.set_model(&name) {
                        writeln!(writer, "Active model switched to {:?}", session.active_brain)?;
                    } else {
                        writeln!(writer, "Unknown model '{name}'. Available: coding, design, general")?;
                    }
                } else {
                    let next = session.cycle_model();
                    writeln!(writer, "Model switched to {:?}", next)?;
                }
            }
            ReplCommand::Prompt(prompt) => {
                if prompt.is_empty() {
                    continue;
                }
                match agent.run_turn(session.active_brain, &prompt).await {
                    Ok(resp) => {
                        writeln!(writer, "\n{resp}\n")?;
                    }
                    Err(e) => {
                        writeln!(writer, "\n[Error from {:?}]: {e}\n", session.active_brain)?;
                    }
                }
            }
        }
        writer.flush()?;
    }

    Ok(())
}

/// Run the interactive streaming agent REPL against standard terminal stdin and stdout.
pub async fn run_interactive_repl(router: &ModelRouter, _config: &AppConfig) -> Result<()> {
    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
    let dispatcher = ToolDispatcher::new(cwd);
    let stdin = io::stdin();
    let reader = stdin.lock();
    let stdout = io::stdout();
    run_repl_loop(router, &dispatcher, reader, stdout).await
}
