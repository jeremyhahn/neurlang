//! Extension Loader
//!
//! Loads and manages extension code from installed packages.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ir::{Assembler, Program};

use super::manifest::{ExtensionExport, ExtensionManifest};
use super::registry::{ExtensionInfo, ExtensionRegistry, RegistryError};

/// A loaded extension with its compiled program
#[derive(Debug)]
pub struct LoadedExtension {
    /// Extension manifest
    pub manifest: ExtensionManifest,
    /// Compiled program
    pub program: Program,
    /// Source path
    pub path: PathBuf,
    /// Export name to entry point offset mapping
    pub exports: HashMap<String, usize>,
}

impl LoadedExtension {
    /// Get an export by name
    pub fn get_export(&self, name: &str) -> Option<&ExtensionExport> {
        self.manifest.exports.iter().find(|e| e.name == name)
    }

    /// Get the entry point offset for an export
    pub fn entry_point(&self, name: &str) -> Option<usize> {
        self.exports.get(name).copied()
    }
}

/// Extension loader
pub struct ExtensionLoader {
    /// Registry for finding extensions
    registry: ExtensionRegistry,
    /// Loaded extensions cache
    loaded: HashMap<String, LoadedExtension>,
    /// Assembler for compiling extensions
    assembler: Assembler,
}

impl ExtensionLoader {
    /// Create a new loader with the default registry
    pub fn new() -> Result<Self, LoadError> {
        let registry = ExtensionRegistry::new().map_err(LoadError::Registry)?;
        Ok(Self {
            registry,
            loaded: HashMap::new(),
            assembler: Assembler::new(),
        })
    }

    /// Create a loader with a custom registry
    pub fn with_registry(registry: ExtensionRegistry) -> Self {
        Self {
            registry,
            loaded: HashMap::new(),
            assembler: Assembler::new(),
        }
    }

    /// Load an extension by import path
    pub fn load(&mut self, import_path: &str) -> Result<&LoadedExtension, LoadError> {
        // Check cache first
        if self.loaded.contains_key(import_path) {
            return Ok(self.loaded.get(import_path).unwrap());
        }

        // Find in registry
        let info = self
            .registry
            .get(import_path)
            .ok_or_else(|| LoadError::NotFound(import_path.to_string()))?
            .clone();

        // Load the extension
        let loaded = self.load_from_info(&info)?;
        self.loaded.insert(import_path.to_string(), loaded);

        Ok(self.loaded.get(import_path).unwrap())
    }

    /// Load an extension from info
    fn load_from_info(&mut self, info: &ExtensionInfo) -> Result<LoadedExtension, LoadError> {
        let entry_path = info.path.join(&info.manifest.entry);

        // Read source
        let source = fs::read_to_string(&entry_path).map_err(|e| LoadError::Io(e.to_string()))?;

        // Compile
        let program = self
            .assembler
            .assemble(&source)
            .map_err(|e| LoadError::Compile(e.to_string()))?;

        // Map exports to entry points
        // For now, we use label positions if available, or default to 0
        let mut exports = HashMap::new();
        let code_labels: HashMap<_, _> = self
            .assembler
            .code_labels()
            .map(|(name, idx)| (name.clone(), idx))
            .collect();
        for export in &info.manifest.exports {
            // Try to find label with export name
            if let Some(&offset) = code_labels.get(&export.name) {
                exports.insert(export.name.clone(), offset);
            } else {
                // Default to entry point
                exports.insert(export.name.clone(), program.entry_point);
            }
        }

        Ok(LoadedExtension {
            manifest: info.manifest.clone(),
            program,
            path: info.path.clone(),
            exports,
        })
    }

    /// Load extension from a file path directly
    pub fn load_file(&mut self, path: impl AsRef<Path>) -> Result<LoadedExtension, LoadError> {
        let path = path.as_ref();

        // Check for manifest
        let manifest_path = if path.is_dir() {
            path.join("neurlang.json")
        } else {
            path.parent()
                .map(|p| p.join("neurlang.json"))
                .unwrap_or_else(|| PathBuf::from("neurlang.json"))
        };

        let manifest = if manifest_path.exists() {
            ExtensionManifest::load(&manifest_path)
                .map_err(|e| LoadError::Manifest(e.to_string()))?
        } else {
            // Create a default manifest
            let name = path
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("unnamed")
                .to_string();
            ExtensionManifest::new(name, "0.0.0")
        };

        // Determine entry path
        let entry_path = if path.is_dir() {
            path.join(&manifest.entry)
        } else {
            path.to_path_buf()
        };

        // Read and compile
        let source = fs::read_to_string(&entry_path).map_err(|e| LoadError::Io(e.to_string()))?;

        let program = self
            .assembler
            .assemble(&source)
            .map_err(|e| LoadError::Compile(e.to_string()))?;

        let mut exports = HashMap::new();
        let code_labels: HashMap<_, _> = self
            .assembler
            .code_labels()
            .map(|(name, idx)| (name.clone(), idx))
            .collect();
        for export in &manifest.exports {
            if let Some(&offset) = code_labels.get(&export.name) {
                exports.insert(export.name.clone(), offset);
            } else {
                exports.insert(export.name.clone(), program.entry_point);
            }
        }

        Ok(LoadedExtension {
            manifest,
            program,
            path: path.to_path_buf(),
            exports,
        })
    }

    /// Load an extension from source code
    pub fn load_source(&mut self, name: &str, source: &str) -> Result<LoadedExtension, LoadError> {
        let program = self
            .assembler
            .assemble(source)
            .map_err(|e| LoadError::Compile(e.to_string()))?;

        let manifest = ExtensionManifest::new(name, "0.0.0");

        let mut exports = HashMap::new();
        for (label, offset) in self.assembler.code_labels() {
            exports.insert(label.clone(), offset);
        }

        Ok(LoadedExtension {
            manifest,
            program,
            path: PathBuf::new(),
            exports,
        })
    }

    /// Install and load an extension
    pub fn install_and_load(&mut self, url: &str) -> Result<&LoadedExtension, LoadError> {
        // Install via registry
        let info = self.registry.install(url).map_err(LoadError::Registry)?;

        // Load it
        let loaded = self.load_from_info(&info)?;

        let import_path = url.to_string();
        self.loaded.insert(import_path.clone(), loaded);

        Ok(self.loaded.get(&import_path).unwrap())
    }

    /// Unload an extension from cache
    pub fn unload(&mut self, import_path: &str) -> bool {
        self.loaded.remove(import_path).is_some()
    }

    /// Clear all loaded extensions
    pub fn clear(&mut self) {
        self.loaded.clear();
    }

    /// Get a reference to the registry
    pub fn registry(&self) -> &ExtensionRegistry {
        &self.registry
    }

    /// Get a mutable reference to the registry
    pub fn registry_mut(&mut self) -> &mut ExtensionRegistry {
        &mut self.registry
    }

    /// List all loaded extensions
    pub fn loaded(&self) -> impl Iterator<Item = (&String, &LoadedExtension)> {
        self.loaded.iter()
    }
}

impl Default for ExtensionLoader {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            registry: ExtensionRegistry::default(),
            loaded: HashMap::new(),
            assembler: Assembler::new(),
        })
    }
}

/// Load error types
#[derive(Debug)]
pub enum LoadError {
    /// Registry error
    Registry(RegistryError),
    /// Extension not found
    NotFound(String),
    /// I/O error
    Io(String),
    /// Compilation error
    Compile(String),
    /// Manifest error
    Manifest(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Registry(e) => write!(f, "Registry error: {}", e),
            LoadError::NotFound(p) => write!(f, "Extension not found: {}", p),
            LoadError::Io(e) => write!(f, "I/O error: {}", e),
            LoadError::Compile(e) => write!(f, "Compilation error: {}", e),
            LoadError::Manifest(e) => write!(f, "Manifest error: {}", e),
        }
    }
}

impl std::error::Error for LoadError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_source() {
        let mut loader = ExtensionLoader::default();

        let source = r#"
            mov r0, 42
            halt
        "#;

        let ext = loader.load_source("test", source).unwrap();
        assert_eq!(ext.manifest.name, "test");
        assert!(!ext.program.instructions.is_empty());
    }

    #[test]
    fn test_load_source_with_labels() {
        let mut loader = ExtensionLoader::default();

        let source = r#"
            main:
                mov r0, 10
                call add_one
                halt
            add_one:
                addi r0, r0, 1
                ret
        "#;

        let ext = loader.load_source("math", source).unwrap();
        assert!(ext.exports.contains_key("main"));
        assert!(ext.exports.contains_key("add_one"));
    }
}
