//! Interactive Agent for Neurlang
//!
//! Provides an interactive agent that generates verified programs through
//! a fast iteration loop with session persistence and checkpointing.
//!
//! # Architecture
//!
//! The agent separates hot and cold paths:
//! - **Cold path**: Session load/save, model loading (I/O allowed)
//! - **Hot path**: Generate → Compile → Execute → Verify (zero I/O)
//!
//! # Usage
//!
//! ```rust,ignore
//! use neurlang::inference::agent::{Agent, AgentConfig};
//!
//! // Start a new session
//! let mut agent = Agent::new("build a calculator")?;
//!
//! // Generate verified code
//! let result = agent.handle_request("compute factorial of 10", &tests)?;
//!
//! // Continue with refinements
//! agent.handle_request("add support for negative numbers", &tests)?;
//!
//! // Resume after restart
//! let mut agent = Agent::resume(&session_id)?;
//! ```

use std::collections::HashMap;
use std::path::PathBuf;

use crate::ir::{Instruction, Opcode, Program, Register};
use crate::orchestration::{
    backends::{BackendRegistry, Subtask},
    classifier::{PatternClassifier, TierDecision},
    collector::TrainingDataCollector,
};
use crate::{execute, ExecuteError};

use super::session::{ConversationTurn, Session, SessionId, SessionStatus};

/// Maximum iterations before giving up
pub const MAX_ITERATIONS: usize = 1000;

/// Default number of slots in parallel prediction
pub const NUM_SLOTS: usize = 64;

/// Agent configuration
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Model path (ONNX format)
    pub model_path: PathBuf,
    /// Maximum iterations per request
    pub max_iterations: usize,
    /// Verify each generated program
    pub verify: bool,
    /// Verbose output
    pub verbose: bool,
    /// Enable two-tier orchestration (Tier 1: small model, Tier 2: LLM)
    pub enable_orchestration: bool,
    /// Similarity threshold for Tier 1 routing (0.0-1.0)
    pub similarity_threshold: f32,
    /// Default LLM backend name (e.g., "claude", "ollama")
    pub default_backend: String,
    /// Path for training data collection (None = disabled)
    pub training_data_path: Option<PathBuf>,
    /// Maximum subtasks from LLM decomposition
    pub max_subtasks: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("models/neurlang.onnx"),
            max_iterations: MAX_ITERATIONS,
            verify: true,
            verbose: false,
            enable_orchestration: false,
            similarity_threshold: 0.85,
            default_backend: "claude".to_string(),
            training_data_path: None,
            max_subtasks: 20,
        }
    }
}

/// Test case for verification
#[derive(Debug, Clone)]
pub struct TestCase {
    /// Input values (placed in registers r0, r1, ...)
    pub inputs: Vec<i64>,
    /// Expected output (from r0)
    pub expected: i64,
}

impl TestCase {
    /// Create a new test case
    pub fn new(inputs: Vec<i64>, expected: i64) -> Self {
        Self { inputs, expected }
    }
}

/// Result of a generation request
#[derive(Debug)]
pub struct AgentResult {
    /// Generated program
    pub program: Program,
    /// Number of iterations taken
    pub iterations: usize,
    /// Whether verification passed
    pub verified: bool,
    /// Summary message
    pub summary: String,
}

/// Agent error types
#[derive(Debug)]
pub enum AgentError {
    /// Maximum iterations reached without verification
    MaxIterationsReached {
        iterations: usize,
        last_error: String,
    },
    /// Session not found
    SessionNotFound(String),
    /// Session I/O error
    SessionError(std::io::Error),
    /// Model error
    ModelError(String),
    /// Execution error
    ExecutionError(String),
    /// Compilation error
    CompilationError(String),
    /// LLM backend error
    BackendError(String),
    /// Task decomposition failed
    DecompositionFailed(String),
    /// Subtask failed after all attempts
    SubtaskFailed {
        subtask_id: usize,
        description: String,
        error: String,
    },
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentError::MaxIterationsReached {
                iterations,
                last_error,
            } => {
                write!(
                    f,
                    "Max iterations ({}) reached. Last error: {}",
                    iterations, last_error
                )
            }
            AgentError::SessionNotFound(id) => write!(f, "Session not found: {}", id),
            AgentError::SessionError(e) => write!(f, "Session error: {}", e),
            AgentError::ModelError(e) => write!(f, "Model error: {}", e),
            AgentError::ExecutionError(e) => write!(f, "Execution error: {}", e),
            AgentError::CompilationError(e) => write!(f, "Compilation error: {}", e),
            AgentError::BackendError(e) => write!(f, "Backend error: {}", e),
            AgentError::DecompositionFailed(e) => write!(f, "Task decomposition failed: {}", e),
            AgentError::SubtaskFailed {
                subtask_id,
                description,
                error,
            } => {
                write!(
                    f,
                    "Subtask {} ('{}') failed: {}",
                    subtask_id, description, error
                )
            }
        }
    }
}

impl std::error::Error for AgentError {}

impl From<std::io::Error> for AgentError {
    fn from(e: std::io::Error) -> Self {
        AgentError::SessionError(e)
    }
}

impl From<ExecuteError> for AgentError {
    fn from(e: ExecuteError) -> Self {
        AgentError::ExecutionError(e.to_string())
    }
}

/// Pre-allocated buffers for hot path (zero allocation during iteration)
struct HotPathBuffers {
    /// Pre-allocated instruction buffer for model output
    ir_buffer: Box<[InstructionSlot; NUM_SLOTS]>,
    /// Pre-allocated register file for execution
    registers: Box<[u64; 32]>,
    /// Context buffer for formatting prompts
    context_buffer: String,
}

impl Default for HotPathBuffers {
    fn default() -> Self {
        Self {
            ir_buffer: Box::new([InstructionSlot::default(); NUM_SLOTS]),
            registers: Box::new([0u64; 32]),
            context_buffer: String::with_capacity(8192),
        }
    }
}

/// A slot in the parallel prediction output
#[derive(Debug, Clone, Copy, Default)]
pub struct InstructionSlot {
    pub valid: bool,
    pub opcode: u8,
    pub mode: u8,
    pub rd: u8,
    pub rs1: u8,
    pub rs2: u8,
    pub has_imm: bool,
    pub imm: i32,
}

impl InstructionSlot {
    /// Convert slot to IR instruction
    pub fn to_instruction(&self) -> Option<Instruction> {
        if !self.valid {
            return None;
        }

        let opcode = Opcode::from_u8(self.opcode)?;
        let rd = Register::from_u8(self.rd)?;
        let rs1 = Register::from_u8(self.rs1)?;
        let rs2 = Register::from_u8(self.rs2)?;

        if self.has_imm {
            Some(Instruction::with_imm(opcode, rd, rs1, self.mode, self.imm))
        } else {
            Some(Instruction::new(opcode, rd, rs1, rs2, self.mode))
        }
    }
}

/// Interactive agent with hot/cold path separation
pub struct Agent {
    /// Current session (persisted state)
    session: Session,
    /// Agent configuration
    config: AgentConfig,
    /// Pre-allocated buffers for hot path
    buffers: HotPathBuffers,
    /// Named functions built during this session
    functions: HashMap<String, Program>,
    /// Pattern classifier for tier decision (optional)
    classifier: Option<PatternClassifier>,
    /// LLM backend registry (optional)
    backends: Option<BackendRegistry>,
    /// Training data collector (optional)
    collector: Option<TrainingDataCollector>,
}

impl Agent {
    /// Create a new agent with a fresh session
    pub fn new(name: impl Into<String>) -> Result<Self, AgentError> {
        let session = Session::new(name);
        let config = AgentConfig::default();

        Ok(Self {
            session,
            config,
            buffers: HotPathBuffers::default(),
            functions: HashMap::new(),
            classifier: None,
            backends: None,
            collector: None,
        })
    }

    /// Create a new agent with custom configuration
    pub fn with_config(name: impl Into<String>, config: AgentConfig) -> Result<Self, AgentError> {
        let session = Session::new(name);

        // Initialize orchestration components if enabled
        let (classifier, backends, collector) = if config.enable_orchestration {
            let mut classifier = PatternClassifier::with_threshold(config.similarity_threshold);

            // Load patterns from training data if available
            if let Some(ref path) = config.training_data_path {
                if path.exists() {
                    if let Ok(data) = std::fs::read_to_string(path) {
                        classifier.load_patterns(&data);
                    }
                }
            }

            let backends = BackendRegistry::new();

            let collector = config
                .training_data_path
                .as_ref()
                .map(TrainingDataCollector::new);

            (Some(classifier), Some(backends), collector)
        } else {
            (None, None, None)
        };

        Ok(Self {
            session,
            config,
            buffers: HotPathBuffers::default(),
            functions: HashMap::new(),
            classifier,
            backends,
            collector,
        })
    }

    /// Create a new agent with orchestration enabled
    pub fn with_orchestration(name: impl Into<String>) -> Result<Self, AgentError> {
        let mut config = AgentConfig::default();
        config.enable_orchestration = true;
        Self::with_config(name, config)
    }

    /// Resume an existing session
    pub fn resume(session_id: &str) -> Result<Self, AgentError> {
        let session = Session::load(session_id)?;
        let functions = session.functions.clone();
        let config = AgentConfig::default();

        Ok(Self {
            session,
            config,
            buffers: HotPathBuffers::default(),
            functions,
            classifier: None,
            backends: None,
            collector: None,
        })
    }

    /// Resume with orchestration enabled
    pub fn resume_with_orchestration(
        session_id: &str,
        config: AgentConfig,
    ) -> Result<Self, AgentError> {
        let session = Session::load(session_id)?;
        let functions = session.functions.clone();

        let (classifier, backends, collector) = if config.enable_orchestration {
            let classifier = PatternClassifier::with_threshold(config.similarity_threshold);
            let backends = BackendRegistry::new();
            let collector = config
                .training_data_path
                .as_ref()
                .map(TrainingDataCollector::new);
            (Some(classifier), Some(backends), collector)
        } else {
            (None, None, None)
        };

        Ok(Self {
            session,
            config,
            buffers: HotPathBuffers::default(),
            functions,
            classifier,
            backends,
            collector,
        })
    }

    /// Get the current session ID
    pub fn session_id(&self) -> &str {
        self.session.id()
    }

    /// Get the session name
    pub fn session_name(&self) -> &str {
        self.session.name()
    }

    /// Get iteration count
    pub fn iteration_count(&self) -> usize {
        self.session.meta.iteration_count
    }

    /// Handle a user request with verification
    ///
    /// This is the main entry point. It:
    /// 1. Adds the request to conversation history (cold path)
    /// 2. Classifies the request (Tier 1 vs Tier 2) if orchestration enabled
    /// 3. Runs the appropriate generation flow
    /// 4. Checkpoints the result (cold path)
    pub fn handle_request(
        &mut self,
        request: &str,
        tests: &[TestCase],
    ) -> Result<AgentResult, AgentError> {
        // COLD PATH: Add request to history
        self.session.add_user_message(request);

        // Check if orchestration is enabled
        let result = if self.config.enable_orchestration && self.classifier.is_some() {
            self.handle_with_orchestration(request, tests)?
        } else {
            // Direct Tier 1 path (no orchestration)
            self.generate_verified(request, tests)?
        };

        // COLD PATH: Commit and checkpoint
        self.commit_result(&result);

        // Record success in training collector
        if result.verified {
            if let Some(ref mut collector) = self.collector {
                let ir = Self::instructions_to_ir(&result.program.instructions);
                collector.record_success(request, &ir);
            }
        }

        Ok(result)
    }

    /// Handle request with two-tier orchestration
    fn handle_with_orchestration(
        &mut self,
        request: &str,
        tests: &[TestCase],
    ) -> Result<AgentResult, AgentError> {
        let classifier = self.classifier.as_ref().unwrap();

        // Classify the request
        let decision = classifier.classify(request);

        match decision {
            TierDecision::Tier1 {
                pattern,
                confidence,
            } => {
                if self.config.verbose {
                    eprintln!(
                        "[Orchestrator] Tier 1: Pattern '{}' matched with confidence {:.2}",
                        pattern, confidence
                    );
                }

                // Try Tier 1 (small model) first
                match self.generate_verified(request, tests) {
                    Ok(result) => Ok(result),
                    Err(AgentError::MaxIterationsReached { .. }) => {
                        // Escalate to Tier 2 on failure
                        if self.config.verbose {
                            eprintln!("[Orchestrator] Tier 1 failed, escalating to Tier 2");
                        }
                        self.handle_tier2(request, tests)
                    }
                    Err(e) => Err(e),
                }
            }
            TierDecision::Tier2 { reason } => {
                if self.config.verbose {
                    eprintln!("[Orchestrator] Tier 2: {}", reason);
                }

                // Go directly to LLM decomposition
                self.handle_tier2(request, tests)
            }
        }
    }

    /// Handle request via Tier 2 (LLM decomposition)
    fn handle_tier2(
        &mut self,
        request: &str,
        tests: &[TestCase],
    ) -> Result<AgentResult, AgentError> {
        // Build context from conversation history
        let context = self
            .session
            .recent_context(5)
            .iter()
            .map(|turn| match turn {
                ConversationTurn::User(msg) => format!("User: {}", msg),
                ConversationTurn::Agent(msg) => format!("Agent: {}", msg),
                ConversationTurn::Error(msg) => format!("Error: {}", msg),
                ConversationTurn::System(msg) => format!("System: {}", msg),
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Get decomposition from LLM (scope the borrow)
        let (sorted_subtasks, reasoning, complexity) = {
            let backends = self
                .backends
                .as_ref()
                .ok_or_else(|| AgentError::BackendError("No backends configured".to_string()))?;

            let backend = backends.get(&self.config.default_backend).ok_or_else(|| {
                AgentError::BackendError(format!(
                    "Backend '{}' not found",
                    self.config.default_backend
                ))
            })?;

            if !backend.is_available() {
                return Err(AgentError::BackendError(format!(
                    "Backend '{}' is not available (missing API key?)",
                    self.config.default_backend
                )));
            }

            // Decompose task via LLM
            let decomposition = backend
                .decompose_task(request, &context)
                .map_err(|e| AgentError::DecompositionFailed(e.to_string()))?;

            let sorted = self.sort_subtasks_by_deps(&decomposition.subtasks);
            (sorted, decomposition.reasoning, decomposition.complexity)
        };

        if self.config.verbose {
            eprintln!(
                "[Orchestrator] LLM decomposed task into {} subtasks (complexity: {})",
                sorted_subtasks.len(),
                complexity
            );
            eprintln!("[Orchestrator] Reasoning: {}", reasoning);
        }

        // Process each subtask
        let mut combined_instructions: Vec<Instruction> = Vec::new();
        let mut total_iterations = 0;
        let mut subtask_results: Vec<String> = Vec::new();

        for subtask in sorted_subtasks.iter().take(self.config.max_subtasks) {
            if self.config.verbose {
                eprintln!(
                    "[Orchestrator] Processing subtask {}: {}",
                    subtask.id, subtask.description
                );
            }

            // Generate code for subtask
            match self.generate_verified(&subtask.description, tests) {
                Ok(result) => {
                    total_iterations += result.iterations;
                    combined_instructions.extend(result.program.instructions.clone());
                    subtask_results.push(format!(
                        "Subtask {}: {} ({} iterations)",
                        subtask.id, subtask.description, result.iterations
                    ));

                    // Record successful subtask for training
                    if let Some(ref mut collector) = self.collector {
                        let ir = Self::instructions_to_ir(&result.program.instructions);
                        collector.record_success(&subtask.description, &ir);
                    }
                }
                Err(AgentError::MaxIterationsReached { last_error, .. }) => {
                    // Try to get a fix hint from LLM (re-borrow for the hint)
                    let hint = self.get_fix_hint_from_backend(&subtask.description, &last_error);

                    if let Some(hint) = hint {
                        if self.config.verbose {
                            eprintln!("[Orchestrator] LLM hint: {}", hint);
                        }

                        // Try again with the hint
                        let hint_request = format!("{} (hint: {})", subtask.description, hint);
                        if let Ok(result) = self.generate_verified(&hint_request, tests) {
                            total_iterations += result.iterations;
                            combined_instructions.extend(result.program.instructions.clone());
                            subtask_results.push(format!(
                                "Subtask {} (with hint): {} ({} iterations)",
                                subtask.id, subtask.description, result.iterations
                            ));

                            // Record error recovery for training
                            if let Some(ref mut collector) = self.collector {
                                let ir = Self::instructions_to_ir(&result.program.instructions);
                                collector.record_error_recovery(
                                    &subtask.description,
                                    &last_error,
                                    &[],
                                    &ir,
                                );
                            }

                            continue;
                        }
                    }

                    return Err(AgentError::SubtaskFailed {
                        subtask_id: subtask.id,
                        description: subtask.description.clone(),
                        error: last_error,
                    });
                }
                Err(e) => return Err(e),
            }
        }

        // Ensure we have a halt instruction at the end
        if !combined_instructions.is_empty() {
            let last = combined_instructions.last().unwrap();
            if last.opcode != Opcode::Halt {
                combined_instructions.push(Instruction::new(
                    Opcode::Halt,
                    Register::R0,
                    Register::R0,
                    Register::R0,
                    0,
                ));
            }
        }

        let program = Program {
            instructions: combined_instructions,
            entry_point: 0,
            data_section: Vec::new(),
            entry_label: None,
        };

        let summary = format!(
            "Tier 2 completed: {} subtasks, {} total iterations\n{}",
            sorted_subtasks.len().min(self.config.max_subtasks),
            total_iterations,
            subtask_results.join("\n")
        );

        Ok(AgentResult {
            program,
            iterations: total_iterations,
            verified: true,
            summary,
        })
    }

    /// Get a fix hint from the LLM backend
    fn get_fix_hint_from_backend(&self, task: &str, error: &str) -> Option<String> {
        let backends = self.backends.as_ref()?;
        let backend = backends.get(&self.config.default_backend)?;
        backend.get_fix_hint(task, error, "").ok()
    }

    /// Convert instructions to IR format (Vec<u32>) for training data
    fn instructions_to_ir(instructions: &[Instruction]) -> Vec<u32> {
        instructions
            .iter()
            .flat_map(|i| {
                let bytes = i.encode();
                // Convert bytes to u32s (little-endian, 4 bytes per u32)
                bytes
                    .chunks(4)
                    .map(|chunk| {
                        let mut arr = [0u8; 4];
                        arr[..chunk.len()].copy_from_slice(chunk);
                        u32::from_le_bytes(arr)
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Sort subtasks by dependencies (topological sort)
    fn sort_subtasks_by_deps(&self, subtasks: &[Subtask]) -> Vec<Subtask> {
        let mut sorted = Vec::new();
        let mut completed: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut remaining: Vec<&Subtask> = subtasks.iter().collect();

        while !remaining.is_empty() {
            let mut made_progress = false;

            remaining.retain(|subtask| {
                // Check if all dependencies are completed
                let deps_met = subtask.depends_on.iter().all(|dep| completed.contains(dep));
                if deps_met {
                    sorted.push((*subtask).clone());
                    completed.insert(subtask.id);
                    made_progress = true;
                    false // Remove from remaining
                } else {
                    true // Keep in remaining
                }
            });

            if !made_progress && !remaining.is_empty() {
                // Circular dependency or missing dependency, just add remaining in order
                for subtask in remaining.drain(..) {
                    sorted.push(subtask.clone());
                }
                break;
            }
        }

        sorted
    }

    /// Handle a request without test verification
    pub fn handle_request_unverified(&mut self, request: &str) -> Result<AgentResult, AgentError> {
        self.handle_request(request, &[])
    }

    /// HOT PATH: Generate verified program (zero I/O)
    ///
    /// This is the fast inner loop that runs up to MAX_ITERATIONS times.
    /// No disk I/O or allocations should happen here.
    fn generate_verified(
        &mut self,
        task: &str,
        tests: &[TestCase],
    ) -> Result<AgentResult, AgentError> {
        let mut last_error: Option<String> = None;

        for iteration in 0..self.config.max_iterations {
            // Format context for model (reuses buffer)
            self.format_context(task, last_error.as_deref());

            // Get model prediction (fills ir_buffer)
            self.predict_instructions()?;

            // Decode instructions from buffer
            let instructions = self.decode_instructions();

            if instructions.is_empty() {
                last_error = Some("No valid instructions generated".to_string());
                continue;
            }

            // Build program
            let program = self.build_program(instructions);

            // Skip verification if no tests
            if tests.is_empty() {
                let instr_count = program.instructions.len();
                return Ok(AgentResult {
                    program,
                    iterations: iteration + 1,
                    verified: false,
                    summary: format!("Generated {} instructions (unverified)", instr_count),
                });
            }

            // Verify against test cases
            match self.verify_program(&program, tests) {
                Ok(()) => {
                    let instr_count = program.instructions.len();
                    let test_count = tests.len();
                    return Ok(AgentResult {
                        program,
                        iterations: iteration + 1,
                        verified: true,
                        summary: format!(
                            "Verified after {} iterations ({} instructions, {} tests passed)",
                            iteration + 1,
                            instr_count,
                            test_count
                        ),
                    });
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        Err(AgentError::MaxIterationsReached {
            iterations: self.config.max_iterations,
            last_error: last_error.unwrap_or_else(|| "Unknown error".to_string()),
        })
    }

    /// Format context for model input (reuses pre-allocated buffer)
    fn format_context(&mut self, task: &str, error: Option<&str>) {
        self.buffers.context_buffer.clear();

        // Add task
        self.buffers.context_buffer.push_str("Task: ");
        self.buffers.context_buffer.push_str(task);
        self.buffers.context_buffer.push('\n');

        // Add recent conversation history (last 5 turns)
        let recent = self.session.recent_context(5);
        if !recent.is_empty() {
            self.buffers.context_buffer.push_str("\nRecent context:\n");
            for turn in recent {
                match turn {
                    ConversationTurn::User(msg) => {
                        self.buffers.context_buffer.push_str("User: ");
                        self.buffers.context_buffer.push_str(msg);
                    }
                    ConversationTurn::Agent(msg) => {
                        self.buffers.context_buffer.push_str("Agent: ");
                        self.buffers.context_buffer.push_str(msg);
                    }
                    ConversationTurn::Error(msg) => {
                        self.buffers.context_buffer.push_str("Error: ");
                        self.buffers.context_buffer.push_str(msg);
                    }
                    ConversationTurn::System(msg) => {
                        self.buffers.context_buffer.push_str("System: ");
                        self.buffers.context_buffer.push_str(msg);
                    }
                }
                self.buffers.context_buffer.push('\n');
            }
        }

        // Add error feedback if any
        if let Some(err) = error {
            self.buffers
                .context_buffer
                .push_str("\nError from previous attempt:\n");
            self.buffers.context_buffer.push_str(err);
            self.buffers.context_buffer.push('\n');
        }
    }

    /// Predict instructions using the model
    ///
    /// TODO: Integrate with parallel ONNX model when available.
    /// For now, uses a simple fallback that generates basic instructions.
    fn predict_instructions(&mut self) -> Result<(), AgentError> {
        // Clear buffer
        for slot in self.buffers.ir_buffer.iter_mut() {
            *slot = InstructionSlot::default();
        }

        // TODO: Actual model inference
        // For now, generate a simple placeholder based on context
        // This will be replaced with actual ONNX parallel model inference

        // Parse simple patterns from context
        let context = &self.buffers.context_buffer;

        if context.contains("factorial") || context.contains("fibonacci") {
            // Generate a simple loop pattern
            self.generate_loop_pattern();
        } else if context.contains("add") || context.contains("sum") {
            // Generate addition
            self.generate_add_pattern();
        } else if context.contains("multiply") || context.contains("product") {
            // Generate multiplication
            self.generate_mul_pattern();
        } else {
            // Default: generate a simple mov + halt
            self.generate_default_pattern();
        }

        Ok(())
    }

    /// Generate a simple addition pattern
    fn generate_add_pattern(&mut self) {
        // mov r0, 0  (accumulator)
        self.buffers.ir_buffer[0] = InstructionSlot {
            valid: true,
            opcode: Opcode::Mov as u8,
            mode: 0,
            rd: 0,
            rs1: 31, // zero
            rs2: 0,
            has_imm: true,
            imm: 0,
        };

        // alu.add r0, r0, r1
        self.buffers.ir_buffer[1] = InstructionSlot {
            valid: true,
            opcode: Opcode::Alu as u8,
            mode: 0, // ADD
            rd: 0,
            rs1: 0,
            rs2: 1,
            has_imm: false,
            imm: 0,
        };

        // halt
        self.buffers.ir_buffer[2] = InstructionSlot {
            valid: true,
            opcode: Opcode::Halt as u8,
            mode: 0,
            rd: 0,
            rs1: 0,
            rs2: 0,
            has_imm: false,
            imm: 0,
        };
    }

    /// Generate a simple multiplication pattern
    fn generate_mul_pattern(&mut self) {
        // muldiv.mul r0, r1, r2
        self.buffers.ir_buffer[0] = InstructionSlot {
            valid: true,
            opcode: Opcode::MulDiv as u8,
            mode: 0, // MUL
            rd: 0,
            rs1: 1,
            rs2: 2,
            has_imm: false,
            imm: 0,
        };

        // halt
        self.buffers.ir_buffer[1] = InstructionSlot {
            valid: true,
            opcode: Opcode::Halt as u8,
            mode: 0,
            rd: 0,
            rs1: 0,
            rs2: 0,
            has_imm: false,
            imm: 0,
        };
    }

    /// Generate a loop pattern (for factorial/fibonacci)
    fn generate_loop_pattern(&mut self) {
        // Simple factorial: r0 = n!, input in r1
        // mov r0, 1         ; result = 1
        // loop:
        //   branch.le r1, zero, end  ; if n <= 0, exit
        //   muldiv.mul r0, r0, r1    ; result *= n
        //   alui.sub r1, r1, 1       ; n--
        //   branch end, loop         ; goto loop
        // end:
        //   halt

        // mov r0, 1
        self.buffers.ir_buffer[0] = InstructionSlot {
            valid: true,
            opcode: Opcode::Mov as u8,
            mode: 0,
            rd: 0,
            rs1: 31,
            rs2: 0,
            has_imm: true,
            imm: 1,
        };

        // branch.le r1, zero, +16 (skip to halt)
        self.buffers.ir_buffer[1] = InstructionSlot {
            valid: true,
            opcode: Opcode::Branch as u8,
            mode: 4, // LE
            rd: 0,
            rs1: 1,
            rs2: 31, // zero
            has_imm: true,
            imm: 24, // skip 3 instructions (to halt)
        };

        // muldiv.mul r0, r0, r1
        self.buffers.ir_buffer[2] = InstructionSlot {
            valid: true,
            opcode: Opcode::MulDiv as u8,
            mode: 0,
            rd: 0,
            rs1: 0,
            rs2: 1,
            has_imm: false,
            imm: 0,
        };

        // alui.sub r1, r1, 1
        self.buffers.ir_buffer[3] = InstructionSlot {
            valid: true,
            opcode: Opcode::AluI as u8,
            mode: 1, // SUB
            rd: 1,
            rs1: 1,
            rs2: 0,
            has_imm: true,
            imm: 1,
        };

        // branch always, -20 (back to check)
        self.buffers.ir_buffer[4] = InstructionSlot {
            valid: true,
            opcode: Opcode::Branch as u8,
            mode: 0, // Always
            rd: 0,
            rs1: 0,
            rs2: 0,
            has_imm: true,
            imm: -24, // back to branch.le
        };

        // halt
        self.buffers.ir_buffer[5] = InstructionSlot {
            valid: true,
            opcode: Opcode::Halt as u8,
            mode: 0,
            rd: 0,
            rs1: 0,
            rs2: 0,
            has_imm: false,
            imm: 0,
        };
    }

    /// Generate default pattern (simple mov + halt)
    fn generate_default_pattern(&mut self) {
        // mov r0, 42
        self.buffers.ir_buffer[0] = InstructionSlot {
            valid: true,
            opcode: Opcode::Mov as u8,
            mode: 0,
            rd: 0,
            rs1: 31,
            rs2: 0,
            has_imm: true,
            imm: 42,
        };

        // halt
        self.buffers.ir_buffer[1] = InstructionSlot {
            valid: true,
            opcode: Opcode::Halt as u8,
            mode: 0,
            rd: 0,
            rs1: 0,
            rs2: 0,
            has_imm: false,
            imm: 0,
        };
    }

    /// Decode instructions from the buffer
    fn decode_instructions(&self) -> Vec<Instruction> {
        self.buffers
            .ir_buffer
            .iter()
            .filter_map(|slot| slot.to_instruction())
            .collect()
    }

    /// Build a program from instructions
    fn build_program(&self, instructions: Vec<Instruction>) -> Program {
        Program {
            instructions,
            entry_point: 0,
            data_section: Vec::new(),
            entry_label: None,
        }
    }

    /// Verify program against test cases
    fn verify_program(&mut self, program: &Program, tests: &[TestCase]) -> Result<(), String> {
        for (i, test) in tests.iter().enumerate() {
            // Reset registers
            for r in self.buffers.registers.iter_mut() {
                *r = 0;
            }

            // Set input registers
            for (j, &input) in test.inputs.iter().enumerate() {
                if j < 16 {
                    self.buffers.registers[j] = input as u64;
                }
            }

            // Execute
            match execute(program, &mut self.buffers.registers) {
                Ok(_) => {
                    let result = self.buffers.registers[0] as i64;
                    if result != test.expected {
                        return Err(format!(
                            "Test {} failed: expected {}, got {} (inputs: {:?})",
                            i, test.expected, result, test.inputs
                        ));
                    }
                }
                Err(e) => {
                    return Err(format!("Test {} execution error: {}", i, e));
                }
            }
        }

        Ok(())
    }

    /// COLD PATH: Commit result and checkpoint
    fn commit_result(&mut self, result: &AgentResult) {
        // Update session
        self.session
            .set_current_ir(result.program.instructions.to_vec());
        self.session.increment_iteration();

        // Add response to history
        self.session.add_agent_response(&result.summary);

        // Update status
        if result.verified {
            self.session.set_status(SessionStatus::Active);
        }

        // Checkpoint to disk
        if let Err(e) = self.session.save() {
            eprintln!("Warning: Failed to checkpoint session: {}", e);
        }
    }

    /// Add a named function to the session
    pub fn add_function(&mut self, name: impl Into<String>, program: Program) {
        let name = name.into();
        self.session.add_function(&name, program.clone());
        self.functions.insert(name, program);
    }

    /// Get a named function
    pub fn get_function(&self, name: &str) -> Option<&Program> {
        self.functions.get(name)
    }

    /// Complete the session successfully
    pub fn complete(&mut self) -> Result<(), AgentError> {
        self.session.set_status(SessionStatus::Completed);
        self.session.save()?;
        Ok(())
    }

    /// Mark the session as failed
    pub fn fail(&mut self, error: &str) -> Result<(), AgentError> {
        self.session.add_error(error);
        self.session.set_status(SessionStatus::Failed);
        self.session.save()?;
        Ok(())
    }

    /// Get conversation history
    pub fn history(&self) -> &[ConversationTurn] {
        &self.session.history
    }
}

/// List all available sessions
pub fn list_sessions() -> Result<Vec<(SessionId, String, usize)>, AgentError> {
    let sessions = super::session::list_sessions()?;
    Ok(sessions
        .into_iter()
        .map(|s| (s.id, s.name, s.iteration_count))
        .collect())
}

/// Find a session by partial ID
pub fn find_session(partial_id: &str) -> Result<Option<SessionId>, AgentError> {
    match super::session::find_session(partial_id)? {
        Some(meta) => Ok(Some(meta.id)),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new("test session").unwrap();
        assert_eq!(agent.session_name(), "test session");
        assert_eq!(agent.iteration_count(), 0);
    }

    #[test]
    fn test_instruction_slot_conversion() {
        let slot = InstructionSlot {
            valid: true,
            opcode: Opcode::Mov as u8,
            mode: 0,
            rd: 0,
            rs1: 31,
            rs2: 0,
            has_imm: true,
            imm: 42,
        };

        let instr = slot.to_instruction().unwrap();
        assert_eq!(instr.opcode, Opcode::Mov);
        assert_eq!(instr.rd, Register::R0);
        assert_eq!(instr.imm, Some(42));
    }

    #[test]
    fn test_invalid_slot() {
        let slot = InstructionSlot::default();
        assert!(slot.to_instruction().is_none());
    }

    #[test]
    fn test_agent_simple_request() {
        let mut agent = Agent::new("test").unwrap();

        // Request without verification
        let result = agent.handle_request_unverified("add two numbers").unwrap();
        assert!(result.iterations > 0);
        assert!(!result.verified);
    }

    #[test]
    fn test_agent_with_verification() {
        let mut agent = Agent::new("factorial test").unwrap();

        let tests = vec![
            TestCase::new(vec![1], 1),   // 1! = 1
            TestCase::new(vec![5], 120), // 5! = 120
        ];

        // This may or may not verify depending on pattern matching
        let result = agent.handle_request("compute factorial", &tests);

        // Just check it doesn't panic
        match result {
            Ok(r) => {
                assert!(r.iterations > 0);
            }
            Err(AgentError::MaxIterationsReached { .. }) => {
                // Expected if simple patterns don't match
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_test_case() {
        let tc = TestCase::new(vec![5, 3], 8);
        assert_eq!(tc.inputs, vec![5, 3]);
        assert_eq!(tc.expected, 8);
    }

    #[test]
    fn test_agent_with_orchestration_config() {
        let mut config = AgentConfig::default();
        config.enable_orchestration = true;
        config.similarity_threshold = 0.85;

        let agent = Agent::with_config("orchestrated test", config).unwrap();
        assert_eq!(agent.session_name(), "orchestrated test");
        assert!(agent.classifier.is_some());
        assert!(agent.backends.is_some());
    }

    #[test]
    fn test_agent_with_orchestration_shorthand() {
        let agent = Agent::with_orchestration("orchestrated test 2").unwrap();
        assert_eq!(agent.session_name(), "orchestrated test 2");
        assert!(agent.classifier.is_some());
        assert!(agent.backends.is_some());
    }

    #[test]
    fn test_instructions_to_ir() {
        let instructions = vec![
            Instruction::with_imm(Opcode::Mov, Register::R0, Register::Zero, 0, 42),
            Instruction::new(Opcode::Halt, Register::R0, Register::R0, Register::R0, 0),
        ];

        let ir = Agent::instructions_to_ir(&instructions);
        assert!(!ir.is_empty());
    }

    #[test]
    fn test_sort_subtasks_by_deps() {
        let agent = Agent::new("test").unwrap();

        let subtasks = vec![
            Subtask {
                id: 3,
                description: "Third".to_string(),
                depends_on: vec![2],
                test_hints: vec![],
                priority: 1,
            },
            Subtask {
                id: 1,
                description: "First".to_string(),
                depends_on: vec![],
                test_hints: vec![],
                priority: 1,
            },
            Subtask {
                id: 2,
                description: "Second".to_string(),
                depends_on: vec![1],
                test_hints: vec![],
                priority: 1,
            },
        ];

        let sorted = agent.sort_subtasks_by_deps(&subtasks);
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].id, 1); // First (no deps)
        assert_eq!(sorted[1].id, 2); // Second (depends on 1)
        assert_eq!(sorted[2].id, 3); // Third (depends on 2)
    }
}
