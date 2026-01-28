#!/usr/bin/env python3
"""
Test inference with the trained parallel model.

Usage:
    python train/parallel/test_inference.py --model models/parallel.onnx
    python train/parallel/test_inference.py --pytorch models/model.pt
"""

import argparse
import sys
from pathlib import Path
import numpy as np

sys.path.insert(0, str(Path(__file__).parent))
from model import ParallelInstructionModel, NUM_SLOTS, NUM_OPCODES

# Opcode names for display
OPCODE_NAMES = [
    "NOP", "MOV", "LD", "ST", "ALU", "ALUI", "MULDIV", "MULHI",
    "BRANCH", "JUMP", "CALL", "RET", "COMPARE", "SETCC", "SEXT", "ZEXT",
    "SHIFT", "SHIFTI", "BITOP", "BITOPI", "LEA", "SPECIAL", "FLOAT", "FLOATI",
    "SIMD", "SIMDI", "DEBUG", "HINT", "SYS", "EXT.CALL", "EXT.LOAD", "EXT.STORE",
    "HALT"
]

# Register names
REGISTER_NAMES = [f"r{i}" for i in range(31)] + ["zero"]


def tokenize(text: str, max_len: int = 256) -> np.ndarray:
    """Convert text to token IDs (byte-level)."""
    bytes_data = text.encode('utf-8')[:max_len]
    tokens = list(bytes_data)

    # Pad to max_len
    if len(tokens) < max_len:
        tokens = tokens + [256] * (max_len - len(tokens))

    return np.array([tokens], dtype=np.int64)


def decode_instruction(valid, opcode, mode, rd, rs1, rs2, has_imm, imm_bin) -> str:
    """Decode instruction to human-readable format."""
    if not valid:
        return None

    opname = OPCODE_NAMES[opcode] if opcode < len(OPCODE_NAMES) else f"OP{opcode}"
    rd_name = REGISTER_NAMES[rd] if rd < len(REGISTER_NAMES) else f"r{rd}"
    rs1_name = REGISTER_NAMES[rs1] if rs1 < len(REGISTER_NAMES) else f"r{rs1}"
    rs2_name = REGISTER_NAMES[rs2] if rs2 < len(REGISTER_NAMES) else f"r{rs2}"

    if has_imm:
        # Decode immediate from bin (centered around 128 = 0)
        imm = imm_bin - 128
        return f"{opname}.{mode} {rd_name}, {rs1_name}, #{imm}"
    else:
        return f"{opname}.{mode} {rd_name}, {rs1_name}, {rs2_name}"


def test_pytorch_inference(model_path: str, prompts: list):
    """Test inference with PyTorch model."""
    import torch

    print(f"Loading PyTorch model from {model_path}...")
    checkpoint = torch.load(model_path, map_location="cpu", weights_only=False)

    if isinstance(checkpoint, dict) and 'model_state_dict' in checkpoint:
        state_dict = checkpoint['model_state_dict']
    else:
        state_dict = checkpoint

    model = ParallelInstructionModel(max_seq_len=256)
    model.load_state_dict(state_dict)
    model.eval()

    print(f"Model parameters: {model.count_parameters():,}")
    print()

    for prompt in prompts:
        print(f"Prompt: '{prompt}'")
        print("-" * 50)

        tokens = torch.tensor(tokenize(prompt), dtype=torch.long)

        with torch.no_grad():
            output = model.predict(tokens)

        # Decode instructions
        valid = output['valid'][0].numpy()
        opcode = output['opcode'][0].numpy()
        mode = output['mode'][0].numpy()
        rd = output['rd'][0].numpy()
        rs1 = output['rs1'][0].numpy()
        rs2 = output['rs2'][0].numpy()
        has_imm = output['has_imm'][0].numpy()
        imm_bin = output['imm_bin'][0].numpy()

        instructions = []
        for i in range(NUM_SLOTS):
            instr = decode_instruction(
                valid[i], opcode[i], mode[i], rd[i], rs1[i], rs2[i], has_imm[i], imm_bin[i]
            )
            if instr:
                instructions.append(instr)

        if instructions:
            print(f"Generated {len(instructions)} instructions:")
            for i, instr in enumerate(instructions):
                print(f"  {i:02d}: {instr}")
        else:
            print("No valid instructions generated")

        print()


def test_onnx_inference(model_path: str, prompts: list):
    """Test inference with ONNX model."""
    import onnxruntime as ort

    print(f"Loading ONNX model from {model_path}...")
    session = ort.InferenceSession(model_path, providers=['CPUExecutionProvider'])

    # Print model info
    inputs = session.get_inputs()
    outputs = session.get_outputs()
    print(f"Inputs: {[i.name for i in inputs]}")
    print(f"Outputs: {[o.name for o in outputs]}")
    print()

    for prompt in prompts:
        print(f"Prompt: '{prompt}'")
        print("-" * 50)

        tokens = tokenize(prompt)

        # Run inference
        result = session.run(None, {'input_ids': tokens})

        # Outputs are in order: valid, opcode, mode, rd, rs1, rs2, has_imm, imm_bin
        valid_logits, opcode_logits, mode_logits, rd_logits, rs1_logits, rs2_logits, has_imm_logits, imm_bin_logits = result

        # Get predictions (argmax)
        valid = np.argmax(valid_logits[0], axis=-1)
        opcode = np.argmax(opcode_logits[0], axis=-1)
        mode = np.argmax(mode_logits[0], axis=-1)
        rd = np.argmax(rd_logits[0], axis=-1)
        rs1 = np.argmax(rs1_logits[0], axis=-1)
        rs2 = np.argmax(rs2_logits[0], axis=-1)
        has_imm = np.argmax(has_imm_logits[0], axis=-1)
        imm_bin = np.argmax(imm_bin_logits[0], axis=-1)

        # Decode instructions
        instructions = []
        for i in range(NUM_SLOTS):
            instr = decode_instruction(
                valid[i], opcode[i], mode[i], rd[i], rs1[i], rs2[i], has_imm[i], imm_bin[i]
            )
            if instr:
                instructions.append(instr)

        if instructions:
            print(f"Generated {len(instructions)} instructions:")
            for i, instr in enumerate(instructions):
                print(f"  {i:02d}: {instr}")
        else:
            print("No valid instructions generated")

        print()


def main():
    parser = argparse.ArgumentParser(description="Test parallel model inference")
    parser.add_argument("--model", type=str, default="models/parallel.onnx", help="ONNX model path")
    parser.add_argument("--pytorch", type=str, default=None, help="PyTorch model path (instead of ONNX)")
    args = parser.parse_args()

    # Test prompts
    prompts = [
        "compute the factorial of 5",
        "add two numbers together",
        "multiply 10 by 20",
        "calculate fibonacci of 10",
        "implement a loop that counts from 1 to 10",
        "store value in memory",
        "load data from address",
    ]

    if args.pytorch:
        test_pytorch_inference(args.pytorch, prompts)
    else:
        test_onnx_inference(args.model, prompts)


if __name__ == "__main__":
    main()
