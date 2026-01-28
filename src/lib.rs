#![recursion_limit = "512"]
//! Neurlang - AI-Optimized Binary Programming Language
//!
//! A revolutionary AI-native programming language achieving 1000x+ faster code generation
//! with minimal cost. Uses a 33-opcode binary IR with implicit security, compiled via
//! copy-and-patch in under 5 microseconds.
//!
//! # Features
//!
//! - **33-opcode binary IR**: Complete language with security, concurrency, I/O, and computation
//! - **Copy-and-patch compilation**: <5μs compile time using pre-compiled code stencils
//! - **Implicit security**: Capability-based memory access with automatic bounds checking
//! - **Taint tracking**: Information flow security built into the language
//! - **Concurrency primitives**: Lightweight tasks, channels, and atomic operations
//! - **Sandboxed I/O**: File, network, console, and time operations with permissions
//! - **Math extensions**: FPU, random, and bit manipulation operations
//! - **AI Integration**: ONNX-based model inference with retry loop
//!
//! # Three-Tier Library/Framework Architecture
//!
//! Neurlang uses a hybrid approach for code reuse:
//!
//! - **Tier 0: Core Opcodes** - AI writes from scratch for simple operations
//! - **Tier 1: Intrinsics** - ~50 macro tokens that expand to optimized IR at assembly time
//! - **Tier 2: Rust FFI Extensions** - Complex operations (crypto, etc.) in safe Rust
//!
//! ## Intrinsics (Tier 1)
//!
//! ```text
//! @memcpy r0, r1, 256    ; Zero-cost, expands to optimized loop
//! @strlen r0             ; String length calculation
//! @gcd r0, r1            ; Euclidean GCD algorithm
//! @sha256_hash r0, r1    ; (via extension call for security)
//! ```
//!
//! ## Extensions (Tier 2)
//!
//! ```text
//! ext.call r0, sha256, r1, r2       ; Call registered Rust extension
//! ext.call r0, aes256_encrypt, ...  ; Cryptographic operations in safe Rust
//! ```
//!
//! # Example
//!
//! ```rust
//! use neurlang::ir::Assembler;
//! use neurlang::interp::Interpreter;
//!
//! // Assemble a simple program
//! let mut asm = Assembler::new();
//! let program = asm.assemble(r#"
//!     mov r0, 42
//!     halt
//! "#).unwrap();
//!
//! // Execute with interpreter
//! let mut interp = Interpreter::new(1024);
//! let _result = interp.execute(&program);
//!
//! assert_eq!(interp.registers[0], 42);
//! ```
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────┐
//! │   AI Model      │  Generates binary IR directly (ONNX)
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │   Binary IR     │  32 opcodes, 4-8 byte instructions
//! └────────┬────────┘
//!          │
//!     ┌────┴────┐
//!     ▼         ▼
//! ┌───────┐  ┌──────────┐
//! │Interp │  │ Compiler │  <5μs copy-and-patch
//! └───────┘  └────┬─────┘
//!                 │
//!                 ▼
//! ┌─────────────────────┐
//! │   Native Code       │  Runs at full speed
//! └─────────────────────┘
//! ```

// TODO: Re-enable missing_docs once API stabilizes
#![allow(missing_docs)]
#![warn(clippy::all)]

pub mod arch;
pub mod codegen;
pub mod compile;
pub mod compiler;
pub mod config;
pub mod extensions;
pub mod ffi;
pub mod inference;
pub mod interp;
pub mod ir;
pub mod jit;
pub mod orchestration;
pub mod runtime;
pub mod slot;
pub mod stencil;
pub mod train;
pub mod training;
pub mod wrappers;

// Re-export commonly used types
pub use compile::{CompileError, CompiledCode, Compiler};
#[cfg(feature = "ort-backend")]
pub use inference::OnnxEmbedder;
pub use inference::{
    cosine_similarity, create_embedder, create_embedder_auto, Chunk, Embedder, EmbedderConfig,
    EmbedderError, IndexConfig, IndexError, OllamaEmbedder, SearchResult, VectorIndex,
    EMBEDDING_DIM, MXBAI_EMBED_DIM, NOMIC_EMBED_DIM,
};
pub use inference::{Agent, AgentConfig, AgentError, AgentResult, TestCase as AgentTestCase};
pub use inference::{ConversationTurn, Session, SessionId, SessionMeta, SessionStatus};
pub use inference::{ErrorFormatter, InferenceEngine, OrchResult, Orchestrator};
pub use interp::{
    execute_fast_with_extensions, execute_with_mocks, ExtensionMock, InterpResult, Interpreter,
};
pub use ir::{Assembler, Disassembler, Instruction, Opcode, Program, Register};
pub use jit::{JitContext, JitExecutor, JitResult};
pub use stencil::{IOPermissions, IORuntime};

// Tier 1: Intrinsics
pub use ir::{
    IntrinsicArg, IntrinsicCall, IntrinsicCategory, IntrinsicDef, IntrinsicError, IntrinsicRegistry,
};

// Tier 2: Extensions
pub use runtime::{
    ext_ids, CapPermissions, ExtCategory, ExtError, ExtensionEntry, ExtensionRegistry, SafeBuffer,
};

// Extension utilities (RAG seed bank)
pub use extensions::{create_configured_assembler, extension_count, list_all_extensions};

// Safe Wrappers (Tier 2+ for production)
pub use wrappers::{OwnedBuffer, WrapperCategory, WrapperError, WrapperRegistry, WrapperResult};

// FFI (Foreign Function Interface) for Go/C interop
pub use ffi::{
    DynamicLibrary, FfiError, FfiFunctionInfo, FfiRegistry, FfiSignature, FfiType, FfiValue,
    LibraryLoader,
};

// Two-Tier Orchestration
pub use orchestration::{
    backends::{BackendError, BackendRegistry, DecomposeResult, LlmBackend, Subtask},
    classifier::{PatternClassifier, PatternInfo, TierDecision},
    collector::{
        count_training_examples, read_training_data, TrainingDataCollector, TrainingExample,
    },
    OrchestrationResult, OrchestratorConfig, OrchestratorError, SubtaskResult,
};

// Slot-Based Code Generation
pub use slot::{
    extract_captures,
    extract_slot_training_data,
    parse_protocol_spec,
    parse_protocol_spec_str,
    quick_detect,
    quick_generate,
    quick_route,
    AssembleError,
    AssembleResult,
    BufferOffset,
    CacheConfig,
    CacheStats,
    Capture,
    CaptureSpec,
    CaptureType,
    CommandHandler,
    CounterLimit,
    DataItem,
    DataType,
    DataValue,
    ExpanderConfig,
    ExtractionStats,
    FillError,
    FillResult,
    FilledSlot,
    FillerConfig,
    GenerationResult,
    HandlerType,
    IntentParser,
    IntentParserConfig,
    LoopCondition,
    MemWidth,
    MockBackend,
    ParseError,
    ParsedIntent,
    ProtocolCommand,
    ProtocolError,
    ProtocolSpec,
    ProtocolState,
    RouteDecision,
    Router,
    RouterConfig,
    RouterError,
    Slot,
    SlotAssembler,
    SlotCache,
    SlotCategory,
    SlotContext,
    // Phase 3: Slot Filling & Verification
    SlotFiller,
    SlotFillerBackend,
    SlotInput,
    SlotParams,
    SlotSpec,
    SlotTest,
    SlotTrainingExample,
    // Phase 3: Training Data Extraction
    SlotTrainingExtractor,
    SlotType,
    SlotVerifier,
    TemplateBackend,
    TemplateError,
    TemplateExpander,
    TestCase,
    TestExpected,
    TestInput,
    TestStep,
    ValidationConfig,
    ValidationType,
    VerifyError,
    VerifyResult,
};

/// Threshold for choosing between interpreter and compiler
///
/// Programs with fewer than this many instructions will use the interpreter.
/// NOTE: Currently set high to use interpreter by default while JIT compiler
/// branch handling is being fixed.
pub const INTERP_THRESHOLD: usize = 1000;

/// Execute a program, automatically choosing interpreter or compiler
pub fn execute(program: &Program, registers: &mut [u64; 32]) -> Result<u64, ExecuteError> {
    if program.instructions.len() < INTERP_THRESHOLD {
        // Use interpreter for small programs
        let result = interp::execute_fast(program, registers);
        match result {
            InterpResult::Ok(_) | InterpResult::Halted => Ok(registers[0]),
            InterpResult::Trapped(t) => Err(ExecuteError::Trapped(format!("{:?}", t))),
            InterpResult::DivByZero => Err(ExecuteError::DivByZero),
            InterpResult::OutOfBounds => Err(ExecuteError::OutOfBounds),
            InterpResult::InvalidInstruction => Err(ExecuteError::InvalidInstruction),
            InterpResult::CapabilityViolation => Err(ExecuteError::CapabilityViolation),
            InterpResult::MaxInstructionsExceeded => Err(ExecuteError::MaxInstructionsExceeded),
        }
    } else {
        // Use compiler for larger programs
        let mut compiler = Compiler::new();
        unsafe {
            match compiler.compile_and_run(program, registers) {
                Ok(result) => Ok(result),
                Err(e) => Err(ExecuteError::CompileError(e.to_string())),
            }
        }
    }
}

/// Error type for execution
#[derive(Debug, Clone)]
pub enum ExecuteError {
    /// Compilation failed
    CompileError(String),
    /// Trapped by a trap instruction
    Trapped(String),
    /// Division by zero
    DivByZero,
    /// Out of bounds memory access
    OutOfBounds,
    /// Invalid instruction
    InvalidInstruction,
    /// Capability violation
    CapabilityViolation,
    /// Maximum instructions exceeded
    MaxInstructionsExceeded,
}

impl std::fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecuteError::CompileError(e) => write!(f, "Compile error: {}", e),
            ExecuteError::Trapped(t) => write!(f, "Trapped: {}", t),
            ExecuteError::DivByZero => write!(f, "Division by zero"),
            ExecuteError::OutOfBounds => write!(f, "Out of bounds memory access"),
            ExecuteError::InvalidInstruction => write!(f, "Invalid instruction"),
            ExecuteError::CapabilityViolation => write!(f, "Capability violation"),
            ExecuteError::MaxInstructionsExceeded => write!(f, "Maximum instructions exceeded"),
        }
    }
}

impl std::error::Error for ExecuteError {}

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_small_program() {
        let mut asm = Assembler::new();
        let program = asm.assemble("mov r0, 42\nhalt").unwrap();

        let mut registers = [0u64; 32];
        let result = execute(&program, &mut registers);

        assert!(result.is_ok());
        assert_eq!(registers[0], 42);
    }

    #[test]
    fn test_threshold() {
        // Threshold set high to use interpreter while JIT branch handling is being fixed
        assert_eq!(INTERP_THRESHOLD, 1000);
    }
}
