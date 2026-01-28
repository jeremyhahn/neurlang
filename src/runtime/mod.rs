//! Runtime module
//!
//! Contains buffer pool, execution utilities, async I/O, and extension support.
//!
//! # Async I/O
//!
//! The async_io module provides non-blocking I/O operations with platform-specific
//! event loops:
//! - Linux: epoll
//! - macOS/BSD: kqueue
//! - Windows: IOCP
//!
//! # Extensions (Tier 2)
//!
//! The extensions module provides Rust FFI extensions for complex operations
//! like cryptography that shouldn't be written from scratch by the AI.
//!
//! Built-in crypto extensions include:
//! - SHA-256, HMAC-SHA256
//! - AES-256-GCM encryption/decryption
//! - Ed25519 signing/verification
//! - X25519 key exchange
//! - PBKDF2-SHA256 key derivation
//! - Secure random number generation

pub mod async_io;
pub mod buffer_pool;
pub mod extensions;
pub mod stdlib;

pub use async_io::{AsyncFile, AsyncRuntime, AsyncSocket, Event, Interest, Token};
pub use buffer_pool::{BufferPool, ExecutableBuffer, DEFAULT_BUFFER_SIZE};
pub use extensions::{
    ext_ids, CapPermissions, ExtCategory, ExtError, ExtFn, ExtSignature, ExtensionEntry,
    ExtensionRegistry, HandleManager, OwnedBuffer, SafeBuffer,
};
