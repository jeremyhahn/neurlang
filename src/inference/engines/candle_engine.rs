//! Candle Inference Engine
//!
//! Hugging Face's Rust ML framework. Provides a good balance between
//! performance and features, with CUDA support.
//!
//! Best for:
//! - Hugging Face ecosystem integration
//! - Safetensors model format
//! - CUDA acceleration without ONNX Runtime

use super::{Engine, EngineError, EnginePrediction, EngineType};
use crate::inference::tokenizer::{extract_numbers, FastTokenizer, MAX_SEQ_LEN};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use std::path::Path;
use std::time::Instant;

/// Candle inference engine
pub struct CandleEngine {
    model: MultiHeadModel,
    device: Device,
    tokenizer: FastTokenizer,
}

/// Simple multi-head model for candle
struct MultiHeadModel {
    encoder: Encoder,
    intent_head: Linear,
    count_head: Linear,
}

struct Encoder {
    embedding: candle_nn::Embedding,
    conv1: candle_nn::Conv1d,
    conv2: candle_nn::Conv1d,
    conv3: candle_nn::Conv1d,
}

struct Linear {
    weight: Tensor,
    bias: Tensor,
}

impl MultiHeadModel {
    fn load(vb: VarBuilder, vocab_size: usize, num_intents: usize) -> Result<Self, EngineError> {
        let embed_dim = 64;
        let hidden_dim = 256;

        // Create encoder
        let encoder = Encoder::load(vb.pp("encoder"), vocab_size, embed_dim, hidden_dim)?;

        // Create heads
        let intent_head = Linear::load(vb.pp("intent_head"), hidden_dim, num_intents)?;
        let count_head = Linear::load(vb.pp("count_head"), hidden_dim, 5)?;

        Ok(Self {
            encoder,
            intent_head,
            count_head,
        })
    }

    fn forward(&self, tokens: &Tensor) -> Result<(Tensor, Tensor), EngineError> {
        // Encode tokens
        let features = self.encoder.forward(tokens)?;

        // Predict intent and count
        let intent_logits = self.intent_head.forward(&features)?;
        let count_logits = self.count_head.forward(&features)?;

        Ok((intent_logits, count_logits))
    }
}

impl Encoder {
    fn load(
        vb: VarBuilder,
        vocab_size: usize,
        embed_dim: usize,
        hidden_dim: usize,
    ) -> Result<Self, EngineError> {
        let embedding = candle_nn::embedding(vocab_size, embed_dim, vb.pp("embedding"))
            .map_err(|e| EngineError::LoadError(format!("Failed to load embedding: {}", e)))?;

        let conv1_config = candle_nn::Conv1dConfig {
            padding: 1,
            ..Default::default()
        };
        let conv1 = candle_nn::conv1d(embed_dim, 64, 3, conv1_config, vb.pp("conv1"))
            .map_err(|e| EngineError::LoadError(format!("Failed to load conv1: {}", e)))?;

        let conv2 = candle_nn::conv1d(64, 128, 3, conv1_config, vb.pp("conv2"))
            .map_err(|e| EngineError::LoadError(format!("Failed to load conv2: {}", e)))?;

        let conv3 = candle_nn::conv1d(128, hidden_dim, 3, conv1_config, vb.pp("conv3"))
            .map_err(|e| EngineError::LoadError(format!("Failed to load conv3: {}", e)))?;

        Ok(Self {
            embedding,
            conv1,
            conv2,
            conv3,
        })
    }

    fn forward(&self, tokens: &Tensor) -> Result<Tensor, EngineError> {
        // Embed tokens: (batch, seq) -> (batch, seq, embed_dim)
        let x = self
            .embedding
            .forward(tokens)
            .map_err(|e| EngineError::InferenceError(format!("Embedding forward failed: {}", e)))?;

        // Transpose for conv1d: (batch, embed_dim, seq)
        let x = x
            .transpose(1, 2)
            .map_err(|e| EngineError::InferenceError(format!("Transpose failed: {}", e)))?;

        // Conv layers with ReLU
        let x = self
            .conv1
            .forward(&x)
            .map_err(|e| EngineError::InferenceError(format!("Conv1 forward failed: {}", e)))?;
        let x = x
            .relu()
            .map_err(|e| EngineError::InferenceError(format!("ReLU failed: {}", e)))?;

        let x = self
            .conv2
            .forward(&x)
            .map_err(|e| EngineError::InferenceError(format!("Conv2 forward failed: {}", e)))?;
        let x = x
            .relu()
            .map_err(|e| EngineError::InferenceError(format!("ReLU failed: {}", e)))?;

        let x = self
            .conv3
            .forward(&x)
            .map_err(|e| EngineError::InferenceError(format!("Conv3 forward failed: {}", e)))?;
        let x = x
            .relu()
            .map_err(|e| EngineError::InferenceError(format!("ReLU failed: {}", e)))?;

        // Global max pooling: (batch, hidden_dim, seq) -> (batch, hidden_dim)
        let (_, _, seq_len) = x
            .dims3()
            .map_err(|e| EngineError::InferenceError(format!("Dims failed: {}", e)))?;
        let x = x
            .max(2)
            .map_err(|e| EngineError::InferenceError(format!("Max pool failed: {}", e)))?;

        // Squeeze the seq dimension
        let x = x.squeeze(2).unwrap_or(x); // Already squeezed

        Ok(x)
    }
}

impl Linear {
    fn load(vb: VarBuilder, in_features: usize, out_features: usize) -> Result<Self, EngineError> {
        let weight = vb
            .get((out_features, in_features), "weight")
            .map_err(|e| EngineError::LoadError(format!("Failed to load weight: {}", e)))?;

        let bias = vb
            .get(out_features, "bias")
            .map_err(|e| EngineError::LoadError(format!("Failed to load bias: {}", e)))?;

        Ok(Self { weight, bias })
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor, EngineError> {
        // Linear: y = x @ weight.T + bias
        let y = x
            .matmul(&self.weight.t().map_err(|e| {
                EngineError::InferenceError(format!("Weight transpose failed: {}", e))
            })?)
            .map_err(|e| EngineError::InferenceError(format!("Matmul failed: {}", e)))?;

        let y = y
            .broadcast_add(&self.bias)
            .map_err(|e| EngineError::InferenceError(format!("Bias add failed: {}", e)))?;

        Ok(y)
    }
}

impl CandleEngine {
    /// Load a model from an ONNX or safetensors file
    pub fn load(model_path: &Path) -> Result<Self, EngineError> {
        // Select device (CUDA if available, otherwise CPU)
        let device = Device::cuda_if_available(0).unwrap_or(Device::Cpu);

        let ext = model_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let model = match ext.as_str() {
            "safetensors" => {
                // Load from safetensors
                let vb = unsafe {
                    VarBuilder::from_mmaped_safetensors(&[model_path], DType::F32, &device)
                        .map_err(|e| {
                            EngineError::LoadError(format!("Failed to load safetensors: {}", e))
                        })?
                };
                MultiHeadModel::load(vb, 261, 54)?
            }
            "onnx" => {
                // For ONNX, we create a simple fallback model
                // In production, you'd want to properly convert the ONNX model
                return Err(EngineError::LoadError(
                    "Candle ONNX loading not fully implemented. Use safetensors format."
                        .to_string(),
                ));
            }
            _ => {
                return Err(EngineError::UnsupportedFormat(ext));
            }
        };

        Ok(Self {
            model,
            device,
            tokenizer: FastTokenizer::new(),
        })
    }

    /// Run inference
    fn run_inference(&self, tokens: Vec<i64>) -> Result<(usize, f32, usize), EngineError> {
        // Create input tensor
        let input = Tensor::new(tokens.as_slice(), &self.device)
            .map_err(|e| EngineError::InferenceError(format!("Failed to create tensor: {}", e)))?
            .unsqueeze(0) // Add batch dimension
            .map_err(|e| EngineError::InferenceError(format!("Unsqueeze failed: {}", e)))?
            .to_dtype(DType::I64)
            .map_err(|e| EngineError::InferenceError(format!("Type cast failed: {}", e)))?;

        // Forward pass
        let (intent_logits, count_logits) = self.model.forward(&input)?;

        // Extract predictions
        let intent_vec: Vec<f32> = intent_logits
            .squeeze(0)
            .map_err(|e| EngineError::InferenceError(format!("Squeeze failed: {}", e)))?
            .to_vec1()
            .map_err(|e| EngineError::InferenceError(format!("To vec failed: {}", e)))?;

        let count_vec: Vec<f32> = count_logits
            .squeeze(0)
            .map_err(|e| EngineError::InferenceError(format!("Squeeze failed: {}", e)))?
            .to_vec1()
            .map_err(|e| EngineError::InferenceError(format!("To vec failed: {}", e)))?;

        let intent_id = argmax(&intent_vec);
        let confidence = softmax_max(&intent_vec);
        let operand_count = argmax(&count_vec).min(4);

        Ok((intent_id, confidence, operand_count))
    }
}

impl Engine for CandleEngine {
    fn engine_type(&self) -> EngineType {
        EngineType::Candle
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
            engine: EngineType::Candle,
        })
    }

    fn is_ready(&self) -> bool {
        true
    }

    fn model_info(&self) -> String {
        let device_str = match &self.device {
            Device::Cpu => "CPU",
            Device::Cuda(_) => "CUDA",
            Device::Metal(_) => "Metal",
        };
        format!("Candle Engine ({})", device_str)
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
    }
}
