"""Optimizer builders and parameter grouping for LDrago training."""

from __future__ import annotations

from typing import Iterable, List, Optional, Tuple, Union

import torch
import torch.nn as nn

try:
    from models.transformer import RMSNorm
except ImportError:
    RMSNorm = None


def build_adamw(
    model: torch.nn.Module,
    lr: float = 1e-3,
    weight_decay: float = 0.01,
    betas: Tuple[float, float] = (0.9, 0.95),
    eps: float = 1e-8,
) -> torch.optim.AdamW:
    """Build AdamW optimizer with partitioned weight decay.

    Partitions model parameters so:
    - Biases, LayerNorm, RMSNorm weights, and 1D parameters have weight_decay = 0.0.
    - Matrix weights and multidimensional tensors have weight_decay = weight_decay.
    - Tied / shared parameters are deduplicated to avoid duplicate parameter errors in PyTorch.

    Args:
        model: PyTorch model whose parameters to optimize.
        lr: Learning rate (default: 1e-3).
        weight_decay: Weight decay factor for matrix parameters (default: 0.01).
        betas: Adam beta parameters (default: (0.9, 0.95)).
        eps: Epsilon term for numerical stability (default: 1e-8).

    Returns:
        Configured torch.optim.AdamW instance with partitioned parameter groups.
    """
    # Collect IDs of parameters belonging to normalization modules
    norm_param_ids = set()
    norm_classes = [nn.LayerNorm, nn.GroupNorm, nn.BatchNorm1d, nn.BatchNorm2d, nn.BatchNorm3d]
    if RMSNorm is not None:
        norm_classes.append(RMSNorm)
    norm_tuple = tuple(norm_classes)

    for m in model.modules():
        if isinstance(m, norm_tuple) or m.__class__.__name__.lower().endswith("norm"):
            for p in m.parameters(recurse=False):
                norm_param_ids.add(id(p))

    decay_params: List[nn.Parameter] = []
    no_decay_params: List[nn.Parameter] = []
    seen_ids = set()

    for name, param in model.named_parameters():
        if not param.requires_grad:
            continue
        if id(param) in seen_ids:
            continue
        seen_ids.add(id(param))

        # Criteria for zero weight decay:
        # 1. Parameter belongs to a normalization layer (LayerNorm, RMSNorm, etc.)
        # 2. Parameter is a bias (name ends with .bias or is named 'bias')
        # 3. Parameter is 1D (ndim < 2, including norm weights, 1D biases, skip scales like Mamba D)
        if id(param) in norm_param_ids or name.endswith(".bias") or name == "bias" or param.ndim < 2:
            no_decay_params.append(param)
        else:
            decay_params.append(param)

    optim_groups = []
    if decay_params:
        optim_groups.append({
            "params": decay_params,
            "weight_decay": weight_decay,
        })
    if no_decay_params:
        optim_groups.append({
            "params": no_decay_params,
            "weight_decay": 0.0,
        })

    # If both lists are empty (e.g. model with no trainable params), pass empty list
    return torch.optim.AdamW(optim_groups, lr=lr, betas=betas, eps=eps)
