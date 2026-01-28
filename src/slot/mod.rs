//! Slot-Based Code Generation
//!
//! This module implements a slot-based architecture for fast code generation,
//! achieving 15-600x speedup compared to LLM-based approaches.
//!
//! # Architecture
//!
//! ```text
//! USER REQUEST -> DECOMPOSITION -> SLOT FILLING -> ASSEMBLY -> VERIFICATION
//!                      |               |
//!              Rule-based OR LLM   Parallel batch
//!              produces SlotSpec   ~50ms total
//! ```
//!
//! # Key Concepts
//!
//! - **SlotSpec**: Universal intermediate format with skeleton + slots
//! - **SlotType**: 20 slot types that combine to build any protocol
//! - **Protocol Spec**: YAML files defining protocols (SMTP, HTTP, etc.)
//!
//! # Usage
//!
//! ```ignore
//! use neurlang::slot::{Router, RouterConfig};
//!
//! let router = Router::with_defaults();
//! let result = router.generate("SMTP server")?;
//! println!("Generated {} slots", result.spec.slots.len());
//! ```
//!
//! # Performance
//!
//! | Scenario | Time |
//! |----------|------|
//! | Offline with cache | ~20ms |
//! | Offline cold start | ~100ms |
//! | LLM decomposition + fill | ~3-5s |

pub mod assembler;
pub mod cache;
pub mod filler;
pub mod intent;
pub mod parser;
pub mod router;
pub mod spec;
pub mod template;
pub mod training;
pub mod types;
pub mod validator;
pub mod verifier;

// Re-export commonly used types
pub use assembler::{AssembleError, AssembleResult, SlotAssembler};
pub use cache::{CacheConfig, CacheStats, SlotCache};
pub use filler::{
    FillError, FillResult, FilledSlot, FillerConfig, MockBackend, SlotFiller, SlotFillerBackend,
    TemplateBackend,
};
pub use intent::{quick_detect, IntentParser, IntentParserConfig, ParsedIntent};
pub use parser::{
    parse_protocol_spec, parse_protocol_spec_str, CaptureSpec, CommandHandler, HandlerType,
    ParseError, ProtocolCommand, ProtocolError, ProtocolSpec, ProtocolState, ValidationConfig,
    ValidationType,
};
pub use router::{
    quick_generate, quick_route, GenerationResult, RouteDecision, Router, RouterConfig, RouterError,
};
pub use spec::{
    DataItem, DataType, DataValue, Slot, SlotContext, SlotSpec, SlotTest, TestCase, TestExpected,
    TestInput, TestStep,
};
pub use template::{extract_captures, ExpanderConfig, TemplateError, TemplateExpander};
pub use training::{
    extract_slot_training_data, ExtractionStats, SlotInput, SlotParams, SlotTrainingExample,
    SlotTrainingExtractor,
};
pub use types::{
    BufferOffset, Capture, CaptureType, CounterLimit, LoopCondition, MemWidth, SlotCategory,
    SlotType,
};
pub use validator::{
    validate_spec, validate_spec_file, SpecStats, SpecValidator, ValidationError, ValidationResult,
    ValidationWarning,
};
pub use verifier::{SlotVerifier, VerifyError, VerifyResult};
