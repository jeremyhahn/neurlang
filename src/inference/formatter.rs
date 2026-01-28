//! Error Formatter for Neurlang
//!
//! Converts execution errors into structured feedback for the model.
//! Provides assembly view, human-readable explanation, and suggested fix.

use crate::ir::{Disassembler, Program};

/// Execution error from running compiled code
#[derive(Debug, Clone)]
pub struct ExecError {
    /// Type of error
    pub kind: ErrorKind,
    /// Program counter (byte offset)
    pub pc: usize,
    /// Instruction index where error occurred
    pub instruction_index: usize,
    /// Additional context
    pub context: String,
}

/// Types of execution errors
#[derive(Debug, Clone)]
pub enum ErrorKind {
    /// Attempted to access memory outside capability bounds
    OutOfBounds {
        address: u64,
        cap_base: u64,
        cap_length: u32,
    },
    /// Invalid register number
    InvalidRegister { reg: u8 },
    /// Division by zero
    DivisionByZero,
    /// Invalid opcode
    InvalidOpcode { opcode: u8 },
    /// Capability violation (permission denied)
    CapabilityViolation { required: String, actual: String },
    /// Taint violation (used unsanitized data)
    TaintViolation { operation: String },
    /// Stack overflow
    StackOverflow,
    /// Stack underflow
    StackUnderflow,
    /// Invalid instruction encoding
    InvalidEncoding,
    /// I/O permission denied
    IOPermissionDenied { operation: String },
    /// Timeout
    Timeout,
    /// Generic error
    Other(String),
}

/// Structured error feedback for retry prompts
#[derive(Debug, Clone)]
pub struct ErrorFeedback {
    /// Disassembled view with error highlighted
    pub assembly: String,
    /// Human-readable explanation
    pub english: String,
    /// How to fix it
    pub suggestion: String,
    /// Severity level (1-3)
    pub severity: u8,
}

/// Error formatter that converts execution errors to feedback
pub struct ErrorFormatter {
    /// Disassembler for formatting error context (used in error display)
    #[allow(dead_code)]
    disassembler: Disassembler,
}

impl ErrorFormatter {
    pub fn new() -> Self {
        Self {
            disassembler: Disassembler::new().with_offsets(true).with_bytes(true),
        }
    }

    /// Format an execution error into structured feedback
    pub fn format(&self, error: &ExecError, program: &[u8]) -> ErrorFeedback {
        let assembly = self.highlight_line(program, error.instruction_index, error.pc);
        let (english, suggestion, severity) = self.explain_error(&error.kind);

        ErrorFeedback {
            assembly,
            english,
            suggestion,
            severity,
        }
    }

    /// Create a retry prompt combining original request with error feedback
    pub fn retry_prompt(&self, original_prompt: &str, code: &[u8], error: &ExecError) -> String {
        let feedback = self.format(error, code);
        let full_disasm = self.disassemble_with_marker(code, error.instruction_index);

        format!(
            r#"Original request: {}

Your previous attempt produced this program:
```asm
{}
```

Error on instruction {}:
```
{}
```

Problem: {}

Fix: {}

Please provide the corrected program that addresses this error."#,
            original_prompt,
            full_disasm,
            error.instruction_index,
            feedback.assembly,
            feedback.english,
            feedback.suggestion,
        )
    }

    /// Highlight the error line in the disassembly
    fn highlight_line(&self, program: &[u8], instr_index: usize, pc: usize) -> String {
        if let Some(prog) = Program::decode(program) {
            if instr_index < prog.instructions.len() {
                let instr = &prog.instructions[instr_index];
                format!("{:04x}:  {} <-- ERROR HERE", pc, instr)
            } else {
                format!("{:04x}:  <invalid instruction>", pc)
            }
        } else {
            format!("{:04x}:  <could not decode program>", pc)
        }
    }

    /// Disassemble with a marker on the error line
    fn disassemble_with_marker(&self, program: &[u8], error_index: usize) -> String {
        if let Some(prog) = Program::decode(program) {
            let mut output = String::new();
            let mut offset = 0;

            for (i, instr) in prog.instructions.iter().enumerate() {
                let marker = if i == error_index { " >>> " } else { "     " };
                output.push_str(&format!("{}{:04x}:  {}\n", marker, offset, instr));
                offset += instr.size();
            }

            output
        } else {
            "<could not decode program>".to_string()
        }
    }

    /// Explain the error in human terms
    fn explain_error(&self, kind: &ErrorKind) -> (String, String, u8) {
        match kind {
            ErrorKind::OutOfBounds {
                address,
                cap_base,
                cap_length,
            } => {
                let english =
                    format!(
                    "Attempted to access memory at address {:#x}, but the capability only allows \
                    access to addresses {:#x} to {:#x} (length {} bytes).",
                    address, cap_base, cap_base + *cap_length as u64, cap_length
                );
                let suggestion = if *address < *cap_base {
                    "The address is before the start of the buffer. Check for negative offsets or underflow."
                } else {
                    "The address exceeds the buffer bounds. Check array index calculations or allocate a larger buffer."
                };
                (english, suggestion.to_string(), 2)
            }

            ErrorKind::InvalidRegister { reg } => {
                let english = format!(
                    "Register r{} does not exist. Valid registers are r0-r15, sp, fp, lr, pc, and zero.",
                    reg
                );
                let suggestion = "Use a valid register number (0-31). Common registers: r0-r15 for general purpose, sp for stack, zero for constant 0.".to_string();
                (english, suggestion, 3)
            }

            ErrorKind::DivisionByZero => {
                let english = "Attempted to divide by zero.".to_string();
                let suggestion = "Check the divisor before dividing. Add a conditional branch to handle the zero case.".to_string();
                (english, suggestion, 2)
            }

            ErrorKind::InvalidOpcode { opcode } => {
                let english = format!(
                    "Opcode {:#x} is not a valid Neurlang opcode. Valid opcodes are 0x00-0x1F.",
                    opcode
                );
                let suggestion =
                    "Check the instruction encoding. Use a valid opcode from the 32-opcode set."
                        .to_string();
                (english, suggestion, 3)
            }

            ErrorKind::CapabilityViolation { required, actual } => {
                let english = format!(
                    "Operation requires {} permission, but the capability only has {}.",
                    required, actual
                );
                let suggestion = "Use a capability with the required permissions, or request elevated privileges.".to_string();
                (english, suggestion, 2)
            }

            ErrorKind::TaintViolation { operation } => {
                let english = format!(
                    "Used tainted (untrusted) data in operation '{}' without sanitization.",
                    operation
                );
                let suggestion = "Call 'sanitize' on the tainted value after validating it, before using it in security-sensitive operations.".to_string();
                (english, suggestion, 2)
            }

            ErrorKind::StackOverflow => {
                let english =
                    "Stack overflow: too many nested function calls or too much stack space used."
                        .to_string();
                let suggestion = "Reduce recursion depth, use tail recursion, or allocate large arrays on the heap instead of the stack.".to_string();
                (english, suggestion, 2)
            }

            ErrorKind::StackUnderflow => {
                let english =
                    "Stack underflow: attempted to pop more values than were pushed.".to_string();
                let suggestion = "Check that every pop/return has a corresponding push/call. Verify function calling convention.".to_string();
                (english, suggestion, 2)
            }

            ErrorKind::InvalidEncoding => {
                let english = "The instruction has an invalid binary encoding.".to_string();
                let suggestion = "Check the instruction format. Each instruction should be 4 or 8 bytes with valid fields.".to_string();
                (english, suggestion, 3)
            }

            ErrorKind::IOPermissionDenied { operation } => {
                let english = format!("I/O operation '{}' was denied by the sandbox.", operation);
                let suggestion = "This operation is not allowed in the current security context. Request appropriate I/O permissions.".to_string();
                (english, suggestion, 2)
            }

            ErrorKind::Timeout => {
                let english = "Execution timed out, possibly due to an infinite loop.".to_string();
                let suggestion = "Check loop termination conditions. Ensure counters are decremented and exit conditions can be reached.".to_string();
                (english, suggestion, 2)
            }

            ErrorKind::Other(msg) => {
                let english = msg.clone();
                let suggestion = "Check the program logic and try again.".to_string();
                (english, suggestion, 1)
            }
        }
    }
}

impl Default for ErrorFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Create an ExecError from various error types
impl ExecError {
    pub fn out_of_bounds(
        address: u64,
        cap_base: u64,
        cap_length: u32,
        pc: usize,
        index: usize,
    ) -> Self {
        Self {
            kind: ErrorKind::OutOfBounds {
                address,
                cap_base,
                cap_length,
            },
            pc,
            instruction_index: index,
            context: String::new(),
        }
    }

    pub fn invalid_register(reg: u8, pc: usize, index: usize) -> Self {
        Self {
            kind: ErrorKind::InvalidRegister { reg },
            pc,
            instruction_index: index,
            context: String::new(),
        }
    }

    pub fn division_by_zero(pc: usize, index: usize) -> Self {
        Self {
            kind: ErrorKind::DivisionByZero,
            pc,
            instruction_index: index,
            context: String::new(),
        }
    }

    pub fn invalid_opcode(opcode: u8, pc: usize, index: usize) -> Self {
        Self {
            kind: ErrorKind::InvalidOpcode { opcode },
            pc,
            instruction_index: index,
            context: String::new(),
        }
    }

    pub fn timeout(pc: usize, index: usize) -> Self {
        Self {
            kind: ErrorKind::Timeout,
            pc,
            instruction_index: index,
            context: String::new(),
        }
    }

    pub fn other(message: String, pc: usize, index: usize) -> Self {
        Self {
            kind: ErrorKind::Other(message),
            pc,
            instruction_index: index,
            context: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{AluOp, Instruction, Opcode, Program, Register};

    fn create_test_program() -> Vec<u8> {
        let mut program = Program::new();
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            42,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Alu,
            Register::R1,
            Register::R0,
            Register::R0,
            AluOp::Add as u8,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));
        program.encode()
    }

    #[test]
    fn test_format_out_of_bounds() {
        let formatter = ErrorFormatter::new();
        let program = create_test_program();

        let error = ExecError::out_of_bounds(0x2000, 0x1000, 256, 8, 1);
        let feedback = formatter.format(&error, &program);

        assert!(feedback.english.contains("0x2000"));
        assert!(feedback.english.contains("0x1000"));
        assert!(!feedback.suggestion.is_empty());
    }

    #[test]
    fn test_format_division_by_zero() {
        let formatter = ErrorFormatter::new();
        let program = create_test_program();

        let error = ExecError::division_by_zero(4, 0);
        let feedback = formatter.format(&error, &program);

        assert!(feedback.english.contains("divide by zero"));
        assert!(feedback.suggestion.contains("divisor"));
    }

    #[test]
    fn test_retry_prompt() {
        let formatter = ErrorFormatter::new();
        let program = create_test_program();

        let error = ExecError::division_by_zero(4, 0);
        let prompt = formatter.retry_prompt("compute 10 / 0", &program, &error);

        assert!(prompt.contains("compute 10 / 0"));
        assert!(prompt.contains("divide by zero"));
        assert!(prompt.contains("corrected program"));
    }
}
