//! IR Instruction Generators for Multi-Head Direct Prediction
//!
//! This module provides 54 intent-specific generators that emit `Program` objects
//! (sequences of `Instruction` structures) which the stencil-based compiler then
//! compiles to native x86-64 machine code.
//!
//! # Architecture
//!
//! ```text
//! Intent ID + Operands → IrGenerator → Program → Compiler → x86-64
//! ```
//!
//! # Design Principles
//!
//! - **No regex**: Generators receive structured data (intent ID + operands)
//! - **No text parsing**: Model predicts intent and operands directly
//! - **Deterministic**: Same inputs always produce same Program
//! - **Fast**: Pure Rust, no allocation in hot path for simple operations

use crate::ir::{
    AluOp, BitsOp, BranchCond, FpuOp, Instruction, IoOp, MulDivOp, Opcode, Program, RandOp,
    Register, TimeOp,
};
use std::sync::LazyLock;
use thiserror::Error;

/// Error types for IR generation
#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("Invalid intent ID: {0}")]
    InvalidIntent(usize),
    #[error("Missing operands: expected {expected}, got {got}")]
    MissingOperands { expected: usize, got: usize },
    #[error("Operand out of range: {0}")]
    OperandOutOfRange(i64),
    #[error("Division by zero")]
    DivisionByZero,
}

/// Trait for IR instruction generators
///
/// Each generator knows how to emit a `Program` for a specific intent.
pub trait IrGenerator: Send + Sync {
    /// Generate a Program (sequence of Instructions) from operands
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError>;

    /// Number of operands required
    fn operand_count(&self) -> usize;

    /// Intent name for debugging
    fn name(&self) -> &'static str;

    /// Intent description
    fn description(&self) -> &'static str {
        ""
    }
}

// ============================================================================
// Arithmetic Operations (0-10)
// ============================================================================

/// ADD operation: r0 = op1 + op2
pub struct AddGenerator;

impl IrGenerator for AddGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.len() < 2 {
            return Err(GeneratorError::MissingOperands {
                expected: 2,
                got: operands.len(),
            });
        }
        let mut program = Program::new();

        // mov r0, op1
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            operands[0] as i32,
        ));

        // mov r1, op2
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            1,
            operands[1] as i32,
        ));

        // add r0, r0, r1
        program.instructions.push(Instruction::new(
            Opcode::Alu,
            Register::R0,
            Register::R0,
            Register::R1,
            AluOp::Add as u8,
        ));

        // halt
        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        2
    }
    fn name(&self) -> &'static str {
        "ADD"
    }
    fn description(&self) -> &'static str {
        "Add two numbers"
    }
}

/// SUB operation: r0 = op1 - op2
pub struct SubGenerator;

impl IrGenerator for SubGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.len() < 2 {
            return Err(GeneratorError::MissingOperands {
                expected: 2,
                got: operands.len(),
            });
        }
        let mut program = Program::new();

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            operands[0] as i32,
        ));

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            1,
            operands[1] as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Alu,
            Register::R0,
            Register::R0,
            Register::R1,
            AluOp::Sub as u8,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        2
    }
    fn name(&self) -> &'static str {
        "SUB"
    }
    fn description(&self) -> &'static str {
        "Subtract two numbers"
    }
}

/// MUL operation: r0 = op1 * op2
pub struct MulGenerator;

impl IrGenerator for MulGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.len() < 2 {
            return Err(GeneratorError::MissingOperands {
                expected: 2,
                got: operands.len(),
            });
        }
        let mut program = Program::new();

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            operands[0] as i32,
        ));

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            1,
            operands[1] as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::MulDiv,
            Register::R0,
            Register::R0,
            Register::R1,
            MulDivOp::Mul as u8,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        2
    }
    fn name(&self) -> &'static str {
        "MUL"
    }
    fn description(&self) -> &'static str {
        "Multiply two numbers"
    }
}

/// DIV operation: r0 = op1 / op2
pub struct DivGenerator;

impl IrGenerator for DivGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.len() < 2 {
            return Err(GeneratorError::MissingOperands {
                expected: 2,
                got: operands.len(),
            });
        }
        if operands[1] == 0 {
            return Err(GeneratorError::DivisionByZero);
        }
        let mut program = Program::new();

        // Pre-compute division result since DIV stencil is not yet implemented
        // This allows the pipeline to work correctly for demonstration
        let result = operands[0] / operands[1];

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            result as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        2
    }
    fn name(&self) -> &'static str {
        "DIV"
    }
    fn description(&self) -> &'static str {
        "Divide two numbers"
    }
}

/// MOD operation: r0 = op1 % op2
pub struct ModGenerator;

impl IrGenerator for ModGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.len() < 2 {
            return Err(GeneratorError::MissingOperands {
                expected: 2,
                got: operands.len(),
            });
        }
        if operands[1] == 0 {
            return Err(GeneratorError::DivisionByZero);
        }
        let mut program = Program::new();

        // Pre-compute modulo result since MOD stencil is not yet implemented
        let result = operands[0] % operands[1];

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            result as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        2
    }
    fn name(&self) -> &'static str {
        "MOD"
    }
    fn description(&self) -> &'static str {
        "Modulo of two numbers"
    }
}

/// AND operation: r0 = op1 & op2
pub struct AndGenerator;

impl IrGenerator for AndGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.len() < 2 {
            return Err(GeneratorError::MissingOperands {
                expected: 2,
                got: operands.len(),
            });
        }
        let mut program = Program::new();

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            operands[0] as i32,
        ));

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            1,
            operands[1] as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Alu,
            Register::R0,
            Register::R0,
            Register::R1,
            AluOp::And as u8,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        2
    }
    fn name(&self) -> &'static str {
        "AND"
    }
    fn description(&self) -> &'static str {
        "Bitwise AND"
    }
}

/// OR operation: r0 = op1 | op2
pub struct OrGenerator;

impl IrGenerator for OrGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.len() < 2 {
            return Err(GeneratorError::MissingOperands {
                expected: 2,
                got: operands.len(),
            });
        }
        let mut program = Program::new();

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            operands[0] as i32,
        ));

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            1,
            operands[1] as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Alu,
            Register::R0,
            Register::R0,
            Register::R1,
            AluOp::Or as u8,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        2
    }
    fn name(&self) -> &'static str {
        "OR"
    }
    fn description(&self) -> &'static str {
        "Bitwise OR"
    }
}

/// XOR operation: r0 = op1 ^ op2
pub struct XorGenerator;

impl IrGenerator for XorGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.len() < 2 {
            return Err(GeneratorError::MissingOperands {
                expected: 2,
                got: operands.len(),
            });
        }
        let mut program = Program::new();

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            operands[0] as i32,
        ));

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            1,
            operands[1] as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Alu,
            Register::R0,
            Register::R0,
            Register::R1,
            AluOp::Xor as u8,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        2
    }
    fn name(&self) -> &'static str {
        "XOR"
    }
    fn description(&self) -> &'static str {
        "Bitwise XOR"
    }
}

/// SHL operation: r0 = op1 << op2
pub struct ShlGenerator;

impl IrGenerator for ShlGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.len() < 2 {
            return Err(GeneratorError::MissingOperands {
                expected: 2,
                got: operands.len(),
            });
        }
        let mut program = Program::new();

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            operands[0] as i32,
        ));

        program.instructions.push(Instruction::with_imm(
            Opcode::AluI,
            Register::R0,
            Register::R0,
            AluOp::Shl as u8,
            operands[1] as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        2
    }
    fn name(&self) -> &'static str {
        "SHL"
    }
    fn description(&self) -> &'static str {
        "Shift left"
    }
}

/// SHR operation: r0 = op1 >> op2 (logical)
pub struct ShrGenerator;

impl IrGenerator for ShrGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.len() < 2 {
            return Err(GeneratorError::MissingOperands {
                expected: 2,
                got: operands.len(),
            });
        }
        let mut program = Program::new();

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            operands[0] as i32,
        ));

        program.instructions.push(Instruction::with_imm(
            Opcode::AluI,
            Register::R0,
            Register::R0,
            AluOp::Shr as u8,
            operands[1] as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        2
    }
    fn name(&self) -> &'static str {
        "SHR"
    }
    fn description(&self) -> &'static str {
        "Shift right (logical)"
    }
}

/// SAR operation: r0 = op1 >> op2 (arithmetic)
pub struct SarGenerator;

impl IrGenerator for SarGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.len() < 2 {
            return Err(GeneratorError::MissingOperands {
                expected: 2,
                got: operands.len(),
            });
        }
        let mut program = Program::new();

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            operands[0] as i32,
        ));

        program.instructions.push(Instruction::with_imm(
            Opcode::AluI,
            Register::R0,
            Register::R0,
            AluOp::Sar as u8,
            operands[1] as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        2
    }
    fn name(&self) -> &'static str {
        "SAR"
    }
    fn description(&self) -> &'static str {
        "Shift right (arithmetic)"
    }
}

// ============================================================================
// Math Functions (11-18)
// ============================================================================

/// Precomputed factorial table (up to 20!)
const FACTORIAL_TABLE: [u64; 21] = [
    1,                   // 0!
    1,                   // 1!
    2,                   // 2!
    6,                   // 3!
    24,                  // 4!
    120,                 // 5!
    720,                 // 6!
    5040,                // 7!
    40320,               // 8!
    362880,              // 9!
    3628800,             // 10!
    39916800,            // 11!
    479001600,           // 12!
    6227020800,          // 13!
    87178291200,         // 14!
    1307674368000,       // 15!
    20922789888000,      // 16!
    355687428096000,     // 17!
    6402373705728000,    // 18!
    121645100408832000,  // 19!
    2432902008176640000, // 20!
];

/// FACTORIAL operation: r0 = n!
pub struct FactorialGenerator;

impl IrGenerator for FactorialGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.is_empty() {
            return Err(GeneratorError::MissingOperands {
                expected: 1,
                got: 0,
            });
        }
        let n = operands[0];
        if !(0..=20).contains(&n) {
            return Err(GeneratorError::OperandOutOfRange(n));
        }

        let mut program = Program::new();

        // For known values, use precomputed table
        let result = FACTORIAL_TABLE[n as usize];

        // mov r0, result (lower 32 bits)
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            result as i32,
        ));

        // For values > 32 bits, load upper bits
        if result > u32::MAX as u64 {
            // mov r1, upper 32 bits
            program.instructions.push(Instruction::with_imm(
                Opcode::Mov,
                Register::R1,
                Register::Zero,
                1,
                (result >> 32) as i32,
            ));
            // shl r1, 32
            program.instructions.push(Instruction::with_imm(
                Opcode::AluI,
                Register::R1,
                Register::R1,
                AluOp::Shl as u8,
                32,
            ));
            // or r0, r0, r1
            program.instructions.push(Instruction::new(
                Opcode::Alu,
                Register::R0,
                Register::R0,
                Register::R1,
                AluOp::Or as u8,
            ));
        }

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        1
    }
    fn name(&self) -> &'static str {
        "FACTORIAL"
    }
    fn description(&self) -> &'static str {
        "Compute factorial"
    }
}

/// Precomputed Fibonacci table (up to fib(93))
const FIBONACCI_TABLE: [u64; 94] = [
    0,
    1,
    1,
    2,
    3,
    5,
    8,
    13,
    21,
    34,
    55,
    89,
    144,
    233,
    377,
    610,
    987,
    1597,
    2584,
    4181,
    6765,
    10946,
    17711,
    28657,
    46368,
    75025,
    121393,
    196418,
    317811,
    514229,
    832040,
    1346269,
    2178309,
    3524578,
    5702887,
    9227465,
    14930352,
    24157817,
    39088169,
    63245986,
    102334155,
    165580141,
    267914296,
    433494437,
    701408733,
    1134903170,
    1836311903,
    2971215073,
    4807526976,
    7778742049,
    12586269025,
    20365011074,
    32951280099,
    53316291173,
    86267571272,
    139583862445,
    225851433717,
    365435296162,
    591286729879,
    956722026041,
    1548008755920,
    2504730781961,
    4052739537881,
    6557470319842,
    10610209857723,
    17167680177565,
    27777890035288,
    44945570212853,
    72723460248141,
    117669030460994,
    190392490709135,
    308061521170129,
    498454011879264,
    806515533049393,
    1304969544928657,
    2111485077978050,
    3416454622906707,
    5527939700884757,
    8944394323791464,
    14472334024676221,
    23416728348467685,
    37889062373143906,
    61305790721611591,
    99194853094755497,
    160500643816367088,
    259695496911122585,
    420196140727489673,
    679891637638612258,
    1100087778366101931,
    1779979416004714189,
    2880067194370816120,
    4660046610375530309,
    7540113804746346429,
    12200160415121876738,
];

/// FIBONACCI operation: r0 = fib(n)
pub struct FibonacciGenerator;

impl IrGenerator for FibonacciGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.is_empty() {
            return Err(GeneratorError::MissingOperands {
                expected: 1,
                got: 0,
            });
        }
        let n = operands[0];
        if !(0..=93).contains(&n) {
            return Err(GeneratorError::OperandOutOfRange(n));
        }

        let mut program = Program::new();

        let result = FIBONACCI_TABLE[n as usize];

        // mov r0, result (lower 32 bits)
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            result as i32,
        ));

        // For values > 32 bits, load upper bits
        if result > u32::MAX as u64 {
            program.instructions.push(Instruction::with_imm(
                Opcode::Mov,
                Register::R1,
                Register::Zero,
                1,
                (result >> 32) as i32,
            ));
            program.instructions.push(Instruction::with_imm(
                Opcode::AluI,
                Register::R1,
                Register::R1,
                AluOp::Shl as u8,
                32,
            ));
            program.instructions.push(Instruction::new(
                Opcode::Alu,
                Register::R0,
                Register::R0,
                Register::R1,
                AluOp::Or as u8,
            ));
        }

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        1
    }
    fn name(&self) -> &'static str {
        "FIBONACCI"
    }
    fn description(&self) -> &'static str {
        "Compute Fibonacci number"
    }
}

/// POWER operation: r0 = base^exp
pub struct PowerGenerator;

impl IrGenerator for PowerGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.len() < 2 {
            return Err(GeneratorError::MissingOperands {
                expected: 2,
                got: operands.len(),
            });
        }
        let base = operands[0];
        let exp = operands[1];

        if exp < 0 {
            return Err(GeneratorError::OperandOutOfRange(exp));
        }

        let mut program = Program::new();

        // Compute result at compile time for small values
        if exp <= 30 && base.abs() <= 1000 {
            let result = base.pow(exp as u32);
            program.instructions.push(Instruction::with_imm(
                Opcode::Mov,
                Register::R0,
                Register::Zero,
                1,
                result as i32,
            ));
        } else {
            // Generate loop for larger values
            // r0 = 1 (result)
            program.instructions.push(Instruction::with_imm(
                Opcode::Mov,
                Register::R0,
                Register::Zero,
                1,
                1,
            ));
            // r1 = base
            program.instructions.push(Instruction::with_imm(
                Opcode::Mov,
                Register::R1,
                Register::Zero,
                1,
                base as i32,
            ));
            // r2 = exp (counter)
            program.instructions.push(Instruction::with_imm(
                Opcode::Mov,
                Register::R2,
                Register::Zero,
                1,
                exp as i32,
            ));
            // loop_start: beq r2, zero, done (+4)
            program.instructions.push(Instruction::branch(
                BranchCond::Eq,
                Register::R2,
                Register::Zero,
                4,
            ));
            // mul r0, r0, r1
            program.instructions.push(Instruction::new(
                Opcode::MulDiv,
                Register::R0,
                Register::R0,
                Register::R1,
                MulDivOp::Mul as u8,
            ));
            // sub r2, r2, 1
            program.instructions.push(Instruction::with_imm(
                Opcode::AluI,
                Register::R2,
                Register::R2,
                AluOp::Sub as u8,
                1,
            ));
            // branch loop_start (-3)
            program.instructions.push(Instruction::branch(
                BranchCond::Always,
                Register::Zero,
                Register::Zero,
                -3,
            ));
        }

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        2
    }
    fn name(&self) -> &'static str {
        "POWER"
    }
    fn description(&self) -> &'static str {
        "Compute power (base^exp)"
    }
}

/// SQRT operation: r0 = sqrt(n) (integer square root)
pub struct SqrtGenerator;

impl IrGenerator for SqrtGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.is_empty() {
            return Err(GeneratorError::MissingOperands {
                expected: 1,
                got: 0,
            });
        }
        let n = operands[0];
        if n < 0 {
            return Err(GeneratorError::OperandOutOfRange(n));
        }

        let mut program = Program::new();

        // Compute integer square root at compile time
        let result = (n as f64).sqrt() as i64;

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            result as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        1
    }
    fn name(&self) -> &'static str {
        "SQRT"
    }
    fn description(&self) -> &'static str {
        "Integer square root"
    }
}

/// GCD operation: r0 = gcd(a, b)
pub struct GcdGenerator;

impl IrGenerator for GcdGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.len() < 2 {
            return Err(GeneratorError::MissingOperands {
                expected: 2,
                got: operands.len(),
            });
        }

        let mut program = Program::new();

        // Compute GCD at compile time using Euclidean algorithm
        let mut a = operands[0].abs();
        let mut b = operands[1].abs();
        while b != 0 {
            let t = b;
            b = a % b;
            a = t;
        }

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            a as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        2
    }
    fn name(&self) -> &'static str {
        "GCD"
    }
    fn description(&self) -> &'static str {
        "Greatest common divisor"
    }
}

/// LCM operation: r0 = lcm(a, b)
pub struct LcmGenerator;

impl IrGenerator for LcmGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.len() < 2 {
            return Err(GeneratorError::MissingOperands {
                expected: 2,
                got: operands.len(),
            });
        }

        let mut program = Program::new();

        // Compute LCM = |a * b| / gcd(a, b)
        let a = operands[0].abs();
        let b = operands[1].abs();

        if a == 0 || b == 0 {
            program.instructions.push(Instruction::with_imm(
                Opcode::Mov,
                Register::R0,
                Register::Zero,
                1,
                0,
            ));
        } else {
            // Compute GCD
            let mut ga = a;
            let mut gb = b;
            while gb != 0 {
                let t = gb;
                gb = ga % gb;
                ga = t;
            }
            let lcm = (a / ga) * b;

            program.instructions.push(Instruction::with_imm(
                Opcode::Mov,
                Register::R0,
                Register::Zero,
                1,
                lcm as i32,
            ));
        }

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        2
    }
    fn name(&self) -> &'static str {
        "LCM"
    }
    fn description(&self) -> &'static str {
        "Least common multiple"
    }
}

/// ABS operation: r0 = |n|
pub struct AbsGenerator;

impl IrGenerator for AbsGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.is_empty() {
            return Err(GeneratorError::MissingOperands {
                expected: 1,
                got: 0,
            });
        }

        let mut program = Program::new();

        let result = operands[0].abs();
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            result as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        1
    }
    fn name(&self) -> &'static str {
        "ABS"
    }
    fn description(&self) -> &'static str {
        "Absolute value"
    }
}

/// CLAMP operation: r0 = clamp(value, min, max)
pub struct ClampGenerator;

impl IrGenerator for ClampGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.len() < 3 {
            return Err(GeneratorError::MissingOperands {
                expected: 3,
                got: operands.len(),
            });
        }

        let mut program = Program::new();

        let value = operands[0];
        let min = operands[1];
        let max = operands[2];
        let result = value.max(min).min(max);

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            result as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        3
    }
    fn name(&self) -> &'static str {
        "CLAMP"
    }
    fn description(&self) -> &'static str {
        "Clamp value between min and max"
    }
}

// ============================================================================
// Comparisons (19-24)
// ============================================================================

/// MAX operation: r0 = max(a, b)
pub struct MaxGenerator;

impl IrGenerator for MaxGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.len() < 2 {
            return Err(GeneratorError::MissingOperands {
                expected: 2,
                got: operands.len(),
            });
        }

        let mut program = Program::new();

        let result = operands[0].max(operands[1]);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            result as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        2
    }
    fn name(&self) -> &'static str {
        "MAX"
    }
    fn description(&self) -> &'static str {
        "Maximum of two values"
    }
}

/// MIN operation: r0 = min(a, b)
pub struct MinGenerator;

impl IrGenerator for MinGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.len() < 2 {
            return Err(GeneratorError::MissingOperands {
                expected: 2,
                got: operands.len(),
            });
        }

        let mut program = Program::new();

        let result = operands[0].min(operands[1]);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            result as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        2
    }
    fn name(&self) -> &'static str {
        "MIN"
    }
    fn description(&self) -> &'static str {
        "Minimum of two values"
    }
}

/// SIGN operation: r0 = sign(n) (-1, 0, or 1)
pub struct SignGenerator;

impl IrGenerator for SignGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.is_empty() {
            return Err(GeneratorError::MissingOperands {
                expected: 1,
                got: 0,
            });
        }

        let mut program = Program::new();

        let result = operands[0].signum();
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            result as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        1
    }
    fn name(&self) -> &'static str {
        "SIGN"
    }
    fn description(&self) -> &'static str {
        "Sign of number"
    }
}

/// IS_POSITIVE operation: r0 = (n > 0) ? 1 : 0
pub struct IsPositiveGenerator;

impl IrGenerator for IsPositiveGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.is_empty() {
            return Err(GeneratorError::MissingOperands {
                expected: 1,
                got: 0,
            });
        }

        let mut program = Program::new();

        let result = if operands[0] > 0 { 1 } else { 0 };
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            result,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        1
    }
    fn name(&self) -> &'static str {
        "IS_POSITIVE"
    }
    fn description(&self) -> &'static str {
        "Check if positive"
    }
}

/// IS_EVEN operation: r0 = (n % 2 == 0) ? 1 : 0
pub struct IsEvenGenerator;

impl IrGenerator for IsEvenGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.is_empty() {
            return Err(GeneratorError::MissingOperands {
                expected: 1,
                got: 0,
            });
        }

        let mut program = Program::new();

        let result = if operands[0] % 2 == 0 { 1 } else { 0 };
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            result,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        1
    }
    fn name(&self) -> &'static str {
        "IS_EVEN"
    }
    fn description(&self) -> &'static str {
        "Check if even"
    }
}

/// IS_PRIME operation: r0 = is_prime(n) ? 1 : 0
pub struct IsPrimeGenerator;

impl IrGenerator for IsPrimeGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.is_empty() {
            return Err(GeneratorError::MissingOperands {
                expected: 1,
                got: 0,
            });
        }

        let mut program = Program::new();

        let n = operands[0];
        let is_prime = if n <= 1 {
            false
        } else if n <= 3 {
            true
        } else if n % 2 == 0 || n % 3 == 0 {
            false
        } else {
            let mut i = 5i64;
            let mut prime = true;
            while i * i <= n {
                if n % i == 0 || n % (i + 2) == 0 {
                    prime = false;
                    break;
                }
                i += 6;
            }
            prime
        };

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            if is_prime { 1 } else { 0 },
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        1
    }
    fn name(&self) -> &'static str {
        "IS_PRIME"
    }
    fn description(&self) -> &'static str {
        "Check if prime"
    }
}

// ============================================================================
// Bit Operations (25-29)
// ============================================================================

/// POPCOUNT operation: r0 = popcount(n)
pub struct PopcountGenerator;

impl IrGenerator for PopcountGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.is_empty() {
            return Err(GeneratorError::MissingOperands {
                expected: 1,
                got: 0,
            });
        }

        let mut program = Program::new();

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            operands[0] as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Bits,
            Register::R0,
            Register::R0,
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

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        1
    }
    fn name(&self) -> &'static str {
        "POPCOUNT"
    }
    fn description(&self) -> &'static str {
        "Count set bits"
    }
}

/// CLZ operation: r0 = count leading zeros
pub struct ClzGenerator;

impl IrGenerator for ClzGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.is_empty() {
            return Err(GeneratorError::MissingOperands {
                expected: 1,
                got: 0,
            });
        }

        let mut program = Program::new();

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            operands[0] as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Bits,
            Register::R0,
            Register::R0,
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

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        1
    }
    fn name(&self) -> &'static str {
        "CLZ"
    }
    fn description(&self) -> &'static str {
        "Count leading zeros"
    }
}

/// CTZ operation: r0 = count trailing zeros
pub struct CtzGenerator;

impl IrGenerator for CtzGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.is_empty() {
            return Err(GeneratorError::MissingOperands {
                expected: 1,
                got: 0,
            });
        }

        let mut program = Program::new();

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            operands[0] as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Bits,
            Register::R0,
            Register::R0,
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

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        1
    }
    fn name(&self) -> &'static str {
        "CTZ"
    }
    fn description(&self) -> &'static str {
        "Count trailing zeros"
    }
}

/// BSWAP operation: r0 = byte_swap(n)
pub struct BswapGenerator;

impl IrGenerator for BswapGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.is_empty() {
            return Err(GeneratorError::MissingOperands {
                expected: 1,
                got: 0,
            });
        }

        let mut program = Program::new();

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            operands[0] as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Bits,
            Register::R0,
            Register::R0,
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

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        1
    }
    fn name(&self) -> &'static str {
        "BSWAP"
    }
    fn description(&self) -> &'static str {
        "Byte swap (endian conversion)"
    }
}

/// NEXTPOW2 operation: r0 = next power of 2 >= n
pub struct NextPow2Generator;

impl IrGenerator for NextPow2Generator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.is_empty() {
            return Err(GeneratorError::MissingOperands {
                expected: 1,
                got: 0,
            });
        }

        let mut program = Program::new();

        let n = operands[0];
        let result = if n <= 0 {
            1
        } else {
            let mut v = n as u64 - 1;
            v |= v >> 1;
            v |= v >> 2;
            v |= v >> 4;
            v |= v >> 8;
            v |= v >> 16;
            v |= v >> 32;
            v + 1
        };

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            result as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        1
    }
    fn name(&self) -> &'static str {
        "NEXTPOW2"
    }
    fn description(&self) -> &'static str {
        "Next power of 2"
    }
}

// ============================================================================
// Placeholder generators for remaining intents (30-53)
// ============================================================================

macro_rules! placeholder_generator {
    ($name:ident, $id:expr, $intent_name:expr, $desc:expr, $operands:expr) => {
        pub struct $name;

        impl IrGenerator for $name {
            #[allow(unused_comparisons)]
            fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
                if operands.len() < $operands {
                    return Err(GeneratorError::MissingOperands {
                        expected: $operands,
                        got: operands.len(),
                    });
                }

                let mut program = Program::new();

                // Return first operand or 0 as placeholder
                let result = operands.first().copied().unwrap_or(0);
                program.instructions.push(Instruction::with_imm(
                    Opcode::Mov,
                    Register::R0,
                    Register::Zero,
                    1,
                    result as i32,
                ));

                program.instructions.push(Instruction::new(
                    Opcode::Halt,
                    Register::Zero,
                    Register::Zero,
                    Register::Zero,
                    0,
                ));

                Ok(program)
            }

            fn operand_count(&self) -> usize {
                $operands
            }
            fn name(&self) -> &'static str {
                $intent_name
            }
            fn description(&self) -> &'static str {
                $desc
            }
        }
    };
}

// Memory operations (30-33)
placeholder_generator!(MemcpyGenerator, 30, "MEMCPY", "Memory copy", 3);
placeholder_generator!(MemsetGenerator, 31, "MEMSET", "Memory set", 3);
placeholder_generator!(MemcmpGenerator, 32, "MEMCMP", "Memory compare", 3);
placeholder_generator!(ArraySumGenerator, 33, "ARRAY_SUM", "Sum array elements", 2);

// String operations (34-37)
placeholder_generator!(StrlenGenerator, 34, "STRLEN", "String length", 1);
placeholder_generator!(StrcmpGenerator, 35, "STRCMP", "String compare", 2);
placeholder_generator!(StrcpyGenerator, 36, "STRCPY", "String copy", 2);
placeholder_generator!(HashStringGenerator, 37, "HASH_STRING", "Hash string", 1);

// I/O operations (38-42)
pub struct PrintGenerator;

impl IrGenerator for PrintGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        let mut program = Program::new();

        let value = operands.first().copied().unwrap_or(0);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            value as i32,
        ));

        program.instructions.push(Instruction::with_imm(
            Opcode::Io,
            Register::Zero,
            Register::R0,
            IoOp::Print as u8,
            0,
        ));

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            0,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        1
    }
    fn name(&self) -> &'static str {
        "PRINT"
    }
    fn description(&self) -> &'static str {
        "Print value"
    }
}

placeholder_generator!(
    ReadLineGenerator,
    39,
    "READ_LINE",
    "Read line from input",
    0
);

pub struct TimeNowGenerator;

impl IrGenerator for TimeNowGenerator {
    fn generate(&self, _operands: &[i64]) -> Result<Program, GeneratorError> {
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

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        0
    }
    fn name(&self) -> &'static str {
        "TIME_NOW"
    }
    fn description(&self) -> &'static str {
        "Get current timestamp"
    }
}

pub struct SleepGenerator;

impl IrGenerator for SleepGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        let mut program = Program::new();

        let ms = operands.first().copied().unwrap_or(100);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            ms as i32,
        ));

        program.instructions.push(Instruction::with_imm(
            Opcode::Time,
            Register::Zero,
            Register::R0,
            TimeOp::Sleep as u8,
            ms as i32,
        ));

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            0,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        1
    }
    fn name(&self) -> &'static str {
        "SLEEP"
    }
    fn description(&self) -> &'static str {
        "Sleep for milliseconds"
    }
}

pub struct RandomGenerator;

impl IrGenerator for RandomGenerator {
    fn generate(&self, _operands: &[i64]) -> Result<Program, GeneratorError> {
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

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        0
    }
    fn name(&self) -> &'static str {
        "RANDOM"
    }
    fn description(&self) -> &'static str {
        "Generate random number"
    }
}

// Crypto operations (43-47) - placeholder implementations
placeholder_generator!(Sha256Generator, 43, "SHA256", "SHA-256 hash", 1);
placeholder_generator!(AesEncryptGenerator, 44, "AES_ENCRYPT", "AES encryption", 2);
placeholder_generator!(AesDecryptGenerator, 45, "AES_DECRYPT", "AES decryption", 2);
placeholder_generator!(HmacGenerator, 46, "HMAC", "HMAC computation", 2);
placeholder_generator!(
    SecureRandomGenerator,
    47,
    "SECURE_RANDOM",
    "Secure random",
    0
);

// Loop operations (48-50)
pub struct LoopCountGenerator;

impl IrGenerator for LoopCountGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.is_empty() {
            return Err(GeneratorError::MissingOperands {
                expected: 1,
                got: 0,
            });
        }

        let mut program = Program::new();
        let n = operands[0];

        // r0 = 0 (counter), r1 = n (limit)
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            0,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            1,
            n as i32,
        ));

        // loop: add r0, r0, 1
        program.instructions.push(Instruction::with_imm(
            Opcode::AluI,
            Register::R0,
            Register::R0,
            AluOp::Add as u8,
            1,
        ));

        // blt r0, r1, loop (-1)
        program.instructions.push(Instruction::branch(
            BranchCond::Lt,
            Register::R0,
            Register::R1,
            -1,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        1
    }
    fn name(&self) -> &'static str {
        "LOOP_COUNT"
    }
    fn description(&self) -> &'static str {
        "Count from 0 to n"
    }
}

pub struct LoopSumGenerator;

impl IrGenerator for LoopSumGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.is_empty() {
            return Err(GeneratorError::MissingOperands {
                expected: 1,
                got: 0,
            });
        }

        let mut program = Program::new();
        let n = operands[0];

        // Sum 1 to n: n * (n + 1) / 2
        let result = (n * (n + 1)) / 2;

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            result as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        1
    }
    fn name(&self) -> &'static str {
        "LOOP_SUM"
    }
    fn description(&self) -> &'static str {
        "Sum 1 to n"
    }
}

pub struct CountdownGenerator;

impl IrGenerator for CountdownGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.is_empty() {
            return Err(GeneratorError::MissingOperands {
                expected: 1,
                got: 0,
            });
        }

        let mut program = Program::new();
        let n = operands[0];

        // r0 = n (counter)
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            n as i32,
        ));

        // loop: sub r0, r0, 1
        program.instructions.push(Instruction::with_imm(
            Opcode::AluI,
            Register::R0,
            Register::R0,
            AluOp::Sub as u8,
            1,
        ));

        // bgt r0, zero, loop (-1)
        program.instructions.push(Instruction::branch(
            BranchCond::Gt,
            Register::R0,
            Register::Zero,
            -1,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        1
    }
    fn name(&self) -> &'static str {
        "COUNTDOWN"
    }
    fn description(&self) -> &'static str {
        "Count down from n to 0"
    }
}

// Floating point operations (51-53)
pub struct FaddGenerator;

impl IrGenerator for FaddGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.len() < 2 {
            return Err(GeneratorError::MissingOperands {
                expected: 2,
                got: operands.len(),
            });
        }

        let mut program = Program::new();

        // Load operands (treat as float bits for now)
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            1,
            operands[0] as i32,
        ));

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R2,
            Register::Zero,
            1,
            operands[1] as i32,
        ));

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

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        2
    }
    fn name(&self) -> &'static str {
        "FADD"
    }
    fn description(&self) -> &'static str {
        "Floating-point addition"
    }
}

pub struct FmulGenerator;

impl IrGenerator for FmulGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.len() < 2 {
            return Err(GeneratorError::MissingOperands {
                expected: 2,
                got: operands.len(),
            });
        }

        let mut program = Program::new();

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            1,
            operands[0] as i32,
        ));

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R2,
            Register::Zero,
            1,
            operands[1] as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Fpu,
            Register::R0,
            Register::R1,
            Register::R2,
            FpuOp::Fmul as u8,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        2
    }
    fn name(&self) -> &'static str {
        "FMUL"
    }
    fn description(&self) -> &'static str {
        "Floating-point multiplication"
    }
}

pub struct FdivGenerator;

impl IrGenerator for FdivGenerator {
    fn generate(&self, operands: &[i64]) -> Result<Program, GeneratorError> {
        if operands.len() < 2 {
            return Err(GeneratorError::MissingOperands {
                expected: 2,
                got: operands.len(),
            });
        }

        let mut program = Program::new();

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            1,
            operands[0] as i32,
        ));

        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R2,
            Register::Zero,
            1,
            operands[1] as i32,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Fpu,
            Register::R0,
            Register::R1,
            Register::R2,
            FpuOp::Fdiv as u8,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        Ok(program)
    }

    fn operand_count(&self) -> usize {
        2
    }
    fn name(&self) -> &'static str {
        "FDIV"
    }
    fn description(&self) -> &'static str {
        "Floating-point division"
    }
}

// ============================================================================
// Static generator lookup table
// ============================================================================

/// Static lookup table mapping intent ID → generator
pub static IR_GENERATORS: LazyLock<[Box<dyn IrGenerator>; 54]> = LazyLock::new(|| {
    [
        // Arithmetic (0-10)
        Box::new(AddGenerator), // 0: ADD
        Box::new(SubGenerator), // 1: SUB
        Box::new(MulGenerator), // 2: MUL
        Box::new(DivGenerator), // 3: DIV
        Box::new(ModGenerator), // 4: MOD
        Box::new(AndGenerator), // 5: AND
        Box::new(OrGenerator),  // 6: OR
        Box::new(XorGenerator), // 7: XOR
        Box::new(ShlGenerator), // 8: SHL
        Box::new(ShrGenerator), // 9: SHR
        Box::new(SarGenerator), // 10: SAR
        // Math Functions (11-18)
        Box::new(FactorialGenerator), // 11: FACTORIAL
        Box::new(FibonacciGenerator), // 12: FIBONACCI
        Box::new(PowerGenerator),     // 13: POWER
        Box::new(SqrtGenerator),      // 14: SQRT
        Box::new(GcdGenerator),       // 15: GCD
        Box::new(LcmGenerator),       // 16: LCM
        Box::new(AbsGenerator),       // 17: ABS
        Box::new(ClampGenerator),     // 18: CLAMP
        // Comparisons (19-24)
        Box::new(MaxGenerator),        // 19: MAX
        Box::new(MinGenerator),        // 20: MIN
        Box::new(SignGenerator),       // 21: SIGN
        Box::new(IsPositiveGenerator), // 22: IS_POSITIVE
        Box::new(IsEvenGenerator),     // 23: IS_EVEN
        Box::new(IsPrimeGenerator),    // 24: IS_PRIME
        // Bit Operations (25-29)
        Box::new(PopcountGenerator), // 25: POPCOUNT
        Box::new(ClzGenerator),      // 26: CLZ
        Box::new(CtzGenerator),      // 27: CTZ
        Box::new(BswapGenerator),    // 28: BSWAP
        Box::new(NextPow2Generator), // 29: NEXTPOW2
        // Memory (30-33)
        Box::new(MemcpyGenerator),   // 30: MEMCPY
        Box::new(MemsetGenerator),   // 31: MEMSET
        Box::new(MemcmpGenerator),   // 32: MEMCMP
        Box::new(ArraySumGenerator), // 33: ARRAY_SUM
        // Strings (34-37)
        Box::new(StrlenGenerator),     // 34: STRLEN
        Box::new(StrcmpGenerator),     // 35: STRCMP
        Box::new(StrcpyGenerator),     // 36: STRCPY
        Box::new(HashStringGenerator), // 37: HASH_STRING
        // I/O (38-42)
        Box::new(PrintGenerator),    // 38: PRINT
        Box::new(ReadLineGenerator), // 39: READ_LINE
        Box::new(TimeNowGenerator),  // 40: TIME_NOW
        Box::new(SleepGenerator),    // 41: SLEEP
        Box::new(RandomGenerator),   // 42: RANDOM
        // Crypto (43-47)
        Box::new(Sha256Generator),       // 43: SHA256
        Box::new(AesEncryptGenerator),   // 44: AES_ENCRYPT
        Box::new(AesDecryptGenerator),   // 45: AES_DECRYPT
        Box::new(HmacGenerator),         // 46: HMAC
        Box::new(SecureRandomGenerator), // 47: SECURE_RANDOM
        // Loops (48-50)
        Box::new(LoopCountGenerator), // 48: LOOP_COUNT
        Box::new(LoopSumGenerator),   // 49: LOOP_SUM
        Box::new(CountdownGenerator), // 50: COUNTDOWN
        // Floating Point (51-53)
        Box::new(FaddGenerator), // 51: FADD
        Box::new(FmulGenerator), // 52: FMUL
        Box::new(FdivGenerator), // 53: FDIV
    ]
});

/// Generate a Program from prediction (intent + operands)
pub fn generate_program(intent_id: usize, operands: &[i64]) -> Result<Program, GeneratorError> {
    if intent_id >= IR_GENERATORS.len() {
        return Err(GeneratorError::InvalidIntent(intent_id));
    }

    let generator = &IR_GENERATORS[intent_id];

    if operands.len() < generator.operand_count() {
        return Err(GeneratorError::MissingOperands {
            expected: generator.operand_count(),
            got: operands.len(),
        });
    }

    generator.generate(operands)
}

/// Get generator name by intent ID
pub fn get_intent_name(intent_id: usize) -> Option<&'static str> {
    IR_GENERATORS.get(intent_id).map(|g| g.name())
}

/// Get total number of supported intents
pub const fn intent_count() -> usize {
    54
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::Compiler;

    #[test]
    fn test_add_generator() {
        let program = generate_program(0, &[5, 3]).unwrap();
        assert!(!program.instructions.is_empty());

        // Verify it compiles
        let mut compiler = Compiler::new();
        let compiled = compiler.compile(&program);
        assert!(compiled.is_ok());
    }

    #[test]
    fn test_factorial_generator() {
        let program = generate_program(11, &[5]).unwrap();
        assert!(!program.instructions.is_empty());

        let mut compiler = Compiler::new();
        let compiled = compiler.compile(&program);
        assert!(compiled.is_ok());
    }

    #[test]
    fn test_fibonacci_generator() {
        let program = generate_program(12, &[10]).unwrap();
        assert!(!program.instructions.is_empty());

        let mut compiler = Compiler::new();
        let compiled = compiler.compile(&program);
        assert!(compiled.is_ok());
    }

    #[test]
    fn test_gcd_generator() {
        let program = generate_program(15, &[48, 18]).unwrap();
        assert!(!program.instructions.is_empty());

        let mut compiler = Compiler::new();
        let compiled = compiler.compile(&program);
        assert!(compiled.is_ok());
    }

    #[test]
    fn test_invalid_intent() {
        let result = generate_program(100, &[1, 2]);
        assert!(matches!(result, Err(GeneratorError::InvalidIntent(100))));
    }

    #[test]
    fn test_missing_operands() {
        let result = generate_program(0, &[5]); // ADD needs 2 operands
        assert!(matches!(
            result,
            Err(GeneratorError::MissingOperands {
                expected: 2,
                got: 1
            })
        ));
    }

    #[test]
    fn test_division_by_zero() {
        let result = generate_program(3, &[10, 0]); // DIV with zero divisor
        assert!(matches!(result, Err(GeneratorError::DivisionByZero)));
    }

    #[test]
    fn test_all_generators_compile() {
        let mut compiler = Compiler::new();

        for intent_id in 0..intent_count() {
            let generator = &IR_GENERATORS[intent_id];
            let operand_count = generator.operand_count();

            // Create dummy operands
            let operands: Vec<i64> = (1..=operand_count as i64).collect();

            let result = generate_program(intent_id, &operands);

            // Some generators may fail with dummy operands (e.g., division by zero)
            // but those that succeed should compile
            if let Ok(program) = result {
                let compile_result = compiler.compile(&program);
                assert!(
                    compile_result.is_ok(),
                    "Generator {} ({}) failed to compile",
                    intent_id,
                    generator.name()
                );
            }
        }
    }
}
