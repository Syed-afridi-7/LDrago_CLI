"""Tests for custom Byte-Pair Encoding (BPE) Tokenizer."""

import os
import tempfile
import pytest

from tokenizer.bpe import BPETokenizer, DEFAULT_SPECIAL_TOKENS


def test_special_tokens_initialization():
    tokenizer = BPETokenizer()
    # 8 special tokens + 256 byte tokens = 264 initial vocab
    assert tokenizer.vocab_size == 264
    assert tokenizer.pad_id == 0
    assert tokenizer.bos_id == 1
    assert tokenizer.eos_id == 2
    assert tokenizer.unk_id == 3

    for tok in DEFAULT_SPECIAL_TOKENS:
        assert tok in tokenizer.special_to_id
        tid = tokenizer.special_to_id[tok]
        assert tokenizer.decode([tid]) == tok


def test_roundtrip_plain_text():
    tokenizer = BPETokenizer()
    text = "Hello, world! This is a test of zero-dependency BPE tokenizer."
    token_ids = tokenizer.encode(text)
    decoded = tokenizer.decode(token_ids)
    assert decoded == text


def test_roundtrip_code_snippet():
    tokenizer = BPETokenizer()
    code = (
        "fn main() {\n"
        "    println!(\"Hello, LDrago CLI!\");\n"
        "    let mut count: usize = 42;\n"
        "    for i in 0..10 {\n"
        "        count += i;\n"
        "    }\n"
        "}\n"
    )
    token_ids = tokenizer.encode(code)
    decoded = tokenizer.decode(token_ids)
    assert decoded == code


def test_roundtrip_unicode_and_emojis():
    tokenizer = BPETokenizer()
    text = "🦀 Rust + 🐍 Python = ⚡ Blazing fast AI system! こんにちは世界 🚀✨"
    token_ids = tokenizer.encode(text)
    decoded = tokenizer.decode(token_ids)
    assert decoded == text


def test_special_tokens_in_text():
    tokenizer = BPETokenizer()
    text = "<BOS><CODE>def solve():\n    return 42</CODE><FLOW><STATE><EOS>"
    token_ids = tokenizer.encode(text)

    # First token must be BOS (id 1)
    assert token_ids[0] == tokenizer.bos_id
    # Last token must be EOS (id 2)
    assert token_ids[-1] == tokenizer.eos_id

    decoded = tokenizer.decode(token_ids)
    assert decoded == text


def test_bpe_training_and_compression():
    tokenizer = BPETokenizer()
    training_corpus = (
        "function calculateMetrics() {\n"
        "    let metrics = getSystemMetrics();\n"
        "    processMetrics(metrics);\n"
        "    logMetrics(metrics);\n"
        "    return metrics;\n"
        "}\n"
    ) * 20

    initial_size = tokenizer.vocab_size
    target_vocab_size = 300
    tokenizer.train(training_corpus, vocab_size=target_vocab_size)

    # Vocab size should expand beyond base 264
    assert tokenizer.vocab_size > initial_size
    assert tokenizer.vocab_size <= target_vocab_size

    # Merged encoding should use fewer tokens than raw byte encoding
    raw_tokenizer = BPETokenizer()
    raw_encoded = raw_tokenizer.encode(training_corpus)
    trained_encoded = tokenizer.encode(training_corpus)
    assert len(trained_encoded) < len(raw_encoded)

    # Roundtrip must still be 100% lossless
    assert tokenizer.decode(trained_encoded) == training_corpus


def test_save_and_load_roundtrip():
    tokenizer = BPETokenizer()
    corpus = "struct TokenizerTest { id: u64, name: String }\n" * 15
    tokenizer.train(corpus, vocab_size=290)

    test_input = "<BOS><CODE>struct TokenizerTest { id: 1, name: \"LDrago\".to_string() }</CODE><EOS>"
    expected_ids = tokenizer.encode(test_input)

    with tempfile.TemporaryDirectory() as tmp_dir:
        save_path = os.path.join(tmp_dir, "tokenizer.json")
        tokenizer.save(save_path)
        assert os.path.exists(save_path)

        loaded_tok = BPETokenizer.from_file(save_path)
        assert loaded_tok.vocab_size == tokenizer.vocab_size
        loaded_ids = loaded_tok.encode(test_input)
        assert loaded_ids == expected_ids
        assert loaded_tok.decode(loaded_ids) == test_input
