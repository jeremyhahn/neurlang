#!/usr/bin/env python3
"""
Export trained parallel model to ONNX format for Rust inference.

Usage:
    python train/parallel/export_onnx.py --model models/model.pt --output models/parallel.onnx
"""

import argparse
import sys
from pathlib import Path

import torch

sys.path.insert(0, str(Path(__file__).parent))
from model import ParallelInstructionModel, LightParallelModel


def export_to_onnx(model_path: str, output_path: str, max_seq_len: int = 256, light: bool = False):
    """Export parallel model to ONNX format."""
    print(f"Loading model from {model_path}...")

    # Load checkpoint - supports both full checkpoint and state_dict only
    checkpoint = torch.load(model_path, map_location="cpu", weights_only=False)

    # Handle both checkpoint formats
    if isinstance(checkpoint, dict):
        if 'model_state_dict' in checkpoint:
            state_dict = checkpoint['model_state_dict']
        elif 'model' in checkpoint:
            state_dict = checkpoint['model']
        else:
            state_dict = checkpoint
    else:
        state_dict = checkpoint

    # Create model
    if light:
        model = LightParallelModel()
        print("Using LightParallelModel")
    else:
        model = ParallelInstructionModel(max_seq_len=max_seq_len)
        print("Using ParallelInstructionModel")

    # Load weights
    model.load_state_dict(state_dict)
    model.eval()

    print(f"Model parameters: {model.count_parameters():,}")

    # Create dummy input
    batch_size = 1
    seq_len = max_seq_len
    dummy_input = torch.randint(0, 256, (batch_size, seq_len), dtype=torch.long)

    print(f"Exporting to ONNX with input shape: {dummy_input.shape}")

    # Run inference to get output names
    with torch.no_grad():
        output = model(dummy_input)
        output_names = list(output.keys())
        print(f"Output heads: {output_names}")

    # Create wrapper that returns tuple instead of dict (required for ONNX)
    class ModelWrapper(torch.nn.Module):
        def __init__(self, model):
            super().__init__()
            self.model = model

        def forward(self, x):
            output = self.model(x)
            # Return in fixed order
            return (
                output['valid'],
                output['opcode'],
                output['mode'],
                output['rd'],
                output['rs1'],
                output['rs2'],
                output['has_imm'],
                output['imm_bin'],
            )

    wrapped_model = ModelWrapper(model)
    wrapped_model.eval()

    # Export
    torch.onnx.export(
        wrapped_model,
        dummy_input,
        output_path,
        export_params=True,
        opset_version=14,
        do_constant_folding=True,
        input_names=['input_ids'],
        output_names=['valid', 'opcode', 'mode', 'rd', 'rs1', 'rs2', 'has_imm', 'imm_bin'],
        dynamic_axes={
            'input_ids': {0: 'batch_size'},
            'valid': {0: 'batch_size'},
            'opcode': {0: 'batch_size'},
            'mode': {0: 'batch_size'},
            'rd': {0: 'batch_size'},
            'rs1': {0: 'batch_size'},
            'rs2': {0: 'batch_size'},
            'has_imm': {0: 'batch_size'},
            'imm_bin': {0: 'batch_size'},
        }
    )

    print(f"Exported to {output_path}")

    # Verify
    import onnx
    onnx_model = onnx.load(output_path)
    onnx.checker.check_model(onnx_model)
    print("ONNX model verified successfully!")

    # Print model size
    import os
    size_mb = os.path.getsize(output_path) / (1024 * 1024)
    print(f"Model size: {size_mb:.2f} MB")

    return output_path


def main():
    parser = argparse.ArgumentParser(description="Export parallel model to ONNX")
    parser.add_argument("--model", type=str, default="models/model.pt", help="Input PyTorch model")
    parser.add_argument("--output", type=str, default="models/parallel.onnx", help="Output ONNX model")
    parser.add_argument("--max-seq-len", type=int, default=256, help="Max sequence length")
    parser.add_argument("--light", action="store_true", help="Use lightweight model")

    args = parser.parse_args()
    export_to_onnx(args.model, args.output, args.max_seq_len, args.light)


if __name__ == "__main__":
    main()
