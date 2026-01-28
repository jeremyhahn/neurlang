#!/usr/bin/env python3
"""
Parallel Instruction Slot Prediction Model for Neurlang

This model predicts up to 64 instructions in a single forward pass using
parallel slot prediction with cross-attention.

Architecture:
    ┌─────────────────────────────────────────────────────────────────┐
    │                    PARALLEL INSTRUCTION MODEL                    │
    ├─────────────────────────────────────────────────────────────────┤
    │                                                                 │
    │  Encoder (CNN with positional encoding):                        │
    │  ├── Embedding: 261 vocab → 128-dim                             │
    │  ├── Positional encoding (learned)                              │
    │  ├── Conv1d layers: 128 → 256 → 512 channels                    │
    │  └── Output: (batch, 512, seq_len) - preserve sequence          │
    │                                                                 │
    │  Decoder (parallel slot prediction via cross-attention):        │
    │  ├── Learned slot queries: 64 × 512-dim vectors                 │
    │  ├── Cross-attention: slots attend to encoder output            │
    │  └── Output: (batch, 64, 512) - one vector per slot             │
    │                                                                 │
    │  Prediction Heads (per slot, all in parallel):                  │
    │  ├── valid_head: 2 classes (valid/padding)                      │
    │  ├── opcode_head: 33 classes (0x00-0x20)                        │
    │  ├── mode_head: 8 classes (operation subtype)                   │
    │  ├── rd_head: 32 classes (destination register)                 │
    │  ├── rs1_head: 32 classes (source register 1)                   │
    │  ├── rs2_head: 32 classes (source register 2)                   │
    │  ├── has_imm_head: 2 classes (yes/no)                           │
    │  └── imm_head: 256 bins (quantized to 8-bit)                    │
    │                                                                 │
    └─────────────────────────────────────────────────────────────────┘

Key Features:
- Single forward pass predicts all 64 instruction slots
- Factored prediction (separate heads) for better learning
- Valid mask enables variable-length output
- Cross-attention allows each slot to focus on relevant input parts
"""

import torch
import torch.nn as nn
import torch.nn.functional as F
import math
from typing import Dict, Tuple, Optional


# Constants matching Neurlang IR spec
NUM_SLOTS = 64          # Max instructions per program
NUM_OPCODES = 33        # Opcodes 0x00-0x20
NUM_MODES = 8           # Mode/subtype variants
NUM_REGISTERS = 32      # r0-r31
IMM_BINS = 256          # Quantized immediate values


class PositionalEncoding(nn.Module):
    """Sinusoidal positional encoding."""

    def __init__(self, d_model: int, max_len: int = 512):
        super().__init__()
        pe = torch.zeros(max_len, d_model)
        position = torch.arange(0, max_len, dtype=torch.float).unsqueeze(1)
        div_term = torch.exp(torch.arange(0, d_model, 2).float() * (-math.log(10000.0) / d_model))
        pe[:, 0::2] = torch.sin(position * div_term)
        pe[:, 1::2] = torch.cos(position * div_term)
        self.register_buffer('pe', pe.unsqueeze(0))  # (1, max_len, d_model)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        # x: (batch, seq_len, d_model)
        return x + self.pe[:, :x.size(1)]


class MultiHeadCrossAttention(nn.Module):
    """Multi-head cross-attention for slot queries attending to encoder output."""

    def __init__(self, d_model: int, num_heads: int = 8, dropout: float = 0.1):
        super().__init__()
        assert d_model % num_heads == 0
        self.d_model = d_model
        self.num_heads = num_heads
        self.head_dim = d_model // num_heads

        self.q_proj = nn.Linear(d_model, d_model)
        self.k_proj = nn.Linear(d_model, d_model)
        self.v_proj = nn.Linear(d_model, d_model)
        self.out_proj = nn.Linear(d_model, d_model)
        self.dropout = nn.Dropout(dropout)

    def forward(
        self,
        query: torch.Tensor,   # (batch, num_slots, d_model)
        key: torch.Tensor,     # (batch, seq_len, d_model)
        value: torch.Tensor,   # (batch, seq_len, d_model)
    ) -> torch.Tensor:
        batch_size = query.size(0)

        # Project
        q = self.q_proj(query).view(batch_size, -1, self.num_heads, self.head_dim).transpose(1, 2)
        k = self.k_proj(key).view(batch_size, -1, self.num_heads, self.head_dim).transpose(1, 2)
        v = self.v_proj(value).view(batch_size, -1, self.num_heads, self.head_dim).transpose(1, 2)

        # Attention
        scores = torch.matmul(q, k.transpose(-2, -1)) / math.sqrt(self.head_dim)
        attn = F.softmax(scores, dim=-1)
        attn = self.dropout(attn)

        # Combine
        out = torch.matmul(attn, v)
        out = out.transpose(1, 2).contiguous().view(batch_size, -1, self.d_model)
        return self.out_proj(out)


class ParallelInstructionModel(nn.Module):
    """
    Parallel instruction prediction model.

    Predicts up to 64 instructions in a single forward pass using:
    - CNN encoder to extract features from input text
    - Learned slot queries for each instruction position
    - Cross-attention for slot queries to attend to encoder output
    - Parallel prediction heads for each slot
    """

    def __init__(
        self,
        vocab_size: int = 261,      # 256 bytes + 5 special tokens
        embed_dim: int = 128,
        hidden_dim: int = 512,
        num_slots: int = NUM_SLOTS,
        num_heads: int = 8,
        dropout: float = 0.1,
        max_seq_len: int = 512,
    ):
        super().__init__()

        self.vocab_size = vocab_size
        self.embed_dim = embed_dim
        self.hidden_dim = hidden_dim
        self.num_slots = num_slots

        # Encoder
        self.embedding = nn.Embedding(vocab_size, embed_dim, padding_idx=256)
        self.pos_encoding = PositionalEncoding(embed_dim, max_seq_len)

        # CNN encoder with increasing channels
        self.encoder = nn.Sequential(
            nn.Conv1d(embed_dim, 256, kernel_size=3, padding=1),
            nn.GELU(),
            nn.BatchNorm1d(256),
            nn.Conv1d(256, 512, kernel_size=3, padding=1),
            nn.GELU(),
            nn.BatchNorm1d(512),
            nn.Conv1d(512, hidden_dim, kernel_size=3, padding=1),
            nn.GELU(),
        )

        # Learned slot queries (64 slots × hidden_dim)
        self.slot_queries = nn.Parameter(torch.randn(1, num_slots, hidden_dim) * 0.02)

        # Cross-attention layers
        self.cross_attention = nn.ModuleList([
            MultiHeadCrossAttention(hidden_dim, num_heads, dropout)
            for _ in range(2)  # 2 layers of cross-attention
        ])
        self.layer_norms = nn.ModuleList([
            nn.LayerNorm(hidden_dim) for _ in range(2)
        ])

        # Feed-forward after cross-attention
        self.ffn = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim * 4),
            nn.GELU(),
            nn.Dropout(dropout),
            nn.Linear(hidden_dim * 4, hidden_dim),
            nn.Dropout(dropout),
        )
        self.ffn_norm = nn.LayerNorm(hidden_dim)

        # Prediction heads (all predict for each slot in parallel)
        self.valid_head = nn.Linear(hidden_dim, 2)
        self.opcode_head = nn.Linear(hidden_dim, NUM_OPCODES)
        self.mode_head = nn.Linear(hidden_dim, NUM_MODES)
        self.rd_head = nn.Linear(hidden_dim, NUM_REGISTERS)
        self.rs1_head = nn.Linear(hidden_dim, NUM_REGISTERS)
        self.rs2_head = nn.Linear(hidden_dim, NUM_REGISTERS)
        self.has_imm_head = nn.Linear(hidden_dim, 2)
        self.imm_head = nn.Linear(hidden_dim, IMM_BINS)

    def forward(
        self,
        x: torch.Tensor,  # (batch, seq_len) token IDs
    ) -> Dict[str, torch.Tensor]:
        """
        Forward pass.

        Returns dict with logits for each prediction head:
            valid: (batch, num_slots, 2)
            opcode: (batch, num_slots, 33)
            mode: (batch, num_slots, 8)
            rd: (batch, num_slots, 32)
            rs1: (batch, num_slots, 32)
            rs2: (batch, num_slots, 32)
            has_imm: (batch, num_slots, 2)
            imm_bin: (batch, num_slots, 256)
        """
        batch_size = x.size(0)

        # Encode input
        embedded = self.embedding(x)  # (batch, seq, embed_dim)
        embedded = self.pos_encoding(embedded)

        # CNN encoder expects (batch, channels, seq)
        encoded = self.encoder(embedded.permute(0, 2, 1))
        encoded = encoded.permute(0, 2, 1)  # Back to (batch, seq, hidden)

        # Expand slot queries for batch
        slots = self.slot_queries.expand(batch_size, -1, -1)  # (batch, num_slots, hidden)

        # Cross-attention layers
        for attn, norm in zip(self.cross_attention, self.layer_norms):
            slots = norm(slots + attn(slots, encoded, encoded))

        # Feed-forward
        slots = self.ffn_norm(slots + self.ffn(slots))

        # Parallel prediction heads
        return {
            'valid': self.valid_head(slots),
            'opcode': self.opcode_head(slots),
            'mode': self.mode_head(slots),
            'rd': self.rd_head(slots),
            'rs1': self.rs1_head(slots),
            'rs2': self.rs2_head(slots),
            'has_imm': self.has_imm_head(slots),
            'imm_bin': self.imm_head(slots),
        }

    def predict(
        self,
        x: torch.Tensor,
    ) -> Dict[str, torch.Tensor]:
        """
        Get predictions as class indices.

        Returns dict with predictions for each slot:
            valid: (batch, num_slots) 0/1
            opcode: (batch, num_slots) 0-32
            mode: (batch, num_slots) 0-7
            rd: (batch, num_slots) 0-31
            rs1: (batch, num_slots) 0-31
            rs2: (batch, num_slots) 0-31
            has_imm: (batch, num_slots) 0/1
            imm_bin: (batch, num_slots) 0-255
        """
        logits = self.forward(x)
        return {k: torch.argmax(v, dim=-1) for k, v in logits.items()}

    def count_parameters(self) -> int:
        return sum(p.numel() for p in self.parameters() if p.requires_grad)


class ParallelInstructionLoss(nn.Module):
    """
    Combined loss for parallel instruction prediction.

    Uses cross-entropy for all heads with:
    - Valid mask to only compute loss for valid slots
    - Per-head weighting for importance balancing
    """

    def __init__(
        self,
        valid_weight: float = 1.0,
        opcode_weight: float = 2.0,    # Most important
        mode_weight: float = 1.0,
        register_weight: float = 1.0,
        imm_weight: float = 0.5,
    ):
        super().__init__()

        self.weights = {
            'valid': valid_weight,
            'opcode': opcode_weight,
            'mode': mode_weight,
            'rd': register_weight,
            'rs1': register_weight,
            'rs2': register_weight,
            'has_imm': imm_weight,
            'imm_bin': imm_weight,
        }

        self.ce_loss = nn.CrossEntropyLoss(reduction='none')

    def forward(
        self,
        logits: Dict[str, torch.Tensor],
        targets: Dict[str, torch.Tensor],
    ) -> Tuple[torch.Tensor, Dict[str, float]]:
        """
        Compute combined loss.

        Args:
            logits: Dict of (batch, num_slots, num_classes) tensors
            targets: Dict of (batch, num_slots) tensors

        Returns:
            total_loss: Scalar loss
            losses: Dict of individual loss values for logging
        """
        losses = {}
        total_loss = 0.0

        # Get valid mask from targets
        valid_mask = targets['valid'].float()  # (batch, num_slots)

        for name in ['valid', 'opcode', 'mode', 'rd', 'rs1', 'rs2', 'has_imm', 'imm_bin']:
            # Reshape for cross-entropy: (batch * slots, classes) vs (batch * slots)
            logit = logits[name].view(-1, logits[name].size(-1))
            target = targets[name].view(-1)

            # Compute per-element loss
            loss = self.ce_loss(logit, target)
            loss = loss.view(targets[name].shape)  # (batch, num_slots)

            # Mask invalid slots (except for 'valid' head which we always train)
            if name != 'valid':
                loss = loss * valid_mask
                # Average over valid slots only
                loss = loss.sum() / (valid_mask.sum() + 1e-8)
            else:
                loss = loss.mean()

            losses[name] = loss.item()
            total_loss = total_loss + self.weights[name] * loss

        losses['total'] = total_loss.item()
        return total_loss, losses


class LightParallelModel(nn.Module):
    """
    Lightweight version for faster inference (~5M parameters).

    Uses simpler architecture while maintaining parallel prediction.
    """

    def __init__(
        self,
        vocab_size: int = 261,
        embed_dim: int = 64,
        hidden_dim: int = 256,
        num_slots: int = NUM_SLOTS,
    ):
        super().__init__()

        self.num_slots = num_slots

        # Simple encoder
        self.embedding = nn.Embedding(vocab_size, embed_dim, padding_idx=256)

        self.encoder = nn.Sequential(
            nn.Conv1d(embed_dim, 128, kernel_size=3, padding=1),
            nn.ReLU(),
            nn.Conv1d(128, hidden_dim, kernel_size=3, padding=1),
            nn.ReLU(),
            nn.AdaptiveAvgPool1d(1),  # Pool to single vector
            nn.Flatten(),
        )

        # Project to slot space
        self.slot_proj = nn.Linear(hidden_dim, num_slots * hidden_dim)

        # Prediction heads
        self.heads = nn.ModuleDict({
            'valid': nn.Linear(hidden_dim, 2),
            'opcode': nn.Linear(hidden_dim, NUM_OPCODES),
            'mode': nn.Linear(hidden_dim, NUM_MODES),
            'rd': nn.Linear(hidden_dim, NUM_REGISTERS),
            'rs1': nn.Linear(hidden_dim, NUM_REGISTERS),
            'rs2': nn.Linear(hidden_dim, NUM_REGISTERS),
            'has_imm': nn.Linear(hidden_dim, 2),
            'imm_bin': nn.Linear(hidden_dim, IMM_BINS),
        })

    def forward(self, x: torch.Tensor) -> Dict[str, torch.Tensor]:
        batch_size = x.size(0)

        # Encode
        embedded = self.embedding(x).permute(0, 2, 1)
        features = self.encoder(embedded)  # (batch, hidden)

        # Project to slots
        slots = self.slot_proj(features).view(batch_size, self.num_slots, -1)

        # Predict
        return {name: head(slots) for name, head in self.heads.items()}

    def count_parameters(self) -> int:
        return sum(p.numel() for p in self.parameters() if p.requires_grad)


if __name__ == "__main__":
    # Test models
    print("Testing ParallelInstructionModel...")

    model = ParallelInstructionModel()
    print(f"Full model parameters: {model.count_parameters():,}")

    light_model = LightParallelModel()
    print(f"Light model parameters: {light_model.count_parameters():,}")

    # Test forward pass
    batch_size = 4
    seq_len = 128
    x = torch.randint(0, 256, (batch_size, seq_len))

    output = model(x)
    print("\nFull model output shapes:")
    for name, tensor in output.items():
        print(f"  {name}: {tensor.shape}")

    output_light = light_model(x)
    print("\nLight model output shapes:")
    for name, tensor in output_light.items():
        print(f"  {name}: {tensor.shape}")

    # Test loss
    print("\nTesting loss computation...")
    criterion = ParallelInstructionLoss()

    # Create dummy targets
    targets = {
        'valid': torch.randint(0, 2, (batch_size, 64)),
        'opcode': torch.randint(0, 33, (batch_size, 64)),
        'mode': torch.randint(0, 8, (batch_size, 64)),
        'rd': torch.randint(0, 32, (batch_size, 64)),
        'rs1': torch.randint(0, 32, (batch_size, 64)),
        'rs2': torch.randint(0, 32, (batch_size, 64)),
        'has_imm': torch.randint(0, 2, (batch_size, 64)),
        'imm_bin': torch.randint(0, 256, (batch_size, 64)),
    }

    loss, losses = criterion(output, targets)
    print(f"Total loss: {loss.item():.4f}")
    print("Individual losses:")
    for name, value in losses.items():
        print(f"  {name}: {value:.4f}")
