# Phase 5: Interactive TUI, Distribution & Production Hardening Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement an interactive terminal user interface (TUI) powered by `ratatui` & `crossterm`, build multi-target automated CI release workflows, write cross-platform one-line installers, and finalize documentation.

**Architecture:** 
- `crates/ld-cli/src/tui/`: Event-driven terminal UI utilizing `ratatui` with a 3-pane responsive layout (Engine Switcher, Chat/Prompt Stream, and Telemetry/Security Panel).
- Non-blocking async event loop using `tokio::sync::mpsc` for seamless prompt submission and real-time streaming token reception from PyTorch and Ollama backends.
- `.github/workflows/release.yml`: Multi-platform matrix build across `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`, and `aarch64-apple-darwin`.
- Standalone installer scripts: PowerShell (`install.ps1`) for Windows and Bash (`install.sh`) for macOS/Linux.

**Tech Stack:**
- Rust 2021 (`ratatui`, `crossterm`, `tokio`, `clap`, `serde`)
- GitHub Actions CI/CD matrix
- PowerShell & POSIX Shell

---

### Task 1: TUI Dependencies & State Management
- Add `ratatui = "0.28"` and `crossterm = "0.28"` to workspace dependencies.
- Create `crates/ld-cli/src/tui/app.rs` with `AppState`, `ActivePane`, input buffer, conversation turns, and brain statuses.
- Write unit tests verifying state transitions, input keyhandling, and pane navigation in `crates/ld-cli/tests/tui_state_tests.rs`.

### Task 2: Responsive Terminal UI Rendering
- Implement `crates/ld-cli/src/tui/ui.rs`:
  - Panel 1: Brain status and selection with color-coded health indicators (🟢 / 🔴).
  - Panel 2: Interactive message history with Beyblade Dragon banner and user input area.
  - Panel 3: Live telemetry (KV-cache tokens/sec, policy gate verdicts, checkpoint count).
- Implement `crates/ld-cli/src/tui/mod.rs` with terminal raw mode setup, event polling, and graceful cleanup on exit.

### Task 3: CLI Subcommand Integration
- Register `ldrago tui` subcommand in `crates/ld-cli/src/lib.rs` and `main.rs`.
- Wire `execute_tui` to connect to `ModelRouter` and stream responses dynamically into the UI.
- Write headless UI render test verifying layout initialization.

### Task 4: Cross-Platform Release CI/CD & Automated Installers
- Create `.github/workflows/release.yml` with cross-compilation matrix, SHA256 checksums, and GitHub Releases asset attachments.
- Create `install.ps1` (PowerShell installer for Windows with PATH detection).
- Create `install.sh` (POSIX curl installer for Linux/macOS with arch detection).

### Task 5: Documentation Polish & Full 100% Roadmap Achievement
- Update `README.md` with:
  - Exclusive L-Drago Beyblade Dragon terminal banner
  - Architecture specifications (Transformer + SSM Mamba + Ollama)
  - Quickstart guide and command reference table (`ldrago tui`, `ldrago benchmark`, `ldrago checkpoint`, `ldrago run`, `ldrago patch`)
- Update `crates/ld-cli/src/commands/progress.rs` to display **100% completion (6/6 phases)**.
- Run workspace tests (`cargo test --workspace -j 1`) and Python test suite.
- Update `progress-dashboard.md` artifact to **100% complete**.
- Commit and push to `origin/main`.
