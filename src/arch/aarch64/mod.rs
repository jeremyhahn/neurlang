//! AArch64 (ARM64) Architecture Implementation
//!
//! Stub implementation of the Architecture trait for AArch64.
//! This provides the foundation for future ARM64 support.
//!
//! # Status
//!
//! Currently a stub with no stencil implementations. The interpreter
//! fallback is used for ARM64 execution.
//!
//! # Register Mapping (Planned)
//!
//! | Neurlang | ARM64 | Notes |
//! |----------|-------|-------|
//! | r0-r7 | x0-x7 | Argument/scratch registers |
//! | r8-r15 | x8-x15 | Additional scratch registers |
//! | sp | sp | Stack pointer |
//! | fp | x29 | Frame pointer |
//! | lr | x30 | Link register |
//!
//! # Key Differences from x86-64
//!
//! - Fixed 32-bit instruction encoding (vs variable length)
//! - Register-register ALU operations only (no reg-mem)
//! - Large immediates require multiple instructions (MOVZ/MOVK)
//! - Different atomic instruction encoding (LDXR/STXR pairs)
//! - PC-relative addressing modes

use super::{Architecture, CallingConvention, StencilTemplate};
use crate::ir::Opcode;

/// AArch64 architecture implementation
pub struct AArch64;

impl Architecture for AArch64 {
    const NAME: &'static str = "aarch64";
    const REGISTER_COUNT: usize = 31; // x0-x30 (x31 is SP/ZR)
    const POINTER_SIZE: usize = 8;
    const WORD_SIZE: usize = 64;
    const LITTLE_ENDIAN: bool = true; // ARM64 is typically little-endian

    fn calling_convention() -> CallingConvention {
        CallingConvention::Aapcs64
    }

    fn return_instruction() -> &'static [u8] {
        // ret (return via x30/lr)
        &[0xC0, 0x03, 0x5F, 0xD6]
    }

    fn nop_instruction() -> &'static [u8] {
        // nop
        &[0x1F, 0x20, 0x03, 0xD5]
    }

    fn generate_stencil(_opcode: Opcode, _mode: u8) -> Option<StencilTemplate> {
        // Stencils not yet implemented for ARM64
        // Falls back to interpreter
        None
    }

    fn patch_register(code: &mut [u8], offset: usize, reg: u8) {
        // ARM64 registers are typically 5 bits in various positions
        // This is a placeholder implementation
        if offset < code.len() {
            // In real implementation, would need to handle different
            // instruction formats (Rd, Rn, Rm positions vary)
            let _ = reg;
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
        // ARM64 branch offsets are in words (4-byte units)
        let word_offset = target >> 2;
        Self::patch_imm32(code, offset, word_offset);
    }
}

impl AArch64 {
    /// Map a Neurlang register to ARM64 register encoding
    pub fn map_register(reg: u8) -> u8 {
        match reg {
            0..=15 => reg, // r0-r15 -> x0-x15
            16 => 31,      // sp -> sp (x31 in certain contexts)
            17 => 29,      // fp -> x29 (frame pointer)
            18 => 30,      // lr -> x30 (link register)
            31 => 31,      // zero -> xzr (x31 in certain contexts)
            _ => 0,
        }
    }

    /// Encode an ADD instruction
    /// ADD Xd, Xn, Xm
    pub fn encode_add(rd: u8, rn: u8, rm: u8) -> u32 {
        // ADD (shifted register): 0b10001011000MMMMM00000NNNNNDDDDDD
        0x8B000000 | ((rm as u32 & 0x1F) << 16) | ((rn as u32 & 0x1F) << 5) | (rd as u32 & 0x1F)
    }

    /// Encode a SUB instruction
    /// SUB Xd, Xn, Xm
    pub fn encode_sub(rd: u8, rn: u8, rm: u8) -> u32 {
        // SUB (shifted register): 0b11001011000MMMMM00000NNNNNDDDDDD
        0xCB000000 | ((rm as u32 & 0x1F) << 16) | ((rn as u32 & 0x1F) << 5) | (rd as u32 & 0x1F)
    }

    /// Encode a MOV instruction (register-to-register)
    /// MOV Xd, Xm (alias for ORR Xd, XZR, Xm)
    pub fn encode_mov_reg(rd: u8, rm: u8) -> u32 {
        // ORR (shifted register): 0b10101010000MMMMM00000NNNNNDDDDDD
        // MOV is ORR with Rn=XZR (31)
        0xAA0003E0 // Base encoding with Rn=XZR
            | ((rm as u32 & 0x1F) << 16)
            | (rd as u32 & 0x1F)
    }

    /// Encode MOVZ (move wide with zero)
    /// MOVZ Xd, #imm16, LSL #shift
    pub fn encode_movz(rd: u8, imm16: u16, shift: u8) -> u32 {
        // MOVZ: 0b110100101HHIIIIIIIIIIIIIIIIDDDDD
        // HH = shift / 16 (0, 16, 32, 48)
        let hw = (shift / 16) as u32 & 0x3;
        0xD2800000 | (hw << 21) | ((imm16 as u32) << 5) | (rd as u32 & 0x1F)
    }

    /// Encode MOVK (move wide with keep)
    /// MOVK Xd, #imm16, LSL #shift
    pub fn encode_movk(rd: u8, imm16: u16, shift: u8) -> u32 {
        let hw = (shift / 16) as u32 & 0x3;
        0xF2800000 | (hw << 21) | ((imm16 as u32) << 5) | (rd as u32 & 0x1F)
    }

    /// Encode LDR (load register)
    /// LDR Xt, [Xn, #offset]
    pub fn encode_ldr(rt: u8, rn: u8, offset: i32) -> u32 {
        // LDR (immediate, unsigned offset): 0b11111001010IIIIIIIIIINNNNNTTTTT
        // offset must be 8-byte aligned and positive
        let imm12 = ((offset as u32) >> 3) & 0xFFF;
        0xF9400000 | (imm12 << 10) | ((rn as u32 & 0x1F) << 5) | (rt as u32 & 0x1F)
    }

    /// Encode STR (store register)
    /// STR Xt, [Xn, #offset]
    pub fn encode_str(rt: u8, rn: u8, offset: i32) -> u32 {
        let imm12 = ((offset as u32) >> 3) & 0xFFF;
        0xF9000000 | (imm12 << 10) | ((rn as u32 & 0x1F) << 5) | (rt as u32 & 0x1F)
    }

    /// Encode B (unconditional branch)
    /// B label (PC-relative)
    pub fn encode_b(offset: i32) -> u32 {
        // B: 0b000101IIIIIIIIIIIIIIIIIIIIIIIIII
        let imm26 = (offset >> 2) as u32 & 0x3FFFFFF;
        0x14000000 | imm26
    }

    /// Encode B.cond (conditional branch)
    pub fn encode_bcond(cond: u8, offset: i32) -> u32 {
        // B.cond: 0b01010100IIIIIIIIIIIIIIIIII0CCCC
        let imm19 = ((offset >> 2) as u32) & 0x7FFFF;
        0x54000000 | (imm19 << 5) | (cond as u32 & 0xF)
    }

    /// Encode RET (return from subroutine)
    pub fn encode_ret() -> u32 {
        // RET Xn (typically X30)
        0xD65F03C0
    }

    /// Condition codes for B.cond
    pub const CC_EQ: u8 = 0; // Equal
    pub const CC_NE: u8 = 1; // Not equal
    pub const CC_HS: u8 = 2; // Unsigned higher or same (carry set)
    pub const CC_LO: u8 = 3; // Unsigned lower (carry clear)
    pub const CC_MI: u8 = 4; // Minus/negative
    pub const CC_PL: u8 = 5; // Plus/positive or zero
    pub const CC_VS: u8 = 6; // Overflow set
    pub const CC_VC: u8 = 7; // Overflow clear
    pub const CC_HI: u8 = 8; // Unsigned higher
    pub const CC_LS: u8 = 9; // Unsigned lower or same
    pub const CC_GE: u8 = 10; // Signed greater or equal
    pub const CC_LT: u8 = 11; // Signed less than
    pub const CC_GT: u8 = 12; // Signed greater than
    pub const CC_LE: u8 = 13; // Signed less or equal
    pub const CC_AL: u8 = 14; // Always
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aarch64_properties() {
        assert_eq!(AArch64::NAME, "aarch64");
        assert_eq!(AArch64::REGISTER_COUNT, 31);
        assert_eq!(AArch64::POINTER_SIZE, 8);
        assert!(AArch64::LITTLE_ENDIAN);
    }

    #[test]
    fn test_encode_add() {
        // ADD X0, X1, X2
        let instr = AArch64::encode_add(0, 1, 2);
        assert_eq!(instr & 0xFF000000, 0x8B000000); // Check opcode
    }

    #[test]
    fn test_encode_mov_reg() {
        // MOV X0, X1
        let instr = AArch64::encode_mov_reg(0, 1);
        // Check it's an ORR with XZR
        assert_eq!(instr & 0xFF000000, 0xAA000000);
    }

    #[test]
    fn test_encode_ret() {
        let instr = AArch64::encode_ret();
        assert_eq!(instr, 0xD65F03C0);
    }

    #[test]
    fn test_return_instruction() {
        let ret = AArch64::return_instruction();
        assert_eq!(ret.len(), 4);
        let instr = u32::from_le_bytes([ret[0], ret[1], ret[2], ret[3]]);
        assert_eq!(instr, 0xD65F03C0);
    }
}
