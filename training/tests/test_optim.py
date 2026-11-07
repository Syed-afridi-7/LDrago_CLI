"""Tests for AdamW optimizer and CosineWarmupScheduler."""

import math
import pytest
import torch
import torch.nn as nn

from models.config import LDragoConfig, SSMConfig
from models.ssm import LDragoDesigner
from models.transformer import LDragoCoder, RMSNorm
from train.optimizer import build_adamw
from train.scheduler import CosineWarmupScheduler


class ToyHybridModel(nn.Module):
    """Toy model containing Linear, LayerNorm, RMSNorm, and custom biases."""

    def __init__(self, in_features: int = 16, out_features: int = 16) -> None:
        super().__init__()
        self.linear = nn.Linear(in_features, out_features, bias=True)
        self.layernorm = nn.LayerNorm(out_features)
        self.rmsnorm = RMSNorm(out_features)
        self.skip_scale = nn.Parameter(torch.ones(out_features))

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        x = self.linear(x)
        x = self.layernorm(x)
        x = self.rmsnorm(x)
        return x * self.skip_scale


def test_build_adamw_weight_decay_separation():
    model = ToyHybridModel(16, 16)
    lr = 2e-3
    weight_decay = 0.05
    betas = (0.9, 0.98)

    optimizer = build_adamw(model, lr=lr, weight_decay=weight_decay, betas=betas)

    assert len(optimizer.param_groups) == 2
    decay_group = optimizer.param_groups[0]
    no_decay_group = optimizer.param_groups[1]

    assert decay_group["weight_decay"] == pytest.approx(weight_decay)
    assert no_decay_group["weight_decay"] == 0.0
    assert decay_group["lr"] == lr
    assert decay_group["betas"] == betas

    # Linear weight (2D) must be in decay group
    decay_params = set(decay_group["params"])
    assert model.linear.weight in decay_params

    # Biases, LayerNorm, RMSNorm, and 1D params must be in no-decay group
    no_decay_params = set(no_decay_group["params"])
    assert model.linear.bias in no_decay_params
    assert model.layernorm.weight in no_decay_params
    assert model.layernorm.bias in no_decay_params
    assert model.rmsnorm.weight in no_decay_params
    assert model.skip_scale in no_decay_params

    # All model trainable parameters must be accounted for and disjoint
    all_model_params = set(model.parameters())
    assert decay_params.isdisjoint(no_decay_params)
    assert decay_params.union(no_decay_params) == all_model_params


def test_build_adamw_with_transformer_and_tied_weights():
    cfg = LDragoConfig(
        vocab_size=64,
        dim=32,
        n_layers=2,
        n_heads=2,
        n_kv_heads=2,
        hidden_dim=64,
        max_seq_len=32,
        tie_weights=True,
    )
    model = LDragoCoder(cfg)
    optimizer = build_adamw(model, lr=1e-3, weight_decay=0.01)

    decay_group = optimizer.param_groups[0]
    no_decay_group = optimizer.param_groups[1]

    # Verify no duplicate parameter entries despite weight tying
    all_opt_params = decay_group["params"] + no_decay_group["params"]
    assert len(all_opt_params) == len(set(all_opt_params))

    # tok_embeddings weight is 2D matrix -> decay
    assert model.tok_embeddings.weight in set(decay_group["params"])
    # final norm weight is RMSNorm -> no decay
    assert model.norm.weight in set(no_decay_group["params"])


def test_build_adamw_with_ssm():
    cfg = SSMConfig(
        vocab_size=64,
        d_model=32,
        n_layers=1,
        d_state=8,
        d_conv=4,
        expand=2,
        dt_rank=4,
        max_seq_len=32,
    )
    model = LDragoDesigner(cfg)
    optimizer = build_adamw(model, lr=1e-3, weight_decay=0.01)

    decay_group = optimizer.param_groups[0]
    no_decay_group = optimizer.param_groups[1]

    decay_params = set(decay_group["params"])
    no_decay_params = set(no_decay_group["params"])

    # Matrix parameters
    assert model.tok_embeddings.weight in decay_params
    mamba = model.layers[0]["mamba"]
    assert mamba.in_proj.weight in decay_params
    assert mamba.out_proj.weight in decay_params
    assert mamba.conv1d.weight in decay_params

    # Biases and 1D parameters
    assert mamba.conv1d.bias in no_decay_params
    assert mamba.dt_proj.bias in no_decay_params
    assert mamba.D in no_decay_params
    assert model.norm.weight in no_decay_params


def test_cosine_warmup_scheduler_linear_warmup():
    param = nn.Parameter(torch.zeros(1))
    base_lr = 1e-2
    warmup_steps = 10
    total_steps = 100
    min_lr = 0.0

    optimizer = torch.optim.SGD([param], lr=base_lr)
    scheduler = CosineWarmupScheduler(
        optimizer,
        warmup_steps=warmup_steps,
        total_steps=total_steps,
        min_lr=min_lr,
    )

    # Initial state (step 0): lr should be min_lr
    assert optimizer.param_groups[0]["lr"] == pytest.approx(0.0)

    # Step through warmup
    for s in range(1, warmup_steps + 1):
        optimizer.step()
        scheduler.step()
        expected_lr = base_lr * (s / warmup_steps)
        assert optimizer.param_groups[0]["lr"] == pytest.approx(expected_lr, rel=1e-5)


def test_cosine_warmup_scheduler_decay_and_min_lr():
    param = nn.Parameter(torch.zeros(1))
    base_lr = 1.0
    warmup_steps = 10
    total_steps = 100
    min_lr = 0.1

    optimizer = torch.optim.SGD([param], lr=base_lr)
    scheduler = CosineWarmupScheduler(
        optimizer,
        warmup_steps=warmup_steps,
        total_steps=total_steps,
        min_lr=min_lr,
    )

    # Fast forward through warmup to step 10
    for _ in range(warmup_steps):
        optimizer.step()
        scheduler.step()
    assert optimizer.param_groups[0]["lr"] == pytest.approx(base_lr, rel=1e-5)

    # Step to midpoint of decay (step 55: progress = (55 - 10) / (100 - 10) = 0.5)
    for _ in range(45):
        optimizer.step()
        scheduler.step()
    expected_mid = min_lr + (base_lr - min_lr) * 0.5
    assert optimizer.param_groups[0]["lr"] == pytest.approx(expected_mid, rel=1e-4)

    # Step to total_steps (step 100)
    for _ in range(45):
        optimizer.step()
        scheduler.step()
    assert optimizer.param_groups[0]["lr"] == pytest.approx(min_lr, rel=1e-4)

    # Step beyond total_steps (step 105)
    for _ in range(5):
        optimizer.step()
        scheduler.step()
    assert optimizer.param_groups[0]["lr"] == pytest.approx(min_lr, rel=1e-4)


def test_cosine_warmup_scheduler_zero_warmup():
    param = nn.Parameter(torch.zeros(1))
    base_lr = 0.05
    warmup_steps = 0
    total_steps = 20
    min_lr = 0.005

    optimizer = torch.optim.SGD([param], lr=base_lr)
    scheduler = CosineWarmupScheduler(
        optimizer,
        warmup_steps=warmup_steps,
        total_steps=total_steps,
        min_lr=min_lr,
    )

    # Step 0 starts directly at base_lr
    assert optimizer.param_groups[0]["lr"] == pytest.approx(base_lr)

    # Step to end
    for _ in range(total_steps):
        optimizer.step()
        scheduler.step()
    assert optimizer.param_groups[0]["lr"] == pytest.approx(min_lr, rel=1e-4)
