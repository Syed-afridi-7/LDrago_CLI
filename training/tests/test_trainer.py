"""Tests for CheckpointManager and LDragoTrainer."""

import json
import pytest
import torch
import torch.nn as nn
from torch.utils.data import DataLoader, TensorDataset

from models.config import LDragoConfig
from models.transformer import LDragoCoder
from train.checkpoint import CheckpointManager
from train.optimizer import build_adamw
from train.scheduler import CosineWarmupScheduler
from train.trainer import LDragoTrainer


@pytest.fixture
def micro_transformer() -> LDragoCoder:
    cfg = LDragoConfig(
        vocab_size=32,
        dim=32,
        n_layers=1,
        n_heads=2,
        n_kv_heads=2,
        hidden_dim=64,
        max_seq_len=32,
        dropout=0.0,
    )
    return LDragoCoder(cfg)


def test_checkpoint_manager_save_and_load(tmp_path, micro_transformer):
    ckpt_dir = tmp_path / "checkpoints"
    manager = CheckpointManager(ckpt_dir)

    model = micro_transformer
    optimizer = build_adamw(model, lr=1e-3)
    scheduler = CosineWarmupScheduler(optimizer, warmup_steps=5, total_steps=20)

    # Perform a dummy step
    x = torch.randint(0, 32, (2, 8))
    loss = model(x).sum()
    loss.backward()
    optimizer.step()
    scheduler.step()

    # Save checkpoint at step 10
    meta = {"experiment": "test_exp", "epoch": 1}
    saved_path = manager.save(
        step=10,
        model=model,
        optimizer=optimizer,
        scheduler=scheduler,
        loss=1.234,
        metadata=meta,
    )

    assert saved_path.exists()
    assert (ckpt_dir / "checkpoint_step_10.pt").exists()
    assert (ckpt_dir / "metadata_step_10.json").exists()
    assert (ckpt_dir / "latest.json").exists()

    # Check JSON metadata
    meta_loaded = manager.get_metadata(10)
    assert meta_loaded["step"] == 10
    assert meta_loaded["loss"] == pytest.approx(1.234)
    assert meta_loaded["metadata"]["experiment"] == "test_exp"

    # Verify list_checkpoints
    assert manager.list_checkpoints() == [10]

    # Save another at step 20
    manager.save(step=20, model=model, loss=0.5)
    assert manager.list_checkpoints() == [10, 20]

    # Test resume_latest with a new fresh model
    new_model = LDragoCoder(model.config)
    # Ensure fresh model weights are different before load
    with torch.no_grad():
        for p in new_model.parameters():
            p.add_(1.0)

    resumed_step = manager.resume_latest(new_model)
    assert resumed_step == 20

    # Verify weights match
    for p1, p2 in zip(model.parameters(), new_model.parameters()):
        assert torch.allclose(p1, p2)


def test_checkpoint_manager_resume_empty_dir(tmp_path, micro_transformer):
    manager = CheckpointManager(tmp_path / "empty_ckpts")
    resumed = manager.resume_latest(micro_transformer)
    assert resumed == 0
    assert manager.list_checkpoints() == []


def test_trainer_single_step_and_grad_accum(tmp_path, micro_transformer):
    model = micro_transformer
    optimizer = build_adamw(model, lr=5e-3)
    scheduler = CosineWarmupScheduler(optimizer, warmup_steps=2, total_steps=10)

    # Grad accum steps = 2
    trainer = LDragoTrainer(
        model=model,
        optimizer=optimizer,
        scheduler=scheduler,
        device="cpu",
        grad_accum_steps=2,
        max_grad_norm=1.0,
    )

    batch1 = torch.randint(0, 32, (2, 8))
    batch2 = torch.randint(0, 32, (2, 8))

    # Step 1: accumulated, global_step should NOT increment yet
    loss1 = trainer.train_step(batch1)
    assert isinstance(loss1, float)
    assert loss1 > 0.0
    assert trainer.global_step == 0

    # Step 2: boundary reached, global_step should increment to 1
    loss2 = trainer.train_step(batch2)
    assert isinstance(loss2, float)
    assert trainer.global_step == 1


def test_trainer_train_epoch_and_evaluate(tmp_path, micro_transformer):
    model = micro_transformer
    optimizer = build_adamw(model, lr=5e-3)
    ckpt_mgr = CheckpointManager(tmp_path / "ckpts")

    # Synthetic DataLoader
    data = torch.randint(0, 32, (8, 8))
    loader = DataLoader(TensorDataset(data), batch_size=2)
    val_loader = DataLoader(TensorDataset(data[:4]), batch_size=2)

    trainer = LDragoTrainer(
        model=model,
        train_loader=loader,
        val_loader=val_loader,
        optimizer=optimizer,
        checkpoint_manager=ckpt_mgr,
        device="cpu",
        grad_accum_steps=1,
    )

    # Evaluate before training
    initial_val_loss = trainer.evaluate()
    assert isinstance(initial_val_loss, float)
    assert initial_val_loss > 0.0

    # Train 1 epoch
    avg_loss = trainer.train_epoch(epoch=0)
    assert isinstance(avg_loss, float)
    assert avg_loss > 0.0
    assert trainer.global_step == 4  # 8 samples / batch_size 2 = 4 steps

    # Save checkpoint
    saved = trainer.save_checkpoint(loss=avg_loss)
    assert saved is not None
    assert saved.exists()

    # Resume checkpoint
    new_trainer = LDragoTrainer(
        model=LDragoCoder(model.config),
        optimizer=build_adamw(micro_transformer, lr=1e-3),
        checkpoint_manager=ckpt_mgr,
    )
    restored_step = new_trainer.resume_checkpoint()
    assert restored_step == 4
    assert new_trainer.global_step == 4
