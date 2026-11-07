# 🐉 LDrago CLI — Local Twin AI Platform

```
           __  __            ____                                 
          / / / /   ____    / __ \ _____ ____ _ ____ _ ____       
         / / / /   / __ \  / / / // ___// __ `// __ `// __ \      
        / /_/ /___/ /_/ / / /_/ // /   / /_/ // /_/ // /_/ /      
       /_____//_____/___/ /_____//_/    \__,_/ \__, / \____/       
                                              /____/              
   ⚡ LIGHTNING / METEO L-DRAGO • LOCAL TWIN AI PLATFORM ⚡
```

[![CI Release](https://github.com/Syed-afridi-7/LDrago_CLI/actions/workflows/release.yml/badge.svg)](https://github.com/Syed-afridi-7/LDrago_CLI/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-gold.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-2021_Edition-crimson.svg)](https://www.rust-lang.org/)
[![PyTorch](https://img.shields.io/badge/PyTorch-2.0%2B_From_Scratch-blue.svg)](https://pytorch.org/)
[![Tests Passing](https://img.shields.io/badge/Tests-146%20Passing-brightgreen.svg)]()
[![Roadmap](https://img.shields.io/badge/Roadmap-100%25%20Complete-gold.svg)]()

> **LDrago CLI** is a modular, high-performance, production-ready local AI terminal platform. It features **two custom neural network architectures engineered completely from first principles in pure PyTorch** (Transformer & State-Space Model / Mamba) alongside local Ollama runtime integration, a zero-trust policy engine, an atomic sandboxed workspace mutator, and an interactive 3-pane Terminal User Interface (TUI).

---

## 🌟 Key Architectural Pillars

### 1. 🧠 Custom Brain #1: Coding Engine (Pure PyTorch Transformer)
- Built from scratch in `training/src/models/transformer.py` (`LDragoCoder`).
- **Grouped-Query Attention (GQA)**: Configurable query heads with shared key/value heads for memory efficiency.
- **Rotary Position Embeddings (RoPE)**: Relative position encoding precomputed across complex frequencies.
- **Pre-RMSNorm**: Root Mean Square Layer Normalization with learnable scaling factors.
- **SwiGLU Feed-Forward Networks**: Gated activation non-linearities for gradient stability.
- **Static KV-Cache**: Zero-allocation static tensor buffers enabling $O(1)$ token step latency.
- **Native Dynamic INT8 Quantization**: 8-bit dynamic symmetric matrix multiplication saving ~75% RAM with zero third-party dependencies.

### 2. 🌀 Custom Brain #2: Design Engine (Pure PyTorch State-Space Model / Mamba)
- Built from scratch in `training/src/models/ssm.py` (`LDragoDesigner`).
- **1D Causal Depthwise Convolution**: Local temporal receptive field filtering.
- **Selective Scan Recurrence**: Discretized continuous state-space parameters ($\Delta$, $A$, $B$, $C$) varying dynamically per input token.
- **Constant Memory Footprint**: Linear time complexity $O(L)$ during training, $O(1)$ state evolution per generation step.

### 3. 🦙 Local Engine #3: General Runtime (Ollama Adapter)
- High-throughput local runtime integration via `ld-models` for broad tasks and fallback capabilities.

### 4. 🛡️ Zero-Trust Policy Engine & Path Containment Sandbox (`crates/ld-policy`, `crates/ld-sandbox`)
- **Safety Gate**: Rejects prompt injection, jailbreaks, destructive shell commands, and secret credential leaks.
- **Path Containment**: Workspace boundary traversal validator blocking `../` escaping or unauthorized symlink manipulation.
- **Subprocess Runner**: Scrubbed execution environment automatically stripping API secrets (`AWS_ACCESS`, `TOKEN`, `KEY`, `PASSWORD`) and enforcing execution timeouts.
- **Atomic Diff Patching & Reversible Checkpoints**: Unified diff previews with automatic `.bak` backups and instant workspace rollback (`ldrago rollback` / `ldrago checkpoint restore`).

### 5. 🖥️ Interactive 3-Pane Terminal UI (`crates/ld-cli/src/tui`)
- Built with `ratatui` and `crossterm`.
- **Engines Panel**: Live health probes and instant switching between Coding, Design, and General brains.
- **Chat Panel**: Dynamic conversational stream with TrueColor formatting, auto-scrolling, and interactive input.
- **Telemetry Panel**: Real-time generation velocity (tokens/sec), active checkpoints count, policy engine state, and path containment status.

---

## 🚀 Quickstart & Installation

### Option A: One-Line Automated Installer

#### Windows (PowerShell):
```powershell
irm https://raw.githubusercontent.com/Syed-afridi-7/LDrago_CLI/main/install.ps1 | iex
```

#### macOS & Linux (Bash/cURL):
```bash
curl -fsSL https://raw.githubusercontent.com/Syed-afridi-7/LDrago_CLI/main/install.sh | bash
```

---

### Option B: Build From Source

Prerequisites:
- [Rust 1.80+](https://rustup.rs)
- Python 3.10+ & [uv](https://github.com/astral-sh/uv)

```bash
# 1. Clone repository
git clone https://github.com/Syed-afridi-7/LDrago_CLI.git
cd LDrago_CLI

# 2. Build the Rust workspace
cargo build --release --workspace

# 3. Setup Python training environment and start local model daemon
cd training
uv venv
uv run uvicorn src.serve.server:app --host 127.0.0.1 --port 8741
```

In another terminal:
```bash
# Verify system connectivity
ldrago doctor

# Launch the interactive terminal UI
ldrago tui
```

---

## 🕹️ CLI Command Reference

| Command | Description | Example |
|---|---|---|
| `ldrago tui` | Launch the full 3-pane interactive Terminal UI | `ldrago tui` |
| `ldrago doctor` | Run diagnostic health probes on all 3 brains | `ldrago doctor` |
| `ldrago progress` | Display live dynamic development roadmap | `ldrago progress` |
| `ldrago benchmark` | Profile throughput (tok/s) and latency | `ldrago benchmark --brain coding --tokens 32` |
| `ldrago logo` | Display exclusive L-Drago Beyblade Dragon logo | `ldrago logo` |
| `ldrago run` | Execute a command safely in the scrubbed sandbox | `ldrago run "cargo check" --yes` |
| `ldrago patch` | Atomically apply unified diff with `.bak` safety | `ldrago patch src/lib.rs --content "..." --yes` |
| `ldrago rollback` | Revert file to backup generated during patch | `ldrago rollback src/lib.rs` |
| `ldrago checkpoint`| Create, list, or restore workspace snapshot | `ldrago checkpoint create -d "pre-refactor"` |
| `ldrago plan` | Generate grounded plan validated with schemas | `ldrago plan "Build user auth service"` |
| `ldrago context` | Scan workspace tree, manifest, or symbols | `ldrago context tree` |
| `ldrago ask` | Send prompt to a brain with verified citations | `ldrago ask "Write binary search in Rust" --brain coding` |
| `ldrago train` | Run local PyTorch training harness | `ldrago train coding --micro --steps 20` |

---

## 🧪 Comprehensive Automated Test Suite

LDrago maintains **146 / 146 automated tests passing** (83 Python tests + 63 Rust tests):

```bash
# Run Rust workspace test suite
cargo test --workspace -j 1

# Run Python training, model, and eval test suite
cd training
uv run pytest -v
```

---

## 🗺️ 6-Phase Roadmap & Delivery Matrix

| Phase | Milestone | Deliverables | Tests | Status |
| :---: | :--- | :--- | :---: | :---: |
| **Phase 0** | **System Scaffolding & Core Neural Architectures** | Pure PyTorch `LDragoCoder` (GQA/RoPE/SwiGLU), `LDragoDesigner` (Mamba SSM), custom 256-byte BPE tokenizer, FastAPI daemon, Rust model contracts. | 35 / 35 | ✅ **100%** |
| **Phase 1** | **Synthetic Data, Training Loop & Convergence** | Secret redaction, license filtering, `build_adamw`, `CosineWarmupScheduler`, `CheckpointManager`, micro-convergence (>93% loss decrease). | 57 / 57 | ✅ **100%** |
| **Phase 2** | **Context Engine, Policy Enforcement & Grounded Planning** | High-throughput scanner (`ignore`), keyword symbol indexer, verified file citation extractor, zero-trust safety gate, JSONL audit trail. | 14 / 14 | ✅ **100%** |
| **Phase 3** | **Scale Pretraining & Model Optimization** | 125M parameter tier configs, pure PyTorch dynamic INT8 quantization, static pre-allocated KV-cache, `torchrun` DDP harness, RunPod A100 provisioning. | 15 / 15 | ✅ **100%** |
| **Phase 4** | **Sandboxing, Mutations & Reversible Workspaces** | `PathContainment` boundary validation, `CommandRunner` subprocess sandbox with env scrubbing, `PatchEngine` atomic diffs, `CheckpointManager`. | 14 / 14 | ✅ **100%** |
| **Phase 5** | **Interactive TUI, Packaging & Production Hardening** | Responsive 3-pane `ratatui` + `crossterm` TUI, multi-target GitHub Actions release workflow, one-line PowerShell & Bash installers. | 11 / 11 | ✅ **100%** |

---

## ⚖️ Zero Third-Party Foundation Model Guarantee

LDrago's custom brains (`LDragoCoder` and `LDragoDesigner`) do **NOT** depend on HuggingFace Transformers, OpenAI API, Anthropic API, or third-party pre-trained weights. Every neural layer, attention matrix, convolution state, and tokenizer merge is written from first principles in standard PyTorch and Rust.

---

## 📜 License
Distributed under the MIT License. See `LICENSE` for more information.
