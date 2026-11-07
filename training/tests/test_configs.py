import pytest
import torch
from src.models.configs import (
    ModelScale,
    get_coder_config,
    get_designer_config,
    count_parameters,
)
from src.models.transformer import LDragoCoder
from src.models.ssm import LDragoDesigner


def test_micro_configs():
    micro_coder = get_coder_config(ModelScale.MICRO)
    assert micro_coder.dim == 32
    assert micro_coder.n_layers == 1

    micro_designer = get_designer_config(ModelScale.MICRO)
    assert micro_designer.dim == 32
    assert micro_designer.n_layers == 1


def test_125m_coder_config_and_parameter_count():
    config = get_coder_config(ModelScale.BASE_125M)
    assert config.dim == 768
    assert config.n_layers == 12
    assert config.n_heads == 12
    assert config.n_kv_heads == 4
    assert config.vocab_size == 8192

    # Instantiate model and verify parameter count is ~85M to 130M
    model = LDragoCoder(config)
    total_params, trainable_params = count_parameters(model)
    assert 80_000_000 <= total_params <= 135_000_000
    assert total_params == trainable_params


def test_125m_designer_config_and_parameter_count():
    config = get_designer_config(ModelScale.BASE_125M)
    assert config.dim == 768
    assert config.n_layers == 24
    assert config.d_state == 16
    assert config.vocab_size == 8192

    model = LDragoDesigner(config)
    total_params, trainable_params = count_parameters(model)
    assert 80_000_000 <= total_params <= 135_000_000
    assert total_params == trainable_params
