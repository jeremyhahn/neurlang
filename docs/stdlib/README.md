# Standard Library Development Guide

The Neurlang standard library is written in Rust and compiled to Neurlang assembly (.nl files). This guarantees correctness through verified Rust implementations while providing optimized IR for training.

## Overview

```
stdlib/src/*.rs (Rust source - you write this)
       ↓
   nl stdlib --build (Rust→IR compiler)
       ↓
lib/*.nl (generated Neurlang assembly)
       ↓
Training data generator
       ↓
Model learns stdlib patterns
```

**Key Insight**: The model trains on GENERATED IR from verified Rust. This means:
- Correctness is guaranteed (Rust is the oracle)
- Model learns real implementation patterns
- No manual maintenance of .nl files
- Rust ecosystem for testing/debugging

## Directory Structure

```
neurlang/
├── stdlib/                 # Rust source (you edit this)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # Module declarations
│       ├── math.rs         # 12 functions: factorial, fibonacci, gcd, etc.
│       ├── float.rs        # 20 functions: sqrt, abs, floor, ceil, etc.
│       ├── string.rs       # 25 functions: strlen, atoi, itoa, etc.
│       ├── array.rs        # 22 functions: sum, reverse, search, sort
│       ├── bitwise.rs      # 16 functions: popcount, clz, bit ops
│       └── collections.rs  # 18 functions: stack, queue, hashtable
│
├── lib/                    # GENERATED (don't edit manually) - 113 total
│   ├── math/               # factorial.nl, fibonacci.nl, gcd.nl, ...
│   ├── float/              # sqrt.nl, abs.nl, floor.nl, ...
│   ├── string/             # strlen.nl, atoi.nl, itoa.nl, ...
│   ├── array/              # sum_array.nl, array_reverse.nl, ...
│   ├── bitwise/            # popcount.nl, clz.nl, ctz.nl, ...
│   └── collections/        # stack_push.nl, queue_enqueue.nl, ...
│
└── neurlang.toml           # Configuration
```

## Configuration

Control stdlib compilation via `neurlang.toml`:

```toml
[package]
name = "neurlang-stdlib"
version = "0.1.0"
description = "Neurlang Standard Library"

# Stdlib modules to include (compiled from Rust→IR)
# Set to true to include, false to exclude
[stdlib]
math = true         # factorial, fibonacci, gcd, power, is_prime
float = true        # FPU operations (sqrt, abs, floor, ceil)
string = true       # strlen, strcmp, strcpy, atoi, itoa
array = true        # sum, min, max, search, sort
bitwise = true      # popcount, clz, ctz, rotl, rotr
collections = true  # stack, queue, hashtable

[build]
# Include source comments in generated .nl files
include_comments = true

# Generate @test annotations from doc comments
generate_tests = true

# Maximum instructions per function (for safety)
max_instructions = 1000

# Output directory for generated .nl files
output_dir = "lib"
```

## Writing Stdlib Functions

### Doc Comment Format

Functions use special doc comment sections that the compiler extracts:

```rust
/// Calculate factorial of n iteratively.
///
/// # Neurlang Export
/// - Category: algorithm/math
/// - Difficulty: 2
///
/// # Prompts
/// - compute factorial of {n}
/// - {n}!
/// - calculate {n} factorial
/// - what is {n}!
/// - factorial({n})
/// - multiply 1 * 2 * ... * {n}
/// - product of integers from 1 to {n}
/// - n! where n={n}
/// - iterative factorial for {n}
/// - compute {n} factorial using a loop
///
/// # Parameters
/// - n=r0 "The number to compute factorial of"
///
/// # Test Cases
/// - factorial(0) = 1
/// - factorial(5) = 120
/// - factorial(10) = 3628800
#[inline(never)]
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

### Doc Comment Sections

| Section | Purpose | Format |
|---------|---------|--------|
| `# Neurlang Export` | Metadata | `- Category: ...`, `- Difficulty: 1-5` |
| `# Prompts` | Training prompts | `- natural language with {param} placeholders` |
| `# Parameters` | Parameter docs | `- name=register "description"` |
| `# Test Cases` | Expected results | `- function(args) = result` |

**Guidelines for Prompts:**
- Include 10-15 diverse phrasings per function
- Use `{param_name}` placeholders for parameters
- Mix formal ("compute factorial") and informal ("{n}!")
- Include mathematical notation where applicable
- The training generator expands placeholders with sample values

### Supported Types

| Rust Type | Neurlang Type | Register Usage |
|-----------|---------------|----------------|
| `u64` | 64-bit unsigned | Single register |
| `i64` | 64-bit signed | Single register |
| `f64` | 64-bit float | Single register (FPU) |
| `bool` | 1-bit | Register (0 or 1) |

### Supported Constructs

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
return expr;
```

### What's NOT Supported

- Heap allocation (`Vec`, `String`, `Box`)
- Generics and traits
- Closures and higher-order functions
- Async/await
- Pattern matching beyond simple if-else
- References and borrowing
- Struct/enum definitions

## Example: Math Module

```rust
// stdlib/src/math.rs

/// Calculate factorial of n
#[neurlang_export]
pub fn factorial(n: u64) -> u64 {
    let mut result = 1u64;
    let mut i = n;
    while i > 0 {
        result *= i;
        i -= 1;
    }
    result
}

/// Calculate nth Fibonacci number
#[neurlang_export]
pub fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }
    let mut a = 0u64;
    let mut b = 1u64;
    let mut i = 2u64;
    while i <= n {
        let temp = a + b;
        a = b;
        b = temp;
        i += 1;
    }
    b
}

/// Calculate GCD using Euclidean algorithm
#[neurlang_export]
pub fn gcd(a: u64, b: u64) -> u64 {
    let mut x = a;
    let mut y = b;
    while y != 0 {
        let temp = y;
        y = x % y;
        x = temp;
    }
    x
}

/// Check if n is prime
#[neurlang_export]
pub fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    let mut i = 3u64;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i += 2;
    }
    true
}
```

## Generated Output

Running `nl stdlib --build` generates:

### lib/math/factorial.nl

```asm
; @name: Factorial
; @description: Calculate factorial of n iteratively.
; @category: algorithm/math
; @difficulty: 2
;
; @prompt: compute factorial of {n}
; @prompt: {n}!
; @prompt: calculate {n} factorial
; @prompt: what is {n}!
; @prompt: factorial({n})
; @prompt: multiply 1 * 2 * ... * {n}
; @prompt: product of integers from 1 to {n}
; @prompt: n! where n={n}
; @prompt: iterative factorial for {n}
; @prompt: compute {n} factorial using a loop
; @prompt: find the factorial of {n}
; @prompt: calculate {n}! iteratively
;
; @param: n=r0 "The number to compute factorial of"
;
; @test: r0=0 -> r0=1
; @test: r0=1 -> r0=1
; @test: r0=5 -> r0=120
; @test: r0=10 -> r0=3628800
; @test: r0=20 -> r0=2432902008176640000
;
; @export: factorial
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r1, 1           ; result = 1
    mov r2, r0          ; i = n
.while_0:
    nop
    mov r15, r2         ; i
    mov r14, 0          ; 0
    bgt r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endwhile_1
    mov r15, r2         ; i
    muldiv.Mul r1, r1, r15
    mov r15, 1          ; 1
    alu.Sub r2, r2, r15
    b .while_0
.endwhile_1:
    nop
    mov r0, r1          ; result
    halt
```

**Key annotations in generated output:**
- `@prompt:` - Natural language prompts extracted from `# Prompts` section
- `@param:` - Parameter documentation from `# Parameters` section
- `@difficulty:` - Complexity level from `# Neurlang Export`
- `@category:` - Category from `# Neurlang Export`
- `@export:` - Function name for symbol table

## Test Annotation Generation

The compiler automatically generates `@test` annotations by:

1. Parsing doc comments and examples
2. Generating representative test inputs based on types
3. Executing the Rust function to get expected outputs
4. Emitting `@test:` lines in the assembly

### Test Input Generation Strategy

| Type | Generated Inputs |
|------|-----------------|
| `u64` | 0, 1, 5, 10, small primes, edge cases |
| `i64` | -10, -1, 0, 1, 10, edge cases |
| `f64` | 0.0, 1.0, -1.0, 0.5, large values |
| `bool` | true, false |

## Building and Testing

```bash
# Build all stdlib modules
nl stdlib --build

# Build with verbose output
nl stdlib --build --verbose

# Verify Rust output == Neurlang output
nl stdlib --verify

# Run tests on generated files
nl test -p lib

# Generate training data from stdlib
python train/generate_training_data.py --stdlib-dir lib
```

## Adding a New Module

1. Create `stdlib/src/newmodule.rs`:

```rust
//! New module description

/// Function documentation
#[neurlang_export]
pub fn my_function(x: u64) -> u64 {
    // implementation
}
```

2. Add to `stdlib/src/lib.rs`:

```rust
pub mod newmodule;
```

3. Enable in `neurlang.toml`:

```toml
[stdlib]
newmodule = true
```

4. Build:

```bash
nl stdlib --build
```

5. Verify:

```bash
nl test -p lib/newmodule
```

## Training Data Pipeline

The stdlib is the primary source of training data for the model:

```
stdlib/src/*.rs
       ↓
   nl stdlib --build
       ↓
lib/*.nl (with @test annotations)
       ↓
train/generate_training_data.py
       ↓
train/stdlib_training.jsonl
       ↓
Model training
       ↓
Model understands:
  - How stdlib functions are implemented
  - Patterns for loops, conditionals, memory access
  - How to compose functions
```

### Training Data Format

```json
{
  "prompt": "calculate factorial of {n}",
  "assembly": "mov r1, 1\nmov r2, r0\n...",
  "test_cases": [
    {"inputs": {"r0": 5}, "outputs": {"r0": 120}},
    {"inputs": {"r0": 10}, "outputs": {"r0": 3628800}}
  ]
}
```

## Best Practices

1. **Keep functions simple** - Each function should do one thing
2. **Use descriptive names** - `calculate_gcd` not `gcd_impl_v2`
3. **Add doc comments** - They become @description annotations
4. **Include edge cases** - Test with 0, 1, max values
5. **Avoid recursion** - Use iterative implementations (simpler IR)
6. **Minimize variables** - Fewer registers = simpler code
7. **Use explicit types** - `1u64` not `1`

## Troubleshooting

### "Unsupported construct" error

The Rust→IR compiler only supports a subset of Rust. Check:
- No heap allocation (`Vec`, `String`)
- No generics or traits
- No closures
- No async/await

### "Test failed" after build

Run `nl stdlib --verify` to compare Rust vs Neurlang output. Common issues:
- Integer overflow differences
- Floating point precision
- Off-by-one in loops

### "Too many instructions"

Split large functions or increase `max_instructions` in config:

```toml
[build]
max_instructions = 2000
```

## See Also

- [Rust→IR Compiler](../compiler/README.md#rustir-compiler)
- [CLI stdlib Command](../cli/commands.md#stdlib)
- [Training Documentation](../training/README.md)
- [neurlang.toml Configuration](../config/README.md)
