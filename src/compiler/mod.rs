//! Rust→Neurlang IR Compiler
//!
//! Compiles a subset of Rust source code to Neurlang IR assembly.
//! This enables writing stdlib functions in Rust with full testing,
//! then generating verified .nl assembly files.
//!
//! # Supported Rust Subset
//!
//! - Integer types: u64, i64, u8
//! - Floating point: f64 (compiles to FPU opcodes)
//! - Control flow: if/else, while, loop, for i in 0..n
//! - Functions with simple parameter types
//! - Local variables (let, let mut)
//! - Basic arithmetic, bitwise, and comparison operations
//!
//! # Example
//!
//! ```rust,ignore
//! use neurlang::compiler::{RustCompiler, CompilerConfig};
//!
//! let source = r#"
//!     pub fn factorial(n: u64) -> u64 {
//!         let mut result = 1u64;
//!         let mut i = n;
//!         while i > 0 {
//!             result = result * i;
//!             i = i - 1;
//!         }
//!         result
//!     }
//! "#;
//!
//! let compiler = RustCompiler::new(CompilerConfig::default());
//! let nl_source = compiler.compile(source)?;
//! ```

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod test_gen;
pub mod verify;

use std::path::Path;
use thiserror::Error;

pub use analyzer::{AnalyzeError, AnalyzedFunction, Analyzer, Variable};
pub use codegen::{CodeGenerator, GeneratedFunction};
pub use parser::{FunctionParam, ParsedFunction, ParsedModule, TypeInfo};
pub use test_gen::{TestCase, TestGenerator};

/// Configuration for the Rust→IR compiler.
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// Include source comments in generated .nl files
    pub include_comments: bool,
    /// Generate @test annotations from doc comments
    pub generate_tests: bool,
    /// Category prefix for generated functions
    pub category: Option<String>,
    /// Maximum allowed instructions per function
    pub max_instructions: usize,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            include_comments: true,
            generate_tests: true,
            category: None,
            max_instructions: 1000,
        }
    }
}

/// Errors that can occur during compilation.
#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Analysis error: {0}")]
    Analysis(#[from] AnalyzeError),

    #[error("Code generation error: {0}")]
    CodeGen(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported Rust feature: {0}")]
    Unsupported(String),
}

/// Result type for compiler operations.
pub type CompilerResult<T> = Result<T, CompilerError>;

/// The main Rust→Neurlang IR compiler.
pub struct RustCompiler {
    config: CompilerConfig,
}

impl RustCompiler {
    /// Create a new compiler with the given configuration.
    pub fn new(config: CompilerConfig) -> Self {
        Self { config }
    }

    /// Compile Rust source code to Neurlang assembly.
    pub fn compile(&self, source: &str) -> CompilerResult<String> {
        // Phase 1: Parse
        let module =
            parser::parse_module(source).map_err(|e| CompilerError::Parse(e.to_string()))?;

        // Phase 2: Analyze each function
        let mut analyzer = Analyzer::new();
        let mut analyzed_functions = Vec::new();

        for func in &module.functions {
            let analyzed = analyzer.analyze(func)?;
            analyzed_functions.push(analyzed);
        }

        // Phase 3: Generate code
        let mut codegen = CodeGenerator::new();
        let mut output = String::new();

        for analyzed in &analyzed_functions {
            let generated = codegen.generate(analyzed, &self.config)?;
            output.push_str(&generated.to_nl_source(&self.config));
            output.push_str("\n\n");
        }

        Ok(output)
    }

    /// Compile a single Rust function to Neurlang assembly.
    pub fn compile_function(&self, source: &str) -> CompilerResult<String> {
        // Wrap in a module for parsing
        let module_source = format!("mod temp {{ {} }}", source);
        self.compile(&module_source)
    }

    /// Compile all .rs files in a directory to .nl files.
    pub fn compile_directory(&self, src_dir: &Path, out_dir: &Path) -> CompilerResult<Vec<String>> {
        let mut generated_files = Vec::new();

        for entry in std::fs::read_dir(src_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "rs") {
                if path
                    .file_name()
                    .is_some_and(|name| name == "lib.rs" || name == "mod.rs")
                {
                    continue; // Skip module files
                }

                let source = std::fs::read_to_string(&path)?;
                let nl_source = self.compile(&source)?;

                let stem = path.file_stem().unwrap().to_str().unwrap();
                let out_path = out_dir.join(format!("{}.nl", stem));

                std::fs::create_dir_all(out_path.parent().unwrap())?;
                std::fs::write(&out_path, nl_source)?;

                generated_files.push(out_path.to_string_lossy().to_string());
            }
        }

        Ok(generated_files)
    }
}

/// Build the stdlib from Rust sources.
///
/// This is the entry point for `nl stdlib build`.
pub fn build_stdlib(
    stdlib_dir: &Path,
    lib_dir: &Path,
    verbose: bool,
) -> CompilerResult<BuildResult> {
    let config = CompilerConfig {
        include_comments: true,
        generate_tests: true,
        category: Some("stdlib".to_string()),
        max_instructions: 1000,
    };

    let compiler = RustCompiler::new(config);
    let src_dir = stdlib_dir.join("src");

    let mut result = BuildResult {
        files_compiled: 0,
        functions_generated: 0,
        tests_generated: 0,
        errors: Vec::new(),
    };

    // Process each module
    for module_name in &["math", "float", "string", "array", "bitwise", "collections"] {
        let src_path = src_dir.join(format!("{}.rs", module_name));
        if !src_path.exists() {
            continue;
        }

        let out_dir = lib_dir.join(module_name);
        std::fs::create_dir_all(&out_dir)?;

        if verbose {
            println!("Compiling {}...", module_name);
        }

        match std::fs::read_to_string(&src_path) {
            Ok(source) => {
                match compiler.compile(&source) {
                    Ok(_nl_source) => {
                        // Parse individual functions and write separate files
                        let module = parser::parse_module(&source)
                            .map_err(|e| CompilerError::Parse(e.to_string()))?;

                        for func in &module.functions {
                            let mut analyzer = Analyzer::new();
                            let analyzed = analyzer.analyze(func)?;

                            let mut codegen = CodeGenerator::new();
                            let generated = codegen.generate(&analyzed, &compiler.config)?;

                            let out_path = out_dir.join(format!("{}.nl", func.name));
                            std::fs::write(&out_path, generated.to_nl_source(&compiler.config))?;

                            result.functions_generated += 1;
                            result.tests_generated += generated.tests.len();

                            if verbose {
                                println!(
                                    "  Generated {} ({} instructions, {} tests)",
                                    func.name,
                                    generated.instructions.len(),
                                    generated.tests.len()
                                );
                            }
                        }

                        result.files_compiled += 1;
                    }
                    Err(e) => {
                        result.errors.push(format!("{}: {}", module_name, e));
                    }
                }
            }
            Err(e) => {
                result.errors.push(format!("{}: {}", module_name, e));
            }
        }
    }

    Ok(result)
}

/// Result of building the stdlib.
#[derive(Debug)]
pub struct BuildResult {
    pub files_compiled: usize,
    pub functions_generated: usize,
    pub tests_generated: usize,
    pub errors: Vec<String>,
}

impl BuildResult {
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Verify that Rust and Neurlang implementations produce the same output.
///
/// This is the entry point for `nl stdlib verify`.
pub fn verify_stdlib(
    _stdlib_dir: &Path,
    lib_dir: &Path,
    verbose: bool,
) -> CompilerResult<VerifyResult> {
    let stats = verify::verify_all(lib_dir, verbose).map_err(CompilerError::CodeGen)?;

    let failures: Vec<String> = stats
        .results
        .iter()
        .filter(|r| !r.passed)
        .map(|r| {
            format!(
                "{}({:?}): Rust={}, Neurlang={}",
                r.function, r.inputs, r.rust_output, r.neurlang_output
            )
        })
        .collect();

    Ok(VerifyResult {
        functions_verified: stats.functions_verified,
        tests_passed: stats.passed,
        tests_failed: stats.failed,
        failures,
    })
}

/// Result of verifying stdlib.
#[derive(Debug)]
pub struct VerifyResult {
    pub functions_verified: usize,
    pub tests_passed: usize,
    pub tests_failed: usize,
    pub failures: Vec<String>,
}

impl VerifyResult {
    pub fn is_success(&self) -> bool {
        self.tests_failed == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_function() {
        let source = r#"
            pub fn add(a: u64, b: u64) -> u64 {
                a + b
            }
        "#;

        let compiler = RustCompiler::new(CompilerConfig::default());
        let result = compiler.compile(source);

        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
        let nl_source = result.unwrap();
        assert!(nl_source.contains("add"));
    }

    #[test]
    fn test_compile_factorial() {
        let source = r#"
            pub fn factorial(n: u64) -> u64 {
                let mut result: u64 = 1;
                let mut i: u64 = n;
                while i > 0 {
                    result = result * i;
                    i = i - 1;
                }
                result
            }
        "#;

        let compiler = RustCompiler::new(CompilerConfig::default());
        let result = compiler.compile(source);

        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
    }
}
