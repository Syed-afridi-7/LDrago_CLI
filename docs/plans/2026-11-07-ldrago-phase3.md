# Phase 3: Scale Pretraining & Model Optimization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build scale pretraining configurations (125M parameter tier for both PyTorch brains), native INT8 weight quantization, optimized KV-cache autoregressive inference profiling, distributed multi-GPU training launch recipes (A100/H100), and a live CLI benchmarking command (`ldrago benchmark`).

**Architecture:** Parameterized architecture scaling configs (`training/src/models/configs.py`), zero-dependency native INT8 matrix quantization (`training/src/models/quantization.py`), static pre-allocated GQA `KVCache` (`training/src/models/kv_cache.py`), distributed multi-GPU DDP training harness (`training/src/train/train_distributed.py`), and Rust CLI benchmark command (`crates/ld-cli/src/commands/benchmark.rs`).

**Tech Stack:** Python 3.12, PyTorch 2.6, Torch DDP / Torchrun, Rust, Tokio, Reqwest.

---

### Task 1: 125M Parameter Scale Model Configurations

**Files:**
- Create: `training/src/models/configs.py`
- Test: `training/tests/test_configs.py`

**Step 1: Write the failing test**
Verify `get_125m_coder_config()` and `get_125m_designer_config()` produce valid parameter counts in the 100M-130M range.

**Step 2: Run test to verify it fails**
Run: `uv run pytest training/tests/test_configs.py`
Expected: ModuleNotFoundError: No module named 'src.models.configs'

**Step 3: Implement `training/src/models/configs.py`**
Provide `ModelScale` enum (`MICRO`, `BASE_125M`), scaling parameter dictionaries, and `count_parameters(model)` utility.

**Step 4: Run test to verify it passes**
Run: `uv run pytest training/tests/test_configs.py`
Expected: PASS

**Step 5: Commit**
`feat(models): implement 125M parameter scale configurations for coder and designer brains`

---

### Task 2: Native Pure PyTorch INT8 Weight Quantization

**Files:**
- Create: `training/src/models/quantization.py`
- Test: `training/tests/test_quantization.py`

**Step 1: Write the failing test**
Verify dynamic INT8 linear quantization preserves output tensor shapes, reduces parameter memory by ~75%, and maintains causal forward pass fidelity.

**Step 2: Run test to verify it fails**
Run: `uv run pytest training/tests/test_quantization.py`
Expected: ModuleNotFoundError: No module named 'src.models.quantization'

**Step 3: Implement `training/src/models/quantization.py`**
Zero third-party dependencies: `QuantizedLinear` module storing `weight_int8` (`torch.int8`) and per-channel `scale` (`torch.float32`), with `quantize_model(model)` recursive module replacer.

**Step 4: Run test to verify it passes**
Run: `uv run pytest training/tests/test_quantization.py`
Expected: PASS

**Step 5: Commit**
`feat(optim): implement zero-dependency pure PyTorch INT8 weight quantization`

---

### Task 3: Static Pre-Allocated KV-Cache Engine

**Files:**
- Create: `training/src/models/kv_cache.py`
- Modify: `training/src/models/transformer.py`
- Test: `training/tests/test_kv_cache.py`

**Step 1: Write the failing test**
Verify pre-allocated `StaticKVCache` eliminates dynamic concatenation allocations and reproduces causal attention outputs token-by-token.

**Step 2: Run test to verify it fails**
Run: `uv run pytest training/tests/test_kv_cache.py`
Expected: ModuleNotFoundError: No module named 'src.models.kv_cache'

**Step 3: Implement `StaticKVCache` in `training/src/models/kv_cache.py`**
Pre-allocated tensor buffer `(batch_size, n_kv_heads, max_seq_len, head_dim)` with $O(1)$ slice updates.

**Step 4: Run test to verify it passes**
Run: `uv run pytest training/tests/test_kv_cache.py`
Expected: PASS

**Step 5: Commit**
`feat(inference): implement static pre-allocated KV-cache engine for coding Transformer`

---

### Task 4: Distributed Training & Cloud GPU Provisioning Scripts

**Files:**
- Create: `training/src/train/train_distributed.py`
- Create: `infra/runpod/setup_a100.sh`
- Create: `infra/runpod/run_pretrain.sh`
- Test: `training/tests/test_distributed.py`

**Step 1: Write the failing test**
Verify DDP setup arguments, rank-0 checkpointing logic, and FP16 GradScaler initialization.

**Step 2: Run test to verify it fails**
Run: `uv run pytest training/tests/test_distributed.py`
Expected: ModuleNotFoundError: No module named 'src.train.train_distributed'

**Step 3: Implement `train_distributed.py` and bash provisioning scripts**
Add multi-GPU `torchrun` support with gradient scaling and RunPod A100 automated environment bootstrapping.

**Step 4: Run test to verify it passes**
Run: `uv run pytest training/tests/test_distributed.py`
Expected: PASS

**Step 5: Commit**
`feat(infra): add distributed DDP pretraining harness and cloud A100 automation scripts`

---

### Task 5: Live Benchmark CLI Command (`ldrago benchmark`)

**Files:**
- Create: `crates/ld-cli/src/commands/benchmark.rs`
- Modify: `crates/ld-cli/src/commands/mod.rs`
- Modify: `crates/ld-cli/src/lib.rs`
- Modify: `crates/ld-cli/src/main.rs`
- Test: `crates/ld-cli/tests/benchmark_tests.rs`

**Step 1: Write the failing test**
Verify `ldrago benchmark --brain coding --tokens 20` outputs throughput metrics (tokens/sec, latency, engine details).

**Step 2: Run test to verify it fails**
Run: `cargo test -p ld-cli --test benchmark_tests`
Expected: Subcommand `benchmark` not found

**Step 3: Implement `crates/ld-cli/src/commands/benchmark.rs` and CLI wiring**
Calculates tokens/second, time to first token (TTFT), and renders a formatted ANSI benchmark table.

**Step 4: Run test to verify it passes**
Run: `cargo test -p ld-cli --test benchmark_tests`
Expected: PASS

**Step 5: Commit**
`feat(cli): add live model benchmark command with throughput and latency profiling`

---

### Task 6: Full Verification, Dashboard Update & GitHub Sync

- Run all Rust tests: `cargo test --workspace -j 1`
- Run all Python tests: `uv run pytest -v`
- Test live CLI command: `cargo run --bin ldrago -- benchmark --brain coding --tokens 30`
- Update `progress-dashboard.md` artifact to 66.7% completion (Phase 3 complete)
- Git commit and push to `origin/main`
