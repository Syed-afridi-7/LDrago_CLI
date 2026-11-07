"""Unit tests for data sanitization, curation, filtering, and PyTorch CausalTextDataset."""

import os
import tempfile
import torch
from torch.utils.data import DataLoader

from data.filters import deduplicate_texts, is_valid_license, redact_secrets
from data.dataset import CausalTextDataset
from data.prepare import (
    generate_seed_coding_corpus,
    generate_seed_design_corpus,
    save_corpus_to_file,
)
from tokenizer.bpe import BPETokenizer


# ---------------------------------------------------------------------------
# Filters: Secret Redaction Tests
# ---------------------------------------------------------------------------

def test_redact_secrets_openai_key():
    raw = "Client created with api_key='sk-abc123def456ghi789jkl0123456789'."
    redacted = redact_secrets(raw)
    assert "[REDACTED_SECRET]" in redacted
    assert "sk-abc123def456ghi789jkl0123456789" not in redacted


def test_redact_secrets_github_token():
    raw = "git remote set-url origin https://ghp_1234567890abcdefghijklmnopqrstuvwxyz@github.com"
    redacted = redact_secrets(raw)
    assert "[REDACTED_SECRET]" in redacted
    assert "ghp_1234567890abcdefghijklmnopqrstuvwxyz" not in redacted


def test_redact_secrets_aws_key():
    raw = "export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE\nexport AWS_REGION=us-west-2"
    redacted = redact_secrets(raw)
    assert "[REDACTED_SECRET]" in redacted
    assert "AKIAIOSFODNN7EXAMPLE" not in redacted
    assert "export AWS_REGION=us-west-2" in redacted


def test_redact_secrets_generic_hex():
    hex_secret = "4f8b9c2d1e0a3b4c5d6e7f8a9b0c1d2e"  # 32 chars hex
    raw = f"session_secret = '{hex_secret}'"
    redacted = redact_secrets(raw)
    assert "[REDACTED_SECRET]" in redacted
    assert hex_secret not in redacted


def test_redact_secrets_private_key():
    raw = (
        "-----BEGIN RSA PRIVATE KEY-----\n"
        "MIIEowIBAAKCAQEA0Y1o...\n"
        "-----END RSA PRIVATE KEY-----\n"
        "print('key loaded')"
    )
    redacted = redact_secrets(raw)
    assert "[REDACTED_SECRET]" in redacted
    assert "MIIEowIBAAKCAQEA0Y1o..." not in redacted
    assert "print('key loaded')" in redacted


def test_redact_secrets_passwords():
    raw1 = 'database_password = "superSecretPassword123!"'
    raw2 = "db_pwd: myDatabasePassword"
    assert "[REDACTED_SECRET]" in redact_secrets(raw1)
    assert "superSecretPassword123!" not in redact_secrets(raw1)
    assert "[REDACTED_SECRET]" in redact_secrets(raw2)
    assert "myDatabasePassword" not in redact_secrets(raw2)


def test_redact_secrets_preserves_clean_code():
    code = "def add(a: int, b: int) -> int:\n    return a + b\n"
    assert redact_secrets(code) == code


# ---------------------------------------------------------------------------
# Filters: License Validation Tests
# ---------------------------------------------------------------------------

def test_is_valid_license_mit():
    assert is_valid_license("MIT")
    assert is_valid_license("// SPDX-License-Identifier: MIT")
    assert is_valid_license("Permission is hereby granted, free of charge, to any person obtaining a copy...")
    assert is_valid_license("/* Licensed under the MIT License */")


def test_is_valid_license_apache():
    assert is_valid_license("Apache-2.0")
    assert is_valid_license("Apache 2.0")
    assert is_valid_license("# SPDX-License-Identifier: Apache-2.0")
    assert is_valid_license("Licensed under the Apache License, Version 2.0 (the 'License');")


def test_is_valid_license_bsd():
    assert is_valid_license("BSD-3-Clause")
    assert is_valid_license("BSD-2-Clause")
    assert is_valid_license("SPDX-License-Identifier: BSD-3-Clause")
    assert is_valid_license("Redistribution and use in source and binary forms, with or without modification, are permitted")


def test_is_valid_license_rejects_restrictive():
    assert not is_valid_license("GPL-3.0")
    assert not is_valid_license("SPDX-License-Identifier: GPL-3.0-only")
    assert not is_valid_license("GNU General Public License v3.0")
    assert not is_valid_license("Proprietary - all unauthorized distribution prohibited")
    assert not is_valid_license("Commercial license required for production usage.")
    assert not is_valid_license("")
    assert not is_valid_license("Just normal code def foo(): pass")


# ---------------------------------------------------------------------------
# Filters: Deduplication Tests
# ---------------------------------------------------------------------------

def test_deduplicate_texts():
    texts = [
        "def foo(): return 1",
        "def bar(): return 2",
        "def foo(): return 1",
        "def baz(): return 3",
        "def bar(): return 2",
    ]
    unique = deduplicate_texts(texts)
    assert len(unique) == 3
    assert unique == [
        "def foo(): return 1",
        "def bar(): return 2",
        "def baz(): return 3",
    ]


def test_deduplicate_empty():
    assert deduplicate_texts([]) == []


# ---------------------------------------------------------------------------
# Prepare: Seed Corpora Tests
# ---------------------------------------------------------------------------

def test_generate_seed_coding_corpus():
    corpus = generate_seed_coding_corpus()
    assert isinstance(corpus, list)
    assert len(corpus) >= 15
    joined = " ".join(corpus)
    # Check for core algorithmic concepts
    assert "quicksort" in joined.lower() or "partition" in joined.lower()
    assert "mergesort" in joined.lower()
    assert "binary_search" in joined.lower()
    assert "trie" in joined.lower()
    assert "async" in joined.lower()
    assert "fastapi" in joined.lower() or "http" in joined.lower()


def test_generate_seed_design_corpus():
    corpus = generate_seed_design_corpus()
    assert isinstance(corpus, list)
    assert len(corpus) >= 15
    joined = " ".join(corpus)
    # Check for design & UX concepts
    assert "flow:" in joined.lower()
    assert "tokens:" in joined.lower()
    assert "wcag" in joined.lower()
    assert "contrast" in joined.lower()
    assert "aria" in joined.lower()


def test_save_corpus_to_file():
    with tempfile.TemporaryDirectory() as tmpdir:
        out_file = os.path.join(tmpdir, "corpus.txt")
        corpus = ["snippet 1", "snippet 2"]
        save_corpus_to_file(corpus, out_file)
        assert os.path.isfile(out_file)
        with open(out_file, "r", encoding="utf-8") as f:
            content = f.read()
        assert "snippet 1\n\nsnippet 2" == content


# ---------------------------------------------------------------------------
# Dataset: CausalTextDataset Tests
# ---------------------------------------------------------------------------

def test_causal_text_dataset_shapes_and_shifts():
    tok = BPETokenizer()
    tok.train("def hello_world(): return 42\n", target_vocab_size=270)
    
    texts = [
        "def add(a, b): return a + b\n",
        "def sub(a, b): return a - b\n",
    ]
    max_seq_len = 16
    ds = CausalTextDataset(texts, tok, max_seq_len=max_seq_len)
    assert len(ds) > 0

    item = ds[0]
    assert isinstance(item, dict)
    assert "input_ids" in item and "labels" in item
    assert isinstance(item["input_ids"], torch.Tensor)
    assert isinstance(item["labels"], torch.Tensor)
    assert item["input_ids"].shape == (max_seq_len,)
    assert item["labels"].shape == (max_seq_len,)
    assert item["input_ids"].dtype == torch.long
    assert item["labels"].dtype == torch.long

    # Verify shift: labels[i] should match input_ids[i + 1]
    input_ids = item["input_ids"]
    labels = item["labels"]
    assert torch.equal(input_ids[1:], labels[:-1])


def test_causal_text_dataset_dataloader():
    tok = BPETokenizer()
    tok.train("let mut x = 10; x += 1; println!(\"{}\", x);\n", target_vocab_size=270)

    corpus = [
        "let a = 1; let b = 2;\n",
        "let c = a + b;\n",
        "println!(\"sum={}\", c);\n",
    ]
    max_seq_len = 12
    ds = CausalTextDataset(corpus, tok, max_seq_len=max_seq_len)
    loader = DataLoader(ds, batch_size=2, shuffle=False)

    for batch in loader:
        assert "input_ids" in batch and "labels" in batch
        assert batch["input_ids"].dim() == 2
        assert batch["input_ids"].shape[1] == max_seq_len
        assert batch["labels"].shape[1] == max_seq_len
        break


def test_causal_text_dataset_short_text_padding():
    tok = BPETokenizer()
    tok.train("hi", target_vocab_size=265)

    ds = CausalTextDataset(["hi"], tok, max_seq_len=32, drop_last=False)
    assert len(ds) == 1
    sample = ds[0]
    assert sample["input_ids"].shape == (32,)
    assert sample["labels"].shape == (32,)
    # Pad tokens should be present
    assert tok.pad_id in sample["input_ids"].tolist()
