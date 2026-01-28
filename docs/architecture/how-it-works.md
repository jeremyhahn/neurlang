# How Neurlang Works

This document provides a technical deep dive into Neurlang's architecture, explaining how it achieves 1000x faster iteration than traditional AI coding approaches.

## Core Concept: AI-Optimized Binary Programming

Neurlang is a custom ISA (Instruction Set Architecture) designed for AI code generation. Instead of generating human-readable code token-by-token through a large language model, Neurlang uses parallel instruction prediction to generate entire programs in a single forward pass.

```
Traditional AI Code Generation:
===============================

  User Request
       |
       v
  +------------------+
  | LLM Inference    |  50-200ms per token
  | (token by token) |  ~100 tokens = 5-20 seconds
  +--------+---------+
           |
           v
  +------------------+
  | Parse & Compile  |  100-500ms
  +--------+---------+
           |
           v
  +------------------+
  | Execute & Test   |  Variable
  +--------+---------+
           |
           v
  Total iteration: 5-30 seconds


Neurlang:
=========

  User Request
       |
       v
  +------------------+
  | Parallel Predict |  ~30ms (64 instructions)
  | (single pass)    |
  +--------+---------+
           |
           v
  +------------------+
  | Copy-and-Patch   |  ~5μs
  | Compile          |
  +--------+---------+
           |
           v
  +------------------+
  | Execute & Test   |  Variable
  +--------+---------+
           |
           v
  Total iteration: 30-50 milliseconds (1000x faster)
```

## The Language: 32-Opcode Assembly

Neurlang programs are written in a simple assembly language with 32 opcodes. This is the **actual syntax** used:

```asm
; Factorial example
; Input: n in r0
; Output: n! in r0

.entry:
    mov r1, 1           ; result = 1
    mov r2, r0          ; i = n
.loop:
    beq r2, zero, .done ; if i == 0, exit
    mul r1, r1, r2      ; result *= i
    subi r2, r2, 1      ; i--
    b .loop
.done:
    mov r0, r1          ; return result
    halt
```

### Why Assembly?

1. **Minimal vocabulary**: 32 opcodes vs 1000+ tokens in Python/JavaScript
2. **Fixed instruction size**: 4 or 8 bytes, predictable parsing
3. **No syntax ambiguity**: Every instruction has exactly one meaning
4. **Fast compilation**: Copy-and-patch in microseconds

## Parallel Instruction Prediction

The key innovation enabling fast code generation is parallel instruction prediction: generating 64 instructions in a single forward pass instead of token-by-token generation.

### 64-Slot Prediction Architecture

```
+-------------------------------------------------------------------------+
|                   Parallel Instruction Prediction                        |
+-------------------------------------------------------------------------+
|                                                                         |
|   Input: Natural language request + context                             |
|          |                                                              |
|          v                                                              |
|   +-------------------+                                                 |
|   | Encoder           |  Embed and encode input                         |
|   +--------+----------+                                                 |
|            |                                                            |
|            v                                                            |
|   +-------------------+                                                 |
|   | Shared Backbone   |  256-dim feature vector                         |
|   +--------+----------+                                                 |
|            |                                                            |
|            +----------+----------+----------+-----...-----+             |
|            |          |          |          |             |             |
|            v          v          v          v             v             |
|   +--------+--+ +-----+----+ +---+------+ +-+--------+ +--+-------+     |
|   | Slot 0    | | Slot 1   | | Slot 2   | | Slot 3   | | Slot 63  |     |
|   | Head      | | Head     | | Head     | | Head     | | Head     |     |
|   +-----------+ +----------+ +----------+ +----------+ +----------+     |
|        |             |            |            |             |          |
|        v             v            v            v             v          |
|   +----------+  +----------+ +----------+ +----------+ +----------+     |
|   | opcode   |  | opcode   | | opcode   | | opcode   | | opcode   |     |
|   | operands |  | operands | | operands | | operands | | operands |     |
|   | valid    |  | valid    | | valid    | | valid    | | valid    |     |
|   +----------+  +----------+ +----------+ +----------+ +----------+     |
|                                                                         |
|   Output: 64 instruction predictions (variable number actually used)    |
|                                                                         |
+-------------------------------------------------------------------------+
```

### Per-Slot Output Structure

Each of the 64 slots predicts:

| Field | Size | Description |
|-------|------|-------------|
| opcode | 5 bits | One of 32 opcodes |
| mode | 3 bits | Operation variant |
| dest_reg | 5 bits | Destination register |
| src1_reg | 5 bits | Source register 1 |
| src2_reg | 5 bits | Source register 2 |
| immediate | 32 bits | Immediate value |
| valid | 1 bit | Is this slot used? |
| confidence | float | Prediction confidence |

### Why 64 Slots?

- Average Neurlang function: 10-30 instructions
- 64 slots covers 95% of generated programs
- Larger programs: multiple generation passes
- Unused slots: marked invalid, no overhead

## Copy-and-Patch Compilation

The copy-and-patch compiler achieves ~5μs compilation by using pre-compiled code stencils.

### Stencil Table

```
Stencil Table (loaded at startup):
==================================

  +--------+--------------------------------------------------+----------+
  | Opcode | Stencil Template (x86-64)                        | Size     |
  +--------+--------------------------------------------------+----------+
  | MOV    | 48 b8 [IMM64] 48 89 04 c7                        | 16 bytes |
  | ADD    | 48 8b 04 c7 48 8b 0c cf 48 01 c8 48 89 04 c7     | 14 bytes |
  | SUB    | 48 8b 04 c7 48 8b 0c cf 48 29 c8 48 89 04 c7     | 14 bytes |
  | MUL    | 48 8b 04 c7 48 8b 0c cf 48 0f af c1 48 89 04 c7  | 15 bytes |
  | BRANCH | e9 [REL32]                                       | 5 bytes  |
  | HALT   | c3                                               | 1 byte   |
  | ...    | ...                                              | ...      |
  +--------+--------------------------------------------------+----------+
```

### Compilation Process

```
Copy-and-Patch Process:
=======================

  IR Program:
  [ MOV r0, 42 ]
  [ MOV r1, 10 ]
  [ ADD r0, r0, r1 ]
  [ HALT ]
         |
         v
  +------------------+
  | Allocate buffer  |  Get executable memory
  | (pre-allocated   |  from pool (~200ns)
  |  pool)           |
  +--------+---------+
         |
         v
  +------------------+
  | For each instr:  |
  | 1. Copy stencil  |  memcpy template bytes
  | 2. Patch holes   |  Fill in register indices, immediates
  +--------+---------+
         |
         v
  +------------------+
  | Result:          |
  | 48 b8 2a 00...   |  MOV r0, 42
  | 48 b8 0a 00...   |  MOV r1, 10
  | 48 8b 04 c7...   |  ADD r0, r0, r1
  | c3               |  RET
  +------------------+
         |
         v
  Native executable code (no parsing, no optimization passes)
```

## Stdlib: Rust as Source of Truth

The standard library is written in Rust and compiled to Neurlang assembly. This ensures correctness through verified Rust implementations.

```
stdlib/src/*.rs (Rust source)
       ↓
   nl stdlib --build (Rust→IR compiler)
       ↓
lib/*.nl (generated Neurlang assembly)
       ↓
Training data generator
       ↓
Model learns stdlib patterns
```

### Example: Rust to Assembly

**Input (stdlib/src/math.rs):**
```rust
pub fn factorial(n: u64) -> u64 {
    let mut result: u64 = 1;
    let mut i: u64 = n;
    while i > 0 {
        result = result * i;
        i = i - 1;
    }
    result
}
```

**Output (lib/math/factorial.nl):**
```asm
; @name: factorial
; @description: Calculate factorial of n
; @test: r0=0 -> r0=1
; @test: r0=5 -> r0=120

.entry:
    mov r1, 1           ; result = 1
    mov r2, r0          ; i = n
.loop:
    beq r2, zero, .done
    mul r1, r1, r2      ; result *= i
    subi r2, r2, 1      ; i--
    b .loop
.done:
    mov r0, r1
    halt
```

## Extension System

For operations that can't be expressed in 32 opcodes (crypto, JSON, HTTP), Neurlang uses Rust extensions resolved via RAG.

```asm
; Model outputs intent description
ext.call r0, @"parse JSON string", r1, r2

; RAG resolves to extension ID at assembly time
; Becomes: ext.call r0, 200, r1, r2  ; json_parse
```

### Built-in Extensions

| Range | Category | Examples |
|-------|----------|----------|
| 1-99 | Crypto | sha256, aes256, ed25519 |
| 200-219 | JSON | json_parse, json_stringify |
| 220-239 | HTTP | http_get, http_post |
| 240-259 | File System | file_read, file_write |

## Performance Summary

| Component | Latency | Notes |
|-----------|---------|-------|
| Model Inference | ~30ms | 64 slots, single pass |
| IR Generation | ~0.01μs | Lookup table |
| Compilation | ~5μs | Copy-and-patch |
| **Generation Total** | **~30ms** | Hot path |
| Native Execution | Variable | Depends on program |
| Verification | ~1ms | Run tests |

## Comparison with Traditional Approaches

| Aspect | Traditional LLM | Neurlang | Improvement |
|--------|-----------------|----------|-------------|
| Generation Method | Token-by-token | Parallel (64 slots) | 1000x faster |
| Language | Python/JS (1000s of tokens) | 32-opcode assembly | Simpler |
| Compilation | Complex parser + optimizer | Copy-and-patch | 100,000x faster |
| Iteration Cycle | 5-30 seconds | 30-50ms | 1000x faster |
| Verification | Manual testing | Automated @test | Built-in |

## See Also

- [Architecture Overview](./overview.md) - High-level system design
- [IR Specification](../ir/) - Instruction set details
- [Assembly Guide](../ir/assembly.md) - Complete syntax reference
- [Stdlib Development](../stdlib/README.md) - Writing stdlib in Rust
