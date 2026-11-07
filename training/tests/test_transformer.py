"""Tests for LDragoCoder pure PyTorch Transformer architecture."""

import pytest
import torch

from models.config import LDragoConfig
from models.transformer import (
    GroupedQueryAttention,
    LDragoCoder,
    RMSNorm,
    SwiGLUFFN,
    apply_rope,
    precompute_rope_freqs,
)


@pytest.fixture
def small_config() -> LDragoConfig:
    return LDragoConfig(
        vocab_size=128,
        dim=64,
        n_layers=2,
        n_heads=4,
        n_kv_heads=2,  # GQA: 4 Q heads, 2 KV heads
        hidden_dim=128,
        max_seq_len=256,
        tie_weights=True,
    )


def test_rmsnorm():
    dim = 32
    norm = RMSNorm(dim=dim, eps=1e-5)
    x = torch.randn(2, 8, dim) * 5.0 + 3.0
    out = norm(x)
    assert out.shape == x.shape
    # Check that root mean square along last dimension is close to 1
    rms = torch.sqrt(torch.mean(out.pow(2), dim=-1))
    assert torch.allclose(rms, torch.ones_like(rms), atol=1e-2)


def test_rope_precompute_and_apply():
    head_dim = 16
    max_seq_len = 64
    cos, sin = precompute_rope_freqs(head_dim=head_dim, max_seq_len=max_seq_len)
    assert cos.shape == (max_seq_len, head_dim)
    assert sin.shape == (max_seq_len, head_dim)

    # Apply RoPE to tensor of shape (batch, seq_len, n_heads, head_dim)
    x = torch.randn(2, 10, 4, head_dim)
    x_rot = apply_rope(x, cos, sin)
    assert x_rot.shape == x.shape
    assert not torch.isnan(x_rot).any()
    assert not torch.isinf(x_rot).any()


def test_swiglu_ffn(small_config):
    ffn = SwiGLUFFN(small_config)
    x = torch.randn(2, 8, small_config.dim)
    out = ffn(x)
    assert out.shape == x.shape


def test_gqa_forward_and_cache(small_config):
    attn = GroupedQueryAttention(small_config)
    cos, sin = precompute_rope_freqs(
        head_dim=small_config.head_dim,
        max_seq_len=small_config.max_seq_len,
    )

    batch_size = 2
    seq_len = 8
    x = torch.randn(batch_size, seq_len, small_config.dim)

    # Forward without cache
    out, cache = attn(x, cos, sin)
    assert out.shape == (batch_size, seq_len, small_config.dim)
    assert cache is None

    # Forward with cache
    out1, cache1 = attn(x, cos, sin, kv_cache=None)
    # Simulate single-token step using cache:
    x_step = torch.randn(batch_size, 1, small_config.dim)
    # Start cache with step 1
    _, cache_init = attn(x, cos, sin, kv_cache=([], []))
    assert cache_init is not None
    k_cached, v_cached = cache_init
    assert k_cached.shape == (batch_size, seq_len, small_config.n_kv_heads, small_config.head_dim)


def test_ldrago_coder_forward_shape_and_tying(small_config):
    model = LDragoCoder(small_config)
    model.eval()

    # Verify weight tying
    assert model.lm_head.weight is model.tok_embeddings.weight

    batch_size = 2
    seq_len = 16
    input_ids = torch.randint(0, small_config.vocab_size, (batch_size, seq_len))

    with torch.no_grad():
        logits = model(input_ids)

    # Verify output shape is strictly (batch, seq_len, vocab_size)
    assert logits.shape == (batch_size, seq_len, small_config.vocab_size)
    assert not torch.isnan(logits).any()


def test_ldrago_coder_causal_attention(small_config):
    """Verify strictly causal attention: changes to future tokens do not affect past token logits."""
    model = LDragoCoder(small_config)
    model.eval()

    seq_len = 8
    seq_a = torch.tensor([[10, 20, 30, 40, 50, 60, 70, 80]])
    seq_b = seq_a.clone()
    # Modify future tokens (at positions 4, 5, 6, 7)
    seq_b[0, 4:] = torch.tensor([1, 2, 3, 4])

    with torch.no_grad():
        logits_a = model(seq_a)
        logits_b = model(seq_b)

    # Positions 0, 1, 2, 3 must have identical logits between seq_a and seq_b
    assert torch.allclose(logits_a[:, :4, :], logits_b[:, :4, :], atol=1e-5), (
        "Causal masking violation: Past tokens were affected by changes in future tokens!"
    )
    # Position 4 onwards should differ
    assert not torch.allclose(logits_a[:, 4:, :], logits_b[:, 4:, :], atol=1e-5)
