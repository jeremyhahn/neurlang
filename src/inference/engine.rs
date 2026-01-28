//! ONNX Inference Engine for Neurlang
//!
//! Provides in-process model execution for code generation.
//! Uses ONNX Runtime when available, with fallback options.

use std::path::Path;
use std::time::{Duration, Instant};

/// Inference engine errors
#[derive(Debug, Clone)]
pub enum InferenceError {
    /// Model file not found
    ModelNotFound(String),
    /// Model loading failed
    LoadError(String),
    /// Inference failed
    InferenceError(String),
    /// Tokenization failed
    TokenizationError(String),
    /// Decoding failed
    DecodingError(String),
    /// Timeout during inference
    Timeout,
}

impl std::fmt::Display for InferenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InferenceError::ModelNotFound(p) => write!(f, "Model not found: {}", p),
            InferenceError::LoadError(e) => write!(f, "Failed to load model: {}", e),
            InferenceError::InferenceError(e) => write!(f, "Inference failed: {}", e),
            InferenceError::TokenizationError(e) => write!(f, "Tokenization failed: {}", e),
            InferenceError::DecodingError(e) => write!(f, "Decoding failed: {}", e),
            InferenceError::Timeout => write!(f, "Inference timed out"),
        }
    }
}

impl std::error::Error for InferenceError {}

/// Inference configuration
#[derive(Debug, Clone)]
pub struct InferenceConfig {
    /// Maximum output tokens
    pub max_tokens: usize,
    /// Sampling temperature (0.0 = deterministic)
    pub temperature: f32,
    /// Top-p sampling threshold
    pub top_p: f32,
    /// Inference timeout
    pub timeout: Duration,
    /// Use GPU if available
    pub use_gpu: bool,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.9,
            timeout: Duration::from_secs(30),
            use_gpu: true,
        }
    }
}

/// Inference result with timing information
#[derive(Debug)]
pub struct InferenceResult {
    /// Generated binary IR
    pub binary_ir: Vec<u8>,
    /// Number of tokens generated
    pub tokens_generated: usize,
    /// Inference latency
    pub latency: Duration,
    /// Whether GPU was used
    pub used_gpu: bool,
}

/// Abstract inference engine trait
pub trait InferenceBackend: Send + Sync {
    /// Generate binary IR from a prompt
    fn generate(
        &self,
        prompt: &str,
        config: &InferenceConfig,
    ) -> Result<InferenceResult, InferenceError>;

    /// Check if the model is loaded
    fn is_loaded(&self) -> bool;

    /// Get model info
    fn model_info(&self) -> String;
}

/// ONNX-based inference engine
pub struct InferenceEngine {
    backend: Box<dyn InferenceBackend>,
    config: InferenceConfig,
}

impl InferenceEngine {
    /// Load model from file
    pub fn load(model_path: &Path) -> Result<Self, InferenceError> {
        Self::load_with_config(model_path, InferenceConfig::default())
    }

    /// Load model with custom configuration
    ///
    /// When built with `--features onnx`, loads the real ONNX model.
    /// Otherwise, uses MockBackend for testing.
    pub fn load_with_config(
        model_path: &Path,
        config: InferenceConfig,
    ) -> Result<Self, InferenceError> {
        if !model_path.exists() {
            return Err(InferenceError::ModelNotFound(
                model_path.display().to_string(),
            ));
        }

        #[cfg(feature = "onnx")]
        let backend: Box<dyn InferenceBackend> = { Box::new(OnnxBackend::load(model_path)?) };

        #[cfg(not(feature = "onnx"))]
        let backend: Box<dyn InferenceBackend> = Box::new(MockBackend::new());

        Ok(Self { backend, config })
    }

    /// Create engine with mock backend (for testing)
    pub fn mock() -> Self {
        Self {
            backend: Box::new(MockBackend::new()),
            config: InferenceConfig::default(),
        }
    }

    /// Generate binary IR from a prompt
    pub fn generate(&self, prompt: &str) -> Result<InferenceResult, InferenceError> {
        self.backend.generate(prompt, &self.config)
    }

    /// Generate with custom config
    pub fn generate_with_config(
        &self,
        prompt: &str,
        config: &InferenceConfig,
    ) -> Result<InferenceResult, InferenceError> {
        self.backend.generate(prompt, config)
    }

    /// Check if model is loaded
    pub fn is_loaded(&self) -> bool {
        self.backend.is_loaded()
    }

    /// Get model info
    pub fn model_info(&self) -> String {
        self.backend.model_info()
    }

    /// Get current config
    pub fn config(&self) -> &InferenceConfig {
        &self.config
    }

    /// Update config
    pub fn set_config(&mut self, config: InferenceConfig) {
        self.config = config;
    }
}

/// Mock backend for testing and when ONNX is unavailable
struct MockBackend {
    loaded: bool,
}

impl MockBackend {
    fn new() -> Self {
        Self { loaded: true }
    }
}

/// Parse symbolic expressions like "5 + 3", "7 * 6", "2^5", etc.
fn parse_symbolic_expression(prompt: &str) -> Option<(String, i32, i32)> {
    // Try to match patterns: N op M
    let prompt = prompt.trim();

    // Check for power notation: 2^5, 2**5, 2 ^ 5, 2 ** 5
    if let Some((base, exp)) = parse_power_expr(prompt) {
        return Some(("^".to_string(), base, exp));
    }

    // Standard operators: +, -, *, /, %, &, |
    for op in &[" + ", " - ", " * ", " / ", " % ", " & ", " | "] {
        if let Some(pos) = prompt.find(op) {
            let left = prompt[..pos].trim();
            let right = prompt[pos + op.len()..].trim();

            // Extract the last number from left side
            let a = extract_last_number(left)?;
            // Extract the first number from right side
            let b = extract_first_number(right)?;

            return Some((op.trim().to_string(), a, b));
        }
    }

    None
}

/// Parse power expressions like "2^5", "2**5"
fn parse_power_expr(prompt: &str) -> Option<(i32, i32)> {
    // Handle 2^5 or 2**5
    if let Some(pos) = prompt.find("**") {
        let left = prompt[..pos].trim();
        let right = prompt[pos + 2..].trim();
        let a = extract_last_number(left)?;
        let b = extract_first_number(right)?;
        return Some((a, b));
    }
    if let Some(pos) = prompt.find('^') {
        let left = prompt[..pos].trim();
        let right = prompt[pos + 1..].trim();
        let a = extract_last_number(left)?;
        let b = extract_first_number(right)?;
        return Some((a, b));
    }
    None
}

/// Extract the last number from a string
fn extract_last_number(s: &str) -> Option<i32> {
    // Find all digit sequences and take the last one
    let mut last_num: Option<i32> = None;
    let mut current = String::new();
    let mut in_number = false;

    for c in s.chars() {
        if c.is_ascii_digit() || (c == '-' && !in_number) {
            current.push(c);
            in_number = true;
        } else if in_number {
            if let Ok(n) = current.parse::<i32>() {
                last_num = Some(n);
            }
            current.clear();
            in_number = false;
        }
    }

    if in_number && !current.is_empty() {
        if let Ok(n) = current.parse::<i32>() {
            last_num = Some(n);
        }
    }

    last_num
}

/// Extract the first number from a string
fn extract_first_number(s: &str) -> Option<i32> {
    let mut current = String::new();
    let mut in_number = false;

    for c in s.chars() {
        if c.is_ascii_digit() || (c == '-' && !in_number) {
            current.push(c);
            in_number = true;
        } else if in_number {
            break;
        }
    }

    if !current.is_empty() {
        current.parse::<i32>().ok()
    } else {
        None
    }
}

impl InferenceBackend for MockBackend {
    fn generate(
        &self,
        prompt: &str,
        _config: &InferenceConfig,
    ) -> Result<InferenceResult, InferenceError> {
        let start = Instant::now();

        // Generate simple mock program based on prompt keywords
        let binary_ir = generate_mock_program(prompt);

        Ok(InferenceResult {
            binary_ir,
            tokens_generated: 10,
            latency: start.elapsed(),
            used_gpu: false,
        })
    }

    fn is_loaded(&self) -> bool {
        self.loaded
    }

    fn model_info(&self) -> String {
        "Mock Backend (no ONNX)".to_string()
    }
}

/// Generate a mock program based on prompt keywords
fn generate_mock_program(prompt: &str) -> Vec<u8> {
    use crate::ir::{
        AluOp, BitsOp, BranchCond, FpuOp, Instruction, IoOp, MulDivOp, Opcode, Program, RandOp,
        Register, TimeOp,
    };

    let mut program = Program::new();
    let prompt_lower = prompt.to_lowercase();

    // Extract numbers from prompt (handle both positive and negative)
    let numbers: Vec<i32> = prompt_lower
        .split(|c: char| !c.is_ascii_digit() && c != '-')
        .filter_map(|s| s.parse().ok())
        .collect();
    let n = numbers.first().copied().unwrap_or(10);

    // Check for symbolic patterns FIRST (5 + 3, 7 * 6, etc.)
    // These need to be detected before keyword-based patterns
    let symbolic_result = parse_symbolic_expression(&prompt_lower);
    if let Some((op, a, b)) = symbolic_result {
        match op.as_str() {
            "+" => {
                program.instructions.push(Instruction::with_imm(
                    Opcode::Mov,
                    Register::R0,
                    Register::Zero,
                    0,
                    a,
                ));
                program.instructions.push(Instruction::with_imm(
                    Opcode::Mov,
                    Register::R1,
                    Register::Zero,
                    0,
                    b,
                ));
                program.instructions.push(Instruction::new(
                    Opcode::Alu,
                    Register::R0,
                    Register::R0,
                    Register::R1,
                    AluOp::Add as u8,
                ));
            }
            "-" => {
                program.instructions.push(Instruction::with_imm(
                    Opcode::Mov,
                    Register::R0,
                    Register::Zero,
                    0,
                    a,
                ));
                program.instructions.push(Instruction::with_imm(
                    Opcode::Mov,
                    Register::R1,
                    Register::Zero,
                    0,
                    b,
                ));
                program.instructions.push(Instruction::new(
                    Opcode::Alu,
                    Register::R0,
                    Register::R0,
                    Register::R1,
                    AluOp::Sub as u8,
                ));
            }
            "*" => {
                program.instructions.push(Instruction::with_imm(
                    Opcode::Mov,
                    Register::R0,
                    Register::Zero,
                    0,
                    a,
                ));
                program.instructions.push(Instruction::with_imm(
                    Opcode::Mov,
                    Register::R1,
                    Register::Zero,
                    0,
                    b,
                ));
                program.instructions.push(Instruction::new(
                    Opcode::MulDiv,
                    Register::R0,
                    Register::R0,
                    Register::R1,
                    MulDivOp::Mul as u8,
                ));
            }
            "/" => {
                program.instructions.push(Instruction::with_imm(
                    Opcode::Mov,
                    Register::R0,
                    Register::Zero,
                    0,
                    a,
                ));
                program.instructions.push(Instruction::with_imm(
                    Opcode::Mov,
                    Register::R1,
                    Register::Zero,
                    0,
                    b,
                ));
                program.instructions.push(Instruction::new(
                    Opcode::MulDiv,
                    Register::R0,
                    Register::R0,
                    Register::R1,
                    MulDivOp::Div as u8,
                ));
            }
            "%" => {
                program.instructions.push(Instruction::with_imm(
                    Opcode::Mov,
                    Register::R0,
                    Register::Zero,
                    0,
                    a,
                ));
                program.instructions.push(Instruction::with_imm(
                    Opcode::Mov,
                    Register::R1,
                    Register::Zero,
                    0,
                    b,
                ));
                program.instructions.push(Instruction::new(
                    Opcode::MulDiv,
                    Register::R0,
                    Register::R0,
                    Register::R1,
                    MulDivOp::Mod as u8,
                ));
            }
            "&" => {
                program.instructions.push(Instruction::with_imm(
                    Opcode::Mov,
                    Register::R0,
                    Register::Zero,
                    0,
                    a,
                ));
                program.instructions.push(Instruction::with_imm(
                    Opcode::Mov,
                    Register::R1,
                    Register::Zero,
                    0,
                    b,
                ));
                program.instructions.push(Instruction::new(
                    Opcode::Alu,
                    Register::R0,
                    Register::R0,
                    Register::R1,
                    AluOp::And as u8,
                ));
            }
            "|" => {
                program.instructions.push(Instruction::with_imm(
                    Opcode::Mov,
                    Register::R0,
                    Register::Zero,
                    0,
                    a,
                ));
                program.instructions.push(Instruction::with_imm(
                    Opcode::Mov,
                    Register::R1,
                    Register::Zero,
                    0,
                    b,
                ));
                program.instructions.push(Instruction::new(
                    Opcode::Alu,
                    Register::R0,
                    Register::R0,
                    Register::R1,
                    AluOp::Or as u8,
                ));
            }
            "^" | "**" => {
                // Power operation
                program.instructions.push(Instruction::with_imm(
                    Opcode::Mov,
                    Register::R0,
                    Register::Zero,
                    0,
                    1, // result = 1
                ));
                program.instructions.push(Instruction::with_imm(
                    Opcode::Mov,
                    Register::R1,
                    Register::Zero,
                    0,
                    a, // base
                ));
                program.instructions.push(Instruction::with_imm(
                    Opcode::Mov,
                    Register::R2,
                    Register::Zero,
                    0,
                    b, // exponent
                ));
                // Loop: result *= base; exp--
                program.instructions.push(Instruction::new(
                    Opcode::MulDiv,
                    Register::R0,
                    Register::R0,
                    Register::R1,
                    MulDivOp::Mul as u8,
                ));
                program.instructions.push(Instruction::with_imm(
                    Opcode::AluI,
                    Register::R2,
                    Register::R2,
                    AluOp::Sub as u8,
                    1,
                ));
                program.instructions.push(Instruction::with_imm(
                    Opcode::Branch,
                    Register::Zero,
                    Register::R2,
                    5,
                    -2, // bgt r2, zero, loop
                ));
            }
            _ => {}
        }
        // Add halt and return
        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));
        return program.encode();
    }

    // Check for n! notation (factorial)
    if prompt_lower.contains('!') {
        let n = numbers.first().copied().unwrap_or(5);
        if n <= 0 {
            // factorial(0) = 1
            program.instructions.push(Instruction::with_imm(
                Opcode::Mov,
                Register::R0,
                Register::Zero,
                0,
                1,
            ));
        } else {
            program.instructions.push(Instruction::with_imm(
                Opcode::Mov,
                Register::R0,
                Register::Zero,
                0,
                n.min(12),
            ));
            program.instructions.push(Instruction::with_imm(
                Opcode::Mov,
                Register::R1,
                Register::Zero,
                0,
                1,
            ));
            // Loop
            program.instructions.push(Instruction::new(
                Opcode::MulDiv,
                Register::R1,
                Register::R1,
                Register::R0,
                MulDivOp::Mul as u8,
            ));
            program.instructions.push(Instruction::with_imm(
                Opcode::AluI,
                Register::R0,
                Register::R0,
                AluOp::Sub as u8,
                1,
            ));
            program.instructions.push(Instruction::with_imm(
                Opcode::Branch,
                Register::Zero,
                Register::R0,
                5,
                -2,
            ));
            program.instructions.push(Instruction::new(
                Opcode::Mov,
                Register::R0,
                Register::R1,
                Register::Zero,
                0,
            ));
        }
        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));
        return program.encode();
    }

    // Parse patterns from prompt
    if prompt_lower.contains("fibonacci") || prompt_lower.contains("fib") {
        // Generate fibonacci program - fib(n) where fib(0)=0, fib(1)=1, fib(2)=1, etc.
        // Edge cases first
        if n <= 0 {
            // fib(0) = 0
            program.instructions.push(Instruction::with_imm(
                Opcode::Mov,
                Register::R0,
                Register::Zero,
                0,
                0,
            ));
        } else if n == 1 {
            // fib(1) = 1
            program.instructions.push(Instruction::with_imm(
                Opcode::Mov,
                Register::R0,
                Register::Zero,
                0,
                1,
            ));
        } else {
            // fib(n) for n >= 2
            // r0 = counter (n-1 iterations), r1 = fib(i), r2 = fib(i+1)
            program.instructions.push(Instruction::with_imm(
                Opcode::Mov,
                Register::R0,
                Register::Zero,
                0,
                n - 1, // n-1 iterations
            ));
            program.instructions.push(Instruction::with_imm(
                Opcode::Mov,
                Register::R1,
                Register::Zero,
                0,
                0, // fib(0)
            ));
            program.instructions.push(Instruction::with_imm(
                Opcode::Mov,
                Register::R2,
                Register::Zero,
                0,
                1, // fib(1)
            ));
            // Loop: r3 = r1 + r2; r1 = r2; r2 = r3; r0--
            program.instructions.push(Instruction::new(
                Opcode::Alu,
                Register::R3,
                Register::R1,
                Register::R2,
                AluOp::Add as u8,
            ));
            program.instructions.push(Instruction::new(
                Opcode::Mov,
                Register::R1,
                Register::R2,
                Register::Zero,
                0,
            ));
            program.instructions.push(Instruction::new(
                Opcode::Mov,
                Register::R2,
                Register::R3,
                Register::Zero,
                0,
            ));
            program.instructions.push(Instruction::with_imm(
                Opcode::AluI,
                Register::R0,
                Register::R0,
                AluOp::Sub as u8,
                1,
            ));
            // Branch back if R0 > 0 (loop body is 4 instructions)
            program.instructions.push(Instruction::with_imm(
                Opcode::Branch,
                Register::Zero,
                Register::R0,
                5,
                -4, // bgt r0, zero, -4
            ));
            // Result is in r2 (fib(n))
            program.instructions.push(Instruction::new(
                Opcode::Mov,
                Register::R0,
                Register::R2,
                Register::Zero,
                0,
            ));
        }
    } else if prompt_lower.contains("factorial") || prompt_lower.contains("fact") {
        // Generate factorial program - factorial(0) = 1
        if n <= 0 {
            program.instructions.push(Instruction::with_imm(
                Opcode::Mov,
                Register::R0,
                Register::Zero,
                0,
                1, // 0! = 1
            ));
        } else if n == 1 {
            program.instructions.push(Instruction::with_imm(
                Opcode::Mov,
                Register::R0,
                Register::Zero,
                0,
                1, // 1! = 1
            ));
        } else {
            program.instructions.push(Instruction::with_imm(
                Opcode::Mov,
                Register::R0,
                Register::Zero,
                0,
                n.min(12), // Limit to avoid overflow
            ));
            program.instructions.push(Instruction::with_imm(
                Opcode::Mov,
                Register::R1,
                Register::Zero,
                0,
                1,
            ));
            // Loop
            program.instructions.push(Instruction::new(
                Opcode::MulDiv,
                Register::R1,
                Register::R1,
                Register::R0,
                MulDivOp::Mul as u8,
            ));
            program.instructions.push(Instruction::with_imm(
                Opcode::AluI,
                Register::R0,
                Register::R0,
                AluOp::Sub as u8,
                1,
            ));
            // Branch back if R0 > 0
            program.instructions.push(Instruction::with_imm(
                Opcode::Branch,
                Register::Zero,
                Register::R0,
                5,
                -2,
            ));
            program.instructions.push(Instruction::new(
                Opcode::Mov,
                Register::R0,
                Register::R1,
                Register::Zero,
                0,
            ));
        }
    } else if prompt_lower.contains("squared") || prompt_lower.contains("square of") {
        // n squared = n * n
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            n,
        ));
        program.instructions.push(Instruction::new(
            Opcode::MulDiv,
            Register::R0,
            Register::R0,
            Register::R0,
            MulDivOp::Mul as u8,
        ));
    } else if prompt_lower.contains("cubed") || prompt_lower.contains("cube of") {
        // n cubed = n * n * n
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            n,
        ));
        program.instructions.push(Instruction::new(
            Opcode::MulDiv,
            Register::R1,
            Register::R0,
            Register::R0,
            MulDivOp::Mul as u8,
        ));
        program.instructions.push(Instruction::new(
            Opcode::MulDiv,
            Register::R0,
            Register::R1,
            Register::R0,
            MulDivOp::Mul as u8,
        ));
    } else if prompt_lower.contains("to the fourth") {
        // n^4
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            n,
        ));
        program.instructions.push(Instruction::new(
            Opcode::MulDiv,
            Register::R0,
            Register::R0,
            Register::R0,
            MulDivOp::Mul as u8,
        ));
        program.instructions.push(Instruction::new(
            Opcode::MulDiv,
            Register::R0,
            Register::R0,
            Register::R0,
            MulDivOp::Mul as u8,
        ));
    } else if prompt_lower.contains("to the power")
        || prompt_lower.contains("raised to")
        || prompt_lower.contains("power of")
    {
        // Power operation - extract base and exponent
        let base = numbers.first().copied().unwrap_or(2);
        let exp = numbers.get(1).copied().unwrap_or(3);
        // Generate power loop: result = 1; for i in 0..exp: result *= base
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            1, // result
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            0,
            base, // base
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R2,
            Register::Zero,
            0,
            exp, // counter
        ));
        // Loop: result *= base; counter--
        program.instructions.push(Instruction::new(
            Opcode::MulDiv,
            Register::R0,
            Register::R0,
            Register::R1,
            MulDivOp::Mul as u8,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::AluI,
            Register::R2,
            Register::R2,
            AluOp::Sub as u8,
            1,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Branch,
            Register::Zero,
            Register::R2,
            5,
            -2, // bgt r2, zero, loop
        ));
    } else if prompt_lower.contains("sqrt") || prompt_lower.contains("square root") {
        // Floating-point square root
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            n,
        ));
        // Convert int to float (simplified - use immediate as float bits)
        program.instructions.push(Instruction::new(
            Opcode::Fpu,
            Register::R1,
            Register::R0,
            Register::Zero,
            FpuOp::Fsqrt as u8,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Mov,
            Register::R0,
            Register::R1,
            Register::Zero,
            0,
        ));
    } else if prompt_lower.contains("popcount")
        || prompt_lower.contains("count bits")
        || prompt_lower.contains("set bits")
    {
        // Count set bits
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            n,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Bits,
            Register::R1,
            Register::R0,
            Register::Zero,
            BitsOp::Popcount as u8,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Mov,
            Register::R0,
            Register::R1,
            Register::Zero,
            0,
        ));
    } else if prompt_lower.contains("leading zero") || prompt_lower.contains("clz") {
        // Count leading zeros
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            n,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Bits,
            Register::R1,
            Register::R0,
            Register::Zero,
            BitsOp::Clz as u8,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Mov,
            Register::R0,
            Register::R1,
            Register::Zero,
            0,
        ));
    } else if prompt_lower.contains("trailing zero") || prompt_lower.contains("ctz") {
        // Count trailing zeros
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            n,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Bits,
            Register::R1,
            Register::R0,
            Register::Zero,
            BitsOp::Ctz as u8,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Mov,
            Register::R0,
            Register::R1,
            Register::Zero,
            0,
        ));
    } else if prompt_lower.contains("byte swap")
        || prompt_lower.contains("endian")
        || prompt_lower.contains("bswap")
    {
        // Byte swap
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            n,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Bits,
            Register::R1,
            Register::R0,
            Register::Zero,
            BitsOp::Bswap as u8,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Mov,
            Register::R0,
            Register::R1,
            Register::Zero,
            0,
        ));
    } else if prompt_lower.contains("random") || prompt_lower.contains("rand") {
        // Random number
        program.instructions.push(Instruction::new(
            Opcode::Rand,
            Register::R0,
            Register::Zero,
            Register::Zero,
            RandOp::RandU64 as u8,
        ));
    } else if prompt_lower.contains("time")
        || prompt_lower.contains("timestamp")
        || prompt_lower.contains("now")
    {
        // Get current timestamp
        program.instructions.push(Instruction::new(
            Opcode::Time,
            Register::R0,
            Register::Zero,
            Register::Zero,
            TimeOp::Now as u8,
        ));
    } else if prompt_lower.contains("sleep")
        || prompt_lower.contains("wait")
        || prompt_lower.contains("delay")
    {
        // Sleep for n milliseconds
        let ms = if n > 0 { n } else { 100 };
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            ms,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Time,
            Register::Zero,
            Register::R0,
            TimeOp::Sleep as u8,
            ms,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            0, // Return 0 on success
        ));
    } else if prompt_lower.contains("multiply") || prompt_lower.contains("product") {
        // Multiplication
        let a = numbers.first().copied().unwrap_or(6);
        let b = numbers.get(1).copied().unwrap_or(7);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            a,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            0,
            b,
        ));
        program.instructions.push(Instruction::new(
            Opcode::MulDiv,
            Register::R0,
            Register::R0,
            Register::R1,
            MulDivOp::Mul as u8,
        ));
    } else if prompt_lower.contains("divide") || prompt_lower.contains("division") {
        // Division
        let a = numbers.first().copied().unwrap_or(100);
        let b = numbers.get(1).copied().unwrap_or(5);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            a,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            0,
            b,
        ));
        program.instructions.push(Instruction::new(
            Opcode::MulDiv,
            Register::R0,
            Register::R0,
            Register::R1,
            MulDivOp::Div as u8,
        ));
    } else if prompt_lower.contains("mod") || prompt_lower.contains("remainder") {
        // Modulo
        let a = numbers.first().copied().unwrap_or(17);
        let b = numbers.get(1).copied().unwrap_or(5);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            a,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            0,
            b,
        ));
        program.instructions.push(Instruction::new(
            Opcode::MulDiv,
            Register::R0,
            Register::R0,
            Register::R1,
            MulDivOp::Mod as u8,
        ));
    } else if prompt_lower.contains("power")
        || prompt_lower.contains("exponent")
        || prompt_lower.contains("pow")
    {
        // Power of 2 (simple shift-based)
        let exp = numbers.first().copied().unwrap_or(3).min(30);
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
            exp,
        ));
    } else if prompt_lower.contains("add")
        || prompt_lower.contains("sum")
        || prompt_lower.contains("plus")
    {
        // Addition
        let a = numbers.first().copied().unwrap_or(10);
        let b = numbers.get(1).copied().unwrap_or(20);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            a,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            0,
            b,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Alu,
            Register::R0,
            Register::R0,
            Register::R1,
            AluOp::Add as u8,
        ));
    } else if prompt_lower.contains("subtract")
        || prompt_lower.contains("minus")
        || prompt_lower.contains("difference")
    {
        // Subtraction
        let a = numbers.first().copied().unwrap_or(100);
        let b = numbers.get(1).copied().unwrap_or(30);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            a,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            0,
            b,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Alu,
            Register::R0,
            Register::R0,
            Register::R1,
            AluOp::Sub as u8,
        ));
    } else if (prompt_lower.contains("bitwise") && prompt_lower.contains("and"))
        || (prompt_lower.contains("and of") && numbers.len() >= 2)
    {
        // Bitwise AND
        let a = numbers.first().copied().unwrap_or(0xFF);
        let b = numbers.get(1).copied().unwrap_or(0x0F);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            a,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            0,
            b,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Alu,
            Register::R0,
            Register::R0,
            Register::R1,
            AluOp::And as u8,
        ));
    } else if (prompt_lower.contains("bitwise") && prompt_lower.contains(" or"))
        || (prompt_lower.contains("or of") && numbers.len() >= 2)
    {
        // Bitwise OR
        let a = numbers.first().copied().unwrap_or(0xF0);
        let b = numbers.get(1).copied().unwrap_or(0x0F);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            a,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            0,
            b,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Alu,
            Register::R0,
            Register::R0,
            Register::R1,
            AluOp::Or as u8,
        ));
    } else if prompt_lower.contains("xor of")
        || (prompt_lower.contains("xor") && numbers.len() >= 2)
        || (prompt_lower.contains("bitwise") && prompt_lower.contains("xor"))
    {
        // Bitwise XOR
        let a = numbers.first().copied().unwrap_or(0xFF);
        let b = numbers.get(1).copied().unwrap_or(0xAA);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            a,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            0,
            b,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Alu,
            Register::R0,
            Register::R0,
            Register::R1,
            AluOp::Xor as u8,
        ));
    } else if prompt_lower.contains(" times ") {
        // "7 times 6" style multiplication
        let a = numbers.first().copied().unwrap_or(6);
        let b = numbers.get(1).copied().unwrap_or(7);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            a,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            0,
            b,
        ));
        program.instructions.push(Instruction::new(
            Opcode::MulDiv,
            Register::R0,
            Register::R0,
            Register::R1,
            MulDivOp::Mul as u8,
        ));
    } else if prompt_lower.contains("shift left") || prompt_lower.contains("shl") {
        // Shift left
        let val = numbers.first().copied().unwrap_or(1);
        let amt = numbers.get(1).copied().unwrap_or(4);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            val,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::AluI,
            Register::R0,
            Register::R0,
            AluOp::Shl as u8,
            amt,
        ));
    } else if prompt_lower.contains("shift right") || prompt_lower.contains("shr") {
        // Shift right
        let val = numbers.first().copied().unwrap_or(256);
        let amt = numbers.get(1).copied().unwrap_or(4);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            val,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::AluI,
            Register::R0,
            Register::R0,
            AluOp::Shr as u8,
            amt,
        ));
    } else if prompt_lower.contains("gcd") || prompt_lower.contains("greatest common") {
        // GCD using Euclidean algorithm
        let a = numbers.first().copied().unwrap_or(48);
        let b = numbers.get(1).copied().unwrap_or(18);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            a,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            0,
            b,
        ));
        // Loop: while r1 != 0
        // Indices: 2=beq, 3=mod, 4=mov, 5=mov, 6=branch, 7=halt
        // beq jumps to halt (index 7), which is +5 from index 2
        program.instructions.push(Instruction::with_imm(
            Opcode::Branch,
            Register::Zero,
            Register::R1,
            1,
            5, // beq r1, zero, end
        ));
        // r2 = r0 % r1
        program.instructions.push(Instruction::new(
            Opcode::MulDiv,
            Register::R2,
            Register::R0,
            Register::R1,
            MulDivOp::Mod as u8,
        ));
        // r0 = r1
        program.instructions.push(Instruction::new(
            Opcode::Mov,
            Register::R0,
            Register::R1,
            Register::Zero,
            0,
        ));
        // r1 = r2
        program.instructions.push(Instruction::new(
            Opcode::Mov,
            Register::R1,
            Register::R2,
            Register::Zero,
            0,
        ));
        // branch back to beq (index 6 -> index 2 = -4)
        program.instructions.push(Instruction::with_imm(
            Opcode::Branch,
            Register::Zero,
            Register::Zero,
            0,
            -4,
        ));
    } else if prompt_lower.contains("max")
        || prompt_lower.contains("maximum")
        || prompt_lower.contains("larger")
    {
        // Find maximum of two numbers
        let a = numbers.first().copied().unwrap_or(10);
        let b = numbers.get(1).copied().unwrap_or(20);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            a,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            0,
            b,
        ));
        // if r0 >= r1, skip (index 2 -> index 4 = +2)
        program.instructions.push(Instruction::branch(
            BranchCond::Ge,
            Register::R0,
            Register::R1,
            2,
        ));
        // r0 = r1
        program.instructions.push(Instruction::new(
            Opcode::Mov,
            Register::R0,
            Register::R1,
            Register::Zero,
            0,
        ));
    } else if prompt_lower.contains("min")
        || prompt_lower.contains("minimum")
        || prompt_lower.contains("smaller")
    {
        // Find minimum of two numbers
        let a = numbers.first().copied().unwrap_or(10);
        let b = numbers.get(1).copied().unwrap_or(20);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            a,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            0,
            b,
        ));
        // if r0 <= r1, skip (index 2 -> index 4 = +2)
        program.instructions.push(Instruction::branch(
            BranchCond::Le,
            Register::R0,
            Register::R1,
            2,
        ));
        // r0 = r1
        program.instructions.push(Instruction::new(
            Opcode::Mov,
            Register::R0,
            Register::R1,
            Register::Zero,
            0,
        ));
    } else if prompt_lower.contains("abs") || prompt_lower.contains("absolute") {
        // Absolute value
        let val = numbers.first().copied().unwrap_or(-10);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            val,
        ));
        // if r0 >= 0, skip (index 1 -> index 3 = +2)
        program.instructions.push(Instruction::with_imm(
            Opcode::Branch,
            Register::R0,
            Register::Zero,
            6,
            2, // bge r0, zero, skip
        ));
        // r0 = 0 - r0
        program.instructions.push(Instruction::new(
            Opcode::Alu,
            Register::R0,
            Register::Zero,
            Register::R0,
            AluOp::Sub as u8,
        ));
    } else if prompt_lower.contains("print")
        || prompt_lower.contains("hello")
        || prompt_lower.contains("output")
    {
        // Print operation
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            0,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            0,
            13, // "Hello, World!" length
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Io,
            Register::Zero,
            Register::R0,
            IoOp::Print as u8,
            13,
        ));
        program.data_section = b"Hello, World!".to_vec();
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            0,
        ));
    } else if prompt_lower.contains("count") && prompt_lower.contains("loop") {
        // Count from 0 to n
        let limit = numbers.first().copied().unwrap_or(10);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            0, // counter
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            0,
            limit, // limit
        ));
        // Loop: r0 = r0 + 1
        program.instructions.push(Instruction::with_imm(
            Opcode::AluI,
            Register::R0,
            Register::R0,
            AluOp::Add as u8,
            1,
        ));
        // Branch back if r0 < r1 (index 3 -> index 2 = -1)
        program.instructions.push(Instruction::branch(
            BranchCond::Lt,
            Register::R0,
            Register::R1,
            -1,
        ));
    } else if prompt_lower.contains("prime") {
        // Check if n is prime (simplified - just returns 1 for primes 2-7)
        let num = numbers.first().copied().unwrap_or(7);
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            num,
        ));
        // Very simplified prime check
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            0,
            1, // assume prime
        ));
    } else {
        // Default: just return the number or 42
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            n,
        ));
    }

    // Always add halt
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    program.encode()
}

// ONNX Runtime backend for real model inference
#[cfg(feature = "onnx")]
mod onnx_backend {
    use super::*;
    use ort::session::Session;
    use ort::value::Tensor;
    use std::path::Path;
    use std::sync::Mutex;

    /// ONNX Runtime backend for real model inference
    ///
    /// The model outputs binary IR bytes DIRECTLY (not assembly text).
    /// Architecture: prompt bytes → model → binary IR bytes → JIT
    ///
    /// Binary IR format starts with "NRLG" magic bytes (78, 82, 76, 71) for Neurlang.
    pub struct OnnxBackend {
        session: Mutex<Session>,
        block_size: usize,
    }

    impl OnnxBackend {
        pub fn load(model_path: &Path) -> Result<Self, InferenceError> {
            // Load model with ort 2.0 API
            let session = Session::builder()
                .map_err(|e| InferenceError::LoadError(format!("Session builder error: {}", e)))?
                .commit_from_file(model_path)
                .map_err(|e| InferenceError::LoadError(format!("Failed to load model: {}", e)))?;

            Ok(Self {
                session: Mutex::new(session),
                block_size: 512, // Match training config
            })
        }

        /// Generate binary IR directly from prompt using autoregressive generation
        ///
        /// The model outputs raw bytes that ARE the binary IR (starting with "NRLG" magic).
        fn generate_binary(
            &self,
            prompt: &str,
            config: &InferenceConfig,
        ) -> Result<Vec<u8>, InferenceError> {
            // Tokenize: prompt bytes + newline separator
            let mut tokens: Vec<i64> = prompt.bytes().map(|b| b as i64).collect();
            tokens.push(b'\n' as i64);

            let prompt_len = tokens.len();

            // Autoregressive generation
            for _ in 0..config.max_tokens {
                // Prepare input (batch=1, seq_len)
                let seq_len = tokens.len().min(self.block_size);
                let input_tokens: Vec<i64> = if tokens.len() > self.block_size {
                    tokens[tokens.len() - self.block_size..].to_vec()
                } else {
                    tokens.clone()
                };

                // Create tensor for input
                let shape = vec![1i64, input_tokens.len() as i64];
                let input_tensor = Tensor::from_array((shape, input_tokens))
                    .map_err(|e| InferenceError::InferenceError(format!("Tensor error: {}", e)))?;

                // Run inference
                let mut session = self.session.lock().unwrap();
                let outputs = session
                    .run(ort::inputs![input_tensor])
                    .map_err(|e| InferenceError::InferenceError(format!("Run error: {}", e)))?;

                // Get first output (logits)
                let logits_output = &outputs[0];

                // Extract tensor data - returns (shape, data slice)
                let (logits_shape, logits_data) = logits_output
                    .try_extract_tensor::<f32>()
                    .map_err(|e| InferenceError::InferenceError(format!("Extract error: {}", e)))?;

                // Get shape info - Shape derefs to &[i64]
                if logits_shape.len() != 3 {
                    return Err(InferenceError::InferenceError(format!(
                        "Expected 3D logits, got {}D",
                        logits_shape.len()
                    )));
                }
                let vocab_size = logits_shape[2] as usize;
                let last_pos = seq_len - 1;

                // Get logits for last position - data is a flat slice
                // Index: [0, last_pos, vocab_idx] = 0 * dims[1] * dims[2] + last_pos * dims[2] + vocab_idx
                let offset = last_pos * vocab_size;
                let mut last_logits: Vec<f32> = logits_data[offset..offset + vocab_size].to_vec();

                // Apply temperature
                if config.temperature > 0.0 {
                    for l in &mut last_logits {
                        *l /= config.temperature;
                    }
                }

                // Softmax
                let max_logit = last_logits
                    .iter()
                    .cloned()
                    .fold(f32::NEG_INFINITY, f32::max);
                let exp_sum: f32 = last_logits.iter().map(|&l| (l - max_logit).exp()).sum();
                let probs: Vec<f32> = last_logits
                    .iter()
                    .map(|&l| (l - max_logit).exp() / exp_sum)
                    .collect();

                // Sample next token (greedy)
                let next_token: usize = probs
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                    .map(|(i, _)| i)
                    .unwrap_or(0);

                // Add token to sequence (don't stop on 0 - zeros are valid in binary IR)
                tokens.push(next_token as i64);

                // Check for end of generation patterns in binary IR
                let generated: Vec<u8> = tokens[prompt_len..].iter().map(|&t| t as u8).collect();
                let len = generated.len();

                // Binary IR format: NRLG header (8 bytes) + size field (8 bytes) + instructions
                // Training data shows all programs end with 0x7F (127) byte
                // Don't stop on zeros - they're valid in binary IR
                if len >= 16 && generated[len - 1] == 127 {
                    break;
                }
            }

            // Extract generated binary IR (after prompt)
            let binary_ir: Vec<u8> = tokens[prompt_len..].iter().map(|&t| t as u8).collect();

            // Strip trailing zeros (padding)
            let binary_ir: Vec<u8> = binary_ir
                .into_iter()
                .rev()
                .skip_while(|&b| b == 0)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();

            // Validate binary IR starts with "NRLG" magic
            if binary_ir.len() < 4 || &binary_ir[0..4] != b"NRLG" {
                return Err(InferenceError::DecodingError(format!(
                    "Invalid binary IR: expected NRLG magic, got {:?}",
                    binary_ir.get(0..4.min(binary_ir.len()))
                )));
            }

            Ok(binary_ir)
        }
    }

    impl InferenceBackend for OnnxBackend {
        fn generate(
            &self,
            prompt: &str,
            config: &InferenceConfig,
        ) -> Result<InferenceResult, InferenceError> {
            let start = Instant::now();

            // Generate binary IR directly (model outputs binary, not text)
            let binary_ir = self.generate_binary(prompt, config)?;
            let tokens_generated = binary_ir.len();

            Ok(InferenceResult {
                binary_ir,
                tokens_generated,
                latency: start.elapsed(),
                used_gpu: false, // ONNX Runtime handles CPU/GPU automatically
            })
        }

        fn is_loaded(&self) -> bool {
            true
        }

        fn model_info(&self) -> String {
            "ONNX Runtime Backend".to_string()
        }
    }
}

#[cfg(feature = "onnx")]
pub use onnx_backend::OnnxBackend;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_backend() {
        let engine = InferenceEngine::mock();
        assert!(engine.is_loaded());

        let result = engine.generate("compute fibonacci(10)").unwrap();
        assert!(!result.binary_ir.is_empty());
    }

    #[test]
    fn test_fibonacci_generation() {
        let binary = generate_mock_program("compute fibonacci");
        assert!(!binary.is_empty());

        // Verify it starts with NRLG magic
        assert_eq!(&binary[0..4], b"NRLG");
    }

    #[test]
    fn test_factorial_generation() {
        let binary = generate_mock_program("calculate factorial of 5");
        assert!(!binary.is_empty());
    }

    #[test]
    fn test_default_generation() {
        let binary = generate_mock_program("something unknown");
        assert!(!binary.is_empty());
    }
}
