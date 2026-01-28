//! Orchestrator for Neurlang code generation
//!
//! Handles the retry loop: prompt → model → compile → execute → retry if error.
//! The model stays simple (single-turn), while Rust handles iteration.

use std::time::{Duration, Instant};

use crate::inference::engine::{InferenceEngine, InferenceError};
use crate::inference::formatter::{ErrorFeedback, ErrorFormatter, ExecError};
use crate::ir::Program;

/// Result of orchestrated code generation
#[derive(Debug)]
pub enum OrchResult {
    /// Program executed successfully
    Success {
        /// The compiled binary
        binary: Vec<u8>,
        /// Execution result (value in R0)
        output: u64,
        /// Number of attempts taken
        attempts: usize,
        /// Total time spent
        total_time: Duration,
    },
    /// Program failed after max retries
    Failed {
        /// Last generated binary
        binary: Vec<u8>,
        /// Error feedback from last attempt
        error: ErrorFeedback,
        /// Number of attempts taken
        attempts: usize,
        /// Total time spent
        total_time: Duration,
    },
    /// Inference itself failed (model error)
    InferenceError {
        error: InferenceError,
        attempts: usize,
    },
}

/// Orchestrator configuration
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Maximum retry attempts
    pub max_retries: usize,
    /// Execution timeout per attempt
    pub exec_timeout: Duration,
    /// Total timeout for all attempts
    pub total_timeout: Duration,
    /// Whether to show verbose output
    pub verbose: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            exec_timeout: Duration::from_secs(5),
            total_timeout: Duration::from_secs(60),
            verbose: false,
        }
    }
}

/// The orchestrator manages the prompt → generate → compile → execute loop
pub struct Orchestrator {
    model: InferenceEngine,
    formatter: ErrorFormatter,
    config: OrchestratorConfig,
}

impl Orchestrator {
    /// Create a new orchestrator with the given model
    pub fn new(model: InferenceEngine) -> Self {
        Self::with_config(model, OrchestratorConfig::default())
    }

    /// Create orchestrator with custom config
    pub fn with_config(model: InferenceEngine, config: OrchestratorConfig) -> Self {
        Self {
            model,
            formatter: ErrorFormatter::new(),
            config,
        }
    }

    /// Run the orchestration loop for a user prompt
    pub fn run(&self, user_prompt: &str) -> OrchResult {
        let start_time = Instant::now();
        let mut prompt = user_prompt.to_string();
        let mut last_binary = Vec::new();
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            // Check total timeout
            if start_time.elapsed() > self.config.total_timeout {
                return self.make_failed_result(
                    last_binary,
                    last_error,
                    attempt,
                    start_time.elapsed(),
                );
            }

            if self.config.verbose {
                eprintln!("[Attempt {}] Generating code...", attempt + 1);
            }

            // Generate code from model
            let inference_result = match self.model.generate(&prompt) {
                Ok(result) => result,
                Err(e) => {
                    return OrchResult::InferenceError {
                        error: e,
                        attempts: attempt + 1,
                    };
                }
            };

            let binary = inference_result.binary_ir.clone();
            last_binary = binary.clone();

            if self.config.verbose {
                eprintln!(
                    "[Attempt {}] Generated {} bytes in {:?}",
                    attempt + 1,
                    binary.len(),
                    inference_result.latency
                );
            }

            // Try to compile and execute
            match self.compile_and_execute(&binary) {
                Ok(output) => {
                    return OrchResult::Success {
                        binary,
                        output,
                        attempts: attempt + 1,
                        total_time: start_time.elapsed(),
                    };
                }
                Err(error) => {
                    if self.config.verbose {
                        let feedback = self.formatter.format(&error, &binary);
                        eprintln!("[Attempt {}] Error: {}", attempt + 1, feedback.english);
                    }

                    // If we have retries left, format a retry prompt
                    if attempt < self.config.max_retries {
                        prompt = self.formatter.retry_prompt(user_prompt, &binary, &error);
                    }

                    last_error = Some(error);
                }
            }
        }

        self.make_failed_result(
            last_binary,
            last_error,
            self.config.max_retries + 1,
            start_time.elapsed(),
        )
    }

    /// Compile and execute a binary, returning the result or error
    fn compile_and_execute(&self, binary: &[u8]) -> Result<u64, ExecError> {
        // Decode the program
        let program = Program::decode(binary)
            .ok_or_else(|| ExecError::other("Invalid program format".to_string(), 0, 0))?;

        // For now, use the interpreter for execution
        // In production, this would use the JIT compiler
        self.interpret(&program)
    }

    /// Interpret a program (fallback when JIT unavailable)
    fn interpret(&self, program: &Program) -> Result<u64, ExecError> {
        use crate::ir::{AluOp, BranchCond, MulDivOp, Opcode, Register};

        // Simple register file
        let mut regs = [0u64; 32];
        regs[Register::Zero as usize] = 0;

        let mut pc: usize = 0;
        let mut step_count = 0u64;
        let max_steps = 1_000_000u64;

        while pc < program.instructions.len() {
            if step_count > max_steps {
                return Err(ExecError::timeout(pc * 4, pc));
            }
            step_count += 1;

            let instr = &program.instructions[pc];
            let rd = instr.rd as usize;
            let rs1 = instr.rs1 as usize;
            let rs2 = instr.rs2 as usize;

            match instr.opcode {
                Opcode::Alu => {
                    let v1 = regs[rs1];
                    let v2 = regs[rs2];
                    let result = match AluOp::from_u8(instr.mode) {
                        Some(AluOp::Add) => v1.wrapping_add(v2),
                        Some(AluOp::Sub) => v1.wrapping_sub(v2),
                        Some(AluOp::And) => v1 & v2,
                        Some(AluOp::Or) => v1 | v2,
                        Some(AluOp::Xor) => v1 ^ v2,
                        Some(AluOp::Shl) => v1 << (v2 & 63),
                        Some(AluOp::Shr) => v1 >> (v2 & 63),
                        Some(AluOp::Sar) => ((v1 as i64) >> (v2 & 63)) as u64,
                        None => return Err(ExecError::invalid_opcode(instr.mode, pc * 4, pc)),
                    };
                    if rd != Register::Zero as usize && rd != Register::Pc as usize {
                        regs[rd] = result;
                    }
                    pc += 1;
                }

                Opcode::AluI => {
                    let v1 = regs[rs1];
                    let imm = instr.imm.unwrap_or(0) as i64 as u64;
                    let result = match AluOp::from_u8(instr.mode) {
                        Some(AluOp::Add) => v1.wrapping_add(imm),
                        Some(AluOp::Sub) => v1.wrapping_sub(imm),
                        Some(AluOp::And) => v1 & imm,
                        Some(AluOp::Or) => v1 | imm,
                        Some(AluOp::Xor) => v1 ^ imm,
                        Some(AluOp::Shl) => v1 << (imm & 63),
                        Some(AluOp::Shr) => v1 >> (imm & 63),
                        Some(AluOp::Sar) => ((v1 as i64) >> (imm & 63)) as u64,
                        None => return Err(ExecError::invalid_opcode(instr.mode, pc * 4, pc)),
                    };
                    if rd != Register::Zero as usize && rd != Register::Pc as usize {
                        regs[rd] = result;
                    }
                    pc += 1;
                }

                Opcode::MulDiv => {
                    let v1 = regs[rs1];
                    let v2 = regs[rs2];
                    let result = match MulDivOp::from_u8(instr.mode) {
                        Some(MulDivOp::Mul) => v1.wrapping_mul(v2),
                        Some(MulDivOp::MulH) => {
                            let wide = (v1 as u128) * (v2 as u128);
                            (wide >> 64) as u64
                        }
                        Some(MulDivOp::Div) => {
                            if v2 == 0 {
                                return Err(ExecError::division_by_zero(pc * 4, pc));
                            }
                            v1 / v2
                        }
                        Some(MulDivOp::Mod) => {
                            if v2 == 0 {
                                return Err(ExecError::division_by_zero(pc * 4, pc));
                            }
                            v1 % v2
                        }
                        None => return Err(ExecError::invalid_opcode(instr.mode, pc * 4, pc)),
                    };
                    if rd != Register::Zero as usize && rd != Register::Pc as usize {
                        regs[rd] = result;
                    }
                    pc += 1;
                }

                Opcode::Mov => {
                    // Check if rs1 is a real register (not Zero) for register-to-register move
                    // MOV is always extended, so imm is always Some after decode
                    let value = if rs1 != Register::Zero as usize {
                        // Register-to-register move: rd = rs1
                        regs[rs1]
                    } else if let Some(imm) = instr.imm {
                        // Load immediate: rd = imm
                        imm as i64 as u64
                    } else {
                        0
                    };
                    if rd != Register::Zero as usize && rd != Register::Pc as usize {
                        regs[rd] = value;
                    }
                    pc += 1;
                }

                Opcode::Branch => {
                    let v1 = regs[rs1] as i64;
                    let v2 = regs[rs2] as i64;
                    let take_branch = match BranchCond::from_u8(instr.mode) {
                        Some(BranchCond::Always) => true,
                        Some(BranchCond::Eq) => v1 == v2,
                        Some(BranchCond::Ne) => v1 != v2,
                        Some(BranchCond::Lt) => v1 < v2,
                        Some(BranchCond::Le) => v1 <= v2,
                        Some(BranchCond::Gt) => v1 > v2,
                        Some(BranchCond::Ge) => v1 >= v2,
                        Some(BranchCond::Ltu) => (v1 as u64) < (v2 as u64),
                        None => return Err(ExecError::invalid_opcode(instr.mode, pc * 4, pc)),
                    };

                    if take_branch {
                        if let Some(offset) = instr.imm {
                            // Offset is a relative instruction index (not bytes)
                            let new_pc = (pc as i64 + offset as i64) as usize;
                            pc = new_pc;
                        } else {
                            pc += 1;
                        }
                    } else {
                        pc += 1;
                    }
                }

                Opcode::Jump => {
                    if let Some(offset) = instr.imm {
                        // Offset is a relative instruction index (not bytes)
                        let new_pc = (pc as i64 + offset as i64) as usize;
                        pc = new_pc;
                    } else {
                        // Indirect jump
                        pc = regs[rs1] as usize;
                    }
                }

                Opcode::Halt => {
                    return Ok(regs[Register::R0 as usize]);
                }

                Opcode::Nop => {
                    pc += 1;
                }

                Opcode::Ret => {
                    // In simple model, just halt
                    return Ok(regs[Register::R0 as usize]);
                }

                // For unimplemented opcodes, just skip
                _ => {
                    pc += 1;
                }
            }
        }

        // Reached end of program
        Ok(regs[Register::R0 as usize])
    }

    /// Create a failed result from the last error
    fn make_failed_result(
        &self,
        binary: Vec<u8>,
        error: Option<ExecError>,
        attempts: usize,
        total_time: Duration,
    ) -> OrchResult {
        let feedback = if let Some(e) = error {
            self.formatter.format(&e, &binary)
        } else {
            ErrorFeedback {
                assembly: String::new(),
                english: "Unknown error".to_string(),
                suggestion: "Try a different approach".to_string(),
                severity: 1,
            }
        };

        OrchResult::Failed {
            binary,
            error: feedback,
            attempts,
            total_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_success() {
        let model = InferenceEngine::mock();
        let orch = Orchestrator::new(model);

        let result = orch.run("compute fibonacci(10)");

        match result {
            OrchResult::Success {
                output: _,
                attempts,
                ..
            } => {
                assert!(attempts <= 4);
                // Mock generates fibonacci, should have some output
            }
            _ => panic!("Expected success"),
        }
    }

    #[test]
    fn test_orchestrator_simple() {
        let model = InferenceEngine::mock();
        let orch = Orchestrator::new(model);

        let result = orch.run("return 42");

        match result {
            OrchResult::Success { output, .. } => {
                assert_eq!(output, 42);
            }
            _ => panic!("Expected success"),
        }
    }

    #[test]
    fn test_orchestrator_config() {
        let config = OrchestratorConfig {
            max_retries: 1,
            verbose: true,
            ..Default::default()
        };

        let model = InferenceEngine::mock();
        let orch = Orchestrator::with_config(model, config);

        assert_eq!(orch.config.max_retries, 1);
    }
}
