//! Claude Code-style interactive streaming prompt and REPL runner.

use anyhow::Result;
use ld_models::ModelRouter;
use ld_schemas::BrainType;
use std::env;
use std::io::{self, BufRead, Write};
use std::process::Command;

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

    let border = "\x1b[38;2;75;85;99m";
    let cyan = "\x1b[38;2;0;220;255m";
    let green = "\x1b[38;2;34;197;94m";
    let dim = "\x1b[38;2;156;163;175m";
    let bold_white = "\x1b[1;37m";
    let reset = "\x1b[0m";

    // Print Claude Code-style framed header once
    writeln!(writer, "\n{}", super::banner::session_banner_text(session.active_brain))?;
    writeln!(writer, "{dim}Type your prompt, or use /model, /status, /help, /exit{reset}\n")?;
    writer.flush()?;

    let mut line = String::new();

    while session.running {
        // Styled prompt box
        let label = match session.active_brain {
            BrainType::Coding => "Coding",
            BrainType::Design => "Design",
            BrainType::General => "General",
        };
        write!(
            writer,
            "{border}╭─{reset} {cyan}●{reset} {bold_white}[{label}]{reset} {border}─────────────────────────────────────────╮{reset}\n{border}│{reset} {}",
            session.prompt_prefix()
        )?;
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
                writeln!(writer, "{dim}Exiting LDrago.{reset}")?;
                session.running = false;
            }
            ReplCommand::Clear => {
                write!(writer, "\x1b[2J\x1b[1;1H")?;
                writeln!(writer, "\n{}", super::banner::session_banner_text(session.active_brain))?;
                writer.flush()?;
            }
            ReplCommand::Help => {
                writeln!(
                    writer,
                    "\n{border}╭─ Help & Commands ────────────────────────────────────────────────────────╮{reset}\n\
                     {border}│{reset}  {cyan}/model [coding|design|general]{reset}  - View or switch active brain\n\
                     {border}│{reset}  {cyan}/status{reset}                        - Technical health & workspace status\n\
                     {border}│{reset}  {cyan}/clear{reset}                         - Clear terminal screen\n\
                     {border}│{reset}  {cyan}/exit{reset}                          - Exit session\n\
                     {border}│{reset}  {dim}[prompt]{reset}                       - Ask the local brain or give a coding task\n\
                     {border}╰──────────────────────────────────────────────────────────────────────────╯{reset}\n"
                )?;
            }
            ReplCommand::Status => {
                let model_str = match session.active_brain {
                    BrainType::Coding => "Coding (Local Transformer)",
                    BrainType::Design => "Design (Local SSM Mamba)",
                    BrainType::General => "General (Ollama)",
                };
                writeln!(
                    writer,
                    "\n{border}╭─ System Status ──────────────────────────────────────────────────────────╮{reset}\n\
                     {border}│{reset}  Active Brain:    {model_str}\n\
                     {border}│{reset}  Workspace:       {}\n\
                     {border}│{reset}  Sandbox:         Contained (Zero-Trust Policy Active)\n\
                     {border}╰──────────────────────────────────────────────────────────────────────────╯{reset}\n",
                    dispatcher.workspace_root.display()
                )?;
            }
            ReplCommand::Model(target) => {
                if let Some(name) = target {
                    if session.set_model(&name) {
                        writeln!(
                            writer,
                            "\n{border}╭─ Model Switcher ─────────────────────────────────────────────────────────╮{reset}\n\
                             {border}│{reset}  {green}✔{reset} Active model switched to {:?} (Model switched to {:?})\n\
                             {border}╰──────────────────────────────────────────────────────────────────────────╯{reset}\n",
                            session.active_brain, session.active_brain
                        )?;
                    } else {
                        writeln!(
                            writer,
                            "\n{border}╭─ Model Switcher ─────────────────────────────────────────────────────────╮{reset}\n\
                             {border}│{reset}  \x1b[31m✖ Unknown model '{name}'.{reset} Available: {cyan}coding{reset}, {cyan}design{reset}, {cyan}general{reset}\n\
                             {border}╰──────────────────────────────────────────────────────────────────────────╯{reset}\n"
                        )?;
                    }
                } else {
                    let next = session.cycle_model();
                    let b1 = if next == BrainType::Coding {
                        format!("{green}● Coding{reset}   {dim}(Local PyTorch Transformer @ 127.0.0.1:8741){reset}")
                    } else {
                        format!("{dim}○ Coding   (Local PyTorch Transformer){reset}")
                    };
                    let b2 = if next == BrainType::Design {
                        format!("{green}● Design{reset}   {dim}(Local PyTorch SSM Mamba @ 127.0.0.1:8741){reset}")
                    } else {
                        format!("{dim}○ Design   (Local PyTorch SSM Mamba){reset}")
                    };
                    let b3 = if next == BrainType::General {
                        format!("{green}● General{reset}  {dim}(Local Ollama Adapter @ 127.0.0.1:11434){reset}")
                    } else {
                        format!("{dim}○ General  (Local Ollama Adapter){reset}")
                    };

                    writeln!(
                        writer,
                        "\n{border}╭─ Active Brain Switcher ──────────────────────────────────────────────────╮{reset}\n\
                         {border}│{reset}  [1] {b1}\n\
                         {border}│{reset}  [2] {b2}\n\
                         {border}│{reset}  [3] {b3}\n\
                         {border}│{reset}\n\
                         {border}│{reset}  {green}✔{reset} Active model switched to {:?} (Model switched to {:?})\n\
                         {border}╰──────────────────────────────────────────────────────────────────────────╯{reset}\n",
                        next, next
                    )?;
                }
            }
            ReplCommand::Prompt(prompt) => {
                if prompt.is_empty() {
                    continue;
                }
                match agent.run_turn(session.active_brain, &prompt).await {
                    Ok(resp) => {
                        let brain_label = match session.active_brain {
                            BrainType::Coding => "Coding Brain",
                            BrainType::Design => "Design Brain",
                            BrainType::General => "General Brain",
                        };

                        if resp.starts_with("\x1b[38;2;75;85;99m╭─") || resp.starts_with("╭─") {
                            // Already formatted card
                            writeln!(writer, "\n{resp}\n")?;
                        } else {
                            writeln!(
                                writer,
                                "\n{border}╭─ {cyan}● LDrago [{brain_label}]{reset} {border}───────────────────────────────────────╮{reset}\n\
                                 {resp}\n\
                                 {border}╰──────────────────────────────────────────────────────────────────────────╯{reset}\n"
                            )?;
                        }
                    }
                    Err(e) => {
                        let offline_card = crate::agent::loop_engine::format_engine_offline_notice(
                            session.active_brain,
                            &e.to_string(),
                        );
                        writeln!(writer, "\n{offline_card}\n")?;
                    }
                }
            }
        }
        writer.flush()?;
    }

    Ok(())
}

/// Auto-start the local PyTorch model daemon if it is not currently responding.
fn ensure_local_daemon_running() {
    let check = std::net::TcpStream::connect("127.0.0.1:8741");
    if check.is_err() {
        let training_dir = env::current_dir()
            .map(|d| d.join("training"))
            .unwrap_or_else(|_| "training".into());

        if training_dir.exists() {
            let _ = Command::new("uv")
                .current_dir(&training_dir)
                .args(["run", "uvicorn", "src.serve.server:app", "--host", "127.0.0.1", "--port", "8741"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
        }
    }
}

/// Run the interactive streaming agent REPL against standard terminal stdin and stdout.
pub async fn run_interactive_repl(router: &ModelRouter, _config: &AppConfig) -> Result<()> {
    ensure_local_daemon_running();
    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
    let dispatcher = ToolDispatcher::new(cwd);
    let stdin = io::stdin();
    let reader = stdin.lock();
    let stdout = io::stdout();
    run_repl_loop(router, &dispatcher, reader, stdout).await
}
