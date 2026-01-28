//! Tract Inference Engine
//!
//! Pure Rust ONNX inference engine. Produces the smallest binary size
//! and has no external dependencies, but is CPU-only.
//!
//! Best for:
//! - Embedded systems
//! - Minimal binary deployments
//! - CPU-only inference

use super::{Engine, EngineError, EnginePrediction, EngineType};
use crate::inference::tokenizer::{extract_numbers, FastTokenizer, MAX_SEQ_LEN};
use std::path::Path;
use std::time::Instant;
use tract_onnx::prelude::*;

/// Tract inference engine
pub struct TractEngine {
    model: SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>,
    tokenizer: FastTokenizer,
}

impl TractEngine {
    /// Load a model from an ONNX file
    pub fn load(model_path: &Path) -> Result<Self, EngineError> {
        // Load and optimize the ONNX model
        let model = tract_onnx::onnx()
            .model_for_path(model_path)
            .map_err(|e| EngineError::LoadError(format!("Failed to load ONNX model: {}", e)))?
            // Set input shape (batch=1, seq_len=MAX_SEQ_LEN)
            .with_input_fact(0, f64::fact([1, MAX_SEQ_LEN]).into())
            .map_err(|e| EngineError::LoadError(format!("Failed to set input shape: {}", e)))?
            // Optimize for inference
            .into_optimized()
            .map_err(|e| EngineError::LoadError(format!("Failed to optimize model: {}", e)))?
            // Create runnable plan
            .into_runnable()
            .map_err(|e| EngineError::LoadError(format!("Failed to create runnable: {}", e)))?;

        Ok(Self {
            model,
            tokenizer: FastTokenizer::new(),
        })
    }

    /// Run inference on tokenized input
    fn run_inference(&self, tokens: Vec<i64>) -> Result<(usize, f32, usize), EngineError> {
        // Create input tensor (tract uses f32 for input indices, need to cast)
        let input: Tensor = tract_ndarray::Array2::from_shape_vec(
            (1, MAX_SEQ_LEN),
            tokens.into_iter().map(|t| t as f32).collect(),
        )
        .map_err(|e| EngineError::InferenceError(format!("Failed to create tensor: {}", e)))?
        .into();

        // Run inference
        let outputs = self
            .model
            .run(tvec![input.into()])
            .map_err(|e| EngineError::InferenceError(format!("Inference failed: {}", e)))?;

        // Extract intent logits from first output
        let intent_tensor = outputs[0]
            .to_array_view::<f32>()
            .map_err(|e| EngineError::InferenceError(format!("Failed to extract output: {}", e)))?;

        let intent_logits: Vec<f32> = intent_tensor.iter().cloned().collect();
        let intent_id = argmax(&intent_logits);
        let confidence = softmax_max(&intent_logits);

        // Extract operand count from second output if available
        let operand_count = if outputs.len() > 1 {
            let count_tensor = outputs[1].to_array_view::<f32>().map_err(|e| {
                EngineError::InferenceError(format!("Failed to extract count output: {}", e))
            })?;
            let count_logits: Vec<f32> = count_tensor.iter().cloned().collect();
            argmax(&count_logits).min(4)
        } else {
            crate::inference::lookup::operand_count(intent_id).unwrap_or(2)
        };

        Ok((intent_id, confidence, operand_count))
    }
}

impl Engine for TractEngine {
    fn engine_type(&self) -> EngineType {
        EngineType::Tract
    }

    fn predict(&self, text: &str) -> Result<EnginePrediction, EngineError> {
        let start = Instant::now();

        // Tokenize input
        let tokens = self.tokenizer.encode(text);

        // Run inference
        let (intent_id, confidence, operand_count) = self.run_inference(tokens)?;

        // Extract operands from text
        let operands: Vec<i64> = extract_numbers(text)
            .into_iter()
            .take(operand_count)
            .collect();

        // Compute signs
        let signs: Vec<usize> = operands
            .iter()
            .map(|&op| if op < 0 { 1 } else { 0 })
            .collect();

        Ok(EnginePrediction {
            intent_id,
            confidence,
            operand_count,
            operands,
            signs,
            latency: start.elapsed(),
            engine: EngineType::Tract,
        })
    }

    fn is_ready(&self) -> bool {
        true
    }

    fn model_info(&self) -> String {
        "Tract Engine (Pure Rust ONNX)".to_string()
    }
}

/// Find argmax of a slice
fn argmax(values: &[f32]) -> usize {
    values
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Get softmax probability of max element
fn softmax_max(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let max_val = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exp_sum: f32 = values.iter().map(|&v| (v - max_val).exp()).sum();
    let max_exp = (values[argmax(values)] - max_val).exp();
    max_exp / exp_sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argmax() {
        assert_eq!(argmax(&[1.0, 2.0, 3.0, 0.5]), 2);
        assert_eq!(argmax(&[5.0, 2.0, 3.0, 0.5]), 0);
    }

    #[test]
    fn test_softmax_max() {
        let probs = softmax_max(&[1.0, 2.0, 3.0, 0.5]);
        assert!(probs > 0.5);
        assert!(probs < 1.0);
    }
}
