//! Neurlang Project Configuration
//!
//! Handles parsing and management of neurlang.toml configuration files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

/// Configuration errors.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("Config file not found: {0}")]
    NotFound(String),
}

/// Result type for configuration operations.
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Root configuration structure matching neurlang.toml.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NeurlangConfig {
    /// Package metadata
    #[serde(default)]
    pub package: PackageConfig,

    /// Stdlib module configuration
    #[serde(default)]
    pub stdlib: StdlibConfig,

    /// External extensions
    #[serde(default)]
    pub extensions: ExtensionsConfig,

    /// Build configuration
    #[serde(default)]
    pub build: BuildConfig,

    /// Compiler settings
    #[serde(default)]
    pub compiler: CompilerConfig,

    /// Training data generation
    #[serde(default)]
    pub training: TrainingConfig,
}

impl NeurlangConfig {
    /// Load configuration from a file path.
    pub fn load(path: &Path) -> ConfigResult<Self> {
        if !path.exists() {
            return Err(ConfigError::NotFound(path.display().to_string()));
        }
        let content = std::fs::read_to_string(path)?;
        let config: NeurlangConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration from the current directory or parents.
    pub fn load_from_cwd() -> ConfigResult<Self> {
        let cwd = std::env::current_dir().map_err(ConfigError::Io)?;
        Self::find_and_load(&cwd)
    }

    /// Find and load configuration by searching up from the given directory.
    pub fn find_and_load(start_dir: &Path) -> ConfigResult<Self> {
        let mut dir = start_dir.to_path_buf();
        loop {
            let config_path = dir.join("neurlang.toml");
            if config_path.exists() {
                return Self::load(&config_path);
            }
            if !dir.pop() {
                // Reached root without finding config
                return Ok(Self::default());
            }
        }
    }

    /// Save configuration to a file.
    pub fn save(&self, path: &Path) -> ConfigResult<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get list of enabled stdlib modules.
    pub fn enabled_stdlib_modules(&self) -> Vec<&'static str> {
        let mut modules = Vec::new();
        if self.stdlib.math {
            modules.push("math");
        }
        if self.stdlib.float {
            modules.push("float");
        }
        if self.stdlib.string {
            modules.push("string");
        }
        if self.stdlib.array {
            modules.push("array");
        }
        if self.stdlib.bitwise {
            modules.push("bitwise");
        }
        if self.stdlib.collections {
            modules.push("collections");
        }
        modules
    }
}

/// Package metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    /// Package name
    #[serde(default = "default_package_name")]
    pub name: String,

    /// Package version
    #[serde(default = "default_version")]
    pub version: String,

    /// Package description
    #[serde(default)]
    pub description: String,
}

fn default_package_name() -> String {
    "neurlang-project".to_string()
}

fn default_version() -> String {
    "0.1.0".to_string()
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self {
            name: default_package_name(),
            version: default_version(),
            description: String::new(),
        }
    }
}

/// Stdlib module enable/disable flags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdlibConfig {
    /// Math functions (factorial, fibonacci, gcd, etc.)
    #[serde(default = "default_true")]
    pub math: bool,

    /// Float/FPU operations
    #[serde(default = "default_true")]
    pub float: bool,

    /// String manipulation
    #[serde(default = "default_true")]
    pub string: bool,

    /// Array operations
    #[serde(default = "default_true")]
    pub array: bool,

    /// Bitwise operations
    #[serde(default = "default_true")]
    pub bitwise: bool,

    /// Collections (stack, queue, hashtable)
    #[serde(default = "default_true")]
    pub collections: bool,
}

fn default_true() -> bool {
    true
}

impl Default for StdlibConfig {
    fn default() -> Self {
        Self {
            math: true,
            float: true,
            string: true,
            array: true,
            bitwise: true,
            collections: true,
        }
    }
}

/// External extensions configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtensionsConfig {
    /// Rust crates from crates.io
    #[serde(default)]
    pub crates: HashMap<String, CrateExtension>,

    /// Go packages
    #[serde(default)]
    pub go: HashMap<String, GoExtension>,

    /// C libraries
    #[serde(default)]
    pub c: HashMap<String, CExtension>,

    /// Local extensions
    #[serde(default)]
    pub local: HashMap<String, LocalExtension>,
}

/// Rust crate extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateExtension {
    /// Crate version
    pub version: String,

    /// Optional features to enable
    #[serde(default)]
    pub features: Vec<String>,

    /// Optional: use default features
    #[serde(default = "default_true")]
    pub default_features: bool,
}

/// Go package extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoExtension {
    /// Go package path (e.g., "github.com/user/pkg")
    pub package: String,

    /// Git tag or version
    #[serde(default)]
    pub tag: Option<String>,

    /// Git branch
    #[serde(default)]
    pub branch: Option<String>,
}

/// C library extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CExtension {
    /// Library version
    #[serde(default)]
    pub version: Option<String>,

    /// Use system library via pkg-config
    #[serde(default)]
    pub system: bool,

    /// Path to vendored library
    #[serde(default)]
    pub path: Option<String>,
}

/// Local extension (any language, auto-detected).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalExtension {
    /// Path to extension directory
    pub path: String,
}

/// Build configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Include source comments in generated .nl files
    #[serde(default = "default_true")]
    pub include_comments: bool,

    /// Generate @test annotations from doc comments
    #[serde(default = "default_true")]
    pub generate_tests: bool,

    /// Maximum instructions per function
    #[serde(default = "default_max_instructions")]
    pub max_instructions: usize,

    /// Output directory for generated files
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
}

fn default_max_instructions() -> usize {
    1000
}

fn default_output_dir() -> String {
    "lib".to_string()
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            include_comments: true,
            generate_tests: true,
            max_instructions: 1000,
            output_dir: "lib".to_string(),
        }
    }
}

/// Compiler settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    /// Optimization level (0-2)
    #[serde(default = "default_optimization")]
    pub optimization: u8,

    /// Enable debug wrappers for extensions
    #[serde(default)]
    pub debug_mode: bool,

    /// Target architecture
    #[serde(default = "default_target")]
    pub target: String,
}

fn default_optimization() -> u8 {
    1
}

fn default_target() -> String {
    "x86_64".to_string()
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            optimization: 1,
            debug_mode: false,
            target: "x86_64".to_string(),
        }
    }
}

/// Training data generation settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    /// Enable training data generation
    #[serde(default)]
    pub enabled: bool,

    /// Output file path
    #[serde(default = "default_training_output")]
    pub output: String,

    /// Number of synthetic samples to generate
    #[serde(default)]
    pub synthetic_samples: usize,
}

fn default_training_output() -> String {
    "train/stdlib_training.jsonl".to_string()
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            output: default_training_output(),
            synthetic_samples: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NeurlangConfig::default();
        assert!(config.stdlib.math);
        assert!(config.stdlib.float);
        assert_eq!(config.build.max_instructions, 1000);
    }

    #[test]
    fn test_parse_config() {
        let toml_str = r#"
[package]
name = "test-project"
version = "1.0.0"

[stdlib]
math = true
float = false

[build]
include_comments = false
max_instructions = 500
"#;
        let config: NeurlangConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.package.name, "test-project");
        assert!(config.stdlib.math);
        assert!(!config.stdlib.float);
        assert!(!config.build.include_comments);
        assert_eq!(config.build.max_instructions, 500);
    }

    #[test]
    fn test_enabled_modules() {
        let mut config = NeurlangConfig::default();
        config.stdlib.float = false;
        config.stdlib.collections = false;

        let modules = config.enabled_stdlib_modules();
        assert!(modules.contains(&"math"));
        assert!(!modules.contains(&"float"));
        assert!(!modules.contains(&"collections"));
    }
}
