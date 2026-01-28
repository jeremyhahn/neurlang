//! FFI Module for Neurlang
//!
//! Provides safe foreign function interface for calling Go and C libraries
//! from Neurlang programs.
//!
//! # Architecture
//!
//! ```text
//! Neurlang Program
//!       │
//!       ▼
//! ext.call @"ffi:mylib.my_function", args...
//!       │
//!       ▼
//! FFI Registry (resolves library + function)
//!       │
//!       ▼
//! Dynamic Loader (libloading)
//!       │
//!       ▼
//! Native Function Call
//! ```
//!
//! # Supported Libraries
//!
//! - **C Libraries**: Direct FFI via dynamic loading
//! - **Go Libraries**: Via cgo-generated C bindings (export C functions from Go)
//!
//! # Example
//!
//! ```ignore
//! // Load a C library
//! let mut registry = FfiRegistry::new();
//! registry.load_library("mylib", "/path/to/libmylib.so")?;
//!
//! // Call a function
//! let result = registry.call("mylib", "add", &[1u64, 2u64])?;
//! ```

mod loader;
mod registry;
mod types;

pub use loader::{DynamicLibrary, LibraryLoader};
pub use registry::{FfiError, FfiFunctionInfo, FfiRegistry};
pub use types::{FfiSignature, FfiType, FfiValue};

#[cfg(test)]
mod tests;
