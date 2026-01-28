# Compiler Documentation

Neurlang has two compiler systems:

1. **Copy-and-Patch Compiler** - Compiles binary IR to native x86-64 code in <5μs
2. **Rust→IR Compiler** - Compiles Rust source code to Neurlang assembly (.nl files)

## Table of Contents

- [Copy-and-Patch Compiler](#copy-and-patch-compiler) - Runtime compilation to native code
- [Rust→IR Compiler](#rustir-compiler) - Stdlib generation from Rust

---

# Copy-and-Patch Compiler

The Neurlang copy-and-patch compiler achieves <5μs compilation times.

## Overview

```
┌───────────────────────────────────────────────────────────────────┐
│                     Compiler Architecture                         │
└───────────────────────────────────────────────────────────────────┘

    Input: Binary IR          Output: Executable Code
         │                              │
         ▼                              ▼
  ┌─────────────┐              ┌─────────────┐
  │ Instruction │              │   Machine   │
  │   Stream    │              │    Code     │
  └──────┬──────┘              └──────▲──────┘
         │                            │
         │    ┌─────────────────┐     │
         └───▶│    Compiler     │─────┘
              │                 │
              │ • Stencil Table │
              │ • Patch Engine  │
              │ • Buffer Pool   │
              └─────────────────┘
```

## Components

| Component | Purpose | Location |
|-----------|---------|----------|
| Compiler | Main compilation engine | `src/compile/engine.rs` |
| StencilTable | Pre-compiled code templates | `src/stencil/table.rs` |
| BufferPool | Executable memory allocation | `src/runtime/buffer_pool.rs` |
| FastCompiler | Single-instruction compiler | `src/compile/engine.rs` |
| AotCompiler | Ahead-of-time compiler | `src/compile/engine.rs` |

## Usage

### Basic Compilation

```rust
use neurlang::compile::Compiler;
use neurlang::ir::Program;

let mut compiler = Compiler::new();
let program: Program = /* ... */;

// Compile to executable
let compiled = compiler.compile(&program)?;

// Get compile time
println!("Compiled in {}μs", compiled.compile_time_us());

// Execute
let mut registers = [0u64; 32];
let result = unsafe { compiled.as_fn()(registers.as_mut_ptr()) };
```

### Compile and Execute

```rust
let mut compiler = Compiler::new();
let mut registers = [0u64; 32];

// One-step compile and run
let result = unsafe { compiler.compile_and_run(&program, &mut registers)? };
```

### AOT Compilation

```rust
use neurlang::compile::AotCompiler;

let compiler = AotCompiler::new();

// Compile to raw bytes
let code = compiler.compile_to_bytes(&program)?;

// Compile to ELF (Linux)
#[cfg(target_os = "linux")]
let elf = compiler.compile_to_elf(&program)?;
```

## Compilation Process

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Compilation Steps                                 │
└─────────────────────────────────────────────────────────────────────┘

  Step 1: Buffer Acquisition (~200ns)
  ┌─────────────────────────────────────────────────────────────────┐
  │ • Pop from lock-free queue                                       │
  │ • No syscall (pre-allocated RWX)                                 │
  │ • Return 4KB executable buffer                                   │
  └─────────────────────────────────────────────────────────────────┘
            │
            ▼
  Step 2: Instruction Processing (~100ns each)
  ┌─────────────────────────────────────────────────────────────────┐
  │ For each instruction:                                            │
  │   1. Decode opcode + mode                                        │
  │   2. Lookup stencil in table                                     │
  │   3. Copy stencil to buffer                                      │
  │   4. Patch register indices and immediates                       │
  └─────────────────────────────────────────────────────────────────┘
            │
            ▼
  Step 3: Finalization (~50ns)
  ┌─────────────────────────────────────────────────────────────────┐
  │ • Add return instruction                                         │
  │ • Return CompiledCode handle                                     │
  └─────────────────────────────────────────────────────────────────┘
```

## Stencil Patching

```
┌─────────────────────────────────────────────────────────────────────┐
│                      Patch Process                                   │
└─────────────────────────────────────────────────────────────────────┘

  Stencil template (before patching):
  ┌────────────────────────────────────────────────────────────────┐
  │ 48 b8 22 22 22 22 22 22 22 22   ; movabs rax, PLACEHOLDER_SRC1 │
  │ 48 8b 04 c7                     ; mov rax, [rdi + rax*8]       │
  │ ...                                                             │
  └────────────────────────────────────────────────────────────────┘

  Instruction: add r5, r3, r7

  After patching:
  ┌────────────────────────────────────────────────────────────────┐
  │ 48 b8 03 00 00 00 00 00 00 00   ; movabs rax, 3 (r3)           │
  │ 48 8b 04 c7                     ; mov rax, [rdi + rax*8]       │
  │ 48 b9 07 00 00 00 00 00 00 00   ; movabs rcx, 7 (r7)           │
  │ ...                                                             │
  │ 48 b9 05 00 00 00 00 00 00 00   ; movabs rcx, 5 (r5)           │
  └────────────────────────────────────────────────────────────────┘
```

## CPU Feature Detection

The compiler detects CPU capabilities at startup and selects optimal stencils for the target hardware.

### Detection Process

```
┌───────────────────────────────────────────────────────────────────┐
│                  CPU Feature Detection Flow                       │
└───────────────────────────────────────────────────────────────────┘

  Compiler Initialization (once):
  ┌─────────────────────────────────────────────────────────────────┐
  │                                                                 │
  │   1. Execute CPUID instruction                                  │
  │      ├── EAX=1: Basic feature flags                             │
  │      ├── EAX=7: Extended features (AVX-512, etc.)               │
  │      └── EAX=0x80000001: AMD-specific features                  │
  │                                                                 │
  │   2. Store feature flags in CpuFeatures struct                  │
  │                                                                 │
  │   3. Select stencil variants based on features                  │
  │                                                                 │
  └─────────────────────────────────────────────────────────────────┘
```

### Detected Features

```rust
pub struct CpuFeatures {
    // Bit manipulation
    pub has_popcnt: bool,    // Population count (count set bits)
    pub has_lzcnt: bool,     // Leading zero count
    pub has_bmi1: bool,      // Bit manipulation instructions set 1
    pub has_bmi2: bool,      // Bit manipulation instructions set 2

    // SIMD
    pub has_sse2: bool,      // Required for x86-64 (always true)
    pub has_avx: bool,       // 256-bit vectors
    pub has_avx2: bool,      // More 256-bit operations
    pub has_avx512f: bool,   // 512-bit vectors (foundation)

    // Cryptography
    pub has_aesni: bool,     // Hardware AES
    pub has_sha: bool,       // Hardware SHA-256
    pub has_rdrand: bool,    // Hardware random numbers

    // Atomics
    pub has_cmpxchg16b: bool, // 128-bit compare-and-swap
}
```

### Feature-Specific Stencils

| Feature | If Present | If Absent |
|---------|------------|-----------|
| POPCNT | `popcnt rax, rax` (1 cycle) | Brian Kernighan's algorithm (~15 cycles) |
| LZCNT | `lzcnt rax, rax` (1 cycle) | `bsr` + subtract (~5 cycles) |
| TZCNT | `tzcnt rax, rax` (1 cycle) | `bsf` or loop (~5 cycles) |
| AES-NI | `aesenc xmm0, xmm1` (4 cycles) | Software AES (~200 cycles/block) |
| RDRAND | `rdrand rax` (varies) | OS entropy source (syscall) |

### Example: BITS.POPCNT Stencil Selection

```rust
// Compiler initialization
let features = CpuFeatures::detect();

// When compiling BITS.POPCNT instruction:
let stencil = if features.has_popcnt {
    &POPCNT_HARDWARE  // Single instruction
} else {
    &POPCNT_SOFTWARE  // Loop-based fallback
};
```

**Hardware stencil (with POPCNT):**
```asm
; BITS.POPCNT rd, rs
mov     rax, [rdi + SRC*8]     ; Load source register
popcnt  rax, rax               ; Count set bits (1 cycle!)
mov     [rdi + DST*8], rax     ; Store result
```

**Software fallback (without POPCNT):**
```asm
; BITS.POPCNT rd, rs (Brian Kernighan's algorithm)
mov     rax, [rdi + SRC*8]     ; Load source register
xor     ecx, ecx               ; count = 0
.loop:
    test  rax, rax
    jz    .done
    mov   rdx, rax
    dec   rdx
    and   rax, rdx             ; n &= (n - 1)  clears lowest set bit
    inc   ecx                  ; count++
    jmp   .loop
.done:
    mov   [rdi + DST*8], rcx   ; Store result
```

### Performance Impact

| Operation | With Hardware | Without Hardware | Speedup |
|-----------|---------------|------------------|---------|
| POPCNT (64-bit) | ~1 cycle | ~15 cycles | 15x |
| CLZ (64-bit) | ~1 cycle | ~5 cycles | 5x |
| AES (16-byte block) | ~4 cycles | ~200 cycles | 50x |
| RDRAND (64-bit) | ~300 cycles | ~5000 cycles | 17x |

### Usage

```rust
use neurlang::compile::CpuFeatures;

// Auto-detect (done automatically by Compiler::new())
let features = CpuFeatures::detect();

// Check specific features
if features.has_avx512f {
    println!("AVX-512 available for SIMD operations");
}

// Compiler uses features automatically
let compiler = Compiler::new();  // Detects CPU features internally
```

---

## Performance

| Program Size | Compile Time | Notes |
|--------------|--------------|-------|
| 1 instruction | ~1μs | Dominated by buffer acquisition |
| 10 instructions | ~2μs | Linear scaling |
| 32 instructions | ~4μs | Typical target |
| 100 instructions | ~10μs | Still fast |
| 1000 instructions | ~100μs | Consider splitting |

## Error Handling

```rust
pub enum CompileError {
    MissingStencil(Opcode, u8),     // Unknown opcode/mode
    BufferAllocationFailed,          // Pool exhausted
    ProgramTooLarge(usize, usize),  // Exceeds max size
    InvalidInstruction(usize),       // Bad instruction at offset
}
```

## Configuration

```rust
// Custom buffer pool size
let compiler = Compiler::with_buffer_count(128);

// Check statistics
let stats = compiler.stats();
println!("Pool size: {}", stats.buffer_pool_size);
```

---

# Rust→IR Compiler

The Rust→IR compiler converts Rust source files into Neurlang assembly (.nl files). This is used to generate the standard library from verified Rust implementations.

## Overview

```
┌───────────────────────────────────────────────────────────────────┐
│                    Rust→IR Compiler Architecture                   │
└───────────────────────────────────────────────────────────────────┘

    stdlib/src/*.rs           lib/*.nl
         │                        │
         ▼                        ▼
  ┌─────────────┐          ┌─────────────┐
  │ Rust Source │          │  Neurlang   │
  │  (verified) │          │  Assembly   │
  └──────┬──────┘          └──────▲──────┘
         │                        │
         │    ┌───────────────────┤
         │    │                   │
         ▼    │                   │
  ┌──────────────────────────────────────┐
  │            Compiler Pipeline          │
  │                                       │
  │  1. Parser (syn crate)                │
  │  2. Analyzer (type check, scope)      │
  │  3. Code Generator (IR emission)      │
  │  4. Test Generator (@test from Rust)  │
  └───────────────────────────────────────┘
```

## Components

| Component | Purpose | Location |
|-----------|---------|----------|
| Parser | Parse Rust AST via syn | `src/compiler/parser.rs` |
| Analyzer | Type checking, variable scoping | `src/compiler/analyzer.rs` |
| CodeGen | Emit Neurlang IR instructions | `src/compiler/codegen.rs` |
| TestGen | Generate @test annotations | `src/compiler/test_gen.rs` |

### Parser: Neurlang Metadata Extraction

The parser extracts metadata from special doc comment sections:

```rust
// In src/compiler/parser.rs

/// Parameter documentation extracted from # Parameters section
pub struct ParamDoc {
    pub name: String,       // e.g., "n"
    pub register: String,   // e.g., "r0"
    pub description: String, // e.g., "The number to compute factorial of"
}

/// Metadata extracted from # Neurlang Export, # Prompts, # Parameters sections
pub struct NeurlangMetadata {
    pub prompts: Vec<String>,       // Natural language prompts with {param} placeholders
    pub param_docs: Vec<ParamDoc>,  // Parameter documentation
    pub category: Option<String>,   // e.g., "algorithm/math"
    pub difficulty: Option<u8>,     // 1-5 complexity level
}
```

**Extracted from doc comments:**
- `# Neurlang Export` → `category`, `difficulty`
- `# Prompts` → `prompts` vector
- `# Parameters` → `param_docs` vector

## Supported Rust Subset

The compiler supports a subset of Rust designed for stdlib implementation:

### Tier 1: Core (Fully Supported)

```rust
// Integer arithmetic
a + b, a - b, a * b, a / b, a % b

// Floating point (f64)
a + b, a - b, a * b, a / b
a.sqrt(), a.abs(), a.floor(), a.ceil()

// Bitwise operations
a & b, a | b, a ^ b, a << b, a >> b

// Comparisons
a == b, a != b, a < b, a <= b, a > b, a >= b

// Variables
let x = expr;
let mut x = expr;
x = expr;  // reassignment

// Control flow
if cond { ... } else { ... }
while cond { ... }
loop { ... break; }
for i in 0..n { ... }

// Functions
fn name(a: u64, b: u64) -> u64 { ... }
fn name(a: f64, b: f64) -> f64 { ... }
return expr;
```

### Not Supported

- Heap allocation (use extensions for dynamic memory)
- Generics/traits (use concrete types)
- Closures (use named functions)
- Async/await (use blocking calls)
- String/slice types (use raw pointers via extensions)

## Usage

### Command Line

```bash
# Build all stdlib modules
nl stdlib --build

# Build with verbose output
nl stdlib --build --verbose

# Verify Rust == Neurlang output
nl stdlib --verify
```

### Programmatic

```rust
use neurlang::compiler::{RustCompiler, CompilerConfig};

let config = CompilerConfig::default();
let compiler = RustCompiler::new(config);

// Compile a single file
let result = compiler.compile_file("stdlib/src/math.rs")?;

// Get generated assembly
for (name, code) in result.functions {
    println!("Function: {}", name);
    println!("{}", code.to_assembly());
}
```

## Code Generation

### Example: Factorial

**Input (Rust):**
```rust
pub fn factorial(n: u64) -> u64 {
    let mut result = 1u64;
    let mut i = n;
    while i > 0 {
        result *= i;
        i -= 1;
    }
    result
}
```

**Output (Neurlang Assembly):**
```asm
; @name: factorial
; @description: Calculate factorial of n
; @param: n=r0
;
; @test: r0=0 -> r0=1
; @test: r0=5 -> r0=120
; @test: r0=10 -> r0=3628800

.entry:
    mov r1, 1           ; result = 1
    mov r2, r0          ; i = n
.loop:
    beq r2, zero, .done ; if i == 0, exit
    muldiv.mul r1, r1, r2  ; result *= i
    alui.sub r2, r2, 1     ; i--
    b .loop
.done:
    mov r0, r1          ; return result
    halt
```

### Register Allocation

The compiler uses a simple register allocation strategy:

| Register | Usage |
|----------|-------|
| r0 | Return value, first argument |
| r1-r3 | Additional arguments |
| r4-r15 | Local variables |
| r16-r31 | Temporary computations |
| zero | Constant 0 (read-only) |

### Branch Label Generation

Branch instructions use symbolic labels:

```asm
.loop:          ; Label for loop start
    ...
    bne r0, zero, .loop  ; Branch back to loop
.done:          ; Label for loop exit
```

## Test Generation

The compiler automatically generates `@test` annotations by running the Rust function with test inputs:

```rust
// In Rust source
/// Calculate factorial of n
pub fn factorial(n: u64) -> u64 { ... }
```

The compiler:
1. Detects parameter types and ranges
2. Generates representative test inputs
3. Executes the Rust function to get expected outputs
4. Emits `@test:` annotations in the assembly

## Configuration

Configure compilation via `neurlang.toml`:

```toml
[build]
# Include source comments in generated .nl files
include_comments = true

# Generate @test annotations from doc comments
generate_tests = true

# Maximum instructions per function (safety limit)
max_instructions = 1000

# Output directory for generated .nl files
output_dir = "lib"

[compiler]
# Optimization level: 0 = none, 1 = basic, 2 = aggressive
optimization = 1

# Enable debug wrappers for extensions
debug_mode = false

# Target architecture (x86_64, aarch64)
target = "x86_64"
```

## Compilation Pipeline

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Compilation Pipeline                              │
└─────────────────────────────────────────────────────────────────────┘

  Phase 1: Parse
  ┌─────────────────────────────────────────────────────────────────┐
  │ • Read Rust source file                                          │
  │ • Parse with syn crate                                           │
  │ • Extract function definitions, types, doc comments              │
  └─────────────────────────────────────────────────────────────────┘
            │
            ▼
  Phase 2: Analyze
  ┌─────────────────────────────────────────────────────────────────┐
  │ • Type checking (u64, i64, f64, bool)                            │
  │ • Variable scoping and shadowing                                 │
  │ • Loop and branch structure analysis                             │
  │ • Register pre-allocation                                        │
  └─────────────────────────────────────────────────────────────────┘
            │
            ▼
  Phase 3: Generate
  ┌─────────────────────────────────────────────────────────────────┐
  │ • Emit Neurlang instructions                                     │
  │ • Handle control flow (if/while/loop/for)                        │
  │ • Generate branch labels                                         │
  │ • Map FPU operations to fpu.* opcodes                            │
  └─────────────────────────────────────────────────────────────────┘
            │
            ▼
  Phase 4: Test Generation
  ┌─────────────────────────────────────────────────────────────────┐
  │ • Generate test inputs based on parameter types                  │
  │ • Execute Rust function to get expected outputs                  │
  │ • Emit @test annotations                                         │
  └─────────────────────────────────────────────────────────────────┘
            │
            ▼
  Output: lib/*.nl files with @test annotations
```

## See Also

- [Stdlib Development Guide](../stdlib/README.md)
- [CLI stdlib Command](../cli/commands.md#stdlib)
- [neurlang.toml Configuration](../config/README.md)
