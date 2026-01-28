#!/usr/bin/env python3
"""
Training Script for Parallel Instruction Prediction Model

Trains on parallel format data where each example contains:
- context: The prompt/description
- instructions: List of {valid, opcode, mode, rd, rs1, rs2, has_imm, imm_bin}
- test_cases: For verification

Usage:
    python train.py --data training_data_parallel.jsonl --epochs 50
"""

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from collections import defaultdict

import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import Dataset, DataLoader, random_split
from tqdm import tqdm

from parallel.model import (
    ParallelInstructionModel,
    LightParallelModel,
    ParallelInstructionLoss,
    NUM_SLOTS,
)


class ParallelDataset(Dataset):
    """Dataset for parallel instruction prediction training."""

    def __init__(
        self,
        data_path: str,
        max_seq_len: int = 256,
        num_slots: int = NUM_SLOTS,
        max_samples: Optional[int] = None,
    ):
        self.max_seq_len = max_seq_len
        self.num_slots = num_slots
        self.samples = []
        self.categories = defaultdict(int)

        with open(data_path, 'r') as f:
            for i, line in enumerate(f):
                if max_samples and i >= max_samples:
                    break

                try:
                    item = json.loads(line)
                    # Support both 'context' (new format) and 'prompt' (old format)
                    text = item.get('context') or item.get('prompt', '')
                    instructions = item.get('instructions', [])
                    category = item.get('category', 'unknown')

                    if not text or not instructions:
                        continue

                    self.samples.append({
                        'text': text,
                        'instructions': instructions,
                        'category': category,
                    })
                    self.categories[category] += 1

                except (json.JSONDecodeError, KeyError) as e:
                    continue

        print(f"Loaded {len(self.samples)} samples from {data_path}")
        print("Category distribution:")
        for cat, count in sorted(self.categories.items(), key=lambda x: -x[1]):
            print(f"  {cat}: {count} ({count/len(self.samples):.1%})")

    def __len__(self) -> int:
        return len(self.samples)

    def __getitem__(self, idx: int) -> Dict[str, torch.Tensor]:
        sample = self.samples[idx]

        # Tokenize text (simple byte-level)
        text_bytes = sample['text'].encode('utf-8')[:self.max_seq_len]
        tokens = list(text_bytes)

        # Pad to max_seq_len
        if len(tokens) < self.max_seq_len:
            tokens = tokens + [256] * (self.max_seq_len - len(tokens))  # 256 = padding

        # Process instructions (pad to num_slots)
        instructions = sample['instructions'][:self.num_slots]

        # Initialize targets with zeros (padding)
        targets = {
            'valid': [0] * self.num_slots,
            'opcode': [0] * self.num_slots,
            'mode': [0] * self.num_slots,
            'rd': [0] * self.num_slots,
            'rs1': [0] * self.num_slots,
            'rs2': [0] * self.num_slots,
            'has_imm': [0] * self.num_slots,
            'imm_bin': [0] * self.num_slots,
        }

        # Fill in actual instructions
        for i, instr in enumerate(instructions):
            if i >= self.num_slots:
                break
            targets['valid'][i] = instr.get('valid', 0)
            targets['opcode'][i] = min(instr.get('opcode', 0), 32)  # Clamp to valid range
            targets['mode'][i] = min(instr.get('mode', 0), 7)
            targets['rd'][i] = min(instr.get('rd', 0), 31)
            targets['rs1'][i] = min(instr.get('rs1', 0), 31)
            targets['rs2'][i] = min(instr.get('rs2', 0), 31)
            targets['has_imm'][i] = instr.get('has_imm', 0)
            # Convert to unsigned: negative values (backward branches) become 256+val
            imm = instr.get('imm_bin', 0)
            targets['imm_bin'][i] = imm % 256  # Python % handles negatives: -24 % 256 = 232

        return {
            'tokens': torch.tensor(tokens, dtype=torch.long),
            'valid': torch.tensor(targets['valid'], dtype=torch.long),
            'opcode': torch.tensor(targets['opcode'], dtype=torch.long),
            'mode': torch.tensor(targets['mode'], dtype=torch.long),
            'rd': torch.tensor(targets['rd'], dtype=torch.long),
            'rs1': torch.tensor(targets['rs1'], dtype=torch.long),
            'rs2': torch.tensor(targets['rs2'], dtype=torch.long),
            'has_imm': torch.tensor(targets['has_imm'], dtype=torch.long),
            'imm_bin': torch.tensor(targets['imm_bin'], dtype=torch.long),
            'category': sample['category'],
        }


def collate_fn(batch: List[Dict]) -> Dict[str, torch.Tensor]:
    """Custom collate function to handle dict batches."""
    result = {}
    for key in batch[0].keys():
        if key == 'category':
            result[key] = [item[key] for item in batch]
        else:
            result[key] = torch.stack([item[key] for item in batch])
    return result


def train_epoch(
    model: nn.Module,
    loader: DataLoader,
    criterion: ParallelInstructionLoss,
    optimizer: optim.Optimizer,
    device: torch.device,
    scaler: Optional[torch.amp.GradScaler] = None,
) -> Dict[str, float]:
    """Train for one epoch."""
    model.train()

    total_losses = defaultdict(float)
    num_batches = 0

    pbar = tqdm(loader, desc="Training")
    for batch in pbar:
        tokens = batch['tokens'].to(device)

        # Prepare targets dict
        targets = {
            'valid': batch['valid'].to(device),
            'opcode': batch['opcode'].to(device),
            'mode': batch['mode'].to(device),
            'rd': batch['rd'].to(device),
            'rs1': batch['rs1'].to(device),
            'rs2': batch['rs2'].to(device),
            'has_imm': batch['has_imm'].to(device),
            'imm_bin': batch['imm_bin'].to(device),
        }

        optimizer.zero_grad()

        # Forward pass (with optional mixed precision)
        if scaler is not None:
            with torch.amp.autocast('cuda'):
                logits = model(tokens)
                loss, losses = criterion(logits, targets)
            scaler.scale(loss).backward()
            scaler.unscale_(optimizer)
            torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            scaler.step(optimizer)
            scaler.update()
        else:
            logits = model(tokens)
            loss, losses = criterion(logits, targets)
            loss.backward()
            torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            optimizer.step()

        # Accumulate losses
        for k, v in losses.items():
            total_losses[k] += v
        num_batches += 1

        pbar.set_postfix({
            'loss': f"{losses['total']:.4f}",
            'op': f"{losses['opcode']:.4f}",
        })

    return {k: v / num_batches for k, v in total_losses.items()}


def evaluate(
    model: nn.Module,
    loader: DataLoader,
    criterion: ParallelInstructionLoss,
    device: torch.device,
) -> Dict[str, float]:
    """Evaluate model."""
    model.eval()

    total_losses = defaultdict(float)
    correct_valid = 0
    correct_opcode = 0
    total_valid = 0
    total_slots = 0

    with torch.no_grad():
        for batch in tqdm(loader, desc="Evaluating"):
            tokens = batch['tokens'].to(device)

            targets = {
                'valid': batch['valid'].to(device),
                'opcode': batch['opcode'].to(device),
                'mode': batch['mode'].to(device),
                'rd': batch['rd'].to(device),
                'rs1': batch['rs1'].to(device),
                'rs2': batch['rs2'].to(device),
                'has_imm': batch['has_imm'].to(device),
                'imm_bin': batch['imm_bin'].to(device),
            }

            # Forward pass
            logits = model(tokens)
            _, losses = criterion(logits, targets)

            # Accumulate losses
            for k, v in losses.items():
                total_losses[k] += v

            # Compute accuracy
            valid_pred = torch.argmax(logits['valid'], dim=-1)
            opcode_pred = torch.argmax(logits['opcode'], dim=-1)

            # Valid prediction accuracy (all slots)
            correct_valid += (valid_pred == targets['valid']).sum().item()
            total_slots += tokens.size(0) * valid_pred.size(1)

            # Opcode accuracy (only for valid slots)
            valid_mask = targets['valid'] == 1
            if valid_mask.sum() > 0:
                correct_opcode += ((opcode_pred == targets['opcode']) & valid_mask).sum().item()
                total_valid += valid_mask.sum().item()

    num_batches = len(loader)
    avg_losses = {k: v / num_batches for k, v in total_losses.items()}
    avg_losses['valid_acc'] = correct_valid / max(1, total_slots)
    avg_losses['opcode_acc'] = correct_opcode / max(1, total_valid)

    return avg_losses


def main():
    parser = argparse.ArgumentParser(description="Train parallel instruction model")
    parser.add_argument('--data', type=str, required=True, help="Training data JSONL file")
    parser.add_argument('--output', type=str, default='model.pt', help="Output model path")
    parser.add_argument('--epochs', type=int, default=50, help="Number of epochs")
    parser.add_argument('--batch-size', type=int, default=64, help="Batch size")
    parser.add_argument('--lr', type=float, default=1e-3, help="Learning rate")
    parser.add_argument('--val-split', type=float, default=0.1, help="Validation split")
    parser.add_argument('--save-dir', type=str, default=None, help="Save directory (derived from --output if not set)")
    parser.add_argument('--light', action='store_true', help="Use lightweight model")
    parser.add_argument('--device', type=str, default='cuda' if torch.cuda.is_available() else 'cpu')
    parser.add_argument('--max-samples', type=int, default=None, help="Max training samples")
    parser.add_argument('--max-seq-len', type=int, default=256, help="Max sequence length")
    parser.add_argument('--patience', type=int, default=5, help="Early stopping patience")
    parser.add_argument('--mixed-precision', action='store_true', help="Use mixed precision training")
    parser.add_argument('--num-workers', type=int, default=4, help="DataLoader workers")
    parser.add_argument('--checkpoint', type=str, default=None, help="Checkpoint to resume from (for fine-tuning)")
    args = parser.parse_args()

    # Derive save_dir from output path if not specified
    if args.save_dir is None:
        args.save_dir = str(Path(args.output).parent)
        if not args.save_dir or args.save_dir == '.':
            args.save_dir = 'checkpoints'

    device = torch.device(args.device)
    print(f"Using device: {device}")

    # Create save directory
    save_dir = Path(args.save_dir)
    save_dir.mkdir(exist_ok=True)

    # Load data
    dataset = ParallelDataset(
        args.data,
        max_seq_len=args.max_seq_len,
        max_samples=args.max_samples,
    )

    # Split into train/val
    val_size = int(len(dataset) * args.val_split)
    train_size = len(dataset) - val_size
    train_dataset, val_dataset = random_split(dataset, [train_size, val_size])

    train_loader = DataLoader(
        train_dataset,
        batch_size=args.batch_size,
        shuffle=True,
        num_workers=args.num_workers,
        pin_memory=True if args.device == 'cuda' else False,
        collate_fn=collate_fn,
    )
    val_loader = DataLoader(
        val_dataset,
        batch_size=args.batch_size,
        shuffle=False,
        num_workers=args.num_workers,
        pin_memory=True if args.device == 'cuda' else False,
        collate_fn=collate_fn,
    )

    print(f"\nTrain samples: {len(train_dataset)}, Val samples: {len(val_dataset)}")

    # Create model
    if args.light:
        model = LightParallelModel()
    else:
        model = ParallelInstructionModel(max_seq_len=args.max_seq_len)

    # Load checkpoint for fine-tuning if provided
    start_epoch = 0
    if args.checkpoint:
        print(f"Loading checkpoint from {args.checkpoint}...")
        checkpoint = torch.load(args.checkpoint, map_location=device, weights_only=False)
        if isinstance(checkpoint, dict) and 'model_state_dict' in checkpoint:
            model.load_state_dict(checkpoint['model_state_dict'])
            start_epoch = checkpoint.get('epoch', 0)
            print(f"  Resuming from epoch {start_epoch}")
        else:
            # Direct state dict
            model.load_state_dict(checkpoint)
        print(f"  Checkpoint loaded successfully")

    model = model.to(device)
    print(f"Model parameters: {model.count_parameters():,}")

    # Loss and optimizer
    criterion = ParallelInstructionLoss()
    optimizer = optim.AdamW(model.parameters(), lr=args.lr, weight_decay=0.01)
    scheduler = optim.lr_scheduler.CosineAnnealingLR(optimizer, T_max=args.epochs)

    # Mixed precision scaler
    scaler = torch.amp.GradScaler('cuda') if args.mixed_precision and args.device == 'cuda' else None

    # Training loop
    best_val_loss = float('inf')
    best_opcode_acc = 0.0
    patience_counter = 0

    for epoch in range(start_epoch, args.epochs):
        print(f"\nEpoch {epoch + 1}/{args.epochs}")

        # Train
        train_losses = train_epoch(model, train_loader, criterion, optimizer, device, scaler)
        print(f"Train - Loss: {train_losses['total']:.4f}, "
              f"Opcode: {train_losses['opcode']:.4f}, "
              f"Valid: {train_losses['valid']:.4f}")

        # Evaluate
        val_losses = evaluate(model, val_loader, criterion, device)
        print(f"Val - Loss: {val_losses['total']:.4f}, "
              f"Valid Acc: {val_losses['valid_acc']:.4f}, "
              f"Opcode Acc: {val_losses['opcode_acc']:.4f}")

        # Save best model
        if val_losses['opcode_acc'] > best_opcode_acc:
            best_opcode_acc = val_losses['opcode_acc']
            best_val_loss = val_losses['total']
            patience_counter = 0

            checkpoint = {
                'epoch': epoch,
                'model_state_dict': model.state_dict(),
                'optimizer_state_dict': optimizer.state_dict(),
                'val_loss': best_val_loss,
                'opcode_acc': best_opcode_acc,
            }
            torch.save(checkpoint, save_dir / 'best_model.pt')
            print(f"  -> Saved best model (opcode_acc: {best_opcode_acc:.4f})")
        else:
            patience_counter += 1
            if patience_counter >= args.patience:
                print(f"Early stopping at epoch {epoch + 1}")
                break

        scheduler.step()

    # Save final model to output path
    output_path = Path(args.output)
    torch.save(model.state_dict(), output_path.with_suffix('.pt'))
    print(f"\nTraining complete. Best opcode accuracy: {best_opcode_acc:.4f}")
    print(f"Model saved to: {output_path.with_suffix('.pt')}")

    # Save config
    config = {
        'model_type': 'light' if args.light else 'full',
        'max_seq_len': args.max_seq_len,
        'best_opcode_acc': best_opcode_acc,
        'best_val_loss': best_val_loss,
        'epochs_trained': epoch + 1,
    }
    config_path = output_path.with_suffix('.config.json')
    with open(config_path, 'w') as f:
        json.dump(config, f, indent=2)
    print(f"Config saved to: {config_path}")


if __name__ == "__main__":
    main()
