//! Interactive Terminal User Interface (TUI) execution command for LDrago.

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ld_models::{CustomBrainAdapter, ModelAdapter, ModelRouter, OllamaAdapter};
use ld_policy::{SafetyGate, SafetyVerdict};
use ld_schemas::{BrainType, CompletionRequest, HealthStatus};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};
use std::path::Path;
use std::time::{Duration, Instant};

use crate::config::AppConfig;
use crate::tui::{ActivePane, AppState};

/// RAII guard ensuring the terminal restores normal mode and alternate screen on exit or error.
struct TerminalCleanup;

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            crossterm::cursor::Show
        );
    }
}

/// Execute the interactive LDrago TUI session.
pub async fn execute_tui(router: &ModelRouter, config: &AppConfig) -> Result<()> {
    let mut app = AppState::new();

    // Health probes: Asynchronously check coding, design, and ollama endpoints
    let coder_probe = async {
        let adapter = CustomBrainAdapter::new(&config.coding_endpoint, BrainType::Coding);
        tokio::time::timeout(Duration::from_millis(600), adapter.health_check())
            .await
            .ok()
            .and_then(|r| r.ok())
            .map(|s| s == HealthStatus::Healthy)
            .unwrap_or(false)
    };
    let design_probe = async {
        let adapter = CustomBrainAdapter::new(&config.design_endpoint, BrainType::Design);
        tokio::time::timeout(Duration::from_millis(600), adapter.health_check())
            .await
            .ok()
            .and_then(|r| r.ok())
            .map(|s| s == HealthStatus::Healthy)
            .unwrap_or(false)
    };
    let ollama_probe = async {
        let adapter = OllamaAdapter::new(&config.ollama_endpoint, &config.ollama_model);
        tokio::time::timeout(Duration::from_millis(600), adapter.health_check())
            .await
            .ok()
            .and_then(|r| r.ok())
            .map(|s| s == HealthStatus::Healthy)
            .unwrap_or(false)
    };

    let (coder_ok, design_ok, ollama_ok) = tokio::join!(coder_probe, design_probe, ollama_probe);
    app.set_health(coder_ok, design_ok, ollama_ok);

    // Checkpoints count: Count folders in .ldrago/checkpoints/
    let checkpoints_dir = Path::new(".ldrago").join("checkpoints");
    let checkpoint_count = if checkpoints_dir.exists() {
        std::fs::read_dir(&checkpoints_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .count()
            })
            .unwrap_or(0)
    } else {
        0
    };
    app.active_checkpoints = checkpoint_count;

    // Terminal setup with immediate RAII guard
    enable_raw_mode()?;
    let _cleanup = TerminalCleanup;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Main event loop
    while app.running {
        terminal.draw(|f| crate::tui::ui::render(f, &app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Global quit: Esc or Ctrl+C
                if (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c'))
                    || key.code == KeyCode::Esc
                {
                    app.quit();
                    break;
                }

                match key.code {
                    // Global navigation: Tab (forward), Shift+Tab or BackTab (reverse)
                    KeyCode::Tab => {
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            app.cycle_focus_reverse();
                        } else {
                            app.cycle_focus();
                        }
                    }
                    KeyCode::BackTab => {
                        app.cycle_focus_reverse();
                    }

                    _ => match app.active_pane {
                        ActivePane::Engines => match key.code {
                            KeyCode::Char('1') => app.set_brain(BrainType::Coding),
                            KeyCode::Char('2') => app.set_brain(BrainType::Design),
                            KeyCode::Char('3') => app.set_brain(BrainType::General),
                            KeyCode::Char(' ') | KeyCode::Up | KeyCode::Down => app.cycle_brain(),
                            KeyCode::Char('q') | KeyCode::Char('Q') => {
                                app.quit();
                                break;
                            }
                            _ => {}
                        },

                        ActivePane::Chat => match key.code {
                            KeyCode::Enter => {
                                if let Some(prompt) = app.submit_input() {
                                    app.add_message("User", &prompt);

                                    // Zero-Trust Safety Gate policy check
                                    let safety = SafetyGate::default();
                                    if let SafetyVerdict::Violation { reason, risk } = safety.check_prompt(&prompt) {
                                        app.add_message("LDrago", &format!("[SECURITY BLOCK] Policy Violation: {reason} (Risk: {risk})"));
                                        continue;
                                    }

                                    app.add_message("LDrago", "Thinking...");
                                    terminal.draw(|f| crate::tui::ui::render(f, &app))?;

                                    let start = Instant::now();
                                    let target_brain = app.selected_brain;
                                    let req = CompletionRequest {
                                        prompt: prompt.clone(),
                                        brain: target_brain,
                                        max_tokens: Some(128),
                                        temperature: Some(0.7),
                                        stream: false,
                                    };

                                    let result = match router.route(target_brain).await {
                                        Ok(adapter) => adapter.complete(&req).await,
                                        Err(e) => Err(e),
                                    };

                                    let elapsed = start.elapsed();
                                    let response_text = match result {
                                        Ok(resp) => {
                                            let secs = elapsed.as_secs_f64();
                                            if secs > 0.0 {
                                                let tokens = resp.tokens_generated.unwrap_or_else(|| {
                                                    resp.text.split_whitespace().count().max(1) as u32
                                                });
                                                app.tokens_per_sec = (tokens as f64) / secs;
                                            }
                                            resp.text
                                        }
                                        Err(err) => {
                                            format!("[Error from {target_brain} engine: {err}]")
                                        }
                                    };

                                    app.replace_last_message("LDrago", &response_text);
                                }
                            }
                            KeyCode::Char(c)
                                if !key.modifiers.contains(KeyModifiers::CONTROL)
                                    && !key.modifiers.contains(KeyModifiers::ALT) =>
                            {
                                app.handle_char(c);
                            }
                            KeyCode::Backspace => app.delete_char(),
                            KeyCode::Up => app.scroll_up(),
                            KeyCode::Down => app.scroll_down(),
                            _ => {}
                        },

                        ActivePane::Telemetry => match key.code {
                            KeyCode::Up => app.scroll_up(),
                            KeyCode::Down => app.scroll_down(),
                            KeyCode::Char('q') | KeyCode::Char('Q') => {
                                app.quit();
                                break;
                            }
                            _ => {}
                        },
                    },
                }
            }
        }
    }

    Ok(())
}
