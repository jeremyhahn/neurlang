//! JIT Execution Engine with Full I/O Support
//!
//! This module provides a proper JIT execution engine that:
//! - Handles all 32 opcodes including control flow
//! - Integrates with IORuntime for file, network, and time operations
//! - Manages memory for program execution
//! - Runs at native speed without instruction limits

mod context;
mod executor;
mod handlers;

pub use context::JitContext;
pub use executor::{
    execute_multi_worker, execute_with_strategy, JitExecutor, JitResult, WorkerStrategy,
};
