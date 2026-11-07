"""Scalable Tokenizer Training script for LDrago domain-specialized models."""

from __future__ import annotations

import argparse
import os
from typing import List, Optional

from tokenizer.bpe import BPETokenizer

BASE_SPECIAL_TOKENS: List[str] = ["<PAD>", "<BOS>", "<EOS>", "<UNK>"]

CODING_DOMAIN_TOKENS: List[str] = [
    "<CODE>",
    "<ENDCODE>",
    "<FILE>",
    "<ENDFILE>",
    "<DIFF>",
    "<TEST>",
]

DESIGN_DOMAIN_TOKENS: List[str] = [
    "<FLOW>",
    "<STATE>",
    "<HANDOFF>",
    "<CRITIQUE>",
    "<TOKENS>",
]


def get_domain_special_tokens(domain: str = "coding") -> List[str]:
    """Return the ordered list of special tokens tailored to the target domain."""
    d = domain.lower()
    tokens = list(BASE_SPECIAL_TOKENS)
    if d == "coding":
        for tok in CODING_DOMAIN_TOKENS:
            if tok not in tokens:
                tokens.append(tok)
    elif d == "design":
        for tok in DESIGN_DOMAIN_TOKENS:
            if tok not in tokens:
                tokens.append(tok)
    else:
        # Fallback combining both domains
        for tok in CODING_DOMAIN_TOKENS + DESIGN_DOMAIN_TOKENS:
            if tok not in tokens:
                tokens.append(tok)
    return tokens


def load_corpus_text(corpus_path: str) -> str:
    """Read corpus text from either a single file or a directory of text files."""
    if not os.path.exists(corpus_path):
        raise FileNotFoundError(f"Corpus path does not exist: {corpus_path}")

    if os.path.isfile(corpus_path):
        with open(corpus_path, "r", encoding="utf-8", errors="replace") as f:
            return f.read()

    # If directory, concatenate all text-like files
    snippets: List[str] = []
    for root, _, files in os.walk(corpus_path):
        for fname in sorted(files):
            # Read common code/text extensions
            fpath = os.path.join(root, fname)
            try:
                with open(fpath, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
                    if content.strip():
                        snippets.append(content)
            except Exception:
                continue

    return "\n\n".join(snippets)


def train_and_save_tokenizer(
    corpus_path: str,
    output_path: str,
    vocab_size: int = 4096,
    domain: str = "coding",
) -> BPETokenizer:
    """Train a Byte-Pair Encoding (BPE) tokenizer on a domain corpus and save to disk.

    Args:
        corpus_path: Path to a text corpus file or directory containing corpus files.
        output_path: Path where the trained tokenizer JSON will be written.
        vocab_size: Target vocabulary size (base bytes + special tokens + BPE merges).
        domain: Specialization domain ("coding" or "design").

    Returns:
        The trained and saved BPETokenizer instance.
    """
    special_tokens = get_domain_special_tokens(domain)
    tokenizer = BPETokenizer(special_tokens=special_tokens)

    corpus_text = load_corpus_text(corpus_path)
    if corpus_text.strip():
        tokenizer.train(corpus_text, target_vocab_size=vocab_size)

    output_dir = os.path.dirname(os.path.abspath(output_path))
    if output_dir:
        os.makedirs(output_dir, exist_ok=True)

    tokenizer.save(output_path)
    return tokenizer


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Train domain-specific BPE Tokenizer for LDrago CLI."
    )
    parser.add_argument(
        "--corpus",
        type=str,
        required=True,
        help="Path to corpus text file or directory of text files",
    )
    parser.add_argument(
        "--output",
        type=str,
        required=True,
        help="Output JSON file path for saving trained tokenizer",
    )
    parser.add_argument(
        "--vocab-size",
        type=int,
        default=4096,
        help="Target vocabulary size (default: 4096)",
    )
    parser.add_argument(
        "--domain",
        type=str,
        choices=["coding", "design"],
        default="coding",
        help="Model domain ('coding' or 'design')",
    )

    args = parser.parse_args()
    tokenizer = train_and_save_tokenizer(
        corpus_path=args.corpus,
        output_path=args.output,
        vocab_size=args.vocab_size,
        domain=args.domain,
    )
    print(
        f"Successfully trained {args.domain} tokenizer with vocab_size={tokenizer.vocab_size} -> {args.output}"
    )


if __name__ == "__main__":
    main()
