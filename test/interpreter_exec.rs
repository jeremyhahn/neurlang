//! Interpreter Execution Tests
//!
//! Tests that actually run programs through the interpreter and verify results.
//! These tests validate the correctness of opcode implementations.

use neurlang::interp::{InterpResult, Interpreter};
use neurlang::ir::{
    AluOp, BitsOp, FpuOp, Instruction, MulDivOp, Opcode, Program, RandOp, Register, TimeOp,
};

/// Helper to run a program and get the result in R0
fn run_program(program: &Program) -> u64 {
    let binary = program.encode();
    let decoded = Program::decode(&binary).expect("Failed to decode program");
    let mut interp = Interpreter::new(1024); // 1KB memory
    let result = interp.execute(&decoded);

    match result {
        InterpResult::Ok(_) | InterpResult::Halted => interp.registers[0],
        other => panic!("Execution failed: {:?}", other),
    }
}

// ============================================================================
// Basic Arithmetic Tests
// ============================================================================

#[test]
fn test_exec_mov_immediate() {
    let mut program = Program::new();
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R0,
        Register::Zero,
        0,
        42,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    assert_eq!(run_program(&program), 42);
}

#[test]
fn test_exec_add() {
    let mut program = Program::new();
    // r1 = 10
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R1,
        Register::Zero,
        0,
        10,
    ));
    // r2 = 20
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R2,
        Register::Zero,
        0,
        20,
    ));
    // r0 = r1 + r2
    program.instructions.push(Instruction::new(
        Opcode::Alu,
        Register::R0,
        Register::R1,
        Register::R2,
        AluOp::Add as u8,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    assert_eq!(run_program(&program), 30);
}

#[test]
fn test_exec_sub() {
    let mut program = Program::new();
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R1,
        Register::Zero,
        0,
        100,
    ));
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R2,
        Register::Zero,
        0,
        30,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Alu,
        Register::R0,
        Register::R1,
        Register::R2,
        AluOp::Sub as u8,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    assert_eq!(run_program(&program), 70);
}

#[test]
fn test_exec_mul() {
    let mut program = Program::new();
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R1,
        Register::Zero,
        0,
        6,
    ));
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R2,
        Register::Zero,
        0,
        7,
    ));
    program.instructions.push(Instruction::new(
        Opcode::MulDiv,
        Register::R0,
        Register::R1,
        Register::R2,
        MulDivOp::Mul as u8,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    assert_eq!(run_program(&program), 42);
}

#[test]
fn test_exec_div() {
    let mut program = Program::new();
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R1,
        Register::Zero,
        0,
        100,
    ));
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R2,
        Register::Zero,
        0,
        5,
    ));
    program.instructions.push(Instruction::new(
        Opcode::MulDiv,
        Register::R0,
        Register::R1,
        Register::R2,
        MulDivOp::Div as u8,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    assert_eq!(run_program(&program), 20);
}

#[test]
fn test_exec_mod() {
    let mut program = Program::new();
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R1,
        Register::Zero,
        0,
        17,
    ));
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R2,
        Register::Zero,
        0,
        5,
    ));
    program.instructions.push(Instruction::new(
        Opcode::MulDiv,
        Register::R0,
        Register::R1,
        Register::R2,
        MulDivOp::Mod as u8,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    assert_eq!(run_program(&program), 2);
}

// ============================================================================
// Bitwise Operation Tests
// ============================================================================

#[test]
fn test_exec_and() {
    let mut program = Program::new();
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R1,
        Register::Zero,
        0,
        0xFF,
    ));
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R2,
        Register::Zero,
        0,
        0x0F,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Alu,
        Register::R0,
        Register::R1,
        Register::R2,
        AluOp::And as u8,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    assert_eq!(run_program(&program), 0x0F);
}

#[test]
fn test_exec_or() {
    let mut program = Program::new();
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R1,
        Register::Zero,
        0,
        0xF0,
    ));
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R2,
        Register::Zero,
        0,
        0x0F,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Alu,
        Register::R0,
        Register::R1,
        Register::R2,
        AluOp::Or as u8,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    assert_eq!(run_program(&program), 0xFF);
}

#[test]
fn test_exec_xor() {
    let mut program = Program::new();
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R1,
        Register::Zero,
        0,
        0xFF,
    ));
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R2,
        Register::Zero,
        0,
        0xAA,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Alu,
        Register::R0,
        Register::R1,
        Register::R2,
        AluOp::Xor as u8,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    assert_eq!(run_program(&program), 0x55);
}

#[test]
fn test_exec_shl() {
    let mut program = Program::new();
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R0,
        Register::Zero,
        0,
        1,
    ));
    program.instructions.push(Instruction::with_imm(
        Opcode::AluI,
        Register::R0,
        Register::R0,
        AluOp::Shl as u8,
        4,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    assert_eq!(run_program(&program), 16);
}

#[test]
fn test_exec_shr() {
    let mut program = Program::new();
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R0,
        Register::Zero,
        0,
        256,
    ));
    program.instructions.push(Instruction::with_imm(
        Opcode::AluI,
        Register::R0,
        Register::R0,
        AluOp::Shr as u8,
        4,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    assert_eq!(run_program(&program), 16);
}

// ============================================================================
// Bit Manipulation Tests (BITS opcode)
// ============================================================================

#[test]
fn test_exec_popcount() {
    let mut program = Program::new();
    // 0xFF = 8 bits set
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R1,
        Register::Zero,
        0,
        0xFF,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Bits,
        Register::R0,
        Register::R1,
        Register::Zero,
        BitsOp::Popcount as u8,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    assert_eq!(run_program(&program), 8);
}

#[test]
fn test_exec_clz() {
    let mut program = Program::new();
    // 1 has 63 leading zeros in 64-bit
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R1,
        Register::Zero,
        0,
        1,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Bits,
        Register::R0,
        Register::R1,
        Register::Zero,
        BitsOp::Clz as u8,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    assert_eq!(run_program(&program), 63);
}

#[test]
fn test_exec_ctz() {
    let mut program = Program::new();
    // 8 = 0b1000 has 3 trailing zeros
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R1,
        Register::Zero,
        0,
        8,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Bits,
        Register::R0,
        Register::R1,
        Register::Zero,
        BitsOp::Ctz as u8,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    assert_eq!(run_program(&program), 3);
}

#[test]
fn test_exec_bswap() {
    let mut program = Program::new();
    // 0x12345678 -> 0x78563412
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R1,
        Register::Zero,
        0,
        0x12345678_u32 as i32,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Bits,
        Register::R0,
        Register::R1,
        Register::Zero,
        BitsOp::Bswap as u8,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    let result = run_program(&program);
    // The bswap operates on 64-bit, so we check the low 32 bits
    assert_eq!((result >> 32) as u32, 0x78563412);
}

// ============================================================================
// FPU Tests
// ============================================================================

#[test]
fn test_exec_fpu_add() {
    let mut program = Program::new();
    // Load 2.0 into r1
    let two = 2.0_f64.to_bits();
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R1,
        Register::Zero,
        0,
        (two >> 32) as i32,
    ));
    program.instructions.push(Instruction::with_imm(
        Opcode::AluI,
        Register::R1,
        Register::R1,
        AluOp::Shl as u8,
        32,
    ));
    program.instructions.push(Instruction::with_imm(
        Opcode::AluI,
        Register::R1,
        Register::R1,
        AluOp::Or as u8,
        (two & 0xFFFFFFFF) as i32,
    ));

    // Load 3.0 into r2
    let three = 3.0_f64.to_bits();
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R2,
        Register::Zero,
        0,
        (three >> 32) as i32,
    ));
    program.instructions.push(Instruction::with_imm(
        Opcode::AluI,
        Register::R2,
        Register::R2,
        AluOp::Shl as u8,
        32,
    ));
    program.instructions.push(Instruction::with_imm(
        Opcode::AluI,
        Register::R2,
        Register::R2,
        AluOp::Or as u8,
        (three & 0xFFFFFFFF) as i32,
    ));

    // r0 = r1 + r2
    program.instructions.push(Instruction::new(
        Opcode::Fpu,
        Register::R0,
        Register::R1,
        Register::R2,
        FpuOp::Fadd as u8,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    let result = run_program(&program);
    let result_f64 = f64::from_bits(result);
    assert!(
        (result_f64 - 5.0).abs() < 0.001,
        "Expected 5.0, got {}",
        result_f64
    );
}

// ============================================================================
// Time Tests
// ============================================================================

#[test]
fn test_exec_time_now() {
    let mut program = Program::new();
    program.instructions.push(Instruction::new(
        Opcode::Time,
        Register::R0,
        Register::Zero,
        Register::Zero,
        TimeOp::Now as u8,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    let result = run_program(&program);
    // Should be a Unix timestamp > 2024-01-01 (1704067200)
    assert!(result > 1704067200, "Timestamp too old: {}", result);
    // And < 2100-01-01 (4102444800)
    assert!(
        result < 4102444800,
        "Timestamp too far in future: {}",
        result
    );
}

#[test]
fn test_exec_time_monotonic() {
    let mut program = Program::new();
    program.instructions.push(Instruction::new(
        Opcode::Time,
        Register::R0,
        Register::Zero,
        Register::Zero,
        TimeOp::Monotonic as u8,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    let result = run_program(&program);
    // Should return some non-zero value (nanoseconds)
    assert!(result > 0, "Monotonic time should be non-zero");
}

// ============================================================================
// Random Tests
// ============================================================================

#[test]
fn test_exec_rand_u64() {
    let mut program = Program::new();
    program.instructions.push(Instruction::new(
        Opcode::Rand,
        Register::R0,
        Register::Zero,
        Register::Zero,
        RandOp::RandU64 as u8,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    // Run twice and verify we get different values (very high probability)
    let result1 = run_program(&program);
    let result2 = run_program(&program);

    // It's astronomically unlikely to get the same random value twice
    assert_ne!(result1, result2, "Random values should differ");
}

// ============================================================================
// Loop Tests
// ============================================================================

#[test]
fn test_exec_factorial() {
    let mut program = Program::new();

    // Compute 5! = 120
    // r0 = n = 5
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R0,
        Register::Zero,
        0,
        5,
    ));
    // r1 = result = 1
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R1,
        Register::Zero,
        0,
        1,
    ));

    // Loop start (instruction 2):
    // r1 = r1 * r0
    program.instructions.push(Instruction::new(
        Opcode::MulDiv,
        Register::R1,
        Register::R1,
        Register::R0,
        MulDivOp::Mul as u8,
    ));
    // r0 = r0 - 1
    program.instructions.push(Instruction::with_imm(
        Opcode::AluI,
        Register::R0,
        Register::R0,
        AluOp::Sub as u8,
        1,
    ));
    // if r0 > 0, branch back to loop start
    // Instruction 4: branch, wants to go to instruction 2 (mul)
    // offset = 2 - 4 = -2
    let mut branch_instr = Instruction::with_imm(
        Opcode::Branch,
        Register::Zero,
        Register::R0,
        5,  // bgt (greater than)
        -2, // offset to instruction 2 (mul)
    );
    branch_instr.rs2 = Register::Zero; // compare r0 > 0
    program.instructions.push(branch_instr);

    // r0 = r1 (result)
    program.instructions.push(Instruction::new(
        Opcode::Mov,
        Register::R0,
        Register::R1,
        Register::Zero,
        0,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    assert_eq!(run_program(&program), 120);
}

#[test]
fn test_exec_fibonacci() {
    let mut program = Program::new();

    // Compute fib(10) = 55
    // r0 = n = 10
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R0,
        Register::Zero,
        0,
        10,
    ));
    // r1 = fib(i-2) = 0
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R1,
        Register::Zero,
        0,
        0,
    ));
    // r2 = fib(i-1) = 1
    program.instructions.push(Instruction::with_imm(
        Opcode::Mov,
        Register::R2,
        Register::Zero,
        0,
        1,
    ));

    // Decrement n before loop (need n-1 iterations to get fib(n))
    // This is instruction 3
    program.instructions.push(Instruction::with_imm(
        Opcode::AluI,
        Register::R0,
        Register::R0,
        AluOp::Sub as u8,
        1,
    ));

    // Loop start (instruction 4):
    // r3 = r1 + r2 (new fibonacci value)
    program.instructions.push(Instruction::new(
        Opcode::Alu,
        Register::R3,
        Register::R1,
        Register::R2,
        AluOp::Add as u8,
    ));
    // r1 = r2
    program.instructions.push(Instruction::new(
        Opcode::Mov,
        Register::R1,
        Register::R2,
        Register::Zero,
        0,
    ));
    // r2 = r3
    program.instructions.push(Instruction::new(
        Opcode::Mov,
        Register::R2,
        Register::R3,
        Register::Zero,
        0,
    ));
    // r0 = r0 - 1
    program.instructions.push(Instruction::with_imm(
        Opcode::AluI,
        Register::R0,
        Register::R0,
        AluOp::Sub as u8,
        1,
    ));
    // if r0 > 0, branch back to loop start
    let mut branch_instr = Instruction::with_imm(
        Opcode::Branch,
        Register::Zero,
        Register::R0,
        5,  // bgt
        -4, // back 4 instructions (from instruction 8 to instruction 4: add)
    );
    branch_instr.rs2 = Register::Zero; // compare r0 > 0
    program.instructions.push(branch_instr);

    // r0 = r2 (result)
    program.instructions.push(Instruction::new(
        Opcode::Mov,
        Register::R0,
        Register::R2,
        Register::Zero,
        0,
    ));
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    assert_eq!(run_program(&program), 55);
}
