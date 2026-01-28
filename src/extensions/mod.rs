//! Extension System for Neurlang (Go-Style Package Management)
//!
//! Provides a Go-style extension system where the import path IS the source URL.
//!
//! # Design Philosophy
//!
//! - **No central registry**: Extensions are identified by their git URL
//! - **Versioned**: Support for semantic versioning with @version syntax
//! - **Local-first**: All extensions cached locally after first install
//! - **Hermetic**: Each session can have its own extension versions
//!
//! # Usage
//!
//! ```bash
//! # Install from git
//! neurlang extension add github.com/user/csv-parser
//! neurlang extension add github.com/user/csv-parser@v1.2.0
//!
//! # Save current program as extension
//! neurlang extension save my-utils
//!
//! # List installed extensions
//! neurlang extension list
//! ```
//!
//! # Directory Structure
//!
//! ```text
//! ~/.neurlang/extensions/
//! ├── local/                    # User-created extensions
//! │   └── my-utils/
//! │       ├── neurlang.json     # Manifest
//! │       └── main.nl           # Entry point
//! ├── cache/                    # Git-installed extensions
//! │   └── github.com/
//! │       └── user/
//! │           └── csv-parser@v1.2.0/
//! │               ├── neurlang.json
//! │               └── main.nl
//! └── extensions.lock           # Version lock file
//! ```

pub mod loader;
pub mod manifest;
pub mod registry;

pub use loader::{ExtensionLoader, LoadError, LoadedExtension};
pub use manifest::{ExtensionDependency, ExtensionExport, ExtensionManifest};
pub use registry::{ExtensionInfo, ExtensionRegistry, ExtensionSource};

use crate::ir::{rag_resolver::RagResolver, Assembler};

/// Create an Assembler with all extensions (bundled + user) registered in the RAG resolver.
///
/// This is the "seed bank" initialization that loads:
/// 1. All bundled extensions (crypto, JSON, HTTP, FS, SQLite, etc.) - ~100 functions
/// 2. All user-installed extensions from ~/.neurlang/extensions/
///
/// # Example
///
/// ```rust,ignore
/// use neurlang::extensions::create_configured_assembler;
///
/// let assembler = create_configured_assembler()?;
/// let program = assembler.assemble(r#"
///     ; Use bundled JSON extension via RAG
///     ext.call r0, @"parse JSON string", r1, r0
///     ; Or use explicit name
///     ext.call r1, @json_stringify, r0, r0
///     halt
/// "#)?;
/// ```
pub fn create_configured_assembler() -> Result<Assembler, std::io::Error> {
    // Start with a RAG resolver that has all bundled extensions
    let mut rag = RagResolver::new();

    // Load user extensions from registry
    if let Ok(registry) = ExtensionRegistry::new() {
        let count = registry.register_with_rag(&mut rag);
        // User extensions registered (count available for debugging if needed)
        let _ = count;
    }

    // Create assembler with the configured RAG resolver
    Ok(Assembler::with_rag_resolver(rag))
}

/// Get information about all registered extensions (bundled + user).
///
/// Returns a list of (name, description, id) tuples.
pub fn list_all_extensions() -> Vec<(String, String, u32)> {
    let mut rag = RagResolver::new();

    // Load user extensions
    if let Ok(registry) = ExtensionRegistry::new() {
        registry.register_with_rag(&mut rag);
    }

    // Get all extensions
    rag.all_extensions()
        .into_iter()
        .map(|ext| (ext.name, ext.description, ext.id))
        .collect()
}

/// Count of registered extensions.
///
/// Returns (bundled_count, user_count).
pub fn extension_count() -> (usize, usize) {
    let bundled_rag = RagResolver::new();
    let bundled_count = bundled_rag.all_extensions().len();

    let user_count = if let Ok(registry) = ExtensionRegistry::new() {
        registry
            .list()
            .map(|(_, info)| info.manifest.exports.len())
            .sum()
    } else {
        0
    };

    (bundled_count, user_count)
}
