# Architecture Overview

Neurlang is an AI-optimized binary programming language with a custom 32-opcode ISA designed for 1000x faster code generation than traditional LLM approaches.

## Vision

Traditional AI coding agents are powerful but constrained by LLM inference latency. Each iteration requires a full forward pass through a large language model, limiting the speed of the edit-compile-test loop.

Neurlang solves this by combining:
- **Custom ISA** - 32-opcode assembly language optimized for AI prediction
- **Parallel instruction prediction** - 64 instructions predicted in a single forward pass
- **Copy-and-patch compilation** - Native x86-64 code in ~5μs
- **Rust-verified stdlib** - Correctness guaranteed through verified implementations
- **RAG-based extensions** - Unlimited capabilities via semantic lookup

The result: 1000x faster iteration compared to traditional LLM-based code generation.

## System Architecture

```
+---------------------------------------------------------------------------+
|                         USER                                               |
|  "compute factorial of 10"                                                 |
+---------------------------------------------------------------------------+
                              |
                              v
+---------------------------------------------------------------------------+
|                    PARALLEL INSTRUCTION PREDICTION                         |
|                                                                            |
|  50-100M parameter model, 64 slots, ~30ms inference                        |
|  Predicts: opcode, registers, immediate, valid flag                        |
+---------------------------------------------------------------------------+
                              |
                              v
+---------------------------------------------------------------------------+
|                    COPY-AND-PATCH COMPILER                                 |
|                                                                            |
|  Pre-compiled stencils + patching: ~5μs to native x86-64                   |
+---------------------------------------------------------------------------+
                              |
                              v
+---------------------------------------------------------------------------+
|                 EXECUTE + VERIFY                                           |
|                                                                            |
|  Native execution -> Test verification via @test annotations               |
+---------------------------------------------------------------------------+
```

## The Language

Neurlang programs are written in assembly with 32 opcodes:

```asm
; Fibonacci example
; Input: n in r0
; Output: fib(n) in r0

.entry:
    mov r1, 0           ; a = 0
    mov r2, 1           ; b = 1
    beq r0, zero, .ret_a
    subi r0, r0, 1
    beq r0, zero, .ret_b
.loop:
    add r3, r1, r2      ; next = a + b
    mov r1, r2          ; a = b
    mov r2, r3          ; b = next
    subi r0, r0, 1
    bne r0, zero, .loop
.ret_b:
    mov r0, r2
    b .done
.ret_a:
    mov r0, r1
.done:
    halt
```

### Opcode Categories

| Category | Opcodes | Examples |
|----------|---------|----------|
| Arithmetic | ALU, ALUI, MULDIV | `add`, `sub`, `mul`, `div` |
| Memory | LOAD, STORE, ATOMIC | `load.d`, `store.d`, `cas` |
| Control | BRANCH, CALL, RET, JUMP | `beq`, `bne`, `call`, `ret` |
| I/O | FILE, NET, IO, TIME | `file.open`, `net.send`, `io.print` |
| Math | FPU, RAND, BITS | `fpu.sqrt`, `rand.u64`, `bits.popcount` |
| System | MOV, NOP, HALT | `mov`, `halt` |
| Extensions | EXT.CALL | `ext.call r0, sha256, r1, r2` |

## Stdlib Architecture

The standard library is written in Rust and compiled to Neurlang assembly:

```
stdlib/src/*.rs (Rust source - verified)
       ↓
   nl stdlib --build (Rust→IR compiler)
       ↓
lib/*.nl (generated Neurlang assembly)
       ↓
   nl stdlib --verify (compare outputs)
       ↓
Training data for model
```

### Verification

The `nl stdlib --verify` command runs both Rust and Neurlang implementations with the same inputs and verifies identical outputs:

```bash
$ nl stdlib --verify
Verifying stdlib implementations...

✓ Verification passed:
  36 functions verified
  119 tests passed
```

## Generation Loop

```
Generation Loop (per iteration):
================================

  +-------------------------------------------------------------------------+
  |                           GENERATION PHASE                              |
  +-------------------------------------------------------------------------+
  |                                                                         |
  |   Input Text                                                            |
  |       |                                                                 |
  |       v                                                                 |
  |   +-------------------+                                                 |
  |   | Tokenizer         |  ~0.1ms                                         |
  |   +--------+----------+                                                 |
  |            |                                                            |
  |            v                                                            |
  |   +-------------------+                                                 |
  |   | Parallel Predict  |  ~30ms (64 slots, single forward pass)          |
  |   | Multi-Head Model  |                                                 |
  |   +--------+----------+                                                 |
  |            |                                                            |
  |            v                                                            |
  |   +-------------------+                                                 |
  |   | Copy-and-Patch    |  ~5μs (stencil compilation)                     |
  |   | Compiler          |                                                 |
  |   +--------+----------+                                                 |
  |                                                                         |
  +-------------------------------------------------------------------------+
              |
              v
  +-------------------------------------------------------------------------+
  |                          EXECUTION PHASE                                |
  +-------------------------------------------------------------------------+
  |                                                                         |
  |   +-------------------+    +-------------------+                        |
  |   | Native Execution  |--->| Test Verification |                        |
  |   +-------------------+    +-------------------+                        |
  |                                                                         |
  +-------------------------------------------------------------------------+
```

## Parallel Instruction Prediction

Predicts 64 instructions in a single forward pass:

```
Single Forward Pass Output:
+---------+------------------+----------------------+
| Slot    | Prediction       | Confidence           |
+---------+------------------+----------------------+
| Slot 0  | mov r0, 5        | 0.98                 |
| Slot 1  | mov r1, 1        | 0.97                 |
| Slot 2  | beq r0, zero, L3 | 0.95                 |
| Slot 3  | mul r1, r1, r0   | 0.94                 |
| ...     | ...              | ...                  |
| Slot 63 | (unused)         | 0.00                 |
+---------+------------------+----------------------+

Benefits:
- Amortized inference cost across 64 instructions
- No sequential token generation
- Predictable latency regardless of program length
```

## Extension System

For complex operations beyond the 32 opcodes:

```asm
; Call crypto extension
ext.call r0, sha256, r1, r2           ; SHA-256 hash
ext.call r0, aes256_gcm_encrypt, ...  ; AES-256-GCM

; Call via RAG resolution
ext.call r0, @"parse JSON", r1, r2    ; Resolved at assembly time
```

### Extension Ranges

| Range | Category | Examples |
|-------|----------|----------|
| 1-99 | Crypto | sha256, aes256, ed25519 |
| 200-219 | JSON | json_parse, json_stringify |
| 220-239 | HTTP | http_get, http_post |
| 240-259 | File System | file_read, file_write |
| 260-279 | SQLite | sqlite_open, sqlite_query |
| 280-299 | Regex | regex_match, regex_replace |

## Performance Characteristics

| Metric | Traditional LLM | Neurlang | Speedup |
|--------|-----------------|----------|---------|
| Token Generation | 50-200ms/token | N/A | - |
| Full Program (10 lines) | 500-2000ms | ~30ms | 50x |
| Compilation | 100-500ms | ~5μs | 100,000x |
| Iteration Cycle | 5-30 seconds | 30-50ms | 1000x |

## File Extensions

Neurlang programs use the `.nl` file extension:

```
lib/
  math/
    factorial.nl    # Generated from stdlib/src/math.rs
    fibonacci.nl
    gcd.nl
  string/
    strlen.nl
    atoi.nl
examples/
  algorithm/
    quicksort.nl    # Hand-written examples
```

## CLI Commands

```bash
# Run a program
nl run -i program.nl

# Assemble to binary
nl asm -i program.nl -o program.nlb

# Build stdlib from Rust
nl stdlib --build

# Verify Rust == Neurlang output
nl stdlib --verify

# Run tests
nl test -p lib
```

## RAG-Enhanced Inference (NEW)

For even faster inference, Neurlang includes an optional **RAG-enhanced pipeline** that uses in-memory intent classification:

```
Query → FastEmbedder → IntentIndex → confidence > 0.7?
                                          │
                        YES ──────────────┴─────────── NO
                         │                              │
                    Direct Generator              Full Model
                     (~0.1ms)                     (~0.4ms)
```

**Benefits:**
- 3x faster average latency (0.12ms vs 0.35ms)
- 90% of queries hit the fast path
- Graceful degradation for novel queries

See [RAG Intent Index](./rag-intent-index.md) for details.

## See Also

- [How It Works](./how-it-works.md) - Technical deep dive
- [RAG Intent Index](./rag-intent-index.md) - In-memory intent classification
- [Assembly Guide](../ir/assembly.md) - Complete syntax reference
- [Opcode Reference](../ir/opcodes.md) - All 32 opcodes
- [Stdlib Development](../stdlib/README.md) - Writing stdlib in Rust
- [CLI Commands](../cli/commands.md) - Full command reference
