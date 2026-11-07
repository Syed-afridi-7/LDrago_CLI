"""PyTorch Dataset implementations for Causal Language Modeling in LDrago."""

from __future__ import annotations

from typing import List, Optional, Sequence, Union
import torch
from torch.utils.data import Dataset

from tokenizer.bpe import BPETokenizer


class CausalTextDataset(Dataset):
    """PyTorch Dataset for Causal Language Modeling.

    Tokenizes text corpora, concatenates the resulting token streams,
    and slices the stream into windows of `max_seq_len`.

    Returns dictionaries containing:
        - "input_ids": torch.LongTensor of shape (max_seq_len,)
        - "labels": torch.LongTensor of shape (max_seq_len,) shifted by one position.
    """

    def __init__(
        self,
        texts: Union[Sequence[str], str],
        tokenizer: BPETokenizer,
        max_seq_len: int = 512,
        stride: Optional[int] = None,
        drop_last: bool = False,
        add_eos: bool = True,
    ) -> None:
        super().__init__()
        self.tokenizer = tokenizer
        self.max_seq_len = max_seq_len
        self.stride = stride or max_seq_len
        self.drop_last = drop_last
        self.add_eos = add_eos

        if isinstance(texts, str):
            text_list = [texts]
        else:
            text_list = list(texts)

        # Tokenize entire stream
        all_tokens: List[int] = []
        for text in text_list:
            if not text:
                continue
            toks = tokenizer.encode(text)
            if toks:
                all_tokens.extend(toks)
                if self.add_eos and tokenizer.eos_id is not None:
                    all_tokens.append(tokenizer.eos_id)

        self.samples: List[tuple[torch.Tensor, torch.Tensor]] = []
        self._build_windows(all_tokens)

    def _build_windows(self, tokens: List[int]) -> None:
        if not tokens:
            return

        target_window_size = self.max_seq_len + 1

        # If total tokens is smaller than the required window size
        if len(tokens) < target_window_size:
            if not self.drop_last and len(tokens) > 0:
                pad_count = target_window_size - len(tokens)
                padded = tokens + [self.tokenizer.pad_id] * pad_count
                inp = torch.tensor(padded[:-1], dtype=torch.long)
                tgt = torch.tensor(padded[1:], dtype=torch.long)
                self.samples.append((inp, tgt))
            return

        # Slicing windows across tokens
        for start_idx in range(0, len(tokens) - self.max_seq_len, self.stride):
            chunk = tokens[start_idx : start_idx + target_window_size]
            if len(chunk) == target_window_size:
                inp = torch.tensor(chunk[:-1], dtype=torch.long)
                tgt = torch.tensor(chunk[1:], dtype=torch.long)
                self.samples.append((inp, tgt))
            elif not self.drop_last and len(chunk) > 0:
                pad_count = target_window_size - len(chunk)
                padded = chunk + [self.tokenizer.pad_id] * pad_count
                inp = torch.tensor(padded[:-1], dtype=torch.long)
                tgt = torch.tensor(padded[1:], dtype=torch.long)
                self.samples.append((inp, tgt))

    def __len__(self) -> int:
        return len(self.samples)

    def __getitem__(self, idx: int) -> dict[str, torch.Tensor]:
        inp, tgt = self.samples[idx]
        return {
            "input_ids": inp,
            "labels": tgt,
        }
