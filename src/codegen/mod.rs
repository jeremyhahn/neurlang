//! Code Generation Module for Neurlang IR
//!
//! This module provides transpilation from Neurlang IR to various target languages.
//! It uses a visitor-based pattern where each language implements the `CodeGenerator` trait.
//!
//! # Supported Targets
//!
//! - **C**: Direct mapping of IR operations to C code
//! - **Go**: Maps concurrency primitives to goroutines/channels
//! - **Rust**: Safe Rust with explicit memory management
//! - **Pseudocode**: Human-readable description (the "human lens")
//!
//! # Architecture
//!
//! ```text
//! ┌───────────────────────────────────────────────────────┐
//! │                    Program (IR)                        │
//! └───────────────────────┬───────────────────────────────┘
//!                         │
//!                         ▼
//! ┌───────────────────────────────────────────────────────┐
//! │              CodeGenerator Trait                       │
//! │  fn emit_alu(&mut self, op, rd, rs1, rs2)             │
//! │  fn emit_load(&mut self, width, rd, base, offset)     │
//! │  fn emit_store(&mut self, width, src, base, offset)   │
//! │  fn emit_branch(&mut self, cond, target)              │
//! │  ...                                                   │
//! └───────────┬───────────┬───────────┬───────────────────┘
//!             │           │           │
//!      ┌──────┴───┐  ┌────┴────┐  ┌───┴────┐
//!      │ CCodeGen │  │ GoGen   │  │ RustGen│  ...
//!      └──────────┘  └─────────┘  └────────┘
//! ```

pub mod c;
pub mod common;
pub mod go;
pub mod pseudocode;
pub mod rust;

pub use c::CCodeGenerator;
pub use common::{CodeGenContext, CodeGenOptions, IndentWriter};
pub use go::GoCodeGenerator;
pub use pseudocode::PseudocodeGenerator;
pub use rust::RustCodeGenerator;

use crate::ir::{
    AluOp, AtomicOp, BitsOp, BranchCond, ChanOp, FenceMode, FileOp, FpuOp, Instruction, IoOp,
    MemWidth, MulDivOp, NetOp, Opcode, Program, RandOp, Register, TimeOp, TrapType,
};
use thiserror::Error;

/// Errors that can occur during code generation
#[derive(Debug, Error)]
pub enum CodeGenError {
    #[error("Unsupported opcode: {0:?}")]
    UnsupportedOpcode(Opcode),

    #[error("Invalid register: {0:?}")]
    InvalidRegister(Register),

    #[error("Missing immediate value for instruction at index {0}")]
    MissingImmediate(usize),

    #[error("Invalid branch target: {0}")]
    InvalidBranchTarget(i32),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Code generation failed: {0}")]
    GenerationFailed(String),
}

/// Result type for code generation operations
pub type CodeGenResult<T> = Result<T, CodeGenError>;

/// Trait for generating code from Neurlang IR
///
/// Implementations should produce valid source code in the target language
/// that is semantically equivalent to the IR program.
pub trait CodeGenerator {
    /// Get the target language name
    fn target_name(&self) -> &'static str;

    /// Get the file extension for the target language
    fn file_extension(&self) -> &'static str;

    /// Generate complete source code from a program
    fn generate(&mut self, program: &Program) -> CodeGenResult<String>;

    /// Emit program prologue (includes, imports, boilerplate)
    fn emit_prologue(&mut self) -> CodeGenResult<()>;

    /// Emit program epilogue (cleanup, main function end)
    fn emit_epilogue(&mut self) -> CodeGenResult<()>;

    /// Emit a single instruction
    fn emit_instruction(&mut self, instr: &Instruction, index: usize) -> CodeGenResult<()>;

    // ALU operations
    fn emit_alu(
        &mut self,
        op: AluOp,
        rd: Register,
        rs1: Register,
        rs2: Register,
    ) -> CodeGenResult<()>;
    fn emit_alu_imm(
        &mut self,
        op: AluOp,
        rd: Register,
        rs1: Register,
        imm: i32,
    ) -> CodeGenResult<()>;
    fn emit_muldiv(
        &mut self,
        op: MulDivOp,
        rd: Register,
        rs1: Register,
        rs2: Register,
    ) -> CodeGenResult<()>;

    // Memory operations
    fn emit_load(
        &mut self,
        width: MemWidth,
        rd: Register,
        base: Register,
        offset: i32,
    ) -> CodeGenResult<()>;
    fn emit_store(
        &mut self,
        width: MemWidth,
        src: Register,
        base: Register,
        offset: i32,
    ) -> CodeGenResult<()>;
    fn emit_atomic(
        &mut self,
        op: AtomicOp,
        rd: Register,
        rs1: Register,
        rs2: Register,
    ) -> CodeGenResult<()>;

    // Control flow
    fn emit_branch(
        &mut self,
        cond: BranchCond,
        rs1: Register,
        rs2: Register,
        target: i32,
    ) -> CodeGenResult<()>;
    fn emit_call(&mut self, target: i32) -> CodeGenResult<()>;
    fn emit_ret(&mut self) -> CodeGenResult<()>;
    fn emit_jump(&mut self, target: i32, indirect: bool) -> CodeGenResult<()>;

    // Capabilities
    fn emit_cap_new(&mut self, rd: Register, base: Register, len: Register) -> CodeGenResult<()>;
    fn emit_cap_restrict(
        &mut self,
        rd: Register,
        src: Register,
        len: Register,
    ) -> CodeGenResult<()>;
    fn emit_cap_query(&mut self, rd: Register, cap: Register, query_type: i32)
        -> CodeGenResult<()>;

    // Concurrency
    fn emit_spawn(&mut self, rd: Register, target: i32, arg: Register) -> CodeGenResult<()>;
    fn emit_join(&mut self, task: Register) -> CodeGenResult<()>;
    fn emit_chan(&mut self, op: ChanOp, rd: Register, rs1: Register) -> CodeGenResult<()>;
    fn emit_fence(&mut self, mode: FenceMode) -> CodeGenResult<()>;
    fn emit_yield(&mut self) -> CodeGenResult<()>;

    // Taint tracking
    fn emit_taint(&mut self, rd: Register, rs1: Register) -> CodeGenResult<()>;
    fn emit_sanitize(&mut self, rd: Register, rs1: Register) -> CodeGenResult<()>;

    // I/O operations
    fn emit_file(
        &mut self,
        op: FileOp,
        rd: Register,
        rs1: Register,
        rs2: Register,
        imm: Option<i32>,
    ) -> CodeGenResult<()>;
    fn emit_net(
        &mut self,
        op: NetOp,
        rd: Register,
        rs1: Register,
        rs2: Register,
        imm: Option<i32>,
    ) -> CodeGenResult<()>;
    fn emit_io(
        &mut self,
        op: IoOp,
        rd: Register,
        rs1: Register,
        rs2: Register,
    ) -> CodeGenResult<()>;
    fn emit_time(&mut self, op: TimeOp, rd: Register, imm: Option<i32>) -> CodeGenResult<()>;

    // Math extensions
    fn emit_fpu(
        &mut self,
        op: FpuOp,
        rd: Register,
        rs1: Register,
        rs2: Register,
    ) -> CodeGenResult<()>;
    fn emit_rand(&mut self, op: RandOp, rd: Register, rs1: Register) -> CodeGenResult<()>;
    fn emit_bits(&mut self, op: BitsOp, rd: Register, rs1: Register) -> CodeGenResult<()>;

    // System
    fn emit_mov(&mut self, rd: Register, rs1: Register, imm: Option<i32>) -> CodeGenResult<()>;
    fn emit_trap(&mut self, trap_type: TrapType, imm: Option<i32>) -> CodeGenResult<()>;
    fn emit_nop(&mut self) -> CodeGenResult<()>;
    fn emit_halt(&mut self) -> CodeGenResult<()>;

    // Extensions
    fn emit_ext_call(
        &mut self,
        rd: Register,
        ext_id: i32,
        rs1: Register,
        rs2: Register,
    ) -> CodeGenResult<()>;

    // Label management
    fn emit_label(&mut self, index: usize) -> CodeGenResult<()>;
}

/// Generate code for a program using the specified generator
pub fn generate_code<G: CodeGenerator>(
    generator: &mut G,
    program: &Program,
) -> CodeGenResult<String> {
    generator.generate(program)
}

/// Quick function to generate C code from a program
pub fn to_c(program: &Program) -> CodeGenResult<String> {
    let mut gen = CCodeGenerator::new();
    gen.generate(program)
}

/// Quick function to generate Go code from a program
pub fn to_go(program: &Program) -> CodeGenResult<String> {
    let mut gen = GoCodeGenerator::new();
    gen.generate(program)
}

/// Quick function to generate Rust code from a program
pub fn to_rust(program: &Program) -> CodeGenResult<String> {
    let mut gen = RustCodeGenerator::new();
    gen.generate(program)
}

/// Quick function to generate pseudocode from a program
pub fn to_pseudocode(program: &Program) -> CodeGenResult<String> {
    let mut gen = PseudocodeGenerator::new();
    gen.generate(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Assembler;

    // Helper to assemble a test program
    fn assemble(source: &str) -> Program {
        let mut asm = Assembler::new();
        asm.assemble(source).unwrap()
    }

    // =========================================================================
    // C Code Generator Tests
    // =========================================================================

    #[test]
    fn test_c_simple_mov() {
        let program = assemble("mov r0, 42\nhalt");
        let code = to_c(&program).unwrap();
        assert!(code.contains("42"));
        assert!(code.contains("regs[0]"));
    }

    #[test]
    fn test_c_alu_operations() {
        let program = assemble("mov r0, 10\nmov r1, 20\nadd r2, r0, r1\nhalt");
        let code = to_c(&program).unwrap();
        assert!(code.contains("10"));
        assert!(code.contains("20"));
        assert!(code.contains("+"));
    }

    #[test]
    fn test_c_sub_operation() {
        let program = assemble("mov r0, 30\nmov r1, 10\nsub r2, r0, r1\nhalt");
        let code = to_c(&program).unwrap();
        assert!(code.contains("-"));
    }

    #[test]
    fn test_c_mul_operation() {
        let program = assemble("mov r0, 5\nmov r1, 6\nmul r2, r0, r1\nhalt");
        let code = to_c(&program).unwrap();
        assert!(code.contains("*"));
    }

    #[test]
    fn test_c_logical_operations() {
        let program = assemble(
            "mov r0, 0xFF\nmov r1, 0x0F\nand r2, r0, r1\nor r3, r0, r1\nxor r4, r0, r1\nhalt",
        );
        let code = to_c(&program).unwrap();
        assert!(code.contains("&"));
        assert!(code.contains("|"));
        assert!(code.contains("^"));
    }

    #[test]
    fn test_c_branch() {
        let program = assemble("mov r0, 5\nmov r1, 10\nbeq r0, r1, skip\nhalt\nskip:\nnop");
        let code = to_c(&program).unwrap();
        assert!(code.contains("if"));
        assert!(code.contains("goto"));
    }

    #[test]
    fn test_c_file_extension() {
        let gen = CCodeGenerator::new();
        assert_eq!(gen.file_extension(), "c");
        assert_eq!(gen.target_name(), "C");
    }

    // =========================================================================
    // Go Code Generator Tests
    // =========================================================================

    #[test]
    fn test_go_simple_mov() {
        let program = assemble("mov r0, 42\nhalt");
        let code = to_go(&program).unwrap();
        assert!(code.contains("42"));
        assert!(code.contains("regs[0]"));
        assert!(code.contains("package main"));
    }

    #[test]
    fn test_go_alu_operations() {
        let program = assemble("mov r0, 10\nmov r1, 20\nadd r2, r0, r1\nhalt");
        let code = to_go(&program).unwrap();
        assert!(code.contains("+"));
    }

    #[test]
    fn test_go_sub_operation() {
        let program = assemble("mov r0, 30\nmov r1, 10\nsub r2, r0, r1\nhalt");
        let code = to_go(&program).unwrap();
        assert!(code.contains("-"));
    }

    #[test]
    fn test_go_branch() {
        let program = assemble("mov r0, 5\nmov r1, 10\nbne r0, r1, skip\nhalt\nskip:\nnop");
        let code = to_go(&program).unwrap();
        assert!(code.contains("if"));
        assert!(code.contains("goto"));
    }

    #[test]
    fn test_go_file_extension() {
        let gen = GoCodeGenerator::new();
        assert_eq!(gen.file_extension(), "go");
        assert_eq!(gen.target_name(), "Go");
    }

    // =========================================================================
    // Rust Code Generator Tests
    // =========================================================================

    #[test]
    fn test_rust_simple_mov() {
        let program = assemble("mov r0, 42\nhalt");
        let code = to_rust(&program).unwrap();
        assert!(code.contains("42"));
        assert!(code.contains("regs[0]"));
        assert!(code.contains("fn main()"));
    }

    #[test]
    fn test_rust_alu_operations() {
        let program = assemble("mov r0, 10\nmov r1, 20\nadd r2, r0, r1\nhalt");
        let code = to_rust(&program).unwrap();
        assert!(code.contains("wrapping_add"));
    }

    #[test]
    fn test_rust_sub_operation() {
        let program = assemble("mov r0, 30\nmov r1, 10\nsub r2, r0, r1\nhalt");
        let code = to_rust(&program).unwrap();
        assert!(code.contains("wrapping_sub"));
    }

    #[test]
    fn test_rust_mul_operation() {
        let program = assemble("mov r0, 5\nmov r1, 6\nmul r2, r0, r1\nhalt");
        let code = to_rust(&program).unwrap();
        assert!(code.contains("wrapping_mul"));
    }

    #[test]
    fn test_rust_branch() {
        let program = assemble("mov r0, 5\nmov r1, 10\nblt r0, r1, skip\nhalt\nskip:\nnop");
        let code = to_rust(&program).unwrap();
        assert!(code.contains("if"));
    }

    #[test]
    fn test_rust_file_extension() {
        let gen = RustCodeGenerator::new();
        assert_eq!(gen.file_extension(), "rs");
        assert_eq!(gen.target_name(), "Rust");
    }

    // =========================================================================
    // Pseudocode Generator Tests
    // =========================================================================

    #[test]
    fn test_pseudocode_simple_mov() {
        let program = assemble("mov r0, 42\nhalt");
        let code = to_pseudocode(&program).unwrap();
        assert!(code.contains("42"));
        assert!(code.contains("Set"));
    }

    #[test]
    fn test_pseudocode_alu_operations() {
        let program = assemble("mov r0, 10\nmov r1, 20\nadd r2, r0, r1\nhalt");
        let code = to_pseudocode(&program).unwrap();
        assert!(code.contains("Add"));
    }

    #[test]
    fn test_pseudocode_sub_operation() {
        let program = assemble("mov r0, 30\nmov r1, 10\nsub r2, r0, r1\nhalt");
        let code = to_pseudocode(&program).unwrap();
        assert!(code.contains("Subtract"));
    }

    #[test]
    fn test_pseudocode_mul_operation() {
        let program = assemble("mov r0, 5\nmov r1, 6\nmul r2, r0, r1\nhalt");
        let code = to_pseudocode(&program).unwrap();
        assert!(code.contains("Multiply"));
    }

    #[test]
    fn test_pseudocode_branch() {
        let program = assemble("mov r0, 5\nmov r1, 10\nbeq r0, r1, skip\nhalt\nskip:\nnop");
        let code = to_pseudocode(&program).unwrap();
        assert!(code.to_lowercase().contains("if") || code.to_lowercase().contains("equal"));
    }

    #[test]
    fn test_pseudocode_file_extension() {
        let gen = PseudocodeGenerator::new();
        assert_eq!(gen.file_extension(), "txt");
        assert_eq!(gen.target_name(), "Pseudocode");
    }

    #[test]
    fn test_pseudocode_halt() {
        let program = assemble("halt");
        let code = to_pseudocode(&program).unwrap();
        assert!(code.to_uppercase().contains("HALT"));
    }

    // =========================================================================
    // Common Utilities Tests
    // =========================================================================

    #[test]
    fn test_register_name() {
        assert_eq!(common::register_name(Register::R0), "r0");
        assert_eq!(common::register_name(Register::R15), "r15");
        assert_eq!(common::register_name(Register::Zero), "zero");
        assert_eq!(common::register_name(Register::Sp), "sp");
    }

    #[test]
    fn test_format_number() {
        assert_eq!(common::format_number(42), "42");
        assert_eq!(common::format_number(255), "255");
        assert_eq!(common::format_number(256), "0x100");
        assert_eq!(common::format_number(-1), "-1");
    }

    #[test]
    fn test_analyze_branch_targets() {
        let program = assemble("mov r0, 0\nbeq r0, r0, target\nadd r0, r0, r0\ntarget:\nhalt");
        let targets = common::analyze_branch_targets(&program);
        // Should contain target index 3 (0-based instruction)
        assert!(!targets.is_empty());
    }

    #[test]
    fn test_indent_writer() {
        let mut writer = common::IndentWriter::new();
        writer.writeln("line1");
        writer.indent();
        writer.writeln("line2");
        writer.dedent();
        writer.writeln("line3");
        let output = writer.into_output();
        assert!(output.contains("line1\n"));
        assert!(output.contains("    line2\n"));
        assert!(output.contains("line3"));
    }

    // =========================================================================
    // Edge Cases and Error Handling
    // =========================================================================

    #[test]
    fn test_empty_program_c() {
        let program = Program::new();
        let result = to_c(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_program_go() {
        let program = Program::new();
        let result = to_go(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_program_rust() {
        let program = Program::new();
        let result = to_rust(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_program_pseudocode() {
        let program = Program::new();
        let result = to_pseudocode(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_complex_program() {
        // Test a more complex program with multiple operations
        let program = assemble(
            r#"
            mov r0, 100
            mov r1, 0
        loop:
            add r1, r1, 1
            bne r1, r0, loop
            halt
        "#,
        );

        let c_code = to_c(&program).unwrap();
        assert!(c_code.contains("100"));

        let go_code = to_go(&program).unwrap();
        assert!(go_code.contains("100"));

        let rust_code = to_rust(&program).unwrap();
        assert!(rust_code.contains("100"));

        let pseudo_code = to_pseudocode(&program).unwrap();
        assert!(pseudo_code.contains("100"));
    }

    #[test]
    fn test_nop_instruction() {
        let program = assemble("nop\nhalt");

        let c_code = to_c(&program).unwrap();
        assert!(c_code.contains("nop") || c_code.contains("/* nop */") || c_code.contains(";"));

        let pseudo_code = to_pseudocode(&program).unwrap();
        assert!(
            pseudo_code.to_lowercase().contains("no operation")
                || pseudo_code.to_lowercase().contains("nop")
        );
    }
}
