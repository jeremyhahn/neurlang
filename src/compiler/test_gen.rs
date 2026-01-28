//! Test Case Generator
//!
//! Generates @test annotations from doc comments and function signatures.
//! Supports memory setup for string/array tests.

use super::analyzer::AnalyzedParam;
use regex_lite::Regex;

/// Memory setup for a test case (address -> string data)
#[derive(Debug, Clone)]
pub struct MemorySetup {
    /// Address to place data
    pub address: u64,
    /// String data (will be null-terminated)
    pub data: String,
}

/// A test case for a Neurlang function.
#[derive(Debug, Clone)]
pub struct TestCase {
    /// Input values (register assignments)
    pub inputs: Vec<(String, u64)>,
    /// Expected output values (register assignments)
    pub outputs: Vec<(String, u64)>,
    /// Memory setup for string/array tests
    pub memory: Vec<MemorySetup>,
}

/// Base address for test string data
const TEST_STRING_BASE: u64 = 0x1000;

impl TestCase {
    /// Format inputs for @test annotation (includes memory setup).
    pub fn inputs_str(&self) -> String {
        let mut parts = Vec::new();

        // Register assignments
        for (reg, val) in &self.inputs {
            parts.push(format!("{}={}", reg, val));
        }

        // Memory setup: [addr]="string"
        for mem in &self.memory {
            parts.push(format!(
                "[0x{:x}]=\"{}\"",
                mem.address,
                escape_string(&mem.data)
            ));
        }

        parts.join(" ")
    }

    /// Format outputs for @test annotation.
    pub fn outputs_str(&self) -> String {
        self.outputs
            .iter()
            .map(|(reg, val)| format!("{}={}", reg, val))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Escape special characters in a string for annotation output
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
        .replace('\0', "\\0")
}

/// Parsed argument: either a number or a string with its memory address
#[derive(Debug, Clone)]
enum ParsedArg {
    Number(u64),
    String(String, u64),
}

/// Test case generator.
pub struct TestGenerator;

impl TestGenerator {
    /// Generate test cases from doc comments.
    pub fn generate_from_doc(
        func_name: &str,
        doc: Option<&str>,
        params: &[AnalyzedParam],
    ) -> Vec<TestCase> {
        let mut tests = Vec::new();

        if let Some(doc_comment) = doc {
            // Parse test cases from doc comment
            // Format: - func_name(arg1, arg2) = result
            // Or: - func_name(arg1) = result
            tests.extend(Self::parse_doc_tests(func_name, doc_comment, params));
        }

        // If no tests found in docs, generate default tests
        if tests.is_empty() {
            tests.extend(Self::generate_default_tests(func_name, params));
        }

        tests
    }

    /// Parse test cases from doc comment.
    /// Supports both numeric and string arguments:
    /// - func_name(5, 3) = 8
    /// - strlen("hello") = 5
    /// - strcmp("abc", "abd") = 0
    fn parse_doc_tests(func_name: &str, doc: &str, params: &[AnalyzedParam]) -> Vec<TestCase> {
        let mut tests = Vec::new();
        let mut string_addr = TEST_STRING_BASE;

        // Pattern: func_name(...) = result
        // We'll parse the args manually to handle both strings and numbers
        let pattern = format!(
            r"{}[(]([^)]*)[)]\s*=\s*(\d+)",
            regex_lite::escape(func_name)
        );
        let re = Regex::new(&pattern).ok();

        if let Some(regex) = re {
            for line in doc.lines() {
                let line = line.trim();

                // Skip lines that don't look like test cases
                if !line.contains(func_name) || !line.contains('=') {
                    continue;
                }

                if let Some(caps) = regex.captures(line) {
                    let args_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    let result_str = caps.get(2).map(|m| m.as_str()).unwrap_or("0");

                    // Parse arguments (may include strings like "hello" or numbers like 42)
                    let parsed_args = Self::parse_mixed_args(args_str, &mut string_addr);

                    if let Ok(result) = result_str.parse::<u64>() {
                        let mut inputs = Vec::new();
                        let mut memory = Vec::new();

                        for (i, arg) in parsed_args.iter().enumerate() {
                            let reg = if i < params.len() {
                                format!("r{}", params[i].register)
                            } else {
                                format!("r{}", i)
                            };

                            match arg {
                                ParsedArg::Number(n) => {
                                    inputs.push((reg, *n));
                                }
                                ParsedArg::String(s, addr) => {
                                    inputs.push((reg, *addr));
                                    memory.push(MemorySetup {
                                        address: *addr,
                                        data: s.clone(),
                                    });
                                }
                            }
                        }

                        tests.push(TestCase {
                            inputs,
                            outputs: vec![("r0".to_string(), result)],
                            memory,
                        });
                    }
                }
            }
        }

        // Also look for inline test format: - input1, input2 -> output
        let inline_re = Regex::new(r"^\s*[-*]\s*(\d+(?:\s*,\s*\d+)*)\s*->\s*(\d+)").ok();
        if let Some(regex) = inline_re {
            for line in doc.lines() {
                if let Some(caps) = regex.captures(line) {
                    let inputs_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    let output_str = caps.get(2).map(|m| m.as_str()).unwrap_or("0");

                    let args: Vec<u64> = inputs_str
                        .split(',')
                        .filter_map(|s| s.trim().parse().ok())
                        .collect();

                    if let Ok(output) = output_str.parse::<u64>() {
                        let mut inputs = Vec::new();
                        for (i, arg) in args.iter().enumerate() {
                            let reg = format!("r{}", i);
                            inputs.push((reg, *arg));
                        }

                        tests.push(TestCase {
                            inputs,
                            outputs: vec![("r0".to_string(), output)],
                            memory: Vec::new(),
                        });
                    }
                }
            }
        }

        tests
    }

    /// Parse mixed arguments that may include strings ("hello") or numbers (42).
    /// Allocates memory addresses for strings starting from string_addr.
    fn parse_mixed_args(args_str: &str, string_addr: &mut u64) -> Vec<ParsedArg> {
        let mut args = Vec::new();
        let mut chars = args_str.chars().peekable();

        while chars.peek().is_some() {
            // Skip whitespace and commas
            while chars.peek().is_some_and(|c| c.is_whitespace() || *c == ',') {
                chars.next();
            }

            if chars.peek().is_none() {
                break;
            }

            // Check if this is a string argument
            if chars.peek() == Some(&'"') {
                chars.next(); // Skip opening quote
                let mut string_content = String::new();

                while let Some(&c) = chars.peek() {
                    if c == '"' {
                        chars.next(); // Skip closing quote
                        break;
                    }
                    if c == '\\' {
                        chars.next();
                        if let Some(&escaped) = chars.peek() {
                            match escaped {
                                'n' => string_content.push('\n'),
                                't' => string_content.push('\t'),
                                '0' => string_content.push('\0'),
                                '\\' => string_content.push('\\'),
                                '"' => string_content.push('"'),
                                _ => string_content.push(escaped),
                            }
                            chars.next();
                        }
                    } else {
                        string_content.push(c);
                        chars.next();
                    }
                }

                // Allocate address for this string
                let addr = *string_addr;
                // Align to 8 bytes and add space for string + null terminator
                *string_addr += (string_content.len() as u64 + 1).div_ceil(8) * 8;

                args.push(ParsedArg::String(string_content, addr));
            } else {
                // Numeric argument
                let mut num_str = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit()
                        || c == '-'
                        || c == 'x'
                        || c == 'X'
                        || ('a'..='f').contains(&c)
                        || ('A'..='F').contains(&c)
                    {
                        num_str.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }

                if !num_str.is_empty() {
                    let value = if num_str.starts_with("0x") || num_str.starts_with("0X") {
                        u64::from_str_radix(&num_str[2..], 16).unwrap_or(0)
                    } else if num_str.starts_with('-') {
                        num_str.parse::<i64>().unwrap_or(0) as u64
                    } else {
                        num_str.parse::<u64>().unwrap_or(0)
                    };
                    args.push(ParsedArg::Number(value));
                }
            }
        }

        args
    }

    /// Generate default test cases based on function name and parameters.
    fn generate_default_tests(func_name: &str, params: &[AnalyzedParam]) -> Vec<TestCase> {
        let mut tests = Vec::new();

        // Generate test cases based on common function patterns
        match func_name {
            "factorial" => {
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 0)],
                    outputs: vec![("r0".to_string(), 1)],
                    memory: Vec::new(),
                });
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 5)],
                    outputs: vec![("r0".to_string(), 120)],
                    memory: Vec::new(),
                });
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 10)],
                    outputs: vec![("r0".to_string(), 3628800)],
                    memory: Vec::new(),
                });
            }
            "fibonacci" => {
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 0)],
                    outputs: vec![("r0".to_string(), 0)],
                    memory: Vec::new(),
                });
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 1)],
                    outputs: vec![("r0".to_string(), 1)],
                    memory: Vec::new(),
                });
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 10)],
                    outputs: vec![("r0".to_string(), 55)],
                    memory: Vec::new(),
                });
            }
            "gcd" => {
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 48), ("r1".to_string(), 18)],
                    outputs: vec![("r0".to_string(), 6)],
                    memory: Vec::new(),
                });
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 100), ("r1".to_string(), 35)],
                    outputs: vec![("r0".to_string(), 5)],
                    memory: Vec::new(),
                });
            }
            "lcm" => {
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 4), ("r1".to_string(), 6)],
                    outputs: vec![("r0".to_string(), 12)],
                    memory: Vec::new(),
                });
            }
            "power" => {
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 2), ("r1".to_string(), 10)],
                    outputs: vec![("r0".to_string(), 1024)],
                    memory: Vec::new(),
                });
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 2), ("r1".to_string(), 0)],
                    outputs: vec![("r0".to_string(), 1)],
                    memory: Vec::new(),
                });
            }
            "is_prime" => {
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 2)],
                    outputs: vec![("r0".to_string(), 1)],
                    memory: Vec::new(),
                });
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 4)],
                    outputs: vec![("r0".to_string(), 0)],
                    memory: Vec::new(),
                });
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 17)],
                    outputs: vec![("r0".to_string(), 1)],
                    memory: Vec::new(),
                });
            }
            "min" | "fmin" => {
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 5), ("r1".to_string(), 3)],
                    outputs: vec![("r0".to_string(), 3)],
                    memory: Vec::new(),
                });
            }
            "max" | "fmax" => {
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 5), ("r1".to_string(), 3)],
                    outputs: vec![("r0".to_string(), 5)],
                    memory: Vec::new(),
                });
            }
            "abs_i64" | "fabs" => {
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 5)],
                    outputs: vec![("r0".to_string(), 5)],
                    memory: Vec::new(),
                });
            }
            "isqrt" => {
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 100)],
                    outputs: vec![("r0".to_string(), 10)],
                    memory: Vec::new(),
                });
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 0)],
                    outputs: vec![("r0".to_string(), 0)],
                    memory: Vec::new(),
                });
            }
            "popcount" => {
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 0)],
                    outputs: vec![("r0".to_string(), 0)],
                    memory: Vec::new(),
                });
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 255)],
                    outputs: vec![("r0".to_string(), 8)],
                    memory: Vec::new(),
                });
            }
            "clz" => {
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 1)],
                    outputs: vec![("r0".to_string(), 63)],
                    memory: Vec::new(),
                });
            }
            "ctz" => {
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 8)],
                    outputs: vec![("r0".to_string(), 3)],
                    memory: Vec::new(),
                });
            }
            "is_power_of_2" => {
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 16)],
                    outputs: vec![("r0".to_string(), 1)],
                    memory: Vec::new(),
                });
                tests.push(TestCase {
                    inputs: vec![("r0".to_string(), 17)],
                    outputs: vec![("r0".to_string(), 0)],
                    memory: Vec::new(),
                });
            }
            _ => {
                // Generate simple identity test for single-param functions
                if params.len() == 1 {
                    tests.push(TestCase {
                        inputs: vec![("r0".to_string(), 0)],
                        outputs: vec![("r0".to_string(), 0)],
                        memory: Vec::new(),
                    });
                } else if params.len() == 2 {
                    // For two-param functions, test with zeros
                    tests.push(TestCase {
                        inputs: vec![("r0".to_string(), 0), ("r1".to_string(), 0)],
                        outputs: vec![("r0".to_string(), 0)],
                        memory: Vec::new(),
                    });
                }
            }
        }

        tests
    }

    /// Run a Rust function with given inputs and capture output.
    /// This is used for verification.
    pub fn run_rust_function<F>(func: F, inputs: &[u64]) -> u64
    where
        F: Fn(&[u64]) -> u64,
    {
        func(inputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_doc_tests() {
        let doc = r#"
            Calculate factorial of n.

            # Test Cases
            - factorial(0) = 1
            - factorial(5) = 120
            - factorial(10) = 3628800
        "#;

        let tests = TestGenerator::parse_doc_tests("factorial", doc, &[]);
        assert!(!tests.is_empty());
    }

    #[test]
    fn test_parse_string_tests() {
        let doc = r#"
            Calculate string length.

            # Test Cases
            - strlen("hello") = 5
            - strlen("") = 0
            - strlen("a") = 1
        "#;

        let tests = TestGenerator::parse_doc_tests("strlen", doc, &[]);
        assert_eq!(tests.len(), 3, "Expected 3 test cases");

        // First test: strlen("hello") = 5
        assert_eq!(tests[0].outputs[0].1, 5);
        assert!(!tests[0].memory.is_empty(), "Should have memory setup");
        assert_eq!(tests[0].memory[0].data, "hello");
    }

    #[test]
    fn test_default_tests() {
        let tests = TestGenerator::generate_default_tests("factorial", &[]);
        assert!(!tests.is_empty());

        // Should have test for factorial(5) = 120
        assert!(tests
            .iter()
            .any(|t| t.inputs.iter().any(|(_, v)| *v == 5)
                && t.outputs.iter().any(|(_, v)| *v == 120)));
    }
}
