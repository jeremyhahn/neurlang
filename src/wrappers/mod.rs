//! Safe Wrappers Module
//!
//! Production-grade safe wrappers providing memory-safe access to native capabilities.
//!
//! # Design Principles
//!
//! 1. **Correct Output** - Every operation produces correct results or returns error.
//!    No crashes. No undefined behavior. No memory corruption.
//!
//! 2. **Verified Implementation** - All wrappers are Rust code we control and test.
//!    No arbitrary C code running in our process.
//!
//! 3. **Guaranteed to Work** - Simple APIs that are hard to misuse.
//!    `@"compress buffer"` just works.
//!
//! 4. **Simple Mental Model** - Model learns one way to do things (the safe way).
//!    No "raw vs wrapper" decisions.
//!
//! # Usage
//!
//! ```text
//! ; Simple, consistent API - all operations return results or errors
//!
//! ; Compression
//! ext.call r0, @"compress", input_buffer
//! ext.call r1, @"decompress", compressed_buffer
//!
//! ; Hashing
//! ext.call r0, @"hash sha256", data
//!
//! ; TLS
//! ext.call r0, @"tls connect", host, port
//! ```

pub mod bridge;
pub mod buffer;
pub mod compression;
pub mod datetime;
pub mod encoding;
pub mod fs;
pub mod regex;
pub mod synonyms;
pub mod tls;
pub mod x509;

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

pub use bridge::register_wrappers;
pub use buffer::{BufferHandle, HandleManager, OwnedBuffer};
pub use synonyms::{expand_synonyms, SYNONYMS};

/// Error type for wrapper operations
#[derive(Debug, Clone)]
pub enum WrapperError {
    /// Invalid argument provided
    InvalidArg(String),
    /// IO operation failed
    IoError(String),
    /// Encoding/decoding failed
    EncodingError(String),
    /// Crypto operation failed
    CryptoError(String),
    /// TLS operation failed
    TlsError(String),
    /// X509 certificate operation failed
    X509Error(String),
    /// Compression/decompression failed
    CompressionError(String),
    /// Regex operation failed
    RegexError(String),
    /// DateTime operation failed
    DateTimeError(String),
    /// Handle not found
    HandleNotFound(u64),
    /// Operation not supported
    NotSupported(String),
    /// Not connected
    NotConnected,
    /// Buffer too small
    BufferTooSmall { required: usize, provided: usize },
}

impl fmt::Display for WrapperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WrapperError::InvalidArg(msg) => write!(f, "Invalid argument: {}", msg),
            WrapperError::IoError(msg) => write!(f, "IO error: {}", msg),
            WrapperError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            WrapperError::CryptoError(msg) => write!(f, "Crypto error: {}", msg),
            WrapperError::TlsError(msg) => write!(f, "TLS error: {}", msg),
            WrapperError::X509Error(msg) => write!(f, "X509 error: {}", msg),
            WrapperError::CompressionError(msg) => write!(f, "Compression error: {}", msg),
            WrapperError::RegexError(msg) => write!(f, "Regex error: {}", msg),
            WrapperError::DateTimeError(msg) => write!(f, "DateTime error: {}", msg),
            WrapperError::HandleNotFound(h) => write!(f, "Handle not found: {}", h),
            WrapperError::NotSupported(msg) => write!(f, "Not supported: {}", msg),
            WrapperError::NotConnected => write!(f, "Not connected"),
            WrapperError::BufferTooSmall { required, provided } => {
                write!(
                    f,
                    "Buffer too small: required {}, provided {}",
                    required, provided
                )
            }
        }
    }
}

impl std::error::Error for WrapperError {}

impl From<std::io::Error> for WrapperError {
    fn from(e: std::io::Error) -> Self {
        WrapperError::IoError(e.to_string())
    }
}

impl From<std::str::Utf8Error> for WrapperError {
    fn from(e: std::str::Utf8Error) -> Self {
        WrapperError::EncodingError(e.to_string())
    }
}

impl From<std::string::FromUtf8Error> for WrapperError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        WrapperError::EncodingError(e.to_string())
    }
}

/// Result type for wrapper operations
pub type WrapperResult<T> = Result<T, WrapperError>;

/// Function signature for a wrapper
pub type WrapperFn = Arc<dyn Fn(&[OwnedBuffer]) -> WrapperResult<OwnedBuffer> + Send + Sync>;

/// Information about a registered wrapper
#[derive(Clone)]
pub struct WrapperInfo {
    /// Unique ID for this wrapper
    pub id: u64,
    /// Primary name of the wrapper
    pub name: String,
    /// Description of what the wrapper does
    pub description: String,
    /// Keywords for RAG search
    pub keywords: Vec<String>,
    /// Category for organization
    pub category: WrapperCategory,
    /// Number of input arguments
    pub arg_count: usize,
}

impl fmt::Debug for WrapperInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WrapperInfo")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("category", &self.category)
            .field("arg_count", &self.arg_count)
            .finish()
    }
}

/// Categories of wrappers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WrapperCategory {
    /// Compression/decompression
    Compression,
    /// Cryptographic operations
    Crypto,
    /// HTTP operations
    Http,
    /// JSON operations
    Json,
    /// File system operations
    FileSystem,
    /// String operations
    Strings,
    /// Regular expressions
    Regex,
    /// Date/time operations
    DateTime,
    /// Encoding/decoding (base64, hex, url)
    Encoding,
    /// X509 certificates
    X509,
    /// TLS connections
    Tls,
    /// Other
    Other,
}

impl WrapperCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            WrapperCategory::Compression => "compression",
            WrapperCategory::Crypto => "crypto",
            WrapperCategory::Http => "http",
            WrapperCategory::Json => "json",
            WrapperCategory::FileSystem => "filesystem",
            WrapperCategory::Strings => "strings",
            WrapperCategory::Regex => "regex",
            WrapperCategory::DateTime => "datetime",
            WrapperCategory::Encoding => "encoding",
            WrapperCategory::X509 => "x509",
            WrapperCategory::Tls => "tls",
            WrapperCategory::Other => "other",
        }
    }
}

/// Registry of all available wrappers with RAG-based discovery
pub struct WrapperRegistry {
    /// Wrapper ID -> Wrapper info
    wrappers: HashMap<u64, WrapperInfo>,
    /// Wrapper ID -> Wrapper function
    functions: HashMap<u64, WrapperFn>,
    /// Name -> Wrapper ID (exact lookup)
    by_name: HashMap<String, u64>,
    /// Keyword -> [Wrapper IDs] (RAG search)
    keywords: HashMap<String, Vec<u64>>,
    /// Next available ID
    next_id: u64,
}

impl WrapperRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            wrappers: HashMap::new(),
            functions: HashMap::new(),
            by_name: HashMap::new(),
            keywords: HashMap::new(),
            next_id: 1,
        }
    }

    /// Create a new registry with all bundled wrappers registered
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        registry.register_builtins();
        registry
    }

    /// Register all built-in wrappers
    pub fn register_builtins(&mut self) {
        compression::register(self);
        encoding::register(self);
        fs::register(self);
        datetime::register(self);
        regex::register(self);
        x509::register(self);
        tls::register(self);
    }

    /// Register a wrapper
    pub fn register_wrapper<F>(
        &mut self,
        name: &str,
        description: &str,
        category: WrapperCategory,
        arg_count: usize,
        keywords: &[&str],
        func: F,
    ) -> u64
    where
        F: Fn(&[OwnedBuffer]) -> WrapperResult<OwnedBuffer> + Send + Sync + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;

        // Expand keywords with synonyms
        let mut all_keywords: Vec<String> = keywords.iter().map(|s| s.to_lowercase()).collect();
        all_keywords.extend(expand_synonyms(keywords));

        // Index keywords for RAG
        for kw in &all_keywords {
            self.keywords.entry(kw.clone()).or_default().push(id);
        }

        let info = WrapperInfo {
            id,
            name: name.to_string(),
            description: description.to_string(),
            keywords: all_keywords,
            category,
            arg_count,
        };

        self.wrappers.insert(id, info);
        self.functions.insert(id, Arc::new(func));
        self.by_name.insert(name.to_lowercase(), id);

        id
    }

    /// Get a wrapper by ID
    pub fn get(&self, id: u64) -> Option<&WrapperInfo> {
        self.wrappers.get(&id)
    }

    /// Get a wrapper by exact name
    pub fn get_by_name(&self, name: &str) -> Option<&WrapperInfo> {
        self.by_name
            .get(&name.to_lowercase())
            .and_then(|id| self.wrappers.get(id))
    }

    /// Get a wrapper ID by name
    pub fn get_id(&self, name: &str) -> Option<u64> {
        self.by_name.get(&name.to_lowercase()).copied()
    }

    /// RAG search: @"shrink file size" -> compress
    pub fn search(&self, query: &str) -> Option<u64> {
        let query_lower = query.to_lowercase();
        let words: Vec<&str> = query_lower.split_whitespace().collect();

        // Score each wrapper by keyword overlap
        let mut scores: HashMap<u64, f32> = HashMap::new();
        for word in &words {
            // Exact keyword match
            if let Some(ids) = self.keywords.get(*word) {
                for &id in ids {
                    *scores.entry(id).or_default() += 1.0;
                }
            }
            // Partial match
            for (kw, ids) in &self.keywords {
                if kw.contains(*word) || word.contains(kw.as_str()) {
                    for &id in ids {
                        *scores.entry(id).or_default() += 0.5;
                    }
                }
            }
        }

        // Return best match
        scores
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .filter(|(_, score)| *score >= 0.5)
            .map(|(id, _)| id)
    }

    /// Search for wrappers matching a query and return top N results
    pub fn search_top(&self, query: &str, limit: usize) -> Vec<&WrapperInfo> {
        let query_lower = query.to_lowercase();
        let words: Vec<&str> = query_lower.split_whitespace().collect();

        // Score each wrapper
        let mut scores: Vec<(u64, f32)> = Vec::new();
        for (id, info) in &self.wrappers {
            let mut score = 0.0f32;
            for word in &words {
                // Description match (worth more)
                if info.description.to_lowercase().contains(*word) {
                    score += 2.0;
                }
                // Keyword match
                for kw in &info.keywords {
                    if kw == *word {
                        score += 1.0;
                    } else if kw.contains(*word) || word.contains(kw.as_str()) {
                        score += 0.5;
                    }
                }
            }
            if score > 0.0 {
                scores.push((*id, score));
            }
        }

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scores
            .into_iter()
            .take(limit)
            .filter_map(|(id, _)| self.wrappers.get(&id))
            .collect()
    }

    /// Call a wrapper by ID
    pub fn call(&self, id: u64, args: &[OwnedBuffer]) -> WrapperResult<OwnedBuffer> {
        let func = self
            .functions
            .get(&id)
            .ok_or(WrapperError::HandleNotFound(id))?;
        func(args)
    }

    /// Call a wrapper by name
    pub fn call_by_name(&self, name: &str, args: &[OwnedBuffer]) -> WrapperResult<OwnedBuffer> {
        let id = self
            .by_name
            .get(&name.to_lowercase())
            .ok_or_else(|| WrapperError::InvalidArg(format!("Unknown wrapper: {}", name)))?;
        self.call(*id, args)
    }

    /// List all registered wrappers
    pub fn list(&self) -> Vec<&WrapperInfo> {
        self.wrappers.values().collect()
    }

    /// List wrappers by category
    pub fn list_by_category(&self, category: WrapperCategory) -> Vec<&WrapperInfo> {
        self.wrappers
            .values()
            .filter(|w| w.category == category)
            .collect()
    }

    /// Get the number of registered wrappers
    pub fn len(&self) -> usize {
        self.wrappers.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.wrappers.is_empty()
    }
}

impl Default for WrapperRegistry {
    fn default() -> Self {
        Self::with_builtins()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = WrapperRegistry::with_builtins();
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_wrapper_registration() {
        let mut registry = WrapperRegistry::new();

        let id = registry.register_wrapper(
            "test_wrapper",
            "A test wrapper for unit testing",
            WrapperCategory::Other,
            1,
            &["test", "wrapper", "unit"],
            |args| {
                if args.is_empty() {
                    return Err(WrapperError::InvalidArg("No input".to_string()));
                }
                Ok(args[0].clone())
            },
        );

        assert!(registry.get(id).is_some());
        assert!(registry.get_by_name("test_wrapper").is_some());
    }

    #[test]
    fn test_rag_search() {
        let mut registry = WrapperRegistry::new();

        registry.register_wrapper(
            "compress",
            "Compress data using zlib",
            WrapperCategory::Compression,
            1,
            &["compress", "shrink", "deflate", "zip"],
            |_| Ok(OwnedBuffer::new()),
        );

        // Should find by keyword
        let id = registry.search("shrink data");
        assert!(id.is_some());

        // Should find by partial match
        let id = registry.search("compress file");
        assert!(id.is_some());
    }
}
