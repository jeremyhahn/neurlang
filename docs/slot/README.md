# Slot-Based Code Generation

This document describes Neurlang's slot-based code generation architecture, which enables:

- **Offline operation** - No LLM dependency for core functionality
- **1000x faster iteration** - 30ms per attempt vs 3-30s with LLMs
- **Zero token cost** - Local small model inference

## Architecture Overview

```
+---------------------------------------------------------------------------+
|                         USER REQUEST                                       |
|  "Build an SMTP server with recipient validation"                          |
+---------------------------------------------------------------------------+
                              |
                              v
+---------------------------------------------------------------------------+
|  STAGE 1: DECOMPOSITION (Rule-Based OR LLM, ~1-5ms or ~2s)                 |
|                                                                            |
|  +----------------------+         +----------------------+                 |
|  |   RULE-BASED PATH   |   OR    |     LLM PATH         |                 |
|  |   (Offline, Fast)   |         |   (Flexible)         |                 |
|  |                     |         |                      |                 |
|  |  Intent Parser      |         |  LLM decomposes      |                 |
|  |  -> Spec Lookup     |         |  -> Outputs SlotSpec |                 |
|  |  -> Template Expand |         |                      |                 |
|  +----------------------+         +----------------------+                 |
|                   |                       |                                |
|                   +-----------+-----------+                                |
|                               |                                            |
|                               v                                            |
|                    +---------------------+                                 |
|                    |  SlotSpec (Common)  |                                 |
|                    |  - Skeleton code    |                                 |
|                    |  - N typed slots    |                                 |
|                    |  - Test cases       |                                 |
|                    +---------------------+                                 |
+---------------------------------------------------------------------------+
                              |
                              v
+---------------------------------------------------------------------------+
|  STAGE 2: SLOT FILLING (Small Model, PARALLEL)                             |
|                                                                            |
|  +-------------------------------------------------------------+          |
|  |  PARALLEL SLOT GENERATION                                   |          |
|  |                                                             |          |
|  |  All N slots generated SIMULTANEOUSLY:                      |          |
|  |  +---------+ +---------+ +---------+ +---------+           |          |
|  |  | SLOT 1  | | SLOT 2  | | SLOT 3  | | SLOT N  |           |          |
|  |  | 50-200  | | 50-200  | | 50-200  | | 50-200  |           |          |
|  |  | instrs  | | instrs  | | instrs  | | instrs  |           |          |
|  |  +----+----+ +----+----+ +----+----+ +----+----+           |          |
|  |       |           |           |           |                |          |
|  |       +-----------+-----------+-----------+                |          |
|  |                       |                                    |          |
|  |                       v                                    |          |
|  |  +-----------------------------------------------------+   |          |
|  |  |              BATCH VERIFICATION                      |   |          |
|  |  |  Run all slot unit tests in parallel (~10ms total)   |   |          |
|  |  +-----------------------------------------------------+   |          |
|  |                       |                                    |          |
|  |           +-----------+-----------+                        |          |
|  |           |                       |                        |          |
|  |       All Pass               Some Fail                     |          |
|  |           |                       |                        |          |
|  |           v                       v                        |          |
|  |       ACCEPT              Regenerate ONLY failed slots     |          |
|  |                           (up to 100 attempts per slot)    |          |
|  +-------------------------------------------------------------+          |
|                                                                            |
|  PERFORMANCE: ~50ms for entire program (not per slot!)                     |
+---------------------------------------------------------------------------+
                              |
                              v
+---------------------------------------------------------------------------+
|  STAGE 3: ASSEMBLY (Deterministic, ~1ms)                                   |
|  - Combine skeleton + filled slots                                         |
|  - Resolve labels                                                          |
|  - Output complete .nl program                                             |
+---------------------------------------------------------------------------+
                              |
                              v
+---------------------------------------------------------------------------+
|  STAGE 4: INTEGRATION TEST (From spec, ~10ms)                              |
|  - Run against spec test cases                                             |
|  - If fail: identify failing slot, retry Stage 2                           |
+---------------------------------------------------------------------------+
                              |
                              v
+---------------------------------------------------------------------------+
|  OUTPUT: Verified working program                                          |
+---------------------------------------------------------------------------+
```

## Key Concepts

### SlotSpec

The universal intermediate format that both rule-based and LLM paths produce:

```rust
pub struct SlotSpec {
    pub name: String,
    pub description: String,
    pub data_items: Vec<DataItem>,    // Constants, buffers
    pub skeleton: String,              // Assembly with {{SLOT_N}} markers
    pub slots: Vec<Slot>,              // Slots to be filled
    pub tests: Vec<TestCase>,          // Integration tests
}

pub struct Slot {
    pub id: String,                    // "SLOT_1", "SLOT_2", etc.
    pub slot_type: SlotType,           // One of 20 slot types
    pub context: SlotContext,          // Available registers, labels
    pub unit_test: Option<SlotTest>,   // Per-slot verification
}
```

### Slot Types

There are 20 slot types organized into categories:

| Category | Types | Instructions/Slot |
|----------|-------|-------------------|
| String/Pattern | PatternMatch, PatternSwitch, ResponseBuilder, StringCompare, StringCopy | 50-150 |
| Numeric | IntToString, StringToInt, RangeCheck | 20-50 |
| Control Flow | StateCheck, StateTransition, LoopUntil | 20-50 |
| I/O | SendResponse, ReadUntil, ReadNBytes | 30-80 |
| Extension | ExtensionCall, ValidationHook | 10-30 |
| Error | ErrorResponse | 20-40 |
| Data | BufferWrite, BufferRead | 10-30 |

See [slot-types.md](./slot-types.md) for complete documentation.

### Protocol Specifications

Protocols are defined in YAML files that describe:
- States and transitions
- Commands and their patterns
- Response formats
- Error handling
- Test cases

See [protocol-specs.md](./protocol-specs.md) for the YAML format reference.

## Performance Architecture

### Why 50-200 Instructions Per Slot?

The constraint is accuracy, not capability:

| Slot Size | Accuracy | Use Case |
|-----------|----------|----------|
| 20-50 instructions | ~98% | Simple patterns (compare, copy) |
| 50-100 instructions | ~95% | Medium patterns (parse command) |
| 100-200 instructions | ~90% | Complex patterns (full handler) |
| 200+ instructions | ~80% | Entire functions (not recommended) |

**Strategy**: Adaptive slot sizing based on complexity:
- Simple slot types (StateCheck, SendResponse) -> 20-50 instructions
- Medium slot types (PatternMatch, ResponseBuilder) -> 50-100 instructions
- Complex slot types (PatternSwitch, ValidationHook) -> 100-200 instructions

### Parallel Generation

All slots are generated in a single forward pass:

```
BATCH INPUT: [slot_1_context, slot_2_context, ..., slot_N]
                              |
                              v
                    +------------------+
                    |   Small Model    |
                    |   Single Pass    |
                    |    (~50ms)       |
                    +------------------+
                              |
                              v
BATCH OUTPUT: [slot_1_code, slot_2_code, ..., slot_N_code]

Total time: ~50ms for ALL slots (not per slot)
```

### Caching Strategy

Many slot types are IDENTICAL across programs:

```
PatternMatch("HELO {domain}") -> Always same code
PatternMatch("QUIT") -> Always same code
StateCheck([STATE_A, STATE_B]) -> Deterministic
SendResponse(socket, buffer) -> Identical pattern

CACHE KEY: hash(slot_type + slot_params)
CACHE HIT: ~0.1ms (vs 50ms generation)

Expected hit rate after warmup: 60-80%
```

### Performance Summary

| Scenario | Time | Notes |
|----------|------|-------|
| Cold start (no cache) | ~100ms | Generate all slots parallel |
| Warm (cache hits) | ~20ms | Only regenerate misses |
| Retry on failure | ~30ms | Only failed slots |
| **Total for typical program** | **<200ms** | With verification |

**Comparison**:
- LLM approach: 3-30 seconds per attempt
- Slot-based (cold): ~200ms per attempt
- Slot-based (warm): ~50ms per attempt
- **Speedup: 15-600x**

## Usage

### Generate Command

```bash
# Generate from protocol spec (offline, fast)
nl generate "SMTP server" --offline
nl generate "HTTP server with REST API"

# Generate with LLM decomposition (for novel requests)
nl generate "custom binary protocol with heartbeat" --llm

# Show generated SlotSpec without filling (dry run)
nl generate "SMTP server" --dry-run

# Benchmark generation time
nl generate "HTTP server" --benchmark
```

### Verification Commands

```bash
# Validate protocol spec structure
nl protocol -i specs/protocols/smtp.json --validate
# Output: "Spec 'smtp': ✓ VALID (6 states, 8 commands, 4 tests)"

# Run integration tests from spec
nl protocol -i specs/protocols/smtp.json --test --program smtp_server.nl

# Show spec statistics with verbose output
nl protocol -i specs/protocols/smtp.json --stats --verbose

# Test stdlib correctness (Rust == .nl output)
nl stdlib verify

# Run all @test cases in examples
nl test -p examples
```

See [verification.md](./verification.md) for complete verification documentation.

### Training Data Format

Slot-level training data is stored in JSONL format:

```jsonl
{"slot_type":"PatternMatch","input":{"pattern":"HELO {domain}","input_reg":"r0","captures":[{"name":"domain","output_reg":"r3","capture_type":"Word"}],"match_label":"helo_match","no_match_label":"try_next"},"output":"load.b r1, [r0]\nmov r2, 0x48\nbne r1, r2, try_next\n..."}
{"slot_type":"ResponseBuilder","input":{"template":"250 Hello {domain}\\r\\n","variables":{"domain":"r3"},"output_reg":"r6","length_reg":"r7"},"output":"mov r0, r6\nmov r1, 0x32\nstore.b r1, [r0]\n..."}
```

## Files

| File | Purpose |
|------|---------|
| `src/slot/mod.rs` | Module exports |
| `src/slot/types.rs` | SlotType enum (20 types) |
| `src/slot/spec.rs` | SlotSpec struct |
| `src/slot/parser.rs` | JSON protocol spec parser |
| `src/slot/template.rs` | Template expander |
| `src/slot/intent.rs` | Intent parser (rule-based) |
| `src/slot/router.rs` | Rule vs LLM router |
| `src/slot/filler.rs` | Parallel slot filling |
| `src/slot/cache.rs` | Slot cache (LRU, persistence) |
| `src/slot/verifier.rs` | Per-slot verification |
| `src/slot/assembler.rs` | Skeleton + slot combiner |
| `src/slot/validator.rs` | Protocol spec validation |
| `src/slot/training.rs` | Training data extraction |
| `specs/protocols/*.json` | Protocol specifications |
| `templates/*.skeleton` | Code skeleton templates |

## See Also

- [Slot Types Reference](./slot-types.md)
- [Protocol Specification Format](./protocol-specs.md)
- [Training Data Format](./training-format.md)
- [Verification System](./verification.md) - How code correctness is verified
- [Glossary](../GLOSSARY.md) - All terms and concepts explained
- [CLI: nl generate](../cli/generate.md)
