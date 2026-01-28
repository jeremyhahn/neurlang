//! Test Runner for Integration Tests
//!
//! Compiles and executes example programs, verifying correctness.

use super::test_cases::{ExtensionTestCase, TestCase};
use neurlang::ExtensionMock;
use std::path::PathBuf;

/// Result of running a test
#[derive(Debug)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub expected: Option<u64>,
    pub actual: Option<u64>,
    pub error: Option<String>,
    pub compile_time_us: u64,
    pub run_time_us: u64,
}

/// Test runner that compiles and executes programs
pub struct TestRunner {
    #[allow(dead_code)]
    project_root: PathBuf,
    examples_dir: PathBuf,
    #[allow(dead_code)]
    temp_dir: PathBuf,
}

impl TestRunner {
    /// Create a new test runner
    pub fn new() -> Self {
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let examples_dir = project_root.join("examples");
        let temp_dir = std::env::temp_dir().join("neurlang_integration_tests");

        // Create temp directory
        std::fs::create_dir_all(&temp_dir).ok();

        Self {
            project_root,
            examples_dir,
            temp_dir,
        }
    }

    /// Get path to example source file
    pub fn example_path(&self, source: &str) -> PathBuf {
        self.examples_dir.join(source)
    }

    /// Get path to compiled binary
    #[allow(dead_code)]
    pub fn binary_path(&self, name: &str) -> PathBuf {
        self.temp_dir.join(format!("{}.bin", name))
    }

    /// Run a single test case using the library directly
    pub fn run_test(&self, test: &TestCase) -> TestResult {
        use std::time::Instant;

        let source_path = self.example_path(test.source);

        // Check source exists
        if !source_path.exists() {
            return TestResult {
                name: test.name.to_string(),
                passed: false,
                expected: test.expected_r0,
                actual: None,
                error: Some(format!("Source file not found: {}", source_path.display())),
                compile_time_us: 0,
                run_time_us: 0,
            };
        }

        // Read and assemble the source
        let source = match std::fs::read_to_string(&source_path) {
            Ok(s) => s,
            Err(e) => {
                return TestResult {
                    name: test.name.to_string(),
                    passed: false,
                    expected: test.expected_r0,
                    actual: None,
                    error: Some(format!("Failed to read source: {}", e)),
                    compile_time_us: 0,
                    run_time_us: 0,
                };
            }
        };

        // Assemble
        let compile_start = Instant::now();
        let program = match neurlang::ir::Assembler::new().assemble(&source) {
            Ok(p) => p,
            Err(e) => {
                return TestResult {
                    name: test.name.to_string(),
                    passed: false,
                    expected: test.expected_r0,
                    actual: None,
                    error: Some(format!("Assembly failed: {:?}", e)),
                    compile_time_us: compile_start.elapsed().as_micros() as u64,
                    run_time_us: 0,
                };
            }
        };
        let compile_time = compile_start.elapsed();

        // Execute
        let run_start = Instant::now();
        let mut registers = [0u64; 32];

        // Set input registers from test case
        for &(reg, value) in test.input_regs {
            if reg < 32 {
                registers[reg] = value;
            }
        }

        let result = neurlang::execute(&program, &mut registers);
        let run_time = run_start.elapsed();

        match result {
            Ok(_) => {
                let actual = registers[0];
                let passed = self.check_result(test, actual, &registers);

                TestResult {
                    name: test.name.to_string(),
                    passed,
                    expected: test.expected_r0,
                    actual: Some(actual),
                    error: if passed {
                        None
                    } else {
                        Some("Value mismatch".to_string())
                    },
                    compile_time_us: compile_time.as_micros() as u64,
                    run_time_us: run_time.as_micros() as u64,
                }
            }
            Err(e) => TestResult {
                name: test.name.to_string(),
                passed: false,
                expected: test.expected_r0,
                actual: None,
                error: Some(format!("Execution failed: {}", e)),
                compile_time_us: compile_time.as_micros() as u64,
                run_time_us: run_time.as_micros() as u64,
            },
        }
    }

    /// Check if result matches expected
    fn check_result(&self, test: &TestCase, actual_r0: u64, registers: &[u64; 32]) -> bool {
        // Skip value check for non-deterministic tests
        if test.time_sensitive || test.uses_random {
            return true; // Just verify it ran without error
        }

        // Check R0 if expected value is set
        if let Some(expected) = test.expected_r0 {
            if test.is_float {
                let expected_f = f64::from_bits(expected);
                let actual_f = f64::from_bits(actual_r0);
                if (expected_f - actual_f).abs() > test.float_tolerance {
                    return false;
                }
            } else if actual_r0 != expected {
                return false;
            }
        }

        // Check additional registers
        for &(reg, expected) in test.expected_regs {
            if registers[reg] != expected {
                return false;
            }
        }

        true
    }

    /// Run all provided test cases
    pub fn run_tests(&self, tests: &[&TestCase]) -> Vec<TestResult> {
        tests.iter().map(|t| self.run_test(t)).collect()
    }

    /// Run a test with mock extensions
    #[allow(dead_code)]
    pub fn run_test_with_mocks(&self, test: &TestCase, mocks: &[ExtensionMock]) -> TestResult {
        use std::time::Instant;

        let source_path = self.example_path(test.source);

        // Check source exists
        if !source_path.exists() {
            return TestResult {
                name: test.name.to_string(),
                passed: false,
                expected: test.expected_r0,
                actual: None,
                error: Some(format!("Source file not found: {}", source_path.display())),
                compile_time_us: 0,
                run_time_us: 0,
            };
        }

        // Read and assemble the source
        let source = match std::fs::read_to_string(&source_path) {
            Ok(s) => s,
            Err(e) => {
                return TestResult {
                    name: test.name.to_string(),
                    passed: false,
                    expected: test.expected_r0,
                    actual: None,
                    error: Some(format!("Failed to read source: {}", e)),
                    compile_time_us: 0,
                    run_time_us: 0,
                };
            }
        };

        // Assemble
        let compile_start = Instant::now();
        let program = match neurlang::ir::Assembler::new().assemble(&source) {
            Ok(p) => p,
            Err(e) => {
                return TestResult {
                    name: test.name.to_string(),
                    passed: false,
                    expected: test.expected_r0,
                    actual: None,
                    error: Some(format!("Assembly failed: {:?}", e)),
                    compile_time_us: compile_start.elapsed().as_micros() as u64,
                    run_time_us: 0,
                };
            }
        };
        let compile_time = compile_start.elapsed();

        // Execute with mocks
        let run_start = Instant::now();
        let mut registers = [0u64; 32];

        // Set input registers from test case
        for &(reg, value) in test.input_regs {
            if reg < 32 {
                registers[reg] = value;
            }
        }

        let result = neurlang::execute_with_mocks(&program, &mut registers, mocks);
        let run_time = run_start.elapsed();

        match result {
            neurlang::InterpResult::Ok(_) | neurlang::InterpResult::Halted => {
                let actual = registers[0];
                let passed = self.check_result(test, actual, &registers);

                TestResult {
                    name: test.name.to_string(),
                    passed,
                    expected: test.expected_r0,
                    actual: Some(actual),
                    error: if passed {
                        None
                    } else {
                        Some("Value mismatch".to_string())
                    },
                    compile_time_us: compile_time.as_micros() as u64,
                    run_time_us: run_time.as_micros() as u64,
                }
            }
            _ => TestResult {
                name: test.name.to_string(),
                passed: false,
                expected: test.expected_r0,
                actual: None,
                error: Some(format!("Execution failed: {:?}", result)),
                compile_time_us: compile_time.as_micros() as u64,
                run_time_us: run_time.as_micros() as u64,
            },
        }
    }

    /// Run an extension test case with mocks
    pub fn run_extension_test(&self, test: &ExtensionTestCase) -> TestResult {
        use std::time::Instant;

        let source_path = self.example_path(test.source);

        // Check source exists
        if !source_path.exists() {
            return TestResult {
                name: test.name.to_string(),
                passed: false,
                expected: test.expected_r0,
                actual: None,
                error: Some(format!("Source file not found: {}", source_path.display())),
                compile_time_us: 0,
                run_time_us: 0,
            };
        }

        // Read and assemble the source
        let source = match std::fs::read_to_string(&source_path) {
            Ok(s) => s,
            Err(e) => {
                return TestResult {
                    name: test.name.to_string(),
                    passed: false,
                    expected: test.expected_r0,
                    actual: None,
                    error: Some(format!("Failed to read source: {}", e)),
                    compile_time_us: 0,
                    run_time_us: 0,
                };
            }
        };

        // Assemble
        let compile_start = Instant::now();
        let program = match neurlang::ir::Assembler::new().assemble(&source) {
            Ok(p) => p,
            Err(e) => {
                return TestResult {
                    name: test.name.to_string(),
                    passed: false,
                    expected: test.expected_r0,
                    actual: None,
                    error: Some(format!("Assembly failed: {:?}", e)),
                    compile_time_us: compile_start.elapsed().as_micros() as u64,
                    run_time_us: 0,
                };
            }
        };
        let compile_time = compile_start.elapsed();

        // Convert MockSpec to ExtensionMock (using ID-based mocking)
        let mocks: Vec<ExtensionMock> = test
            .mocks
            .iter()
            .map(|m| ExtensionMock {
                name: String::new(),
                id: m.id,
                return_value: m.return_value,
                outputs: m.outputs.to_vec(),
            })
            .collect();

        // Execute with mocks
        let run_start = Instant::now();
        let mut registers = [0u64; 32];

        let result = neurlang::execute_with_mocks(&program, &mut registers, &mocks);
        let run_time = run_start.elapsed();

        match result {
            neurlang::InterpResult::Ok(_) | neurlang::InterpResult::Halted => {
                let actual = registers[0];
                let passed = if let Some(expected) = test.expected_r0 {
                    actual == expected
                } else {
                    true // No expected value, just verify it ran
                };

                TestResult {
                    name: test.name.to_string(),
                    passed,
                    expected: test.expected_r0,
                    actual: Some(actual),
                    error: if passed {
                        None
                    } else {
                        Some("Value mismatch".to_string())
                    },
                    compile_time_us: compile_time.as_micros() as u64,
                    run_time_us: run_time.as_micros() as u64,
                }
            }
            _ => TestResult {
                name: test.name.to_string(),
                passed: false,
                expected: test.expected_r0,
                actual: None,
                error: Some(format!("Execution failed: {:?}", result)),
                compile_time_us: compile_time.as_micros() as u64,
                run_time_us: run_time.as_micros() as u64,
            },
        }
    }

    /// Run all extension test cases
    pub fn run_extension_tests(&self, tests: &[ExtensionTestCase]) -> Vec<TestResult> {
        tests.iter().map(|t| self.run_extension_test(t)).collect()
    }

    /// Print test results summary
    pub fn print_summary(results: &[TestResult]) {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;

        println!("\n{}", "=".repeat(60));
        println!(
            "Test Results: {} passed, {} failed, {} total",
            passed, failed, total
        );
        println!("{}", "=".repeat(60));

        // Print failed tests first
        for result in results.iter().filter(|r| !r.passed) {
            println!("\n❌ FAILED: {}", result.name);
            if let Some(expected) = result.expected {
                println!("   Expected: {}", expected);
            }
            if let Some(actual) = result.actual {
                println!("   Actual:   {}", actual);
            }
            if let Some(ref error) = result.error {
                println!("   Error:    {}", error);
            }
        }

        // Print passed tests summary
        if passed > 0 {
            println!("\n✓ Passed tests:");
            for result in results.iter().filter(|r| r.passed) {
                println!(
                    "  {} (compile: {}μs, run: {}μs)",
                    result.name, result.compile_time_us, result.run_time_us
                );
            }
        }

        println!();
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}
