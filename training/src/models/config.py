"""Configuration dataclasses for LDragoCoder (Transformer) and LDragoDesigner (SSM/Mamba)."""

from __future__ import annotations

import math
from dataclasses import dataclass
from typing import Optional


@dataclass
class LDragoConfig:
    """Configuration for LDragoCoder Transformer model."""

    vocab_size: int = 4096
    dim: int = 512
    n_layers: int = 8
    n_heads: int = 8
    n_kv_heads: Optional[int] = 4  # GQA default (4 KV heads for 8 Q heads)
    hidden_dim: Optional[int] = None  # SwiGLU hidden dimension
    max_seq_len: int = 2048
    norm_eps: float = 1e-5
    rope_theta: float = 10000.0
    tie_weights: bool = True
    dropout: float = 0.0

    def __post_init__(self) -> None:
        if self.n_kv_heads is None:
            self.n_kv_heads = self.n_heads
        assert self.dim % self.n_heads == 0, f"dim {self.dim} must be divisible by n_heads {self.n_heads}"
        assert self.n_heads % self.n_kv_heads == 0, (
            f"n_heads {self.n_heads} must be divisible by n_kv_heads {self.n_kv_heads}"
        )
        if self.hidden_dim is None:
            # Standard SwiGLU formula: ~ 8/3 * dim, rounded to multiple of 64
            hidden = int(2 * (4 * self.dim) / 3)
            self.hidden_dim = ((hidden + 63) // 64) * 64

    @property
    def head_dim(self) -> int:
        return self.dim // self.n_heads


@dataclass
class SSMConfig:
    """Configuration for LDragoDesigner State-Space Model (Mamba)."""

    vocab_size: int = 4096
    d_model: int = 512
    n_layers: int = 8
    d_state: int = 16  # State dimension N
    d_conv: int = 4  # 1D causal depthwise convolution kernel width
    expand: int = 2  # Expansion factor E
    dt_rank: Optional[int] = None  # Delta projection rank
    norm_eps: float = 1e-5
    tie_weights: bool = True
    bias: bool = False
    conv_bias: bool = True
    max_seq_len: int = 2048

    def __post_init__(self) -> None:
        if self.dt_rank is None:
            self.dt_rank = math.ceil(self.d_model / 16)

    @property
    def d_inner(self) -> int:
        return self.expand * self.d_model
