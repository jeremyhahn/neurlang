//! Test Case Definitions
//!
//! Defines expected inputs and outputs for all example programs.

/// A test case for a Neurlang program
#[derive(Debug, Clone)]
pub struct TestCase {
    /// Name of the test
    pub name: &'static str,
    /// Source file (relative to examples/)
    pub source: &'static str,
    /// Input register values (register_index, value)
    pub input_regs: &'static [(usize, u64)],
    /// Expected value in R0 after execution
    pub expected_r0: Option<u64>,
    /// Expected values in other registers
    pub expected_regs: &'static [(usize, u64)],
    /// Whether floating-point comparison is needed
    pub is_float: bool,
    /// Tolerance for float comparison
    pub float_tolerance: f64,
    /// Whether the test requires I/O (may need to skip in some environments)
    pub requires_io: bool,
    /// Whether the test is time-sensitive (may vary between runs)
    pub time_sensitive: bool,
    /// Whether the test uses random numbers (result varies)
    pub uses_random: bool,
    /// Description of what the test verifies
    #[allow(dead_code)]
    pub description: &'static str,
}

impl TestCase {
    /// Create a simple test case with input and expected R0 value
    pub const fn with_input(
        name: &'static str,
        source: &'static str,
        input_regs: &'static [(usize, u64)],
        expected_r0: u64,
        description: &'static str,
    ) -> Self {
        Self {
            name,
            source,
            input_regs,
            expected_r0: Some(expected_r0),
            expected_regs: &[],
            is_float: false,
            float_tolerance: 0.0,
            requires_io: false,
            time_sensitive: false,
            uses_random: false,
            description,
        }
    }

    /// Create a simple test case with no inputs (uses program defaults)
    pub const fn simple(
        name: &'static str,
        source: &'static str,
        expected_r0: u64,
        description: &'static str,
    ) -> Self {
        Self {
            name,
            source,
            input_regs: &[],
            expected_r0: Some(expected_r0),
            expected_regs: &[],
            is_float: false,
            float_tolerance: 0.0,
            requires_io: false,
            time_sensitive: false,
            uses_random: false,
            description,
        }
    }

    /// Create a test case for floating-point result
    #[allow(dead_code)]
    pub const fn float(
        name: &'static str,
        source: &'static str,
        expected: f64,
        tolerance: f64,
        description: &'static str,
    ) -> Self {
        Self {
            name,
            source,
            input_regs: &[],
            expected_r0: Some(expected.to_bits()),
            expected_regs: &[],
            is_float: true,
            float_tolerance: tolerance,
            requires_io: false,
            time_sensitive: false,
            uses_random: false,
            description,
        }
    }

    /// Create a test that only verifies execution completes
    #[allow(dead_code)]
    pub const fn runs(name: &'static str, source: &'static str, description: &'static str) -> Self {
        Self {
            name,
            source,
            input_regs: &[],
            expected_r0: None,
            expected_regs: &[],
            is_float: false,
            float_tolerance: 0.0,
            requires_io: false,
            time_sensitive: false,
            uses_random: false,
            description,
        }
    }
}

/// All test cases for example programs
pub const TEST_CASES: &[TestCase] = &[
    // ========================================================================
    // Basic Algorithms
    // ========================================================================
    TestCase::with_input(
        "fibonacci_10",
        "algorithm/math/fibonacci.nl",
        &[(0, 10)], // r0 = 10
        55,
        "Computes fib(10) = 55 using iterative loop",
    ),
    TestCase::with_input(
        "factorial_5",
        "algorithm/math/factorial.nl",
        &[(0, 5)], // r0 = 5
        120,
        "Computes 5! = 120 using multiplication loop",
    ),
    TestCase::with_input(
        "gcd_48_18",
        "algorithm/math/gcd.nl",
        &[(0, 48), (1, 18)], // r0 = 48, r1 = 18
        6,
        "Computes GCD(48, 18) = 6 using Euclidean algorithm",
    ),
    TestCase::simple(
        "sum_array",
        "algorithm/array/sum_array.nl",
        150,
        "Sums array [10, 20, 30, 40, 50] = 150 (hardcoded data)",
    ),
    TestCase::with_input(
        "power_of_two",
        "algorithm/math/power_of_two.nl",
        &[(0, 10)], // r0 = 10
        1024,
        "Computes 2^10 = 1024 using bit shift",
    ),
    TestCase::with_input(
        "absolute_value",
        "algorithm/math/absolute_value.nl",
        &[(0, 42)], // r0 = 42 (positive, returns unchanged)
        42,
        "Computes |42| = 42",
    ),
    // ========================================================================
    // Bit Manipulation
    // ========================================================================
    TestCase {
        name: "popcount_0xff",
        source: "algorithm/bitwise/bitcount.nl",
        input_regs: &[(0, 0xFF00FF)], // r0 = 0xFF00FF
        expected_r0: Some(16),        // popcount of 0xFF00FF
        expected_regs: &[],
        is_float: false,
        float_tolerance: 0.0,
        requires_io: false,
        time_sensitive: false,
        uses_random: false,
        description: "Tests popcount (0xFF00FF has 16 bits set)",
    },
    // ========================================================================
    // Floating Point
    // ========================================================================

    // Note: Float tests use bit representation comparison with tolerance
    TestCase {
        name: "float_sqrt_2",
        source: "algorithm/math/float_sqrt.nl",
        input_regs: &[],   // Uses hardcoded input
        expected_r0: None, // sqrt(2) ≈ 1.414...
        expected_regs: &[],
        is_float: true,
        float_tolerance: 0.001,
        requires_io: false,
        time_sensitive: false,
        uses_random: false,
        description: "Computes sqrt(2.0) using FPU",
    },
    TestCase {
        name: "float_circle_area",
        source: "algorithm/math/float_circle.nl",
        input_regs: &[],   // Uses hardcoded input
        expected_r0: None, // π * 2² ≈ 12.566
        expected_regs: &[],
        is_float: true,
        float_tolerance: 0.01,
        requires_io: false,
        time_sensitive: false,
        uses_random: false,
        description: "Computes circle area π×r² for r=2",
    },
    // ========================================================================
    // Time Operations
    // ========================================================================
    TestCase {
        name: "time_now",
        source: "system/time_example.nl",
        input_regs: &[],
        expected_r0: None,
        expected_regs: &[],
        is_float: false,
        float_tolerance: 0.0,
        requires_io: false,
        time_sensitive: true,
        uses_random: false,
        description: "Gets current Unix timestamp",
    },
    // ========================================================================
    // Random Numbers
    // ========================================================================
    TestCase {
        name: "random_u64",
        source: "crypto/random.nl",
        input_regs: &[],
        expected_r0: None,
        expected_regs: &[],
        is_float: false,
        float_tolerance: 0.0,
        requires_io: false,
        time_sensitive: false,
        uses_random: true,
        description: "Generates random u64 value",
    },
    // ========================================================================
    // I/O Operations
    // ========================================================================
    TestCase {
        name: "hello_world",
        source: "io/console/hello_world.nl",
        input_regs: &[],
        expected_r0: Some(0),
        expected_regs: &[],
        is_float: false,
        float_tolerance: 0.0,
        requires_io: true,
        time_sensitive: false,
        uses_random: false,
        description: "Prints 'Hello, Neurlang!' to stdout",
    },
    // ========================================================================
    // Concurrency (may need special handling)
    // ========================================================================
    TestCase {
        name: "concurrent_sum",
        source: "concurrency/concurrent.nl",
        input_regs: &[],
        expected_r0: Some(60), // 10*2 + 20*2 = 60
        expected_regs: &[],
        is_float: false,
        float_tolerance: 0.0,
        requires_io: false,
        time_sensitive: true, // Concurrency not implemented in interpreter
        uses_random: false,
        description: "Spawns workers that double values and sum results",
    },
    // ========================================================================
    // Security Features
    // ========================================================================
    TestCase {
        name: "capability_demo",
        source: "security/capability_demo.nl",
        input_regs: &[],
        expected_r0: Some(42),
        expected_regs: &[],
        is_float: false,
        float_tolerance: 0.0,
        requires_io: false,
        time_sensitive: false,
        uses_random: false,
        description: "Demonstrates capability-based memory access",
    },
];

/// Get test cases filtered by category
pub fn get_deterministic_tests() -> Vec<&'static TestCase> {
    TEST_CASES
        .iter()
        .filter(|t| !t.time_sensitive && !t.uses_random && !t.requires_io)
        .collect()
}

/// Get all arithmetic/algorithm tests
pub fn get_algorithm_tests() -> Vec<&'static TestCase> {
    TEST_CASES
        .iter()
        .filter(|t| {
            t.name.starts_with("fibonacci")
                || t.name.starts_with("factorial")
                || t.name.starts_with("gcd")
                || t.name.starts_with("sum")
                || t.name.starts_with("power")
                || t.name.starts_with("absolute")
        })
        .collect()
}

/// Get all floating-point tests
#[allow(dead_code)]
pub fn get_float_tests() -> Vec<&'static TestCase> {
    TEST_CASES.iter().filter(|t| t.is_float).collect()
}

/// Get tests that can run without special environment
pub fn get_basic_tests() -> Vec<&'static TestCase> {
    TEST_CASES
        .iter()
        .filter(|t| !t.requires_io && !t.time_sensitive && !t.uses_random)
        .collect()
}

// ============================================================================
// Extension Mock Test Cases
// ============================================================================

/// Mock specification for extension testing
pub struct MockSpec {
    /// Extension ID to mock
    pub id: u32,
    /// Return value
    pub return_value: i64,
    /// Output values
    pub outputs: &'static [u64],
}

/// Test case for extension-based examples (uses mocks)
pub struct ExtensionTestCase {
    /// Name of the test
    pub name: &'static str,
    /// Source file (relative to examples/)
    pub source: &'static str,
    /// Mocks to set up before running
    pub mocks: &'static [MockSpec],
    /// Expected value in R0 after execution
    pub expected_r0: Option<u64>,
    /// Description of what the test verifies
    #[allow(dead_code)]
    pub description: &'static str,
}

/// Extension test cases with mocks
/// Extension IDs from examples' @note annotations:
/// - json_parse = 200, json_get = 202, json_free = 209
/// - http_get = 220, http_response_status = 226, http_response_body = 227, http_free = 231
/// - uuid_v4 = 330, uuid_to_string = 333
/// - sqlite_open = 260, sqlite_exec = 264, sqlite_close = 261, etc.
pub const EXTENSION_TEST_CASES: &[ExtensionTestCase] = &[
    // UUID generation
    ExtensionTestCase {
        name: "uuid_generate",
        source: "extension/uuid/uuid_generate.nl",
        mocks: &[
            MockSpec {
                id: 330,                  // uuid_v4
                return_value: 0x12345678, // Mock UUID bytes handle
                outputs: &[],
            },
            MockSpec {
                id: 333,               // uuid_to_string
                return_value: 0x20000, // Mock string pointer
                outputs: &[],
            },
        ],
        expected_r0: Some(0x20000), // Should return string pointer
        description: "Generates UUID v4 using mocked extensions",
    },
    // JSON parse field
    ExtensionTestCase {
        name: "json_parse_field",
        source: "extension/json/json_parse_field.nl",
        mocks: &[
            MockSpec {
                id: 200,              // json_parse
                return_value: 0x1000, // Mock JSON handle
                outputs: &[],
            },
            MockSpec {
                id: 202,              // json_get
                return_value: 0x2000, // Mock field value pointer
                outputs: &[],
            },
            MockSpec {
                id: 209, // json_free
                return_value: 0,
                outputs: &[],
            },
        ],
        expected_r0: Some(0x2000), // Should return field value
        description: "Parses JSON and extracts field using mocked extensions",
    },
    // HTTP client GET
    ExtensionTestCase {
        name: "http_client_get",
        source: "extension/http-client/http_client_get.nl",
        mocks: &[
            MockSpec {
                id: 220,              // http_get
                return_value: 0x5000, // Mock response handle
                outputs: &[],
            },
            MockSpec {
                id: 226,           // http_response_status
                return_value: 200, // HTTP 200 OK
                outputs: &[],
            },
            MockSpec {
                id: 227,              // http_response_body
                return_value: 0x6000, // Mock body pointer
                outputs: &[],
            },
            MockSpec {
                id: 231, // http_free
                return_value: 0,
                outputs: &[],
            },
        ],
        expected_r0: Some(0x6000), // Should return body pointer
        description: "HTTP GET request with mocked response",
    },
    // SQLite CRUD insert
    ExtensionTestCase {
        name: "sqlite_crud_insert",
        source: "extension/database/sqlite_crud_insert.nl",
        mocks: &[
            MockSpec {
                id: 260,              // sqlite_open
                return_value: 0x3000, // Mock DB handle
                outputs: &[],
            },
            MockSpec {
                id: 264,         // sqlite_exec
                return_value: 0, // Success
                outputs: &[],
            },
            MockSpec {
                id: 261, // sqlite_close
                return_value: 0,
                outputs: &[],
            },
            // Additional sqlite operations
            MockSpec {
                id: 266,              // sqlite_prepare
                return_value: 0x4000, // Mock stmt handle
                outputs: &[],
            },
            MockSpec {
                id: 268,         // sqlite_step
                return_value: 0, // SQLITE_DONE
                outputs: &[],
            },
            MockSpec {
                id: 270, // sqlite_finalize
                return_value: 0,
                outputs: &[],
            },
        ],
        expected_r0: Some(0),
        description: "SQLite INSERT with mocked database",
    },
];

/// Get all extension test cases
#[allow(dead_code)]
pub fn get_extension_tests() -> &'static [ExtensionTestCase] {
    EXTENSION_TEST_CASES
}
