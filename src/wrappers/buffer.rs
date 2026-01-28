//! Owned Buffer Type
//!
//! Memory-safe buffer for passing data between wrapper operations.
//!
//! This module re-exports the canonical OwnedBuffer and HandleManager from
//! `runtime::extensions::buffer` to ensure a single type is used throughout
//! the codebase.
//!
//! # Example
//!
//! ```rust
//! use neurlang::wrappers::OwnedBuffer;
//!
//! // Create from data
//! let buf = OwnedBuffer::from_slice(b"Hello, World!");
//! assert_eq!(buf.len(), 13);
//!
//! // Access data
//! assert_eq!(buf.as_slice(), b"Hello, World!");
//! ```

// Re-export from the canonical location to avoid type duplication
pub use crate::runtime::extensions::buffer::{BufferHandle, HandleManager, OwnedBuffer};
