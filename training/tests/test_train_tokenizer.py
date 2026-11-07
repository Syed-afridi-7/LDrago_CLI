"""Unit tests for scalable BPE tokenizer training script."""

import json
import os
import tempfile
import pytest

from tokenizer.bpe import BPETokenizer
from tokenizer.train_tokenizer import (
    CODING_DOMAIN_TOKENS,
    DESIGN_DOMAIN_TOKENS,
    get_domain_special_tokens,
    train_and_save_tokenizer,
)


def test_domain_special_tokens_list():
    coding_tokens = get_domain_special_tokens("coding")
    for tok in CODING_DOMAIN_TOKENS:
        assert tok in coding_tokens

    design_tokens = get_domain_special_tokens("design")
    for tok in DESIGN_DOMAIN_TOKENS:
        assert tok in design_tokens


def test_train_and_save_coding_tokenizer():
    with tempfile.TemporaryDirectory() as tmpdir:
        corpus_path = os.path.join(tmpdir, "corpus.txt")
        output_path = os.path.join(tmpdir, "coding_tokenizer.json")

        sample_code = (
            "<FILE> src/main.rs </FILE>\n"
            "<CODE>\n"
            "fn main() {\n"
            "    println!(\"Hello LDrago CLI\");\n"
            "}\n"
            "</CODE>\n"
            "<TEST>\n"
            "assert_eq!(1 + 1, 2);\n"
            "</TEST>\n"
        )
        with open(corpus_path, "w", encoding="utf-8") as f:
            f.write(sample_code * 10)

        tokenizer = train_and_save_tokenizer(
            corpus_path=corpus_path,
            output_path=output_path,
            vocab_size=300,
            domain="coding",
        )

        assert os.path.isfile(output_path)
        assert tokenizer.vocab_size >= len(tokenizer.special_tokens) + 256

        # Check special tokens in tokenizer
        for tok in CODING_DOMAIN_TOKENS:
            assert tok in tokenizer.special_to_id

        # Verify from_file loading
        loaded_tok = BPETokenizer.from_file(output_path)
        assert loaded_tok.vocab_size == tokenizer.vocab_size

        # Roundtrip encoding/decoding with special tokens
        text = "<FILE> test.py </FILE> <CODE> def run(): pass </CODE>"
        token_ids = loaded_tok.encode(text)
        decoded = loaded_tok.decode(token_ids)
        assert "<FILE>" in decoded
        assert "</FILE>" not in loaded_tok.special_to_id  # plain text encoded
        assert "<CODE>" in decoded
        assert "</CODE>" in decoded or "<ENDCODE>" in loaded_tok.special_to_id
        assert "def run(): pass" in decoded


def test_train_and_save_design_tokenizer():
    with tempfile.TemporaryDirectory() as tmpdir:
        corpus_path = os.path.join(tmpdir, "corpus.txt")
        output_path = os.path.join(tmpdir, "design_tokenizer.json")

        sample_design = (
            "<FLOW> UserAuth\n"
            "State: IDLE -> SUBMITTING -> AUTH_SUCCESS\n"
            "</FLOW>\n"
            "<TOKENS>\n"
            "color.primary: #6366f1\n"
            "spacing.md: 16px\n"
            "</TOKENS>\n"
            "<CRITIQUE>\n"
            "WCAG 2.2 AA Contrast: 4.5:1 required\n"
            "</CRITIQUE>\n"
        )
        with open(corpus_path, "w", encoding="utf-8") as f:
            f.write(sample_design * 10)

        tokenizer = train_and_save_tokenizer(
            corpus_path=corpus_path,
            output_path=output_path,
            vocab_size=300,
            domain="design",
        )

        assert os.path.isfile(output_path)
        for tok in DESIGN_DOMAIN_TOKENS:
            assert tok in tokenizer.special_to_id

        # Reload and check roundtrip
        loaded = BPETokenizer.from_file(output_path)
        text = "<FLOW> Flow: Test </FLOW> <TOKENS> color: #fff </TOKENS>"
        ids = loaded.encode(text)
        decoded = loaded.decode(ids)
        assert "<FLOW>" in decoded
        assert "<TOKENS>" in decoded


def test_train_from_directory():
    with tempfile.TemporaryDirectory() as tmpdir:
        corpus_dir = os.path.join(tmpdir, "corpus_dir")
        os.makedirs(corpus_dir, exist_ok=True)
        output_path = os.path.join(tmpdir, "dir_tokenizer.json")

        # Create multiple text files
        for i in range(5):
            fpath = os.path.join(corpus_dir, f"file_{i}.txt")
            with open(fpath, "w", encoding="utf-8") as f:
                f.write(f"function compute_{i}() {{ return {i} * 10; }}\n" * 5)

        tok = train_and_save_tokenizer(
            corpus_path=corpus_dir,
            output_path=output_path,
            vocab_size=280,
            domain="coding",
        )
        assert os.path.isfile(output_path)
        assert tok.vocab_size >= 266


def test_train_nonexistent_corpus_raises():
    with pytest.raises(FileNotFoundError):
        train_and_save_tokenizer(
            corpus_path="nonexistent_corpus_path_abc123.txt",
            output_path="out.json",
        )
