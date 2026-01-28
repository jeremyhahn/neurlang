//! Compilation module
//!
//! Contains the copy-and-patch compiler and related utilities.

pub mod engine;

pub use engine::{AotCompiler, CompileError, CompiledCode, Compiler, CompilerStats, FastCompiler};
