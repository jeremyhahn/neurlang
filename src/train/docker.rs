//! Docker/Podman Training Backend
//!
//! Provides containerized PyTorch training for GPU-optimized workflows.
//! Automatically builds the training image on first use.

use super::backend::{find_container_runtime, ContainerRuntime, TrainError};
use std::path::PathBuf;
use std::process::Command;

/// Docker training image name
const IMAGE_NAME: &str = "neurlang-train";

/// Configuration for Docker training
#[derive(Debug, Clone)]
pub struct DockerTrainConfig {
    /// Path to training data
    pub data_path: PathBuf,
    /// Output model path
    pub output_path: PathBuf,
    /// Number of epochs
    pub epochs: usize,
    /// Batch size
    pub batch_size: usize,
    /// Learning rate
    pub learning_rate: f64,
    /// Export to ONNX after training
    pub export_onnx: bool,
}

impl Default for DockerTrainConfig {
    fn default() -> Self {
        Self {
            data_path: PathBuf::from("training_data.jsonl"),
            output_path: PathBuf::from("model.onnx"),
            epochs: 50,
            batch_size: 256,
            learning_rate: 0.001,
            export_onnx: true,
        }
    }
}

/// Docker-based trainer
pub struct DockerTrainer {
    runtime: ContainerRuntime,
}

impl DockerTrainer {
    /// Create a new Docker trainer
    pub fn new() -> Result<Self, TrainError> {
        let runtime = find_container_runtime().ok_or(TrainError::DockerNotFound)?;

        Ok(Self { runtime })
    }

    /// Get the container runtime being used
    pub fn runtime(&self) -> ContainerRuntime {
        self.runtime
    }

    /// Check if the training image exists
    pub fn image_exists(&self) -> Result<bool, TrainError> {
        let output = Command::new(self.runtime.command())
            .args(["images", "-q", IMAGE_NAME])
            .output()?;

        Ok(!output.stdout.is_empty())
    }

    /// Build the training image
    pub fn build_image(&self) -> Result<(), TrainError> {
        println!(
            "Building training image '{}' (this may take a few minutes)...",
            IMAGE_NAME
        );

        let status = Command::new(self.runtime.command())
            .env("DOCKER_BUILDKIT", "0") // Use legacy builder for compatibility
            .args([
                "build",
                "-t",
                IMAGE_NAME,
                "-f",
                "docker/Dockerfile.train",
                ".",
            ])
            .status()?;

        if !status.success() {
            return Err(TrainError::BuildFailed(
                "Container image build failed".to_string(),
            ));
        }

        println!("Training image built successfully.");
        Ok(())
    }

    /// Ensure the training image exists, building if necessary
    pub fn ensure_image(&self) -> Result<(), TrainError> {
        if self.image_exists()? {
            return Ok(());
        }

        // Check if Dockerfile exists
        let dockerfile = PathBuf::from("docker/Dockerfile.train");
        if !dockerfile.exists() {
            println!("Creating default training Dockerfile...");
            self.create_dockerfile()?;
        }

        self.build_image()
    }

    /// Create the default training Dockerfile
    fn create_dockerfile(&self) -> Result<(), TrainError> {
        use std::fs;

        // Ensure docker directory exists
        fs::create_dir_all("docker")?;

        let dockerfile = r#"# Neurlang Training Container
# Multi-head PyTorch model training with GPU support

FROM nvidia/cuda:12.1-runtime-ubuntu22.04

# Install Python and dependencies
RUN apt-get update && apt-get install -y \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Install PyTorch and training dependencies
RUN pip3 install --no-cache-dir \
    torch==2.1.0 \
    numpy==1.26.0 \
    tqdm==4.66.0 \
    onnx==1.15.0 \
    onnxruntime==1.16.0

# Copy training code
COPY train/multihead /app/train

WORKDIR /app

# Entrypoint
ENTRYPOINT ["python3", "-m", "train"]
"#;

        fs::write("docker/Dockerfile.train", dockerfile)?;

        Ok(())
    }

    /// Check if NVIDIA GPU support is available
    pub fn has_gpu_support(&self) -> bool {
        // Check if nvidia-smi exists
        which::which("nvidia-smi").is_ok()
    }

    /// Run training in container
    pub fn train(&self, config: &DockerTrainConfig) -> Result<(), TrainError> {
        // Ensure image exists
        self.ensure_image()?;

        // Get absolute paths (use current dir if no parent)
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let data_parent = config
            .data_path
            .parent()
            .filter(|p| !p.as_os_str().is_empty());
        let data_dir = data_parent
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| cwd.clone())
            .canonicalize()
            .unwrap_or_else(|_| cwd.clone());

        let model_parent = config
            .output_path
            .parent()
            .filter(|p| !p.as_os_str().is_empty());
        let model_dir = model_parent
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| cwd.clone())
            .canonicalize()
            .unwrap_or_else(|_| cwd.clone());

        // Build run arguments
        let mut args = vec![
            "run".to_string(),
            "--rm".to_string(),
            "-v".to_string(),
            format!("{}:/data", data_dir.display()),
            "-v".to_string(),
            format!("{}:/models", model_dir.display()),
        ];

        // Add GPU support if available
        if self.has_gpu_support() {
            args.extend(["--gpus".to_string(), "all".to_string()]);
        }

        // Add image and training arguments
        args.push(IMAGE_NAME.to_string());
        args.extend([
            "--data".to_string(),
            format!(
                "/data/{}",
                config.data_path.file_name().unwrap().to_str().unwrap()
            ),
            "--output".to_string(),
            format!(
                "/models/{}",
                config.output_path.file_name().unwrap().to_str().unwrap()
            ),
            "--epochs".to_string(),
            config.epochs.to_string(),
            "--batch-size".to_string(),
            config.batch_size.to_string(),
            "--lr".to_string(),
            config.learning_rate.to_string(),
            "--device".to_string(),
            if self.has_gpu_support() {
                "cuda"
            } else {
                "cpu"
            }
            .to_string(),
        ]);

        println!(
            "Starting training with {} backend...",
            self.runtime.display_name()
        );
        println!("  Data: {}", config.data_path.display());
        println!("  Output: {}", config.output_path.display());
        println!("  Epochs: {}", config.epochs);
        println!("  Batch size: {}", config.batch_size);
        println!(
            "  GPU: {}",
            if self.has_gpu_support() { "Yes" } else { "No" }
        );
        println!();
        println!(
            "  Docker command: {} {}",
            self.runtime.command(),
            args.join(" ")
        );
        println!();

        // Run training
        let status = Command::new(self.runtime.command()).args(&args).status()?;

        if !status.success() {
            return Err(TrainError::TrainingFailed(
                "Container training failed".to_string(),
            ));
        }

        println!("\nTraining complete!");
        println!("Model saved to: {}", config.output_path.display());

        Ok(())
    }

    /// Evaluate a trained model
    pub fn evaluate(&self, model_path: &PathBuf, test_data: &PathBuf) -> Result<(), TrainError> {
        self.ensure_image()?;

        let model_dir = model_path
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .canonicalize()?;

        let data_dir = test_data
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .canonicalize()?;

        let mut args = vec![
            "run".to_string(),
            "--rm".to_string(),
            "-v".to_string(),
            format!("{}:/models", model_dir.display()),
            "-v".to_string(),
            format!("{}:/data", data_dir.display()),
        ];

        if self.has_gpu_support() {
            args.extend(["--gpus".to_string(), "all".to_string()]);
        }

        args.push(IMAGE_NAME.to_string());
        args.extend([
            "evaluate".to_string(),
            "--model".to_string(),
            format!(
                "/models/{}",
                model_path.file_name().unwrap().to_str().unwrap()
            ),
            "--data".to_string(),
            format!("/data/{}", test_data.file_name().unwrap().to_str().unwrap()),
        ]);

        let status = Command::new(self.runtime.command()).args(&args).status()?;

        if !status.success() {
            return Err(TrainError::TrainingFailed("Evaluation failed".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DockerTrainConfig::default();
        assert_eq!(config.epochs, 50);
        assert_eq!(config.batch_size, 256);
        assert!(config.export_onnx);
    }
}
