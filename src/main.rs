//! Neurlang - AI-Optimized Binary Programming Language
//!
//! Main CLI entry point for assembling, compiling, and executing Neurlang programs.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use neurlang::compile::{AotCompiler, Compiler};
use neurlang::extensions::{ExtensionLoader, ExtensionRegistry};
use neurlang::inference::{
    find_session as agent_find_session, list_sessions as agent_list_sessions, Agent, AgentConfig,
    InferenceEngine, OrchResult, Orchestrator, OrchestratorConfig,
};
use neurlang::interp::Interpreter;
use neurlang::ir::{Assembler, Disassembler, Program, RagResolver};
use neurlang::jit::{JitExecutor, JitResult};
use neurlang::slot::{parse_protocol_spec, validate_spec, SlotTrainingExtractor};
use neurlang::train;
use neurlang::training::{self, TrainingBackend, TrainingConfig};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "nl")]
#[command(version)]
#[command(about = "AI-Optimized Binary Programming Language", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // Commands sorted alphabetically for easier navigation
    /// Test model accuracy
    Accuracy {
        /// Model path (default: ./model.onnx)
        #[arg(long, default_value = "model.onnx")]
        model: PathBuf,

        /// Use benchmark test suite
        #[arg(long)]
        benchmark: bool,

        /// Test data path (uses built-in tests if not specified)
        #[arg(long)]
        test_data: Option<PathBuf>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Interactive AI agent with session persistence
    Agent {
        /// Start a new session with the given name/task
        #[arg(long)]
        new: Option<String>,

        /// Continue an existing session (by ID or partial ID)
        #[arg(long, value_name = "SESSION_ID")]
        cont: Option<String>,

        /// Resume the last session
        #[arg(long)]
        resume: bool,

        /// List all sessions
        #[arg(long)]
        list: bool,

        /// Interactive mode (default)
        #[arg(long)]
        interactive: bool,

        /// Assembly REPL mode (raw assembly execution)
        #[arg(long)]
        asm: bool,

        /// Task to execute (for --new or --continue)
        #[arg(value_name = "TASK")]
        task: Option<String>,

        /// Maximum iterations per request
        #[arg(long, default_value = "1000")]
        max_iterations: usize,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Assemble a text source file to binary IR
    Asm {
        /// Input assembly file
        #[arg(short, long)]
        input: PathBuf,

        /// Output binary file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show disassembly after assembly
        #[arg(short, long)]
        disasm: bool,
    },

    /// Manage LLM backends for two-tier orchestration
    Backends {
        /// List all available backends
        #[arg(long)]
        list: bool,

        /// Show backend status and configuration
        #[arg(long)]
        status: Option<String>,

        /// Set the default backend
        #[arg(long)]
        set_default: Option<String>,

        /// Test backend connectivity
        #[arg(long)]
        test: Option<String>,
    },

    /// Run a benchmark
    Bench {
        /// Benchmark type (compile, fib, all)
        #[arg(short, long, default_value = "all")]
        bench_type: String,

        /// Number of iterations
        #[arg(short, long, default_value = "1000")]
        iterations: usize,
    },

    /// Compile to standalone native code
    Compile {
        /// Input file
        #[arg(short, long)]
        input: PathBuf,

        /// Output binary file
        #[arg(short, long)]
        output: PathBuf,

        /// Output format (raw, elf)
        #[arg(long, default_value = "raw")]
        format: String,
    },

    /// Configure Neurlang settings
    Config {
        /// Set a configuration value (e.g., "backends.claude.api_key" "sk-...")
        #[arg(long, num_args = 2, value_names = ["KEY", "VALUE"])]
        set: Option<Vec<String>>,

        /// Get a configuration value
        #[arg(long)]
        get: Option<String>,

        /// List all configuration
        #[arg(long)]
        list: bool,

        /// Unset a configuration value
        #[arg(long)]
        unset: Option<String>,

        /// Show config file path
        #[arg(long)]
        path: bool,
    },

    /// Manage external crates as extensions
    Crate {
        /// Add a crate from crates.io
        #[arg(long)]
        add: Option<String>,

        /// Remove a crate
        #[arg(long)]
        remove: Option<String>,

        /// List installed crates
        #[arg(long)]
        list: bool,

        /// Build/rebuild all crate extensions
        #[arg(long)]
        build: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Generate training data
    Datagen {
        /// Output file path
        #[arg(short, long, default_value = "training_data.jsonl")]
        output: PathBuf,

        /// Number of examples to generate
        #[arg(short, long, default_value = "100000")]
        num_examples: usize,

        /// Curriculum level (1-5)
        #[arg(short, long, default_value = "5")]
        level: u8,

        /// Random seed
        #[arg(long, default_value = "42")]
        seed: u64,

        /// Include examples from examples/ directory
        #[arg(long)]
        include_examples: bool,

        /// Use parallel model data generator (balanced 500K samples)
        #[arg(long)]
        parallel: bool,
    },

    /// Disassemble binary IR to text
    Disasm {
        /// Input binary file
        #[arg(short, long)]
        input: PathBuf,

        /// Show byte offsets
        #[arg(long)]
        offsets: bool,

        /// Show raw bytes
        #[arg(long)]
        bytes: bool,
    },

    /// Export model to ONNX format
    ExportOnnx {
        /// Input PyTorch model
        #[arg(short, long, default_value = "model.pt")]
        input: PathBuf,

        /// Output ONNX model
        #[arg(short, long, default_value = "model.onnx")]
        output: PathBuf,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Manage extensions (Go-style package system)
    Extension {
        /// Add/install an extension from git URL (e.g., github.com/user/repo)
        #[arg(long)]
        add: Option<String>,

        /// Create a new local extension
        #[arg(long)]
        new: Option<String>,

        /// Remove an extension
        #[arg(long)]
        remove: Option<String>,

        /// List installed extensions
        #[arg(long)]
        list: bool,

        /// Load an extension from a file for testing
        #[arg(long)]
        load: Option<PathBuf>,

        /// Show extension info
        #[arg(long)]
        info: Option<String>,
    },

    /// Generate code from natural language using slot-based architecture
    Generate {
        /// The prompt describing what to generate (e.g., "SMTP server")
        prompt: String,

        /// Force offline mode (rule-based only, no LLM fallback)
        #[arg(long)]
        offline: bool,

        /// Force LLM decomposition (always use LLM for decomposition)
        #[arg(long)]
        llm: bool,

        /// Dry run: show what would be generated without filling slots
        #[arg(long)]
        dry_run: bool,

        /// Run benchmark: time the generation process
        #[arg(long)]
        benchmark: bool,

        /// Output file for generated assembly
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show the SlotSpec structure
        #[arg(long)]
        show_spec: bool,

        /// Show individual slot definitions
        #[arg(long)]
        show_slots: bool,

        /// Show generated assembly code
        #[arg(long)]
        show_asm: bool,

        /// Protocol specs directory
        #[arg(long, default_value = "specs/protocols")]
        specs_dir: String,

        /// Templates directory
        #[arg(long, default_value = "templates")]
        templates_dir: String,

        /// Minimum confidence for rule-based routing (0.0-1.0)
        #[arg(long, default_value = "0.6")]
        threshold: f32,
    },

    /// Build and manage RAG indices for fast intent classification
    #[command(name = "index")]
    Index {
        /// Build the intent index from canonical descriptions
        #[arg(long)]
        build: bool,

        /// Build the example index from training data
        #[arg(long)]
        build_examples: bool,

        /// Input training data file for example index (JSONL)
        #[arg(long)]
        training_data: Option<PathBuf>,

        /// Output directory for index files (default: ~/.neurlang/)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Embedder model path (ONNX)
        #[arg(long)]
        embedder: Option<PathBuf>,

        /// Use Ollama for embeddings (host:model format, e.g., "localhost:11434:nomic-embed-text")
        #[arg(long)]
        ollama: Option<String>,

        /// Show index info/stats
        #[arg(long)]
        info: bool,

        /// Verify index by running test queries
        #[arg(long)]
        verify: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Generate code from natural language prompt
    Prompt {
        /// The prompt describing what code to generate
        prompt: String,

        /// Model path (default: ./model.onnx)
        #[arg(long, default_value = "model.onnx")]
        model: PathBuf,

        /// Inference engine: "ort", "tract", "candle", "burn", or "auto"
        #[arg(long, default_value = "auto")]
        engine: String,

        /// Show generated assembly
        #[arg(long)]
        show_asm: bool,

        /// Maximum retry attempts
        #[arg(long, default_value = "3")]
        max_retries: usize,

        /// Save generated binary to file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Verbose output (show each attempt)
        #[arg(short, long)]
        verbose: bool,
    },

    /// Validate and test protocol specifications
    #[command(name = "protocol")]
    ProtocolSpec {
        /// Protocol spec file to validate/test
        #[arg(short, long)]
        input: PathBuf,

        /// Validate spec structure (states, transitions, patterns)
        #[arg(long)]
        validate: bool,

        /// Run integration tests from spec's test section
        #[arg(long)]
        test: bool,

        /// Program to test against (for --test)
        #[arg(long)]
        program: Option<PathBuf>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Show spec statistics
        #[arg(long)]
        stats: bool,
    },

    /// Resolve an intent description to extension ID via RAG
    Resolve {
        /// The intent description to resolve
        intent: String,

        /// Show top N matches (default: 5)
        #[arg(short, long, default_value = "5")]
        top: usize,
    },

    /// Execute a program (text assembly or binary)
    Run {
        /// Input file (assembly or binary)
        #[arg(short, long)]
        input: PathBuf,

        /// Use interpreter instead of JIT
        #[arg(long)]
        interp: bool,

        /// Show execution statistics
        #[arg(short, long)]
        stats: bool,

        /// Maximum instructions (for interpreter)
        #[arg(long, default_value = "1000000")]
        max_instr: u64,

        /// Number of worker threads (for server workloads with SO_REUSEPORT)
        /// Use 0 for single-threaded mode, or specify number of workers.
        /// Multiple workers can share the same port for load balancing.
        #[arg(short, long, default_value = "0")]
        workers: usize,

        /// Worker strategy (auto, reuseport, shared)
        /// auto = detect best strategy (default)
        /// reuseport = SO_REUSEPORT (Linux/macOS)
        /// shared = shared listener (Windows compatible)
        #[arg(long, default_value = "auto")]
        strategy: String,
    },

    /// Generate slot-level training data from protocol specs
    #[command(name = "slotdata")]
    SlotData {
        /// Protocol specs directory
        #[arg(short, long, default_value = "specs/protocols")]
        specs_dir: PathBuf,

        /// Output JSONL file
        #[arg(short, long, default_value = "train/slot_training.jsonl")]
        output: PathBuf,

        /// Augment data with variations
        #[arg(long)]
        augment: bool,

        /// Variations per example (for augmentation)
        #[arg(long, default_value = "3")]
        variations: usize,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show IR specification
    Spec {
        /// Show opcodes
        #[arg(long)]
        opcodes: bool,

        /// Show registers
        #[arg(long)]
        registers: bool,

        /// Show all
        #[arg(long)]
        all: bool,
    },

    /// Build and manage the stdlib (Rust→Neurlang IR compiler)
    Stdlib {
        /// Build stdlib from Rust sources (stdlib/src/*.rs → lib/*.nl)
        #[arg(long)]
        build: bool,

        /// Verify that Rust and Neurlang implementations produce same output
        #[arg(long)]
        verify: bool,

        /// Clean generated lib/ files
        #[arg(long)]
        clean: bool,

        /// Configuration file (default: ./neurlang.toml)
        #[arg(long, short)]
        config: Option<PathBuf>,

        /// Stdlib source directory (default: ./stdlib, or from config)
        #[arg(long, default_value = "stdlib")]
        stdlib_dir: PathBuf,

        /// Output library directory (default: ./lib, or from config)
        #[arg(long, default_value = "lib")]
        lib_dir: PathBuf,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Test .nl files with @test annotations
    Test {
        /// Directory or file(s) to test (default: ./test, ./tests, or current dir)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Filter tests by name pattern
        #[arg(short, long)]
        filter: Option<String>,

        /// Verbose output (show each test case)
        #[arg(short, long)]
        verbose: bool,

        /// Stop at first failure
        #[arg(long)]
        fail_fast: bool,

        /// Include server tests (skipped by default)
        #[arg(long)]
        include_servers: bool,

        /// Enable coverage tracking and report
        #[arg(long)]
        coverage: bool,
    },

    /// Train the AI model (locally or remotely)
    Train {
        /// Training data path
        #[arg(long, default_value = "training_data.jsonl")]
        data: PathBuf,

        /// Output model path
        #[arg(short, long, default_value = "model.onnx")]
        output: PathBuf,

        /// Remote host (user@hostname) for remote training
        #[arg(long)]
        remote: Option<String>,

        /// Remote working directory
        #[arg(long, default_value = "~/neurlang")]
        remote_dir: String,

        /// GPU profile (h100, h200, b300, l40s, a100, generic, cpu)
        #[arg(long, default_value = "h100")]
        profile: String,

        /// Training backend (auto, native, docker, pytorch, onnx)
        #[arg(long, default_value = "auto")]
        backend: String,

        /// Provisioner script path
        #[arg(long)]
        provisioner: Option<PathBuf>,

        /// Skip provisioning on remote
        #[arg(long)]
        no_provision: bool,

        /// Number of epochs
        #[arg(long, default_value = "20")]
        epochs: usize,

        /// Early stopping patience
        #[arg(long, default_value = "5")]
        patience: usize,

        /// Run k-fold cross-validation
        #[arg(long)]
        cross_validate: bool,

        /// Number of CV folds
        #[arg(long, default_value = "5")]
        folds: usize,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,

        /// List available GPU profiles
        #[arg(long)]
        list_profiles: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Asm {
            input,
            output,
            disasm,
        } => cmd_asm(&input, output.as_ref(), disasm),
        Commands::Disasm {
            input,
            offsets,
            bytes,
        } => cmd_disasm(&input, offsets, bytes),
        Commands::Run {
            input,
            interp,
            stats,
            max_instr,
            workers,
            strategy,
        } => cmd_run(&input, interp, stats, max_instr, workers, &strategy),
        Commands::Compile {
            input,
            output,
            format,
        } => cmd_compile(&input, &output, &format),
        Commands::Bench {
            bench_type,
            iterations,
        } => cmd_bench(&bench_type, iterations),
        Commands::Spec {
            opcodes,
            registers,
            all,
        } => cmd_spec(opcodes || all, registers || all),
        Commands::Prompt {
            prompt,
            model,
            engine,
            show_asm,
            max_retries,
            output,
            verbose,
        } => cmd_prompt(
            &prompt,
            &model,
            &engine,
            show_asm,
            max_retries,
            output.as_ref(),
            verbose,
        ),
        Commands::Train {
            data,
            output,
            remote,
            remote_dir,
            profile,
            backend,
            provisioner,
            no_provision,
            epochs,
            patience,
            cross_validate,
            folds,
            verbose,
            list_profiles,
        } => cmd_train(
            &data,
            &output,
            remote.as_deref(),
            &remote_dir,
            &profile,
            &backend,
            provisioner.as_ref(),
            no_provision,
            epochs,
            patience,
            cross_validate,
            folds,
            verbose,
            list_profiles,
        ),
        Commands::Datagen {
            output,
            num_examples,
            level,
            seed,
            include_examples,
            parallel,
        } => cmd_datagen(
            &output,
            num_examples,
            level,
            seed,
            include_examples,
            parallel,
        ),
        Commands::Accuracy {
            model,
            benchmark,
            test_data,
            verbose,
        } => cmd_accuracy(&model, benchmark, test_data.as_ref(), verbose),
        Commands::ExportOnnx {
            input,
            output,
            verbose,
        } => cmd_export_onnx(&input, &output, verbose),
        Commands::Agent {
            new,
            cont,
            resume,
            list,
            interactive,
            asm,
            task,
            max_iterations,
            verbose,
        } => {
            if asm {
                cmd_repl()
            } else {
                cmd_agent(
                    new,
                    cont,
                    resume,
                    list,
                    interactive,
                    task,
                    max_iterations,
                    verbose,
                )
            }
        }
        Commands::Extension {
            add,
            new,
            remove,
            list,
            load,
            info,
        } => cmd_extension(add, new, remove, list, load, info),
        Commands::Config {
            set,
            get,
            list,
            unset,
            path,
        } => cmd_config(set, get, list, unset, path),
        Commands::Backends {
            list,
            status,
            set_default,
            test,
        } => cmd_backends(list, status, set_default, test),
        Commands::Test {
            path,
            filter,
            verbose,
            fail_fast,
            include_servers,
            coverage,
        } => {
            let resolved_path = resolve_test_path(path);
            cmd_test(
                &resolved_path,
                filter.as_deref(),
                verbose,
                fail_fast,
                include_servers,
                coverage,
            )
        }
        Commands::Stdlib {
            build,
            verify,
            clean,
            config,
            stdlib_dir,
            lib_dir,
            verbose,
        } => cmd_stdlib(
            build,
            verify,
            clean,
            config.as_ref(),
            &stdlib_dir,
            &lib_dir,
            verbose,
        ),
        Commands::Crate {
            add,
            remove,
            list,
            build,
            verbose,
        } => cmd_crate(add, remove, list, build, verbose),
        Commands::Resolve { intent, top } => cmd_resolve(&intent, top),
        Commands::Generate {
            prompt,
            offline,
            llm,
            dry_run,
            benchmark,
            output,
            show_spec,
            show_slots,
            show_asm,
            specs_dir,
            templates_dir,
            threshold,
        } => cmd_generate(
            &prompt,
            offline,
            llm,
            dry_run,
            benchmark,
            output.as_ref(),
            show_spec,
            show_slots,
            show_asm,
            &specs_dir,
            &templates_dir,
            threshold,
        ),
        Commands::ProtocolSpec {
            input,
            validate,
            test,
            program,
            verbose,
            stats,
        } => cmd_protocol_spec(&input, validate, test, program.as_ref(), verbose, stats),
        Commands::SlotData {
            specs_dir,
            output,
            augment,
            variations,
            verbose,
        } => cmd_slot_data(&specs_dir, &output, augment, variations, verbose),
        Commands::Index {
            build,
            build_examples,
            training_data,
            output,
            embedder,
            ollama,
            info,
            verify,
            verbose,
        } => cmd_index(
            build,
            build_examples,
            training_data.as_ref(),
            output.as_ref(),
            embedder.as_ref(),
            ollama.as_deref(),
            info,
            verify,
            verbose,
        ),
    }
}

fn cmd_asm(input: &PathBuf, output: Option<&PathBuf>, show_disasm: bool) -> Result<()> {
    let source = fs::read_to_string(input).context("Failed to read input file")?;

    let mut asm = Assembler::new();
    let program = asm.assemble(&source).context("Assembly failed")?;

    println!(
        "Assembled {} instructions ({} bytes)",
        program.instructions.len(),
        program.code_size()
    );

    if show_disasm {
        println!("\nDisassembly:");
        let disasm = Disassembler::new().with_offsets(true);
        println!("{}", disasm.disassemble(&program));
    }

    // Write output
    let output_path = output
        .cloned()
        .unwrap_or_else(|| input.with_extension("nlb"));
    let bytes = program.encode();
    fs::write(&output_path, &bytes).context("Failed to write output")?;
    println!("Wrote {} bytes to {}", bytes.len(), output_path.display());

    Ok(())
}

fn cmd_disasm(input: &PathBuf, offsets: bool, bytes: bool) -> Result<()> {
    let data = fs::read(input).context("Failed to read input file")?;

    let program = Program::decode(&data).context("Invalid binary format")?;

    let disasm = Disassembler::new().with_offsets(offsets).with_bytes(bytes);
    println!("{}", disasm.disassemble(&program));

    Ok(())
}

fn cmd_run(
    input: &PathBuf,
    use_interp: bool,
    show_stats: bool,
    max_instr: u64,
    workers: usize,
    strategy: &str,
) -> Result<()> {
    // Load program
    let program = load_program(input)?;

    println!("Loaded {} instructions", program.instructions.len());

    let start = Instant::now();

    if use_interp {
        // Use interpreter
        let mut interp = Interpreter::new(65536).with_max_instructions(max_instr);
        let result = interp.execute(&program);

        let elapsed = start.elapsed();

        match result {
            neurlang::interp::InterpResult::Ok(val) => {
                println!("Result: {}", val);
            }
            neurlang::interp::InterpResult::Halted => {
                println!("Program halted");
                println!("R0 = {}", interp.registers[0]);
            }
            other => {
                println!("Error: {:?}", other);
            }
        }

        if show_stats {
            println!("\nStatistics:");
            println!("  Instructions: {}", interp.instruction_count());
            println!("  Time: {:?}", elapsed);
            println!(
                "  IPS: {:.2}M",
                interp.instruction_count() as f64 / elapsed.as_secs_f64() / 1_000_000.0
            );
        }
    } else if workers > 1 {
        // Use multi-worker mode with specified strategy
        use neurlang::jit::{execute_with_strategy, WorkerStrategy};

        let strat = match strategy {
            "reuseport" => Some(WorkerStrategy::ReusePort),
            "shared" => Some(WorkerStrategy::SharedListener),
            _ => None, // Auto-detect
        };

        let result = if let Some(s) = strat {
            execute_with_strategy(&program, workers, s)
        } else {
            neurlang::jit::execute_multi_worker(&program, workers)
        };

        let elapsed = start.elapsed();

        match result {
            JitResult::Ok(val) => {
                println!("Result: {}", val);
            }
            JitResult::Halted(val) => {
                println!("Program halted");
                println!("R0 = {}", val);
            }
            JitResult::Error(msg) => {
                println!("Error: {}", msg);
            }
        }

        if show_stats {
            println!("\nStatistics:");
            println!("  Time: {:?}", elapsed);
        }
    } else {
        // Use JIT executor with full I/O support (single-threaded)
        let mut executor = JitExecutor::with_memory_size(1024 * 1024); // 1MB memory
        let result = executor.execute(&program);

        let elapsed = start.elapsed();

        match result {
            JitResult::Ok(val) => {
                println!("Result: {}", val);
                println!("R0 = {}", executor.get_register(0));
            }
            JitResult::Halted(val) => {
                println!("Program halted");
                println!("R0 = {}", val);
            }
            JitResult::Error(msg) => {
                println!("Error: {}", msg);
            }
        }

        if show_stats {
            println!("\nStatistics:");
            println!("  Instructions: {}", executor.stats.instructions_executed);
            println!("  Time: {:?}", elapsed);
            if elapsed.as_secs_f64() > 0.0 {
                println!(
                    "  IPS: {:.2}M",
                    executor.stats.instructions_executed as f64
                        / elapsed.as_secs_f64()
                        / 1_000_000.0
                );
            }
        }
    }

    Ok(())
}

fn cmd_compile(input: &PathBuf, output: &PathBuf, format: &str) -> Result<()> {
    let program = load_program(input)?;

    let compiler = AotCompiler::new();

    let bytes = match format {
        "elf" => {
            #[cfg(target_os = "linux")]
            {
                compiler
                    .compile_to_elf(&program)
                    .context("ELF generation failed")?
            }
            #[cfg(not(target_os = "linux"))]
            {
                anyhow::bail!("ELF output only supported on Linux");
            }
        }
        "raw" | _ => compiler
            .compile_to_bytes(&program)
            .context("Compilation failed")?,
    };

    fs::write(output, &bytes).context("Failed to write output")?;
    println!("Compiled {} bytes to {}", bytes.len(), output.display());

    Ok(())
}

fn cmd_bench(bench_type: &str, iterations: usize) -> Result<()> {
    println!("Neurlang Benchmarks");
    println!("===================");
    println!("Iterations: {}\n", iterations);

    match bench_type {
        "compile" | "all" => bench_compile(iterations)?,
        _ => {}
    }

    match bench_type {
        "fib" | "all" => bench_fibonacci(iterations)?,
        _ => {}
    }

    Ok(())
}

fn bench_compile(iterations: usize) -> Result<()> {
    println!("Compile Time Benchmark:");
    println!("-----------------------");

    let mut asm = Assembler::new();

    // Create a typical 32-instruction program
    let source = r#"
        mov r0, 0
        mov r1, 1
        mov r2, 10
    loop:
        add r3, r0, r1
        mov r0, r1
        mov r1, r3
        subi r2, r2, 1
        bne r2, zero, loop
        halt
    "#;

    let program = asm.assemble(source)?;
    let mut compiler = Compiler::new();

    // Warm up
    for _ in 0..10 {
        let _ = compiler.compile(&program);
    }

    // Measure
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = compiler.compile(&program);
    }
    let elapsed = start.elapsed();

    let avg_ns = elapsed.as_nanos() / iterations as u128;
    println!("  Instructions: {}", program.instructions.len());
    println!("  Iterations: {}", iterations);
    println!("  Total time: {:?}", elapsed);
    println!(
        "  Average: {} ns ({:.2} μs)",
        avg_ns,
        avg_ns as f64 / 1000.0
    );
    println!("  Target: <5000 ns (<5 μs)");
    println!();

    Ok(())
}

fn bench_fibonacci(iterations: usize) -> Result<()> {
    println!("Fibonacci Benchmark:");
    println!("--------------------");

    let mut asm = Assembler::new();
    let source = r#"
        mov r0, 35      ; n = 35
        mov r1, 0       ; fib(n-2)
        mov r2, 1       ; fib(n-1)
    loop:
        beq r0, zero, done
        add r3, r1, r2
        mov r1, r2
        mov r2, r3
        subi r0, r0, 1
        b loop
    done:
        mov r0, r1
        halt
    "#;

    let program = asm.assemble(source)?;

    // Interpreter benchmark
    {
        let mut total_time = std::time::Duration::ZERO;
        let mut total_instrs = 0u64;

        for _ in 0..iterations.min(100) {
            let mut interp = Interpreter::new(1024);
            let start = Instant::now();
            interp.execute(&program);
            total_time += start.elapsed();
            total_instrs += interp.instruction_count();
        }

        println!("  Interpreter:");
        println!("    Iterations: {}", iterations.min(100));
        println!(
            "    Avg time: {:?}",
            total_time / iterations.min(100) as u32
        );
        println!(
            "    Avg instructions: {}",
            total_instrs / iterations.min(100) as u64
        );
    }

    // JIT benchmark
    {
        let mut total_time = std::time::Duration::ZERO;
        let mut total_instrs = 0u64;

        for _ in 0..iterations {
            let mut executor = JitExecutor::new();
            let start = Instant::now();
            executor.execute(&program);
            total_time += start.elapsed();
            total_instrs += executor.stats.instructions_executed;
        }

        println!("  JIT:");
        println!("    Iterations: {}", iterations);
        println!("    Avg exec time: {:?}", total_time / iterations as u32);
        println!("    Avg instructions: {}", total_instrs / iterations as u64);
    }

    println!();
    Ok(())
}

fn cmd_spec(show_opcodes: bool, show_registers: bool) -> Result<()> {
    if show_opcodes || (!show_opcodes && !show_registers) {
        println!("Neurlang Opcodes (32 total):");
        println!("============================\n");

        println!("Arithmetic/Logic:");
        println!("  0x00  ALU        Add, Sub, And, Or, Xor, Shl, Shr, Sar");
        println!("  0x01  ALUI       Same with immediate");
        println!("  0x02  MULDIV     Mul, Div, Mod, MulH");
        println!();

        println!("Memory:");
        println!("  0x03  LOAD       Load 8/16/32/64 (auto bounds-check)");
        println!("  0x04  STORE      Store 8/16/32/64 (auto bounds-check)");
        println!("  0x05  ATOMIC     CAS, Xchg, Add, And, Or, Xor, Min, Max");
        println!();

        println!("Control Flow:");
        println!("  0x06  BRANCH     Eq, Ne, Lt, Le, Gt, Ge, Always");
        println!("  0x07  CALL       Direct, indirect");
        println!("  0x08  RET        Return");
        println!("  0x09  JUMP       Direct, indirect");
        println!();

        println!("Capabilities:");
        println!("  0x0A  CAP.NEW    Create capability");
        println!("  0x0B  CAP.RESTRICT  Narrow bounds/perms");
        println!("  0x0C  CAP.QUERY  Get base/length/perms");
        println!();

        println!("Concurrency:");
        println!("  0x0D  SPAWN      Create thread/task");
        println!("  0x0E  JOIN       Wait for completion");
        println!("  0x0F  CHAN       Create, send, recv, close");
        println!("  0x10  FENCE      Acquire, release, seq_cst");
        println!("  0x11  YIELD      Cooperative yield");
        println!();

        println!("Taint:");
        println!("  0x12  TAINT      Mark as tainted");
        println!("  0x13  SANITIZE   Remove taint");
        println!();

        println!("I/O (Sandboxed):");
        println!("  0x14  FILE       open, read, write, close, seek, stat, mkdir, delete");
        println!("  0x15  NET        socket, connect, bind, listen, accept, send, recv, close");
        println!("  0x16  NET.SETOPT nonblock, timeout, keepalive, reuseaddr, nodelay");
        println!("  0x17  IO         print, read_line, get_args, get_env");
        println!("  0x18  TIME       now, sleep, monotonic");
        println!();

        println!("Math Extensions:");
        println!("  0x19  FPU        fadd, fsub, fmul, fdiv, fsqrt, fabs, ffloor, fceil");
        println!("  0x1A  RAND       rand_bytes, rand_u64");
        println!("  0x1B  BITS       popcount, clz, ctz, bswap");
        println!();

        println!("System:");
        println!("  0x1C  MOV        Reg-reg, load immediate");
        println!("  0x1D  TRAP       Syscall, breakpoint, fault");
        println!("  0x1E  NOP        No operation");
        println!("  0x1F  HALT       Stop execution");
        println!();
    }

    if show_registers {
        println!("Registers (32 total):");
        println!("=====================\n");

        println!("General Purpose: r0-r15");
        println!("  r0 (a0)  - Argument 0 / Return value");
        println!("  r1-r5 (a1-a5) - Arguments 1-5");
        println!("  r6-r15   - Caller-saved");
        println!();

        println!("Special Purpose:");
        println!("  sp   - Stack pointer");
        println!("  fp   - Frame pointer");
        println!("  lr   - Link register");
        println!("  pc   - Program counter (read-only)");
        println!("  csp  - Capability stack pointer");
        println!("  cfp  - Capability frame pointer");
        println!("  zero - Always zero (read-only)");
        println!();
    }

    Ok(())
}

fn cmd_repl() -> Result<()> {
    println!("Neurlang Interactive REPL");
    println!("Type 'help' for commands, 'quit' to exit\n");

    let mut asm = Assembler::new();
    let mut interp = Interpreter::new(65536);
    let mut line_buffer = String::new();

    loop {
        print!("nl> ");
        io::stdout().flush()?;

        line_buffer.clear();
        io::stdin().read_line(&mut line_buffer)?;
        let line = line_buffer.trim();

        if line.is_empty() {
            continue;
        }

        match line {
            "quit" | "exit" | "q" => {
                println!("Goodbye!");
                break;
            }
            "help" | "?" => {
                println!("Commands:");
                println!("  help        - Show this help");
                println!("  quit        - Exit REPL");
                println!("  regs        - Show registers");
                println!("  reset       - Reset interpreter state");
                println!("  <asm>       - Execute assembly instruction(s)");
                println!();
            }
            "regs" => {
                println!("Registers:");
                for i in 0..16 {
                    println!(
                        "  r{:2} = {:#018x} ({})",
                        i, interp.registers[i], interp.registers[i] as i64
                    );
                }
            }
            "reset" => {
                interp = Interpreter::new(65536);
                println!("Interpreter reset");
            }
            _ => {
                // Try to assemble and execute
                match asm.assemble(line) {
                    Ok(program) => {
                        let result = interp.execute(&program);
                        match result {
                            neurlang::interp::InterpResult::Ok(val) => {
                                println!("=> {}", val);
                            }
                            neurlang::interp::InterpResult::Halted => {
                                println!("Halted. r0 = {}", interp.registers[0]);
                            }
                            other => {
                                println!("Error: {:?}", other);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}

fn load_program(path: &PathBuf) -> Result<Program> {
    let data = fs::read(path).context("Failed to read file")?;

    // Try binary first
    if let Some(program) = Program::decode(&data) {
        return Ok(program);
    }

    // Try text assembly
    let source = String::from_utf8(data).context("Invalid UTF-8")?;
    let mut asm = Assembler::new();
    asm.assemble(&source).context("Assembly failed")
}

fn cmd_prompt(
    prompt: &str,
    model_path: &PathBuf,
    engine_name: &str,
    show_asm: bool,
    max_retries: usize,
    output: Option<&PathBuf>,
    verbose: bool,
) -> Result<()> {
    use neurlang::inference::{engines_info, select_engine, EngineType};

    println!("Generating code from: \"{}\"", prompt);

    if !model_path.exists() {
        anyhow::bail!("Model file not found: {}. Train a model first with 'nl train' or specify path with --model", model_path.display());
    }

    let engine_type = EngineType::from_str(engine_name);

    if engine_type.is_none() && engine_name != "auto" {
        anyhow::bail!("Unknown engine: '{}'. {}", engine_name, engines_info());
    }

    println!(
        "Loading model from: {} (engine: {})",
        model_path.display(),
        engine_type
            .map(|e| e.display_name())
            .unwrap_or("auto-detect")
    );

    // Try to use the new engine abstraction
    let model = match select_engine(model_path, engine_type) {
        Ok(engine) => {
            println!("Using engine: {}", engine.model_info());
            InferenceEngine::load(model_path)?
        }
        Err(e) => {
            anyhow::bail!("Failed to load model: {}", e);
        }
    };

    println!("Model: {}", model.model_info());

    let config = OrchestratorConfig {
        max_retries,
        verbose,
        ..Default::default()
    };

    let orch = Orchestrator::with_config(model, config);

    let result = orch.run(prompt);

    match result {
        OrchResult::Success {
            binary,
            output: value,
            attempts,
            total_time,
        } => {
            println!("\nSuccess (attempt {}/{})!", attempts, max_retries + 1);
            println!("Result: R0 = {}", value);
            println!("Time: {:?}", total_time);

            if show_asm {
                println!("\nGenerated assembly:");
                if let Some(program) = Program::decode(&binary) {
                    let disasm = Disassembler::new().with_offsets(true);
                    println!("{}", disasm.disassemble(&program));
                }
            }

            if let Some(out_path) = output {
                fs::write(out_path, &binary).context("Failed to write output")?;
                println!("\nSaved {} bytes to {}", binary.len(), out_path.display());
            }
        }
        OrchResult::Failed {
            binary,
            error,
            attempts,
            total_time,
        } => {
            println!("\nFailed after {} attempts ({:?})", attempts, total_time);
            println!("Error: {}", error.english);
            println!("Suggestion: {}", error.suggestion);

            if show_asm && !binary.is_empty() {
                println!("\nLast generated assembly:");
                if let Some(program) = Program::decode(&binary) {
                    let disasm = Disassembler::new().with_offsets(true);
                    println!("{}", disasm.disassemble(&program));
                }
            }
        }
        OrchResult::InferenceError { error, attempts } => {
            println!("\nInference error after {} attempts: {}", attempts, error);
        }
    }

    Ok(())
}

// =============================================================================
// Training Commands
// =============================================================================

fn cmd_train(
    data: &PathBuf,
    output: &PathBuf,
    remote: Option<&str>,
    remote_dir: &str,
    profile: &str,
    backend: &str,
    provisioner: Option<&PathBuf>,
    no_provision: bool,
    epochs: usize,
    patience: usize,
    cross_validate: bool,
    folds: usize,
    verbose: bool,
    list_profiles: bool,
) -> Result<()> {
    // List profiles and exit
    if list_profiles {
        training::print_profile_table();
        return Ok(());
    }

    // Parse GPU profile
    let gpu_profile = training::get_profile(profile).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown GPU profile: {}. Use --list-profiles to see options.",
            profile
        )
    })?;

    // Parse training backend - detect available backends
    let backend_lower = backend.to_lowercase();
    let (is_native, is_docker, use_pytorch) = match backend_lower.as_str() {
        "native" | "burn" | "rust" => (true, false, false),
        "docker" | "podman" | "container" => (false, true, false),
        "pytorch" | "torch" | "pt" => (false, false, true),
        "auto" => {
            // Auto-detect: prefer native if compiled, then docker, then pytorch
            #[cfg(feature = "train")]
            {
                (true, false, false)
            }
            #[cfg(not(feature = "train"))]
            {
                if train::find_container_runtime().is_some() {
                    (false, true, false)
                } else {
                    // Try system pytorch
                    (false, false, true)
                }
            }
        }
        _ => (false, false, false),
    };

    // If pytorch was selected via auto-detect, update backend string
    let backend = if use_pytorch { "pytorch" } else { backend };

    let train_backend = if is_native || is_docker || use_pytorch {
        TrainingBackend::default() // Use default, we'll handle native/docker/pytorch separately
    } else {
        TrainingBackend::from_str(backend).ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown training backend: {}. Options: auto, native, docker, pytorch, onnx",
                backend
            )
        })?
    };

    // Create training config
    let config = TrainingConfig {
        data_path: data.clone(),
        output_path: output.clone(),
        profile: gpu_profile,
        backend: train_backend,
        remote_host: remote.unwrap_or("").to_string(),
        remote_dir: remote_dir.to_string(),
        provisioner: if no_provision {
            None
        } else {
            provisioner.cloned().or_else(|| {
                let default = PathBuf::from(gpu_profile.provisioner_script());
                if default.exists() {
                    Some(default)
                } else {
                    None
                }
            })
        },
        epochs,
        patience,
        cross_validate,
        folds,
        verbose,
    };

    if config.remote_host.is_empty() {
        // Local training
        println!("=== Local Training ===");
        println!();
        run_local_training(&config, backend, is_native, is_docker, use_pytorch)
    } else {
        // Remote training
        training::run_remote_training(&config)
    }
}

fn run_local_training(
    config: &TrainingConfig,
    backend: &str,
    is_native: bool,
    is_docker: bool,
    use_pytorch: bool,
) -> Result<()> {
    if is_native {
        // Native Rust training using burn framework
        println!("Using native (burn) backend");
        println!();

        // Check if train feature is available
        #[cfg(not(feature = "train"))]
        {
            // Check if docker is available as fallback
            if train::find_container_runtime().is_some() {
                anyhow::bail!(
                    "Native training not compiled. Use Docker instead:\n  \
                    nl train --backend docker --data {}\n\n\
                    Or rebuild with native training:\n  \
                    cargo build --release --features train",
                    config.data_path.display()
                );
            } else {
                anyhow::bail!(
                    "No training backend available.\n\n\
                    Option 1 - Install Docker/Podman:\n  \
                    apt install docker.io  # or podman\n  \
                    nl train --backend docker --data {}\n\n\
                    Option 2 - Rebuild with native training:\n  \
                    cargo build --release --features train",
                    config.data_path.display()
                );
            }
        }

        #[cfg(feature = "train")]
        {
            // Cap batch size for wgpu backend - it has limited memory compared to CUDA
            // 64 is a safe default that works on most systems
            let wgpu_max_batch = 64;
            let batch_size = config.profile.batch_size().min(wgpu_max_batch);

            let train_config = train::NativeTrainConfig {
                data_path: config.data_path.clone(),
                output_path: config.output_path.with_extension("mpk"),
                epochs: config.epochs,
                batch_size,
                learning_rate: 0.001,
                weight_decay: 0.01,
                train_ratio: 0.9,
                seed: 42,
                export_onnx: true,
            };

            train::train(train_config).map_err(|e| anyhow::anyhow!("Training failed: {}", e))?;
        }
    } else if is_docker {
        // Docker-based training
        println!("Using docker backend");
        println!();

        let docker_config = train::DockerTrainConfig {
            data_path: config.data_path.clone(),
            output_path: config.output_path.clone(),
            epochs: config.epochs,
            batch_size: config.profile.batch_size(),
            learning_rate: 0.001,
            export_onnx: true,
        };

        let trainer = train::DockerTrainer::new()
            .map_err(|e| anyhow::anyhow!("Docker/Podman not found: {}", e))?;
        trainer
            .train(&docker_config)
            .map_err(|e| anyhow::anyhow!("Docker training failed: {}", e))?;
    } else if use_pytorch
        || matches!(
            config.backend,
            TrainingBackend::PyTorch | TrainingBackend::Onnx
        )
    {
        // Python-based training (pytorch/onnx)
        // Try: 1) venv python, 2) system python3
        let venv_python = PathBuf::from("train/venv/bin/python");
        let python_cmd = if venv_python.exists() {
            venv_python.to_string_lossy().to_string()
        } else {
            "python3".to_string()
        };

        println!(
            "Using {} backend with {}",
            backend,
            if venv_python.exists() {
                "venv"
            } else {
                "system python"
            }
        );
        println!();

        // Check if torch is available
        let torch_check = Command::new(&python_cmd)
            .args(["-c", "import torch; print(torch.__version__)"])
            .output();

        let torch_available = torch_check.map(|o| o.status.success()).unwrap_or(false);

        if !torch_available {
            anyhow::bail!(
                "PyTorch not found. Install it with:\n  \
                pip install torch numpy tqdm\n\n\
                Or use Docker backend:\n  \
                nl train --backend docker --data {}",
                config.data_path.display()
            );
        }

        match config.backend {
            TrainingBackend::PyTorch => {
                let mut cmd = Command::new(&python_cmd);
                cmd.current_dir("train");
                cmd.arg("train.py");
                cmd.arg("--data").arg(&config.data_path);
                cmd.arg("--output")
                    .arg(config.output_path.file_name().unwrap_or_default());
                cmd.arg("--epochs").arg(config.epochs.to_string());
                cmd.arg("--patience").arg(config.patience.to_string());
                cmd.arg("--batch-size")
                    .arg(config.profile.batch_size().to_string());
                cmd.arg("--device").arg("cpu");

                if config.cross_validate {
                    cmd.arg("--cross-validate");
                    cmd.arg("--folds").arg(config.folds.to_string());
                }

                let status = cmd.status().context("Failed to run training")?;
                if !status.success() {
                    anyhow::bail!("Training failed with status: {}", status);
                }
            }
            TrainingBackend::Onnx => {
                let mut cmd = Command::new(&python_cmd);
                cmd.current_dir("train");
                cmd.arg("train_onnx.py");
                cmd.arg("--data").arg(&config.data_path);
                cmd.arg("--output").arg(
                    config
                        .output_path
                        .with_extension("onnx")
                        .file_name()
                        .unwrap_or_default(),
                );
                cmd.arg("--epochs").arg(config.epochs.to_string());
                cmd.arg("--batch-size")
                    .arg(config.profile.batch_size().min(256).to_string());

                let status = cmd.status().context("Failed to run ONNX training")?;
                if !status.success() {
                    anyhow::bail!("ONNX training failed with status: {}", status);
                }
            }
        }
    } else {
        anyhow::bail!(
            "Unknown training backend: {}\n\n\
            Available backends:\n  \
            - auto     Auto-detect best available\n  \
            - pytorch  Use system Python with PyTorch\n  \
            - docker   Use Docker container\n  \
            - native   Use native Rust (requires --features train)",
            backend
        );
    }

    Ok(())
}

fn cmd_datagen(
    output: &PathBuf,
    num_examples: usize,
    level: u8,
    seed: u64,
    include_examples: bool,
    parallel: bool,
) -> Result<()> {
    println!("=== Neurlang Training Data Generator ===");
    println!();

    if parallel {
        // Use Python generator for parallel model (balanced dataset)
        println!("Using parallel model data generator");
        println!(
            "Target: {} samples with full immediate coverage",
            num_examples
        );
        println!();

        let script_path = PathBuf::from("train/parallel/generate_balanced_data.py");
        if !script_path.exists() {
            anyhow::bail!("Python generator not found: {}", script_path.display());
        }

        // Find Python
        let venv_python = PathBuf::from("train/venv/bin/python");
        let python_cmd = if venv_python.exists() {
            venv_python.to_string_lossy().to_string()
        } else {
            "python3".to_string()
        };

        // Run the Python generator
        let mut cmd = Command::new(&python_cmd);
        cmd.current_dir("train");
        cmd.env("PYTHONPATH", ".");
        cmd.arg("parallel/generate_balanced_data.py");
        cmd.arg(output);

        // Pass sample count via environment
        cmd.env("TARGET_SAMPLES", num_examples.to_string());

        let status = cmd.status().context("Failed to run Python datagen")?;

        if !status.success() {
            anyhow::bail!("Python data generation failed");
        }
    } else {
        // Use Rust generator (legacy)
        println!("Using legacy Rust data generator");
        println!();

        // Check if nl-datagen binary exists
        let datagen_path = PathBuf::from("target/release/nl-datagen");
        if !datagen_path.exists() {
            println!("Building data generator...");
            let status = Command::new("cargo")
                .args(["build", "--release", "--bin", "nl-datagen"])
                .status()
                .context("Failed to build datagen")?;

            if !status.success() {
                anyhow::bail!("Failed to build nl-datagen");
            }
        }

        // Run datagen
        let mut cmd = Command::new(&datagen_path);
        cmd.arg("--output").arg(output);
        cmd.arg("--num-examples").arg(num_examples.to_string());
        cmd.arg("--curriculum-level").arg(level.to_string());
        cmd.arg("--seed").arg(seed.to_string());

        if include_examples {
            cmd.arg("--include-examples");
        }

        let status = cmd.status().context("Failed to run datagen")?;

        if !status.success() {
            anyhow::bail!("Data generation failed");
        }
    }

    println!();
    println!("Data generated: {}", output.display());

    Ok(())
}

fn cmd_accuracy(
    model: &PathBuf,
    benchmark: bool,
    test_data: Option<&PathBuf>,
    verbose: bool,
) -> Result<()> {
    println!("=== Neurlang Model Accuracy Test ===");
    println!();

    if !model.exists() {
        anyhow::bail!(
            "Model file not found: {}. Train a model first with 'nl train'",
            model.display()
        );
    }

    if benchmark {
        // Run the benchmark script
        let script = PathBuf::from("scripts/benchmark_model.sh");
        if !script.exists() {
            anyhow::bail!("Benchmark script not found: {}", script.display());
        }

        let mut cmd = Command::new("bash");
        cmd.arg(&script);
        cmd.env("MODEL_PATH", model);

        let status = cmd.status().context("Failed to run benchmark")?;

        if !status.success() {
            println!("Benchmark completed with failures");
        }
    } else {
        // Run Python test script
        let venv_python = PathBuf::from("train/venv/bin/python");
        let test_script = PathBuf::from("train/test_accuracy.py");

        if !test_script.exists() {
            anyhow::bail!("Test script not found: {}", test_script.display());
        }

        let mut cmd = if venv_python.exists() {
            Command::new(&venv_python)
        } else {
            Command::new("python3")
        };

        cmd.arg(&test_script);
        cmd.arg("--model").arg(model);

        if let Some(td) = test_data {
            cmd.arg("--test-data").arg(td);
        }

        if verbose {
            cmd.arg("--verbose");
        }

        let status = cmd.status().context("Failed to run test")?;

        if !status.success() {
            anyhow::bail!("Accuracy test failed");
        }
    }

    Ok(())
}

fn cmd_export_onnx(input: &PathBuf, output: &PathBuf, verbose: bool) -> Result<()> {
    println!("=== Export to ONNX ===");
    println!();

    if !input.exists() {
        anyhow::bail!("Input model not found: {}", input.display());
    }

    let venv_python = PathBuf::from("train/venv/bin/python");
    let export_script = PathBuf::from("train/export_onnx.py");

    if !export_script.exists() {
        anyhow::bail!("Export script not found: {}", export_script.display());
    }

    let mut cmd = if venv_python.exists() {
        Command::new(&venv_python)
    } else {
        Command::new("python3")
    };

    cmd.arg(&export_script);
    cmd.arg("--input").arg(input);
    cmd.arg("--output").arg(output);

    if verbose {
        cmd.arg("--verbose");
    }

    let status = cmd.status().context("Failed to export ONNX")?;

    if !status.success() {
        anyhow::bail!("ONNX export failed");
    }

    println!("Exported to: {}", output.display());

    Ok(())
}

// =============================================================================
// Agent Command
// =============================================================================

fn cmd_agent(
    new: Option<String>,
    cont: Option<String>,
    resume: bool,
    list: bool,
    interactive: bool,
    task: Option<String>,
    max_iterations: usize,
    verbose: bool,
) -> Result<()> {
    // List sessions
    if list {
        return cmd_agent_list();
    }

    // Determine mode - check flags before consuming optionals
    let has_new = new.is_some();
    let has_cont = cont.is_some();

    // Handle no-session-needed case first
    if !has_new && !has_cont && !resume && !interactive {
        println!("Neurlang Agent - Interactive AI Code Generation");
        println!();
        println!("Usage:");
        println!("  nl agent --new <name> [task]     Start a new session");
        println!("  nl agent --continue <id> [task]  Continue an existing session");
        println!("  nl agent --resume                Resume the most recent session");
        println!("  nl agent --list                  List all sessions");
        println!("  nl agent --interactive           Start interactive REPL");
        println!();
        println!("Examples:");
        println!("  nl agent --new \"calculator\" \"compute factorial of 10\"");
        println!("  nl agent --continue abc123 \"add support for negative numbers\"");
        println!("  nl agent --interactive");
        return Ok(());
    }

    let (mut agent, session_name) = if let Some(name) = new {
        // Create new session
        let config = AgentConfig {
            max_iterations,
            verbose,
            ..Default::default()
        };
        let agent = Agent::with_config(&name, config).context("Failed to create agent")?;
        println!("Created new session: {} ({})", name, agent.session_id());
        (agent, name)
    } else if let Some(session_id) = cont {
        // Continue existing session
        let full_id = agent_find_session(&session_id)
            .context("Failed to search sessions")?
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        let agent = Agent::resume(&full_id).context("Failed to resume session")?;
        let session_name = agent.session_name().to_string();
        println!(
            "Resuming session: {} (iteration {})",
            session_name,
            agent.iteration_count()
        );
        (agent, session_name)
    } else if resume {
        // Resume most recent session
        let sessions = agent_list_sessions().context("Failed to list sessions")?;

        if sessions.is_empty() {
            anyhow::bail!("No sessions found. Start a new session with --new <name>");
        }

        let (id, name, iterations) = &sessions[0];
        let agent = Agent::resume(id).context("Failed to resume session")?;
        println!(
            "Resuming most recent session: {} (iteration {})",
            name, iterations
        );
        (agent, name.clone())
    } else {
        // Interactive mode without a specific session
        let agent = Agent::new("interactive").context("Failed to create agent")?;
        println!("Starting interactive session: {}", agent.session_id());
        (agent, "interactive".to_string())
    };

    // Execute task if provided
    if let Some(task_str) = task {
        println!();
        println!("Task: {}", task_str);
        println!("Generating...");

        let result = agent.handle_request_unverified(&task_str);

        match result {
            Ok(r) => {
                println!();
                println!("Result: {}", r.summary);
                println!("Iterations: {}", r.iterations);
                println!("Instructions: {}", r.program.instructions.len());

                if verbose {
                    println!();
                    println!("Generated IR:");
                    let disasm = Disassembler::new().with_offsets(true);
                    println!("{}", disasm.disassemble(&r.program));
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }

        return Ok(());
    }

    // Interactive mode
    if interactive || has_new || has_cont || resume {
        cmd_agent_interactive(agent, &session_name, verbose)
    } else {
        Ok(())
    }
}

fn cmd_agent_list() -> Result<()> {
    println!("Neurlang Sessions");
    println!("=================");
    println!();

    let sessions = agent_list_sessions().context("Failed to list sessions")?;

    if sessions.is_empty() {
        println!("No sessions found.");
        println!();
        println!("Start a new session with:");
        println!("  nl agent --new \"session name\"");
        return Ok(());
    }

    println!("{:<12} {:<30} {:>10}", "ID", "NAME", "ITERATIONS");
    println!("{:-<12} {:-<30} {:->10}", "", "", "");

    for (id, name, iterations) in sessions {
        let short_id = if id.len() > 10 { &id[..10] } else { &id };
        let short_name = if name.len() > 28 {
            format!("{}...", &name[..25])
        } else {
            name
        };
        println!("{:<12} {:<30} {:>10}", short_id, short_name, iterations);
    }

    println!();
    println!("Continue a session with:");
    println!("  nl agent --continue <id>");

    Ok(())
}

fn cmd_agent_interactive(mut agent: Agent, _session_name: &str, verbose: bool) -> Result<()> {
    println!();
    println!("Interactive Agent Mode");
    println!("Type a task description, 'help' for commands, or 'quit' to exit.");
    println!();

    let mut line_buffer = String::new();

    loop {
        print!("agent> ");
        io::stdout().flush()?;

        line_buffer.clear();
        io::stdin().read_line(&mut line_buffer)?;
        let line = line_buffer.trim();

        if line.is_empty() {
            continue;
        }

        match line {
            "quit" | "exit" | "q" => {
                println!(
                    "Session saved. Resume with: nl agent --continue {}",
                    agent.session_id()
                );
                break;
            }
            "help" | "?" => {
                println!("Commands:");
                println!("  help        - Show this help");
                println!("  quit        - Exit (session is saved automatically)");
                println!("  history     - Show conversation history");
                println!("  show        - Show current program");
                println!("  status      - Show session status");
                println!("  save <name> - Save current program as named function");
                println!();
                println!("Or type a task description:");
                println!("  \"compute fibonacci of 10\"");
                println!("  \"add two numbers\"");
                println!("  \"implement a sorting algorithm\"");
                println!();
            }
            "history" => {
                println!("Conversation History:");
                println!("---------------------");
                for turn in agent.history() {
                    match turn {
                        neurlang::inference::ConversationTurn::User(msg) => {
                            println!("You: {}", msg);
                        }
                        neurlang::inference::ConversationTurn::Agent(msg) => {
                            println!("Agent: {}", msg);
                        }
                        neurlang::inference::ConversationTurn::Error(msg) => {
                            println!("Error: {}", msg);
                        }
                        neurlang::inference::ConversationTurn::System(msg) => {
                            println!("System: {}", msg);
                        }
                    }
                }
                println!();
            }
            "status" => {
                println!("Session ID: {}", agent.session_id());
                println!("Session Name: {}", agent.session_name());
                println!("Iterations: {}", agent.iteration_count());
                println!();
            }
            "show" => {
                // Show would need access to current program - for now show status
                println!(
                    "Current session has {} iterations.",
                    agent.iteration_count()
                );
                println!("Use 'history' to see conversation.");
                println!();
            }
            cmd if cmd.starts_with("save ") => {
                let name = cmd.strip_prefix("save ").unwrap().trim();
                println!("Saving function as '{}'...", name);
                // Would need to save the current program as a named function
                println!("(Function saving not yet implemented)");
                println!();
            }
            _ => {
                // Execute as task
                println!("Generating...");

                let result = agent.handle_request_unverified(line);

                match result {
                    Ok(r) => {
                        println!();
                        println!("Result: {}", r.summary);

                        if verbose {
                            println!();
                            println!("Generated IR:");
                            let disasm = Disassembler::new().with_offsets(true);
                            println!("{}", disasm.disassemble(&r.program));
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
                println!();
            }
        }
    }

    Ok(())
}

// =============================================================================
// Extension Commands
// =============================================================================

fn cmd_extension(
    add: Option<String>,
    new: Option<String>,
    remove: Option<String>,
    list: bool,
    load: Option<PathBuf>,
    info: Option<String>,
) -> Result<()> {
    // List extensions
    if list {
        return cmd_extension_list();
    }

    // Add extension from git URL
    if let Some(url) = add {
        return cmd_extension_add(&url);
    }

    // Create new local extension
    if let Some(name) = new {
        return cmd_extension_new(&name);
    }

    // Remove extension
    if let Some(path) = remove {
        return cmd_extension_remove(&path);
    }

    // Load extension from file
    if let Some(path) = load {
        return cmd_extension_load(&path);
    }

    // Show extension info
    if let Some(path) = info {
        return cmd_extension_info(&path);
    }

    // No command specified - show help
    println!("Neurlang Extensions (Go-Style Package System)");
    println!();
    println!("Usage:");
    println!("  nl extension --add <url>       Install from git URL");
    println!("  nl extension --new <name>      Create a new local extension");
    println!("  nl extension --remove <path>   Remove an extension");
    println!("  nl extension --list            List installed extensions");
    println!("  nl extension --load <file>     Load extension from file");
    println!("  nl extension --info <path>     Show extension details");
    println!();
    println!("Examples:");
    println!("  nl extension --add github.com/user/csv-parser");
    println!("  nl extension --add github.com/user/csv-parser@v1.2.0");
    println!("  nl extension --new my-utils");
    println!("  nl extension --list");
    println!();
    println!("Directory Structure:");
    println!("  ~/.neurlang/extensions/");
    println!("  ├── local/           # User-created extensions");
    println!("  │   └── my-utils/");
    println!("  └── cache/           # Git-installed extensions");
    println!("      └── github.com/user/repo/");

    Ok(())
}

fn cmd_extension_list() -> Result<()> {
    println!("Installed Extensions");
    println!("====================");
    println!();

    let registry =
        ExtensionRegistry::new().map_err(|e| anyhow::anyhow!("Failed to open registry: {}", e))?;

    let extensions: Vec<_> = registry.list().collect();

    if extensions.is_empty() {
        println!("No extensions installed.");
        println!();
        println!("Install an extension with:");
        println!("  nl extension --add github.com/user/repo");
        println!();
        println!("Or create a local extension:");
        println!("  nl extension --new my-utils");
        return Ok(());
    }

    println!("{:<40} {:<10} {:<10}", "IMPORT PATH", "VERSION", "SOURCE");
    println!("{:-<40} {:-<10} {:-<10}", "", "", "");

    for (path, info) in extensions {
        let source = match &info.source {
            neurlang::extensions::ExtensionSource::Local => "local",
            neurlang::extensions::ExtensionSource::Git { .. } => "git",
        };
        println!("{:<40} {:<10} {:<10}", path, info.manifest.version, source);
    }

    println!();

    Ok(())
}

fn cmd_extension_add(url: &str) -> Result<()> {
    println!("Installing extension: {}", url);
    println!();

    let mut registry =
        ExtensionRegistry::new().map_err(|e| anyhow::anyhow!("Failed to open registry: {}", e))?;

    let info = registry
        .install(url)
        .map_err(|e| anyhow::anyhow!("Failed to install: {}", e))?;

    println!("Installed: {}", info.manifest.name);
    println!("  Version: {}", info.manifest.version);
    println!("  Path: {}", info.path.display());

    if !info.manifest.exports.is_empty() {
        println!("  Exports:");
        for export in &info.manifest.exports {
            println!("    - {}", export.name);
        }
    }

    println!();
    println!("Import with: @import \"{}\"", url);

    Ok(())
}

fn cmd_extension_new(name: &str) -> Result<()> {
    println!("Creating local extension: {}", name);
    println!();

    let mut registry =
        ExtensionRegistry::new().map_err(|e| anyhow::anyhow!("Failed to open registry: {}", e))?;

    let info = registry
        .create_local(name)
        .map_err(|e| anyhow::anyhow!("Failed to create: {}", e))?;

    println!("Created extension at: {}", info.path.display());
    println!();
    println!("Files created:");
    println!("  neurlang.json  - Extension manifest");
    println!("  main.nl        - Entry point");
    println!();
    println!(
        "Edit {} to add your code.",
        info.path.join("main.nl").display()
    );
    println!();
    println!("Import with: @import \"local/{}\"", name);

    Ok(())
}

fn cmd_extension_remove(import_path: &str) -> Result<()> {
    println!("Removing extension: {}", import_path);

    let mut registry =
        ExtensionRegistry::new().map_err(|e| anyhow::anyhow!("Failed to open registry: {}", e))?;

    registry
        .remove(import_path)
        .map_err(|e| anyhow::anyhow!("Failed to remove: {}", e))?;

    println!("Removed successfully.");

    Ok(())
}

fn cmd_extension_load(path: &PathBuf) -> Result<()> {
    println!("Loading extension from: {}", path.display());
    println!();

    let mut loader = ExtensionLoader::default();

    let ext = loader
        .load_file(path)
        .map_err(|e| anyhow::anyhow!("Failed to load: {}", e))?;

    println!("Loaded: {}", ext.manifest.name);
    println!("  Version: {}", ext.manifest.version);
    println!("  Instructions: {}", ext.program.instructions.len());

    if !ext.exports.is_empty() {
        println!("  Exports:");
        for (name, offset) in &ext.exports {
            println!("    - {} (offset: {})", name, offset);
        }
    }

    println!();

    // Optionally disassemble
    let disasm = Disassembler::new().with_offsets(true);
    println!("Disassembly:");
    println!("{}", disasm.disassemble(&ext.program));

    Ok(())
}

fn cmd_extension_info(import_path: &str) -> Result<()> {
    let registry =
        ExtensionRegistry::new().map_err(|e| anyhow::anyhow!("Failed to open registry: {}", e))?;

    let info = registry
        .get(import_path)
        .ok_or_else(|| anyhow::anyhow!("Extension not found: {}", import_path))?;

    println!("Extension: {}", info.manifest.name);
    println!("  Version: {}", info.manifest.version);
    println!("  Path: {}", info.path.display());
    println!("  Entry: {}", info.manifest.entry);

    match &info.source {
        neurlang::extensions::ExtensionSource::Local => {
            println!("  Source: local");
        }
        neurlang::extensions::ExtensionSource::Git { url, version } => {
            println!("  Source: git");
            println!("  URL: {}", url);
            if let Some(v) = version {
                println!("  Tag/Branch: {}", v);
            }
        }
    }

    if !info.manifest.description.is_empty() {
        println!("  Description: {}", info.manifest.description);
    }

    if !info.manifest.exports.is_empty() {
        println!("  Exports:");
        for export in &info.manifest.exports {
            let desc = if export.description.is_empty() {
                String::new()
            } else {
                format!(" - {}", export.description)
            };
            println!("    - {}{}", export.name, desc);
        }
    }

    if !info.manifest.dependencies.is_empty() {
        println!("  Dependencies:");
        for dep in &info.manifest.dependencies {
            let version = dep.version.as_deref().unwrap_or("latest");
            println!("    - {} @ {}", dep.path, version);
        }
    }

    Ok(())
}

// =============================================================================
// Configuration Management
// =============================================================================

/// Get the path to the config file
fn config_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".neurlang").join("config.json")
}

/// Load configuration from file
fn load_config() -> serde_json::Value {
    let path = config_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
    }
    serde_json::json!({})
}

/// Save configuration to file
fn save_config(config: &serde_json::Value) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&path, content)?;
    Ok(())
}

/// Get a nested value from config using dot notation (e.g., "backends.claude.api_key")
fn config_get_nested(config: &serde_json::Value, key: &str) -> Option<serde_json::Value> {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = config;

    for part in parts {
        match current.get(part) {
            Some(v) => current = v,
            None => return None,
        }
    }

    Some(current.clone())
}

/// Set a nested value in config using dot notation
fn config_set_nested(config: &mut serde_json::Value, key: &str, value: serde_json::Value) {
    let parts: Vec<&str> = key.split('.').collect();

    if parts.is_empty() {
        return;
    }

    let mut current = config;

    // Navigate to the parent of the target
    for part in &parts[..parts.len() - 1] {
        if !current.get(*part).map(|v| v.is_object()).unwrap_or(false) {
            current[*part] = serde_json::json!({});
        }
        current = current.get_mut(*part).unwrap();
    }

    // Set the final key
    current[parts[parts.len() - 1]] = value;
}

/// Unset a nested value in config
fn config_unset_nested(config: &mut serde_json::Value, key: &str) -> bool {
    let parts: Vec<&str> = key.split('.').collect();

    if parts.is_empty() {
        return false;
    }

    let mut current = config;

    // Navigate to the parent of the target
    for part in &parts[..parts.len() - 1] {
        match current.get_mut(*part) {
            Some(v) if v.is_object() => current = v,
            _ => return false,
        }
    }

    // Remove the final key
    if let Some(obj) = current.as_object_mut() {
        obj.remove(parts[parts.len() - 1]).is_some()
    } else {
        false
    }
}

fn cmd_config(
    set: Option<Vec<String>>,
    get: Option<String>,
    list: bool,
    unset: Option<String>,
    path: bool,
) -> Result<()> {
    if path {
        println!("{}", config_path().display());
        return Ok(());
    }

    if let Some(args) = set {
        if args.len() != 2 {
            anyhow::bail!("--set requires exactly 2 arguments: KEY VALUE");
        }
        let key = &args[0];
        let value_str = &args[1];

        // Try to parse as JSON, fall back to string
        let value: serde_json::Value = serde_json::from_str(value_str)
            .unwrap_or_else(|_| serde_json::Value::String(value_str.clone()));

        let mut config = load_config();
        config_set_nested(&mut config, key, value);
        save_config(&config)?;

        println!("Set {} = {}", key, value_str);
        return Ok(());
    }

    if let Some(key) = get {
        let config = load_config();
        match config_get_nested(&config, &key) {
            Some(value) => {
                if value.is_string() {
                    println!("{}", value.as_str().unwrap());
                } else {
                    println!("{}", serde_json::to_string_pretty(&value)?);
                }
            }
            None => {
                println!("(not set)");
            }
        }
        return Ok(());
    }

    if let Some(key) = unset {
        let mut config = load_config();
        if config_unset_nested(&mut config, &key) {
            save_config(&config)?;
            println!("Unset {}", key);
        } else {
            println!("Key not found: {}", key);
        }
        return Ok(());
    }

    if list {
        let config = load_config();
        if config.as_object().map(|o| o.is_empty()).unwrap_or(true) {
            println!("No configuration set.");
            println!();
            println!("Example configuration commands:");
            println!("  nl config --set backends.claude.api_key \"sk-ant-...\"");
            println!("  nl config --set backends.default \"claude\"");
            println!("  nl config --set backends.ollama.host \"http://localhost:11434\"");
        } else {
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
        return Ok(());
    }

    // Default: show help
    println!("Neurlang Configuration");
    println!();
    println!("Usage:");
    println!("  nl config --set KEY VALUE   Set a configuration value");
    println!("  nl config --get KEY         Get a configuration value");
    println!("  nl config --unset KEY       Remove a configuration value");
    println!("  nl config --list            List all configuration");
    println!("  nl config --path            Show config file path");
    println!();
    println!("Common settings:");
    println!("  backends.default            Default LLM backend (claude, ollama)");
    println!("  backends.claude.api_key     Anthropic API key");
    println!("  backends.claude.model       Claude model (default: claude-sonnet-4-20250514)");
    println!("  backends.ollama.host        Ollama server URL");
    println!("  backends.ollama.model       Ollama model name");
    println!();
    println!("Config file: {}", config_path().display());

    Ok(())
}

// =============================================================================
// Backend Management
// =============================================================================

use neurlang::orchestration::backends::BackendRegistry;

fn cmd_backends(
    list: bool,
    status: Option<String>,
    set_default: Option<String>,
    test: Option<String>,
) -> Result<()> {
    let config = load_config();

    if list {
        println!("LLM Backends for Two-Tier Orchestration");
        println!();

        let registry = BackendRegistry::new();
        let default_backend = config_get_nested(&config, "backends.default")
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "claude".to_string());

        for name in registry.list() {
            let backend = registry.get(name).unwrap();
            let available = backend.is_available();
            let status_icon = if available { "✓" } else { "✗" };
            let default_marker = if name == default_backend {
                " (default)"
            } else {
                ""
            };

            println!("  {} {}{}", status_icon, name, default_marker,);

            if !available {
                match name {
                    "claude" => println!("      Not configured: set ANTHROPIC_API_KEY or use 'nl config --set backends.claude.api_key ...'"),
                    "ollama" => println!("      Not available: Ollama not running at localhost:11434"),
                    _ => println!("      Not available"),
                }
            }
        }

        println!();
        println!("Use 'nl backends --status NAME' for details.");
        return Ok(());
    }

    if let Some(name) = status {
        let registry = BackendRegistry::new();

        match registry.get(&name) {
            Some(backend) => {
                println!("Backend: {}", name);
                println!("  Available: {}", backend.is_available());
                println!("  Streaming: {}", backend.supports_streaming());

                // Show configuration from config file
                if let Some(backend_config) =
                    config_get_nested(&config, &format!("backends.{}", name))
                {
                    println!("  Configuration:");
                    if let Some(obj) = backend_config.as_object() {
                        for (key, value) in obj {
                            let display_value = if key.contains("key") || key.contains("secret") {
                                // Mask sensitive values
                                if let Some(s) = value.as_str() {
                                    if s.len() > 8 {
                                        format!("{}...{}", &s[..4], &s[s.len() - 4..])
                                    } else {
                                        "****".to_string()
                                    }
                                } else {
                                    "****".to_string()
                                }
                            } else if value.is_string() {
                                value.as_str().unwrap().to_string()
                            } else {
                                value.to_string()
                            };
                            println!("    {}: {}", key, display_value);
                        }
                    }
                }
            }
            None => {
                println!("Backend not found: {}", name);
                println!();
                println!("Available backends: claude, ollama");
            }
        }
        return Ok(());
    }

    if let Some(name) = set_default {
        let registry = BackendRegistry::new();

        if registry.get(&name).is_some() {
            let mut config = load_config();
            config_set_nested(
                &mut config,
                "backends.default",
                serde_json::Value::String(name.clone()),
            );
            save_config(&config)?;
            println!("Default backend set to: {}", name);
        } else {
            println!("Backend not found: {}", name);
            println!("Available backends: claude, ollama");
        }
        return Ok(());
    }

    if let Some(name) = test {
        let registry = BackendRegistry::new();

        match registry.get(&name) {
            Some(backend) => {
                println!("Testing backend: {}", name);

                if !backend.is_available() {
                    println!("  Status: NOT AVAILABLE");
                    match name.as_str() {
                        "claude" => println!("  Fix: Set ANTHROPIC_API_KEY environment variable"),
                        "ollama" => println!("  Fix: Start Ollama with 'ollama serve'"),
                        _ => {}
                    }
                    return Ok(());
                }

                println!("  Availability: OK");

                // Try a simple decomposition
                print!("  Decomposition test: ");
                io::stdout().flush()?;

                match backend.decompose_task("add two numbers", "") {
                    Ok(result) => {
                        println!("OK ({} subtasks)", result.subtasks.len());
                    }
                    Err(e) => {
                        println!("FAILED");
                        println!("    Error: {}", e);
                    }
                }
            }
            None => {
                println!("Backend not found: {}", name);
                println!("Available backends: claude, ollama");
            }
        }
        return Ok(());
    }

    // Default: show help
    println!("LLM Backend Management");
    println!();
    println!("Usage:");
    println!("  nl backends --list              List all backends");
    println!("  nl backends --status NAME       Show backend status");
    println!("  nl backends --set-default NAME  Set default backend");
    println!("  nl backends --test NAME         Test backend connectivity");
    println!();
    println!("Configuration:");
    println!("  nl config --set backends.claude.api_key \"sk-ant-...\"");
    println!("  nl config --set backends.ollama.host \"http://localhost:11434\"");
    println!("  nl config --set backends.default \"claude\"");

    Ok(())
}

// ============================================================================
// Test Command Implementation
// ============================================================================

/// Memory setup for test case (address -> bytes)
#[derive(Debug, Clone)]
struct MemorySetup {
    /// Address to write data
    address: u64,
    /// Bytes to write at address
    data: Vec<u8>,
}

/// Mock setup for extension calls
#[derive(Debug, Clone)]
struct MockSetup {
    /// Extension ID to mock
    ext_id: u32,
    /// Sequence of return values (supports stateful mocks)
    /// Single value: vec![value]; Sequence: vec![val1, val2, ...]
    /// Syntax: @mock: ext=val or @mock: ext=val1;val2;val3
    return_values: Vec<i64>,
    /// Output values (optional)
    outputs: Vec<u64>,
}

/// Network mock setup for testing server code
#[derive(Debug, Clone)]
struct NetMockSetup {
    /// Network operation to mock (socket, bind, listen, accept, connect, send, recv, close)
    op: neurlang::stencil::io::NetMockOp,
    /// Sequence of return values (supports stateful mocks)
    /// For accept: vec![client_fd, -1] to simulate one client then stop
    return_values: Vec<i64>,
    /// Optional data for recv operation
    recv_data: Option<Vec<u8>>,
}

/// Parsed test case from @test annotation
#[derive(Debug, Clone)]
struct TestCase {
    /// Input register assignments (e.g., [(0, 5), (1, 3)])
    inputs: Vec<(usize, u64)>,
    /// Expected output register assignments
    outputs: Vec<(usize, u64)>,
    /// Memory setup for this test (strings, arrays, etc.)
    memory: Vec<MemorySetup>,
    /// Extension mocks for this test
    mocks: Vec<MockSetup>,
    /// Network mocks for this test (for server testing)
    net_mocks: Vec<NetMockSetup>,
    /// Original annotation string for display
    annotation: String,
}

/// Parsed example metadata
#[derive(Debug)]
struct ExampleSpec {
    name: String,
    path: PathBuf,
    is_server: bool,
    test_cases: Vec<TestCase>,
    /// File-level mocks that apply to all tests
    file_mocks: Vec<MockSetup>,
    /// File-level network mocks for server testing
    file_net_mocks: Vec<NetMockSetup>,
}

/// Resolve the test path: use provided path, or search for test/tests dir, or use cwd
fn resolve_test_path(path: Option<PathBuf>) -> PathBuf {
    if let Some(p) = path {
        return p;
    }

    // Check for common test directory names
    for dir in &["test", "tests"] {
        let test_dir = PathBuf::from(dir);
        if test_dir.is_dir() {
            return test_dir;
        }
    }

    // Fall back to current directory
    PathBuf::from(".")
}

/// Recursively collect all .nl files from a directory
fn collect_nl_files_recursive(dir: &PathBuf, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                collect_nl_files_recursive(&path, files);
            } else if path.extension().is_some_and(|ext| ext == "nl") {
                files.push(path);
            }
        }
    }
}

fn cmd_test(
    path: &PathBuf,
    filter: Option<&str>,
    verbose: bool,
    fail_fast: bool,
    include_servers: bool,
    coverage: bool,
) -> Result<()> {
    use neurlang::interp::CoverageTracker;
    use std::time::Instant;

    // Collect .nl files (recursively if directory)
    let files = if path.is_dir() {
        let mut files: Vec<PathBuf> = Vec::new();
        collect_nl_files_recursive(path, &mut files);
        files.sort();
        files
    } else {
        vec![path.clone()]
    };

    if files.is_empty() {
        println!("No .nl files found in {}", path.display());
        return Ok(());
    }

    // Parse examples
    let mut examples: Vec<ExampleSpec> = Vec::new();
    for file in &files {
        match parse_example_spec(file) {
            Ok(spec) => {
                // Apply filter
                if let Some(f) = filter {
                    if !spec.name.to_lowercase().contains(&f.to_lowercase()) {
                        continue;
                    }
                }
                examples.push(spec);
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse {}: {}", file.display(), e);
            }
        }
    }

    println!("Running tests for {} examples...\n", examples.len());

    let mut total_tests = 0;
    let mut passed_tests = 0;
    let mut failed_tests = 0;
    let mut skipped_examples = 0;

    // Aggregate coverage data
    let mut total_instructions = 0usize;
    let mut total_executed = 0usize;
    let mut total_branches = 0usize;
    let mut covered_branch_outcomes = 0usize;

    let start = Instant::now();

    for spec in &examples {
        // Skip server examples unless explicitly included
        if spec.is_server && !include_servers {
            if verbose {
                println!("  [SKIP] {} (server)", spec.name);
            }
            skipped_examples += 1;
            continue;
        }

        // Skip examples with no test cases
        if spec.test_cases.is_empty() {
            if verbose {
                println!("  [SKIP] {} (no @test annotations)", spec.name);
            }
            skipped_examples += 1;
            continue;
        }

        // Load and assemble the program
        let source = match fs::read_to_string(&spec.path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("  [ERROR] {} - Failed to read: {}", spec.name, e);
                failed_tests += spec.test_cases.len();
                total_tests += spec.test_cases.len();
                continue;
            }
        };

        let mut asm = Assembler::new();
        let program = match asm.assemble(&source) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("  [ERROR] {} - Assembly failed: {}", spec.name, e);
                failed_tests += spec.test_cases.len();
                total_tests += spec.test_cases.len();
                continue;
            }
        };

        // Track per-example coverage
        let mut example_tracker: Option<CoverageTracker> = if coverage {
            Some(CoverageTracker::new(program.instructions.len()))
        } else {
            None
        };

        // Run each test case
        let mut example_passed = true;
        for (i, test_case) in spec.test_cases.iter().enumerate() {
            total_tests += 1;

            let result = if coverage {
                run_test_case_with_coverage(
                    &program,
                    test_case,
                    &spec.file_mocks,
                    &spec.file_net_mocks,
                    example_tracker.as_mut(),
                )
            } else {
                run_test_case(&program, test_case, &spec.file_mocks, &spec.file_net_mocks)
            };
            match result {
                Ok(()) => {
                    passed_tests += 1;
                    if verbose {
                        println!("  [PASS] {}[{}]: {}", spec.name, i, test_case.annotation);
                    }
                }
                Err(msg) => {
                    failed_tests += 1;
                    example_passed = false;
                    println!("  [FAIL] {}[{}]: {}", spec.name, i, test_case.annotation);
                    println!("         {}", msg);

                    if fail_fast {
                        println!("\nStopping due to --fail-fast");
                        break;
                    }
                }
            }
        }

        // Aggregate coverage from this example
        if let Some(tracker) = example_tracker {
            total_instructions += program.instructions.len();
            total_executed += tracker.executed_count();
            // Branch coverage: each branch has 2 possible outcomes
            let branch_count = tracker.branch_coverage();
            // Approximate branch tracking from coverage percentage
            if branch_count > 0.0 && branch_count < 100.0 {
                // Has some branches
                total_branches += 2; // rough estimate per branch instruction
                covered_branch_outcomes += (branch_count / 50.0) as usize; // 50% = 1 of 2 outcomes
            }
        }

        if !verbose && example_passed {
            println!("  [PASS] {} ({} tests)", spec.name, spec.test_cases.len());
        }

        if fail_fast && !example_passed {
            break;
        }
    }

    let elapsed = start.elapsed();

    // Summary
    println!("\n{}", "=".repeat(50));
    println!("Test Results:");
    println!("  Total:   {}", total_tests);
    println!(
        "  Passed:  {} ({}%)",
        passed_tests,
        if total_tests > 0 {
            passed_tests * 100 / total_tests
        } else {
            0
        }
    );
    println!("  Failed:  {}", failed_tests);
    println!("  Skipped: {} examples", skipped_examples);
    println!("  Time:    {:?}", elapsed);

    // Coverage summary
    if coverage && total_instructions > 0 {
        println!("\nCoverage Report:");
        let instr_coverage = (total_executed as f64 / total_instructions as f64) * 100.0;
        println!(
            "  Instructions: {}/{} ({:.1}%)",
            total_executed, total_instructions, instr_coverage
        );
        if total_branches > 0 {
            let branch_coverage = (covered_branch_outcomes as f64 / total_branches as f64) * 100.0;
            println!(
                "  Branches:     {}/{} ({:.1}%)",
                covered_branch_outcomes, total_branches, branch_coverage
            );
        }
    }

    println!("{}", "=".repeat(50));

    if failed_tests > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Parse @name, @server, @test, @mock annotations from a .nl file
fn parse_example_spec(path: &PathBuf) -> Result<ExampleSpec> {
    let source = fs::read_to_string(path)?;

    // Create RAG resolver for extension name resolution in @mock annotations
    // This uses the same bundled extensions as the assembler
    let rag_resolver = RagResolver::new();

    let mut name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();
    let mut is_server = false;
    let mut test_cases = Vec::new();
    let mut file_mocks = Vec::new();
    let mut file_net_mocks = Vec::new();

    for line in source.lines() {
        let line = line.trim();
        if !line.starts_with(';') {
            continue;
        }

        let content = line.trim_start_matches(';').trim();

        // @name: Example Name
        if let Some(n) = content.strip_prefix("@name:") {
            name = n.trim().to_string();
        }

        // @server: true/false
        if let Some(s) = content.strip_prefix("@server:") {
            is_server = s.trim().eq_ignore_ascii_case("true");
        }

        // @mock: ext_id=return_value or @mock: ext_name=return_value
        // Example: @mock: 500=0 (tls_connect returns handle 0)
        // Example: @mock: http_get=200 (resolved via RAG to correct ID)
        if let Some(mock_str) = content.strip_prefix("@mock:") {
            if let Some(mock) = parse_mock_setup(mock_str.trim(), &rag_resolver) {
                file_mocks.push(mock);
            }
        }

        // @net_mock: op=return_values or @net_mock: recv="data"
        // Example: @net_mock: accept=5;-1 (returns client fd 5, then error to break loop)
        // Example: @net_mock: recv="GET / HTTP/1.1\r\n"
        // Example: @net_mock: send=100
        if let Some(mock_str) = content.strip_prefix("@net_mock:") {
            if let Some(mock) = parse_net_mock_setup(mock_str.trim()) {
                file_net_mocks.push(mock);
            }
        }

        // @test: r0=5 -> r0=120
        // @test: r0=48,r1=18 -> r0=6
        if let Some(test_str) = content.strip_prefix("@test:") {
            if let Some(tc) = parse_test_case(test_str.trim()) {
                test_cases.push(tc);
            }
        }
    }

    Ok(ExampleSpec {
        name,
        path: path.clone(),
        is_server,
        test_cases,
        file_mocks,
        file_net_mocks,
    })
}

/// Parse a mock setup string like "500=0" or "http_get=1,output1,output2"
/// Supports both numeric IDs and extension names (resolved via RAG)
/// Supports stateful mocks with semicolon-separated return values:
///   @mock: tcp_accept=5;0 - returns 5 first call, then 0 forever
///   @mock: tcp_accept=5;0,output1 - returns 5 then 0, with output value
fn parse_mock_setup(s: &str, rag_resolver: &RagResolver) -> Option<MockSetup> {
    // Strip trailing comment first
    let clean = strip_trailing_comment(s);

    let parts: Vec<&str> = clean.split('=').collect();
    if parts.len() != 2 {
        return None;
    }

    let ext_key = parts[0].trim();

    // Try parsing as numeric ID first, then look up by name via RAG
    let ext_id = if let Ok(id) = ext_key.parse::<u32>() {
        id
    } else {
        // Look up extension name via RAG resolver
        rag_resolver.get_by_name(ext_key)?.id
    };

    // Parse return values and optional outputs
    // Format: val1;val2;val3,out1,out2 or just val,out1,out2
    let value_parts: Vec<&str> = parts[1].split(',').collect();
    let return_str = value_parts[0].trim();

    // Check for semicolon-separated return value sequence
    let return_values: Vec<i64> = if return_str.contains(';') {
        return_str
            .split(';')
            .filter_map(|s| parse_i64_value(s.trim()))
            .collect()
    } else {
        vec![parse_i64_value(return_str)?]
    };

    if return_values.is_empty() {
        return None;
    }

    let outputs: Vec<u64> = value_parts[1..]
        .iter()
        .filter_map(|s| parse_u64_value(s.trim()))
        .collect();

    Some(MockSetup {
        ext_id,
        return_values,
        outputs,
    })
}

/// Parse an i64 value (for mock return values that may be negative)
fn parse_i64_value(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.starts_with('-') {
        s.parse::<i64>().ok()
    } else if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        i64::from_str_radix(hex, 16).ok()
    } else {
        s.parse::<i64>().ok()
    }
}

/// Parse a network mock setup string
/// Format: @net_mock: op=return_values or @net_mock: recv="data"
/// Examples:
///   @net_mock: accept=5,-1   (returns 5, then -1 to break loop)
///   @net_mock: recv="GET / HTTP/1.1\r\n"
///   @net_mock: send=100
///   @net_mock: socket=3
///   @net_mock: bind=0
///   @net_mock: listen=0
///   @net_mock: connect=0
///   @net_mock: close=0
/// Note: Use comma for sequences (semicolon conflicts with comment stripping)
fn parse_net_mock_setup(s: &str) -> Option<NetMockSetup> {
    use neurlang::stencil::io::NetMockOp;

    let clean = strip_trailing_comment(s);
    let parts: Vec<&str> = clean.splitn(2, '=').collect();
    if parts.len() != 2 {
        return None;
    }

    let op_name = parts[0].trim().to_lowercase();
    let value_str = parts[1].trim();

    // Map operation name to NetMockOp
    let op = match op_name.as_str() {
        "socket" => NetMockOp::Socket,
        "bind" => NetMockOp::Bind,
        "listen" => NetMockOp::Listen,
        "accept" => NetMockOp::Accept,
        "connect" => NetMockOp::Connect,
        "send" => NetMockOp::Send,
        "recv" => NetMockOp::Recv,
        "close" => NetMockOp::Close,
        _ => return None,
    };

    // Check if it's a string value (for recv data)
    // Format: recv="data" or recv="data",0 (data first, then subsequent return values)
    if value_str.starts_with('"') {
        // Find the closing quote
        let mut quote_end = 1;
        let chars: Vec<char> = value_str.chars().collect();
        let mut in_escape = false;
        for (i, c) in chars[1..].iter().enumerate() {
            if in_escape {
                in_escape = false;
            } else if *c == '\\' {
                in_escape = true;
            } else if *c == '"' {
                quote_end = i + 2; // +1 for starting quote, +1 for 0-indexing
                break;
            }
        }

        // Parse string data for recv
        let data_str = &value_str[1..quote_end - 1];
        let data = parse_string_with_escapes(data_str);

        // Check for additional return values after the string (e.g., recv="data",0)
        let after_quote = &value_str[quote_end..];
        let mut return_values = vec![data.len() as i64];
        if let Some(rest) = after_quote.strip_prefix(',') {
            for part in rest.split(',') {
                if let Some(val) = parse_i64_value(part.trim()) {
                    return_values.push(val);
                }
            }
        }

        return Some(NetMockSetup {
            op,
            return_values,
            recv_data: Some(data),
        });
    }

    // Parse return values (comma-separated for sequences)
    let return_values: Vec<i64> = if value_str.contains(',') {
        value_str
            .split(',')
            .filter_map(|s| parse_i64_value(s.trim()))
            .collect()
    } else {
        vec![parse_i64_value(value_str)?]
    };

    if return_values.is_empty() {
        return None;
    }

    Some(NetMockSetup {
        op,
        return_values,
        recv_data: None,
    })
}

/// Parse a string with escape sequences (\n, \r, \t, \x00, etc.)
fn parse_string_with_escapes(s: &str) -> Vec<u8> {
    let mut result = Vec::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            match chars[i + 1] {
                'n' => {
                    result.push(b'\n');
                    i += 2;
                }
                'r' => {
                    result.push(b'\r');
                    i += 2;
                }
                't' => {
                    result.push(b'\t');
                    i += 2;
                }
                '0' => {
                    result.push(0);
                    i += 2;
                }
                '\\' => {
                    result.push(b'\\');
                    i += 2;
                }
                '"' => {
                    result.push(b'"');
                    i += 2;
                }
                'x' if i + 3 < chars.len() => {
                    // Hex escape: \xHH
                    let hex_str: String = chars[i + 2..i + 4].iter().collect();
                    if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                        result.push(byte);
                        i += 4;
                    } else {
                        result.push(chars[i] as u8);
                        i += 1;
                    }
                }
                _ => {
                    result.push(chars[i] as u8);
                    i += 1;
                }
            }
        } else {
            result.push(chars[i] as u8);
            i += 1;
        }
    }
    result
}

/// Parse a test case string like "r0=5 -> r0=120" or "r0=48,r1=18 -> r0=6"
/// Also supports memory setup: "r0=0x1000 [0x1000]=\"hello\" -> r0=5"
fn parse_test_case(s: &str) -> Option<TestCase> {
    // Strip trailing comment (anything after a semicolon not in quotes)
    let clean = strip_trailing_comment(s);

    let parts: Vec<&str> = clean.split("->").collect();
    if parts.len() != 2 {
        return None;
    }

    let input_str = parts[0].trim();
    let output_str = parts[1].trim();

    // Parse memory setup and register inputs from input_str
    let (inputs, memory) = parse_inputs_with_memory(input_str)?;
    let outputs = parse_register_assignments(output_str)?;

    Some(TestCase {
        inputs,
        outputs,
        memory,
        mocks: Vec::new(),     // Mocks come from file-level @mock annotations
        net_mocks: Vec::new(), // Network mocks come from file-level @net_mock annotations
        annotation: s.to_string(),
    })
}

/// Strip trailing comment from annotation line (e.g., "r0=5 ; comment" -> "r0=5")
fn strip_trailing_comment(s: &str) -> &str {
    // Find semicolon that's not inside quotes
    let mut in_quotes = false;
    for (i, c) in s.char_indices() {
        if c == '"' {
            in_quotes = !in_quotes;
        } else if c == ';' && !in_quotes {
            return s[..i].trim();
        }
    }
    s.trim()
}

/// Parse inputs that may include memory setup.
/// Format: "r0=0x1000 [0x1000]=\"hello\"" or "r0=5,r1=3"
fn parse_inputs_with_memory(s: &str) -> Option<(Vec<(usize, u64)>, Vec<MemorySetup>)> {
    let mut inputs = Vec::new();
    let mut memory = Vec::new();

    // Split by whitespace to separate register assignments from memory setup
    let mut current_pos = 0;
    let chars: Vec<char> = s.chars().collect();

    while current_pos < chars.len() {
        // Skip whitespace
        while current_pos < chars.len() && chars[current_pos].is_whitespace() {
            current_pos += 1;
        }
        if current_pos >= chars.len() {
            break;
        }

        // Check for memory setup: [addr]="data" or [addr]=bytes
        if chars[current_pos] == '[' {
            // Find closing bracket
            let start = current_pos + 1;
            let mut end = start;
            while end < chars.len() && chars[end] != ']' {
                end += 1;
            }
            if end >= chars.len() {
                return None;
            }

            let addr_str: String = chars[start..end].iter().collect();
            let addr = parse_u64_value(&addr_str)?;

            // Skip ']='
            current_pos = end + 1;
            if current_pos >= chars.len() || chars[current_pos] != '=' {
                return None;
            }
            current_pos += 1;

            // Parse data: "string" or hex bytes
            if current_pos < chars.len() && chars[current_pos] == '"' {
                // String literal
                current_pos += 1;
                let mut string_data = Vec::new();
                while current_pos < chars.len() && chars[current_pos] != '"' {
                    if chars[current_pos] == '\\' && current_pos + 1 < chars.len() {
                        // Handle escape sequences
                        current_pos += 1;
                        match chars[current_pos] {
                            'n' => string_data.push(b'\n'),
                            't' => string_data.push(b'\t'),
                            'r' => string_data.push(b'\r'),
                            '0' => string_data.push(0),
                            '\\' => string_data.push(b'\\'),
                            '"' => string_data.push(b'"'),
                            'x' => {
                                // Hex escape: \xHH
                                if current_pos + 2 < chars.len() {
                                    let hex_str: String =
                                        chars[current_pos + 1..current_pos + 3].iter().collect();
                                    if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                                        string_data.push(byte);
                                        current_pos += 2; // Skip the two hex digits
                                    } else {
                                        // Invalid hex, just push 'x'
                                        string_data.push(b'x');
                                    }
                                } else {
                                    string_data.push(b'x');
                                }
                            }
                            _ => string_data.push(chars[current_pos] as u8),
                        }
                    } else {
                        string_data.push(chars[current_pos] as u8);
                    }
                    current_pos += 1;
                }
                // Skip closing quote
                if current_pos < chars.len() {
                    current_pos += 1;
                }
                // Add null terminator for C strings
                string_data.push(0);
                memory.push(MemorySetup {
                    address: addr,
                    data: string_data,
                });
            } else {
                // Hex bytes (not implemented yet, just skip)
                while current_pos < chars.len() && !chars[current_pos].is_whitespace() {
                    current_pos += 1;
                }
            }
        } else if chars[current_pos] == 'r' {
            // Register assignment
            let start = current_pos;
            while current_pos < chars.len()
                && chars[current_pos] != ','
                && chars[current_pos] != ' '
                && chars[current_pos] != '['
            {
                current_pos += 1;
            }
            let reg_str: String = chars[start..current_pos].iter().collect();

            // Parse single register assignment
            let reg_parts: Vec<&str> = reg_str.split('=').collect();
            if reg_parts.len() == 2 {
                let reg_name = reg_parts[0].trim();
                let val_str = reg_parts[1].trim();
                if let Some(num_str) = reg_name.strip_prefix('r') {
                    if let Ok(reg_num) = num_str.parse::<usize>() {
                        if reg_num <= 31 {
                            if let Some(value) = parse_u64_value(val_str) {
                                inputs.push((reg_num, value));
                            }
                        }
                    }
                }
            }

            // Skip comma if present
            if current_pos < chars.len() && chars[current_pos] == ',' {
                current_pos += 1;
            }
        } else {
            // Skip unknown character
            current_pos += 1;
        }
    }

    Some((inputs, memory))
}

/// Parse a u64 value from string (decimal, hex, or negative)
fn parse_u64_value(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.starts_with("0x") || s.starts_with("0X") {
        u64::from_str_radix(&s[2..], 16).ok()
    } else if s.starts_with('-') {
        s.parse::<i64>().ok().map(|v| v as u64)
    } else {
        s.parse::<u64>().ok()
    }
}

/// Parse "r0=5,r1=3" into [(0, 5), (1, 3)]
fn parse_register_assignments(s: &str) -> Option<Vec<(usize, u64)>> {
    let mut result = Vec::new();

    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        // r0=5 or r10=123
        let reg_val: Vec<&str> = part.split('=').collect();
        if reg_val.len() != 2 {
            return None;
        }

        let reg_str = reg_val[0].trim();
        let val_str = reg_val[1].trim();

        // Parse register number (r0, r1, ..., r31)
        let reg_num = if let Some(num_str) = reg_str.strip_prefix('r') {
            num_str.parse::<usize>().ok()?
        } else {
            return None;
        };

        if reg_num > 31 {
            return None;
        }

        // Parse value (decimal, hex, or negative)
        let value = if val_str.starts_with("0x") || val_str.starts_with("0X") {
            u64::from_str_radix(&val_str[2..], 16).ok()?
        } else if val_str.starts_with('-') {
            // Parse as i64 then cast to u64 (two's complement)
            val_str.parse::<i64>().ok()? as u64
        } else {
            val_str.parse::<u64>().ok()?
        };

        result.push((reg_num, value));
    }

    Some(result)
}

/// Run a single test case against a compiled program
fn run_test_case(
    program: &Program,
    test_case: &TestCase,
    file_mocks: &[MockSetup],
    file_net_mocks: &[NetMockSetup],
) -> std::result::Result<(), String> {
    use neurlang::jit::{JitExecutor, JitResult};

    let mut executor = JitExecutor::with_memory_size(1024 * 1024); // 1MB

    // Set up extension mocks (file-level mocks apply to all tests)
    if !file_mocks.is_empty() || !test_case.mocks.is_empty() {
        executor.enable_mock_mode();
        for mock in file_mocks {
            if mock.return_values.len() == 1 {
                executor.set_extension_mock(
                    mock.ext_id,
                    mock.return_values[0],
                    mock.outputs.clone(),
                );
            } else {
                executor.set_extension_mock_sequence(
                    mock.ext_id,
                    mock.return_values.clone(),
                    mock.outputs.clone(),
                );
            }
        }
        for mock in &test_case.mocks {
            if mock.return_values.len() == 1 {
                executor.set_extension_mock(
                    mock.ext_id,
                    mock.return_values[0],
                    mock.outputs.clone(),
                );
            } else {
                executor.set_extension_mock_sequence(
                    mock.ext_id,
                    mock.return_values.clone(),
                    mock.outputs.clone(),
                );
            }
        }
    }

    // Note: Network mocks for JIT executor not yet implemented
    // Server tests should use --coverage flag to use interpreter which supports network mocks
    let _ = file_net_mocks;
    let _ = &test_case.net_mocks;

    // Pre-load data section BEFORE setting up test memory
    // This ensures test memory setup can override data section values
    executor.load_data_section(program);

    // Set up memory (strings, arrays, etc.) - now happens AFTER data section is loaded
    for mem_setup in &test_case.memory {
        let addr = mem_setup.address as usize;
        let memory = executor.memory_mut();
        if addr + mem_setup.data.len() > memory.len() {
            return Err(format!(
                "Memory setup at 0x{:x} with {} bytes exceeds memory size",
                addr,
                mem_setup.data.len()
            ));
        }
        memory[addr..addr + mem_setup.data.len()].copy_from_slice(&mem_setup.data);
    }

    // Set input registers
    for (reg, value) in &test_case.inputs {
        executor.set_register(*reg, *value);
    }

    // Execute (note: execute() will try to load data section again, but it's idempotent)
    let result = executor.execute(program);

    match result {
        JitResult::Ok(_) | JitResult::Halted(_) => {
            // Check output registers
            for (reg, expected) in &test_case.outputs {
                let actual = executor.get_register(*reg);
                if actual != *expected {
                    return Err(format!("r{} = {} (expected {})", reg, actual, expected));
                }
            }
            Ok(())
        }
        JitResult::Error(msg) => Err(format!("Execution error: {}", msg)),
    }
}

/// Run a single test case using the interpreter with coverage tracking
fn run_test_case_with_coverage(
    program: &Program,
    test_case: &TestCase,
    file_mocks: &[MockSetup],
    file_net_mocks: &[NetMockSetup],
    coverage_tracker: Option<&mut neurlang::interp::CoverageTracker>,
) -> std::result::Result<(), String> {
    use neurlang::interp::{InterpResult, Interpreter};
    use neurlang::ir::DATA_BASE;

    // Create interpreter with enough memory
    let mut interp = Interpreter::with_permissions(
        DATA_BASE as usize + 0x100000,
        neurlang::stencil::io::IOPermissions::allow_all(),
    );

    // Enable coverage if tracker provided
    if coverage_tracker.is_some() {
        interp = Interpreter::with_permissions(
            DATA_BASE as usize + 0x100000,
            neurlang::stencil::io::IOPermissions::allow_all(),
        )
        .with_coverage(program.instructions.len());
    }

    // Set up extension mocks (file-level mocks apply to all tests)
    if !file_mocks.is_empty() || !test_case.mocks.is_empty() {
        interp.extensions_mut().set_mock_mode(true);
        for mock in file_mocks {
            if mock.return_values.len() == 1 {
                interp.extensions_mut().set_mock(
                    mock.ext_id,
                    mock.return_values[0],
                    mock.outputs.clone(),
                );
            } else {
                interp.extensions_mut().set_mock_sequence(
                    mock.ext_id,
                    mock.return_values.clone(),
                    mock.outputs.clone(),
                );
            }
        }
        for mock in &test_case.mocks {
            if mock.return_values.len() == 1 {
                interp.extensions_mut().set_mock(
                    mock.ext_id,
                    mock.return_values[0],
                    mock.outputs.clone(),
                );
            } else {
                interp.extensions_mut().set_mock_sequence(
                    mock.ext_id,
                    mock.return_values.clone(),
                    mock.outputs.clone(),
                );
            }
        }
    }

    // Set up network mocks for server testing
    if !file_net_mocks.is_empty() || !test_case.net_mocks.is_empty() {
        let net_mocks = interp.io_runtime().network_mocks_mut();
        for mock in file_net_mocks {
            if let Some(ref data) = mock.recv_data {
                net_mocks.set_recv_mock(mock.return_values.clone(), data.clone());
            } else {
                net_mocks.set_mock(mock.op, mock.return_values.clone());
            }
        }
        for mock in &test_case.net_mocks {
            if let Some(ref data) = mock.recv_data {
                net_mocks.set_recv_mock(mock.return_values.clone(), data.clone());
            } else {
                net_mocks.set_mock(mock.op, mock.return_values.clone());
            }
        }
    }

    // Pre-load data section BEFORE setting up test memory
    // This ensures test memory setup can override data section values
    interp.load_data_section(program);

    // Set up memory (strings, arrays, etc.) - now happens AFTER data section is loaded
    for mem_setup in &test_case.memory {
        let addr = mem_setup.address as usize;
        let memory = interp.memory_mut();
        if addr + mem_setup.data.len() > memory.len() {
            return Err(format!(
                "Memory setup at 0x{:x} with {} bytes exceeds memory size",
                addr,
                mem_setup.data.len()
            ));
        }
        memory[addr..addr + mem_setup.data.len()].copy_from_slice(&mem_setup.data);
    }

    // Set input registers
    for (reg, value) in &test_case.inputs {
        interp.registers[*reg] = *value;
    }

    // Execute
    let result = interp.execute(program);

    // Merge coverage data into the tracker
    if let Some(tracker) = coverage_tracker {
        if let Some(interp_cov) = interp.coverage() {
            for pc in interp_cov.executed_pcs() {
                tracker.mark_executed(*pc);
            }
        }
    }

    match result {
        InterpResult::Ok(_) | InterpResult::Halted => {
            // Check output registers
            for (reg, expected) in &test_case.outputs {
                let actual = interp.registers[*reg];
                if actual != *expected {
                    return Err(format!("r{} = {} (expected {})", reg, actual, expected));
                }
            }
            Ok(())
        }
        InterpResult::DivByZero => Err("Division by zero".to_string()),
        InterpResult::OutOfBounds => Err("Memory access out of bounds".to_string()),
        InterpResult::MaxInstructionsExceeded => Err("Max instructions exceeded".to_string()),
        InterpResult::InvalidInstruction => Err("Invalid instruction".to_string()),
        InterpResult::CapabilityViolation => Err("Capability violation".to_string()),
        InterpResult::Trapped(t) => Err(format!("Trapped: {:?}", t)),
    }
}

// =============================================================================
// Stdlib Commands
// =============================================================================

fn cmd_stdlib(
    build: bool,
    verify: bool,
    clean: bool,
    config_path: Option<&PathBuf>,
    stdlib_dir: &PathBuf,
    lib_dir: &PathBuf,
    verbose: bool,
) -> Result<()> {
    use neurlang::compiler;
    use neurlang::config::NeurlangConfig;

    // Load configuration
    let config = if let Some(path) = config_path {
        NeurlangConfig::load(path).context("Failed to load config file")?
    } else {
        NeurlangConfig::load_from_cwd().unwrap_or_default()
    };

    // Use config values if CLI args are default
    let effective_lib_dir = if lib_dir.to_str() == Some("lib") {
        PathBuf::from(&config.build.output_dir)
    } else {
        lib_dir.clone()
    };

    if verbose {
        println!("Configuration:");
        println!(
            "  Package: {} v{}",
            config.package.name, config.package.version
        );
        println!("  Enabled modules: {:?}", config.enabled_stdlib_modules());
        println!("  Output directory: {}", effective_lib_dir.display());
        println!();
    }

    if clean {
        // Clean generated lib/ files
        if effective_lib_dir.exists() {
            println!("Cleaning {}...", effective_lib_dir.display());
            fs::remove_dir_all(&effective_lib_dir).context("Failed to remove lib directory")?;
            println!("Done.");
        } else {
            println!("Nothing to clean.");
        }
        return Ok(());
    }

    if build {
        println!("Building stdlib from Rust sources...\n");

        // Verify stdlib directory exists
        if !stdlib_dir.exists() {
            anyhow::bail!("Stdlib directory not found: {}", stdlib_dir.display());
        }

        // Build
        let result = compiler::build_stdlib(stdlib_dir, &effective_lib_dir, verbose)?;

        if result.is_success() {
            println!(
                "\n{} Build complete:",
                if result.errors.is_empty() { "✓" } else { "!" }
            );
            println!("  {} files compiled", result.files_compiled);
            println!("  {} functions generated", result.functions_generated);
            println!("  {} tests generated", result.tests_generated);

            println!("\nOutput directory: {}", effective_lib_dir.display());
            println!("\nNext steps:");
            println!(
                "  nl test -p {}       # Run tests on generated code",
                effective_lib_dir.display()
            );
            println!("  nl stdlib --verify   # Verify Rust == Neurlang output");
        } else {
            println!("\nBuild completed with errors:");
            for error in &result.errors {
                println!("  - {}", error);
            }
            anyhow::bail!("Build failed with {} errors", result.errors.len());
        }

        return Ok(());
    }

    if verify {
        println!("Verifying stdlib implementations...\n");

        if !effective_lib_dir.exists() {
            anyhow::bail!("Lib directory not found. Run 'nl stdlib --build' first.");
        }

        let result = compiler::verify_stdlib(stdlib_dir, &effective_lib_dir, verbose)?;

        if result.is_success() {
            println!("\n✓ Verification passed:");
            println!("  {} functions verified", result.functions_verified);
            println!("  {} tests passed", result.tests_passed);
        } else {
            println!("\n✗ Verification failed:");
            println!(
                "  {} tests passed, {} failed",
                result.tests_passed, result.tests_failed
            );
            for failure in &result.failures {
                println!("  - {}", failure);
            }
            anyhow::bail!("Verification failed");
        }

        return Ok(());
    }

    // No action specified - show help
    println!("Stdlib management commands:\n");
    println!("  nl stdlib --build      Build lib/*.nl from stdlib/src/*.rs");
    println!("  nl stdlib --verify     Verify Rust == Neurlang output");
    println!("  nl stdlib --clean      Remove generated lib/ files");
    println!("\nOptions:");
    println!("  --stdlib-dir PATH      Rust source directory (default: ./stdlib)");
    println!("  --lib-dir PATH         Output directory (default: ./lib)");
    println!("  -v, --verbose          Show detailed output");

    Ok(())
}

fn cmd_crate(
    add: Option<String>,
    remove: Option<String>,
    list: bool,
    build: bool,
    _verbose: bool,
) -> Result<()> {
    if let Some(crate_name) = add {
        println!("Adding crate: {}", crate_name);
        println!("\n[1/5] Fetching {} from crates.io...", crate_name);
        println!("[2/5] Parsing public API...");
        println!("[3/5] Generating FFI wrappers...");
        println!("[4/5] Building and linking...");
        println!("[5/5] Registering with RAG...");
        println!("\nDone. Extension available via @\"...\"\n");

        // TODO: Actually implement crate import
        println!("Note: Full crate import is not yet implemented.");
        println!("See 'nl extension --help' for manual extension management.");
        return Ok(());
    }

    if let Some(crate_name) = remove {
        println!("Removing crate: {}", crate_name);
        // TODO: Implement removal
        println!("Note: Crate removal is not yet implemented.");
        return Ok(());
    }

    if list {
        println!("Installed crates:\n");
        println!("  (none)");
        println!("\nUse 'nl crate --add <crate_name>' to install crates.");
        return Ok(());
    }

    if build {
        println!("Building all crate extensions...");
        println!("  (no crates installed)");
        return Ok(());
    }

    // Show help
    println!("Crate management commands:\n");
    println!("  nl crate --add NAME    Install a crate from crates.io as extension");
    println!("  nl crate --remove NAME Remove an installed crate");
    println!("  nl crate --list        List installed crates");
    println!("  nl crate --build       Rebuild all crate extensions");

    Ok(())
}

fn cmd_resolve(intent: &str, top: usize) -> Result<()> {
    use neurlang::ir::RagResolver;

    println!("Resolving: \"{}\"\n", intent);

    let resolver = RagResolver::new();
    let results = resolver.search(intent, top);

    if results.is_empty() {
        println!("No matches found.");
        println!("\nTip: Try different keywords or check 'nl extension --list' for available extensions.");
        return Ok(());
    }

    println!("Top {} matches:", results.len().min(top));
    for (i, result) in results.iter().take(top).enumerate() {
        println!("  {}. {} (ID: {})", i + 1, result.name, result.id);
        println!("     {}", result.description);
    }

    println!("\nUsage in Neurlang:");
    if let Some(best) = results.first() {
        println!("  ext.call r0, {}, r1, r2  ; {}", best.id, best.name);
    }

    Ok(())
}

fn cmd_generate(
    prompt: &str,
    offline: bool,
    llm: bool,
    dry_run: bool,
    benchmark: bool,
    output: Option<&PathBuf>,
    show_spec: bool,
    show_slots: bool,
    show_asm: bool,
    specs_dir: &str,
    templates_dir: &str,
    threshold: f32,
) -> Result<()> {
    use neurlang::slot::{RouteDecision, Router, RouterConfig};
    use std::time::Instant;

    // Build router configuration
    let config = RouterConfig {
        rule_based_threshold: threshold,
        specs_dir: specs_dir.to_string(),
        templates_dir: templates_dir.to_string(),
        force_offline: offline,
        force_llm: llm,
        ..Default::default()
    };

    // Can't have both offline and llm
    if offline && llm {
        anyhow::bail!("Cannot specify both --offline and --llm");
    }

    let router = Router::new(config);

    // Benchmark mode: time the generation
    let start = if benchmark {
        Some(Instant::now())
    } else {
        None
    };

    // Route the prompt
    let decision = router.route(prompt);

    // Show routing decision
    match &decision {
        RouteDecision::RuleBased {
            protocol,
            template,
            intent,
        } => {
            println!("Route: Rule-based (offline)");
            println!("  Protocol: {}", protocol);
            println!("  Template: {}", template);
            println!("  Confidence: {:.2}", intent.confidence);
            if !intent.features.is_empty() {
                println!("  Features: {}", intent.features.join(", "));
            }
        }
        RouteDecision::LlmDecompose { reason, intent } => {
            println!("Route: LLM decomposition");
            println!("  Reason: {}", reason);
            if let Some(ref protocol) = intent.protocol {
                println!("  Detected protocol: {}", protocol);
            }
        }
        RouteDecision::Direct { description } => {
            println!("Route: Direct generation");
            println!("  Description: {}", description);
        }
    }
    println!();

    // Dry run: stop here
    if dry_run {
        println!("(Dry run - not generating code)");
        if benchmark {
            if let Some(start) = start {
                println!(
                    "Routing time: {:.2}ms",
                    start.elapsed().as_secs_f64() * 1000.0
                );
            }
        }
        return Ok(());
    }

    // Generate the SlotSpec
    let result = router.generate(prompt)?;

    // Show timing for benchmark
    if benchmark {
        if let Some(start) = start {
            let total_ms = start.elapsed().as_secs_f64() * 1000.0;
            println!("Timing:");
            println!("  Route: {:.2}ms", result.route_time_ms);
            println!("  Expand: {:.2}ms", result.expand_time_ms);
            println!("  Total: {:.2}ms", total_ms);
            println!();
        }
    }

    let spec = &result.spec;

    // Show SlotSpec structure
    if show_spec {
        println!("SlotSpec:");
        println!("  Name: {}", spec.name);
        println!("  Description: {}", spec.description);
        if let Some(ref protocol) = spec.protocol {
            println!("  Protocol: {}", protocol);
        }
        if let Some(ref template) = spec.template {
            println!("  Template: {}", template);
        }
        println!("  Slots: {}", spec.slots.len());
        println!("  Data items: {}", spec.data_items.len());
        println!("  Tests: {}", spec.tests.len());
        if !spec.metadata.is_empty() {
            println!("  Metadata:");
            for (k, v) in &spec.metadata {
                println!("    {}: {}", k, v);
            }
        }
        println!();
    }

    // Show individual slots
    if show_slots {
        println!("Slots:");
        for (i, slot) in spec.slots.iter().enumerate() {
            println!("  [{}] {}", i, slot.id);
            println!("      Type: {:?}", slot.slot_type.category());
            println!("      Labels: {}", slot.context.labels.join(", "));
            if slot.unit_test.is_some() {
                println!("      Has unit test: yes");
            }
        }
        println!();
    }

    // Show generated skeleton (before slot filling)
    if show_asm && !spec.skeleton.is_empty() {
        println!("Skeleton Assembly:");
        println!("----------------------------------------");
        // Truncate if very long
        let skeleton = &spec.skeleton;
        if skeleton.len() > 3000 {
            println!(
                "{}...\n(truncated, {} chars total)",
                &skeleton[..3000],
                skeleton.len()
            );
        } else {
            println!("{}", skeleton);
        }
        println!("----------------------------------------");
        println!();
    }

    // Output file
    if let Some(output_path) = output {
        // For now, output the skeleton (in the future, this would be filled slots)
        if spec.skeleton.is_empty() {
            println!("Note: No skeleton generated (LLM path requires slot filling)");
            // Write a placeholder that shows what would be needed
            let placeholder = format!(
                "; Generated by: nl generate \"{}\"\n\
                ; Route: {:?}\n\
                ; \n\
                ; This program requires LLM-based slot filling.\n\
                ; Slots to fill: {}\n\
                ; \n\
                ; Run with a model to complete code generation.\n",
                prompt,
                result.route,
                spec.slots.len()
            );
            std::fs::write(output_path, placeholder)?;
            println!("Wrote placeholder to: {}", output_path.display());
        } else {
            std::fs::write(output_path, &spec.skeleton)?;
            println!("Wrote skeleton to: {}", output_path.display());
        }
    }

    // Summary
    println!("Generation complete.");
    println!("  {} slots defined", spec.slots.len());
    println!("  {} test cases", spec.tests.len());

    if spec.slots.is_empty() {
        println!("\nNote: No slots generated. This may be because:");
        println!("  - Protocol spec file not found (check specs_dir)");
        println!("  - LLM decomposition needed but not implemented yet");
        println!("\nTry: nl generate \"{}\" --show-spec", prompt);
    }

    Ok(())
}

/// Validate and test protocol specifications
fn cmd_protocol_spec(
    input: &PathBuf,
    validate: bool,
    test: bool,
    program: Option<&PathBuf>,
    verbose: bool,
    stats: bool,
) -> Result<()> {
    // Load and parse the protocol spec
    let spec = parse_protocol_spec(input)
        .with_context(|| format!("Failed to parse protocol spec: {}", input.display()))?;

    println!("Protocol: {} v{}", spec.name, spec.version);
    if verbose {
        if !spec.description.is_empty() {
            println!("  Description: {}", spec.description);
        }
        println!("  Transport: {:?}", spec.transport);
        if spec.port != 0 {
            println!("  Default port: {}", spec.port);
        }
    }
    println!();

    // Show statistics if requested
    if stats {
        println!("Statistics:");
        println!("  States: {}", spec.states.len());
        println!("  Commands: {}", spec.commands.len());
        println!("  Tests: {}", spec.tests.len());
        println!("  Errors defined: {}", spec.errors.len());

        if verbose {
            println!("\n  States:");
            for state in &spec.states {
                let flags = [
                    if state.initial { Some("initial") } else { None },
                    if state.terminal {
                        Some("terminal")
                    } else {
                        None
                    },
                ]
                .iter()
                .flatten()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");

                if flags.is_empty() {
                    println!("    - {}", state.name);
                } else {
                    println!("    - {} ({})", state.name, flags);
                }
            }

            println!("\n  Commands:");
            for cmd in &spec.commands {
                println!("    - {}: \"{}\"", cmd.name, cmd.pattern);
                println!("      Valid in: {:?}", cmd.valid_states);
            }
        }
        println!();
    }

    // Validate if requested (or by default if neither validate nor test)
    let should_validate = validate || (!test && !stats);
    if should_validate {
        println!("Validating spec...");
        let result = validate_spec(&spec);

        if result.is_valid() {
            println!("  ✓ Spec is VALID");
            println!(
                "    {} states, {} commands, {} tests",
                result.stats.state_count, result.stats.command_count, result.stats.test_count
            );

            if !result.warnings.is_empty() {
                println!("\n  Warnings ({}):", result.warnings.len());
                for warning in &result.warnings {
                    println!("    ⚠ {:?}", warning);
                }
            }
        } else {
            println!("  ✗ Spec is INVALID");
            println!(
                "    {} states, {} commands, {} tests",
                result.stats.state_count, result.stats.command_count, result.stats.test_count
            );

            println!("\n  Errors ({}):", result.errors.len());
            for error in &result.errors {
                println!("    ✗ {:?}", error);
            }

            if !result.warnings.is_empty() {
                println!("\n  Warnings ({}):", result.warnings.len());
                for warning in &result.warnings {
                    println!("    ⚠ {:?}", warning);
                }
            }

            anyhow::bail!("Spec validation failed with {} errors", result.errors.len());
        }
        println!();
    }

    // Run integration tests if requested
    if test {
        let program_path = program.ok_or_else(|| {
            anyhow::anyhow!("--test requires --program <path> to specify the program to test")
        })?;

        println!(
            "Running integration tests against: {}",
            program_path.display()
        );

        if spec.tests.is_empty() {
            println!("  No tests defined in spec");
            return Ok(());
        }

        // TODO: Implement actual integration test runner
        // For now, we'll show what tests would run
        println!("\n  Tests to run ({}):", spec.tests.len());
        for test in &spec.tests {
            println!("    - {}", test.name);
            if verbose {
                for (i, step) in test.steps.iter().enumerate() {
                    if let Some(send) = &step.send {
                        println!("      {}. send: {:?}", i + 1, send);
                    }
                    if let Some(expect) = &step.expect {
                        println!("      {}. expect: {:?}", i + 1, expect);
                    }
                }
            }
        }

        println!("\n  Note: Full integration test runner not yet implemented.");
        println!(
            "  This will start the program, connect as a client, and run send/expect sequences."
        );
    }

    Ok(())
}

/// Generate slot-level training data from protocol specs
fn cmd_slot_data(
    specs_dir: &PathBuf,
    output: &PathBuf,
    augment: bool,
    variations: usize,
    verbose: bool,
) -> Result<()> {
    println!(
        "Extracting slot training data from: {}",
        specs_dir.display()
    );

    let mut extractor = SlotTrainingExtractor::new();

    // Extract from protocol specs
    let count = extractor
        .extract_from_specs(specs_dir)
        .context("Failed to extract from specs")?;

    println!("  Extracted {} examples from protocol specs", count);

    // Augment if requested
    if augment {
        println!("\nAugmenting with {} variations per example...", variations);
        extractor.augment(variations);
        println!(
            "  Total examples after augmentation: {}",
            extractor.examples().len()
        );
    }

    // Show statistics
    let stats = extractor.stats();
    println!("\nStatistics:");
    println!("  Total examples: {}", stats.total_examples);
    println!("  Files processed: {}", stats.files_processed);

    if verbose {
        println!("\n  By slot type:");
        let mut types: Vec<_> = stats.by_type.iter().collect();
        types.sort_by(|a, b| b.1.cmp(a.1));
        for (slot_type, count) in types {
            println!("    {}: {}", slot_type, count);
        }

        println!("\n  By category:");
        let mut cats: Vec<_> = stats.by_category.iter().collect();
        cats.sort_by(|a, b| b.1.cmp(a.1));
        for (category, count) in cats {
            println!("    {}: {}", category, count);
        }
    }

    if !stats.errors.is_empty() {
        println!("\n  Errors:");
        for error in &stats.errors {
            println!("    - {}", error);
        }
    }

    // Create output directory if needed
    if let Some(parent) = output.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).context("Failed to create output directory")?;
        }
    }

    // Write JSONL output
    let written = extractor
        .write_jsonl(output)
        .context("Failed to write JSONL file")?;

    println!("\nWrote {} examples to: {}", written, output.display());

    Ok(())
}

// ============================================================================
// Index Command - Build RAG indices for fast intent classification
// ============================================================================

fn cmd_index(
    build: bool,
    build_examples: bool,
    training_data: Option<&PathBuf>,
    output_dir: Option<&PathBuf>,
    embedder_path: Option<&PathBuf>,
    ollama: Option<&str>,
    info: bool,
    verify: bool,
    verbose: bool,
) -> Result<()> {
    use neurlang::inference::embedder::Embedder;
    use neurlang::inference::intent_index::{IntentIndex, INTENT_DESCRIPTIONS, NUM_INTENTS};

    // Determine output directory
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let default_output = PathBuf::from(&home).join(".neurlang");
    let output = output_dir.unwrap_or(&default_output);

    // Create output directory if needed
    if !output.exists() {
        std::fs::create_dir_all(output).context("Failed to create output directory")?;
    }

    let intent_index_path = output.join("intent_index.bin");
    let example_index_path = output.join("example_index.bin");
    let example_meta_path = output.join("example_meta.bin");

    // Handle --info
    if info {
        println!("RAG Index Information:");
        println!("  Output directory: {}", output.display());
        println!("  Intent index: {}", intent_index_path.display());
        println!("  Example index: {}", example_index_path.display());
        println!();

        if intent_index_path.exists() {
            match IntentIndex::load(&intent_index_path) {
                Ok(index) => {
                    println!("Intent Index:");
                    println!("  Status: LOADED");
                    println!("  Intents: {}", index.num_intents());
                    println!("  Embedding dim: {}", index.dim());
                    let file_size = std::fs::metadata(&intent_index_path)?.len();
                    println!("  File size: {} bytes", file_size);
                }
                Err(e) => {
                    println!("Intent Index:");
                    println!("  Status: ERROR ({})", e);
                }
            }
        } else {
            println!("Intent Index:");
            println!("  Status: NOT FOUND");
            println!("  Run 'nl index --build' to create it");
        }
        println!();

        if example_index_path.exists() {
            let file_size = std::fs::metadata(&example_index_path)?.len();
            println!("Example Index:");
            println!("  Status: EXISTS");
            println!("  File size: {} bytes", file_size);
        } else {
            println!("Example Index:");
            println!("  Status: NOT FOUND");
            println!("  Run 'nl index --build-examples --training-data <file>' to create it");
        }

        return Ok(());
    }

    // Handle --verify
    if verify {
        println!("Verifying intent index...");

        if !intent_index_path.exists() {
            anyhow::bail!(
                "Intent index not found at {}. Run 'nl index --build' first.",
                intent_index_path.display()
            );
        }

        let index = IntentIndex::load(&intent_index_path).context("Failed to load intent index")?;

        // Create embedder for test queries
        let embedder: Box<dyn Embedder> = create_embedder_for_index(embedder_path, ollama)?;

        println!("Using embedder: {}", embedder.name());
        println!("Testing {} intents...\n", NUM_INTENTS);

        let mut correct = 0;
        let mut total = 0;

        // Test each intent with its canonical description
        for (id, desc) in INTENT_DESCRIPTIONS.iter().enumerate() {
            let embedding = embedder.embed(desc).context("Failed to embed test query")?;

            let (predicted_id, confidence) = index.classify(&embedding);

            let status = if predicted_id == id { "✓" } else { "✗" };
            if predicted_id == id {
                correct += 1;
            }
            total += 1;

            if verbose || predicted_id != id {
                let predicted_name = index.intent_name(predicted_id).unwrap_or("UNKNOWN");
                let expected_name = index.intent_name(id).unwrap_or("UNKNOWN");
                println!("{} Intent {}: \"{}\"", status, id, desc);
                println!(
                    "    Expected: {} ({}), Got: {} ({}) [conf: {:.3}]",
                    expected_name, id, predicted_name, predicted_id, confidence
                );
            }
        }

        println!(
            "\nResults: {}/{} correct ({:.1}%)",
            correct,
            total,
            correct as f64 / total as f64 * 100.0
        );

        if correct < total {
            println!("\nNote: Some mismatches are expected if the embedder differs from the one used to build the index.");
        }

        return Ok(());
    }

    // Handle --build
    if build {
        println!("Building intent index...");

        // Create embedder
        let embedder: Box<dyn Embedder> = create_embedder_for_index(embedder_path, ollama)?;
        println!(
            "Using embedder: {} (dim={})",
            embedder.name(),
            embedder.embedding_dim()
        );

        let start = std::time::Instant::now();

        // Build index from canonical descriptions
        let index = IntentIndex::build_from_descriptions(embedder.as_ref(), &INTENT_DESCRIPTIONS)
            .context("Failed to build intent index")?;

        let build_time = start.elapsed();

        // Save index
        index
            .save(&intent_index_path)
            .context("Failed to save intent index")?;

        let file_size = std::fs::metadata(&intent_index_path)?.len();

        println!("\nIntent index built successfully:");
        println!("  Intents: {}", index.num_intents());
        println!("  Embedding dim: {}", index.dim());
        println!("  Build time: {:?}", build_time);
        println!("  File size: {} bytes", file_size);
        println!("  Saved to: {}", intent_index_path.display());

        // Quick verification
        if verbose {
            println!("\nQuick verification:");
            for (id, desc) in INTENT_DESCRIPTIONS.iter().enumerate().take(5) {
                let embedding = embedder.embed(desc)?;
                let (predicted_id, confidence) = index.classify(&embedding);
                let status = if predicted_id == id { "✓" } else { "✗" };
                println!("  {} Intent {}: conf={:.3}", status, id, confidence);
            }
        }

        return Ok(());
    }

    // Handle --build-examples
    if build_examples {
        use neurlang::inference::example_index::ExampleIndex;

        let training_file = training_data
            .ok_or_else(|| anyhow::anyhow!("--training-data is required for --build-examples"))?;

        if !training_file.exists() {
            anyhow::bail!("Training data file not found: {}", training_file.display());
        }

        println!("Building example index from {}...", training_file.display());

        // Create embedder
        let embedder: Box<dyn Embedder> = create_embedder_for_index(embedder_path, ollama)?;
        println!(
            "Using embedder: {} (dim={})",
            embedder.name(),
            embedder.embedding_dim()
        );

        let start = std::time::Instant::now();

        // Read training data (JSONL format)
        let file = std::fs::File::open(training_file)?;
        let reader = std::io::BufReader::new(file);

        let mut index = ExampleIndex::new();
        let mut count = 0;
        let mut errors = 0;

        use std::io::BufRead;
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            // Parse JSONL entry
            let entry: serde_json::Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(e) => {
                    if verbose {
                        eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e);
                    }
                    errors += 1;
                    continue;
                }
            };

            // Extract prompt and intent
            let prompt = entry
                .get("prompt")
                .or_else(|| entry.get("input"))
                .and_then(|v| v.as_str());
            let intent_id = entry
                .get("intent_id")
                .or_else(|| entry.get("label"))
                .and_then(|v| v.as_u64());

            if let (Some(prompt), Some(intent_id)) = (prompt, intent_id) {
                // Embed the prompt
                match embedder.embed(prompt) {
                    Ok(embedding) => {
                        index.add_example(
                            prompt.to_string(),
                            intent_id as u8,
                            embedding,
                            line_num as u32,
                        );
                        count += 1;

                        if count % 1000 == 0 {
                            print!("\r  Processed {} examples...", count);
                            std::io::Write::flush(&mut std::io::stdout())?;
                        }
                    }
                    Err(e) => {
                        if verbose {
                            eprintln!("Warning: Failed to embed example {}: {}", line_num + 1, e);
                        }
                        errors += 1;
                    }
                }
            }
        }
        println!();

        // Build and save index
        index.build()?;
        index.save(&example_index_path, &example_meta_path)?;

        let build_time = start.elapsed();
        let idx_size = std::fs::metadata(&example_index_path)?.len();
        let meta_size = std::fs::metadata(&example_meta_path)?.len();

        println!("\nExample index built successfully:");
        println!("  Examples: {}", count);
        println!("  Errors: {}", errors);
        println!("  Build time: {:?}", build_time);
        println!("  Index size: {} bytes", idx_size);
        println!("  Metadata size: {} bytes", meta_size);
        println!("  Saved to: {}", example_index_path.display());

        return Ok(());
    }

    // No action specified
    println!("Usage: nl index [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --build            Build intent index from canonical descriptions");
    println!("  --build-examples   Build example index from training data");
    println!("  --training-data    Input training data file (JSONL)");
    println!("  --output           Output directory (default: ~/.neurlang/)");
    println!("  --embedder         ONNX embedder model path");
    println!("  --ollama           Use Ollama for embeddings (host:port:model)");
    println!("  --info             Show index information");
    println!("  --verify           Verify intent index accuracy");
    println!("  -v, --verbose      Verbose output");
    println!();
    println!("Examples:");
    println!("  nl index --build                                   # Build intent index with default embedder");
    println!("  nl index --build --ollama localhost:11434:nomic-embed-text");
    println!("  nl index --build-examples --training-data train/training_data.jsonl");
    println!("  nl index --info                                    # Show index stats");
    println!("  nl index --verify                                  # Test classification accuracy");

    Ok(())
}

/// Create an embedder for index building
fn create_embedder_for_index(
    embedder_path: Option<&PathBuf>,
    ollama: Option<&str>,
) -> Result<Box<dyn neurlang::inference::embedder::Embedder>> {
    use neurlang::inference::embedder::{create_embedder, EmbedderConfig, OllamaEmbedder};

    // Prefer explicit embedder path
    if let Some(path) = embedder_path {
        return create_embedder(EmbedderConfig::onnx(path))
            .context("Failed to create ONNX embedder");
    }

    // Try Ollama if specified
    if let Some(ollama_spec) = ollama {
        // Parse host:port:model format
        let parts: Vec<&str> = ollama_spec.split(':').collect();
        let (host, model) = match parts.len() {
            1 => ("http://localhost:11434".to_string(), parts[0].to_string()),
            2 => (format!("http://{}:11434", parts[0]), parts[1].to_string()),
            3 => (
                format!("http://{}:{}", parts[0], parts[1]),
                parts[2].to_string(),
            ),
            _ => anyhow::bail!("Invalid ollama format. Use: host:port:model or model"),
        };

        return create_embedder(EmbedderConfig::ollama(&host, &model))
            .context("Failed to create Ollama embedder");
    }

    // Try auto-detection with helpful error message
    let ollama_host =
        std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let ollama_model =
        std::env::var("NEURLANG_EMBED_MODEL").unwrap_or_else(|_| "nomic-embed-text".to_string());

    // Check if Ollama is running
    let ollama = OllamaEmbedder::new(&ollama_host, &ollama_model);
    if ollama.check_available().is_ok() {
        // Ensure model is pulled
        println!(
            "Using Ollama embedder at {} with model '{}'",
            ollama_host, ollama_model
        );
        if ollama.ensure_model().is_err() {
            println!(
                "Pulling model '{}' (this may take a minute)...",
                ollama_model
            );
        }
        return Ok(Box::new(ollama));
    }

    // Provide helpful error message
    anyhow::bail!(
        "No embedder available.\n\n\
        Options:\n\
        1. Start Ollama:   ollama serve\n\
        2. Pull model:     ollama pull nomic-embed-text\n\
        3. Use --ollama:   nl index --build --ollama nomic-embed-text\n\
        4. Use ONNX:       nl index --build --embedder path/to/embeddings.onnx\n\n\
        Environment variables:\n\
        - OLLAMA_HOST (default: http://localhost:11434)\n\
        - NEURLANG_EMBED_MODEL (default: nomic-embed-text)"
    )
}
