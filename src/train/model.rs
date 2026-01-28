//! Parallel Instruction Prediction Model for Neurlang
//!
//! Native Rust implementation using the burn framework.
//! Predicts up to 64 instructions in a single forward pass.
//!
//! Architecture:
//!     ┌─────────────────────────────────────┐
//!     │      Encoder (CNN + Positional)     │
//!     │   Input: tokens (batch, seq_len)    │
//!     │   Output: features (batch, hidden, seq)│
//!     └───────────────┬─────────────────────┘
//!                     │
//!     ┌───────────────┴─────────────────────┐
//!     │      Slot Decoder (Cross-Attention) │
//!     │   Slot queries: (64, hidden)        │
//!     │   Output: slot_features (batch, 64, hidden)│
//!     └───────────────┬─────────────────────┘
//!                     │
//!         ┌───────────┼───────────┬───────────┐
//!         ▼           ▼           ▼           ▼
//!     ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
//!     │ Valid   │ │ Opcode  │ │ Regs    │ │ Imm     │
//!     │ Head    │ │ Head    │ │ Heads   │ │ Heads   │
//!     └─────────┘ └─────────┘ └─────────┘ └─────────┘
//!         │           │           │           │
//!         ▼           ▼           ▼           ▼
//!     2 classes   33 classes  32×3 classes  256 bins

use burn::module::Param;
use burn::nn::{
    conv::{Conv1d, Conv1dConfig},
    BatchNorm, BatchNormConfig, Dropout, DropoutConfig, Embedding, EmbeddingConfig, Linear,
    LinearConfig,
};
use burn::prelude::*;

/// Number of instruction slots (max instructions per generation)
pub const NUM_SLOTS: usize = 64;

/// Number of opcodes (0x00-0x20 = 33)
pub const NUM_OPCODES: usize = 33;

/// Number of registers (0-31)
pub const NUM_REGISTERS: usize = 32;

/// Number of mode bits (0-7)
pub const NUM_MODES: usize = 8;

/// Number of immediate bins (quantized)
pub const NUM_IMM_BINS: usize = 256;

/// Model configuration
#[derive(Config, Debug)]
pub struct ParallelModelConfig {
    /// Vocabulary size (256 bytes + 5 special tokens)
    #[config(default = 261)]
    pub vocab_size: usize,
    /// Embedding dimension
    #[config(default = 128)]
    pub embed_dim: usize,
    /// Hidden dimension (CNN output / slot features)
    #[config(default = 512)]
    pub hidden_dim: usize,
    /// Number of instruction slots
    #[config(default = 64)]
    pub num_slots: usize,
    /// Maximum sequence length
    #[config(default = 128)]
    pub max_seq_len: usize,
    /// Number of attention heads
    #[config(default = 8)]
    pub num_heads: usize,
    /// Dropout rate
    #[config(default = 0.1)]
    pub dropout: f64,
}

/// Parallel instruction prediction model
#[derive(Module, Debug)]
pub struct ParallelInstructionModel<B: Backend> {
    // Encoder
    embedding: Embedding<B>,
    pos_encoding: Param<Tensor<B, 2>>, // (max_seq, embed_dim)
    conv1: Conv1d<B>,
    bn1: BatchNorm<B>,
    conv2: Conv1d<B>,
    bn2: BatchNorm<B>,
    conv3: Conv1d<B>,
    encoder_proj: Linear<B>, // Project to hidden_dim

    // Slot decoder
    slot_queries: Param<Tensor<B, 2>>, // (num_slots, hidden_dim)
    query_proj: Linear<B>,
    key_proj: Linear<B>,
    value_proj: Linear<B>,
    output_proj: Linear<B>,

    // Prediction heads (per slot)
    valid_head: Linear<B>,   // 2 classes (valid/padding)
    opcode_head: Linear<B>,  // 33 classes
    mode_head: Linear<B>,    // 8 classes
    rd_head: Linear<B>,      // 32 classes
    rs1_head: Linear<B>,     // 32 classes
    rs2_head: Linear<B>,     // 32 classes
    has_imm_head: Linear<B>, // 2 classes
    imm_head: Linear<B>,     // 256 bins

    dropout: Dropout,

    // Config
    num_heads: usize,
    head_dim: usize,
}

impl ParallelModelConfig {
    /// Initialize the model
    pub fn init<B: Backend>(&self, device: &B::Device) -> ParallelInstructionModel<B> {
        let head_dim = self.hidden_dim / self.num_heads;
        let dropout_config = DropoutConfig::new(self.dropout);

        // Create positional encoding (sinusoidal)
        let pos_encoding =
            Self::create_positional_encoding(self.max_seq_len, self.embed_dim, device);

        // Create learned slot queries
        let slot_queries = Tensor::random(
            [self.num_slots, self.hidden_dim],
            burn::tensor::Distribution::Normal(0.0, 0.02),
            device,
        );

        ParallelInstructionModel {
            // Encoder
            embedding: EmbeddingConfig::new(self.vocab_size, self.embed_dim).init(device),
            pos_encoding: Param::from_tensor(pos_encoding),

            conv1: Conv1dConfig::new(self.embed_dim, 256, 3)
                .with_padding(burn::nn::PaddingConfig1d::Same)
                .init(device),
            bn1: BatchNormConfig::new(256).init(device),

            conv2: Conv1dConfig::new(256, 384, 3)
                .with_padding(burn::nn::PaddingConfig1d::Same)
                .init(device),
            bn2: BatchNormConfig::new(384).init(device),

            conv3: Conv1dConfig::new(384, self.hidden_dim, 3)
                .with_padding(burn::nn::PaddingConfig1d::Same)
                .init(device),
            encoder_proj: LinearConfig::new(self.hidden_dim, self.hidden_dim).init(device),

            // Slot decoder (cross-attention)
            slot_queries: Param::from_tensor(slot_queries),
            query_proj: LinearConfig::new(self.hidden_dim, self.hidden_dim).init(device),
            key_proj: LinearConfig::new(self.hidden_dim, self.hidden_dim).init(device),
            value_proj: LinearConfig::new(self.hidden_dim, self.hidden_dim).init(device),
            output_proj: LinearConfig::new(self.hidden_dim, self.hidden_dim).init(device),

            // Prediction heads
            valid_head: LinearConfig::new(self.hidden_dim, 2).init(device),
            opcode_head: LinearConfig::new(self.hidden_dim, NUM_OPCODES).init(device),
            mode_head: LinearConfig::new(self.hidden_dim, NUM_MODES).init(device),
            rd_head: LinearConfig::new(self.hidden_dim, NUM_REGISTERS).init(device),
            rs1_head: LinearConfig::new(self.hidden_dim, NUM_REGISTERS).init(device),
            rs2_head: LinearConfig::new(self.hidden_dim, NUM_REGISTERS).init(device),
            has_imm_head: LinearConfig::new(self.hidden_dim, 2).init(device),
            imm_head: LinearConfig::new(self.hidden_dim, NUM_IMM_BINS).init(device),

            dropout: dropout_config.init(),

            num_heads: self.num_heads,
            head_dim,
        }
    }

    /// Create sinusoidal positional encoding
    fn create_positional_encoding<B: Backend>(
        max_len: usize,
        dim: usize,
        device: &B::Device,
    ) -> Tensor<B, 2> {
        let mut pe = vec![0.0f32; max_len * dim];

        for pos in 0..max_len {
            for i in 0..dim {
                let angle = (pos as f32) / 10000.0f32.powf((2 * (i / 2)) as f32 / dim as f32);
                pe[pos * dim + i] = if i % 2 == 0 { angle.sin() } else { angle.cos() };
            }
        }

        Tensor::<B, 1>::from_floats(pe.as_slice(), device).reshape([max_len, dim])
    }
}

impl<B: Backend> ParallelInstructionModel<B> {
    /// Forward pass
    ///
    /// Args:
    ///     tokens: Input token IDs (batch, seq_len)
    ///
    /// Returns:
    ///     ParallelOutput with predictions for all 64 slots
    pub fn forward(&self, tokens: Tensor<B, 2, Int>) -> ParallelOutput<B> {
        let [batch_size, seq_len] = tokens.dims();

        // === Encoder ===
        // Embed: (batch, seq) -> (batch, seq, embed_dim)
        let x = self.embedding.forward(tokens);

        // Add positional encoding
        let pos = self
            .pos_encoding
            .val()
            .clone()
            .slice([0..seq_len])
            .unsqueeze::<3>(); // (1, seq, embed_dim)
        let x = x + pos;

        // Transpose for Conv1d: (batch, embed_dim, seq)
        let x = x.swap_dims(1, 2);

        // CNN encoder with BatchNorm and ReLU
        let x = self.conv1.forward(x);
        let x = self.bn1.forward(x);
        let x = burn::tensor::activation::relu(x);
        let x = self.dropout.forward(x);

        let x = self.conv2.forward(x);
        let x = self.bn2.forward(x);
        let x = burn::tensor::activation::relu(x);
        let x = self.dropout.forward(x);

        let x = self.conv3.forward(x);
        let x = burn::tensor::activation::relu(x);

        // (batch, hidden_dim, seq) -> (batch, seq, hidden_dim)
        let encoder_output = x.swap_dims(1, 2);
        let encoder_output = self.encoder_proj.forward(encoder_output);

        // === Slot Decoder (Cross-Attention) ===
        // slot_queries: (num_slots, hidden_dim) -> (batch, num_slots, hidden_dim)
        let queries = self
            .slot_queries
            .val()
            .clone()
            .unsqueeze::<3>()
            .repeat_dim(0, batch_size);

        // Project Q, K, V
        let q = self.query_proj.forward(queries); // (batch, num_slots, hidden)
        let k = self.key_proj.forward(encoder_output.clone()); // (batch, seq, hidden)
        let v = self.value_proj.forward(encoder_output); // (batch, seq, hidden)

        // Multi-head attention
        let slot_features = self.multi_head_attention(q, k, v, batch_size);
        let slot_features = self.output_proj.forward(slot_features);
        let slot_features = self.dropout.forward(slot_features);

        // === Prediction Heads ===
        // All heads operate on (batch, num_slots, hidden) -> (batch, num_slots, classes)
        let valid = self.valid_head.forward(slot_features.clone());
        let opcode = self.opcode_head.forward(slot_features.clone());
        let mode = self.mode_head.forward(slot_features.clone());
        let rd = self.rd_head.forward(slot_features.clone());
        let rs1 = self.rs1_head.forward(slot_features.clone());
        let rs2 = self.rs2_head.forward(slot_features.clone());
        let has_imm = self.has_imm_head.forward(slot_features.clone());
        let imm = self.imm_head.forward(slot_features);

        ParallelOutput {
            valid,   // (batch, 64, 2)
            opcode,  // (batch, 64, 33)
            mode,    // (batch, 64, 8)
            rd,      // (batch, 64, 32)
            rs1,     // (batch, 64, 32)
            rs2,     // (batch, 64, 32)
            has_imm, // (batch, 64, 2)
            imm,     // (batch, 64, 256)
        }
    }

    /// Multi-head cross-attention
    fn multi_head_attention(
        &self,
        q: Tensor<B, 3>, // (batch, num_slots, hidden)
        k: Tensor<B, 3>, // (batch, seq, hidden)
        v: Tensor<B, 3>, // (batch, seq, hidden)
        batch_size: usize,
    ) -> Tensor<B, 3> {
        let num_slots = NUM_SLOTS;
        let seq_len = k.dims()[1];

        // Reshape for multi-head: (batch, heads, slots/seq, head_dim)
        let q = q
            .reshape([batch_size, num_slots, self.num_heads, self.head_dim])
            .swap_dims(1, 2); // (batch, heads, slots, head_dim)
        let k = k
            .reshape([batch_size, seq_len, self.num_heads, self.head_dim])
            .swap_dims(1, 2); // (batch, heads, seq, head_dim)
        let v = v
            .reshape([batch_size, seq_len, self.num_heads, self.head_dim])
            .swap_dims(1, 2); // (batch, heads, seq, head_dim)

        // Attention scores: (batch, heads, slots, seq)
        let scale = (self.head_dim as f32).sqrt();
        let k_t = k.swap_dims(2, 3); // (batch, heads, head_dim, seq)
        let scores = q.matmul(k_t) / scale;

        // Softmax over seq dimension
        let attn_weights = burn::tensor::activation::softmax(scores, 3);

        // Apply attention: (batch, heads, slots, head_dim)
        let context = attn_weights.matmul(v);

        // Reshape back: (batch, slots, hidden)
        context
            .swap_dims(1, 2) // (batch, slots, heads, head_dim)
            .reshape([batch_size, num_slots, self.num_heads * self.head_dim])
    }

    /// Get predictions as class indices
    pub fn predict(&self, tokens: Tensor<B, 2, Int>) -> ParallelPrediction<B> {
        let output = self.forward(tokens);

        ParallelPrediction {
            valid: output.valid.argmax(2).squeeze(),     // (batch, 64)
            opcode: output.opcode.argmax(2).squeeze(),   // (batch, 64)
            mode: output.mode.argmax(2).squeeze(),       // (batch, 64)
            rd: output.rd.argmax(2).squeeze(),           // (batch, 64)
            rs1: output.rs1.argmax(2).squeeze(),         // (batch, 64)
            rs2: output.rs2.argmax(2).squeeze(),         // (batch, 64)
            has_imm: output.has_imm.argmax(2).squeeze(), // (batch, 64)
            imm: output.imm.argmax(2).squeeze(),         // (batch, 64)
        }
    }
}

/// Output from parallel model (logits)
#[derive(Debug)]
pub struct ParallelOutput<B: Backend> {
    /// Valid/padding logits (batch, 64, 2)
    pub valid: Tensor<B, 3>,
    /// Opcode logits (batch, 64, 33)
    pub opcode: Tensor<B, 3>,
    /// Mode logits (batch, 64, 8)
    pub mode: Tensor<B, 3>,
    /// Destination register logits (batch, 64, 32)
    pub rd: Tensor<B, 3>,
    /// Source register 1 logits (batch, 64, 32)
    pub rs1: Tensor<B, 3>,
    /// Source register 2 logits (batch, 64, 32)
    pub rs2: Tensor<B, 3>,
    /// Has immediate logits (batch, 64, 2)
    pub has_imm: Tensor<B, 3>,
    /// Immediate value logits (batch, 64, 256)
    pub imm: Tensor<B, 3>,
}

/// Predictions from parallel model (class indices)
#[derive(Debug)]
pub struct ParallelPrediction<B: Backend> {
    /// Valid flags (batch, 64)
    pub valid: Tensor<B, 2, Int>,
    /// Opcode indices (batch, 64)
    pub opcode: Tensor<B, 2, Int>,
    /// Mode indices (batch, 64)
    pub mode: Tensor<B, 2, Int>,
    /// Destination register indices (batch, 64)
    pub rd: Tensor<B, 2, Int>,
    /// Source register 1 indices (batch, 64)
    pub rs1: Tensor<B, 2, Int>,
    /// Source register 2 indices (batch, 64)
    pub rs2: Tensor<B, 2, Int>,
    /// Has immediate flags (batch, 64)
    pub has_imm: Tensor<B, 2, Int>,
    /// Immediate bin indices (batch, 64)
    pub imm: Tensor<B, 2, Int>,
}

// ============================================================================
// Legacy Multi-Head Model (kept for backwards compatibility)
// ============================================================================

/// Legacy model configuration
#[derive(Config, Debug)]
pub struct MultiHeadModelConfig {
    /// Vocabulary size (256 bytes + 5 special tokens)
    #[config(default = 261)]
    pub vocab_size: usize,
    /// Embedding dimension
    #[config(default = 64)]
    pub embed_dim: usize,
    /// Number of intent classes
    #[config(default = 54)]
    pub num_intents: usize,
    /// Hidden dimension (CNN output)
    #[config(default = 256)]
    pub hidden_dim: usize,
    /// Maximum number of operands
    #[config(default = 4)]
    pub max_operands: usize,
    /// Number of bins for operand classification
    #[config(default = 256)]
    pub operand_bins: usize,
    /// Dropout rate
    #[config(default = 0.2)]
    pub dropout: f64,
}

/// Legacy multi-head prediction model
#[derive(Module, Debug)]
pub struct MultiHeadModel<B: Backend> {
    // Shared encoder
    embedding: Embedding<B>,
    conv1: Conv1d<B>,
    bn1: BatchNorm<B>,
    conv2: Conv1d<B>,
    bn2: BatchNorm<B>,
    conv3: Conv1d<B>,

    // Intent head (hidden_dim -> 128 -> num_intents)
    intent_fc1: Linear<B>,
    intent_dropout: Dropout,
    intent_fc2: Linear<B>,

    // Count head (hidden_dim -> 64 -> 5)
    count_fc1: Linear<B>,
    count_fc2: Linear<B>,

    // Operand heads (4 x hidden_dim -> 128 -> operand_bins)
    op0_fc1: Linear<B>,
    op0_dropout: Dropout,
    op0_fc2: Linear<B>,
    op1_fc1: Linear<B>,
    op1_dropout: Dropout,
    op1_fc2: Linear<B>,
    op2_fc1: Linear<B>,
    op2_dropout: Dropout,
    op2_fc2: Linear<B>,
    op3_fc1: Linear<B>,
    op3_dropout: Dropout,
    op3_fc2: Linear<B>,

    // Sign heads (4 x hidden_dim -> 2)
    sign0: Linear<B>,
    sign1: Linear<B>,
    sign2: Linear<B>,
    sign3: Linear<B>,
}

impl MultiHeadModelConfig {
    /// Initialize the model
    pub fn init<B: Backend>(&self, device: &B::Device) -> MultiHeadModel<B> {
        let dropout_config = DropoutConfig::new(self.dropout);

        MultiHeadModel {
            // Embedding layer
            embedding: EmbeddingConfig::new(self.vocab_size, self.embed_dim).init(device),

            // CNN encoder
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

            // Intent head
            intent_fc1: LinearConfig::new(self.hidden_dim, 128).init(device),
            intent_dropout: dropout_config.init(),
            intent_fc2: LinearConfig::new(128, self.num_intents).init(device),

            // Count head
            count_fc1: LinearConfig::new(self.hidden_dim, 64).init(device),
            count_fc2: LinearConfig::new(64, 5).init(device),

            // Operand heads (4 heads)
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

            // Sign heads (4 heads)
            sign0: LinearConfig::new(self.hidden_dim, 2).init(device),
            sign1: LinearConfig::new(self.hidden_dim, 2).init(device),
            sign2: LinearConfig::new(self.hidden_dim, 2).init(device),
            sign3: LinearConfig::new(self.hidden_dim, 2).init(device),
        }
    }
}

impl<B: Backend> MultiHeadModel<B> {
    /// Forward pass
    pub fn forward(&self, tokens: Tensor<B, 2, Int>) -> MultiHeadOutput<B> {
        // Embed: (batch, seq) -> (batch, seq, embed_dim)
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

        // Global max pooling: (batch, hidden_dim, seq) -> (batch, hidden_dim)
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

        // Operand heads
        let op0 =
            self.forward_operand_head(&features, &self.op0_fc1, &self.op0_dropout, &self.op0_fc2);
        let op1 =
            self.forward_operand_head(&features, &self.op1_fc1, &self.op1_dropout, &self.op1_fc2);
        let op2 =
            self.forward_operand_head(&features, &self.op2_fc1, &self.op2_dropout, &self.op2_fc2);
        let op3 =
            self.forward_operand_head(&features, &self.op3_fc1, &self.op3_dropout, &self.op3_fc2);

        // Sign heads
        let sign0 = self.sign0.forward(features.clone());
        let sign1 = self.sign1.forward(features.clone());
        let sign2 = self.sign2.forward(features.clone());
        let sign3 = self.sign3.forward(features);

        MultiHeadOutput {
            intent,
            count,
            operands: vec![op0, op1, op2, op3],
            signs: vec![sign0, sign1, sign2, sign3],
        }
    }

    /// Forward pass for a single operand head
    fn forward_operand_head(
        &self,
        features: &Tensor<B, 2>,
        fc1: &Linear<B>,
        dropout: &Dropout,
        fc2: &Linear<B>,
    ) -> Tensor<B, 2> {
        let x = fc1.forward(features.clone());
        let x = burn::tensor::activation::relu(x);
        let x = dropout.forward(x);
        fc2.forward(x)
    }

    /// Get predictions as class indices
    pub fn predict(&self, tokens: Tensor<B, 2, Int>) -> MultiHeadPrediction<B> {
        let output = self.forward(tokens);

        // argmax returns 2D tensor, squeeze to 1D
        let intent = output.intent.argmax(1).squeeze::<1>();
        let count = output.count.argmax(1).squeeze::<1>();

        let operands: Vec<_> = output
            .operands
            .iter()
            .map(|op| op.clone().argmax(1).squeeze::<1>())
            .collect();

        let signs: Vec<_> = output
            .signs
            .iter()
            .map(|sign| sign.clone().argmax(1).squeeze::<1>())
            .collect();

        MultiHeadPrediction {
            intent,
            count,
            operands,
            signs,
        }
    }
}

/// Output from multi-head model (logits)
#[derive(Debug)]
pub struct MultiHeadOutput<B: Backend> {
    /// Intent logits (batch, num_intents)
    pub intent: Tensor<B, 2>,
    /// Count logits (batch, 5)
    pub count: Tensor<B, 2>,
    /// Operand logits (4 x (batch, operand_bins))
    pub operands: Vec<Tensor<B, 2>>,
    /// Sign logits (4 x (batch, 2))
    pub signs: Vec<Tensor<B, 2>>,
}

/// Predictions from multi-head model (class indices)
#[derive(Debug)]
pub struct MultiHeadPrediction<B: Backend> {
    /// Predicted intent IDs (batch,)
    pub intent: Tensor<B, 1, Int>,
    /// Predicted operand counts (batch,)
    pub count: Tensor<B, 1, Int>,
    /// Predicted operand values (4 x (batch,))
    pub operands: Vec<Tensor<B, 1, Int>>,
    /// Predicted signs (4 x (batch,))
    pub signs: Vec<Tensor<B, 1, Int>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::backend::NdArray;

    type TestBackend = NdArray;

    #[test]
    fn test_parallel_model_config() {
        let config = ParallelModelConfig::new();
        assert_eq!(config.vocab_size, 261);
        assert_eq!(config.hidden_dim, 512);
        assert_eq!(config.num_slots, 64);
    }

    #[test]
    fn test_parallel_model_forward() {
        let device = Default::default();
        let model: ParallelInstructionModel<TestBackend> = ParallelModelConfig::new().init(&device);

        // Create dummy input
        let batch_size = 2;
        let seq_len = 128;
        let tokens = Tensor::<TestBackend, 2, Int>::zeros([batch_size, seq_len], &device);

        let output = model.forward(tokens);

        // Check output shapes
        assert_eq!(output.valid.dims(), [batch_size, 64, 2]);
        assert_eq!(output.opcode.dims(), [batch_size, 64, NUM_OPCODES]);
        assert_eq!(output.mode.dims(), [batch_size, 64, NUM_MODES]);
        assert_eq!(output.rd.dims(), [batch_size, 64, NUM_REGISTERS]);
        assert_eq!(output.rs1.dims(), [batch_size, 64, NUM_REGISTERS]);
        assert_eq!(output.rs2.dims(), [batch_size, 64, NUM_REGISTERS]);
        assert_eq!(output.has_imm.dims(), [batch_size, 64, 2]);
        assert_eq!(output.imm.dims(), [batch_size, 64, NUM_IMM_BINS]);
    }

    #[test]
    fn test_parallel_model_predict() {
        let device = Default::default();
        let model: ParallelInstructionModel<TestBackend> = ParallelModelConfig::new().init(&device);

        let batch_size = 2;
        let seq_len = 128;
        let tokens = Tensor::<TestBackend, 2, Int>::zeros([batch_size, seq_len], &device);

        let pred = model.predict(tokens);

        // Check prediction shapes
        assert_eq!(pred.valid.dims(), [batch_size, 64]);
        assert_eq!(pred.opcode.dims(), [batch_size, 64]);
        assert_eq!(pred.mode.dims(), [batch_size, 64]);
        assert_eq!(pred.rd.dims(), [batch_size, 64]);
        assert_eq!(pred.rs1.dims(), [batch_size, 64]);
        assert_eq!(pred.rs2.dims(), [batch_size, 64]);
        assert_eq!(pred.has_imm.dims(), [batch_size, 64]);
        assert_eq!(pred.imm.dims(), [batch_size, 64]);
    }

    #[test]
    fn test_legacy_model_config() {
        let config = MultiHeadModelConfig::new();
        assert_eq!(config.vocab_size, 261);
        assert_eq!(config.num_intents, 54);
        assert_eq!(config.hidden_dim, 256);
    }

    #[test]
    fn test_legacy_model_forward() {
        let device = Default::default();
        let model: MultiHeadModel<TestBackend> = MultiHeadModelConfig::new().init(&device);

        // Create dummy input
        let batch_size = 2;
        let seq_len = 128;
        let tokens = Tensor::<TestBackend, 2, Int>::zeros([batch_size, seq_len], &device);

        let output = model.forward(tokens);

        // Check output shapes
        assert_eq!(output.intent.dims(), [batch_size, 54]);
        assert_eq!(output.count.dims(), [batch_size, 5]);
        assert_eq!(output.operands.len(), 4);
        assert_eq!(output.signs.len(), 4);

        for op in &output.operands {
            assert_eq!(op.dims(), [batch_size, 256]);
        }

        for sign in &output.signs {
            assert_eq!(sign.dims(), [batch_size, 2]);
        }
    }
}
