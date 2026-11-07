"""Native dynamic INT8 weight quantization in pure PyTorch with zero external dependencies."""

from __future__ import annotations

import copy
from typing import Optional, Set

import torch
import torch.nn as nn
import torch.nn.functional as F


class QuantizedLinear(nn.Module):
    """Dynamic INT8 Quantized Linear layer.
    
    Quantizes weights to 8-bit integers per-channel, cutting memory by ~75%
    while executing dequantized linear operations with minimal loss of fidelity.
    """

    def __init__(
        self,
        in_features: int,
        out_features: int,
        bias: bool = True,
    ) -> None:
        super().__init__()
        self.in_features = in_features
        self.out_features = out_features

        # Register int8 weights and fp32 scale buffers
        self.register_buffer(
            "weight_int8",
            torch.zeros((out_features, in_features), dtype=torch.int8),
        )
        self.register_buffer(
            "scale",
            torch.ones((out_features, 1), dtype=torch.float32),
        )

        if bias:
            self.register_buffer("bias", torch.zeros(out_features, dtype=torch.float32))
        else:
            self.bias = None

    @classmethod
    def from_linear(cls, linear: nn.Linear) -> QuantizedLinear:
        """Construct a QuantizedLinear instance from an existing nn.Linear module."""
        has_bias = linear.bias is not None
        q_mod = cls(linear.in_features, linear.out_features, bias=has_bias)

        with torch.no_grad():
            w = linear.weight.detach().float()
            # Calculate per-channel max scale factor
            max_val = w.abs().amax(dim=-1, keepdim=True).clamp(min=1e-8)
            scale = max_val / 127.0
            w_int8 = (w / scale).round().clamp(-128, 127).to(torch.int8)

            q_mod.weight_int8.copy_(w_int8)
            q_mod.scale.copy_(scale)

            if has_bias and linear.bias is not None:
                q_mod.bias.copy_(linear.bias.detach().float())

        return q_mod

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """Forward pass with on-the-fly dequantization."""
        # Dequantize weight to the activation's dtype
        w_dequant = self.weight_int8.to(x.dtype) * self.scale.to(x.dtype)
        bias = self.bias.to(x.dtype) if self.bias is not None else None
        return F.linear(x, w_dequant, bias)

    def extra_repr(self) -> str:
        return f"in_features={self.in_features}, out_features={self.out_features}, bias={self.bias is not None}, dtype=torch.int8"


def quantize_model(
    model: nn.Module,
    skip_names: Optional[Set[str]] = None,
) -> nn.Module:
    """Recursively replace all candidate nn.Linear modules with QuantizedLinear.
    
    Skips modules listed in skip_names (e.g. 'lm_head' to preserve tied embeddings).
    """
    if skip_names is None:
        # Default: skip lm_head so weight tying to tok_embeddings is preserved
        skip_names = {"lm_head"}

    def _replace_linear(module: nn.Module, prefix: str = "") -> None:
        for name, child in list(module.named_children()):
            full_name = f"{prefix}.{name}" if prefix else name

            if isinstance(child, nn.Linear) and name not in skip_names and full_name not in skip_names:
                setattr(module, name, QuantizedLinear.from_linear(child))
            else:
                _replace_linear(child, full_name)

    _replace_linear(model)
    return model
