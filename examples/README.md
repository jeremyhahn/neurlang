# Neurlang Example Programs

This directory contains example programs demonstrating real-world application patterns that compose stdlib functions.

**Note:** Basic algorithms (factorial, fibonacci, gcd, string operations, array operations, etc.) are now in `lib/` - generated from verified Rust implementations in `stdlib/src/`. This directory focuses on **composition patterns** and **application-level examples** that teach how to combine stdlib functions.

## Running Examples

To run an example:

```bash
# Run a stdlib function
nl run -i lib/math/factorial.nl

# Run an application example
nl run -i examples/network/http_server.nl
```

Or use the REPL for quick testing:

```bash
nl repl
> mov r0, 42
> halt
```

## Directory Structure

```
examples/
├── network/          # TCP, HTTP, REST API patterns (20 files)
├── extension/        # JSON, SQLite, UUID usage (11 files)
├── application/      # Service patterns (12 files)
├── io/               # File/console I/O (2 files)
├── concurrency/      # Threading patterns (1 file)
├── crypto/           # Random numbers (1 file)
├── encoding/         # Base64 (1 file)
├── parsing/          # Config parsing (1 file)
└── security/         # Capability demo (1 file)
```

## Example Categories

### Network Examples (`network/`)

| File | Description | Key Features |
|------|-------------|--------------|
| `http_server.nl` | Basic HTTP server | TCP, request parsing |
| `rest_api.nl` | REST API endpoints | JSON, routing |
| `websocket.nl` | WebSocket server | Bidirectional communication |

### Extension Usage (`extension/`)

| File | Description | Key Features |
|------|-------------|--------------|
| `json_parse.nl` | Parse JSON strings | ext.call for JSON |
| `sqlite_query.nl` | SQLite database | ext.call for SQL |
| `uuid_generate.nl` | Generate UUIDs | ext.call for UUID |

### Application Patterns (`application/`)

| File | Description | Key Features |
|------|-------------|--------------|
| `user_service.nl` | User CRUD service | Composition pattern |
| `auth_service.nl` | Authentication | Token handling |

### System Operations

| File | Description | Key Features |
|------|-------------|--------------|
| `io/hello_world.nl` | Print to console | IO opcode |
| `concurrency/concurrent.nl` | Multi-threaded computation | SPAWN, JOIN, CHAN |
| `security/capability_demo.nl` | Memory safety | CAP opcodes |

## Stdlib vs Examples

| Location | Purpose | Source |
|----------|---------|--------|
| `lib/` | Core algorithms (factorial, strlen, etc.) | Generated from `stdlib/src/*.rs` |
| `examples/` | Application patterns, compositions | Hand-written |

For basic algorithms, see `lib/`:
- `lib/math/` - factorial, fibonacci, gcd, power, is_prime, etc.
- `lib/float/` - sqrt, abs, floor, ceil, sin, cos, etc.
- `lib/string/` - strlen, strcmp, atoi, itoa, etc.
- `lib/array/` - sum, min, max, reverse, search, sort, etc.
- `lib/bitwise/` - popcount, clz, ctz, rotl, rotr, etc.
- `lib/collections/` - stack, queue, hashtable operations

## Assembly Syntax

```assembly
; Comments start with semicolon
.entry:              ; Entry point

    mov r0, 42       ; Load immediate
    alu.add r1, r0, r0  ; r1 = r0 + r0
    halt             ; Stop execution
```

## Register Conventions

- `r0`: Return value / first argument
- `r1-r3`: Additional arguments
- `r4-r9`: Caller-saved temporaries
- `r10-r15`: Callee-saved, server state
- `r16-r31`: General purpose
- `zero`: Always zero (hardwired)

## See Also

- [Stdlib Documentation](../docs/stdlib/README.md) - Core functions in lib/
- [CLI Commands](../docs/cli/README.md) - Running and testing programs
- [Training Documentation](../docs/training/README.md) - How examples contribute to training
