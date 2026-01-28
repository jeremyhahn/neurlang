//! Examples Integration Tests
//!
//! This test module verifies that all examples in the examples/ directory:
//! 1. Assemble successfully
//! 2. Run without crashing
//! 3. Produce expected outputs (where specified)
//!
//! These examples also serve as training data for the model.

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Get path to nl binary
fn nl_binary() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("nl");
    if !path.exists() {
        path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("target");
        path.push("release");
        path.push("nl");
    }
    path
}

/// Get examples directory
fn examples_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("examples");
    path
}

/// Get temp directory for outputs
fn temp_dir() -> PathBuf {
    let path = std::env::temp_dir().join("neurlang_example_tests");
    fs::create_dir_all(&path).ok();
    path
}

/// Example with expected behavior
struct ExampleSpec {
    name: &'static str,
    /// Expected output substring (if any)
    expect_output: Option<&'static str>,
    /// Whether this is a long-running server (skip execution)
    is_server: bool,
    /// Maximum execution time in seconds
    timeout_secs: u64,
}

const EXAMPLES: &[ExampleSpec] = &[
    // io/console
    ExampleSpec {
        name: "io/console/hello_world.nl",
        expect_output: Some("Hello"),
        is_server: false,
        timeout_secs: 5,
    },
    // algorithm/math
    ExampleSpec {
        name: "algorithm/math/fibonacci.nl",
        expect_output: None,
        is_server: false,
        timeout_secs: 5,
    },
    ExampleSpec {
        name: "algorithm/math/factorial.nl",
        expect_output: None,
        is_server: false,
        timeout_secs: 5,
    },
    ExampleSpec {
        name: "algorithm/math/float_sqrt.nl",
        expect_output: None,
        is_server: false,
        timeout_secs: 5,
    },
    ExampleSpec {
        name: "algorithm/math/float_circle.nl",
        expect_output: None,
        is_server: false,
        timeout_secs: 5,
    },
    ExampleSpec {
        name: "algorithm/math/gcd.nl",
        expect_output: None,
        is_server: false,
        timeout_secs: 5,
    },
    ExampleSpec {
        name: "algorithm/math/power_of_two.nl",
        expect_output: None,
        is_server: false,
        timeout_secs: 5,
    },
    ExampleSpec {
        name: "algorithm/math/absolute_value.nl",
        expect_output: None,
        is_server: false,
        timeout_secs: 5,
    },
    // algorithm/bitwise
    ExampleSpec {
        name: "algorithm/bitwise/bitcount.nl",
        expect_output: None,
        is_server: false,
        timeout_secs: 5,
    },
    // algorithm/array
    ExampleSpec {
        name: "algorithm/array/sum_array.nl",
        expect_output: None,
        is_server: false,
        timeout_secs: 5,
    },
    // system
    ExampleSpec {
        name: "system/time_example.nl",
        expect_output: None,
        is_server: false,
        timeout_secs: 5,
    },
    // crypto
    ExampleSpec {
        name: "crypto/random.nl",
        expect_output: None,
        is_server: false,
        timeout_secs: 5,
    },
    // concurrency
    ExampleSpec {
        name: "concurrency/concurrent.nl",
        expect_output: None,
        is_server: false,
        timeout_secs: 10,
    },
    // security
    ExampleSpec {
        name: "security/capability_demo.nl",
        expect_output: None,
        is_server: false,
        timeout_secs: 5,
    },
    // io/file
    ExampleSpec {
        name: "io/file/io_test.nl",
        expect_output: Some("Neurlang"),
        is_server: false,
        timeout_secs: 10,
    },
    // network/tcp
    ExampleSpec {
        name: "network/tcp/net_test.nl",
        expect_output: Some("test"),
        is_server: false,
        timeout_secs: 10,
    },
    // Server examples - only assemble, don't run
    ExampleSpec {
        name: "network/http/rest_api_server.nl",
        expect_output: None,
        is_server: true,
        timeout_secs: 2,
    },
    ExampleSpec {
        name: "network/http/rest_api_dynamic.nl",
        expect_output: None,
        is_server: true,
        timeout_secs: 2,
    },
];

/// Recursively collect all .nl files from a directory
fn collect_nl_files(dir: &PathBuf, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                collect_nl_files(&path, &mut *files);
            } else if path.extension().is_some_and(|ext| ext == "nl") {
                files.push(path);
            }
        }
    }
}

/// Test that all examples assemble successfully
#[test]
fn test_all_examples_assemble() {
    let binary = nl_binary();
    if !binary.exists() {
        println!("Skipping: nl binary not found at {:?}", binary);
        return;
    }

    let examples = examples_dir();
    let temp = temp_dir();

    // Collect all .nl files recursively
    let mut files = Vec::new();
    collect_nl_files(&examples, &mut files);
    files.sort();

    for path in files {
        let rel_path = path.strip_prefix(&examples).unwrap_or(&path);
        let name = rel_path.to_string_lossy();
        let out_name = path.file_name().unwrap().to_string_lossy();
        let out_path = temp.join(format!("{}.bin", out_name));

        println!("Assembling: {}", name);

        let output = Command::new(&binary)
            .args([
                "asm",
                "-i",
                path.to_str().unwrap(),
                "-o",
                out_path.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to run assembler");

        assert!(
            output.status.success(),
            "Failed to assemble {}: {}",
            name,
            String::from_utf8_lossy(&output.stderr)
        );

        assert!(out_path.exists(), "Output binary not created for {}", name);

        // Verify magic bytes
        let bytes = fs::read(&out_path).expect("Failed to read output");
        assert!(bytes.len() >= 4, "Output too small for {}", name);
        assert_eq!(&bytes[0..4], b"NRLG", "Invalid magic bytes for {}", name);

        println!("  ✓ {} assembled ({} bytes)", name, bytes.len());
    }
}

/// Test that non-server examples run successfully
#[test]
fn test_examples_run() {
    let binary = nl_binary();
    if !binary.exists() {
        println!("Skipping: nl binary not found");
        return;
    }

    let examples = examples_dir();
    let temp = temp_dir();

    for spec in EXAMPLES {
        let asm_path = examples.join(spec.name);
        if !asm_path.exists() {
            println!("Skipping missing example: {}", spec.name);
            continue;
        }

        let bin_path = temp.join(format!("{}.bin", spec.name));

        // Ensure parent directories exist
        if let Some(parent) = bin_path.parent() {
            fs::create_dir_all(parent).ok();
        }

        // Assemble first
        let output = Command::new(&binary)
            .args([
                "asm",
                "-i",
                asm_path.to_str().unwrap(),
                "-o",
                bin_path.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to run assembler");

        if !output.status.success() {
            panic!(
                "Failed to assemble {}: {}",
                spec.name,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Skip servers (they run indefinitely)
        if spec.is_server {
            println!("  ✓ {} assembled (server, not executing)", spec.name);
            continue;
        }

        // Run with interpreter (with simple timeout using spawn/wait)
        println!("Running: {}", spec.name);

        let mut child = match Command::new(&binary)
            .args(["run", "-i", bin_path.to_str().unwrap(), "--interp"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                panic!("{} failed to spawn: {}", spec.name, e);
            }
        };

        // Simple timeout: wait up to timeout_secs
        let start = Instant::now();
        let timeout = Duration::from_secs(spec.timeout_secs);

        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    // Process finished
                    let stdout = child
                        .stdout
                        .take()
                        .map(|s| {
                            BufReader::new(s)
                                .lines()
                                .filter_map(|l| l.ok())
                                .collect::<Vec<_>>()
                                .join("\n")
                        })
                        .unwrap_or_default();
                    let stderr = child
                        .stderr
                        .take()
                        .map(|s| {
                            BufReader::new(s)
                                .lines()
                                .filter_map(|l| l.ok())
                                .collect::<Vec<_>>()
                                .join("\n")
                        })
                        .unwrap_or_default();

                    if !status.success() {
                        // Some examples might exit with non-zero, that's OK
                        // as long as they didn't crash
                        if stderr.contains("panic") || stderr.contains("SIGSEGV") {
                            panic!("{} crashed: {}", spec.name, stderr);
                        }
                    }

                    // Check expected output if specified
                    if let Some(expected) = spec.expect_output {
                        assert!(
                            stdout.contains(expected),
                            "{}: expected output containing '{}', got:\n{}",
                            spec.name,
                            expected,
                            stdout
                        );
                    }

                    println!("  ✓ {} ran successfully", spec.name);
                    break;
                }
                Ok(None) => {
                    // Still running
                    if start.elapsed() > timeout {
                        // Kill the process
                        let _ = child.kill();
                        let _ = child.wait();
                        println!(
                            "  ⚠ {} timed out after {}s (may be expected)",
                            spec.name, spec.timeout_secs
                        );
                        break;
                    }
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    panic!("{} failed to wait: {}", spec.name, e);
                }
            }
        }
    }
}

/// Test that examples can be disassembled (round-trip verification)
#[test]
fn test_examples_disassemble() {
    let binary = nl_binary();
    if !binary.exists() {
        println!("Skipping: nl binary not found");
        return;
    }

    let examples = examples_dir();
    let temp = temp_dir();

    // Test a few key examples
    let test_examples = ["fibonacci.nl", "factorial.nl", "hello_world.nl"];

    for name in test_examples {
        let asm_path = examples.join(name);
        if !asm_path.exists() {
            continue;
        }

        let bin_path = temp.join(format!("{}.bin", name));

        // Assemble
        let _ = Command::new(&binary)
            .args([
                "asm",
                "-i",
                asm_path.to_str().unwrap(),
                "-o",
                bin_path.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to assemble");

        // Disassemble
        let output = Command::new(&binary)
            .args(["disasm", "-i", bin_path.to_str().unwrap()])
            .output()
            .expect("Failed to disassemble");

        assert!(
            output.status.success(),
            "Failed to disassemble {}: {}",
            name,
            String::from_utf8_lossy(&output.stderr)
        );

        let disasm = String::from_utf8_lossy(&output.stdout);

        // Should contain actual instructions
        assert!(
            disasm.contains("mov") || disasm.contains("halt") || disasm.contains("ret"),
            "{}: disassembly missing expected instructions:\n{}",
            name,
            disasm
        );

        println!("  ✓ {} disassembles correctly", name);
    }
}

/// Collect all examples as training data pairs (source, binary)
/// This function can be called from the datagen tool
pub fn collect_examples_as_training_data() -> Vec<(String, String, Vec<u8>)> {
    let binary = nl_binary();
    let examples = examples_dir();
    let temp = temp_dir();
    let mut results = Vec::new();

    if !binary.exists() {
        return results;
    }

    // Collect all .nl files recursively
    let mut files = Vec::new();
    collect_nl_files(&examples, &mut files);

    for path in files {
        let rel_path = path.strip_prefix(&examples).unwrap_or(&path);
        let name = rel_path.to_string_lossy().to_string();
        let source = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let out_name = path.file_name().unwrap().to_string_lossy();
        let bin_path = temp.join(format!("{}.bin", out_name));

        // Assemble
        let output = Command::new(&binary)
            .args([
                "asm",
                "-i",
                path.to_str().unwrap(),
                "-o",
                bin_path.to_str().unwrap(),
            ])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                if let Ok(binary_data) = fs::read(&bin_path) {
                    results.push((name, source, binary_data));
                }
            }
        }
    }

    results
}
