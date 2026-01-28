//! x86-64 Architecture Implementation
//!
//! Full implementation of the Architecture trait for x86-64 (AMD64).
//! This is the primary target architecture with complete stencil support.
//!
//! # Register Mapping
//!
//! Neurlang registers are mapped to x86-64 registers for JIT compilation:
//!
//! | Neurlang | x86-64 | Notes |
//! |----------|--------|-------|
//! | r0 | rax | Return value |
//! | r1 | rcx | Arg 4 (Win64) / scratch |
//! | r2 | rdx | Arg 3 |
//! | r3 | rbx | Callee-saved |
//! | r4 | rsp | Stack pointer (special) |
//! | r5 | rbp | Frame pointer (callee-saved) |
//! | r6 | rsi | Arg 2 |
//! | r7 | rdi | Arg 1 (this pointer for register file) |
//! | r8-r15 | r8-r15 | Additional registers |
//!
//! # Stencil Format
//!
//! Stencils use placeholder values that get patched at runtime:
//! - `0xDEADBEEF11111111` - Destination register index
//! - `0xDEADBEEF22222222` - Source register 1 index
//! - `0xDEADBEEF33333333` - Source register 2 index
//! - `0xDEADBEEF` - 32-bit immediate value

pub mod stencils;

use super::{Architecture, CallingConvention, StencilTemplate};
use crate::ir::Opcode;

/// x86-64 architecture implementation
pub struct X86_64;

impl Architecture for X86_64 {
    const NAME: &'static str = "x86_64";
    const REGISTER_COUNT: usize = 16;
    const POINTER_SIZE: usize = 8;
    const WORD_SIZE: usize = 64;
    const LITTLE_ENDIAN: bool = true;

    fn calling_convention() -> CallingConvention {
        #[cfg(target_os = "windows")]
        {
            CallingConvention::Win64
        }
        #[cfg(not(target_os = "windows"))]
        {
            CallingConvention::SysVAmd64
        }
    }

    fn return_instruction() -> &'static [u8] {
        &[0xC3] // ret
    }

    fn nop_instruction() -> &'static [u8] {
        &[0x90] // nop
    }

    fn generate_stencil(opcode: Opcode, mode: u8) -> Option<StencilTemplate> {
        stencils::get_stencil(opcode, mode)
    }

    fn patch_register(code: &mut [u8], offset: usize, reg: u8) {
        if offset + 8 <= code.len() {
            let bytes = (reg as u64).to_le_bytes();
            code[offset..offset + 8].copy_from_slice(&bytes);
        }
    }

    fn patch_imm32(code: &mut [u8], offset: usize, imm: i32) {
        if offset + 4 <= code.len() {
            let bytes = imm.to_le_bytes();
            code[offset..offset + 4].copy_from_slice(&bytes);
        }
    }

    fn patch_imm64(code: &mut [u8], offset: usize, imm: i64) {
        if offset + 8 <= code.len() {
            let bytes = imm.to_le_bytes();
            code[offset..offset + 8].copy_from_slice(&bytes);
        }
    }

    fn patch_branch(code: &mut [u8], offset: usize, target: i32) {
        Self::patch_imm32(code, offset, target);
    }
}

impl X86_64 {
    /// Map a Neurlang register to x86-64 register encoding
    pub fn map_register(reg: u8) -> u8 {
        // x86-64 register encoding:
        // 0=rax, 1=rcx, 2=rdx, 3=rbx, 4=rsp, 5=rbp, 6=rsi, 7=rdi
        // 8=r8, 9=r9, 10=r10, 11=r11, 12=r12, 13=r13, 14=r14, 15=r15
        match reg {
            0 => 0,                          // r0 -> rax
            1 => 1,                          // r1 -> rcx
            2 => 2,                          // r2 -> rdx
            3 => 3,                          // r3 -> rbx
            4 => 6,                          // r4 -> rsi
            5 => 5,                          // r5 -> rbp
            6 => 6,                          // r6 -> rsi (shadow)
            7 => 7,                          // r7 -> rdi
            r if (8..=15).contains(&r) => r, // r8-r15 -> r8-r15
            16 => 4,                         // sp -> rsp
            17 => 5,                         // fp -> rbp
            18 => 0,                         // lr -> rax (stored in memory)
            _ => 0,                          // default to rax
        }
    }

    /// Get ModR/M byte for register-to-register operation
    pub fn modrm_reg(dst: u8, src: u8) -> u8 {
        0xC0 | ((dst & 0x7) << 3) | (src & 0x7)
    }

    /// Get REX prefix for 64-bit operands
    pub fn rex_w(dst_high: bool, src_high: bool) -> u8 {
        let mut rex = 0x48; // REX.W
        if dst_high {
            rex |= 0x04; // REX.R
        }
        if src_high {
            rex |= 0x01; // REX.B
        }
        rex
    }

    /// Generate ADD rd, rs1, rs2 instruction
    pub fn emit_add(dst: u8, src1: u8, src2: u8) -> Vec<u8> {
        let mut code = Vec::new();

        // mov dst, src1
        let rex1 = Self::rex_w(dst >= 8, src1 >= 8);
        code.push(rex1);
        code.push(0x8B); // MOV r64, r/m64
        code.push(Self::modrm_reg(dst, src1));

        // add dst, src2
        let rex2 = Self::rex_w(dst >= 8, src2 >= 8);
        code.push(rex2);
        code.push(0x03); // ADD r64, r/m64
        code.push(Self::modrm_reg(dst, src2));

        code
    }

    /// Generate SUB rd, rs1, rs2 instruction
    pub fn emit_sub(dst: u8, src1: u8, src2: u8) -> Vec<u8> {
        let mut code = Vec::new();

        // mov dst, src1
        let rex1 = Self::rex_w(dst >= 8, src1 >= 8);
        code.push(rex1);
        code.push(0x8B);
        code.push(Self::modrm_reg(dst, src1));

        // sub dst, src2
        let rex2 = Self::rex_w(dst >= 8, src2 >= 8);
        code.push(rex2);
        code.push(0x2B); // SUB r64, r/m64
        code.push(Self::modrm_reg(dst, src2));

        code
    }

    /// Generate MOV rd, imm64 instruction
    pub fn emit_mov_imm64(dst: u8, imm: i64) -> Vec<u8> {
        let mut code = Vec::new();

        // movabs dst, imm64
        let rex = Self::rex_w(false, dst >= 8);
        code.push(rex);
        code.push(0xB8 + (dst & 0x7)); // MOV r64, imm64
        code.extend_from_slice(&imm.to_le_bytes());

        code
    }

    /// Generate conditional jump
    pub fn emit_jcc(condition: u8, offset: i32) -> Vec<u8> {
        let mut code = Vec::new();

        // Jcc rel32
        code.push(0x0F);
        code.push(0x80 + condition); // JO=0, JNO=1, JB=2, JNB=3, JE=4, JNE=5, etc.
        code.extend_from_slice(&offset.to_le_bytes());

        code
    }

    /// Condition codes for Jcc instructions
    pub const CC_O: u8 = 0; // Overflow
    pub const CC_NO: u8 = 1; // Not overflow
    pub const CC_B: u8 = 2; // Below (unsigned <)
    pub const CC_AE: u8 = 3; // Above or equal (unsigned >=)
    pub const CC_E: u8 = 4; // Equal
    pub const CC_NE: u8 = 5; // Not equal
    pub const CC_BE: u8 = 6; // Below or equal (unsigned <=)
    pub const CC_A: u8 = 7; // Above (unsigned >)
    pub const CC_S: u8 = 8; // Sign (negative)
    pub const CC_NS: u8 = 9; // Not sign (non-negative)
    pub const CC_L: u8 = 12; // Less than (signed <)
    pub const CC_GE: u8 = 13; // Greater or equal (signed >=)
    pub const CC_LE: u8 = 14; // Less or equal (signed <=)
    pub const CC_G: u8 = 15; // Greater than (signed >)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x86_64_properties() {
        assert_eq!(X86_64::NAME, "x86_64");
        assert_eq!(X86_64::REGISTER_COUNT, 16);
        assert_eq!(X86_64::return_instruction(), &[0xC3]);
    }

    #[test]
    fn test_register_mapping() {
        assert_eq!(X86_64::map_register(0), 0); // r0 -> rax
        assert_eq!(X86_64::map_register(16), 4); // sp -> rsp
    }

    #[test]
    fn test_emit_add() {
        let code = X86_64::emit_add(0, 1, 2);
        assert!(!code.is_empty());
        // Check for REX.W prefix
        assert!((code[0] & 0x48) == 0x48);
    }

    #[test]
    fn test_emit_mov_imm64() {
        let code = X86_64::emit_mov_imm64(0, 0x12345678);
        assert_eq!(code.len(), 10); // REX + opcode + 8 bytes immediate
    }
}
