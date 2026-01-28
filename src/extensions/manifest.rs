//! Extension Manifest (neurlang.json)
//!
//! Defines the structure of extension manifests for Neurlang packages.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Extension manifest (neurlang.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
    /// Extension name (without path prefix)
    pub name: String,

    /// Semantic version
    pub version: String,

    /// Brief description
    #[serde(default)]
    pub description: String,

    /// Entry point file (relative to manifest)
    #[serde(default = "default_entry")]
    pub entry: String,

    /// Exported functions
    #[serde(default)]
    pub exports: Vec<ExtensionExport>,

    /// Dependencies (other extensions)
    #[serde(default)]
    pub dependencies: Vec<ExtensionDependency>,

    /// Minimum neurlang version required
    #[serde(default)]
    pub neurlang_version: Option<String>,

    /// Author information
    #[serde(default)]
    pub authors: Vec<String>,

    /// License (SPDX identifier)
    #[serde(default)]
    pub license: Option<String>,

    /// Repository URL
    #[serde(default)]
    pub repository: Option<String>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

fn default_entry() -> String {
    "main.nl".to_string()
}

impl ExtensionManifest {
    /// Create a new manifest with minimal required fields
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: String::new(),
            entry: default_entry(),
            exports: Vec::new(),
            dependencies: Vec::new(),
            neurlang_version: None,
            authors: Vec::new(),
            license: None,
            repository: None,
            metadata: HashMap::new(),
        }
    }

    /// Load manifest from a file
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ManifestError> {
        let file = File::open(path.as_ref()).map_err(|e| ManifestError::Io(e.to_string()))?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).map_err(|e| ManifestError::Parse(e.to_string()))
    }

    /// Save manifest to a file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ManifestError> {
        let file = File::create(path.as_ref()).map_err(|e| ManifestError::Io(e.to_string()))?;
        serde_json::to_writer_pretty(file, self)
            .map_err(|e| ManifestError::Serialize(e.to_string()))
    }

    /// Parse from JSON string
    pub fn from_json(json: &str) -> Result<Self, ManifestError> {
        serde_json::from_str(json).map_err(|e| ManifestError::Parse(e.to_string()))
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, ManifestError> {
        serde_json::to_string_pretty(self).map_err(|e| ManifestError::Serialize(e.to_string()))
    }

    /// Add an export
    pub fn add_export(&mut self, export: ExtensionExport) {
        self.exports.push(export);
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, dep: ExtensionDependency) {
        self.dependencies.push(dep);
    }

    /// Get the full import path for this extension
    pub fn import_path(&self) -> String {
        if let Some(ref repo) = self.repository {
            // Strip protocol prefix if present
            let path = repo
                .strip_prefix("https://")
                .or_else(|| repo.strip_prefix("http://"))
                .or_else(|| repo.strip_prefix("git://"))
                .unwrap_or(repo);
            format!("{}@{}", path, self.version)
        } else {
            format!("local/{}", self.name)
        }
    }
}

/// Exported function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionExport {
    /// Function name
    pub name: String,

    /// Input parameter types
    #[serde(default)]
    pub inputs: Vec<ParamType>,

    /// Output type
    #[serde(default)]
    pub output: Option<ParamType>,

    /// Brief description
    #[serde(default)]
    pub description: String,
}

impl ExtensionExport {
    /// Create a new export
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            inputs: Vec::new(),
            output: None,
            description: String::new(),
        }
    }

    /// Add an input parameter
    pub fn with_input(mut self, param: ParamType) -> Self {
        self.inputs.push(param);
        self
    }

    /// Set the output type
    pub fn with_output(mut self, output: ParamType) -> Self {
        self.output = Some(output);
        self
    }

    /// Set the description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
}

/// Parameter type for exports
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    /// 64-bit integer
    Int,
    /// Floating point
    Float,
    /// String (pointer + length)
    String,
    /// Byte buffer (pointer + length)
    Buffer,
    /// Boolean
    Bool,
    /// Array of a type
    Array(Box<ParamType>),
}

/// Extension dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionDependency {
    /// Import path (e.g., "github.com/user/lib")
    pub path: String,

    /// Version constraint (e.g., "^1.0.0", ">=1.2.0", "1.2.3")
    #[serde(default)]
    pub version: Option<String>,
}

impl ExtensionDependency {
    /// Create a new dependency
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            version: None,
        }
    }

    /// Add version constraint
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Parse a dependency string (e.g., "github.com/user/lib@v1.0.0")
    pub fn parse(s: &str) -> Self {
        if let Some((path, version)) = s.split_once('@') {
            Self {
                path: path.to_string(),
                version: Some(version.to_string()),
            }
        } else {
            Self {
                path: s.to_string(),
                version: None,
            }
        }
    }
}

/// Manifest error types
#[derive(Debug)]
pub enum ManifestError {
    /// I/O error
    Io(String),
    /// Parse error
    Parse(String),
    /// Serialization error
    Serialize(String),
    /// Validation error
    Validation(String),
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::Io(e) => write!(f, "I/O error: {}", e),
            ManifestError::Parse(e) => write!(f, "Parse error: {}", e),
            ManifestError::Serialize(e) => write!(f, "Serialization error: {}", e),
            ManifestError::Validation(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for ManifestError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let manifest = ExtensionManifest::new("my-extension", "1.0.0");
        assert_eq!(manifest.name, "my-extension");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.entry, "main.nl");
    }

    #[test]
    fn test_manifest_json_roundtrip() {
        let mut manifest = ExtensionManifest::new("test-ext", "2.0.0");
        manifest.description = "A test extension".to_string();
        manifest.add_export(
            ExtensionExport::new("parse")
                .with_input(ParamType::String)
                .with_output(ParamType::Int),
        );
        manifest.add_dependency(ExtensionDependency::parse("github.com/other/lib@v1.0.0"));

        let json = manifest.to_json().unwrap();
        let parsed = ExtensionManifest::from_json(&json).unwrap();

        assert_eq!(parsed.name, "test-ext");
        assert_eq!(parsed.version, "2.0.0");
        assert_eq!(parsed.exports.len(), 1);
        assert_eq!(parsed.dependencies.len(), 1);
    }

    #[test]
    fn test_dependency_parse() {
        let dep1 = ExtensionDependency::parse("github.com/user/lib@v1.2.3");
        assert_eq!(dep1.path, "github.com/user/lib");
        assert_eq!(dep1.version, Some("v1.2.3".to_string()));

        let dep2 = ExtensionDependency::parse("github.com/user/lib");
        assert_eq!(dep2.path, "github.com/user/lib");
        assert_eq!(dep2.version, None);
    }

    #[test]
    fn test_export_builder() {
        let export = ExtensionExport::new("calculate")
            .with_input(ParamType::Int)
            .with_input(ParamType::Int)
            .with_output(ParamType::Int)
            .with_description("Add two numbers");

        assert_eq!(export.name, "calculate");
        assert_eq!(export.inputs.len(), 2);
        assert!(export.output.is_some());
        assert!(!export.description.is_empty());
    }
}
