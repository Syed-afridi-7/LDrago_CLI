"""Pure PyTorch training loop and evaluation harness for LDrago models."""

from __future__ import annotations

from pathlib import Path
from typing import Any, Dict, Iterable, Optional, Tuple, Union

import torch
import torch.nn as nn
import torch.nn.functional as F

from .checkpoint import CheckpointManager


class LDragoTrainer:
    """Trainer harness for LDragoCoder (Transformer) and LDragoDesigner (SSM) models.

    Supports:
    - Causal language modeling cross-entropy loss (logits[:, :-1] vs labels[:, 1:])
    - Multi-step gradient accumulation
    - Gradient norm clipping via torch.nn.utils.clip_grad_norm_
    - Learning rate scheduler stepping
    - Checkpoint save / resume integration
    - CPU / CUDA device portability
    """

    def __init__(
        self,
        model: nn.Module,
        train_loader: Optional[Iterable[Any]] = None,
        val_loader: Optional[Iterable[Any]] = None,
        optimizer: Optional[torch.optim.Optimizer] = None,
        scheduler: Optional[Any] = None,
        checkpoint_manager: Optional[CheckpointManager] = None,
        device: Union[str, torch.device] = "cpu",
        grad_accum_steps: int = 1,
        max_grad_norm: float = 1.0,
    ) -> None:
        self.device = torch.device(device)
        self.model = model.to(self.device)
        self.train_loader = train_loader
        self.val_loader = val_loader
        self.optimizer = optimizer
        self.scheduler = scheduler
        self.checkpoint_manager = checkpoint_manager
        self.grad_accum_steps = max(1, int(grad_accum_steps))
        self.max_grad_norm = float(max_grad_norm)

        self.global_step: int = 0
        self._accum_counter: int = 0

        # Ensure model is in train mode and zero gradients initially
        self.model.train()
        if self.optimizer is not None:
            self.optimizer.zero_grad()

    def _unpack_batch(self, batch: Any) -> Tuple[torch.Tensor, torch.Tensor]:
        """Normalize batch into input_ids and labels tensors on self.device."""
        if isinstance(batch, dict):
            input_ids = batch.get("input_ids", batch.get("tokens"))
            labels = batch.get("labels", input_ids)
        elif isinstance(batch, (tuple, list)):
            if len(batch) >= 2:
                input_ids, labels = batch[0], batch[1]
            else:
                input_ids, labels = batch[0], batch[0]
        elif isinstance(batch, torch.Tensor):
            input_ids, labels = batch, batch
        else:
            raise TypeError(f"Unsupported batch type: {type(batch)}")

        assert isinstance(input_ids, torch.Tensor), f"input_ids must be a Tensor, got {type(input_ids)}"
        assert isinstance(labels, torch.Tensor), f"labels must be a Tensor, got {type(labels)}"

        return input_ids.to(self.device), labels.to(self.device)

    def train_step(self, batch: Any) -> float:
        """Execute a single forward-backward training step.

        Computes causal LM cross-entropy loss between logits[:, :-1] and
        labels[:, 1:]. Performs gradient accumulation, gradient clipping,
        optimizer step, and scheduler step when accumulation window is reached.

        Args:
            batch: Tensor, tuple/list, or dict with input_ids and optional labels.

        Returns:
            The raw unscaled cross-entropy loss (float).
        """
        self.model.train()
        input_ids, labels = self._unpack_batch(batch)

        # Forward pass
        logits = self.model(input_ids)

        # Causal language modeling target shift: predict next token
        shift_logits = logits[:, :-1, :].contiguous()
        shift_labels = labels[:, 1:].contiguous()

        loss = F.cross_entropy(
            shift_logits.view(-1, shift_logits.size(-1)),
            shift_labels.view(-1),
        )

        # Scale loss for gradient accumulation and backprop
        loss_scaled = loss / self.grad_accum_steps
        loss_scaled.backward()
        self._accum_counter += 1

        # Perform optimization step upon reaching accumulation boundary
        if self._accum_counter % self.grad_accum_steps == 0:
            if self.max_grad_norm > 0.0:
                torch.nn.utils.clip_grad_norm_(self.model.parameters(), self.max_grad_norm)

            if self.optimizer is not None:
                self.optimizer.step()
                self.optimizer.zero_grad()

            if self.scheduler is not None:
                self.scheduler.step()

            self.global_step += 1

        return float(loss.item())

    def train_epoch(self, epoch: int = 0) -> float:
        """Train over the entire train_loader for one epoch.

        Args:
            epoch: Current epoch index.

        Returns:
            Average batch loss over the epoch.
        """
        if self.train_loader is None:
            raise ValueError("train_loader must be provided to run train_epoch")

        self.model.train()
        total_loss = 0.0
        num_batches = 0

        for batch in self.train_loader:
            batch_loss = self.train_step(batch)
            total_loss += batch_loss
            num_batches += 1

        # Flush any trailing uncommitted gradients at epoch boundary
        if self._accum_counter % self.grad_accum_steps != 0:
            if self.max_grad_norm > 0.0:
                torch.nn.utils.clip_grad_norm_(self.model.parameters(), self.max_grad_norm)
            if self.optimizer is not None:
                self.optimizer.step()
                self.optimizer.zero_grad()
            if self.scheduler is not None:
                self.scheduler.step()
            self.global_step += 1

        avg_loss = total_loss / max(1, num_batches)
        return float(avg_loss)

    def evaluate(self) -> float:
        """Evaluate the model on val_loader without gradient computation.

        Returns:
            Average cross-entropy loss over val_loader, or 0.0 if val_loader is None.
        """
        if self.val_loader is None:
            return 0.0

        self.model.eval()
        total_loss = 0.0
        num_batches = 0

        with torch.no_grad():
            for batch in self.val_loader:
                input_ids, labels = self._unpack_batch(batch)
                logits = self.model(input_ids)
                shift_logits = logits[:, :-1, :].contiguous()
                shift_labels = labels[:, 1:].contiguous()
                loss = F.cross_entropy(
                    shift_logits.view(-1, shift_logits.size(-1)),
                    shift_labels.view(-1),
                )
                total_loss += float(loss.item())
                num_batches += 1

        self.model.train()
        return float(total_loss / max(1, num_batches))

    def save_checkpoint(
        self,
        step: Optional[int] = None,
        loss: Optional[float] = None,
        metadata: Optional[Dict[str, Any]] = None,
    ) -> Optional[Path]:
        """Save training checkpoint using configured CheckpointManager."""
        if self.checkpoint_manager is None:
            return None
        save_step = self.global_step if step is None else step
        return self.checkpoint_manager.save(
            step=save_step,
            model=self.model,
            optimizer=self.optimizer,
            scheduler=self.scheduler,
            loss=loss,
            metadata=metadata,
        )

    def resume_checkpoint(self) -> int:
        """Resume model and optimizer states from latest checkpoint."""
        if self.checkpoint_manager is None:
            return 0
        restored_step = self.checkpoint_manager.resume_latest(
            model=self.model,
            optimizer=self.optimizer,
            scheduler=self.scheduler,
            map_location=str(self.device),
        )
        self.global_step = restored_step
        return restored_step
