//! Synthetic training data generator for Neurlang
//!
//! Generates high-quality training examples for the AI model.
//! Uses rule-based generation (no API costs) with curriculum learning.
//!
//! Supports two output formats:
//! - Legacy: binary_ir bytes (for backwards compatibility)
//! - Parallel: instruction-level data (for parallel slot prediction model)

use clap::Parser;
use neurlang::ir::{Assembler, Instruction};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufWriter, Write};

#[derive(Parser)]
#[command(name = "nl-datagen")]
#[command(about = "Generate synthetic training data for Neurlang")]
struct Args {
    /// Output file path
    #[arg(short, long, default_value = "training_data.jsonl")]
    output: String,

    /// Number of examples to generate
    #[arg(short, long, default_value = "50000")]
    num_examples: usize,

    /// Random seed for reproducibility
    #[arg(short, long, default_value = "42")]
    seed: u64,

    /// Curriculum level (1-5, higher = more complex)
    #[arg(short, long, default_value = "3")]
    curriculum_level: u8,

    /// Output format (jsonl, binary, both)
    #[arg(short, long, default_value = "jsonl")]
    format: String,

    /// Include examples from the examples/ directory
    #[arg(long, default_value = "false")]
    include_examples: bool,

    /// Path to examples directory
    #[arg(long, default_value = "../examples")]
    examples_dir: String,

    /// Output in parallel format (instruction-level data for parallel slot prediction)
    #[arg(long, default_value = "false")]
    parallel: bool,
}

/// Training example for binary IR output (legacy format)
///
/// The model learns: prompt → binary IR bytes (not assembly text)
/// This matches the architecture: model → binary IR → copy-and-patch compiler
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrainingExample {
    /// Natural language prompt
    prompt: String,
    /// Binary IR bytes as array of u8 values
    /// This is what the model learns to output directly
    binary_ir: Vec<u8>,
    /// Assembly text for debugging/visualization only (not used in training)
    #[serde(skip_serializing_if = "Option::is_none")]
    assembly: Option<String>,
    /// Expected output value (r0 after execution) - for evaluation
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_output: Option<i64>,
    /// Complexity level
    level: u8,
    /// Category
    category: String,
}

/// Instruction data for parallel slot prediction model
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct InstructionData {
    /// Whether this slot is valid (1) or padding (0)
    #[serde(default)]
    valid: u8,
    /// Opcode index (0-32)
    opcode: u8,
    /// Mode bits (0-7)
    #[serde(default)]
    mode: u8,
    /// Destination register (0-31)
    #[serde(default)]
    rd: u8,
    /// Source register 1 (0-31)
    #[serde(default)]
    rs1: u8,
    /// Source register 2 (0-31)
    #[serde(default)]
    rs2: u8,
    /// Whether instruction has immediate
    #[serde(default)]
    has_imm: u8,
    /// Immediate value bin (0-255, quantized)
    #[serde(default)]
    imm_bin: u8,
}

/// Test case for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestCaseData {
    /// Input values (register r0-r5)
    input: Vec<i64>,
    /// Expected output (r0)
    expected: i64,
}

/// Training example for parallel slot prediction model
///
/// This format supports the 64-slot parallel instruction prediction architecture.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParallelTrainingExample {
    /// Task description / prompt
    context: String,
    /// Partial IR built so far (for incremental generation)
    #[serde(default)]
    partial_ir: Vec<InstructionData>,
    /// Error feedback from previous attempt (for error correction training)
    #[serde(default)]
    error_feedback: Option<String>,
    /// Expected output instructions (up to 64)
    instructions: Vec<InstructionData>,
    /// Optional test cases for verification
    #[serde(default)]
    test_cases: Vec<TestCaseData>,
    /// Category (for analysis)
    #[serde(default)]
    category: String,
}

/// Number of instruction slots (matches model architecture)
const NUM_SLOTS: usize = 64;

/// Convert an Instruction to InstructionData
fn instruction_to_data(instr: &Instruction) -> InstructionData {
    InstructionData {
        valid: 1,
        opcode: instr.opcode as u8,
        mode: instr.mode,
        rd: instr.rd as u8,
        rs1: instr.rs1 as u8,
        rs2: instr.rs2 as u8,
        has_imm: if instr.imm.is_some() { 1 } else { 0 },
        imm_bin: quantize_immediate(instr.imm.unwrap_or(0)),
    }
}

/// Quantize immediate value to 8-bit bin
fn quantize_immediate(imm: i32) -> u8 {
    // For small values, direct mapping
    if (0..128).contains(&imm) {
        return imm as u8;
    }
    if (-128..0).contains(&imm) {
        return (256 + imm) as u8;
    }
    // For larger values, use logarithmic binning
    let magnitude = (imm.abs() as f32).log2() as i32;
    let bin = (magnitude.min(15) + 128) as u8;
    if imm < 0 {
        bin | 0x80
    } else {
        bin
    }
}

/// Program generator
struct Generator {
    rng: ChaCha8Rng,
    curriculum_level: u8,
}

impl Generator {
    fn new(seed: u64, curriculum_level: u8) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            curriculum_level,
        }
    }

    /// Pick a random prompt variation from a list
    fn pick_prompt(&mut self, variations: &[&str]) -> String {
        variations[self.rng.gen_range(0..variations.len())].to_string()
    }

    /// Format prompt with 1 value
    fn fmt1(&mut self, variations: &[&str], a: i32) -> String {
        self.pick_prompt(variations).replace("{a}", &a.to_string())
    }

    /// Format prompt with 2 values
    fn fmt2(&mut self, variations: &[&str], a: i32, b: i32) -> String {
        self.pick_prompt(variations)
            .replace("{a}", &a.to_string())
            .replace("{b}", &b.to_string())
    }

    /// Generate a training example with binary IR output
    fn generate(&mut self) -> Option<TrainingExample> {
        // Choose category based on curriculum level
        let categories = self.get_categories();
        let category = categories[self.rng.gen_range(0..categories.len())];

        // Get assembly-based example with expected output
        // Returns: (prompt, assembly, level, category, expected_output)
        let (prompt, assembly, level, category, expected_output) = match category {
            "arithmetic" => self.gen_arithmetic_raw(),
            "loops" => self.gen_loop_raw(),
            "conditionals" => self.gen_conditional_raw(),
            "memory" => self.gen_memory_raw(),
            "functions" => self.gen_function_raw(),
            "algorithms" => self.gen_algorithm_raw(),
            "concurrency" => self.gen_concurrency_raw(),
            "security" => self.gen_security_raw(),
            "io" => self.gen_io_raw(),
            "intrinsics" => self.gen_intrinsic_raw(),
            "extensions" => self.gen_extension_raw(),
            "fpu" => self.gen_fpu_raw(),
            "crypto" => self.gen_crypto_raw(),
            "stdlib" => self.gen_stdlib_raw(),
            _ => self.gen_arithmetic_raw(),
        };

        // Convert assembly to binary IR
        let binary_ir = self.to_binary_bytes(&assembly)?;

        Some(TrainingExample {
            prompt,
            binary_ir,
            assembly: Some(assembly), // Include for two-stage training (text→assembly→binary)
            expected_output,
            level,
            category,
        })
    }

    /// Generate a parallel format training example
    fn generate_parallel(&mut self) -> Option<ParallelTrainingExample> {
        // Choose category based on curriculum level
        let categories = self.get_categories();
        let category = categories[self.rng.gen_range(0..categories.len())];

        // Get assembly-based example with expected output
        let (prompt, assembly, _level, category, expected_output) = match category {
            "arithmetic" => self.gen_arithmetic_raw(),
            "loops" => self.gen_loop_raw(),
            "conditionals" => self.gen_conditional_raw(),
            "memory" => self.gen_memory_raw(),
            "functions" => self.gen_function_raw(),
            "algorithms" => self.gen_algorithm_raw(),
            "concurrency" => self.gen_concurrency_raw(),
            "security" => self.gen_security_raw(),
            "io" => self.gen_io_raw(),
            "intrinsics" => self.gen_intrinsic_raw(),
            "extensions" => self.gen_extension_raw(),
            "fpu" => self.gen_fpu_raw(),
            "crypto" => self.gen_crypto_raw(),
            "stdlib" => self.gen_stdlib_raw(),
            _ => self.gen_arithmetic_raw(),
        };

        // Convert assembly to Program
        let mut asm = Assembler::new();
        let program = asm.assemble(&assembly).ok()?;

        // Convert instructions to InstructionData format
        let mut instructions: Vec<InstructionData> = program
            .instructions
            .iter()
            .take(NUM_SLOTS)
            .map(instruction_to_data)
            .collect();

        // Pad with invalid (zero) instructions to NUM_SLOTS
        while instructions.len() < NUM_SLOTS {
            instructions.push(InstructionData::default());
        }

        // Create test case if we have expected output
        let test_cases = expected_output
            .map(|expected| {
                vec![TestCaseData {
                    input: vec![], // No input for simple examples
                    expected,
                }]
            })
            .unwrap_or_default();

        Some(ParallelTrainingExample {
            context: prompt,
            partial_ir: vec![],
            error_feedback: None,
            instructions,
            test_cases,
            category,
        })
    }

    /// Convert assembly to binary bytes
    fn to_binary_bytes(&self, assembly: &str) -> Option<Vec<u8>> {
        let mut asm = Assembler::new();
        match asm.assemble(assembly) {
            Ok(prog) => Some(prog.encode()),
            Err(_) => None,
        }
    }

    fn get_categories(&self) -> Vec<&'static str> {
        match self.curriculum_level {
            1 => vec!["arithmetic"],
            2 => vec!["arithmetic", "conditionals"],
            3 => vec![
                "arithmetic",
                "conditionals",
                "loops",
                "memory",
                "intrinsics",
                "fpu",
            ],
            4 => vec![
                "arithmetic",
                "conditionals",
                "loops",
                "memory",
                "functions",
                "algorithms",
                "intrinsics",
                "fpu",
                "stdlib",
            ],
            _ => vec![
                "arithmetic",
                "conditionals",
                "loops",
                "memory",
                "functions",
                "algorithms",
                "concurrency",
                "security",
                "io",
                "intrinsics",
                "extensions",
                "fpu",
                "crypto",
                "stdlib",
            ],
        }
    }

    /// Generate arithmetic program - returns (prompt, assembly, level, category, expected_output)
    fn gen_arithmetic_raw(&mut self) -> (String, String, u8, String, Option<i64>) {
        let ops = [
            "add", "sub", "mul", "div", "mod", "and", "or", "xor", "shl", "shr", "sar",
        ];
        let op = ops[self.rng.gen_range(0..ops.len())];

        let a: i32 = self.rng.gen_range(1..100);
        let b: i32 = self.rng.gen_range(1..100);

        // Compute expected output for each operation
        let expected: i64 = match op {
            "add" => (a + b) as i64,
            "sub" => (a - b) as i64,
            "mul" => (a * b) as i64,
            "div" => (a / b.max(1)) as i64,
            "mod" => (a % b.max(1)) as i64,
            "and" => (a & b) as i64,
            "or" => (a | b) as i64,
            "xor" => (a ^ b) as i64,
            "shl" => (a as i64) << b.min(8),
            "shr" => ((a as u32) >> b.min(8) as u32) as i64,
            "sar" => (a >> b.min(8)) as i64,
            _ => a as i64,
        };

        // Prompt variations for each operation - include bare symbolic forms
        let prompt = match op {
            "add" => self.fmt2(
                &[
                    "Add {a} and {b}",
                    "Calculate {a} + {b}",
                    "Compute {a} plus {b}",
                    "Sum {a} and {b}",
                    "What is {a} + {b}",
                    "{a} + {b}",
                    "{a}+{b}",
                    "add {a} and {b}",
                    "sum {a} {b}",
                    "{a} plus {b}",
                    "what's {a} + {b}",
                    "what's {a} plus {b}",
                ],
                a,
                b,
            ),
            "sub" => self.fmt2(
                &[
                    "Subtract {b} from {a}",
                    "Calculate {a} - {b}",
                    "Compute {a} minus {b}",
                    "{a} minus {b}",
                    "What is {a} - {b}",
                    "{a} - {b}",
                    "{a}-{b}",
                    "subtract {b} from {a}",
                    "difference of {a} and {b}",
                    "what's {a} - {b}",
                    "what's {a} minus {b}",
                ],
                a,
                b,
            ),
            "mul" => self.fmt2(
                &[
                    "Multiply {a} by {b}",
                    "Calculate {a} * {b}",
                    "Compute {a} times {b}",
                    "{a} times {b}",
                    "What is {a} * {b}",
                    "{a} * {b}",
                    "{a}*{b}",
                    "{a}x{b}",
                    "multiply {a} and {b}",
                    "{a} multiplied by {b}",
                    "product of {a} and {b}",
                    "what's {a} * {b}",
                    "what's {a} times {b}",
                ],
                a,
                b,
            ),
            "div" => self.fmt2(
                &[
                    "Divide {a} by {b}",
                    "Calculate {a} / {b}",
                    "Compute {a} divided by {b}",
                    "{a} divided by {b}",
                    "What is {a} / {b}",
                    "{a} / {b}",
                    "{a}/{b}",
                    "divide {a} by {b}",
                    "quotient of {a} and {b}",
                    "what's {a} / {b}",
                    "what's {a} divided by {b}",
                ],
                a,
                b.max(1),
            ),
            "mod" => self.fmt2(
                &[
                    "Calculate {a} modulo {b}",
                    "Compute {a} mod {b}",
                    "{a} mod {b}",
                    "What is {a} % {b}",
                    "{a} % {b}",
                    "{a}%{b}",
                    "remainder of {a} divided by {b}",
                    "compute {a} modulo {b}",
                    "what's {a} % {b}",
                    "what's {a} mod {b}",
                ],
                a,
                b.max(1),
            ),
            "and" => self.fmt2(
                &[
                    "Bitwise AND of {a} and {b}",
                    "Calculate {a} AND {b}",
                    "Compute {a} & {b}",
                    "{a} & {b}",
                    "{a}&{b}",
                    "bitwise and of {a} and {b}",
                    "{a} bitwise AND {b}",
                ],
                a,
                b,
            ),
            "or" => self.fmt2(
                &[
                    "Bitwise OR of {a} and {b}",
                    "Calculate {a} OR {b}",
                    "Compute {a} | {b}",
                    "{a} | {b}",
                    "{a}|{b}",
                    "bitwise or of {a} and {b}",
                    "{a} bitwise OR {b}",
                ],
                a,
                b,
            ),
            // NOTE: Use spaces in "{a} ^ {b}" for XOR to disambiguate from power "{a}^{b}"
            // Benchmark: "12 ^ 5" (spaces) = XOR, "2^5" (no spaces) = power
            "xor" => self.fmt2(
                &[
                    "Bitwise XOR of {a} and {b}",
                    "Calculate {a} XOR {b}",
                    "Compute {a} ^ {b}",
                    "{a} ^ {b}",
                    "bitwise xor of {a} and {b}",
                    "{a} bitwise XOR {b}",
                    "{a} xor {b}",
                ],
                a,
                b,
            ),
            "shl" => {
                let shift = b.min(8); // Limit shift amount for reasonable results
                self.fmt2(
                    &[
                        "Shift {a} left by {b} bits",
                        "{a} << {b}",
                        "{a}<<{b}",
                        "left shift {a} by {b}",
                        "shl({a}, {b})",
                        "logical left shift {a} by {b}",
                        "{a} shifted left {b}",
                    ],
                    a,
                    shift,
                )
            }
            "shr" => {
                let shift = b.min(8);
                self.fmt2(
                    &[
                        "Shift {a} right by {b} bits (logical)",
                        "{a} >> {b}",
                        "{a}>>{b}",
                        "logical right shift {a} by {b}",
                        "shr({a}, {b})",
                        "unsigned right shift {a} by {b}",
                        "{a} shifted right {b}",
                    ],
                    a,
                    shift,
                )
            }
            "sar" => {
                let shift = b.min(8);
                self.fmt2(
                    &[
                        "Arithmetic right shift {a} by {b}",
                        "signed right shift {a} by {b}",
                        "sar({a}, {b})",
                        "arithmetic shift right {a} by {b}",
                        "{a} arithmetic shift right {b}",
                    ],
                    a,
                    shift,
                )
            }
            _ => format!("Calculate {} {} {}", a, op, b),
        };

        // All arithmetic operations must leave result in r0
        let program = match op {
            "add" => format!("mov r0, {}\nmov r1, {}\nadd r0, r0, r1\nhalt", a, b),
            "sub" => format!("mov r0, {}\nmov r1, {}\nsub r0, r0, r1\nhalt", a, b),
            "mul" => format!("mov r0, {}\nmov r1, {}\nmul r0, r0, r1\nhalt", a, b),
            "div" => format!("mov r0, {}\nmov r1, {}\ndiv r0, r0, r1\nhalt", a, b.max(1)),
            "mod" => format!("mov r0, {}\nmov r1, {}\nmod r0, r0, r1\nhalt", a, b.max(1)),
            "and" => format!("mov r0, {}\nmov r1, {}\nand r0, r0, r1\nhalt", a, b),
            "or" => format!("mov r0, {}\nmov r1, {}\nor r0, r0, r1\nhalt", a, b),
            "xor" => format!("mov r0, {}\nmov r1, {}\nxor r0, r0, r1\nhalt", a, b),
            "shl" => format!("mov r0, {}\nmov r1, {}\nshl r0, r0, r1\nhalt", a, b.min(8)),
            "shr" => format!("mov r0, {}\nmov r1, {}\nshr r0, r0, r1\nhalt", a, b.min(8)),
            "sar" => format!("mov r0, {}\nmov r1, {}\nsar r0, r0, r1\nhalt", a, b.min(8)),
            _ => format!("mov r0, {}\nhalt", a),
        };

        (prompt, program, 1, "arithmetic".to_string(), Some(expected))
    }

    /// Generate loop program - returns (prompt, assembly, level, category)
    fn gen_loop_raw(&mut self) -> (String, String, u8, String, Option<i64>) {
        let n: i32 = self.rng.gen_range(5..20);
        let loop_types = ["sum", "countdown", "power", "power_special", "mulh"];
        let loop_type = loop_types[self.rng.gen_range(0..loop_types.len())];

        let (prompt, program, expected): (String, String, Option<i64>) = match loop_type {
            "sum" => {
                let prompt = self.fmt1(
                    &[
                        "Calculate the sum of 1 to {a}",
                        "Sum all numbers from 1 to {a}",
                        "Compute 1 + 2 + ... + {a}",
                        "sum of 1 to {a}",
                        "Add all integers from 1 to {a}",
                        "1+2+...+{a}",
                        "sum from 1 to {a}",
                        "what's 1+2+...+{a}",
                    ],
                    n,
                );
                // Sum from 1 to n = n * (n + 1) / 2
                let expected = (n as i64) * ((n + 1) as i64) / 2;
                (
                    prompt,
                    format!(
                        r#"mov r1, {}
mov r0, 0
mov r2, 1
loop:
    add r0, r0, r2
    addi r2, r2, 1
    bgt r2, r1, done
    b loop
done:
halt"#,
                        n
                    ),
                    Some(expected),
                )
            }
            "countdown" => {
                let prompt = self.fmt1(
                    &[
                        "Count down from {a} to 0",
                        "Countdown from {a}",
                        "Decrement from {a} to zero",
                    ],
                    n,
                );
                // Countdown ends at 0
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
loop:
    beq r0, zero, done
    subi r0, r0, 1
    b loop
done:
    halt"#,
                        n
                    ),
                    Some(0),
                )
            }
            "power" => {
                let base: i32 = self.rng.gen_range(2..10);
                let exp: i32 = self.rng.gen_range(2..8);
                let prompt = self.fmt2(
                    &[
                        "Calculate {a} to the power of {b}",
                        "{a} raised to power {b}",
                        "{a} to the {b}",
                        "Compute {a}^{b}",
                        "{a}^{b}",
                        "{a}**{b}",
                        "power of {a} to {b}",
                        "{a} ** {b}",
                        "compute {a} raised to the power {b}",
                        "calculate {a} to the power {b}",
                        "{a} to the power of {b}",
                        "power({a},{b})",
                        "power({a}, {b})",
                        "pow({a},{b})",
                        "pow({a}, {b})",
                        "what's {a}^{b}",
                        "what is {a}^{b}",
                    ],
                    base,
                    exp,
                );
                // base ^ exp
                let expected = (base as i64).pow(exp as u32);
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, {}
mov r2, 1
loop:
    beq r1, zero, done
    mul r2, r2, r0
    subi r1, r1, 1
    b loop
done:
mov r0, r2
halt"#,
                        base, exp
                    ),
                    Some(expected),
                )
            }
            "power_special" => {
                // Generate squared and cubed examples specifically
                let base: i32 = self.rng.gen_range(2..20);
                let exp_type = self.rng.gen_range(0..3);
                let (exp, prompt) = match exp_type {
                    0 => (
                        2,
                        self.fmt1(
                            &[
                                "{a} squared",
                                "square of {a}",
                                "{a}^2",
                                "compute {a} squared",
                                "{a} * {a}",
                                "{a}*{a}",
                                "{a} to the second power",
                                "what's {a} squared",
                                "what is {a}^2",
                            ],
                            base,
                        ),
                    ),
                    1 => (
                        3,
                        self.fmt1(
                            &[
                                "{a} cubed",
                                "cube of {a}",
                                "{a}^3",
                                "compute {a} cubed",
                                "{a} to the third power",
                                "what's {a} cubed",
                                "what is {a}^3",
                            ],
                            base,
                        ),
                    ),
                    _ => (
                        4,
                        self.fmt1(
                            &[
                                "{a} to the fourth",
                                "{a}^4",
                                "{a} to the fourth power",
                                "what's {a}^4",
                                "what is {a}^4",
                            ],
                            base,
                        ),
                    ),
                };
                // base ^ exp
                let expected = (base as i64).pow(exp as u32);
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, {}
mov r2, 1
loop:
    beq r1, zero, done
    mul r2, r2, r0
    subi r1, r1, 1
    b loop
done:
mov r0, r2
halt"#,
                        base, exp
                    ),
                    Some(expected),
                )
            }
            "mulh" => {
                // High multiply - upper 64 bits of 128-bit product
                // For 32-bit values, mulh is always 0
                let a: i32 = self.rng.gen_range(1000000..10000000);
                let b: i32 = self.rng.gen_range(1000000..10000000);
                let prompt = self.fmt2(
                    &[
                        "High bits of {a} * {b}",
                        "mulh({a}, {b})",
                        "Upper 64 bits of {a} times {b}",
                        "High multiply of {a} and {b}",
                        "Compute mulh({a}, {b})",
                        "Upper half of {a} * {b}",
                    ],
                    a,
                    b,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, {}
mulh r0, r0, r1
halt"#,
                        a, b
                    ),
                    Some(0), // For values that fit in 32 bits, mulh is 0
                )
            }
            _ => (
                format!("Count from 1 to {}", n),
                format!("mov r0, {}\nhalt", n),
                Some(n as i64),
            ),
        };

        (prompt, program, 3, "loops".to_string(), expected)
    }

    /// Generate conditional program - returns (prompt, assembly, level, category, expected_output)
    fn gen_conditional_raw(&mut self) -> (String, String, u8, String, Option<i64>) {
        let a: i32 = self.rng.gen_range(1..100);
        let b: i32 = self.rng.gen_range(1..100);
        let cond_types = ["max", "min", "abs", "sign"];
        let cond_type = cond_types[self.rng.gen_range(0..cond_types.len())];

        let (prompt, program, expected): (String, String, Option<i64>) = match cond_type {
            "max" => {
                let prompt = self.fmt2(
                    &[
                        "Find the maximum of {a} and {b}",
                        "Maximum of {a} and {b}",
                        "max({a}, {b})",
                        "larger of {a} and {b}",
                        "What is greater, {a} or {b}",
                        "Return the bigger of {a} and {b}",
                        "maximum of {a} and {b}",
                        "max({a},{b})",
                        "what's the max of {a} and {b}",
                    ],
                    a,
                    b,
                );
                let expected = a.max(b) as i64;
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, {}
bge r0, r1, done
mov r0, r1
done:
halt"#,
                        a, b
                    ),
                    Some(expected),
                )
            }
            "min" => {
                let prompt = self.fmt2(
                    &[
                        "Find the minimum of {a} and {b}",
                        "Minimum of {a} and {b}",
                        "min({a}, {b})",
                        "smaller of {a} and {b}",
                        "What is less, {a} or {b}",
                        "Return the smaller of {a} and {b}",
                        "minimum of {a} and {b}",
                        "min({a},{b})",
                        "what's the min of {a} and {b}",
                    ],
                    a,
                    b,
                );
                let expected = a.min(b) as i64;
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, {}
ble r0, r1, done
mov r0, r1
done:
halt"#,
                        a, b
                    ),
                    Some(expected),
                )
            }
            "abs" => {
                let val = a - 50;
                let prompt = self.fmt1(
                    &[
                        "Calculate the absolute value of {a}",
                        "Absolute value of {a}",
                        "abs({a})",
                        "|{a}|",
                        "compute abs of {a}",
                        "absolute value of {a}",
                        "what's the absolute value of {a}",
                    ],
                    val,
                );
                let expected = (val as i64).abs();
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
bge r0, zero, done
sub r0, zero, r0
done:
halt"#,
                        val
                    ),
                    Some(expected),
                )
            }
            "sign" => {
                let val = a - 50;
                let prompt = self.fmt1(
                    &[
                        "Return the sign of {a} (1, 0, or -1)",
                        "Sign of {a}",
                        "signum({a})",
                        "Is {a} positive, negative, or zero",
                        "sign of {a}",
                        "what's the sign of {a}",
                    ],
                    val,
                );
                let expected = (val as i64).signum();
                (
                    prompt,
                    format!(
                        r#"mov r1, {}
beq r1, zero, is_zero
blt r1, zero, is_negative
mov r0, 1
b done
is_negative:
mov r0, -1
b done
is_zero:
mov r0, 0
done:
halt"#,
                        val
                    ),
                    Some(expected),
                )
            }
            _ => (
                format!("Check if {} is positive", a),
                format!("mov r0, {}\nhalt", a),
                Some(a as i64),
            ),
        };

        (prompt, program, 2, "conditionals".to_string(), expected)
    }

    /// Generate memory operations program - returns (prompt, assembly, level, category)
    fn gen_memory_raw(&mut self) -> (String, String, u8, String, Option<i64>) {
        let mem_types = [
            "store_load",
            "array_sum",
            "swap",
            "copy",
            "fill",
            "load_offset",
        ];
        let mem_type = mem_types[self.rng.gen_range(0..mem_types.len())];

        let (prompt, program) = match mem_type {
            "store_load" => {
                let value: i32 = self.rng.gen_range(1..10000);
                let offset: i32 = self.rng.gen_range(0..100) * 8;
                let prompt = self.fmt2(
                    &[
                        "Store {a} at offset {b} and load it back",
                        "Write {a} to memory[{b}], then read",
                        "store.d {a} at {b}, then load.d",
                        "Memory round-trip: value={a}, offset={b}",
                    ],
                    value,
                    offset,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, {}
store.d r0, [r1]
load.d r2, [r1]
halt"#,
                        value, offset
                    ),
                )
            }
            "array_sum" => {
                let n: i32 = self.rng.gen_range(3..12);
                let prompt = self.fmt1(
                    &[
                        "Sum {a} elements from array in memory",
                        "Add up {a} memory values",
                        "Array sum of {a} elements",
                        "Sum memory[0..{a}]",
                    ],
                    n,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, 0
mov r1, 0
mov r2, {}
mov r3, 8
loop:
    mul r4, r1, r3
    load.d r5, [r4]
    add r0, r0, r5
    addi r1, r1, 1
    blt r1, r2, loop
halt"#,
                        n
                    ),
                )
            }
            "swap" => {
                let off1: i32 = self.rng.gen_range(0..50) * 8;
                let off2: i32 = off1 + 8;
                let prompt = self.fmt2(
                    &[
                        "Swap values at offset {a} and {b}",
                        "Exchange memory[{a}] and memory[{b}]",
                        "Memory swap: {a} <-> {b}",
                        "Swap elements at positions {a} and {b}",
                    ],
                    off1,
                    off2,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, {}
load.d r2, [r0]
load.d r3, [r1]
store.d r3, [r0]
store.d r2, [r1]
halt"#,
                        off1, off2
                    ),
                )
            }
            "copy" => {
                let src: i32 = self.rng.gen_range(0..50) * 8;
                let dst: i32 = self.rng.gen_range(50..100) * 8;
                let prompt = self.fmt2(
                    &[
                        "Copy value from offset {a} to {b}",
                        "Memory copy: {a} -> {b}",
                        "Read from {a}, write to {b}",
                        "Copy memory[{a}] to memory[{b}]",
                    ],
                    src,
                    dst,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, {}
load.d r2, [r0]
store.d r2, [r1]
halt"#,
                        src, dst
                    ),
                )
            }
            "fill" => {
                let value: i32 = self.rng.gen_range(0..256);
                let count: i32 = self.rng.gen_range(2..8);
                let prompt = self.fmt2(
                    &[
                        "Fill {b} memory slots with value {a}",
                        "Initialize {b} elements to {a}",
                        "Set memory[0..{b}] = {a}",
                        "Fill array: value={a}, count={b}",
                    ],
                    value,
                    count,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, 0
mov r2, {}
mov r3, 8
loop:
    mul r4, r1, r3
    store.d r0, [r4]
    addi r1, r1, 1
    blt r1, r2, loop
halt"#,
                        value, count
                    ),
                )
            }
            "load_offset" => {
                let base: i32 = self.rng.gen_range(0..100) * 8;
                let prompt = self.fmt1(
                    &[
                        "Load 64-bit value from offset {a}",
                        "Read memory at {a}",
                        "load.d from address {a}",
                        "Fetch value at offset {a}",
                    ],
                    base,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
load.d r1, [r0]
halt"#,
                        base
                    ),
                )
            }
            _ => ("Store a value".to_string(), "mov r0, 42\nhalt".to_string()),
        };

        (prompt, program, 3, "memory".to_string(), None)
    }

    /// Generate function call program - returns (prompt, assembly, level, category, expected_output)
    fn gen_function_raw(&mut self) -> (String, String, u8, String, Option<i64>) {
        let func_types = ["factorial", "fibonacci", "gcd"];
        let func_type = func_types[self.rng.gen_range(0..func_types.len())];

        // Helper functions for calculating expected outputs
        fn factorial(n: i32) -> i64 {
            if n <= 1 {
                1
            } else {
                (2..=n as i64).product()
            }
        }
        fn fibonacci(n: i32) -> i64 {
            if n <= 0 {
                0
            } else if n == 1 {
                1
            } else {
                let (mut a, mut b) = (0i64, 1i64);
                for _ in 2..=n {
                    let c = a + b;
                    a = b;
                    b = c;
                }
                b
            }
        }
        fn gcd(mut a: i32, mut b: i32) -> i64 {
            while b != 0 {
                let t = b;
                b = a % b;
                a = t;
            }
            a as i64
        }

        let (prompt, program, expected): (String, String, Option<i64>) = match func_type {
            "factorial" => {
                // Include edge cases: 20% chance of 0, 1, or 2
                let n: i32 = if self.rng.gen_bool(0.2) {
                    self.rng.gen_range(0..3) // Edge cases: 0, 1, 2
                } else {
                    self.rng.gen_range(3..10)
                };
                let prompt = self.fmt1(
                    &[
                        "Calculate factorial of {a}",
                        "Factorial of {a}",
                        "{a}!",
                        "factorial({a})",
                        "Compute {a} factorial",
                        "{a} factorial",
                        "compute factorial of {a}",
                        "what is {a}!",
                        "what's {a}!",
                        "factorial of {a}",
                    ],
                    n,
                );
                // Handle n=0 and n=1 specially (both return 1)
                let program = if n <= 1 {
                    "mov r0, 1\nhalt".to_string()
                } else {
                    format!(
                        r#"mov r0, {}
mov r1, 1
loop:
    beq r0, zero, done
    mul r1, r1, r0
    subi r0, r0, 1
    b loop
done:
mov r0, r1
halt"#,
                        n
                    )
                };
                (prompt, program, Some(factorial(n)))
            }
            "fibonacci" => {
                // Include edge cases: 20% chance of 0, 1, or 2
                let n: i32 = if self.rng.gen_bool(0.2) {
                    self.rng.gen_range(0..3) // Edge cases: 0, 1, 2
                } else {
                    self.rng.gen_range(3..15)
                };
                let prompt = self.fmt1(
                    &[
                        "Calculate fibonacci({a})",
                        "Fibonacci of {a}",
                        "fib({a})",
                        "{a}th fibonacci number",
                        "fibonacci number {a}",
                        "compute fibonacci of {a}",
                        "fibonacci sequence {a}",
                        "what is fib({a})",
                        "what's fib({a})",
                        "fibonacci({a})",
                    ],
                    n,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, 0
mov r2, 1
beq r0, zero, return_r1
subi r0, r0, 1
beq r0, zero, return_r2
loop:
    add r3, r1, r2
    mov r1, r2
    mov r2, r3
    subi r0, r0, 1
    bne r0, zero, loop
return_r2:
    mov r0, r2
    b done
return_r1:
    mov r0, r1
done:
halt"#,
                        n
                    ),
                    Some(fibonacci(n)),
                )
            }
            "gcd" => {
                let a: i32 = self.rng.gen_range(10..100);
                let b: i32 = self.rng.gen_range(10..100);
                let prompt = self.fmt2(
                    &[
                        "Calculate GCD of {a} and {b}",
                        "GCD of {a} and {b}",
                        "gcd({a}, {b})",
                        "gcd({a},{b})",
                        "Greatest common divisor of {a} and {b}",
                        "greatest common divisor of {a} and {b}",
                        "Find the GCD of {a} and {b}",
                        "gcd of {a} and {b}",
                        "compute gcd of {a} and {b}",
                        "what is gcd({a}, {b})",
                        "what's gcd({a}, {b})",
                    ],
                    a,
                    b,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, {}
loop:
    beq r1, zero, done
    mod r2, r0, r1
    mov r0, r1
    mov r1, r2
    b loop
done:
halt"#,
                        a, b
                    ),
                    Some(gcd(a, b)),
                )
            }
            _ => (
                "Calculate something".to_string(),
                "mov r0, 42\nhalt".to_string(),
                Some(42),
            ),
        };

        (prompt, program, 4, "functions".to_string(), expected)
    }

    /// Generate algorithm program - returns (prompt, assembly, level, category)
    fn gen_algorithm_raw(&mut self) -> (String, String, u8, String, Option<i64>) {
        let algo_types = [
            "linear_search",
            "bubble_sort_step",
            "is_prime",
            "sum_array",
            "find_max",
            "count_matches",
        ];
        let algo_type = algo_types[self.rng.gen_range(0..algo_types.len())];

        let (prompt, program) = match algo_type {
            "linear_search" => {
                let target: i32 = self.rng.gen_range(1..100);
                let size: i32 = self.rng.gen_range(5..20);
                let prompt = self.fmt2(
                    &[
                        "Linear search for {a} in array of {b} elements",
                        "Find {a} in {b}-element array",
                        "Search for value {a} in array (size {b})",
                        "Linear search: target={a}, size={b}",
                    ],
                    target,
                    size,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, 0
mov r2, {}
mov r3, 8
loop:
    bge r1, r2, not_found
    mul r4, r1, r3
    load.d r5, [r4]
    beq r5, r0, found
    addi r1, r1, 1
    b loop
found:
    mov r0, r1
    b done
not_found:
    mov r0, -1
done:
halt"#,
                        target, size
                    ),
                )
            }
            "bubble_sort_step" => {
                let offset: i32 = self.rng.gen_range(0..10) * 8;
                let prompt = self.fmt1(
                    &[
                        "Bubble sort step at offset {a}",
                        "Compare and swap adjacent elements at {a}",
                        "Sort step: compare elements at {a} and {a}+8",
                        "Bubble sort iteration at position {a}",
                    ],
                    offset,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
addi r1, r0, 8
load.d r2, [r0]
load.d r3, [r1]
ble r2, r3, no_swap
store.d r3, [r0]
store.d r2, [r1]
no_swap:
halt"#,
                        offset
                    ),
                )
            }
            "is_prime" => {
                let n: i32 = self.rng.gen_range(2..200);
                let prompt = self.fmt1(
                    &[
                        "Check if {a} is prime",
                        "Is {a} a prime number",
                        "Primality test for {a}",
                        "is_prime({a})",
                        "Test if {a} is prime",
                    ],
                    n,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, 2
mov r2, 0
mov r3, 2
blt r0, r3, not_prime
loop:
    mul r4, r1, r1
    bgt r4, r0, is_prime
    mod r5, r0, r1
    beq r5, zero, not_prime
    addi r1, r1, 1
    b loop
not_prime:
    mov r2, 1
    b done
is_prime:
    mov r2, 0
done:
halt"#,
                        n
                    ),
                )
            }
            "sum_array" => {
                let size: i32 = self.rng.gen_range(3..15);
                let prompt = self.fmt1(
                    &[
                        "Sum all elements in {a}-element array",
                        "Array sum (size {a})",
                        "Add up {a} array elements",
                        "Calculate sum of {a} values in memory",
                    ],
                    size,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, 0
mov r1, 0
mov r2, {}
mov r3, 8
loop:
    bge r1, r2, done
    mul r4, r1, r3
    load.d r5, [r4]
    add r0, r0, r5
    addi r1, r1, 1
    b loop
done:
halt"#,
                        size
                    ),
                )
            }
            "find_max" => {
                let size: i32 = self.rng.gen_range(3..15);
                let prompt = self.fmt1(
                    &[
                        "Find maximum in {a}-element array",
                        "Array max (size {a})",
                        "Get largest of {a} array elements",
                        "Find max value in {a}-element array",
                    ],
                    size,
                );
                (
                    prompt,
                    format!(
                        r#"load.d r0, [zero]
mov r1, 1
mov r2, {}
mov r3, 8
loop:
    bge r1, r2, done
    mul r4, r1, r3
    load.d r5, [r4]
    ble r5, r0, skip
    mov r0, r5
skip:
    addi r1, r1, 1
    b loop
done:
halt"#,
                        size
                    ),
                )
            }
            "count_matches" => {
                let target: i32 = self.rng.gen_range(0..50);
                let size: i32 = self.rng.gen_range(5..20);
                let prompt = self.fmt2(
                    &[
                        "Count occurrences of {a} in {b}-element array",
                        "How many times does {a} appear in array of {b}",
                        "Count {a} in array (size {b})",
                        "Array count: value={a}, size={b}",
                    ],
                    target,
                    size,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, 0
mov r2, {}
mov r3, 8
mov r6, 0
loop:
    bge r1, r2, done
    mul r4, r1, r3
    load.d r5, [r4]
    bne r5, r0, skip
    addi r6, r6, 1
skip:
    addi r1, r1, 1
    b loop
done:
mov r0, r6
halt"#,
                        target, size
                    ),
                )
            }
            _ => (
                "Perform some algorithm".to_string(),
                "mov r0, 42\nhalt".to_string(),
            ),
        };

        (prompt, program, 4, "algorithms".to_string(), None)
    }

    /// Generate concurrency program - returns (prompt, assembly, level, category)
    fn gen_concurrency_raw(&mut self) -> (String, String, u8, String, Option<i64>) {
        let conc_types = [
            "atomic_inc",
            "atomic_add",
            "channel_send",
            "fence_release",
            "fence_acquire",
            "atomic_xchg",
            "atomic_cas",
            "atomic_and",
            "atomic_or",
            "atomic_xor",
            "atomic_min",
            "atomic_max",
            "spawn",
            "join",
            "yield",
            "chan_close",
        ];
        let conc_type = conc_types[self.rng.gen_range(0..conc_types.len())];

        let (prompt, program) = match conc_type {
            "atomic_inc" => {
                let prompt = self.pick_prompt(&[
                    "Atomically increment a counter",
                    "Atomic increment by 1",
                    "atomic.add with value 1",
                    "Increment counter atomically",
                    "Thread-safe increment",
                ]);
                (
                    prompt,
                    r#"mov r0, 0
mov r1, 1
atomic.add r2, r0, r1
halt"#
                        .to_string(),
                )
            }
            "atomic_add" => {
                let val: i32 = self.rng.gen_range(2..100);
                let prompt = self.fmt1(
                    &[
                        "Atomically add {a} to a counter",
                        "atomic.add with value {a}",
                        "Add {a} atomically",
                        "Thread-safe add {a}",
                    ],
                    val,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, 0
mov r1, {}
atomic.add r2, r0, r1
halt"#,
                        val
                    ),
                )
            }
            "channel_send" => {
                let val: i32 = self.rng.gen_range(1..1000);
                let prompt = self.fmt1(
                    &[
                        "Create a channel and send value {a}",
                        "Channel send/recv with value {a}",
                        "Send {a} through channel",
                        "chan.create then send {a}",
                    ],
                    val,
                );
                (
                    prompt,
                    format!(
                        r#"chan.create r0
mov r1, {}
send r0, r1
recv r2, r0
halt"#,
                        val
                    ),
                )
            }
            "fence_release" => {
                let val: i32 = self.rng.gen_range(1..1000);
                let prompt = self.fmt1(
                    &[
                        "Store {a} with release fence",
                        "Release fence after storing {a}",
                        "fence.release with value {a}",
                        "Memory barrier (release) storing {a}",
                    ],
                    val,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
store.d r0, [zero]
fence.release
halt"#,
                        val
                    ),
                )
            }
            "fence_acquire" => {
                let prompt = self.pick_prompt(&[
                    "Load with acquire fence",
                    "Acquire fence before load",
                    "fence.acquire then load",
                    "Memory barrier (acquire) before read",
                ]);
                (
                    prompt,
                    r#"fence.acquire
load.d r0, [zero]
halt"#
                        .to_string(),
                )
            }
            "atomic_xchg" => {
                let val: i32 = self.rng.gen_range(1..1000);
                let prompt = self.fmt1(
                    &[
                        "Atomic exchange with value {a}",
                        "atomic.xchg setting {a}",
                        "Swap atomically with {a}",
                        "Exchange value {a} atomically",
                    ],
                    val,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, 0
mov r1, {}
atomic.xchg r2, r0, r1
halt"#,
                        val
                    ),
                )
            }
            "atomic_cas" => {
                let expected: i32 = self.rng.gen_range(1..100);
                let new_val: i32 = self.rng.gen_range(100..200);
                let prompt = self.fmt2(
                    &[
                        "Compare and swap: if memory equals {a}, set to {b}",
                        "atomic.cas expected={a} new={b}",
                        "CAS operation: compare {a}, swap {b}",
                        "Atomic compare-and-swap {a} with {b}",
                    ],
                    expected,
                    new_val,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, 0
mov r1, {}
mov r2, {}
atomic.cas r3, r0, r1, r2
halt"#,
                        expected, new_val
                    ),
                )
            }
            "atomic_and" => {
                let mask: i32 = self.rng.gen_range(1..255);
                let prompt = self.fmt1(
                    &[
                        "Atomic AND with mask {a}",
                        "atomic.and with {a}",
                        "Atomically AND memory with {a}",
                        "Thread-safe bitwise AND with {a}",
                    ],
                    mask,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, 0
mov r1, {}
atomic.and r2, r0, r1
halt"#,
                        mask
                    ),
                )
            }
            "atomic_or" => {
                let mask: i32 = self.rng.gen_range(1..255);
                let prompt = self.fmt1(
                    &[
                        "Atomic OR with mask {a}",
                        "atomic.or with {a}",
                        "Atomically OR memory with {a}",
                        "Thread-safe bitwise OR with {a}",
                    ],
                    mask,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, 0
mov r1, {}
atomic.or r2, r0, r1
halt"#,
                        mask
                    ),
                )
            }
            "atomic_xor" => {
                let mask: i32 = self.rng.gen_range(1..255);
                let prompt = self.fmt1(
                    &[
                        "Atomic XOR with mask {a}",
                        "atomic.xor with {a}",
                        "Atomically XOR memory with {a}",
                        "Thread-safe bitwise XOR with {a}",
                    ],
                    mask,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, 0
mov r1, {}
atomic.xor r2, r0, r1
halt"#,
                        mask
                    ),
                )
            }
            "atomic_min" => {
                let val: i32 = self.rng.gen_range(1..1000);
                let prompt = self.fmt1(
                    &[
                        "Atomic minimum with value {a}",
                        "atomic.min with {a}",
                        "Atomically update to minimum of current and {a}",
                        "Thread-safe min with {a}",
                    ],
                    val,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, 0
mov r1, {}
atomic.min r2, r0, r1
halt"#,
                        val
                    ),
                )
            }
            "atomic_max" => {
                let val: i32 = self.rng.gen_range(1..1000);
                let prompt = self.fmt1(
                    &[
                        "Atomic maximum with value {a}",
                        "atomic.max with {a}",
                        "Atomically update to maximum of current and {a}",
                        "Thread-safe max with {a}",
                    ],
                    val,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, 0
mov r1, {}
atomic.max r2, r0, r1
halt"#,
                        val
                    ),
                )
            }
            "spawn" => {
                let prompt = self.pick_prompt(&[
                    "Spawn a new task",
                    "Create a new thread",
                    "spawn task",
                    "Start parallel task",
                    "Fork a task",
                ]);
                (
                    prompt,
                    r#"mov r1, 16
spawn r0, r1
halt"#
                        .to_string(),
                )
            }
            "join" => {
                let prompt = self.pick_prompt(&[
                    "Wait for task to complete",
                    "Join spawned task",
                    "Wait for thread",
                    "join task",
                    "Synchronize with task",
                ]);
                (
                    prompt,
                    r#"mov r1, 16
spawn r0, r1
join r0
halt"#
                        .to_string(),
                )
            }
            "yield" => {
                let prompt = self.pick_prompt(&[
                    "Yield to other tasks",
                    "Cooperative yield",
                    "yield",
                    "Give up CPU",
                    "Let other tasks run",
                ]);
                (
                    prompt,
                    r#"yield
halt"#
                        .to_string(),
                )
            }
            "chan_close" => {
                let prompt = self.pick_prompt(&[
                    "Create and close a channel",
                    "Close channel after use",
                    "chan.close",
                    "Cleanup channel",
                ]);
                (
                    prompt,
                    r#"chan.create r0
chan.close r0
halt"#
                        .to_string(),
                )
            }
            _ => (
                "Concurrent operation".to_string(),
                "mov r0, 42\nhalt".to_string(),
            ),
        };

        (prompt, program, 5, "concurrency".to_string(), None)
    }

    /// Generate security-related program - returns (prompt, assembly, level, category)
    fn gen_security_raw(&mut self) -> (String, String, u8, String, Option<i64>) {
        let sec_types = [
            "taint_check",
            "capability_restrict",
            "bounds_check",
            "capability_new",
            "sanitize",
        ];
        let sec_type = sec_types[self.rng.gen_range(0..sec_types.len())];

        let (prompt, program) = match sec_type {
            "taint_check" => {
                let offset: i32 = self.rng.gen_range(0..100) * 8;
                let prompt = self.fmt1(
                    &[
                        "Mark data at offset {a} as tainted and sanitize",
                        "Taint check on memory at {a}",
                        "Load from {a}, taint, then sanitize",
                        "Apply taint tracking to data at {a}",
                    ],
                    offset,
                );
                (
                    prompt,
                    format!(
                        r#"mov r1, {}
load.d r0, [r1]
taint r0
sanitize r0
halt"#,
                        offset
                    ),
                )
            }
            "capability_restrict" => {
                let perms: i32 = self.rng.gen_range(1..8);
                let prompt = self.fmt1(
                    &[
                        "Restrict capability permissions to {a}",
                        "cap.restrict with perms {a}",
                        "Narrow capability to permission level {a}",
                        "Set capability perms to {a}",
                    ],
                    perms,
                );
                (
                    prompt,
                    format!(
                        r#"cap.new r0, r1, r2
mov r3, {}
cap.restrict r0, r0, r3
cap.query r1, r0
halt"#,
                        perms
                    ),
                )
            }
            "bounds_check" => {
                let size: i32 = self.rng.gen_range(64..512);
                let offset: i32 = self.rng.gen_range(0..size / 2);
                let prompt = self.fmt2(
                    &[
                        "Create {a}-byte capability and load at offset {b}",
                        "Bounds-checked load: {a} byte buffer, offset {b}",
                        "cap.new size {a}, load at {b}",
                        "Safe array access: size {a}, index {b}",
                    ],
                    size,
                    offset,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, 0
mov r1, {}
cap.new r3, r0, r1
mov r4, {}
load.d r5, [r4]
halt"#,
                        size, offset
                    ),
                )
            }
            "capability_new" => {
                let base: i32 = self.rng.gen_range(0..1000) * 8;
                let size: i32 = self.rng.gen_range(8..1024);
                let prompt = self.fmt2(
                    &[
                        "Create capability at base {a} with size {b}",
                        "cap.new base={a} length={b}",
                        "New capability: start {a}, size {b}",
                        "Allocate {b}-byte capability at {a}",
                    ],
                    base,
                    size,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, {}
cap.new r2, r0, r1
halt"#,
                        base, size
                    ),
                )
            }
            "sanitize" => {
                let prompt = self.pick_prompt(&[
                    "Sanitize user input before use",
                    "Remove taint from validated data",
                    "Mark data as safe after validation",
                    "Clear taint flag on register",
                    "sanitize after input validation",
                ]);
                (
                    prompt,
                    r#"load.d r0, [zero]
taint r0
sanitize r0
halt"#
                        .to_string(),
                )
            }
            _ => (
                "Security operation".to_string(),
                "mov r0, 42\nhalt".to_string(),
            ),
        };

        (prompt, program, 5, "security".to_string(), None)
    }

    /// Generate I/O programs - returns (prompt, assembly, level, category)
    /// Includes console I/O, file operations, and networking
    fn gen_io_raw(&mut self) -> (String, String, u8, String, Option<i64>) {
        let io_types = [
            // Console I/O
            "print",
            "time",
            "read_input",
            "sleep",
            "get_args",
            "get_env",
            // File operations
            "file_open",
            "file_read",
            "file_write",
            "file_seek",
            "file_stat",
            "file_mkdir",
            "file_delete",
            // Networking
            "net_socket",
            "net_connect",
            "net_bind",
            "net_listen",
            "net_accept",
            "net_send",
            "net_recv",
            "net_close",
        ];
        let io_type = io_types[self.rng.gen_range(0..io_types.len())];

        let (prompt, program) = match io_type {
            "print" => {
                let len: i32 = self.rng.gen_range(5..50);
                let prompt = self.fmt1(
                    &[
                        "Print a message of {a} bytes to console",
                        "Output {a} characters to stdout",
                        "Write {a} bytes to console",
                        "Display {a} bytes on screen",
                        "Print {a} byte message",
                        "io.print with length {a}",
                    ],
                    len,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, 0
mov r1, {}
io.print r0, r1
halt"#,
                        len
                    ),
                )
            }
            "time" => {
                let prompt = self.pick_prompt(&[
                    "Get the current time",
                    "Get current Unix timestamp",
                    "Read system time",
                    "Get time now",
                    "time.now",
                    "What time is it",
                    "Current timestamp",
                ]);
                (
                    prompt,
                    r#"time.now r0
halt"#
                        .to_string(),
                )
            }
            "read_input" => {
                let max_len: i32 = self.rng.gen_range(32..256);
                let prompt = self.fmt1(
                    &[
                        "Read up to {a} bytes from stdin",
                        "Read input line (max {a} chars)",
                        "io.read_line with buffer size {a}",
                        "Get user input (max {a} bytes)",
                    ],
                    max_len,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, 0
mov r1, {}
io.read_line r2, r0, r1
halt"#,
                        max_len
                    ),
                )
            }
            "sleep" => {
                let ms: i32 = self.rng.gen_range(10..1000);
                let prompt = self.fmt1(
                    &[
                        "Sleep for {a} milliseconds",
                        "Wait {a}ms",
                        "Pause for {a} milliseconds",
                        "time.sleep {a}",
                        "Delay {a}ms",
                    ],
                    ms,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
time.sleep r0
halt"#,
                        ms
                    ),
                )
            }
            // Extended console I/O
            "get_args" => {
                let prompt = self.pick_prompt(&[
                    "Get command line arguments",
                    "Get program arguments",
                    "io.get_args",
                    "Access argv",
                    "Read CLI args",
                ]);
                (
                    prompt,
                    r#"io.get_args r0, r1
halt"#
                        .to_string(),
                )
            }
            "get_env" => {
                let prompt = self.pick_prompt(&[
                    "Get environment variable",
                    "Read env var",
                    "io.get_env",
                    "Access environment",
                    "Get env value",
                ]);
                (
                    prompt,
                    r#"mov r0, 0
mov r1, 4
io.get_env r2, r0, r1
halt"#
                        .to_string(),
                )
            }
            // File operations
            "file_open" => {
                let flags: i32 = self.rng.gen_range(0..4);
                let prompt = self.fmt1(
                    &[
                        "Open a file with flags {a}",
                        "File open with mode {a}",
                        "file.open with flags {a}",
                        "Open file (flags={a})",
                    ],
                    flags,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, 0
mov r1, 8
file.open r2, r0, r1, {}
halt"#,
                        flags
                    ),
                )
            }
            "file_read" => {
                let len: i32 = self.rng.gen_range(64..1024);
                let prompt = self.fmt1(
                    &[
                        "Read {a} bytes from file",
                        "file.read with length {a}",
                        "Read {a} bytes from fd",
                        "File read operation ({a} bytes)",
                    ],
                    len,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, 0
mov r1, 8
file.open r2, r0, r1, 0
mov r3, 256
mov r4, {}
file.read r5, r2, r3, r4
file.close r0, r2
halt"#,
                        len
                    ),
                )
            }
            "file_write" => {
                let len: i32 = self.rng.gen_range(16..256);
                let prompt = self.fmt1(
                    &[
                        "Write {a} bytes to file",
                        "file.write with length {a}",
                        "Write {a} bytes to fd",
                        "File write operation ({a} bytes)",
                    ],
                    len,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, 0
mov r1, 8
file.open r2, r0, r1, 1
mov r3, 256
mov r4, {}
file.write r5, r2, r3, r4
file.close r0, r2
halt"#,
                        len
                    ),
                )
            }
            "file_seek" => {
                let offset: i32 = self.rng.gen_range(0..1024);
                let whence: i32 = self.rng.gen_range(0..3);
                let prompt = self.fmt2(
                    &[
                        "Seek to offset {a} with whence {b}",
                        "file.seek offset={a} whence={b}",
                        "Move file position to {a} (mode {b})",
                    ],
                    offset,
                    whence,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, 0
mov r1, 8
file.open r2, r0, r1, 0
mov r3, {}
file.seek r4, r2, r3, {}
file.close r0, r2
halt"#,
                        offset, whence
                    ),
                )
            }
            "file_stat" => {
                let prompt = self.pick_prompt(&[
                    "Get file status/metadata",
                    "file.stat",
                    "Get file size and mtime",
                    "Query file info",
                ]);
                (
                    prompt,
                    r#"mov r0, 0
mov r1, 8
file.stat r2, r0, r1
halt"#
                        .to_string(),
                )
            }
            "file_mkdir" => {
                let prompt = self.pick_prompt(&[
                    "Create a directory",
                    "file.mkdir",
                    "Make directory",
                    "Create folder",
                ]);
                (
                    prompt,
                    r#"mov r0, 0
mov r1, 8
file.mkdir r2, r0, r1
halt"#
                        .to_string(),
                )
            }
            "file_delete" => {
                let prompt = self.pick_prompt(&[
                    "Delete a file",
                    "file.delete",
                    "Remove file",
                    "Unlink file",
                ]);
                (
                    prompt,
                    r#"mov r0, 0
mov r1, 8
file.delete r2, r0, r1
halt"#
                        .to_string(),
                )
            }
            // Networking operations
            "net_socket" => {
                let domain: i32 = self.rng.gen_range(0..3); // AF_INET, AF_INET6, AF_UNIX
                let sock_type: i32 = self.rng.gen_range(0..2); // SOCK_STREAM, SOCK_DGRAM
                let prompt = self.fmt2(
                    &[
                        "Create socket with domain {a} type {b}",
                        "net.socket domain={a} type={b}",
                        "Open network socket (domain {a}, type {b})",
                        "Create TCP/UDP socket",
                    ],
                    domain,
                    sock_type,
                );
                (
                    prompt,
                    format!(
                        r#"mov r1, {}
mov r2, {}
net.socket r0, r1, r2
halt"#,
                        domain, sock_type
                    ),
                )
            }
            "net_connect" => {
                let port: i32 = self.rng.gen_range(80..9000);
                let prompt = self.fmt1(
                    &[
                        "Connect socket to port {a}",
                        "net.connect to port {a}",
                        "Connect to server on port {a}",
                        "Establish connection to port {a}",
                    ],
                    port,
                );
                (
                    prompt,
                    format!(
                        r#"mov r1, 2
mov r2, 1
net.socket r0, r1, r2
mov r3, 0
mov r4, {}
net.connect r5, r0, r3, r4
halt"#,
                        port
                    ),
                )
            }
            "net_bind" => {
                let port: i32 = self.rng.gen_range(1024..65535);
                let prompt = self.fmt1(
                    &[
                        "Bind socket to port {a}",
                        "net.bind to port {a}",
                        "Bind server to port {a}",
                        "Listen on port {a}",
                    ],
                    port,
                );
                (
                    prompt,
                    format!(
                        r#"mov r1, 2
mov r2, 1
net.socket r0, r1, r2
mov r3, 0
mov r4, {}
net.bind r5, r0, r3, r4
halt"#,
                        port
                    ),
                )
            }
            "net_listen" => {
                let backlog: i32 = self.rng.gen_range(1..128);
                let port: i32 = self.rng.gen_range(1024..65535);
                let prompt = self.fmt2(
                    &[
                        "Listen on port with backlog {b}",
                        "net.listen with backlog {b}",
                        "Start server (backlog {b})",
                        "Accept connections (queue {b})",
                    ],
                    port,
                    backlog,
                );
                (
                    prompt,
                    format!(
                        r#"mov r1, 2
mov r2, 1
net.socket r0, r1, r2
mov r3, 0
mov r4, {}
net.bind r5, r0, r3, r4
mov r6, {}
net.listen r7, r0, r6
halt"#,
                        port, backlog
                    ),
                )
            }
            "net_accept" => {
                let port: i32 = self.rng.gen_range(1024..65535);
                let prompt = self.fmt1(
                    &[
                        "Accept incoming connection on port {a}",
                        "net.accept on port {a}",
                        "Wait for client connection",
                        "Accept client socket",
                    ],
                    port,
                );
                (
                    prompt,
                    format!(
                        r#"mov r1, 2
mov r2, 1
net.socket r0, r1, r2
mov r3, 0
mov r4, {}
net.bind r5, r0, r3, r4
mov r6, 10
net.listen r7, r0, r6
net.accept r8, r0
halt"#,
                        port
                    ),
                )
            }
            "net_send" => {
                let len: i32 = self.rng.gen_range(16..1024);
                let prompt = self.fmt1(
                    &[
                        "Send {a} bytes over socket",
                        "net.send {a} bytes",
                        "Transmit {a} bytes to peer",
                        "Write {a} bytes to socket",
                    ],
                    len,
                );
                (
                    prompt,
                    format!(
                        r#"mov r1, 2
mov r2, 1
net.socket r0, r1, r2
mov r3, 256
mov r4, {}
nsend r5, r0, r3, r4
halt"#,
                        len
                    ),
                )
            }
            "net_recv" => {
                let len: i32 = self.rng.gen_range(64..2048);
                let prompt = self.fmt1(
                    &[
                        "Receive up to {a} bytes from socket",
                        "net.recv max {a} bytes",
                        "Read {a} bytes from socket",
                        "Receive data ({a} byte buffer)",
                    ],
                    len,
                );
                (
                    prompt,
                    format!(
                        r#"mov r1, 2
mov r2, 1
net.socket r0, r1, r2
mov r3, 256
mov r4, {}
nrecv r5, r0, r3, r4
halt"#,
                        len
                    ),
                )
            }
            "net_close" => {
                let prompt = self.pick_prompt(&[
                    "Close network socket",
                    "net.close socket",
                    "Disconnect socket",
                    "Shutdown connection",
                ]);
                (
                    prompt,
                    r#"mov r1, 2
mov r2, 1
net.socket r0, r1, r2
net.close r3, r0
halt"#
                        .to_string(),
                )
            }
            _ => ("I/O operation".to_string(), "mov r0, 42\nhalt".to_string()),
        };

        (prompt, program, 5, "io".to_string(), None)
    }

    /// Generate intrinsic usage examples - returns (prompt, assembly, level, category, expected_output)
    /// Simplified to basic operations that can be assembled
    fn gen_intrinsic_raw(&mut self) -> (String, String, u8, String, Option<i64>) {
        let intrinsic_types = ["abs", "min", "max", "popcount", "clz", "ctz", "bswap"];
        let intrinsic_type = intrinsic_types[self.rng.gen_range(0..intrinsic_types.len())];

        let (prompt, program, expected): (String, String, Option<i64>) = match intrinsic_type {
            "abs" => {
                let val: i32 = self.rng.gen_range(-100..100);
                let prompt = self.fmt1(
                    &[
                        "Calculate absolute value of {a}",
                        "abs({a})",
                        "|{a}|",
                        "absolute value of {a}",
                    ],
                    val,
                );
                let expected = (val as i64).abs();
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
bge r0, zero, done
sub r0, zero, r0
done:
halt"#,
                        val
                    ),
                    Some(expected),
                )
            }
            "min" => {
                let a: i32 = self.rng.gen_range(1..100);
                let b: i32 = self.rng.gen_range(1..100);
                let prompt = self.fmt2(
                    &[
                        "Find minimum of {a} and {b}",
                        "min({a}, {b})",
                        "smaller of {a} and {b}",
                        "minimum of {a} and {b}",
                    ],
                    a,
                    b,
                );
                let expected = a.min(b) as i64;
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, {}
ble r0, r1, done
mov r0, r1
done:
halt"#,
                        a, b
                    ),
                    Some(expected),
                )
            }
            "max" => {
                let a: i32 = self.rng.gen_range(1..100);
                let b: i32 = self.rng.gen_range(1..100);
                let prompt = self.fmt2(
                    &[
                        "Find maximum of {a} and {b}",
                        "max({a}, {b})",
                        "larger of {a} and {b}",
                        "maximum of {a} and {b}",
                    ],
                    a,
                    b,
                );
                let expected = a.max(b) as i64;
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, {}
bge r0, r1, done
mov r0, r1
done:
halt"#,
                        a, b
                    ),
                    Some(expected),
                )
            }
            "popcount" => {
                let val: i32 = self.rng.gen_range(0..255);
                let prompt = self.fmt1(
                    &[
                        "Count set bits in {a}",
                        "popcount({a})",
                        "Number of 1 bits in {a}",
                        "bit count of {a}",
                    ],
                    val,
                );
                let expected = (val as u32).count_ones() as i64;
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
bits.popcount r0, r0
halt"#,
                        val
                    ),
                    Some(expected),
                )
            }
            "clz" => {
                let val: i32 = self.rng.gen_range(1..255);
                let prompt = self.fmt1(
                    &[
                        "Count leading zeros in {a}",
                        "clz({a})",
                        "leading zeros of {a}",
                    ],
                    val,
                );
                // 64-bit CLZ for the value
                let expected = (val as u64).leading_zeros() as i64;
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
bits.clz r0, r0
halt"#,
                        val
                    ),
                    Some(expected),
                )
            }
            "ctz" => {
                let val: i32 = self.rng.gen_range(1..255);
                let prompt = self.fmt1(
                    &[
                        "Count trailing zeros in {a}",
                        "ctz({a})",
                        "trailing zeros of {a}",
                        "number of trailing zero bits in {a}",
                    ],
                    val,
                );
                let expected = (val as u64).trailing_zeros() as i64;
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
bits.ctz r0, r0
halt"#,
                        val
                    ),
                    Some(expected),
                )
            }
            "bswap" => {
                let val: i32 = self.rng.gen_range(0..65535);
                let prompt = self.fmt1(
                    &["Byte swap {a}", "bswap({a})", "Reverse bytes of {a}"],
                    val,
                );
                let expected = (val as u64).swap_bytes() as i64;
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
bits.bswap r0, r0
halt"#,
                        val
                    ),
                    Some(expected),
                )
            }
            _ => (
                "Use an intrinsic".to_string(),
                "mov r0, 42\nhalt".to_string(),
                Some(42),
            ),
        };

        (prompt, program, 3, "intrinsics".to_string(), expected)
    }

    /// Generate extension call examples - returns (prompt, assembly, level, category)
    /// Simplified to basic operations that can be assembled
    fn gen_extension_raw(&mut self) -> (String, String, u8, String, Option<i64>) {
        let ext_types = ["random_u64", "random_bytes", "rand_range"];
        let ext_type = ext_types[self.rng.gen_range(0..ext_types.len())];

        let (prompt, program) = match ext_type {
            "random_u64" => {
                let prompt = self.pick_prompt(&[
                    "Generate a random 64-bit number",
                    "Random u64",
                    "rand.u64",
                    "Get random number",
                    "Generate random integer",
                    "Random 64-bit value",
                    "Secure random number",
                ]);
                (
                    prompt,
                    r#"rand.u64 r0
halt"#
                        .to_string(),
                )
            }
            "random_bytes" => {
                let len: i32 = self.rng.gen_range(8..128);
                let prompt = self.fmt1(
                    &[
                        "Generate {a} random bytes",
                        "rand.bytes with length {a}",
                        "Fill buffer with {a} random bytes",
                        "Random {a}-byte buffer",
                        "Get {a} bytes of randomness",
                    ],
                    len,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, 0
mov r1, {}
rand.bytes r0, r1
halt"#,
                        len
                    ),
                )
            }
            "rand_range" => {
                let max: i32 = self.rng.gen_range(10..1000);
                let prompt = self.fmt1(
                    &[
                        "Random number from 0 to {a}",
                        "Generate random in range 0-{a}",
                        "Random integer less than {a}",
                        "rand.u64 mod {a}",
                    ],
                    max,
                );
                (
                    prompt,
                    format!(
                        r#"rand.u64 r0
mov r1, {}
mod r0, r0, r1
halt"#,
                        max
                    ),
                )
            }
            _ => (
                "Call an extension".to_string(),
                "mov r0, 42\nhalt".to_string(),
            ),
        };

        (prompt, program, 5, "extensions".to_string(), None)
    }

    /// Generate FPU (floating-point) operations - returns (prompt, assembly, level, category)
    /// NOTE: Uses integer placeholder values since the assembler doesn't support 64-bit literals.
    /// The model learns the FPU instruction syntax; values will be loaded from memory in real use.
    fn gen_fpu_raw(&mut self) -> (String, String, u8, String, Option<i64>) {
        let fpu_types = [
            "fadd", "fsub", "fmul", "fdiv", "fsqrt", "fabs", "ffloor", "fceil",
        ];
        let fpu_type = fpu_types[self.rng.gen_range(0..fpu_types.len())];

        // Use small integer values as placeholders (actual floats would be loaded from memory)
        let a: i32 = self.rng.gen_range(1..100);
        let b: i32 = self.rng.gen_range(1..100);

        let (prompt, program) = match fpu_type {
            "fadd" => {
                let prompt = self.fmt2(
                    &[
                        "Add {a} + {b} (floating point)",
                        "Float add {a} and {b}",
                        "fpu.fadd({a}, {b})",
                        "Floating-point addition of {a} and {b}",
                    ],
                    a,
                    b,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, {}
fpu.fadd r0, r0, r1
halt"#,
                        a, b
                    ),
                )
            }
            "fsub" => {
                let prompt = self.fmt2(
                    &[
                        "Subtract {a} - {b} (floating point)",
                        "Float subtract {b} from {a}",
                        "fpu.fsub({a}, {b})",
                        "Floating-point subtraction of {a} and {b}",
                    ],
                    a,
                    b,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, {}
fpu.fsub r0, r0, r1
halt"#,
                        a, b
                    ),
                )
            }
            "fmul" => {
                let prompt = self.fmt2(
                    &[
                        "Multiply {a} * {b} (floating point)",
                        "Float multiply {a} and {b}",
                        "fpu.fmul({a}, {b})",
                        "Floating-point multiplication of {a} and {b}",
                    ],
                    a,
                    b,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, {}
fpu.fmul r0, r0, r1
halt"#,
                        a, b
                    ),
                )
            }
            "fdiv" => {
                let prompt = self.fmt2(
                    &[
                        "Divide {a} / {b} (floating point)",
                        "Float divide {a} by {b}",
                        "fpu.fdiv({a}, {b})",
                        "Floating-point division of {a} by {b}",
                    ],
                    a,
                    b,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
mov r1, {}
fpu.fdiv r0, r0, r1
halt"#,
                        a, b
                    ),
                )
            }
            "fsqrt" => {
                let prompt = self.fmt1(
                    &[
                        "Square root of {a} (floating point)",
                        "Float sqrt of {a}",
                        "fpu.fsqrt({a})",
                        "Floating-point square root of {a}",
                    ],
                    a,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
fpu.fsqrt r0, r0
halt"#,
                        a
                    ),
                )
            }
            "fabs" => {
                let prompt = self.fmt1(
                    &[
                        "Absolute value of {a} (floating point)",
                        "Float abs of {a}",
                        "fpu.fabs({a})",
                        "Floating-point absolute value of {a}",
                    ],
                    a,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
fpu.fabs r0, r0
halt"#,
                        a
                    ),
                )
            }
            "ffloor" => {
                let prompt = self.fmt1(
                    &[
                        "Floor of {a} (floating point)",
                        "Float floor of {a}",
                        "fpu.ffloor({a})",
                        "Floating-point floor of {a}",
                    ],
                    a,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
fpu.ffloor r0, r0
halt"#,
                        a
                    ),
                )
            }
            "fceil" => {
                let prompt = self.fmt1(
                    &[
                        "Ceiling of {a} (floating point)",
                        "Float ceil of {a}",
                        "fpu.fceil({a})",
                        "Floating-point ceiling of {a}",
                    ],
                    a,
                );
                (
                    prompt,
                    format!(
                        r#"mov r0, {}
fpu.fceil r0, r0
halt"#,
                        a
                    ),
                )
            }
            _ => ("FPU operation".to_string(), "mov r0, 0\nhalt".to_string()),
        };

        (prompt, program, 3, "fpu".to_string(), None)
    }

    /// Generate crypto extension call examples - returns (prompt, assembly, level, category)
    /// Uses the actual extension IDs from ext_ids module (1-10)
    fn gen_crypto_raw(&mut self) -> (String, String, u8, String, Option<i64>) {
        // Comprehensive crypto operations - all 44 extension types
        let crypto_types = [
            // Core crypto (ext_ids 1-14)
            "sha256",
            "hmac_sha256",
            "aes_gcm_encrypt",
            "aes_gcm_decrypt",
            "constant_time_eq",
            "secure_random",
            "pbkdf2",
            "ed25519_sign",
            "ed25519_verify",
            "x25519_derive",
            "chacha20_poly1305_encrypt",
            "chacha20_poly1305_decrypt",
            "xchacha20_poly1305_encrypt",
            "xchacha20_poly1305_decrypt",
            // Extended hashes (ext_ids 15-20)
            "sha384",
            "sha512",
            "sha3_256",
            "sha3_512",
            "blake2b_512",
            "blake2s_256",
            // Extended HMAC (ext_ids 21-22)
            "hmac_sha384",
            "hmac_sha512",
            // HKDF (ext_ids 23-24)
            "hkdf_extract",
            "hkdf_expand",
            // ECDSA/ECDH P-256/P-384 (ext_ids 25-30)
            "p256_ecdsa_sign",
            "p256_ecdsa_verify",
            "p256_ecdh",
            "p384_ecdsa_sign",
            "p384_ecdsa_verify",
            "p384_ecdh",
            // RSA (ext_ids 31-34)
            "rsa_sign",
            "rsa_verify",
            "rsa_encrypt",
            "rsa_decrypt",
            // Password hashing (ext_id 35)
            "argon2id",
            // X.509 (ext_ids 36-37)
            "x509_parse",
            "x509_get_pubkey",
            // Key generation (ext_ids 38-40)
            "rsa_keygen",
            "p256_keygen",
            "p384_keygen",
            // P-521 ECDSA/ECDH (ext_ids 41-44)
            "p521_ecdsa_sign",
            "p521_ecdsa_verify",
            "p521_ecdh",
            "p521_keygen",
        ];
        let crypto_type = crypto_types[self.rng.gen_range(0..crypto_types.len())];

        let (prompt, program) = match crypto_type {
            "sha256" => {
                let len: i32 = self.rng.gen_range(16..256);
                let prompt = self.fmt1(
                    &[
                        "Compute SHA256 hash of {a} bytes",
                        "SHA256 of {a} byte buffer",
                        "Hash {a} bytes with SHA256",
                        "sha256({a} bytes)",
                    ],
                    len,
                );
                // ext_ids::SHA256 = 1
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, 256
ext.call r0, 1, r1, r2
halt"#,
                        len
                    ),
                )
            }
            "hmac_sha256" => {
                let key_len: i32 = self.rng.gen_range(16..64);
                let data_len: i32 = self.rng.gen_range(32..256);
                let prompt = self.fmt2(
                    &[
                        "HMAC-SHA256 with {a}-byte key on {b} bytes",
                        "Compute HMAC-SHA256: key={a}b, data={b}b",
                        "hmac_sha256 key_len={a} data_len={b}",
                    ],
                    key_len,
                    data_len,
                );
                // ext_ids::HMAC_SHA256 = 2
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, {}
ext.call r0, 2, r1, r2
halt"#,
                        key_len, data_len
                    ),
                )
            }
            "aes_gcm_encrypt" => {
                let plaintext_len: i32 = self.rng.gen_range(16..512);
                let prompt = self.fmt1(
                    &[
                        "AES-256-GCM encrypt {a} bytes",
                        "Encrypt {a} bytes with AES-256-GCM",
                        "aes256_gcm_encrypt({a} bytes)",
                    ],
                    plaintext_len,
                );
                // ext_ids::AES256_GCM_ENCRYPT = 3
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, 512
ext.call r0, 3, r1, r2
halt"#,
                        plaintext_len
                    ),
                )
            }
            "aes_gcm_decrypt" => {
                let ciphertext_len: i32 = self.rng.gen_range(16..512);
                let prompt = self.fmt1(
                    &[
                        "AES-256-GCM decrypt {a} bytes",
                        "Decrypt {a} bytes with AES-256-GCM",
                        "aes256_gcm_decrypt({a} bytes)",
                    ],
                    ciphertext_len,
                );
                // ext_ids::AES256_GCM_DECRYPT = 4
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, 512
ext.call r0, 4, r1, r2
halt"#,
                        ciphertext_len
                    ),
                )
            }
            "constant_time_eq" => {
                let len: i32 = self.rng.gen_range(16..64);
                let prompt = self.fmt1(
                    &[
                        "Constant-time compare {a} bytes",
                        "Compare {a} bytes in constant time",
                        "constant_time_eq({a} bytes)",
                        "Timing-safe comparison of {a} bytes",
                    ],
                    len,
                );
                // ext_ids::CONSTANT_TIME_EQ = 5
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, {}
ext.call r0, 5, r1, r2
halt"#,
                        len, len
                    ),
                )
            }
            "secure_random" => {
                let len: i32 = self.rng.gen_range(16..128);
                let prompt = self.fmt1(
                    &[
                        "Generate {a} secure random bytes",
                        "Fill buffer with {a} random bytes",
                        "secure_random({a})",
                        "Cryptographically secure random {a} bytes",
                    ],
                    len,
                );
                // ext_ids::SECURE_RANDOM = 6
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
ext.call r0, 6, r1, r2
halt"#,
                        len
                    ),
                )
            }
            "pbkdf2" => {
                let iterations: i32 = self.rng.gen_range(1000..10000);
                let prompt = self.fmt1(
                    &[
                        "PBKDF2-SHA256 with {a} iterations",
                        "Derive key using PBKDF2 ({a} rounds)",
                        "pbkdf2_sha256 iterations={a}",
                    ],
                    iterations,
                );
                // ext_ids::PBKDF2_SHA256 = 7
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, 32
mov r3, {}
ext.call r0, 7, r1, r2
halt"#,
                        iterations
                    ),
                )
            }
            "ed25519_sign" => {
                let prompt = self.pick_prompt(&[
                    "Sign message with Ed25519",
                    "Ed25519 digital signature",
                    "Create Ed25519 signature",
                    "ed25519_sign(message)",
                ]);
                // ext_ids::ED25519_SIGN = 8
                (
                    prompt,
                    r#"mov r1, 0
mov r2, 32
mov r3, 64
ext.call r0, 8, r1, r2
halt"#
                        .to_string(),
                )
            }
            "ed25519_verify" => {
                let prompt = self.pick_prompt(&[
                    "Verify Ed25519 signature",
                    "Ed25519 signature verification",
                    "Check Ed25519 signature",
                    "ed25519_verify(signature, message)",
                ]);
                // ext_ids::ED25519_VERIFY = 9
                (
                    prompt,
                    r#"mov r1, 0
mov r2, 32
mov r3, 64
ext.call r0, 9, r1, r2
halt"#
                        .to_string(),
                )
            }
            "x25519_derive" => {
                let prompt = self.pick_prompt(&[
                    "X25519 key exchange",
                    "Derive shared secret with X25519",
                    "x25519_derive(secret, public)",
                    "ECDH using X25519",
                ]);
                // ext_ids::X25519_DERIVE = 10
                (
                    prompt,
                    r#"mov r1, 0
mov r2, 32
mov r3, 32
ext.call r0, 10, r1, r2
halt"#
                        .to_string(),
                )
            }
            "chacha20_poly1305_encrypt" => {
                let len: i32 = self.rng.gen_range(16..512);
                let prompt = self.fmt1(
                    &[
                        "ChaCha20-Poly1305 encrypt {a} bytes",
                        "Encrypt {a} bytes with ChaCha20-Poly1305",
                        "chacha20_poly1305_encrypt({a} bytes)",
                    ],
                    len,
                );
                // ext_ids::CHACHA20_POLY1305_ENCRYPT = 11
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, 512
ext.call r0, 11, r1, r2
halt"#,
                        len
                    ),
                )
            }
            "chacha20_poly1305_decrypt" => {
                let len: i32 = self.rng.gen_range(16..512);
                let prompt = self.fmt1(
                    &[
                        "ChaCha20-Poly1305 decrypt {a} bytes",
                        "Decrypt {a} bytes with ChaCha20-Poly1305",
                        "chacha20_poly1305_decrypt({a} bytes)",
                    ],
                    len,
                );
                // ext_ids::CHACHA20_POLY1305_DECRYPT = 12
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, 512
ext.call r0, 12, r1, r2
halt"#,
                        len
                    ),
                )
            }
            "xchacha20_poly1305_encrypt" => {
                let len: i32 = self.rng.gen_range(16..512);
                let prompt = self.fmt1(
                    &[
                        "XChaCha20-Poly1305 encrypt {a} bytes",
                        "Encrypt {a} bytes with XChaCha20-Poly1305",
                        "xchacha20_poly1305_encrypt({a} bytes)",
                    ],
                    len,
                );
                // ext_ids::XCHACHA20_POLY1305_ENCRYPT = 13
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, 512
ext.call r0, 13, r1, r2
halt"#,
                        len
                    ),
                )
            }
            "xchacha20_poly1305_decrypt" => {
                let len: i32 = self.rng.gen_range(16..512);
                let prompt = self.fmt1(
                    &[
                        "XChaCha20-Poly1305 decrypt {a} bytes",
                        "Decrypt {a} bytes with XChaCha20-Poly1305",
                        "xchacha20_poly1305_decrypt({a} bytes)",
                    ],
                    len,
                );
                // ext_ids::XCHACHA20_POLY1305_DECRYPT = 14
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, 512
ext.call r0, 14, r1, r2
halt"#,
                        len
                    ),
                )
            }
            "sha384" => {
                let len: i32 = self.rng.gen_range(16..256);
                let prompt = self.fmt1(
                    &[
                        "Compute SHA-384 hash of {a} bytes",
                        "SHA-384 of {a} byte buffer",
                        "Hash {a} bytes with SHA-384",
                        "sha384({a} bytes)",
                    ],
                    len,
                );
                // ext_ids::SHA384 = 15
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, 320
ext.call r0, 15, r1, r2
halt"#,
                        len
                    ),
                )
            }
            "sha512" => {
                let len: i32 = self.rng.gen_range(16..256);
                let prompt = self.fmt1(
                    &[
                        "Compute SHA-512 hash of {a} bytes",
                        "SHA-512 of {a} byte buffer",
                        "Hash {a} bytes with SHA-512",
                        "sha512({a} bytes)",
                    ],
                    len,
                );
                // ext_ids::SHA512 = 16
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, 320
ext.call r0, 16, r1, r2
halt"#,
                        len
                    ),
                )
            }
            "sha3_256" => {
                let len: i32 = self.rng.gen_range(16..256);
                let prompt = self.fmt1(
                    &[
                        "Compute SHA3-256 hash of {a} bytes",
                        "SHA3-256 of {a} byte buffer",
                        "Hash {a} bytes with SHA3-256",
                        "sha3_256({a} bytes)",
                    ],
                    len,
                );
                // ext_ids::SHA3_256 = 17
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, 256
ext.call r0, 17, r1, r2
halt"#,
                        len
                    ),
                )
            }
            "sha3_512" => {
                let len: i32 = self.rng.gen_range(16..256);
                let prompt = self.fmt1(
                    &[
                        "Compute SHA3-512 hash of {a} bytes",
                        "SHA3-512 of {a} byte buffer",
                        "Hash {a} bytes with SHA3-512",
                        "sha3_512({a} bytes)",
                    ],
                    len,
                );
                // ext_ids::SHA3_512 = 18
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, 320
ext.call r0, 18, r1, r2
halt"#,
                        len
                    ),
                )
            }
            "blake2b_512" => {
                let len: i32 = self.rng.gen_range(16..256);
                let prompt = self.fmt1(
                    &[
                        "Compute BLAKE2b-512 hash of {a} bytes",
                        "BLAKE2b-512 of {a} byte buffer",
                        "Hash {a} bytes with BLAKE2b",
                        "blake2b_512({a} bytes)",
                    ],
                    len,
                );
                // ext_ids::BLAKE2B_512 = 19
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, 320
ext.call r0, 19, r1, r2
halt"#,
                        len
                    ),
                )
            }
            "blake2s_256" => {
                let len: i32 = self.rng.gen_range(16..256);
                let prompt = self.fmt1(
                    &[
                        "Compute BLAKE2s-256 hash of {a} bytes",
                        "BLAKE2s-256 of {a} byte buffer",
                        "Hash {a} bytes with BLAKE2s",
                        "blake2s_256({a} bytes)",
                    ],
                    len,
                );
                // ext_ids::BLAKE2S_256 = 20
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, 256
ext.call r0, 20, r1, r2
halt"#,
                        len
                    ),
                )
            }
            "hmac_sha384" => {
                let key_len: i32 = self.rng.gen_range(16..64);
                let data_len: i32 = self.rng.gen_range(32..256);
                let prompt = self.fmt2(
                    &[
                        "HMAC-SHA384 with {a}-byte key on {b} bytes",
                        "Compute HMAC-SHA384: key={a}b, data={b}b",
                        "hmac_sha384 key_len={a} data_len={b}",
                    ],
                    key_len,
                    data_len,
                );
                // ext_ids::HMAC_SHA384 = 21
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, {}
ext.call r0, 21, r1, r2
halt"#,
                        key_len, data_len
                    ),
                )
            }
            "hmac_sha512" => {
                let key_len: i32 = self.rng.gen_range(16..64);
                let data_len: i32 = self.rng.gen_range(32..256);
                let prompt = self.fmt2(
                    &[
                        "HMAC-SHA512 with {a}-byte key on {b} bytes",
                        "Compute HMAC-SHA512: key={a}b, data={b}b",
                        "hmac_sha512 key_len={a} data_len={b}",
                    ],
                    key_len,
                    data_len,
                );
                // ext_ids::HMAC_SHA512 = 22
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, {}
ext.call r0, 22, r1, r2
halt"#,
                        key_len, data_len
                    ),
                )
            }
            "hkdf_extract" => {
                let ikm_len: i32 = self.rng.gen_range(16..64);
                let prompt = self.fmt1(
                    &[
                        "HKDF-SHA256 extract with {a}-byte IKM",
                        "Key derivation HKDF extract ({a} bytes)",
                        "hkdf_sha256_extract ikm_len={a}",
                        "TLS 1.3 key extract from {a} bytes",
                    ],
                    ikm_len,
                );
                // ext_ids::HKDF_SHA256_EXTRACT = 23
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, 32
ext.call r0, 23, r1, r2
halt"#,
                        ikm_len
                    ),
                )
            }
            "hkdf_expand" => {
                let okm_len: i32 = self.rng.gen_range(32..128);
                let prompt = self.fmt1(
                    &[
                        "HKDF-SHA256 expand to {a} bytes",
                        "Key derivation HKDF expand ({a} bytes)",
                        "hkdf_sha256_expand okm_len={a}",
                        "TLS 1.3 key expand to {a} bytes",
                    ],
                    okm_len,
                );
                // ext_ids::HKDF_SHA256_EXPAND = 24
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, 32
mov r3, {}
ext.call r0, 24, r1, r2
halt"#,
                        okm_len
                    ),
                )
            }
            "p256_ecdsa_sign" => {
                let prompt = self.pick_prompt(&[
                    "Sign with P-256 ECDSA",
                    "ECDSA P-256 digital signature",
                    "Create NIST P-256 signature",
                    "p256_ecdsa_sign(message)",
                ]);
                // ext_ids::P256_ECDSA_SIGN = 25
                (
                    prompt,
                    r#"mov r1, 0
mov r2, 32
mov r3, 64
ext.call r0, 25, r1, r2
halt"#
                        .to_string(),
                )
            }
            "p256_ecdsa_verify" => {
                let prompt = self.pick_prompt(&[
                    "Verify P-256 ECDSA signature",
                    "ECDSA P-256 signature verification",
                    "Check NIST P-256 signature",
                    "p256_ecdsa_verify(signature, message)",
                ]);
                // ext_ids::P256_ECDSA_VERIFY = 26
                (
                    prompt,
                    r#"mov r1, 0
mov r2, 32
mov r3, 64
ext.call r0, 26, r1, r2
halt"#
                        .to_string(),
                )
            }
            "p256_ecdh" => {
                let prompt = self.pick_prompt(&[
                    "P-256 ECDH key exchange",
                    "Derive shared secret with P-256",
                    "p256_ecdh(secret, public)",
                    "ECDH using NIST P-256",
                ]);
                // ext_ids::P256_ECDH = 27
                (
                    prompt,
                    r#"mov r1, 0
mov r2, 32
mov r3, 32
ext.call r0, 27, r1, r2
halt"#
                        .to_string(),
                )
            }
            "p384_ecdsa_sign" => {
                let prompt = self.pick_prompt(&[
                    "Sign with P-384 ECDSA",
                    "ECDSA P-384 digital signature",
                    "Create NIST P-384 signature",
                    "p384_ecdsa_sign(message)",
                ]);
                // ext_ids::P384_ECDSA_SIGN = 28
                (
                    prompt,
                    r#"mov r1, 0
mov r2, 48
mov r3, 96
ext.call r0, 28, r1, r2
halt"#
                        .to_string(),
                )
            }
            "p384_ecdsa_verify" => {
                let prompt = self.pick_prompt(&[
                    "Verify P-384 ECDSA signature",
                    "ECDSA P-384 signature verification",
                    "Check NIST P-384 signature",
                    "p384_ecdsa_verify(signature, message)",
                ]);
                // ext_ids::P384_ECDSA_VERIFY = 29
                (
                    prompt,
                    r#"mov r1, 0
mov r2, 48
mov r3, 96
ext.call r0, 29, r1, r2
halt"#
                        .to_string(),
                )
            }
            "p384_ecdh" => {
                let prompt = self.pick_prompt(&[
                    "P-384 ECDH key exchange",
                    "Derive shared secret with P-384",
                    "p384_ecdh(secret, public)",
                    "ECDH using NIST P-384",
                ]);
                // ext_ids::P384_ECDH = 30
                (
                    prompt,
                    r#"mov r1, 0
mov r2, 48
mov r3, 48
ext.call r0, 30, r1, r2
halt"#
                        .to_string(),
                )
            }
            "rsa_sign" => {
                let key_size = [2048, 3072, 4096][self.rng.gen_range(0..3)];
                let prompt = self.fmt1(
                    &[
                        "RSA-{a} PKCS#1 v1.5 sign",
                        "Sign with RSA-{a}",
                        "rsa_pkcs1_sign({a} bits)",
                        "Create RSA-{a} signature",
                    ],
                    key_size,
                );
                // ext_ids::RSA_PKCS1_SIGN_SHA256 = 31
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, {}
ext.call r0, 31, r1, r2
halt"#,
                        key_size / 8,
                        key_size / 8
                    ),
                )
            }
            "rsa_verify" => {
                let key_size = [2048, 3072, 4096][self.rng.gen_range(0..3)];
                let prompt = self.fmt1(
                    &[
                        "RSA-{a} PKCS#1 v1.5 verify",
                        "Verify RSA-{a} signature",
                        "rsa_pkcs1_verify({a} bits)",
                        "Check RSA-{a} signature",
                    ],
                    key_size,
                );
                // ext_ids::RSA_PKCS1_VERIFY_SHA256 = 32
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, {}
ext.call r0, 32, r1, r2
halt"#,
                        key_size / 8,
                        key_size / 8
                    ),
                )
            }
            "rsa_encrypt" => {
                let key_size = [2048, 3072, 4096][self.rng.gen_range(0..3)];
                let prompt = self.fmt1(
                    &[
                        "RSA-{a} OAEP encrypt",
                        "Encrypt with RSA-{a} OAEP",
                        "rsa_oaep_encrypt({a} bits)",
                        "RSA-{a} asymmetric encryption",
                    ],
                    key_size,
                );
                // ext_ids::RSA_OAEP_ENCRYPT_SHA256 = 33
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, 32
mov r3, {}
ext.call r0, 33, r1, r2
halt"#,
                        key_size / 8
                    ),
                )
            }
            "rsa_decrypt" => {
                let key_size = [2048, 3072, 4096][self.rng.gen_range(0..3)];
                let prompt = self.fmt1(
                    &[
                        "RSA-{a} OAEP decrypt",
                        "Decrypt with RSA-{a} OAEP",
                        "rsa_oaep_decrypt({a} bits)",
                        "RSA-{a} asymmetric decryption",
                    ],
                    key_size,
                );
                // ext_ids::RSA_OAEP_DECRYPT_SHA256 = 34
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, {}
mov r3, 512
ext.call r0, 34, r1, r2
halt"#,
                        key_size / 8
                    ),
                )
            }
            "argon2id" => {
                let mem_cost: i32 = self.rng.gen_range(16..64);
                let prompt = self.fmt1(
                    &[
                        "Argon2id hash with {a}MB memory",
                        "Password hash with Argon2id ({a}MB)",
                        "argon2id_hash memory={a}MB",
                        "Secure password hashing Argon2id",
                    ],
                    mem_cost,
                );
                // ext_ids::ARGON2ID_HASH = 35
                (
                    prompt,
                    format!(
                        r#"mov r1, 0
mov r2, 32
mov r3, {}
ext.call r0, 35, r1, r2
halt"#,
                        mem_cost
                    ),
                )
            }
            "x509_parse" => {
                let prompt = self.pick_prompt(&[
                    "Parse X.509 certificate",
                    "X.509 certificate parsing",
                    "x509_parse_cert(der_bytes)",
                    "Extract info from X.509 cert",
                ]);
                // ext_ids::X509_PARSE_CERT = 36
                (
                    prompt,
                    r#"mov r1, 0
mov r2, 1024
mov r3, 256
ext.call r0, 36, r1, r2
halt"#
                        .to_string(),
                )
            }
            "x509_get_pubkey" => {
                let prompt = self.pick_prompt(&[
                    "Get public key from X.509 cert",
                    "X.509 extract public key",
                    "x509_get_public_key(cert)",
                    "Extract pubkey from certificate",
                ]);
                // ext_ids::X509_GET_PUBLIC_KEY = 37
                (
                    prompt,
                    r#"mov r1, 0
mov r2, 1024
mov r3, 256
ext.call r0, 37, r1, r2
halt"#
                        .to_string(),
                )
            }
            "rsa_keygen" => {
                let key_size = [2048, 3072, 4096][self.rng.gen_range(0..3)];
                let prompt = self.fmt1(
                    &[
                        "Generate RSA-{a} keypair",
                        "RSA-{a} key generation",
                        "rsa_generate_keypair({a} bits)",
                        "Create new RSA-{a} keys",
                    ],
                    key_size,
                );
                // ext_ids::RSA_GENERATE_KEYPAIR = 38
                (
                    prompt,
                    format!(
                        r#"mov r1, {}
mov r2, 0
mov r3, 2048
ext.call r0, 38, r1, r2
halt"#,
                        key_size
                    ),
                )
            }
            "p256_keygen" => {
                let prompt = self.pick_prompt(&[
                    "Generate P-256 keypair",
                    "NIST P-256 key generation",
                    "p256_generate_keypair()",
                    "Create new P-256 ECDSA keys",
                ]);
                // ext_ids::P256_GENERATE_KEYPAIR = 39
                (
                    prompt,
                    r#"mov r1, 0
mov r2, 32
mov r3, 64
ext.call r0, 39, r1, r2
halt"#
                        .to_string(),
                )
            }
            "p384_keygen" => {
                let prompt = self.pick_prompt(&[
                    "Generate P-384 keypair",
                    "NIST P-384 key generation",
                    "p384_generate_keypair()",
                    "Create new P-384 ECDSA keys",
                ]);
                // ext_ids::P384_GENERATE_KEYPAIR = 40
                (
                    prompt,
                    r#"mov r1, 0
mov r2, 48
mov r3, 96
ext.call r0, 40, r1, r2
halt"#
                        .to_string(),
                )
            }
            "p521_ecdsa_sign" => {
                let prompt = self.pick_prompt(&[
                    "Sign with P-521 ECDSA",
                    "ECDSA P-521 digital signature",
                    "Create NIST P-521 signature",
                    "p521_ecdsa_sign(message)",
                ]);
                // ext_ids::P521_ECDSA_SIGN = 41
                (
                    prompt,
                    r#"mov r1, 0
mov r2, 66
mov r3, 132
ext.call r0, 41, r1, r2
halt"#
                        .to_string(),
                )
            }
            "p521_ecdsa_verify" => {
                let prompt = self.pick_prompt(&[
                    "Verify P-521 ECDSA signature",
                    "ECDSA P-521 signature verification",
                    "Check NIST P-521 signature",
                    "p521_ecdsa_verify(signature, message)",
                ]);
                // ext_ids::P521_ECDSA_VERIFY = 42
                (
                    prompt,
                    r#"mov r1, 0
mov r2, 66
mov r3, 132
ext.call r0, 42, r1, r2
halt"#
                        .to_string(),
                )
            }
            "p521_ecdh" => {
                let prompt = self.pick_prompt(&[
                    "P-521 ECDH key exchange",
                    "Derive shared secret with P-521",
                    "p521_ecdh(secret, public)",
                    "ECDH using NIST P-521",
                ]);
                // ext_ids::P521_ECDH = 43
                (
                    prompt,
                    r#"mov r1, 0
mov r2, 66
mov r3, 66
ext.call r0, 43, r1, r2
halt"#
                        .to_string(),
                )
            }
            "p521_keygen" => {
                let prompt = self.pick_prompt(&[
                    "Generate P-521 keypair",
                    "NIST P-521 key generation",
                    "p521_generate_keypair()",
                    "Create new P-521 ECDSA keys",
                ]);
                // ext_ids::P521_GENERATE_KEYPAIR = 44
                (
                    prompt,
                    r#"mov r1, 0
mov r2, 66
mov r3, 133
ext.call r0, 44, r1, r2
halt"#
                        .to_string(),
                )
            }
            _ => (
                "Crypto operation".to_string(),
                "mov r0, 0\nhalt".to_string(),
            ),
        };

        (prompt, program, 5, "crypto".to_string(), None)
    }

    /// Generate stdlib (collections, strings, JSON) operations
    /// Extension IDs: Vec 100-112, HashMap 120-129, String 140-158, JSON 170-181
    fn gen_stdlib_raw(&mut self) -> (String, String, u8, String, Option<i64>) {
        let stdlib_types = [
            // Vec operations (ext_ids 100-112)
            "vec_new",
            "vec_push",
            "vec_pop",
            "vec_get",
            "vec_set",
            "vec_len",
            "vec_insert",
            "vec_remove",
            "vec_clear",
            // HashMap operations (ext_ids 120-129)
            "hashmap_new",
            "hashmap_insert",
            "hashmap_get",
            "hashmap_remove",
            "hashmap_contains",
            "hashmap_len",
            "hashmap_keys",
            // String operations (ext_ids 140-158)
            "string_new",
            "string_concat",
            "string_len",
            "string_substr",
            "string_find",
            "string_replace",
            "string_split",
            "string_trim",
            "string_to_upper",
            "string_to_lower",
            "string_parse_int",
            "string_from_int",
            // JSON operations (ext_ids 170-181)
            "json_parse",
            "json_stringify",
            "json_get",
            "json_set",
            "json_array_len",
            "json_array_get",
            "json_object_keys",
            // HTTP operations (ext_ids 190-198)
            "http_get",
            "http_post",
            "http_put",
            "http_delete",
            "http_response_status",
            "http_response_body",
        ];
        let stdlib_type = stdlib_types[self.rng.gen_range(0..stdlib_types.len())];

        let (prompt, program) = match stdlib_type {
            // ===== VEC OPERATIONS =====
            "vec_new" => {
                let prompt = self.pick_prompt(&[
                    "Create a new empty vector",
                    "Initialize empty Vec",
                    "vec_new()",
                    "Create dynamic array",
                    "New vector for storing values",
                ]);
                // ext_ids::VEC_NEW = 100
                (
                    prompt,
                    r#"ext.call r0, 100, r0, r0
halt"#
                        .to_string(),
                )
            }
            "vec_push" => {
                let value: i32 = self.rng.gen_range(1..1000);
                let prompt = self.fmt1(
                    &[
                        "Push {a} onto a vector",
                        "Append {a} to vector",
                        "vec.push({a})",
                        "Add {a} to end of vector",
                    ],
                    value,
                );
                // ext_ids::VEC_NEW = 100, VEC_PUSH = 102
                (
                    prompt,
                    format!(
                        r#"ext.call r1, 100, r0, r0
mov r2, {}
ext.call r0, 102, r1, r2
halt"#,
                        value
                    ),
                )
            }
            "vec_pop" => {
                let prompt = self.pick_prompt(&[
                    "Pop last element from vector",
                    "Remove and return last item",
                    "vec.pop()",
                    "Get last element from vector",
                ]);
                // ext_ids::VEC_NEW = 100, VEC_PUSH = 102, VEC_POP = 103
                (
                    prompt,
                    r#"ext.call r1, 100, r0, r0
mov r2, 42
ext.call r0, 102, r1, r2
ext.call r0, 103, r1, r0
halt"#
                        .to_string(),
                )
            }
            "vec_get" => {
                let index: i32 = self.rng.gen_range(0..10);
                let prompt = self.fmt1(
                    &[
                        "Get element at index {a} from vector",
                        "Vector element at position {a}",
                        "vec[{a}]",
                        "Access vector index {a}",
                    ],
                    index,
                );
                // ext_ids::VEC_GET = 104
                (
                    prompt,
                    format!(
                        r#"mov r2, {}
ext.call r0, 104, r1, r2
halt"#,
                        index
                    ),
                )
            }
            "vec_set" => {
                let index: i32 = self.rng.gen_range(0..10);
                let value: i32 = self.rng.gen_range(1..1000);
                let prompt = self.fmt2(
                    &[
                        "Set vector[{a}] = {b}",
                        "Assign {b} to vector index {a}",
                        "vec.set({a}, {b})",
                        "Update position {a} to {b}",
                    ],
                    index,
                    value,
                );
                // ext_ids::VEC_SET = 105
                (
                    prompt,
                    format!(
                        r#"mov r2, {}
mov r3, {}
ext.call r0, 105, r1, r2
halt"#,
                        index, value
                    ),
                )
            }
            "vec_len" => {
                let prompt = self.pick_prompt(&[
                    "Get vector length",
                    "How many elements in vector",
                    "vec.len()",
                    "Vector size",
                    "Count of vector elements",
                ]);
                // ext_ids::VEC_LEN = 106
                (
                    prompt,
                    r#"ext.call r0, 106, r1, r0
halt"#
                        .to_string(),
                )
            }
            "vec_insert" => {
                let index: i32 = self.rng.gen_range(0..5);
                let value: i32 = self.rng.gen_range(1..100);
                let prompt = self.fmt2(
                    &[
                        "Insert {b} at index {a} in vector",
                        "vec.insert({a}, {b})",
                        "Add {b} at position {a}",
                    ],
                    index,
                    value,
                );
                // ext_ids::VEC_INSERT = 111
                (
                    prompt,
                    format!(
                        r#"mov r2, {}
mov r3, {}
ext.call r0, 111, r1, r2
halt"#,
                        index, value
                    ),
                )
            }
            "vec_remove" => {
                let index: i32 = self.rng.gen_range(0..5);
                let prompt = self.fmt1(
                    &[
                        "Remove element at index {a}",
                        "vec.remove({a})",
                        "Delete item at position {a}",
                    ],
                    index,
                );
                // ext_ids::VEC_REMOVE = 112
                (
                    prompt,
                    format!(
                        r#"mov r2, {}
ext.call r0, 112, r1, r2
halt"#,
                        index
                    ),
                )
            }
            "vec_clear" => {
                let prompt = self.pick_prompt(&[
                    "Clear all elements from vector",
                    "Empty the vector",
                    "vec.clear()",
                    "Remove all items from vector",
                ]);
                // ext_ids::VEC_CLEAR = 108
                (
                    prompt,
                    r#"ext.call r0, 108, r1, r0
halt"#
                        .to_string(),
                )
            }

            // ===== HASHMAP OPERATIONS =====
            "hashmap_new" => {
                let prompt = self.pick_prompt(&[
                    "Create a new empty hashmap",
                    "Initialize HashMap",
                    "hashmap_new()",
                    "New key-value store",
                    "Create dictionary",
                ]);
                // ext_ids::HASHMAP_NEW = 120
                (
                    prompt,
                    r#"ext.call r0, 120, r0, r0
halt"#
                        .to_string(),
                )
            }
            "hashmap_insert" => {
                let key: i32 = self.rng.gen_range(1..100);
                let value: i32 = self.rng.gen_range(1..1000);
                let prompt = self.fmt2(
                    &[
                        "Insert key {a} with value {b} into hashmap",
                        "hashmap[{a}] = {b}",
                        "map.insert({a}, {b})",
                        "Store {b} at key {a}",
                    ],
                    key,
                    value,
                );
                // ext_ids::HASHMAP_NEW = 120, HASHMAP_INSERT = 121
                (
                    prompt,
                    format!(
                        r#"ext.call r1, 120, r0, r0
mov r2, {}
mov r3, {}
ext.call r0, 121, r1, r2
halt"#,
                        key, value
                    ),
                )
            }
            "hashmap_get" => {
                let key: i32 = self.rng.gen_range(1..100);
                let prompt = self.fmt1(
                    &[
                        "Get value for key {a} from hashmap",
                        "hashmap[{a}]",
                        "map.get({a})",
                        "Lookup key {a}",
                    ],
                    key,
                );
                // ext_ids::HASHMAP_GET = 122
                (
                    prompt,
                    format!(
                        r#"mov r2, {}
ext.call r0, 122, r1, r2
halt"#,
                        key
                    ),
                )
            }
            "hashmap_remove" => {
                let key: i32 = self.rng.gen_range(1..100);
                let prompt = self.fmt1(
                    &[
                        "Remove key {a} from hashmap",
                        "map.remove({a})",
                        "Delete key {a}",
                    ],
                    key,
                );
                // ext_ids::HASHMAP_REMOVE = 123
                (
                    prompt,
                    format!(
                        r#"mov r2, {}
ext.call r0, 123, r1, r2
halt"#,
                        key
                    ),
                )
            }
            "hashmap_contains" => {
                let key: i32 = self.rng.gen_range(1..100);
                let prompt = self.fmt1(
                    &[
                        "Check if hashmap contains key {a}",
                        "map.contains({a})",
                        "Is key {a} in map?",
                        "Does hashmap have key {a}",
                    ],
                    key,
                );
                // ext_ids::HASHMAP_CONTAINS = 124
                (
                    prompt,
                    format!(
                        r#"mov r2, {}
ext.call r0, 124, r1, r2
halt"#,
                        key
                    ),
                )
            }
            "hashmap_len" => {
                let prompt = self.pick_prompt(&[
                    "Get hashmap size",
                    "Number of entries in map",
                    "map.len()",
                    "How many key-value pairs",
                ]);
                // ext_ids::HASHMAP_LEN = 125
                (
                    prompt,
                    r#"ext.call r0, 125, r1, r0
halt"#
                        .to_string(),
                )
            }
            "hashmap_keys" => {
                let prompt = self.pick_prompt(&[
                    "Get all keys from hashmap",
                    "map.keys()",
                    "List hashmap keys",
                    "Iterate over keys",
                ]);
                // ext_ids::HASHMAP_KEYS = 128
                (
                    prompt,
                    r#"ext.call r0, 128, r1, r0
halt"#
                        .to_string(),
                )
            }

            // ===== STRING OPERATIONS =====
            "string_new" => {
                let prompt = self.pick_prompt(&[
                    "Create a new empty string",
                    "Initialize String",
                    "string_new()",
                    "New string buffer",
                ]);
                // ext_ids::STRING_NEW = 140
                (
                    prompt,
                    r#"ext.call r0, 140, r0, r0
halt"#
                        .to_string(),
                )
            }
            "string_concat" => {
                let prompt = self.pick_prompt(&[
                    "Concatenate two strings",
                    "Join strings together",
                    "str1 + str2",
                    "string.concat()",
                    "Append string to another",
                ]);
                // ext_ids::STRING_CONCAT = 143
                (
                    prompt,
                    r#"ext.call r0, 143, r1, r2
halt"#
                        .to_string(),
                )
            }
            "string_len" => {
                let prompt = self.pick_prompt(&[
                    "Get string length",
                    "string.len()",
                    "How long is the string",
                    "Number of characters in string",
                ]);
                // ext_ids::STRING_LEN = 142
                (
                    prompt,
                    r#"ext.call r0, 142, r1, r0
halt"#
                        .to_string(),
                )
            }
            "string_substr" => {
                let start: i32 = self.rng.gen_range(0..10);
                let len: i32 = self.rng.gen_range(1..20);
                let prompt = self.fmt2(
                    &[
                        "Get substring from {a} with length {b}",
                        "string.substr({a}, {b})",
                        "Extract {b} chars starting at {a}",
                    ],
                    start,
                    len,
                );
                // ext_ids::STRING_SUBSTR = 144
                (
                    prompt,
                    format!(
                        r#"mov r2, {}
mov r3, {}
ext.call r0, 144, r1, r2
halt"#,
                        start, len
                    ),
                )
            }
            "string_find" => {
                let prompt = self.pick_prompt(&[
                    "Find substring in string",
                    "string.find(needle)",
                    "Search for pattern",
                    "Index of substring",
                ]);
                // ext_ids::STRING_FIND = 145
                (
                    prompt,
                    r#"ext.call r0, 145, r1, r2
halt"#
                        .to_string(),
                )
            }
            "string_replace" => {
                let prompt = self.pick_prompt(&[
                    "Replace substring in string",
                    "string.replace(old, new)",
                    "Substitute pattern",
                    "Find and replace",
                ]);
                // ext_ids::STRING_REPLACE = 146
                (
                    prompt,
                    r#"ext.call r0, 146, r1, r2
halt"#
                        .to_string(),
                )
            }
            "string_split" => {
                let prompt = self.pick_prompt(&[
                    "Split string by delimiter",
                    "string.split(delim)",
                    "Tokenize string",
                    "Break string into parts",
                ]);
                // ext_ids::STRING_SPLIT = 147
                (
                    prompt,
                    r#"ext.call r0, 147, r1, r2
halt"#
                        .to_string(),
                )
            }
            "string_trim" => {
                let prompt = self.pick_prompt(&[
                    "Trim whitespace from string",
                    "string.trim()",
                    "Remove leading/trailing spaces",
                    "Strip whitespace",
                ]);
                // ext_ids::STRING_TRIM = 148
                (
                    prompt,
                    r#"ext.call r0, 148, r1, r0
halt"#
                        .to_string(),
                )
            }
            "string_to_upper" => {
                let prompt = self.pick_prompt(&[
                    "Convert string to uppercase",
                    "string.toUpperCase()",
                    "Make string all caps",
                    "Uppercase the string",
                ]);
                // ext_ids::STRING_TO_UPPER = 149
                (
                    prompt,
                    r#"ext.call r0, 149, r1, r0
halt"#
                        .to_string(),
                )
            }
            "string_to_lower" => {
                let prompt = self.pick_prompt(&[
                    "Convert string to lowercase",
                    "string.toLowerCase()",
                    "Make string all lowercase",
                    "Lowercase the string",
                ]);
                // ext_ids::STRING_TO_LOWER = 150
                (
                    prompt,
                    r#"ext.call r0, 150, r1, r0
halt"#
                        .to_string(),
                )
            }
            "string_parse_int" => {
                let prompt = self.pick_prompt(&[
                    "Parse string to integer",
                    "parseInt(string)",
                    "Convert string to number",
                    "String to int",
                ]);
                // ext_ids::STRING_PARSE_INT = 155
                (
                    prompt,
                    r#"ext.call r0, 155, r1, r0
halt"#
                        .to_string(),
                )
            }
            "string_from_int" => {
                let value: i32 = self.rng.gen_range(1..10000);
                let prompt = self.fmt1(
                    &[
                        "Convert {a} to string",
                        "String.valueOf({a})",
                        "Integer {a} as string",
                        "Stringify number {a}",
                    ],
                    value,
                );
                // ext_ids::STRING_FROM_INT = 157
                (
                    prompt,
                    format!(
                        r#"mov r1, {}
ext.call r0, 157, r1, r0
halt"#,
                        value
                    ),
                )
            }

            // ===== JSON OPERATIONS =====
            "json_parse" => {
                let prompt = self.pick_prompt(&[
                    "Parse JSON string",
                    "JSON.parse()",
                    "Decode JSON data",
                    "String to JSON object",
                ]);
                // ext_ids::JSON_PARSE = 170
                (
                    prompt,
                    r#"ext.call r0, 170, r1, r0
halt"#
                        .to_string(),
                )
            }
            "json_stringify" => {
                let prompt = self.pick_prompt(&[
                    "Convert JSON to string",
                    "JSON.stringify()",
                    "Serialize JSON",
                    "JSON object to string",
                ]);
                // ext_ids::JSON_STRINGIFY = 171
                (
                    prompt,
                    r#"ext.call r0, 171, r1, r0
halt"#
                        .to_string(),
                )
            }
            "json_get" => {
                let prompt = self.pick_prompt(&[
                    "Get value from JSON object",
                    "json.get(key)",
                    "Access JSON field",
                    "JSON property lookup",
                ]);
                // ext_ids::JSON_GET = 172
                (
                    prompt,
                    r#"ext.call r0, 172, r1, r2
halt"#
                        .to_string(),
                )
            }
            "json_set" => {
                let prompt = self.pick_prompt(&[
                    "Set value in JSON object",
                    "json.set(key, value)",
                    "Update JSON field",
                    "Assign JSON property",
                ]);
                // ext_ids::JSON_SET = 173
                (
                    prompt,
                    r#"ext.call r0, 173, r1, r2
halt"#
                        .to_string(),
                )
            }
            "json_array_len" => {
                let prompt = self.pick_prompt(&[
                    "Get JSON array length",
                    "json_array.length",
                    "Count elements in JSON array",
                    "JSON array size",
                ]);
                // ext_ids::JSON_ARRAY_LEN = 175
                (
                    prompt,
                    r#"ext.call r0, 175, r1, r0
halt"#
                        .to_string(),
                )
            }
            "json_array_get" => {
                let index: i32 = self.rng.gen_range(0..10);
                let prompt = self.fmt1(
                    &[
                        "Get element {a} from JSON array",
                        "json_array[{a}]",
                        "JSON array index {a}",
                    ],
                    index,
                );
                // ext_ids::JSON_ARRAY_GET = 176
                (
                    prompt,
                    format!(
                        r#"mov r2, {}
ext.call r0, 176, r1, r2
halt"#,
                        index
                    ),
                )
            }
            "json_object_keys" => {
                let prompt = self.pick_prompt(&[
                    "Get all keys from JSON object",
                    "Object.keys(json)",
                    "List JSON fields",
                    "JSON object properties",
                ]);
                // ext_ids::JSON_OBJECT_KEYS = 178
                (
                    prompt,
                    r#"ext.call r0, 178, r1, r0
halt"#
                        .to_string(),
                )
            }

            // ===== HTTP OPERATIONS =====
            "http_get" => {
                let prompt = self.pick_prompt(&[
                    "HTTP GET request",
                    "Fetch URL",
                    "http.get(url)",
                    "Download from URL",
                    "GET request",
                ]);
                // ext_ids::HTTP_GET = 190
                (
                    prompt,
                    r#"ext.call r0, 190, r1, r0
halt"#
                        .to_string(),
                )
            }
            "http_post" => {
                let prompt = self.pick_prompt(&[
                    "HTTP POST request",
                    "POST data to URL",
                    "http.post(url, body)",
                    "Send POST request",
                    "Submit form data",
                ]);
                // ext_ids::HTTP_POST = 191
                (
                    prompt,
                    r#"ext.call r0, 191, r1, r2
halt"#
                        .to_string(),
                )
            }
            "http_put" => {
                let prompt = self.pick_prompt(&[
                    "HTTP PUT request",
                    "PUT data to URL",
                    "http.put(url, body)",
                    "Update resource",
                ]);
                // ext_ids::HTTP_PUT = 192
                (
                    prompt,
                    r#"ext.call r0, 192, r1, r2
halt"#
                        .to_string(),
                )
            }
            "http_delete" => {
                let prompt = self.pick_prompt(&[
                    "HTTP DELETE request",
                    "Delete resource at URL",
                    "http.delete(url)",
                    "Remove resource",
                ]);
                // ext_ids::HTTP_DELETE = 193
                (
                    prompt,
                    r#"ext.call r0, 193, r1, r0
halt"#
                        .to_string(),
                )
            }
            "http_response_status" => {
                let prompt = self.pick_prompt(&[
                    "Get HTTP response status code",
                    "response.status",
                    "HTTP status code",
                    "Check response code",
                ]);
                // ext_ids::HTTP_RESPONSE_STATUS = 194
                (
                    prompt,
                    r#"ext.call r0, 194, r1, r0
halt"#
                        .to_string(),
                )
            }
            "http_response_body" => {
                let prompt = self.pick_prompt(&[
                    "Get HTTP response body",
                    "response.body",
                    "Read response content",
                    "Get response text",
                ]);
                // ext_ids::HTTP_RESPONSE_BODY = 195
                (
                    prompt,
                    r#"ext.call r0, 195, r1, r0
halt"#
                        .to_string(),
                )
            }
            _ => (
                "Stdlib operation".to_string(),
                "mov r0, 0\nhalt".to_string(),
            ),
        };

        (prompt, program, 4, "stdlib".to_string(), None)
    }
}

/// Extract training examples from the examples/ directory
/// Parses comments to extract prompts
fn load_examples_from_directory(dir: &str) -> Vec<TrainingExample> {
    use std::path::Path;

    let mut examples = Vec::new();
    let path = Path::new(dir);

    if !path.exists() {
        eprintln!("Warning: Examples directory '{}' not found", dir);
        return examples;
    }

    let entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Warning: Failed to read examples directory: {}", e);
            return examples;
        }
    };

    for entry in entries.flatten() {
        let file_path = entry.path();
        if file_path.extension().is_some_and(|e| e == "nl") {
            if let Some(example) = parse_example_file(&file_path) {
                examples.push(example);
            }
        }
    }

    examples
}

/// Parse a single example file and extract training data with binary IR
fn parse_example_file(path: &std::path::Path) -> Option<TrainingExample> {
    let content = std::fs::read_to_string(path).ok()?;
    let filename = path.file_stem()?.to_string_lossy();

    // Extract prompt from first comment block
    let prompt = extract_prompt_from_comments(&content, &filename);

    // Assemble to binary IR
    let mut asm = Assembler::new();
    let program = match asm.assemble(&content) {
        Ok(prog) => prog,
        Err(e) => {
            eprintln!(
                "Warning: Example '{}' failed to assemble: {:?}, skipping",
                filename, e
            );
            return None;
        }
    };

    // Encode to binary IR bytes
    let binary_ir = program.encode();

    // Determine category from filename or content
    let category = categorize_example(&filename, &content);

    // Determine complexity level
    let level = estimate_complexity(&content);

    Some(TrainingExample {
        prompt,
        binary_ir,
        assembly: None, // Not used in training
        expected_output: None,
        level,
        category,
    })
}

/// Extract a prompt from the comments at the start of the file
fn extract_prompt_from_comments(content: &str, filename: &str) -> String {
    let mut prompt_lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(';') {
            // Extract comment content
            let comment = trimmed.trim_start_matches(';').trim();
            // Skip separator lines
            if comment
                .chars()
                .all(|c| c == '=' || c == '-' || c.is_whitespace())
            {
                continue;
            }
            // Skip empty comments
            if !comment.is_empty() {
                prompt_lines.push(comment.to_string());
            }
        } else if !trimmed.is_empty() && !trimmed.starts_with('.') {
            // Hit actual code, stop
            break;
        }
    }

    if prompt_lines.is_empty() {
        // Generate prompt from filename
        format!("Implement {}", filename.replace('_', " "))
    } else {
        // Use first few lines of comments as prompt
        prompt_lines
            .into_iter()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Categorize an example based on filename and content
fn categorize_example(filename: &str, content: &str) -> String {
    let filename_lower = filename.to_lowercase();

    if filename_lower.contains("rest")
        || filename_lower.contains("net")
        || content.contains("net.socket")
    {
        "io".to_string()
    } else if filename_lower.contains("concurrent")
        || content.contains("spawn")
        || content.contains("chan.")
    {
        "concurrency".to_string()
    } else if filename_lower.contains("capability")
        || content.contains("cap.new")
        || content.contains("taint")
    {
        "security".to_string()
    } else if filename_lower.contains("float") || content.contains("fpu.") {
        "algorithms".to_string()
    } else if filename_lower.contains("random") || content.contains("rand.") {
        "algorithms".to_string()
    } else if filename_lower.contains("io")
        || content.contains("file.")
        || content.contains("io.print")
    {
        "io".to_string()
    } else if content.contains("call ") || content.contains("ret\n") || content.contains("ret\r") {
        "functions".to_string()
    } else if filename_lower.contains("fib")
        || filename_lower.contains("fact")
        || filename_lower.contains("gcd")
        || filename_lower.contains("prime")
    {
        "algorithms".to_string()
    } else if content.contains("loop:") || content.contains("blt ") || content.contains("bge ") {
        "loops".to_string()
    } else if content.contains("beq ") || content.contains("bne ") {
        "conditionals".to_string()
    } else if content.contains("load.") || content.contains("store.") {
        "memory".to_string()
    } else {
        "arithmetic".to_string()
    }
}

/// Estimate the complexity level (1-5) of an example
fn estimate_complexity(content: &str) -> u8 {
    let lines: Vec<_> = content
        .lines()
        .filter(|l| !l.trim().starts_with(';') && !l.trim().is_empty())
        .collect();

    let instruction_count = lines.len();
    let has_functions = content.contains("call ") && content.contains("ret");
    let has_loops = content.contains("loop:") || content.contains("_loop:");
    let has_io = content.contains("file.") || content.contains("net.") || content.contains("io.");
    let has_concurrency = content.contains("spawn") || content.contains("chan.");
    let has_security = content.contains("cap.") || content.contains("taint");

    if has_io && instruction_count > 100 {
        5
    } else if has_concurrency || has_security {
        5
    } else if has_io {
        4
    } else if has_functions && has_loops {
        4
    } else if has_functions || (has_loops && instruction_count > 30) {
        3
    } else if has_loops {
        2
    } else {
        1
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("Neurlang Training Data Generator");
    println!("================================");
    println!("Output: {}", args.output);
    println!("Synthetic Examples: {}", args.num_examples);
    println!("Seed: {}", args.seed);
    println!("Curriculum Level: {}", args.curriculum_level);
    println!("Include Real Examples: {}", args.include_examples);
    println!(
        "Output Format: {}",
        if args.parallel {
            "parallel (instruction-level)"
        } else {
            "legacy (binary IR)"
        }
    );
    println!();

    let mut generator = Generator::new(args.seed, args.curriculum_level);

    let file = File::create(&args.output)?;
    let mut writer = BufWriter::new(file);

    let mut stats: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut total_examples = 0usize;

    // Handle parallel format differently
    if args.parallel {
        // Parallel format: instruction-level data for 64-slot parallel model
        println!(
            "\nGenerating {} parallel format examples...",
            args.num_examples
        );
        let mut skipped = 0usize;
        let mut generated = 0usize;

        while generated < args.num_examples {
            match generator.generate_parallel() {
                Some(example) => {
                    // Update stats
                    *stats.entry(example.category.clone()).or_insert(0) += 1;
                    total_examples += 1;
                    generated += 1;

                    // Write JSONL
                    let json = serde_json::to_string(&example)?;
                    writeln!(writer, "{}", json)?;

                    if generated.is_multiple_of(10000) {
                        println!("Generated {} examples (skipped {})...", generated, skipped);
                    }
                }
                None => {
                    skipped += 1;
                }
            }
        }

        println!("Total skipped (assembly errors): {}", skipped);
    } else {
        // Legacy format: binary IR bytes
        // First, include real examples from the examples directory
        if args.include_examples {
            println!("Loading examples from {}...", args.examples_dir);
            let real_examples = load_examples_from_directory(&args.examples_dir);
            println!("Found {} valid examples", real_examples.len());

            for example in real_examples {
                // Update stats
                *stats.entry(example.category.clone()).or_insert(0) += 1;
                total_examples += 1;

                // Write JSONL
                let json = serde_json::to_string(&example)?;
                writeln!(writer, "{}", json)?;
            }
            println!(
                "Added {} real examples from examples directory",
                total_examples
            );
        }

        // Then generate synthetic examples with binary IR output
        println!(
            "\nGenerating {} synthetic examples with binary IR...",
            args.num_examples
        );
        let mut skipped = 0usize;
        let mut generated = 0usize;

        while generated < args.num_examples {
            match generator.generate() {
                Some(example) => {
                    // Update stats
                    *stats.entry(example.category.clone()).or_insert(0) += 1;
                    total_examples += 1;
                    generated += 1;

                    // Write JSONL
                    let json = serde_json::to_string(&example)?;
                    writeln!(writer, "{}", json)?;

                    if generated.is_multiple_of(10000) {
                        println!("Generated {} examples (skipped {})...", generated, skipped);
                    }
                }
                None => {
                    // Assembly failed to convert to binary, skip
                    skipped += 1;
                }
            }
        }

        println!("Total skipped (assembly errors): {}", skipped);
    }

    writer.flush()?;

    println!("\nGeneration complete!");
    println!("Total examples: {}", total_examples);
    println!("\nCategory distribution:");
    let mut sorted_stats: Vec<_> = stats.iter().collect();
    sorted_stats.sort_by_key(|(_, v)| std::cmp::Reverse(*v));
    for (cat, count) in sorted_stats {
        println!(
            "  {}: {} ({:.1}%)",
            cat,
            count,
            (*count as f64 / total_examples as f64) * 100.0
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator() {
        let mut gen = Generator::new(42, 3);
        let mut success_count = 0;
        for _ in 0..100 {
            if let Some(example) = gen.generate() {
                assert!(!example.prompt.is_empty());
                assert!(!example.binary_ir.is_empty());
                // Verify binary starts with NRLG magic header (Neurlang)
                assert_eq!(&example.binary_ir[0..4], b"NRLG");
                success_count += 1;
            }
        }
        // Should generate at least some valid examples
        assert!(success_count > 50);
    }

    #[test]
    fn test_curriculum_levels() {
        for level in 1..=5 {
            let mut gen = Generator::new(42, level);
            // Try a few times to get a valid example
            for _ in 0..10 {
                if let Some(example) = gen.generate() {
                    assert!(example.level <= level);
                    break;
                }
            }
        }
    }

    #[test]
    fn test_parallel_generator() {
        let mut gen = Generator::new(42, 3);
        let mut success_count = 0;
        for _ in 0..100 {
            if let Some(example) = gen.generate_parallel() {
                assert!(!example.context.is_empty());
                assert_eq!(example.instructions.len(), NUM_SLOTS);
                // At least some instructions should be valid
                let valid_count: usize =
                    example.instructions.iter().map(|i| i.valid as usize).sum();
                assert!(
                    valid_count > 0,
                    "Should have at least one valid instruction"
                );
                success_count += 1;
            }
        }
        // Should generate at least some valid examples
        assert!(success_count > 50);
    }

    #[test]
    fn test_instruction_data_format() {
        let mut gen = Generator::new(123, 2);
        if let Some(example) = gen.generate_parallel() {
            for instr in &example.instructions {
                if instr.valid == 1 {
                    // Opcode should be valid (0-32)
                    assert!(instr.opcode <= 32, "Invalid opcode: {}", instr.opcode);
                    // Registers should be valid (0-31)
                    assert!(instr.rd < 32, "Invalid rd: {}", instr.rd);
                    assert!(instr.rs1 < 32, "Invalid rs1: {}", instr.rs1);
                    assert!(instr.rs2 < 32, "Invalid rs2: {}", instr.rs2);
                    // Mode should be valid (0-7)
                    assert!(instr.mode < 8, "Invalid mode: {}", instr.mode);
                }
            }
        }
    }
}
