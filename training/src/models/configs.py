"""Predefined scaling profiles and parameter counter for LDrago model architectures."""

from __future__ import annotations

from enum import Enum
from typing import Tuple

import torch.nn as nn

from .config import LDragoConfig, SSMConfig


class ModelScale(str, Enum):
    MICRO = "micro"
    BASE_125M = "base_125m"
    CUSTOM = "custom"


def get_coder_config(scale: ModelScale | str) -> LDragoConfig:
    """Return parameterized Transformer configuration for LDragoCoder."""
    scale_str = scale.value if isinstance(scale, ModelScale) else str(scale).lower()

    if scale_str == "micro":
        return LDragoConfig(
            vocab_size=256,
            dim=32,
            n_layers=1,
            n_heads=4,
            n_kv_heads=2,
            max_seq_len=64,
            tie_weights=True,
        )
    elif scale_str in ("base_125m", "125m"):
        return LDragoConfig(
            vocab_size=8192,
            dim=768,
            n_layers=12,
            n_heads=12,
            n_kv_heads=4,
            max_seq_len=2048,
            tie_weights=True,
        )
    else:
        # Default standard 60M
        return LDragoConfig()


def get_designer_config(scale: ModelScale | str) -> SSMConfig:
    """Return parameterized State-Space Model configuration for LDragoDesigner."""
    scale_str = scale.value if isinstance(scale, ModelScale) else str(scale).lower()

    if scale_str == "micro":
        return SSMConfig(
            vocab_size=256,
            d_model=32,
            n_layers=1,
            d_state=16,
            max_seq_len=64,
            tie_weights=True,
        )
    elif scale_str in ("base_125m", "125m"):
        return SSMConfig(
            vocab_size=8192,
            d_model=768,
            n_layers=24,
            d_state=16,
            d_conv=4,
            expand=2,
            max_seq_len=2048,
            tie_weights=True,
        )
    else:
        # Default standard 60M
        return SSMConfig()


def count_parameters(model: nn.Module) -> Tuple[int, int]:
    """Calculate total and trainable parameters in a PyTorch module.
    
    Handles tied weights by tracking unique tensor storage pointers.
    """
    unique_params = {}
    for name, param in model.named_parameters():
        storage_ptr = param.data.data_ptr()
        if storage_ptr not in unique_params:
            unique_params[storage_ptr] = param

    total = sum(p.numel() for p in unique_params.values())
    trainable = sum(p.numel() for p in unique_params.values() if p.requires_grad)
    return total, trainable
