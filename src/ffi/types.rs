//! FFI Type System
//!
//! Defines types for FFI interop between Neurlang and native code.

use std::fmt;

/// FFI value types supported for function parameters and return values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiType {
    /// Void (no value)
    Void,
    /// 8-bit unsigned integer
    U8,
    /// 16-bit unsigned integer
    U16,
    /// 32-bit unsigned integer
    U32,
    /// 64-bit unsigned integer (Neurlang native)
    U64,
    /// 8-bit signed integer
    I8,
    /// 16-bit signed integer
    I16,
    /// 32-bit signed integer
    I32,
    /// 64-bit signed integer
    I64,
    /// 32-bit floating point
    F32,
    /// 64-bit floating point (Neurlang native)
    F64,
    /// Pointer (usize, platform-dependent)
    Ptr,
    /// Null-terminated C string (const char*)
    CStr,
    /// Buffer with length (const uint8_t*, size_t)
    Buffer,
}

impl FfiType {
    /// Get the size in bytes of this type
    pub fn size(&self) -> usize {
        match self {
            FfiType::Void => 0,
            FfiType::U8 | FfiType::I8 => 1,
            FfiType::U16 | FfiType::I16 => 2,
            FfiType::U32 | FfiType::I32 | FfiType::F32 => 4,
            FfiType::U64 | FfiType::I64 | FfiType::F64 => 8,
            FfiType::Ptr | FfiType::CStr => std::mem::size_of::<usize>(),
            FfiType::Buffer => std::mem::size_of::<usize>() * 2, // ptr + len
        }
    }

    /// Check if this type is an integer type
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            FfiType::U8
                | FfiType::U16
                | FfiType::U32
                | FfiType::U64
                | FfiType::I8
                | FfiType::I16
                | FfiType::I32
                | FfiType::I64
        )
    }

    /// Check if this type is a floating point type
    pub fn is_float(&self) -> bool {
        matches!(self, FfiType::F32 | FfiType::F64)
    }

    /// Check if this type is a pointer type
    pub fn is_pointer(&self) -> bool {
        matches!(self, FfiType::Ptr | FfiType::CStr | FfiType::Buffer)
    }

    /// Parse from a string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "void" => Some(FfiType::Void),
            "u8" | "uint8" | "uint8_t" | "byte" => Some(FfiType::U8),
            "u16" | "uint16" | "uint16_t" => Some(FfiType::U16),
            "u32" | "uint32" | "uint32_t" => Some(FfiType::U32),
            "u64" | "uint64" | "uint64_t" | "ulong" => Some(FfiType::U64),
            "i8" | "int8" | "int8_t" => Some(FfiType::I8),
            "i16" | "int16" | "int16_t" => Some(FfiType::I16),
            "i32" | "int32" | "int32_t" | "int" => Some(FfiType::I32),
            "i64" | "int64" | "int64_t" | "long" => Some(FfiType::I64),
            "f32" | "float" => Some(FfiType::F32),
            "f64" | "double" => Some(FfiType::F64),
            "ptr" | "pointer" | "void*" => Some(FfiType::Ptr),
            "cstr" | "string" | "char*" | "const char*" => Some(FfiType::CStr),
            "buffer" | "bytes" => Some(FfiType::Buffer),
            _ => None,
        }
    }
}

impl fmt::Display for FfiType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FfiType::Void => write!(f, "void"),
            FfiType::U8 => write!(f, "u8"),
            FfiType::U16 => write!(f, "u16"),
            FfiType::U32 => write!(f, "u32"),
            FfiType::U64 => write!(f, "u64"),
            FfiType::I8 => write!(f, "i8"),
            FfiType::I16 => write!(f, "i16"),
            FfiType::I32 => write!(f, "i32"),
            FfiType::I64 => write!(f, "i64"),
            FfiType::F32 => write!(f, "f32"),
            FfiType::F64 => write!(f, "f64"),
            FfiType::Ptr => write!(f, "ptr"),
            FfiType::CStr => write!(f, "cstr"),
            FfiType::Buffer => write!(f, "buffer"),
        }
    }
}

/// A value that can be passed to or returned from FFI functions
#[derive(Debug, Clone)]
pub enum FfiValue {
    /// No value
    Void,
    /// 64-bit unsigned integer (covers all integer types)
    Integer(u64),
    /// 64-bit floating point
    Float(f64),
    /// Pointer value
    Pointer(usize),
    /// Owned string (for CStr)
    String(String),
    /// Owned buffer (for Buffer)
    Buffer(Vec<u8>),
}

impl FfiValue {
    /// Create a value from a u64 (Neurlang register value)
    pub fn from_u64(value: u64, ty: FfiType) -> Self {
        match ty {
            FfiType::Void => FfiValue::Void,
            FfiType::U8
            | FfiType::U16
            | FfiType::U32
            | FfiType::U64
            | FfiType::I8
            | FfiType::I16
            | FfiType::I32
            | FfiType::I64 => FfiValue::Integer(value),
            FfiType::F32 => {
                let bits = value as u32;
                FfiValue::Float(f32::from_bits(bits) as f64)
            }
            FfiType::F64 => FfiValue::Float(f64::from_bits(value)),
            FfiType::Ptr | FfiType::CStr | FfiType::Buffer => FfiValue::Pointer(value as usize),
        }
    }

    /// Convert to a u64 (for Neurlang register)
    pub fn to_u64(&self) -> u64 {
        match self {
            FfiValue::Void => 0,
            FfiValue::Integer(v) => *v,
            FfiValue::Float(v) => v.to_bits(),
            FfiValue::Pointer(v) => *v as u64,
            FfiValue::String(s) => s.as_ptr() as u64,
            FfiValue::Buffer(b) => b.as_ptr() as u64,
        }
    }

    /// Get the type of this value
    pub fn get_type(&self) -> FfiType {
        match self {
            FfiValue::Void => FfiType::Void,
            FfiValue::Integer(_) => FfiType::U64,
            FfiValue::Float(_) => FfiType::F64,
            FfiValue::Pointer(_) => FfiType::Ptr,
            FfiValue::String(_) => FfiType::CStr,
            FfiValue::Buffer(_) => FfiType::Buffer,
        }
    }

    /// Check if this is a void value
    pub fn is_void(&self) -> bool {
        matches!(self, FfiValue::Void)
    }
}

/// Function signature for FFI calls
#[derive(Debug, Clone)]
pub struct FfiSignature {
    /// Function name
    pub name: String,
    /// Parameter types
    pub params: Vec<FfiType>,
    /// Return type
    pub return_type: FfiType,
    /// Whether this function is variadic
    pub variadic: bool,
}

impl FfiSignature {
    /// Create a new function signature
    pub fn new(name: impl Into<String>, params: Vec<FfiType>, return_type: FfiType) -> Self {
        Self {
            name: name.into(),
            params,
            return_type,
            variadic: false,
        }
    }

    /// Create a variadic function signature
    pub fn variadic(name: impl Into<String>, params: Vec<FfiType>, return_type: FfiType) -> Self {
        Self {
            name: name.into(),
            params,
            return_type,
            variadic: true,
        }
    }

    /// Validate argument count
    pub fn validate_args(&self, arg_count: usize) -> bool {
        if self.variadic {
            arg_count >= self.params.len()
        } else {
            arg_count == self.params.len()
        }
    }

    /// Parse from a C-style signature string
    /// Format: "return_type function_name(param1_type, param2_type, ...)"
    pub fn parse(signature: &str) -> Option<Self> {
        // Simple parser for "type name(type, type, ...)"
        let signature = signature.trim();

        // Find function name and return type
        let paren_pos = signature.find('(')?;
        let before_paren = signature[..paren_pos].trim();
        let after_paren = signature[paren_pos + 1..].trim_end_matches(')').trim();

        // Split return type and name
        let parts: Vec<&str> = before_paren.rsplitn(2, char::is_whitespace).collect();
        if parts.len() < 2 {
            return None;
        }

        let name = parts[0].trim();
        let return_type_str = parts[1].trim();
        let return_type = FfiType::from_str(return_type_str)?;

        // Parse parameters
        let mut params = Vec::new();
        let variadic = after_paren.contains("...");

        for param in after_paren.split(',') {
            let param = param.trim();
            if param.is_empty() || param == "..." {
                continue;
            }
            // Take last word as param name, everything before as type
            let type_str = param.split_whitespace().next()?;
            let param_type = FfiType::from_str(type_str)?;
            params.push(param_type);
        }

        Some(Self {
            name: name.to_string(),
            params,
            return_type,
            variadic,
        })
    }
}

impl fmt::Display for FfiSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}(", self.return_type, self.name)?;
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", param)?;
        }
        if self.variadic {
            if !self.params.is_empty() {
                write!(f, ", ")?;
            }
            write!(f, "...")?;
        }
        write!(f, ")")
    }
}
