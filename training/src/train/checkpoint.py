"""Atomic checkpoint manager for LDrago model weights and training states."""

from __future__ import annotations

import json
import os
import re
from pathlib import Path
from typing import Any, Dict, List, Optional, Union

import torch


class CheckpointManager:
    """Manages saving, loading, and resuming training checkpoints atomically.

    Each checkpoint consists of:
    - PyTorch state dictionary (`checkpoint_step_{step}.pt`) containing model,
      optimizer, scheduler, and loss states.
    - JSON metadata (`metadata_step_{step}.json`) for inspection without
      requiring PyTorch or unpickling.
    """

    def __init__(self, checkpoint_dir: Union[str, Path]) -> None:
        self.checkpoint_dir = Path(checkpoint_dir)
        self.checkpoint_dir.mkdir(parents=True, exist_ok=True)

    def _get_ckpt_path(self, step: int) -> Path:
        return self.checkpoint_dir / f"checkpoint_step_{step}.pt"

    def _get_meta_path(self, step: int) -> Path:
        return self.checkpoint_dir / f"metadata_step_{step}.json"

    def save(
        self,
        step: int,
        model: torch.nn.Module,
        optimizer: Optional[torch.optim.Optimizer] = None,
        scheduler: Optional[Any] = None,
        loss: Optional[float] = None,
        metadata: Optional[Dict[str, Any]] = None,
    ) -> Path:
        """Save a training checkpoint atomically with JSON metadata.

        Args:
            step: Global training step count.
            model: Model whose state dict will be saved.
            optimizer: Optional optimizer whose state dict will be saved.
            scheduler: Optional LR scheduler whose state dict will be saved.
            loss: Optional current loss value.
            metadata: Optional dictionary of additional metadata.

        Returns:
            Path to the saved checkpoint file.
        """
        ckpt_path = self._get_ckpt_path(step)
        meta_path = self._get_meta_path(step)

        tmp_ckpt = self.checkpoint_dir / f".tmp_{ckpt_path.name}"
        tmp_meta = self.checkpoint_dir / f".tmp_{meta_path.name}"

        payload: Dict[str, Any] = {
            "step": step,
            "model_state_dict": model.state_dict(),
            "optimizer_state_dict": optimizer.state_dict() if optimizer is not None else None,
            "scheduler_state_dict": scheduler.state_dict() if scheduler is not None else None,
            "loss": float(loss) if loss is not None else None,
            "metadata": metadata or {},
        }

        # 1. Save PyTorch state dict to temporary file, then atomic rename
        torch.save(payload, tmp_ckpt)
        os.replace(tmp_ckpt, ckpt_path)

        # 2. Save JSON metadata to temporary file, then atomic rename
        meta_payload: Dict[str, Any] = {
            "step": step,
            "loss": float(loss) if loss is not None else None,
            "metadata": metadata or {},
            "checkpoint_file": ckpt_path.name,
        }
        with open(tmp_meta, "w", encoding="utf-8") as f:
            json.dump(meta_payload, f, indent=2)
        os.replace(tmp_meta, meta_path)

        # 3. Update pointer file latest.json atomically
        tmp_latest = self.checkpoint_dir / ".tmp_latest.json"
        latest_path = self.checkpoint_dir / "latest.json"
        with open(tmp_latest, "w", encoding="utf-8") as f:
            json.dump({"latest_step": step, "checkpoint_file": ckpt_path.name}, f, indent=2)
        os.replace(tmp_latest, latest_path)

        return ckpt_path

    def list_checkpoints(self) -> List[int]:
        """List all available checkpoint step numbers in ascending order."""
        steps: List[int] = []
        pattern = re.compile(r"^checkpoint_(?:step_)?(\d+)\.pt$")
        if self.checkpoint_dir.exists():
            for entry in self.checkpoint_dir.iterdir():
                match = pattern.match(entry.name)
                if match:
                    steps.append(int(match.group(1)))
        return sorted(steps)

    def load(
        self,
        step: int,
        model: Optional[torch.nn.Module] = None,
        optimizer: Optional[torch.optim.Optimizer] = None,
        scheduler: Optional[Any] = None,
        map_location: str = "cpu",
    ) -> Dict[str, Any]:
        """Load a checkpoint by step number.

        If model, optimizer, or scheduler are provided, their state dicts
        are updated in-place.

        Args:
            step: Step number to load.
            model: Optional model to restore.
            optimizer: Optional optimizer to restore.
            scheduler: Optional scheduler to restore.
            map_location: Device mapping for torch.load (default: 'cpu').

        Returns:
            The loaded checkpoint dictionary.
        """
        ckpt_path = self._get_ckpt_path(step)
        if not ckpt_path.exists():
            # Fallback to alternate naming convention
            alt_path = self.checkpoint_dir / f"checkpoint_{step}.pt"
            if alt_path.exists():
                ckpt_path = alt_path
            else:
                raise FileNotFoundError(f"Checkpoint for step {step} not found in {self.checkpoint_dir}")

        ckpt = torch.load(ckpt_path, map_location=map_location, weights_only=False)

        if model is not None and "model_state_dict" in ckpt:
            model.load_state_dict(ckpt["model_state_dict"])
        if optimizer is not None and ckpt.get("optimizer_state_dict") is not None:
            optimizer.load_state_dict(ckpt["optimizer_state_dict"])
        if scheduler is not None and ckpt.get("scheduler_state_dict") is not None:
            scheduler.load_state_dict(ckpt["scheduler_state_dict"])

        return ckpt

    def get_metadata(self, step: int) -> Dict[str, Any]:
        """Load JSON metadata dictionary for a step without loading PyTorch weights."""
        meta_path = self._get_meta_path(step)
        if not meta_path.exists():
            alt_path = self.checkpoint_dir / f"metadata_{step}.json"
            if alt_path.exists():
                meta_path = alt_path
            else:
                raise FileNotFoundError(f"Metadata for step {step} not found in {self.checkpoint_dir}")

        with open(meta_path, "r", encoding="utf-8") as f:
            return json.load(f)

    def resume_latest(
        self,
        model: torch.nn.Module,
        optimizer: Optional[torch.optim.Optimizer] = None,
        scheduler: Optional[Any] = None,
        map_location: str = "cpu",
    ) -> int:
        """Find the latest checkpoint and restore model/optimizer/scheduler states.

        Args:
            model: Model to restore.
            optimizer: Optional optimizer to restore.
            scheduler: Optional scheduler to restore.
            map_location: Device mapping (default: 'cpu').

        Returns:
            The restored step number, or 0 if no checkpoints exist.
        """
        steps = self.list_checkpoints()
        if not steps:
            return 0

        latest_step = steps[-1]
        self.load(
            latest_step,
            model=model,
            optimizer=optimizer,
            scheduler=scheduler,
            map_location=map_location,
        )
        return latest_step
