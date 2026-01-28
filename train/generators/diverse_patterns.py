#!/usr/bin/env python3
"""
Diverse Pattern Generator for Neurlang Training Data
=====================================================

Generates truly diverse code patterns with:
1. Register allocation variations
2. Instruction reordering
3. Multiple algorithmic implementations
4. Constant embedding variations
5. Control flow alternatives
6. Extension call compositions
"""

import random
from typing import List, Tuple, Dict, Any, Optional
from dataclasses import dataclass

# Opcodes
OP_ALU = 0x00
OP_ALUI = 0x01
OP_MULDIV = 0x02
OP_LOAD = 0x03
OP_STORE = 0x04
OP_BRANCH = 0x06
OP_CALL = 0x07
OP_RET = 0x08
OP_JUMP = 0x09
OP_MOV = 0x1C
OP_HALT = 0x1D
OP_EXT_CALL = 0x20

# ALU modes
ALU_ADD = 0
ALU_SUB = 1
ALU_AND = 2
ALU_OR = 3
ALU_XOR = 4
ALU_SHL = 5
ALU_SHR = 6
ALU_SAR = 7

# Branch modes
BR_EQ = 0
BR_NE = 1
BR_LT = 2
BR_GE = 3
BR_LE = 4
BR_GT = 5

# MulDiv modes
MD_MUL = 0
MD_DIV = 1
MD_MOD = 2

# Load/Store modes
MEM_BYTE = 0
MEM_WORD = 1
MEM_DWORD = 2
MEM_QWORD = 3


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


class RegisterAllocator:
    """Allocates registers with randomization for diversity."""

    def __init__(self, avoid_regs: List[int] = None):
        self.avoid = set(avoid_regs or [])
        self.available = [r for r in range(32) if r not in self.avoid and r != 31]  # r31 is zero
        random.shuffle(self.available)
        self.allocated = {}
        self.next_idx = 0

    def alloc(self, name: str) -> int:
        """Allocate a register for a named variable."""
        if name in self.allocated:
            return self.allocated[name]
        if self.next_idx >= len(self.available):
            # Reuse registers if we run out
            self.next_idx = 0
        reg = self.available[self.next_idx]
        self.next_idx += 1
        self.allocated[name] = reg
        return reg

    def get(self, name: str) -> int:
        return self.allocated.get(name, 0)


class DiversePatternGenerator:
    """Generates diverse code patterns for training."""

    def __init__(self, seed: int = None):
        if seed is not None:
            random.seed(seed)

        # Pattern generators with weights
        self.generators = [
            # Arithmetic patterns (high diversity needed)
            (100, self.gen_arithmetic_diverse),
            (80, self.gen_multi_op_arithmetic),
            (60, self.gen_compound_arithmetic),

            # Loop patterns (many variations)
            (80, self.gen_count_loop),
            (60, self.gen_while_loop),
            (40, self.gen_do_while_loop),
            (50, self.gen_nested_loop),

            # Conditional patterns
            (70, self.gen_if_else),
            (50, self.gen_if_chain),
            (40, self.gen_min_max),
            (40, self.gen_clamp),
            (30, self.gen_abs_value),

            # Memory patterns
            (60, self.gen_array_access),
            (50, self.gen_array_sum),
            (40, self.gen_array_search),
            (30, self.gen_swap_memory),
            (40, self.gen_copy_memory),

            # Function patterns
            (50, self.gen_function_call),
            (40, self.gen_recursive_factorial),
            (30, self.gen_recursive_fibonacci),
            (30, self.gen_tail_recursive),

            # Algorithmic patterns
            (40, self.gen_factorial_iterative_up),
            (40, self.gen_factorial_iterative_down),
            (40, self.gen_fibonacci_iterative),
            (30, self.gen_gcd_euclidean),
            (30, self.gen_gcd_subtraction),
            (30, self.gen_power_iterative),
            (20, self.gen_power_binary),

            # Bitwise patterns
            (50, self.gen_bitwise_ops),
            (40, self.gen_bit_count),
            (30, self.gen_bit_shift_multiply),
            (30, self.gen_bit_mask),
            (20, self.gen_bit_reverse),

            # Comparison chains
            (40, self.gen_compare_chain),
            (30, self.gen_bounds_check),
            (30, self.gen_range_check),

            # Constant loading variations
            (60, self.gen_load_constant),
            (40, self.gen_load_large_constant),

            # Extension call patterns
            (50, self.gen_single_ext_call),
            (40, self.gen_ext_call_chain),
            (30, self.gen_ext_call_with_check),

            # String operation patterns
            (40, self.gen_string_length),
            (30, self.gen_string_copy),
            (30, self.gen_string_compare),
        ]

        self.total_weight = sum(w for w, _ in self.generators)

    def generate(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate a random diverse pattern."""
        r = random.random() * self.total_weight
        cumsum = 0
        for weight, gen_func in self.generators:
            cumsum += weight
            if r <= cumsum:
                return gen_func()
        return self.generators[-1][1]()

    # =========================================================================
    # ARITHMETIC PATTERNS
    # =========================================================================

    def gen_arithmetic_diverse(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate arithmetic with random register allocation."""
        alloc = RegisterAllocator()

        ops = [
            ('add', ALU_ADD, ['+', 'plus', 'sum of', 'add']),
            ('subtract', ALU_SUB, ['-', 'minus', 'difference of', 'subtract']),
            ('multiply', MD_MUL, ['*', 'times', 'product of', 'multiply']),
            ('divide', MD_DIV, ['/', 'divided by', 'quotient of', 'divide']),
            ('modulo', MD_MOD, ['%', 'mod', 'remainder of', 'modulo']),
            ('and', ALU_AND, ['&', 'and', 'bitwise and']),
            ('or', ALU_OR, ['|', 'or', 'bitwise or']),
            ('xor', ALU_XOR, ['^', 'xor', 'exclusive or']),
        ]

        op_name, mode, op_words = random.choice(ops)
        a = random.randint(1, 1000)
        b = random.randint(1, 100)

        # Avoid division by zero
        if op_name in ('divide', 'modulo') and b == 0:
            b = 1

        ra = alloc.alloc('a')
        rb = alloc.alloc('b')
        rd = alloc.alloc('result')

        prompt_templates = [
            f"{random.choice(op_words)} {a} and {b}",
            f"compute {a} {op_name} {b}",
            f"calculate {a} {random.choice(op_words)} {b}",
            f"what is {a} {random.choice(op_words)} {b}",
            f"{op_name} {a} {b}",
            f"r{rd} = {a} {random.choice(op_words)} {b}",
        ]

        instructions = [
            Instruction(OP_MOV, 0, ra, 0, 0, 1, a),
            Instruction(OP_MOV, 0, rb, 0, 0, 1, b),
        ]

        if op_name in ('multiply', 'divide', 'modulo'):
            instructions.append(Instruction(OP_MULDIV, mode, rd, ra, rb, 0, 0))
        else:
            instructions.append(Instruction(OP_ALU, mode, rd, ra, rb, 0, 0))

        # Random decision: move result to r0 or leave in rd
        if rd != 0 and random.random() < 0.5:
            instructions.append(Instruction(OP_MOV, 0, 0, rd, 0, 0, 0))

        instructions.append(Instruction(OP_HALT, 0, 0, 0, 0, 0, 0))

        return random.choice(prompt_templates), instructions, {
            'category': 'arithmetic',
            'difficulty': 1,
            'operation': op_name
        }

    def gen_multi_op_arithmetic(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate multi-operation arithmetic expressions."""
        alloc = RegisterAllocator()

        # Generate expression like: (a + b) * c - d
        a = random.randint(1, 100)
        b = random.randint(1, 100)
        c = random.randint(1, 20)
        d = random.randint(1, 50)

        ops = random.sample(['+', '-', '*'], 2)

        ra = alloc.alloc('a')
        rb = alloc.alloc('b')
        rc = alloc.alloc('c')
        rt = alloc.alloc('temp')

        prompts = [
            f"calculate ({a} {ops[0]} {b}) {ops[1]} {c}",
            f"compute ({a} {ops[0]} {b}) {ops[1]} {c}",
            f"({a} {ops[0]} {b}) {ops[1]} {c}",
            f"evaluate {a} {ops[0]} {b} then {ops[1]} {c}",
        ]

        op_map = {'+': ALU_ADD, '-': ALU_SUB, '*': MD_MUL}

        instructions = [
            Instruction(OP_MOV, 0, ra, 0, 0, 1, a),
            Instruction(OP_MOV, 0, rb, 0, 0, 1, b),
            Instruction(OP_MOV, 0, rc, 0, 0, 1, c),
        ]

        # First operation
        if ops[0] == '*':
            instructions.append(Instruction(OP_MULDIV, MD_MUL, rt, ra, rb, 0, 0))
        else:
            instructions.append(Instruction(OP_ALU, op_map[ops[0]], rt, ra, rb, 0, 0))

        # Second operation
        if ops[1] == '*':
            instructions.append(Instruction(OP_MULDIV, MD_MUL, 0, rt, rc, 0, 0))
        else:
            instructions.append(Instruction(OP_ALU, op_map[ops[1]], 0, rt, rc, 0, 0))

        instructions.append(Instruction(OP_HALT, 0, 0, 0, 0, 0, 0))

        return random.choice(prompts), instructions, {
            'category': 'arithmetic',
            'difficulty': 2,
            'operation': 'compound'
        }

    def gen_compound_arithmetic(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate compound assignment patterns."""
        alloc = RegisterAllocator()

        patterns = [
            ('increment', 'add 1 to', lambda r: [Instruction(OP_ALUI, ALU_ADD, r, r, 0, 1, 1)]),
            ('decrement', 'subtract 1 from', lambda r: [Instruction(OP_ALUI, ALU_SUB, r, r, 0, 1, 1)]),
            ('double', 'multiply by 2', lambda r: [Instruction(OP_ALU, ALU_ADD, r, r, r, 0, 0)]),
            ('square', 'square', lambda r: [Instruction(OP_MULDIV, MD_MUL, r, r, r, 0, 0)]),
            ('negate', 'negate', lambda r: [
                Instruction(OP_MOV, 0, 30, 0, 0, 1, 0),
                Instruction(OP_ALU, ALU_SUB, r, 30, r, 0, 0)
            ]),
        ]

        pat_name, pat_verb, pat_gen = random.choice(patterns)

        rd = alloc.alloc('x')
        val = random.randint(1, 100)

        prompts = [
            f"{pat_name} {val}",
            f"{pat_verb} {val}",
            f"{pat_name} the value {val}",
            f"compute {pat_name} of {val}",
        ]

        instructions = [Instruction(OP_MOV, 0, rd, 0, 0, 1, val)]
        instructions.extend(pat_gen(rd))
        if rd != 0:
            instructions.append(Instruction(OP_MOV, 0, 0, rd, 0, 0, 0))
        instructions.append(Instruction(OP_HALT, 0, 0, 0, 0, 0, 0))

        return random.choice(prompts), instructions, {
            'category': 'arithmetic',
            'difficulty': 1,
            'operation': pat_name
        }

    # =========================================================================
    # LOOP PATTERNS
    # =========================================================================

    def gen_count_loop(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate counting loop (for i = 0; i < n; i++)."""
        alloc = RegisterAllocator()

        n = random.randint(5, 50)
        ri = alloc.alloc('i')
        rn = alloc.alloc('n')
        rsum = alloc.alloc('sum')

        prompts = [
            f"sum integers from 1 to {n}",
            f"add 1 + 2 + ... + {n}",
            f"calculate sum of first {n} numbers",
            f"compute 1+2+...+{n}",
            f"gauss sum to {n}",
        ]

        # Loop: sum = 0; for i = n; i > 0; i-- do sum += i
        instructions = [
            Instruction(OP_MOV, 0, rn, 0, 0, 1, n),
            Instruction(OP_MOV, 0, ri, rn, 0, 0, 0),
            Instruction(OP_MOV, 0, rsum, 0, 0, 1, 0),
            # loop:
            Instruction(OP_ALU, ALU_ADD, rsum, rsum, ri, 0, 0),
            Instruction(OP_ALUI, ALU_SUB, ri, ri, 0, 1, 1),
            Instruction(OP_BRANCH, BR_GT, 0, ri, 31, 1, -2),  # branch if i > 0
            Instruction(OP_MOV, 0, 0, rsum, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'loop',
            'difficulty': 2,
            'loop_type': 'count_down'
        }

    def gen_while_loop(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate while loop pattern."""
        alloc = RegisterAllocator()

        n = random.randint(3, 20)

        # Count digits in a number
        prompts = [
            f"count digits in {n * 111}",
            f"how many digits in {n * 111}",
            f"digit count of {n * 111}",
        ]

        rn = alloc.alloc('n')
        rcount = alloc.alloc('count')
        rten = alloc.alloc('ten')

        instructions = [
            Instruction(OP_MOV, 0, rn, 0, 0, 1, n * 111),
            Instruction(OP_MOV, 0, rcount, 0, 0, 1, 0),
            Instruction(OP_MOV, 0, rten, 0, 0, 1, 10),
            # while n > 0
            Instruction(OP_BRANCH, BR_LE, 0, rn, 31, 1, 4),  # exit if n <= 0
            Instruction(OP_ALUI, ALU_ADD, rcount, rcount, 0, 1, 1),
            Instruction(OP_MULDIV, MD_DIV, rn, rn, rten, 0, 0),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, -4),
            Instruction(OP_MOV, 0, 0, rcount, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'loop',
            'difficulty': 2,
            'loop_type': 'while'
        }

    def gen_do_while_loop(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate do-while loop pattern."""
        alloc = RegisterAllocator()

        n = random.randint(2, 10)

        # Factorial
        prompts = [
            f"compute factorial of {n} with do-while",
            f"calculate {n}! using loop",
            f"factorial({n}) iteratively",
        ]

        rn = alloc.alloc('n')
        rresult = alloc.alloc('result')

        instructions = [
            Instruction(OP_MOV, 0, rn, 0, 0, 1, n),
            Instruction(OP_MOV, 0, rresult, 0, 0, 1, 1),
            # do { result *= n; n--; } while (n > 0)
            Instruction(OP_MULDIV, MD_MUL, rresult, rresult, rn, 0, 0),
            Instruction(OP_ALUI, ALU_SUB, rn, rn, 0, 1, 1),
            Instruction(OP_BRANCH, BR_GT, 0, rn, 31, 1, -2),
            Instruction(OP_MOV, 0, 0, rresult, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'loop',
            'difficulty': 2,
            'loop_type': 'do_while'
        }

    def gen_nested_loop(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate nested loop pattern."""
        alloc = RegisterAllocator()

        n = random.randint(3, 8)
        m = random.randint(3, 8)

        prompts = [
            f"compute {n} * {m} using nested loops",
            f"multiply {n} by {m} with addition loops",
            f"nested loop multiply {n} x {m}",
        ]

        ri = alloc.alloc('i')
        rj = alloc.alloc('j')
        rsum = alloc.alloc('sum')

        instructions = [
            Instruction(OP_MOV, 0, ri, 0, 0, 1, n),
            Instruction(OP_MOV, 0, rsum, 0, 0, 1, 0),
            # outer loop
            Instruction(OP_MOV, 0, rj, 0, 0, 1, m),
            # inner loop
            Instruction(OP_ALUI, ALU_ADD, rsum, rsum, 0, 1, 1),
            Instruction(OP_ALUI, ALU_SUB, rj, rj, 0, 1, 1),
            Instruction(OP_BRANCH, BR_GT, 0, rj, 31, 1, -2),
            Instruction(OP_ALUI, ALU_SUB, ri, ri, 0, 1, 1),
            Instruction(OP_BRANCH, BR_GT, 0, ri, 31, 1, -5),
            Instruction(OP_MOV, 0, 0, rsum, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'loop',
            'difficulty': 3,
            'loop_type': 'nested'
        }

    # =========================================================================
    # CONDITIONAL PATTERNS
    # =========================================================================

    def gen_if_else(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate if-else pattern."""
        alloc = RegisterAllocator()

        threshold = random.randint(10, 100)

        prompts = [
            f"return 1 if input > {threshold}, else 0",
            f"check if r0 > {threshold}",
            f"compare r0 to {threshold}",
            f"is r0 greater than {threshold}?",
        ]

        rt = alloc.alloc('threshold')

        instructions = [
            Instruction(OP_MOV, 0, rt, 0, 0, 1, threshold),
            Instruction(OP_BRANCH, BR_LE, 0, 0, rt, 1, 3),  # if r0 <= threshold, goto else
            Instruction(OP_MOV, 0, 0, 0, 0, 1, 1),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, 2),
            # else:
            Instruction(OP_MOV, 0, 0, 0, 0, 1, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'conditional',
            'difficulty': 2,
            'pattern': 'if_else'
        }

    def gen_if_chain(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate if-else-if chain pattern."""
        alloc = RegisterAllocator()

        t1 = random.randint(10, 30)
        t2 = random.randint(50, 80)

        prompts = [
            f"return 0 if r0 < {t1}, 1 if < {t2}, else 2",
            f"classify: small (<{t1}), medium (<{t2}), large",
            f"three-way comparison at {t1} and {t2}",
        ]

        rt1 = alloc.alloc('t1')
        rt2 = alloc.alloc('t2')

        instructions = [
            Instruction(OP_MOV, 0, rt1, 0, 0, 1, t1),
            Instruction(OP_MOV, 0, rt2, 0, 0, 1, t2),
            # if r0 < t1
            Instruction(OP_BRANCH, BR_GE, 0, 0, rt1, 1, 3),
            Instruction(OP_MOV, 0, 0, 0, 0, 1, 0),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, 6),
            # else if r0 < t2
            Instruction(OP_BRANCH, BR_GE, 0, 0, rt2, 1, 3),
            Instruction(OP_MOV, 0, 0, 0, 0, 1, 1),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, 2),
            # else
            Instruction(OP_MOV, 0, 0, 0, 0, 1, 2),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'conditional',
            'difficulty': 3,
            'pattern': 'if_chain'
        }

    def gen_min_max(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate min/max pattern."""
        alloc = RegisterAllocator()

        is_min = random.random() < 0.5
        op_name = 'min' if is_min else 'max'
        branch_cond = BR_LE if is_min else BR_GE

        a = random.randint(1, 100)
        b = random.randint(1, 100)

        prompts = [
            f"{op_name} of {a} and {b}",
            f"compute {op_name}({a}, {b})",
            f"return {op_name}imum of {a}, {b}",
            f"find {op_name} value between {a} and {b}",
        ]

        ra = alloc.alloc('a')
        rb = alloc.alloc('b')

        instructions = [
            Instruction(OP_MOV, 0, ra, 0, 0, 1, a),
            Instruction(OP_MOV, 0, rb, 0, 0, 1, b),
            Instruction(OP_BRANCH, branch_cond, 0, ra, rb, 1, 2),
            Instruction(OP_MOV, 0, 0, rb, 0, 0, 0),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, 1),
            Instruction(OP_MOV, 0, 0, ra, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'conditional',
            'difficulty': 2,
            'pattern': op_name
        }

    def gen_clamp(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate clamp pattern."""
        alloc = RegisterAllocator()

        lo = random.randint(0, 20)
        hi = random.randint(80, 100)

        prompts = [
            f"clamp r0 between {lo} and {hi}",
            f"bound r0 to range [{lo}, {hi}]",
            f"constrain value to {lo}-{hi}",
        ]

        rlo = alloc.alloc('lo')
        rhi = alloc.alloc('hi')

        instructions = [
            Instruction(OP_MOV, 0, rlo, 0, 0, 1, lo),
            Instruction(OP_MOV, 0, rhi, 0, 0, 1, hi),
            # if r0 < lo: r0 = lo
            Instruction(OP_BRANCH, BR_GE, 0, 0, rlo, 1, 2),
            Instruction(OP_MOV, 0, 0, rlo, 0, 0, 0),
            # if r0 > hi: r0 = hi
            Instruction(OP_BRANCH, BR_LE, 0, 0, rhi, 1, 2),
            Instruction(OP_MOV, 0, 0, rhi, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'conditional',
            'difficulty': 2,
            'pattern': 'clamp'
        }

    def gen_abs_value(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate absolute value pattern."""
        alloc = RegisterAllocator()

        prompts = [
            "compute absolute value of r0",
            "return |r0|",
            "make r0 positive",
            "abs(r0)",
            "absolute value",
        ]

        rt = alloc.alloc('temp')

        # Method 1: conditional negate
        if random.random() < 0.5:
            instructions = [
                Instruction(OP_BRANCH, BR_GE, 0, 0, 31, 1, 2),
                Instruction(OP_ALU, ALU_SUB, 0, 31, 0, 0, 0),
                Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
            ]
        # Method 2: using XOR trick (for signed)
        else:
            instructions = [
                Instruction(OP_MOV, 0, rt, 0, 0, 0, 0),
                Instruction(OP_ALU, ALU_SAR, rt, rt, 0, 1, 63),  # sign extend
                Instruction(OP_ALU, ALU_XOR, 0, 0, rt, 0, 0),
                Instruction(OP_ALU, ALU_SUB, 0, 0, rt, 0, 0),
                Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
            ]

        return random.choice(prompts), instructions, {
            'category': 'conditional',
            'difficulty': 2,
            'pattern': 'abs'
        }

    # =========================================================================
    # MEMORY PATTERNS
    # =========================================================================

    def gen_array_access(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate array access pattern."""
        alloc = RegisterAllocator()

        base = 0x10000 + random.randint(0, 0x1000)
        idx = random.randint(0, 10)
        elem_size = random.choice([1, 4, 8])
        load_mode = {1: MEM_BYTE, 4: MEM_DWORD, 8: MEM_QWORD}[elem_size]

        prompts = [
            f"load array[{idx}] from 0x{base:x}",
            f"read element {idx} from array at 0x{base:x}",
            f"get arr[{idx}] (base=0x{base:x}, elem_size={elem_size})",
        ]

        rbase = alloc.alloc('base')
        ridx = alloc.alloc('idx')
        roff = alloc.alloc('offset')

        instructions = [
            Instruction(OP_MOV, 0, rbase, 0, 0, 1, base),
            Instruction(OP_MOV, 0, ridx, 0, 0, 1, idx),
            Instruction(OP_MOV, 0, roff, 0, 0, 1, elem_size),
            Instruction(OP_MULDIV, MD_MUL, roff, ridx, roff, 0, 0),
            Instruction(OP_ALU, ALU_ADD, rbase, rbase, roff, 0, 0),
            Instruction(OP_LOAD, load_mode, 0, rbase, 0, 1, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'memory',
            'difficulty': 2,
            'pattern': 'array_access'
        }

    def gen_array_sum(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate array sum pattern."""
        alloc = RegisterAllocator()

        n = random.randint(5, 20)
        base = 0x10000

        prompts = [
            f"sum {n} elements from array at 0x{base:x}",
            f"compute sum of array with {n} elements",
            f"add all {n} values in array",
        ]

        rptr = alloc.alloc('ptr')
        rcount = alloc.alloc('count')
        rsum = alloc.alloc('sum')
        rval = alloc.alloc('val')

        instructions = [
            Instruction(OP_MOV, 0, rptr, 0, 0, 1, base),
            Instruction(OP_MOV, 0, rcount, 0, 0, 1, n),
            Instruction(OP_MOV, 0, rsum, 0, 0, 1, 0),
            # loop:
            Instruction(OP_LOAD, MEM_QWORD, rval, rptr, 0, 1, 0),
            Instruction(OP_ALU, ALU_ADD, rsum, rsum, rval, 0, 0),
            Instruction(OP_ALUI, ALU_ADD, rptr, rptr, 0, 1, 8),
            Instruction(OP_ALUI, ALU_SUB, rcount, rcount, 0, 1, 1),
            Instruction(OP_BRANCH, BR_GT, 0, rcount, 31, 1, -4),
            Instruction(OP_MOV, 0, 0, rsum, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'memory',
            'difficulty': 3,
            'pattern': 'array_sum'
        }

    def gen_array_search(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate linear search pattern."""
        alloc = RegisterAllocator()

        n = random.randint(5, 20)
        target = random.randint(1, 100)

        prompts = [
            f"search for {target} in array of {n} elements",
            f"find index of {target} in array",
            f"linear search for {target}",
        ]

        rptr = alloc.alloc('ptr')
        rcount = alloc.alloc('count')
        ridx = alloc.alloc('idx')
        rval = alloc.alloc('val')
        rtarget = alloc.alloc('target')

        instructions = [
            Instruction(OP_MOV, 0, rptr, 0, 0, 1, 0x10000),
            Instruction(OP_MOV, 0, rcount, 0, 0, 1, n),
            Instruction(OP_MOV, 0, ridx, 0, 0, 1, 0),
            Instruction(OP_MOV, 0, rtarget, 0, 0, 1, target),
            # loop:
            Instruction(OP_LOAD, MEM_QWORD, rval, rptr, 0, 1, 0),
            Instruction(OP_BRANCH, BR_EQ, 0, rval, rtarget, 1, 6),  # found
            Instruction(OP_ALUI, ALU_ADD, rptr, rptr, 0, 1, 8),
            Instruction(OP_ALUI, ALU_ADD, ridx, ridx, 0, 1, 1),
            Instruction(OP_ALUI, ALU_SUB, rcount, rcount, 0, 1, 1),
            Instruction(OP_BRANCH, BR_GT, 0, rcount, 31, 1, -5),
            # not found
            Instruction(OP_MOV, 0, 0, 0, 0, 1, -1),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, 1),
            # found:
            Instruction(OP_MOV, 0, 0, ridx, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'memory',
            'difficulty': 3,
            'pattern': 'linear_search'
        }

    def gen_swap_memory(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate memory swap pattern."""
        alloc = RegisterAllocator()

        addr1 = 0x10000
        addr2 = 0x10008

        prompts = [
            f"swap values at 0x{addr1:x} and 0x{addr2:x}",
            "swap two memory locations",
            "exchange values in memory",
        ]

        rp1 = alloc.alloc('p1')
        rp2 = alloc.alloc('p2')
        rt1 = alloc.alloc('t1')
        rt2 = alloc.alloc('t2')

        instructions = [
            Instruction(OP_MOV, 0, rp1, 0, 0, 1, addr1),
            Instruction(OP_MOV, 0, rp2, 0, 0, 1, addr2),
            Instruction(OP_LOAD, MEM_QWORD, rt1, rp1, 0, 1, 0),
            Instruction(OP_LOAD, MEM_QWORD, rt2, rp2, 0, 1, 0),
            Instruction(OP_STORE, MEM_QWORD, rt2, rp1, 0, 1, 0),
            Instruction(OP_STORE, MEM_QWORD, rt1, rp2, 0, 1, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'memory',
            'difficulty': 2,
            'pattern': 'swap'
        }

    def gen_copy_memory(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate memory copy pattern."""
        alloc = RegisterAllocator()

        n = random.randint(4, 16)

        prompts = [
            f"copy {n} bytes from src to dst",
            f"memcpy {n} bytes",
            f"copy memory block of size {n}",
        ]

        rsrc = alloc.alloc('src')
        rdst = alloc.alloc('dst')
        rcount = alloc.alloc('count')
        rval = alloc.alloc('val')

        instructions = [
            Instruction(OP_MOV, 0, rsrc, 0, 0, 0, 0),  # r0 = src
            Instruction(OP_MOV, 0, rdst, 1, 0, 0, 0),  # r1 = dst
            Instruction(OP_MOV, 0, rcount, 0, 0, 1, n),
            # loop:
            Instruction(OP_LOAD, MEM_BYTE, rval, rsrc, 0, 1, 0),
            Instruction(OP_STORE, MEM_BYTE, rval, rdst, 0, 1, 0),
            Instruction(OP_ALUI, ALU_ADD, rsrc, rsrc, 0, 1, 1),
            Instruction(OP_ALUI, ALU_ADD, rdst, rdst, 0, 1, 1),
            Instruction(OP_ALUI, ALU_SUB, rcount, rcount, 0, 1, 1),
            Instruction(OP_BRANCH, BR_GT, 0, rcount, 31, 1, -5),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'memory',
            'difficulty': 3,
            'pattern': 'memcpy'
        }

    # =========================================================================
    # FUNCTION PATTERNS
    # =========================================================================

    def gen_function_call(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate function call pattern."""
        prompts = [
            "call helper function and return result",
            "invoke subroutine then halt",
            "call function at label",
        ]

        instructions = [
            Instruction(OP_CALL, 0, 0, 0, 0, 1, 3),  # call to offset +3
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
            # helper function:
            Instruction(OP_MOV, 0, 0, 0, 0, 1, 42),
            Instruction(OP_RET, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'function',
            'difficulty': 2,
            'pattern': 'call'
        }

    def gen_recursive_factorial(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate recursive factorial pattern."""
        n = random.randint(3, 8)

        prompts = [
            f"recursive factorial of {n}",
            f"compute {n}! recursively",
            f"factorial({n}) using recursion",
        ]

        # Simplified recursive pattern (uses stack simulation)
        instructions = [
            Instruction(OP_MOV, 0, 0, 0, 0, 1, n),
            Instruction(OP_CALL, 0, 0, 0, 0, 1, 2),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
            # factorial:
            Instruction(OP_BRANCH, BR_LE, 0, 0, 31, 1, 4),  # if n <= 0, return 1
            Instruction(OP_MOV, 0, 1, 0, 0, 0, 0),  # save n
            Instruction(OP_ALUI, ALU_SUB, 0, 0, 0, 1, 1),
            Instruction(OP_CALL, 0, 0, 0, 0, 1, -3),  # recurse
            Instruction(OP_MULDIV, MD_MUL, 0, 0, 1, 0, 0),
            Instruction(OP_RET, 0, 0, 0, 0, 0, 0),
            # base case:
            Instruction(OP_MOV, 0, 0, 0, 0, 1, 1),
            Instruction(OP_RET, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'function',
            'difficulty': 4,
            'pattern': 'recursive_factorial'
        }

    def gen_recursive_fibonacci(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate recursive fibonacci pattern."""
        n = random.randint(5, 12)

        prompts = [
            f"recursive fibonacci({n})",
            f"compute fib({n}) recursively",
            f"fibonacci {n} using recursion",
        ]

        # Simplified iterative version (true recursion would be complex)
        instructions = [
            Instruction(OP_MOV, 0, 0, 0, 0, 1, n),
            Instruction(OP_MOV, 0, 1, 0, 0, 1, 0),  # fib(0)
            Instruction(OP_MOV, 0, 2, 0, 0, 1, 1),  # fib(1)
            # loop:
            Instruction(OP_BRANCH, BR_LE, 0, 0, 31, 1, 5),
            Instruction(OP_ALU, ALU_ADD, 3, 1, 2, 0, 0),
            Instruction(OP_MOV, 0, 1, 2, 0, 0, 0),
            Instruction(OP_MOV, 0, 2, 3, 0, 0, 0),
            Instruction(OP_ALUI, ALU_SUB, 0, 0, 0, 1, 1),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, -5),
            Instruction(OP_MOV, 0, 0, 1, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'function',
            'difficulty': 3,
            'pattern': 'fibonacci'
        }

    def gen_tail_recursive(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate tail-recursive pattern."""
        n = random.randint(5, 20)

        prompts = [
            f"tail-recursive sum to {n}",
            f"sum 1..{n} with tail call",
            f"tail recursion for sum({n})",
        ]

        # Tail call optimized as loop
        instructions = [
            Instruction(OP_MOV, 0, 0, 0, 0, 1, n),
            Instruction(OP_MOV, 0, 1, 0, 0, 1, 0),  # accumulator
            # loop (simulating tail call):
            Instruction(OP_BRANCH, BR_LE, 0, 0, 31, 1, 4),
            Instruction(OP_ALU, ALU_ADD, 1, 1, 0, 0, 0),
            Instruction(OP_ALUI, ALU_SUB, 0, 0, 0, 1, 1),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, -3),
            Instruction(OP_MOV, 0, 0, 1, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'function',
            'difficulty': 3,
            'pattern': 'tail_recursive'
        }

    # =========================================================================
    # ALGORITHMIC PATTERNS
    # =========================================================================

    def gen_factorial_iterative_up(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Factorial counting up: 1 * 2 * 3 * ... * n."""
        alloc = RegisterAllocator()
        n = random.randint(3, 12)

        prompts = [
            f"factorial({n}) counting up",
            f"compute {n}! as 1*2*3*...*{n}",
            f"{n} factorial iterative ascending",
        ]

        ri = alloc.alloc('i')
        rn = alloc.alloc('n')
        rresult = alloc.alloc('result')

        instructions = [
            Instruction(OP_MOV, 0, rn, 0, 0, 1, n),
            Instruction(OP_MOV, 0, ri, 0, 0, 1, 1),
            Instruction(OP_MOV, 0, rresult, 0, 0, 1, 1),
            # loop: while i <= n
            Instruction(OP_BRANCH, BR_GT, 0, ri, rn, 1, 4),
            Instruction(OP_MULDIV, MD_MUL, rresult, rresult, ri, 0, 0),
            Instruction(OP_ALUI, ALU_ADD, ri, ri, 0, 1, 1),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, -3),
            Instruction(OP_MOV, 0, 0, rresult, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'algorithm',
            'difficulty': 2,
            'pattern': 'factorial_up'
        }

    def gen_factorial_iterative_down(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Factorial counting down: n * (n-1) * ... * 1."""
        alloc = RegisterAllocator()
        n = random.randint(3, 12)

        prompts = [
            f"factorial({n}) counting down",
            f"compute {n}! as {n}*{n-1}*...*1",
            f"{n} factorial iterative descending",
        ]

        ri = alloc.alloc('i')
        rresult = alloc.alloc('result')

        instructions = [
            Instruction(OP_MOV, 0, ri, 0, 0, 1, n),
            Instruction(OP_MOV, 0, rresult, 0, 0, 1, 1),
            # loop: while i > 0
            Instruction(OP_BRANCH, BR_LE, 0, ri, 31, 1, 4),
            Instruction(OP_MULDIV, MD_MUL, rresult, rresult, ri, 0, 0),
            Instruction(OP_ALUI, ALU_SUB, ri, ri, 0, 1, 1),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, -3),
            Instruction(OP_MOV, 0, 0, rresult, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'algorithm',
            'difficulty': 2,
            'pattern': 'factorial_down'
        }

    def gen_fibonacci_iterative(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Iterative fibonacci."""
        alloc = RegisterAllocator()
        n = random.randint(5, 20)

        prompts = [
            f"fibonacci({n}) iteratively",
            f"compute fib({n}) with loop",
            f"{n}th fibonacci number",
        ]

        rn = alloc.alloc('n')
        ra = alloc.alloc('a')
        rb = alloc.alloc('b')
        rt = alloc.alloc('temp')

        instructions = [
            Instruction(OP_MOV, 0, rn, 0, 0, 1, n),
            Instruction(OP_MOV, 0, ra, 0, 0, 1, 0),
            Instruction(OP_MOV, 0, rb, 0, 0, 1, 1),
            # loop:
            Instruction(OP_BRANCH, BR_LE, 0, rn, 31, 1, 6),
            Instruction(OP_ALU, ALU_ADD, rt, ra, rb, 0, 0),
            Instruction(OP_MOV, 0, ra, rb, 0, 0, 0),
            Instruction(OP_MOV, 0, rb, rt, 0, 0, 0),
            Instruction(OP_ALUI, ALU_SUB, rn, rn, 0, 1, 1),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, -5),
            Instruction(OP_MOV, 0, 0, ra, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'algorithm',
            'difficulty': 2,
            'pattern': 'fibonacci'
        }

    def gen_gcd_euclidean(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Euclidean GCD algorithm."""
        alloc = RegisterAllocator()
        a = random.randint(10, 100)
        b = random.randint(10, 100)

        prompts = [
            f"gcd({a}, {b}) euclidean",
            f"greatest common divisor of {a} and {b}",
            f"euclidean algorithm for gcd({a}, {b})",
        ]

        ra = alloc.alloc('a')
        rb = alloc.alloc('b')
        rt = alloc.alloc('temp')

        instructions = [
            Instruction(OP_MOV, 0, ra, 0, 0, 1, a),
            Instruction(OP_MOV, 0, rb, 0, 0, 1, b),
            # while b != 0
            Instruction(OP_BRANCH, BR_EQ, 0, rb, 31, 1, 5),
            Instruction(OP_MULDIV, MD_MOD, rt, ra, rb, 0, 0),
            Instruction(OP_MOV, 0, ra, rb, 0, 0, 0),
            Instruction(OP_MOV, 0, rb, rt, 0, 0, 0),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, -4),
            Instruction(OP_MOV, 0, 0, ra, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'algorithm',
            'difficulty': 2,
            'pattern': 'gcd_euclidean'
        }

    def gen_gcd_subtraction(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """GCD using subtraction."""
        alloc = RegisterAllocator()
        a = random.randint(10, 50)
        b = random.randint(10, 50)

        prompts = [
            f"gcd({a}, {b}) by subtraction",
            f"gcd using subtract method for {a}, {b}",
            f"subtraction-based gcd({a}, {b})",
        ]

        ra = alloc.alloc('a')
        rb = alloc.alloc('b')

        instructions = [
            Instruction(OP_MOV, 0, ra, 0, 0, 1, a),
            Instruction(OP_MOV, 0, rb, 0, 0, 1, b),
            # while a != b
            Instruction(OP_BRANCH, BR_EQ, 0, ra, rb, 1, 5),
            Instruction(OP_BRANCH, BR_LT, 0, ra, rb, 1, 2),
            Instruction(OP_ALU, ALU_SUB, ra, ra, rb, 0, 0),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, 1),
            Instruction(OP_ALU, ALU_SUB, rb, rb, ra, 0, 0),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, -5),
            Instruction(OP_MOV, 0, 0, ra, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'algorithm',
            'difficulty': 3,
            'pattern': 'gcd_subtraction'
        }

    def gen_power_iterative(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Power by repeated multiplication."""
        alloc = RegisterAllocator()
        base = random.randint(2, 5)
        exp = random.randint(2, 8)

        prompts = [
            f"compute {base}^{exp} iteratively",
            f"power({base}, {exp}) using loop",
            f"{base} to the {exp} power",
        ]

        rbase = alloc.alloc('base')
        rexp = alloc.alloc('exp')
        rresult = alloc.alloc('result')

        instructions = [
            Instruction(OP_MOV, 0, rbase, 0, 0, 1, base),
            Instruction(OP_MOV, 0, rexp, 0, 0, 1, exp),
            Instruction(OP_MOV, 0, rresult, 0, 0, 1, 1),
            # loop:
            Instruction(OP_BRANCH, BR_LE, 0, rexp, 31, 1, 4),
            Instruction(OP_MULDIV, MD_MUL, rresult, rresult, rbase, 0, 0),
            Instruction(OP_ALUI, ALU_SUB, rexp, rexp, 0, 1, 1),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, -3),
            Instruction(OP_MOV, 0, 0, rresult, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'algorithm',
            'difficulty': 2,
            'pattern': 'power_iterative'
        }

    def gen_power_binary(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Power using binary exponentiation (fast power)."""
        alloc = RegisterAllocator()
        base = random.randint(2, 5)
        exp = random.randint(2, 10)

        prompts = [
            f"fast power {base}^{exp}",
            f"binary exponentiation for {base}^{exp}",
            f"power({base}, {exp}) using squaring",
        ]

        rbase = alloc.alloc('base')
        rexp = alloc.alloc('exp')
        rresult = alloc.alloc('result')
        rone = alloc.alloc('one')

        instructions = [
            Instruction(OP_MOV, 0, rbase, 0, 0, 1, base),
            Instruction(OP_MOV, 0, rexp, 0, 0, 1, exp),
            Instruction(OP_MOV, 0, rresult, 0, 0, 1, 1),
            Instruction(OP_MOV, 0, rone, 0, 0, 1, 1),
            # while exp > 0
            Instruction(OP_BRANCH, BR_LE, 0, rexp, 31, 1, 7),
            # if exp & 1
            Instruction(OP_ALU, ALU_AND, 30, rexp, rone, 0, 0),
            Instruction(OP_BRANCH, BR_EQ, 0, 30, 31, 1, 1),
            Instruction(OP_MULDIV, MD_MUL, rresult, rresult, rbase, 0, 0),
            Instruction(OP_MULDIV, MD_MUL, rbase, rbase, rbase, 0, 0),
            Instruction(OP_ALU, ALU_SHR, rexp, rexp, rone, 0, 0),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, -6),
            Instruction(OP_MOV, 0, 0, rresult, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'algorithm',
            'difficulty': 4,
            'pattern': 'power_binary'
        }

    # =========================================================================
    # BITWISE PATTERNS
    # =========================================================================

    def gen_bitwise_ops(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate bitwise operation patterns."""
        alloc = RegisterAllocator()

        ops = [
            ('and', ALU_AND),
            ('or', ALU_OR),
            ('xor', ALU_XOR),
            ('shift left', ALU_SHL),
            ('shift right', ALU_SHR),
        ]

        op_name, mode = random.choice(ops)
        a = random.randint(1, 255)
        b = random.randint(1, 8) if 'shift' in op_name else random.randint(1, 255)

        prompts = [
            f"{a} {op_name} {b}",
            f"bitwise {op_name} of {a} and {b}",
            f"compute {a} {op_name} {b}",
        ]

        ra = alloc.alloc('a')
        rb = alloc.alloc('b')

        instructions = [
            Instruction(OP_MOV, 0, ra, 0, 0, 1, a),
            Instruction(OP_MOV, 0, rb, 0, 0, 1, b),
            Instruction(OP_ALU, mode, 0, ra, rb, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'bitwise',
            'difficulty': 1,
            'operation': op_name
        }

    def gen_bit_count(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Count set bits (popcount)."""
        alloc = RegisterAllocator()
        val = random.randint(1, 1000)

        prompts = [
            f"count bits in {val}",
            f"popcount({val})",
            f"number of 1s in binary of {val}",
            f"hamming weight of {val}",
        ]

        rn = alloc.alloc('n')
        rcount = alloc.alloc('count')
        rone = alloc.alloc('one')

        instructions = [
            Instruction(OP_MOV, 0, rn, 0, 0, 1, val),
            Instruction(OP_MOV, 0, rcount, 0, 0, 1, 0),
            Instruction(OP_MOV, 0, rone, 0, 0, 1, 1),
            # while n != 0
            Instruction(OP_BRANCH, BR_EQ, 0, rn, 31, 1, 5),
            Instruction(OP_ALU, ALU_AND, 30, rn, rone, 0, 0),
            Instruction(OP_ALU, ALU_ADD, rcount, rcount, 30, 0, 0),
            Instruction(OP_ALU, ALU_SHR, rn, rn, rone, 0, 0),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, -4),
            Instruction(OP_MOV, 0, 0, rcount, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'bitwise',
            'difficulty': 3,
            'pattern': 'popcount'
        }

    def gen_bit_shift_multiply(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Multiply by power of 2 using shifts."""
        alloc = RegisterAllocator()
        val = random.randint(1, 100)
        shift = random.randint(1, 5)
        mult = 1 << shift

        prompts = [
            f"multiply {val} by {mult} using shift",
            f"{val} * {mult} with left shift",
            f"fast multiply {val} by {mult}",
        ]

        rv = alloc.alloc('val')
        rs = alloc.alloc('shift')

        instructions = [
            Instruction(OP_MOV, 0, rv, 0, 0, 1, val),
            Instruction(OP_MOV, 0, rs, 0, 0, 1, shift),
            Instruction(OP_ALU, ALU_SHL, 0, rv, rs, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'bitwise',
            'difficulty': 1,
            'pattern': 'shift_multiply'
        }

    def gen_bit_mask(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate bit mask patterns."""
        alloc = RegisterAllocator()
        bits = random.randint(1, 16)

        prompts = [
            f"create {bits}-bit mask",
            f"mask with {bits} ones",
            f"generate 0x{'f' * (bits // 4)}{'f' if bits % 4 else ''}",
        ]

        rm = alloc.alloc('mask')
        rs = alloc.alloc('shift')

        instructions = [
            Instruction(OP_MOV, 0, rm, 0, 0, 1, 1),
            Instruction(OP_MOV, 0, rs, 0, 0, 1, bits),
            Instruction(OP_ALU, ALU_SHL, rm, rm, rs, 0, 0),
            Instruction(OP_ALUI, ALU_SUB, 0, rm, 0, 1, 1),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'bitwise',
            'difficulty': 2,
            'pattern': 'mask'
        }

    def gen_bit_reverse(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Reverse bits (8-bit for simplicity)."""
        alloc = RegisterAllocator()
        val = random.randint(1, 255)

        prompts = [
            f"reverse bits of {val}",
            f"bit reverse {val}",
            f"mirror bits in {val}",
        ]

        rn = alloc.alloc('n')
        rresult = alloc.alloc('result')
        rcount = alloc.alloc('count')
        rone = alloc.alloc('one')

        instructions = [
            Instruction(OP_MOV, 0, rn, 0, 0, 1, val),
            Instruction(OP_MOV, 0, rresult, 0, 0, 1, 0),
            Instruction(OP_MOV, 0, rcount, 0, 0, 1, 8),
            Instruction(OP_MOV, 0, rone, 0, 0, 1, 1),
            # loop:
            Instruction(OP_BRANCH, BR_LE, 0, rcount, 31, 1, 7),
            Instruction(OP_ALU, ALU_SHL, rresult, rresult, rone, 0, 0),
            Instruction(OP_ALU, ALU_AND, 30, rn, rone, 0, 0),
            Instruction(OP_ALU, ALU_OR, rresult, rresult, 30, 0, 0),
            Instruction(OP_ALU, ALU_SHR, rn, rn, rone, 0, 0),
            Instruction(OP_ALUI, ALU_SUB, rcount, rcount, 0, 1, 1),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, -6),
            Instruction(OP_MOV, 0, 0, rresult, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'bitwise',
            'difficulty': 4,
            'pattern': 'reverse'
        }

    # =========================================================================
    # COMPARISON PATTERNS
    # =========================================================================

    def gen_compare_chain(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate comparison chain pattern."""
        alloc = RegisterAllocator()
        a, b, c = sorted([random.randint(1, 100) for _ in range(3)])

        prompts = [
            f"check if {a} <= {b} <= {c}",
            f"is {b} between {a} and {c}?",
            f"range check: {a} <= x <= {c} for x={b}",
        ]

        ra = alloc.alloc('a')
        rb = alloc.alloc('b')
        rc = alloc.alloc('c')

        instructions = [
            Instruction(OP_MOV, 0, ra, 0, 0, 1, a),
            Instruction(OP_MOV, 0, rb, 0, 0, 1, b),
            Instruction(OP_MOV, 0, rc, 0, 0, 1, c),
            Instruction(OP_BRANCH, BR_GT, 0, ra, rb, 1, 4),  # if a > b, false
            Instruction(OP_BRANCH, BR_GT, 0, rb, rc, 1, 3),  # if b > c, false
            Instruction(OP_MOV, 0, 0, 0, 0, 1, 1),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, 2),
            Instruction(OP_MOV, 0, 0, 0, 0, 1, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'comparison',
            'difficulty': 2,
            'pattern': 'chain'
        }

    def gen_bounds_check(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate bounds checking pattern."""
        alloc = RegisterAllocator()
        size = random.randint(10, 100)

        prompts = [
            f"check if r0 < {size}",
            f"bounds check for array of size {size}",
            f"validate index < {size}",
        ]

        rsize = alloc.alloc('size')

        instructions = [
            Instruction(OP_MOV, 0, rsize, 0, 0, 1, size),
            Instruction(OP_BRANCH, BR_GE, 0, 0, rsize, 1, 2),  # if r0 >= size, fail
            Instruction(OP_MOV, 0, 0, 0, 0, 1, 1),  # success
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, 1),
            Instruction(OP_MOV, 0, 0, 0, 0, 1, 0),  # fail
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'comparison',
            'difficulty': 1,
            'pattern': 'bounds'
        }

    def gen_range_check(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate range checking pattern."""
        alloc = RegisterAllocator()
        lo = random.randint(0, 50)
        hi = random.randint(51, 100)

        prompts = [
            f"check if r0 in range [{lo}, {hi})",
            f"validate {lo} <= r0 < {hi}",
            f"range check for [{lo}, {hi})",
        ]

        rlo = alloc.alloc('lo')
        rhi = alloc.alloc('hi')

        instructions = [
            Instruction(OP_MOV, 0, rlo, 0, 0, 1, lo),
            Instruction(OP_MOV, 0, rhi, 0, 0, 1, hi),
            Instruction(OP_BRANCH, BR_LT, 0, 0, rlo, 1, 4),  # if r0 < lo, fail
            Instruction(OP_BRANCH, BR_GE, 0, 0, rhi, 1, 3),  # if r0 >= hi, fail
            Instruction(OP_MOV, 0, 0, 0, 0, 1, 1),  # success
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, 1),
            Instruction(OP_MOV, 0, 0, 0, 0, 1, 0),  # fail
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'comparison',
            'difficulty': 2,
            'pattern': 'range'
        }

    # =========================================================================
    # CONSTANT LOADING PATTERNS
    # =========================================================================

    def gen_load_constant(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate constant loading with variations."""
        alloc = RegisterAllocator()

        # Various ways to load common constants
        patterns = [
            (0, ["load zero", "set r0 to 0", "r0 = 0"]),
            (1, ["load one", "set r0 to 1", "r0 = 1"]),
            (-1, ["load -1", "set r0 to all ones", "r0 = 0xFFFFFFFF"]),
            (random.randint(2, 127), None),  # small positive
            (random.randint(128, 65535), None),  # medium
        ]

        val, prompts = random.choice(patterns)
        if prompts is None:
            prompts = [f"load {val}", f"set r0 to {val}", f"r0 = {val}"]

        rd = alloc.alloc('dest')

        # Different ways to load
        if val == 0:
            instructions = [
                Instruction(OP_ALU, ALU_XOR, rd, rd, rd, 0, 0),  # xor with self
            ]
        elif val == -1:
            instructions = [
                Instruction(OP_MOV, 0, rd, 0, 0, 1, 0),
                Instruction(OP_ALUI, ALU_SUB, rd, rd, 0, 1, 1),
            ]
        else:
            instructions = [
                Instruction(OP_MOV, 0, rd, 0, 0, 1, val),
            ]

        if rd != 0:
            instructions.append(Instruction(OP_MOV, 0, 0, rd, 0, 0, 0))
        instructions.append(Instruction(OP_HALT, 0, 0, 0, 0, 0, 0))

        return random.choice(prompts), instructions, {
            'category': 'constant',
            'difficulty': 1,
            'value': val
        }

    def gen_load_large_constant(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Load large constants that need multiple instructions."""
        alloc = RegisterAllocator()

        # Values that need special handling
        val = random.choice([
            0x10000,  # DATA_BASE
            0xDEADBEEF,
            random.randint(0x100000, 0xFFFFFF),
        ])

        prompts = [
            f"load 0x{val:x}",
            f"set r0 to {val}",
            f"r0 = 0x{val:x}",
        ]

        rd = alloc.alloc('dest')

        # For large values, may need to build in parts
        instructions = [
            Instruction(OP_MOV, 0, rd, 0, 0, 1, val & 0xFFFFFFFF),
        ]

        if rd != 0:
            instructions.append(Instruction(OP_MOV, 0, 0, rd, 0, 0, 0))
        instructions.append(Instruction(OP_HALT, 0, 0, 0, 0, 0, 0))

        return random.choice(prompts), instructions, {
            'category': 'constant',
            'difficulty': 2,
            'value': val
        }

    # =========================================================================
    # EXTENSION CALL PATTERNS
    # =========================================================================

    def gen_single_ext_call(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate single extension call pattern."""
        alloc = RegisterAllocator()

        # Common extension calls
        ext_patterns = [
            (1, 'sha256', ["hash with sha256", "compute sha256", "sha256 digest"]),
            (170, 'json_parse', ["parse json", "json decode", "parse json string"]),
            (190, 'http_get', ["http get request", "fetch url", "get http"]),
            (260, 'sqlite_open', ["open database", "sqlite open", "connect to db"]),
            (330, 'uuid_v4', ["generate uuid", "create uuid", "new uuid"]),
            (440, 'datetime_now', ["get current time", "now timestamp", "current datetime"]),
        ]

        ext_id, ext_name, prompts = random.choice(ext_patterns)

        rarg = alloc.alloc('arg')

        instructions = [
            Instruction(OP_MOV, 0, rarg, 0, 0, 1, 0x10000),  # arg = data ptr
            Instruction(OP_EXT_CALL, ext_id, 0, rarg, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'extension',
            'difficulty': 2,
            'extension': ext_name
        }

    def gen_ext_call_chain(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate chained extension calls."""
        alloc = RegisterAllocator()

        # Common chains
        chains = [
            ([190, 195], ["http get and check status", "fetch url and get status"]),
            ([170, 171], ["parse json and get field", "json parse then extract"]),
            ([260, 264, 268], ["open db and query", "sqlite open prepare step"]),
            ([1, 420], ["hash then base64 encode", "sha256 and encode"]),
        ]

        ext_ids, prompts = random.choice(chains)

        rarg = alloc.alloc('arg')
        rresult = alloc.alloc('result')

        instructions = [
            Instruction(OP_MOV, 0, rarg, 0, 0, 1, 0x10000),
        ]

        for ext_id in ext_ids:
            instructions.append(Instruction(OP_EXT_CALL, ext_id, rresult, rarg, 0, 0, 0))
            rarg = rresult  # Chain results

        instructions.append(Instruction(OP_MOV, 0, 0, rresult, 0, 0, 0))
        instructions.append(Instruction(OP_HALT, 0, 0, 0, 0, 0, 0))

        return random.choice(prompts), instructions, {
            'category': 'extension',
            'difficulty': 3,
            'pattern': 'chain'
        }

    def gen_ext_call_with_check(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate extension call with error checking."""
        alloc = RegisterAllocator()

        ext_id = random.choice([190, 260, 500])  # http_get, sqlite_open, tls_connect

        prompts = [
            "call extension and check for error",
            "extension call with error handling",
            "call and validate result",
        ]

        rarg = alloc.alloc('arg')
        rresult = alloc.alloc('result')

        instructions = [
            Instruction(OP_MOV, 0, rarg, 0, 0, 1, 0x10000),
            Instruction(OP_EXT_CALL, ext_id, rresult, rarg, 0, 0, 0),
            Instruction(OP_BRANCH, BR_LT, 0, rresult, 31, 1, 2),  # if result < 0, error
            Instruction(OP_MOV, 0, 0, rresult, 0, 0, 0),  # success
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, 1),
            Instruction(OP_MOV, 0, 0, 0, 0, 1, -1),  # error
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'extension',
            'difficulty': 3,
            'pattern': 'error_check'
        }

    # =========================================================================
    # STRING PATTERNS
    # =========================================================================

    def gen_string_length(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate strlen pattern."""
        alloc = RegisterAllocator()

        prompts = [
            "compute string length",
            "strlen of string at r0",
            "count characters in string",
            "find null terminator",
        ]

        rptr = alloc.alloc('ptr')
        rlen = alloc.alloc('len')
        rval = alloc.alloc('val')

        instructions = [
            Instruction(OP_MOV, 0, rptr, 0, 0, 0, 0),  # ptr = r0 (string)
            Instruction(OP_MOV, 0, rlen, 0, 0, 1, 0),
            # loop:
            Instruction(OP_LOAD, MEM_BYTE, rval, rptr, 0, 1, 0),
            Instruction(OP_BRANCH, BR_EQ, 0, rval, 31, 1, 4),  # if *ptr == 0, done
            Instruction(OP_ALUI, ALU_ADD, rlen, rlen, 0, 1, 1),
            Instruction(OP_ALUI, ALU_ADD, rptr, rptr, 0, 1, 1),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, -4),
            Instruction(OP_MOV, 0, 0, rlen, 0, 0, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'string',
            'difficulty': 2,
            'pattern': 'strlen'
        }

    def gen_string_copy(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate strcpy pattern."""
        alloc = RegisterAllocator()

        prompts = [
            "copy string from r0 to r1",
            "strcpy src to dst",
            "duplicate string",
        ]

        rsrc = alloc.alloc('src')
        rdst = alloc.alloc('dst')
        rval = alloc.alloc('val')

        instructions = [
            Instruction(OP_MOV, 0, rsrc, 0, 0, 0, 0),  # src = r0
            Instruction(OP_MOV, 0, rdst, 1, 0, 0, 0),  # dst = r1
            # loop:
            Instruction(OP_LOAD, MEM_BYTE, rval, rsrc, 0, 1, 0),
            Instruction(OP_STORE, MEM_BYTE, rval, rdst, 0, 1, 0),
            Instruction(OP_BRANCH, BR_EQ, 0, rval, 31, 1, 4),  # if *src == 0, done
            Instruction(OP_ALUI, ALU_ADD, rsrc, rsrc, 0, 1, 1),
            Instruction(OP_ALUI, ALU_ADD, rdst, rdst, 0, 1, 1),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, -5),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'string',
            'difficulty': 3,
            'pattern': 'strcpy'
        }

    def gen_string_compare(self) -> Tuple[str, List[Instruction], Dict[str, Any]]:
        """Generate strcmp pattern."""
        alloc = RegisterAllocator()

        prompts = [
            "compare strings at r0 and r1",
            "strcmp two strings",
            "check if strings equal",
        ]

        rs1 = alloc.alloc('s1')
        rs2 = alloc.alloc('s2')
        rc1 = alloc.alloc('c1')
        rc2 = alloc.alloc('c2')

        instructions = [
            Instruction(OP_MOV, 0, rs1, 0, 0, 0, 0),
            Instruction(OP_MOV, 0, rs2, 1, 0, 0, 0),
            # loop:
            Instruction(OP_LOAD, MEM_BYTE, rc1, rs1, 0, 1, 0),
            Instruction(OP_LOAD, MEM_BYTE, rc2, rs2, 0, 1, 0),
            Instruction(OP_BRANCH, BR_NE, 0, rc1, rc2, 1, 6),  # if c1 != c2, done
            Instruction(OP_BRANCH, BR_EQ, 0, rc1, 31, 1, 5),  # if c1 == 0, equal
            Instruction(OP_ALUI, ALU_ADD, rs1, rs1, 0, 1, 1),
            Instruction(OP_ALUI, ALU_ADD, rs2, rs2, 0, 1, 1),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, -6),
            # not equal:
            Instruction(OP_ALU, ALU_SUB, 0, rc1, rc2, 0, 0),
            Instruction(OP_JUMP, 0, 0, 0, 0, 1, 1),
            # equal:
            Instruction(OP_MOV, 0, 0, 0, 0, 1, 0),
            Instruction(OP_HALT, 0, 0, 0, 0, 0, 0),
        ]

        return random.choice(prompts), instructions, {
            'category': 'string',
            'difficulty': 3,
            'pattern': 'strcmp'
        }


def generate_diverse_samples(num_samples: int, seed: int = None) -> List[Dict[str, Any]]:
    """Generate diverse training samples."""
    gen = DiversePatternGenerator(seed=seed)
    samples = []

    for _ in range(num_samples):
        prompt, instructions, metadata = gen.generate()

        # Pad to 64 instructions
        instr_dicts = [i.to_dict() for i in instructions]
        while len(instr_dicts) < 64:
            instr_dicts.append({
                'valid': 0,
                'opcode': 0,
                'mode': 0,
                'rd': 0,
                'rs1': 0,
                'rs2': 0,
                'has_imm': 0,
                'imm_bin': 0,
            })

        samples.append({
            'context': prompt,
            'instructions': instr_dicts[:64],
            'metadata': {
                'source': 'diverse-synthetic',
                **metadata
            }
        })

    return samples


if __name__ == '__main__':
    import sys

    num = int(sys.argv[1]) if len(sys.argv) > 1 else 1000
    samples = generate_diverse_samples(num, seed=42)

    print(f"Generated {len(samples)} diverse samples")

    # Show category distribution
    cats = {}
    for s in samples:
        cat = s['metadata'].get('category', 'unknown')
        cats[cat] = cats.get(cat, 0) + 1

    print("\nCategory distribution:")
    for cat, count in sorted(cats.items(), key=lambda x: -x[1]):
        print(f"  {cat}: {count}")
