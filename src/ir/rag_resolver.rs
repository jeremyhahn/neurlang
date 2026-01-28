//! RAG-Based Extension Resolution
//!
//! Resolves intent descriptions to extension IDs using semantic similarity search.
//!
//! # Overview
//!
//! Instead of training the model on specific extension IDs, the model emits
//! intent descriptions like `@"parse JSON string"`. This module resolves those
//! intents to actual extension IDs via simple keyword/cosine similarity matching.
//!
//! # Example
//!
//! ```text
//! Model output:   EXT.CALL @"parse JSON string", r0, r1
//!                          ↓
//! RAG matches:    "parse JSON string" → json_parse → ID 170
//!                          ↓
//! Resolved:       EXT.CALL 170, r0, r1
//! ```
//!
//! # Benefits
//!
//! - Zero training cost for new extensions
//! - Self-documenting code: `@"parse JSON"` vs `170`
//! - Future-proof: new extensions work immediately via RAG
//! - User extensions work without retraining
//!
//! # Single Source of Truth
//!
//! Extension IDs are imported from `runtime::extensions::ext_ids` to ensure
//! consistency between assembly-time resolution and runtime execution.

use crate::runtime::extensions::ext_ids;
use std::collections::HashMap;

/// Information about a resolved extension
#[derive(Debug, Clone)]
pub struct ResolvedExtension {
    /// Extension ID to use in EXT.CALL
    pub id: u32,
    /// Extension name (for error messages)
    pub name: String,
    /// Number of input parameters expected
    pub input_count: usize,
    /// Brief description
    pub description: String,
}

/// RAG-based extension resolver
///
/// Uses simple keyword matching and cosine similarity to resolve
/// intent descriptions to extension IDs.
pub struct RagResolver {
    /// Extension database: description keywords -> extension info
    extensions: Vec<ExtensionEntry>,
    /// Name lookup: exact name -> extension ID
    name_lookup: HashMap<String, u32>,
}

/// Internal entry for an extension
#[derive(Debug, Clone)]
struct ExtensionEntry {
    id: u32,
    name: String,
    description: String,
    keywords: Vec<String>,
    input_count: usize,
}

impl RagResolver {
    /// Create a new resolver with bundled extensions
    pub fn new() -> Self {
        let mut resolver = Self {
            extensions: Vec::new(),
            name_lookup: HashMap::new(),
        };

        // Register bundled extensions (from docs/extensions/bundled.md)
        resolver.register_bundled_extensions();

        resolver
    }

    /// Register all bundled extensions
    /// Uses ext_ids constants as single source of truth
    fn register_bundled_extensions(&mut self) {
        // Crypto extensions (1-99)
        self.register(
            ext_ids::SHA256,
            "sha256",
            "calculate SHA256 hash",
            &["sha256", "hash", "digest", "sha-256"],
            2,
        );
        self.register(
            ext_ids::HMAC_SHA256,
            "hmac_sha256",
            "calculate HMAC-SHA256",
            &["hmac", "hmac-sha256", "message authentication"],
            3,
        );
        self.register(
            ext_ids::AES256_GCM_ENCRYPT,
            "aes256_gcm_encrypt",
            "encrypt with AES-GCM",
            &["aes", "encrypt", "aes-256", "gcm", "encryption"],
            4,
        );
        self.register(
            ext_ids::AES256_GCM_DECRYPT,
            "aes256_gcm_decrypt",
            "decrypt with AES-GCM",
            &["aes", "decrypt", "aes-256", "gcm", "decryption"],
            4,
        );
        self.register(
            ext_ids::CONSTANT_TIME_EQ,
            "constant_time_eq",
            "compare constant time",
            &[
                "constant",
                "time",
                "compare",
                "timing-safe",
                "secure compare",
            ],
            2,
        );
        self.register(
            ext_ids::SECURE_RANDOM,
            "secure_random",
            "generate random bytes",
            &["random", "secure", "bytes", "rng", "cryptographic random"],
            2,
        );
        self.register(
            ext_ids::PBKDF2_SHA256,
            "pbkdf2_sha256",
            "derive key with PBKDF2",
            &["pbkdf2", "key derivation", "password", "derive key"],
            4,
        );
        self.register(
            ext_ids::ED25519_SIGN,
            "ed25519_sign",
            "sign with Ed25519",
            &["ed25519", "sign", "signature", "signing"],
            3,
        );
        self.register(
            ext_ids::ED25519_VERIFY,
            "ed25519_verify",
            "verify Ed25519 signature",
            &["ed25519", "verify", "signature", "verification"],
            4,
        );
        self.register(
            ext_ids::X25519_DERIVE,
            "x25519_derive",
            "derive shared secret",
            &["x25519", "shared secret", "key exchange", "diffie-hellman"],
            3,
        );
        self.register(
            ext_ids::SHA1,
            "sha1",
            "calculate SHA1 hash (legacy)",
            &["sha1", "hash", "legacy"],
            2,
        );

        // Collections (100-119)
        self.register(
            ext_ids::VEC_NEW,
            "vec_new",
            "create new vector",
            &["vec", "vector", "new", "create"],
            0,
        );
        self.register(
            ext_ids::VEC_WITH_CAPACITY,
            "vec_with_capacity",
            "create vector with capacity",
            &["vec", "vector", "capacity", "allocate"],
            1,
        );
        self.register(
            ext_ids::VEC_PUSH,
            "vec_push",
            "push to vector",
            &["vec", "vector", "push", "append", "add"],
            2,
        );
        self.register(
            ext_ids::VEC_POP,
            "vec_pop",
            "pop from vector",
            &["vec", "vector", "pop", "remove last"],
            1,
        );
        self.register(
            ext_ids::VEC_GET,
            "vec_get",
            "get vector element",
            &["vec", "vector", "get", "index", "at"],
            2,
        );
        self.register(
            ext_ids::VEC_SET,
            "vec_set",
            "set vector element",
            &["vec", "vector", "set", "assign"],
            3,
        );
        self.register(
            ext_ids::VEC_LEN,
            "vec_len",
            "get vector length",
            &["vec", "vector", "len", "length", "size"],
            1,
        );
        self.register(
            ext_ids::VEC_FREE,
            "vec_free",
            "free vector",
            &["vec", "vector", "free", "deallocate"],
            1,
        );
        self.register(
            ext_ids::HASHMAP_NEW,
            "hashmap_new",
            "create hashmap",
            &["hashmap", "map", "dict", "new", "create"],
            0,
        );
        self.register(
            ext_ids::HASHMAP_INSERT,
            "hashmap_insert",
            "insert into hashmap",
            &["hashmap", "map", "insert", "put", "set"],
            3,
        );
        self.register(
            ext_ids::HASHMAP_GET,
            "hashmap_get",
            "get from hashmap",
            &["hashmap", "map", "get", "lookup", "find"],
            2,
        );
        self.register(
            ext_ids::HASHMAP_REMOVE,
            "hashmap_remove",
            "remove from hashmap",
            &["hashmap", "map", "remove", "delete"],
            2,
        );
        self.register(
            ext_ids::HASHMAP_FREE,
            "hashmap_free",
            "free hashmap",
            &["hashmap", "map", "free", "deallocate"],
            1,
        );

        // Strings (140-169)
        self.register(
            ext_ids::STRING_NEW,
            "string_new",
            "create new string",
            &["string", "new", "create"],
            0,
        );
        self.register(
            ext_ids::STRING_FROM_BYTES,
            "string_from_bytes",
            "create string from bytes",
            &["string", "from", "bytes", "convert"],
            2,
        );
        self.register(
            ext_ids::STRING_LEN,
            "string_len",
            "get string length",
            &["string", "len", "length", "size"],
            1,
        );
        self.register(
            ext_ids::STRING_CONCAT,
            "string_concat",
            "concatenate strings",
            &["string", "concat", "concatenate", "join", "append"],
            2,
        );
        self.register(
            ext_ids::STRING_FREE,
            "string_free",
            "free string",
            &["string", "free", "deallocate"],
            1,
        );

        // JSON extensions (170-189)
        self.register(
            ext_ids::JSON_PARSE,
            "json_parse",
            "parse JSON string",
            &["json", "parse", "decode", "deserialize", "from json"],
            1,
        );
        self.register(
            ext_ids::JSON_STRINGIFY,
            "json_stringify",
            "convert to JSON string",
            &["json", "stringify", "encode", "serialize", "to json"],
            1,
        );
        self.register(
            ext_ids::JSON_GET,
            "json_get",
            "get JSON field",
            &["json", "get", "field", "property", "key", "access"],
            2,
        );
        self.register(
            ext_ids::JSON_SET,
            "json_set",
            "set JSON field",
            &["json", "set", "field", "property", "update"],
            3,
        );
        self.register(
            ext_ids::JSON_GET_TYPE,
            "json_get_type",
            "get JSON type",
            &["json", "type", "typeof"],
            1,
        );
        self.register(
            ext_ids::JSON_ARRAY_LEN,
            "json_array_len",
            "get JSON array length",
            &["json", "array", "length", "len", "size"],
            1,
        );
        self.register(
            ext_ids::JSON_ARRAY_GET,
            "json_array_get",
            "get JSON array element",
            &["json", "array", "get", "element", "index", "at"],
            2,
        );
        self.register(
            ext_ids::JSON_ARRAY_PUSH,
            "json_array_push",
            "add to JSON array",
            &["json", "array", "push", "append", "add"],
            2,
        );
        self.register(
            ext_ids::JSON_OBJECT_KEYS,
            "json_object_keys",
            "get JSON object keys",
            &["json", "object", "keys", "enumerate"],
            1,
        );
        self.register(
            ext_ids::JSON_FREE,
            "json_free",
            "free JSON handle",
            &["json", "free", "deallocate", "release"],
            1,
        );
        self.register(
            ext_ids::JSON_NEW_OBJECT,
            "json_new_object",
            "create JSON object",
            &["json", "object", "new", "create", "empty object"],
            0,
        );
        self.register(
            ext_ids::JSON_NEW_ARRAY,
            "json_new_array",
            "create JSON array",
            &["json", "array", "new", "create", "empty array"],
            0,
        );

        // HTTP extensions (190-209)
        self.register(
            ext_ids::HTTP_GET,
            "http_get",
            "make HTTP GET request",
            &["http", "get", "fetch", "request", "url"],
            1,
        );
        self.register(
            ext_ids::HTTP_POST,
            "http_post",
            "make HTTP POST request",
            &["http", "post", "request", "send"],
            2,
        );
        self.register(
            ext_ids::HTTP_PUT,
            "http_put",
            "make HTTP PUT request",
            &["http", "put", "request", "update"],
            2,
        );
        self.register(
            ext_ids::HTTP_DELETE,
            "http_delete",
            "make HTTP DELETE request",
            &["http", "delete", "request", "remove"],
            1,
        );
        self.register(
            ext_ids::HTTP_RESPONSE_STATUS,
            "http_response_status",
            "get HTTP status code",
            &["http", "status", "code", "response status"],
            1,
        );
        self.register(
            ext_ids::HTTP_RESPONSE_BODY,
            "http_response_body",
            "get HTTP response body",
            &["http", "body", "response", "content"],
            1,
        );
        self.register(
            ext_ids::HTTP_RESPONSE_FREE,
            "http_free",
            "free HTTP response",
            &["http", "free", "response", "deallocate"],
            1,
        );
        self.register(
            ext_ids::HTTP_GET_WITH_HEADERS,
            "http_get_with_headers",
            "HTTP GET with headers",
            &["http", "get", "headers", "request"],
            2,
        );
        self.register(
            ext_ids::HTTP_POST_WITH_HEADERS,
            "http_post_with_headers",
            "HTTP POST with headers",
            &["http", "post", "headers", "request"],
            3,
        );

        // Compression (400-419)
        self.register(
            ext_ids::ZLIB_COMPRESS,
            "zlib_compress",
            "compress with zlib",
            &["zlib", "compress", "deflate"],
            1,
        );
        self.register(
            ext_ids::ZLIB_DECOMPRESS,
            "zlib_decompress",
            "decompress with zlib",
            &["zlib", "decompress", "inflate"],
            1,
        );
        self.register(
            ext_ids::GZIP_COMPRESS,
            "gzip_compress",
            "compress with gzip",
            &["gzip", "compress"],
            1,
        );
        self.register(
            ext_ids::GZIP_DECOMPRESS,
            "gzip_decompress",
            "decompress with gzip",
            &["gzip", "decompress"],
            1,
        );

        // Encoding (420-439)
        self.register(
            ext_ids::BASE64_ENCODE,
            "base64_encode",
            "encode as base64",
            &["base64", "encode", "to base64"],
            1,
        );
        self.register(
            ext_ids::BASE64_DECODE,
            "base64_decode",
            "decode base64",
            &["base64", "decode", "from base64"],
            1,
        );
        self.register(
            ext_ids::HEX_ENCODE,
            "hex_encode",
            "encode as hex",
            &["hex", "encode", "to hex"],
            1,
        );
        self.register(
            ext_ids::HEX_DECODE,
            "hex_decode",
            "decode hex",
            &["hex", "decode", "from hex"],
            1,
        );
        self.register(
            ext_ids::URL_ENCODE,
            "url_encode",
            "URL encode",
            &["url", "encode", "percent-encode", "urlencode"],
            1,
        );
        self.register(
            ext_ids::URL_DECODE,
            "url_decode",
            "URL decode",
            &["url", "decode", "percent-decode", "urldecode"],
            1,
        );

        // DateTime (440-459)
        self.register(
            ext_ids::DATETIME_NOW,
            "datetime_now",
            "get current time",
            &["datetime", "now", "current", "time", "utc"],
            0,
        );
        self.register(
            ext_ids::DATETIME_PARSE,
            "datetime_parse",
            "parse date string",
            &["datetime", "parse", "date", "string", "strptime"],
            2,
        );
        self.register(
            ext_ids::DATETIME_FORMAT,
            "datetime_format",
            "format date",
            &["datetime", "format", "strftime", "to string"],
            2,
        );
        self.register(
            ext_ids::DATETIME_ADD_DAYS,
            "datetime_add_days",
            "add days to date",
            &["datetime", "add", "days", "date arithmetic"],
            2,
        );
        self.register(
            ext_ids::DATETIME_DIFF,
            "datetime_diff",
            "get time difference",
            &["datetime", "diff", "difference", "seconds", "delta"],
            2,
        );

        // Regex (460-479)
        self.register(
            ext_ids::REGEX_MATCH,
            "regex_match",
            "check regex match",
            &["regex", "match", "test", "is match", "check"],
            2,
        );
        self.register(
            ext_ids::REGEX_FIND,
            "regex_find",
            "find regex match",
            &["regex", "find", "search", "first match"],
            2,
        );
        self.register(
            ext_ids::REGEX_REPLACE,
            "regex_replace",
            "replace with regex",
            &["regex", "replace", "substitute", "sub"],
            3,
        );
        self.register(
            ext_ids::REGEX_SPLIT,
            "regex_split",
            "split by regex",
            &["regex", "split", "tokenize"],
            2,
        );

        // FileSystem (480-499)
        self.register(
            ext_ids::FS_READ,
            "fs_read",
            "read file contents",
            &["file", "read", "fs", "load", "open file"],
            1,
        );
        self.register(
            ext_ids::FS_WRITE,
            "fs_write",
            "write file",
            &["file", "write", "save", "fs", "create file"],
            2,
        );
        self.register(
            ext_ids::FS_EXISTS,
            "fs_exists",
            "check file exists",
            &["file", "exists", "check", "fs", "is file"],
            1,
        );
        self.register(
            ext_ids::FS_DELETE,
            "fs_delete",
            "delete file",
            &["file", "delete", "remove", "rm", "fs"],
            1,
        );
        self.register(
            ext_ids::FS_MKDIR,
            "fs_mkdir",
            "create directory",
            &["mkdir", "create directory", "make dir", "fs"],
            1,
        );
        self.register(
            ext_ids::FS_LIST_DIR,
            "fs_list_dir",
            "list directory",
            &["list", "directory", "ls", "dir", "fs", "readdir"],
            1,
        );

        // TLS (500-519)
        self.register(
            ext_ids::TLS_CONNECT,
            "tls_connect",
            "connect with TLS",
            &["tls", "connect", "ssl", "secure connection"],
            2,
        );
        self.register(
            ext_ids::TLS_SEND,
            "tls_send",
            "send over TLS",
            &["tls", "send", "write", "ssl send"],
            2,
        );
        self.register(
            ext_ids::TLS_RECV,
            "tls_recv",
            "receive over TLS",
            &["tls", "recv", "receive", "read", "ssl recv"],
            2,
        );
        self.register(
            ext_ids::TLS_CLOSE,
            "tls_close",
            "close TLS connection",
            &["tls", "close", "disconnect", "ssl close"],
            1,
        );

        // X509 (520-539)
        self.register(
            ext_ids::X509_CREATE_SELF_SIGNED,
            "x509_create_self_signed",
            "create self-signed certificate",
            &["x509", "self signed", "certificate", "create"],
            2,
        );
        self.register(
            ext_ids::X509_PARSE,
            "x509_parse",
            "parse X509 certificate",
            &["x509", "parse", "certificate"],
            1,
        );
        self.register(
            ext_ids::X509_VERIFY,
            "x509_verify",
            "verify X509 certificate",
            &["x509", "verify", "certificate"],
            2,
        );

        // SQLite (260-279)
        self.register(
            ext_ids::SQLITE_OPEN,
            "sqlite_open",
            "open SQLite database",
            &["sqlite", "open", "database", "db", "connect"],
            1,
        );
        self.register(
            ext_ids::SQLITE_CLOSE,
            "sqlite_close",
            "close SQLite database",
            &["sqlite", "close", "database", "db", "disconnect"],
            1,
        );
        self.register(
            ext_ids::SQLITE_EXEC,
            "sqlite_exec",
            "execute SQLite statement",
            &["sqlite", "exec", "execute", "sql", "run"],
            2,
        );
        self.register(
            ext_ids::SQLITE_QUERY,
            "sqlite_query",
            "query SQLite database",
            &["sqlite", "query", "select", "sql"],
            2,
        );
        self.register(
            ext_ids::SQLITE_PREPARE,
            "sqlite_prepare",
            "prepare SQLite statement",
            &["sqlite", "prepare", "statement", "sql"],
            2,
        );
        self.register(
            ext_ids::SQLITE_BIND_INT,
            "sqlite_bind_int",
            "bind integer parameter",
            &["sqlite", "bind", "int", "parameter"],
            3,
        );
        self.register(
            ext_ids::SQLITE_BIND_TEXT,
            "sqlite_bind_text",
            "bind text parameter",
            &["sqlite", "bind", "text", "string", "parameter"],
            3,
        );
        self.register(
            ext_ids::SQLITE_BIND_BLOB,
            "sqlite_bind_blob",
            "bind blob parameter",
            &["sqlite", "bind", "blob", "binary", "parameter"],
            3,
        );
        self.register(
            ext_ids::SQLITE_STEP,
            "sqlite_step",
            "step SQLite statement",
            &["sqlite", "step", "next", "row"],
            1,
        );
        self.register(
            ext_ids::SQLITE_RESET,
            "sqlite_reset",
            "reset SQLite statement",
            &["sqlite", "reset", "statement"],
            1,
        );
        self.register(
            ext_ids::SQLITE_FINALIZE,
            "sqlite_finalize",
            "finalize SQLite statement",
            &["sqlite", "finalize", "statement", "close"],
            1,
        );
        self.register(
            ext_ids::SQLITE_COLUMN_INT,
            "sqlite_column_int",
            "get integer column",
            &["sqlite", "column", "int", "integer", "get"],
            2,
        );
        self.register(
            ext_ids::SQLITE_COLUMN_TEXT,
            "sqlite_column_text",
            "get text column",
            &["sqlite", "column", "text", "string", "get"],
            2,
        );
        self.register(
            ext_ids::SQLITE_COLUMN_BLOB,
            "sqlite_column_blob",
            "get blob column",
            &["sqlite", "column", "blob", "binary", "get"],
            2,
        );
        self.register(
            ext_ids::SQLITE_LAST_INSERT_ID,
            "sqlite_last_insert_id",
            "get last insert ID",
            &["sqlite", "last", "insert", "id", "rowid"],
            1,
        );
        self.register(
            ext_ids::SQLITE_CHANGES,
            "sqlite_changes",
            "get number of changes",
            &["sqlite", "changes", "affected", "rows"],
            1,
        );

        // UUID (330-339)
        self.register(
            ext_ids::UUID_V4,
            "uuid_v4",
            "generate UUID v4",
            &["uuid", "v4", "random", "generate", "guid"],
            0,
        );
        self.register(
            ext_ids::UUID_V5,
            "uuid_v5",
            "generate UUID v5",
            &["uuid", "v5", "namespace", "name", "deterministic"],
            2,
        );
        self.register(
            ext_ids::UUID_PARSE,
            "uuid_parse",
            "parse UUID string",
            &["uuid", "parse", "string", "from string"],
            1,
        );
        self.register(
            ext_ids::UUID_TO_STRING,
            "uuid_to_string",
            "convert UUID to string",
            &["uuid", "to string", "format", "stringify"],
            1,
        );
        self.register(
            ext_ids::UUID_V7,
            "uuid_v7",
            "generate UUID v7",
            &["uuid", "v7", "timestamp", "time-ordered"],
            0,
        );
    }

    /// Register a single extension
    fn register(
        &mut self,
        id: u32,
        name: &str,
        description: &str,
        keywords: &[&str],
        input_count: usize,
    ) {
        let keywords: Vec<String> = keywords.iter().map(|s| s.to_lowercase()).collect();

        self.extensions.push(ExtensionEntry {
            id,
            name: name.to_string(),
            description: description.to_string(),
            keywords,
            input_count,
        });

        // Add to name lookup (multiple variations)
        self.name_lookup.insert(name.to_lowercase(), id);
        self.name_lookup
            .insert(format!("ext_{}", name.to_lowercase()), id);
        self.name_lookup
            .insert(format!("@{}", name.to_lowercase()), id);
    }

    /// Register a user extension from manifest
    pub fn register_extension(
        &mut self,
        id: u32,
        name: &str,
        description: &str,
        input_count: usize,
    ) {
        // Extract keywords from description
        let keywords: Vec<String> = description
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .map(|s| s.to_string())
            .collect();

        self.extensions.push(ExtensionEntry {
            id,
            name: name.to_string(),
            description: description.to_string(),
            keywords,
            input_count,
        });

        self.name_lookup.insert(name.to_lowercase(), id);
        self.name_lookup
            .insert(format!("ext_{}", name.to_lowercase()), id);
        self.name_lookup
            .insert(format!("@{}", name.to_lowercase()), id);
    }

    /// Resolve an intent description to an extension
    ///
    /// Returns the best matching extension, or None if no match found.
    pub fn resolve(&self, intent: &str) -> Option<ResolvedExtension> {
        let intent_lower = intent.to_lowercase();
        let intent_words: Vec<&str> = intent_lower.split_whitespace().collect();

        // First, try exact name lookup
        if let Some(&id) = self.name_lookup.get(&intent_lower) {
            return self.get_by_id(id);
        }

        // Score each extension by keyword overlap
        let mut best_score = 0.0;
        let mut best_match: Option<&ExtensionEntry> = None;

        for ext in &self.extensions {
            let score = self.compute_similarity(&intent_words, ext);
            if score > best_score {
                best_score = score;
                best_match = Some(ext);
            }
        }

        // Require minimum score threshold
        if best_score >= 0.3 {
            best_match.map(|ext| ResolvedExtension {
                id: ext.id,
                name: ext.name.clone(),
                input_count: ext.input_count,
                description: ext.description.clone(),
            })
        } else {
            None
        }
    }

    /// Get extension by exact ID
    pub fn get_by_id(&self, id: u32) -> Option<ResolvedExtension> {
        self.extensions
            .iter()
            .find(|e| e.id == id)
            .map(|ext| ResolvedExtension {
                id: ext.id,
                name: ext.name.clone(),
                input_count: ext.input_count,
                description: ext.description.clone(),
            })
    }

    /// Get extension by exact name
    pub fn get_by_name(&self, name: &str) -> Option<ResolvedExtension> {
        let name_lower = name.to_lowercase();
        self.name_lookup
            .get(&name_lower)
            .and_then(|&id| self.get_by_id(id))
    }

    /// Compute similarity between intent and extension
    fn compute_similarity(&self, intent_words: &[&str], ext: &ExtensionEntry) -> f64 {
        let mut match_count = 0;
        let mut partial_count = 0;

        // Check description match
        let desc_lower = ext.description.to_lowercase();
        for word in intent_words {
            if desc_lower.contains(word) {
                match_count += 2; // Description match is worth more
            }
        }

        // Check keyword match
        for word in intent_words {
            for keyword in &ext.keywords {
                if keyword == *word {
                    match_count += 1;
                } else if keyword.contains(word) || word.contains(keyword.as_str()) {
                    partial_count += 1;
                }
            }
        }

        // Normalize score
        let total_words = intent_words.len().max(1);
        let score =
            (match_count as f64 * 1.0 + partial_count as f64 * 0.5) / (total_words as f64 * 2.0);

        score.min(1.0)
    }

    /// Get all registered extensions
    pub fn all_extensions(&self) -> Vec<ResolvedExtension> {
        self.extensions
            .iter()
            .map(|ext| ResolvedExtension {
                id: ext.id,
                name: ext.name.clone(),
                input_count: ext.input_count,
                description: ext.description.clone(),
            })
            .collect()
    }

    /// Search for extensions matching a query
    pub fn search(&self, query: &str, limit: usize) -> Vec<ResolvedExtension> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut scored: Vec<(f64, &ExtensionEntry)> = self
            .extensions
            .iter()
            .map(|ext| (self.compute_similarity(&query_words, ext), ext))
            .filter(|(score, _)| *score > 0.1)
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored
            .into_iter()
            .take(limit)
            .map(|(_, ext)| ResolvedExtension {
                id: ext.id,
                name: ext.name.clone(),
                input_count: ext.input_count,
                description: ext.description.clone(),
            })
            .collect()
    }
}

impl Default for RagResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_name_lookup() {
        let resolver = RagResolver::new();

        // Exact name - uses ext_ids::JSON_PARSE as single source of truth
        let ext = resolver.get_by_name("json_parse").unwrap();
        assert_eq!(ext.id, ext_ids::JSON_PARSE);

        // With ext_ prefix
        let ext = resolver.get_by_name("ext_json_parse").unwrap();
        assert_eq!(ext.id, ext_ids::JSON_PARSE);

        // With @ prefix
        let ext = resolver.get_by_name("@json_parse").unwrap();
        assert_eq!(ext.id, ext_ids::JSON_PARSE);
    }

    #[test]
    fn test_intent_resolution() {
        let resolver = RagResolver::new();

        // Natural language intent - json_parse
        let ext = resolver.resolve("parse JSON string").unwrap();
        assert_eq!(ext.id, ext_ids::JSON_PARSE);
        assert_eq!(ext.name, "json_parse");

        // Another natural language intent - http_get
        let ext = resolver.resolve("make HTTP GET request").unwrap();
        assert_eq!(ext.id, ext_ids::HTTP_GET);
        assert_eq!(ext.name, "http_get");

        // Crypto operation
        let ext = resolver.resolve("calculate SHA256 hash").unwrap();
        assert_eq!(ext.id, ext_ids::SHA256);
        assert_eq!(ext.name, "sha256");
    }

    #[test]
    fn test_fuzzy_matching() {
        let resolver = RagResolver::new();

        // Partial match - fs_read
        let ext = resolver.resolve("read a file").unwrap();
        assert_eq!(ext.id, ext_ids::FS_READ);
        assert_eq!(ext.name, "fs_read");

        // HTTP request phrasing - http_get
        let ext = resolver.resolve("make HTTP request to URL").unwrap();
        assert_eq!(ext.id, ext_ids::HTTP_GET);
        assert_eq!(ext.name, "http_get");

        // Crypto - generate random
        let ext = resolver.resolve("generate secure random bytes").unwrap();
        assert_eq!(ext.id, ext_ids::SECURE_RANDOM);
        assert_eq!(ext.name, "secure_random");
    }

    #[test]
    fn test_search() {
        let resolver = RagResolver::new();

        let results = resolver.search("json", 5);
        assert!(!results.is_empty());
        assert!(results.iter().all(|r| r.name.contains("json")));
    }

    #[test]
    fn test_no_match() {
        let resolver = RagResolver::new();

        // Should not match anything
        let ext = resolver.resolve("xyzzy frobnicator");
        assert!(ext.is_none());
    }

    #[test]
    fn test_get_by_id() {
        let resolver = RagResolver::new();

        // Uses ext_ids as single source of truth
        let ext = resolver.get_by_id(ext_ids::JSON_PARSE).unwrap();
        assert_eq!(ext.name, "json_parse");

        let ext = resolver.get_by_id(9999);
        assert!(ext.is_none());
    }
}
