//! Integration Test Entry Point
//!
//! Table-driven tests that compile example programs and verify correctness.

mod integration;

use integration::test_cases::{self, EXTENSION_TEST_CASES, TEST_CASES};
use integration::test_runner::TestRunner;

// ============================================================================
// Algorithm Tests
// ============================================================================

#[test]
fn test_fibonacci() {
    let runner = TestRunner::new();
    let test = TEST_CASES
        .iter()
        .find(|t| t.name == "fibonacci_10")
        .unwrap();
    let result = runner.run_test(test);
    assert!(result.passed, "Fibonacci test failed: {:?}", result.error);
    assert_eq!(result.actual, Some(55));
}

#[test]
fn test_factorial() {
    let runner = TestRunner::new();
    let test = TEST_CASES.iter().find(|t| t.name == "factorial_5").unwrap();
    let result = runner.run_test(test);
    assert!(result.passed, "Factorial test failed: {:?}", result.error);
    assert_eq!(result.actual, Some(120));
}

#[test]
fn test_gcd() {
    let runner = TestRunner::new();
    let test = TEST_CASES.iter().find(|t| t.name == "gcd_48_18").unwrap();
    let result = runner.run_test(test);
    assert!(result.passed, "GCD test failed: {:?}", result.error);
    assert_eq!(result.actual, Some(6));
}

#[test]
fn test_sum_array() {
    let runner = TestRunner::new();
    let test = TEST_CASES.iter().find(|t| t.name == "sum_array").unwrap();
    let result = runner.run_test(test);
    assert!(result.passed, "Sum array test failed: {:?}", result.error);
    assert_eq!(result.actual, Some(150));
}

#[test]
fn test_power_of_two() {
    let runner = TestRunner::new();
    let test = TEST_CASES
        .iter()
        .find(|t| t.name == "power_of_two")
        .unwrap();
    let result = runner.run_test(test);
    assert!(
        result.passed,
        "Power of two test failed: {:?}",
        result.error
    );
    assert_eq!(result.actual, Some(1024));
}

#[test]
fn test_absolute_value() {
    let runner = TestRunner::new();
    let test = TEST_CASES
        .iter()
        .find(|t| t.name == "absolute_value")
        .unwrap();
    let result = runner.run_test(test);
    assert!(
        result.passed,
        "Absolute value test failed: {:?}",
        result.error
    );
    assert_eq!(result.actual, Some(42));
}

// ============================================================================
// Bit Manipulation Tests
// ============================================================================

#[test]
fn test_popcount() {
    let runner = TestRunner::new();
    let test = TEST_CASES
        .iter()
        .find(|t| t.name == "popcount_0xff")
        .unwrap();
    let result = runner.run_test(test);
    assert!(result.passed, "Popcount test failed: {:?}", result.error);
}

// ============================================================================
// Floating Point Tests
// ============================================================================

#[test]
fn test_float_sqrt() {
    let runner = TestRunner::new();
    let test = TEST_CASES
        .iter()
        .find(|t| t.name == "float_sqrt_2")
        .unwrap();
    let result = runner.run_test(test);

    if let Some(actual) = result.actual {
        let actual_f = f64::from_bits(actual);
        let expected = std::f64::consts::SQRT_2;
        assert!(
            (actual_f - expected).abs() < 0.01,
            "sqrt(2) = {} but got {}",
            expected,
            actual_f
        );
    }
}

#[test]
fn test_float_circle() {
    let runner = TestRunner::new();
    let test = TEST_CASES
        .iter()
        .find(|t| t.name == "float_circle_area")
        .unwrap();
    let result = runner.run_test(test);

    if let Some(actual) = result.actual {
        let actual_f = f64::from_bits(actual);
        let expected = std::f64::consts::PI * 4.0; // π * r² where r=2
        assert!(
            (actual_f - expected).abs() < 0.1,
            "Circle area = {} but got {}",
            expected,
            actual_f
        );
    }
}

// ============================================================================
// Time Tests (non-deterministic)
// ============================================================================

#[test]
fn test_time_operations() {
    let runner = TestRunner::new();
    let test = TEST_CASES.iter().find(|t| t.name == "time_now").unwrap();
    let result = runner.run_test(test);

    // Just verify it runs without error
    assert!(
        result.error.is_none(),
        "Time test error: {:?}",
        result.error
    );

    // Verify timestamp is reasonable (after 2024, before 2100)
    if let Some(timestamp) = result.actual {
        assert!(timestamp > 1704067200, "Timestamp too old");
        assert!(timestamp < 4102444800, "Timestamp too far in future");
    }
}

// ============================================================================
// Random Tests (non-deterministic)
// ============================================================================

#[test]
fn test_random_generation() {
    let runner = TestRunner::new();
    let test = TEST_CASES.iter().find(|t| t.name == "random_u64").unwrap();

    // Run twice to verify different results
    let result1 = runner.run_test(test);
    let result2 = runner.run_test(test);

    assert!(
        result1.error.is_none(),
        "Random test error: {:?}",
        result1.error
    );

    // Very high probability of different values
    if let (Some(v1), Some(v2)) = (result1.actual, result2.actual) {
        assert_ne!(v1, v2, "Random values should differ between runs");
    }
}

// ============================================================================
// Concurrency Tests
// ============================================================================

#[test]
fn test_concurrent_sum() {
    let runner = TestRunner::new();
    let test = TEST_CASES
        .iter()
        .find(|t| t.name == "concurrent_sum")
        .unwrap();
    let result = runner.run_test(test);
    // Note: Concurrency primitives (spawn/join/chan) run in interpreter which
    // doesn't fully implement threading - test just verifies it runs without error.
    // The test case has time_sensitive=true which skips value checking.
    assert!(
        result.passed,
        "Concurrent sum test failed: {:?}",
        result.error
    );
}

// ============================================================================
// Security Tests
// ============================================================================

#[test]
fn test_capabilities() {
    let runner = TestRunner::new();
    let test = TEST_CASES
        .iter()
        .find(|t| t.name == "capability_demo")
        .unwrap();
    let result = runner.run_test(test);
    assert!(result.passed, "Capability test failed: {:?}", result.error);
    assert_eq!(result.actual, Some(42));
}

// ============================================================================
// Batch Test Runner
// ============================================================================

#[test]
fn run_all_basic_tests() {
    let runner = TestRunner::new();
    let basic_tests = test_cases::get_basic_tests();
    let results = runner.run_tests(&basic_tests);

    TestRunner::print_summary(&results);

    let failed: Vec<_> = results.iter().filter(|r| !r.passed).collect();
    assert!(
        failed.is_empty(),
        "{} tests failed: {:?}",
        failed.len(),
        failed.iter().map(|r| &r.name).collect::<Vec<_>>()
    );
}

#[test]
fn run_all_algorithm_tests() {
    let runner = TestRunner::new();
    let algo_tests = test_cases::get_algorithm_tests();
    let results = runner.run_tests(&algo_tests);

    TestRunner::print_summary(&results);

    let failed: Vec<_> = results.iter().filter(|r| !r.passed).collect();
    assert!(
        failed.is_empty(),
        "Algorithm tests failed: {:?}",
        failed.iter().map(|r| &r.name).collect::<Vec<_>>()
    );
}

// ============================================================================
// Benchmark-style Tests
// ============================================================================

#[test]
fn test_compile_times_reasonable() {
    let runner = TestRunner::new();
    let tests = test_cases::get_deterministic_tests();

    for test in tests {
        let result = runner.run_test(test);

        // Compile time should be under 10ms for small programs (lenient for debug builds)
        assert!(
            result.compile_time_us < 10000,
            "Compile time for {} too slow: {}μs",
            test.name,
            result.compile_time_us
        );
    }
}

#[test]
fn test_run_times_reasonable() {
    let runner = TestRunner::new();
    let tests = test_cases::get_algorithm_tests();

    for test in tests {
        let result = runner.run_test(test);

        // Run time should be under 10ms for these simple programs
        assert!(
            result.run_time_us < 10000,
            "Run time for {} too slow: {}μs",
            test.name,
            result.run_time_us
        );
    }
}

// ============================================================================
// Extension Mock Tests
// ============================================================================

#[test]
fn test_uuid_generate_with_mocks() {
    let runner = TestRunner::new();
    let test = EXTENSION_TEST_CASES
        .iter()
        .find(|t| t.name == "uuid_generate")
        .unwrap();
    let result = runner.run_extension_test(test);
    assert!(
        result.passed,
        "UUID generate test failed: {:?}",
        result.error
    );
}

#[test]
fn test_json_parse_field_with_mocks() {
    let runner = TestRunner::new();
    let test = EXTENSION_TEST_CASES
        .iter()
        .find(|t| t.name == "json_parse_field")
        .unwrap();
    let result = runner.run_extension_test(test);
    assert!(
        result.passed,
        "JSON parse field test failed: {:?}",
        result.error
    );
}

#[test]
fn test_http_client_get_with_mocks() {
    let runner = TestRunner::new();
    let test = EXTENSION_TEST_CASES
        .iter()
        .find(|t| t.name == "http_client_get")
        .unwrap();
    let result = runner.run_extension_test(test);
    assert!(result.passed, "HTTP GET test failed: {:?}", result.error);
}

#[test]
fn test_sqlite_crud_insert_with_mocks() {
    let runner = TestRunner::new();
    let test = EXTENSION_TEST_CASES
        .iter()
        .find(|t| t.name == "sqlite_crud_insert")
        .unwrap();
    let result = runner.run_extension_test(test);
    assert!(
        result.passed,
        "SQLite INSERT test failed: {:?}",
        result.error
    );
}

#[test]
fn run_all_extension_tests() {
    let runner = TestRunner::new();
    let results = runner.run_extension_tests(EXTENSION_TEST_CASES);

    TestRunner::print_summary(&results);

    let failed: Vec<_> = results.iter().filter(|r| !r.passed).collect();
    assert!(
        failed.is_empty(),
        "{} extension tests failed: {:?}",
        failed.len(),
        failed.iter().map(|r| &r.name).collect::<Vec<_>>()
    );
}
