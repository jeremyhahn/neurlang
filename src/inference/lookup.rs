//! Intent ID Lookup and Mapping
//!
//! This module provides bidirectional mapping between intent names and IDs,
//! supporting the multi-head prediction architecture.

use std::collections::HashMap;
use std::sync::LazyLock;

/// Intent name to ID mapping
pub static INTENT_MAP: LazyLock<HashMap<&'static str, usize>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    // Arithmetic (0-10)
    map.insert("ADD", 0);
    map.insert("SUB", 1);
    map.insert("MUL", 2);
    map.insert("DIV", 3);
    map.insert("MOD", 4);
    map.insert("AND", 5);
    map.insert("OR", 6);
    map.insert("XOR", 7);
    map.insert("SHL", 8);
    map.insert("SHR", 9);
    map.insert("SAR", 10);

    // Math Functions (11-18)
    map.insert("FACTORIAL", 11);
    map.insert("FIBONACCI", 12);
    map.insert("POWER", 13);
    map.insert("SQRT", 14);
    map.insert("GCD", 15);
    map.insert("LCM", 16);
    map.insert("ABS", 17);
    map.insert("CLAMP", 18);

    // Comparisons (19-24)
    map.insert("MAX", 19);
    map.insert("MIN", 20);
    map.insert("SIGN", 21);
    map.insert("IS_POSITIVE", 22);
    map.insert("IS_EVEN", 23);
    map.insert("IS_PRIME", 24);

    // Bit Operations (25-29)
    map.insert("POPCOUNT", 25);
    map.insert("CLZ", 26);
    map.insert("CTZ", 27);
    map.insert("BSWAP", 28);
    map.insert("NEXTPOW2", 29);

    // Memory (30-33)
    map.insert("MEMCPY", 30);
    map.insert("MEMSET", 31);
    map.insert("MEMCMP", 32);
    map.insert("ARRAY_SUM", 33);

    // Strings (34-37)
    map.insert("STRLEN", 34);
    map.insert("STRCMP", 35);
    map.insert("STRCPY", 36);
    map.insert("HASH_STRING", 37);

    // I/O (38-42)
    map.insert("PRINT", 38);
    map.insert("READ_LINE", 39);
    map.insert("TIME_NOW", 40);
    map.insert("SLEEP", 41);
    map.insert("RANDOM", 42);

    // Crypto (43-47)
    map.insert("SHA256", 43);
    map.insert("AES_ENCRYPT", 44);
    map.insert("AES_DECRYPT", 45);
    map.insert("HMAC", 46);
    map.insert("SECURE_RANDOM", 47);

    // Loops (48-50)
    map.insert("LOOP_COUNT", 48);
    map.insert("LOOP_SUM", 49);
    map.insert("COUNTDOWN", 50);

    // Floating Point (51-53)
    map.insert("FADD", 51);
    map.insert("FMUL", 52);
    map.insert("FDIV", 53);

    map
});

/// Intent ID to name mapping
pub static INTENT_NAMES: LazyLock<[&'static str; 54]> = LazyLock::new(|| {
    [
        // Arithmetic (0-10)
        "ADD",
        "SUB",
        "MUL",
        "DIV",
        "MOD",
        "AND",
        "OR",
        "XOR",
        "SHL",
        "SHR",
        "SAR",
        // Math Functions (11-18)
        "FACTORIAL",
        "FIBONACCI",
        "POWER",
        "SQRT",
        "GCD",
        "LCM",
        "ABS",
        "CLAMP",
        // Comparisons (19-24)
        "MAX",
        "MIN",
        "SIGN",
        "IS_POSITIVE",
        "IS_EVEN",
        "IS_PRIME",
        // Bit Operations (25-29)
        "POPCOUNT",
        "CLZ",
        "CTZ",
        "BSWAP",
        "NEXTPOW2",
        // Memory (30-33)
        "MEMCPY",
        "MEMSET",
        "MEMCMP",
        "ARRAY_SUM",
        // Strings (34-37)
        "STRLEN",
        "STRCMP",
        "STRCPY",
        "HASH_STRING",
        // I/O (38-42)
        "PRINT",
        "READ_LINE",
        "TIME_NOW",
        "SLEEP",
        "RANDOM",
        // Crypto (43-47)
        "SHA256",
        "AES_ENCRYPT",
        "AES_DECRYPT",
        "HMAC",
        "SECURE_RANDOM",
        // Loops (48-50)
        "LOOP_COUNT",
        "LOOP_SUM",
        "COUNTDOWN",
        // Floating Point (51-53)
        "FADD",
        "FMUL",
        "FDIV",
    ]
});

/// Intent categories for grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentCategory {
    Arithmetic,
    MathFunction,
    Comparison,
    BitOperation,
    Memory,
    String,
    Io,
    Crypto,
    Loop,
    FloatingPoint,
}

impl IntentCategory {
    /// Get the category for a given intent ID
    pub fn from_intent_id(intent_id: usize) -> Option<Self> {
        match intent_id {
            0..=10 => Some(IntentCategory::Arithmetic),
            11..=18 => Some(IntentCategory::MathFunction),
            19..=24 => Some(IntentCategory::Comparison),
            25..=29 => Some(IntentCategory::BitOperation),
            30..=33 => Some(IntentCategory::Memory),
            34..=37 => Some(IntentCategory::String),
            38..=42 => Some(IntentCategory::Io),
            43..=47 => Some(IntentCategory::Crypto),
            48..=50 => Some(IntentCategory::Loop),
            51..=53 => Some(IntentCategory::FloatingPoint),
            _ => None,
        }
    }

    /// Get all intent IDs in this category
    pub fn intent_range(&self) -> std::ops::Range<usize> {
        match self {
            IntentCategory::Arithmetic => 0..11,
            IntentCategory::MathFunction => 11..19,
            IntentCategory::Comparison => 19..25,
            IntentCategory::BitOperation => 25..30,
            IntentCategory::Memory => 30..34,
            IntentCategory::String => 34..38,
            IntentCategory::Io => 38..43,
            IntentCategory::Crypto => 43..48,
            IntentCategory::Loop => 48..51,
            IntentCategory::FloatingPoint => 51..54,
        }
    }
}

/// Get intent ID from name
pub fn intent_id_from_name(name: &str) -> Option<usize> {
    INTENT_MAP.get(name.to_uppercase().as_str()).copied()
}

/// Get intent name from ID
pub fn intent_name_from_id(id: usize) -> Option<&'static str> {
    INTENT_NAMES.get(id).copied()
}

/// Operand count for each intent
pub static OPERAND_COUNTS: LazyLock<[usize; 54]> = LazyLock::new(|| {
    [
        // Arithmetic (0-10): all take 2 operands
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // Math Functions (11-18)
        1, // FACTORIAL: n
        1, // FIBONACCI: n
        2, // POWER: base, exp
        1, // SQRT: n
        2, // GCD: a, b
        2, // LCM: a, b
        1, // ABS: n
        3, // CLAMP: value, min, max
        // Comparisons (19-24)
        2, // MAX: a, b
        2, // MIN: a, b
        1, // SIGN: n
        1, // IS_POSITIVE: n
        1, // IS_EVEN: n
        1, // IS_PRIME: n
        // Bit Operations (25-29)
        1, // POPCOUNT: n
        1, // CLZ: n
        1, // CTZ: n
        1, // BSWAP: n
        1, // NEXTPOW2: n
        // Memory (30-33)
        3, // MEMCPY: dst, src, len
        3, // MEMSET: dst, val, len
        3, // MEMCMP: a, b, len
        2, // ARRAY_SUM: ptr, len
        // Strings (34-37)
        1, // STRLEN: ptr
        2, // STRCMP: a, b
        2, // STRCPY: dst, src
        1, // HASH_STRING: ptr
        // I/O (38-42)
        1, // PRINT: value
        0, // READ_LINE
        0, // TIME_NOW
        1, // SLEEP: ms
        0, // RANDOM
        // Crypto (43-47)
        1, // SHA256: ptr
        2, // AES_ENCRYPT: data, key
        2, // AES_DECRYPT: data, key
        2, // HMAC: data, key
        0, // SECURE_RANDOM
        // Loops (48-50)
        1, // LOOP_COUNT: n
        1, // LOOP_SUM: n
        1, // COUNTDOWN: n
        // Floating Point (51-53)
        2, // FADD: a, b
        2, // FMUL: a, b
        2, // FDIV: a, b
    ]
});

/// Get operand count for intent
pub fn operand_count(intent_id: usize) -> Option<usize> {
    OPERAND_COUNTS.get(intent_id).copied()
}

/// Intent keywords for pattern matching (used in fallback detection)
pub static INTENT_KEYWORDS: LazyLock<HashMap<&'static str, usize>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    // ADD variants
    map.insert("add", 0);
    map.insert("sum", 0);
    map.insert("plus", 0);
    map.insert("+", 0);

    // SUB variants
    map.insert("subtract", 1);
    map.insert("minus", 1);
    map.insert("difference", 1);
    map.insert("-", 1);

    // MUL variants
    map.insert("multiply", 2);
    map.insert("product", 2);
    map.insert("times", 2);
    map.insert("*", 2);

    // DIV variants
    map.insert("divide", 3);
    map.insert("division", 3);
    map.insert("quotient", 3);
    map.insert("/", 3);

    // MOD variants
    map.insert("modulo", 4);
    map.insert("mod", 4);
    map.insert("remainder", 4);
    map.insert("%", 4);

    // Bitwise
    map.insert("and", 5);
    map.insert("or", 6);
    map.insert("xor", 7);
    map.insert("shift left", 8);
    map.insert("shl", 8);
    map.insert("shift right", 9);
    map.insert("shr", 9);
    map.insert("arithmetic shift", 10);
    map.insert("sar", 10);

    // Math functions
    map.insert("factorial", 11);
    map.insert("fact", 11);
    map.insert("!", 11);
    map.insert("fibonacci", 12);
    map.insert("fib", 12);
    map.insert("power", 13);
    map.insert("pow", 13);
    map.insert("exponent", 13);
    map.insert("^", 13);
    map.insert("**", 13);
    map.insert("sqrt", 14);
    map.insert("square root", 14);
    map.insert("gcd", 15);
    map.insert("greatest common divisor", 15);
    map.insert("lcm", 16);
    map.insert("least common multiple", 16);
    map.insert("abs", 17);
    map.insert("absolute", 17);
    map.insert("clamp", 18);

    // Comparisons
    map.insert("max", 19);
    map.insert("maximum", 19);
    map.insert("larger", 19);
    map.insert("min", 20);
    map.insert("minimum", 20);
    map.insert("smaller", 20);
    map.insert("sign", 21);
    map.insert("positive", 22);
    map.insert("even", 23);
    map.insert("prime", 24);

    // Bit operations
    map.insert("popcount", 25);
    map.insert("count bits", 25);
    map.insert("set bits", 25);
    map.insert("clz", 26);
    map.insert("leading zeros", 26);
    map.insert("ctz", 27);
    map.insert("trailing zeros", 27);
    map.insert("bswap", 28);
    map.insert("byte swap", 28);
    map.insert("endian", 28);
    map.insert("next power", 29);
    map.insert("nextpow2", 29);

    // I/O
    map.insert("print", 38);
    map.insert("output", 38);
    map.insert("echo", 38);
    map.insert("read", 39);
    map.insert("input", 39);
    map.insert("time", 40);
    map.insert("timestamp", 40);
    map.insert("now", 40);
    map.insert("sleep", 41);
    map.insert("wait", 41);
    map.insert("delay", 41);
    map.insert("random", 42);
    map.insert("rand", 42);

    // Loops
    map.insert("count", 48);
    map.insert("loop", 48);
    map.insert("countdown", 50);

    // Floating point
    map.insert("fadd", 51);
    map.insert("fmul", 52);
    map.insert("fdiv", 53);

    map
});

/// Try to detect intent from text using keyword matching
/// Returns (intent_id, confidence) or None
pub fn detect_intent_from_keywords(text: &str) -> Option<(usize, f32)> {
    let text_lower = text.to_lowercase();

    // Sort keywords by length (longest first) to prioritize more specific matches
    // This prevents "or" from matching before "factorial"
    let mut keywords: Vec<_> = INTENT_KEYWORDS.iter().collect();
    keywords.sort_by(|a, b| {
        // First by length (descending), then by name (for determinism with same-length keywords)
        b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(b.0))
    });

    // Check matches (longer keywords first for higher specificity)
    // Use word boundary matching for short keywords to avoid false positives
    for (keyword, &intent_id) in keywords {
        // For short keywords that could be substrings (like "or", "and"), use word boundaries
        if keyword.len() <= 3 && keyword.chars().all(|c| c.is_alphabetic()) {
            // Check for word boundary match
            let pattern = format!(r"\b{}\b", regex_lite::escape(keyword));
            if let Ok(re) = regex_lite::Regex::new(&pattern) {
                if re.is_match(&text_lower) {
                    return Some((intent_id, 0.8));
                }
            }
        } else if text_lower.contains(keyword) {
            return Some((intent_id, 0.8));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_map_consistency() {
        // Verify all names map correctly
        for (id, &name) in INTENT_NAMES.iter().enumerate() {
            assert_eq!(
                INTENT_MAP.get(name),
                Some(&id),
                "Name {} should map to ID {}",
                name,
                id
            );
        }
    }

    #[test]
    fn test_intent_id_from_name() {
        assert_eq!(intent_id_from_name("ADD"), Some(0));
        assert_eq!(intent_id_from_name("add"), Some(0)); // Case insensitive
        assert_eq!(intent_id_from_name("FACTORIAL"), Some(11));
        assert_eq!(intent_id_from_name("FIBONACCI"), Some(12));
        assert_eq!(intent_id_from_name("UNKNOWN"), None);
    }

    #[test]
    fn test_intent_name_from_id() {
        assert_eq!(intent_name_from_id(0), Some("ADD"));
        assert_eq!(intent_name_from_id(11), Some("FACTORIAL"));
        assert_eq!(intent_name_from_id(54), None);
    }

    #[test]
    fn test_intent_category() {
        assert_eq!(
            IntentCategory::from_intent_id(0),
            Some(IntentCategory::Arithmetic)
        );
        assert_eq!(
            IntentCategory::from_intent_id(11),
            Some(IntentCategory::MathFunction)
        );
        assert_eq!(
            IntentCategory::from_intent_id(25),
            Some(IntentCategory::BitOperation)
        );
        assert_eq!(IntentCategory::from_intent_id(100), None);
    }

    #[test]
    fn test_operand_count() {
        assert_eq!(operand_count(0), Some(2)); // ADD
        assert_eq!(operand_count(11), Some(1)); // FACTORIAL
        assert_eq!(operand_count(18), Some(3)); // CLAMP
        assert_eq!(operand_count(40), Some(0)); // TIME_NOW
        assert_eq!(operand_count(100), None);
    }

    #[test]
    fn test_keyword_detection() {
        assert_eq!(detect_intent_from_keywords("add 5 and 3"), Some((0, 0.8)));
        assert_eq!(
            detect_intent_from_keywords("factorial of 10"),
            Some((11, 0.8))
        );
        assert_eq!(
            detect_intent_from_keywords("compute fibonacci(20)"),
            Some((12, 0.8))
        );
    }
}
