"""Custom Byte-Pair Encoding (BPE) Tokenizer built from scratch with zero external dependencies.

Starts with 256 byte-level tokens + special domain tokens:
<PAD>, <BOS>, <EOS>, <UNK>, <CODE>, <ENDCODE>, <FLOW>, <STATE>
"""

from __future__ import annotations

import json
import os
import re
from typing import Dict, List, Optional, Tuple

DEFAULT_SPECIAL_TOKENS: list[str] = [
    "<PAD>",
    "<BOS>",
    "<EOS>",
    "<UNK>",
    "<CODE>",
    "<ENDCODE>",
    "<FLOW>",
    "<STATE>",
]


class BPETokenizer:
    """Byte-level Byte-Pair Encoding Tokenizer with domain-specific special tokens."""

    def __init__(self, special_tokens: Optional[list[str]] = None) -> None:
        self.special_tokens = list(special_tokens or DEFAULT_SPECIAL_TOKENS)
        self._init_vocab()

    def _init_vocab(self) -> None:
        self.special_to_id: Dict[str, int] = {
            tok: idx for idx, tok in enumerate(self.special_tokens)
        }
        self.id_to_special: Dict[int, str] = {
            idx: tok for idx, tok in enumerate(self.special_tokens)
        }
        self.special_offset = len(self.special_tokens)

        # 256 byte-level tokens mapped to IDs [special_offset, special_offset + 255]
        self.byte_to_id: Dict[int, int] = {
            b: self.special_offset + b for b in range(256)
        }
        self.id_to_bytes: Dict[int, bytes] = {
            self.special_offset + b: bytes([b]) for b in range(256)
        }

        # Merges: (id1, id2) -> new_token_id
        self.merges: Dict[Tuple[int, int], int] = {}
        self._compile_special_regex()

    def _compile_special_regex(self) -> None:
        if self.special_tokens:
            # Escape and sort by length descending to match longest tokens first
            escaped = [re.escape(tok) for tok in sorted(self.special_tokens, key=len, reverse=True)]
            self._special_pattern = re.compile(f"({'|'.join(escaped)})")
        else:
            self._special_pattern = None

    @property
    def pad_id(self) -> int:
        return self.special_to_id.get("<PAD>", 0)

    @property
    def bos_id(self) -> int:
        return self.special_to_id.get("<BOS>", 1)

    @property
    def eos_id(self) -> int:
        return self.special_to_id.get("<EOS>", 2)

    @property
    def unk_id(self) -> int:
        return self.special_to_id.get("<UNK>", 3)

    @property
    def vocab_size(self) -> int:
        return self.special_offset + 256 + len(self.merges)

    def train(
        self,
        text: str,
        vocab_size: Optional[int] = None,
        target_vocab_size: Optional[int] = None,
    ) -> None:
        """Train BPE merges on input text until vocab_size is reached or no frequent pairs remain."""
        if vocab_size is None:
            vocab_size = target_vocab_size
        if vocab_size is None:
            raise ValueError("Must provide vocab_size or target_vocab_size")

        # Re-initialize vocabulary to clear previous merges
        self._init_vocab()

        base_vocab_size = self.special_offset + 256
        if vocab_size <= base_vocab_size:
            return

        num_merges = vocab_size - base_vocab_size
        raw_bytes = text.encode("utf-8")
        if not raw_bytes:
            return

        # Start with byte tokens
        ids: List[int] = [self.byte_to_id[b] for b in raw_bytes]

        for _ in range(num_merges):
            if len(ids) < 2:
                break

            # Count all adjacent pairs
            pair_counts: Dict[Tuple[int, int], int] = {}
            for i in range(len(ids) - 1):
                pair = (ids[i], ids[i + 1])
                pair_counts[pair] = pair_counts.get(pair, 0) + 1

            if not pair_counts:
                break

            # Find most frequent pair; ties broken deterministically
            best_pair, max_count = max(pair_counts.items(), key=lambda item: (item[1], -item[0][0], -item[0][1]))
            if max_count < 2:
                # No repeating pairs found
                break

            new_id = self.vocab_size
            self.merges[best_pair] = new_id
            self.id_to_bytes[new_id] = (
                self.id_to_bytes[best_pair[0]] + self.id_to_bytes[best_pair[1]]
            )

            # Apply merge to current ids sequence
            new_ids: List[int] = []
            i = 0
            while i < len(ids):
                if i < len(ids) - 1 and (ids[i], ids[i + 1]) == best_pair:
                    new_ids.append(new_id)
                    i += 2
                else:
                    new_ids.append(ids[i])
                    i += 1
            ids = new_ids

    def _encode_chunk(self, raw_bytes: bytes) -> List[int]:
        """Encode raw bytes using learned BPE merges."""
        if not raw_bytes:
            return []

        ids: List[int] = [self.byte_to_id[b] for b in raw_bytes]
        if len(ids) < 2 or not self.merges:
            return ids

        while len(ids) >= 2:
            # Find candidate pair with the lowest merge id (highest priority)
            best_pair: Optional[Tuple[int, int]] = None
            min_merge_id = float("inf")
            for i in range(len(ids) - 1):
                pair = (ids[i], ids[i + 1])
                mid = self.merges.get(pair)
                if mid is not None and mid < min_merge_id:
                    min_merge_id = mid
                    best_pair = pair

            if best_pair is None:
                break

            # Replace occurrences of best_pair
            new_ids: List[int] = []
            i = 0
            target_id = int(min_merge_id)
            while i < len(ids):
                if i < len(ids) - 1 and (ids[i], ids[i + 1]) == best_pair:
                    new_ids.append(target_id)
                    i += 2
                else:
                    new_ids.append(ids[i])
                    i += 1
            ids = new_ids

        return ids

    def encode(self, text: str) -> List[int]:
        """Encode text into token IDs, recognizing special tokens."""
        if not text:
            return []

        if not self._special_pattern:
            return self._encode_chunk(text.encode("utf-8"))

        parts = self._special_pattern.split(text)
        result: List[int] = []
        for part in parts:
            if not part:
                continue
            if part in self.special_to_id:
                result.append(self.special_to_id[part])
            else:
                result.extend(self._encode_chunk(part.encode("utf-8")))

        return result

    def decode(self, ids: List[int]) -> str:
        """Decode a list of token IDs back into a string, preserving special tokens."""
        chunks: List[str] = []
        byte_buffer = bytearray()

        def flush_buffer() -> None:
            if byte_buffer:
                chunks.append(byte_buffer.decode("utf-8", errors="replace"))
                byte_buffer.clear()

        for token_id in ids:
            if token_id in self.id_to_special:
                flush_buffer()
                chunks.append(self.id_to_special[token_id])
            elif token_id in self.id_to_bytes:
                byte_buffer.extend(self.id_to_bytes[token_id])
            else:
                flush_buffer()
                chunks.append(self.id_to_special.get(self.unk_id, "<UNK>"))

        flush_buffer()
        return "".join(chunks)

    def save(self, path: str) -> None:
        """Save tokenizer vocabulary and merges to a JSON file."""
        os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
        data = {
            "special_tokens": self.special_tokens,
            "merges": [
                {"pair": [p[0], p[1]], "id": nid}
                for p, nid in sorted(self.merges.items(), key=lambda x: x[1])
            ],
            "vocab_size": self.vocab_size,
        }
        with open(path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2, ensure_ascii=False)

    def load(self, path: str) -> None:
        """Load tokenizer vocabulary and merges from a JSON file."""
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)

        self.special_tokens = list(data.get("special_tokens", DEFAULT_SPECIAL_TOKENS))
        self._init_vocab()

        for merge_info in data.get("merges", []):
            p0, p1 = merge_info["pair"]
            nid = merge_info["id"]
            pair = (int(p0), int(p1))
            self.merges[pair] = int(nid)
            self.id_to_bytes[int(nid)] = (
                self.id_to_bytes[pair[0]] + self.id_to_bytes[pair[1]]
            )

    @classmethod
    def from_file(cls, path: str) -> BPETokenizer:
        """Factory method to load a BPETokenizer directly from file."""
        tokenizer = cls()
        tokenizer.load(path)
        return tokenizer
