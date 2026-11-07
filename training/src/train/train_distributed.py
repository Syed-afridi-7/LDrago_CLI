"""Multi-GPU Distributed Data Parallel (DDP) pretraining harness with FP16 mixed precision.

Run via torchrun:
    torchrun --nproc_per_node=4 -m src.train.train_distributed --brain coding --scale base_125m --fp16
"""

from __future__ import annotations

import argparse
import os
from typing import Tuple

import torch
import torch.distributed as dist
from torch.nn.parallel import DistributedDataParallel as DDP

from src.models.configs import ModelScale, get_coder_config, get_designer_config, count_parameters
from src.models.transformer import LDragoCoder
from src.models.ssm import LDragoDesigner
from src.train.optimizer import build_adamw
from src.train.scheduler import CosineWarmupScheduler
from src.train.checkpoint import CheckpointManager


def is_distributed() -> bool:
    """Return True if running in a distributed PyTorch environment."""
    return "WORLD_SIZE" in os.environ and int(os.environ["WORLD_SIZE"]) > 1


def get_device_and_rank() -> Tuple[torch.device, int, int]:
    """Detect local rank and target compute device."""
    if is_distributed():
        rank = int(os.environ["RANK"])
        local_rank = int(os.environ.get("LOCAL_RANK", 0))
        world_size = int(os.environ["WORLD_SIZE"])
        if torch.cuda.is_available():
            torch.cuda.set_device(local_rank)
            device = torch.device("cuda", local_rank)
        else:
            device = torch.device("cpu")
        return device, rank, world_size
    else:
        rank = 0
        world_size = 1
        device = torch.device("cuda:0" if torch.cuda.is_available() else "cpu")
        return device, rank, world_size


def parse_args(args=None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Distributed Pretraining for LDrago")
    parser.add_argument("--brain", type=str, default="coding", choices=["coding", "design"])
    parser.add_argument("--scale", type=str, default="base_125m", choices=["micro", "base_125m"])
    parser.add_argument("--batch-size", type=int, default=4)
    parser.add_argument("--lr", type=float, default=3e-4)
    parser.add_argument("--steps", type=int, default=1000)
    parser.add_argument("--fp16", action="store_true", help="Enable FP16 mixed precision")
    parser.add_argument("--checkpoint-dir", type=str, default="checkpoints/distributed")
    return parser.parse_args(args)


def main():
    args = parse_args()
    device, rank, world_size = get_device_and_rank()

    if is_distributed():
        backend = "nccl" if torch.cuda.is_available() else "gloo"
        dist.init_process_group(backend=backend)

    if rank == 0:
        print(f"[DDP] Initialized process group: World Size = {world_size}, Device = {device}")
        print(f"[DDP] Brain: {args.brain}, Scale: {args.scale}, FP16: {args.fp16}")

    # Build model according to scale
    if args.brain == "coding":
        cfg = get_coder_config(args.scale)
        model = LDragoCoder(cfg)
    else:
        cfg = get_designer_config(args.scale)
        model = LDragoDesigner(cfg)

    model = model.to(device)
    if rank == 0:
        total_p, trainable_p = count_parameters(model)
        print(f"[DDP] Model instantiated with {total_p:,} total parameters ({trainable_p:,} trainable)")

    if is_distributed():
        device_ids = [device.index] if device.type == "cuda" else None
        model = DDP(model, device_ids=device_ids)

    optimizer = build_adamw(model, lr=args.lr)
    scheduler = CosineWarmupScheduler(optimizer, warmup_steps=100, max_steps=args.steps)
    scaler = torch.amp.GradScaler('cuda') if (args.fp16 and device.type == "cuda") else None
    checkpoint_mgr = CheckpointManager(args.checkpoint_dir) if rank == 0 else None

    if rank == 0:
        print("[DDP] Ready for distributed pretraining batches.")

    if is_distributed():
        dist.destroy_process_group()


if __name__ == "__main__":
    main()
