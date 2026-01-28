//! Inference Engine Abstraction Layer
//!
//! Provides a unified interface for multiple ML inference backends:
//!
//! - **ort** (ONNX Runtime): Fastest, GPU support via CUDA/TensorRT/CoreML
//! - **tract**: Pure Rust, smallest binary, CPU-only
//! - **candle**: Hugging Face's Rust ML, balanced performance
//! - **burn**: For native .mpk format models from burn training
//!
//! # Usage
//!
//! ```rust,ignore
//! use neurlang::inference::engines::{InferenceEngine, select_engine, EngineType};
//!
//! // Auto-select best engine for model format
//! let engine = select_engine(model_path, None)?;
//!
//! // Or specify an engine explicitly
//! let engine = select_engine(model_path, Some(EngineType::Tract))?;
//!
//! // Run inference
//! let prediction = engine.predict("add 5 and 3")?;
//! ```

#[cfg(feature = "ort-backend")]
pub mod ort_engine;

#[cfg(feature = "tract")]
pub mod tract_engine;

#[cfg(feature = "candle")]
pub mod candle_engine;

#[cfg(feature = "train")]
pub mod burn_engine;

use std::path::Path;
use std::time::Duration;
use thiserror::Error;

/// Errors from inference engines
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Failed to load model: {0}")]
    LoadError(String),
    #[error("Inference error: {0}")]
    InferenceError(String),
    #[error("Unsupported model format: {0}")]
    UnsupportedFormat(String),
    #[error("No inference engine available for this model format")]
    NoEngineAvailable,
    #[error("Engine not compiled in: {0}. Rebuild with --features {1}")]
    EngineNotAvailable(String, String),
}

/// Available inference engine types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineType {
    /// ONNX Runtime - fastest, GPU support
    Ort,
    /// Tract - pure Rust, smallest binary
    Tract,
    /// Candle - Hugging Face, balanced
    Candle,
    /// Burn - for native .mpk models
    Burn,
}

impl EngineType {
    /// Parse engine type from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ort" | "onnxruntime" | "onnx-runtime" => Some(Self::Ort),
            "tract" => Some(Self::Tract),
            "candle" => Some(Self::Candle),
            "burn" => Some(Self::Burn),
            "auto" | "" => None, // Auto-detect
            _ => None,
        }
    }

    /// Get the feature flag name for this engine
    pub fn feature_name(&self) -> &'static str {
        match self {
            Self::Ort => "ort-backend",
            Self::Tract => "tract",
            Self::Candle => "candle",
            Self::Burn => "train",
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Ort => "ONNX Runtime",
            Self::Tract => "Tract",
            Self::Candle => "Candle",
            Self::Burn => "Burn",
        }
    }

    /// Check if this engine is compiled in
    pub fn is_available(&self) -> bool {
        match self {
            #[cfg(feature = "ort-backend")]
            Self::Ort => true,
            #[cfg(not(feature = "ort-backend"))]
            Self::Ort => false,

            #[cfg(feature = "tract")]
            Self::Tract => true,
            #[cfg(not(feature = "tract"))]
            Self::Tract => false,

            #[cfg(feature = "candle")]
            Self::Candle => true,
            #[cfg(not(feature = "candle"))]
            Self::Candle => false,

            #[cfg(feature = "train")]
            Self::Burn => true,
            #[cfg(not(feature = "train"))]
            Self::Burn => false,
        }
    }

    /// Check if this engine supports the given model format
    pub fn supports_format(&self, ext: &str) -> bool {
        match self {
            Self::Ort => ext == "onnx",
            Self::Tract => ext == "onnx",
            Self::Candle => ext == "onnx" || ext == "safetensors",
            Self::Burn => ext == "mpk" || ext == "bin",
        }
    }
}

/// Prediction result from an inference engine
#[derive(Debug, Clone)]
pub struct EnginePrediction {
    /// Predicted intent ID (0-53)
    pub intent_id: usize,
    /// Prediction confidence (0.0-1.0)
    pub confidence: f32,
    /// Predicted operand count
    pub operand_count: usize,
    /// Operand values
    pub operands: Vec<i64>,
    /// Operand signs (0=positive, 1=negative)
    pub signs: Vec<usize>,
    /// Inference latency
    pub latency: Duration,
    /// Engine used
    pub engine: EngineType,
}

/// Trait for inference engine implementations
pub trait Engine: Send + Sync {
    /// Get the engine type
    fn engine_type(&self) -> EngineType;

    /// Predict intent and operands from text
    fn predict(&self, text: &str) -> Result<EnginePrediction, EngineError>;

    /// Check if the engine is ready
    fn is_ready(&self) -> bool;

    /// Get model info string
    fn model_info(&self) -> String;
}

/// Select the best available engine for a model file
///
/// # Arguments
/// * `model_path` - Path to the model file
/// * `preferred` - Optional preferred engine type
///
/// # Returns
/// A boxed Engine implementation
pub fn select_engine(
    model_path: &Path,
    preferred: Option<EngineType>,
) -> Result<Box<dyn Engine>, EngineError> {
    // Check model file exists
    if !model_path.exists() {
        return Err(EngineError::ModelNotFound(model_path.display().to_string()));
    }

    // Get file extension
    let ext = model_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // If user specified an engine, try to use it
    if let Some(engine_type) = preferred {
        return create_engine(engine_type, model_path, &ext);
    }

    // Auto-detect based on file extension
    match ext.as_str() {
        // .mpk (burn format) - only works with burn
        "mpk" | "bin" => {
            #[cfg(feature = "train")]
            return create_engine(EngineType::Burn, model_path, &ext);

            #[cfg(not(feature = "train"))]
            return Err(EngineError::EngineNotAvailable(
                "Burn".to_string(),
                "train".to_string(),
            ));
        }

        // .onnx - prefer ort, then tract, then candle
        "onnx" => {
            #[cfg(feature = "ort-backend")]
            return create_engine(EngineType::Ort, model_path, &ext);

            #[cfg(all(not(feature = "ort-backend"), feature = "tract"))]
            return create_engine(EngineType::Tract, model_path, &ext);

            #[cfg(all(
                not(feature = "ort-backend"),
                not(feature = "tract"),
                feature = "candle"
            ))]
            return create_engine(EngineType::Candle, model_path, &ext);

            #[cfg(all(
                not(feature = "ort-backend"),
                not(feature = "tract"),
                not(feature = "candle")
            ))]
            return Err(EngineError::NoEngineAvailable);
        }

        // .safetensors - prefer candle, then burn
        "safetensors" => {
            #[cfg(feature = "candle")]
            return create_engine(EngineType::Candle, model_path, &ext);

            #[cfg(all(not(feature = "candle"), feature = "train"))]
            return create_engine(EngineType::Burn, model_path, &ext);

            #[cfg(all(not(feature = "candle"), not(feature = "train")))]
            return Err(EngineError::NoEngineAvailable);
        }

        _ => Err(EngineError::UnsupportedFormat(ext)),
    }
}

/// Create an engine of the specified type
#[allow(unused_variables)]
fn create_engine(
    engine_type: EngineType,
    model_path: &Path,
    ext: &str,
) -> Result<Box<dyn Engine>, EngineError> {
    // Check if engine supports this format
    if !engine_type.supports_format(ext) {
        return Err(EngineError::UnsupportedFormat(format!(
            "{} engine does not support .{} format",
            engine_type.display_name(),
            ext
        )));
    }

    // Check if engine is compiled in
    if !engine_type.is_available() {
        return Err(EngineError::EngineNotAvailable(
            engine_type.display_name().to_string(),
            engine_type.feature_name().to_string(),
        ));
    }

    // Create the engine
    match engine_type {
        #[cfg(feature = "ort-backend")]
        EngineType::Ort => Ok(Box::new(ort_engine::OrtEngine::load(model_path)?)),

        #[cfg(feature = "tract")]
        EngineType::Tract => Ok(Box::new(tract_engine::TractEngine::load(model_path)?)),

        #[cfg(feature = "candle")]
        EngineType::Candle => Ok(Box::new(candle_engine::CandleEngine::load(model_path)?)),

        #[cfg(feature = "train")]
        EngineType::Burn => Ok(Box::new(burn_engine::BurnEngine::load(model_path)?)),

        #[allow(unreachable_patterns)]
        _ => Err(EngineError::EngineNotAvailable(
            engine_type.display_name().to_string(),
            engine_type.feature_name().to_string(),
        )),
    }
}

/// List available engines that are compiled in
#[allow(unused_mut)]
pub fn available_engines() -> Vec<EngineType> {
    let mut engines = Vec::new();

    #[cfg(feature = "ort-backend")]
    engines.push(EngineType::Ort);

    #[cfg(feature = "tract")]
    engines.push(EngineType::Tract);

    #[cfg(feature = "candle")]
    engines.push(EngineType::Candle);

    #[cfg(feature = "train")]
    engines.push(EngineType::Burn);

    engines
}

/// Get a human-readable description of available engines
pub fn engines_info() -> String {
    let engines = available_engines();
    if engines.is_empty() {
        return "No inference engines compiled in. Rebuild with --features ort-backend or --features tract".to_string();
    }

    let names: Vec<_> = engines.iter().map(|e| e.display_name()).collect();
    format!("Available engines: {}", names.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_type_from_str() {
        assert_eq!(EngineType::from_str("ort"), Some(EngineType::Ort));
        assert_eq!(EngineType::from_str("tract"), Some(EngineType::Tract));
        assert_eq!(EngineType::from_str("candle"), Some(EngineType::Candle));
        assert_eq!(EngineType::from_str("burn"), Some(EngineType::Burn));
        assert_eq!(EngineType::from_str("auto"), None);
        assert_eq!(EngineType::from_str(""), None);
        assert_eq!(EngineType::from_str("unknown"), None);
    }

    #[test]
    fn test_engine_supports_format() {
        assert!(EngineType::Ort.supports_format("onnx"));
        assert!(!EngineType::Ort.supports_format("mpk"));

        assert!(EngineType::Tract.supports_format("onnx"));
        assert!(!EngineType::Tract.supports_format("mpk"));

        assert!(EngineType::Candle.supports_format("onnx"));
        assert!(EngineType::Candle.supports_format("safetensors"));

        assert!(EngineType::Burn.supports_format("mpk"));
        assert!(!EngineType::Burn.supports_format("onnx"));
    }
}
