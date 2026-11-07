"""Pre-allocated static Key-Value Cache for zero-allocation autoregressive decoding."""

from __future__ import annotations

from typing import Optional, Tuple

import torch


class StaticKVCache:
    """Pre-allocated tensor buffer for caching Key and Value states across all Transformer layers.
    
    Eliminates dynamic memory reallocations during token generation,
    enabling fast O(1) buffer slicing and high-throughput inference.
    """

    def __init__(
        self,
        n_layers: int,
        batch_size: int,
        max_seq_len: int,
        n_kv_heads: int,
        head_dim: int,
        device: Optional[torch.device] = None,
        dtype: torch.dtype = torch.float32,
    ) -> None:
        self.n_layers = n_layers
        self.batch_size = batch_size
        self.max_seq_len = max_seq_len
        self.n_kv_heads = n_kv_heads
        self.head_dim = head_dim
        self.device = device if device is not None else torch.device("cpu")
        self.dtype = dtype

        # Shape: (n_layers, batch_size, max_seq_len, n_kv_heads, head_dim)
        self.k = torch.zeros(
            (n_layers, batch_size, max_seq_len, n_kv_heads, head_dim),
            device=self.device,
            dtype=self.dtype,
        )
        self.v = torch.zeros(
            (n_layers, batch_size, max_seq_len, n_kv_heads, head_dim),
            device=self.device,
            dtype=self.dtype,
        )

    def update(
        self,
        layer_idx: int,
        start_pos: int,
        k: torch.Tensor,
        v: torch.Tensor,
    ) -> Tuple[torch.Tensor, torch.Tensor]:
        """Insert incoming K, V tensors into the pre-allocated cache and return the active prefix view.
        
        Args:
            layer_idx: Layer index (0 to n_layers - 1)
            start_pos: Starting sequence position for the update
            k: (batch, seq_len, n_kv_heads, head_dim)
            v: (batch, seq_len, n_kv_heads, head_dim)
        
        Returns:
            Tuple of (k_view, v_view) spanning from 0 to start_pos + seq_len.
        """
        batch_size, seq_len, _, _ = k.shape
        end_pos = start_pos + seq_len

        # In-place copy into static buffer
        self.k[layer_idx, :batch_size, start_pos:end_pos, :, :].copy_(k)
        self.v[layer_idx, :batch_size, start_pos:end_pos, :, :].copy_(v)

        # Return view of entire historical prefix up to current token
        k_view = self.k[layer_idx, :batch_size, :end_pos, :, :]
        v_view = self.v[layer_idx, :batch_size, :end_pos, :, :]

        return k_view, v_view

    def reset(self) -> None:
        """Clear cached tensors."""
        self.k.zero_()
        self.v.zero_()
