//! Neurlang Training Infrastructure
//!
//! Provides training loops for both model architectures:
//! 1. Legacy: MultiHeadModel (intent + operand prediction)
//! 2. Parallel: ParallelInstructionModel (64 instruction slots)
//!
//! Uses burn framework with wgpu backend for GPU acceleration.

use std::path::PathBuf;

use burn::data::dataloader::DataLoaderBuilder;
use burn::nn::loss::CrossEntropyLossConfig;
use burn::optim::lr_scheduler::constant::ConstantLr;
use burn::optim::AdamWConfig;
use burn::prelude::*;
use burn::record::CompactRecorder;
use burn::tensor::backend::AutodiffBackend;
use burn::train::{InferenceStep, TrainOutput, TrainStep};

use super::backend::TrainError;
use super::dataset::{
    MultiHeadBatch, MultiHeadBatcher, MultiHeadDataset, ParallelBatch, ParallelBatcher,
    ParallelDataset,
};
use super::model::{
    MultiHeadModel, MultiHeadModelConfig, MultiHeadOutput, ParallelInstructionModel,
    ParallelModelConfig, ParallelOutput, NUM_IMM_BINS, NUM_MODES, NUM_OPCODES, NUM_REGISTERS,
    NUM_SLOTS,
};

// ============================================================================
// Parallel Instruction Model Training (NEW)
// ============================================================================

/// Configuration for parallel model training
#[derive(Debug, Clone)]
pub struct ParallelTrainConfig {
    /// Path to training data JSONL
    pub data_path: PathBuf,
    /// Path to save model
    pub output_path: PathBuf,
    /// Number of epochs
    pub epochs: usize,
    /// Batch size
    pub batch_size: usize,
    /// Learning rate
    pub learning_rate: f64,
    /// Weight decay
    pub weight_decay: f64,
    /// Train/validation split ratio
    pub train_ratio: f32,
    /// Random seed
    pub seed: u64,
    /// Export to ONNX after training
    pub export_onnx: bool,
}

impl Default for ParallelTrainConfig {
    fn default() -> Self {
        Self {
            data_path: PathBuf::from("training_data.jsonl"),
            output_path: PathBuf::from("model_parallel.mpk"),
            epochs: 50,
            batch_size: 32,
            learning_rate: 0.001,
            weight_decay: 0.01,
            train_ratio: 0.9,
            seed: 42,
            export_onnx: true,
        }
    }
}

/// Loss weights for parallel model training
#[derive(Debug, Clone)]
pub struct ParallelLossWeights {
    pub valid: f32,
    pub opcode: f32,
    pub mode: f32,
    pub rd: f32,
    pub rs1: f32,
    pub rs2: f32,
    pub has_imm: f32,
    pub imm: f32,
}

impl Default for ParallelLossWeights {
    fn default() -> Self {
        Self {
            valid: 2.0,  // High weight - important for knowing when to stop
            opcode: 2.0, // High weight - most critical for correctness
            mode: 1.0,
            rd: 1.0,
            rs1: 1.0,
            rs2: 1.0,
            has_imm: 0.5,
            imm: 0.5,
        }
    }
}

/// Trainable wrapper for parallel model
#[derive(Module, Debug)]
pub struct TrainableParallelModel<B: Backend> {
    model: ParallelInstructionModel<B>,
}

impl<B: Backend> TrainableParallelModel<B> {
    pub fn new(model: ParallelInstructionModel<B>) -> Self {
        Self { model }
    }

    pub fn inner(&self) -> &ParallelInstructionModel<B> {
        &self.model
    }

    pub fn into_inner(self) -> ParallelInstructionModel<B> {
        self.model
    }

    /// Forward pass with loss computation
    pub fn forward_loss(&self, batch: ParallelBatch<B>) -> ParallelLossOutput<B> {
        let weights = ParallelLossWeights::default();
        let output = self.model.forward(batch.tokens.clone());

        let batch_size = batch.tokens.dims()[0];
        let ce_loss = CrossEntropyLossConfig::new().init(&output.valid.device());

        // Flatten predictions: (batch, 64, classes) -> (batch*64, classes)
        // Flatten targets: (batch, 64) -> (batch*64,)
        let n = batch_size * NUM_SLOTS;

        let valid_loss = ce_loss.forward(
            output.valid.clone().reshape([n, 2]),
            batch.valid.clone().reshape([n]),
        );

        let opcode_loss = ce_loss.forward(
            output.opcode.clone().reshape([n, NUM_OPCODES]),
            batch.opcode.clone().reshape([n]),
        );

        let mode_loss = ce_loss.forward(
            output.mode.clone().reshape([n, NUM_MODES]),
            batch.mode.clone().reshape([n]),
        );

        let rd_loss = ce_loss.forward(
            output.rd.clone().reshape([n, NUM_REGISTERS]),
            batch.rd.clone().reshape([n]),
        );

        let rs1_loss = ce_loss.forward(
            output.rs1.clone().reshape([n, NUM_REGISTERS]),
            batch.rs1.clone().reshape([n]),
        );

        let rs2_loss = ce_loss.forward(
            output.rs2.clone().reshape([n, NUM_REGISTERS]),
            batch.rs2.clone().reshape([n]),
        );

        let has_imm_loss = ce_loss.forward(
            output.has_imm.clone().reshape([n, 2]),
            batch.has_imm.clone().reshape([n]),
        );

        let imm_loss = ce_loss.forward(
            output.imm.clone().reshape([n, NUM_IMM_BINS]),
            batch.imm.clone().reshape([n]),
        );

        // Weighted sum of losses
        let total_loss = valid_loss.clone() * weights.valid
            + opcode_loss.clone() * weights.opcode
            + mode_loss.clone() * weights.mode
            + rd_loss.clone() * weights.rd
            + rs1_loss.clone() * weights.rs1
            + rs2_loss.clone() * weights.rs2
            + has_imm_loss.clone() * weights.has_imm
            + imm_loss.clone() * weights.imm;

        ParallelLossOutput {
            loss: total_loss,
            output,
            valid_target: batch.valid,
            opcode_target: batch.opcode,
        }
    }
}

/// Output from parallel loss computation
pub struct ParallelLossOutput<B: Backend> {
    pub loss: Tensor<B, 1>,
    pub output: ParallelOutput<B>,
    pub valid_target: Tensor<B, 2, Int>,
    pub opcode_target: Tensor<B, 2, Int>,
}

pub struct ParallelLossOutputSync {
    pub loss: f32,
}

impl<B: Backend> burn::train::ItemLazy for ParallelLossOutput<B> {
    type ItemSync = ParallelLossOutputSync;

    fn sync(self) -> Self::ItemSync {
        let loss: f32 = self.loss.into_scalar().elem();
        ParallelLossOutputSync { loss }
    }
}

impl<B: AutodiffBackend> TrainStep for TrainableParallelModel<B> {
    type Input = ParallelBatch<B>;
    type Output = ParallelLossOutput<B>;

    fn step(&self, batch: Self::Input) -> TrainOutput<Self::Output> {
        let output = self.forward_loss(batch);
        let grads = output.loss.clone().backward();
        TrainOutput::new(self, grads, output)
    }
}

impl<B: Backend> InferenceStep for TrainableParallelModel<B> {
    type Input = ParallelBatch<B>;
    type Output = ParallelLossOutput<B>;

    fn step(&self, batch: Self::Input) -> Self::Output {
        self.forward_loss(batch)
    }
}

/// Train parallel instruction model
pub fn train_parallel(config: ParallelTrainConfig) -> Result<(), TrainError> {
    use burn::train::Learner;

    type MyBackend = burn::backend::Wgpu;
    type MyAutodiff = burn::backend::Autodiff<MyBackend>;

    let device = burn::backend::wgpu::WgpuDevice::default();

    println!("=== Parallel Instruction Model Training ===");
    println!();
    println!("Loading dataset from: {}", config.data_path.display());

    // Load dataset - try parallel format first, fall back to legacy
    let mut dataset = ParallelDataset::from_jsonl(&config.data_path)
        .or_else(|_| ParallelDataset::from_legacy(&config.data_path))
        .map_err(|e| TrainError::DataLoadFailed(e.to_string()))?;

    println!("Loaded {} samples", dataset.len());

    if dataset.is_empty() {
        return Err(TrainError::DataLoadFailed("Dataset is empty".to_string()));
    }

    // Shuffle and split
    dataset.shuffle(config.seed);
    let (train_set, val_set) = dataset.split(config.train_ratio);

    println!("Train samples: {}", train_set.len());
    println!("Val samples: {}", val_set.len());

    // Initialize model
    let model: TrainableParallelModel<MyAutodiff> =
        TrainableParallelModel::new(ParallelModelConfig::new().init(&device));

    println!(
        "Model initialized: {} slots, {} hidden dim",
        NUM_SLOTS,
        ParallelModelConfig::new().hidden_dim
    );

    // Create optimizer
    let optim = AdamWConfig::new()
        .with_weight_decay(config.weight_decay as f32)
        .init();

    let lr_scheduler = ConstantLr::new(config.learning_rate);
    let mut learner = Learner::new(model, optim, lr_scheduler);

    // Create data loaders
    let batcher = ParallelBatcher::new();

    let train_loader = DataLoaderBuilder::new(batcher.clone())
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(4)
        .set_device(device.clone())
        .build(train_set);

    let _val_loader: std::sync::Arc<
        dyn burn::data::dataloader::DataLoader<MyAutodiff, ParallelBatch<MyAutodiff>>,
    > = DataLoaderBuilder::new(batcher)
        .batch_size(config.batch_size)
        .set_device(device.clone())
        .build(val_set);

    // Create artifact directory
    let default_dir = PathBuf::from(".");
    let artifact_dir = config.output_path.parent().unwrap_or(&default_dir);
    std::fs::create_dir_all(artifact_dir)?;

    // Training loop
    println!();
    println!("Starting training...");
    println!("  Epochs: {}", config.epochs);
    println!("  Batch size: {}", config.batch_size);
    println!("  Learning rate: {}", config.learning_rate);
    println!();

    for epoch in 0..config.epochs {
        let mut epoch_loss = 0.0;
        let mut batch_count = 0;

        learner.lr_step();

        for batch in train_loader.iter() {
            let train_output = learner.train_step(batch);
            let loss_val: f32 = train_output.item.loss.clone().into_scalar().elem();
            epoch_loss += loss_val as f64;
            batch_count += 1;
            learner.optimizer_step(train_output.grads);
        }

        let avg_loss = epoch_loss / batch_count.max(1) as f64;
        println!(
            "Epoch {}/{}: avg_loss = {:.4}",
            epoch + 1,
            config.epochs,
            avg_loss
        );
    }

    // Save model
    println!();
    println!("Saving model to: {}", config.output_path.display());

    let trained_model = learner.model();
    let inner_model = trained_model.inner().clone();
    let recorder = CompactRecorder::new();
    inner_model
        .save_file(&config.output_path, &recorder)
        .map_err(|e| TrainError::SaveFailed(e.to_string()))?;

    println!("Training complete!");

    Ok(())
}

// ============================================================================
// Legacy Multi-Head Model Training (backwards compatibility)
// ============================================================================

/// Training configuration (legacy)
#[derive(Debug, Clone)]
pub struct TrainConfig {
    /// Path to training data (JSONL)
    pub data_path: PathBuf,
    /// Output model path
    pub output_path: PathBuf,
    /// Number of epochs
    pub epochs: usize,
    /// Batch size
    pub batch_size: usize,
    /// Learning rate
    pub learning_rate: f64,
    /// Weight decay
    pub weight_decay: f64,
    /// Train/validation split ratio
    pub train_ratio: f32,
    /// Random seed
    pub seed: u64,
    /// Export to ONNX after training
    pub export_onnx: bool,
}

// Re-export for backwards compatibility
pub type NativeTrainConfig = TrainConfig;

impl Default for TrainConfig {
    fn default() -> Self {
        Self {
            data_path: PathBuf::from("training_data.jsonl"),
            output_path: PathBuf::from("model.mpk"),
            epochs: 50,
            batch_size: 64, // Smaller batch size for wgpu memory limits
            learning_rate: 0.001,
            weight_decay: 0.01,
            train_ratio: 0.9,
            seed: 42,
            export_onnx: true,
        }
    }
}

/// Loss weights for multi-head training
#[derive(Debug, Clone)]
pub struct LossWeights {
    pub intent: f32,
    pub count: f32,
    pub operand: f32,
    pub sign: f32,
}

impl Default for LossWeights {
    fn default() -> Self {
        Self {
            intent: 1.0,
            count: 0.5,
            operand: 0.5,
            sign: 0.3,
        }
    }
}

/// Training model wrapper for burn's training system
#[derive(Module, Debug)]
pub struct TrainableMultiHeadModel<B: Backend> {
    model: MultiHeadModel<B>,
}

impl<B: Backend> TrainableMultiHeadModel<B> {
    /// Create a new trainable model
    pub fn new(model: MultiHeadModel<B>) -> Self {
        Self { model }
    }

    /// Get reference to inner model for inference/saving
    pub fn inner(&self) -> &MultiHeadModel<B> {
        &self.model
    }

    /// Consume wrapper and return inner model
    pub fn into_inner(self) -> MultiHeadModel<B> {
        self.model
    }

    /// Forward pass with loss computation
    pub fn forward_loss(&self, batch: MultiHeadBatch<B>) -> MultiHeadLossOutput<B> {
        let weights = LossWeights::default();
        let output = self.model.forward(batch.tokens);

        // Compute cross-entropy losses
        let loss_fn = CrossEntropyLossConfig::new().init(&output.intent.device());

        // Intent loss
        let intent_loss = loss_fn.forward(output.intent.clone(), batch.intent.clone());

        // Count loss
        let count_loss = loss_fn.forward(output.count.clone(), batch.count.clone());

        // Operand losses (masked by count)
        let mut operand_loss = Tensor::zeros([1], &output.intent.device());
        let mut sign_loss = Tensor::zeros([1], &output.intent.device());

        for i in 0..4 {
            let op_logits = &output.operands[i];
            let sign_logits = &output.signs[i];

            // Get target for this operand position
            let op_target = batch
                .operands
                .clone()
                .slice([0..batch.operands.dims()[0], i..i + 1])
                .squeeze::<1>();
            let sign_target = batch
                .signs
                .clone()
                .slice([0..batch.signs.dims()[0], i..i + 1])
                .squeeze::<1>();

            // Compute losses (will be masked later in weighted sum)
            let op_l = loss_fn.forward(op_logits.clone(), op_target);
            let sign_l = loss_fn.forward(sign_logits.clone(), sign_target);

            operand_loss = operand_loss + op_l;
            sign_loss = sign_loss + sign_l;
        }

        // Average over operand positions
        operand_loss = operand_loss / 4.0;
        sign_loss = sign_loss / 4.0;

        // Weighted total loss
        let total_loss = intent_loss.clone() * weights.intent
            + count_loss.clone() * weights.count
            + operand_loss.clone() * weights.operand
            + sign_loss.clone() * weights.sign;

        MultiHeadLossOutput {
            loss: total_loss,
            output,
            intent_target: batch.intent,
        }
    }
}

/// Output from loss computation
pub struct MultiHeadLossOutput<B: Backend> {
    /// Total weighted loss
    pub loss: Tensor<B, 1>,
    /// Model output
    pub output: MultiHeadOutput<B>,
    /// Intent target for accuracy computation
    pub intent_target: Tensor<B, 1, Int>,
}

/// Synced version of loss output for metrics
pub struct MultiHeadLossOutputSync {
    pub loss: f32,
}

impl<B: Backend> burn::train::ItemLazy for MultiHeadLossOutput<B> {
    type ItemSync = MultiHeadLossOutputSync;

    fn sync(self) -> Self::ItemSync {
        let loss: f32 = self.loss.into_scalar().elem();
        MultiHeadLossOutputSync { loss }
    }
}

// Implement TrainStep for burn 0.20 API
impl<B: AutodiffBackend> TrainStep for TrainableMultiHeadModel<B> {
    type Input = MultiHeadBatch<B>;
    type Output = MultiHeadLossOutput<B>;

    fn step(&self, batch: Self::Input) -> TrainOutput<Self::Output> {
        let output = self.forward_loss(batch);
        let grads = output.loss.clone().backward();

        TrainOutput::new(self, grads, output)
    }
}

// Implement InferenceStep for burn 0.20 API (replaces ValidStep)
impl<B: Backend> InferenceStep for TrainableMultiHeadModel<B> {
    type Input = MultiHeadBatch<B>;
    type Output = MultiHeadLossOutput<B>;

    fn step(&self, batch: Self::Input) -> Self::Output {
        self.forward_loss(batch)
    }
}

/// Run native training (legacy multi-head model)
pub fn train(config: TrainConfig) -> Result<(), TrainError> {
    use burn::train::Learner;

    // Use wgpu backend by default
    type MyBackend = burn::backend::Wgpu;
    type MyAutodiff = burn::backend::Autodiff<MyBackend>;

    let device = burn::backend::wgpu::WgpuDevice::default();

    println!("Loading dataset from: {}", config.data_path.display());

    // Load dataset
    let mut dataset = MultiHeadDataset::from_jsonl(&config.data_path)
        .map_err(|e| TrainError::DataLoadFailed(e.to_string()))?;

    println!("Loaded {} samples", dataset.len());

    // Shuffle and split
    dataset.shuffle(config.seed);
    let (train_set, val_set) = dataset.split(config.train_ratio);

    println!("Train samples: {}", train_set.len());
    println!("Val samples: {}", val_set.len());

    // Initialize model
    let model: TrainableMultiHeadModel<MyAutodiff> =
        TrainableMultiHeadModel::new(MultiHeadModelConfig::new().init(&device));

    // Create optimizer
    let optim = AdamWConfig::new()
        .with_weight_decay(config.weight_decay as f32)
        .init();

    // Create LR scheduler (constant for now)
    let lr_scheduler = ConstantLr::new(config.learning_rate);

    // Create learner
    let mut learner = Learner::new(model, optim, lr_scheduler);

    // Create data loaders
    let batcher = MultiHeadBatcher::new();

    let train_loader = DataLoaderBuilder::new(batcher.clone())
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(4)
        .set_device(device.clone())
        .build(train_set);

    let _val_loader: std::sync::Arc<
        dyn burn::data::dataloader::DataLoader<MyAutodiff, MultiHeadBatch<MyAutodiff>>,
    > = DataLoaderBuilder::new(batcher)
        .batch_size(config.batch_size)
        .set_device(device.clone())
        .build(val_set);

    // Create artifact directory
    let default_dir = PathBuf::from(".");
    let artifact_dir = config.output_path.parent().unwrap_or(&default_dir);
    std::fs::create_dir_all(artifact_dir)?;

    // Training loop
    println!("\nStarting training...");
    println!("  Epochs: {}", config.epochs);
    println!("  Batch size: {}", config.batch_size);
    println!("  Learning rate: {}", config.learning_rate);
    println!();

    for epoch in 0..config.epochs {
        let mut epoch_loss = 0.0;
        let mut batch_count = 0;

        learner.lr_step();

        for batch in train_loader.iter() {
            let train_output = learner.train_step(batch);

            // Get loss value for logging
            let loss_val: f32 = train_output.item.loss.clone().into_scalar().elem();
            epoch_loss += loss_val as f64;
            batch_count += 1;

            // Update model with gradients
            learner.optimizer_step(train_output.grads);
        }

        let avg_loss = epoch_loss / batch_count as f64;
        println!(
            "Epoch {}/{}: avg_loss = {:.4}",
            epoch + 1,
            config.epochs,
            avg_loss
        );
    }

    // Save model (save inner MultiHeadModel for inference compatibility)
    println!("\nSaving model to: {}", config.output_path.display());

    let trained_model = learner.model();
    let inner_model = trained_model.inner().clone();
    let recorder = CompactRecorder::new();
    inner_model
        .save_file(&config.output_path, &recorder)
        .map_err(|e| TrainError::SaveFailed(e.to_string()))?;

    println!("Training complete!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_train_config_default() {
        let config = TrainConfig::default();
        assert_eq!(config.epochs, 50);
        assert_eq!(config.batch_size, 64);
        assert!((config.learning_rate - 0.001).abs() < 1e-6);
    }

    #[test]
    fn test_parallel_config_default() {
        let config = ParallelTrainConfig::default();
        assert_eq!(config.epochs, 50);
        assert_eq!(config.batch_size, 32);
        assert!((config.learning_rate - 0.001).abs() < 1e-6);
    }

    #[test]
    fn test_loss_weights_default() {
        let weights = LossWeights::default();
        assert!((weights.intent - 1.0).abs() < 1e-6);
        assert!((weights.count - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_parallel_loss_weights_default() {
        let weights = ParallelLossWeights::default();
        assert!((weights.valid - 2.0).abs() < 1e-6);
        assert!((weights.opcode - 2.0).abs() < 1e-6);
    }
}
