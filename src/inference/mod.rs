//! Inference Integration Layer for Neurlang
//!
//! Provides AI model integration for code generation with two modes:
//!
//! 1. **Legacy mode**: Binary IR generation via autoregressive model
//! 2. **Multi-head mode**: Direct intent + operand prediction (NEW)
//!
//! # Multi-Head Architecture (Recommended)
//!
//! ```text
//! User Prompt → MultiHeadInference → (intent, operands)
//!                                          │
//!                     ┌────────────────────┘
//!                     ▼
//!               IR Generator
//!                     │
//!                     ▼
//!               Program { instructions }
//!                     │
//!                     ▼
//!             Copy-and-Patch Compiler
//!                     │
//!                     ▼
//!             Native x86-64 Code
//!                     │
//!                     ▼
//!               Execution Result
//! ```
//!
//! # Performance
//!
//! | Stage       | Latency  |
//! |-------------|----------|
//! | Inference   | < 0.3ms  |
//! | Generation  | < 0.01ms |
//! | Compilation | < 0.005ms|
//! | Execution   | < 0.001ms|
//! | **Total**   | **< 0.5ms** |

// Legacy modules
pub mod engine;
pub mod formatter;
pub mod orchestrator;

// Multi-head direct prediction modules (NEW)
pub mod generators;
pub mod lookup;
pub mod multihead;
pub mod pipeline;
pub mod tokenizer;

// Inference engine backends
pub mod engines;

// Interactive agent (parallel prediction)
pub mod agent;
pub mod embedder;
pub mod index;
pub mod session;
pub mod verify;

// RAG-enhanced inference (in-memory intent classification)
pub mod example_index;
pub mod intent_index;

// Legacy exports
pub use engine::{InferenceEngine, InferenceError};
pub use formatter::{ErrorFeedback, ErrorFormatter};
pub use orchestrator::{OrchResult, Orchestrator, OrchestratorConfig};

// Multi-head exports
pub use generators::{generate_program, GeneratorError, IrGenerator, IR_GENERATORS};
pub use lookup::{
    detect_intent_from_keywords, intent_id_from_name, intent_name_from_id, operand_count,
    IntentCategory, INTENT_MAP, INTENT_NAMES, OPERAND_COUNTS,
};
pub use multihead::{MultiHeadError, MultiHeadInference, MultiHeadPrediction};
pub use pipeline::{
    benchmark_rag_pipeline,
    confidence,
    FastPipeline,
    InferencePath,
    NeurlangPipeline,
    PipelineError,
    PipelineResult,
    // RAG-enhanced pipeline exports
    RagPipeline,
    RagPipelineConfig,
    RagPipelineResult,
};
pub use tokenizer::{
    extract_features, extract_numbers, parse_symbolic_expression, FastTokenizer, SymbolicExpr,
    TextFeatures, MAX_SEQ_LEN, VOCAB_SIZE,
};

// Engine exports
pub use engines::{
    available_engines, engines_info, select_engine, Engine, EngineError, EnginePrediction,
    EngineType,
};

// Agent exports (parallel prediction with verification loop)
pub use agent::{
    find_session, list_sessions, Agent, AgentConfig, AgentError, AgentResult, InstructionSlot,
    TestCase, MAX_ITERATIONS, NUM_SLOTS,
};
pub use session::{
    sessions_base_dir, ConversationTurn, Session, SessionId, SessionMeta, SessionStatus,
};

// Verification exports
pub use verify::{
    common as verify_common, TestCase as VerifyTestCase, TestResult, TestSuite, VerificationResult,
    Verifier, VerifyError,
};

// Vector index exports
pub use index::{
    hash_file, hash_files, Chunk, IndexConfig, IndexError, SearchResult, VectorIndex,
    CHUNK_OVERLAP, EMBEDDING_DIM, MAX_CHUNK_SIZE,
};

// Embedder exports
pub use embedder::{
    cosine_similarity, create_embedder, create_embedder_auto, Embedder, EmbedderConfig,
    EmbedderError, FastEmbedder, OllamaEmbedder, EMBEDDING_DIM as EMBEDDER_DIM, MXBAI_EMBED_DIM,
    NOMIC_EMBED_DIM,
};

#[cfg(feature = "ort-backend")]
pub use embedder::OnnxEmbedder;

// Intent index exports (RAG)
pub use intent_index::{
    IntentIndex, IntentIndexError, INTENT_DESCRIPTIONS, INTENT_EMBEDDING_DIM, NUM_INTENTS,
};

// Example index exports (RAG)
pub use example_index::{ExampleIndex, ExampleIndexError, ExampleMeta, ExampleSearchResult};
