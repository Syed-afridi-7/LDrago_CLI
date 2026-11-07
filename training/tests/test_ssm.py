"""Tests for LDragoDesigner pure PyTorch State-Space Model / Mamba architecture."""

import pytest
import torch

from models.config import SSMConfig
from models.ssm import LDragoDesigner, MambaBlock, selective_scan


@pytest.fixture
def small_ssm_config() -> SSMConfig:
    return SSMConfig(
        vocab_size=128,
        d_model=64,
        n_layers=2,
        d_state=8,
        d_conv=3,
        expand=2,
        dt_rank=4,
        max_seq_len=256,
        tie_weights=True,
    )


def test_selective_scan_pure_pytorch():
    batch_size = 2
    seq_len = 8
    d_inner = 32
    d_state = 8

    u = torch.randn(batch_size, seq_len, d_inner)
    delta = torch.rand(batch_size, seq_len, d_inner) * 0.1
    A = -torch.rand(d_inner, d_state) - 0.1
    B = torch.randn(batch_size, seq_len, d_state)
    C = torch.randn(batch_size, seq_len, d_state)
    D = torch.randn(d_inner)

    y = selective_scan(u, delta, A, B, C, D)

    assert y.shape == (batch_size, seq_len, d_inner)
    assert not torch.isnan(y).any()
    assert not torch.isinf(y).any()


def test_mamba_block_forward(small_ssm_config):
    block = MambaBlock(small_ssm_config)
    batch_size = 2
    seq_len = 10
    x = torch.randn(batch_size, seq_len, small_ssm_config.d_model)

    out = block(x)
    assert out.shape == (batch_size, seq_len, small_ssm_config.d_model)
    assert not torch.isnan(out).any()


def test_ldrago_designer_forward_and_weight_tying(small_ssm_config):
    model = LDragoDesigner(small_ssm_config)
    model.eval()

    # Verify weight tying
    assert model.lm_head.weight is model.tok_embeddings.weight

    batch_size = 2
    seq_len = 12
    input_ids = torch.randint(0, small_ssm_config.vocab_size, (batch_size, seq_len))

    with torch.no_grad():
        logits = model(input_ids)

    # Verify output shape is strictly (batch, seq_len, vocab_size)
    assert logits.shape == (batch_size, seq_len, small_ssm_config.vocab_size)
    assert not torch.isnan(logits).any()


def test_ldrago_designer_causality(small_ssm_config):
    """Verify strictly causal sequence modeling: future tokens do not affect past predictions."""
    model = LDragoDesigner(small_ssm_config)
    model.eval()

    seq_len = 8
    seq_a = torch.tensor([[5, 15, 25, 35, 45, 55, 65, 75]])
    seq_b = seq_a.clone()
    # Modify future tokens (at positions 4, 5, 6, 7)
    seq_b[0, 4:] = torch.tensor([1, 2, 3, 4])

    with torch.no_grad():
        logits_a = model(seq_a)
        logits_b = model(seq_b)

    # Positions 0, 1, 2, 3 must have identical outputs between seq_a and seq_b
    assert torch.allclose(logits_a[:, :4, :], logits_b[:, :4, :], atol=1e-5), (
        "SSM Causality violation: Past predictions were influenced by future tokens!"
    )
    # Positions 4+ should differ
    assert not torch.allclose(logits_a[:, 4:, :], logits_b[:, 4:, :], atol=1e-5)
