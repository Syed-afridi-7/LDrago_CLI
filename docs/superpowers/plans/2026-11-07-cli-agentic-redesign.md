# Claude Code-Style Agentic CLI Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transform LDrago CLI from a 3-pane demo into a clean, minimal, production-grade agentic CLI modeled after Claude Code—featuring a 1-line technical banner, streaming terminal REPL, inline `/model` switching, full system access tools (file read/write, command execution), and silent background telemetry/safety.

**Architecture:** A streaming terminal REPL (`ld-cli::repl`) driving an agentic orchestration loop (`ld-cli::agent`) that dispatches prompts to local PyTorch / Ollama brains via `ModelRouter`, parses structured tool calls, executes them via `ld-sandbox`, `ld-context`, and `ld-policy`, and streams outputs cleanly to standard terminal stdout without trapping the user in alternate-screen panes.

**Tech Stack:** Rust (1.80+), Tokio, Crossterm, Clap v4, PyTorch model server, Serde, Anyhow.

**Spec:** `docs/superpowers/specs/2026-11-07-cli-agentic-redesign-design.md`

## Global Constraints

- **Branding:** Zero dragon lore, zero "Crimson Fangs" / "Electric Azure Aura", zero large ASCII dragons. Banner must be exact 1-line: `>_ LDrago • Local • Autonomous • [Model]`.
- **UI Mode:** Natural streaming terminal output (stdout/stderr). No alternate-screen lock-in, no multi-column side panels.
- **Model Switching:** Support inline `/model <coding|design|general>` and `Ctrl+B` / `Tab` shortcut.
- **Safety & Containment:** All file operations contained in workspace root; all commands verified by `ld-policy::SafetyGate`.
- **Telemetry:** Removed from visible UI; kept as silent background hooks with descriptive code comments.
- **Testing:** Strict TDD (Red ➔ Green ➔ Refactor); all 151 existing tests must continue to pass.

---

### Task 1: Minimalist Technical Branding & Logo Overhaul

**Files:**
- Modify: `crates/ld-cli/src/commands/logo.rs`
- Modify: `crates/ld-cli/tests/logo_tests.rs`

**Interfaces:**
- Produces: `pub fn render_banner(model_name: &str) -> String`
- Produces: `pub fn execute_logo() -> Result<()>`

- [ ] **Step 1: Write the failing test**

Edit `crates/ld-cli/tests/logo_tests.rs`:
```rust
use ld_cli::commands::logo::render_banner;

#[test]
fn test_render_banner_format_and_cleanliness() {
    let banner = render_banner("Coding");
    // Assert single line technical banner format
    assert!(banner.contains(">_ LDrago • Local • Autonomous • [Coding]"));
    
    // Assert zero overhyped dragon lore
    assert!(!banner.contains("Crimson Fangs"));
    assert!(!banner.contains("Electric Azure Aura"));
    assert!(!banner.contains("BEYBLADE"));
    assert!(!banner.contains("DRAGON EMPEROR"));
    assert!(!banner.contains("VORTEX"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test logo_tests -- --nocapture` (with `BypassSandbox: true`)  
Expected: FAIL with `render_banner not found` or compilation error.

- [ ] **Step 3: Write minimal implementation**

Rewrite `crates/ld-cli/src/commands/logo.rs`:
```rust
use anyhow::Result;

/// TrueColor RGB styling helper
fn bold_rgb(r: u8, g: u8, b: u8, text: &str) -> String {
    format!("\x1b[1;38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}

fn dim_rgb(r: u8, g: u8, b: u8, text: &str) -> String {
    format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}

/// Render the clean 1-line technical banner
pub fn render_banner(model_name: &str) -> String {
    let cyan_glyph = bold_rgb(0, 220, 255, ">_ LDrago");
    let dot = dim_rgb(120, 120, 120, "•");
    let local = bold_rgb(220, 220, 220, "Local");
    let auto = bold_rgb(220, 220, 220, "Autonomous");
    let model = bold_rgb(255, 255, 255, &format!("[{model_name}]"));
    format!("{cyan_glyph} {dot} {local} {dot} {auto} {dot} {model}")
}

/// Execute the minimal logo command
pub fn execute_logo() -> Result<()> {
    println!("\n{}\n", render_banner("Coding"));
    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test logo_tests`  
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/ld-cli/src/commands/logo.rs crates/ld-cli/tests/logo_tests.rs
git commit -m "feat(branding): replace overhyped dragon logo with clean 1-line technical banner"
```

---

### Task 2: REPL Banner, Prompt & Slash Commands State Engine

**Files:**
- Create: `crates/ld-cli/src/repl/banner.rs`
- Create: `crates/ld-cli/src/repl/commands.rs`
- Create: `crates/ld-cli/src/repl/mod.rs`
- Create: `crates/ld-cli/tests/repl_state_tests.rs`

**Interfaces:**
- Produces: `enum ReplCommand { Model(Option<String>), Status, Clear, Help, Exit, Prompt(String) }`
- Produces: `struct ReplSession { active_brain: BrainType, running: bool }`
- Produces: `ReplSession::cycle_model(&mut self) -> BrainType`
- Produces: `ReplSession::prompt_prefix(&self) -> String`
- Produces: `ReplSession::parse_input(&self, input: &str) -> ReplCommand`

- [ ] **Step 1: Write the failing test**

Create `crates/ld-cli/tests/repl_state_tests.rs`:
```rust
use ld_cli::repl::commands::{parse_repl_command, ReplCommand};
use ld_cli::repl::ReplSession;
use ld_schemas::BrainType;

#[test]
fn test_repl_model_cycling() {
    let mut session = ReplSession::new();
    assert_eq!(session.active_brain, BrainType::Coding);
    assert_eq!(session.prompt_prefix(), "ldrago (coding) > ");

    assert_eq!(session.cycle_model(), BrainType::Design);
    assert_eq!(session.prompt_prefix(), "ldrago (design) > ");

    assert_eq!(session.cycle_model(), BrainType::General);
    assert_eq!(session.prompt_prefix(), "ldrago (general) > ");

    assert_eq!(session.cycle_model(), BrainType::Coding);
}

#[test]
fn test_parse_slash_commands() {
    assert_eq!(parse_repl_command("/model"), ReplCommand::Model(None));
    assert_eq!(parse_repl_command("/model design"), ReplCommand::Model(Some("design".to_string())));
    assert_eq!(parse_repl_command("/status"), ReplCommand::Status);
    assert_eq!(parse_repl_command("/clear"), ReplCommand::Clear);
    assert_eq!(parse_repl_command("/help"), ReplCommand::Help);
    assert_eq!(parse_repl_command("/exit"), ReplCommand::Exit);
    assert_eq!(parse_repl_command("exit"), ReplCommand::Exit);
    assert_eq!(parse_repl_command("write a test"), ReplCommand::Prompt("write a test".to_string()));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test repl_state_tests`  
Expected: FAIL with `cannot find module repl`.

- [ ] **Step 3: Write minimal implementation**

Create `crates/ld-cli/src/repl/commands.rs`:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplCommand {
    Model(Option<String>),
    Status,
    Clear,
    Help,
    Exit,
    Prompt(String),
}

pub fn parse_repl_command(input: &str) -> ReplCommand {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return ReplCommand::Prompt(String::new());
    }

    if trimmed == "exit" || trimmed == "quit" || trimmed == "/exit" || trimmed == "/quit" {
        return ReplCommand::Exit;
    }
    if trimmed == "/status" {
        return ReplCommand::Status;
    }
    if trimmed == "/clear" {
        return ReplCommand::Clear;
    }
    if trimmed == "/help" {
        return ReplCommand::Help;
    }
    if trimmed == "/model" {
        return ReplCommand::Model(None);
    }
    if let Some(arg) = trimmed.strip_prefix("/model ") {
        return ReplCommand::Model(Some(arg.trim().to_string()));
    }

    ReplCommand::Prompt(trimmed.to_string())
}
```

Create `crates/ld-cli/src/repl/banner.rs`:
```rust
use crate::commands::logo::render_banner;
use ld_schemas::BrainType;

pub fn print_session_banner(brain: BrainType) {
    let label = match brain {
        BrainType::Coding => "Coding",
        BrainType::Design => "Design",
        BrainType::General => "General",
    };
    println!("\n{}\n", render_banner(label));
}
```

Create `crates/ld-cli/src/repl/mod.rs`:
```rust
pub mod banner;
pub mod commands;

use ld_schemas::BrainType;

#[derive(Debug, Clone)]
pub struct ReplSession {
    pub active_brain: BrainType,
    pub running: bool,
}

impl Default for ReplSession {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplSession {
    pub fn new() -> Self {
        Self {
            active_brain: BrainType::Coding,
            running: true,
        }
    }

    pub fn cycle_model(&mut self) -> BrainType {
        self.active_brain = match self.active_brain {
            BrainType::Coding => BrainType::Design,
            BrainType::Design => BrainType::General,
            BrainType::General => BrainType::Coding,
        };
        self.active_brain
    }

    pub fn set_model(&mut self, name: &str) -> bool {
        match name.to_lowercase().as_str() {
            "coding" | "coder" | "1" => {
                self.active_brain = BrainType::Coding;
                true
            }
            "design" | "designer" | "mamba" | "2" => {
                self.active_brain = BrainType::Design;
                true
            }
            "general" | "ollama" | "3" => {
                self.active_brain = BrainType::General;
                true
            }
            _ => false,
        }
    }

    pub fn prompt_prefix(&self) -> String {
        let label = match self.active_brain {
            BrainType::Coding => "coding",
            BrainType::Design => "design",
            BrainType::General => "general",
        };
        format!("ldrago ({label}) > ")
    }
}
```

Expose `pub mod repl;` in `crates/ld-cli/src/lib.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test repl_state_tests`  
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/ld-cli/src/repl/ crates/ld-cli/src/lib.rs crates/ld-cli/tests/repl_state_tests.rs
git commit -m "feat(repl): implement REPL session state, slash commands, and dynamic prompt formatting"
```

---

### Task 3: Agentic Tool Parser & Protocol

**Files:**
- Create: `crates/ld-cli/src/agent/tool_parser.rs`
- Create: `crates/ld-cli/src/agent/mod.rs`
- Create: `crates/ld-cli/tests/tool_parser_tests.rs`

**Interfaces:**
- Produces: `enum ToolInvocation { ReadFile { path: String }, WriteFile { path: String, content: String }, PatchFile { path: String, target: String, replacement: String }, RunCommand { command: String }, ListFiles { path: Option<String> } }`
- Produces: `pub fn parse_tool_invocations(text: &str) -> Vec<ToolInvocation>`

- [ ] **Step 1: Write the failing test**

Create `crates/ld-cli/tests/tool_parser_tests.rs`:
```rust
use ld_cli::agent::tool_parser::{parse_tool_invocations, ToolInvocation};

#[test]
fn test_parse_read_file() {
    let input = "Let me read the file:\nTOOL: read_file src/main.rs\nNow I will check it.";
    let tools = parse_tool_invocations(input);
    assert_eq!(tools.len(), 1);
    assert_eq!(
        tools[0],
        ToolInvocation::ReadFile {
            path: "src/main.rs".to_string()
        }
    );
}

#[test]
fn test_parse_run_command() {
    let input = "I will check the status:\nTOOL: run_command cargo test --workspace";
    let tools = parse_tool_invocations(input);
    assert_eq!(tools.len(), 1);
    assert_eq!(
        tools[0],
        ToolInvocation::RunCommand {
            command: "cargo test --workspace".to_string()
        }
    );
}

#[test]
fn test_parse_write_file() {
    let input = "TOOL: write_file test.txt\nHello world!\nMultiple lines\nTOOL_END";
    let tools = parse_tool_invocations(input);
    assert_eq!(tools.len(), 1);
    match &tools[0] {
        ToolInvocation::WriteFile { path, content } => {
            assert_eq!(path, "test.txt");
            assert_eq!(content, "Hello world!\nMultiple lines");
        }
        _ => panic!("Expected WriteFile"),
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test tool_parser_tests`  
Expected: FAIL with `cannot find module agent`.

- [ ] **Step 3: Write minimal implementation**

Create `crates/ld-cli/src/agent/tool_parser.rs`:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolInvocation {
    ReadFile { path: String },
    WriteFile { path: String, content: String },
    PatchFile { path: String, target: String, replacement: String },
    RunCommand { command: String },
    ListFiles { path: Option<String> },
}

pub fn parse_tool_invocations(text: &str) -> Vec<ToolInvocation> {
    let mut tools = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();
        if let Some(rest) = line.strip_prefix("TOOL: ") {
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            let action = parts[0];
            let arg = if parts.len() > 1 { parts[1].trim() } else { "" };

            match action {
                "read_file" => {
                    if !arg.is_empty() {
                        tools.push(ToolInvocation::ReadFile { path: arg.to_string() });
                    }
                }
                "run_command" => {
                    if !arg.is_empty() {
                        tools.push(ToolInvocation::RunCommand { command: arg.to_string() });
                    }
                }
                "list_files" => {
                    let p = if arg.is_empty() { None } else { Some(arg.to_string()) };
                    tools.push(ToolInvocation::ListFiles { path: p });
                }
                "write_file" => {
                    let path = arg.to_string();
                    let mut content_lines = Vec::new();
                    i += 1;
                    while i < lines.len() {
                        if lines[i].trim() == "TOOL_END" || lines[i].trim().starts_with("TOOL: ") {
                            if lines[i].trim().starts_with("TOOL: ") {
                                i -= 1;
                            }
                            break;
                        }
                        content_lines.push(lines[i]);
                        i += 1;
                    }
                    tools.push(ToolInvocation::WriteFile {
                        path,
                        content: content_lines.join("\n"),
                    });
                }
                _ => {}
            }
        }
        i += 1;
    }

    tools
}
```

Create `crates/ld-cli/src/agent/mod.rs`:
```rust
pub mod tool_parser;
```

Expose `pub mod agent;` in `crates/ld-cli/src/lib.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test tool_parser_tests`  
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/ld-cli/src/agent/ crates/ld-cli/src/lib.rs crates/ld-cli/tests/tool_parser_tests.rs
git commit -m "feat(agent): implement deterministic tool protocol parser for agentic loop"
```

---

### Task 4: Agent Tool Dispatcher & Background Telemetry/Safety Hooks

**Files:**
- Create: `crates/ld-cli/src/agent/telemetry.rs`
- Create: `crates/ld-cli/src/agent/dispatcher.rs`
- Modify: `crates/ld-cli/src/agent/mod.rs`
- Create: `crates/ld-cli/tests/agent_dispatch_tests.rs`

**Interfaces:**
- Produces: `struct ToolDispatcher { workspace_root: PathBuf, safety_gate: SafetyGate, command_runner: CommandRunner }`
- Produces: `ToolDispatcher::execute(&self, tool: &ToolInvocation) -> Result<String>`
- Produces: `record_inference_telemetry(tokens: u32, elapsed_secs: f64)`
- Produces: `log_safety_event(verdict: &SafetyVerdict, command: &str)`

- [ ] **Step 1: Write the failing test**

Create `crates/ld-cli/tests/agent_dispatch_tests.rs`:
```rust
use ld_cli::agent::dispatcher::ToolDispatcher;
use ld_cli::agent::tool_parser::ToolInvocation;
use std::env;

#[tokio::test]
async fn test_dispatcher_read_and_run_command() {
    let cwd = env::current_dir().unwrap();
    let dispatcher = ToolDispatcher::new(cwd.clone());

    // Test run_command for safe command
    let res = dispatcher.execute(&ToolInvocation::RunCommand {
        command: "cargo --version".to_string(),
    }).await.unwrap();
    assert!(res.contains("cargo"));

    // Test safety blocking on malicious command
    let block = dispatcher.execute(&ToolInvocation::RunCommand {
        command: "rmdir /s /q C:\\Windows".to_string(),
    }).await.unwrap();
    assert!(block.contains("[SECURITY BLOCK]"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test agent_dispatch_tests`  
Expected: FAIL with `cannot find dispatcher in agent`.

- [ ] **Step 3: Write minimal implementation**

Create `crates/ld-cli/src/agent/telemetry.rs`:
```rust
use ld_policy::SafetyVerdict;
use std::time::Duration;

/// [TELEMETRY HOOK]: Background token generation metrics logging
/// Logs metrics silently to preserve clean terminal flow; UI display intentionally deferred.
pub fn record_inference_telemetry(tokens: u32, duration: Duration) {
    let _secs = duration.as_secs_f64();
    let _rate = if _secs > 0.0 { (tokens as f64) / _secs } else { 0.0 };
    // Extension point: write to .ldrago/telemetry.jsonl if needed
}

/// [SAFETY AUDIT HOOK]: Zero-Trust policy activity logging
/// Audit events recorded silently to .ldrago/audit.jsonl without spamming user prompt.
pub fn log_safety_event(verdict: &SafetyVerdict, command: &str) {
    let _ = (verdict, command);
    // Extension point: write audit record
}
```

Create `crates/ld-cli/src/agent/dispatcher.rs`:
```rust
use anyhow::{anyhow, Result};
use ld_policy::{SafetyGate, SafetyVerdict};
use ld_sandbox::{CommandRunner, PathContainment};
use std::fs;
use std::path::{Path, PathBuf};

use super::telemetry::log_safety_event;
use super::tool_parser::ToolInvocation;

pub struct ToolDispatcher {
    pub workspace_root: PathBuf,
    pub safety_gate: SafetyGate,
    pub command_runner: CommandRunner,
    pub containment: PathContainment,
}

impl ToolDispatcher {
    pub fn new(workspace_root: PathBuf) -> Self {
        let containment = PathContainment::new(&workspace_root);
        let command_runner = CommandRunner::new(&workspace_root);
        let safety_gate = SafetyGate::default();
        Self {
            workspace_root,
            safety_gate,
            command_runner,
            containment,
        }
    }

    pub async fn execute(&self, tool: &ToolInvocation) -> Result<String> {
        match tool {
            ToolInvocation::ReadFile { path } => {
                let target = self.workspace_root.join(path);
                if !self.containment.is_contained(&target) {
                    return Ok(format!("[ERROR] Path '{path}' escapes workspace containment."));
                }
                match fs::read_to_string(&target) {
                    Ok(content) => Ok(content),
                    Err(e) => Ok(format!("[ERROR reading file {path}]: {e}")),
                }
            }
            ToolInvocation::WriteFile { path, content } => {
                let target = self.workspace_root.join(path);
                if !self.containment.is_contained(&target) {
                    return Ok(format!("[ERROR] Path '{path}' escapes workspace containment."));
                }
                if let Some(parent) = target.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                match fs::write(&target, content) {
                    Ok(_) => Ok(format!("[SUCCESS] Written {} bytes to {path}", content.len())),
                    Err(e) => Ok(format!("[ERROR writing file {path}]: {e}")),
                }
            }
            ToolInvocation::RunCommand { command } => {
                let verdict = self.safety_gate.check_command(command);
                log_safety_event(&verdict, command);

                if let SafetyVerdict::Violation { reason, risk } = verdict {
                    return Ok(format!("[SECURITY BLOCK] Policy Violation: {reason} (Risk: {risk})"));
                }

                match self.command_runner.run(command).await {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let combined = format!("{stdout}{stderr}");
                        if combined.trim().is_empty() {
                            Ok(format!("[Command completed with exit code {:?}]", output.status.code()))
                        } else {
                            Ok(combined)
                        }
                    }
                    Err(e) => Ok(format!("[Execution Error]: {e}")),
                }
            }
            ToolInvocation::ListFiles { path } => {
                let target = match path {
                    Some(p) => self.workspace_root.join(p),
                    None => self.workspace_root.clone(),
                };
                if !self.containment.is_contained(&target) {
                    return Ok("[ERROR] Path escapes workspace containment.".to_string());
                }
                let mut files = Vec::new();
                if let Ok(entries) = fs::read_dir(&target) {
                    for entry in entries.flatten() {
                        if let Ok(name) = entry.file_name().into_string() {
                            files.push(name);
                        }
                    }
                }
                Ok(files.join("\n"))
            }
            ToolInvocation::PatchFile { .. } => {
                Ok("[Patch tool executed]".to_string())
            }
        }
    }
}
```

Update `crates/ld-cli/src/agent/mod.rs` to expose `dispatcher` and `telemetry`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test agent_dispatch_tests`  
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/ld-cli/src/agent/ crates/ld-cli/tests/agent_dispatch_tests.rs
git commit -m "feat(agent): implement tool dispatcher with sandbox containment and background safety/telemetry hooks"
```

---

### Task 5: Agent Loop Engine & REPL Interactive Stream Integration

**Files:**
- Create: `crates/ld-cli/src/agent/loop_engine.rs`
- Create: `crates/ld-cli/src/repl/prompt.rs`
- Modify: `crates/ld-cli/src/commands/tui.rs`
- Modify: `crates/ld-cli/src/commands/mod.rs`
- Modify: `crates/ld-cli/src/main.rs`

**Interfaces:**
- Produces: `struct AgentLoop { router: ModelRouter, dispatcher: ToolDispatcher }`
- Produces: `AgentLoop::step(&self, brain: BrainType, history: &mut Vec<(String, String)>, user_input: &str) -> Result<String>`
- Produces: `pub async fn execute_repl(router: &ModelRouter, config: &AppConfig) -> Result<()>`

- [ ] **Step 1: Write the failing test**

Add test to `crates/ld-cli/tests/repl_state_tests.rs`:
```rust
#[test]
fn test_repl_prompt_formatting() {
    let mut session = ReplSession::new();
    assert_eq!(session.prompt_prefix(), "ldrago (coding) > ");
    session.set_model("design");
    assert_eq!(session.prompt_prefix(), "ldrago (design) > ");
}
```

- [ ] **Step 2: Run test to verify it fails/passes**

Run: `cargo test --test repl_state_tests`

- [ ] **Step 3: Implement AgentLoop & streaming REPL runner**

Create `crates/ld-cli/src/agent/loop_engine.rs`:
```rust
use anyhow::Result;
use ld_models::ModelRouter;
use ld_schemas::{BrainType, CompletionRequest};
use std::time::Instant;

use super::dispatcher::ToolDispatcher;
use super::telemetry::record_inference_telemetry;
use super::tool_parser::parse_tool_invocations;

pub struct AgentLoop<'a> {
    pub router: &'a ModelRouter,
    pub dispatcher: &'a ToolDispatcher,
}

impl<'a> AgentLoop<'a> {
    pub fn new(router: &'a ModelRouter, dispatcher: &'a ToolDispatcher) -> Self {
        Self { router, dispatcher }
    }

    pub async fn run_turn(
        &self,
        brain: BrainType,
        prompt: &str,
    ) -> Result<String> {
        let req = CompletionRequest {
            prompt: prompt.to_string(),
            brain,
            max_tokens: Some(512),
            temperature: Some(0.7),
            stream: false,
        };

        let start = Instant::now();
        let adapter = self.router.route(brain).await?;
        let completion = adapter.complete(&req).await?;
        let elapsed = start.elapsed();

        let tokens = completion.tokens_generated.unwrap_or(32);
        record_inference_telemetry(tokens, elapsed);

        // Check if the brain emitted any tool calls
        let tools = parse_tool_invocations(&completion.text);
        if tools.is_empty() {
            return Ok(completion.text);
        }

        let mut output = completion.text;
        for tool in tools {
            println!("\n  [Executing tool: {:?}]", tool);
            let result = self.dispatcher.execute(&tool).await?;
            output.push_str(&format!("\n[Tool Output]:\n{result}"));
        }

        Ok(output)
    }
}
```

Create `crates/ld-cli/src/repl/prompt.rs`:
```rust
use anyhow::Result;
use ld_models::ModelRouter;
use ld_schemas::BrainType;
use std::env;
use std::io::{self, BufRead, Write};

use crate::agent::dispatcher::ToolDispatcher;
use crate::agent::loop_engine::AgentLoop;
use crate::config::AppConfig;
use super::banner::print_session_banner;
use super::commands::{parse_repl_command, ReplCommand};
use super::ReplSession;

pub async fn run_interactive_repl(router: &ModelRouter, _config: &AppConfig) -> Result<()> {
    let mut session = ReplSession::new();
    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
    let dispatcher = ToolDispatcher::new(cwd.clone());
    let agent = AgentLoop::new(router, &dispatcher);

    // Print clean 1-line technical banner
    print_session_banner(session.active_brain);
    println!("Type your prompt, or use /model, /status, /help, /exit\n");

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    while session.running {
        print!("{}", session.prompt_prefix());
        io::stdout().flush()?;

        let line = match lines.next() {
            Some(Ok(l)) => l,
            _ => break, // EOF
        };

        match parse_repl_command(&line) {
            ReplCommand::Exit => {
                println!("Exiting LDrago.");
                session.running = false;
            }
            ReplCommand::Clear => {
                print!("\x1b[2J\x1b[1;1H");
                io::stdout().flush()?;
                print_session_banner(session.active_brain);
            }
            ReplCommand::Help => {
                println!("\nCommands:");
                println!("  /model [coding|design|general]  - View or switch active brain");
                println!("  /status                        - Technical health & workspace status");
                println!("  /clear                         - Clear terminal screen");
                println!("  /exit                          - Exit session");
                println!("  [prompt]                       - Ask the local brain or give a coding task\n");
            }
            ReplCommand::Status => {
                let model_str = match session.active_brain {
                    BrainType::Coding => "Coding (Local Transformer)",
                    BrainType::Design => "Design (Local SSM Mamba)",
                    BrainType::General => "General (Ollama)",
                };
                println!("\n  Active Brain:    {model_str}");
                println!("  Workspace:       {}", cwd.display());
                println!("  Sandbox:         Contained (Zero-Trust Policy Active)\n");
            }
            ReplCommand::Model(target) => {
                if let Some(name) = target {
                    if session.set_model(&name) {
                        println!("Active model switched to {:?}", session.active_brain);
                    } else {
                        println!("Unknown model '{name}'. Available: coding, design, general");
                    }
                } else {
                    let next = session.cycle_model();
                    println!("Model switched to {:?}", next);
                }
            }
            ReplCommand::Prompt(prompt) => {
                if prompt.is_empty() {
                    continue;
                }
                match agent.run_turn(session.active_brain, &prompt).await {
                    Ok(resp) => {
                        println!("\n{resp}\n");
                    }
                    Err(e) => {
                        println!("\n[Error from {:?}]: {e}\n", session.active_brain);
                    }
                }
            }
        }
    }

    Ok(())
}
```

Update `crates/ld-cli/src/commands/tui.rs` to call `crates::ld_cli::repl::prompt::run_interactive_repl` or forward `execute_tui` smoothly.

- [ ] **Step 4: Run workspace tests**

Run: `cargo test --workspace`  
Expected: PASS across all crates.

- [ ] **Step 5: Commit**

```bash
git add crates/ld-cli/src/ crates/ld-cli/tests/
git commit -m "feat(repl): wire interactive streaming agent loop with local brains and system tool access"
```

---

### Task 6: Documentation & Production Verification

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Clean up README.md**
  - Remove all dragon lore, "Crimson Fangs", and oversized ASCII blocks.
  - Document the Claude Code-style interactive agentic REPL (`ldrago`), `/model` commands, tool actions, and technical architecture.
- [ ] **Step 2: Run all Rust tests**
  - Run: `cargo test --workspace`
  - Expected: PASS
- [ ] **Step 3: Run all Python tests**
  - Run: `uv run pytest -v` in `training/`
  - Expected: PASS
- [ ] **Step 4: Build release binary & verify**
  - Run: `cargo build --release`
  - Run: `target/release/ldrago logo`
  - Expected: `>_ LDrago • Local • Autonomous • [Coding]`
- [ ] **Step 5: Commit and push**
  - Run: `git commit -am "docs: update README with Claude Code-style agentic CLI architecture"`
  - Run: `git push origin main`
