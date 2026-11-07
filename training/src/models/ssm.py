"""Custom State-Space Model / Mamba (LDragoDesigner) in pure PyTorch with zero external model libraries.

Components:
- selective_scan recurrence implementation in pure PyTorch tensor operations
- MambaBlock: input projection, 1D causal depthwise Conv1d (groups=d_inner),
  input-dependent delta, B, C projections, log-space A parameter,
  selective scan recurrence, SiLU gating, output projection
- LDragoDesigner full model with token embeddings, stacked Mamba blocks,
  RMSNorm, and tied language model head.
"""

from __future__ import annotations

from typing import Optional

import torch
import torch.nn as nn
import torch.nn.functional as F

from .config import SSMConfig
from .transformer import RMSNorm


def selective_scan(
    u: torch.Tensor,
    delta: torch.Tensor,
    A: torch.Tensor,
    B: torch.Tensor,
    C: torch.Tensor,
    D: Optional[torch.Tensor] = None,
) -> torch.Tensor:
    """Selective scan recurrence in pure PyTorch tensor operations.

    Args:
        u: Input signal (batch, seq_len, d_inner)
        delta: Discretization step (batch, seq_len, d_inner)
        A: Transition matrix parameter (d_inner, d_state)
        B: Input-to-state projection (batch, seq_len, d_state)
        C: State-to-output projection (batch, seq_len, d_state)
        D: Optional skip parameter (d_inner,)

    Returns:
        y: Transformed output tensor (batch, seq_len, d_inner)
    """
    batch_size, seq_len, d_inner = u.shape
    d_state = A.shape[1]

    # Discretize continuous A into deltaA = exp(delta * A)
    # delta: (B, L, D, 1), A: (1, 1, D, N) -> deltaA: (B, L, D, N)
    deltaA = torch.exp(delta.unsqueeze(-1) * A.unsqueeze(0).unsqueeze(0))

    # Discretize continuous B into deltaB = delta * B
    # delta: (B, L, D, 1), B: (B, L, 1, N) -> deltaB: (B, L, D, N)
    deltaB = delta.unsqueeze(-1) * B.unsqueeze(2)

    # Input driving term deltaB_u = deltaB * u
    # u: (B, L, D, 1) -> deltaB_u: (B, L, D, N)
    deltaB_u = deltaB * u.unsqueeze(-1)

    # Recurrent scan over sequence length
    h = torch.zeros(batch_size, d_inner, d_state, device=u.device, dtype=u.dtype)
    ys = []

    for t in range(seq_len):
        h = deltaA[:, t] * h + deltaB_u[:, t]
        # State to output contraction: y_t = sum_n (h_{b,d,n} * C_{b,n})
        y_t = (h * C[:, t].unsqueeze(1)).sum(dim=-1)  # (batch, d_inner)
        ys.append(y_t)

    y = torch.stack(ys, dim=1)  # (batch, seq_len, d_inner)

    if D is not None:
        y = y + u * D.unsqueeze(0).unsqueeze(0)

    return y


class MambaBlock(nn.Module):
    """Mamba selective state-space sequence model block."""

    def __init__(self, config: SSMConfig) -> None:
        super().__init__()
        self.d_model = config.d_model
        self.d_inner = config.d_inner
        self.d_state = config.d_state
        self.d_conv = config.d_conv
        self.dt_rank = config.dt_rank or 16

        # Projection from d_model to 2 * d_inner (split into x and z branches)
        self.in_proj = nn.Linear(self.d_model, 2 * self.d_inner, bias=config.bias)

        # 1D causal depthwise convolution
        self.conv1d = nn.Conv1d(
            in_channels=self.d_inner,
            out_channels=self.d_inner,
            kernel_size=self.d_conv,
            groups=self.d_inner,
            bias=config.conv_bias,
            padding=self.d_conv - 1,  # Left-aligned causal padding
        )

        # Projections for delta_raw, B, and C
        self.x_proj = nn.Linear(self.d_inner, self.dt_rank + 2 * self.d_state, bias=False)

        # Projection from dt_rank to d_inner
        self.dt_proj = nn.Linear(self.dt_rank, self.d_inner, bias=True)

        # Log-space A parameter: A is strictly negative: A = -exp(A_log)
        A = torch.arange(1, self.d_state + 1, dtype=torch.float32).repeat(self.d_inner, 1)
        self.A_log = nn.Parameter(torch.log(A))

        # Learnable skip parameter D
        self.D = nn.Parameter(torch.ones(self.d_inner))

        # Output projection back to d_model
        self.out_proj = nn.Linear(self.d_inner, self.d_model, bias=config.bias)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """Forward pass through Mamba block.

        x: (batch, seq_len, d_model)
        Returns: (batch, seq_len, d_model)
        """
        batch_size, seq_len, _ = x.shape

        # Step 1: Input projection to (x_branch, z_branch)
        xz = self.in_proj(x)
        x_branch, z_branch = torch.chunk(xz, chunks=2, dim=-1)

        # Step 2: 1D causal depthwise convolution along sequence length
        # Permute to (batch, d_inner, seq_len)
        x_conv = self.conv1d(x_branch.transpose(1, 2))
        # Enforce causality by truncating to original seq_len
        x_conv = x_conv[:, :, :seq_len].transpose(1, 2)
        x_conv = F.silu(x_conv)

        # Step 3: Input-dependent projections for delta, B, C
        x_dbl = self.x_proj(x_conv)  # (batch, seq_len, dt_rank + 2 * d_state)
        delta_raw, B, C = torch.split(
            x_dbl, [self.dt_rank, self.d_state, self.d_state], dim=-1
        )

        delta = F.softplus(self.dt_proj(delta_raw))  # (batch, seq_len, d_inner)
        A = -torch.exp(self.A_log.float())  # (d_inner, d_state)

        # Step 4: Selective scan
        y = selective_scan(x_conv, delta, A, B, C, self.D)

        # Step 5: SiLU gating with branch z
        y = y * F.silu(z_branch)

        # Step 6: Output projection
        out = self.out_proj(y)
        return out


class LDragoDesigner(nn.Module):
    """LDragoDesigner State-Space Model with stacked Mamba blocks and tied language model head."""

    def __init__(self, config: SSMConfig) -> None:
        super().__init__()
        self.config = config
        self.tok_embeddings = nn.Embedding(config.vocab_size, config.d_model)

        self.layers = nn.ModuleList([
            nn.ModuleDict({
                "norm": RMSNorm(config.d_model, eps=config.norm_eps),
                "mamba": MambaBlock(config),
            })
            for _ in range(config.n_layers)
        ])

        self.norm = RMSNorm(config.d_model, eps=config.norm_eps)
        self.lm_head = nn.Linear(config.d_model, config.vocab_size, bias=False)

        if config.tie_weights:
            self.lm_head.weight = self.tok_embeddings.weight

    def forward(self, input_ids: torch.Tensor) -> torch.Tensor:
        """Forward pass.

        input_ids: (batch, seq_len)
        Returns:
            logits: (batch, seq_len, vocab_size)
        """
        x = self.tok_embeddings(input_ids)

        for layer in self.layers:
            norm_layer = layer["norm"]
            mamba_layer = layer["mamba"]
            x = x + mamba_layer(norm_layer(x))

        x = self.norm(x)
        logits = self.lm_head(x)  # (batch, seq_len, vocab_size)
        return logits
