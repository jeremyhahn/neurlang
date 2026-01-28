# Multi-Architecture Support

Neurlang is designed with multi-architecture support from the ground up. The architecture abstraction layer enables the same IR to target different CPU architectures.

## Current Status

| Architecture | JIT Stencils | Code Generation | Status |
|--------------|--------------|-----------------|--------|
| **x86-64** | ✅ Full | ✅ Full | Production ready |
| **ARM64** | ⚡ Stub | ✅ Full | Fallback to interpreter |
| **RISC-V 64** | ⚡ Stub | ✅ Full | Fallback to interpreter |

## Architecture Abstraction

### The Architecture Trait

Located in `src/arch/mod.rs`, the `Architecture` trait defines the interface for architecture-specific implementations:

```rust
pub trait Architecture: Send + Sync {
    /// Architecture name (e.g., "x86-64", "aarch64")
    fn name(&self) -> &'static str;

    /// Number of general-purpose registers
    fn register_count(&self) -> usize;

    /// Word size in bytes
    fn word_size(&self) -> usize;

    /// Get a stencil for the given opcode
    fn get_stencil(&self, opcode: Opcode, mode: u8) -> Option<&Stencil>;

    /// Patch an immediate value into generated code
    fn patch_immediate(&self, code: &mut [u8], offset: usize, value: i64);

    /// Patch a register reference into generated code
    fn patch_register(&self, code: &mut [u8], offset: usize, reg: u8);

    /// Get the return instruction sequence
    fn return_sequence(&self) -> &[u8];

    /// Get the calling convention
    fn calling_convention(&self) -> CallingConvention;

    /// Check if this architecture is the native one
    fn is_native(&self) -> bool;
}
```

### Calling Convention

Each architecture defines its calling convention:

```rust
pub struct CallingConvention {
    /// Registers used for passing arguments
    pub arg_registers: &'static [u8],
    /// Register for return value
    pub return_register: u8,
    /// Registers preserved across calls (callee-saved)
    pub preserved_registers: &'static [u8],
    /// Registers that may be clobbered (caller-saved)
    pub scratch_registers: &'static [u8],
    /// Stack alignment requirement
    pub stack_alignment: usize,
}
```

## x86-64 Implementation

The x86-64 backend is fully implemented with optimized stencils:

### Key Files
- `src/arch/x86_64/mod.rs` - Architecture implementation
- `src/arch/x86_64/stencils.rs` - Pre-compiled code stencils
- `src/arch/x86_64/abi.rs` - System V AMD64 calling convention

### Calling Convention (System V AMD64)

| Purpose | Registers |
|---------|-----------|
| Arguments | RDI, RSI, RDX, RCX, R8, R9 |
| Return value | RAX |
| Callee-saved | RBX, RBP, R12-R15 |
| Caller-saved | RAX, RCX, RDX, RSI, RDI, R8-R11 |

### Stencil Generation

Stencils are generated at build time via `build.rs`:

```rust
// Example stencil for ADD instruction
fn generate_add_stencil() -> Vec<u8> {
    // mov rax, [rdi + rs1*8]      ; Load source 1
    // add rax, [rdi + rs2*8]      ; Add source 2
    // mov [rdi + rd*8], rax       ; Store result
    vec![0x48, 0x8b, 0x04, 0xc7, ...]
}
```

The stencil system uses placeholder values (`0xDEADBEEF`) that are patched at runtime with actual register offsets and immediates.

## ARM64 Implementation (Stub)

The ARM64 backend provides a stub implementation that falls back to the interpreter:

### Key Files
- `src/arch/aarch64/mod.rs` - Architecture stub
- `src/arch/aarch64/abi.rs` - AAPCS64 calling convention

### Calling Convention (AAPCS64)

| Purpose | Registers |
|---------|-----------|
| Arguments | X0-X7 |
| Return value | X0 |
| Callee-saved | X19-X28 |
| Caller-saved | X0-X18 |

### Implementation Roadmap

To complete ARM64 support:

1. **Stencil Generation**: Implement ARM64 instruction encoding in `build.rs`
2. **Operand Patching**: ARM64 uses different immediate encoding (split across instruction fields)
3. **Atomics**: Use LDAXR/STLXR exclusive pairs
4. **Testing**: Validate on actual ARM64 hardware

## RISC-V 64 Implementation (Stub)

The RISC-V backend provides a stub implementation:

### Key Files
- `src/arch/riscv64/mod.rs` - Architecture stub
- `src/arch/riscv64/abi.rs` - RISC-V calling convention

### Calling Convention

| Purpose | Registers |
|---------|-----------|
| Arguments | a0-a7 (x10-x17) |
| Return value | a0 (x10) |
| Callee-saved | s0-s11 (x8-x9, x18-x27) |
| Caller-saved | t0-t6 (x5-x7, x28-x31) |

### Implementation Challenges

1. **Limited Immediates**: 12-bit signed immediates require LUI/AUIPC for larger values
2. **Atomics**: Requires A extension (AMOADD.W, etc.)
3. **Instruction Encoding**: More complex than x86-64 for some operations

## Runtime Architecture Detection

The runtime automatically detects the native architecture:

```rust
pub fn detect_architecture() -> Box<dyn Architecture> {
    #[cfg(target_arch = "x86_64")]
    {
        Box::new(X86_64::new())
    }
    #[cfg(target_arch = "aarch64")]
    {
        Box::new(AArch64::new())
    }
    #[cfg(target_arch = "riscv64")]
    {
        Box::new(RiscV64::new())
    }
}
```

## Fallback Strategy

When running on an architecture without full stencil support:

1. **Check Native**: If architecture has JIT stencils, use them
2. **Fallback to Interpreter**: Otherwise, use the platform-independent interpreter
3. **Code Generation**: Generate C/Go/Rust code that can be compiled natively

```
┌─────────────────────────────────────────────────────────┐
│                    Neurlang IR                          │
└─────────────────────┬───────────────────────────────────┘
                      │
        ┌─────────────┼─────────────┐
        ▼             ▼             ▼
┌───────────┐   ┌───────────┐   ┌───────────┐
│  x86-64   │   │  ARM64    │   │  RISC-V   │
│   JIT     │   │ Interpret │   │ Interpret │
│   <5μs    │   │   ~5ms    │   │   ~5ms    │
└───────────┘   └───────────┘   └───────────┘
        │             │             │
        ▼             ▼             ▼
┌─────────────────────────────────────────────────────────┐
│                  Native Execution                        │
└─────────────────────────────────────────────────────────┘
```

## Adding a New Architecture

1. Create directory: `src/arch/<arch>/`
2. Implement the Architecture trait:
   ```rust
   pub struct NewArch { /* ... */ }

   impl Architecture for NewArch {
       fn name(&self) -> &'static str { "new-arch" }
       fn register_count(&self) -> usize { 32 }
       // ... implement other methods
   }
   ```
3. Add to `src/arch/mod.rs`
4. Add build-time stencil generation in `build.rs`
5. Add tests in `src/arch/<arch>/tests.rs`

## Performance Comparison

| Architecture | JIT Compile | Interpreted | Code Gen |
|--------------|-------------|-------------|----------|
| x86-64 | <5μs | ~5ms | ~1ms |
| ARM64 (stub) | N/A | ~5ms | ~1ms |
| RISC-V (stub) | N/A | ~5ms | ~1ms |

For architectures without JIT support, the code generation path (C/Go/Rust) provides an alternative that achieves near-native performance after compilation.

## Testing

Architecture-specific tests:

```bash
# Run all architecture tests
cargo test arch

# Run x86-64 specific tests
cargo test arch::x86_64

# Run with architecture feature flags
cargo test --features "aarch64,riscv64"
```

The test suite includes 24 architecture tests covering:
- Register operations
- Calling conventions
- Stencil validation (x86-64)
- Fallback behavior
