//! ONNX Runtime Inference Engine
//!
//! Provides the fastest inference with GPU support via CUDA, TensorRT, or CoreML.
//! This is the recommended engine for production deployments.

use super::{Engine, EngineError, EnginePrediction, EngineType};
use crate::inference::tokenizer::{extract_numbers, FastTokenizer, MAX_SEQ_LEN};
use ort::session::Session;
use ort::value::Tensor;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

/// ONNX Runtime inference engine
pub struct OrtEngine {
    session: Mutex<Session>,
    tokenizer: FastTokenizer,
}

impl OrtEngine {
    /// Load a model from an ONNX file
    pub fn load(model_path: &Path) -> Result<Self, EngineError> {
        let session = Session::builder()
            .map_err(|e| EngineError::LoadError(format!("Session builder error: {}", e)))?
            .commit_from_file(model_path)
            .map_err(|e| EngineError::LoadError(format!("Failed to load model: {}", e)))?;

        Ok(Self {
            session: Mutex::new(session),
            tokenizer: FastTokenizer::new(),
        })
    }

    /// Run inference on tokenized input
    fn run_inference(&self, tokens: Vec<i64>) -> Result<(usize, f32, usize), EngineError> {
        let mut session = self.session.lock().unwrap();

        // Create input tensor
        let input_shape = [1_usize, MAX_SEQ_LEN];
        let input_tensor = Tensor::from_array((&input_shape[..], tokens))
            .map_err(|e| EngineError::InferenceError(format!("Tensor creation error: {}", e)))?;

        // Run inference
        let outputs = session
            .run(ort::inputs![input_tensor])
            .map_err(|e| EngineError::InferenceError(format!("Run error: {}", e)))?;

        // Extract intent prediction (first output)
        let intent_output = &outputs[0];
        let (_, intent_logits) = intent_output
            .try_extract_tensor::<f32>()
            .map_err(|e| EngineError::InferenceError(format!("Extract error: {}", e)))?;

        let intent_id = argmax(intent_logits);
        let confidence = softmax_max(intent_logits);

        // Extract operand count prediction (second output if available)
        let operand_count = if outputs.len() > 1 {
            let count_output = &outputs[1];
            let (_, count_logits) = count_output
                .try_extract_tensor::<f32>()
                .map_err(|e| EngineError::InferenceError(format!("Extract error: {}", e)))?;
            argmax(count_logits).min(4)
        } else {
            // Infer from intent
            crate::inference::lookup::operand_count(intent_id).unwrap_or(2)
        };

        Ok((intent_id, confidence, operand_count))
    }
}

impl Engine for OrtEngine {
    fn engine_type(&self) -> EngineType {
        EngineType::Ort
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

        // Compute signs (0 = positive, 1 = negative)
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
            engine: EngineType::Ort,
        })
    }

    fn is_ready(&self) -> bool {
        true
    }

    fn model_info(&self) -> String {
        "ONNX Runtime Engine".to_string()
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
