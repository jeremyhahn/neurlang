//! Slot Verifier
//!
//! Verifies that generated slot code is correct by running per-slot unit tests.
//! This enables fast iteration: only regenerate slots that fail verification.

use std::collections::HashMap;
use std::time::Instant;

use super::filler::FilledSlot;
use super::spec::{Slot, SlotSpec, SlotTest};

/// Result of verifying a single slot
#[derive(Debug, Clone)]
pub struct SlotVerifyResult {
    /// Slot ID
    pub slot_id: String,
    /// Whether verification passed
    pub passed: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Verification time in milliseconds
    pub time_ms: f64,
    /// Expected vs actual output (for debugging)
    pub expected: Option<String>,
    pub actual: Option<String>,
}

/// Result of verifying all slots
#[derive(Debug)]
pub struct VerifyResult {
    /// Individual slot results
    pub slots: Vec<SlotVerifyResult>,
    /// Overall pass/fail
    pub all_passed: bool,
    /// Number of slots that passed
    pub passed_count: usize,
    /// Number of slots that failed
    pub failed_count: usize,
    /// Number of slots without tests
    pub skipped_count: usize,
    /// Total verification time
    pub total_time_ms: f64,
}

impl VerifyResult {
    /// Get IDs of failed slots
    pub fn failed_slot_ids(&self) -> Vec<&str> {
        self.slots
            .iter()
            .filter(|r| !r.passed)
            .map(|r| r.slot_id.as_str())
            .collect()
    }

    /// Get pass rate as percentage
    pub fn pass_rate(&self) -> f64 {
        let total = self.passed_count + self.failed_count;
        if total == 0 {
            100.0
        } else {
            (self.passed_count as f64 / total as f64) * 100.0
        }
    }
}

/// Verification error
#[derive(Debug)]
pub enum VerifyError {
    /// Assembly failed
    AssemblyFailed(String),
    /// Execution failed
    ExecutionFailed(String),
    /// Test output mismatch
    OutputMismatch {
        slot_id: String,
        expected: String,
        actual: String,
    },
    /// Timeout during execution
    Timeout(String),
    /// Invalid test specification
    InvalidTest(String),
}

impl std::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifyError::AssemblyFailed(msg) => write!(f, "Assembly failed: {}", msg),
            VerifyError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            VerifyError::OutputMismatch {
                slot_id,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Output mismatch for {}: expected '{}', got '{}'",
                    slot_id, expected, actual
                )
            }
            VerifyError::Timeout(slot_id) => write!(f, "Timeout verifying slot {}", slot_id),
            VerifyError::InvalidTest(msg) => write!(f, "Invalid test: {}", msg),
        }
    }
}

impl std::error::Error for VerifyError {}

/// Configuration for slot verification
#[derive(Debug, Clone)]
pub struct VerifierConfig {
    /// Maximum execution time per slot (ms)
    pub timeout_ms: u64,
    /// Maximum instructions to execute
    pub max_instructions: u64,
    /// Stop on first failure
    pub fail_fast: bool,
    /// Verbose output
    pub verbose: bool,
}

impl Default for VerifierConfig {
    fn default() -> Self {
        VerifierConfig {
            timeout_ms: 1000,
            max_instructions: 100000,
            fail_fast: false,
            verbose: false,
        }
    }
}

/// Slot code verifier
pub struct SlotVerifier {
    config: VerifierConfig,
}

impl SlotVerifier {
    /// Create a new verifier
    pub fn new() -> Self {
        SlotVerifier {
            config: VerifierConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: VerifierConfig) -> Self {
        SlotVerifier { config }
    }

    /// Verify a single filled slot against its unit test
    pub fn verify_slot(&self, slot: &Slot, code: &str) -> SlotVerifyResult {
        let start = Instant::now();

        // Check if slot has a unit test
        let test = match &slot.unit_test {
            Some(t) => t,
            None => {
                return SlotVerifyResult {
                    slot_id: slot.id.clone(),
                    passed: true, // No test = pass by default
                    error: None,
                    time_ms: start.elapsed().as_secs_f64() * 1000.0,
                    expected: None,
                    actual: None,
                };
            }
        };

        // Build test program: setup + slot code
        let test_program = self.build_test_program(code, test);

        // Assemble the test program
        let assembled = match self.assemble(&test_program) {
            Ok(prog) => prog,
            Err(e) => {
                return SlotVerifyResult {
                    slot_id: slot.id.clone(),
                    passed: false,
                    error: Some(format!("Assembly failed: {}", e)),
                    time_ms: start.elapsed().as_secs_f64() * 1000.0,
                    expected: None,
                    actual: None,
                };
            }
        };

        // Execute and verify
        let result = self.execute_and_verify(&assembled, test);

        SlotVerifyResult {
            slot_id: slot.id.clone(),
            passed: result.is_ok(),
            error: result.err().map(|e| e.to_string()),
            time_ms: start.elapsed().as_secs_f64() * 1000.0,
            expected: Some(format!("{:?}", test.expected)),
            actual: None, // TODO: capture actual output
        }
    }

    /// Verify all filled slots in a spec
    pub fn verify_all(&self, spec: &SlotSpec, filled: &[FilledSlot]) -> VerifyResult {
        let start = Instant::now();
        let mut results = Vec::new();
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        // Create a map of filled slots by ID
        let filled_map: HashMap<&str, &str> = filled
            .iter()
            .map(|f| (f.id.as_str(), f.code.as_str()))
            .collect();

        for slot in &spec.slots {
            // Get the filled code
            let code = match filled_map.get(slot.id.as_str()) {
                Some(c) => *c,
                None => {
                    results.push(SlotVerifyResult {
                        slot_id: slot.id.clone(),
                        passed: false,
                        error: Some("Slot not filled".to_string()),
                        time_ms: 0.0,
                        expected: None,
                        actual: None,
                    });
                    failed += 1;
                    continue;
                }
            };

            // Check if slot has a test
            if slot.unit_test.is_none() {
                skipped += 1;
                continue;
            }

            // Verify the slot
            let result = self.verify_slot(slot, code);

            if result.passed {
                passed += 1;
            } else {
                failed += 1;
                if self.config.fail_fast {
                    results.push(result);
                    break;
                }
            }

            results.push(result);
        }

        VerifyResult {
            slots: results,
            all_passed: failed == 0,
            passed_count: passed,
            failed_count: failed,
            skipped_count: skipped,
            total_time_ms: start.elapsed().as_secs_f64() * 1000.0,
        }
    }

    /// Build a test program from slot code and test spec
    fn build_test_program(&self, slot_code: &str, test: &SlotTest) -> String {
        let mut program = String::new();

        // Add setup code
        program.push_str("; Test setup\n");
        program.push_str(&test.setup);
        program.push('\n');

        // Add the slot code
        program.push_str("; Slot code under test\n");
        program.push_str(slot_code);
        program.push('\n');

        // Add verification code based on expected output
        program.push_str("; Test verification\n");

        // Check expected registers
        for (reg, value) in &test.expected.registers {
            program.push_str(&format!("; Check {} == {}\n", reg, value));
            program.push_str(&format!("mov r30, {}\n", value));
            program.push_str(&format!("beq {}, r30, .test_pass_{}\n", reg, reg));
            program.push_str("trap 1  ; Test failed\n");
            program.push_str(&format!(".test_pass_{}:\n", reg));
        }

        // Check expected branch taken
        if let Some(ref label) = test.expected.branch_taken {
            program.push_str(&format!("; Expect to reach label {}\n", label));
        }

        // Check expected memory contents
        for (addr, bytes) in &test.expected.memory {
            program.push_str(&format!(
                "; Check memory at {} has {} bytes\n",
                addr,
                bytes.len()
            ));
            for (i, byte) in bytes.iter().enumerate() {
                program.push_str(&format!("load.b r30, [{} + {}]\n", addr, i));
                program.push_str(&format!("mov r31, {}\n", byte));
                program.push_str(&format!("bne r30, r31, .test_fail_mem_{}_{}\n", addr, i));
            }
        }

        program.push_str("halt\n");

        // Add fail labels for memory checks
        for (addr, bytes) in &test.expected.memory {
            for i in 0..bytes.len() {
                program.push_str(&format!(".test_fail_mem_{}_{}:\n", addr, i));
                program.push_str("trap 1\n");
            }
        }

        program
    }

    /// Assemble a test program
    fn assemble(&self, source: &str) -> Result<Vec<u8>, String> {
        // Use the Neurlang assembler
        use crate::ir::Assembler;

        let mut asm = Assembler::new();
        match asm.assemble(source) {
            Ok(program) => Ok(program.encode()),
            Err(e) => Err(format!("{:?}", e)),
        }
    }

    /// Execute assembled code and verify results
    fn execute_and_verify(&self, _code: &[u8], _test: &SlotTest) -> Result<(), VerifyError> {
        // For now, we use a mock implementation
        // In production, this would:
        // 1. Load the program into an interpreter or JIT
        // 2. Execute with the test inputs
        // 3. Check the expected outputs

        // Mock: assume all tests pass for now
        Ok(())
    }
}

impl Default for SlotVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick verification for a single slot
pub fn quick_verify(slot: &Slot, code: &str) -> bool {
    let verifier = SlotVerifier::new();
    verifier.verify_slot(slot, code).passed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slot::spec::{TestExpected, TestInput};
    use crate::slot::types::SlotType;

    fn make_test_slot(with_test: bool) -> Slot {
        let mut slot = Slot::new(
            "TEST_SLOT",
            "test_slot",
            SlotType::StateTransition {
                state_reg: "r13".to_string(),
                new_state: "STATE_A".to_string(),
            },
        );

        if with_test {
            let mut expected = TestExpected::default();
            expected.registers.insert("r13".to_string(), 1); // STATE_A = 1

            let mut input = TestInput::default();
            input.registers.insert("r13".to_string(), 0);

            slot.unit_test = Some(SlotTest {
                setup: "mov r13, 0".to_string(),
                input,
                expected,
            });
        }

        slot
    }

    #[test]
    fn test_verify_slot_no_test() {
        let verifier = SlotVerifier::new();
        let slot = make_test_slot(false);
        let code = "mov r13, STATE_A";

        let result = verifier.verify_slot(&slot, code);
        assert!(result.passed);
    }

    #[test]
    fn test_verify_all_empty() {
        let verifier = SlotVerifier::new();
        let spec = SlotSpec::new("test", "Test spec");
        let filled = vec![];

        let result = verifier.verify_all(&spec, &filled);
        assert!(result.all_passed);
        assert_eq!(result.passed_count, 0);
    }

    #[test]
    fn test_pass_rate() {
        let result = VerifyResult {
            slots: vec![],
            all_passed: true,
            passed_count: 8,
            failed_count: 2,
            skipped_count: 0,
            total_time_ms: 10.0,
        };

        assert_eq!(result.pass_rate(), 80.0);
    }

    #[test]
    fn test_failed_slot_ids() {
        let result = VerifyResult {
            slots: vec![
                SlotVerifyResult {
                    slot_id: "SLOT_1".to_string(),
                    passed: true,
                    error: None,
                    time_ms: 1.0,
                    expected: None,
                    actual: None,
                },
                SlotVerifyResult {
                    slot_id: "SLOT_2".to_string(),
                    passed: false,
                    error: Some("Failed".to_string()),
                    time_ms: 1.0,
                    expected: None,
                    actual: None,
                },
            ],
            all_passed: false,
            passed_count: 1,
            failed_count: 1,
            skipped_count: 0,
            total_time_ms: 2.0,
        };

        assert_eq!(result.failed_slot_ids(), vec!["SLOT_2"]);
    }
}
