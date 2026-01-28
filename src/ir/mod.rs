//! IR (Intermediate Representation) module
//!
//! Defines the 32-opcode binary IR format and assembler/disassembler.
//!
//! # Intrinsics (Tier 1)
//!
//! The intrinsics module provides ~50 common algorithms as "macro tokens"
//! that expand to optimized Neurlang IR at generation time. This gives zero
//! runtime overhead while guaranteeing correctness on first try.
//!
//! Usage: `@memcpy r0, r1, 256` expands to an optimized copy loop.
//!
//! # RAG-Based Extension Resolution
//!
//! The rag_resolver module provides semantic intent resolution for extensions.
//! Instead of hardcoding extension IDs, the model can emit:
//!
//! ```text
//! EXT.CALL @"parse JSON string", r0, r1
//! ```
//!
//! The RAG resolver matches the intent to the appropriate extension ID.

pub mod assembler;
pub mod format;
pub mod intrinsics;
pub mod rag_resolver;

pub use assembler::{AsmError, Assembler, Disassembler, DATA_BASE};
pub use format::*;
pub use intrinsics::{
    ArgType, IntrinsicArg, IntrinsicCall, IntrinsicCategory, IntrinsicDef, IntrinsicError,
    IntrinsicRegistry, PatchField, PatchPoint,
};
pub use rag_resolver::{RagResolver, ResolvedExtension};
