# LDrago CLI — Local Autonomous Coding Agent

```text
>_ LDrago • Local • Autonomous • [Coding]
```

[![CI Release](https://github.com/Syed-afridi-7/LDrago_CLI/actions/workflows/release.yml/badge.svg)](https://github.com/Syed-afridi-7/LDrago_CLI/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-2021_Edition-orange.svg)](https://www.rust-lang.org/)
[![PyTorch](https://img.shields.io/badge/PyTorch-2.0%2B_From_Scratch-ee4c2c.svg)](https://pytorch.org/)
[![Tests Passing](https://img.shields.io/badge/Tests-All%20Passing-brightgreen.svg)]()

> **LDrago CLI** is a local-first, autonomous coding agent and multi-engine AI orchestrator. It couples **two custom neural architectures engineered from first principles in pure PyTorch** (Transformer & State-Space Model / Mamba) with local Ollama runtime adapters, a zero-trust policy engine, an atomic sandboxed workspace mutator, and a streaming Claude Code-style interactive agentic REPL.

---

## Key Capabilities

- **Interactive Agentic REPL**: Natural terminal streaming interface with full scrollback, dynamic prompt prefixes (`ldrago (coding) > `), slash commands (`/model`, `/status`, `/help`, `/clear`, `/exit`), and autonomous tool execution.
- **Local Brain System Access**: Deterministic tool execution for autonomous file reading (`read_file`), whole-file creation (`write_file`), atomic unified diff patching (`patch_file`), directory inspection (`list_files`), and sandboxed execution (`run_command`).
- **Workspace Path Containment**: Strict boundary enforcement preventing path traversal (`../`) and unauthorized symlink manipulation outside the active repository.
- **Zero-Trust Security Gate**: Automated AST/regex safety gate blocking prompt injections, destructive commands (`rm -rf`, disk wipes), and secret credential leakage.
- **Silent Background Telemetry**: Sub-millisecond latency profiling and token generation metrics collected silently without terminal output pollution.
- **Custom Neural Architectures**: Zero third-party foundation model dependencies. Two custom architectures written from first principles in standard PyTorch and Rust.

---

## Interactive Agentic REPL

Launch the agent by executing `ldrago` with no arguments (or via `ldrago repl`):

```bash
$ ldrago
>_ LDrago • Local • Autonomous • [Coding]

ldrago (coding) > Create a Rust function that calculates Fibonacci numbers and run tests
[Executing tool: write_file src/fib.rs]
[Tool Output]:
Successfully wrote 142 bytes to src/fib.rs

[Executing tool: run_command cargo test]
[Tool Output]:
   Compiling demo v0.1.0
    Finished test [unoptimized + debuginfo] target(s)
     Running unittests src/lib.rs
test fib::tests::test_fib ... ok
test result: ok. 1 passed; 0 failed

I have created `src/fib.rs` and verified that all unit tests pass.
ldrago (coding) > 
```

### Slash Commands

| Command | Arguments | Description |
|---|---|---|
| `/model` | `[coding \| design \| general]` | Switch active model brain. Without arguments, cycles through available models. |
| `/status` | *(none)* | Displays active brain engine, workspace path, containment state, and session statistics. |
| `/help` | *(none)* | Prints interactive command reference and tool usage guidelines. |
| `/clear` | *(none)* | Clears terminal screen and re-renders the clean 1-line technical banner. |
| `/exit` | *(none)* | Gracefully terminates the REPL session (`quit` and `exit` also supported). |

---

## Autonomous Tool Protocol

LDrago's agent loop monitors model completions for deterministic tool invocation markers. When a tool call is detected, the agent pauses generation, validates security constraints, executes the operation, and feeds the output back into context for iterative multi-turn reasoning.

### Supported Tools

1. **`read_file <path>`**  
   Reads file content from the active workspace subject to path containment checks.  
   ```text
   TOOL: read_file crates/ld-cli/src/lib.rs
   ```

2. **`write_file <path>`**  
   Atomically writes content to disk, automatically generating `.bak` backups of modified files.  
   ```text
   TOOL: write_file src/math.rs
   pub fn add(a: i32, b: i32) -> i32 { a + b }
   TOOL_END
   ```

3. **`patch_file <path>`**  
   Applies deterministic block replacements without overwriting the entire file.  
   ```text
   TOOL: patch_file src/math.rs
   <<<TARGET
   pub fn add(a: i32, b: i32) -> i32 { a + b }
   ===REPLACE
   pub fn add(a: i64, b: i64) -> i64 { a + b }
   >>>
   ```

4. **`run_command <command>`**  
   Executes a shell command inside an environment-scrubbed sandbox subprocess. Sensitive tokens (`AWS_SECRET`, `GITHUB_TOKEN`, passwords) are stripped before execution.  
   ```text
   TOOL: run_command cargo check
   ```

5. **`list_files [optional_path]`**  
   Lists repository files while respecting `.gitignore` patterns and directory boundaries.  
   ```text
   TOOL: list_files src
   ```

---

## Zero-Trust Safety & Background Telemetry

### Path Containment Sandbox (`crates/ld-sandbox`)
All filesystem and execution tools run inside a sandboxed dispatcher:
- **Canonicalization**: Paths are resolved to their canonical root; attempts to escape via `../` or symlinks return immediate containment errors.
- **Environment Scrubbing**: Subprocesses run in an isolated environment with credentials and secrets filtered out.
- **Automatic Rollbacks**: Any file patch generates an atomic backup (`.bak`), restorable at any time with `ldrago rollback <file>`.

### Audit Logging & Telemetry Hooks (`crates/ld-cli/src/agent/dispatcher.rs`)
- **JSONL Audit Trail**: All tool calls, system commands, and policy decisions are appended quietly to `~/.ldrago/audit.jsonl` with timestamps and hashes.
- **Silent Telemetry**: Generation latency and token rates (tokens/sec) are recorded in the background without polluting terminal stdout.
- **Extension Points**: The codebase provides explicit code comment hooks for enterprise SIEM forwarding and custom authorization layers:
  - `record_tool_audit_trail`: Hook for structured SIEM and compliance logging.
  - `record_inference_telemetry`: Hook for latency, throughput, and hardware metrics.

---

## Core Neural Architectures

LDrago's native brains are engineered entirely from first principles with zero dependencies on third-party model weights:

### 1. Coding Brain: Pure PyTorch Transformer (`LDragoCoder`)
- Located in `training/src/models/transformer.py`.
- **Grouped-Query Attention (GQA)**: Configurable query heads with shared key/value heads for memory efficiency.
- **Rotary Position Embeddings (RoPE)**: Relative position encoding precomputed across complex frequencies.
- **Pre-RMSNorm**: Root Mean Square Layer Normalization with learnable scaling factors.
- **SwiGLU Non-Linearities**: Gated activation functions providing training stability.
- **Static KV-Cache**: Pre-allocated static tensor buffers enabling $O(1)$ token step latency during autoregressive generation.
- **Native Dynamic INT8 Quantization**: Custom 8-bit dynamic symmetric matrix multiplication saving ~75% RAM with zero external C++ extensions.

### 2. Design Brain: Pure PyTorch State-Space Model (`LDragoDesigner`)
- Located in `training/src/models/ssm.py`.
- **1D Causal Depthwise Convolution**: Local temporal receptive field filtering.
- **Selective Scan Recurrence**: Discretized continuous state-space parameters ($\Delta$, $A$, $B$, $C$) varying dynamically per input token.
- **Constant Memory Footprint**: Linear time complexity $O(L)$ during training, with $O(1)$ state evolution per generation step.

### 3. General Runtime Engine (Ollama Adapter)
- High-throughput integration with local Ollama runtimes for general conversational queries and fallback tasks.

---

## Installation & Quickstart

### Automated Installer

#### Windows (PowerShell):
```powershell
irm https://raw.githubusercontent.com/Syed-afridi-7/LDrago_CLI/main/install.ps1 | iex
```

#### macOS & Linux (Bash/cURL):
```bash
curl -fsSL https://raw.githubusercontent.com/Syed-afridi-7/LDrago_CLI/main/install.sh | bash
```

---

### Build From Source

**Prerequisites:**
- [Rust 1.80+](https://rustup.rs)
- Python 3.10+ & [uv](https://github.com/astral-sh/uv)

```bash
# 1. Clone repository
git clone https://github.com/Syed-afridi-7/LDrago_CLI.git
cd LDrago_CLI

# 2. Build release binary
cargo build --release --workspace

# 3. Setup Python training environment and start local model daemon
cd training
uv venv
uv run uvicorn src.serve.server:app --host 127.0.0.1 --port 8741
```

In a separate terminal, launch the agent:
```bash
# Run health diagnostics
ldrago doctor

# Launch the interactive agentic REPL
ldrago
```

---

## CLI Command Reference

| Command | Description | Example |
|---|---|---|
| `ldrago` / `ldrago repl` | Launch the Claude Code-style interactive agentic REPL | `ldrago` |
| `ldrago doctor` | Run diagnostic health probes across all registered engines | `ldrago doctor` |
| `ldrago model list` | List registered model engines and connection status | `ldrago model list` |
| `ldrago progress` | Display development roadmap and milestones | `ldrago progress` |
| `ldrago benchmark` | Profile throughput (tokens/sec) and latency | `ldrago benchmark --brain coding --tokens 32` |
| `ldrago logo` | Display clean single-line technical banner | `ldrago logo` |
| `ldrago run` | Execute a command safely in the scrubbed sandbox | `ldrago run "cargo check" --yes` |
| `ldrago patch` | Atomically apply unified diff with `.bak` safety | `ldrago patch src/lib.rs --content "..." --yes` |
| `ldrago rollback` | Revert file to backup generated during patch | `ldrago rollback src/lib.rs` |
| `ldrago checkpoint` | Create, list, or restore workspace snapshots | `ldrago checkpoint create -d "pre-refactor"` |
| `ldrago plan` | Generate grounded plan validated with JSON schemas | `ldrago plan "Build user auth service"` |
| `ldrago context` | Scan workspace tree, manifest, or symbols | `ldrago context tree` |
| `ldrago ask` | Non-interactive one-shot prompt evaluation | `ldrago ask "Write binary search in Rust" --brain coding` |
| `ldrago train` | Run local PyTorch training harness | `ldrago train coding --micro --steps 20` |
| `ldrago version` | Print LDrago version and system architecture info | `ldrago version` |

---

## Verification & Testing

LDrago maintains a comprehensive test suite across Rust and Python workspaces:

```bash
# Run Rust workspace test suite
cargo test --workspace -j 1

# Run specific REPL and agent dispatch integration tests
cargo test --test repl_integration_tests --test agent_dispatch_tests --test tool_parser_tests --test repl_state_tests --test logo_tests

# Run Python training, model, and evaluation test suite
cd training
uv run pytest -v
```

---

## Zero Third-Party Model Guarantee

LDrago's custom brains (`LDragoCoder` and `LDragoDesigner`) do **NOT** depend on HuggingFace Transformers, OpenAI API, Anthropic API, or third-party pre-trained weights. Every neural layer, attention matrix, convolution state, and tokenizer merge is written from first principles in standard PyTorch and Rust.

---

## License

Distributed under the MIT License. See `LICENSE` for more information.
