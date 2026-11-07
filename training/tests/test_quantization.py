import pytest
import torch
import torch.nn as nn
from src.models.quantization import QuantizedLinear, quantize_model
from src.models.configs import get_coder_config, get_designer_config, ModelScale
from src.models.transformer import LDragoCoder
from src.models.ssm import LDragoDesigner


def test_quantized_linear_basic_forward_and_accuracy():
    torch.manual_seed(42)
    linear = nn.Linear(64, 128, bias=True)
    qlinear = QuantizedLinear.from_linear(linear)

    assert qlinear.weight_int8.dtype == torch.int8
    assert qlinear.scale.dtype == torch.float32

    # Memory comparison: int8 weight is 1/4 the size of float32
    assert qlinear.weight_int8.numel() == linear.weight.numel()
    assert qlinear.weight_int8.element_size() == 1
    assert linear.weight.element_size() == 4

    x = torch.randn(2, 10, 64)
    y_fp32 = linear(x)
    y_int8 = qlinear(x)

    assert y_int8.shape == y_fp32.shape
    # Dynamic int8 quantization should achieve high cosine similarity
    cos_sim = torch.cosine_similarity(y_fp32.flatten(), y_int8.flatten(), dim=0)
    assert cos_sim.item() > 0.99


def test_quantize_transformer_coder_model():
    torch.manual_seed(42)
    cfg = get_coder_config(ModelScale.MICRO)
    model = LDragoCoder(cfg)
    model.eval()

    input_ids = torch.randint(0, cfg.vocab_size, (1, 8))
    with torch.no_grad():
        orig_out = model(input_ids)

    q_model = quantize_model(model)
    with torch.no_grad():
        quant_out = q_model(input_ids)

    assert quant_out.shape == orig_out.shape
    cos_sim = torch.cosine_similarity(orig_out.flatten(), quant_out.flatten(), dim=0)
    assert cos_sim.item() > 0.98


def test_quantize_ssm_designer_model():
    torch.manual_seed(42)
    cfg = get_designer_config(ModelScale.MICRO)
    model = LDragoDesigner(cfg)
    model.eval()

    input_ids = torch.randint(0, cfg.vocab_size, (1, 8))
    with torch.no_grad():
        orig_out = model(input_ids)

    q_model = quantize_model(model)
    with torch.no_grad():
        quant_out = q_model(input_ids)

    assert quant_out.shape == orig_out.shape
    cos_sim = torch.cosine_similarity(orig_out.flatten(), quant_out.flatten(), dim=0)
    assert cos_sim.item() > 0.98
