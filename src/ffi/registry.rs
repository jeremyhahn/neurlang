//! FFI Registry
//!
//! Central registry for FFI libraries and functions.

use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::sync::{Arc, RwLock};

use super::loader::{DynamicLibrary, LibraryLoader};
use super::types::{FfiSignature, FfiType, FfiValue};

/// Error type for FFI operations
#[derive(Debug, Clone)]
pub enum FfiError {
    /// Failed to load a library
    LoadError(String),
    /// Symbol not found in library
    SymbolNotFound(String),
    /// Invalid symbol name
    InvalidSymbol(String),
    /// Library not found
    LibraryNotFound(String),
    /// Function not found
    FunctionNotFound(String),
    /// Invalid argument count
    InvalidArgCount { expected: usize, got: usize },
    /// Invalid argument type
    InvalidArgType { expected: FfiType, got: FfiType },
    /// Too many arguments
    TooManyArgs(usize),
    /// FFI call failed
    CallFailed(String),
    /// Type conversion error
    ConversionError(String),
}

impl fmt::Display for FfiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FfiError::LoadError(msg) => write!(f, "Load error: {}", msg),
            FfiError::SymbolNotFound(msg) => write!(f, "Symbol not found: {}", msg),
            FfiError::InvalidSymbol(msg) => write!(f, "Invalid symbol: {}", msg),
            FfiError::LibraryNotFound(name) => write!(f, "Library not found: {}", name),
            FfiError::FunctionNotFound(name) => write!(f, "Function not found: {}", name),
            FfiError::InvalidArgCount { expected, got } => {
                write!(
                    f,
                    "Invalid argument count: expected {}, got {}",
                    expected, got
                )
            }
            FfiError::InvalidArgType { expected, got } => {
                write!(
                    f,
                    "Invalid argument type: expected {}, got {}",
                    expected, got
                )
            }
            FfiError::TooManyArgs(count) => {
                write!(f, "Too many arguments: {} (max 6)", count)
            }
            FfiError::CallFailed(msg) => write!(f, "FFI call failed: {}", msg),
            FfiError::ConversionError(msg) => write!(f, "Type conversion error: {}", msg),
        }
    }
}

impl std::error::Error for FfiError {}

/// Information about an FFI function
#[derive(Debug, Clone)]
pub struct FfiFunctionInfo {
    /// Library name
    pub library: String,
    /// Function signature
    pub signature: FfiSignature,
    /// Description
    pub description: String,
    /// Keywords for RAG search
    pub keywords: Vec<String>,
}

impl FfiFunctionInfo {
    /// Create a new function info
    pub fn new(
        library: impl Into<String>,
        signature: FfiSignature,
        description: impl Into<String>,
    ) -> Self {
        Self {
            library: library.into(),
            signature,
            description: description.into(),
            keywords: Vec::new(),
        }
    }

    /// Add keywords for RAG search
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }
}

/// FFI Registry - central hub for FFI libraries and functions
pub struct FfiRegistry {
    /// Library loader
    loader: LibraryLoader,
    /// Registered functions (qualified_name -> function_info)
    functions: HashMap<String, FfiFunctionInfo>,
    /// Keywords index for RAG search
    keywords_index: HashMap<String, Vec<String>>,
}

impl FfiRegistry {
    /// Create a new FFI registry
    pub fn new() -> Self {
        Self {
            loader: LibraryLoader::new(),
            functions: HashMap::new(),
            keywords_index: HashMap::new(),
        }
    }

    /// Add a search path for libraries
    pub fn add_search_path(&mut self, path: impl AsRef<Path>) {
        self.loader.add_search_path(path);
    }

    /// Load a library from a path or search for it by name
    pub fn load_library(&mut self, name: &str, path: Option<&str>) -> Result<(), FfiError> {
        let path = path.unwrap_or(name);
        self.loader.load(path)?;
        Ok(())
    }

    /// Register a function from a loaded library
    pub fn register_function(&mut self, info: FfiFunctionInfo) -> Result<(), FfiError> {
        // Verify the library is loaded
        if self.loader.get(&info.library).is_none() {
            return Err(FfiError::LibraryNotFound(info.library.clone()));
        }

        // Create qualified name
        let qualified_name = format!("{}:{}", info.library, info.signature.name);

        // Index keywords
        for keyword in &info.keywords {
            self.keywords_index
                .entry(keyword.to_lowercase())
                .or_default()
                .push(qualified_name.clone());
        }

        // Store function info
        self.functions.insert(qualified_name, info);

        Ok(())
    }

    /// Call a function by qualified name (library:function)
    pub fn call(&mut self, qualified_name: &str, args: &[u64]) -> Result<u64, FfiError> {
        // Parse qualified name
        let parts: Vec<&str> = qualified_name.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(FfiError::FunctionNotFound(qualified_name.to_string()));
        }

        let library_name = parts[0];
        let _function_name = parts[1];

        // Get function info
        let info = self
            .functions
            .get(qualified_name)
            .ok_or_else(|| FfiError::FunctionNotFound(qualified_name.to_string()))?
            .clone();

        // Get library
        let library = self
            .loader
            .get(library_name)
            .ok_or_else(|| FfiError::LibraryNotFound(library_name.to_string()))?;

        // Convert arguments
        let ffi_args: Vec<FfiValue> = args
            .iter()
            .zip(info.signature.params.iter())
            .map(|(&v, &t)| FfiValue::from_u64(v, t))
            .collect();

        // Call function
        let mut lib = library
            .write()
            .map_err(|e| FfiError::CallFailed(format!("Failed to lock library: {}", e)))?;

        let result = lib.call(&info.signature, &ffi_args)?;

        Ok(result.to_u64())
    }

    /// Search for functions by keyword (RAG-style)
    pub fn search(&self, query: &str) -> Vec<&FfiFunctionInfo> {
        let query_lower = query.to_lowercase();
        let words: Vec<&str> = query_lower.split_whitespace().collect();

        // Score each function by keyword overlap
        let mut scores: HashMap<&str, f32> = HashMap::new();

        for word in &words {
            // Exact keyword match
            if let Some(names) = self.keywords_index.get(*word) {
                for name in names {
                    *scores.entry(name.as_str()).or_default() += 1.0;
                }
            }

            // Partial match in description
            for (name, info) in &self.functions {
                if info.description.to_lowercase().contains(*word) {
                    *scores.entry(name.as_str()).or_default() += 0.5;
                }
            }
        }

        // Sort by score and return
        let mut results: Vec<_> = scores
            .into_iter()
            .filter(|(_, score)| *score >= 0.5)
            .collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        results
            .into_iter()
            .filter_map(|(name, _)| self.functions.get(name))
            .collect()
    }

    /// Get a function by qualified name
    pub fn get_function(&self, qualified_name: &str) -> Option<&FfiFunctionInfo> {
        self.functions.get(qualified_name)
    }

    /// List all registered functions
    pub fn list_functions(&self) -> Vec<&FfiFunctionInfo> {
        self.functions.values().collect()
    }

    /// List all loaded libraries
    pub fn list_libraries(&self) -> Vec<&str> {
        self.loader.loaded_libraries()
    }
}

impl Default for FfiRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Go Library Support
// =============================================================================

/// Helper for loading Go libraries built with cgo
///
/// Go libraries must be built with cgo to export C-compatible functions:
/// ```go
/// package main
///
/// import "C"
///
/// //export Add
/// func Add(a, b C.int) C.int {
///     return a + b
/// }
///
/// func main() {}
/// ```
///
/// Build with: `go build -buildmode=c-shared -o libmylib.so`
#[allow(dead_code)]
pub struct GoLibrary {
    /// The underlying dynamic library
    library: Arc<RwLock<DynamicLibrary>>,
    /// Library name
    name: String,
}

#[allow(dead_code)]
impl GoLibrary {
    /// Load a Go library
    pub fn load(name: &str, path: impl AsRef<Path>) -> Result<Self, FfiError> {
        let library = DynamicLibrary::load(path)?;
        Ok(Self {
            library: Arc::new(RwLock::new(library)),
            name: name.to_string(),
        })
    }

    /// Get the library name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Call a function exported from Go
    pub fn call(&self, signature: &FfiSignature, args: &[FfiValue]) -> Result<FfiValue, FfiError> {
        let mut lib = self
            .library
            .write()
            .map_err(|e| FfiError::CallFailed(format!("Failed to lock library: {}", e)))?;
        lib.call(signature, args)
    }

    /// Register all exported functions with an FFI registry
    ///
    /// Note: Go doesn't provide a standard way to enumerate exports,
    /// so functions must be registered manually or via a manifest file.
    pub fn register_with(
        &self,
        registry: &mut FfiRegistry,
        functions: Vec<FfiFunctionInfo>,
    ) -> Result<(), FfiError> {
        for info in functions {
            registry
                .functions
                .insert(format!("{}:{}", self.name, info.signature.name), info);
        }
        Ok(())
    }
}

// =============================================================================
// C Library Support
// =============================================================================

/// Helper for loading C libraries
#[allow(dead_code)]
pub struct CLibrary {
    /// The underlying dynamic library
    library: Arc<RwLock<DynamicLibrary>>,
    /// Library name
    name: String,
}

#[allow(dead_code)]
impl CLibrary {
    /// Load a C library
    pub fn load(name: &str, path: impl AsRef<Path>) -> Result<Self, FfiError> {
        let library = DynamicLibrary::load(path)?;
        Ok(Self {
            library: Arc::new(RwLock::new(library)),
            name: name.to_string(),
        })
    }

    /// Get the library name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Call a C function
    pub fn call(&self, signature: &FfiSignature, args: &[FfiValue]) -> Result<FfiValue, FfiError> {
        let mut lib = self
            .library
            .write()
            .map_err(|e| FfiError::CallFailed(format!("Failed to lock library: {}", e)))?;
        lib.call(signature, args)
    }

    /// Register functions from a header file (basic parsing)
    ///
    /// This is a simple parser for C function declarations.
    /// For complex headers, use a proper C parser like bindgen.
    pub fn register_from_header(
        &self,
        _registry: &mut FfiRegistry,
        _header_content: &str,
    ) -> Result<(), FfiError> {
        // TODO: Implement basic C header parsing
        // For now, functions must be registered manually
        Ok(())
    }

    /// Register a single function
    pub fn register_function(
        &self,
        registry: &mut FfiRegistry,
        info: FfiFunctionInfo,
    ) -> Result<(), FfiError> {
        registry
            .functions
            .insert(format!("{}:{}", self.name, info.signature.name), info);
        Ok(())
    }
}
