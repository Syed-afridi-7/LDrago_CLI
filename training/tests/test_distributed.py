import pytest
import torch
from src.train.train_distributed import parse_args, is_distributed, get_device_and_rank


def test_distributed_environment_detection_standalone():
    # In normal single-process test runs, distributed should be false
    assert not is_distributed()

    device, rank, world_size = get_device_and_rank()
    assert rank == 0
    assert world_size == 1
    assert isinstance(device, torch.device)


def test_distributed_arg_parser_defaults():
    args = parse_args(["--brain", "coding", "--scale", "base_125m"])
    assert args.brain == "coding"
    assert args.scale == "base_125m"
    assert args.fp16 is False
    assert args.batch_size == 4
