# CLI Commands Reference

Complete reference for all CLI commands.

## Quick Reference

| Command | Description |
|---------|-------------|
| `nl asm` | Assemble text source to binary IR |
| `nl disasm` | Disassemble binary IR to text |
| `nl run` | Execute a program |
| `nl compile` | Compile to standalone native code |
| `nl bench` | Run benchmarks |
| `nl spec` | Show IR specification |
| `nl prompt` | Generate code from natural language |
| `nl generate` | **NEW** Generate programs via slot-based architecture |
| `nl train` | Train the AI model |
| `nl datagen` | Generate training data |
| `nl accuracy` | Test model accuracy |
| `nl export-onnx` | Export PyTorch model to ONNX |
| `nl agent` | Interactive AI agent with sessions |
| `nl extension` | Manage extensions (Go-style packages) |
| `nl config` | Manage configuration settings |
| `nl backends` | Manage LLM backends |
| `nl stdlib` | Build stdlib from Rust sources |
| `nl resolve` | Resolve intent descriptions to extension IDs |
| `nl crate` | Manage crate extensions |
| `nl test` | Run tests on .nl files |
| `nl index` | Build and manage RAG intent indices |

---

## asm

Assemble text source to binary IR.

```
nl asm [OPTIONS] -i <INPUT>
```

### Options

| Option | Description |
|--------|-------------|
| `-i, --input <FILE>` | Input assembly file (required) |
| `-o, --output <FILE>` | Output binary file (default: input.nlb) |
| `-d, --disasm` | Show disassembly after assembly |

### Examples

```bash
# Basic assembly
nl asm -i program.nl

# Specify output file
nl asm -i program.nl -o program.nlb

# Show disassembly
nl asm -i program.nl --disasm
```

---

## disasm

Disassemble binary IR to text.

```
nl disasm [OPTIONS] -i <INPUT>
```

### Options

| Option | Description |
|--------|-------------|
| `-i, --input <FILE>` | Input binary file (required) |
| `--offsets` | Show byte offsets |
| `--bytes` | Show raw bytes |

---

## run

Execute a program.

```
nl run [OPTIONS] -i <INPUT>
```

### Options

| Option | Description |
|--------|-------------|
| `-i, --input <FILE>` | Input file (assembly or binary) |
| `--interp` | Use interpreter instead of JIT |
| `-s, --stats` | Show execution statistics |
| `--max-instr <N>` | Maximum instructions (default: 1000000) |
| `-w, --workers <N>` | Number of worker threads (0 = single-threaded) |
| `--strategy <S>` | Worker strategy: auto, reuseport, shared |

### Examples

```bash
# Run assembly file
nl run -i factorial.nl

# Run with statistics
nl run -i factorial.nl --stats

# Multi-worker server
nl run -i server.nl --workers 4
```

---

## compile

Compile to standalone native code.

```
nl compile [OPTIONS] -i <INPUT> -o <OUTPUT>
```

### Options

| Option | Description |
|--------|-------------|
| `-i, --input <FILE>` | Input file |
| `-o, --output <FILE>` | Output binary file |
| `--format <FORMAT>` | Output format: raw, elf (default: raw) |

---

## bench

Run benchmarks.

```
nl bench [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `-b, --bench-type <TYPE>` | Benchmark type: compile, fib, all (default: all) |
| `-i, --iterations <N>` | Number of iterations (default: 1000) |

---

## spec

Show IR specification.

```
nl spec [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--opcodes` | Show opcodes |
| `--registers` | Show registers |
| `--all` | Show all |

---

## prompt

Generate code from natural language prompt.

```
nl prompt [OPTIONS] <PROMPT>
```

### Options

| Option | Description |
|--------|-------------|
| `--model <PATH>` | Model path (default: ./model.onnx) |
| `--engine <ENGINE>` | Inference engine: ort, tract, candle, burn, auto |
| `--show-asm` | Show generated assembly |
| `--max-retries <N>` | Maximum retry attempts (default: 3) |
| `-o, --output <FILE>` | Save generated binary to file |
| `-v, --verbose` | Verbose output |

### Examples

```bash
# Generate code
nl prompt "add 5 and 3"

# Show assembly
nl prompt "factorial of 10" --show-asm

# Save to file
nl prompt "fibonacci sequence" -o fib.nlb
```

---

## generate

Generate complete programs using slot-based code generation.

This command uses the slot-based architecture for 15-600x faster generation compared to LLM-based approaches. See [generate.md](generate.md) and [Slot Architecture](../slot/README.md) for details.

```
nl generate [OPTIONS] <PROMPT>
```

### Options

| Option | Description |
|--------|-------------|
| `-o, --output <FILE>` | Output file path |
| `--show-spec` | Show SlotSpec before filling |
| `--show-slots` | Show each filled slot |
| `--show-asm` | Show final assembly |
| `--offline` | Use only rule-based decomposition (no LLM) |
| `--llm` | Force LLM decomposition |
| `--backend <NAME>` | LLM backend: claude, ollama |
| `--no-cache` | Disable slot cache |
| `--max-retries <N>` | Max retries per slot (default: 100) |
| `--benchmark` | Show timing breakdown |
| `--dry-run` | Show SlotSpec without filling |

### Examples

```bash
# Generate SMTP server (auto-detects protocol spec)
nl generate "SMTP server"

# Generate HTTP REST API
nl generate "HTTP REST API" -o my_api.nl

# Offline-only generation
nl generate "FTP server" --offline

# Force LLM for novel requests
nl generate "custom binary protocol" --llm

# Show timing breakdown
nl generate "HTTP server" --benchmark

# Dry run (show spec only)
nl generate "Redis server" --dry-run
```

### Performance

| Scenario | Time |
|----------|------|
| Offline with cache | ~20ms |
| Offline cold | ~100ms |
| LLM decomposition | ~3-5s |

See [generate.md](generate.md) for complete documentation.

---

## train

Train the AI model.

```
nl train [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--data <PATH>` | Training data path (default: training_data.jsonl) |
| `-o, --output <PATH>` | Output model path (default: model.onnx) |
| `--remote <HOST>` | Remote host for GPU training (user@hostname) |
| `--remote-dir <DIR>` | Remote working directory (default: ~/neurlang) |
| `--profile <PROFILE>` | GPU profile: h100, h200, a100, l40s, cpu |
| `--backend <BACKEND>` | Training backend: auto, native, docker, pytorch |
| `--epochs <N>` | Number of epochs (default: 20) |
| `--patience <N>` | Early stopping patience (default: 5) |
| `--cross-validate` | Run k-fold cross-validation |
| `--folds <N>` | Number of CV folds (default: 5) |
| `--list-profiles` | List available GPU profiles |
| `-v, --verbose` | Verbose output |

### Examples

```bash
# Local training
nl train --data training_data.jsonl --epochs 50

# Remote GPU training
nl train --remote user@gpu-server --profile h100

# Cross-validation
nl train --cross-validate --folds 5
```

---

## datagen

Generate training data.

```
nl datagen [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `-o, --output <PATH>` | Output file (default: training_data.jsonl) |
| `-n, --num-examples <N>` | Number of examples (default: 100000) |
| `-l, --level <N>` | Curriculum level 1-5 (default: 5) |
| `--seed <N>` | Random seed (default: 42) |
| `--include-examples` | Include examples from examples/ directory |

### Examples

```bash
# Generate 100K examples
nl datagen -o train.jsonl -n 100000

# Include example programs
nl datagen --include-examples
```

---

## accuracy

Test model accuracy.

```
nl accuracy [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--model <PATH>` | Model path (default: ./model.onnx) |
| `--benchmark` | Use benchmark test suite |
| `--test-data <PATH>` | Test data path |
| `-v, --verbose` | Verbose output |

### Examples

```bash
# Test accuracy
nl accuracy --model model.onnx --benchmark

# Test with custom data
nl accuracy --test-data test.jsonl
```

---

## export-onnx

Export PyTorch model to ONNX format.

```
nl export-onnx [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `-i, --input <PATH>` | Input PyTorch model (default: model.pt) |
| `-o, --output <PATH>` | Output ONNX model (default: model.onnx) |
| `-v, --verbose` | Verbose output |

---

## agent

Interactive AI agent with session persistence.

```
nl agent [OPTIONS] [TASK]
```

### Options

| Option | Description |
|--------|-------------|
| `--new <NAME>` | Start a new session with the given task |
| `--cont <ID>` | Continue an existing session |
| `--resume` | Resume the last session |
| `--list` | List all sessions |
| `--interactive` | Interactive mode (default) |
| `--asm` | Assembly REPL mode (raw assembly execution) |
| `--max-iterations <N>` | Maximum iterations per request (default: 1000) |
| `-v, --verbose` | Verbose output |

### Examples

```bash
# Start new session
nl agent --new "build a REST API with auth"

# Continue session
nl agent --cont abc123 "add rate limiting"

# Resume after crash
nl agent --resume

# List sessions
nl agent --list

# Assembly REPL mode
nl agent --asm
```

See [agent.md](agent.md) for detailed documentation.

---

## extension

Manage extensions (Go-style package system).

```
nl extension [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--add <URL>` | Install extension from git URL |
| `--new <NAME>` | Create a new local extension |
| `--remove <PATH>` | Remove an installed extension |
| `--list` | List all installed extensions |
| `--load <FILE>` | Load extension from file for testing |
| `--info <PATH>` | Show extension details |

### Examples

```bash
# Install from git
nl extension --add github.com/user/csv-parser
nl extension --add github.com/user/csv-parser@v1.2.0

# Create local extension
nl extension --new my-utils

# List installed
nl extension --list

# Show info
nl extension --info local/my-utils
```

See [extension.md](extension.md) for detailed documentation.

---

## config

Manage neurlang configuration settings.

```
nl config [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--set <KEY> <VALUE>` | Set a configuration value |
| `--get <KEY>` | Get a configuration value |
| `--unset <KEY>` | Remove a configuration value |
| `--list` | List all configuration values |
| `--path` | Show configuration file path |

### Configuration Keys

| Key | Description | Example |
|-----|-------------|---------|
| `backends.default` | Default LLM backend | `ollama` or `claude` |
| `backends.claude.api_key` | Claude API key | `sk-ant-...` |
| `backends.ollama.host` | Ollama host URL | `http://localhost:11434` |
| `backends.ollama.model` | Ollama model | `llama3` |

### Examples

```bash
# Set default backend
nl config --set backends.default ollama

# Set Claude API key
nl config --set backends.claude.api_key sk-ant-xxxxx

# Get a value
nl config --get backends.default

# List all settings
nl config --list

# Show config file path
nl config --path
```

Configuration is stored in `~/.neurlang/config.json`.

---

## backends

Manage LLM backends for two-tier orchestration.

```
nl backends [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--list` | List all available backends |
| `--status <NAME>` | Check status of a specific backend |
| `--set-default <NAME>` | Set the default backend |
| `--test <NAME>` | Test a backend connection |

### Available Backends

| Backend | Description | Requirements |
|---------|-------------|--------------|
| `ollama` | Local LLM via Ollama | Ollama running locally |
| `claude` | Anthropic Claude API | API key configured |

### Examples

```bash
# List backends
nl backends --list

# Check Ollama status
nl backends --status ollama

# Set default backend
nl backends --set-default ollama

# Test Claude connection
nl backends --test claude
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `OLLAMA_HOST` | Ollama host URL (default: `http://localhost:11434`) |
| `OLLAMA_MODEL` | Ollama model (default: `llama3`) |
| `ANTHROPIC_API_KEY` | Claude API key |

---

## stdlib

Build standard library from Rust sources.

The `stdlib` command compiles Rust source files in `stdlib/src/` to Neurlang assembly files in `lib/`. This is the primary way to generate verified stdlib implementations with auto-generated test annotations.

```
nl stdlib [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--build` | Build all stdlib modules from Rust sources |
| `--verify` | Verify Rust output matches Neurlang output |
| `--config <PATH>` | Path to neurlang.toml configuration |
| `-v, --verbose` | Verbose output showing compilation progress |

### Configuration

The build is controlled by `neurlang.toml` in the project root:

```toml
[stdlib]
math = true         # factorial, fibonacci, gcd, power, is_prime
float = true        # FPU operations (sqrt, abs, floor, ceil)
string = true       # strlen, strcmp, strcpy, atoi, itoa
array = true        # sum, min, max, search, sort
bitwise = true      # popcount, clz, ctz, rotl, rotr
collections = true  # stack, queue, hashtable

[build]
include_comments = true
generate_tests = true
max_instructions = 1000
output_dir = "lib"
```

### Examples

```bash
# Build all enabled stdlib modules
nl stdlib --build

# Build with verbose output
nl stdlib --build --verbose

# Verify Rust == Neurlang output
nl stdlib --verify

# Use custom config
nl stdlib --build --config ./custom.toml
```

### Output

```
$ nl stdlib --build --verbose

[1/4] Parsing stdlib/src/*.rs...
      Found 77 functions across 6 modules

[2/4] Compiling to Neurlang IR...
      math/factorial.nl: 12 instructions
      math/fibonacci.nl: 18 instructions
      ...

[3/4] Generating tests...
      Generated 215 @test annotations

[4/4] Writing lib/...
      Created 77 files in lib/

Done. Run 'nl test -p lib' to verify.
```

### Pipeline

```
stdlib/src/*.rs (Rust source - you write this)
       ↓
   nl stdlib --build
       ↓
lib/*.nl (generated Neurlang assembly)
       ↓
Training data generator
       ↓
Model learns stdlib patterns
```

---

## resolve

Resolve intent descriptions to extension IDs via RAG.

The `resolve` command performs semantic search to find matching extensions for a natural language description. This is the same resolution that happens automatically when using `@"description"` syntax in assembly.

```
nl resolve [OPTIONS] <INTENT>
```

### Options

| Option | Description |
|--------|-------------|
| `-n, --top <N>` | Number of results to show (default: 5) |
| `--threshold <SCORE>` | Minimum similarity threshold (0.0-1.0) |

### Examples

```bash
# Find extension for JSON parsing
nl resolve "parse JSON string"

# Show more results
nl resolve "compress data" --top 10

# With threshold filter
nl resolve "hash password" --threshold 0.8
```

### Output

```
$ nl resolve "parse JSON string"

Top 5 matches for "parse JSON string":

  1. json_parse (ID: 200)
     Description: Parse JSON string to structured data
     Similarity: 0.94

  2. json_validate (ID: 201)
     Description: Validate JSON string format
     Similarity: 0.78

  3. json_get_field (ID: 202)
     Description: Get field value from JSON object
     Similarity: 0.65
  ...
```

### Usage in Assembly

```asm
; Use resolved extension by intent
ext.call @"parse JSON string", r0, r1

; Assembler calls RAG resolver internally:
; "parse JSON string" → json_parse → ID 200

; Equivalent to:
ext.call 200, r0, r1
```

---

## crate

Manage Rust crate extensions.

The `crate` command allows you to add, remove, and list Rust crates from crates.io as Neurlang extensions. Added crates are automatically registered with RAG for intent-based resolution.

```
nl crate [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--add <NAME>` | Add a crate from crates.io |
| `--remove <NAME>` | Remove an installed crate |
| `--list` | List all installed crate extensions |
| `--update <NAME>` | Update a crate to latest version |
| `--build` | Rebuild all crate extensions |

### Configuration

Crate extensions are configured in `neurlang.toml`:

```toml
[extensions.crates]
serde_json = { version = "1.0", features = ["std"] }
regex = { version = "1.10" }
chrono = { version = "0.4" }
base64 = { version = "0.21" }
uuid = { version = "1.0", features = ["v4"] }
```

### Examples

```bash
# Add serde_json crate
nl crate --add serde_json

# Add with specific version
nl crate --add "regex@1.10"

# List installed crates
nl crate --list

# Remove a crate
nl crate --remove chrono

# Rebuild all crate extensions
nl crate --build
```

### Output

```
$ nl crate --add serde_json

[1/5] Fetching serde_json from crates.io...
[2/5] Parsing public API (12 functions found)...
[3/5] Generating FFI wrappers...
[4/5] Building and linking...
[5/5] Registering with RAG (IDs 500-511)...

Done. Extension available:
  @"parse JSON from string"  → serde_json:from_str (ID 500)
  @"serialize to JSON"       → serde_json:to_string (ID 501)
  ...
```

### Multi-Language Extensions

Beyond Rust crates, you can also configure Go packages and C libraries:

```toml
# Go packages (compiled with cgo)
[extensions.go]
golib = { package = "github.com/user/golib", tag = "v1.0.0" }

# C libraries (linked via system or vendored)
[extensions.c]
zlib = { version = "1.3", system = true }
openssl = { version = "3.0", system = true }

# Local extensions (auto-detect language from path)
[extensions.local]
myutils = { path = "./extensions/myutils" }
```

---

## test

Run tests on Neurlang assembly files.

The `test` command executes test annotations (`@test:`) in `.nl` files and reports pass/fail results. This is the primary way to verify that generated code is correct.

```
nl test [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `-p, --path <PATH>` | Directory or file to test |
| `-f, --filter <PATTERN>` | Only run tests matching pattern |
| `--timeout <MS>` | Test timeout in milliseconds (default: 1000) |
| `-v, --verbose` | Verbose output with timing info |
| `--json` | Output results as JSON |

### Test Annotation Format

```asm
; @test: inputs -> expected_outputs
; @test: r0=5 -> r0=120           ; Single input/output
; @test: r0=10, r1=3 -> r0=1      ; Multiple inputs
; @test: r0=0 -> r0=1, r1=0       ; Multiple outputs
```

### Examples

```bash
# Test all files in examples/
nl test -p examples

# Test stdlib
nl test -p lib

# Test specific file
nl test -p lib/math/factorial.nl

# Filter tests by name
nl test -p examples --filter factorial

# Verbose with timing
nl test -p lib -v
```

### Output

```
$ nl test -p lib

Running tests in lib/...

  lib/math/factorial.nl
    ✓ r0=0 -> r0=1        (0.12ms)
    ✓ r0=5 -> r0=120      (0.15ms)
    ✓ r0=10 -> r0=3628800 (0.18ms)

  lib/math/fibonacci.nl
    ✓ r0=0 -> r0=0        (0.11ms)
    ✓ r0=10 -> r0=55      (0.25ms)
    ✗ r0=20 -> r0=6765    (expected 6765, got 0)

Results: 46 passed, 3 failed, 15 skipped
Time: 1.24s
```

### Skip Annotations

```asm
; @server: true     ; Skip this file (server example, can't unit test)
; @note: random     ; Document non-deterministic behavior
```

---

## index

Build and manage RAG-enhanced intent classification indices.

The `index` command builds pre-computed embedding indices for fast intent classification. These indices enable ~0.1ms inference by avoiding the full 25M model for high-confidence queries.

```
nl index [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--build` | Build intent index from canonical descriptions |
| `--build-examples` | Build example index for borderline queries |
| `--info` | Show index status and statistics |
| `--verify` | Verify index accuracy against test cases |
| `--ollama <MODEL>` | Use Ollama embedder with specified model |
| `--embedder <TYPE>` | Embedder type: ollama, onnx, fast |
| `-v, --verbose` | Verbose output |

### Examples

```bash
# Build intent index (54 intents × 384 dims = ~82KB)
nl index --build

# Build with Ollama embedder
nl index --build --ollama nomic-embed-text

# Build example index for borderline confidence queries
nl index --build-examples

# Show index status
nl index --info

# Verify classification accuracy
nl index --verify
```

### Output

```
$ nl index --build --verbose

Building intent index...
[1/3] Loading embedder (ollama:nomic-embed-text)...
[2/3] Embedding 54 intent descriptions...
      Intent 0: "add two numbers together" -> [0.12, -0.34, ...]
      Intent 1: "subtract one number from another" -> [0.08, 0.21, ...]
      ...
[3/3] Saving to ~/.neurlang/intent_index.bin...

Done. Index built:
  Intents: 54
  Dimensions: 384
  File size: 82,944 bytes

Run 'nl index --verify' to test accuracy.
```

### Index Files

| File | Size | Contents |
|------|------|----------|
| `~/.neurlang/intent_index.bin` | ~82KB | 54 intent embeddings |
| `~/.neurlang/example_index.bin` | ~147MB | 100K example embeddings |
| `~/.neurlang/example_meta.bin` | ~0.5MB | Example metadata |

### Performance Impact

| Pipeline | Without Index | With Index | Speedup |
|----------|---------------|------------|---------|
| Intent classification | 0.3ms | 0.05ms | **6x** |
| Full pipeline (avg) | 0.35ms | 0.12ms | **3x** |

See [RAG Intent Index](../architecture/rag-intent-index.md) for architecture details.
