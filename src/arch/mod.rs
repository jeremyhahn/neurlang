//! Architecture Abstraction Module for Neurlang
//!
//! This module provides a unified interface for different CPU architectures,
//! enabling cross-platform code generation and execution.
//!
//! # Supported Architectures
//!
//! | Architecture | Status | Notes |
//! |--------------|--------|-------|
//! | x86-64 | Full | Primary target, all stencils implemented |
//! | ARM64 (AArch64) | Stub | Foundation for future implementation |
//! | RISC-V 64 | Stub | Foundation for future implementation |
//!
//! # Architecture Abstraction
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Architecture Trait                        │
//! │  fn name() -> &str                                          │
//! │  fn register_count() -> usize                               │
//! │  fn generate_stencil(opcode, mode) -> StencilTemplate       │
//! │  fn patch_register(code, offset, reg)                       │
//! │  fn patch_immediate(code, offset, imm)                      │
//! │  fn return_instruction() -> &[u8]                           │
//! │  fn calling_convention() -> CallingConvention               │
//! └────────────┬───────────────────────┬───────────────────────┘
//!              │                       │
//!       ┌──────┴───────┐        ┌──────┴───────┐
//!       │   X86_64     │        │   AArch64    │   ...
//!       └──────────────┘        └──────────────┘
//! ```

pub mod aarch64;
pub mod riscv64;
pub mod x86_64;

use crate::ir::Opcode;

/// Re-export architecture implementations
pub use aarch64::AArch64;
pub use riscv64::RiscV64;
pub use x86_64::X86_64;

/// Calling convention for function calls
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallingConvention {
    /// System V AMD64 ABI (Linux, macOS, BSD)
    SysVAmd64,
    /// Windows x64 calling convention
    Win64,
    /// ARM64 AAPCS (Procedure Call Standard)
    Aapcs64,
    /// RISC-V calling convention
    RiscV,
}

impl CallingConvention {
    /// Get the registers used for integer arguments
    pub fn arg_registers(&self) -> &'static [u8] {
        match self {
            CallingConvention::SysVAmd64 => &[7, 6, 2, 1, 8, 9], // rdi, rsi, rdx, rcx, r8, r9
            CallingConvention::Win64 => &[1, 2, 8, 9],           // rcx, rdx, r8, r9
            CallingConvention::Aapcs64 => &[0, 1, 2, 3, 4, 5, 6, 7], // x0-x7
            CallingConvention::RiscV => &[10, 11, 12, 13, 14, 15, 16, 17], // a0-a7
        }
    }

    /// Get the register used for return values
    pub fn return_register(&self) -> u8 {
        match self {
            CallingConvention::SysVAmd64 | CallingConvention::Win64 => 0, // rax
            CallingConvention::Aapcs64 => 0,                              // x0
            CallingConvention::RiscV => 10,                               // a0
        }
    }

    /// Get the callee-saved registers
    pub fn callee_saved(&self) -> &'static [u8] {
        match self {
            CallingConvention::SysVAmd64 => &[3, 5, 12, 13, 14, 15], // rbx, rbp, r12-r15
            CallingConvention::Win64 => &[3, 5, 6, 7, 12, 13, 14, 15], // rbx, rbp, rsi, rdi, r12-r15
            CallingConvention::Aapcs64 => &[19, 20, 21, 22, 23, 24, 25, 26, 27, 28], // x19-x28
            CallingConvention::RiscV => &[8, 9, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27], // s0-s11
        }
    }
}

/// Stencil template with placeholders for patching
#[derive(Debug, Clone)]
pub struct StencilTemplate {
    /// Raw machine code bytes with placeholders
    pub code: Vec<u8>,
    /// Patch locations: (offset, patch_type)
    pub patches: Vec<PatchLocation>,
    /// Total size in bytes
    pub size: usize,
}

/// Location and type of a patch in a stencil
#[derive(Debug, Clone, Copy)]
pub struct PatchLocation {
    /// Byte offset in the stencil
    pub offset: usize,
    /// Type of patch to apply
    pub kind: PatchKind,
}

/// Type of value to patch into a stencil
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchKind {
    /// Destination register index
    DstReg,
    /// Source register 1 index
    Src1Reg,
    /// Source register 2 index
    Src2Reg,
    /// 8-bit immediate
    Imm8,
    /// 16-bit immediate
    Imm16,
    /// 32-bit immediate
    Imm32,
    /// 64-bit immediate
    Imm64,
    /// Relative branch offset (32-bit)
    BranchRel32,
    /// Absolute address (64-bit)
    AbsAddr64,
}

/// Trait for CPU architecture implementations
pub trait Architecture: Sized {
    /// Architecture name (e.g., "x86_64", "aarch64")
    const NAME: &'static str;

    /// Number of general-purpose registers
    const REGISTER_COUNT: usize;

    /// Pointer size in bytes
    const POINTER_SIZE: usize;

    /// Native word size in bits
    const WORD_SIZE: usize;

    /// Whether the architecture is little-endian
    const LITTLE_ENDIAN: bool;

    /// Get the calling convention for this architecture
    fn calling_convention() -> CallingConvention;

    /// Get the return instruction bytes
    fn return_instruction() -> &'static [u8];

    /// Get the NOP instruction bytes
    fn nop_instruction() -> &'static [u8];

    /// Generate a stencil template for an opcode
    fn generate_stencil(opcode: Opcode, mode: u8) -> Option<StencilTemplate>;

    /// Patch a register value into machine code
    fn patch_register(code: &mut [u8], offset: usize, reg: u8);

    /// Patch a 32-bit immediate value into machine code
    fn patch_imm32(code: &mut [u8], offset: usize, imm: i32);

    /// Patch a 64-bit immediate value into machine code
    fn patch_imm64(code: &mut [u8], offset: usize, imm: i64);

    /// Patch a relative branch offset
    fn patch_branch(code: &mut [u8], offset: usize, target: i32);

    /// Check if the current system supports this architecture
    fn is_native() -> bool {
        #[cfg(target_arch = "x86_64")]
        {
            Self::NAME == "x86_64"
        }
        #[cfg(target_arch = "aarch64")]
        {
            Self::NAME == "aarch64"
        }
        #[cfg(target_arch = "riscv64")]
        {
            Self::NAME == "riscv64"
        }
        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "riscv64"
        )))]
        {
            false
        }
    }
}

/// Get the native architecture type
#[cfg(target_arch = "x86_64")]
pub type NativeArch = X86_64;

#[cfg(target_arch = "aarch64")]
pub type NativeArch = AArch64;

#[cfg(target_arch = "riscv64")]
pub type NativeArch = RiscV64;

#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "riscv64"
)))]
pub type NativeArch = X86_64; // Fallback for unsupported architectures

/// Detect the current runtime architecture
pub fn detect_arch() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    {
        "x86_64"
    }
    #[cfg(target_arch = "aarch64")]
    {
        "aarch64"
    }
    #[cfg(target_arch = "riscv64")]
    {
        "riscv64"
    }
    #[cfg(target_arch = "arm")]
    {
        "arm"
    }
    #[cfg(target_arch = "x86")]
    {
        "x86"
    }
    #[cfg(not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64",
        target_arch = "arm",
        target_arch = "x86"
    )))]
    {
        "unknown"
    }
}

/// Check if JIT compilation is available for the current platform
pub fn jit_available() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        true // Full support
    }
    #[cfg(target_arch = "aarch64")]
    {
        false // Stubs only
    }
    #[cfg(target_arch = "riscv64")]
    {
        false // Stubs only
    }
    #[cfg(not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64"
    )))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_arch() {
        let arch = detect_arch();
        assert!(!arch.is_empty());
        println!("Detected architecture: {}", arch);
    }

    #[test]
    fn test_calling_convention() {
        let cc = CallingConvention::SysVAmd64;
        assert_eq!(cc.return_register(), 0);
        assert!(!cc.arg_registers().is_empty());
    }

    #[test]
    fn test_x86_64_properties() {
        assert_eq!(X86_64::NAME, "x86_64");
        assert_eq!(X86_64::REGISTER_COUNT, 16);
        assert_eq!(X86_64::POINTER_SIZE, 8);
        assert!(X86_64::LITTLE_ENDIAN);
    }

    #[test]
    fn test_aarch64_properties() {
        assert_eq!(AArch64::NAME, "aarch64");
        assert_eq!(AArch64::REGISTER_COUNT, 31);
        assert_eq!(AArch64::POINTER_SIZE, 8);
        assert!(AArch64::LITTLE_ENDIAN);
    }
}
