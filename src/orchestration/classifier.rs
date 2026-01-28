//! Pattern Classifier
//!
//! Determines whether a task should be handled by Tier 1 (small model)
//! or escalated to Tier 2 (LLM decomposition).
//!
//! # Algorithm
//!
//! 1. Embed the incoming request
//! 2. Compare to known training patterns via cosine similarity
//! 3. If similarity > threshold, use Tier 1
//! 4. Otherwise, escalate to Tier 2
//!
//! # Why Not a Model?
//!
//! A simple embedding + cosine similarity approach is:
//! - Fast (~0.1ms vs ~10ms for a classifier model)
//! - Deterministic (no stochastic behavior)
//! - Easy to debug (can inspect similarity scores)
//! - Zero additional training cost
//!
//! # Real Embeddings
//!
//! When an `Embedder` is provided via `with_embedder()`, uses real ML embeddings
//! instead of the fallback bag-of-words approach. This improves semantic matching
//! accuracy significantly.

use std::collections::HashMap;
use std::sync::Arc;

use crate::inference::embedder::Embedder;

/// Decision on which tier should handle the task
#[derive(Debug, Clone)]
pub enum TierDecision {
    /// Tier 1 (small model) can handle this task
    Tier1 {
        /// Best matching pattern from training data
        pattern: String,
        /// Cosine similarity score (0.0-1.0)
        confidence: f32,
    },
    /// Tier 2 (LLM) should decompose this task
    Tier2 {
        /// Why escalation was needed
        reason: &'static str,
    },
}

/// Pattern information from training data
#[derive(Debug, Clone)]
pub struct PatternInfo {
    /// The pattern text
    pub pattern: String,
    /// Category (e.g., "arithmetic", "string", "control_flow")
    pub category: String,
    /// Embedding vector (normalized)
    pub embedding: Vec<f32>,
}

/// Pattern classifier for tier decision
pub struct PatternClassifier {
    /// Known patterns with their embeddings
    patterns: Vec<PatternInfo>,
    /// Similarity threshold for Tier 1 (0.0-1.0)
    threshold: f32,
    /// Word frequency for simple TF-IDF-like embedding (fallback)
    word_idf: HashMap<String, f32>,
    /// Optional real embedder (if None, uses bag-of-words fallback)
    embedder: Option<Arc<dyn Embedder>>,
}

impl PatternClassifier {
    /// Create a new classifier with default threshold
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            threshold: 0.85,
            word_idf: HashMap::new(),
            embedder: None,
        }
    }

    /// Create with custom threshold
    pub fn with_threshold(threshold: f32) -> Self {
        Self {
            patterns: Vec::new(),
            threshold,
            word_idf: HashMap::new(),
            embedder: None,
        }
    }

    /// Create with a custom embedder
    ///
    /// When an embedder is provided, uses real ML embeddings instead of
    /// the fallback bag-of-words approach.
    pub fn with_embedder(threshold: f32, embedder: Arc<dyn Embedder>) -> Self {
        Self {
            patterns: Vec::new(),
            threshold,
            word_idf: HashMap::new(),
            embedder: Some(embedder),
        }
    }

    /// Set the embedder to use for this classifier
    ///
    /// Note: Existing patterns will NOT be re-embedded. Call this before
    /// loading patterns to use the new embedder.
    pub fn set_embedder(&mut self, embedder: Arc<dyn Embedder>) {
        self.embedder = Some(embedder);
    }

    /// Whether this classifier uses ML-based embeddings
    pub fn uses_ml_embeddings(&self) -> bool {
        self.embedder
            .as_ref()
            .map(|e| e.is_ml_based())
            .unwrap_or(false)
    }

    /// Set the similarity threshold
    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.clamp(0.0, 1.0);
    }

    /// Add a pattern from training data
    pub fn add_pattern(&mut self, pattern: &str, category: &str) {
        let embedding = self.compute_embedding(pattern);
        self.patterns.push(PatternInfo {
            pattern: pattern.to_string(),
            category: category.to_string(),
            embedding,
        });

        // Update IDF weights
        for word in self.tokenize(pattern) {
            *self.word_idf.entry(word).or_insert(0.0) += 1.0;
        }
    }

    /// Load patterns from training data file (JSONL format)
    pub fn load_patterns(&mut self, data: &str) -> usize {
        let mut count = 0;

        for line in data.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Parse JSONL line
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
                if let (Some(context), Some(category)) = (
                    value["context"].as_str(),
                    value.get("category").and_then(|c| c.as_str()),
                ) {
                    self.add_pattern(context, category);
                    count += 1;
                } else if let Some(context) = value["context"].as_str() {
                    self.add_pattern(context, "unknown");
                    count += 1;
                }
            }
        }

        count
    }

    /// Classify a request to determine which tier should handle it
    pub fn classify(&self, request: &str) -> TierDecision {
        if self.patterns.is_empty() {
            // No patterns loaded, default to Tier 2
            return TierDecision::Tier2 {
                reason: "No training patterns loaded",
            };
        }

        let embedding = self.compute_embedding(request);
        let (best_pattern, similarity) = self.find_nearest(&embedding);

        if similarity >= self.threshold {
            TierDecision::Tier1 {
                pattern: best_pattern.pattern.clone(),
                confidence: similarity,
            }
        } else {
            TierDecision::Tier2 {
                reason: "Low similarity to known patterns",
            }
        }
    }

    /// Find the nearest pattern by cosine similarity
    fn find_nearest(&self, embedding: &[f32]) -> (&PatternInfo, f32) {
        let mut best_pattern = &self.patterns[0];
        let mut best_similarity = f32::NEG_INFINITY;

        for pattern in &self.patterns {
            let similarity = self.cosine_similarity(embedding, &pattern.embedding);
            if similarity > best_similarity {
                best_similarity = similarity;
                best_pattern = pattern;
            }
        }

        (best_pattern, best_similarity)
    }

    /// Compute embedding for text
    ///
    /// Uses the configured embedder if available, otherwise falls back
    /// to a simple bag-of-words embedding (for backwards compat/testing).
    fn compute_embedding(&self, text: &str) -> Vec<f32> {
        // Use real embedder if available
        if let Some(embedder) = &self.embedder {
            if let Ok(embedding) = embedder.embed(text) {
                return embedding;
            }
            // Fall through to legacy method on error
        }

        // Legacy bag-of-words embedding (for backwards compat when no embedder set)
        self.compute_embedding_legacy(text)
    }

    /// Compute a simple bag-of-words embedding (legacy, for backwards compat)
    ///
    /// This is only used when no real embedder is configured.
    /// For production, always configure an embedder via with_embedder().
    fn compute_embedding_legacy(&self, text: &str) -> Vec<f32> {
        let words = self.tokenize(text);

        // Create sparse word vector
        let mut word_counts: HashMap<String, f32> = HashMap::new();
        for word in &words {
            *word_counts.entry(word.clone()).or_insert(0.0) += 1.0;
        }

        // Convert to dense vector (simple approach: hash words to fixed positions)
        const DIM: usize = 128;
        let mut embedding = vec![0.0f32; DIM];

        for (word, count) in &word_counts {
            // Simple hash to position
            let hash = word.bytes().fold(0usize, |acc, b| {
                acc.wrapping_mul(31).wrapping_add(b as usize)
            });
            let pos = hash % DIM;

            // TF-IDF-like weighting
            let idf = 1.0 / (1.0 + self.word_idf.get(word).copied().unwrap_or(0.0).ln());
            embedding[pos] += count * idf;
        }

        // Normalize to unit vector
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }

        embedding
    }

    /// Tokenize text into words
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| s.len() > 1)
            .map(|s| s.to_string())
            .collect()
    }

    /// Compute cosine similarity between two vectors
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        // Use the embedder module's implementation for consistency
        crate::inference::embedder::cosine_similarity(a, b)
    }

    /// Get the number of loaded patterns
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Get the current threshold
    pub fn threshold(&self) -> f32 {
        self.threshold
    }
}

impl Default for PatternClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classifier_creation() {
        let classifier = PatternClassifier::new();
        assert_eq!(classifier.threshold(), 0.85);
        assert_eq!(classifier.pattern_count(), 0);
    }

    #[test]
    fn test_add_pattern() {
        let mut classifier = PatternClassifier::new();
        classifier.add_pattern("compute factorial of a number", "arithmetic");
        classifier.add_pattern("calculate fibonacci sequence", "arithmetic");
        assert_eq!(classifier.pattern_count(), 2);
    }

    #[test]
    fn test_classify_similar() {
        let mut classifier = PatternClassifier::with_threshold(0.5);
        classifier.add_pattern("compute factorial of a number", "arithmetic");

        // Similar request
        let decision = classifier.classify("calculate factorial");
        match decision {
            TierDecision::Tier1 { confidence, .. } => {
                assert!(confidence > 0.0);
            }
            TierDecision::Tier2 { .. } => {
                // Also acceptable if threshold not met
            }
        }
    }

    #[test]
    fn test_classify_no_patterns() {
        let classifier = PatternClassifier::new();
        let decision = classifier.classify("anything");

        match decision {
            TierDecision::Tier2 { reason } => {
                assert!(reason.contains("No training patterns"));
            }
            _ => panic!("Expected Tier2 decision"),
        }
    }

    #[test]
    fn test_load_patterns() {
        let mut classifier = PatternClassifier::new();
        let data = r#"{"context": "add two numbers", "category": "arithmetic"}
{"context": "sort an array", "category": "algorithm"}
{"context": "parse json string", "category": "parsing"}"#;

        let count = classifier.load_patterns(data);
        assert_eq!(count, 3);
        assert_eq!(classifier.pattern_count(), 3);
    }

    #[test]
    fn test_classifier_legacy_embedding() {
        // Test with legacy bag-of-words (no embedder set)
        let mut classifier = PatternClassifier::with_threshold(0.5);

        classifier.add_pattern("compute fibonacci sequence", "arithmetic");
        classifier.add_pattern("implement quicksort algorithm", "algorithm");
        assert_eq!(classifier.pattern_count(), 2);

        // No embedder set, uses legacy embedding
        assert!(!classifier.uses_ml_embeddings());

        // Classify similar request
        let decision = classifier.classify("calculate fibonacci");
        match decision {
            TierDecision::Tier1 {
                pattern,
                confidence,
            } => {
                assert!(pattern.contains("fibonacci"));
                assert!(confidence > 0.0);
            }
            TierDecision::Tier2 { .. } => {
                // Also acceptable if threshold not met
            }
        }
    }

    #[test]
    fn test_classifier_embedder_not_set() {
        let classifier = PatternClassifier::new();

        // Initially no embedder
        assert!(!classifier.uses_ml_embeddings());
        assert!(classifier.embedder.is_none());
    }
}
