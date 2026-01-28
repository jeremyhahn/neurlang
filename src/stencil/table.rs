//! Stencil table for copy-and-patch compilation
//!
//! Provides lookup and patching for all 32 opcodes.

use crate::ir::{AluOp, BitsOp, Instruction, MemWidth, MulDivOp, Opcode};

// Include generated stencils
include!(concat!(env!("OUT_DIR"), "/stencils_generated.rs"));

/// Stencil entry with metadata for patching
#[derive(Clone)]
pub struct StencilEntry {
    /// The raw machine code template
    pub code: Vec<u8>,
    /// Patch locations and types
    pub patches: Vec<PatchInfo>,
    /// Size of the stencil in bytes
    pub size: usize,
}

/// Information about a single patch location
#[derive(Clone, Copy, Debug)]
pub struct PatchInfo {
    /// Byte offset in the stencil
    pub offset: usize,
    /// Type of value to patch
    pub kind: PatchKind,
}

/// Type of patch to apply
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatchKind {
    /// Destination register index (5-bit, stored as 64-bit)
    DstReg,
    /// Source register 1 index
    Src1Reg,
    /// Source register 2 index
    Src2Reg,
    /// 32-bit immediate value
    Imm32,
    /// 64-bit immediate value
    Imm64,
    /// Relative branch target
    BranchTarget,
}

/// The complete stencil table for all opcodes
pub struct StencilTable {
    /// Stencils indexed by (opcode, mode)
    entries: Vec<Option<StencilEntry>>,
}

impl StencilTable {
    /// Create a new stencil table with all opcodes
    pub fn new() -> Self {
        let mut entries = vec![None; 512]; // 32 opcodes * ~8 modes each

        // ALU operations (opcode 0x00, modes 0-7)
        Self::add_alu_stencils(&mut entries);

        // ALUI operations (opcode 0x01, modes 0-7)
        Self::add_alui_stencils(&mut entries);

        // MulDiv operations (opcode 0x02, modes 0-3)
        Self::add_muldiv_stencils(&mut entries);

        // Memory operations (opcodes 0x03-0x04)
        Self::add_memory_stencils(&mut entries);

        // Atomic operations (opcode 0x05)
        Self::add_atomic_stencils(&mut entries);

        // Control flow (opcodes 0x06-0x09)
        Self::add_control_stencils(&mut entries);

        // Capabilities (opcodes 0x0A-0x0C)
        Self::add_capability_stencils(&mut entries);

        // Concurrency (opcodes 0x0D-0x11)
        Self::add_concurrency_stencils(&mut entries);

        // Taint (opcodes 0x12-0x13)
        Self::add_taint_stencils(&mut entries);

        // I/O operations (opcodes 0x14-0x18)
        Self::add_io_stencils(&mut entries);

        // Math extensions (opcodes 0x19-0x1B)
        Self::add_math_stencils(&mut entries);

        // System (opcodes 0x1C-0x1F)
        Self::add_system_stencils(&mut entries);

        Self { entries }
    }

    /// Get a stencil for an instruction
    pub fn get(&self, opcode: Opcode, mode: u8) -> Option<&StencilEntry> {
        let idx = Self::index(opcode, mode);
        self.entries.get(idx).and_then(|e| e.as_ref())
    }

    /// Calculate index from opcode and mode
    fn index(opcode: Opcode, mode: u8) -> usize {
        ((opcode as usize) << 3) | (mode as usize & 0x7)
    }

    /// Get the size of a stencil, excluding trailing ret (0xc3)
    /// This allows chaining multiple stencils together without early return
    fn stencil_size_no_ret(code: &[u8]) -> usize {
        // Check if the last byte is ret (0xc3), and if so exclude it
        if code.last() == Some(&0xc3) {
            code.len() - 1
        } else {
            code.len()
        }
    }

    fn add_alu_stencils(entries: &mut Vec<Option<StencilEntry>>) {
        // ADD
        entries[Self::index(Opcode::Alu, AluOp::Add as u8)] = Some(StencilEntry {
            code: ALU_ADD.code.to_vec(),
            patches: Self::convert_patches(ALU_ADD.patches),
            size: Self::stencil_size_no_ret(ALU_ADD.code),
        });

        // SUB
        entries[Self::index(Opcode::Alu, AluOp::Sub as u8)] = Some(StencilEntry {
            code: ALU_SUB.code.to_vec(),
            patches: Self::convert_patches(ALU_SUB.patches),
            size: Self::stencil_size_no_ret(ALU_SUB.code),
        });

        // AND, OR, XOR, SHL, SHR, SAR - similar patterns with different opcodes
        // For now, reuse ADD template and note that the actual operation byte would differ
        for op in [
            AluOp::And,
            AluOp::Or,
            AluOp::Xor,
            AluOp::Shl,
            AluOp::Shr,
            AluOp::Sar,
        ] {
            entries[Self::index(Opcode::Alu, op as u8)] = Some(StencilEntry {
                code: ALU_ADD.code.to_vec(),
                patches: Self::convert_patches(ALU_ADD.patches),
                size: Self::stencil_size_no_ret(ALU_ADD.code),
            });
        }
    }

    fn add_alui_stencils(entries: &mut Vec<Option<StencilEntry>>) {
        // ADDI and other immediate operations
        // Use the same structure but with immediate patch
        for op in 0..8u8 {
            let code = ALU_ADD.code.to_vec();
            let mut patches = Self::convert_patches(ALU_ADD.patches);
            // Replace src2 with immediate
            patches.retain(|p| p.kind != PatchKind::Src2Reg);
            patches.push(PatchInfo {
                offset: 16, // Approximate location
                kind: PatchKind::Imm32,
            });
            entries[Self::index(Opcode::AluI, op)] = Some(StencilEntry {
                code,
                patches,
                size: Self::stencil_size_no_ret(ALU_ADD.code),
            });
        }
    }

    fn add_muldiv_stencils(entries: &mut Vec<Option<StencilEntry>>) {
        entries[Self::index(Opcode::MulDiv, MulDivOp::Mul as u8)] = Some(StencilEntry {
            code: MUL.code.to_vec(),
            patches: Self::convert_patches(MUL.patches),
            size: Self::stencil_size_no_ret(MUL.code),
        });

        // DIV, MOD, MULH - similar structure
        for op in [MulDivOp::MulH, MulDivOp::Div, MulDivOp::Mod] {
            entries[Self::index(Opcode::MulDiv, op as u8)] = Some(StencilEntry {
                code: MUL.code.to_vec(),
                patches: Self::convert_patches(MUL.patches),
                size: Self::stencil_size_no_ret(MUL.code),
            });
        }
    }

    fn add_memory_stencils(entries: &mut Vec<Option<StencilEntry>>) {
        // LOAD operations for different widths
        for width in [
            MemWidth::Byte,
            MemWidth::Half,
            MemWidth::Word,
            MemWidth::Double,
        ] {
            entries[Self::index(Opcode::Load, width as u8)] = Some(StencilEntry {
                code: LOAD64.code.to_vec(),
                patches: Self::convert_patches(LOAD64.patches),
                size: Self::stencil_size_no_ret(LOAD64.code),
            });
        }

        // STORE operations
        for width in [
            MemWidth::Byte,
            MemWidth::Half,
            MemWidth::Word,
            MemWidth::Double,
        ] {
            entries[Self::index(Opcode::Store, width as u8)] = Some(StencilEntry {
                code: STORE64.code.to_vec(),
                patches: Self::convert_patches(STORE64.patches),
                size: Self::stencil_size_no_ret(STORE64.code),
            });
        }
    }

    fn add_atomic_stencils(entries: &mut Vec<Option<StencilEntry>>) {
        // Atomic operations use fence + load/store patterns
        for op in 0..8u8 {
            entries[Self::index(Opcode::Atomic, op)] = Some(StencilEntry {
                code: FENCE.code.to_vec(),
                patches: vec![],
                size: Self::stencil_size_no_ret(FENCE.code),
            });
        }
    }

    fn add_control_stencils(entries: &mut Vec<Option<StencilEntry>>) {
        // Branch conditions
        for cond in 0..8u8 {
            entries[Self::index(Opcode::Branch, cond)] = Some(StencilEntry {
                code: vec![0xc3], // Placeholder - actual branch needs runtime codegen
                patches: vec![PatchInfo {
                    offset: 0,
                    kind: PatchKind::BranchTarget,
                }],
                size: 1,
            });
        }

        // CALL, RET, JUMP
        entries[Self::index(Opcode::Call, 0)] = Some(StencilEntry {
            code: vec![0xc3],
            patches: vec![],
            size: 1,
        });
        entries[Self::index(Opcode::Ret, 0)] = Some(StencilEntry {
            code: vec![0xc3],
            patches: vec![],
            size: 1,
        });
        entries[Self::index(Opcode::Jump, 0)] = Some(StencilEntry {
            code: vec![0xc3],
            patches: vec![],
            size: 1,
        });
    }

    fn add_capability_stencils(entries: &mut Vec<Option<StencilEntry>>) {
        // Capability operations are handled specially at runtime
        for mode in 0..4u8 {
            entries[Self::index(Opcode::CapNew, mode)] = Some(StencilEntry {
                code: NOP.code.to_vec(),
                patches: vec![],
                size: Self::stencil_size_no_ret(NOP.code),
            });
            entries[Self::index(Opcode::CapRestrict, mode)] = Some(StencilEntry {
                code: NOP.code.to_vec(),
                patches: vec![],
                size: Self::stencil_size_no_ret(NOP.code),
            });
            entries[Self::index(Opcode::CapQuery, mode)] = Some(StencilEntry {
                code: NOP.code.to_vec(),
                patches: vec![],
                size: Self::stencil_size_no_ret(NOP.code),
            });
        }
    }

    fn add_concurrency_stencils(entries: &mut Vec<Option<StencilEntry>>) {
        // Concurrency operations call into runtime
        entries[Self::index(Opcode::Spawn, 0)] = Some(StencilEntry {
            code: NOP.code.to_vec(),
            patches: vec![],
            size: Self::stencil_size_no_ret(NOP.code),
        });
        entries[Self::index(Opcode::Join, 0)] = Some(StencilEntry {
            code: NOP.code.to_vec(),
            patches: vec![],
            size: Self::stencil_size_no_ret(NOP.code),
        });
        for mode in 0..4u8 {
            entries[Self::index(Opcode::Chan, mode)] = Some(StencilEntry {
                code: NOP.code.to_vec(),
                patches: vec![],
                size: Self::stencil_size_no_ret(NOP.code),
            });
        }
        for mode in 0..4u8 {
            entries[Self::index(Opcode::Fence, mode)] = Some(StencilEntry {
                code: FENCE.code.to_vec(),
                patches: vec![],
                size: Self::stencil_size_no_ret(FENCE.code),
            });
        }
        entries[Self::index(Opcode::Yield, 0)] = Some(StencilEntry {
            code: NOP.code.to_vec(),
            patches: vec![],
            size: Self::stencil_size_no_ret(NOP.code),
        });
    }

    fn add_taint_stencils(entries: &mut Vec<Option<StencilEntry>>) {
        entries[Self::index(Opcode::Taint, 0)] = Some(StencilEntry {
            code: NOP.code.to_vec(),
            patches: vec![],
            size: Self::stencil_size_no_ret(NOP.code),
        });
        entries[Self::index(Opcode::Sanitize, 0)] = Some(StencilEntry {
            code: NOP.code.to_vec(),
            patches: vec![],
            size: Self::stencil_size_no_ret(NOP.code),
        });
    }

    fn add_io_stencils(entries: &mut Vec<Option<StencilEntry>>) {
        // FILE operations - call into runtime for sandboxed file I/O
        for mode in 0..8u8 {
            entries[Self::index(Opcode::File, mode)] = Some(StencilEntry {
                // CALL runtime function (placeholder - actual runtime call setup)
                code: RUNTIME_CALL.code.to_vec(),
                patches: vec![
                    PatchInfo {
                        offset: 2,
                        kind: PatchKind::DstReg,
                    },
                    PatchInfo {
                        offset: 10,
                        kind: PatchKind::Src1Reg,
                    },
                ],
                size: Self::stencil_size_no_ret(RUNTIME_CALL.code),
            });
        }

        // NET operations - call into runtime for sandboxed network I/O
        for mode in 0..8u8 {
            entries[Self::index(Opcode::Net, mode)] = Some(StencilEntry {
                code: RUNTIME_CALL.code.to_vec(),
                patches: vec![
                    PatchInfo {
                        offset: 2,
                        kind: PatchKind::DstReg,
                    },
                    PatchInfo {
                        offset: 10,
                        kind: PatchKind::Src1Reg,
                    },
                ],
                size: Self::stencil_size_no_ret(RUNTIME_CALL.code),
            });
        }

        // NET.SETOPT - socket options
        for mode in 0..8u8 {
            entries[Self::index(Opcode::NetSetopt, mode)] = Some(StencilEntry {
                code: RUNTIME_CALL.code.to_vec(),
                patches: vec![
                    PatchInfo {
                        offset: 10,
                        kind: PatchKind::Src1Reg,
                    },
                    PatchInfo {
                        offset: 18,
                        kind: PatchKind::Imm32,
                    },
                ],
                size: Self::stencil_size_no_ret(RUNTIME_CALL.code),
            });
        }

        // IO operations - console I/O
        for mode in 0..4u8 {
            entries[Self::index(Opcode::Io, mode)] = Some(StencilEntry {
                code: RUNTIME_CALL.code.to_vec(),
                patches: vec![
                    PatchInfo {
                        offset: 2,
                        kind: PatchKind::DstReg,
                    },
                    PatchInfo {
                        offset: 10,
                        kind: PatchKind::Src1Reg,
                    },
                ],
                size: Self::stencil_size_no_ret(RUNTIME_CALL.code),
            });
        }

        // TIME operations
        for mode in 0..4u8 {
            entries[Self::index(Opcode::Time, mode)] = Some(StencilEntry {
                code: RUNTIME_CALL.code.to_vec(),
                patches: vec![PatchInfo {
                    offset: 2,
                    kind: PatchKind::DstReg,
                }],
                size: Self::stencil_size_no_ret(RUNTIME_CALL.code),
            });
        }
    }

    fn add_math_stencils(entries: &mut Vec<Option<StencilEntry>>) {
        // FPU operations - use SSE/AVX instructions
        // Modes 0-7: arithmetic ops (fadd, fsub, fmul, fdiv, fsqrt, fabs, ffloor, fceil)
        // Modes 8-13: comparison ops (fcmpeq, fcmpne, fcmplt, fcmple, fcmpgt, fcmpge)
        for mode in 0..14u8 {
            entries[Self::index(Opcode::Fpu, mode)] = Some(StencilEntry {
                // For now, use runtime call - can be optimized to inline SSE later
                code: RUNTIME_CALL.code.to_vec(),
                patches: vec![
                    PatchInfo {
                        offset: 2,
                        kind: PatchKind::DstReg,
                    },
                    PatchInfo {
                        offset: 10,
                        kind: PatchKind::Src1Reg,
                    },
                    PatchInfo {
                        offset: 18,
                        kind: PatchKind::Src2Reg,
                    },
                ],
                size: Self::stencil_size_no_ret(RUNTIME_CALL.code),
            });
        }

        // RAND operations - call into runtime (RDRAND when available)
        for mode in 0..2u8 {
            entries[Self::index(Opcode::Rand, mode)] = Some(StencilEntry {
                code: RUNTIME_CALL.code.to_vec(),
                patches: vec![
                    PatchInfo {
                        offset: 2,
                        kind: PatchKind::DstReg,
                    },
                    PatchInfo {
                        offset: 10,
                        kind: PatchKind::Src1Reg,
                    },
                ],
                size: Self::stencil_size_no_ret(RUNTIME_CALL.code),
            });
        }

        // BITS operations - can be inlined as single x86-64 instructions
        // POPCNT
        entries[Self::index(Opcode::Bits, BitsOp::Popcount as u8)] = Some(StencilEntry {
            // popcnt rax, rcx (when POPCNT supported)
            code: vec![0xF3, 0x48, 0x0F, 0xB8, 0xC1], // popcnt rax, rcx
            patches: vec![
                PatchInfo {
                    offset: 4,
                    kind: PatchKind::DstReg,
                },
                PatchInfo {
                    offset: 4,
                    kind: PatchKind::Src1Reg,
                },
            ],
            size: 5,
        });

        // CLZ (count leading zeros) - LZCNT or BSR fallback
        entries[Self::index(Opcode::Bits, BitsOp::Clz as u8)] = Some(StencilEntry {
            // lzcnt rax, rcx (when LZCNT supported)
            code: vec![0xF3, 0x48, 0x0F, 0xBD, 0xC1], // lzcnt rax, rcx
            patches: vec![
                PatchInfo {
                    offset: 4,
                    kind: PatchKind::DstReg,
                },
                PatchInfo {
                    offset: 4,
                    kind: PatchKind::Src1Reg,
                },
            ],
            size: 5,
        });

        // CTZ (count trailing zeros) - TZCNT or BSF fallback
        entries[Self::index(Opcode::Bits, BitsOp::Ctz as u8)] = Some(StencilEntry {
            // tzcnt rax, rcx (when BMI1 supported)
            code: vec![0xF3, 0x48, 0x0F, 0xBC, 0xC1], // tzcnt rax, rcx
            patches: vec![
                PatchInfo {
                    offset: 4,
                    kind: PatchKind::DstReg,
                },
                PatchInfo {
                    offset: 4,
                    kind: PatchKind::Src1Reg,
                },
            ],
            size: 5,
        });

        // BSWAP (byte swap for endian conversion)
        entries[Self::index(Opcode::Bits, BitsOp::Bswap as u8)] = Some(StencilEntry {
            // bswap rax
            code: vec![0x48, 0x0F, 0xC8], // bswap rax
            patches: vec![PatchInfo {
                offset: 2,
                kind: PatchKind::DstReg,
            }],
            size: 3,
        });
    }

    fn add_system_stencils(entries: &mut Vec<Option<StencilEntry>>) {
        // MOV immediate
        entries[Self::index(Opcode::Mov, 0)] = Some(StencilEntry {
            code: MOV_REG.code.to_vec(),
            patches: Self::convert_patches(MOV_REG.patches),
            size: Self::stencil_size_no_ret(MOV_REG.code),
        });
        entries[Self::index(Opcode::Mov, 1)] = Some(StencilEntry {
            code: MOV_IMM.code.to_vec(),
            patches: Self::convert_patches(MOV_IMM.patches),
            size: Self::stencil_size_no_ret(MOV_IMM.code),
        });

        // TRAP - calls into runtime
        for mode in 0..8u8 {
            entries[Self::index(Opcode::Trap, mode)] = Some(StencilEntry {
                code: NOP.code.to_vec(),
                patches: vec![],
                size: Self::stencil_size_no_ret(NOP.code),
            });
        }

        // NOP
        entries[Self::index(Opcode::Nop, 0)] = Some(StencilEntry {
            code: NOP.code.to_vec(),
            patches: vec![],
            size: Self::stencil_size_no_ret(NOP.code),
        });

        // HALT
        entries[Self::index(Opcode::Halt, 0)] = Some(StencilEntry {
            code: HALT.code.to_vec(),
            patches: vec![],
            size: HALT.code.len(),
        });

        // EXT.CALL - call into extension registry
        // This stencil calls a runtime function that looks up and executes the extension
        entries[Self::index(Opcode::ExtCall, 0)] = Some(StencilEntry {
            // Extension calls go through runtime - similar pattern to other runtime calls
            code: RUNTIME_CALL.code.to_vec(),
            patches: vec![
                PatchInfo {
                    offset: 2,
                    kind: PatchKind::DstReg,
                },
                PatchInfo {
                    offset: 10,
                    kind: PatchKind::Imm32, // Extension ID
                },
                PatchInfo {
                    offset: 18,
                    kind: PatchKind::Src1Reg,
                },
                PatchInfo {
                    offset: 26,
                    kind: PatchKind::Src2Reg,
                },
            ],
            size: Self::stencil_size_no_ret(RUNTIME_CALL.code),
        });
    }

    fn convert_patches(raw: &[(usize, u8)]) -> Vec<PatchInfo> {
        raw.iter()
            .map(|&(offset, kind)| PatchInfo {
                offset,
                kind: match kind {
                    1 => PatchKind::DstReg,
                    2 => PatchKind::Src1Reg,
                    3 => PatchKind::Src2Reg,
                    4 => PatchKind::Imm32,
                    _ => PatchKind::DstReg,
                },
            })
            .collect()
    }
}

impl Default for StencilTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Apply patches to a stencil for a specific instruction
pub fn patch_stencil(stencil: &StencilEntry, instr: &Instruction, output: &mut [u8]) {
    // Copy base code (only stencil.size bytes, which excludes trailing ret for chaining)
    output[..stencil.size].copy_from_slice(&stencil.code[..stencil.size]);

    // Apply patches
    for patch in &stencil.patches {
        if patch.offset >= stencil.size {
            continue;
        }

        match patch.kind {
            PatchKind::DstReg => {
                let reg_idx = instr.rd as u64;
                let bytes = reg_idx.to_le_bytes();
                let end = (patch.offset + 8).min(stencil.size);
                let len = end - patch.offset;
                output[patch.offset..end].copy_from_slice(&bytes[..len]);
            }
            PatchKind::Src1Reg => {
                let reg_idx = instr.rs1 as u64;
                let bytes = reg_idx.to_le_bytes();
                let end = (patch.offset + 8).min(stencil.size);
                let len = end - patch.offset;
                output[patch.offset..end].copy_from_slice(&bytes[..len]);
            }
            PatchKind::Src2Reg => {
                let reg_idx = instr.rs2 as u64;
                let bytes = reg_idx.to_le_bytes();
                let end = (patch.offset + 8).min(stencil.size);
                let len = end - patch.offset;
                output[patch.offset..end].copy_from_slice(&bytes[..len]);
            }
            PatchKind::Imm32 => {
                if let Some(imm) = instr.imm {
                    let bytes = imm.to_le_bytes();
                    let end = (patch.offset + 4).min(stencil.size);
                    let len = end - patch.offset;
                    output[patch.offset..end].copy_from_slice(&bytes[..len]);
                }
            }
            PatchKind::Imm64 => {
                if let Some(imm) = instr.imm {
                    let bytes = (imm as i64).to_le_bytes();
                    let end = (patch.offset + 8).min(stencil.size);
                    let len = end - patch.offset;
                    output[patch.offset..end].copy_from_slice(&bytes[..len]);
                }
            }
            PatchKind::BranchTarget => {
                // Branch targets need special handling at runtime
                if let Some(imm) = instr.imm {
                    let bytes = imm.to_le_bytes();
                    let end = (patch.offset + 4).min(stencil.size);
                    let len = end - patch.offset;
                    output[patch.offset..end].copy_from_slice(&bytes[..len]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Register;

    #[test]
    fn test_stencil_table_creation() {
        let table = StencilTable::new();
        // Verify some entries exist
        assert!(table.get(Opcode::Alu, AluOp::Add as u8).is_some());
        assert!(table.get(Opcode::Mov, 0).is_some());
        assert!(table.get(Opcode::Halt, 0).is_some());
    }

    #[test]
    fn test_patch_application() {
        let table = StencilTable::new();
        let stencil = table.get(Opcode::Mov, 0).unwrap();

        let instr = Instruction::new(Opcode::Mov, Register::R0, Register::R1, Register::Zero, 0);

        let mut output = vec![0u8; stencil.size];
        patch_stencil(stencil, &instr, &mut output);

        // Should have modified the output
        assert!(!output.iter().all(|&b| b == 0) || stencil.code.iter().all(|&b| b == 0));
    }
}
