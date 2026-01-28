//! Burn Inference Engine
//!
//! Native Rust ML framework for inference on .mpk format models
//! trained with the burn training backend.
//!
//! Best for:
//! - Native burn-trained models
//! - Cross-platform GPU via wgpu
//! - Single-binary deployment with training

use super::{Engine, EngineError, EnginePrediction, EngineType};
use crate::inference::tokenizer::{extract_numbers, FastTokenizer};
use burn::prelude::*;
use burn::record::{CompactRecorder, Recorder};
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

// Use wgpu backend for GPU support
type Backend = burn::backend::Wgpu;

// Re-use the model definition from the train module to ensure compatibility
#[cfg(feature = "train")]
use crate::train::{MultiHeadModel, MultiHeadModelConfig};

// For builds without the train feature, define a minimal model structure
#[cfg(not(feature = "train"))]
mod model {
    use burn::nn::{conv::Conv1d, BatchNorm, Dropout, Embedding, Linear};
    use burn::prelude::*;

    /// Multi-head prediction model (must match training model exactly)
    #[derive(Module, Debug)]
    pub struct MultiHeadModel<B: Backend> {
        // Shared encoder
        pub embedding: Embedding<B>,
        pub conv1: Conv1d<B>,
        pub bn1: BatchNorm<B>,
        pub conv2: Conv1d<B>,
        pub bn2: BatchNorm<B>,
        pub conv3: Conv1d<B>,

        // Intent head
        pub intent_fc1: Linear<B>,
        pub intent_dropout: Dropout,
        pub intent_fc2: Linear<B>,

        // Count head
        pub count_fc1: Linear<B>,
        pub count_fc2: Linear<B>,

        // Operand heads (4 heads)
        pub op0_fc1: Linear<B>,
        pub op0_dropout: Dropout,
        pub op0_fc2: Linear<B>,
        pub op1_fc1: Linear<B>,
        pub op1_dropout: Dropout,
        pub op1_fc2: Linear<B>,
        pub op2_fc1: Linear<B>,
        pub op2_dropout: Dropout,
        pub op2_fc2: Linear<B>,
        pub op3_fc1: Linear<B>,
        pub op3_dropout: Dropout,
        pub op3_fc2: Linear<B>,

        // Sign heads (4 heads)
        pub sign0: Linear<B>,
        pub sign1: Linear<B>,
        pub sign2: Linear<B>,
        pub sign3: Linear<B>,
    }

    impl<B: Backend> MultiHeadModel<B> {
        /// Forward pass
        pub fn forward(&self, tokens: Tensor<B, 2, Int>) -> MultiHeadOutput<B> {
            // Embed: (batch, seq) → (batch, seq, embed_dim)
            let x = self.embedding.forward(tokens);

            // Transpose for Conv1d: (batch, embed_dim, seq)
            let x = x.swap_dims(1, 2);

            // CNN encoder with BatchNorm and ReLU
            let x = self.conv1.forward(x);
            let x = self.bn1.forward(x);
            let x = burn::tensor::activation::relu(x);

            let x = self.conv2.forward(x);
            let x = self.bn2.forward(x);
            let x = burn::tensor::activation::relu(x);

            let x = self.conv3.forward(x);
            let x = burn::tensor::activation::relu(x);

            // Global max pooling: (batch, hidden_dim, seq) → (batch, hidden_dim)
            let features = x.max_dim(2).squeeze::<2>();

            // Intent head
            let intent = self.intent_fc1.forward(features.clone());
            let intent = burn::tensor::activation::relu(intent);
            let intent = self.intent_dropout.forward(intent);
            let intent = self.intent_fc2.forward(intent);

            // Count head
            let count = self.count_fc1.forward(features.clone());
            let count = burn::tensor::activation::relu(count);
            let count = self.count_fc2.forward(count);

            MultiHeadOutput { intent, count }
        }
    }

    /// Output from multi-head model
    #[derive(Debug)]
    pub struct MultiHeadOutput<B: Backend> {
        pub intent: Tensor<B, 2>,
        pub count: Tensor<B, 2>,
    }

    /// Model configuration
    #[derive(burn::config::Config, Debug)]
    pub struct MultiHeadModelConfig {
        #[config(default = 261)]
        pub vocab_size: usize,
        #[config(default = 64)]
        pub embed_dim: usize,
        #[config(default = 54)]
        pub num_intents: usize,
        #[config(default = 256)]
        pub hidden_dim: usize,
        #[config(default = 4)]
        pub max_operands: usize,
        #[config(default = 256)]
        pub operand_bins: usize,
        #[config(default = 0.2)]
        pub dropout: f64,
    }

    impl MultiHeadModelConfig {
        pub fn init<B: Backend>(&self, device: &B::Device) -> MultiHeadModel<B> {
            use burn::nn::{
                conv::Conv1dConfig, BatchNormConfig, DropoutConfig, EmbeddingConfig, LinearConfig,
            };

            let dropout_config = DropoutConfig::new(self.dropout);

            MultiHeadModel {
                embedding: EmbeddingConfig::new(self.vocab_size, self.embed_dim).init(device),

                conv1: Conv1dConfig::new(self.embed_dim, 64, 3)
                    .with_padding(burn::nn::PaddingConfig1d::Same)
                    .init(device),
                bn1: BatchNormConfig::new(64).init(device),

                conv2: Conv1dConfig::new(64, 128, 3)
                    .with_padding(burn::nn::PaddingConfig1d::Same)
                    .init(device),
                bn2: BatchNormConfig::new(128).init(device),

                conv3: Conv1dConfig::new(128, self.hidden_dim, 3)
                    .with_padding(burn::nn::PaddingConfig1d::Same)
                    .init(device),

                intent_fc1: LinearConfig::new(self.hidden_dim, 128).init(device),
                intent_dropout: dropout_config.init(),
                intent_fc2: LinearConfig::new(128, self.num_intents).init(device),

                count_fc1: LinearConfig::new(self.hidden_dim, 64).init(device),
                count_fc2: LinearConfig::new(64, 5).init(device),

                op0_fc1: LinearConfig::new(self.hidden_dim, 128).init(device),
                op0_dropout: dropout_config.init(),
                op0_fc2: LinearConfig::new(128, self.operand_bins).init(device),

                op1_fc1: LinearConfig::new(self.hidden_dim, 128).init(device),
                op1_dropout: dropout_config.init(),
                op1_fc2: LinearConfig::new(128, self.operand_bins).init(device),

                op2_fc1: LinearConfig::new(self.hidden_dim, 128).init(device),
                op2_dropout: dropout_config.init(),
                op2_fc2: LinearConfig::new(128, self.operand_bins).init(device),

                op3_fc1: LinearConfig::new(self.hidden_dim, 128).init(device),
                op3_dropout: dropout_config.init(),
                op3_fc2: LinearConfig::new(128, self.operand_bins).init(device),

                sign0: LinearConfig::new(self.hidden_dim, 2).init(device),
                sign1: LinearConfig::new(self.hidden_dim, 2).init(device),
                sign2: LinearConfig::new(self.hidden_dim, 2).init(device),
                sign3: LinearConfig::new(self.hidden_dim, 2).init(device),
            }
        }
    }
}

#[cfg(not(feature = "train"))]
use model::{MultiHeadModel, MultiHeadModelConfig};

/// Burn inference engine
///
/// Model is wrapped in Mutex because burn's Param types use OnceCell
/// which is not Sync. The Engine trait requires Send + Sync.
pub struct BurnEngine {
    model: Mutex<MultiHeadModel<Backend>>,
    device: burn::backend::wgpu::WgpuDevice,
    tokenizer: FastTokenizer,
}

impl BurnEngine {
    /// Load a model from a .mpk file
    pub fn load(model_path: &Path) -> Result<Self, EngineError> {
        // Initialize device
        let device = burn::backend::wgpu::WgpuDevice::default();

        // Load model config and weights
        let record = CompactRecorder::new()
            .load(model_path.into(), &device)
            .map_err(|e| EngineError::LoadError(format!("Failed to load model: {}", e)))?;

        // Initialize model with config and load weights
        let model = MultiHeadModelConfig::new()
            .init::<Backend>(&device)
            .load_record(record);

        Ok(Self {
            model: Mutex::new(model),
            device,
            tokenizer: FastTokenizer::new(),
        })
    }

    /// Run inference
    fn run_inference(&self, tokens: Vec<i64>) -> Result<(usize, f32, usize), EngineError> {
        // Create input tensor
        let tokens_i32: Vec<i32> = tokens.iter().map(|&t| t as i32).collect();
        let input = Tensor::<Backend, 1, Int>::from_data(tokens_i32.as_slice(), &self.device)
            .unsqueeze::<2>(); // Add batch dimension

        // Lock the model for inference
        let model = self
            .model
            .lock()
            .map_err(|e| EngineError::InferenceError(format!("Failed to lock model: {}", e)))?;

        // Forward pass
        let output = model.forward(input);

        // Extract intent prediction
        let intent_data: Vec<f32> = output.intent.squeeze::<1>().into_data().to_vec().unwrap();
        let intent_id = argmax(&intent_data);
        let confidence = softmax_max(&intent_data);

        // Extract count prediction
        let count_data: Vec<f32> = output.count.squeeze::<1>().into_data().to_vec().unwrap();
        let operand_count = argmax(&count_data).min(4);

        Ok((intent_id, confidence, operand_count))
    }
}

impl Engine for BurnEngine {
    fn engine_type(&self) -> EngineType {
        EngineType::Burn
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
            engine: EngineType::Burn,
        })
    }

    fn is_ready(&self) -> bool {
        true
    }

    fn model_info(&self) -> String {
        "Burn Engine (wgpu backend)".to_string()
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

    #[test]
    fn test_config() {
        let config = MultiHeadModelConfig::new();
        assert_eq!(config.vocab_size, 261);
        assert_eq!(config.num_intents, 54);
    }
}
