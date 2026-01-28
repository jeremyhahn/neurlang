# Design Decisions

This document records the key architectural decisions made in Neurlang and their rationale.

---

## Why 32 Opcodes (Not More, Not Fewer)

### The Tradeoff

```
More opcodes:
  + Easier for compiler (closer to hardware)
  - Harder for AI to learn (larger vocabulary)
  - More stencils to maintain

Fewer opcodes:
  + Easier for AI to learn (smaller vocabulary)
  - Harder for compiler (more work per opcode)
  - Some operations become multi-instruction sequences
```

### What We Compared

| Approach | Opcodes | AI Training Cost | Compile Complexity | Verdict |
|----------|---------|------------------|-------------------|---------|
| Direct x86-64 | 1000+ | $700+ | N/A (already native) | Rejected |
| LLVM IR | ~60 | $150-300 | Still needs backend | Rejected |
| RISC-like | 64 | $100-200 | Moderate | Considered |
| **Neurlang** | **32** | **$25-60** | Low (copy-and-patch) | **Selected** |
| Minimal | 16 | $15-30 | High (complex expansion) | Rejected |

### Why 32 is Optimal

1. **Fits in 6 bits**: Opcode field is 6 bits, allowing room for growth
2. **Mode bits expand to 102+ operations**: AI learns 32 tokens, gets 102+ operations
3. **Covers all real needs**: Arithmetic, memory, control, I/O, security, concurrency
4. **EXT.CALL (0x20) enables extensions**: Rust FFI for crypto and complex operations
5. **Stencil table stays small**: ~30KB for all stencils
6. **Training cost is minimal**: 10-50M parameter model is sufficient

---

## Why Mode Bits Instead of More Opcodes

### The Design

```
Instead of:
  ADD_REG_REG     ; 0x00
  SUB_REG_REG     ; 0x01
  AND_REG_REG     ; 0x02
  OR_REG_REG      ; 0x03
  ... (8 separate opcodes)

We use:
  ALU + mode bits ; 0x00 + 3-bit mode
```

### Benefits

| Metric | Separate Opcodes | Mode Bits |
|--------|------------------|-----------|
| Tokens AI learns | 8 | 1 + pattern |
| Encoding density | 8 opcodes | 1 opcode + 3 bits |
| Stencil lookup | 8 entries | 8 entries (same) |
| Pattern regularity | Low | High |

### Mode Bit Expansion

```
32 opcodes × average 5 modes = ~102 operations

ALU: 8 modes (ADD, SUB, AND, OR, XOR, SHL, SHR, SAR)
BRANCH: 8 modes (EQ, NE, LT, LE, GT, GE, always, never)
LOAD: 4 modes (8/16/32/64 bit)
STORE: 4 modes (8/16/32/64 bit)
ATOMIC: 8 modes (CAS, XCHG, ADD, AND, OR, XOR, MIN, MAX)
FILE: 8 modes (open, read, write, close, seek, stat, mkdir, delete)
...
```

The AI learns **32 patterns**, not 102.

---

## Why Copy-and-Patch (Not Traditional JIT)

### Traditional JIT Compilation

```
Source → Tokenize → Parse → AST → IR → Optimize → Register Alloc → Emit
                                                                    ↓
                                                               100-500ms
```

### Copy-and-Patch

```
Binary IR → Lookup Stencil → memcpy → Patch operands → Done
                                                        ↓
                                                      <5μs
```

### Performance Comparison

| Metric | LLVM -O0 | V8 Liftoff | Copy-and-Patch |
|--------|----------|------------|----------------|
| Compile time | 100-500ms | 10-50ms | **<5μs** |
| Code quality | ~60% of -O2 | ~70% of -O2 | **~85% of -O2** |
| Complexity | Very high | High | **Low** |

### Why It Works for Neurlang

1. **Fixed opcode set**: 32 opcodes = 32 stencil templates (manageable)
2. **Regular encoding**: Patch locations are predictable
3. **No optimization needed**: AI generates "good enough" code
4. **Hot path is trivial**: Just memcpy + scalar writes

---

## Why Runtime Library Calls for I/O

### The Options

| Approach | Pros | Cons |
|----------|------|------|
| Raw syscalls | Fastest | Platform-specific, no sandboxing |
| **Runtime calls** | **Cross-platform, sandboxed** | **Small overhead** |
| Interpreter | Flexible | Too slow |

### Why Runtime Calls Win

```
┌───────────────────────────────────────────────────────────────────┐
│                    I/O via Runtime Library                        │
└───────────────────────────────────────────────────────────────────┘

  Benefits:
  ┌─────────────────────────────────────────────────────────────────┐
  │ 1. Cross-platform: Same Neurlang code runs on Linux/macOS/Windows  │
  │ 2. Sandboxing: Runtime checks I/O permissions before syscall    │
  │ 3. Error handling: Consistent error codes across platforms      │
  │ 4. Abstraction: Implementation can change without IR changes    │
  └─────────────────────────────────────────────────────────────────┘

  Overhead:
  ┌─────────────────────────────────────────────────────────────────┐
  │ • Function call: ~5ns                                           │
  │ • Permission check: ~10ns                                       │
  │ • Syscall: ~1000ns (dominates anyway)                           │
  │                                                                 │
  │ Total overhead: <1% of actual I/O cost                          │
  └─────────────────────────────────────────────────────────────────┘
```

### Sandboxing Benefits

```rust
// Without runtime library (dangerous):
FILE.write(fd, buf, len)  // Writes anywhere!

// With runtime library (safe):
neurlang_file_write(fd, buf, len)
  ├── Check: Is file_write permission granted?
  ├── Check: Is path in allowed whitelist?
  ├── Check: Is capability valid for buffer?
  └── Only then: Perform actual syscall
```

---

## Why Binary IR (Not Text Assembly)

### Token Efficiency

| Representation | Tokens per Instruction | Parse Time |
|----------------|------------------------|------------|
| x86-64 asm text | 5-15 tokens | ~1μs |
| LLVM IR text | 8-20 tokens | ~5μs |
| **Neurlang binary** | **4 or 8 bytes** | **~10ns** |

### Direct AI Output

```
Traditional:
  AI outputs text → Tokenize → Parse → Compile → Execute

Neurlang:
  AI outputs bytes → Copy-and-patch → Execute
                     (no parsing!)
```

### Debugging Still Works

```
Binary IR: 0x00 0x05 0x03 0x07

Disassemble for humans:
  add r5, r3, r7

The assembly syntax exists for debugging, not for the AI.
```

---

## Why Implicit Security (Fat Pointers)

### Traditional Approach

```asm
; Manual bounds checking (error-prone)
cmp   offset, length
jae   .error
mov   rax, [base + offset]
```

### Neurlang Approach

```asm
; Every LOAD stencil includes bounds check
load.d r1, [r2]   ; r2 is a fat pointer with embedded bounds
                   ; Compiler generates check automatically
```

### Fat Pointer Format

```
┌──────────┬──────────┬──────────┬──────────┬──────────────────────┐
│   TAG    │  TAINT   │  PERMS   │  LENGTH  │         BASE         │
│  4 bits  │  4 bits  │  8 bits  │  16 bits │       32 bits        │
└──────────┴──────────┴──────────┴──────────┴──────────────────────┘
```

### Benefits

1. **Zero developer burden**: AI doesn't need to generate bounds checks
2. **Consistent safety**: Every memory access is checked
3. **Performance**: Branch prediction makes checks nearly free
4. **No separate opcodes**: Security is implicit, not explicit

---

## Why No Interpreter Fallback

### Considered Options

| Option | Compile Time | Execution Speed | Complexity |
|--------|--------------|-----------------|------------|
| Interpreter for small programs | 0 | Slow (100-1000x) | +350 LOC |
| **JIT only** | **<5μs** | **Native** | **Simple** |

### Decision: JIT Only

```
Rationale:
  1. 5μs compile time is negligible vs 30ms model inference
  2. Single code path = fewer bugs
  3. Consistent performance characteristics
  4. ~350 LOC saved

Result:
  Always: jit(program) → native execution
```

---

## Why ONNX for Model Inference

### Options Considered

| Runtime | Language | GPU Support | Size |
|---------|----------|-------------|------|
| PyTorch | Python | Yes | ~500MB |
| TensorFlow | Python/C++ | Yes | ~800MB |
| **ONNX Runtime** | **C/Rust** | **Yes** | **~50MB** |
| Custom | Rust | No (CPU only) | ~10MB |

### Decision: ONNX Runtime

```
Benefits:
  1. No Python dependency at runtime
  2. Single binary deployment (binary + model.onnx)
  3. GPU auto-detection with CPU fallback
  4. Optimized inference (~30ms for 25M params)
  5. Industry standard format
```

---

## Why Retry Loop in Rust (Not in Model)

### Options

| Approach | Model Complexity | Training Cost | Reliability |
|----------|------------------|---------------|-------------|
| Multi-turn model | High | +$100-200 | Variable |
| Tool-calling model | Medium | +$50-100 | Better |
| **Rust orchestrator** | **Simple** | **+$0** | **Deterministic** |

### Decision: Rust Orchestrator

```rust
// Model stays simple (single-turn)
loop {
    let code = model.generate(prompt)?;
    match compile_and_run(&code) {
        Ok(result) => return Ok(result),
        Err(e) => prompt = format_retry_prompt(&e, &code),
    }
}
```

**Benefits:**
1. Model training stays simple ($25-60)
2. Error formatting is perfect (no hallucination)
3. Retry logic is deterministic and testable
4. Human can be kept in the loop if needed

---

## Why Three-Tier Code Reuse Architecture

### The Question

How should Neurlang handle code reuse? Options considered:

| Approach | Pros | Cons |
|----------|------|------|
| No libraries | Simple, AI writes everything | Repeats work, possible bugs |
| Native Neurlang libraries | Fast | Complex linking, AI must learn API |
| Rust FFI | Access to Rust ecosystem | Overhead for simple ops |
| **Hybrid (Three-Tier)** | **Best of all worlds** | **Slightly more complex** |

### Decision: Three-Tier Hybrid

```
┌────────────────────────────────────────────────────────────────────┐
│  Tier 0: Core Opcodes (33)     │  Tier 1: Intrinsics (~30)        │
│  AI writes from scratch        │  Zero-cost macro expansion       │
│  For: simple operations        │  For: common algorithms          │
├────────────────────────────────┴──────────────────────────────────┤
│  Tier 2: Rust FFI Extensions                                      │
│  For: crypto, complex parsing, domain-specific logic              │
└───────────────────────────────────────────────────────────────────┘
```

### Why This Design?

**Tier 0 (Core Opcodes)**: For simple operations like `add r0, r1, r2`, generating
from scratch is fine. 30ms generation is fast enough, and the agentic orchestrator
can iterate to fix bugs.

**Tier 1 (Intrinsics)**: Common algorithms (memcpy, strlen, gcd) as zero-cost macros.
- AI emits `@memcpy r0, r1, 256` (one token)
- Expands at assembly time to optimized loop
- Zero runtime overhead
- Guaranteed correct on first try

**Tier 2 (Extensions)**: For crypto and security-critical code:
- Silent bugs are dangerous (can't iterate to correctness)
- Timing attacks require constant-time implementations
- Users can add domain-specific logic in Rust

### Training Implications

| Component | Tokens to Learn | Training Examples |
|-----------|-----------------|-------------------|
| Core opcodes | 33 | 40,000 (existing) |
| Intrinsics | ~30 | 15,000 (new) |
| Extensions | ~10 (crypto) | 3,000 per extension |

**Cost**: +$30-50 over base training for intrinsics and extensions.

---

## Why Two-Tier Orchestration (LLM as Project Manager)

### The Problem

Complex tasks like "build a REST API with auth" are too large for the small model to handle in one shot. Traditional approaches:

| Approach | Problem |
|----------|---------|
| Train larger model | Expensive, slower inference |
| Use LLM for everything | No verification, slow |
| Single-tier iteration | Small model gets stuck on complex patterns |

### The Solution: LLM as Project Manager

The LLM **doesn't write code**. It decomposes tasks into subtasks the small model can handle:

```
User: "build REST API with auth"
                |
                v
        +--------------+
        |     LLM      |  "Break into subtasks"
        +--------------+
                |
                v
Subtasks:
1. "create HTTP server on port 8080"
2. "parse JSON from request body"
3. "hash password with SHA256"
4. "store user in hashmap"
5. "return JSON response"
                |
                v
        +--------------+
        | Small Model  |  Handles each subtask
        +--------------+  with verification loop
```

### Benefits

1. **100% verified output**: All code goes through verification loop
2. **Fast iteration**: Local model runs at 30ms, LLM only for planning
3. **Continuous learning**: Successful patterns captured for training
4. **Offline capability**: Ollama backend works without internet

---

## Why RAG-Based Extension Resolution

### The Problem

Training the model on specific extension IDs is expensive:

```
Traditional:
  Train on "json_parse" = ID 170           +$30-50
  Train on "http_get" = ID 190             +$30-50
  Train on "sha256" = ID 1                 +$30-50
  ... (50 extensions)                      +$1,500-2,500
```

### The Solution: Intent Description + RAG

Train the model to describe what it needs in natural language:

```
Model outputs: EXT.CALL @"parse JSON string", r0, r1
                         |
                         | RAG lookup at assembly time
                         v
Resolved:       EXT.CALL 170, r0, r1

Training cost: ONE pattern ($20-30), not 50 extensions
```

### Benefits

1. **Zero training cost for extensions**: RAG resolves them
2. **Self-documenting code**: `@"parse JSON"` vs `170`
3. **Future-proof**: New extensions work immediately
4. **User extensions**: Custom extensions work without retraining

---

## Why Three-Layer Extension Architecture

### The Question

How should we handle code beyond the 32 core opcodes?

| Layer | Training Cost | Resolution |
|-------|---------------|------------|
| **Stdlib** (Vec, HashMap, String) | $50-100 | Trained |
| **Bundled** (JSON, HTTP, Crypto) | $0 | RAG |
| **User** (custom extensions) | $0 | RAG |

### Why This Split?

**Stdlib is trained** because:
- Used in literally every program
- Model should emit these "instinctively"
- Only ~42 operations (cheap to train)

**Extensions use RAG** because:
- Adding extensions shouldn't require retraining
- Users can add custom extensions immediately
- Cost scales to $0 regardless of extension count

---

## Summary: Design Philosophy

```
+-----------------------------------------------------------------------+
|                     Neurlang Design Philosophy                         |
+-----------------------------------------------------------------------+

  1. MINIMAL VOCABULARY
     AI learns 32 tokens, mode bits expand to 102+ operations

  2. TWO-TIER ORCHESTRATION
     LLM decomposes tasks, small model generates verified code

  3. THREE-LAYER EXTENSIONS
     Stdlib (trained), Bundled (RAG), User (RAG) = minimal training cost

  4. RAG FOR EXTENSIBILITY
     Model describes intent, RAG resolves to extension ID

  5. IMPLICIT SECURITY
     Fat pointers embed bounds, every access is checked

  6. NATIVE EXECUTION
     No interpreter, no VM - direct CPU execution

  7. SIMPLE COMPILER
     Copy-and-patch: lookup + memcpy + patch, that's it

  8. PLATFORM ABSTRACTION
     Runtime library handles I/O, same code runs everywhere

  9. DETERMINISTIC INFRASTRUCTURE
     AI does what it's good at (generation), Rust does the rest

  10. VERIFIED OUTPUT
      100% of code goes through the verification loop
```

---

## See Also

- [How It Works](./architecture/how-it-works.md)
- [Architecture Overview](./architecture/overview.md)
- [Two-Tier Orchestration](./architecture/two-tier-orchestration.md)
- [Three-Layer Architecture](./architecture/three-layers.md)
- [RAG-Based Extension Resolution](./architecture/rag-extensions.md)
- [Training Costs](./training/costs.md)
- [Stencil System](./stencil/README.md)
- [Runtime Library](./runtime/README.md)
