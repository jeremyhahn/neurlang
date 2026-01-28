//! Slot Type Definitions
//!
//! Defines the 20 slot types that combine to build any protocol implementation.
//!
//! # Categories
//!
//! | Category | Types | Instructions/Slot |
//! |----------|-------|-------------------|
//! | String/Pattern | 5 | 50-150 |
//! | Numeric | 3 | 20-50 |
//! | Control Flow | 3 | 20-50 |
//! | I/O | 3 | 30-80 |
//! | Extension | 2 | 10-30 |
//! | Error | 1 | 20-40 |
//! | Data | 2 | 10-30 |

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The 20 slot types that combine to build ANY protocol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum SlotType {
    // ═══════════════════════════════════════════════════════════════
    // STRING/PATTERN OPERATIONS
    // ═══════════════════════════════════════════════════════════════
    /// Match input buffer against a pattern, extract captures
    /// Example: "HELO {domain}" matches "HELO test.com\r\n"
    PatternMatch {
        /// Pattern with {capture} placeholders
        pattern: String,
        /// Register containing input buffer
        input_reg: String,
        /// Where to store extracted values
        captures: Vec<Capture>,
        /// Jump here on match
        match_label: String,
        /// Jump here on no match
        no_match_label: String,
    },

    /// Match against multiple patterns (switch statement)
    PatternSwitch {
        /// Register containing input buffer
        input_reg: String,
        /// (pattern, label) pairs
        cases: Vec<(String, String)>,
        /// Default label if no pattern matches
        default_label: String,
    },

    /// Build formatted response from template + captures
    /// Example: "250 Hello {domain}\r\n"
    ResponseBuilder {
        /// Template with {var} placeholders
        template: String,
        /// var name -> register mapping
        variables: HashMap<String, String>,
        /// Register for output buffer
        output_reg: String,
        /// Register for output length
        length_reg: String,
    },

    /// Compare two null-terminated strings
    StringCompare {
        /// First string register
        str1_reg: String,
        /// Second string register
        str2_reg: String,
        /// Result register (0 if equal, nonzero otherwise)
        result_reg: String,
    },

    /// Copy string to destination buffer
    StringCopy {
        /// Source register
        src_reg: String,
        /// Destination register
        dst_reg: String,
        /// Maximum length to copy
        max_len: u32,
        /// Register to store copied length
        copied_len_reg: String,
    },

    // ═══════════════════════════════════════════════════════════════
    // NUMERIC OPERATIONS
    // ═══════════════════════════════════════════════════════════════
    /// Convert integer to decimal string
    IntToString {
        /// Register containing integer value
        value_reg: String,
        /// Register for output buffer
        output_reg: String,
        /// Register for output length
        length_reg: String,
    },

    /// Parse decimal string to integer
    StringToInt {
        /// Register containing input string
        input_reg: String,
        /// Register for result
        result_reg: String,
        /// Jump here on success
        success_label: String,
        /// Jump here on parse error
        error_label: String,
    },

    /// Validate number is within range
    RangeCheck {
        /// Register containing value to check
        value_reg: String,
        /// Minimum allowed value
        min: i64,
        /// Maximum allowed value
        max: i64,
        /// Jump here if in range
        ok_label: String,
        /// Jump here if out of range
        error_label: String,
    },

    // ═══════════════════════════════════════════════════════════════
    // CONTROL FLOW
    // ═══════════════════════════════════════════════════════════════
    /// Validate current state is one of valid states
    StateCheck {
        /// Register containing current state
        state_reg: String,
        /// List of valid state constant names
        valid_states: Vec<String>,
        /// Jump here if state is valid
        ok_label: String,
        /// Jump here if state is invalid
        error_label: String,
    },

    /// Update state register to new state
    StateTransition {
        /// Register containing state
        state_reg: String,
        /// New state constant name
        new_state: String,
    },

    /// Loop until condition met
    LoopUntil {
        /// Loop termination condition
        condition: LoopCondition,
        /// Label for loop body
        body_label: String,
        /// Label to jump to when loop exits
        exit_label: String,
    },

    // ═══════════════════════════════════════════════════════════════
    // I/O OPERATIONS
    // ═══════════════════════════════════════════════════════════════
    /// Send buffer contents over socket
    SendResponse {
        /// Socket file descriptor register
        socket_reg: String,
        /// Buffer register
        buffer_reg: String,
        /// Length register
        length_reg: String,
    },

    /// Read from socket until delimiter
    ReadUntil {
        /// Socket file descriptor register
        socket_reg: String,
        /// Buffer register
        buffer_reg: String,
        /// Delimiter string (e.g., "\r\n")
        delimiter: String,
        /// Maximum bytes to read
        max_len: u32,
        /// Register to store actual length read
        length_reg: String,
        /// Jump here on EOF/disconnect
        eof_label: String,
    },

    /// Read exactly N bytes
    ReadNBytes {
        /// Socket file descriptor register
        socket_reg: String,
        /// Buffer register
        buffer_reg: String,
        /// Register containing count of bytes to read
        count_reg: String,
        /// Jump here on EOF/disconnect
        eof_label: String,
    },

    // ═══════════════════════════════════════════════════════════════
    // EXTENSION INTEGRATION
    // ═══════════════════════════════════════════════════════════════
    /// Call extension with arguments
    ExtensionCall {
        /// RAG intent or explicit extension name
        extension: String,
        /// Argument registers
        args: Vec<String>,
        /// Result register
        result_reg: String,
    },

    /// Validate value using extension (db lookup, etc.)
    ValidationHook {
        /// Type of validation ("db_lookup", "regex", etc.)
        validation_type: String,
        /// Register containing value to validate
        value_reg: String,
        /// Jump here if valid
        ok_label: String,
        /// Jump here if invalid
        error_label: String,
    },

    // ═══════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════
    /// Send error response and optionally close
    ErrorResponse {
        /// Socket file descriptor register
        socket_reg: String,
        /// Error code number
        error_code: u32,
        /// Error message string
        error_message: String,
        /// Whether to close connection after sending
        close_after: bool,
    },

    // ═══════════════════════════════════════════════════════════════
    // DATA STRUCTURES
    // ═══════════════════════════════════════════════════════════════
    /// Write value to buffer at offset
    BufferWrite {
        /// Buffer register
        buffer_reg: String,
        /// Offset (fixed or register)
        offset: BufferOffset,
        /// Value register
        value_reg: String,
        /// Memory width
        width: MemWidth,
    },

    /// Read value from buffer at offset
    BufferRead {
        /// Buffer register
        buffer_reg: String,
        /// Offset (fixed or register)
        offset: BufferOffset,
        /// Result register
        result_reg: String,
        /// Memory width
        width: MemWidth,
    },
}

impl SlotType {
    /// Returns the slot type name as a string
    pub fn name(&self) -> &'static str {
        match self {
            SlotType::PatternMatch { .. } => "PatternMatch",
            SlotType::PatternSwitch { .. } => "PatternSwitch",
            SlotType::ResponseBuilder { .. } => "ResponseBuilder",
            SlotType::StringCompare { .. } => "StringCompare",
            SlotType::StringCopy { .. } => "StringCopy",
            SlotType::IntToString { .. } => "IntToString",
            SlotType::StringToInt { .. } => "StringToInt",
            SlotType::RangeCheck { .. } => "RangeCheck",
            SlotType::StateCheck { .. } => "StateCheck",
            SlotType::StateTransition { .. } => "StateTransition",
            SlotType::LoopUntil { .. } => "LoopUntil",
            SlotType::SendResponse { .. } => "SendResponse",
            SlotType::ReadUntil { .. } => "ReadUntil",
            SlotType::ReadNBytes { .. } => "ReadNBytes",
            SlotType::ExtensionCall { .. } => "ExtensionCall",
            SlotType::ValidationHook { .. } => "ValidationHook",
            SlotType::ErrorResponse { .. } => "ErrorResponse",
            SlotType::BufferWrite { .. } => "BufferWrite",
            SlotType::BufferRead { .. } => "BufferRead",
        }
    }

    /// Returns the typical instruction count range for this slot type
    pub fn instruction_range(&self) -> (usize, usize) {
        match self {
            SlotType::PatternMatch { pattern, .. } => {
                let base = pattern.len() * 3; // ~3 instructions per char
                (50.max(base), 150.max(base + 50))
            }
            SlotType::PatternSwitch { cases, .. } => {
                let base = cases.len() * 20;
                (100.max(base), 200.max(base + 50))
            }
            SlotType::ResponseBuilder { template, .. } => {
                let base = template.len() * 2;
                (50.max(base), 100.max(base + 30))
            }
            SlotType::StringCompare { .. } => (20, 30),
            SlotType::StringCopy { .. } => (25, 35),
            SlotType::IntToString { .. } => (40, 60),
            SlotType::StringToInt { .. } => (30, 50),
            SlotType::RangeCheck { .. } => (10, 20),
            SlotType::StateCheck { valid_states, .. } => {
                let base = valid_states.len() * 5;
                (15.max(base), 30.max(base + 5))
            }
            SlotType::StateTransition { .. } => (2, 5),
            SlotType::LoopUntil { .. } => (10, 20),
            SlotType::SendResponse { .. } => (15, 25),
            SlotType::ReadUntil { .. } => (50, 80),
            SlotType::ReadNBytes { .. } => (30, 50),
            SlotType::ExtensionCall { .. } => (5, 15),
            SlotType::ValidationHook { .. } => (15, 30),
            SlotType::ErrorResponse { .. } => (30, 50),
            SlotType::BufferWrite { .. } => (3, 8),
            SlotType::BufferRead { .. } => (3, 8),
        }
    }

    /// Returns the category of this slot type
    pub fn category(&self) -> SlotCategory {
        match self {
            SlotType::PatternMatch { .. }
            | SlotType::PatternSwitch { .. }
            | SlotType::ResponseBuilder { .. }
            | SlotType::StringCompare { .. }
            | SlotType::StringCopy { .. } => SlotCategory::StringPattern,

            SlotType::IntToString { .. }
            | SlotType::StringToInt { .. }
            | SlotType::RangeCheck { .. } => SlotCategory::Numeric,

            SlotType::StateCheck { .. }
            | SlotType::StateTransition { .. }
            | SlotType::LoopUntil { .. } => SlotCategory::ControlFlow,

            SlotType::SendResponse { .. }
            | SlotType::ReadUntil { .. }
            | SlotType::ReadNBytes { .. } => SlotCategory::IO,

            SlotType::ExtensionCall { .. } | SlotType::ValidationHook { .. } => {
                SlotCategory::Extension
            }

            SlotType::ErrorResponse { .. } => SlotCategory::Error,

            SlotType::BufferWrite { .. } | SlotType::BufferRead { .. } => SlotCategory::Data,
        }
    }
}

/// Category of slot types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SlotCategory {
    /// String and pattern matching operations
    StringPattern,
    /// Numeric conversion and validation
    Numeric,
    /// State machines and loops
    ControlFlow,
    /// Network and file I/O
    IO,
    /// Extension calls and validation hooks
    Extension,
    /// Error response handling
    Error,
    /// Buffer read/write operations
    Data,
}

/// Capture definition for pattern matching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Capture {
    /// Name of the capture
    pub name: String,
    /// Register to store captured value
    pub output_reg: String,
    /// Type of capture
    pub capture_type: CaptureType,
}

/// Type of pattern capture
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[derive(Default)]
pub enum CaptureType {
    /// Capture until whitespace (default)
    #[default]
    Word,
    /// Capture until specific character
    UntilChar {
        /// Character to stop at
        char: char,
    },
    /// Capture quoted string (between quotes)
    Quoted,
    /// Capture rest of line
    Rest,
    /// Parse as integer
    Integer,
}

/// Loop termination condition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum LoopCondition {
    /// Loop until byte at pointer equals value
    ByteEquals {
        /// Register containing pointer
        reg: String,
        /// Value to compare against
        value: u8,
    },
    /// Loop until byte at pointer does not equal value
    ByteNotEquals {
        /// Register containing pointer
        reg: String,
        /// Value to compare against
        value: u8,
    },
    /// Loop until register is zero
    RegisterZero {
        /// Register to check
        reg: String,
    },
    /// Loop until register is non-zero
    RegisterNonZero {
        /// Register to check
        reg: String,
    },
    /// Loop until counter reaches limit
    CounterReached {
        /// Counter register
        counter_reg: String,
        /// Limit register or value
        limit: CounterLimit,
    },
}

/// Counter limit specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum CounterLimit {
    /// Fixed value
    Fixed { value: u64 },
    /// Value from register
    Register { reg: String },
}

/// Buffer offset specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum BufferOffset {
    /// Fixed offset
    Fixed { value: u32 },
    /// Offset from register
    Register { reg: String },
    /// Register plus fixed offset
    RegisterPlusFixed { reg: String, offset: i32 },
}

/// Memory access width
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemWidth {
    /// 8-bit (byte)
    Byte,
    /// 16-bit (word)
    Word,
    /// 32-bit (dword)
    Dword,
    /// 64-bit (qword)
    Qword,
}

impl MemWidth {
    /// Returns the size in bytes
    pub fn size_bytes(&self) -> usize {
        match self {
            MemWidth::Byte => 1,
            MemWidth::Word => 2,
            MemWidth::Dword => 4,
            MemWidth::Qword => 8,
        }
    }

    /// Returns the Neurlang load/store suffix
    pub fn suffix(&self) -> &'static str {
        match self {
            MemWidth::Byte => ".b",
            MemWidth::Word => ".w",
            MemWidth::Dword => ".d",
            MemWidth::Qword => "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_type_name() {
        let slot = SlotType::PatternMatch {
            pattern: "HELO {domain}".to_string(),
            input_reg: "r0".to_string(),
            captures: vec![],
            match_label: "match".to_string(),
            no_match_label: "no_match".to_string(),
        };
        assert_eq!(slot.name(), "PatternMatch");
    }

    #[test]
    fn test_slot_type_category() {
        let slot = SlotType::StateCheck {
            state_reg: "r20".to_string(),
            valid_states: vec!["STATE_INIT".to_string()],
            ok_label: "ok".to_string(),
            error_label: "err".to_string(),
        };
        assert_eq!(slot.category(), SlotCategory::ControlFlow);
    }

    #[test]
    fn test_instruction_range() {
        let slot = SlotType::StateTransition {
            state_reg: "r20".to_string(),
            new_state: "STATE_GREETED".to_string(),
        };
        let (min, max) = slot.instruction_range();
        assert!(min <= max);
        assert!(min >= 2);
        assert!(max <= 5);
    }

    #[test]
    fn test_mem_width() {
        assert_eq!(MemWidth::Byte.size_bytes(), 1);
        assert_eq!(MemWidth::Word.size_bytes(), 2);
        assert_eq!(MemWidth::Dword.size_bytes(), 4);
        assert_eq!(MemWidth::Qword.size_bytes(), 8);

        assert_eq!(MemWidth::Byte.suffix(), ".b");
        assert_eq!(MemWidth::Qword.suffix(), "");
    }

    #[test]
    fn test_serialize_slot_type() {
        let slot = SlotType::RangeCheck {
            value_reg: "r0".to_string(),
            min: 1,
            max: 65535,
            ok_label: "valid".to_string(),
            error_label: "invalid".to_string(),
        };
        let json = serde_json::to_string(&slot).unwrap();
        assert!(json.contains("RangeCheck"));
        assert!(json.contains("65535"));
    }
}
