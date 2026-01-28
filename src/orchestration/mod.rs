//! Two-Tier Orchestration System
//!
//! This module implements the LLM-as-project-manager pattern for handling
//! complex tasks that are beyond the small model's training data.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    PATTERN CLASSIFIER                           │
//! │  Embed request, compare to known patterns via cosine similarity │
//! │  - If similarity > threshold -> Tier 1 (small model)            │
//! │  - If similarity < threshold -> Tier 2 (LLM decomposes task)    │
//! └─────────────────────────────────────────────────────────────────┘
//!                               │
//!               ┌───────────────┴───────────────┐
//!               ▼                               ▼
//! ┌─────────────────────────┐    ┌─────────────────────────────────┐
//! │   TIER 1: SMALL MODEL   │    │   TIER 2: LLM PROJECT MANAGER   │
//! │   50-100M params, ONNX  │    │   Claude/Ollama/OpenAI          │
//! │   64 parallel slots     │    │   Decomposes complex tasks      │
//! │   ~30ms inference       │    │   Returns subtasks for Tier 1   │
//! └─────────────────────────┘    └─────────────────────────────────┘
//! ```
//!
//! # Key Insight
//!
//! The LLM does NOT write IR directly. It breaks complex tasks into
//! subtasks that the small model can handle. This ensures ALL output
//! goes through the verification loop.
//!
//! # Example
//!
//! ```ignore
//! use neurlang::orchestration::{TwoTierOrchestrator, OrchestratorConfig};
//!
//! let config = OrchestratorConfig::default();
//! let mut orchestrator = TwoTierOrchestrator::new(config)?;
//!
//! // Complex task gets decomposed by LLM, then each subtask
//! // is handled by the small model with verification
//! let result = orchestrator.generate("build REST API with auth", &tests)?;
//! ```

pub mod backends;
pub mod classifier;
pub mod collector;

use std::path::PathBuf;

pub use backends::{BackendRegistry, DecomposeResult, LlmBackend, Subtask};
pub use classifier::{PatternClassifier, TierDecision};
pub use collector::TrainingDataCollector;

/// Configuration for the two-tier orchestrator
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Similarity threshold for tier decision (0.0-1.0)
    /// Higher = more tasks go to Tier 2 (LLM)
    pub similarity_threshold: f32,

    /// Maximum iterations for Tier 1 before escalating to Tier 2
    pub tier1_max_iterations: usize,

    /// Maximum subtasks from Tier 2 decomposition
    pub max_subtasks: usize,

    /// Default LLM backend name
    pub default_backend: String,

    /// Path to store collected training data
    pub training_data_path: Option<PathBuf>,

    /// Whether to collect training data from successful generations
    pub collect_training_data: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.85,
            tier1_max_iterations: 100,
            max_subtasks: 10,
            default_backend: "claude".to_string(),
            training_data_path: None,
            collect_training_data: true,
        }
    }
}

/// Result of orchestration
#[derive(Debug)]
pub struct OrchestrationResult {
    /// Whether the task succeeded
    pub success: bool,

    /// The tier that handled the task (1 or 2)
    pub tier_used: u8,

    /// Number of iterations used
    pub iterations: usize,

    /// Subtasks (if Tier 2 was used)
    pub subtasks: Vec<SubtaskResult>,

    /// Error message if failed
    pub error: Option<String>,
}

/// Result of a single subtask
#[derive(Debug)]
pub struct SubtaskResult {
    /// The subtask description
    pub description: String,

    /// Whether it succeeded
    pub success: bool,

    /// Number of iterations
    pub iterations: usize,
}

/// Orchestration error types
#[derive(Debug)]
pub enum OrchestratorError {
    /// Tier 1 (small model) failed after max iterations
    Tier1Failed {
        iterations: usize,
        last_error: String,
    },

    /// Tier 2 (LLM) failed to decompose task
    DecomposeFailed { reason: String },

    /// Backend not found
    BackendNotFound { name: String },

    /// Backend error
    BackendError { backend: String, error: String },

    /// All subtasks failed
    AllSubtasksFailed { completed: usize, total: usize },

    /// Configuration error
    ConfigError { message: String },
}

impl std::fmt::Display for OrchestratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrchestratorError::Tier1Failed {
                iterations,
                last_error,
            } => {
                write!(
                    f,
                    "Tier 1 failed after {} iterations: {}",
                    iterations, last_error
                )
            }
            OrchestratorError::DecomposeFailed { reason } => {
                write!(f, "Failed to decompose task: {}", reason)
            }
            OrchestratorError::BackendNotFound { name } => {
                write!(f, "Backend not found: {}", name)
            }
            OrchestratorError::BackendError { backend, error } => {
                write!(f, "Backend '{}' error: {}", backend, error)
            }
            OrchestratorError::AllSubtasksFailed { completed, total } => {
                write!(f, "All subtasks failed ({}/{})", completed, total)
            }
            OrchestratorError::ConfigError { message } => {
                write!(f, "Configuration error: {}", message)
            }
        }
    }
}

impl std::error::Error for OrchestratorError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = OrchestratorConfig::default();
        assert_eq!(config.similarity_threshold, 0.85);
        assert_eq!(config.tier1_max_iterations, 100);
        assert_eq!(config.default_backend, "claude");
    }
}
