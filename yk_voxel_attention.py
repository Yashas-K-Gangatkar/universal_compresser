import torch
import torch.nn as nn
import torch.nn.functional as F

class CausalVoxelAttention3D(nn.Module):
    def __init__(self, embed_dim, grid_x, grid_y, grid_z):
        super().__init__()
        self.grid_x, self.grid_y, self.grid_z = grid_x, grid_y, grid_z
        self.query = nn.Linear(embed_dim, embed_dim)
        self.key = nn.Linear(embed_dim, embed_dim)
        self.value = nn.Linear(embed_dim, embed_dim)
        mask = torch.zeros(27)
        idx = 0
        for dz in [-1, 0, 1]:
            for dy in [-1, 0, 1]:
                for dx in [-1, 0, 1]:
                    if dx + (dy * self.grid_x) + (dz * self.grid_x * self.grid_y) > 0:
                        mask[idx] = float('-inf')
                    idx += 1
        self.register_buffer('causal_mask', mask)

    def forward(self, x):
        B, S, E = x.shape
        x_3d = x.view(B, self.grid_x, self.grid_y, self.grid_z, E).permute(0, 4, 3, 2, 1)
        x_padded = F.pad(x_3d, (1,1, 1,1, 1,1), mode='constant', value=0)
        unfolded = x_padded.unfold(2, 3, 1).unfold(3, 3, 1).unfold(4, 3, 1)
        unfolded = unfolded.contiguous().view(B, E, S, 27).permute(0, 2, 3, 1)
        Q = self.query(x).unsqueeze(-2)
        K = self.key(unfolded)
        V = self.value(unfolded)
        scores = (Q * K).sum(dim=-1).unsqueeze(-2) / (E ** 0.5)
        scores = scores + self.causal_mask.view(1, 1, 1, 27)
        weights = F.softmax(scores, dim=-1)
        return torch.matmul(weights, V).squeeze(-2)

class YKTransformerBlock(nn.Module):
    def __init__(self, embed_dim, grid_x, grid_y, grid_z):
        super().__init__()
        self.attn = CausalVoxelAttention3D(embed_dim, grid_x, grid_y, grid_z)
        self.norm1 = nn.LayerNorm(embed_dim)
        self.ffn = nn.Sequential(nn.Linear(embed_dim, embed_dim * 4), nn.GELU(), nn.Linear(embed_dim * 4, embed_dim))
        self.norm2 = nn.LayerNorm(embed_dim)
    def forward(self, x):
        x = x + self.norm1(self.attn(x))
        x = x + self.norm2(self.ffn(x))
        return x

class YKLanguageModel(nn.Module):
    def __init__(self, vocab_size, embed_dim, grid_x, grid_y, grid_z, num_layers=2):
        super().__init__()
        self.token_embed = nn.Embedding(vocab_size, embed_dim)
        self.blocks = nn.ModuleList([YKTransformerBlock(embed_dim, grid_x, grid_y, grid_z) for _ in range(num_layers)])
        self.norm = nn.LayerNorm(embed_dim)
        self.head = nn.Linear(embed_dim, vocab_size)
    def forward(self, idx):
        x = self.token_embed(idx)
        for block in self.blocks: x = block(x)
        return self.head(self.norm(x))
