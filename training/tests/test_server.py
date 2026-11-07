"""Tests for FastAPI model server."""

from fastapi.testclient import TestClient
from serve.server import app

client = TestClient(app)


def test_health_endpoint():
    resp = client.get("/health")
    assert resp.status_code == 200
    data = resp.json()
    assert data["status"] == "ok"
    assert "coding" in data["brains"]
    assert "design" in data["brains"]
    assert data["device"] == "cpu"
    assert data["models"]["coding"]["architecture"] == "transformer"
    assert data["models"]["design"]["architecture"] == "state_space_model"


def test_v1_models_endpoint():
    resp = client.get("/v1/models")
    assert resp.status_code == 200
    models = resp.json()
    assert len(models) == 2
    aliases = {m["alias"] for m in models}
    assert "coder-local" in aliases
    assert "designer-local" in aliases


def test_v1_complete_coding():
    payload = {
        "prompt": "def test():",
        "brain": "coding",
        "max_tokens": 10,
        "temperature": 0.0,
    }
    resp = client.post("/v1/complete", json=payload)
    assert resp.status_code == 200
    data = resp.json()
    assert data["brain"] == "coding"
    assert isinstance(data["text"], str)
    assert data["tokens_generated"] <= 10


def test_v1_complete_design():
    payload = {
        "prompt": "Flow: Settings",
        "brain": "design",
        "max_tokens": 10,
        "temperature": 0.0,
    }
    resp = client.post("/v1/complete", json=payload)
    assert resp.status_code == 200
    data = resp.json()
    assert data["brain"] == "design"
    assert isinstance(data["text"], str)
    assert data["tokens_generated"] <= 10


def test_v1_complete_invalid_brain():
    payload = {
        "prompt": "Hello",
        "brain": "nonexistent",
        "max_tokens": 5,
    }
    resp = client.post("/v1/complete", json=payload)
    assert resp.status_code == 400
