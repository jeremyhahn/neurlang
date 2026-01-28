//! Stdlib Verification
//!
//! Verifies that Rust stdlib functions produce the same output as their
//! generated Neurlang IR counterparts.

use std::collections::HashMap;
use std::path::Path;

use crate::ir::assembler::Assembler;
use crate::jit::{JitExecutor, JitResult};

/// Result of a single test case verification.
#[derive(Debug)]
pub struct TestResult {
    pub function: String,
    pub inputs: Vec<u64>,
    pub rust_output: u64,
    pub neurlang_output: u64,
    pub passed: bool,
}

/// Verification test case.
#[derive(Debug, Clone)]
pub struct VerifyTestCase {
    pub inputs: Vec<u64>,
    pub memory_setup: Option<MemorySetup>,
}

/// Memory setup for functions that operate on pointers.
#[derive(Debug, Clone)]
pub struct MemorySetup {
    pub data: Vec<u8>,
    pub data_ptr_arg: usize, // Which argument is the pointer
}

/// Run verification for all stdlib functions.
pub fn verify_all(lib_dir: &Path, verbose: bool) -> Result<VerifyStats, String> {
    let mut stats = VerifyStats::default();
    let mut all_results = Vec::new();

    // Get test cases for each function
    let test_cases = get_stdlib_test_cases();

    for (func_name, cases) in test_cases {
        let nl_path = find_nl_file(lib_dir, &func_name);

        if nl_path.is_none() {
            if verbose {
                println!("  [SKIP] {} - .nl file not found", func_name);
            }
            stats.skipped += 1;
            continue;
        }

        let nl_path = nl_path.unwrap();

        // Load and assemble the .nl file
        let nl_source = match std::fs::read_to_string(&nl_path) {
            Ok(s) => s,
            Err(e) => {
                if verbose {
                    println!("  [ERROR] {} - Failed to read: {}", func_name, e);
                }
                stats.errors += 1;
                continue;
            }
        };

        let mut asm = Assembler::new();
        let program = match asm.assemble(&nl_source) {
            Ok(p) => p,
            Err(e) => {
                if verbose {
                    println!("  [ERROR] {} - Assembly failed: {}", func_name, e);
                }
                stats.errors += 1;
                continue;
            }
        };

        // Run each test case
        for case in &cases {
            let rust_result =
                run_rust_function(&func_name, &case.inputs, case.memory_setup.as_ref());

            if rust_result.is_none() {
                continue; // Function not supported for direct verification
            }
            let rust_output = rust_result.unwrap();

            // Run through Neurlang JIT
            let mut executor = JitExecutor::new();

            // Set up registers
            for (i, &val) in case.inputs.iter().enumerate() {
                executor.set_register(i, val);
            }

            let nl_output = match executor.execute(&program) {
                JitResult::Ok(val) | JitResult::Halted(val) => val,
                JitResult::Error(e) => {
                    if verbose {
                        println!(
                            "  [ERROR] {}({:?}) - Execution failed: {}",
                            func_name, case.inputs, e
                        );
                    }
                    stats.errors += 1;
                    continue;
                }
            };

            let passed = rust_output == nl_output;

            if passed {
                stats.passed += 1;
            } else {
                stats.failed += 1;
                if verbose {
                    println!("  [FAIL] {}({:?})", func_name, case.inputs);
                    println!("         Rust: {}, Neurlang: {}", rust_output, nl_output);
                }
            }

            all_results.push(TestResult {
                function: func_name.clone(),
                inputs: case.inputs.clone(),
                rust_output,
                neurlang_output: nl_output,
                passed,
            });
        }

        stats.functions_verified += 1;

        if verbose {
            let func_passed = all_results
                .iter()
                .filter(|r| r.function == func_name && r.passed)
                .count();
            let func_total = all_results
                .iter()
                .filter(|r| r.function == func_name)
                .count();
            if func_total > 0 {
                println!(
                    "  [{}] {} ({}/{})",
                    if func_passed == func_total {
                        "PASS"
                    } else {
                        "FAIL"
                    },
                    func_name,
                    func_passed,
                    func_total
                );
            }
        }
    }

    stats.results = all_results;
    Ok(stats)
}

/// Find the .nl file for a function name.
fn find_nl_file(lib_dir: &Path, func_name: &str) -> Option<std::path::PathBuf> {
    // Check each module directory
    for module in &["math", "float", "string", "array", "bitwise", "collections"] {
        let path = lib_dir.join(module).join(format!("{}.nl", func_name));
        if path.exists() {
            return Some(path);
        }
    }
    None
}

/// Run a Rust stdlib function with given inputs.
fn run_rust_function(name: &str, inputs: &[u64], _memory: Option<&MemorySetup>) -> Option<u64> {
    use neurlang_stdlib::*;

    // Math functions
    match name {
        "factorial" => Some(math::factorial(inputs.first().copied().unwrap_or(0))),
        "fibonacci" => Some(math::fibonacci(inputs.first().copied().unwrap_or(0))),
        "gcd" => Some(math::gcd(
            inputs.first().copied().unwrap_or(0),
            inputs.get(1).copied().unwrap_or(0),
        )),
        "lcm" => Some(math::lcm(
            inputs.first().copied().unwrap_or(0),
            inputs.get(1).copied().unwrap_or(0),
        )),
        "power" => Some(math::power(
            inputs.first().copied().unwrap_or(0),
            inputs.get(1).copied().unwrap_or(0),
        )),
        "is_prime" => Some(math::is_prime(inputs.first().copied().unwrap_or(0))),
        "isqrt" => Some(math::isqrt(inputs.first().copied().unwrap_or(0))),
        "abs_i64" => Some(math::abs_i64(inputs.first().copied().unwrap_or(0))),
        "min" => Some(math::min(
            inputs.first().copied().unwrap_or(0),
            inputs.get(1).copied().unwrap_or(0),
        )),
        "max" => Some(math::max(
            inputs.first().copied().unwrap_or(0),
            inputs.get(1).copied().unwrap_or(0),
        )),
        "div_ceil" => Some(math::div_ceil(
            inputs.first().copied().unwrap_or(0),
            inputs.get(1).copied().unwrap_or(1),
        )),
        "triangle_number" => Some(math::triangle_number(inputs.first().copied().unwrap_or(0))),

        // Bitwise functions
        "popcount" => Some(bitwise::popcount(inputs.first().copied().unwrap_or(0))),
        "clz" => Some(bitwise::clz(inputs.first().copied().unwrap_or(0))),
        "ctz" => Some(bitwise::ctz(inputs.first().copied().unwrap_or(0))),
        "bswap" => Some(bitwise::bswap(inputs.first().copied().unwrap_or(0))),
        "rotl" => Some(bitwise::rotl(
            inputs.first().copied().unwrap_or(0),
            inputs.get(1).copied().unwrap_or(0),
        )),
        "rotr" => Some(bitwise::rotr(
            inputs.first().copied().unwrap_or(0),
            inputs.get(1).copied().unwrap_or(0),
        )),
        "is_power_of_2" => Some(bitwise::is_power_of_2(inputs.first().copied().unwrap_or(0))),
        "next_power_of_2" => Some(bitwise::next_power_of_2(
            inputs.first().copied().unwrap_or(0),
        )),
        "reverse_bits" => Some(bitwise::reverse_bits(inputs.first().copied().unwrap_or(0))),
        "highest_set_bit" => Some(bitwise::highest_set_bit(
            inputs.first().copied().unwrap_or(0),
        )),
        "lowest_set_bit" => Some(bitwise::lowest_set_bit(
            inputs.first().copied().unwrap_or(0),
        )),
        "clear_lowest_bit" => Some(bitwise::clear_lowest_bit(
            inputs.first().copied().unwrap_or(0),
        )),
        "isolate_lowest_bit" => Some(bitwise::isolate_lowest_bit(
            inputs.first().copied().unwrap_or(0),
        )),
        "parity" => Some(bitwise::parity(inputs.first().copied().unwrap_or(0))),

        // Float functions (convert u64 bits to f64 for comparison)
        "fadd" => {
            let a = f64::from_bits(inputs.first().copied().unwrap_or(0));
            let b = f64::from_bits(inputs.get(1).copied().unwrap_or(0));
            Some(float::fadd(a, b).to_bits())
        }
        "fsub" => {
            let a = f64::from_bits(inputs.first().copied().unwrap_or(0));
            let b = f64::from_bits(inputs.get(1).copied().unwrap_or(0));
            Some(float::fsub(a, b).to_bits())
        }
        "fmul" => {
            let a = f64::from_bits(inputs.first().copied().unwrap_or(0));
            let b = f64::from_bits(inputs.get(1).copied().unwrap_or(0));
            Some(float::fmul(a, b).to_bits())
        }
        "fdiv" => {
            let a = f64::from_bits(inputs.first().copied().unwrap_or(0));
            let b = f64::from_bits(inputs.get(1).copied().unwrap_or(0));
            Some(float::fdiv(a, b).to_bits())
        }
        "fsqrt" => {
            let a = f64::from_bits(inputs.first().copied().unwrap_or(0));
            Some(float::fsqrt(a).to_bits())
        }
        "fabs" => {
            let a = f64::from_bits(inputs.first().copied().unwrap_or(0));
            Some(float::fabs(a).to_bits())
        }
        "ffloor" => {
            let a = f64::from_bits(inputs.first().copied().unwrap_or(0));
            Some(float::ffloor(a).to_bits())
        }
        "fceil" => {
            let a = f64::from_bits(inputs.first().copied().unwrap_or(0));
            Some(float::fceil(a).to_bits())
        }
        "fmin" => {
            let a = f64::from_bits(inputs.first().copied().unwrap_or(0));
            let b = f64::from_bits(inputs.get(1).copied().unwrap_or(0));
            Some(float::fmin(a, b).to_bits())
        }
        "fmax" => {
            let a = f64::from_bits(inputs.first().copied().unwrap_or(0));
            let b = f64::from_bits(inputs.get(1).copied().unwrap_or(0));
            Some(float::fmax(a, b).to_bits())
        }
        "f64_to_bits" => {
            let a = f64::from_bits(inputs.first().copied().unwrap_or(0));
            Some(float::f64_to_bits(a))
        }
        "f64_from_bits" => {
            let bits = inputs.first().copied().unwrap_or(0);
            Some(float::f64_from_bits(bits).to_bits())
        }

        // Character classification (string module)
        "is_digit" => Some(string::is_digit(inputs.first().copied().unwrap_or(0) as u8)),
        "is_alpha" => Some(string::is_alpha(inputs.first().copied().unwrap_or(0) as u8)),
        "is_alnum" => Some(string::is_alnum(inputs.first().copied().unwrap_or(0) as u8)),
        "is_space" => Some(string::is_space(inputs.first().copied().unwrap_or(0) as u8)),
        "to_upper" => Some(string::to_upper(inputs.first().copied().unwrap_or(0) as u8) as u64),
        "to_lower" => Some(string::to_lower(inputs.first().copied().unwrap_or(0) as u8) as u64),
        "char_to_digit" => Some(string::char_to_digit(
            inputs.first().copied().unwrap_or(0) as u8
        )),

        // Functions requiring memory setup - skip for now
        "strlen" | "strcmp" | "strcpy" | "strcat" | "atoi" | "itoa" | "quicksort"
        | "bubble_sort" | "insertion_sort" | "array_sum" | "array_min" | "array_max"
        | "linear_search" | "binary_search" | "stack_init" | "stack_push" | "stack_pop"
        | "queue_init" | "hashtable_init" | "hashtable_put" | "hashtable_get" => None,

        _ => None,
    }
}

/// Get test cases for stdlib functions.
fn get_stdlib_test_cases() -> HashMap<String, Vec<VerifyTestCase>> {
    let mut cases = HashMap::new();

    // Math functions
    cases.insert(
        "factorial".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![0],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![1],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![5],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![10],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![12],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "fibonacci".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![0],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![1],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![2],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![10],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![20],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "gcd".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![48, 18],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![100, 35],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![17, 13],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![0, 5],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![12, 0],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "lcm".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![4, 6],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![3, 5],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![12, 18],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![0, 5],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "power".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![2, 0],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![2, 10],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![3, 4],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![5, 3],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "is_prime".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![2],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![3],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![4],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![17],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![100],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "isqrt".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![0],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![1],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![4],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![100],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![99],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "abs_i64".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![5],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![0],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![(-5i64) as u64],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "min".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![5, 3],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![3, 5],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![5, 5],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "max".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![5, 3],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![3, 5],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![5, 5],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "div_ceil".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![10, 3],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![9, 3],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![0, 5],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "triangle_number".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![0],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![1],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![5],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![10],
                memory_setup: None,
            },
        ],
    );

    // Bitwise functions
    cases.insert(
        "popcount".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![0],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![1],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![255],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![u64::MAX],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "clz".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![0],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![1],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![u64::MAX],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![1 << 63],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "ctz".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![0],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![1],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![8],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![1 << 63],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "bswap".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![0],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![0x0102030405060708],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "rotl".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![1, 1],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![1, 63],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![0x8000000000000001, 1],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "rotr".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![1, 1],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![2, 1],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![0x8000000000000001, 1],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "is_power_of_2".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![0],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![1],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![2],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![16],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![17],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "next_power_of_2".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![0],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![1],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![5],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![16],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "parity".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![0],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![1],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![3],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![7],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "clear_lowest_bit".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![0],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![1],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![12],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "isolate_lowest_bit".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![0],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![1],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![12],
                memory_setup: None,
            },
        ],
    );

    // Float functions (use bit patterns for well-known values)
    let one_bits = 1.0f64.to_bits();
    let two_bits = 2.0f64.to_bits();
    let three_bits = 3.0f64.to_bits();
    let four_bits = 4.0f64.to_bits();
    let neg_one_bits = (-1.0f64).to_bits();

    cases.insert(
        "fadd".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![one_bits, two_bits],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![0, one_bits],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "fsub".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![three_bits, one_bits],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![one_bits, one_bits],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "fmul".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![two_bits, three_bits],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![one_bits, 0],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "fdiv".to_string(),
        vec![VerifyTestCase {
            inputs: vec![four_bits, two_bits],
            memory_setup: None,
        }],
    );

    cases.insert(
        "fsqrt".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![four_bits],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![one_bits],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "fabs".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![neg_one_bits],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![one_bits],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "fmin".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![one_bits, two_bits],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![three_bits, two_bits],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "fmax".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![one_bits, two_bits],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![three_bits, two_bits],
                memory_setup: None,
            },
        ],
    );

    // Character functions
    cases.insert(
        "is_digit".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![b'0' as u64],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![b'5' as u64],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![b'9' as u64],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![b'a' as u64],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "is_alpha".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![b'a' as u64],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![b'Z' as u64],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![b'0' as u64],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "is_space".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![b' ' as u64],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![b'\t' as u64],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![b'a' as u64],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "to_upper".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![b'a' as u64],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![b'z' as u64],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![b'A' as u64],
                memory_setup: None,
            },
        ],
    );

    cases.insert(
        "to_lower".to_string(),
        vec![
            VerifyTestCase {
                inputs: vec![b'A' as u64],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![b'Z' as u64],
                memory_setup: None,
            },
            VerifyTestCase {
                inputs: vec![b'a' as u64],
                memory_setup: None,
            },
        ],
    );

    cases
}

/// Verification statistics.
#[derive(Debug, Default)]
pub struct VerifyStats {
    pub functions_verified: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub errors: usize,
    pub results: Vec<TestResult>,
}

impl VerifyStats {
    pub fn is_success(&self) -> bool {
        self.failed == 0 && self.errors == 0
    }

    pub fn total_tests(&self) -> usize {
        self.passed + self.failed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_rust_function_factorial() {
        assert_eq!(run_rust_function("factorial", &[5], None), Some(120));
        assert_eq!(run_rust_function("factorial", &[0], None), Some(1));
    }

    #[test]
    fn test_run_rust_function_fibonacci() {
        assert_eq!(run_rust_function("fibonacci", &[10], None), Some(55));
        assert_eq!(run_rust_function("fibonacci", &[0], None), Some(0));
    }

    #[test]
    fn test_run_rust_function_gcd() {
        assert_eq!(run_rust_function("gcd", &[48, 18], None), Some(6));
        assert_eq!(run_rust_function("gcd", &[100, 35], None), Some(5));
    }

    #[test]
    fn test_run_rust_function_popcount() {
        assert_eq!(run_rust_function("popcount", &[0], None), Some(0));
        assert_eq!(run_rust_function("popcount", &[255], None), Some(8));
    }

    #[test]
    fn test_run_rust_function_is_digit() {
        assert_eq!(run_rust_function("is_digit", &[b'5' as u64], None), Some(1));
        assert_eq!(run_rust_function("is_digit", &[b'a' as u64], None), Some(0));
    }
}
