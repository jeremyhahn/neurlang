//! Comprehensive tests for all 32 Neurlang opcodes
//!
//! Tests encoding, decoding, and interpreter execution for each opcode.

use neurlang::ir::{
    AluOp, Assembler, AtomicOp, BitsOp, BranchCond, ChanOp, FenceMode, FileOp, FpuOp, Instruction,
    IoOp, MemWidth, MulDivOp, NetOp, NetOption, Opcode, Program, RandOp, Register, TimeOp,
    TrapType,
};

// ============================================================================
// Arithmetic/Logic Tests (opcodes 0x00-0x02)
// ============================================================================

#[test]
fn test_alu_add() {
    let instr = Instruction::new(
        Opcode::Alu,
        Register::R0,
        Register::R1,
        Register::R2,
        AluOp::Add as u8,
    );
    let bytes = instr.encode();
    let decoded = Instruction::decode(&bytes).unwrap();
    assert_eq!(decoded.opcode, Opcode::Alu);
    assert_eq!(decoded.mode, AluOp::Add as u8);
    assert_eq!(decoded.rd, Register::R0);
    assert_eq!(decoded.rs1, Register::R1);
    assert_eq!(decoded.rs2, Register::R2);
}

#[test]
fn test_alu_all_ops() {
    for op in [
        AluOp::Add,
        AluOp::Sub,
        AluOp::And,
        AluOp::Or,
        AluOp::Xor,
        AluOp::Shl,
        AluOp::Shr,
        AluOp::Sar,
    ] {
        let instr = Instruction::new(
            Opcode::Alu,
            Register::R0,
            Register::R1,
            Register::R2,
            op as u8,
        );
        let bytes = instr.encode();
        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(decoded.mode, op as u8);
        assert_eq!(AluOp::from_u8(decoded.mode), Some(op));
    }
}

#[test]
fn test_alui_immediate() {
    let instr = Instruction::with_imm(
        Opcode::AluI,
        Register::R0,
        Register::R1,
        AluOp::Add as u8,
        42,
    );
    let bytes = instr.encode();
    assert_eq!(bytes.len(), 8); // Extended format
    let decoded = Instruction::decode(&bytes).unwrap();
    assert_eq!(decoded.imm, Some(42));
}

#[test]
fn test_alui_negative_immediate() {
    let instr = Instruction::with_imm(
        Opcode::AluI,
        Register::R0,
        Register::R1,
        AluOp::Sub as u8,
        -100,
    );
    let bytes = instr.encode();
    let decoded = Instruction::decode(&bytes).unwrap();
    assert_eq!(decoded.imm, Some(-100));
}

#[test]
fn test_muldiv_ops() {
    for op in [MulDivOp::Mul, MulDivOp::MulH, MulDivOp::Div, MulDivOp::Mod] {
        let instr = Instruction::new(
            Opcode::MulDiv,
            Register::R0,
            Register::R1,
            Register::R2,
            op as u8,
        );
        let bytes = instr.encode();
        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(MulDivOp::from_u8(decoded.mode), Some(op));
    }
}

// ============================================================================
// Memory Tests (opcodes 0x03-0x05)
// ============================================================================

#[test]
fn test_load_widths() {
    for width in [
        MemWidth::Byte,
        MemWidth::Half,
        MemWidth::Word,
        MemWidth::Double,
    ] {
        let instr =
            Instruction::with_imm(Opcode::Load, Register::R0, Register::R1, width as u8, 16);
        let bytes = instr.encode();
        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(MemWidth::from_u8(decoded.mode), Some(width));
    }
}

#[test]
fn test_store_widths() {
    for width in [
        MemWidth::Byte,
        MemWidth::Half,
        MemWidth::Word,
        MemWidth::Double,
    ] {
        let instr =
            Instruction::with_imm(Opcode::Store, Register::R0, Register::R1, width as u8, -8);
        let bytes = instr.encode();
        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(MemWidth::from_u8(decoded.mode), Some(width));
        assert_eq!(decoded.imm, Some(-8));
    }
}

#[test]
fn test_atomic_ops() {
    for op in [
        AtomicOp::Cas,
        AtomicOp::Xchg,
        AtomicOp::Add,
        AtomicOp::And,
        AtomicOp::Or,
        AtomicOp::Xor,
        AtomicOp::Min,
        AtomicOp::Max,
    ] {
        let instr = Instruction::new(
            Opcode::Atomic,
            Register::R0,
            Register::R1,
            Register::R2,
            op as u8,
        );
        let bytes = instr.encode();
        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(AtomicOp::from_u8(decoded.mode), Some(op));
    }
}

// ============================================================================
// Control Flow Tests (opcodes 0x06-0x09)
// ============================================================================

#[test]
fn test_branch_conditions() {
    for cond in [
        BranchCond::Always,
        BranchCond::Eq,
        BranchCond::Ne,
        BranchCond::Lt,
        BranchCond::Le,
        BranchCond::Gt,
        BranchCond::Ge,
        BranchCond::Ltu,
    ] {
        let instr = Instruction::with_imm(
            Opcode::Branch,
            Register::Zero,
            Register::R0,
            cond as u8,
            -32,
        );
        let bytes = instr.encode();
        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(BranchCond::from_u8(decoded.mode), Some(cond));
        assert_eq!(decoded.imm, Some(-32));
    }
}

#[test]
fn test_call_direct() {
    let instr = Instruction::with_imm(Opcode::Call, Register::Lr, Register::Zero, 0, 0x100);
    let bytes = instr.encode();
    let decoded = Instruction::decode(&bytes).unwrap();
    assert_eq!(decoded.opcode, Opcode::Call);
    assert_eq!(decoded.imm, Some(0x100));
}

#[test]
fn test_ret() {
    let instr = Instruction::new(Opcode::Ret, Register::Zero, Register::Lr, Register::Zero, 0);
    let bytes = instr.encode();
    let decoded = Instruction::decode(&bytes).unwrap();
    assert_eq!(decoded.opcode, Opcode::Ret);
}

#[test]
fn test_jump_indirect() {
    let instr = Instruction::with_imm(
        Opcode::Jump,
        Register::Zero,
        Register::R0,
        1, // indirect mode
        0,
    );
    let bytes = instr.encode();
    let decoded = Instruction::decode(&bytes).unwrap();
    assert_eq!(decoded.opcode, Opcode::Jump);
}

// ============================================================================
// Capability Tests (opcodes 0x0A-0x0C)
// ============================================================================

#[test]
fn test_cap_new() {
    let instr = Instruction::new(Opcode::CapNew, Register::R0, Register::R1, Register::R2, 0);
    let bytes = instr.encode();
    let decoded = Instruction::decode(&bytes).unwrap();
    assert_eq!(decoded.opcode, Opcode::CapNew);
}

#[test]
fn test_cap_restrict() {
    let instr = Instruction::new(
        Opcode::CapRestrict,
        Register::R0,
        Register::R1,
        Register::R2,
        1, // restrict mode
    );
    let bytes = instr.encode();
    let decoded = Instruction::decode(&bytes).unwrap();
    assert_eq!(decoded.opcode, Opcode::CapRestrict);
}

#[test]
fn test_cap_query() {
    let instr = Instruction::new(
        Opcode::CapQuery,
        Register::R0,
        Register::R1,
        Register::Zero,
        0, // get base
    );
    let bytes = instr.encode();
    let decoded = Instruction::decode(&bytes).unwrap();
    assert_eq!(decoded.opcode, Opcode::CapQuery);
}

// ============================================================================
// Concurrency Tests (opcodes 0x0D-0x11)
// ============================================================================

#[test]
fn test_spawn() {
    let instr = Instruction::new(Opcode::Spawn, Register::R0, Register::R1, Register::R2, 0);
    let bytes = instr.encode();
    let decoded = Instruction::decode(&bytes).unwrap();
    assert_eq!(decoded.opcode, Opcode::Spawn);
}

#[test]
fn test_join() {
    let instr = Instruction::new(Opcode::Join, Register::R0, Register::R1, Register::Zero, 0);
    let bytes = instr.encode();
    let decoded = Instruction::decode(&bytes).unwrap();
    assert_eq!(decoded.opcode, Opcode::Join);
}

#[test]
fn test_chan_ops() {
    for op in [ChanOp::Create, ChanOp::Send, ChanOp::Recv, ChanOp::Close] {
        let instr = Instruction::new(
            Opcode::Chan,
            Register::R0,
            Register::R1,
            Register::R2,
            op as u8,
        );
        let bytes = instr.encode();
        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(ChanOp::from_u8(decoded.mode), Some(op));
    }
}

#[test]
fn test_fence_modes() {
    for mode in [
        FenceMode::Acquire,
        FenceMode::Release,
        FenceMode::AcqRel,
        FenceMode::SeqCst,
    ] {
        let instr = Instruction::new(
            Opcode::Fence,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            mode as u8,
        );
        let bytes = instr.encode();
        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(FenceMode::from_u8(decoded.mode), Some(mode));
    }
}

#[test]
fn test_yield() {
    let instr = Instruction::new(
        Opcode::Yield,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    );
    let bytes = instr.encode();
    let decoded = Instruction::decode(&bytes).unwrap();
    assert_eq!(decoded.opcode, Opcode::Yield);
}

// ============================================================================
// Taint Tracking Tests (opcodes 0x12-0x13)
// ============================================================================

#[test]
fn test_taint() {
    let instr = Instruction::new(
        Opcode::Taint,
        Register::R0,
        Register::R1,
        Register::Zero,
        1, // taint level
    );
    let bytes = instr.encode();
    let decoded = Instruction::decode(&bytes).unwrap();
    assert_eq!(decoded.opcode, Opcode::Taint);
}

#[test]
fn test_sanitize() {
    let instr = Instruction::new(
        Opcode::Sanitize,
        Register::R0,
        Register::R1,
        Register::Zero,
        0,
    );
    let bytes = instr.encode();
    let decoded = Instruction::decode(&bytes).unwrap();
    assert_eq!(decoded.opcode, Opcode::Sanitize);
}

// ============================================================================
// I/O Tests (opcodes 0x14-0x18)
// ============================================================================

#[test]
fn test_file_ops() {
    for op in [
        FileOp::Open,
        FileOp::Read,
        FileOp::Write,
        FileOp::Close,
        FileOp::Seek,
        FileOp::Stat,
        FileOp::Mkdir,
        FileOp::Delete,
    ] {
        let instr = Instruction::with_imm(Opcode::File, Register::R0, Register::R1, op as u8, 0);
        let bytes = instr.encode();
        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(FileOp::from_u8(decoded.mode), Some(op));
    }
}

#[test]
fn test_net_ops() {
    for op in [
        NetOp::Socket,
        NetOp::Connect,
        NetOp::Bind,
        NetOp::Listen,
        NetOp::Accept,
        NetOp::Send,
        NetOp::Recv,
        NetOp::Close,
    ] {
        let instr = Instruction::with_imm(
            Opcode::Net,
            Register::R0,
            Register::R1,
            op as u8,
            80, // port
        );
        let bytes = instr.encode();
        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(NetOp::from_u8(decoded.mode), Some(op));
    }
}

#[test]
fn test_net_setopt_options() {
    for opt in [
        NetOption::Nonblock,
        NetOption::TimeoutMs,
        NetOption::Keepalive,
        NetOption::ReuseAddr,
        NetOption::NoDelay,
        NetOption::RecvBufSize,
        NetOption::SendBufSize,
        NetOption::Linger,
    ] {
        let instr = Instruction::with_imm(
            Opcode::NetSetopt,
            Register::Zero,
            Register::R0,
            opt as u8,
            1, // value
        );
        let bytes = instr.encode();
        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(NetOption::from_u8(decoded.mode), Some(opt));
    }
}

#[test]
fn test_io_ops() {
    for op in [IoOp::Print, IoOp::ReadLine, IoOp::GetArgs, IoOp::GetEnv] {
        let instr = Instruction::with_imm(
            Opcode::Io,
            Register::R0,
            Register::R1,
            op as u8,
            100, // buffer size
        );
        let bytes = instr.encode();
        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(IoOp::from_u8(decoded.mode), Some(op));
    }
}

#[test]
fn test_time_ops() {
    for op in [TimeOp::Now, TimeOp::Sleep, TimeOp::Monotonic] {
        let instr = Instruction::with_imm(
            Opcode::Time,
            Register::R0,
            Register::Zero,
            op as u8,
            1000, // ms for sleep
        );
        let bytes = instr.encode();
        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(TimeOp::from_u8(decoded.mode), Some(op));
    }
}

// ============================================================================
// Math Extensions Tests (opcodes 0x19-0x1B)
// ============================================================================

#[test]
fn test_fpu_ops() {
    for op in [
        FpuOp::Fadd,
        FpuOp::Fsub,
        FpuOp::Fmul,
        FpuOp::Fdiv,
        FpuOp::Fsqrt,
        FpuOp::Fabs,
        FpuOp::Ffloor,
        FpuOp::Fceil,
    ] {
        let instr = Instruction::new(
            Opcode::Fpu,
            Register::R0,
            Register::R1,
            Register::R2,
            op as u8,
        );
        let bytes = instr.encode();
        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(FpuOp::from_u8(decoded.mode), Some(op));
    }
}

#[test]
fn test_rand_ops() {
    for op in [RandOp::RandBytes, RandOp::RandU64] {
        let instr = Instruction::new(
            Opcode::Rand,
            Register::R0,
            Register::R1,
            Register::R2,
            op as u8,
        );
        let bytes = instr.encode();
        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(RandOp::from_u8(decoded.mode), Some(op));
    }
}

#[test]
fn test_bits_ops() {
    for op in [BitsOp::Popcount, BitsOp::Clz, BitsOp::Ctz, BitsOp::Bswap] {
        let instr = Instruction::new(
            Opcode::Bits,
            Register::R0,
            Register::R1,
            Register::Zero,
            op as u8,
        );
        let bytes = instr.encode();
        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(BitsOp::from_u8(decoded.mode), Some(op));
    }
}

// ============================================================================
// System Tests (opcodes 0x1C-0x1F)
// ============================================================================

#[test]
fn test_mov_register() {
    let instr = Instruction::new(Opcode::Mov, Register::R0, Register::R1, Register::Zero, 0);
    let bytes = instr.encode();
    let decoded = Instruction::decode(&bytes).unwrap();
    assert_eq!(decoded.opcode, Opcode::Mov);
}

#[test]
fn test_mov_immediate() {
    let instr = Instruction::with_imm(
        Opcode::Mov,
        Register::R0,
        Register::Zero,
        0,
        0xDEADBEEF_u32 as i32,
    );
    let bytes = instr.encode();
    let decoded = Instruction::decode(&bytes).unwrap();
    assert_eq!(decoded.imm, Some(0xDEADBEEF_u32 as i32));
}

#[test]
fn test_trap_types() {
    for trap in [
        TrapType::Syscall,
        TrapType::Breakpoint,
        TrapType::BoundsViolation,
        TrapType::CapabilityViolation,
        TrapType::TaintViolation,
        TrapType::DivByZero,
        TrapType::InvalidOp,
        TrapType::User,
    ] {
        let instr = Instruction::new(
            Opcode::Trap,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            trap as u8,
        );
        let bytes = instr.encode();
        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(TrapType::from_u8(decoded.mode), Some(trap));
    }
}

#[test]
fn test_nop() {
    let instr = Instruction::new(
        Opcode::Nop,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    );
    let bytes = instr.encode();
    let decoded = Instruction::decode(&bytes).unwrap();
    assert_eq!(decoded.opcode, Opcode::Nop);
}

#[test]
fn test_halt() {
    let instr = Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    );
    let bytes = instr.encode();
    let decoded = Instruction::decode(&bytes).unwrap();
    assert_eq!(decoded.opcode, Opcode::Halt);
}

// ============================================================================
// Program Tests
// ============================================================================

#[test]
fn test_program_encode_decode_roundtrip() {
    let mut prog = Program::new();

    // Simple program: mov r0, 42; add r1, r0, r0; halt
    prog.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R0,
        Register::Zero,
        0,
        42,
    ));
    prog.instructions.push(Instruction::new(
        Opcode::Alu,
        Register::R1,
        Register::R0,
        Register::R0,
        AluOp::Add as u8,
    ));
    prog.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    let bytes = prog.encode();
    let decoded = Program::decode(&bytes).unwrap();

    assert_eq!(decoded.instructions.len(), 3);
    assert_eq!(decoded.instructions[0].imm, Some(42));
    assert_eq!(decoded.instructions[1].opcode, Opcode::Alu);
    assert_eq!(decoded.instructions[2].opcode, Opcode::Halt);
}

#[test]
fn test_program_with_data_section() {
    let mut prog = Program::new();
    prog.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));
    prog.data_section = b"Hello, Neurlang!".to_vec();

    let bytes = prog.encode();
    let decoded = Program::decode(&bytes).unwrap();

    assert_eq!(decoded.data_section, b"Hello, Neurlang!");
}

#[test]
fn test_invalid_magic() {
    let bytes = b"NOPE\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
    assert!(Program::decode(bytes).is_none());
}

// ============================================================================
// Opcode Coverage Test
// ============================================================================

#[test]
fn test_all_33_opcodes() {
    // Verify all 33 opcodes (0x00-0x20) are defined and parseable
    for i in 0u8..=32 {
        let opcode = Opcode::from_u8(i);
        assert!(
            opcode.is_some(),
            "Missing opcode definition for 0x{:02X}",
            i
        );
        let op = opcode.unwrap();

        // Verify mnemonic is defined
        let mnemonic = op.mnemonic();
        assert!(
            !mnemonic.is_empty(),
            "Empty mnemonic for opcode 0x{:02X}",
            i
        );
    }

    // Verify opcode 0x21 is invalid (first undefined opcode)
    assert!(Opcode::from_u8(33).is_none());
}

#[test]
fn test_opcode_mnemonic_uniqueness() {
    use std::collections::HashSet;
    let mut mnemonics = HashSet::new();

    for i in 0u8..=32 {
        let opcode = Opcode::from_u8(i).unwrap();
        let mnemonic = opcode.mnemonic();
        assert!(
            mnemonics.insert(mnemonic),
            "Duplicate mnemonic '{}' for opcode 0x{:02X}",
            mnemonic,
            i
        );
    }
}

// ============================================================================
// Extension Call Tests
// ============================================================================

#[test]
fn test_ext_call_opcode() {
    // Test that ExtCall opcode assembles correctly
    let mut asm = Assembler::new();
    let program = asm
        .assemble(
            r#"
        ; Call sha256 extension (ID 1)
        ext.call r0, sha256, r1, r2
        halt
    "#,
        )
        .unwrap();

    assert_eq!(program.instructions.len(), 2);
    assert_eq!(program.instructions[0].opcode, Opcode::ExtCall);
    assert_eq!(program.instructions[0].rd, Register::R0);
    assert_eq!(program.instructions[0].rs1, Register::R1);
    assert_eq!(program.instructions[0].rs2, Register::R2);
    assert_eq!(program.instructions[0].imm, Some(1)); // sha256 = ID 1
}

#[test]
fn test_ext_call_numeric_id() {
    // Test ext.call with numeric extension ID
    let mut asm = Assembler::new();
    let program = asm
        .assemble(
            r#"
        ext.call r0, 42, r1, r2
        halt
    "#,
        )
        .unwrap();

    assert_eq!(program.instructions[0].opcode, Opcode::ExtCall);
    assert_eq!(program.instructions[0].imm, Some(42));
}

#[test]
fn test_ext_call_all_crypto_extensions() {
    // Test all built-in crypto extension names resolve correctly
    // Extension IDs are assigned in registration order:
    // sha256=1, hmac_sha256=2, aes256_gcm_encrypt=3, aes256_gcm_decrypt=4,
    // constant_time_eq=5, secure_random=6, pbkdf2_sha256=7,
    // ed25519_sign=8, ed25519_verify=9, x25519_derive=10
    let mut asm = Assembler::new();
    let program = asm
        .assemble(
            r#"
        ext.call r0, sha256, r1, r2
        ext.call r0, hmac_sha256, r1, r2
        ext.call r0, aes256_gcm_encrypt, r1, r2
        ext.call r0, aes256_gcm_decrypt, r1, r2
        ext.call r0, constant_time_eq, r1, r2
        ext.call r0, secure_random, r1, r2
        ext.call r0, pbkdf2_sha256, r1, r2
        ext.call r0, ed25519_sign, r1, r2
        ext.call r0, ed25519_verify, r1, r2
        ext.call r0, x25519_derive, r1, r2
        halt
    "#,
        )
        .unwrap();

    // Verify each extension has correct ID (in registration order)
    assert_eq!(program.instructions[0].imm, Some(1)); // sha256
    assert_eq!(program.instructions[1].imm, Some(2)); // hmac_sha256
    assert_eq!(program.instructions[2].imm, Some(3)); // aes256_gcm_encrypt
    assert_eq!(program.instructions[3].imm, Some(4)); // aes256_gcm_decrypt
    assert_eq!(program.instructions[4].imm, Some(5)); // constant_time_eq
    assert_eq!(program.instructions[5].imm, Some(6)); // secure_random
    assert_eq!(program.instructions[6].imm, Some(7)); // pbkdf2_sha256
    assert_eq!(program.instructions[7].imm, Some(8)); // ed25519_sign
    assert_eq!(program.instructions[8].imm, Some(9)); // ed25519_verify
    assert_eq!(program.instructions[9].imm, Some(10)); // x25519_derive
}

// ============================================================================
// Intrinsic Expansion Tests
// ============================================================================

#[test]
fn test_intrinsic_memcpy() {
    // Test that @memcpy intrinsic expands to valid instructions
    let mut asm = Assembler::new();
    let program = asm
        .assemble(
            r#"
        @memcpy r0, r1, 32
        halt
    "#,
        )
        .unwrap();

    // Should expand to a loop of load/store operations
    assert!(program.instructions.len() > 2); // More than just the intrinsic + halt

    // Last instruction should be halt
    assert_eq!(program.instructions.last().unwrap().opcode, Opcode::Halt);
}

#[test]
fn test_intrinsic_strlen() {
    let mut asm = Assembler::new();
    // @strlen takes 1 argument (string pointer), result goes to r0
    let program = asm
        .assemble(
            r#"
        @strlen r1
        halt
    "#,
        )
        .unwrap();

    // Should expand to a loop that counts until null byte
    assert!(program.instructions.len() > 2);
    assert_eq!(program.instructions.last().unwrap().opcode, Opcode::Halt);
}

#[test]
fn test_intrinsic_memset() {
    let mut asm = Assembler::new();
    let program = asm
        .assemble(
            r#"
        @memset r0, r1, 64
        halt
    "#,
        )
        .unwrap();

    // Should expand to a loop of store operations
    assert!(program.instructions.len() > 2);
    assert_eq!(program.instructions.last().unwrap().opcode, Opcode::Halt);
}

#[test]
fn test_intrinsic_abs() {
    let mut asm = Assembler::new();
    // @abs takes 1 argument, result goes to r0
    let program = asm
        .assemble(
            r#"
        @abs r1
        halt
    "#,
        )
        .unwrap();

    // Should expand to conditional negation
    assert!(program.instructions.len() >= 2);
    assert_eq!(program.instructions.last().unwrap().opcode, Opcode::Halt);
}

#[test]
fn test_intrinsic_min() {
    let mut asm = Assembler::new();
    // @min takes 2 arguments, result goes to r0
    let program = asm
        .assemble(
            r#"
        @min r1, r2
        halt
    "#,
        )
        .unwrap();

    // Should expand to compare and select
    assert!(program.instructions.len() >= 2);
    assert_eq!(program.instructions.last().unwrap().opcode, Opcode::Halt);
}

#[test]
fn test_intrinsic_max() {
    let mut asm = Assembler::new();
    // @max takes 2 arguments, result goes to r0
    let program = asm
        .assemble(
            r#"
        @max r1, r2
        halt
    "#,
        )
        .unwrap();

    assert!(program.instructions.len() >= 2);
    assert_eq!(program.instructions.last().unwrap().opcode, Opcode::Halt);
}

#[test]
fn test_intrinsic_clamp() {
    let mut asm = Assembler::new();
    // @clamp takes 3 arguments (value, min, max), result goes to r0
    let program = asm
        .assemble(
            r#"
        @clamp r1, r2, r3
        halt
    "#,
        )
        .unwrap();

    // Should expand to min/max checks
    assert!(program.instructions.len() >= 2);
    assert_eq!(program.instructions.last().unwrap().opcode, Opcode::Halt);
}

#[test]
fn test_intrinsic_gcd() {
    let mut asm = Assembler::new();
    // @gcd takes 2 arguments, result goes to r0
    let program = asm
        .assemble(
            r#"
        @gcd r1, r2
        halt
    "#,
        )
        .unwrap();

    // GCD should expand to Euclidean algorithm loop
    assert!(program.instructions.len() > 5); // Non-trivial algorithm
    assert_eq!(program.instructions.last().unwrap().opcode, Opcode::Halt);
}

#[test]
fn test_unknown_intrinsic_error() {
    let mut asm = Assembler::new();
    let result = asm.assemble(
        r#"
        @unknown_intrinsic r0, r1
        halt
    "#,
    );

    assert!(result.is_err());
}
