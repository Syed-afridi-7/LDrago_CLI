"""Learning rate schedulers for LDrago training."""

from __future__ import annotations

import math
from typing import List

import torch
from torch.optim.lr_scheduler import _LRScheduler


class CosineWarmupScheduler(_LRScheduler):
    """Cosine learning rate scheduler with linear warmup.

    Applies linear warmup for `warmup_steps`, followed by cosine decay
    down to `min_lr` over `total_steps`. For steps >= total_steps, the
    learning rate remains constant at `min_lr`.

    Args:
        optimizer: Wrapped optimizer.
        warmup_steps: Number of linear warmup steps.
        total_steps: Total number of training steps for the full schedule.
        min_lr: Minimum learning rate after decay (default: 0.0).
        last_epoch: The index of last epoch (step). Default: -1.
    """

    def __init__(
        self,
        optimizer: torch.optim.Optimizer,
        warmup_steps: int,
        total_steps: int,
        min_lr: float = 0.0,
        last_epoch: int = -1,
    ) -> None:
        self.warmup_steps = max(0, int(warmup_steps))
        self.total_steps = max(1, int(total_steps))
        self.min_lr = float(min_lr)
        super().__init__(optimizer, last_epoch=last_epoch)

    def get_lr(self) -> List[float]:
        step = self.last_epoch
        if self.warmup_steps > 0 and step < self.warmup_steps:
            alpha = float(step) / float(self.warmup_steps)
            return [self.min_lr + (base_lr - self.min_lr) * alpha for base_lr in self.base_lrs]
        elif step >= self.total_steps:
            return [self.min_lr for _ in self.base_lrs]
        else:
            decay_steps = max(1, self.total_steps - self.warmup_steps)
            current_decay_step = step - self.warmup_steps
            progress = min(max(0.0, float(current_decay_step) / float(decay_steps)), 1.0)
            cosine_factor = 0.5 * (1.0 + math.cos(math.pi * progress))
            return [self.min_lr + (base_lr - self.min_lr) * cosine_factor for base_lr in self.base_lrs]
