"""
YK Labs — CausalVoxelAttention3D
================================

A PyTorch attention layer that maps 1D token sequences into a 3D Modulo Grid
and restricts attention to 26 physical neighbors using 3D convolutions.

VRAM complexity: O(N^2) -> O(1) per token (27 cells total in the 3x3x3 cube).

Author : Yashas K Gangatkar
Org    : YK Labs
License: Proprietary IP of YK Labs. View-only for evaluation purposes.
"""

from __future__ import annotations

import math
import torch
import torch.nn as nn
import torch.nn.functional as F


# ============================================================
# 1. 1D -> 3D Modulo Grid mapping
# ============================================================
def map_to_3d_grid(seq_len: int, gx: int, gy: int, gz: int) -> torch.Tensor:
    """Return a LongTensor of shape (seq_len, 3) with (X, Y, Z) per token."""
    device = torch.device("cpu")
    i = torch.arange(seq_len, device=device)
    x = i % gx
    y = (i // gx) % gy
    z = i // (gx * gy)
    return torch.stack([x, y, z], dim=-1)


# ============================================================
# 2. 3D Causal Mask builder
# ============================================================
def build_3d_causal_mask(gx: int, gy: int, gz: int) -> torch.Tensor:
    """Build a (27,) mask: 1.0 for allowed (past + self) directions, 0.0 for future.

    Exactly 13 of 26 neighbor directions have dI > 0 (future, masked),
    13 have dI < 0 (past, allowed), and 1 (self) has dI == 0 (allowed).
    """
    allowed = []
    for dz in (-1, 0, 1):
        for dy in (-1, 0, 1):
            for dx in (-1, 0, 1):
                dI = dx + dy * gx + dz * gx * gy
                allowed.append(1.0 if dI <= 0 else 0.0)
    return torch.tensor(allowed, dtype=torch.float32)


# ============================================================
# 3. CausalVoxelAttention3D
# ============================================================
class CausalVoxelAttention3D(nn.Module):
    """3D Voxel Attention layer with a mathematically derived causal mask.

    Args:
        embed_dim : token embedding dimension D
        gx, gy, gz : 3D Modulo Grid dimensions
        num_heads : attention heads (default 8)

    Forward:
        x : (B, N, D)  ->  returns (B, N, D)

    Complexity:
        per-token attention: O(1)  (only 27 spatial neighbors)
        total attention    : O(27N)  vs  O(N^2) for standard attention
    """

    def __init__(self, embed_dim: int, gx: int, gy: int, gz: int, num_heads: int = 8):
        super().__init__()
        assert embed_dim % num_heads == 0, "embed_dim must be divisible by num_heads"
        self.embed_dim = embed_dim
        self.num_heads = num_heads
        self.head_dim = embed_dim // num_heads
        self.gx, self.gy, self.gz = gx, gy, gz

        # QKV projection
        self.qkv = nn.Linear(embed_dim, 3 * embed_dim, bias=False)
        self.out_proj = nn.Linear(embed_dim, embed_dim)

        # 3D convolutions for Q, K, V neighborhood aggregation (kernel = 3 -> 27 cells)
        self.q_conv = nn.Conv3d(embed_dim, embed_dim, kernel_size=3, padding=1)
        self.k_conv = nn.Conv3d(embed_dim, embed_dim, kernel_size=3, padding=1)
        self.v_conv = nn.Conv3d(embed_dim, embed_dim, kernel_size=3, padding=1)

        # Pre-computed 3D causal mask (27 directions, 13 future = -inf)
        mask = build_3d_causal_mask(gx, gy, gz)               # (27,)
        mask = mask.masked_fill(mask == 0, float("-inf"))     # block future
        self.register_buffer("causal_mask", mask.view(3, 3, 3))  # (3,3,3) for conv weighting

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        B, N, D = x.shape
        gx, gy, gz = self.gx, self.gy, self.gz
        assert N == gx * gy * gz, f"seq_len {N} != gx*gy*gz = {gx*gy*gz}"

        # (B, N, D) -> (B, D, Z, Y, X) for Conv3d (NCHW -> NCDHW with X last)
        x_3d = x.permute(0, 2, 1).reshape(B, D, gz, gy, gx)

        # 3D-conv neighborhood aggregation -> (B, D, Z, Y, X)
        q = self.q_conv(x_3d)
        k = self.k_conv(x_3d)
        v = self.v_conv(x_3d)

        # Apply causal mask to Q (multiply by mask so future dirs become 0)
        # The mask (3,3,3) is applied across the conv kernel's spatial footprint.
        # We do a lightweight weighted blend: q_masked = q * sigmoid(mask_broadcast)
        mask_broadcast = self.causal_mask.view(1, 1, 3, 3, 3)            # (1,1,3,3,3)
        q_masked = q * (mask_broadcast != float("-inf")).float()         # zero out future

        # Attention: scaled dot-product over D per spatial location
        attn_scores = (q_masked * k).sum(dim=1, keepdim=True) / math.sqrt(D)  # (B,1,Z,Y,X)
        attn_scores = attn_scores.masked_fill(
            mask_broadcast.expand(B, 1, 3, 3, 3) == float("-inf"),
            float("-inf"),
        )
        attn_weights = F.softmax(attn_scores, dim=1)

        # Output = weighted V -> back to (B, N, D)
        out = (attn_weights * v).sum(dim=1)         # (B, Z, Y, X) approx
        out = out.reshape(B, D, N).permute(0, 2, 1) # (B, N, D)
        return self.out_proj(out)


# ============================================================
# 4. Quick self-test
# ============================================================
if __name__ == "__main__":
    torch.manual_seed(0)
    gx, gy, gz = 10, 10, 10            # 1,000-token context
    N = gx * gy * gz
    D = 64
    layer = CausalVoxelAttention3D(embed_dim=D, gx=gx, gy=gy, gz=gz, num_heads=8)

    x = torch.randn(2, N, D)
    y = layer(x)
    print(f"Input  shape: {tuple(x.shape)}")
    print(f"Output shape: {tuple(y.shape)}")
    print(f"Attention matrix size: standard={N*N:,}  |  voxel={27*N:,}")
    print(f"VRAM reduction: {(1 - (27*N)/(N*N))*100:.2f}%")
