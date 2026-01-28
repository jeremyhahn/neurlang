//! x86-64 Stencil Definitions
//!
//! Pre-compiled machine code templates for copy-and-patch compilation.
//! These stencils contain placeholder values that are patched at runtime.

use super::super::{PatchKind, PatchLocation, StencilTemplate};
use crate::ir::{AluOp, MemWidth, MulDivOp, Opcode};

/// Placeholder values used in stencils
pub const PLACEHOLDER_DST: u64 = 0xDEADBEEF11111111;
pub const PLACEHOLDER_SRC1: u64 = 0xDEADBEEF22222222;
pub const PLACEHOLDER_SRC2: u64 = 0xDEADBEEF33333333;
pub const PLACEHOLDER_IMM: u32 = 0xDEADBEEF;

/// Get the stencil for a given opcode and mode
pub fn get_stencil(opcode: Opcode, mode: u8) -> Option<StencilTemplate> {
    match opcode {
        Opcode::Alu => get_alu_stencil(mode),
        Opcode::AluI => get_alui_stencil(mode),
        Opcode::MulDiv => get_muldiv_stencil(mode),
        Opcode::Load => get_load_stencil(mode),
        Opcode::Store => get_store_stencil(mode),
        Opcode::Mov => get_mov_stencil(mode),
        Opcode::Nop => Some(nop_stencil()),
        Opcode::Halt => Some(halt_stencil()),
        Opcode::Ret => Some(ret_stencil()),
        Opcode::Fence => Some(fence_stencil()),
        _ => None, // Other opcodes use runtime calls
    }
}

/// ALU operation stencils
fn get_alu_stencil(mode: u8) -> Option<StencilTemplate> {
    let op = AluOp::from_u8(mode)?;

    // Common stencil structure:
    // movabs rax, PLACEHOLDER_SRC1    ; load src1 reg index
    // mov rax, [rdi + rax*8]          ; load src1 value from register file
    // movabs rcx, PLACEHOLDER_SRC2    ; load src2 reg index
    // mov rcx, [rdi + rcx*8]          ; load src2 value from register file
    // <operation> rax, rcx            ; perform ALU operation
    // movabs rcx, PLACEHOLDER_DST     ; load dst reg index
    // mov [rdi + rcx*8], rax          ; store result to register file
    // ret

    let code = match op {
        AluOp::Add => {
            vec![
                // movabs rax, src1_idx
                0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
                // mov rax, [rdi + rax*8]
                0x48, 0x8b, 0x04, 0xc7, // movabs rcx, src2_idx
                0x48, 0xb9, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33,
                // mov rcx, [rdi + rcx*8]
                0x48, 0x8b, 0x0c, 0xcf, // add rax, rcx
                0x48, 0x01, 0xc8, // movabs rcx, dst_idx
                0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
                // mov [rdi + rcx*8], rax
                0x48, 0x89, 0x04, 0xcf, // ret
                0xc3,
            ]
        }
        AluOp::Sub => {
            vec![
                0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x48, 0x8b, 0x04, 0xc7,
                0x48, 0xb9, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x48, 0x8b, 0x0c, 0xcf,
                // sub rax, rcx
                0x48, 0x29, 0xc8, 0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x48,
                0x89, 0x04, 0xcf, 0xc3,
            ]
        }
        AluOp::And => {
            vec![
                0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x48, 0x8b, 0x04, 0xc7,
                0x48, 0xb9, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x48, 0x8b, 0x0c, 0xcf,
                // and rax, rcx
                0x48, 0x21, 0xc8, 0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x48,
                0x89, 0x04, 0xcf, 0xc3,
            ]
        }
        AluOp::Or => {
            vec![
                0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x48, 0x8b, 0x04, 0xc7,
                0x48, 0xb9, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x48, 0x8b, 0x0c, 0xcf,
                // or rax, rcx
                0x48, 0x09, 0xc8, 0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x48,
                0x89, 0x04, 0xcf, 0xc3,
            ]
        }
        AluOp::Xor => {
            vec![
                0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x48, 0x8b, 0x04, 0xc7,
                0x48, 0xb9, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x48, 0x8b, 0x0c, 0xcf,
                // xor rax, rcx
                0x48, 0x31, 0xc8, 0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x48,
                0x89, 0x04, 0xcf, 0xc3,
            ]
        }
        AluOp::Shl => {
            vec![
                0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x48, 0x8b, 0x04, 0xc7,
                0x48, 0xb9, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x48, 0x8b, 0x0c, 0xcf,
                // shl rax, cl
                0x48, 0xd3, 0xe0, 0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x48,
                0x89, 0x04, 0xcf, 0xc3,
            ]
        }
        AluOp::Shr => {
            vec![
                0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x48, 0x8b, 0x04, 0xc7,
                0x48, 0xb9, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x48, 0x8b, 0x0c, 0xcf,
                // shr rax, cl
                0x48, 0xd3, 0xe8, 0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x48,
                0x89, 0x04, 0xcf, 0xc3,
            ]
        }
        AluOp::Sar => {
            vec![
                0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x48, 0x8b, 0x04, 0xc7,
                0x48, 0xb9, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x48, 0x8b, 0x0c, 0xcf,
                // sar rax, cl
                0x48, 0xd3, 0xf8, 0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x48,
                0x89, 0x04, 0xcf, 0xc3,
            ]
        }
    };

    let size = code.len();
    Some(StencilTemplate {
        code,
        patches: vec![
            PatchLocation {
                offset: 2,
                kind: PatchKind::Src1Reg,
            },
            PatchLocation {
                offset: 16,
                kind: PatchKind::Src2Reg,
            },
            PatchLocation {
                offset: 33,
                kind: PatchKind::DstReg,
            },
        ],
        size,
    })
}

/// ALU immediate stencils
fn get_alui_stencil(mode: u8) -> Option<StencilTemplate> {
    let _op = AluOp::from_u8(mode)?;

    // Structure:
    // movabs rax, src1_idx
    // mov rax, [rdi + rax*8]
    // mov ecx, imm32
    // cdqe (sign extend)
    // add rax, rcx (or other op)
    // movabs rcx, dst_idx
    // mov [rdi + rcx*8], rax
    // ret

    let code = vec![
        0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x48, 0x8b, 0x04, 0xc7,
        // mov ecx, imm32
        0xb9, 0xef, 0xbe, 0xad, 0xde, // movsxd rcx, ecx
        0x48, 0x63, 0xc9, // add rax, rcx
        0x48, 0x01, 0xc8, 0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x48, 0x89,
        0x04, 0xcf, 0xc3,
    ];

    let size = code.len();
    Some(StencilTemplate {
        code,
        patches: vec![
            PatchLocation {
                offset: 2,
                kind: PatchKind::Src1Reg,
            },
            PatchLocation {
                offset: 15,
                kind: PatchKind::Imm32,
            },
            PatchLocation {
                offset: 27,
                kind: PatchKind::DstReg,
            },
        ],
        size,
    })
}

/// Multiply/divide stencils
fn get_muldiv_stencil(mode: u8) -> Option<StencilTemplate> {
    let op = MulDivOp::from_u8(mode)?;

    let code = match op {
        MulDivOp::Mul => {
            vec![
                0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x48, 0x8b, 0x04, 0xc7,
                0x48, 0xb9, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x48, 0x8b, 0x0c, 0xcf,
                // imul rax, rcx
                0x48, 0x0f, 0xaf, 0xc1, 0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
                0x48, 0x89, 0x04, 0xcf, 0xc3,
            ]
        }
        MulDivOp::Div | MulDivOp::Mod => {
            vec![
                0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x48, 0x8b, 0x04, 0xc7,
                0x48, 0xb9, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x48, 0x8b, 0x0c, 0xcf,
                // xor rdx, rdx (clear high bits for div)
                0x48, 0x31, 0xd2, // div rcx
                0x48, 0xf7, 0xf1, // For mod, result is in rdx, for div in rax
                0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x48, 0x89, 0x04, 0xcf,
                0xc3,
            ]
        }
        MulDivOp::MulH => {
            // For high bits of multiplication, use imul with rdx:rax result
            vec![
                0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x48, 0x8b, 0x04, 0xc7,
                0x48, 0xb9, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x48, 0x8b, 0x0c, 0xcf,
                // imul rcx (rdx:rax = rax * rcx)
                0x48, 0xf7, 0xe9, // mov rax, rdx (take high bits)
                0x48, 0x89, 0xd0, 0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x48,
                0x89, 0x04, 0xcf, 0xc3,
            ]
        }
    };

    let size = code.len();
    Some(StencilTemplate {
        code,
        patches: vec![
            PatchLocation {
                offset: 2,
                kind: PatchKind::Src1Reg,
            },
            PatchLocation {
                offset: 16,
                kind: PatchKind::Src2Reg,
            },
            PatchLocation {
                offset: if mode == 1 { 36 } else { 34 },
                kind: PatchKind::DstReg,
            },
        ],
        size,
    })
}

/// Load stencils for different widths
fn get_load_stencil(mode: u8) -> Option<StencilTemplate> {
    let width = MemWidth::from_u8(mode)?;

    // Structure:
    // movabs rax, base_idx
    // mov rax, [rdi + rax*8]  ; load base address
    // mov ecx, offset
    // movsxd rcx, ecx
    // <load from [rax + rcx]>
    // movabs rcx, dst_idx
    // mov [rdi + rcx*8], rax
    // ret

    let load_instr = match width {
        MemWidth::Byte => vec![0x48, 0x0f, 0xb6, 0x04, 0x08], // movzx rax, byte [rax+rcx]
        MemWidth::Half => vec![0x48, 0x0f, 0xb7, 0x04, 0x08], // movzx rax, word [rax+rcx]
        MemWidth::Word => vec![0x8b, 0x04, 0x08],             // mov eax, [rax+rcx]
        MemWidth::Double => vec![0x48, 0x8b, 0x04, 0x08],     // mov rax, [rax+rcx]
    };

    let mut code = vec![
        0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x48, 0x8b, 0x04, 0xc7, 0xb9,
        0xef, 0xbe, 0xad, 0xde, 0x48, 0x63, 0xc9,
    ];
    code.extend_from_slice(&load_instr);

    let dst_offset = code.len() + 2;
    code.extend_from_slice(&[
        0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x48, 0x89, 0x04, 0xcf, 0xc3,
    ]);

    let size = code.len();
    Some(StencilTemplate {
        code,
        patches: vec![
            PatchLocation {
                offset: 2,
                kind: PatchKind::Src1Reg,
            },
            PatchLocation {
                offset: 15,
                kind: PatchKind::Imm32,
            },
            PatchLocation {
                offset: dst_offset,
                kind: PatchKind::DstReg,
            },
        ],
        size,
    })
}

/// Store stencils for different widths
fn get_store_stencil(mode: u8) -> Option<StencilTemplate> {
    let width = MemWidth::from_u8(mode)?;

    // Structure:
    // movabs rax, base_idx
    // mov rax, [rdi + rax*8]  ; load base address
    // mov ecx, offset
    // movsxd rcx, ecx
    // add rax, rcx            ; calculate target address
    // movabs rcx, src_idx
    // mov rcx, [rdi + rcx*8]  ; load value to store
    // <store rcx to [rax]>
    // ret

    let store_instr = match width {
        MemWidth::Byte => vec![0x88, 0x08],         // mov [rax], cl
        MemWidth::Half => vec![0x66, 0x89, 0x08],   // mov [rax], cx
        MemWidth::Word => vec![0x89, 0x08],         // mov [rax], ecx
        MemWidth::Double => vec![0x48, 0x89, 0x08], // mov [rax], rcx
    };

    let mut code = vec![
        0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x48, 0x8b, 0x04, 0xc7, 0xb9,
        0xef, 0xbe, 0xad, 0xde, 0x48, 0x63, 0xc9, 0x48, 0x01, 0xc8, // add rax, rcx
        0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x48, 0x8b, 0x0c, 0xcf,
    ];
    code.extend_from_slice(&store_instr);
    code.push(0xc3); // ret

    let size = code.len();
    Some(StencilTemplate {
        code,
        patches: vec![
            PatchLocation {
                offset: 2,
                kind: PatchKind::Src1Reg,
            },
            PatchLocation {
                offset: 15,
                kind: PatchKind::Imm32,
            },
            PatchLocation {
                offset: 28,
                kind: PatchKind::DstReg,
            },
        ],
        size,
    })
}

/// MOV stencils (register or immediate)
fn get_mov_stencil(mode: u8) -> Option<StencilTemplate> {
    if mode == 0 {
        // Register-to-register move
        let code = vec![
            0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x48, 0x8b, 0x04, 0xc7,
            0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x48, 0x89, 0x04, 0xcf,
            0xc3,
        ];
        Some(StencilTemplate {
            size: code.len(),
            code,
            patches: vec![
                PatchLocation {
                    offset: 2,
                    kind: PatchKind::Src1Reg,
                },
                PatchLocation {
                    offset: 16,
                    kind: PatchKind::DstReg,
                },
            ],
        })
    } else {
        // Load immediate
        let code = vec![
            0xb8, 0xef, 0xbe, 0xad, 0xde, // mov eax, imm32
            0x48, 0x98, // cdqe (sign extend)
            0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x48, 0x89, 0x04, 0xcf,
            0xc3,
        ];
        Some(StencilTemplate {
            size: code.len(),
            code,
            patches: vec![
                PatchLocation {
                    offset: 1,
                    kind: PatchKind::Imm32,
                },
                PatchLocation {
                    offset: 9,
                    kind: PatchKind::DstReg,
                },
            ],
        })
    }
}

/// NOP stencil
fn nop_stencil() -> StencilTemplate {
    StencilTemplate {
        code: vec![0xc3], // Just ret
        patches: vec![],
        size: 1,
    }
}

/// HALT stencil
fn halt_stencil() -> StencilTemplate {
    StencilTemplate {
        code: vec![
            0x48, 0xb8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xc3,
        ],
        patches: vec![],
        size: 11,
    }
}

/// RET stencil
fn ret_stencil() -> StencilTemplate {
    StencilTemplate {
        code: vec![0xc3],
        patches: vec![],
        size: 1,
    }
}

/// FENCE stencil (memory barrier)
fn fence_stencil() -> StencilTemplate {
    StencilTemplate {
        code: vec![
            0x0f, 0xae, 0xf0, // mfence
            0xc3, // ret
        ],
        patches: vec![],
        size: 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alu_stencil() {
        let stencil = get_alu_stencil(AluOp::Add as u8).unwrap();
        assert!(!stencil.code.is_empty());
        assert_eq!(stencil.patches.len(), 3);
    }

    #[test]
    fn test_mov_stencil() {
        let reg_mov = get_mov_stencil(0).unwrap();
        assert!(reg_mov.patches.iter().any(|p| p.kind == PatchKind::Src1Reg));

        let imm_mov = get_mov_stencil(1).unwrap();
        assert!(imm_mov.patches.iter().any(|p| p.kind == PatchKind::Imm32));
    }

    #[test]
    fn test_load_stencil() {
        for width in [
            MemWidth::Byte,
            MemWidth::Half,
            MemWidth::Word,
            MemWidth::Double,
        ] {
            let stencil = get_load_stencil(width as u8).unwrap();
            assert!(!stencil.code.is_empty());
        }
    }
}
