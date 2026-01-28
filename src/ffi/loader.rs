//! Dynamic Library Loader
//!
//! Safe wrapper around libloading for loading shared libraries.

use std::collections::HashMap;
use std::ffi::CString;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use libloading::{Library, Symbol};

use super::{FfiError, FfiSignature, FfiType, FfiValue};

/// A dynamically loaded library
pub struct DynamicLibrary {
    /// Path to the library
    path: PathBuf,
    /// The loaded library handle
    library: Library,
    /// Cached function symbols
    symbols: HashMap<String, usize>,
}

impl DynamicLibrary {
    /// Load a library from the given path
    pub fn load(path: impl AsRef<Path>) -> Result<Self, FfiError> {
        let path = path.as_ref().to_path_buf();

        // Safety: We're loading a dynamic library. This is inherently unsafe
        // as the library could contain arbitrary code. However, we're trusting
        // that the user has provided a valid library path.
        let library = unsafe {
            Library::new(&path).map_err(|e| {
                FfiError::LoadError(format!(
                    "Failed to load library '{}': {}",
                    path.display(),
                    e
                ))
            })?
        };

        Ok(Self {
            path,
            library,
            symbols: HashMap::new(),
        })
    }

    /// Get the path to this library
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get a function symbol by name
    pub fn get_symbol(&mut self, name: &str) -> Result<usize, FfiError> {
        // Check cache first
        if let Some(&addr) = self.symbols.get(name) {
            return Ok(addr);
        }

        // Load the symbol
        let c_name = CString::new(name)
            .map_err(|_| FfiError::InvalidSymbol(format!("Invalid symbol name: {}", name)))?;

        // Safety: We're getting a symbol from a loaded library. The symbol
        // could be invalid or have the wrong type, but we handle type safety
        // at the call site via FfiSignature.
        let symbol: Symbol<*const ()> = unsafe {
            self.library.get(c_name.as_bytes_with_nul()).map_err(|e| {
                FfiError::SymbolNotFound(format!(
                    "Symbol '{}' not found in '{}': {}",
                    name,
                    self.path.display(),
                    e
                ))
            })?
        };

        let addr = *symbol as usize;
        self.symbols.insert(name.to_string(), addr);
        Ok(addr)
    }

    /// Call a function with the given signature and arguments
    ///
    /// # Safety
    ///
    /// This function calls into native code. The caller must ensure:
    /// - The signature matches the actual function
    /// - The arguments are valid for the function
    /// - The function doesn't violate memory safety
    pub fn call(
        &mut self,
        signature: &FfiSignature,
        args: &[FfiValue],
    ) -> Result<FfiValue, FfiError> {
        // Validate argument count
        if !signature.validate_args(args.len()) {
            return Err(FfiError::InvalidArgCount {
                expected: signature.params.len(),
                got: args.len(),
            });
        }

        // Get the function address
        let func_addr = self.get_symbol(&signature.name)?;

        // Call the function based on parameter count
        // We use a dispatch table for different arities
        let result = match args.len() {
            0 => self.call_0(func_addr, signature)?,
            1 => self.call_1(func_addr, signature, args)?,
            2 => self.call_2(func_addr, signature, args)?,
            3 => self.call_3(func_addr, signature, args)?,
            4 => self.call_4(func_addr, signature, args)?,
            5 => self.call_5(func_addr, signature, args)?,
            6 => self.call_6(func_addr, signature, args)?,
            _ => return Err(FfiError::TooManyArgs(args.len())),
        };

        Ok(result)
    }

    // Function call implementations for different arities
    // These are necessary because Rust FFI requires knowing the exact
    // number of parameters at compile time.

    fn call_0(&self, addr: usize, sig: &FfiSignature) -> Result<FfiValue, FfiError> {
        type Fn0 = extern "C" fn() -> u64;
        let f: Fn0 = unsafe { std::mem::transmute(addr) };
        let result = f();
        Ok(self.convert_result(result, sig.return_type))
    }

    fn call_1(
        &self,
        addr: usize,
        sig: &FfiSignature,
        args: &[FfiValue],
    ) -> Result<FfiValue, FfiError> {
        type Fn1 = extern "C" fn(u64) -> u64;
        let f: Fn1 = unsafe { std::mem::transmute(addr) };
        let result = f(args[0].to_u64());
        Ok(self.convert_result(result, sig.return_type))
    }

    fn call_2(
        &self,
        addr: usize,
        sig: &FfiSignature,
        args: &[FfiValue],
    ) -> Result<FfiValue, FfiError> {
        type Fn2 = extern "C" fn(u64, u64) -> u64;
        let f: Fn2 = unsafe { std::mem::transmute(addr) };
        let result = f(args[0].to_u64(), args[1].to_u64());
        Ok(self.convert_result(result, sig.return_type))
    }

    fn call_3(
        &self,
        addr: usize,
        sig: &FfiSignature,
        args: &[FfiValue],
    ) -> Result<FfiValue, FfiError> {
        type Fn3 = extern "C" fn(u64, u64, u64) -> u64;
        let f: Fn3 = unsafe { std::mem::transmute(addr) };
        let result = f(args[0].to_u64(), args[1].to_u64(), args[2].to_u64());
        Ok(self.convert_result(result, sig.return_type))
    }

    fn call_4(
        &self,
        addr: usize,
        sig: &FfiSignature,
        args: &[FfiValue],
    ) -> Result<FfiValue, FfiError> {
        type Fn4 = extern "C" fn(u64, u64, u64, u64) -> u64;
        let f: Fn4 = unsafe { std::mem::transmute(addr) };
        let result = f(
            args[0].to_u64(),
            args[1].to_u64(),
            args[2].to_u64(),
            args[3].to_u64(),
        );
        Ok(self.convert_result(result, sig.return_type))
    }

    fn call_5(
        &self,
        addr: usize,
        sig: &FfiSignature,
        args: &[FfiValue],
    ) -> Result<FfiValue, FfiError> {
        type Fn5 = extern "C" fn(u64, u64, u64, u64, u64) -> u64;
        let f: Fn5 = unsafe { std::mem::transmute(addr) };
        let result = f(
            args[0].to_u64(),
            args[1].to_u64(),
            args[2].to_u64(),
            args[3].to_u64(),
            args[4].to_u64(),
        );
        Ok(self.convert_result(result, sig.return_type))
    }

    fn call_6(
        &self,
        addr: usize,
        sig: &FfiSignature,
        args: &[FfiValue],
    ) -> Result<FfiValue, FfiError> {
        type Fn6 = extern "C" fn(u64, u64, u64, u64, u64, u64) -> u64;
        let f: Fn6 = unsafe { std::mem::transmute(addr) };
        let result = f(
            args[0].to_u64(),
            args[1].to_u64(),
            args[2].to_u64(),
            args[3].to_u64(),
            args[4].to_u64(),
            args[5].to_u64(),
        );
        Ok(self.convert_result(result, sig.return_type))
    }

    fn convert_result(&self, value: u64, return_type: FfiType) -> FfiValue {
        match return_type {
            FfiType::Void => FfiValue::Void,
            FfiType::U8
            | FfiType::U16
            | FfiType::U32
            | FfiType::U64
            | FfiType::I8
            | FfiType::I16
            | FfiType::I32
            | FfiType::I64 => FfiValue::Integer(value),
            FfiType::F32 => {
                let bits = value as u32;
                FfiValue::Float(f32::from_bits(bits) as f64)
            }
            FfiType::F64 => FfiValue::Float(f64::from_bits(value)),
            FfiType::Ptr | FfiType::CStr | FfiType::Buffer => FfiValue::Pointer(value as usize),
        }
    }
}

/// Library loader with search paths
pub struct LibraryLoader {
    /// Search paths for libraries
    search_paths: Vec<PathBuf>,
    /// Loaded libraries
    libraries: HashMap<String, Arc<std::sync::RwLock<DynamicLibrary>>>,
}

impl LibraryLoader {
    /// Create a new library loader
    pub fn new() -> Self {
        Self {
            search_paths: default_search_paths(),
            libraries: HashMap::new(),
        }
    }

    /// Add a search path
    pub fn add_search_path(&mut self, path: impl AsRef<Path>) {
        self.search_paths.push(path.as_ref().to_path_buf());
    }

    /// Find a library by name
    pub fn find_library(&self, name: &str) -> Option<PathBuf> {
        // If it's already a path, check if it exists
        let path = Path::new(name);
        if path.exists() {
            return Some(path.to_path_buf());
        }

        // Construct library filename based on platform
        let lib_name = library_filename(name);

        // Search in search paths
        for search_path in &self.search_paths {
            let full_path = search_path.join(&lib_name);
            if full_path.exists() {
                return Some(full_path);
            }
        }

        None
    }

    /// Load a library by name
    pub fn load(&mut self, name: &str) -> Result<Arc<std::sync::RwLock<DynamicLibrary>>, FfiError> {
        // Check if already loaded
        if let Some(lib) = self.libraries.get(name) {
            return Ok(Arc::clone(lib));
        }

        // Find the library
        let path = self
            .find_library(name)
            .ok_or_else(|| FfiError::LoadError(format!("Library '{}' not found", name)))?;

        // Load it
        let library = DynamicLibrary::load(&path)?;
        let lib = Arc::new(std::sync::RwLock::new(library));
        self.libraries.insert(name.to_string(), Arc::clone(&lib));

        Ok(lib)
    }

    /// Get a loaded library
    pub fn get(&self, name: &str) -> Option<Arc<std::sync::RwLock<DynamicLibrary>>> {
        self.libraries.get(name).cloned()
    }

    /// Unload a library
    pub fn unload(&mut self, name: &str) -> bool {
        self.libraries.remove(name).is_some()
    }

    /// List loaded libraries
    pub fn loaded_libraries(&self) -> Vec<&str> {
        self.libraries.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for LibraryLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the default library search paths for this platform
fn default_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Current directory
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd);
    }

    // Standard system paths
    #[cfg(target_os = "linux")]
    {
        paths.push(PathBuf::from("/usr/lib"));
        paths.push(PathBuf::from("/usr/local/lib"));
        paths.push(PathBuf::from("/lib"));
        paths.push(PathBuf::from("/lib64"));
        paths.push(PathBuf::from("/usr/lib64"));

        // LD_LIBRARY_PATH
        if let Ok(ld_path) = std::env::var("LD_LIBRARY_PATH") {
            for p in ld_path.split(':') {
                paths.push(PathBuf::from(p));
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from("/usr/lib"));
        paths.push(PathBuf::from("/usr/local/lib"));
        paths.push(PathBuf::from("/opt/homebrew/lib"));

        // DYLD_LIBRARY_PATH
        if let Ok(dyld_path) = std::env::var("DYLD_LIBRARY_PATH") {
            for p in dyld_path.split(':') {
                paths.push(PathBuf::from(p));
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        paths.push(PathBuf::from("C:\\Windows\\System32"));

        // PATH
        if let Ok(path) = std::env::var("PATH") {
            for p in path.split(';') {
                paths.push(PathBuf::from(p));
            }
        }
    }

    paths
}

/// Construct the platform-specific library filename
fn library_filename(name: &str) -> String {
    #[cfg(target_os = "linux")]
    {
        if name.starts_with("lib") && name.ends_with(".so") {
            name.to_string()
        } else {
            format!("lib{}.so", name)
        }
    }

    #[cfg(target_os = "macos")]
    {
        if name.starts_with("lib") && name.ends_with(".dylib") {
            name.to_string()
        } else {
            format!("lib{}.dylib", name)
        }
    }

    #[cfg(target_os = "windows")]
    {
        if name.ends_with(".dll") {
            name.to_string()
        } else {
            format!("{}.dll", name)
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        name.to_string()
    }
}
