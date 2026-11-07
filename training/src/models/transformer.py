"""Custom Transformer (LDragoCoder) in pure PyTorch with zero external model libraries.

Components:
- RMSNorm
- Rotary Positional Embeddings (RoPE): precompute_rope_freqs and apply_rope
- GroupedQueryAttention (GQA) with causal triangular mask and optional kv_cache
- SwiGLUFFN (SiLU-gated linear projections)
- TransformerBlock
- LDragoCoder full model with weight tying
"""

from __future__ import annotations

import math
from typing import List, Optional, Tuple, Union

import torch
import torch.nn as nn
import torch.nn.functional as F

from .config import LDragoConfig


class RMSNorm(nn.Module):
    """Root Mean Square Layer Normalization."""

    def __init__(self, dim: int, eps: float = 1e-5) -> None:
        super().__init__()
        self.eps = eps
        self.weight = nn.Parameter(torch.ones(dim))

    def _norm(self, x: torch.Tensor) -> torch.Tensor:
        return x * torch.rsqrt(x.pow(2).mean(-1, keepdim=True) + self.eps)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return self._norm(x.float()).type_as(x) * self.weight


def precompute_rope_freqs(
    head_dim: int, max_seq_len: int, theta: float = 10000.0
) -> Tuple[torch.Tensor, torch.Tensor]:
    """Precompute cosine and sine frequencies for Rotary Position Embeddings (RoPE)."""
    assert head_dim % 2 == 0, f"head_dim ({head_dim}) must be even for RoPE"
    inv_freq = 1.0 / (theta ** (torch.arange(0, head_dim, 2).float() / head_dim))
    t = torch.arange(max_seq_len, dtype=torch.float32)
    freqs = torch.outer(t, inv_freq)  # (max_seq_len, head_dim // 2)
    freqs = torch.repeat_interleave(freqs, 2, dim=-1)  # (max_seq_len, head_dim)
    cos = torch.cos(freqs)
    sin = torch.sin(freqs)
    return cos, sin


def rotate_half(x: torch.Tensor) -> torch.Tensor:
    """Rotate half of the hidden dimensions for 2D Givens rotation."""
    x1 = x[..., 0::2]
    x2 = x[..., 1::2]
    return torch.stack((-x2, x1), dim=-1).flatten(-2)


def apply_rope(
    x: torch.Tensor,
    cos: torch.Tensor,
    sin: torch.Tensor,
    start_pos: int = 0,
) -> torch.Tensor:
    """Apply rotary position embedding to input tensor.

    x: (batch, seq_len, n_heads, head_dim)
    cos: (max_seq_len, head_dim)
    sin: (max_seq_len, head_dim)
    """
    seq_len = x.shape[1]
    cos_slice = cos[start_pos : start_pos + seq_len].unsqueeze(0).unsqueeze(2).to(x.device, dtype=x.dtype)
    sin_slice = sin[start_pos : start_pos + seq_len].unsqueeze(0).unsqueeze(2).to(x.device, dtype=x.dtype)
    return (x * cos_slice) + (rotate_half(x) * sin_slice)


class GroupedQueryAttention(nn.Module):
    """Grouped-Query Attention (GQA) with causal masking and optional KV cache."""

    def __init__(self, config: LDragoConfig) -> None:
        super().__init__()
        self.dim = config.dim
        self.n_heads = config.n_heads
        self.n_kv_heads = config.n_kv_heads or config.n_heads
        self.head_dim = config.head_dim
        self.num_rep = self.n_heads // self.n_kv_heads

        self.q_proj = nn.Linear(self.dim, self.n_heads * self.head_dim, bias=False)
        self.k_proj = nn.Linear(self.dim, self.n_kv_heads * self.head_dim, bias=False)
        self.v_proj = nn.Linear(self.dim, self.n_kv_heads * self.head_dim, bias=False)
        self.out_proj = nn.Linear(self.n_heads * self.head_dim, self.dim, bias=False)
        self.dropout = nn.Dropout(config.dropout)

    def forward(
        self,
        x: torch.Tensor,
        cos: torch.Tensor,
        sin: torch.Tensor,
        mask: Optional[torch.Tensor] = None,
        kv_cache: Optional[Tuple[torch.Tensor, torch.Tensor]] = None,
        start_pos: int = 0,
    ) -> Tuple[torch.Tensor, Optional[Tuple[torch.Tensor, torch.Tensor]]]:
        batch_size, seq_len, _ = x.shape

        q = self.q_proj(x).view(batch_size, seq_len, self.n_heads, self.head_dim)
        k = self.k_proj(x).view(batch_size, seq_len, self.n_kv_heads, self.head_dim)
        v = self.v_proj(x).view(batch_size, seq_len, self.n_kv_heads, self.head_dim)

        q = apply_rope(q, cos, sin, start_pos=start_pos)
        k = apply_rope(k, cos, sin, start_pos=start_pos)

        if kv_cache is not None:
            prev_k, prev_v = kv_cache
            if isinstance(prev_k, torch.Tensor) and prev_k.numel() > 0:
                k = torch.cat([prev_k, k], dim=1)
                v = torch.cat([prev_v, v], dim=1)
            new_cache = (k, v)
        else:
            new_cache = None

        total_kv_len = k.shape[1]

        # Expand KV heads to match query heads if using GQA
        if self.num_rep > 1:
            k_exp = torch.repeat_interleave(k, repeats=self.num_rep, dim=2)
            v_exp = torch.repeat_interleave(v, repeats=self.num_rep, dim=2)
        else:
            k_exp = k
            v_exp = v

        # Shape: (batch, n_heads, seq_len, head_dim)
        q = q.transpose(1, 2)
        k_exp = k_exp.transpose(1, 2)
        v_exp = v_exp.transpose(1, 2)

        # Scaled dot-product attention
        scores = torch.matmul(q, k_exp.transpose(-2, -1)) / math.sqrt(self.head_dim)

        # Apply causal mask
        if mask is not None:
            scores = scores + mask
        elif seq_len > 1 or total_kv_len > seq_len:
            # Construct causal triangular mask
            # Allow query at pos (start_pos + i) to attend to key at pos j <= start_pos + i
            q_indices = torch.arange(start_pos, start_pos + seq_len, device=x.device).unsqueeze(1)
            k_indices = torch.arange(total_kv_len, device=x.device).unsqueeze(0)
            causal_mask = torch.where(
                q_indices >= k_indices,
                torch.zeros(1, dtype=scores.dtype, device=x.device),
                torch.full((1,), float("-inf"), dtype=scores.dtype, device=x.device),
            )
            scores = scores + causal_mask.unsqueeze(0).unsqueeze(0)

        probs = F.softmax(scores, dim=-1)
        probs = self.dropout(probs)

        attn_out = torch.matmul(probs, v_exp)  # (batch, n_heads, seq_len, head_dim)
        attn_out = attn_out.transpose(1, 2).contiguous().view(batch_size, seq_len, self.dim)

        return self.out_proj(attn_out), new_cache


class SwiGLUFFN(nn.Module):
    """SwiGLU Feed-Forward Network: down_proj(silu(gate_proj(x)) * up_proj(x))."""

    def __init__(self, config: LDragoConfig) -> None:
        super().__init__()
        assert config.hidden_dim is not None
        self.gate_proj = nn.Linear(config.dim, config.hidden_dim, bias=False)
        self.up_proj = nn.Linear(config.dim, config.hidden_dim, bias=False)
        self.down_proj = nn.Linear(config.hidden_dim, config.dim, bias=False)
        self.dropout = nn.Dropout(config.dropout)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return self.dropout(self.down_proj(F.silu(self.gate_proj(x)) * self.up_proj(x)))


class TransformerBlock(nn.Module):
    """Pre-norm Transformer block with RMSNorm, GQA, and SwiGLU FFN."""

    def __init__(self, config: LDragoConfig) -> None:
        super().__init__()
        self.attn_norm = RMSNorm(config.dim, eps=config.norm_eps)
        self.attn = GroupedQueryAttention(config)
        self.ffn_norm = RMSNorm(config.dim, eps=config.norm_eps)
        self.ffn = SwiGLUFFN(config)

    def forward(
        self,
        x: torch.Tensor,
        cos: torch.Tensor,
        sin: torch.Tensor,
        mask: Optional[torch.Tensor] = None,
        kv_cache: Optional[Tuple[torch.Tensor, torch.Tensor]] = None,
        start_pos: int = 0,
    ) -> Tuple[torch.Tensor, Optional[Tuple[torch.Tensor, torch.Tensor]]]:
        h, new_cache = self.attn(
            self.attn_norm(x),
            cos,
            sin,
            mask=mask,
            kv_cache=kv_cache,
            start_pos=start_pos,
        )
        x = x + h
        x = x + self.ffn(self.ffn_norm(x))
        return x, new_cache


class LDragoCoder(nn.Module):
    """Full LDragoCoder Transformer model with weight tying and pure PyTorch implementation."""

    def __init__(self, config: LDragoConfig) -> None:
        super().__init__()
        self.config = config
        self.tok_embeddings = nn.Embedding(config.vocab_size, config.dim)
        self.layers = nn.ModuleList([TransformerBlock(config) for _ in range(config.n_layers)])
        self.norm = RMSNorm(config.dim, eps=config.norm_eps)
        self.lm_head = nn.Linear(config.dim, config.vocab_size, bias=False)

        if config.tie_weights:
            self.lm_head.weight = self.tok_embeddings.weight

        cos, sin = precompute_rope_freqs(
            head_dim=config.head_dim,
            max_seq_len=config.max_seq_len,
            theta=config.rope_theta,
        )
        self.register_buffer("rope_cos", cos, persistent=False)
        self.register_buffer("rope_sin", sin, persistent=False)

    def forward(
        self,
        input_ids: torch.Tensor,
        kv_caches: Optional[List[Optional[Tuple[torch.Tensor, torch.Tensor]]]] = None,
        start_pos: int = 0,
        return_cache: bool = False,
    ) -> Union[torch.Tensor, Tuple[torch.Tensor, List[Tuple[torch.Tensor, torch.Tensor]]]]:
        """Forward pass.

        input_ids: (batch, seq_len)
        Returns:
            logits: (batch, seq_len, vocab_size)
            optional new_kv_caches when return_cache=True
        """
        batch_size, seq_len = input_ids.shape
        x = self.tok_embeddings(input_ids)

        new_caches: List[Tuple[torch.Tensor, torch.Tensor]] = []
        for i, layer in enumerate(self.layers):
            cache_i = kv_caches[i] if kv_caches is not None else None
            x, updated_cache = layer(
                x,
                self.rope_cos,
                self.rope_sin,
                kv_cache=cache_i,
                start_pos=start_pos,
            )
            if updated_cache is not None:
                new_caches.append(updated_cache)

        x = self.norm(x)
        logits = self.lm_head(x)

        if return_cache:
            return logits, new_caches
        return logits
