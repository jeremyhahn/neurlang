//! In-Memory Intent Index for Fast Classification
//!
//! Provides ultra-fast intent classification using pre-computed embeddings.
//! Designed for the hot path with zero disk I/O and minimal allocations.
//!
//! # Architecture
//!
//! ```text
//! STARTUP (cold path):
//! ┌─────────────────────────────────────────────────────────────────┐
//! │ Load intent_index.bin (54 × 384 × 4 = 82KB)                     │
//! │ All embeddings loaded into RAM                                   │
//! └─────────────────────────────────────────────────────────────────┘
//!
//! HOT PATH (per query, ~0.02ms):
//! ┌─────────────────────────────────────────────────────────────────┐
//! │ classify(query_embedding) → (intent_id, confidence)             │
//! │ 54 dot products with SIMD optimization                          │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Performance
//!
//! - 54 intents × 384 dimensions = ~82KB memory
//! - Classification: ~0.01-0.02ms (54 dot products)
//! - No external dependencies (pure Rust SIMD)
//!
//! # Usage
//!
//! ```rust,ignore
//! use neurlang::inference::intent_index::IntentIndex;
//!
//! // Load pre-built index
//! let index = IntentIndex::load("~/.neurlang/intent_index.bin")?;
//!
//! // Classify query embedding (from FastEmbedder)
//! let (intent_id, confidence) = index.classify(&query_embedding);
//!
//! if confidence > 0.7 {
//!     // High confidence - use direct generation
//! }
//! ```

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use super::lookup::INTENT_NAMES;

/// Embedding dimension (matches all-MiniLM-L6-v2)
pub const INTENT_EMBEDDING_DIM: usize = 384;

/// Number of supported intents
pub const NUM_INTENTS: usize = 54;

/// Index file magic number
const INDEX_MAGIC: &[u8; 4] = b"NLII"; // Neurlang Intent Index

/// Index file version
const INDEX_VERSION: u32 = 1;

/// Error types for intent index operations
#[derive(Debug)]
pub enum IntentIndexError {
    /// I/O error
    Io(std::io::Error),
    /// Invalid file format
    InvalidFormat(String),
    /// Dimension mismatch
    DimensionMismatch { expected: usize, actual: usize },
}

impl std::fmt::Display for IntentIndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntentIndexError::Io(e) => write!(f, "I/O error: {}", e),
            IntentIndexError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            IntentIndexError::DimensionMismatch { expected, actual } => {
                write!(
                    f,
                    "Dimension mismatch: expected {}, got {}",
                    expected, actual
                )
            }
        }
    }
}

impl std::error::Error for IntentIndexError {}

impl From<std::io::Error> for IntentIndexError {
    fn from(e: std::io::Error) -> Self {
        IntentIndexError::Io(e)
    }
}

/// Pre-computed intent embeddings for fast classification
///
/// Stores embeddings for all 54 intents in a contiguous memory layout
/// optimized for cache-friendly sequential access during classification.
pub struct IntentIndex {
    /// Embeddings matrix: [NUM_INTENTS][INTENT_EMBEDDING_DIM]
    /// Stored as flat array for cache efficiency
    embeddings: Vec<f32>,
    /// Embedding dimension
    dim: usize,
    /// Number of intents
    num_intents: usize,
}

impl IntentIndex {
    /// Create an empty index (for building)
    pub fn new() -> Self {
        Self {
            embeddings: vec![0.0; NUM_INTENTS * INTENT_EMBEDDING_DIM],
            dim: INTENT_EMBEDDING_DIM,
            num_intents: NUM_INTENTS,
        }
    }

    /// Create index with custom dimensions
    pub fn with_dimensions(num_intents: usize, dim: usize) -> Self {
        Self {
            embeddings: vec![0.0; num_intents * dim],
            dim,
            num_intents,
        }
    }

    /// Set embedding for a specific intent
    ///
    /// The embedding will be L2-normalized before storage.
    pub fn set_embedding(
        &mut self,
        intent_id: usize,
        embedding: &[f32],
    ) -> Result<(), IntentIndexError> {
        if intent_id >= self.num_intents {
            return Err(IntentIndexError::InvalidFormat(format!(
                "Intent ID {} out of range (max {})",
                intent_id,
                self.num_intents - 1
            )));
        }
        if embedding.len() != self.dim {
            return Err(IntentIndexError::DimensionMismatch {
                expected: self.dim,
                actual: embedding.len(),
            });
        }

        // Normalize and store
        let norm = l2_norm(embedding);
        let offset = intent_id * self.dim;

        if norm > 1e-8 {
            for (i, &val) in embedding.iter().enumerate() {
                self.embeddings[offset + i] = val / norm;
            }
        } else {
            for i in 0..self.dim {
                self.embeddings[offset + i] = embedding[i];
            }
        }

        Ok(())
    }

    /// Get embedding for a specific intent (for debugging/inspection)
    pub fn get_embedding(&self, intent_id: usize) -> Option<&[f32]> {
        if intent_id >= self.num_intents {
            return None;
        }
        let offset = intent_id * self.dim;
        Some(&self.embeddings[offset..offset + self.dim])
    }

    /// Classify a query embedding to find the best matching intent
    ///
    /// Returns (intent_id, confidence) where confidence is the cosine similarity (0-1).
    ///
    /// This is the HOT PATH - optimized for minimal latency:
    /// - Sequential memory access for cache efficiency
    /// - No allocations
    /// - SIMD-friendly operations
    #[inline]
    pub fn classify(&self, query_embedding: &[f32]) -> (usize, f32) {
        debug_assert_eq!(
            query_embedding.len(),
            self.dim,
            "Query embedding dimension mismatch"
        );

        let mut best_id = 0;
        let mut best_score = f32::NEG_INFINITY;

        // Sequential scan through all intents
        // For 54 intents, brute force is faster than any index structure
        for intent_id in 0..self.num_intents {
            let offset = intent_id * self.dim;
            let intent_emb = &self.embeddings[offset..offset + self.dim];

            // Dot product (embeddings are pre-normalized, so this equals cosine similarity)
            let score = dot_product(query_embedding, intent_emb);

            if score > best_score {
                best_score = score;
                best_id = intent_id;
            }
        }

        // Clamp confidence to [0, 1] range
        let confidence = best_score.clamp(0.0, 1.0);
        (best_id, confidence)
    }

    /// Classify and return top-k matches
    ///
    /// Returns vector of (intent_id, confidence) pairs sorted by confidence descending.
    pub fn classify_topk(&self, query_embedding: &[f32], k: usize) -> Vec<(usize, f32)> {
        let mut scores: Vec<(usize, f32)> = (0..self.num_intents)
            .map(|id| {
                let offset = id * self.dim;
                let intent_emb = &self.embeddings[offset..offset + self.dim];
                let score = dot_product(query_embedding, intent_emb);
                (id, score.clamp(0.0, 1.0))
            })
            .collect();

        // Partial sort to get top-k
        let k = k.min(scores.len());
        scores.select_nth_unstable_by(k.saturating_sub(1), |a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });

        scores[..k].sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scores[..k].to_vec()
    }

    /// Get the intent name for an ID
    pub fn intent_name(&self, intent_id: usize) -> Option<&'static str> {
        INTENT_NAMES.get(intent_id).copied()
    }

    /// Get embedding dimension
    #[inline]
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Get number of intents
    #[inline]
    pub fn num_intents(&self) -> usize {
        self.num_intents
    }

    /// Save index to binary file
    ///
    /// File format:
    /// - 4 bytes: magic ("NLII")
    /// - 4 bytes: version (u32 LE)
    /// - 4 bytes: num_intents (u32 LE)
    /// - 4 bytes: dim (u32 LE)
    /// - num_intents * dim * 4 bytes: embeddings (f32 LE)
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), IntentIndexError> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Header
        writer.write_all(INDEX_MAGIC)?;
        writer.write_all(&INDEX_VERSION.to_le_bytes())?;
        writer.write_all(&(self.num_intents as u32).to_le_bytes())?;
        writer.write_all(&(self.dim as u32).to_le_bytes())?;

        // Embeddings
        for &val in &self.embeddings {
            writer.write_all(&val.to_le_bytes())?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Load index from binary file
    pub fn load(path: impl AsRef<Path>) -> Result<Self, IntentIndexError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // Read header
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != INDEX_MAGIC {
            return Err(IntentIndexError::InvalidFormat(
                "Invalid magic number".into(),
            ));
        }

        let mut buf4 = [0u8; 4];

        reader.read_exact(&mut buf4)?;
        let version = u32::from_le_bytes(buf4);
        if version != INDEX_VERSION {
            return Err(IntentIndexError::InvalidFormat(format!(
                "Unsupported version: {}",
                version
            )));
        }

        reader.read_exact(&mut buf4)?;
        let num_intents = u32::from_le_bytes(buf4) as usize;

        reader.read_exact(&mut buf4)?;
        let dim = u32::from_le_bytes(buf4) as usize;

        // Read embeddings
        let total_floats = num_intents * dim;
        let mut embeddings = vec![0.0f32; total_floats];

        for val in &mut embeddings {
            reader.read_exact(&mut buf4)?;
            *val = f32::from_le_bytes(buf4);
        }

        Ok(Self {
            embeddings,
            dim,
            num_intents,
        })
    }

    /// Build index from intent descriptions using an embedder
    ///
    /// Uses canonical descriptions for each intent.
    pub fn build_from_descriptions<E: super::embedder::Embedder + ?Sized>(
        embedder: &E,
        descriptions: &[&str],
    ) -> Result<Self, IntentIndexError> {
        let dim = embedder.embedding_dim();
        let num_intents = descriptions.len();

        let mut index = Self::with_dimensions(num_intents, dim);

        for (id, desc) in descriptions.iter().enumerate() {
            let embedding = embedder.embed(desc).map_err(|e| {
                IntentIndexError::InvalidFormat(format!("Failed to embed intent {}: {}", id, e))
            })?;
            index.set_embedding(id, &embedding)?;
        }

        Ok(index)
    }
}

impl Default for IntentIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Canonical intent descriptions for building the index
///
/// These are the "representative" descriptions for each intent.
/// They should be clear, concise, and distinct from other intents.
pub const INTENT_DESCRIPTIONS: [&str; NUM_INTENTS] = [
    // Arithmetic (0-10)
    "add two numbers together",         // 0: ADD
    "subtract one number from another", // 1: SUB
    "multiply two numbers",             // 2: MUL
    "divide one number by another",     // 3: DIV
    "calculate modulo remainder",       // 4: MOD
    "bitwise AND operation",            // 5: AND
    "bitwise OR operation",             // 6: OR
    "bitwise XOR operation",            // 7: XOR
    "shift bits left",                  // 8: SHL
    "shift bits right",                 // 9: SHR
    "arithmetic shift right",           // 10: SAR
    // Math Functions (11-18)
    "compute factorial of a number",   // 11: FACTORIAL
    "calculate fibonacci sequence",    // 12: FIBONACCI
    "raise to a power exponent",       // 13: POWER
    "calculate square root",           // 14: SQRT
    "find greatest common divisor",    // 15: GCD
    "find least common multiple",      // 16: LCM
    "get absolute value",              // 17: ABS
    "clamp value between min and max", // 18: CLAMP
    // Comparisons (19-24)
    "find maximum of two values",  // 19: MAX
    "find minimum of two values",  // 20: MIN
    "get sign of a number",        // 21: SIGN
    "check if number is positive", // 22: IS_POSITIVE
    "check if number is even",     // 23: IS_EVEN
    "check if number is prime",    // 24: IS_PRIME
    // Bit Operations (25-29)
    "count set bits population count", // 25: POPCOUNT
    "count leading zeros",             // 26: CLZ
    "count trailing zeros",            // 27: CTZ
    "swap byte order endianness",      // 28: BSWAP
    "next power of two",               // 29: NEXTPOW2
    // Memory (30-33)
    "copy memory block",     // 30: MEMCPY
    "set memory to value",   // 31: MEMSET
    "compare memory blocks", // 32: MEMCMP
    "sum array elements",    // 33: ARRAY_SUM
    // Strings (34-37)
    "get string length",   // 34: STRLEN
    "compare two strings", // 35: STRCMP
    "copy string",         // 36: STRCPY
    "hash a string",       // 37: HASH_STRING
    // I/O (38-42)
    "print output value",     // 38: PRINT
    "read line input",        // 39: READ_LINE
    "get current timestamp",  // 40: TIME_NOW
    "sleep wait delay",       // 41: SLEEP
    "generate random number", // 42: RANDOM
    // Crypto (43-47)
    "compute SHA256 hash",          // 43: SHA256
    "encrypt with AES",             // 44: AES_ENCRYPT
    "decrypt with AES",             // 45: AES_DECRYPT
    "compute HMAC authentication",  // 46: HMAC
    "generate secure random bytes", // 47: SECURE_RANDOM
    // Loops (48-50)
    "count loop iterations", // 48: LOOP_COUNT
    "sum numbers in loop",   // 49: LOOP_SUM
    "countdown from number", // 50: COUNTDOWN
    // Floating Point (51-53)
    "add floating point numbers",      // 51: FADD
    "multiply floating point numbers", // 52: FMUL
    "divide floating point numbers",   // 53: FDIV
];

// ============================================================================
// SIMD-Optimized Math Functions
// ============================================================================

/// Compute dot product of two vectors
///
/// For 384-dim vectors, this takes ~0.01ms on modern CPUs.
/// Uses auto-vectorization hints for SIMD optimization.
#[inline]
fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());

    // Sum using iterator - compiler auto-vectorizes this
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Compute L2 norm of a vector
#[inline]
fn l2_norm(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_index_create_and_classify() {
        let mut index = IntentIndex::new();

        // Set up some test embeddings (unit vectors along different dimensions)
        let mut emb0 = vec![0.0f32; INTENT_EMBEDDING_DIM];
        emb0[0] = 1.0; // Intent 0: unit vector along dim 0
        index.set_embedding(0, &emb0).unwrap();

        let mut emb1 = vec![0.0f32; INTENT_EMBEDDING_DIM];
        emb1[1] = 1.0; // Intent 1: unit vector along dim 1
        index.set_embedding(1, &emb1).unwrap();

        // Query with vector along dim 0 should match intent 0
        let query0 = emb0.clone();
        let (id, conf) = index.classify(&query0);
        assert_eq!(id, 0);
        assert!(
            (conf - 1.0).abs() < 0.001,
            "Expected ~1.0 confidence, got {}",
            conf
        );

        // Query with vector along dim 1 should match intent 1
        let query1 = emb1.clone();
        let (id, conf) = index.classify(&query1);
        assert_eq!(id, 1);
        assert!(
            (conf - 1.0).abs() < 0.001,
            "Expected ~1.0 confidence, got {}",
            conf
        );
    }

    #[test]
    fn test_intent_index_save_load() {
        let mut index = IntentIndex::new();

        // Set some embeddings
        let mut emb = vec![0.1f32; INTENT_EMBEDDING_DIM];
        for i in 0..5 {
            emb[i] = (i + 1) as f32;
            index.set_embedding(i, &emb).unwrap();
        }

        // Save to temp file
        let temp_path = std::env::temp_dir().join("test_intent_index.bin");
        index.save(&temp_path).unwrap();

        // Load and verify
        let loaded = IntentIndex::load(&temp_path).unwrap();
        assert_eq!(loaded.num_intents(), index.num_intents());
        assert_eq!(loaded.dim(), index.dim());

        // Verify classification gives same results
        let query = vec![1.0f32; INTENT_EMBEDDING_DIM];
        let (orig_id, orig_conf) = index.classify(&query);
        let (loaded_id, loaded_conf) = loaded.classify(&query);
        assert_eq!(orig_id, loaded_id);
        assert!((orig_conf - loaded_conf).abs() < 0.001);

        // Clean up
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_classify_topk() {
        let mut index = IntentIndex::new();

        // Set up embeddings with decreasing similarity to a test query
        for i in 0..10 {
            let mut emb = vec![0.0f32; INTENT_EMBEDDING_DIM];
            emb[0] = 1.0 - (i as f32 * 0.1); // Decreasing similarity
            emb[i + 1] = 0.5; // Some noise
            index.set_embedding(i, &emb).unwrap();
        }

        // Query
        let mut query = vec![0.0f32; INTENT_EMBEDDING_DIM];
        query[0] = 1.0;

        let topk = index.classify_topk(&query, 3);
        assert_eq!(topk.len(), 3);

        // First result should have highest confidence
        assert!(topk[0].1 >= topk[1].1);
        assert!(topk[1].1 >= topk[2].1);
    }

    #[test]
    fn test_dot_product() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let result = dot_product(&a, &b);
        assert!((result - 32.0).abs() < 0.001); // 1*4 + 2*5 + 3*6 = 32
    }

    #[test]
    fn test_l2_norm() {
        let v = vec![3.0, 4.0];
        let norm = l2_norm(&v);
        assert!((norm - 5.0).abs() < 0.001); // sqrt(9 + 16) = 5
    }

    #[test]
    fn test_intent_descriptions_count() {
        assert_eq!(INTENT_DESCRIPTIONS.len(), NUM_INTENTS);
    }

    #[test]
    fn test_classification_latency() {
        let index = IntentIndex::new();
        let query = vec![0.5f32; INTENT_EMBEDDING_DIM];

        // Warm up
        let _ = index.classify(&query);

        // Measure
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = index.classify(&query);
        }
        let elapsed = start.elapsed();

        // Average should be under 100μs in release mode
        // In debug mode, allow up to 1000μs (1ms) due to no optimizations
        let avg_us = elapsed.as_micros() / 1000;
        let threshold = if cfg!(debug_assertions) { 1000 } else { 100 };
        assert!(
            avg_us < threshold,
            "Classification too slow: {}μs average (threshold: {}μs)",
            avg_us,
            threshold
        );
    }
}
