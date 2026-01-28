//! Example Index for Borderline Confidence Queries
//!
//! Provides similarity search over training examples when intent classification
//! confidence is borderline (0.5-0.7). Uses the existing VectorIndex infrastructure.
//!
//! # Architecture
//!
//! ```text
//! STARTUP (cold path):
//! ┌─────────────────────────────────────────────────────────────────┐
//! │ Load example_index.bin (100K examples × 384 dims = ~147MB)      │
//! │ Load example_meta.bin (intent IDs, source offsets)              │
//! └─────────────────────────────────────────────────────────────────┘
//!
//! HOT PATH (only for borderline queries, ~0.1ms):
//! ┌─────────────────────────────────────────────────────────────────┐
//! │ search(query_embedding, k) → top-k similar examples             │
//! │ Returns example metadata for hint generation                    │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! This index is OPTIONAL. It's only used when:
//! 1. Intent classification confidence is borderline (0.5-0.7)
//! 2. Additional context would help disambiguate
//!
//! ```rust,ignore
//! use neurlang::inference::example_index::ExampleIndex;
//!
//! // Load at startup (optional)
//! let examples = ExampleIndex::load("~/.neurlang/example_index.bin")?;
//!
//! // Use when confidence is borderline
//! if confidence > 0.5 && confidence < 0.7 {
//!     let hints = examples.search(&query_embedding, 3);
//!     // Use hints to inform generation
//! }
//! ```

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use super::index::{IndexConfig, IndexError, VectorIndex};

/// Metadata for a training example
#[derive(Debug, Clone)]
pub struct ExampleMeta {
    /// Intent ID this example demonstrates
    pub intent_id: u8,
    /// Prompt text (for context)
    pub prompt: String,
    /// Source file offset (for lazy loading full example)
    pub source_offset: u32,
}

/// Search result from example index
#[derive(Debug, Clone)]
pub struct ExampleSearchResult {
    /// Example metadata
    pub meta: ExampleMeta,
    /// Similarity score (0-1)
    pub score: f32,
}

/// Index file magic number
const EXAMPLE_INDEX_MAGIC: &[u8; 4] = b"NLEX"; // Neurlang Example Index

/// Index file version
const EXAMPLE_INDEX_VERSION: u32 = 1;

/// Example index error types
#[derive(Debug)]
pub enum ExampleIndexError {
    /// I/O error
    Io(std::io::Error),
    /// Invalid format
    InvalidFormat(String),
    /// Vector index error
    VectorIndex(IndexError),
}

impl std::fmt::Display for ExampleIndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExampleIndexError::Io(e) => write!(f, "I/O error: {}", e),
            ExampleIndexError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            ExampleIndexError::VectorIndex(e) => write!(f, "Vector index error: {}", e),
        }
    }
}

impl std::error::Error for ExampleIndexError {}

impl From<std::io::Error> for ExampleIndexError {
    fn from(e: std::io::Error) -> Self {
        ExampleIndexError::Io(e)
    }
}

impl From<IndexError> for ExampleIndexError {
    fn from(e: IndexError) -> Self {
        ExampleIndexError::VectorIndex(e)
    }
}

/// Example index for similarity search over training examples
///
/// Wraps VectorIndex and adds example-specific metadata.
pub struct ExampleIndex {
    /// Underlying vector index for similarity search
    index: VectorIndex,
    /// Metadata for each example (parallel to index chunks)
    metadata: Vec<ExampleMeta>,
}

impl ExampleIndex {
    /// Create a new empty example index
    pub fn new() -> Self {
        Self {
            index: VectorIndex::new(IndexConfig::default()),
            metadata: Vec::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: IndexConfig) -> Self {
        Self {
            index: VectorIndex::new(config),
            metadata: Vec::new(),
        }
    }

    /// Add an example to the index
    ///
    /// The embedding should be pre-computed.
    pub fn add_example(
        &mut self,
        prompt: String,
        intent_id: u8,
        embedding: Vec<f32>,
        source_offset: u32,
    ) {
        // Add to vector index as a pre-embedded chunk
        self.index.add_chunk(
            prompt.clone(),
            0, // doc_index not used
            source_offset as usize,
            embedding,
        );

        // Add metadata
        self.metadata.push(ExampleMeta {
            intent_id,
            prompt,
            source_offset,
        });
    }

    /// Build the index (finalize for searching)
    pub fn build(&mut self) -> Result<(), ExampleIndexError> {
        // All chunks already have embeddings, so build is a no-op
        // Just mark the index as built
        self.index.build().map_err(ExampleIndexError::VectorIndex)
    }

    /// Search for similar examples
    ///
    /// Returns top-k examples with their similarity scores.
    /// Uses the existing VectorIndex::search_embedding() method.
    pub fn search(&mut self, query_embedding: &[f32], k: usize) -> Vec<ExampleSearchResult> {
        let results = self.index.search_embedding(query_embedding, k);

        results
            .into_iter()
            .filter_map(|r| {
                // Find corresponding metadata by matching chunk text
                self.metadata
                    .iter()
                    .find(|m| m.prompt == r.chunk.text)
                    .map(|meta| ExampleSearchResult {
                        meta: meta.clone(),
                        score: r.score,
                    })
            })
            .collect()
    }

    /// Search and aggregate by intent
    ///
    /// Returns a map of intent_id -> (count, avg_score) for the top-k results.
    /// Useful for boosting confidence in borderline cases.
    pub fn search_intent_votes(
        &mut self,
        query_embedding: &[f32],
        k: usize,
    ) -> Vec<(u8, usize, f32)> {
        let results = self.search(query_embedding, k);

        // Group by intent
        let mut intent_scores: std::collections::HashMap<u8, (usize, f32)> =
            std::collections::HashMap::new();

        for r in results {
            let entry = intent_scores.entry(r.meta.intent_id).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += r.score;
        }

        // Convert to sorted vector
        let mut votes: Vec<_> = intent_scores
            .into_iter()
            .map(|(id, (count, total_score))| {
                let avg_score = if count > 0 {
                    total_score / count as f32
                } else {
                    0.0
                };
                (id, count, avg_score)
            })
            .collect();

        // Sort by count (descending), then by score (descending)
        votes.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal))
        });

        votes
    }

    /// Get number of examples in the index
    pub fn len(&self) -> usize {
        self.metadata.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.metadata.is_empty()
    }

    /// Save index to binary files
    ///
    /// Saves both the vector index and metadata.
    pub fn save(
        &self,
        index_path: impl AsRef<Path>,
        meta_path: impl AsRef<Path>,
    ) -> Result<(), ExampleIndexError> {
        // Save vector index
        self.index.save(&index_path)?;

        // Save metadata
        let file = File::create(meta_path)?;
        let mut writer = BufWriter::new(file);

        // Header
        writer.write_all(EXAMPLE_INDEX_MAGIC)?;
        writer.write_all(&EXAMPLE_INDEX_VERSION.to_le_bytes())?;
        writer.write_all(&(self.metadata.len() as u64).to_le_bytes())?;

        // Metadata entries
        for meta in &self.metadata {
            // Intent ID
            writer.write_all(&[meta.intent_id])?;
            // Source offset
            writer.write_all(&meta.source_offset.to_le_bytes())?;
            // Prompt length and content
            let prompt_bytes = meta.prompt.as_bytes();
            writer.write_all(&(prompt_bytes.len() as u32).to_le_bytes())?;
            writer.write_all(prompt_bytes)?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Load index from binary files
    pub fn load(
        index_path: impl AsRef<Path>,
        meta_path: impl AsRef<Path>,
    ) -> Result<Self, ExampleIndexError> {
        // Load vector index
        let index = VectorIndex::load(&index_path)?;

        // Load metadata
        let file = File::open(meta_path)?;
        let mut reader = BufReader::new(file);

        // Header
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != EXAMPLE_INDEX_MAGIC {
            return Err(ExampleIndexError::InvalidFormat(
                "Invalid magic number".into(),
            ));
        }

        let mut buf4 = [0u8; 4];
        let mut buf8 = [0u8; 8];

        reader.read_exact(&mut buf4)?;
        let version = u32::from_le_bytes(buf4);
        if version != EXAMPLE_INDEX_VERSION {
            return Err(ExampleIndexError::InvalidFormat(format!(
                "Unsupported version: {}",
                version
            )));
        }

        reader.read_exact(&mut buf8)?;
        let count = u64::from_le_bytes(buf8) as usize;

        // Read metadata entries
        let mut metadata = Vec::with_capacity(count);
        for _ in 0..count {
            // Intent ID
            let mut intent_buf = [0u8; 1];
            reader.read_exact(&mut intent_buf)?;
            let intent_id = intent_buf[0];

            // Source offset
            reader.read_exact(&mut buf4)?;
            let source_offset = u32::from_le_bytes(buf4);

            // Prompt
            reader.read_exact(&mut buf4)?;
            let prompt_len = u32::from_le_bytes(buf4) as usize;
            let mut prompt_buf = vec![0u8; prompt_len];
            reader.read_exact(&mut prompt_buf)?;
            let prompt = String::from_utf8(prompt_buf)
                .map_err(|_| ExampleIndexError::InvalidFormat("Invalid UTF-8 in prompt".into()))?;

            metadata.push(ExampleMeta {
                intent_id,
                prompt,
                source_offset,
            });
        }

        Ok(Self { index, metadata })
    }
}

impl Default for ExampleIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_index_basic() {
        let mut index = ExampleIndex::new();

        // Add some examples with simple embeddings
        let mut emb1 = vec![0.0f32; 384];
        emb1[0] = 1.0;
        index.add_example("add 5 and 3".to_string(), 0, emb1.clone(), 0);

        let mut emb2 = vec![0.0f32; 384];
        emb2[1] = 1.0;
        index.add_example("subtract 10 from 20".to_string(), 1, emb2.clone(), 100);

        index.build().unwrap();

        assert_eq!(index.len(), 2);

        // Search should find the first example
        let results = index.search(&emb1, 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].meta.intent_id, 0);
    }

    #[test]
    fn test_intent_votes() {
        let mut index = ExampleIndex::new();

        // Add multiple examples for intent 0
        for i in 0..5 {
            let mut emb = vec![0.0f32; 384];
            emb[0] = 1.0;
            emb[i + 1] = 0.1; // Small variation
            index.add_example(format!("add example {}", i), 0, emb, i as u32 * 100);
        }

        // Add fewer examples for intent 1
        for i in 0..2 {
            let mut emb = vec![0.0f32; 384];
            emb[0] = 0.5; // Less similar
            emb[100 + i] = 1.0;
            index.add_example(format!("sub example {}", i), 1, emb, (i + 10) as u32 * 100);
        }

        index.build().unwrap();

        // Query similar to intent 0
        let mut query = vec![0.0f32; 384];
        query[0] = 1.0;

        let votes = index.search_intent_votes(&query, 5);
        assert!(!votes.is_empty());
        // Intent 0 should have more votes since query is more similar
        assert_eq!(votes[0].0, 0, "Intent 0 should have most votes");
    }

    #[test]
    fn test_save_load() {
        let mut index = ExampleIndex::new();

        let mut emb = vec![0.5f32; 384];
        emb[0] = 1.0;
        index.add_example("test prompt".to_string(), 5, emb.clone(), 42);
        index.build().unwrap();

        // Save
        let idx_path = std::env::temp_dir().join("test_example_idx.bin");
        let meta_path = std::env::temp_dir().join("test_example_meta.bin");
        index.save(&idx_path, &meta_path).unwrap();

        // Load
        let loaded = ExampleIndex::load(&idx_path, &meta_path).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded.metadata[0].intent_id, 5);
        assert_eq!(loaded.metadata[0].prompt, "test prompt");
        assert_eq!(loaded.metadata[0].source_offset, 42);

        // Clean up
        let _ = std::fs::remove_file(&idx_path);
        let _ = std::fs::remove_file(&meta_path);
    }
}
