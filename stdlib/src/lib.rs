//! Neurlang Standard Library - Rust Source
//!
//! This crate contains the Rust source code for stdlib functions that are compiled
//! to Neurlang IR using the Rust→IR compiler (`nl stdlib build`).
//!
//! # Design Philosophy
//!
//! 1. **Rust is the source of truth** - Write functions here, test in Rust
//! 2. **Generated .nl files** - The compiler produces lib/*.nl from this source
//! 3. **Verified correctness** - Tests run against Rust, then verify Neurlang matches
//!
//! # Supported Rust Subset
//!
//! The Rust→IR compiler supports:
//! - Integer arithmetic (u64, i64)
//! - Floating point (f64) via FPU opcodes
//! - Bitwise operations
//! - Control flow (if/else, while, loop, for i in 0..n)
//! - Functions with u64/f64 parameters and return values
//! - Local variables (let, let mut)
//!
//! # Export Convention
//!
//! Functions marked with `#[neurlang_export]` attribute (or public functions)
//! are compiled to Neurlang IR and exported as callable routines.

pub mod math;
pub mod float;
pub mod string;
pub mod array;
pub mod bitwise;
pub mod collections;

// Re-export commonly used functions
pub use math::*;
pub use float::*;
pub use string::*;
pub use array::*;
pub use bitwise::*;
