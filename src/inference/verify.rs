//! Test Verification for Neurlang
//!
//! Provides comprehensive verification of generated programs against test cases.
//!
//! # Features
//!
//! - Execute programs with input/output verification
//! - Error formatting for feedback loop
//! - Performance measurement
//! - Multiple verification strategies
//!
//! # Usage
//!
//! ```rust,ignore
//! use neurlang::inference::verify::{Verifier, TestSuite, TestCase};
//!
//! let verifier = Verifier::new();
//! let suite = TestSuite::new()
//!     .add_case(TestCase::new(vec![5], 120))  // 5! = 120
//!     .add_case(TestCase::new(vec![0], 1));   // 0! = 1
//!
//! let result = verifier.verify(&program, &suite)?;
//! ```

use std::time::{Duration, Instant};

use crate::execute;
use crate::ir::Program;

/// A single test case
#[derive(Debug, Clone)]
pub struct TestCase {
    /// Input values (placed in registers r0, r1, ...)
    pub inputs: Vec<i64>,
    /// Expected output (from r0)
    pub expected: i64,
    /// Optional description
    pub description: Option<String>,
}

impl TestCase {
    /// Create a new test case
    pub fn new(inputs: Vec<i64>, expected: i64) -> Self {
        Self {
            inputs,
            expected,
            description: None,
        }
    }

    /// Create a test case with a description
    pub fn with_description(inputs: Vec<i64>, expected: i64, desc: impl Into<String>) -> Self {
        Self {
            inputs,
            expected,
            description: Some(desc.into()),
        }
    }
}

/// A collection of test cases
#[derive(Debug, Clone, Default)]
pub struct TestSuite {
    /// Test cases
    pub cases: Vec<TestCase>,
    /// Suite name
    pub name: Option<String>,
}

impl TestSuite {
    /// Create a new empty test suite
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a named test suite
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            cases: Vec::new(),
            name: Some(name.into()),
        }
    }

    /// Add a test case
    pub fn add_case(mut self, case: TestCase) -> Self {
        self.cases.push(case);
        self
    }

    /// Add a simple test case
    pub fn add(mut self, inputs: Vec<i64>, expected: i64) -> Self {
        self.cases.push(TestCase::new(inputs, expected));
        self
    }

    /// Add multiple test cases from tuples
    pub fn add_cases(mut self, cases: impl IntoIterator<Item = (Vec<i64>, i64)>) -> Self {
        for (inputs, expected) in cases {
            self.cases.push(TestCase::new(inputs, expected));
        }
        self
    }

    /// Get number of test cases
    pub fn len(&self) -> usize {
        self.cases.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.cases.is_empty()
    }
}

/// Result of running a single test case
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Test case index
    pub index: usize,
    /// Whether the test passed
    pub passed: bool,
    /// Actual result (if execution succeeded)
    pub actual: Option<i64>,
    /// Expected result
    pub expected: i64,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Execution time
    pub duration: Duration,
}

impl TestResult {
    /// Create a passing test result
    pub fn pass(index: usize, actual: i64, expected: i64, duration: Duration) -> Self {
        Self {
            index,
            passed: true,
            actual: Some(actual),
            expected,
            error: None,
            duration,
        }
    }

    /// Create a failing test result (wrong output)
    pub fn fail(index: usize, actual: i64, expected: i64, duration: Duration) -> Self {
        Self {
            index,
            passed: false,
            actual: Some(actual),
            expected,
            error: Some(format!("Expected {}, got {}", expected, actual)),
            duration,
        }
    }

    /// Create a test result for execution error
    pub fn error(index: usize, expected: i64, error: String, duration: Duration) -> Self {
        Self {
            index,
            passed: false,
            actual: None,
            expected,
            error: Some(error),
            duration,
        }
    }
}

/// Result of running a full test suite
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Results for each test case
    pub results: Vec<TestResult>,
    /// Total execution time
    pub total_duration: Duration,
    /// Whether all tests passed
    pub all_passed: bool,
    /// Number of passed tests
    pub passed_count: usize,
    /// Number of failed tests
    pub failed_count: usize,
}

impl VerificationResult {
    /// Format a summary of the results
    pub fn summary(&self) -> String {
        if self.all_passed {
            format!(
                "All {} tests passed in {:?}",
                self.passed_count, self.total_duration
            )
        } else {
            format!(
                "{}/{} tests passed, {} failed in {:?}",
                self.passed_count,
                self.passed_count + self.failed_count,
                self.failed_count,
                self.total_duration
            )
        }
    }

    /// Format error feedback for the model
    pub fn format_errors(&self) -> String {
        let mut feedback = String::new();

        for result in &self.results {
            if !result.passed {
                if let Some(error) = &result.error {
                    feedback.push_str(&format!("Test {}: {}\n", result.index, error));
                }
            }
        }

        feedback
    }

    /// Get the first error (useful for model feedback)
    pub fn first_error(&self) -> Option<&str> {
        self.results
            .iter()
            .find(|r| !r.passed)
            .and_then(|r| r.error.as_deref())
    }
}

/// Error type for verification
#[derive(Debug)]
pub enum VerifyError {
    /// Execution failed
    ExecutionFailed(String),
    /// No test cases provided
    NoTestCases,
}

impl std::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifyError::ExecutionFailed(e) => write!(f, "Execution failed: {}", e),
            VerifyError::NoTestCases => write!(f, "No test cases provided"),
        }
    }
}

impl std::error::Error for VerifyError {}

/// Program verifier
pub struct Verifier {
    /// Pre-allocated register file
    registers: Box<[u64; 32]>,
    /// Maximum execution time per test
    timeout: Duration,
}

impl Default for Verifier {
    fn default() -> Self {
        Self::new()
    }
}

impl Verifier {
    /// Create a new verifier
    pub fn new() -> Self {
        Self {
            registers: Box::new([0u64; 32]),
            timeout: Duration::from_secs(1),
        }
    }

    /// Set timeout per test case
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Verify a program against a test suite
    pub fn verify(&mut self, program: &Program, suite: &TestSuite) -> VerificationResult {
        let start = Instant::now();
        let mut results = Vec::with_capacity(suite.len());
        let mut passed_count = 0;
        let mut failed_count = 0;

        for (i, test) in suite.cases.iter().enumerate() {
            let result = self.run_test(program, i, test);
            if result.passed {
                passed_count += 1;
            } else {
                failed_count += 1;
            }
            results.push(result);
        }

        VerificationResult {
            results,
            total_duration: start.elapsed(),
            all_passed: failed_count == 0,
            passed_count,
            failed_count,
        }
    }

    /// Run a single test case
    fn run_test(&mut self, program: &Program, index: usize, test: &TestCase) -> TestResult {
        let start = Instant::now();

        // Reset registers
        for r in self.registers.iter_mut() {
            *r = 0;
        }

        // Set input registers
        for (j, &input) in test.inputs.iter().enumerate() {
            if j < 16 {
                self.registers[j] = input as u64;
            }
        }

        // Execute
        match execute(program, &mut self.registers) {
            Ok(_) => {
                let actual = self.registers[0] as i64;
                let duration = start.elapsed();

                if actual == test.expected {
                    TestResult::pass(index, actual, test.expected, duration)
                } else {
                    TestResult::fail(index, actual, test.expected, duration)
                }
            }
            Err(e) => TestResult::error(index, test.expected, e.to_string(), start.elapsed()),
        }
    }

    /// Quick verification - stops at first failure
    pub fn verify_quick(&mut self, program: &Program, suite: &TestSuite) -> Result<(), String> {
        for (i, test) in suite.cases.iter().enumerate() {
            let result = self.run_test(program, i, test);
            if !result.passed {
                return Err(result.error.unwrap_or_else(|| "Test failed".to_string()));
            }
        }
        Ok(())
    }
}

/// Common test suites for basic operations
pub mod common {
    use super::TestSuite;

    /// Test suite for addition
    pub fn addition_suite() -> TestSuite {
        TestSuite::named("addition")
            .add(vec![0, 0], 0)
            .add(vec![1, 1], 2)
            .add(vec![5, 3], 8)
            .add(vec![100, 200], 300)
            .add(vec![-5, 3], -2)
            .add(vec![-5, -3], -8)
    }

    /// Test suite for multiplication
    pub fn multiplication_suite() -> TestSuite {
        TestSuite::named("multiplication")
            .add(vec![0, 5], 0)
            .add(vec![1, 5], 5)
            .add(vec![5, 3], 15)
            .add(vec![10, 10], 100)
            .add(vec![-5, 3], -15)
            .add(vec![-5, -3], 15)
    }

    /// Test suite for factorial
    pub fn factorial_suite() -> TestSuite {
        TestSuite::named("factorial")
            .add(vec![0], 1)
            .add(vec![1], 1)
            .add(vec![2], 2)
            .add(vec![3], 6)
            .add(vec![4], 24)
            .add(vec![5], 120)
            .add(vec![6], 720)
            .add(vec![10], 3628800)
    }

    /// Test suite for fibonacci
    pub fn fibonacci_suite() -> TestSuite {
        TestSuite::named("fibonacci")
            .add(vec![0], 0)
            .add(vec![1], 1)
            .add(vec![2], 1)
            .add(vec![3], 2)
            .add(vec![4], 3)
            .add(vec![5], 5)
            .add(vec![6], 8)
            .add(vec![10], 55)
    }

    /// Test suite for GCD
    pub fn gcd_suite() -> TestSuite {
        TestSuite::named("gcd")
            .add(vec![12, 8], 4)
            .add(vec![17, 13], 1)
            .add(vec![100, 25], 25)
            .add(vec![48, 18], 6)
            .add(vec![1, 1], 1)
            .add(vec![0, 5], 5)
    }

    /// Test suite for absolute value
    pub fn abs_suite() -> TestSuite {
        TestSuite::named("abs")
            .add(vec![0], 0)
            .add(vec![5], 5)
            .add(vec![-5], 5)
            .add(vec![100], 100)
            .add(vec![-100], 100)
    }

    /// Test suite for maximum of two values
    pub fn max_suite() -> TestSuite {
        TestSuite::named("max")
            .add(vec![5, 3], 5)
            .add(vec![3, 5], 5)
            .add(vec![5, 5], 5)
            .add(vec![-3, -5], -3)
            .add(vec![0, -1], 0)
    }

    /// Test suite for minimum of two values
    pub fn min_suite() -> TestSuite {
        TestSuite::named("min")
            .add(vec![5, 3], 3)
            .add(vec![3, 5], 3)
            .add(vec![5, 5], 5)
            .add(vec![-3, -5], -5)
            .add(vec![0, -1], -1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Instruction, Opcode, Register};

    fn create_add_program() -> Program {
        // Simple program: r0 = r0 + r1
        Program {
            instructions: vec![
                Instruction::new(Opcode::Alu, Register::R0, Register::R0, Register::R1, 0),
                Instruction::new(
                    Opcode::Halt,
                    Register::Zero,
                    Register::Zero,
                    Register::Zero,
                    0,
                ),
            ],
            entry_point: 0,
            data_section: Vec::new(),
            entry_label: None,
        }
    }

    #[test]
    fn test_verifier_pass() {
        let program = create_add_program();
        let suite = TestSuite::new()
            .add(vec![5, 3], 8)
            .add(vec![0, 0], 0)
            .add(vec![100, 200], 300);

        let mut verifier = Verifier::new();
        let result = verifier.verify(&program, &suite);

        assert!(result.all_passed);
        assert_eq!(result.passed_count, 3);
        assert_eq!(result.failed_count, 0);
    }

    #[test]
    fn test_verifier_fail() {
        let program = create_add_program();
        let suite = TestSuite::new()
            .add(vec![5, 3], 8) // Pass
            .add(vec![5, 3], 9); // Fail (expects 9, gets 8)

        let mut verifier = Verifier::new();
        let result = verifier.verify(&program, &suite);

        assert!(!result.all_passed);
        assert_eq!(result.passed_count, 1);
        assert_eq!(result.failed_count, 1);
    }

    #[test]
    fn test_quick_verify() {
        let program = create_add_program();
        let suite = TestSuite::new().add(vec![5, 3], 8);

        let mut verifier = Verifier::new();
        let result = verifier.verify_quick(&program, &suite);

        assert!(result.is_ok());
    }

    #[test]
    fn test_quick_verify_fail() {
        let program = create_add_program();
        let suite = TestSuite::new().add(vec![5, 3], 100);

        let mut verifier = Verifier::new();
        let result = verifier.verify_quick(&program, &suite);

        assert!(result.is_err());
    }

    #[test]
    fn test_common_suites() {
        // Just verify suites are created correctly
        assert!(!common::addition_suite().is_empty());
        assert!(!common::factorial_suite().is_empty());
        assert!(!common::fibonacci_suite().is_empty());
    }

    #[test]
    fn test_verification_result_summary() {
        let result = VerificationResult {
            results: vec![],
            total_duration: Duration::from_millis(10),
            all_passed: true,
            passed_count: 5,
            failed_count: 0,
        };

        let summary = result.summary();
        assert!(summary.contains("5 tests passed"));
    }

    #[test]
    fn test_format_errors() {
        let result = VerificationResult {
            results: vec![
                TestResult::fail(0, 8, 9, Duration::from_millis(1)),
                TestResult::pass(1, 5, 5, Duration::from_millis(1)),
            ],
            total_duration: Duration::from_millis(2),
            all_passed: false,
            passed_count: 1,
            failed_count: 1,
        };

        let errors = result.format_errors();
        assert!(errors.contains("Test 0"));
        assert!(errors.contains("Expected 9, got 8"));
    }
}
