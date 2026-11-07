# LDrago CLI Phase 0 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the Phase 0 foundational runtime for LDrago CLI, including the Rust CLI skeleton, shared schemas, custom BPE tokenizer, custom Transformer & State-Space Model architectures in pure PyTorch, a local model server, model adapter traits (custom brains & Ollama), and initial commands (`version`, `doctor`, `model list`, `ask`).

**Architecture:** A high-performance Rust CLI (`clap`, `tokio`, `serde`, `reqwest`) serves as the user-facing orchestrator. It connects over local HTTP to a Python FastAPI service hosting two custom neural networks (a Transformer for Coding Brain and a Mamba-style SSM for Design Brain) alongside an adapter for local Ollama runtimes. All models conform to a unified model abstraction trait with seamless switching and context preservation.

**Tech Stack:** Rust 2024 (`clap`, `tokio`, `serde`, `serde_json`, `figment`, `reqwest`, `thiserror`), Python 3.11/3.12 (`torch`, `fastapi`, `uvicorn`, `pydantic`), `uv`, `git`.

---

### Task 1: Rust Workspace Root & Crates Scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `crates/ld-schemas/Cargo.toml`
- Create: `crates/ld-schemas/src/lib.rs`
- Create: `crates/ld-models/Cargo.toml`
- Create: `crates/ld-models/src/lib.rs`
- Create: `crates/ld-cli/Cargo.toml`
- Create: `crates/ld-cli/src/main.rs`
- Create: `.gitignore`

**Step 1: Write root workspace `Cargo.toml` and `.gitignore`**

Create root `Cargo.toml` configuring the workspace members (`crates/ld-schemas`, `crates/ld-models`, `crates/ld-cli`) and common dependencies with unified versions.
Create `.gitignore` for target directories, python venvs, and cache files.

**Step 2: Create initial crate structures**

Create stub files for `ld-schemas`, `ld-models`, and `ld-cli`.

**Step 3: Run build to verify workspace configuration**

Run: `cargo build --workspace`  
Expected: Finished `dev` profile with all three crates compiled cleanly.

**Step 4: Commit**

```bash
git add Cargo.toml .gitignore crates/
git commit -m "feat: scaffold Rust multi-crate workspace for LDrago CLI"
```

---

### Task 2: Core Schemas in `ld-schemas`

**Files:**
- Modify: `crates/ld-schemas/src/lib.rs`
- Create: `crates/ld-schemas/tests/schema_tests.rs`

**Step 1: Write the failing test for schema serialization**

Test serialization and deserialization of `BrainType`, `Role`, `Turn`, `ModelCapabilities`, `CompletionRequest`, and `CompletionResponse`.

```rust
use ld_schemas::*;

#[test]
fn test_serde_turn_and_completion() {
    let turn = Turn {
        role: Role::User,
        content: "Explain auth flow".to_string(),
        tool_calls: vec![],
        timestamp: "2026-11-07T12:00:00Z".to_string(),
    };
    let json = serde_json::to_string(&turn).unwrap();
    let deserialized: Turn = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.role, Role::User);
    assert_eq!(deserialized.content, "Explain auth flow");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p ld-schemas`  
Expected: Compilation failure due to missing types.

**Step 3: Implement core schemas**

Define enums and structs in `crates/ld-schemas/src/lib.rs`:
- `BrainType`: `Coding`, `Design`, `General`
- `Role`: `System`, `User`, `Assistant`, `Tool`
- `Turn`: message sequence struct
- `ModelCapabilities`: alias, endpoint, model_name, capabilities, context_window, privacy, brain
- `CompletionRequest` & `CompletionResponse`
- `HealthStatus`: `Healthy`, `Degraded`, `Unhealthy`, `Unknown`

**Step 4: Run test to verify it passes**

Run: `cargo test -p ld-schemas`  
Expected: PASS with all serialization tests succeeding.

**Step 5: Commit**

```bash
git add crates/ld-schemas/
git commit -m "feat(schemas): implement core data transfer and session types"
```

---

### Task 3: Python Environment & Scaffolding in `training/`

**Files:**
- Create: `training/pyproject.toml`
- Create: `training/src/__init__.py`
- Create: `training/tests/__init__.py`

**Step 1: Define Python project configuration**

Write `training/pyproject.toml` with dependencies: `torch`, `fastapi`, `uvicorn`, `pydantic`, `pytest`, `pyyaml`.

**Step 2: Initialize venv and install dependencies with `uv`**

Run: `cd training && uv sync`  
Expected: Virtual environment created with PyTorch and FastAPI installed.

**Step 3: Write a smoke test for PyTorch availability**

Create `training/tests/test_torch_smoke.py`:
```python
import torch

def test_torch_cpu_tensor():
    t = torch.randn(2, 4)
    assert t.shape == (2, 4)
    assert t.device.type == "cpu"
```

**Step 4: Run test to verify it passes**

Run: `cd training && uv run pytest tests/test_torch_smoke.py -v`  
Expected: 1 passed.

**Step 5: Commit**

```bash
git add training/
git commit -m "chore(training): configure Python environment with PyTorch and pytest via uv"
```

---

### Task 4: Custom BPE Tokenizer from Scratch in Pure Python

**Files:**
- Create: `training/src/tokenizer/bpe.py`
- Create: `training/tests/test_bpe.py`

**Step 1: Write failing test for BPE train, encode, and decode**

```python
from src.tokenizer.bpe import LDragoBPETokenizer

def test_bpe_roundtrip():
    tokenizer = LDragoBPETokenizer()
    corpus = "def authenticate_user(username, password):\n    return True\n"
    tokenizer.train(corpus, target_vocab_size=270)
    
    encoded = tokenizer.encode("authenticate_user")
    decoded = tokenizer.decode(encoded)
    assert decoded == "authenticate_user"
    assert len(encoded) < len("authenticate_user")
```

**Step 2: Run test to verify it fails**

Run: `cd training && uv run pytest tests/test_bpe.py -v`  
Expected: ModuleNotFoundError or test failure.

**Step 3: Implement custom BPE Tokenizer**

In `training/src/tokenizer/bpe.py`:
- Initialize 256 base byte tokens plus `<PAD>`, `<BOS>`, `<EOS>`, `<UNK>` and domain special tokens.
- Compute pair frequencies across byte representations.
- Iteratively merge the most frequent pair and register merge rules `(token_a, token_b) -> new_id`.
- Encode strings using learned merges.
- Decode token ID lists back to valid UTF-8 strings.
- Add `save(path)` and `load(path)` methods.

**Step 4: Run test to verify it passes**

Run: `cd training && uv run pytest tests/test_bpe.py -v`  
Expected: All tests pass.

**Step 5: Commit**

```bash
git add training/src/tokenizer training/tests/test_bpe.py
git commit -m "feat(tokenizer): implement pure Python Byte-Pair Encoding tokenizer from scratch"
```

---

### Task 5: Custom Transformer Architecture (LDragoCoder) in PyTorch

**Files:**
- Create: `training/src/models/transformer.py`
- Create: `training/src/models/config.py`
- Create: `training/tests/test_transformer.py`

**Step 1: Write failing test for Transformer forward pass & shapes**

```python
import torch
from src.models.config import TransformerConfig
from src.models.transformer import LDragoCoder

def test_transformer_forward_pass():
    cfg = TransformerConfig(
        vocab_size=256,
        d_model=64,
        n_layers=2,
        n_heads=4,
        n_kv_heads=2,
        d_ff=128,
        max_seq_len=64,
    )
    model = LDragoCoder(cfg)
    tokens = torch.randint(0, 256, (2, 16))
    logits, _ = model(tokens)
    assert logits.shape == (2, 16, 256)
```

**Step 2: Run test to verify it fails**

Run: `cd training && uv run pytest tests/test_transformer.py -v`  
Expected: ModuleNotFoundError.

**Step 3: Implement LDragoCoder Transformer from scratch**

In `training/src/models/transformer.py`:
- `RMSNorm`: root-mean-square normalization.
- `precompute_rope_freqs` and `apply_rope`: rotary position encodings for complex tensors.
- `GroupedQueryAttention`: multi-head attention with grouped key-value heads and causal masking.
- `SwiGLUFFN`: feed-forward network with SiLU-gated linear projections.
- `TransformerBlock`: pre-norm residual block combining GQA and SwiGLU.
- `LDragoCoder`: embedding, N transformer blocks, final norm, and tied output linear head.

**Step 4: Run test to verify it passes**

Run: `cd training && uv run pytest tests/test_transformer.py -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add training/src/models/ training/tests/test_transformer.py
git commit -m "feat(models): implement LDragoCoder Transformer architecture with GQA and RoPE"
```

---

### Task 6: Custom State-Space Model (LDragoDesigner) in PyTorch

**Files:**
- Create: `training/src/models/ssm.py`
- Create: `training/tests/test_ssm.py`

**Step 1: Write failing test for SSM forward pass & shapes**

```python
import torch
from src.models.config import SSMConfig
from src.models.ssm import LDragoDesigner

def test_ssm_forward_pass():
    cfg = SSMConfig(
        vocab_size=256,
        d_model=64,
        n_layers=2,
        d_state=8,
        d_conv=4,
        expand=2,
        dt_rank=8,
        max_seq_len=64,
    )
    model = LDragoDesigner(cfg)
    tokens = torch.randint(0, 256, (2, 16))
    logits = model(tokens)
    assert logits.shape == (2, 16, 256)
```

**Step 2: Run test to verify it fails**

Run: `cd training && uv run pytest tests/test_ssm.py -v`  
Expected: ModuleNotFoundError.

**Step 3: Implement LDragoDesigner SSM (Mamba-style) from scratch**

In `training/src/models/ssm.py`:
- `selective_scan`: pure PyTorch implementation of the input-dependent discretization and recurrent state transition.
- `MambaBlock`: dual branch split, causal 1D depthwise convolution, low-rank delta projection with softplus, selective scan, and SiLU gating.
- `LDragoDesigner`: full model with token embeddings, stacked Mamba blocks, RMSNorm, and tied language model head.

**Step 4: Run test to verify it passes**

Run: `cd training && uv run pytest tests/test_ssm.py -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add training/src/models/ssm.py training/tests/test_ssm.py
git commit -m "feat(models): implement LDragoDesigner State-Space Model with selective scan"
```

---

### Task 7: FastAPI Model Server Hosting Both Custom Brains

**Files:**
- Create: `training/src/serve/server.py`
- Create: `training/tests/test_server.py`

**Step 1: Write failing test for model server endpoints**

```python
from fastapi.testclient import TestClient
from src.serve.server import app

def test_health_endpoint():
    client = TestClient(app)
    resp = client.get("/health")
    assert resp.status_code == 200
    assert resp.json()["status"] == "ok"
    assert "coding" in resp.json()["brains"]
    assert "design" in resp.json()["brains"]
```

**Step 2: Run test to verify it fails**

Run: `cd training && uv run pytest tests/test_server.py -v`  
Expected: ModuleNotFoundError.

**Step 3: Implement FastAPI model server**

In `training/src/serve/server.py`:
- Instantiate micro versions of `LDragoCoder` and `LDragoDesigner` with initialized weights.
- Initialize lightweight tokenizers for both brains.
- Implement `/health` returning status, available brains, and execution device (`cpu`).
- Implement `/v1/models` returning capability descriptors for each brain.
- Implement `/v1/complete` returning token generation text.

**Step 4: Run test to verify it passes**

Run: `cd training && uv run pytest tests/test_server.py -v`  
Expected: All server endpoint tests pass.

**Step 5: Commit**

```bash
git add training/src/serve/ training/tests/test_server.py
git commit -m "feat(serve): implement FastAPI server hosting both custom PyTorch brains"
```

---

### Task 8: Rust Configuration System in `ld-cli`

**Files:**
- Create: `crates/ld-cli/src/config.rs`
- Modify: `crates/ld-cli/src/main.rs`
- Create: `crates/ld-cli/tests/config_tests.rs`

**Step 1: Write failing test for config parsing**

Test loading configuration with default values, environment variable overrides, and TOML deserialization.

**Step 2: Run test to verify it fails**

Run: `cargo test -p ld-cli --test config_tests`  
Expected: Compilation failure.

**Step 3: Implement configuration system**

In `crates/ld-cli/src/config.rs`:
- Define `AppConfig`, `ModelEndpointConfig`, `PolicyConfig`.
- Implement `AppConfig::load()` using `figment` with TOML file extraction and environment prefix `LDRAGO_`.
- Provide default configuration mapping `coder-local` to `http://127.0.0.1:8741`, `designer-local` to `http://127.0.0.1:8741`, and `general-local` to `http://127.0.0.1:11434`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p ld-cli --test config_tests`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/ld-cli/src/config.rs crates/ld-cli/tests/config_tests.rs
git commit -m "feat(config): implement layered configuration management via figment and TOML"
```

---

### Task 9: Rust Model Abstraction Layer & Adapters in `ld-models`

**Files:**
- Create: `crates/ld-models/src/adapter.rs`
- Create: `crates/ld-models/src/custom_brain.rs`
- Create: `crates/ld-models/src/ollama.rs`
- Modify: `crates/ld-models/src/lib.rs`
- Create: `crates/ld-models/tests/adapter_tests.rs`

**Step 1: Write failing test for `ModelAdapter` trait implementations**

Mock server tests validating that `CustomBrainAdapter` and `OllamaAdapter` properly query `/health` and `/v1/complete`.

**Step 2: Run test to verify it fails**

Run: `cargo test -p ld-models`  
Expected: Missing modules and trait definitions.

**Step 3: Implement `ModelAdapter` trait, `CustomBrainAdapter`, and `OllamaAdapter`**

- `ModelAdapter`: async trait with `health_check`, `complete`, `capabilities`.
- `CustomBrainAdapter`: communicates with Python server at port 8741.
- `OllamaAdapter`: communicates with Ollama at port 11434.

**Step 4: Run test to verify it passes**

Run: `cargo test -p ld-models`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/ld-models/
git commit -m "feat(models): implement ModelAdapter trait with CustomBrainAdapter and OllamaAdapter"
```

---

### Task 10: Model Router & Context Preservation

**Files:**
- Create: `crates/ld-models/src/router.rs`
- Create: `crates/ld-models/src/switching.rs`
- Modify: `crates/ld-models/src/lib.rs`
- Create: `crates/ld-models/tests/router_tests.rs`

**Step 1: Write failing test for model routing & switching**

Test routing logic: selecting Coding Brain for code requests, Design Brain for UI requests, and falling back gracefully when a brain is offline.

**Step 2: Run test to verify it fails**

Run: `cargo test -p ld-models --test router_tests`  
Expected: Compilation failure.

**Step 3: Implement `ModelRouter` and context preservation**

- Filter adapters by health, required brain, and privacy profile.
- Context switching module that maintains conversation turns in a unified format and generates switch provenance records.

**Step 4: Run test to verify it passes**

Run: `cargo test -p ld-models --test router_tests`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/ld-models/src/router.rs crates/ld-models/src/switching.rs crates/ld-models/tests/router_tests.rs
git commit -m "feat(models): implement capability-based ModelRouter with context-preserving switching"
```

---

### Task 11: Core CLI Commands & Main Entrypoint

**Files:**
- Create: `crates/ld-cli/src/commands/mod.rs`
- Create: `crates/ld-cli/src/commands/version.rs`
- Create: `crates/ld-cli/src/commands/doctor.rs`
- Create: `crates/ld-cli/src/commands/model.rs`
- Create: `crates/ld-cli/src/commands/ask.rs`
- Modify: `crates/ld-cli/src/main.rs`

**Step 1: Write CLI argument parsing and command dispatching**

Use `clap` derive parser in `main.rs` defining:
- `ldrago version`
- `ldrago doctor`
- `ldrago model list`
- `ldrago ask <prompt> [--brain <coding|design|auto>]`

**Step 2: Implement command handlers**

- `version`: outputs LDrago version, platform, Rust compiler, and neural architecture metadata.
- `doctor`: checks Python model server health (port 8741) and Ollama health (port 11434).
- `model list`: displays all 3 AI engines with their architecture and status.
- `ask`: sends prompt to the routed engine and prints response.

**Step 3: Run end-to-end command tests**

Run: `cargo run -- version`  
Expected: Clean version output with LDrago branding.  
Run: `cargo run -- doctor`  
Expected: Health checks executed with diagnostic table.

**Step 4: Commit**

```bash
git add crates/ld-cli/src/
git commit -m "feat(cli): implement version, doctor, model, and ask command CLI handlers"
```

---

## Verification Plan

### Automated Test Suite
- `cargo test --workspace` (Rust schemas, models, configs, router)
- `cd training && uv run pytest tests/ -v` (Python tokenizer, Transformer, SSM, FastAPI server)

### End-to-End Verification
1. Start model server: `cd training && uv run uvicorn src.serve.server:app --port 8741`
2. Run `cargo run -- doctor` -> verifies both Python and Ollama status
3. Run `cargo run -- model list` -> lists Coding Brain (Transformer), Design Brain (SSM), and General Runtime (Ollama)
4. Run `cargo run -- ask "Write a hello world function" --brain coding` -> responds from custom Transformer
5. Run `cargo run -- ask "Design a login flow" --brain design` -> responds from custom SSM
