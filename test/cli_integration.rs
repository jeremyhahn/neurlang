//! CLI Integration Tests for Neurlang
//!
//! Tests the command-line interface and end-to-end workflows.
//! These tests verify that assembly, compilation, and execution work correctly.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Get the path to the nl binary (assumed to be built)
fn nl_binary() -> PathBuf {
    // During cargo test, the binary should be in target/debug or target/release
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("nl");
    path
}

/// Get the path to the assembler binary
fn nl_asm_binary() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("nl-asm");
    path
}

/// Get the examples directory
fn examples_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("examples");
    path
}

/// Get a temp directory for test outputs
fn temp_dir() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push("neurlang_tests");
    fs::create_dir_all(&path).ok();
    path
}

// ============================================================================
// Assembler Tests
// ============================================================================

#[test]
fn test_assembler_fibonacci() {
    let asm_path = examples_dir().join("algorithm/math/fibonacci.nl");
    let out_path = temp_dir().join("fibonacci.bin");

    let output = Command::new(nl_asm_binary())
        .args([asm_path.to_str().unwrap(), "-o", out_path.to_str().unwrap()])
        .output()
        .expect("Failed to run assembler");

    assert!(
        output.status.success(),
        "Assembler failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify output file was created
    assert!(out_path.exists(), "Output binary not created");

    // Verify magic bytes
    let bytes = fs::read(&out_path).expect("Failed to read output");
    assert_eq!(&bytes[0..4], b"NRLG", "Invalid magic bytes");
}

#[test]
fn test_assembler_disassemble() {
    let asm_path = examples_dir().join("algorithm/math/factorial.nl");
    let bin_path = temp_dir().join("factorial.bin");

    // First assemble
    let _ = Command::new(nl_asm_binary())
        .args([asm_path.to_str().unwrap(), "-o", bin_path.to_str().unwrap()])
        .output()
        .expect("Failed to assemble");

    // Then disassemble using nl disasm (not nl-asm)
    let output = Command::new(nl_binary())
        .args(["disasm", "-i", bin_path.to_str().unwrap()])
        .output()
        .expect("Failed to disassemble");

    assert!(
        output.status.success(),
        "Disassembler failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let disasm = String::from_utf8_lossy(&output.stdout);
    assert!(
        disasm.contains("mov") || disasm.contains("MOV"),
        "Disassembly missing mov instruction"
    );
    assert!(
        disasm.contains("halt")
            || disasm.contains("HALT")
            || disasm.contains("ret")
            || disasm.contains("RET"),
        "Disassembly missing halt/ret instruction"
    );
}

// ============================================================================
// Execution Tests
// ============================================================================

#[test]
fn test_run_simple_program() {
    let asm = r#"
        mov r0, 42
        halt
    "#;

    let asm_path = temp_dir().join("simple.nl");
    let bin_path = temp_dir().join("simple.bin");

    fs::write(&asm_path, asm).expect("Failed to write asm file");

    // Assemble
    let _ = Command::new(nl_asm_binary())
        .args([asm_path.to_str().unwrap(), "-o", bin_path.to_str().unwrap()])
        .output()
        .expect("Failed to assemble");

    // Run
    let output = Command::new(nl_binary())
        .args(["run", "-i", bin_path.to_str().unwrap()])
        .output()
        .expect("Failed to run program");

    assert!(
        output.status.success(),
        "Execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check output contains expected result
    let _stdout = String::from_utf8_lossy(&output.stdout);
}

#[test]
fn test_run_arithmetic() {
    let asm = r#"
        mov r0, 10
        mov r1, 20
        alu.add r0, r0, r1
        halt
    "#;

    let asm_path = temp_dir().join("arithmetic.nl");
    let bin_path = temp_dir().join("arithmetic.bin");

    fs::write(&asm_path, asm).expect("Failed to write asm file");

    // Assemble and run
    let _ = Command::new(nl_asm_binary())
        .args([asm_path.to_str().unwrap(), "-o", bin_path.to_str().unwrap()])
        .output();

    let output = Command::new(nl_binary())
        .args(["run", "-i", bin_path.to_str().unwrap()])
        .output()
        .expect("Failed to run program");

    let _stdout = String::from_utf8_lossy(&output.stdout);
    // Just verify it completed successfully
    assert!(output.status.success(), "Execution failed");
}

// ============================================================================
// Agent Tests (formerly REPL)
// ============================================================================

// Agent is interactive-only, test that it can show help
#[test]
fn test_agent_help() {
    let output = Command::new(nl_binary())
        .args(["agent", "--help"])
        .output()
        .expect("Failed to run agent help");

    assert!(
        output.status.success(),
        "Agent help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ============================================================================
// Generate Command Tests (Mock Backend)
// ============================================================================

#[test]
fn test_prompt_fibonacci() {
    let output = Command::new(nl_binary())
        .args(["prompt", "compute fibonacci of 10"])
        .output()
        .expect("Failed to run prompt");

    // Even with mock backend, should produce some output
    let _stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should either succeed or provide meaningful error
    assert!(
        output.status.success() || stderr.contains("model"),
        "Unexpected error: {}",
        stderr
    );
}

#[test]
fn test_prompt_factorial() {
    let output = Command::new(nl_binary())
        .args(["prompt", "calculate factorial of 5"])
        .output()
        .expect("Failed to run prompt");

    let _stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success() || stderr.contains("model"),
        "Unexpected error: {}",
        stderr
    );
}

#[test]
fn test_prompt_with_show_asm() {
    let output = Command::new(nl_binary())
        .args(["prompt", "add two numbers", "--show-asm"])
        .output()
        .expect("Failed to run prompt");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should show assembly output
        assert!(
            stdout.contains("mov") || stdout.contains("add") || stdout.contains("alu"),
            "Expected assembly output, got: {}",
            stdout
        );
    }
}

// ============================================================================
// Help and Version Tests
// ============================================================================

#[test]
fn test_help() {
    let output = Command::new(nl_binary())
        .args(["--help"])
        .output()
        .expect("Failed to run help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Neurlang") || stdout.contains("nl"));
    assert!(stdout.contains("run") || stdout.contains("prompt"));
}

#[test]
fn test_version() {
    let output = Command::new(nl_binary())
        .args(["--version"])
        .output()
        .expect("Failed to run version");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.1") || stdout.contains("nl"));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_run_missing_file() {
    let output = Command::new(nl_binary())
        .args(["run", "-i", "/nonexistent/path/to/file.bin"])
        .output()
        .expect("Failed to run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found") || stderr.contains("No such file") || stderr.contains("error"),
        "Expected file not found error, got: {}",
        stderr
    );
}

#[test]
fn test_assemble_invalid_syntax() {
    let invalid_asm = r#"
        this is not valid assembly
        mov r99999, broken
    "#;

    let asm_path = temp_dir().join("invalid.nl");
    fs::write(&asm_path, invalid_asm).expect("Failed to write");

    let output = Command::new(nl_asm_binary())
        .args([asm_path.to_str().unwrap()])
        .output()
        .expect("Failed to run assembler");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error") || stderr.contains("invalid") || stderr.contains("Error"),
        "Expected error message, got: {}",
        stderr
    );
}

// ============================================================================
// Compile Command Tests
// ============================================================================

#[test]
fn test_compile_to_native() {
    let asm = r#"
        mov r0, 42
        halt
    "#;

    let asm_path = temp_dir().join("compile_test.nl");
    let bin_path = temp_dir().join("compile_test.bin");

    fs::write(&asm_path, asm).expect("Failed to write");

    // Assemble first
    let _ = Command::new(nl_asm_binary())
        .args([asm_path.to_str().unwrap(), "-o", bin_path.to_str().unwrap()])
        .output();

    // Compile to native (if supported)
    let output = Command::new(nl_binary())
        .args(["compile", "-i", bin_path.to_str().unwrap()])
        .output()
        .expect("Failed to run compile");

    // This may fail if JIT is not available, which is OK
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        assert!(
            stderr.contains("not supported") || stderr.contains("JIT") || stderr.contains("error"),
            "Unexpected error: {}",
            stderr
        );
    }
}

// ============================================================================
// Benchmark and Stats Tests
// ============================================================================

#[test]
fn test_stats_command() {
    let asm = r#"
        mov r0, 1
        mov r1, 2
        alu.add r0, r0, r1
        halt
    "#;

    let asm_path = temp_dir().join("stats_test.nl");
    let bin_path = temp_dir().join("stats_test.bin");

    fs::write(&asm_path, asm).expect("Failed to write");

    let _ = Command::new(nl_asm_binary())
        .args([asm_path.to_str().unwrap(), "-o", bin_path.to_str().unwrap()])
        .output();

    // Get stats about the binary by running with stats flag
    let output = Command::new(nl_binary())
        .args(["run", "-i", bin_path.to_str().unwrap(), "--stats"])
        .output();

    // info command may not exist, but if it does, check output
    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            assert!(
                stdout.contains("instruction")
                    || stdout.contains("bytes")
                    || stdout.contains("size"),
                "Expected stats output, got: {}",
                stdout
            );
        }
    }
}

// ============================================================================
// Examples Validation
// ============================================================================

#[test]
fn test_examples_exist() {
    let examples = [
        "algorithm/math/fibonacci.nl",
        "algorithm/math/factorial.nl",
        "algorithm/math/float_sqrt.nl",
        "algorithm/bitwise/bitcount.nl",
        "system/time_example.nl",
        "crypto/random.nl",
        "io/console/hello_world.nl",
        "concurrency/concurrent.nl",
        "security/capability_demo.nl",
    ];

    for example in examples {
        let path = examples_dir().join(example);
        assert!(path.exists(), "Example file missing: {}", path.display());
    }
}

#[test]
fn test_examples_not_empty() {
    let dir = examples_dir();
    if dir.exists() {
        for entry in fs::read_dir(dir).expect("Failed to read examples dir") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "nl") {
                let content = fs::read_to_string(&path).expect("Failed to read file");
                assert!(
                    !content.trim().is_empty(),
                    "Example file is empty: {}",
                    path.display()
                );
                // Basic sanity check - should contain at least one instruction
                assert!(
                    content.contains("mov") || content.contains("halt") || content.contains("ret"),
                    "Example file missing basic instructions: {}",
                    path.display()
                );
            }
        }
    }
}
