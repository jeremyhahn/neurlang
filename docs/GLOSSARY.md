# Neurlang Glossary

A comprehensive glossary of terms, concepts, and jargon used in the Neurlang project.

## Core Concepts

### Neurlang
An AI-optimized binary programming language designed for 1000x faster code generation than traditional LLM approaches. The model generates binary IR directly, which is compiled to native x86-64 via copy-and-patch compilation in ~5 microseconds.

### IR (Intermediate Representation)
The 32-opcode binary instruction format that Neurlang uses internally. Each instruction is 4-8 bytes with fixed fields for opcode, mode, registers, and immediates. This is the "assembly language" of Neurlang.

### Copy-and-Patch Compilation
A compilation technique where pre-compiled code "stencils" are copied and patched with actual addresses/values at compile time. Achieves ~5μs compile times vs milliseconds-seconds for traditional compilers like LLVM.

### Stencil
A pre-compiled machine code template with placeholder "holes" that get filled in at compile time. Example: a loop stencil has holes for the loop body address and exit condition.

---

## Slot-Based Architecture

### Slot
A typed code fragment (20-200 instructions) that performs a specific operation. Slots are the building blocks of larger programs. Example: `PatternMatch`, `StateCheck`, `SendResponse`.

### SlotType
One of 20 predefined slot categories that describe what operation a slot performs:

| Category | Types |
|----------|-------|
| **String/Pattern** | PatternMatch, PatternSwitch, ResponseBuilder, StringCompare, StringCopy |
| **Numeric** | IntToString, StringToInt, RangeCheck |
| **Control Flow** | StateCheck, StateTransition, LoopUntil |
| **I/O** | SendResponse, ReadUntil, ReadNBytes |
| **Extension** | ExtensionCall, ValidationHook |
| **Error** | ErrorResponse |
| **Data** | BufferWrite, BufferRead |

### SlotSpec
The universal intermediate format that describes a program as:
- **Skeleton**: Assembly code template with `{{SLOT_N}}` placeholders
- **Slots**: List of typed slots to be filled by the model
- **Data Items**: Constants, strings, buffers for the data section
- **Tests**: Integration test cases

```rust
SlotSpec {
    name: "smtp_server",
    skeleton: "main:\n  {{SLOT_1}}\n  {{SLOT_2}}\n  halt",
    slots: [Slot { id: "SLOT_1", slot_type: PatternMatch {...} }, ...],
    tests: [TestCase { send: "HELO", expect: "250" }],
}
```

### SlotContext
The execution context available to a slot:
- **Registers**: Which registers contain what values
- **Labels**: What labels can be jumped to
- **Data refs**: What data section items are accessible

### Skeleton
Assembly code template with `{{SLOT_N}}` markers where generated slot code gets inserted. The skeleton provides the overall program structure; slots provide the implementation details.

### Slot Filling
The process of generating assembly code for each slot in a SlotSpec. Done by a small trained model (~5-25M parameters) that specializes in generating code for typed slots.

### Slot Cache
An LRU cache that stores generated slot code. Many slots are identical across programs (e.g., `PatternMatch("QUIT")` always generates the same code), so caching provides 60-80% hit rates.

---

## Protocol System

### Protocol Spec
A JSON file that defines a text-based protocol (SMTP, HTTP, Redis, etc.):
- **States**: The state machine (INIT, GREETED, DATA, etc.)
- **Commands**: Patterns to match and how to handle them
- **Errors**: Standard error responses
- **Tests**: Send/expect test cases

```json
{
  "name": "smtp",
  "states": [{"name": "INIT", "initial": true}, ...],
  "commands": [{"name": "HELO", "pattern": "HELO {domain}", ...}],
  "tests": [{"send": "HELO test.com", "expect": "250 Hello"}]
}
```

### Pattern
A string template for matching input. Uses `{name}` for captures and `{name:type}` for typed captures:
- `HELO {domain}` - captures "domain" as a word
- `MAIL FROM:<{sender:until:>}>` - captures until `>`

### Capture
A named value extracted from a pattern match. Stored in a register for use in response building or validation.

### Handler
The code that executes when a command matches. Types include:
- `simple_response` - send a fixed response
- `multi_line_response` - send multiple lines
- `validated_response` - validate before responding
- `multiline_reader` - read until terminator

### State Machine
The finite state machine that governs protocol flow. Commands are only valid in certain states, and handlers transition to new states.

---

## Training System

### 64-Slot Parallel Generation
The original Neurlang training approach where the model generates 64 instruction slots in parallel. Used for small complete programs (3-64 instructions).

### Slot Training Data
JSONL format training data for slot-based generation:
```jsonl
{"slot_type": "PatternMatch", "input": {...}, "output": "assembly code"}
```

### RAG (Retrieval-Augmented Generation)
The system that maps natural language intent descriptions to extension IDs. When the model outputs `ext.call @"parse JSON"`, RAG resolves it to `ext.call 200`.

### Embedding
A vector representation of text used for semantic search. Neurlang uses embeddings to find relevant extensions and code examples.

---

## Stdlib and Extensions

### Stdlib (Standard Library)
113 pre-built functions in `lib/` covering:
- **math**: factorial, fibonacci, gcd, power
- **string**: strlen, strcmp, strcpy, atoi, itoa
- **array**: sum, min, max, sort, search
- **float**: sqrt, abs, floor, ceil
- **bitwise**: popcount, clz, ctz, rotate

Generated from Rust source in `stdlib/src/*.rs` via `nl stdlib build`.

### Extension
A capability exposed to Neurlang programs via the `ext.call` instruction. Extensions are implemented in safe Rust and provide:
- Crypto operations (SHA256, AES)
- JSON parsing
- HTTP client
- Database access
- File I/O

### Extension Registry
The runtime registry that maps extension IDs to implementations. Extensions are registered at startup and called via ID.

### Tier 0/1/2 Architecture
The three-tier system for code reuse:
- **Tier 0**: Core opcodes - AI writes from scratch
- **Tier 1**: Intrinsics - macro tokens that expand to optimized IR
- **Tier 2**: Extensions - complex operations in safe Rust

---

## Verification System

### @test Annotation
Test case annotation in `.nl` files:
```asm
; @test: r0=5 -> r0=120
```
Means: if r0=5 on input, expect r0=120 on output.

### @server Annotation
Marks a program as a server that can't be unit tested (needs network):
```asm
; @server: true
```

### Spec Validator
Tool that validates protocol specs for correctness:
- All states reachable from initial
- No undefined state references
- Valid pattern syntax
- Test coverage

### Slot Verifier
Per-slot verification that runs each slot's unit test to ensure generated code is correct before assembly.

### Integration Test
End-to-end test defined in protocol spec's `tests` section. Runs send/expect sequences against the generated server.

---

## Compilation Pipeline

### Assembler
Converts `.nl` assembly text to binary IR. Handles labels, data sections, and instruction encoding.

### Disassembler
Converts binary IR back to readable assembly text for debugging.

### Interpreter
Executes IR directly without compilation. Used for small programs (<1000 instructions) and debugging.

### JIT (Just-In-Time) Compiler
Compiles IR to native x86-64 using copy-and-patch. Used for larger programs requiring full speed.

### Program
The in-memory representation of compiled code:
- `instructions`: Vector of decoded instructions
- `data`: Data section bytes
- `labels`: Map of label names to addresses

---

## Code Generation Flow

### Intent Parser
Rule-based system that detects what the user wants from their prompt:
- "SMTP server" → SMTP protocol spec
- "HTTP API" → HTTP protocol spec + REST template

### Router
Decides whether to use rule-based (fast, offline) or LLM (flexible, slower) code generation based on the request.

### Template Expander
Converts a ProtocolSpec into a SlotSpec by:
1. Generating the skeleton from a template
2. Creating slots for each command handler
3. Setting up data items
4. Converting spec tests to test cases

### Slot Assembler
Combines skeleton + filled slots into complete assembly:
1. Replace `{{SLOT_N}}` markers with generated code
2. Generate data section
3. Resolve labels
4. Output complete `.nl` file

---

## File Types

### .nl
Neurlang assembly source file. Human-readable text format.

### .nlb
Neurlang binary file. Compiled IR ready for execution.

### .jsonl
JSON Lines format used for training data. One JSON object per line.

### Protocol Spec (.json)
Protocol definition file in `specs/protocols/`.

### Skeleton (.skeleton)
Template file for code generation in `templates/`.

---

## Performance Terms

### Cold Start
First generation with empty cache. Takes ~100-200ms.

### Warm Start
Generation with cached slots. Takes ~20-50ms.

### Cache Hit Rate
Percentage of slot lookups found in cache. Target: 60-80% after warmup.

### Token
In LLM context, a unit of text (~4 characters). Neurlang's binary output means no tokens, just bytes.

### Compile Time
Time to convert IR to native code. Neurlang: ~5μs. LLVM: 10ms-10s.

---

## Register Conventions

| Register | Purpose |
|----------|---------|
| `r0` | Return value, primary accumulator |
| `r0-r3` | Function arguments |
| `r4-r9` | Caller-saved temporaries |
| `r10` | Server socket FD |
| `r11` | Client socket FD |
| `r12` | Request/response length |
| `r13` | Protocol state |
| `r14` | Database handle |
| `r15` | General temporary |
| `r16-r31` | General purpose |
| `zero` | Constant 0 (read-only) |

---

## Common Abbreviations

| Abbrev | Meaning |
|--------|---------|
| IR | Intermediate Representation |
| JIT | Just-In-Time (compilation) |
| LLM | Large Language Model |
| RAG | Retrieval-Augmented Generation |
| FD | File Descriptor |
| FPU | Floating Point Unit |
| ONNX | Open Neural Network Exchange (model format) |
| JSONL | JSON Lines (one JSON per line) |
| LRU | Least Recently Used (cache eviction) |
| TTL | Time To Live (cache expiration) |

---

## See Also

- [Architecture Overview](./architecture/)
- [Slot System](./slot/README.md)
- [CLI Reference](./cli/README.md)
- [Training Guide](../TRAINING_GUIDE.md)
