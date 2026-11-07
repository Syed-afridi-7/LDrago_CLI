import pytest
import torch
from src.models.configs import get_coder_config, ModelScale
from src.models.kv_cache import StaticKVCache
from src.models.transformer import LDragoCoder


def test_static_kv_cache_initialization_and_update():
    cfg = get_coder_config(ModelScale.MICRO)
    batch_size = 2
    max_seq_len = 16

    cache = StaticKVCache(
        n_layers=cfg.n_layers,
        batch_size=batch_size,
        max_seq_len=max_seq_len,
        n_kv_heads=cfg.n_kv_heads,
        head_dim=cfg.head_dim,
    )

    # Initial token prefill (seq_len = 4)
    k_init = torch.randn(batch_size, 4, cfg.n_kv_heads, cfg.head_dim)
    v_init = torch.randn(batch_size, 4, cfg.n_kv_heads, cfg.head_dim)

    k_out, v_out = cache.update(layer_idx=0, start_pos=0, k=k_init, v=v_init)
    assert k_out.shape == (batch_size, 4, cfg.n_kv_heads, cfg.head_dim)
    assert torch.equal(k_out, k_init)
    assert torch.equal(v_out, v_init)

    # Next single token generation (start_pos = 4, seq_len = 1)
    k_next = torch.randn(batch_size, 1, cfg.n_kv_heads, cfg.head_dim)
    v_next = torch.randn(batch_size, 1, cfg.n_kv_heads, cfg.head_dim)

    k_out2, v_out2 = cache.update(layer_idx=0, start_pos=4, k=k_next, v=v_next)
    assert k_out2.shape == (batch_size, 5, cfg.n_kv_heads, cfg.head_dim)
    assert torch.equal(k_out2[:, :4], k_init)
    assert torch.equal(k_out2[:, 4:5], k_next)


def test_autoregressive_generation_with_static_kv_cache():
    torch.manual_seed(42)
    cfg = get_coder_config(ModelScale.MICRO)
    model = LDragoCoder(cfg)
    model.eval()

    prompt_ids = torch.tensor([[10, 20, 30]])
    batch_size = 1
    max_tokens = 5

    # 1. Non-cached sequential generation (ground truth)
    current_ids = prompt_ids.clone()
    for _ in range(max_tokens):
        with torch.no_grad():
            logits = model(current_ids)
            next_token = torch.argmax(logits[:, -1:, :], dim=-1)
            current_ids = torch.cat([current_ids, next_token], dim=1)

    expected_generated = current_ids

    # 2. Cached generation using StaticKVCache
    cache = StaticKVCache(
        n_layers=cfg.n_layers,
        batch_size=batch_size,
        max_seq_len=32,
        n_kv_heads=cfg.n_kv_heads,
        head_dim=cfg.head_dim,
    )

    cached_ids = prompt_ids.clone()
    # Prefill prompt
    with torch.no_grad():
        logits, _ = model(cached_ids, kv_cache_container=cache, start_pos=0)
        next_token = torch.argmax(logits[:, -1:, :], dim=-1)
        cached_ids = torch.cat([cached_ids, next_token], dim=1)

    # Autoregressive steps
    for step in range(1, max_tokens):
        pos = prompt_ids.shape[1] + step - 1
        with torch.no_grad():
            logits, _ = model(next_token, kv_cache_container=cache, start_pos=pos)
            next_token = torch.argmax(logits[:, -1:, :], dim=-1)
            cached_ids = torch.cat([cached_ids, next_token], dim=1)

    assert torch.equal(cached_ids, expected_generated)
