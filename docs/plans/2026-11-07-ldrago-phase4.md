# Phase 4: Mutations, Sandboxing & Reversible Workspaces Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a secure, sandboxed mutation engine (`crates/ld-sandbox`) with path containment, environment sanitization, command timeouts, atomic unified diff patching with backups, workspace checkpoint/rollback snapshots, and human-in-the-loop CLI prompts.

**Architecture:** A path-containment and sanitizing execution runner (`runner.rs`), a unified line-diff patch generator (`patch.rs`), and an atomic file-snapshot rollback engine (`checkpoint.rs`) in `ld-sandbox`, integrated into `ld-cli` with interactive safety prompts (`[y/N]`) and append-only audit logging.

**Tech Stack:** Rust, Tokio, Serde, Serde_json, Similar (unified diffs), Chrono, Clap v4.

---

### Task 1: Scaffolding `crates/ld-sandbox` & Path Containment

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ld-sandbox/Cargo.toml`
- Create: `crates/ld-sandbox/src/lib.rs`
- Create: `crates/ld-sandbox/src/containment.rs`
- Test: `crates/ld-sandbox/tests/containment_tests.rs`

**Step 1: Write the failing test**
Verify `PathContainment::validate(root, requested_path)` allows in-tree paths and rejects directory traversal attacks (e.g. `../../etc/passwd` or `..\..\Windows`).

**Step 2: Run test to verify it fails**
Run: `cargo test -p ld-sandbox --test containment_tests`
Expected: Failed to load manifest for workspace member `ld-sandbox`

**Step 3: Implement containment logic**
Canonicalizes root and candidate path; prevents escaping root boundaries; rejects forbidden absolute roots.

**Step 4: Run test to verify it passes**
Run: `cargo test -p ld-sandbox --test containment_tests`
Expected: PASS

**Step 5: Commit**
`feat(sandbox): scaffold ld-sandbox crate with path containment validator`

---

### Task 2: Sandboxed Command Runner with Sanitization

**Files:**
- Create: `crates/ld-sandbox/src/runner.rs`
- Modify: `crates/ld-sandbox/src/lib.rs`
- Test: `crates/ld-sandbox/tests/runner_tests.rs`

**Step 1: Write the failing test**
Verify command runner executes within timeout, strips secret environment variables, and captures stdout/stderr.

**Step 2: Run test to verify it fails**
Run: `cargo test -p ld-sandbox --test runner_tests`
Expected: Module `runner` not found

**Step 3: Implement `CommandRunner`**
Uses `tokio::process::Command`, applies 30s timeout, sanitizes environment (`AWS_*`, `*KEY*`, `*TOKEN*`, `*SECRET*`), captures stdout/stderr with output size capping.

**Step 4: Run test to verify it passes**
Run: `cargo test -p ld-sandbox --test runner_tests`
Expected: PASS

**Step 5: Commit**
`feat(sandbox): implement sandboxed command runner with timeouts and env sanitization`

---

### Task 3: Unified Diff & Atomic Patch Generator

**Files:**
- Create: `crates/ld-sandbox/src/patch.rs`
- Modify: `crates/ld-sandbox/src/lib.rs`
- Test: `crates/ld-sandbox/tests/patch_tests.rs`

**Step 1: Write the failing test**
Verify unified diff calculation, dry-run preview, atomic file modification, and automatic `.bak` backup creation.

**Step 2: Run test to verify it fails**
Run: `cargo test -p ld-sandbox --test patch_tests`
Expected: Module `patch` not found

**Step 3: Implement `PatchEngine`**
Uses `similar::TextDiff` to generate standard unified diffs; writes atomically; creates `.bak` restore points.

**Step 4: Run test to verify it passes**
Run: `cargo test -p ld-sandbox --test patch_tests`
Expected: PASS

**Step 5: Commit**
`feat(sandbox): implement unified diff patch engine with atomic writes and backup recovery`

---

### Task 4: Checkpoint & Rollback Snapshot Engine

**Files:**
- Create: `crates/ld-sandbox/src/checkpoint.rs`
- Modify: `crates/ld-sandbox/src/lib.rs`
- Test: `crates/ld-sandbox/tests/checkpoint_tests.rs`

**Step 1: Write the failing test**
Verify creating a workspace checkpoint, listing checkpoints, and restoring files back to pre-mutation state.

**Step 2: Run test to verify it fails**
Run: `cargo test -p ld-sandbox --test checkpoint_tests`
Expected: Module `checkpoint` not found

**Step 3: Implement `CheckpointManager`**
Copies modified files into `.ldrago/checkpoints/<id>/`, writes manifest JSON, and restores on rollback.

**Step 4: Run test to verify it passes**
Run: `cargo test -p ld-sandbox --test checkpoint_tests`
Expected: PASS

**Step 5: Commit**
`feat(sandbox): implement workspace checkpoint and atomic rollback engine`

---

### Task 5: CLI Integration (`ldrago run`, `ldrago patch`, `ldrago checkpoint`)

**Files:**
- Modify: `crates/ld-cli/Cargo.toml`
- Create: `crates/ld-cli/src/commands/run.rs`
- Create: `crates/ld-cli/src/commands/patch.rs`
- Create: `crates/ld-cli/src/commands/checkpoint.rs`
- Modify: `crates/ld-cli/src/commands/mod.rs`
- Modify: `crates/ld-cli/src/lib.rs`
- Modify: `crates/ld-cli/src/main.rs`
- Test: `crates/ld-cli/tests/phase4_cli_tests.rs`

**Step 1: Write the failing test**
Verify `ldrago run`, `ldrago patch`, and `ldrago checkpoint` subcommands work via CLI with `--yes` non-interactive bypass.

**Step 2: Run test to verify it fails**
Run: `cargo test -p ld-cli --test phase4_cli_tests`
Expected: Subcommands not recognized

**Step 3: Implement CLI commands and interactive prompt**
Prompt for confirmation `[y/N]` when `--yes` is not set; log executions to `.ldrago/audit.jsonl`.

**Step 4: Run test to verify it passes**
Run: `cargo test -p ld-cli --test phase4_cli_tests`
Expected: PASS

**Step 5: Commit**
`feat(cli): add run, patch, and checkpoint commands with human-in-the-loop approvals`

---

### Task 6: Full Verification, Dashboard Update & GitHub Sync

- Run all tests across workspace: `cargo test --workspace -j 1`
- Run all Python tests: `uv run pytest -v`
- Test live CLI commands: `ldrago checkpoint create`, `ldrago patch`, `ldrago rollback`
- Update `progress-dashboard.md` artifact to 83.3% completion (Phase 4 complete)
- Git commit and push to `origin/main`
