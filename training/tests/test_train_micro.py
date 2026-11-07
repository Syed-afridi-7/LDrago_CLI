"""Tests for CPU micro-model training and monotonic convergence."""

import pytest

from train.train_micro import run_micro_training


def test_micro_training_transformer_convergence():
    steps = 15
    losses = run_micro_training(architecture="transformer", steps=steps, batch_size=2, seed=42)

    assert len(losses) == steps
    # Verify strict monotonic decrease across every step
    for i in range(len(losses) - 1):
        assert losses[i] > losses[i + 1], f"Step {i} ({losses[i]}) not > Step {i+1} ({losses[i+1]})"

    # Verify significant loss reduction (final loss < 15% of initial loss)
    assert losses[-1] < losses[0] * 0.15


def test_micro_training_ssm_convergence():
    steps = 15
    losses = run_micro_training(architecture="ssm", steps=steps, batch_size=2, seed=42)

    assert len(losses) == steps
    # Verify strict monotonic decrease across every step
    for i in range(len(losses) - 1):
        assert losses[i] > losses[i + 1], f"Step {i} ({losses[i]}) not > Step {i+1} ({losses[i+1]})"

    # Verify significant loss reduction (final loss < 15% of initial loss)
    assert losses[-1] < losses[0] * 0.15


def test_micro_training_invalid_arch():
    with pytest.raises(ValueError, match="Unsupported architecture"):
        run_micro_training(architecture="rnn")  # type: ignore


def test_micro_training_reproducibility():
    losses1 = run_micro_training(architecture="transformer", steps=5, seed=123)
    losses2 = run_micro_training(architecture="transformer", steps=5, seed=123)

    assert losses1 == pytest.approx(losses2)
