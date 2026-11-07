# LDrago CLI — Claude Code-Style Agentic Redesign Spec

**Date:** 2026-11-07  
**Status:** Approved  
**Classification:** Architectural Redesign  

---

## 1. Executive Summary

This specification defines the complete architectural overhaul of **LDrago CLI** from an overhyped, multi-pane TUI demo into a clean, minimal, production-grade agentic CLI modeled after **Claude Code**.

### Key Changes:
1. **Minimalist Technical Branding:** Elimination of dragon lore ("Crimson Fangs", "Electric Azure Aura", large multi-line ASCII dragon art) in favor of a crisp 1-line, 3-point technical banner: `>_ LDrago • Local • Autonomous • [Model]`.
2. **Claude Code-Style Streaming REPL:** Replaces the 3-pane Ratatui layout (Engines, Chat, Telemetry) with a standard-output streaming interactive prompt (`ldrago (coding) > `) that preserves normal terminal history and scrollback.
3. **Inline Model Switching:** Fast switching between local brains (Coding, Design, General) via slash commands (`/model <name>`) and prompt shortcuts (`Ctrl+B` or `Tab`).
4. **Full System Access & Agentic Tool Engine:** Directly connects local brains to workspace tools (read files, write/patch code, run shell commands, list files) guarded by path containment and zero-trust safety checks.
5. **Background Telemetry & Safety:** Side panels removed; telemetry, token generation rate, and safety audit logging operate silently in the background with clean code comment hooks.

---

## 2. Architecture & Component Decomposition

```
┌─────────────────────────────────────────────────────────────┐
│                    Interactive CLI REPL                     │
│  • Clean standard terminal stream (natural scrollback)       │
│  • 1-line minimal banner: >_ LDrago • Local • Autonomous    │
│  • Inline commands (/model, /help, /clear, /exit, Ctrl+B)   │
└──────────────────────────────┬──────────────────────────────┘
                               │ User Prompt
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                         Agent Loop                          │
│  • Manages conversational history and turn context          │
│  • Dispatches prompt to active brain via ModelRouter        │
│  • Parses model completions for structured tool calls        │
│  • Feeds tool results back into context for iterative loop  │
└──────────────┬──────────────────────────────┬───────────────┘
               │ Tool Requests                │ Text / Inference
               ▼                              ▼
┌──────────────────────────────┐ ┌────────────────────────────┐
│      Tool Dispatcher         │ │       Model Router         │
│  • read_file (ld-context)    │ │  • [1] Coding (Transformer)│
│  • write_file (ld-sandbox)   │ │  • [2] Design (SSM Mamba)  │
│  • run_command (ld-sandbox)  │ │  • [3] General (Ollama)    │
│  • list_dir / search         │ │  • Active brain selection  │
│  • Zero-Trust SafetyGate     │ └────────────────────────────┘
└──────────────────────────────┘
```

### Module Responsibilities:

- **`crates/ld-cli/src/repl/`**:
  - `prompt.rs`: Terminal prompt handling with crossterm, supporting dynamic model prefix (`ldrago (coding) > `).
  - `commands.rs`: Slash command parser (`/model`, `/status`, `/help`, `/clear`, `/exit`).
  - `banner.rs`: Minimal TrueColor 1-line banner printer.

- **`crates/ld-cli/src/agent/`**:
  - `loop_engine.rs`: Multi-turn conversational loop, coordinating model inference with tool execution.
  - `tool_parser.rs`: Deterministic parser extracting `TOOL: <name> <args>` from model completions.
  - `dispatcher.rs`: Tool execution engine calling `ld-context` and `ld-sandbox`.
  - `telemetry.rs`: Silent background telemetry hooks and safety audit logging with explicit code comments.

- **`crates/ld-cli/src/commands/logo.rs`**:
  - Replaced with clean `render_banner()` function emitting `>_ LDrago • Local • Autonomous • [Model]`.

- **`crates/ld-cli/src/commands/tui.rs`**:
  - Re-routed to the new streaming REPL engine while preserving backward-compatible CLI dispatch.

---

## 3. Interaction & Branding Specification

### 3.1 Startup Banner
Displayed once on CLI launch:
```text
>_ LDrago • Local • Autonomous • [Coding]
```
- Line length: ~45 characters (compact, responsive).
- Colors: Subtle Cyan (`\x1b[38;2;0;220;255m`) for glyph, Dim Slate (`\x1b[38;2;120;120;120m`) for dividers, Bold White (`\x1b[1;37m`) for model name.
- No dragon artwork, no crimson/azure lore, no marketing hype.

### 3.2 Interactive Prompt & Inline Model Switching
Prompt format:
```text
ldrago (coding) > 
```
- **Hotkeys:**
  - `Ctrl+B` or `Tab` (when input buffer is empty): Cycles model immediately: `Coding` ➔ `Design` ➔ `General` ➔ `Coding`.
- **Slash Commands:**
  - `/model`: Displays current model and options.
  - `/model <coding|design|general>`: Instantly switches the active brain.
  - `/status`: Prints a 1-line technical status summary:
    `Status: Healthy | Model: Coding (127.0.0.1:8741) | Sandbox: Contained | Policy: Active`
  - `/clear`: Clears terminal screen.
  - `/help`: Displays quick 4-line cheat sheet.
  - `/exit` or `exit`: Exits the session cleanly.

---

## 4. System Access & Tool Execution Engine

### 4.1 Tool Protocol Syntax
The local brain can emit tool invocations using clear, deterministic markers:

1. **`TOOL: read_file <relative_path>`**
   - Returns file content from workspace.
2. **`TOOL: write_file <relative_path>\n<content>`**
   - Writes content atomically to `<relative_path>` within workspace.
3. **`TOOL: patch_file <relative_path>\n<<<TARGET\n<old_code>\n===REPLACE\n<new_code>\n>>>`**
   - Applies targeted diff patch via `ld-sandbox::PatchEngine`.
4. **`TOOL: run_command <command_line>`**
   - Executes command via `ld-sandbox::CommandRunner` through `ld-policy::SafetyGate`.
5. **`TOOL: list_files <optional_relative_path>`**
   - Returns directory listing via `ld-context`.

### 4.2 Security & Containment
- All file reads and writes are restricted to the active workspace root via `PathContainment`.
- Commands pass through `SafetyGate::check_command(&cmd)`. Disallowed system destruction patterns (e.g., deleting root directories, raw format commands) are blocked immediately with `[SECURITY BLOCK]`.
- Checkpoints can be automatically captured in `.ldrago/checkpoints/` before destructive file operations.

### 4.3 Background Telemetry & Safety Hooks
Side panels are eliminated. Telemetry and safety activities are maintained in code as clean hooks with descriptive comments:

```rust
// [TELEMETRY HOOK]: Background token generation speed & latency
// Silently logged to .ldrago/telemetry.jsonl; UI presentation intentionally deferred.
record_inference_telemetry(tokens_generated, elapsed);

// [SAFETY AUDIT HOOK]: Zero-Trust policy activity logging
// Audit trail maintained in .ldrago/audit.jsonl without disrupting terminal flow.
log_safety_event(&verdict, &command);
```

---

## 5. Testing & Verification Strategy (TDD)

Strict Test-Driven Development (Red ➔ Green ➔ Refactor) is applied across all new and updated components:

1. **Brand Cleanliness Tests (`crates/ld-cli/tests/logo_tests.rs`)**:
   - Assert banner contains `>_ LDrago • Local • Autonomous`.
   - Assert zero occurrences of `Crimson Fangs`, `Electric Azure Aura`, `Beyblade`, or ASCII dragons.

2. **REPL State & Command Tests (`crates/ld-cli/tests/repl_state_tests.rs`)**:
   - Verify prompt prefix formatting.
   - Verify model cycling: `Coding` ➔ `Design` ➔ `General`.
   - Verify slash commands `/model`, `/status`, `/help`, `/exit`.

3. **Tool Parser Tests (`crates/ld-cli/tests/tool_parser_tests.rs`)**:
   - Verify parsing of `read_file`, `write_file`, `patch_file`, `run_command`, `list_files`.
   - Verify handling of multi-turn code and text blocks.

4. **Agent Dispatch & Security Tests (`crates/ld-cli/tests/agent_dispatch_tests.rs`)**:
   - Verify workspace containment on file operations.
   - Verify `SafetyGate` blocking on malicious commands.
   - Verify tool output round-trip into agent context.

5. **Full Regression Suite**:
   - Verify all 151 existing tests across `crates/*` and `training/*` pass.

---

## 6. Implementation Scope & File Plan

- **Files to Modify:**
  - `crates/ld-cli/src/commands/logo.rs`: Replace dragon banner with clean technical banner.
  - `crates/ld-cli/src/commands/tui.rs`: Route `ldrago tui` / `ldrago` to the new agentic REPL.
  - `crates/ld-cli/src/main.rs`: Set default interactive command to agentic REPL.
  - `crates/ld-cli/tests/logo_tests.rs`: Update assertions for minimal technical branding.
  - `README.md`: Update documentation to reflect the professional Claude Code-style interface.

- **Files to Create:**
  - `crates/ld-cli/src/repl/mod.rs`
  - `crates/ld-cli/src/repl/banner.rs`
  - `crates/ld-cli/src/repl/prompt.rs`
  - `crates/ld-cli/src/repl/commands.rs`
  - `crates/ld-cli/src/agent/mod.rs`
  - `crates/ld-cli/src/agent/tool_parser.rs`
  - `crates/ld-cli/src/agent/dispatcher.rs`
  - `crates/ld-cli/src/agent/loop_engine.rs`
  - `crates/ld-cli/src/agent/telemetry.rs`
  - `crates/ld-cli/tests/repl_state_tests.rs`
  - `crates/ld-cli/tests/tool_parser_tests.rs`
  - `crates/ld-cli/tests/agent_dispatch_tests.rs`
