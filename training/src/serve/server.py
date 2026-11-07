"""FastAPI Model Server for LDrago custom neural architectures."""

from __future__ import annotations

import json
from typing import AsyncGenerator, Dict, List, Optional

import torch
from fastapi import FastAPI, HTTPException
from fastapi.responses import StreamingResponse
from pydantic import BaseModel, Field

from models.config import LDragoConfig, SSMConfig
from models.ssm import LDragoDesigner
from models.transformer import LDragoCoder
from tokenizer.bpe import BPETokenizer

app = FastAPI(
    title="LDrago Neural Model Server",
    description="Native inference server hosting custom Transformer (Coding) and SSM (Design) brains",
    version="0.1.0",
)

# ---------------------------------------------------------------------------
# Global State: Brain Instances & Tokenizers
# ---------------------------------------------------------------------------

device = torch.device("cpu")

# Initialize Tokenizers
coder_tokenizer = BPETokenizer()
# Train with seed code tokens
coder_seed = (
    "fn main() { println!(\"LDrago\"); } def solve(): return True\n"
    "class Model(nn.Module): pass import torch\n"
)
coder_tokenizer.train(coder_seed, target_vocab_size=300)

designer_tokenizer = BPETokenizer()
designer_seed = (
    "Flow: login entry: /auth state: loading, error, empty\n"
    "Tokens: color.primary = #0066cc, space.md = 16px\n"
)
designer_tokenizer.train(designer_seed, target_vocab_size=300)

# Initialize Micro-Brains for CPU execution
coder_config = LDragoConfig(
    vocab_size=coder_tokenizer.vocab_size,
    dim=64,
    n_layers=2,
    n_heads=4,
    n_kv_heads=2,
    hidden_dim=128,
    max_seq_len=512,
    tie_weights=True,
)
coder_model = LDragoCoder(coder_config).to(device)
coder_model.eval()

designer_config = SSMConfig(
    vocab_size=designer_tokenizer.vocab_size,
    d_model=64,
    n_layers=2,
    d_state=8,
    d_conv=4,
    expand=2,
    dt_rank=8,
    max_seq_len=512,
    tie_weights=True,
)
designer_model = LDragoDesigner(designer_config).to(device)
designer_model.eval()


# ---------------------------------------------------------------------------
# Pydantic Schemas
# ---------------------------------------------------------------------------

class CompletionRequest(BaseModel):
    prompt: str
    brain: str = Field(default="coding", description="Brain: 'coding' or 'design'")
    max_tokens: int = Field(default=32, ge=1, le=512)
    temperature: float = Field(default=0.7, ge=0.0, le=2.0)
    stream: bool = False


class CompletionResponse(BaseModel):
    text: str
    brain: str
    tokens_generated: int


class ModelCapability(BaseModel):
    alias: str
    endpoint: str
    model_name: str
    capabilities: List[str]
    context_window: int
    privacy: str
    brain: Optional[str] = None


# ---------------------------------------------------------------------------
# Endpoints
# ---------------------------------------------------------------------------

@app.get("/health")
def health() -> Dict[str, object]:
    return {
        "status": "ok",
        "brains": ["coding", "design"],
        "device": str(device),
        "models": {
            "coding": {
                "architecture": "transformer",
                "params": sum(p.numel() for p in coder_model.parameters()),
            },
            "design": {
                "architecture": "state_space_model",
                "params": sum(p.numel() for p in designer_model.parameters()),
            },
        },
    }


@app.get("/v1/models")
def list_models() -> List[ModelCapability]:
    return [
        ModelCapability(
            alias="coder-local",
            endpoint="http://127.0.0.1:8741/v1",
            model_name="ldrago-coder-custom-transformer",
            capabilities=["coding", "reasoning", "tool_calling"],
            context_window=coder_config.max_seq_len,
            privacy="local",
            brain="coding",
        ),
        ModelCapability(
            alias="designer-local",
            endpoint="http://127.0.0.1:8741/v1",
            model_name="ldrago-designer-custom-ssm",
            capabilities=["design", "reasoning"],
            context_window=designer_config.max_seq_len,
            privacy="local",
            brain="design",
        ),
    ]


def _generate_tokens(
    brain_name: str, prompt: str, max_tokens: int, temperature: float
) -> List[int]:
    """Auto-regressive generation loop."""
    if brain_name.lower() == "design":
        tok = designer_tokenizer
        model = designer_model
    else:
        tok = coder_tokenizer
        model = coder_model

    input_ids = tok.encode(prompt)
    if not input_ids:
        input_ids = [tok.bos_id]

    curr_tokens = torch.tensor([input_ids], dtype=torch.long, device=device)
    generated_ids: List[int] = []

    with torch.no_grad():
        for _ in range(max_tokens):
            seq_len = curr_tokens.shape[1]
            if seq_len > 512:
                break
            logits = model(curr_tokens)
            next_token_logits = logits[0, -1, :]

            if temperature <= 0.01:
                next_id = int(torch.argmax(next_token_logits).item())
            else:
                probs = torch.softmax(next_token_logits / temperature, dim=-1)
                next_id = int(torch.multinomial(probs, num_samples=1).item())

            if next_id == tok.eos_id:
                break

            generated_ids.append(next_id)
            next_tensor = torch.tensor([[next_id]], dtype=torch.long, device=device)
            curr_tokens = torch.cat([curr_tokens, next_tensor], dim=1)

    return generated_ids


@app.post("/v1/complete", response_model=CompletionResponse)
def complete(req: CompletionRequest) -> CompletionResponse:
    brain_key = req.brain.lower()
    if brain_key not in ("coding", "design"):
        raise HTTPException(
            status_code=400,
            detail=f"Unknown brain '{req.brain}'. Must be 'coding' or 'design'.",
        )

    tok = designer_tokenizer if brain_key == "design" else coder_tokenizer
    output_tokens = _generate_tokens(
        brain_name=brain_key,
        prompt=req.prompt,
        max_tokens=req.max_tokens,
        temperature=req.temperature,
    )
    generated_text = tok.decode(output_tokens)

    return CompletionResponse(
        text=generated_text,
        brain=brain_key,
        tokens_generated=len(output_tokens),
    )


@app.post("/v1/complete/stream")
async def complete_stream(req: CompletionRequest) -> StreamingResponse:
    brain_key = req.brain.lower()
    if brain_key not in ("coding", "design"):
        raise HTTPException(
            status_code=400,
            detail=f"Unknown brain '{req.brain}'. Must be 'coding' or 'design'.",
        )

    tok = designer_tokenizer if brain_key == "design" else coder_tokenizer

    async def event_generator() -> AsyncGenerator[str, None]:
        tokens = _generate_tokens(
            brain_name=brain_key,
            prompt=req.prompt,
            max_tokens=req.max_tokens,
            temperature=req.temperature,
        )
        for token_id in tokens:
            chunk = tok.decode([token_id])
            data = json.dumps({"text": chunk, "brain": brain_key, "done": False})
            yield f"data: {data}\n\n"
        data_done = json.dumps({"text": "", "brain": brain_key, "done": True})
        yield f"data: {data_done}\n\n"

    return StreamingResponse(event_generator(), media_type="text/event-stream")
