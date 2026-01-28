# Code Generation

Neurlang IR can be transpiled to multiple programming languages, enabling debugging, portability, and integration with existing toolchains.

## Supported Output Languages

| Language | Status | File | Use Case |
|----------|--------|------|----------|
| **C** | ✅ Complete | `src/codegen/c.rs` | Embedding, debugging, portability |
| **Go** | ✅ Complete | `src/codegen/go.rs` | Cloud-native, goroutine mapping |
| **Rust** | ✅ Complete | `src/codegen/rust.rs` | Safety-critical, ecosystem integration |
| **Pseudocode** | ✅ Complete | `src/codegen/pseudocode.rs` | Documentation, human review |

## Architecture

The code generation system uses a visitor-based pattern where each generator implements the `CodeGenerator` trait:

```
┌───────────────────────────────────────────────────────┐
│                    Program (IR)                        │
└───────────────────────┬───────────────────────────────┘
                        │
                        ▼
┌───────────────────────────────────────────────────────┐
│              CodeGenerator Trait                       │
│  fn emit_alu(&mut self, op, rd, rs1, rs2)             │
│  fn emit_load(&mut self, width, rd, base, offset)     │
│  fn emit_store(&mut self, width, src, base, offset)   │
│  fn emit_branch(&mut self, cond, target)              │
│  fn emit_call(&mut self, target)                      │
│  ...                                                   │
└───────────┬───────────┬───────────┬───────────────────┘
            │           │           │
     ┌──────┴───┐  ┌────┴────┐  ┌───┴────┐
     │ CCodeGen │  │ GoGen   │  │ RustGen│  ...
     └──────────┘  └─────────┘  └────────┘
```

## Usage

### Programmatic API

```rust
use neurlang::codegen::{CCodeGenerator, GoCodeGenerator, RustCodeGenerator, PseudocodeGenerator};
use neurlang::codegen::CodeGenerator;
use neurlang::ir::Program;

// Parse or load your program
let program: Program = /* ... */;

// Generate C code
let mut c_gen = CCodeGenerator::new();
c_gen.generate(&program);
let c_code = c_gen.output();

// Generate Go code
let mut go_gen = GoCodeGenerator::new();
go_gen.generate(&program);
let go_code = go_gen.output();

// Generate Rust code
let mut rust_gen = RustCodeGenerator::new();
rust_gen.generate(&program);
let rust_code = rust_gen.output();

// Generate pseudocode for documentation
let mut pseudo_gen = PseudocodeGenerator::new();
pseudo_gen.generate(&program);
let pseudocode = pseudo_gen.output();
```

## Feature Coverage

### Core Operations

| Feature | C | Go | Rust | Pseudocode |
|---------|---|----|----|------------|
| ALU (add, sub, mul, div) | ✅ | ✅ | ✅ | ✅ |
| Memory (load/store) | ✅ | ✅ | ✅ | ✅ |
| Control flow (branch, call, ret) | ✅ | ✅ | ✅ | ✅ |
| Capabilities (fat pointers) | ✅ | ✅ | ✅ | ✅ |
| Intrinsics (memcpy, strlen, etc.) | ✅ | ✅ | ✅ | ✅ |

### Concurrency

| Feature | C | Go | Rust | Pseudocode |
|---------|---|----|----|------------|
| Spawn (create task) | ✅ pthread | ✅ goroutine | ✅ std::thread | ✅ |
| Join (wait for task) | ✅ pthread_join | ✅ WaitGroup | ✅ JoinHandle | ✅ |
| Channels (send/recv) | ✅ mutex+condvar | ✅ native chan | ✅ mpsc | ✅ |
| Atomics (load/store/cas) | ✅ stdatomic | ✅ sync/atomic | ✅ std::sync::atomic | ✅ |
| Atomic Min/Max | ✅ CAS loop | ✅ CAS loop | ✅ compare_exchange | ✅ |
| Fence (memory barrier) | ✅ atomic_thread_fence | ✅ - | ✅ fence | ✅ |

### I/O Operations

| Feature | C | Go | Rust | Pseudocode |
|---------|---|----|----|------------|
| File Open/Close | ✅ fopen/fclose | ✅ os.OpenFile | ✅ File::open | ✅ |
| File Read/Write | ✅ fread/fwrite | ✅ Read/Write | ✅ read/write | ✅ |
| File Seek | ✅ fseek | ✅ Seek | ✅ seek | ✅ |
| File Stat | ✅ fstat | ✅ Stat | ✅ metadata | ✅ |
| Mkdir | ✅ mkdir | ✅ os.Mkdir | ✅ create_dir | ✅ |
| Delete | ✅ remove | ✅ os.Remove | ✅ remove_file | ✅ |
| Socket | ✅ socket() | ✅ net.Dial | ✅ TcpStream | ✅ |
| Connect | ✅ connect() | ✅ net.Dial | ✅ connect | ✅ |
| Bind/Listen/Accept | ✅ BSD sockets | ✅ net.Listen | ✅ TcpListener | ✅ |
| Send/Recv | ✅ send/recv | ✅ Write/Read | ✅ write/read | ✅ |
| Console I/O | ✅ printf/getchar | ✅ fmt/bufio | ✅ print!/stdin | ✅ |
| Time | ✅ gettimeofday | ✅ time.Now | ✅ SystemTime | ✅ |

### Math & Extensions

| Feature | C | Go | Rust | Pseudocode |
|---------|---|----|----|------------|
| FPU (float ops) | ✅ native | ✅ native | ✅ native | ✅ |
| Random | ✅ rand() | ✅ math/rand | ✅ RandomState | ✅ |
| Bit manipulation | ✅ native | ✅ bits pkg | ✅ native | ✅ |
| Extension calls | ✅ FFI stub | ✅ FFI stub | ✅ FFI | ✅ |
| Trap/Syscall | ✅ fprintf/exit | ✅ panic/exit | ✅ eprintln/exit | ✅ |

## Language-Specific Details

### C Output

The C generator produces self-contained C11 code with:

- **Threading**: Uses pthreads (`pthread_create`, `pthread_join`)
- **Channels**: Implemented with mutex + condition variable queues
- **Atomics**: Uses `<stdatomic.h>` with `atomic_compare_exchange_weak` for min/max
- **Networking**: BSD sockets (`socket`, `connect`, `send`, `recv`)
- **File I/O**: Standard C library (`fopen`, `fread`, `fwrite`, `fseek`)

```c
// Thread infrastructure
#define MAX_THREADS 256
typedef struct {
    pthread_t thread;
    uint64_t result;
    int active;
} nl_thread_t;
static nl_thread_t threads[MAX_THREADS];

// Channel infrastructure
typedef struct {
    uint64_t* buffer;
    size_t capacity;
    size_t head;
    size_t tail;
    size_t count;
    pthread_mutex_t mutex;
    pthread_cond_t not_empty;
    pthread_cond_t not_full;
} nl_channel_t;
```

Compile with:
```bash
gcc -std=c11 -pthread -o program output.c
```

### Go Output

The Go generator produces idiomatic Go code with:

- **Goroutines**: Direct mapping from Spawn to `go func()`
- **Channels**: Native Go channels with make/send/receive
- **Sync**: Uses `sync.WaitGroup` for join, `sync/atomic` for atomics
- **Networking**: Uses `net` package (`net.Dial`, `net.Listen`)
- **File I/O**: Uses `os` package

```go
import (
    "sync"
    "sync/atomic"
    "net"
    "os"
)

var taskWg sync.WaitGroup
var taskDone = make(map[uint64]chan uint64)
var channels = make(map[uint64]chan uint64)
```

### Rust Output

The Rust generator produces safe Rust code with:

- **Threading**: Uses `std::thread::spawn` with `JoinHandle`
- **Channels**: Uses `std::sync::mpsc` for message passing
- **Atomics**: Uses `std::sync::atomic` with `Ordering::SeqCst`
- **Networking**: Uses `std::net` (TcpStream, TcpListener)
- **File I/O**: Uses `std::fs` and `std::io`
- **Random**: Uses `std::collections::hash_map::RandomState` (no external crates)

```rust
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::atomic::{AtomicU64, Ordering};
use std::net::{TcpStream, TcpListener};
use std::fs::File;
use std::collections::hash_map::RandomState;
```

### Pseudocode Output

The pseudocode generator produces human-readable output for documentation and review:

```
FUNCTION main:
    SET r0 = 0
    SET r1 = 10
loop:
    IF r0 >= r1 THEN GOTO done
    PRINT r0
    INCREMENT r0
    GOTO loop
done:
    RETURN r0
```

## Adding New Languages

To add a new language generator:

1. Create `src/codegen/<language>.rs`
2. Implement the `CodeGenerator` trait:

```rust
pub struct MyLanguageGenerator {
    writer: IndentWriter,
}

impl CodeGenerator for MyLanguageGenerator {
    fn emit_alu(&mut self, op: AluOp, rd: Register, rs1: Register, rs2: Register) {
        // Generate ALU operation in target language
    }

    fn emit_load(&mut self, width: Width, rd: Register, base: Register, offset: i32) {
        // Generate load operation
    }

    // ... implement other methods
}
```

3. Add to `src/codegen/mod.rs`:
```rust
mod my_language;
pub use my_language::MyLanguageGenerator;
```

## Testing

The code generators are tested with 45 unit tests covering:

- Register formatting
- Immediate value handling
- All ALU operations
- Memory operations
- Control flow
- Capabilities
- Concurrency primitives
- I/O operations
- Edge cases

Run tests:
```bash
cargo test codegen
```

## Performance Considerations

The generated code prioritizes correctness and clarity over performance. For production use:

- **C**: The generated code should perform well; consider adding `-O2` or `-O3`
- **Go**: Goroutine overhead is minimal; GC may affect latency-sensitive code
- **Rust**: Safe Rust may have bounds checking overhead; use `--release` builds

For maximum performance, use the native JIT compiler which compiles in <5μs.
