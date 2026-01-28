//! Multi-Head ONNX Inference for Direct Prediction
//!
//! This module provides ONNX-based inference for the multi-head prediction model
//! that outputs structured tensors (intent + operands) in a single forward pass.
//!
//! # Architecture
//!
//! ```text
//! Input: "add 5 and 3"
//!         │
//!         ▼
//! ┌───────────────────┐
//! │  Shared Encoder   │
//! │  (CNN/Transform)  │
//! └─────────┬─────────┘
//!           │
//!     ┌─────┼─────┬─────┐
//!     ▼     ▼     ▼     ▼
//! ┌──────┐ ┌───┐ ┌───┐ ┌───┐
//! │Intent│ │Op1│ │Op2│ │OpN│
//! │ Head │ │   │ │   │ │   │
//! └──────┘ └───┘ └───┘ └───┘
//!     │     │     │     │
//!     ▼     ▼     ▼     ▼
//!   intent=0  5    3   N/A
//!   (ADD)
//! ```

use crate::inference::tokenizer::{extract_numbers, FastTokenizer};
use std::path::Path;
use std::time::{Duration, Instant};
use thiserror::Error;

#[cfg(feature = "onnx")]
use ort::session::Session;

/// Multi-head inference errors
#[derive(Debug, Error)]
pub enum MultiHeadError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Model load error: {0}")]
    LoadError(String),
    #[error("Inference error: {0}")]
    InferenceError(String),
    #[error("Invalid prediction: {0}")]
    InvalidPrediction(String),
}

/// Prediction result from multi-head model
#[derive(Debug, Clone)]
pub struct MultiHeadPrediction {
    /// Predicted intent ID (0-53)
    pub intent_id: usize,
    /// Intent confidence (0.0 - 1.0)
    pub intent_confidence: f32,
    /// Predicted operand count
    pub operand_count: usize,
    /// Operand values (extracted from text or predicted)
    pub operands: Vec<i64>,
    /// Inference latency
    pub latency: Duration,
}

/// Multi-head inference engine
pub struct MultiHeadInference {
    /// ONNX session (when feature enabled)
    #[cfg(feature = "onnx")]
    session: Option<Session>,
    /// Tokenizer (used in predict methods)
    #[allow(dead_code)]
    tokenizer: FastTokenizer,
    /// Whether model is loaded
    loaded: bool,
}

impl MultiHeadInference {
    /// Load model from file
    #[cfg(feature = "onnx")]
    pub fn load(model_path: &Path) -> Result<Self, MultiHeadError> {
        if !model_path.exists() {
            return Err(MultiHeadError::ModelNotFound(
                model_path.display().to_string(),
            ));
        }

        let session = Session::builder()
            .map_err(|e| MultiHeadError::LoadError(format!("Session builder error: {}", e)))?
            .commit_from_file(model_path)
            .map_err(|e| MultiHeadError::LoadError(format!("Failed to load model: {}", e)))?;

        Ok(Self {
            session: Some(session),
            tokenizer: FastTokenizer::new(),
            loaded: true,
        })
    }

    /// Load model from file (mock when ONNX disabled)
    #[cfg(not(feature = "onnx"))]
    pub fn load(_model_path: &Path) -> Result<Self, MultiHeadError> {
        Ok(Self {
            tokenizer: FastTokenizer::new(),
            loaded: true,
        })
    }

    /// Create mock inference engine for testing
    pub fn mock() -> Self {
        Self {
            #[cfg(feature = "onnx")]
            session: None,
            tokenizer: FastTokenizer::new(),
            loaded: true,
        }
    }

    /// Predict intent and operands from text
    pub fn predict(&mut self, text: &str) -> Result<MultiHeadPrediction, MultiHeadError> {
        let start = Instant::now();

        #[cfg(feature = "onnx")]
        {
            self.predict_onnx(text, start)
        }

        #[cfg(not(feature = "onnx"))]
        {
            self.predict_fallback(text, start)
        }
    }

    /// ONNX-based prediction
    #[cfg(feature = "onnx")]
    fn predict_onnx(
        &mut self,
        text: &str,
        start: Instant,
    ) -> Result<MultiHeadPrediction, MultiHeadError> {
        use ort::value::Tensor;

        // If no session loaded, fall back to keyword detection
        let session = match &mut self.session {
            Some(s) => s,
            None => return self.predict_fallback_onnx(text, start),
        };

        // Tokenize input
        let tokens = self.tokenizer.encode(text);

        // Create input tensor using ort's expected format
        let input_shape = [1_usize, MAX_SEQ_LEN];
        let input_tensor = Tensor::from_array((&input_shape[..], tokens))
            .map_err(|e| MultiHeadError::InferenceError(format!("Tensor creation error: {}", e)))?;

        // Run inference
        let outputs = session
            .run(ort::inputs![input_tensor])
            .map_err(|e| MultiHeadError::InferenceError(format!("Run error: {}", e)))?;

        // Extract intent prediction
        let intent_output = &outputs[0];
        let intent_tensor = intent_output
            .downcast_ref::<ort::value::DynTensorValueType>()
            .map_err(|e| MultiHeadError::InferenceError(format!("Downcast error: {}", e)))?;
        let (_, intent_logits) = intent_tensor
            .try_extract_tensor::<f32>()
            .map_err(|e| MultiHeadError::InferenceError(format!("Extract error: {}", e)))?;

        let intent_id = argmax(intent_logits);
        let intent_confidence = softmax_max(intent_logits);

        // Extract operand count prediction
        let operand_count = if outputs.len() > 1 {
            let count_output = &outputs[1];
            let count_tensor = count_output
                .downcast_ref::<ort::value::DynTensorValueType>()
                .map_err(|e| MultiHeadError::InferenceError(format!("Downcast error: {}", e)))?;
            let (_, count_logits) = count_tensor
                .try_extract_tensor::<f32>()
                .map_err(|e| MultiHeadError::InferenceError(format!("Extract error: {}", e)))?;
            argmax(count_logits).min(4)
        } else {
            // Infer from intent
            crate::inference::lookup::operand_count(intent_id).unwrap_or(2)
        };

        // Extract operands from text (model predicts count, text provides values)
        let operands = extract_numbers(text)
            .into_iter()
            .take(operand_count)
            .collect();

        Ok(MultiHeadPrediction {
            intent_id,
            intent_confidence,
            operand_count,
            operands,
            latency: start.elapsed(),
        })
    }

    /// Fallback prediction for ONNX mode when model not loaded
    #[cfg(feature = "onnx")]
    fn predict_fallback_onnx(
        &mut self,
        text: &str,
        start: Instant,
    ) -> Result<MultiHeadPrediction, MultiHeadError> {
        use crate::inference::lookup::detect_intent_from_keywords;
        use crate::inference::tokenizer::parse_symbolic_expression;

        // Try symbolic expression parsing first (highest accuracy)
        if let Some(expr) = parse_symbolic_expression(text) {
            if let Some(intent_id) = expr.intent_id {
                return Ok(MultiHeadPrediction {
                    intent_id,
                    intent_confidence: 0.95,
                    operand_count: expr.operands.len(),
                    operands: expr.operands,
                    latency: start.elapsed(),
                });
            }
        }

        // Try keyword detection
        if let Some((intent_id, confidence)) = detect_intent_from_keywords(text) {
            let operand_count = crate::inference::lookup::operand_count(intent_id).unwrap_or(2);
            let operands = extract_numbers(text)
                .into_iter()
                .take(operand_count)
                .collect();

            return Ok(MultiHeadPrediction {
                intent_id,
                intent_confidence: confidence,
                operand_count,
                operands,
                latency: start.elapsed(),
            });
        }

        // Default to ADD with extracted numbers
        let operands = extract_numbers(text);
        Ok(MultiHeadPrediction {
            intent_id: 0, // ADD
            intent_confidence: 0.5,
            operand_count: operands.len().min(4),
            operands: operands.into_iter().take(4).collect(),
            latency: start.elapsed(),
        })
    }

    /// Fallback prediction using keyword matching and symbolic parsing
    #[cfg(not(feature = "onnx"))]
    fn predict_fallback(
        &self,
        text: &str,
        start: Instant,
    ) -> Result<MultiHeadPrediction, MultiHeadError> {
        use crate::inference::lookup::detect_intent_from_keywords;
        use crate::inference::tokenizer::parse_symbolic_expression;

        // Try symbolic expression parsing first (highest accuracy)
        if let Some(expr) = parse_symbolic_expression(text) {
            if let Some(intent_id) = expr.intent_id {
                return Ok(MultiHeadPrediction {
                    intent_id,
                    intent_confidence: 0.95,
                    operand_count: expr.operands.len(),
                    operands: expr.operands,
                    latency: start.elapsed(),
                });
            }
        }

        // Try keyword detection
        if let Some((intent_id, confidence)) = detect_intent_from_keywords(text) {
            let operand_count = crate::inference::lookup::operand_count(intent_id).unwrap_or(2);
            let operands = extract_numbers(text)
                .into_iter()
                .take(operand_count)
                .collect();

            return Ok(MultiHeadPrediction {
                intent_id,
                intent_confidence: confidence,
                operand_count,
                operands,
                latency: start.elapsed(),
            });
        }

        // Default to ADD with extracted numbers
        let operands = extract_numbers(text);
        Ok(MultiHeadPrediction {
            intent_id: 0, // ADD
            intent_confidence: 0.5,
            operand_count: operands.len().min(4),
            operands: operands.into_iter().take(4).collect(),
            latency: start.elapsed(),
        })
    }

    /// Check if model is loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }
}

/// Find argmax of a slice
#[allow(dead_code)]
fn argmax(values: &[f32]) -> usize {
    values
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Get softmax probability of max element
#[allow(dead_code)]
fn softmax_max(values: &[f32]) -> f32 {
    let max_val = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exp_sum: f32 = values.iter().map(|&v| (v - max_val).exp()).sum();
    let max_exp = (values[argmax(values)] - max_val).exp();
    max_exp / exp_sum
}

/// Batch prediction for efficiency
#[cfg(feature = "onnx")]
pub struct BatchMultiHeadInference {
    session: Session,
    tokenizer: FastTokenizer,
    #[allow(dead_code)]
    batch_size: usize,
}

#[cfg(feature = "onnx")]
impl BatchMultiHeadInference {
    /// Create batch inference engine
    pub fn new(model_path: &Path, batch_size: usize) -> Result<Self, MultiHeadError> {
        let session = Session::builder()
            .map_err(|e| MultiHeadError::LoadError(format!("Session builder error: {}", e)))?
            .commit_from_file(model_path)
            .map_err(|e| MultiHeadError::LoadError(format!("Failed to load model: {}", e)))?;

        Ok(Self {
            session,
            tokenizer: FastTokenizer::new(),
            batch_size,
        })
    }

    /// Predict batch of texts
    pub fn predict_batch(
        &mut self,
        texts: &[&str],
    ) -> Result<Vec<MultiHeadPrediction>, MultiHeadError> {
        use ort::value::Tensor;

        let start = Instant::now();

        // Tokenize all inputs
        let mut all_tokens = Vec::with_capacity(texts.len() * MAX_SEQ_LEN);
        for text in texts {
            all_tokens.extend(self.tokenizer.encode(text));
        }

        // Create batch input tensor
        let input_shape = [texts.len(), MAX_SEQ_LEN];
        let input_tensor = Tensor::from_array((&input_shape[..], all_tokens))
            .map_err(|e| MultiHeadError::InferenceError(format!("Tensor creation error: {}", e)))?;

        // Run batch inference
        let outputs = self
            .session
            .run(ort::inputs![input_tensor])
            .map_err(|e| MultiHeadError::InferenceError(format!("Run error: {}", e)))?;

        // Extract intent predictions
        let intent_output = &outputs[0];
        let intent_tensor = intent_output
            .downcast_ref::<ort::value::DynTensorValueType>()
            .map_err(|e| MultiHeadError::InferenceError(format!("Downcast error: {}", e)))?;
        let (shape, intent_logits) = intent_tensor
            .try_extract_tensor::<f32>()
            .map_err(|e| MultiHeadError::InferenceError(format!("Extract error: {}", e)))?;

        let batch_size = shape[0] as usize;
        let num_intents = shape[1] as usize;

        let latency = start.elapsed();
        let per_item_latency = Duration::from_nanos(latency.as_nanos() as u64 / batch_size as u64);

        // Process each item in batch
        let mut predictions = Vec::with_capacity(batch_size);
        for i in 0..batch_size {
            let item_logits = &intent_logits[i * num_intents..(i + 1) * num_intents];
            let intent_id = argmax(item_logits);
            let intent_confidence = softmax_max(item_logits);

            let operand_count = crate::inference::lookup::operand_count(intent_id).unwrap_or(2);
            let operands = extract_numbers(texts[i])
                .into_iter()
                .take(operand_count)
                .collect();

            predictions.push(MultiHeadPrediction {
                intent_id,
                intent_confidence,
                operand_count,
                operands,
                latency: per_item_latency,
            });
        }

        Ok(predictions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_inference() {
        // Test fallback prediction using mock()
        let mut inference = MultiHeadInference::mock();

        // This works for both ONNX and non-ONNX modes since mock() sets session to None
        let pred = inference.predict("add 5 and 3").unwrap();
        assert_eq!(pred.intent_id, 0); // ADD
        assert_eq!(pred.operands, vec![5, 3]);

        let pred = inference.predict("factorial of 10").unwrap();
        assert_eq!(pred.intent_id, 11); // FACTORIAL
        assert_eq!(pred.operands, vec![10]);

        let pred = inference.predict("5 + 3").unwrap();
        assert_eq!(pred.intent_id, 0); // ADD
        assert_eq!(pred.operands, vec![5, 3]);

        let pred = inference.predict("fibonacci(20)").unwrap();
        assert_eq!(pred.intent_id, 12); // FIBONACCI
        assert_eq!(pred.operands, vec![20]);
    }

    #[test]
    fn test_argmax() {
        assert_eq!(argmax(&[1.0, 2.0, 3.0, 0.5]), 2);
        assert_eq!(argmax(&[5.0, 2.0, 3.0, 0.5]), 0);
        assert_eq!(argmax(&[1.0, 2.0, 3.0, 10.0]), 3);
    }

    #[test]
    fn test_softmax_max() {
        let probs = softmax_max(&[1.0, 2.0, 3.0, 0.5]);
        assert!(probs > 0.5); // Max element should have high probability
        assert!(probs < 1.0);
    }
}
