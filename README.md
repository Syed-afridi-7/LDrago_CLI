# LDrago_CLI

> **Modular, Production-Ready Local AI CLI Platform**  
> Features two custom neural network architectures built from scratch in pure PyTorch (Transformer & State-Space Model / Mamba) alongside seamless local Ollama runtime integration.

---

## Overview

LDrago is a terminal-first AI CLI tool engineered for software development and UX design workflows. It provides a deterministic, human-controlled orchestration layer around three AI engines:

1. **Coding Brain (Custom Architecture #1):** A GPT-style decoder-only Transformer with Grouped-Query Attention (GQA), Rotary Positional Embeddings (RoPE), Pre-RMSNorm, and SwiGLU FFN built from first principles in pure PyTorch.
2. **Design Brain (Custom Architecture #2):** A State-Space Model (SSM / Mamba-style) with input-dependent selective scan recurrence, 1D causal depthwise convolution, and gated SiLU outputs built from first principles in pure PyTorch.
3. **General Runtime (Engine #3):** A local model adapter connecting to Ollama for production-capable workflows.

---

## Repository Structure

```
LDrago_CLI/
├── Cargo.toml                    # Multi-crate Rust workspace
├── crates/
│   ├── ld-schemas/               # Core data contracts & session types
│   ├── ld-models/                # ModelAdapter trait, custom brain & Ollama adapters, router
│   └── ld-cli/                   # High-performance CLI binary (ldrago)
├── brains/
│   ├── coding/                   # Coding Brain contract, system prompts, schemas, tools
│   └── design/                   # Design Brain contract, system prompts, schemas, tools
├── training/
│   ├── src/
│   │   ├── tokenizer/            # Custom Byte-Pair Encoding (BPE) from scratch
│   │   ├── models/               # Pure PyTorch LDragoCoder (Transformer) & LDragoDesigner (SSM)
│   │   └── serve/                # FastAPI model server hosting custom brains
│   └── tests/                    # Pytest test suite (22 tests passing)
└── docs/
    ├── superpowers/specs/        # Approved design specification
    └── plans/                    # Phase-by-phase implementation plans
```

---

## Quickstart

### 1. Build the Rust CLI
Prerequisites: Rust 1.80+ / Cargo

```bash
# Build the workspace
cargo build --workspace

# Run all Rust tests (13 tests)
cargo test --workspace
```

### 2. Start the Custom Brain Model Server
Prerequisites: Python 3.11+ and `uv`

```bash
cd training

# Install dependencies (CPU PyTorch + FastAPI)
uv sync

# Run all Python tests (22 tests)
uv run pytest tests/ -v

# Start the local model server daemon
uv run uvicorn src.serve.server:app --host 127.0.0.1 --port 8741
```

### 3. Using the LDrago CLI

```bash
# Check system health across all 3 AI engines
cargo run --bin ldrago -- doctor

# List registered models and capability badges
cargo run --bin ldrago -- model list

# Ask Coding Brain (Custom Transformer)
cargo run --bin ldrago -- ask "Write a function to calculate Fibonacci" --brain coding

# Ask Design Brain (Custom SSM / Mamba)
cargo run --bin ldrago -- ask "Design a multi-step checkout user flow" --brain design

# Auto-Routing (detects intent from instruction)
cargo run --bin ldrago -- ask "Create user registration flow"
```

---

## Testing & Quality Bar

- **Rust Suite:** 13/13 passing (`cargo test --workspace`)
- **Python Suite:** 22/22 passing (`uv run pytest tests/ -v`)
- **Zero External Dependencies for Core Brains:** Pure PyTorch tensor algebra and custom BPE tokenization.
