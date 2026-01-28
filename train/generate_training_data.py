#!/usr/bin/env python3
"""
Neurlang Unified Training Data Generator
=========================================

This script generates comprehensive training data by combining ALL sources:
1. lib/ - Generated from Rust stdlib (source of truth)
2. examples/ - Hand-written application examples
3. Synthetic patterns - Generated for broad coverage
4. Extension patterns - Extension composition patterns (crypto, JSON, etc.)
5. HTTP patterns - HTTP protocol patterns (headers, status codes, etc.)
6. REST patterns - REST API patterns (CRUD, pagination, etc.)

Usage:
    # Generate ALL training data (default - includes everything)
    python train/generate_training_data.py train/training_data.jsonl

    # Optional: Filter by difficulty or category
    python train/generate_training_data.py train/training_data.jsonl --difficulty 4
    python train/generate_training_data.py train/training_data.jsonl --category network

Options:
    --difficulty N: Filter to include only samples with difficulty >= N
    --category CAT: Filter to include only samples matching category prefix
    --seed N: Random seed for reproducibility (default: 42)
"""

import os
import sys
import json
import struct
import subprocess
import re
import random
import argparse
from pathlib import Path
from typing import List, Dict, Any, Tuple, Optional, Set
from dataclasses import dataclass

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent))

from generators.extension_patterns import ExtensionPatternGenerator
from generators.http_patterns import HTTPPatternGenerator
from generators.diverse_patterns import generate_diverse_samples


# ============================================================================
# OPCODE AND INSTRUCTION DEFINITIONS
# ============================================================================

EXTENDED_OPCODES = {
    0x01,  # AluI
    0x03,  # Load
    0x04,  # Store
    0x06,  # Branch
    0x07,  # Call
    0x09,  # Jump
    0x14,  # File
    0x15,  # Net
    0x16,  # NetSetopt
    0x17,  # Io
    0x18,  # Time
    0x1C,  # Mov
    0x20,  # ExtCall
}


@dataclass
class Instruction:
    opcode: int
    mode: int
    rd: int
    rs1: int
    rs2: int
    has_imm: int
    imm: int

    def to_dict(self) -> Dict[str, int]:
        return {
            'valid': 1,
            'opcode': self.opcode,
            'mode': self.mode,
            'rd': self.rd,
            'rs1': self.rs1,
            'rs2': self.rs2,
            'has_imm': self.has_imm,
            'imm_bin': self.imm if self.has_imm else 0,
        }


# ============================================================================
# FILE-BASED GENERATION (lib/ and examples/)
# ============================================================================

def parse_example_file(filepath: Path) -> Optional[dict]:
    """Parse a .nl file and extract metadata and prompts."""
    try:
        content = filepath.read_text()
    except Exception:
        return None

    name = filepath.stem
    description = ""
    category = "uncategorized"
    difficulty = 1
    prompts = []
    params = {}
    is_server = False

    for line in content.split('\n'):
        line = line.strip()
        if not line.startswith(';'):
            continue
        text = line.lstrip(';').strip()

        if text.startswith('@name:'):
            name = text.split(':', 1)[1].strip()
        elif text.startswith('@description:'):
            description = text.split(':', 1)[1].strip()
        elif text.startswith('@category:'):
            category = text.split(':', 1)[1].strip()
        elif text.startswith('@difficulty:'):
            try:
                difficulty = int(text.split(':', 1)[1].strip())
            except:
                pass
        elif text.startswith('@prompt:'):
            prompt = text.split(':', 1)[1].strip()
            if prompt:
                prompts.append(prompt)
        elif text.startswith('@param:'):
            param_text = text.split(':', 1)[1].strip()
            match = re.match(r'(\w+)=(\w+)\s*"([^"]*)"', param_text)
            if match:
                param_name, reg, desc = match.groups()
                params[param_name] = (reg, desc)
        elif text.startswith('@server:'):
            is_server = text.split(':', 1)[1].strip().lower() == 'true'

    # Skip server examples (they can't be unit tested, use integration tests)
    if is_server:
        return None

    return {
        'name': name,
        'description': description,
        'category': category,
        'difficulty': difficulty,
        'prompts': prompts,
        'params': params,
        'path': filepath,
    }


def assemble_example(filepath: Path, nl_binary: Path) -> Optional[List[Instruction]]:
    """Assemble a .nl file and decode the instructions."""
    output_path = filepath.with_suffix('.nlb')

    try:
        result = subprocess.run(
            [str(nl_binary), 'asm', '-i', str(filepath), '-o', str(output_path)],
            capture_output=True, text=True, timeout=30,
        )
        if result.returncode != 0:
            return None

        binary_data = output_path.read_bytes()
        instructions = decode_binary(binary_data)
        output_path.unlink()
        return instructions
    except Exception:
        return None


def decode_binary(data: bytes) -> List[Instruction]:
    """Decode a Neurlang binary file into instructions."""
    if len(data) < 16 or data[0:4] != b'NRLG':
        return []

    code_len = struct.unpack('<I', data[8:12])[0]
    code = data[16:16+code_len]
    instructions = []
    offset = 0

    while offset < len(code):
        if offset + 4 > len(code):
            break
        word1 = struct.unpack('<I', code[offset:offset+4])[0]
        opcode = (word1 >> 26) & 0x3F
        rd = (word1 >> 21) & 0x1F
        rs1 = (word1 >> 16) & 0x1F
        rs2 = (word1 >> 11) & 0x1F
        mode = (word1 >> 8) & 0x7
        offset += 4

        has_imm = 0
        imm = 0
        if opcode in EXTENDED_OPCODES and offset + 4 <= len(code):
            imm = struct.unpack('<i', code[offset:offset+4])[0]
            has_imm = 1
            offset += 4

        instructions.append(Instruction(opcode, mode, rd, rs1, rs2, has_imm, imm))

    return instructions


def expand_prompts(prompts: List[str], params: Dict) -> List[str]:
    """Expand prompts with parameter variations."""
    expanded = []
    for prompt in prompts:
        placeholders = re.findall(r'\{(\w+)\}', prompt)
        if not placeholders:
            expanded.append(prompt)
        else:
            for _ in range(5):
                filled = prompt
                for ph in placeholders:
                    if ph in ('n', 'num', 'value', 'x'):
                        values = [1, 2, 3, 5, 7, 10, 12, 15, 20, 100]
                    elif ph in ('a', 'b', 'first', 'second'):
                        values = [2, 3, 5, 7, 10, 12, 15, 24, 48, 100]
                    else:
                        values = [1, 2, 3, 5, 10]
                    filled = filled.replace(f'{{{ph}}}', str(random.choice(values)))
                expanded.append(filled)
    return expanded


def generate_lib_data(
    lib_dir: Path,
    nl_binary: Path,
    samples_per_function: int,
    min_difficulty: int = 0,
    category_filter: Optional[str] = None
) -> Tuple[List[Dict], Set[str]]:
    """Generate training data from lib/ (generated from Rust stdlib).

    Returns samples and a set of function names seen (for deduplication).
    """
    samples = []
    seen_functions = set()

    nl_files = sorted(lib_dir.glob('**/*.nl'))

    for filepath in nl_files:
        spec = parse_example_file(filepath)
        if not spec:
            continue

        # Track function name for deduplication
        seen_functions.add(spec['name'].lower())

        # Apply filters
        if spec['difficulty'] < min_difficulty:
            continue
        if category_filter and not spec['category'].startswith(category_filter):
            continue

        # Skip if no prompts
        if not spec['prompts']:
            continue

        instructions = assemble_example(filepath, nl_binary)
        if not instructions:
            continue

        expanded = expand_prompts(spec['prompts'], spec['params'])
        padded = pad_instructions(instructions)

        for _ in range(samples_per_function):
            prompt = random.choice(expanded)
            samples.append({
                'context': prompt,
                'instructions': padded,
                'metadata': {
                    'source': 'lib',
                    'name': spec['name'],
                    'category': spec['category'],
                    'difficulty': spec['difficulty'],
                }
            })

    return samples, seen_functions


def generate_examples_data(
    examples_dir: Path,
    nl_binary: Path,
    samples_per_example: int,
    skip_functions: Set[str],
    min_difficulty: int = 0,
    category_filter: Optional[str] = None
) -> List[Dict]:
    """Generate training data from examples/, skipping functions already in lib/."""
    samples = []
    nl_files = sorted(examples_dir.glob('**/*.nl'))

    for filepath in nl_files:
        spec = parse_example_file(filepath)
        if not spec or not spec['prompts']:
            continue

        # Skip if this function is already covered by lib/
        if spec['name'].lower() in skip_functions:
            continue

        # Apply filters
        if spec['difficulty'] < min_difficulty:
            continue
        if category_filter and not spec['category'].startswith(category_filter):
            continue

        instructions = assemble_example(filepath, nl_binary)
        if not instructions:
            continue

        expanded = expand_prompts(spec['prompts'], spec['params'])
        padded = pad_instructions(instructions)

        for _ in range(samples_per_example):
            prompt = random.choice(expanded)
            samples.append({
                'context': prompt,
                'instructions': padded,
                'metadata': {
                    'source': 'examples',
                    'name': spec['name'],
                    'category': spec['category'],
                    'difficulty': spec['difficulty'],
                }
            })

    return samples


# ============================================================================
# SYNTHETIC DATA GENERATION
# ============================================================================

class SyntheticGenerator:
    """Generate synthetic training samples for broad coverage."""

    ALU_ADD = 0
    ALU_SUB = 1
    ALU_AND = 2
    ALU_OR = 3
    ALU_XOR = 4

    BR_EQ = 0
    BR_NE = 1
    BR_LT = 2
    BR_GE = 3
    BR_LE = 4
    BR_GT = 5

    MUL = 0
    DIV = 1
    MOD = 2

    def __init__(self):
        self.generators = [
            (30, self.gen_simple_arithmetic),
            (20, self.gen_loop_pattern),
            (15, self.gen_conditional_pattern),
            (10, self.gen_memory_pattern),
            (10, self.gen_function_pattern),
            (5, self.gen_fibonacci_variant),
            (5, self.gen_factorial_variant),
            (3, self.gen_gcd_variant),
            (2, self.gen_array_operation),
        ]

    def generate(self) -> Tuple[str, List[Instruction]]:
        weights = [g[0] for g in self.generators]
        total = sum(weights)
        weights = [w/total for w in weights]
        idx = random.choices(range(len(self.generators)), weights=weights)[0]
        return self.generators[idx][1]()

    def gen_simple_arithmetic(self) -> Tuple[str, List[Instruction]]:
        ops = [
            ('add', self.ALU_ADD, ['add', 'sum', 'plus']),
            ('subtract', self.ALU_SUB, ['subtract', 'minus', 'difference']),
            ('and', self.ALU_AND, ['bitwise and', 'and']),
            ('or', self.ALU_OR, ['bitwise or', 'or']),
            ('xor', self.ALU_XOR, ['xor', 'exclusive or']),
        ]
        op_name, mode, prompts = random.choice(ops)
        a, b = random.randint(1, 100), random.randint(1, 100)

        prompt_templates = [
            f"{random.choice(prompts)} {a} and {b}",
            f"compute {a} {op_name} {b}",
            f"calculate {a} {op_name} {b}",
            f"what is {a} {op_name} {b}",
        ]

        instructions = [
            Instruction(0x1C, 0, 0, 0, 0, 1, a),
            Instruction(0x1C, 0, 1, 0, 0, 1, b),
            Instruction(0x00, mode, 0, 0, 1, 0, 0),
            Instruction(0x1D, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompt_templates), instructions

    def gen_loop_pattern(self) -> Tuple[str, List[Instruction]]:
        n = random.randint(5, 20)
        prompts = [
            f"sum numbers from 1 to {n}",
            f"add 1 + 2 + ... + {n}",
            f"compute sum of first {n} integers",
            f"calculate 1+2+...+{n}",
        ]

        instructions = [
            Instruction(0x1C, 0, 0, 0, 0, 1, n),
            Instruction(0x1C, 0, 1, 0, 0, 1, 0),
            Instruction(0x00, self.ALU_ADD, 1, 1, 0, 0, 0),
            Instruction(0x01, self.ALU_SUB, 0, 0, 0, 1, 1),
            Instruction(0x06, self.BR_GT, 0, 0, 0, 1, -2),
            Instruction(0x1C, 0, 0, 1, 0, 0, 0),
            Instruction(0x1D, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions

    def gen_conditional_pattern(self) -> Tuple[str, List[Instruction]]:
        prompts = [
            "return absolute value of r0",
            "compute |r0|",
            "make r0 positive",
            "get absolute value",
        ]

        instructions = [
            Instruction(0x06, self.BR_GE, 0, 0, 0, 1, 2),
            Instruction(0x01, self.ALU_SUB, 0, 0, 0, 1, 0),
            Instruction(0x1D, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions

    def gen_memory_pattern(self) -> Tuple[str, List[Instruction]]:
        prompts = [
            "load value from memory and double it",
            "read memory and multiply by 2",
            "fetch value and duplicate",
        ]

        instructions = [
            Instruction(0x1C, 0, 1, 0, 0, 1, 0x100),
            Instruction(0x03, 3, 0, 1, 0, 1, 0),
            Instruction(0x00, self.ALU_ADD, 0, 0, 0, 0, 0),
            Instruction(0x1D, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions

    def gen_function_pattern(self) -> Tuple[str, List[Instruction]]:
        prompts = [
            "call helper function and return result",
            "invoke subroutine",
            "call function and use return value",
        ]

        instructions = [
            Instruction(0x07, 0, 0, 0, 0, 1, 3),
            Instruction(0x1D, 0, 0, 0, 0, 0, 0),
            Instruction(0x1C, 0, 0, 0, 0, 1, 42),
            Instruction(0x08, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions

    def gen_fibonacci_variant(self) -> Tuple[str, List[Instruction]]:
        n = random.randint(5, 15)
        prompts = [
            f"compute fibonacci({n})",
            f"calculate fib({n})",
            f"find the {n}th fibonacci number",
            f"fibonacci sequence element {n}",
        ]

        instructions = [
            Instruction(0x1C, 0, 0, 0, 0, 1, n),
            Instruction(0x1C, 0, 1, 0, 0, 1, 0),
            Instruction(0x1C, 0, 2, 0, 0, 1, 1),
            Instruction(0x06, self.BR_LE, 0, 0, 0, 1, 5),
            Instruction(0x00, self.ALU_ADD, 3, 1, 2, 0, 0),
            Instruction(0x1C, 0, 1, 2, 0, 0, 0),
            Instruction(0x1C, 0, 2, 3, 0, 0, 0),
            Instruction(0x01, self.ALU_SUB, 0, 0, 0, 1, 1),
            Instruction(0x06, self.BR_GT, 0, 0, 0, 1, -4),
            Instruction(0x1C, 0, 0, 1, 0, 0, 0),
            Instruction(0x1D, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions

    def gen_factorial_variant(self) -> Tuple[str, List[Instruction]]:
        n = random.randint(3, 10)
        prompts = [
            f"compute factorial of {n}",
            f"calculate {n}!",
            f"{n} factorial",
            f"multiply 1*2*...*{n}",
        ]

        instructions = [
            Instruction(0x1C, 0, 0, 0, 0, 1, n),
            Instruction(0x1C, 0, 1, 0, 0, 1, 1),
            Instruction(0x06, self.BR_EQ, 0, 0, 0, 1, 4),
            Instruction(0x02, self.MUL, 1, 1, 0, 0, 0),
            Instruction(0x01, self.ALU_SUB, 0, 0, 0, 1, 1),
            Instruction(0x06, self.BR_GT, 0, 0, 0, 1, -2),
            Instruction(0x1C, 0, 0, 1, 0, 0, 0),
            Instruction(0x1D, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions

    def gen_gcd_variant(self) -> Tuple[str, List[Instruction]]:
        a, b = random.randint(10, 100), random.randint(10, 100)
        prompts = [
            f"find GCD of {a} and {b}",
            f"gcd({a}, {b})",
            f"greatest common divisor of {a} and {b}",
            f"euclidean algorithm for {a}, {b}",
        ]

        instructions = [
            Instruction(0x1C, 0, 0, 0, 0, 1, a),
            Instruction(0x1C, 0, 1, 0, 0, 1, b),
            Instruction(0x06, self.BR_EQ, 1, 0, 0, 1, 4),
            Instruction(0x02, self.MOD, 2, 0, 1, 0, 0),
            Instruction(0x1C, 0, 0, 1, 0, 0, 0),
            Instruction(0x1C, 0, 1, 2, 0, 0, 0),
            Instruction(0x06, 7, 0, 0, 0, 1, -4),
            Instruction(0x1D, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions

    def gen_array_operation(self) -> Tuple[str, List[Instruction]]:
        size = random.randint(3, 10)
        prompts = [
            f"sum array of {size} elements",
            f"add up {size} numbers in memory",
            f"compute total of array with {size} items",
        ]

        instructions = [
            Instruction(0x1C, 0, 0, 0, 0, 1, 0),
            Instruction(0x1C, 0, 1, 0, 0, 1, 0x100),
            Instruction(0x1C, 0, 2, 0, 0, 1, size),
            Instruction(0x03, 2, 3, 1, 0, 1, 0),
            Instruction(0x00, self.ALU_ADD, 0, 0, 3, 0, 0),
            Instruction(0x01, self.ALU_ADD, 1, 1, 0, 1, 4),
            Instruction(0x01, self.ALU_SUB, 2, 2, 0, 1, 1),
            Instruction(0x06, self.BR_GT, 2, 0, 0, 1, -4),
            Instruction(0x1D, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions


def generate_synthetic_data(
    num_samples: int,
    min_difficulty: int = 0,
    category_filter: Optional[str] = None
) -> List[Dict]:
    """Generate synthetic training samples."""
    # Synthetic samples don't have difficulty/category, so skip if filters are strict
    if min_difficulty > 2 or (category_filter and category_filter not in ['algorithm', 'math', '']):
        return []

    generator = SyntheticGenerator()
    samples = []

    for _ in range(num_samples):
        prompt, instructions = generator.generate()
        samples.append({
            'context': prompt,
            'instructions': pad_instructions(instructions),
            'metadata': {
                'source': 'synthetic',
                'category': 'algorithm',
                'difficulty': 2,
            }
        })

    return samples


# ============================================================================
# UTILITY FUNCTIONS
# ============================================================================

def pad_instructions(instructions: List[Instruction], slots: int = 64) -> List[Dict]:
    """Pad instruction list to fixed slots."""
    result = [instr.to_dict() for instr in instructions]
    while len(result) < slots:
        result.append({
            'valid': 0, 'opcode': 0, 'mode': 0, 'rd': 0,
            'rs1': 0, 'rs2': 0, 'has_imm': 0, 'imm_bin': 0
        })
    return result[:slots]


def filter_samples(samples: List[Dict], min_difficulty: int, category_filter: Optional[str]) -> List[Dict]:
    """Filter samples by difficulty and category."""
    if min_difficulty == 0 and category_filter is None:
        return samples

    filtered = []
    for sample in samples:
        meta = sample.get('metadata', {})
        difficulty = meta.get('difficulty', 1)
        category = meta.get('category', '')

        if difficulty < min_difficulty:
            continue
        if category_filter and not category.startswith(category_filter):
            continue
        filtered.append(sample)

    return filtered


def main():
    parser = argparse.ArgumentParser(
        description='Generate unified Neurlang training data'
    )
    parser.add_argument('output', help='Output JSONL file')
    parser.add_argument('--lib-dir', default='lib',
                       help='Lib directory (generated from Rust stdlib)')
    parser.add_argument('--examples-dir', default='examples',
                       help='Examples directory')
    parser.add_argument('--nl-binary', default='./target/release/nl',
                       help='Path to nl binary')
    parser.add_argument('--lib-samples', type=int, default=1000,
                       help='Samples per lib function')
    parser.add_argument('--examples-samples', type=int, default=1000,
                       help='Samples per example file')
    parser.add_argument('--synthetic-samples', type=int, default=50000,
                       help='Number of synthetic samples')
    parser.add_argument('--extension-samples', type=int, default=30000,
                       help='Number of extension pattern samples')
    parser.add_argument('--http-samples', type=int, default=20000,
                       help='Number of HTTP pattern samples')
    parser.add_argument('--diverse-samples', type=int, default=500000,
                       help='Number of diverse pattern samples (register/algo variations)')
    parser.add_argument('--difficulty', type=int, default=0,
                       help='Minimum difficulty level (1-5)')
    parser.add_argument('--category', type=str, default=None,
                       help='Filter by category prefix')
    parser.add_argument('--seed', type=int, default=42,
                       help='Random seed')

    args = parser.parse_args()
    random.seed(args.seed)

    lib_dir = Path(args.lib_dir)
    examples_dir = Path(args.examples_dir)
    nl_binary = Path(args.nl_binary)
    output_path = Path(args.output)

    print("=" * 70)
    print("Neurlang Unified Training Data Generator")
    print("=" * 70)

    if args.difficulty > 0:
        print(f"Filter: difficulty >= {args.difficulty}")
    if args.category:
        print(f"Filter: category starts with '{args.category}'")

    all_data = []

    # Step 1: Generate from lib/ (source of truth for stdlib)
    print(f"\n1. Generating from lib/ ({args.lib_samples} samples per function)...")
    lib_data = []
    seen_functions = set()
    if lib_dir.exists() and nl_binary.exists():
        lib_data, seen_functions = generate_lib_data(
            lib_dir, nl_binary, args.lib_samples,
            args.difficulty, args.category
        )
        print(f"   Generated {len(lib_data):,} samples from {len(seen_functions)} lib functions")
    else:
        print("   Warning: lib directory or nl binary not found, skipping")

    all_data.extend(lib_data)

    # Step 2: Generate from examples/ (composition patterns, skipping duplicates)
    print(f"\n2. Generating from examples/ ({args.examples_samples} samples per example)...")
    print(f"   Skipping {len(seen_functions)} functions already in lib/")
    examples_data = []
    if examples_dir.exists() and nl_binary.exists():
        examples_data = generate_examples_data(
            examples_dir, nl_binary, args.examples_samples, seen_functions,
            args.difficulty, args.category
        )
        print(f"   Generated {len(examples_data):,} samples from examples")
    else:
        print("   Warning: examples directory or nl binary not found, skipping")

    all_data.extend(examples_data)

    # Step 3: Generate synthetic data
    print(f"\n3. Generating synthetic data ({args.synthetic_samples:,} samples)...")
    synthetic_data = generate_synthetic_data(
        args.synthetic_samples, args.difficulty, args.category
    )
    print(f"   Generated {len(synthetic_data):,} synthetic samples")
    all_data.extend(synthetic_data)

    # Step 4: Generate extension pattern data
    print(f"\n4. Generating extension patterns ({args.extension_samples:,} samples)...")
    ext_generator = ExtensionPatternGenerator()
    extension_data = ext_generator.generate_samples(args.extension_samples)
    extension_data = filter_samples(extension_data, args.difficulty, args.category)
    print(f"   Generated {len(extension_data):,} extension pattern samples")
    all_data.extend(extension_data)

    # Step 5: Generate HTTP pattern data
    print(f"\n5. Generating HTTP patterns ({args.http_samples:,} samples)...")
    http_generator = HTTPPatternGenerator()
    http_data = http_generator.generate_samples(args.http_samples)
    http_data = filter_samples(http_data, args.difficulty, args.category)
    print(f"   Generated {len(http_data):,} HTTP pattern samples")
    all_data.extend(http_data)

    # Step 6: Generate diverse patterns (register allocation, algo variants)
    print(f"\n6. Generating diverse patterns ({args.diverse_samples:,} samples)...")
    diverse_data = generate_diverse_samples(args.diverse_samples, seed=args.seed)
    diverse_data = filter_samples(diverse_data, args.difficulty, args.category)
    print(f"   Generated {len(diverse_data):,} diverse pattern samples")
    all_data.extend(diverse_data)

    # Shuffle all data
    print("\n7. Shuffling all samples...")
    random.shuffle(all_data)

    # Write output
    print(f"\n8. Writing to {output_path}...")
    with open(output_path, 'w') as f:
        for sample in all_data:
            f.write(json.dumps(sample) + '\n')

    # Summary
    print("\n" + "=" * 70)
    print("Summary:")
    print(f"  Lib-based samples:       {len(lib_data):>10,}")
    print(f"  Examples-based samples:  {len(examples_data):>10,}")
    print(f"  Synthetic samples:       {len(synthetic_data):>10,}")
    print(f"  Extension patterns:      {len(extension_data):>10,}")
    print(f"  HTTP patterns:           {len(http_data):>10,}")
    print(f"  Diverse patterns:        {len(diverse_data):>10,}")
    print(f"  {'─' * 30}")
    print(f"  Total samples:           {len(all_data):>10,}")
    print(f"  Output file:             {output_path}")
    print("=" * 70)

    # Category distribution
    print("\nCategory distribution:")
    categories = {}
    for sample in all_data:
        cat = sample.get('metadata', {}).get('category', 'unknown')
        cat_prefix = cat.split('/')[0] if cat else 'unknown'
        categories[cat_prefix] = categories.get(cat_prefix, 0) + 1

    for cat, count in sorted(categories.items(), key=lambda x: -x[1])[:15]:
        pct = count * 100 / len(all_data)
        print(f"  {cat:20s}: {count:>8,} ({pct:5.1f}%)")

    # Source distribution
    print("\nSource distribution:")
    sources = {}
    for sample in all_data:
        src = sample.get('metadata', {}).get('source', 'unknown')
        sources[src] = sources.get(src, 0) + 1

    for src, count in sorted(sources.items(), key=lambda x: -x[1]):
        pct = count * 100 / len(all_data)
        print(f"  {src:20s}: {count:>8,} ({pct:5.1f}%)")

    # Verify output
    print("\nVerifying output format...")
    with open(output_path) as f:
        sample = json.loads(f.readline())
        print(f"  Sample context: '{sample['context'][:50]}...'")
        print(f"  Instructions: {len(sample['instructions'])} slots")
        valid = sum(1 for i in sample['instructions'] if i['valid'])
        print(f"  Valid instructions: {valid}")


if __name__ == '__main__':
    main()
