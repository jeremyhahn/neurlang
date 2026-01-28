//! Fast Tokenizer for Multi-Head Prediction
//!
//! Provides character-level tokenization and number extraction for the
//! multi-head prediction pipeline.

use std::sync::LazyLock;

/// Maximum sequence length
pub const MAX_SEQ_LEN: usize = 128;

/// Vocabulary size (256 bytes + special tokens)
pub const VOCAB_SIZE: usize = 261;

/// Special token IDs
pub const PAD_TOKEN: u32 = 256;
pub const UNK_TOKEN: u32 = 257;
pub const BOS_TOKEN: u32 = 258;
pub const EOS_TOKEN: u32 = 259;
pub const SEP_TOKEN: u32 = 260;

/// Fast character-level tokenizer
///
/// Uses direct byte mapping for O(1) tokenization.
#[derive(Debug, Clone)]
pub struct FastTokenizer {
    max_len: usize,
}

impl FastTokenizer {
    /// Create a new tokenizer with default max length
    pub fn new() -> Self {
        Self {
            max_len: MAX_SEQ_LEN,
        }
    }

    /// Create a new tokenizer with custom max length
    pub fn with_max_len(max_len: usize) -> Self {
        Self { max_len }
    }

    /// Encode text to token IDs
    ///
    /// Returns a fixed-length vector padded to max_len.
    pub fn encode(&self, text: &str) -> Vec<i64> {
        let mut tokens = Vec::with_capacity(self.max_len);

        // Add BOS token
        tokens.push(BOS_TOKEN as i64);

        // Convert each byte to token ID
        for byte in text.bytes().take(self.max_len - 2) {
            tokens.push(byte as i64);
        }

        // Add EOS token
        tokens.push(EOS_TOKEN as i64);

        // Pad to max length
        while tokens.len() < self.max_len {
            tokens.push(PAD_TOKEN as i64);
        }

        tokens
    }

    /// Encode text without padding (variable length output)
    pub fn encode_no_pad(&self, text: &str) -> Vec<i64> {
        let mut tokens = Vec::with_capacity(text.len() + 2);

        tokens.push(BOS_TOKEN as i64);

        for byte in text.bytes() {
            tokens.push(byte as i64);
        }

        tokens.push(EOS_TOKEN as i64);

        tokens
    }

    /// Decode token IDs back to text
    pub fn decode(&self, tokens: &[i64]) -> String {
        let bytes: Vec<u8> = tokens
            .iter()
            .filter(|&&t| (0..256).contains(&t))
            .map(|&t| t as u8)
            .collect();

        String::from_utf8_lossy(&bytes).to_string()
    }
}

impl Default for FastTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract all numbers from text
///
/// Returns numbers in order of appearance, handling:
/// - Positive integers: "5", "123"
/// - Negative integers: "-5", "-123"
/// - Numbers with separators: "1,000" → 1000
/// - Scientific notation (basic): "1e3" → 1000
pub fn extract_numbers(text: &str) -> Vec<i64> {
    let mut numbers = Vec::new();
    let mut current = String::new();
    let mut in_number = false;
    let mut is_negative = false;

    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    for i in 0..len {
        let c = chars[i];

        if c.is_ascii_digit() {
            if !in_number && i > 0 && chars[i - 1] == '-' {
                // Check if the minus is actually a negative sign
                // (preceded by space, paren, or operator)
                if i >= 2 {
                    let prev = chars[i - 2];
                    if prev.is_whitespace() || prev == '(' || prev == ',' || prev == '[' {
                        is_negative = true;
                    }
                } else {
                    is_negative = true;
                }
            }
            current.push(c);
            in_number = true;
        } else if c == ',' && in_number {
            // Skip comma separators in numbers like "1,000"
            continue;
        } else if c == 'e' || c == 'E' {
            if in_number && i + 1 < len && chars[i + 1].is_ascii_digit() {
                // Start of exponent
                current.push(c);
            } else if in_number {
                // End of number
                if let Ok(n) = parse_number(&current, is_negative) {
                    numbers.push(n);
                }
                current.clear();
                in_number = false;
                is_negative = false;
            }
        } else {
            if in_number && !current.is_empty() {
                if let Ok(n) = parse_number(&current, is_negative) {
                    numbers.push(n);
                }
                current.clear();
                is_negative = false;
            }
            in_number = false;
        }
    }

    // Handle last number
    if !current.is_empty() {
        if let Ok(n) = parse_number(&current, is_negative) {
            numbers.push(n);
        }
    }

    numbers
}

/// Parse a number string, handling scientific notation
fn parse_number(s: &str, negative: bool) -> Result<i64, std::num::ParseIntError> {
    let result = if s.contains('e') || s.contains('E') {
        // Handle scientific notation
        let parts: Vec<&str> = s.split(['e', 'E']).collect();
        if parts.len() == 2 {
            let base: f64 = parts[0].parse().unwrap_or(0.0);
            let exp: i32 = parts[1].parse().unwrap_or(0);
            (base * 10f64.powi(exp)) as i64
        } else {
            s.parse::<i64>()?
        }
    } else {
        s.parse::<i64>()?
    };

    Ok(if negative { -result } else { result })
}

/// Extract the first N numbers from text
pub fn extract_first_n_numbers(text: &str, n: usize) -> Vec<i64> {
    extract_numbers(text).into_iter().take(n).collect()
}

/// Pattern matchers for common expressions
static SYMBOLIC_PATTERNS: LazyLock<Vec<(regex_lite::Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        // Binary operators
        (regex_lite::Regex::new(r"(\d+)\s*\+\s*(\d+)").unwrap(), "+"),
        (regex_lite::Regex::new(r"(\d+)\s*-\s*(\d+)").unwrap(), "-"),
        (regex_lite::Regex::new(r"(\d+)\s*\*\s*(\d+)").unwrap(), "*"),
        (regex_lite::Regex::new(r"(\d+)\s*/\s*(\d+)").unwrap(), "/"),
        (regex_lite::Regex::new(r"(\d+)\s*%\s*(\d+)").unwrap(), "%"),
        (regex_lite::Regex::new(r"(\d+)\s*\^\s*(\d+)").unwrap(), "^"),
        (
            regex_lite::Regex::new(r"(\d+)\s*\*\*\s*(\d+)").unwrap(),
            "^",
        ),
        // Factorial
        (regex_lite::Regex::new(r"(\d+)!").unwrap(), "!"),
        // Function calls
        (
            regex_lite::Regex::new(r"(?i)fib(?:onacci)?\s*\(\s*(\d+)\s*\)").unwrap(),
            "fib",
        ),
        (
            regex_lite::Regex::new(r"(?i)fact(?:orial)?\s*\(\s*(\d+)\s*\)").unwrap(),
            "fact",
        ),
        (
            regex_lite::Regex::new(r"(?i)gcd\s*\(\s*(\d+)\s*,\s*(\d+)\s*\)").unwrap(),
            "gcd",
        ),
        (
            regex_lite::Regex::new(r"(?i)lcm\s*\(\s*(\d+)\s*,\s*(\d+)\s*\)").unwrap(),
            "lcm",
        ),
        (
            regex_lite::Regex::new(r"(?i)max\s*\(\s*(\d+)\s*,\s*(\d+)\s*\)").unwrap(),
            "max",
        ),
        (
            regex_lite::Regex::new(r"(?i)min\s*\(\s*(\d+)\s*,\s*(\d+)\s*\)").unwrap(),
            "min",
        ),
        (
            regex_lite::Regex::new(r"(?i)pow\s*\(\s*(\d+)\s*,\s*(\d+)\s*\)").unwrap(),
            "pow",
        ),
        (
            regex_lite::Regex::new(r"(?i)sqrt\s*\(\s*(\d+)\s*\)").unwrap(),
            "sqrt",
        ),
        (
            regex_lite::Regex::new(r"(?i)abs\s*\(\s*-?(\d+)\s*\)").unwrap(),
            "abs",
        ),
    ]
});

/// Symbolic expression detection result
#[derive(Debug, Clone)]
pub struct SymbolicExpr {
    /// Operator or function name
    pub op: String,
    /// Operands
    pub operands: Vec<i64>,
    /// Intent ID (if known)
    pub intent_id: Option<usize>,
}

/// Parse symbolic expression from text
pub fn parse_symbolic_expression(text: &str) -> Option<SymbolicExpr> {
    for (pattern, op) in SYMBOLIC_PATTERNS.iter() {
        if let Some(caps) = pattern.captures(text) {
            let mut operands = Vec::new();

            for i in 1..caps.len() {
                if let Some(m) = caps.get(i) {
                    if let Ok(n) = m.as_str().parse::<i64>() {
                        operands.push(n);
                    }
                }
            }

            if !operands.is_empty() {
                let intent_id = match *op {
                    "+" => Some(0),
                    "-" => Some(1),
                    "*" => Some(2),
                    "/" => Some(3),
                    "%" => Some(4),
                    "^" => Some(13),
                    "!" | "fact" => Some(11),
                    "fib" => Some(12),
                    "gcd" => Some(15),
                    "lcm" => Some(16),
                    "max" => Some(19),
                    "min" => Some(20),
                    "pow" => Some(13),
                    "sqrt" => Some(14),
                    "abs" => Some(17),
                    _ => None,
                };

                return Some(SymbolicExpr {
                    op: op.to_string(),
                    operands,
                    intent_id,
                });
            }
        }
    }

    None
}

/// Word-level features for intent classification
#[derive(Debug, Clone)]
pub struct TextFeatures {
    /// Contains arithmetic keywords
    pub has_arithmetic: bool,
    /// Contains math function keywords
    pub has_math_func: bool,
    /// Contains comparison keywords
    pub has_comparison: bool,
    /// Contains bitwise keywords
    pub has_bitwise: bool,
    /// Contains I/O keywords
    pub has_io: bool,
    /// Number count
    pub number_count: usize,
    /// First few numbers
    pub numbers: Vec<i64>,
}

/// Extract features from text for intent classification
pub fn extract_features(text: &str) -> TextFeatures {
    let text_lower = text.to_lowercase();
    let numbers = extract_numbers(text);

    TextFeatures {
        has_arithmetic: text_lower.contains("add")
            || text_lower.contains("sum")
            || text_lower.contains("plus")
            || text_lower.contains("subtract")
            || text_lower.contains("minus")
            || text_lower.contains("multiply")
            || text_lower.contains("divide")
            || text_lower.contains("mod"),
        has_math_func: text_lower.contains("factorial")
            || text_lower.contains("fibonacci")
            || text_lower.contains("power")
            || text_lower.contains("sqrt")
            || text_lower.contains("gcd")
            || text_lower.contains("lcm"),
        has_comparison: text_lower.contains("max")
            || text_lower.contains("min")
            || text_lower.contains("greater")
            || text_lower.contains("less"),
        has_bitwise: text_lower.contains("and")
            || text_lower.contains("or")
            || text_lower.contains("xor")
            || text_lower.contains("shift"),
        has_io: text_lower.contains("print")
            || text_lower.contains("read")
            || text_lower.contains("time")
            || text_lower.contains("random"),
        number_count: numbers.len(),
        numbers: numbers.into_iter().take(4).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_encode() {
        let tokenizer = FastTokenizer::new();
        let tokens = tokenizer.encode("hello");

        assert_eq!(tokens.len(), MAX_SEQ_LEN);
        assert_eq!(tokens[0], BOS_TOKEN as i64);
        assert_eq!(tokens[1], b'h' as i64);
        assert_eq!(tokens[2], b'e' as i64);
        assert_eq!(tokens[3], b'l' as i64);
        assert_eq!(tokens[4], b'l' as i64);
        assert_eq!(tokens[5], b'o' as i64);
        assert_eq!(tokens[6], EOS_TOKEN as i64);
        assert_eq!(tokens[7], PAD_TOKEN as i64);
    }

    #[test]
    fn test_tokenizer_decode() {
        let tokenizer = FastTokenizer::new();
        let tokens = vec![104, 101, 108, 108, 111]; // "hello" as bytes
        let text = tokenizer.decode(&tokens);
        assert_eq!(text, "hello");
    }

    #[test]
    fn test_extract_numbers() {
        assert_eq!(extract_numbers("add 5 and 3"), vec![5, 3]);
        assert_eq!(extract_numbers("5 + 3"), vec![5, 3]);
        assert_eq!(extract_numbers("factorial of 10"), vec![10]);
        assert_eq!(extract_numbers("gcd(48, 18)"), vec![48, 18]);
        assert_eq!(extract_numbers("no numbers here"), Vec::<i64>::new());
        assert_eq!(extract_numbers("-5 + 3"), vec![-5, 3]); // Negative at start is correctly detected
    }

    #[test]
    fn test_extract_numbers_with_commas() {
        assert_eq!(extract_numbers("1,000"), vec![1000]);
        assert_eq!(extract_numbers("1,000,000"), vec![1000000]);
    }

    #[test]
    fn test_symbolic_expression_parsing() {
        let expr = parse_symbolic_expression("5 + 3").unwrap();
        assert_eq!(expr.op, "+");
        assert_eq!(expr.operands, vec![5, 3]);
        assert_eq!(expr.intent_id, Some(0));

        let expr = parse_symbolic_expression("10!").unwrap();
        assert_eq!(expr.op, "!");
        assert_eq!(expr.operands, vec![10]);
        assert_eq!(expr.intent_id, Some(11));

        let expr = parse_symbolic_expression("fibonacci(20)").unwrap();
        assert_eq!(expr.op, "fib");
        assert_eq!(expr.operands, vec![20]);
        assert_eq!(expr.intent_id, Some(12));

        let expr = parse_symbolic_expression("gcd(48, 18)").unwrap();
        assert_eq!(expr.op, "gcd");
        assert_eq!(expr.operands, vec![48, 18]);
        assert_eq!(expr.intent_id, Some(15));
    }

    #[test]
    fn test_extract_features() {
        let features = extract_features("add 5 and 3");
        assert!(features.has_arithmetic);
        assert!(!features.has_math_func);
        assert_eq!(features.number_count, 2);
        assert_eq!(features.numbers, vec![5, 3]);

        let features = extract_features("compute factorial of 10");
        assert!(!features.has_arithmetic);
        assert!(features.has_math_func);
        assert_eq!(features.number_count, 1);
    }
}
