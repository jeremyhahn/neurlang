//! Unified Extension System for Neurlang
//!
//! All extensions (crypto, collections, strings, JSON, HTTP, compression, etc.)
//! are consolidated here. Extensions are Rust FFI functions that the AI model
//! calls via `ext.call` instructions.
//!
//! # Architecture
//!
//! ```text
//! Model generates: ext.call @"hash sha256", r0, r1, r2
//!                          |
//!                          v
//! RAG resolves:    "hash sha256" -> ID 1
//!                          |
//!                          v
//! Runtime calls:   registry.call(1, args, outputs)
//! ```
//!
//! # Extension Categories
//!
//! | Category | ID Range | Description |
//! |----------|----------|-------------|
//! | Crypto | 1-99 | Hash, encrypt, sign, derive keys |
//! | Collections | 100-119 | Vec, HashMap |
//! | Strings | 140-169 | String operations |
//! | JSON | 170-189 | Parse, stringify, manipulate |
//! | HTTP | 190-209 | HTTP client |
//! | Compression | 400-419 | zlib, lz4, zstd |
//! | Encoding | 420-439 | base64, hex, url |
//! | DateTime | 440-459 | Date/time operations |
//! | Regex | 460-479 | Pattern matching |
//! | FileSystem | 480-499 | File I/O |
//! | TLS | 500-519 | TLS connections |
//! | X509 | 520-539 | Certificate handling |
//! | User | 1000+ | User-installed extensions |

// Submodules - each registers its extensions
pub mod buffer;
pub mod collections;
pub mod compression;
pub mod crypto;
pub mod datetime;
pub mod encoding;
pub mod fs;
pub mod http;
pub mod json;
pub mod regex;
pub mod strings;
pub mod synonyms;
pub mod tls;
pub mod x509;

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

pub use buffer::{BufferHandle, HandleManager, OwnedBuffer};
pub use synonyms::{expand_synonyms, SYNONYMS};

// =============================================================================
// Error Types
// =============================================================================

/// Error type for extension operations
#[derive(Debug, Clone)]
pub enum ExtError {
    /// Extension not found
    NotFound(u32),
    /// Invalid argument count
    InvalidArgCount { expected: usize, got: usize },
    /// Capability violation
    CapabilityViolation(String),
    /// Buffer bounds exceeded
    BoundsViolation {
        offset: usize,
        len: usize,
        cap_len: usize,
    },
    /// Taint violation
    TaintViolation(String),
    /// Extension returned an error
    ExtensionError(String),
    /// Invalid extension ID
    InvalidId,
    /// Extension panicked
    Panic(String),
}

impl fmt::Display for ExtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtError::NotFound(id) => write!(f, "Extension {} not found", id),
            ExtError::InvalidArgCount { expected, got } => {
                write!(f, "Expected {} arguments, got {}", expected, got)
            }
            ExtError::CapabilityViolation(msg) => write!(f, "Capability violation: {}", msg),
            ExtError::BoundsViolation {
                offset,
                len,
                cap_len,
            } => {
                write!(
                    f,
                    "Bounds violation: offset {} + len {} > cap_len {}",
                    offset, len, cap_len
                )
            }
            ExtError::TaintViolation(msg) => write!(f, "Taint violation: {}", msg),
            ExtError::ExtensionError(msg) => write!(f, "Extension error: {}", msg),
            ExtError::InvalidId => write!(f, "Invalid extension ID"),
            ExtError::Panic(msg) => write!(f, "Extension panicked: {}", msg),
        }
    }
}

impl std::error::Error for ExtError {}

// =============================================================================
// Capability Permissions
// =============================================================================

/// Permission flags for capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapPermissions(pub u8);

impl CapPermissions {
    pub const READ: u8 = 0b0001;
    pub const WRITE: u8 = 0b0010;
    pub const EXEC: u8 = 0b0100;

    pub const fn new(bits: u8) -> Self {
        Self(bits)
    }

    pub const fn can_read(&self) -> bool {
        self.0 & Self::READ != 0
    }

    pub const fn can_write(&self) -> bool {
        self.0 & Self::WRITE != 0
    }

    pub const fn can_exec(&self) -> bool {
        self.0 & Self::EXEC != 0
    }

    pub const fn read_only() -> Self {
        Self(Self::READ)
    }

    pub const fn read_write() -> Self {
        Self(Self::READ | Self::WRITE)
    }
}

// =============================================================================
// Safe Buffer
// =============================================================================

/// A safe buffer that enforces capability restrictions
#[derive(Clone)]
pub struct SafeBuffer {
    base: *const u8,
    len: usize,
    perms: CapPermissions,
    taint: u8,
}

unsafe impl Send for SafeBuffer {}
unsafe impl Sync for SafeBuffer {}

impl SafeBuffer {
    pub unsafe fn from_raw(base: *const u8, len: usize, perms: CapPermissions, taint: u8) -> Self {
        Self {
            base,
            len,
            perms,
            taint,
        }
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        Self {
            base: slice.as_ptr(),
            len: slice.len(),
            perms: CapPermissions::read_only(),
            taint: 0,
        }
    }

    pub fn from_mut_slice(slice: &mut [u8]) -> Self {
        Self {
            base: slice.as_ptr(),
            len: slice.len(),
            perms: CapPermissions::read_write(),
            taint: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn taint(&self) -> u8 {
        self.taint
    }

    pub fn is_tainted(&self) -> bool {
        self.taint > 0
    }

    pub fn permissions(&self) -> CapPermissions {
        self.perms
    }

    pub fn as_slice(&self) -> Result<&[u8], ExtError> {
        if !self.perms.can_read() {
            return Err(ExtError::CapabilityViolation(
                "Cannot read buffer".to_string(),
            ));
        }
        Ok(unsafe { std::slice::from_raw_parts(self.base, self.len) })
    }

    pub fn read_range(&self, offset: usize, len: usize) -> Result<&[u8], ExtError> {
        if !self.perms.can_read() {
            return Err(ExtError::CapabilityViolation(
                "Cannot read buffer".to_string(),
            ));
        }
        if offset + len > self.len {
            return Err(ExtError::BoundsViolation {
                offset,
                len,
                cap_len: self.len,
            });
        }
        Ok(unsafe { std::slice::from_raw_parts(self.base.add(offset), len) })
    }

    pub fn write(&self, offset: usize, data: &[u8]) -> Result<usize, ExtError> {
        if !self.perms.can_write() {
            return Err(ExtError::CapabilityViolation(
                "Cannot write to buffer".to_string(),
            ));
        }
        if offset + data.len() > self.len {
            return Err(ExtError::BoundsViolation {
                offset,
                len: data.len(),
                cap_len: self.len,
            });
        }
        unsafe {
            let dest = self.base as *mut u8;
            std::ptr::copy_nonoverlapping(data.as_ptr(), dest.add(offset), data.len());
        }
        Ok(data.len())
    }

    pub fn write_all(&self, data: &[u8]) -> Result<usize, ExtError> {
        self.write(0, data)
    }

    pub fn as_mut_slice(&mut self) -> Result<&mut [u8], ExtError> {
        if !self.perms.can_write() {
            return Err(ExtError::CapabilityViolation(
                "Cannot write to buffer".to_string(),
            ));
        }
        Ok(unsafe { std::slice::from_raw_parts_mut(self.base as *mut u8, self.len) })
    }

    pub fn restrict(
        &self,
        offset: usize,
        new_len: usize,
        new_perms: CapPermissions,
    ) -> Result<SafeBuffer, ExtError> {
        if offset + new_len > self.len {
            return Err(ExtError::BoundsViolation {
                offset,
                len: new_len,
                cap_len: self.len,
            });
        }
        if new_perms.0 & !self.perms.0 != 0 {
            return Err(ExtError::CapabilityViolation(
                "Cannot add permissions when restricting capability".to_string(),
            ));
        }
        Ok(SafeBuffer {
            base: unsafe { self.base.add(offset) },
            len: new_len,
            perms: new_perms,
            taint: self.taint,
        })
    }
}

// =============================================================================
// Extension Category
// =============================================================================

/// Category of extension for organization and documentation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtCategory {
    Crypto,
    Collections,
    Strings,
    Json,
    Http,
    Compression,
    Encoding,
    DateTime,
    Regex,
    FileSystem,
    Tls,
    X509,
    Other,
}

impl fmt::Display for ExtCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtCategory::Crypto => write!(f, "crypto"),
            ExtCategory::Collections => write!(f, "collections"),
            ExtCategory::Strings => write!(f, "strings"),
            ExtCategory::Json => write!(f, "json"),
            ExtCategory::Http => write!(f, "http"),
            ExtCategory::Compression => write!(f, "compression"),
            ExtCategory::Encoding => write!(f, "encoding"),
            ExtCategory::DateTime => write!(f, "datetime"),
            ExtCategory::Regex => write!(f, "regex"),
            ExtCategory::FileSystem => write!(f, "filesystem"),
            ExtCategory::Tls => write!(f, "tls"),
            ExtCategory::X509 => write!(f, "x509"),
            ExtCategory::Other => write!(f, "other"),
        }
    }
}

// =============================================================================
// Extension Function Types
// =============================================================================

/// Extension function signature: (args, outputs) -> Result<return_value, error>
pub type ExtFn = Arc<dyn Fn(&[u64], &mut [u64]) -> Result<i64, ExtError> + Send + Sync>;

/// Extension signature metadata
#[derive(Clone)]
pub struct ExtSignature {
    pub name: String,
    pub description: String,
    pub arg_count: usize,
    pub has_return: bool,
    pub category: ExtCategory,
}

impl fmt::Debug for ExtSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtSignature")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("arg_count", &self.arg_count)
            .field("has_return", &self.has_return)
            .field("category", &self.category)
            .finish()
    }
}

/// A registered extension entry
pub struct ExtensionEntry {
    pub id: u32,
    pub signature: ExtSignature,
    pub func: ExtFn,
}

impl fmt::Debug for ExtensionEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtensionEntry")
            .field("id", &self.id)
            .field("signature", &self.signature)
            .finish()
    }
}

/// Mock response for testing - supports stateful mocks with multiple return values
#[derive(Clone)]
pub struct MockResponse {
    /// Sequence of return values (cycles through on each call)
    pub return_values: Vec<i64>,
    pub outputs: Vec<u64>,
    /// Call counter for stateful mocks (uses Cell for interior mutability)
    call_count: std::cell::Cell<usize>,
}

impl MockResponse {
    pub fn new(return_value: i64, outputs: Vec<u64>) -> Self {
        Self {
            return_values: vec![return_value],
            outputs,
            call_count: std::cell::Cell::new(0),
        }
    }

    pub fn with_sequence(return_values: Vec<i64>, outputs: Vec<u64>) -> Self {
        Self {
            return_values,
            outputs,
            call_count: std::cell::Cell::new(0),
        }
    }

    /// Get the next return value, advancing the call counter
    pub fn next_return_value(&self) -> i64 {
        let count = self.call_count.get();
        let value = if count < self.return_values.len() {
            self.return_values[count]
        } else {
            // Repeat last value if sequence exhausted
            *self.return_values.last().unwrap_or(&0)
        };
        self.call_count.set(count + 1);
        value
    }

    /// Reset call counter (for running multiple tests)
    pub fn reset(&self) {
        self.call_count.set(0);
    }
}

// =============================================================================
// Extension Registry
// =============================================================================

pub struct ExtensionRegistry {
    by_id: HashMap<u32, ExtensionEntry>,
    by_name: HashMap<String, u32>,
    next_id: u32,
    mocks: HashMap<u32, MockResponse>,
    mock_mode: bool,
}

impl ExtensionRegistry {
    /// Create a new registry with all built-in extensions registered
    pub fn new() -> Self {
        let mut registry = Self {
            by_id: HashMap::new(),
            by_name: HashMap::new(),
            next_id: 1,
            mocks: HashMap::new(),
            mock_mode: false,
        };
        registry.register_builtins();
        registry
    }

    /// Create a registry in mock mode for testing
    pub fn new_with_mocks() -> Self {
        let mut registry = Self::new();
        registry.mock_mode = true;
        registry
    }

    pub fn set_mock_mode(&mut self, enabled: bool) {
        self.mock_mode = enabled;
    }

    pub fn is_mock_mode(&self) -> bool {
        self.mock_mode
    }

    pub fn set_mock(&mut self, id: u32, return_value: i64, outputs: Vec<u64>) {
        self.mocks
            .insert(id, MockResponse::new(return_value, outputs));
        self.mock_mode = true;
    }

    /// Set a stateful mock with a sequence of return values
    /// Each call returns the next value; repeats last value when exhausted
    pub fn set_mock_sequence(&mut self, id: u32, return_values: Vec<i64>, outputs: Vec<u64>) {
        self.mocks
            .insert(id, MockResponse::with_sequence(return_values, outputs));
        self.mock_mode = true;
    }

    pub fn set_mock_by_name(&mut self, name: &str, return_value: i64, outputs: Vec<u64>) -> bool {
        if let Some(id) = self.get_id(name) {
            self.set_mock(id, return_value, outputs);
            true
        } else {
            false
        }
    }

    pub fn clear_mock(&mut self, id: u32) {
        self.mocks.remove(&id);
    }

    pub fn clear_all_mocks(&mut self) {
        self.mocks.clear();
        self.mock_mode = false;
    }

    /// Reset all mock call counters (for running multiple tests with same mocks)
    pub fn reset_mock_counters(&self) {
        for mock in self.mocks.values() {
            mock.reset();
        }
    }

    pub fn get_mocked_ids(&self) -> Vec<u32> {
        self.mocks.keys().copied().collect()
    }

    /// Register an extension and return its ID
    pub fn register(
        &mut self,
        name: &str,
        description: &str,
        arg_count: usize,
        has_return: bool,
        category: ExtCategory,
        func: ExtFn,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        let signature = ExtSignature {
            name: name.to_string(),
            description: description.to_string(),
            arg_count,
            has_return,
            category,
        };

        self.by_name.insert(name.to_string(), id);
        self.by_id.insert(
            id,
            ExtensionEntry {
                id,
                signature,
                func,
            },
        );

        id
    }

    /// Register an extension with a specific ID
    pub fn register_with_id(
        &mut self,
        id: u32,
        name: &str,
        description: &str,
        arg_count: usize,
        has_return: bool,
        category: ExtCategory,
        func: ExtFn,
    ) {
        let signature = ExtSignature {
            name: name.to_string(),
            description: description.to_string(),
            arg_count,
            has_return,
            category,
        };

        self.by_name.insert(name.to_string(), id);
        self.by_id.insert(
            id,
            ExtensionEntry {
                id,
                signature,
                func,
            },
        );

        if id >= self.next_id {
            self.next_id = id + 1;
        }
    }

    pub fn get(&self, id: u32) -> Option<&ExtensionEntry> {
        self.by_id.get(&id)
    }

    pub fn get_id(&self, name: &str) -> Option<u32> {
        self.by_name.get(name).copied()
    }

    pub fn get_by_name(&self, name: &str) -> Option<&ExtensionEntry> {
        self.by_name.get(name).and_then(|id| self.by_id.get(id))
    }

    /// Call an extension by ID
    pub fn call(&self, id: u32, args: &[u64], outputs: &mut [u64]) -> Result<i64, ExtError> {
        if self.mock_mode {
            if let Some(mock) = self.mocks.get(&id) {
                for (i, &val) in mock.outputs.iter().enumerate() {
                    if i < outputs.len() {
                        outputs[i] = val;
                    }
                }
                return Ok(mock.next_return_value());
            }
        }

        let ext = self.get(id).ok_or(ExtError::NotFound(id))?;

        if args.len() < ext.signature.arg_count {
            return Err(ExtError::InvalidArgCount {
                expected: ext.signature.arg_count,
                got: args.len(),
            });
        }

        (ext.func)(args, outputs)
    }

    pub fn list(&self) -> Vec<&ExtensionEntry> {
        self.by_id.values().collect()
    }

    pub fn list_by_category(&self, category: ExtCategory) -> Vec<&ExtensionEntry> {
        self.by_id
            .values()
            .filter(|e| e.signature.category == category)
            .collect()
    }

    /// Register all built-in extensions
    fn register_builtins(&mut self) {
        crypto::register_crypto(self);
        collections::register_collections(self);
        strings::register_strings(self);
        json::register_json(self);
        http::register_http(self);
        compression::register_compression(self);
        encoding::register_encoding(self);
        datetime::register_datetime(self);
        regex::register_regex(self);
        fs::register_fs(self);
        tls::register_tls(self);
        x509::register_x509(self);
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Extension IDs
// =============================================================================

/// Predefined extension IDs (stable for training)
pub mod ext_ids {
    // Crypto (1-99)
    pub const SHA256: u32 = 1;
    pub const HMAC_SHA256: u32 = 2;
    pub const AES256_GCM_ENCRYPT: u32 = 3;
    pub const AES256_GCM_DECRYPT: u32 = 4;
    pub const CONSTANT_TIME_EQ: u32 = 5;
    pub const SECURE_RANDOM: u32 = 6;
    pub const PBKDF2_SHA256: u32 = 7;
    pub const ED25519_SIGN: u32 = 8;
    pub const ED25519_VERIFY: u32 = 9;
    pub const X25519_DERIVE: u32 = 10;
    pub const CHACHA20_POLY1305_ENCRYPT: u32 = 11;
    pub const CHACHA20_POLY1305_DECRYPT: u32 = 12;
    pub const XCHACHA20_POLY1305_ENCRYPT: u32 = 13;
    pub const XCHACHA20_POLY1305_DECRYPT: u32 = 14;
    pub const SHA384: u32 = 15;
    pub const SHA512: u32 = 16;
    pub const SHA3_256: u32 = 17;
    pub const SHA3_512: u32 = 18;
    pub const BLAKE2B_512: u32 = 19;
    pub const BLAKE2S_256: u32 = 20;
    pub const HMAC_SHA384: u32 = 21;
    pub const HMAC_SHA512: u32 = 22;
    pub const HKDF_SHA256_EXTRACT: u32 = 23;
    pub const HKDF_SHA256_EXPAND: u32 = 24;
    pub const P256_ECDSA_SIGN: u32 = 25;
    pub const P256_ECDSA_VERIFY: u32 = 26;
    pub const P256_ECDH: u32 = 27;
    pub const P384_ECDSA_SIGN: u32 = 28;
    pub const P384_ECDSA_VERIFY: u32 = 29;
    pub const P384_ECDH: u32 = 30;
    pub const RSA_PKCS1_SIGN_SHA256: u32 = 31;
    pub const RSA_PKCS1_VERIFY_SHA256: u32 = 32;
    pub const RSA_OAEP_ENCRYPT_SHA256: u32 = 33;
    pub const RSA_OAEP_DECRYPT_SHA256: u32 = 34;
    pub const ARGON2ID_HASH: u32 = 35;
    pub const X509_PARSE_CERT: u32 = 36;
    pub const X509_GET_PUBLIC_KEY: u32 = 37;
    pub const SHA1: u32 = 38; // Legacy, for WebSocket handshake
    pub const P256_GENERATE_KEYPAIR: u32 = 39;
    pub const P384_GENERATE_KEYPAIR: u32 = 40;
    pub const P521_ECDSA_SIGN: u32 = 41;
    pub const P521_ECDSA_VERIFY: u32 = 42;
    pub const P521_ECDH: u32 = 43;
    pub const P521_GENERATE_KEYPAIR: u32 = 44;
    pub const RSA_GENERATE_KEYPAIR: u32 = 45; // Fixed: was conflicting with SHA1 at 38

    // Collections (100-119)
    pub const VEC_NEW: u32 = 100;
    pub const VEC_WITH_CAPACITY: u32 = 101;
    pub const VEC_PUSH: u32 = 102;
    pub const VEC_POP: u32 = 103;
    pub const VEC_GET: u32 = 104;
    pub const VEC_SET: u32 = 105;
    pub const VEC_LEN: u32 = 106;
    pub const VEC_CAPACITY: u32 = 107;
    pub const VEC_CLEAR: u32 = 108;
    pub const VEC_FREE: u32 = 109;
    pub const VEC_EXTEND: u32 = 110;
    pub const VEC_INSERT: u32 = 111;
    pub const VEC_REMOVE: u32 = 112;
    pub const HASHMAP_NEW: u32 = 120;
    pub const HASHMAP_INSERT: u32 = 121;
    pub const HASHMAP_GET: u32 = 122;
    pub const HASHMAP_REMOVE: u32 = 123;
    pub const HASHMAP_CONTAINS: u32 = 124;
    pub const HASHMAP_LEN: u32 = 125;
    pub const HASHMAP_CLEAR: u32 = 126;
    pub const HASHMAP_FREE: u32 = 127;
    pub const HASHMAP_KEYS: u32 = 128;
    pub const HASHMAP_VALUES: u32 = 129;

    // Strings (140-169)
    pub const STRING_NEW: u32 = 140;
    pub const STRING_FROM_BYTES: u32 = 141;
    pub const STRING_LEN: u32 = 142;
    pub const STRING_CONCAT: u32 = 143;
    pub const STRING_SUBSTR: u32 = 144;
    pub const STRING_FIND: u32 = 145;
    pub const STRING_REPLACE: u32 = 146;
    pub const STRING_SPLIT: u32 = 147;
    pub const STRING_TRIM: u32 = 148;
    pub const STRING_TO_UPPER: u32 = 149;
    pub const STRING_TO_LOWER: u32 = 150;
    pub const STRING_STARTS_WITH: u32 = 151;
    pub const STRING_ENDS_WITH: u32 = 152;
    pub const STRING_TO_BYTES: u32 = 153;
    pub const STRING_FREE: u32 = 154;
    pub const STRING_PARSE_INT: u32 = 155;
    pub const STRING_PARSE_FLOAT: u32 = 156;
    pub const STRING_FROM_INT: u32 = 157;
    pub const STRING_FROM_FLOAT: u32 = 158;

    // JSON (170-189)
    pub const JSON_PARSE: u32 = 170;
    pub const JSON_STRINGIFY: u32 = 171;
    pub const JSON_GET: u32 = 172;
    pub const JSON_SET: u32 = 173;
    pub const JSON_GET_TYPE: u32 = 174;
    pub const JSON_ARRAY_LEN: u32 = 175;
    pub const JSON_ARRAY_GET: u32 = 176;
    pub const JSON_ARRAY_PUSH: u32 = 177;
    pub const JSON_OBJECT_KEYS: u32 = 178;
    pub const JSON_FREE: u32 = 179;
    pub const JSON_NEW_OBJECT: u32 = 180;
    pub const JSON_NEW_ARRAY: u32 = 181;

    // HTTP (190-209)
    pub const HTTP_GET: u32 = 190;
    pub const HTTP_POST: u32 = 191;
    pub const HTTP_PUT: u32 = 192;
    pub const HTTP_DELETE: u32 = 193;
    pub const HTTP_RESPONSE_STATUS: u32 = 194;
    pub const HTTP_RESPONSE_BODY: u32 = 195;
    pub const HTTP_RESPONSE_FREE: u32 = 196;
    pub const HTTP_GET_WITH_HEADERS: u32 = 197;
    pub const HTTP_POST_WITH_HEADERS: u32 = 198;

    // Compression (400-419)
    pub const ZLIB_COMPRESS: u32 = 400;
    pub const ZLIB_DECOMPRESS: u32 = 401;
    pub const GZIP_COMPRESS: u32 = 402;
    pub const GZIP_DECOMPRESS: u32 = 403;
    pub const LZ4_COMPRESS: u32 = 404;
    pub const LZ4_DECOMPRESS: u32 = 405;
    pub const ZSTD_COMPRESS: u32 = 406;
    pub const ZSTD_DECOMPRESS: u32 = 407;

    // Encoding (420-439)
    pub const BASE64_ENCODE: u32 = 420;
    pub const BASE64_DECODE: u32 = 421;
    pub const HEX_ENCODE: u32 = 422;
    pub const HEX_DECODE: u32 = 423;
    pub const URL_ENCODE: u32 = 424;
    pub const URL_DECODE: u32 = 425;

    // DateTime (440-459)
    pub const DATETIME_NOW: u32 = 440;
    pub const DATETIME_PARSE: u32 = 441;
    pub const DATETIME_FORMAT: u32 = 442;
    pub const DATETIME_ADD_DAYS: u32 = 443;
    pub const DATETIME_DIFF: u32 = 444;

    // Regex (460-479)
    pub const REGEX_MATCH: u32 = 460;
    pub const REGEX_FIND: u32 = 461;
    pub const REGEX_REPLACE: u32 = 462;
    pub const REGEX_SPLIT: u32 = 463;

    // FileSystem (480-499)
    pub const FS_READ: u32 = 480;
    pub const FS_WRITE: u32 = 481;
    pub const FS_EXISTS: u32 = 482;
    pub const FS_DELETE: u32 = 483;
    pub const FS_MKDIR: u32 = 484;
    pub const FS_LIST_DIR: u32 = 485;

    // TLS (500-519)
    pub const TLS_CONNECT: u32 = 500;
    pub const TLS_SEND: u32 = 501;
    pub const TLS_RECV: u32 = 502;
    pub const TLS_CLOSE: u32 = 503;

    // X509 (520-539)
    pub const X509_CREATE_SELF_SIGNED: u32 = 520;
    pub const X509_PARSE: u32 = 521;
    pub const X509_VERIFY: u32 = 522;
    pub const X509_GET_SUBJECT: u32 = 523;
    pub const X509_GET_ISSUER: u32 = 524;

    // SQLite (260-279)
    pub const SQLITE_OPEN: u32 = 260;
    pub const SQLITE_CLOSE: u32 = 261;
    pub const SQLITE_EXEC: u32 = 262;
    pub const SQLITE_QUERY: u32 = 263;
    pub const SQLITE_PREPARE: u32 = 264;
    pub const SQLITE_BIND_INT: u32 = 265;
    pub const SQLITE_BIND_TEXT: u32 = 266;
    pub const SQLITE_BIND_BLOB: u32 = 267;
    pub const SQLITE_STEP: u32 = 268;
    pub const SQLITE_RESET: u32 = 269;
    pub const SQLITE_FINALIZE: u32 = 270;
    pub const SQLITE_COLUMN_INT: u32 = 271;
    pub const SQLITE_COLUMN_TEXT: u32 = 272;
    pub const SQLITE_COLUMN_BLOB: u32 = 273;
    pub const SQLITE_LAST_INSERT_ID: u32 = 274;
    pub const SQLITE_CHANGES: u32 = 275;

    // UUID (330-339)
    pub const UUID_V4: u32 = 330;
    pub const UUID_V5: u32 = 331;
    pub const UUID_PARSE: u32 = 332;
    pub const UUID_TO_STRING: u32 = 333;
    pub const UUID_V7: u32 = 334;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ExtensionRegistry::new();
        assert!(registry.get_id("sha256").is_some());
    }

    #[test]
    fn test_safe_buffer() {
        let data = vec![1u8, 2, 3, 4, 5];
        let buffer = SafeBuffer::from_slice(&data);
        assert_eq!(buffer.len(), 5);
        assert!(buffer.permissions().can_read());
        assert!(!buffer.permissions().can_write());
    }
}
