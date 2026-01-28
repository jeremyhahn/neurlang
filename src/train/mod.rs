//! Native Training Module for Neurlang
//!
//! Provides Rust-native training using the burn framework, with fallback
//! to Docker/Podman for PyTorch training.
//!
//! # Training Backends
//!
//! - **Native (burn)**: Zero dependencies, single binary, wgpu/CUDA GPU support
//! - **Docker/Podman**: Containerized PyTorch, familiar stack, full GPU support
//!
//! # Usage
//!
//! ```rust,ignore
//! use neurlang::train::{TrainBackend, detect_backend};
//!
//! // Auto-detect best available backend
//! let backend = detect_backend(None)?;
//!
//! // Or force a specific backend
//! let backend = detect_backend(Some(TrainBackend::Docker))?;
//!
//! // For native training:
//! #[cfg(feature = "train")]
//! {
//!     use neurlang::train::{train, NativeTrainConfig};
//!     let config = NativeTrainConfig::default();
//!     train(config)?;
//! }
//!
//! // For Docker training:
//! use neurlang::train::{DockerTrainer, DockerTrainConfig};
//! let trainer = DockerTrainer::new()?;
//! let config = DockerTrainConfig::default();
//! trainer.train(&config)?;
//! ```

pub mod backend;
pub mod docker;

#[cfg(feature = "train")]
pub mod dataset;
#[cfg(feature = "train")]
pub mod model;
#[cfg(feature = "train")]
pub mod trainer;

// Core exports (always available)
pub use backend::{
    available_backends, backends_info, detect_backend, find_container_runtime, ContainerRuntime,
    TrainBackend, TrainError,
};
pub use docker::{DockerTrainConfig, DockerTrainer};

// Native training exports (feature-gated)
#[cfg(feature = "train")]
pub use dataset::{
    InstructionData,
    // Legacy dataset
    MultiHeadBatch,
    MultiHeadBatcher,
    MultiHeadDataset,
    MultiHeadSample,
    ParallelBatch,
    ParallelBatcher,
    // Parallel dataset (new)
    ParallelDataset,
    ParallelSample,
    RawParallelSample,
    TestCase,
};
#[cfg(feature = "train")]
pub use model::{
    // Legacy multi-head model
    MultiHeadModel,
    MultiHeadModelConfig,
    MultiHeadOutput,
    MultiHeadPrediction,
    // Parallel model (new)
    ParallelInstructionModel,
    ParallelModelConfig,
    ParallelOutput,
    ParallelPrediction,
    NUM_IMM_BINS,
    NUM_MODES,
    NUM_OPCODES,
    NUM_REGISTERS,
    NUM_SLOTS,
};
#[cfg(feature = "train")]
pub use trainer::{
    // Legacy training
    train,
    // Parallel training (new)
    train_parallel,
    LossWeights,
    ParallelLossWeights,
    ParallelTrainConfig,
    TrainConfig as NativeTrainConfig,
};
