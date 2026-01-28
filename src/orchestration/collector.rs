//! Training Data Collector
//!
//! Captures successful task->IR mappings for continuous learning.
//! When Tier 2 (LLM) successfully decomposes a task and Tier 1 generates
//! verified IR, the mapping is recorded for future training.
//!
//! # Data Flow
//!
//! 1. LLM decomposes complex task into subtasks
//! 2. Small model generates IR for each subtask
//! 3. IR passes verification
//! 4. Collector records: (subtask description, verified IR)
//! 5. On next training run, these examples are included
//! 6. Small model learns the pattern
//! 7. Future similar tasks go directly to Tier 1

use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

/// A training example captured from successful generation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrainingExample {
    /// The task/subtask description
    pub context: String,

    /// Partial IR that was already generated (if any)
    #[serde(default)]
    pub partial_ir: Vec<u32>,

    /// Error feedback that led to this solution (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_feedback: Option<String>,

    /// The verified correct IR
    pub expected_ir: Vec<u32>,

    /// Category/tag for this example
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Timestamp when captured
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Collector for training data
pub struct TrainingDataCollector {
    /// Output path for JSONL file
    output_path: PathBuf,

    /// Buffer for batch writing
    buffer: Vec<TrainingExample>,

    /// Batch size before flushing
    batch_size: usize,

    /// Total examples collected this session
    total_collected: usize,

    /// Whether collection is enabled
    enabled: bool,
}

impl TrainingDataCollector {
    /// Create a new collector
    pub fn new(output_path: impl AsRef<Path>) -> Self {
        Self {
            output_path: output_path.as_ref().to_path_buf(),
            buffer: Vec::new(),
            batch_size: 100,
            total_collected: 0,
            enabled: true,
        }
    }

    /// Set the batch size before auto-flush
    pub fn set_batch_size(&mut self, size: usize) {
        self.batch_size = size.max(1);
    }

    /// Enable or disable collection
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if collection is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Record a successful generation
    pub fn record_success(&mut self, context: &str, ir: &[u32]) {
        if !self.enabled {
            return;
        }

        let example = TrainingExample {
            context: context.to_string(),
            partial_ir: vec![],
            error_feedback: None,
            expected_ir: ir.to_vec(),
            category: None,
            timestamp: Some(chrono_timestamp()),
        };

        self.buffer.push(example);
        self.total_collected += 1;

        if self.buffer.len() >= self.batch_size {
            let _ = self.flush();
        }
    }

    /// Record a successful error recovery
    pub fn record_error_recovery(
        &mut self,
        context: &str,
        error: &str,
        partial: &[u32],
        fixed_ir: &[u32],
    ) {
        if !self.enabled {
            return;
        }

        let example = TrainingExample {
            context: context.to_string(),
            partial_ir: partial.to_vec(),
            error_feedback: Some(error.to_string()),
            expected_ir: fixed_ir.to_vec(),
            category: Some("error_recovery".to_string()),
            timestamp: Some(chrono_timestamp()),
        };

        self.buffer.push(example);
        self.total_collected += 1;

        if self.buffer.len() >= self.batch_size {
            let _ = self.flush();
        }
    }

    /// Record with explicit category
    pub fn record_with_category(&mut self, context: &str, ir: &[u32], category: &str) {
        if !self.enabled {
            return;
        }

        let example = TrainingExample {
            context: context.to_string(),
            partial_ir: vec![],
            error_feedback: None,
            expected_ir: ir.to_vec(),
            category: Some(category.to_string()),
            timestamp: Some(chrono_timestamp()),
        };

        self.buffer.push(example);
        self.total_collected += 1;

        if self.buffer.len() >= self.batch_size {
            let _ = self.flush();
        }
    }

    /// Flush buffered examples to disk
    pub fn flush(&mut self) -> std::io::Result<usize> {
        if self.buffer.is_empty() {
            return Ok(0);
        }

        // Ensure parent directory exists
        if let Some(parent) = self.output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Open file in append mode
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.output_path)?;
        let mut writer = BufWriter::new(file);

        let count = self.buffer.len();

        for example in self.buffer.drain(..) {
            let json = serde_json::to_string(&example).unwrap_or_default();
            writeln!(writer, "{}", json)?;
        }

        writer.flush()?;

        Ok(count)
    }

    /// Get the number of examples in the buffer
    pub fn buffered_count(&self) -> usize {
        self.buffer.len()
    }

    /// Get total examples collected this session
    pub fn total_collected(&self) -> usize {
        self.total_collected
    }

    /// Get the output path
    pub fn output_path(&self) -> &Path {
        &self.output_path
    }
}

impl Drop for TrainingDataCollector {
    fn drop(&mut self) {
        // Flush any remaining examples on drop
        let _ = self.flush();
    }
}

/// Generate a timestamp string
fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    format!("{}", duration.as_secs())
}

/// Read training examples from a JSONL file
pub fn read_training_data(path: impl AsRef<Path>) -> std::io::Result<Vec<TrainingExample>> {
    let content = std::fs::read_to_string(path)?;
    let mut examples = Vec::new();

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(example) = serde_json::from_str::<TrainingExample>(line) {
            examples.push(example);
        }
    }

    Ok(examples)
}

/// Count examples in a JSONL file without loading all into memory
pub fn count_training_examples(path: impl AsRef<Path>) -> std::io::Result<usize> {
    let content = std::fs::read_to_string(path)?;
    Ok(content.lines().filter(|l| !l.trim().is_empty()).count())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_collector_creation() {
        let collector = TrainingDataCollector::new("/tmp/test.jsonl");
        assert!(collector.is_enabled());
        assert_eq!(collector.buffered_count(), 0);
    }

    #[test]
    fn test_record_success() {
        let temp_path = temp_dir().join("neurlang_test_collector.jsonl");
        let _ = std::fs::remove_file(&temp_path);

        let mut collector = TrainingDataCollector::new(&temp_path);
        collector.record_success("compute factorial", &[1, 2, 3, 4]);
        assert_eq!(collector.buffered_count(), 1);
        assert_eq!(collector.total_collected(), 1);

        collector.flush().unwrap();
        assert_eq!(collector.buffered_count(), 0);

        // Read back
        let examples = read_training_data(&temp_path).unwrap();
        assert_eq!(examples.len(), 1);
        assert_eq!(examples[0].context, "compute factorial");

        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_record_error_recovery() {
        let temp_path = temp_dir().join("neurlang_test_recovery.jsonl");
        let _ = std::fs::remove_file(&temp_path);

        let mut collector = TrainingDataCollector::new(&temp_path);
        collector.record_error_recovery(
            "compute factorial",
            "Expected 120, got 119",
            &[1, 2, 3],
            &[1, 2, 3, 4],
        );

        collector.flush().unwrap();

        let examples = read_training_data(&temp_path).unwrap();
        assert_eq!(examples.len(), 1);
        assert!(examples[0].error_feedback.is_some());
        assert_eq!(examples[0].category, Some("error_recovery".to_string()));

        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_disabled_collector() {
        let mut collector = TrainingDataCollector::new("/tmp/disabled.jsonl");
        collector.set_enabled(false);

        collector.record_success("test", &[1, 2, 3]);
        assert_eq!(collector.buffered_count(), 0);
        assert_eq!(collector.total_collected(), 0);
    }
}
