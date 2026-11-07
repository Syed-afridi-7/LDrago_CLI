"""End-to-end CPU micro-model training and convergence verification."""

from __future__ import annotations

from typing import List, Literal

import torch

from models.config import LDragoConfig, SSMConfig
from models.ssm import LDragoDesigner
from models.transformer import LDragoCoder
from .optimizer import build_adamw
from .trainer import LDragoTrainer


def run_micro_training(
    architecture: Literal["transformer", "ssm"] = "transformer",
    steps: int = 15,
    batch_size: int = 2,
    seed: int = 42,
) -> List[float]:
    """Train a micro-model on CPU over synthetic data and verify monotonic loss decrease.

    Args:
        architecture: Either "transformer" (LDragoCoder) or "ssm" (LDragoDesigner).
        steps: Total number of optimization steps to execute (default: 15).
        batch_size: Batch size of synthetic sequence (default: 2).
        seed: Random seed for deterministic reproducibility (default: 42).

    Returns:
        loss_history: List of loss floats across all steps.

    Raises:
        ValueError: If architecture is not 'transformer' or 'ssm'.
        AssertionError: If loss does not decrease monotonically from step 0 to step (steps - 1).
    """
    arch = architecture.lower().strip()
    if arch not in ("transformer", "ssm"):
        raise ValueError(f"Unsupported architecture '{architecture}'. Must be 'transformer' or 'ssm'.")

    # Set seed for deterministic synthetic batch and initialization
    torch.manual_seed(seed)

    vocab_size = 32
    seq_len = 8

    # 1. Build small micro-model
    if arch == "transformer":
        cfg = LDragoConfig(
            vocab_size=vocab_size,
            dim=32,
            n_layers=1,
            n_heads=2,
            n_kv_heads=2,
            hidden_dim=64,
            max_seq_len=32,
            dropout=0.0,
            tie_weights=True,
        )
        model = LDragoCoder(cfg)
    else:
        cfg_ssm = SSMConfig(
            vocab_size=vocab_size,
            d_model=32,
            n_layers=1,
            d_state=8,
            d_conv=4,
            expand=2,
            dt_rank=4,
            max_seq_len=32,
            tie_weights=True,
        )
        model = LDragoDesigner(cfg_ssm)

    # 2. Build AdamW optimizer with zero weight decay on 1D/norm params
    optimizer = build_adamw(model, lr=0.005, weight_decay=0.01)

    # 3. Initialize Trainer on CPU
    trainer = LDragoTrainer(
        model=model,
        optimizer=optimizer,
        scheduler=None,
        device="cpu",
        grad_accum_steps=1,
        max_grad_norm=1.0,
    )

    # 4. Generate small fixed synthetic token batch
    synthetic_batch = torch.randint(1, vocab_size, (batch_size, seq_len))

    # 5. Run training steps and collect loss history
    loss_history: List[float] = []
    for step_idx in range(steps):
        loss = trainer.train_step(synthetic_batch)
        loss_history.append(loss)

    # 6. Verify strictly monotonic loss decrease from step 0 to step (steps - 1)
    for i in range(len(loss_history) - 1):
        assert loss_history[i] > loss_history[i + 1], (
            f"[{arch}] Loss failed monotonic decrease at step {i+1}: "
            f"step {i}={loss_history[i]:.5f} -> step {i+1}={loss_history[i+1]:.5f}"
        )

    return loss_history


if __name__ == "__main__":
    print("Running Transformer micro-training...")
    t_losses = run_micro_training("transformer", steps=15, batch_size=2)
    print("Transformer losses:", [round(x, 4) for x in t_losses])

    print("Running SSM micro-training...")
    s_losses = run_micro_training("ssm", steps=15, batch_size=2)
    print("SSM losses:", [round(x, 4) for x in s_losses])
