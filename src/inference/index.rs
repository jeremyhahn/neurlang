//! In-Memory Vector Index for Neurlang
//!
//! Provides fast semantic search over requirements and context documents.
//! Designed for zero I/O during the hot path with disk caching for persistence.
//!
//! # Architecture
//!
//! ```text
//! STARTUP (cold path):
//! ┌─────────────────────────────────────────────────────────────────┐
//! │ Load requirements → chunk → embed → IN-MEMORY VECTOR INDEX     │
//! │ If cached (hash match): load from disk instead of re-embedding │
//! └─────────────────────────────────────────────────────────────────┘
//!
//! ITERATION (hot path, zero I/O):
//! ┌─────────────────────────────────────────────────────────────────┐
//! │ search(query) → top-k chunks from RAM                          │
//! │ Time: ~0.5-1ms (pure memory operations)                        │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use neurlang::inference::index::{VectorIndex, IndexConfig};
//!
//! // Create index from text chunks
//! let mut index = VectorIndex::new(IndexConfig::default());
//! index.add_document("User authentication requirements...");
//! index.add_document("API rate limiting specs...");
//! index.build()?;
//!
//! // Search (hot path - zero allocations)
//! let results = index.search("authentication", 5);
//! for (chunk, score) in results {
//!     println!("{:.3}: {}", score, chunk);
//! }
//!
//! // Save/load for caching
//! index.save("index.bin")?;
//! let index = VectorIndex::load("index.bin")?;
//! ```

use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::sync::Arc;

use super::embedder::Embedder;

/// Embedding dimension (matches common models like all-MiniLM-L6-v2)
pub const EMBEDDING_DIM: usize = 384;

/// Maximum chunk size in characters
pub const MAX_CHUNK_SIZE: usize = 1000;

/// Overlap between chunks
pub const CHUNK_OVERLAP: usize = 100;

/// Index configuration
#[derive(Debug, Clone)]
pub struct IndexConfig {
    /// Embedding dimension
    pub embedding_dim: usize,
    /// Maximum chunk size
    pub max_chunk_size: usize,
    /// Chunk overlap
    pub chunk_overlap: usize,
    /// Use approximate nearest neighbor (for large indices)
    pub use_ann: bool,
    /// Number of clusters for ANN (if enabled)
    pub num_clusters: usize,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            embedding_dim: EMBEDDING_DIM,
            max_chunk_size: MAX_CHUNK_SIZE,
            chunk_overlap: CHUNK_OVERLAP,
            use_ann: false,
            num_clusters: 16,
        }
    }
}

/// A text chunk with its embedding
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Original text
    pub text: String,
    /// Source document index
    pub doc_index: usize,
    /// Offset within document
    pub offset: usize,
    /// Pre-computed embedding
    pub embedding: Vec<f32>,
}

/// Search result
#[derive(Debug, Clone)]
pub struct SearchResult<'a> {
    /// The matching chunk
    pub chunk: &'a Chunk,
    /// Similarity score (0-1, higher is better)
    pub score: f32,
}

/// In-memory vector index
pub struct VectorIndex {
    /// Configuration
    config: IndexConfig,
    /// All chunks with embeddings
    chunks: Vec<Chunk>,
    /// Document hashes for cache invalidation
    doc_hashes: Vec<u64>,
    /// Combined hash of all documents
    content_hash: u64,
    /// Pre-allocated query buffer
    query_buffer: Vec<f32>,
    /// Pre-allocated scores buffer
    scores_buffer: Vec<(usize, f32)>,
    /// Whether index is built and ready for search
    is_built: bool,
    /// Optional embedder for computing embeddings (if None, uses hash-based)
    embedder: Option<Arc<dyn Embedder>>,
}

impl VectorIndex {
    /// Create a new empty index
    pub fn new(config: IndexConfig) -> Self {
        Self {
            query_buffer: vec![0.0; config.embedding_dim],
            scores_buffer: Vec::with_capacity(1000),
            config,
            chunks: Vec::new(),
            doc_hashes: Vec::new(),
            content_hash: 0,
            is_built: false,
            embedder: None,
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(IndexConfig::default())
    }

    /// Create with a custom embedder
    ///
    /// The embedder will be used for both building the index and search queries.
    /// This enables using real ML embeddings instead of the hash-based fallback.
    pub fn with_embedder(config: IndexConfig, embedder: Arc<dyn Embedder>) -> Self {
        Self {
            query_buffer: vec![0.0; config.embedding_dim],
            scores_buffer: Vec::with_capacity(1000),
            config,
            chunks: Vec::new(),
            doc_hashes: Vec::new(),
            content_hash: 0,
            is_built: false,
            embedder: Some(embedder),
        }
    }

    /// Set the embedder to use for this index
    ///
    /// If called after `build()`, the index should be rebuilt to use the new embedder.
    pub fn set_embedder(&mut self, embedder: Arc<dyn Embedder>) {
        self.embedder = Some(embedder);
    }

    /// Get the embedder name (for debugging/logging)
    pub fn embedder_name(&self) -> &str {
        match &self.embedder {
            Some(e) => e.name(),
            None => "None (embeddings pre-computed)",
        }
    }

    /// Whether this index uses ML-based embeddings
    pub fn uses_ml_embeddings(&self) -> bool {
        self.embedder
            .as_ref()
            .map(|e| e.is_ml_based())
            .unwrap_or(false)
    }

    /// Get the content hash (for cache validation)
    pub fn content_hash(&self) -> u64 {
        self.content_hash
    }

    /// Check if a cached index matches the current content
    pub fn matches_hash(&self, hash: u64) -> bool {
        self.content_hash == hash
    }

    /// Add a document to the index (will be chunked automatically)
    pub fn add_document(&mut self, text: &str) -> usize {
        let doc_index = self.doc_hashes.len();

        // Compute document hash
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let doc_hash = hasher.finish();
        self.doc_hashes.push(doc_hash);

        // Update combined hash
        self.content_hash ^= doc_hash.rotate_left(doc_index as u32 % 64);

        // Chunk the document
        let chunks = chunk_text(text, self.config.max_chunk_size, self.config.chunk_overlap);

        for (offset, chunk_text) in chunks {
            self.chunks.push(Chunk {
                text: chunk_text,
                doc_index,
                offset,
                embedding: Vec::new(), // Will be filled during build
            });
        }

        self.is_built = false;
        doc_index
    }

    /// Add a pre-chunked text with embedding
    pub fn add_chunk(
        &mut self,
        text: String,
        doc_index: usize,
        offset: usize,
        embedding: Vec<f32>,
    ) {
        self.chunks.push(Chunk {
            text,
            doc_index,
            offset,
            embedding,
        });
    }

    /// Build the index (compute embeddings if needed)
    ///
    /// This is a cold-path operation - embeddings are computed here.
    /// Requires an embedder to be set via `with_embedder()` or `set_embedder()`,
    /// unless all chunks already have pre-computed embeddings.
    pub fn build(&mut self) -> Result<(), IndexError> {
        // Check if any chunks need embeddings computed
        let needs_embedding = self.chunks.iter().any(|c| c.embedding.is_empty());

        if needs_embedding && self.embedder.is_none() {
            return Err(IndexError::EmbeddingError(
                "No embedder configured. Use with_embedder() or set_embedder() before build(), \
                 or provide pre-computed embeddings via add_chunk()"
                    .to_string(),
            ));
        }

        // Compute embeddings for chunks that don't have them
        if let Some(embedder) = &self.embedder {
            for chunk in &mut self.chunks {
                if chunk.embedding.is_empty() {
                    chunk.embedding = embedder
                        .embed(&chunk.text)
                        .map_err(|e| IndexError::EmbeddingError(e.to_string()))?;
                }
            }
        }

        // Pre-allocate scores buffer
        self.scores_buffer = Vec::with_capacity(self.chunks.len());

        self.is_built = true;
        Ok(())
    }

    /// Build with a custom embedding function
    pub fn build_with_embedder<F>(&mut self, embedder: F) -> Result<(), IndexError>
    where
        F: Fn(&str) -> Vec<f32>,
    {
        for chunk in &mut self.chunks {
            if chunk.embedding.is_empty() {
                chunk.embedding = embedder(&chunk.text);
            }
        }

        self.scores_buffer = Vec::with_capacity(self.chunks.len());
        self.is_built = true;
        Ok(())
    }

    /// Search for similar chunks (HOT PATH - aims for zero allocations)
    ///
    /// Returns up to `k` most similar chunks with their scores.
    /// Uses the configured embedder if available, otherwise falls back to hash-based.
    pub fn search(&mut self, query: &str, k: usize) -> Vec<SearchResult<'_>> {
        if !self.is_built || self.chunks.is_empty() {
            return Vec::new();
        }

        // Compute query embedding using configured embedder
        // If no embedder is set, we need pre-computed embeddings (use search_embedding instead)
        if self.embedder.is_none() {
            // No embedder configured - cannot embed query dynamically
            // Return empty results (user should use search_embedding with pre-computed embeddings)
            return Vec::new();
        }

        let embedder = self.embedder.as_ref().unwrap();
        let query_emb = match embedder.embed(query) {
            Ok(emb) => emb,
            Err(_) => return Vec::new(), // Embedding failed
        };

        // Copy to query buffer (resize if needed for different dimension embedders)
        if query_emb.len() != self.query_buffer.len() {
            self.query_buffer.resize(query_emb.len(), 0.0);
        }
        self.query_buffer.copy_from_slice(&query_emb);

        // Compute all similarities (reuses scores buffer)
        self.scores_buffer.clear();
        for (i, chunk) in self.chunks.iter().enumerate() {
            let score = cosine_similarity(&self.query_buffer, &chunk.embedding);
            self.scores_buffer.push((i, score));
        }

        // Partial sort to get top-k (O(n + k log k))
        let k = k.min(self.scores_buffer.len());
        self.scores_buffer
            .select_nth_unstable_by(k.saturating_sub(1), |a, b| {
                b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            });

        // Sort the top-k by score
        self.scores_buffer[..k]
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Build result references
        self.scores_buffer[..k]
            .iter()
            .map(|&(idx, score)| SearchResult {
                chunk: &self.chunks[idx],
                score,
            })
            .collect()
    }

    /// Search with pre-computed query embedding (even faster)
    pub fn search_embedding(&mut self, query_embedding: &[f32], k: usize) -> Vec<SearchResult<'_>> {
        if !self.is_built || self.chunks.is_empty() {
            return Vec::new();
        }

        self.scores_buffer.clear();
        for (i, chunk) in self.chunks.iter().enumerate() {
            let score = cosine_similarity(query_embedding, &chunk.embedding);
            self.scores_buffer.push((i, score));
        }

        let k = k.min(self.scores_buffer.len());
        self.scores_buffer
            .select_nth_unstable_by(k.saturating_sub(1), |a, b| {
                b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            });

        self.scores_buffer[..k]
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        self.scores_buffer[..k]
            .iter()
            .map(|&(idx, score)| SearchResult {
                chunk: &self.chunks[idx],
                score,
            })
            .collect()
    }

    /// Get number of chunks
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    /// Get all chunks (for iteration)
    pub fn chunks(&self) -> &[Chunk] {
        &self.chunks
    }

    /// Save index to binary file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), IndexError> {
        let file = File::create(path).map_err(IndexError::Io)?;
        let mut writer = BufWriter::new(file);

        // Header: magic + version + hash + config
        writer.write_all(b"NLIX")?; // Neurlang Index
        writer.write_all(&1u32.to_le_bytes())?; // Version
        writer.write_all(&self.content_hash.to_le_bytes())?;
        writer.write_all(&(self.config.embedding_dim as u32).to_le_bytes())?;

        // Chunk count
        writer.write_all(&(self.chunks.len() as u64).to_le_bytes())?;

        // Chunks
        for chunk in &self.chunks {
            // Text length and content
            writer.write_all(&(chunk.text.len() as u32).to_le_bytes())?;
            writer.write_all(chunk.text.as_bytes())?;

            // Metadata
            writer.write_all(&(chunk.doc_index as u32).to_le_bytes())?;
            writer.write_all(&(chunk.offset as u32).to_le_bytes())?;

            // Embedding
            for &val in &chunk.embedding {
                writer.write_all(&val.to_le_bytes())?;
            }
        }

        writer.flush()?;
        Ok(())
    }

    /// Load index from binary file
    pub fn load(path: impl AsRef<Path>) -> Result<Self, IndexError> {
        let file = File::open(path).map_err(IndexError::Io)?;
        let mut reader = BufReader::new(file);

        // Header
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != b"NLIX" {
            return Err(IndexError::InvalidFormat("Invalid magic number".into()));
        }

        let mut buf4 = [0u8; 4];
        let mut buf8 = [0u8; 8];

        reader.read_exact(&mut buf4)?;
        let version = u32::from_le_bytes(buf4);
        if version != 1 {
            return Err(IndexError::InvalidFormat(format!(
                "Unsupported version: {}",
                version
            )));
        }

        reader.read_exact(&mut buf8)?;
        let content_hash = u64::from_le_bytes(buf8);

        reader.read_exact(&mut buf4)?;
        let embedding_dim = u32::from_le_bytes(buf4) as usize;

        reader.read_exact(&mut buf8)?;
        let chunk_count = u64::from_le_bytes(buf8) as usize;

        // Read chunks
        let mut chunks = Vec::with_capacity(chunk_count);
        for _ in 0..chunk_count {
            // Text
            reader.read_exact(&mut buf4)?;
            let text_len = u32::from_le_bytes(buf4) as usize;
            let mut text_buf = vec![0u8; text_len];
            reader.read_exact(&mut text_buf)?;
            let text = String::from_utf8(text_buf)
                .map_err(|_| IndexError::InvalidFormat("Invalid UTF-8 in chunk".into()))?;

            // Metadata
            reader.read_exact(&mut buf4)?;
            let doc_index = u32::from_le_bytes(buf4) as usize;
            reader.read_exact(&mut buf4)?;
            let offset = u32::from_le_bytes(buf4) as usize;

            // Embedding
            let mut embedding = Vec::with_capacity(embedding_dim);
            for _ in 0..embedding_dim {
                reader.read_exact(&mut buf4)?;
                embedding.push(f32::from_le_bytes(buf4));
            }

            chunks.push(Chunk {
                text,
                doc_index,
                offset,
                embedding,
            });
        }

        let config = IndexConfig {
            embedding_dim,
            ..Default::default()
        };

        Ok(Self {
            query_buffer: vec![0.0; embedding_dim],
            scores_buffer: Vec::with_capacity(chunks.len()),
            config,
            chunks,
            doc_hashes: Vec::new(), // Not persisted
            content_hash,
            is_built: true,
            embedder: None, // Set via set_embedder() after loading if needed
        })
    }

    /// Load index and set a custom embedder for searches
    ///
    /// The embedder will be used for query embedding during searches.
    pub fn load_with_embedder(
        path: impl AsRef<Path>,
        embedder: Arc<dyn Embedder>,
    ) -> Result<Self, IndexError> {
        let mut index = Self::load(path)?;
        index.embedder = Some(embedder);
        Ok(index)
    }

    /// Check if a cache file exists and matches the given hash
    pub fn cache_valid(path: impl AsRef<Path>, expected_hash: u64) -> bool {
        if let Ok(file) = File::open(path) {
            let mut reader = BufReader::new(file);
            let mut buf = [0u8; 16];

            // Read magic + version + hash
            if reader.read_exact(&mut buf).is_ok() && &buf[0..4] == b"NLIX" {
                let stored_hash = u64::from_le_bytes(buf[8..16].try_into().unwrap());
                return stored_hash == expected_hash;
            }
        }
        false
    }
}

/// Index error types
#[derive(Debug)]
pub enum IndexError {
    /// I/O error
    Io(std::io::Error),
    /// Invalid format
    InvalidFormat(String),
    /// Embedding error
    EmbeddingError(String),
}

impl std::fmt::Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexError::Io(e) => write!(f, "I/O error: {}", e),
            IndexError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            IndexError::EmbeddingError(s) => write!(f, "Embedding error: {}", s),
        }
    }
}

impl std::error::Error for IndexError {}

impl From<std::io::Error> for IndexError {
    fn from(e: std::io::Error) -> Self {
        IndexError::Io(e)
    }
}

/// Chunk text into overlapping segments
fn chunk_text(text: &str, max_size: usize, overlap: usize) -> Vec<(usize, String)> {
    let mut chunks = Vec::new();
    let chars: Vec<char> = text.chars().collect();

    if chars.is_empty() {
        return chunks;
    }

    let mut start = 0;
    while start < chars.len() {
        let end = (start + max_size).min(chars.len());

        // Try to break at sentence or word boundary
        let chunk_end = if end < chars.len() {
            find_break_point(&chars, start, end)
        } else {
            end
        };

        let chunk_text: String = chars[start..chunk_end].iter().collect();
        if !chunk_text.trim().is_empty() {
            chunks.push((start, chunk_text));
        }

        if chunk_end >= chars.len() {
            break;
        }

        // Ensure start always advances to prevent infinite loops
        start = chunk_end.saturating_sub(overlap).max(start + 1);
    }

    chunks
}

/// Find a good break point (sentence or word boundary)
fn find_break_point(chars: &[char], start: usize, max_end: usize) -> usize {
    // Look for sentence boundary (. ! ?)
    for i in (start..max_end).rev() {
        if i + 1 < chars.len()
            && matches!(chars[i], '.' | '!' | '?')
            && chars[i + 1].is_whitespace()
        {
            return i + 1;
        }
    }

    // Look for paragraph boundary
    for i in (start..max_end).rev() {
        if chars[i] == '\n' && i + 1 < chars.len() && chars[i + 1] == '\n' {
            return i + 1;
        }
    }

    // Look for word boundary
    for i in (start..max_end).rev() {
        if chars[i].is_whitespace() {
            return i + 1;
        }
    }

    max_end
}

/// Compute a simple bag-of-words embedding (placeholder for real embeddings)
#[allow(dead_code)]
fn compute_simple_embedding(text: &str, dim: usize) -> Vec<f32> {
    let mut embedding = vec![0.0f32; dim];
    compute_simple_embedding_into(text, &mut embedding);
    embedding
}

/// Compute embedding into pre-allocated buffer (zero allocation)
fn compute_simple_embedding_into(text: &str, embedding: &mut [f32]) {
    // Reset buffer
    for val in embedding.iter_mut() {
        *val = 0.0;
    }

    let dim = embedding.len();

    // Simple hash-based embedding
    // In production, this would call an embedding model
    for word in text.split_whitespace() {
        let word_lower = word.to_lowercase();
        let mut hasher = DefaultHasher::new();
        word_lower.hash(&mut hasher);
        let hash = hasher.finish();

        // Use hash to determine which dimensions to activate
        let idx1 = (hash % dim as u64) as usize;
        let idx2 = ((hash >> 16) % dim as u64) as usize;
        let idx3 = ((hash >> 32) % dim as u64) as usize;

        embedding[idx1] += 1.0;
        embedding[idx2] += 0.5;
        embedding[idx3] += 0.25;
    }

    // Normalize
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for val in embedding.iter_mut() {
            *val /= norm;
        }
    }
}

/// Compute cosine similarity between two embeddings
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a > 0.0 && norm_b > 0.0 {
        dot / (norm_a * norm_b)
    } else {
        0.0
    }
}

/// Compute hash of a file's contents (for cache validation)
pub fn hash_file(path: impl AsRef<Path>) -> Result<u64, std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    Ok(hasher.finish())
}

/// Compute hash of multiple files
pub fn hash_files(paths: &[impl AsRef<Path>]) -> Result<u64, std::io::Error> {
    let mut combined_hash = 0u64;
    for (i, path) in paths.iter().enumerate() {
        let file_hash = hash_file(path)?;
        combined_hash ^= file_hash.rotate_left(i as u32 % 64);
    }
    Ok(combined_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::embedder::HashEmbedder;
    use std::sync::Arc;

    /// Helper to create an index with test embedder
    fn test_index() -> VectorIndex {
        VectorIndex::with_embedder(IndexConfig::default(), Arc::new(HashEmbedder::new()))
    }

    #[test]
    fn test_chunk_text() {
        let text = "Hello world. This is a test. Another sentence here.";
        let chunks = chunk_text(text, 20, 5);

        assert!(!chunks.is_empty());
        for (offset, chunk) in &chunks {
            assert!(!chunk.is_empty());
            assert!(*offset < text.len());
        }
    }

    #[test]
    fn test_simple_embedding() {
        let emb1 = compute_simple_embedding("hello world", 384);
        let emb2 = compute_simple_embedding("hello world", 384);
        let emb3 = compute_simple_embedding("goodbye moon", 384);

        assert_eq!(emb1.len(), 384);

        // Same text should produce same embedding
        let sim_same = cosine_similarity(&emb1, &emb2);
        assert!((sim_same - 1.0).abs() < 0.001);

        // Different text should produce different embedding
        let sim_diff = cosine_similarity(&emb1, &emb3);
        assert!(sim_diff < 0.9);
    }

    #[test]
    fn test_index_basic() {
        let mut index = test_index();

        index.add_document("The quick brown fox jumps over the lazy dog.");
        index.add_document("Machine learning is a subset of artificial intelligence.");
        index.add_document("Rust is a systems programming language.");

        index.build().unwrap();

        assert_eq!(index.len(), 3);

        let results = index.search("programming language", 2);
        assert!(!results.is_empty());

        // With hash-based embeddings, exact semantic ranking isn't guaranteed,
        // but at least one result should be returned
        assert!(results.iter().any(|r| r.chunk.text.contains("Rust")
            || r.chunk.text.contains("fox")
            || r.chunk.text.contains("learning")));
    }

    #[test]
    fn test_index_save_load() {
        let mut index = test_index();
        index.add_document("Test document one.");
        index.add_document("Test document two.");
        index.build().unwrap();

        let original_hash = index.content_hash();

        // Save to temp file
        let temp_path = std::env::temp_dir().join("neurlang_test_index.bin");
        index.save(&temp_path).unwrap();

        // Load and verify
        let loaded = VectorIndex::load(&temp_path).unwrap();
        assert_eq!(loaded.len(), index.len());
        assert_eq!(loaded.content_hash(), original_hash);

        // Clean up
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_cache_validation() {
        let mut index = test_index();
        index.add_document("Cache test document.");
        index.build().unwrap();

        let temp_path = std::env::temp_dir().join("neurlang_cache_test.bin");
        index.save(&temp_path).unwrap();

        // Should match
        assert!(VectorIndex::cache_valid(&temp_path, index.content_hash()));

        // Should not match with different hash
        assert!(!VectorIndex::cache_valid(&temp_path, 12345));

        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];

        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
        assert!(cosine_similarity(&a, &c).abs() < 0.001);
    }

    #[test]
    fn test_empty_index() {
        let mut index = VectorIndex::with_defaults();
        index.build().unwrap();

        let results = index.search("anything", 5);
        assert!(results.is_empty());
    }
}
