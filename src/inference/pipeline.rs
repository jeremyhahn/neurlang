//! Complete NL → x86-64 Pipeline
//!
//! This module provides the end-to-end pipeline that takes natural language
//! and produces executable native x86-64 machine code.
//!
//! # Architecture
//!
//! ```text
//! "add 5 and 3"
//!       │
//!       ▼
//! ┌─────────────────────────────────┐
//! │     MultiHeadInference          │  < 0.3ms
//! │     (ONNX or fallback)          │
//! └─────────────┬───────────────────┘
//!               │
//!         intent=0, operands=[5, 3]
//!               │
//!               ▼
//! ┌─────────────────────────────────┐
//! │     IR Generator                │  < 0.01ms
//! │     (Rust lookup table)         │
//! └─────────────┬───────────────────┘
//!               │
//!         Program { instructions: [...] }
//!               │
//!               ▼
//! ┌─────────────────────────────────┐
//! │     Copy-and-Patch Compiler     │  < 0.005ms
//! │     (Stencil-based)             │
//! └─────────────┬───────────────────┘
//!               │
//!         Native x86-64 Machine Code
//!               │
//!               ▼
//! ┌─────────────────────────────────┐
//! │     JIT Executor                │  < 0.001ms
//! └─────────────┬───────────────────┘
//!               │
//!               ▼
//!         Result: 8
//! ```

use crate::compile::{CompileError, CompiledCode, Compiler};
use crate::inference::generators::{generate_program, GeneratorError};
use crate::inference::multihead::{MultiHeadError, MultiHeadInference, MultiHeadPrediction};
use crate::ir::Program;
use std::path::Path;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Pipeline errors
#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("Inference failed: {0}")]
    Inference(#[from] MultiHeadError),
    #[error("Code generation failed: {0}")]
    Generation(#[from] GeneratorError),
    #[error("Compilation failed: {0}")]
    Compilation(#[from] CompileError),
    #[error("Execution failed: {0}")]
    Execution(String),
}

/// Pipeline execution result
#[derive(Debug)]
pub struct PipelineResult {
    /// Final result value (from R0)
    pub result: u64,
    /// Predicted intent ID
    pub intent_id: usize,
    /// Intent name
    pub intent_name: &'static str,
    /// Operands used
    pub operands: Vec<i64>,
    /// Inference latency
    pub inference_latency: Duration,
    /// Generation latency
    pub generation_latency: Duration,
    /// Compilation latency
    pub compilation_latency: Duration,
    /// Execution latency
    pub execution_latency: Duration,
    /// Total latency
    pub total_latency: Duration,
}

impl PipelineResult {
    /// Get total non-execution latency (inference + generation + compilation)
    pub fn overhead_latency(&self) -> Duration {
        self.inference_latency + self.generation_latency + self.compilation_latency
    }
}

/// Complete NL → x86-64 pipeline
pub struct NeurlangPipeline {
    /// Multi-head inference engine
    inference: MultiHeadInference,
    /// Copy-and-patch compiler
    compiler: Compiler,
}

impl NeurlangPipeline {
    /// Create pipeline with ONNX model
    pub fn new(model_path: &Path) -> Result<Self, PipelineError> {
        Ok(Self {
            inference: MultiHeadInference::load(model_path)?,
            compiler: Compiler::new(),
        })
    }

    /// Create pipeline with fallback (no ONNX model)
    pub fn fallback() -> Self {
        Self {
            inference: MultiHeadInference::mock(),
            compiler: Compiler::new(),
        }
    }

    /// Complete flow: Natural Language → Execution Result
    pub fn run(&mut self, prompt: &str) -> Result<PipelineResult, PipelineError> {
        let total_start = Instant::now();

        // Step 1: Predict intent + operands
        let prediction = self.inference.predict(prompt)?;
        let inference_latency = prediction.latency;

        // Step 2: Generate IR Program
        let gen_start = Instant::now();
        let program = generate_program(prediction.intent_id, &prediction.operands)?;
        let generation_latency = gen_start.elapsed();

        // Step 3: Compile to native x86-64
        let compile_start = Instant::now();
        let compiled = self.compiler.compile(&program)?;
        let compilation_latency = compile_start.elapsed();

        // Step 4: Execute native code
        let exec_start = Instant::now();
        let mut registers = [0u64; 32];
        let result = unsafe {
            let func = compiled.as_fn();
            func(registers.as_mut_ptr());
            registers[0]
        };
        let execution_latency = exec_start.elapsed();

        let intent_name = crate::inference::lookup::intent_name_from_id(prediction.intent_id)
            .unwrap_or("UNKNOWN");

        Ok(PipelineResult {
            result,
            intent_id: prediction.intent_id,
            intent_name,
            operands: prediction.operands,
            inference_latency,
            generation_latency,
            compilation_latency,
            execution_latency,
            total_latency: total_start.elapsed(),
        })
    }

    /// Run without execution (for testing compilation)
    pub fn compile_only(&mut self, prompt: &str) -> Result<(Program, CompiledCode), PipelineError> {
        let prediction = self.inference.predict(prompt)?;
        let program = generate_program(prediction.intent_id, &prediction.operands)?;
        let compiled = self.compiler.compile(&program)?;
        Ok((program, compiled))
    }

    /// Get prediction only (for debugging)
    pub fn predict_only(&mut self, prompt: &str) -> Result<MultiHeadPrediction, PipelineError> {
        Ok(self.inference.predict(prompt)?)
    }

    /// Check if pipeline is ready
    pub fn is_ready(&self) -> bool {
        self.inference.is_loaded()
    }
}

/// Fast path for simple expressions
///
/// Bypasses ONNX inference for common patterns.
pub struct FastPipeline {
    compiler: Compiler,
}

impl FastPipeline {
    /// Create fast pipeline
    pub fn new() -> Self {
        Self {
            compiler: Compiler::new(),
        }
    }

    /// Run with symbolic expression detection
    pub fn run(&mut self, prompt: &str) -> Result<PipelineResult, PipelineError> {
        use crate::inference::tokenizer::parse_symbolic_expression;

        let total_start = Instant::now();
        let inference_start = Instant::now();

        // Try symbolic parsing first
        let (intent_id, operands) = if let Some(expr) = parse_symbolic_expression(prompt) {
            if let Some(id) = expr.intent_id {
                (id, expr.operands)
            } else {
                return Err(PipelineError::Execution(
                    "Could not determine intent".to_string(),
                ));
            }
        } else {
            // Fall back to keyword detection
            use crate::inference::lookup::detect_intent_from_keywords;
            use crate::inference::tokenizer::extract_numbers;

            if let Some((id, _)) = detect_intent_from_keywords(prompt) {
                let ops = extract_numbers(prompt);
                (id, ops)
            } else {
                return Err(PipelineError::Execution(
                    "Could not determine intent".to_string(),
                ));
            }
        };

        let inference_latency = inference_start.elapsed();

        // Generate program
        let gen_start = Instant::now();
        let program = generate_program(intent_id, &operands)?;
        let generation_latency = gen_start.elapsed();

        // Compile
        let compile_start = Instant::now();
        let compiled = self.compiler.compile(&program)?;
        let compilation_latency = compile_start.elapsed();

        // Execute
        let exec_start = Instant::now();
        let mut registers = [0u64; 32];
        let result = unsafe {
            let func = compiled.as_fn();
            func(registers.as_mut_ptr());
            registers[0]
        };
        let execution_latency = exec_start.elapsed();

        let intent_name =
            crate::inference::lookup::intent_name_from_id(intent_id).unwrap_or("UNKNOWN");

        Ok(PipelineResult {
            result,
            intent_id,
            intent_name,
            operands,
            inference_latency,
            generation_latency,
            compilation_latency,
            execution_latency,
            total_latency: total_start.elapsed(),
        })
    }
}

impl Default for FastPipeline {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// RAG-Enhanced Pipeline with Intent Index
// ============================================================================

/// Confidence thresholds for routing decisions
pub mod confidence {
    /// High confidence: use direct generation (fast path)
    pub const HIGH: f32 = 0.7;
    /// Medium confidence: use example hints (medium path)
    pub const MEDIUM: f32 = 0.5;
    // Below MEDIUM: fallback to full model inference (slow path)
}

/// Configuration for the RAG-enhanced pipeline
#[derive(Debug, Clone)]
pub struct RagPipelineConfig {
    /// High confidence threshold (default: 0.7)
    pub high_confidence: f32,
    /// Medium confidence threshold (default: 0.5)
    pub medium_confidence: f32,
    /// Whether to use the intent index (can be disabled for A/B testing)
    pub use_intent_index: bool,
    /// Whether to use the example index for borderline cases
    pub use_example_index: bool,
    /// Number of example hints to retrieve for borderline cases
    pub example_hint_count: usize,
}

impl Default for RagPipelineConfig {
    fn default() -> Self {
        Self {
            high_confidence: confidence::HIGH,
            medium_confidence: confidence::MEDIUM,
            use_intent_index: true,
            use_example_index: true,
            example_hint_count: 3,
        }
    }
}

/// Extended pipeline result with RAG metadata
#[derive(Debug)]
pub struct RagPipelineResult {
    /// Base pipeline result
    pub base: PipelineResult,
    /// Which path was taken
    pub path: InferencePath,
    /// Intent classification confidence (from IntentIndex)
    pub confidence: f32,
    /// Embedding latency (separate from inference)
    pub embedding_latency: Duration,
}

/// Which inference path was used
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InferencePath {
    /// High confidence: direct generation from intent index
    Fast,
    /// Medium confidence: used example hints
    WithHints,
    /// Low confidence: full model inference fallback
    Fallback,
    /// Intent index disabled: legacy path
    Legacy,
}

impl std::fmt::Display for InferencePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InferencePath::Fast => write!(f, "fast"),
            InferencePath::WithHints => write!(f, "with_hints"),
            InferencePath::Fallback => write!(f, "fallback"),
            InferencePath::Legacy => write!(f, "legacy"),
        }
    }
}

/// RAG-enhanced pipeline with in-memory intent classification
///
/// This pipeline uses a pre-computed intent index for fast classification,
/// falling back to the full model only when confidence is low.
///
/// # Architecture
///
/// ```text
/// Query "add 5 and 3"
///       │
///       ▼
/// ┌─────────────────┐
/// │ FastEmbedder    │  ~0.05ms (ONNX, in-process)
/// │ (384-dim)       │
/// └────────┬────────┘
///          │
///          ▼
/// ┌─────────────────┐
/// │ IntentIndex     │  ~0.02ms (54 dot products)
/// │ classify()      │
/// └────────┬────────┘
///          │
///    confidence > 0.7?
///       /        \
///     YES         NO
///      │           │
///      ▼           ▼
/// ┌──────────┐  confidence > 0.5?
/// │ Direct   │       /        \
/// │ Generator│     YES         NO
/// │ ~0.01ms  │      │           │
/// └────┬─────┘      ▼           ▼
///      │      ┌──────────┐  ┌──────────────┐
///      │      │ Example  │  │ Full Model   │
///      │      │ Hints    │  │ ~0.3ms       │
///      │      │ ~0.1ms   │  └──────┬───────┘
///      │      └────┬─────┘         │
///      │           │               │
///      └─────┬─────┴───────────────┘
///            │
///            ▼
///      IR Generation → Compile → Execute
/// ```
pub struct RagPipeline {
    /// Configuration
    config: RagPipelineConfig,
    /// Fast embedder for query embedding
    embedder: Option<super::embedder::FastEmbedder>,
    /// Pre-computed intent index
    intent_index: Option<super::intent_index::IntentIndex>,
    /// Example index for borderline cases (optional)
    example_index: Option<super::example_index::ExampleIndex>,
    /// Full model inference (fallback)
    inference: MultiHeadInference,
    /// Copy-and-patch compiler
    compiler: Compiler,
    /// Pre-allocated embedding buffer
    embedding_buffer: [f32; 384],
}

impl RagPipeline {
    /// Create a new RAG pipeline with default configuration
    ///
    /// This creates the pipeline in "legacy mode" without intent index.
    /// Use `with_intent_index()` to enable fast path.
    pub fn new(model_path: &Path) -> Result<Self, PipelineError> {
        Ok(Self {
            config: RagPipelineConfig::default(),
            embedder: None,
            intent_index: None,
            example_index: None,
            inference: MultiHeadInference::load(model_path)?,
            compiler: Compiler::new(),
            embedding_buffer: [0.0f32; 384],
        })
    }

    /// Create with fallback (no model)
    pub fn fallback() -> Self {
        Self {
            config: RagPipelineConfig::default(),
            embedder: None,
            intent_index: None,
            example_index: None,
            inference: MultiHeadInference::mock(),
            compiler: Compiler::new(),
            embedding_buffer: [0.0f32; 384],
        }
    }

    /// Set custom configuration
    pub fn with_config(mut self, config: RagPipelineConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the fast embedder for query embedding
    pub fn with_embedder(mut self, embedder: super::embedder::FastEmbedder) -> Self {
        self.embedder = Some(embedder);
        self
    }

    /// Set the intent index for fast classification
    pub fn with_intent_index(mut self, index: super::intent_index::IntentIndex) -> Self {
        self.intent_index = Some(index);
        self
    }

    /// Set the example index for borderline cases
    pub fn with_example_index(mut self, index: super::example_index::ExampleIndex) -> Self {
        self.example_index = Some(index);
        self
    }

    /// Load intent index from file
    pub fn load_intent_index(mut self, path: &Path) -> Result<Self, PipelineError> {
        let index = super::intent_index::IntentIndex::load(path)
            .map_err(|e| PipelineError::Execution(format!("Failed to load intent index: {}", e)))?;
        self.intent_index = Some(index);
        Ok(self)
    }

    /// Check if fast path is available (embedder + intent index)
    pub fn has_fast_path(&self) -> bool {
        self.embedder.is_some() && self.intent_index.is_some() && self.config.use_intent_index
    }

    /// Run the pipeline with confidence-based routing
    pub fn run(&mut self, prompt: &str) -> Result<RagPipelineResult, PipelineError> {
        let total_start = Instant::now();

        // If fast path is not available, use legacy path
        if !self.has_fast_path() {
            return self.run_legacy(prompt, total_start);
        }

        // Step 1: Embed query
        let embed_start = Instant::now();
        let embedder = self.embedder.as_ref().unwrap();
        if let Err(e) = embedder.embed_into(prompt, &mut self.embedding_buffer) {
            // Embedding failed, fall back to legacy
            eprintln!("Warning: Embedding failed ({}), using fallback", e);
            return self.run_legacy(prompt, total_start);
        }
        let embedding_latency = embed_start.elapsed();

        // Step 2: Classify with intent index
        let intent_index = self.intent_index.as_ref().unwrap();
        let (intent_id, confidence) = intent_index.classify(&self.embedding_buffer);

        // Step 3: Route based on confidence
        if confidence >= self.config.high_confidence {
            // FAST PATH: Direct generation
            self.run_fast_path(
                prompt,
                intent_id,
                confidence,
                embedding_latency,
                total_start,
            )
        } else if confidence >= self.config.medium_confidence && self.config.use_example_index {
            // MEDIUM PATH: Use example hints
            self.run_with_hints(
                prompt,
                intent_id,
                confidence,
                embedding_latency,
                total_start,
            )
        } else {
            // SLOW PATH: Full model inference
            self.run_fallback(prompt, confidence, embedding_latency, total_start)
        }
    }

    /// Fast path: direct generation from intent ID
    fn run_fast_path(
        &mut self,
        prompt: &str,
        intent_id: usize,
        confidence: f32,
        embedding_latency: Duration,
        total_start: Instant,
    ) -> Result<RagPipelineResult, PipelineError> {
        use crate::inference::tokenizer::extract_numbers;

        let inference_start = Instant::now();

        // Extract operands from prompt
        let operands = extract_numbers(prompt);
        let inference_latency = inference_start.elapsed();

        // Generate and execute
        let (base, _) =
            self.generate_and_execute(intent_id, operands, inference_latency, total_start)?;

        Ok(RagPipelineResult {
            base,
            path: InferencePath::Fast,
            confidence,
            embedding_latency,
        })
    }

    /// Medium path: use example hints for ambiguous queries
    fn run_with_hints(
        &mut self,
        prompt: &str,
        initial_intent_id: usize,
        initial_confidence: f32,
        embedding_latency: Duration,
        total_start: Instant,
    ) -> Result<RagPipelineResult, PipelineError> {
        use crate::inference::tokenizer::extract_numbers;

        let inference_start = Instant::now();

        // Get example hints if available
        let (intent_id, confidence) = if let Some(example_index) = &mut self.example_index {
            let votes = example_index
                .search_intent_votes(&self.embedding_buffer, self.config.example_hint_count);

            // If examples agree on a different intent with higher confidence, use that
            if !votes.is_empty() {
                let top_vote = &votes[0];
                let vote_confidence = top_vote.2; // avg score from examples

                // Combine initial confidence with example votes
                // Weight: 60% intent index, 40% examples
                let combined_confidence = initial_confidence * 0.6 + vote_confidence * 0.4;

                if vote_confidence > initial_confidence && top_vote.0 as usize != initial_intent_id
                {
                    // Examples suggest different intent
                    (top_vote.0 as usize, combined_confidence)
                } else {
                    (initial_intent_id, combined_confidence)
                }
            } else {
                (initial_intent_id, initial_confidence)
            }
        } else {
            (initial_intent_id, initial_confidence)
        };

        let operands = extract_numbers(prompt);
        let inference_latency = inference_start.elapsed();

        let (base, _) =
            self.generate_and_execute(intent_id, operands, inference_latency, total_start)?;

        Ok(RagPipelineResult {
            base,
            path: InferencePath::WithHints,
            confidence,
            embedding_latency,
        })
    }

    /// Fallback path: full model inference
    fn run_fallback(
        &mut self,
        prompt: &str,
        initial_confidence: f32,
        embedding_latency: Duration,
        total_start: Instant,
    ) -> Result<RagPipelineResult, PipelineError> {
        // Use full model inference
        let prediction = self.inference.predict(prompt)?;
        let inference_latency = prediction.latency;

        let (base, _) = self.generate_and_execute(
            prediction.intent_id,
            prediction.operands,
            inference_latency,
            total_start,
        )?;

        Ok(RagPipelineResult {
            base,
            path: InferencePath::Fallback,
            confidence: initial_confidence,
            embedding_latency,
        })
    }

    /// Legacy path: no intent index
    fn run_legacy(
        &mut self,
        prompt: &str,
        total_start: Instant,
    ) -> Result<RagPipelineResult, PipelineError> {
        let prediction = self.inference.predict(prompt)?;
        let inference_latency = prediction.latency;

        let (base, _) = self.generate_and_execute(
            prediction.intent_id,
            prediction.operands,
            inference_latency,
            total_start,
        )?;

        Ok(RagPipelineResult {
            base,
            path: InferencePath::Legacy,
            confidence: 1.0, // No confidence info in legacy mode
            embedding_latency: Duration::ZERO,
        })
    }

    /// Generate IR and execute (shared by all paths)
    fn generate_and_execute(
        &mut self,
        intent_id: usize,
        operands: Vec<i64>,
        inference_latency: Duration,
        total_start: Instant,
    ) -> Result<(PipelineResult, ()), PipelineError> {
        // Generate IR Program
        let gen_start = Instant::now();
        let program = generate_program(intent_id, &operands)?;
        let generation_latency = gen_start.elapsed();

        // Compile to native x86-64
        let compile_start = Instant::now();
        let compiled = self.compiler.compile(&program)?;
        let compilation_latency = compile_start.elapsed();

        // Execute native code
        let exec_start = Instant::now();
        let mut registers = [0u64; 32];
        let result = unsafe {
            let func = compiled.as_fn();
            func(registers.as_mut_ptr());
            registers[0]
        };
        let execution_latency = exec_start.elapsed();

        let intent_name =
            crate::inference::lookup::intent_name_from_id(intent_id).unwrap_or("UNKNOWN");

        Ok((
            PipelineResult {
                result,
                intent_id,
                intent_name,
                operands,
                inference_latency,
                generation_latency,
                compilation_latency,
                execution_latency,
                total_latency: total_start.elapsed(),
            },
            (),
        ))
    }

    /// Get configuration
    pub fn config(&self) -> &RagPipelineConfig {
        &self.config
    }

    /// Check if pipeline is ready
    pub fn is_ready(&self) -> bool {
        self.inference.is_loaded() || self.has_fast_path()
    }
}

/// Benchmark the RAG pipeline
pub fn benchmark_rag_pipeline(pipeline: &mut RagPipeline, prompts: &[&str], iterations: usize) {
    println!(
        "Benchmarking RAG pipeline ({} prompts, {} iterations)...",
        prompts.len(),
        iterations
    );
    println!("Fast path available: {}", pipeline.has_fast_path());

    let mut path_counts = std::collections::HashMap::new();
    let mut total_embedding = Duration::ZERO;
    let mut total_inference = Duration::ZERO;
    let mut total_overall = Duration::ZERO;
    let mut count = 0;

    for _ in 0..iterations {
        for prompt in prompts {
            if let Ok(result) = pipeline.run(prompt) {
                *path_counts.entry(result.path).or_insert(0) += 1;
                total_embedding += result.embedding_latency;
                total_inference += result.base.inference_latency;
                total_overall += result.base.total_latency;
                count += 1;
            }
        }
    }

    if count > 0 {
        let n = count as u32;
        println!("Results ({} samples):", count);
        println!("  Avg embedding: {:?}", total_embedding / n);
        println!("  Avg inference: {:?}", total_inference / n);
        println!("  Avg total:     {:?}", total_overall / n);
        println!("Path distribution:");
        for (path, cnt) in &path_counts {
            println!(
                "  {}: {} ({:.1}%)",
                path,
                cnt,
                *cnt as f64 / count as f64 * 100.0
            );
        }
    }
}

/// Benchmark the pipeline
pub fn benchmark_pipeline(pipeline: &mut NeurlangPipeline, prompts: &[&str], iterations: usize) {
    println!(
        "Benchmarking pipeline ({} prompts, {} iterations)...",
        prompts.len(),
        iterations
    );

    let mut total_inference = Duration::ZERO;
    let mut total_generation = Duration::ZERO;
    let mut total_compilation = Duration::ZERO;
    let mut total_execution = Duration::ZERO;
    let mut total_overall = Duration::ZERO;
    let mut count = 0;

    for _ in 0..iterations {
        for prompt in prompts {
            if let Ok(result) = pipeline.run(prompt) {
                total_inference += result.inference_latency;
                total_generation += result.generation_latency;
                total_compilation += result.compilation_latency;
                total_execution += result.execution_latency;
                total_overall += result.total_latency;
                count += 1;
            }
        }
    }

    if count > 0 {
        let n = count as u32;
        println!("Average latencies ({} samples):", count);
        println!("  Inference:   {:?}", total_inference / n);
        println!("  Generation:  {:?}", total_generation / n);
        println!("  Compilation: {:?}", total_compilation / n);
        println!("  Execution:   {:?}", total_execution / n);
        println!("  Total:       {:?}", total_overall / n);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_pipeline() {
        let mut pipeline = FastPipeline::new();

        // Test arithmetic
        let result = pipeline.run("5 + 3").unwrap();
        assert_eq!(result.result, 8);
        assert_eq!(result.intent_id, 0); // ADD

        let result = pipeline.run("10 - 3").unwrap();
        assert_eq!(result.result, 7);
        assert_eq!(result.intent_id, 1); // SUB

        let result = pipeline.run("6 * 7").unwrap();
        assert_eq!(result.result, 42);
        assert_eq!(result.intent_id, 2); // MUL

        let result = pipeline.run("20 / 4").unwrap();
        assert_eq!(result.result, 5);
        assert_eq!(result.intent_id, 3); // DIV
    }

    #[test]
    fn test_fast_pipeline_math_functions() {
        let mut pipeline = FastPipeline::new();

        // Factorial
        let result = pipeline.run("5!").unwrap();
        assert_eq!(result.result, 120);
        assert_eq!(result.intent_id, 11); // FACTORIAL

        // Fibonacci
        let result = pipeline.run("fibonacci(10)").unwrap();
        assert_eq!(result.result, 55);
        assert_eq!(result.intent_id, 12); // FIBONACCI

        // GCD
        let result = pipeline.run("gcd(48, 18)").unwrap();
        assert_eq!(result.result, 6);
        assert_eq!(result.intent_id, 15); // GCD
    }

    #[test]
    fn test_fast_pipeline_latency() {
        let mut pipeline = FastPipeline::new();

        // Warm up
        let _ = pipeline.run("5 + 3");

        // Measure
        let result = pipeline.run("5 + 3").unwrap();

        // Should be well under 1ms total
        assert!(
            result.total_latency.as_millis() < 10,
            "Pipeline too slow: {:?}",
            result.total_latency
        );

        // Compilation should be under 100μs
        assert!(
            result.compilation_latency.as_micros() < 100,
            "Compilation too slow: {:?}",
            result.compilation_latency
        );
    }

    #[test]
    fn test_pipeline_result_display() {
        let mut pipeline = FastPipeline::new();
        let result = pipeline.run("5 + 3").unwrap();

        assert_eq!(result.intent_name, "ADD");
        assert_eq!(result.operands, vec![5, 3]);
        assert_eq!(result.result, 8);
    }
}
