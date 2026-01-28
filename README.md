**WARNING: This project is a R&D thought experiment!!!**

# Neurlang

**1000x faster AI code generation with guaranteed correctness**

Neurlang is an AI-optimized binary programming language designed for rapid automated iteration. A small local model (~50-100M params) generates 64 instructions in parallel, which are compiled to native x86-64 via copy-and-patch in ~5μs. The system iterates 100-1000 times in seconds to produce verified correct output.

## Why Neurlang?

Traditional AI coding tools are powerful but slow—each iteration requires a full cloud LLM forward pass. Neurlang inverts this:

| Aspect | Traditional AI Coding | Neurlang |
|--------|----------------------|----------|
| Model | Cloud LLM (API call) | Local (~50-100M params) |
| Output | Text code | Binary IR |
| Iteration speed | 5-30 seconds | ~30 milliseconds |
| Iterations | 3-5 visible | 100-1000 automated |
| Result | May have bugs | Verified correct |

**Key insight**: Instead of generating human-readable code once, generate machine-optimized code thousands of times until tests pass.

## Key Advantages

### Speed
- **Parallel instruction prediction** — 64 instructions per forward pass (~30ms)
- **Copy-and-patch compilation** — Binary IR to native x86-64 in <5μs
- **1000x faster iteration** — Milliseconds per cycle vs seconds for cloud LLMs
- **Local execution** — No API latency, runs entirely offline

### Safety
- **Memory-safe extensions** — Complex operations via verified Rust FFI
- **Capability-based security** — Fat pointers with automatic bounds checking
- **Sandboxed I/O** — Deny-by-default file and network permissions
- **Verified stdlib** — Rust implementations compiled to IR with auto-generated tests

### Intelligence
- **32-opcode binary IR** — Minimal vocabulary optimized for AI prediction
- **RAG-based extension resolution** — Model describes intent, system resolves to implementation
- **Single source of truth** — Extension IDs defined once, used everywhere
- **Comprehensive testing** — Mock-based testing for all extension calls

## Quick Start

### Build

```bash
# Build the nl binary
make build

# Verify the build
./target/release/nl --version
```

### Run Examples

```bash
# Run a program
./target/release/nl run -i examples/algorithm/factorial.nl

# Generate from natural language
./target/release/nl prompt "factorial of 10" --show-asm

# Test all examples
make examples
```

### Test

```bash
# Run unit tests
make test

# Run integration tests
make integration-test

# Run IR coverage on examples
make coverage-ir
```

## How It Works

```
User: "calculate factorial of n"
         │
         ▼
┌─────────────────────────────────────────────────────────────────┐
│  1. PARALLEL PREDICTION (~30ms)                                 │
│     Small model outputs 64 binary IR instructions               │
└─────────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────────┐
│  2. RAG EXTENSION RESOLUTION                                    │
│     Intent descriptions → Rust FFI extensions                   │
│     @"hash with SHA-256" → sha256 (ID 1)                        │
│     Uses ext_ids module as single source of truth               │
└─────────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────────┐
│  3. COPY-AND-PATCH COMPILE (<5μs)                               │
│     Pre-compiled stencils → Native x86-64                       │
└─────────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────────┐
│  4. EXECUTE & VERIFY                                            │
│     Run native code, check @test annotations                    │
│     If fail: regenerate (100-1000 iterations in seconds)        │
└─────────────────────────────────────────────────────────────────┘
         │
         ▼
Verified Result: 120
```

## Testing

Neurlang uses annotation-based testing embedded in `.nl` source files:

### Test Annotations

```asm
; @test: r0=5 -> r0=120     ; Input r0=5, expect output r0=120
; @test: r0=0 -> r0=1       ; Edge case: factorial(0) = 1
```

### Extension Mocking

Mock external extension calls to test code without real network/database:

```asm
; @mock: http_get=1                ; http_get returns handle 1
; @mock: http_response_status=200  ; returns 200 OK
; @mock: json_parse=1              ; json_parse returns handle 1
;
; @test: r0=0 -> r0=200
```

Extension names are resolved via RAG to their numeric IDs from `ext_ids`:
- `http_get` → ID 190
- `json_parse` → ID 170
- `tls_connect` → ID 500

See [docs/TESTING.md](docs/TESTING.md) for complete testing documentation.

## Extension System

Extensions provide Rust FFI for complex operations the model shouldn't generate from scratch:

| Range | Category | Examples |
|-------|----------|----------|
| 1-99 | Crypto | sha256, aes256, hmac, ed25519 |
| 100-139 | Collections | vec_new, hashmap_get |
| 140-169 | Strings | string_new, string_concat |
| 170-189 | JSON | json_parse, json_stringify |
| 190-209 | HTTP | http_get, http_post |
| 400-419 | Compression | zlib_compress, gzip_compress |
| 420-439 | Encoding | base64_encode, hex_encode |
| 440-459 | DateTime | datetime_now, datetime_parse |
| 460-479 | Regex | regex_match, regex_replace |
| 480-499 | FileSystem | fs_read, fs_write |
| 500-519 | TLS | tls_connect, tls_send |

All IDs are defined in `src/runtime/extensions/mod.rs` (ext_ids module) as the single source of truth.

## Training

### Quick Start

```bash
# Generate training data
make generate-data

# Train the model (requires GPU)
make train

# Export to ONNX for inference
make export-onnx
```

### Full Pipeline

```bash
make train-full  # generate-data + train + export-onnx
```

See [docs/training/README.md](docs/training/README.md) for detailed documentation including:
- GPU profiles (L40S, A100, H100)
- Remote training on cloud GPUs
- Curriculum learning strategies
- Model architecture details

## Project Structure

```
neurlang/
├── src/                    # Rust source code
│   ├── ir/                 # 32-opcode instruction representation
│   │   └── rag_resolver.rs # RAG-based extension resolution
│   ├── jit/                # Copy-and-patch compiler
│   ├── runtime/
│   │   └── extensions/     # Rust FFI extensions
│   │       └── mod.rs      # ext_ids - single source of truth
│   └── main.rs             # CLI entry point
│
├── examples/               # Hand-written examples with @test annotations
├── lib/                    # Generated stdlib (from stdlib/src/*.rs)
├── train/                  # Training scripts
├── docs/                   # Documentation
│   ├── TESTING.md          # Testing and mocking guide
│   └── training/           # Training documentation
└── Makefile                # Build system
```

## Performance

| Metric | Value |
|--------|-------|
| Model inference | ~30ms (64 instructions) |
| Compilation | <5μs (copy-and-patch) |
| End-to-end iteration | ~35ms |
| RAG lookup | ~1ms |
| Typical task completion | 1-10 seconds |

## Documentation

| Document | Description |
|----------|-------------|
| [Testing Guide](docs/TESTING.md) | Test annotations, mocking, coverage |
| [Architecture](docs/architecture/overview.md) | System design |
| [RAG Extensions](docs/architecture/rag-extensions.md) | Extension resolution |
| [Training](docs/training/README.md) | Model training guide |
| [CLI Commands](docs/cli/commands.md) | Full command reference |
| [IR Opcodes](docs/ir/opcodes.md) | 32-opcode reference |
| [Extensions](docs/extensions/bundled.md) | Built-in extension API |

## System Requirements

- **Rust**: 1.70+
- **Python**: 3.8+ (for training)
- **OS**: Linux (primary), macOS, Windows
- **Architecture**: x86-64 (primary), ARM64 (supported)

## Make Targets

```bash
make build          # Build nl binary
make test           # Run unit tests
make integration-test  # Run integration tests
make coverage       # Rust code coverage
make coverage-ir    # IR test coverage on examples
make examples       # Test all examples
make help           # Show all targets
```

## License

MIT
