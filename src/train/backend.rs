//! Training Backend Detection and Selection
//!
//! Detects available training backends and selects the best option.

use thiserror::Error;

/// Training backend errors
#[derive(Debug, Error)]
pub enum TrainError {
    #[error("Native training not available. Rebuild with --features train")]
    NativeNotAvailable,
    #[error("Docker/Podman not found. Install docker or podman.")]
    DockerNotFound,
    #[error("No training backend available")]
    NoBackendAvailable,
    #[error("Container image build failed: {0}")]
    BuildFailed(String),
    #[error("Training failed: {0}")]
    TrainingFailed(String),
    #[error("Model save failed: {0}")]
    SaveFailed(String),
    #[error("Data loading failed: {0}")]
    DataLoadFailed(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Available training backends
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainBackend {
    /// Native burn framework training
    Native,
    /// Docker container-based PyTorch training
    Docker,
}

impl TrainBackend {
    /// Parse backend from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "native" | "burn" | "rust" => Some(Self::Native),
            "docker" | "podman" | "container" => Some(Self::Docker),
            "auto" | "" => None, // Auto-detect
            _ => None,
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Native => "Native (burn)",
            Self::Docker => "Docker (PyTorch)",
        }
    }

    /// Get feature name
    pub fn feature_name(&self) -> &'static str {
        match self {
            Self::Native => "train",
            Self::Docker => "docker",
        }
    }

    /// Check if this backend is available
    pub fn is_available(&self) -> bool {
        match self {
            #[cfg(feature = "train")]
            Self::Native => true,
            #[cfg(not(feature = "train"))]
            Self::Native => false,
            Self::Docker => find_container_runtime().is_some(),
        }
    }
}

/// Container runtime type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerRuntime {
    Docker,
    Podman,
}

impl ContainerRuntime {
    /// Get the command name
    pub fn command(&self) -> &'static str {
        match self {
            Self::Docker => "docker",
            Self::Podman => "podman",
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Docker => "Docker",
            Self::Podman => "Podman",
        }
    }
}

/// Find available container runtime (checks if it actually works)
pub fn find_container_runtime() -> Option<ContainerRuntime> {
    use std::process::Command;

    // Check for docker first - verify it can actually run
    if which::which("docker").is_ok() {
        if let Ok(output) = Command::new("docker").args(["info"]).output() {
            if output.status.success() {
                return Some(ContainerRuntime::Docker);
            }
        }
    }

    // Then check for podman
    if which::which("podman").is_ok() {
        if let Ok(output) = Command::new("podman").args(["info"]).output() {
            if output.status.success() {
                return Some(ContainerRuntime::Podman);
            }
        }
    }

    None
}

/// Detect best available training backend
///
/// # Arguments
/// * `preferred` - Optional preferred backend
///
/// # Returns
/// The detected backend or error if none available
pub fn detect_backend(preferred: Option<TrainBackend>) -> Result<TrainBackend, TrainError> {
    match preferred {
        Some(TrainBackend::Native) => {
            #[cfg(feature = "train")]
            return Ok(TrainBackend::Native);

            #[cfg(not(feature = "train"))]
            return Err(TrainError::NativeNotAvailable);
        }

        Some(TrainBackend::Docker) => {
            if find_container_runtime().is_some() {
                return Ok(TrainBackend::Docker);
            }
            Err(TrainError::DockerNotFound)
        }

        None => {
            // Auto-detect: prefer native if available, else docker
            #[cfg(feature = "train")]
            return Ok(TrainBackend::Native);

            #[cfg(not(feature = "train"))]
            {
                if find_container_runtime().is_some() {
                    return Ok(TrainBackend::Docker);
                }
                Err(TrainError::NoBackendAvailable)
            }
        }
    }
}

/// List available training backends
pub fn available_backends() -> Vec<TrainBackend> {
    let mut backends = Vec::new();

    #[cfg(feature = "train")]
    backends.push(TrainBackend::Native);

    if find_container_runtime().is_some() {
        backends.push(TrainBackend::Docker);
    }

    backends
}

/// Get a human-readable description of available backends
pub fn backends_info() -> String {
    let backends = available_backends();

    if backends.is_empty() {
        return "No training backends available. Install docker/podman or rebuild with --features train".to_string();
    }

    let names: Vec<_> = backends.iter().map(|b| b.display_name()).collect();
    format!("Available backends: {}", names.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_from_str() {
        assert_eq!(TrainBackend::from_str("native"), Some(TrainBackend::Native));
        assert_eq!(TrainBackend::from_str("burn"), Some(TrainBackend::Native));
        assert_eq!(TrainBackend::from_str("docker"), Some(TrainBackend::Docker));
        assert_eq!(TrainBackend::from_str("podman"), Some(TrainBackend::Docker));
        assert_eq!(TrainBackend::from_str("auto"), None);
        assert_eq!(TrainBackend::from_str("unknown"), None);
    }

    #[test]
    fn test_container_runtime_command() {
        assert_eq!(ContainerRuntime::Docker.command(), "docker");
        assert_eq!(ContainerRuntime::Podman.command(), "podman");
    }
}
