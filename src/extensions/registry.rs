//! Extension Registry
//!
//! Manages installed extensions and their metadata.
//!
//! # RAG Integration
//!
//! The registry can register all installed extensions with a RAG resolver
//! for semantic intent-based lookup. User extensions are assigned IDs
//! starting from 500.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::manifest::{ExtensionManifest, ManifestError};
use crate::ir::rag_resolver::RagResolver;

/// Extension source type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionSource {
    /// Locally created extension
    Local,
    /// Installed from git repository
    Git {
        url: String,
        version: Option<String>,
    },
}

/// Information about an installed extension
#[derive(Debug, Clone)]
pub struct ExtensionInfo {
    /// Extension manifest
    pub manifest: ExtensionManifest,
    /// Installation path
    pub path: PathBuf,
    /// Source of the extension
    pub source: ExtensionSource,
}

/// Extension registry for managing installed packages
pub struct ExtensionRegistry {
    /// Base directory for extensions
    base_dir: PathBuf,
    /// Installed extensions (import path -> info)
    extensions: HashMap<String, ExtensionInfo>,
}

impl ExtensionRegistry {
    /// Create a new registry with the default base directory
    pub fn new() -> Result<Self, RegistryError> {
        let base_dir = Self::default_base_dir()?;
        Self::with_base_dir(base_dir)
    }

    /// Create a registry with a custom base directory
    pub fn with_base_dir(base_dir: PathBuf) -> Result<Self, RegistryError> {
        // Create directories if they don't exist
        let local_dir = base_dir.join("local");
        let cache_dir = base_dir.join("cache");

        fs::create_dir_all(&local_dir).map_err(|e| RegistryError::Io(e.to_string()))?;
        fs::create_dir_all(&cache_dir).map_err(|e| RegistryError::Io(e.to_string()))?;

        let mut registry = Self {
            base_dir,
            extensions: HashMap::new(),
        };

        // Scan and load existing extensions
        registry.scan()?;

        Ok(registry)
    }

    /// Get the default base directory
    fn default_base_dir() -> Result<PathBuf, RegistryError> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| RegistryError::NoHomeDir)?;

        Ok(PathBuf::from(home).join(".neurlang").join("extensions"))
    }

    /// Scan for installed extensions
    pub fn scan(&mut self) -> Result<(), RegistryError> {
        self.extensions.clear();

        // Scan local extensions
        let local_dir = self.base_dir.join("local");
        if local_dir.exists() {
            self.scan_directory(&local_dir, ExtensionSource::Local)?;
        }

        // Scan cached extensions
        let cache_dir = self.base_dir.join("cache");
        if cache_dir.exists() {
            self.scan_cache_directory(&cache_dir)?;
        }

        Ok(())
    }

    /// Scan a directory for extensions
    fn scan_directory(&mut self, dir: &Path, source: ExtensionSource) -> Result<(), RegistryError> {
        for entry in fs::read_dir(dir).map_err(|e| RegistryError::Io(e.to_string()))? {
            let entry = entry.map_err(|e| RegistryError::Io(e.to_string()))?;
            let path = entry.path();

            if path.is_dir() {
                let manifest_path = path.join("neurlang.json");
                if manifest_path.exists() {
                    if let Ok(manifest) = ExtensionManifest::load(&manifest_path) {
                        let import_path = match &source {
                            ExtensionSource::Local => format!("local/{}", manifest.name),
                            ExtensionSource::Git { url, version } => {
                                if let Some(v) = version {
                                    format!("{}@{}", url, v)
                                } else {
                                    url.clone()
                                }
                            }
                        };

                        self.extensions.insert(
                            import_path,
                            ExtensionInfo {
                                manifest,
                                path: path.clone(),
                                source: source.clone(),
                            },
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Scan the cache directory (nested structure like github.com/user/repo)
    fn scan_cache_directory(&mut self, dir: &Path) -> Result<(), RegistryError> {
        // Walk through host/user/repo structure
        for host_entry in fs::read_dir(dir).map_err(|e| RegistryError::Io(e.to_string()))? {
            let host_entry = host_entry.map_err(|e| RegistryError::Io(e.to_string()))?;
            let host_path = host_entry.path();

            if !host_path.is_dir() {
                continue;
            }

            let host_name = host_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            for user_entry in
                fs::read_dir(&host_path).unwrap_or_else(|_| fs::read_dir(".").unwrap())
            {
                let user_entry = match user_entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                let user_path = user_entry.path();

                if !user_path.is_dir() {
                    continue;
                }

                let user_name = user_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                for repo_entry in
                    fs::read_dir(&user_path).unwrap_or_else(|_| fs::read_dir(".").unwrap())
                {
                    let repo_entry = match repo_entry {
                        Ok(e) => e,
                        Err(_) => continue,
                    };
                    let repo_path = repo_entry.path();

                    if !repo_path.is_dir() {
                        continue;
                    }

                    let repo_name = repo_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    // Parse version from repo name (e.g., "lib@v1.0.0")
                    let (name, version) = if let Some((n, v)) = repo_name.split_once('@') {
                        (n.to_string(), Some(v.to_string()))
                    } else {
                        (repo_name.to_string(), None)
                    };

                    let manifest_path = repo_path.join("neurlang.json");
                    if manifest_path.exists() {
                        if let Ok(manifest) = ExtensionManifest::load(&manifest_path) {
                            let url = format!("{}/{}/{}", host_name, user_name, name);
                            let import_path = if let Some(ref v) = version {
                                format!("{}@{}", url, v)
                            } else {
                                url.clone()
                            };

                            self.extensions.insert(
                                import_path,
                                ExtensionInfo {
                                    manifest,
                                    path: repo_path.clone(),
                                    source: ExtensionSource::Git { url, version },
                                },
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Install an extension from a git URL
    pub fn install(&mut self, url: &str) -> Result<ExtensionInfo, RegistryError> {
        // Parse URL and version
        let (base_url, version) = if let Some((u, v)) = url.split_once('@') {
            (u.to_string(), Some(v.to_string()))
        } else {
            (url.to_string(), None)
        };

        // Construct cache path
        let cache_path = self.cache_path_for_url(&base_url, version.as_deref());

        // Check if already installed
        if cache_path.exists() {
            let manifest_path = cache_path.join("neurlang.json");
            if manifest_path.exists() {
                let manifest = ExtensionManifest::load(&manifest_path)
                    .map_err(RegistryError::ManifestError)?;

                let info = ExtensionInfo {
                    manifest,
                    path: cache_path,
                    source: ExtensionSource::Git {
                        url: base_url,
                        version,
                    },
                };

                return Ok(info);
            }
        }

        // Clone from git
        fs::create_dir_all(cache_path.parent().unwrap())
            .map_err(|e| RegistryError::Io(e.to_string()))?;

        let git_url = format!("https://{}", base_url);
        let mut cmd = Command::new("git");
        cmd.args(["clone", "--depth", "1"]);

        if let Some(ref v) = version {
            cmd.args(["--branch", v]);
        }

        cmd.args([&git_url, cache_path.to_str().unwrap()]);

        let output = cmd
            .output()
            .map_err(|e| RegistryError::GitError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RegistryError::GitError(stderr.to_string()));
        }

        // Load manifest
        let manifest_path = cache_path.join("neurlang.json");
        if !manifest_path.exists() {
            // Clean up and error
            let _ = fs::remove_dir_all(&cache_path);
            return Err(RegistryError::NoManifest);
        }

        let manifest =
            ExtensionManifest::load(&manifest_path).map_err(RegistryError::ManifestError)?;

        let info = ExtensionInfo {
            manifest,
            path: cache_path,
            source: ExtensionSource::Git {
                url: base_url,
                version,
            },
        };

        // Add to registry
        let import_path = url.to_string();
        self.extensions.insert(import_path, info.clone());

        Ok(info)
    }

    /// Create a new local extension
    pub fn create_local(&mut self, name: &str) -> Result<ExtensionInfo, RegistryError> {
        let local_path = self.base_dir.join("local").join(name);

        if local_path.exists() {
            return Err(RegistryError::AlreadyExists(name.to_string()));
        }

        fs::create_dir_all(&local_path).map_err(|e| RegistryError::Io(e.to_string()))?;

        // Create manifest
        let manifest = ExtensionManifest::new(name, "0.1.0");
        let manifest_path = local_path.join("neurlang.json");
        manifest
            .save(&manifest_path)
            .map_err(RegistryError::ManifestError)?;

        // Create empty entry point
        let entry_path = local_path.join("main.nl");
        fs::write(&entry_path, "; Empty extension\nhalt\n")
            .map_err(|e| RegistryError::Io(e.to_string()))?;

        let info = ExtensionInfo {
            manifest,
            path: local_path,
            source: ExtensionSource::Local,
        };

        let import_path = format!("local/{}", name);
        self.extensions.insert(import_path, info.clone());

        Ok(info)
    }

    /// Remove an extension
    pub fn remove(&mut self, import_path: &str) -> Result<(), RegistryError> {
        if let Some(info) = self.extensions.remove(import_path) {
            fs::remove_dir_all(&info.path).map_err(|e| RegistryError::Io(e.to_string()))?;
            Ok(())
        } else {
            Err(RegistryError::NotFound(import_path.to_string()))
        }
    }

    /// Get an extension by import path
    pub fn get(&self, import_path: &str) -> Option<&ExtensionInfo> {
        self.extensions.get(import_path)
    }

    /// List all installed extensions
    pub fn list(&self) -> impl Iterator<Item = (&String, &ExtensionInfo)> {
        self.extensions.iter()
    }

    /// Get the cache path for a URL
    fn cache_path_for_url(&self, url: &str, version: Option<&str>) -> PathBuf {
        let cache_dir = self.base_dir.join("cache");

        // url is like "github.com/user/repo"
        let path = if let Some(v) = version {
            // Split into parts and add version to last part
            let parts: Vec<&str> = url.split('/').collect();
            if parts.len() >= 3 {
                let versioned_name = format!("{}@{}", parts[2], v);
                cache_dir.join(parts[0]).join(parts[1]).join(versioned_name)
            } else {
                cache_dir.join(format!("{}@{}", url.replace('/', "_"), v))
            }
        } else {
            cache_dir.join(url.replace('/', std::path::MAIN_SEPARATOR_STR))
        };

        path
    }

    /// Register all installed extensions with a RAG resolver
    ///
    /// User extensions are assigned IDs starting from 500.
    /// Each exported function is registered with its description for
    /// intent-based lookup.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let registry = ExtensionRegistry::new()?;
    /// let mut rag = RagResolver::new();
    /// let count = registry.register_with_rag(&mut rag);
    /// println!("Registered {} user extension functions", count);
    /// ```
    pub fn register_with_rag(&self, rag: &mut RagResolver) -> usize {
        const USER_EXT_BASE_ID: u32 = 500;
        let mut next_id = USER_EXT_BASE_ID;
        let mut count = 0;

        for info in self.extensions.values() {
            for export in &info.manifest.exports {
                // Build description from export info
                let description = if export.description.is_empty() {
                    // Fall back to function name if no description
                    format!("{} from {}", export.name, info.manifest.name)
                } else {
                    export.description.clone()
                };

                // Register with RAG resolver
                rag.register_extension(
                    next_id,
                    &format!("{}:{}", info.manifest.name, export.name),
                    &description,
                    export.inputs.len(),
                );

                next_id += 1;
                count += 1;
            }
        }

        count
    }

    /// Get extension info and export by RAG-resolved ID
    ///
    /// For user extensions (ID >= 500), this looks up which extension
    /// and export corresponds to the ID.
    pub fn get_by_rag_id(
        &self,
        id: u32,
    ) -> Option<(&ExtensionInfo, &super::manifest::ExtensionExport)> {
        const USER_EXT_BASE_ID: u32 = 500;

        if id < USER_EXT_BASE_ID {
            // Bundled extension, not in user registry
            return None;
        }

        let mut current_id = USER_EXT_BASE_ID;
        for info in self.extensions.values() {
            for export in &info.manifest.exports {
                if current_id == id {
                    return Some((info, export));
                }
                current_id += 1;
            }
        }

        None
    }

    /// Get the base directory for extensions
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Get the number of installed extensions
    pub fn len(&self) -> usize {
        self.extensions.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.extensions.is_empty()
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            base_dir: PathBuf::from(".neurlang/extensions"),
            extensions: HashMap::new(),
        })
    }
}

/// Registry error types
#[derive(Debug)]
pub enum RegistryError {
    /// I/O error
    Io(String),
    /// Git operation failed
    GitError(String),
    /// Manifest error
    ManifestError(ManifestError),
    /// Extension not found
    NotFound(String),
    /// Extension already exists
    AlreadyExists(String),
    /// No manifest in extension
    NoManifest,
    /// No home directory found
    NoHomeDir,
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::Io(e) => write!(f, "I/O error: {}", e),
            RegistryError::GitError(e) => write!(f, "Git error: {}", e),
            RegistryError::ManifestError(e) => write!(f, "Manifest error: {}", e),
            RegistryError::NotFound(p) => write!(f, "Extension not found: {}", p),
            RegistryError::AlreadyExists(n) => write!(f, "Extension already exists: {}", n),
            RegistryError::NoManifest => write!(f, "No neurlang.json found in extension"),
            RegistryError::NoHomeDir => write!(f, "Could not determine home directory"),
        }
    }
}

impl std::error::Error for RegistryError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_registry_creation() {
        let temp = temp_dir().join("neurlang_test_registry");
        let _ = fs::remove_dir_all(&temp);

        let _registry = ExtensionRegistry::with_base_dir(temp.clone()).unwrap();

        assert!(temp.join("local").exists());
        assert!(temp.join("cache").exists());

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_local_extension_creation() {
        let temp = temp_dir().join("neurlang_test_local");
        let _ = fs::remove_dir_all(&temp);

        let mut registry = ExtensionRegistry::with_base_dir(temp.clone()).unwrap();

        let info = registry.create_local("my-test-ext").unwrap();
        assert_eq!(info.manifest.name, "my-test-ext");
        assert!(info.path.join("neurlang.json").exists());
        assert!(info.path.join("main.nl").exists());

        // Should be findable
        let found = registry.get("local/my-test-ext");
        assert!(found.is_some());

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_cache_path_generation() {
        let temp = temp_dir().join("neurlang_test_cache_path");
        let _registry = ExtensionRegistry::with_base_dir(temp.clone()).unwrap_or_default();

        let path1 = _registry.cache_path_for_url("github.com/user/repo", None);
        assert!(path1.to_string_lossy().contains("github.com"));
        assert!(path1.to_string_lossy().contains("user"));
        assert!(path1.to_string_lossy().contains("repo"));

        let path2 = _registry.cache_path_for_url("github.com/user/repo", Some("v1.0.0"));
        assert!(path2.to_string_lossy().contains("repo@v1.0.0"));

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_rag_registration() {
        use crate::extensions::manifest::{ExtensionExport, ParamType};

        let temp = temp_dir().join("neurlang_test_rag");
        let _ = fs::remove_dir_all(&temp);

        let mut registry = ExtensionRegistry::with_base_dir(temp.clone()).unwrap();

        // Create a local extension with exports
        let info = registry.create_local("math-utils").unwrap();

        // Update the manifest with exports
        let mut manifest = info.manifest.clone();
        manifest.add_export(
            ExtensionExport::new("factorial")
                .with_input(ParamType::Int)
                .with_output(ParamType::Int)
                .with_description("Calculate factorial of a number"),
        );
        manifest.add_export(
            ExtensionExport::new("fibonacci")
                .with_input(ParamType::Int)
                .with_output(ParamType::Int)
                .with_description("Calculate nth Fibonacci number"),
        );
        manifest.save(info.path.join("neurlang.json")).unwrap();

        // Re-scan to pick up changes
        registry.scan().unwrap();

        // Register with RAG
        let mut rag = RagResolver::new();
        let count = registry.register_with_rag(&mut rag);
        assert_eq!(count, 2);

        // Should be able to find via intent
        let resolved = rag.resolve("calculate factorial").unwrap();
        assert_eq!(resolved.id, 500); // First user extension ID

        let resolved = rag.resolve("fibonacci number").unwrap();
        assert_eq!(resolved.id, 501); // Second user extension ID

        // Should be able to look up by ID
        let (ext_info, export) = registry.get_by_rag_id(500).unwrap();
        assert_eq!(ext_info.manifest.name, "math-utils");
        assert_eq!(export.name, "factorial");

        let _ = fs::remove_dir_all(&temp);
    }
}
