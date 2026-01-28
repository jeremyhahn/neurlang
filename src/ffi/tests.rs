//! FFI Module Tests

use super::*;

#[test]
fn test_ffi_type_parsing() {
    assert_eq!(FfiType::from_str("u64"), Some(FfiType::U64));
    assert_eq!(FfiType::from_str("int"), Some(FfiType::I32));
    assert_eq!(FfiType::from_str("double"), Some(FfiType::F64));
    assert_eq!(FfiType::from_str("void*"), Some(FfiType::Ptr));
    assert_eq!(FfiType::from_str("const char*"), Some(FfiType::CStr));
    assert_eq!(FfiType::from_str("invalid"), None);
}

#[test]
fn test_ffi_type_properties() {
    assert!(FfiType::U64.is_integer());
    assert!(!FfiType::U64.is_float());
    assert!(!FfiType::U64.is_pointer());

    assert!(FfiType::F64.is_float());
    assert!(!FfiType::F64.is_integer());

    assert!(FfiType::Ptr.is_pointer());
    assert!(FfiType::CStr.is_pointer());
    assert!(FfiType::Buffer.is_pointer());
}

#[test]
fn test_ffi_value_conversion() {
    // Integer conversion
    let val = FfiValue::from_u64(42, FfiType::U64);
    assert_eq!(val.to_u64(), 42);

    // Float conversion
    let f = 3.14159f64;
    let val = FfiValue::from_u64(f.to_bits(), FfiType::F64);
    if let FfiValue::Float(v) = val {
        assert!((v - f).abs() < 1e-10);
    } else {
        panic!("Expected Float value");
    }

    // Pointer conversion
    let ptr = 0x12345678usize;
    let val = FfiValue::from_u64(ptr as u64, FfiType::Ptr);
    if let FfiValue::Pointer(p) = val {
        assert_eq!(p, ptr);
    } else {
        panic!("Expected Pointer value");
    }
}

#[test]
fn test_signature_parsing() {
    // Simple function
    let sig = FfiSignature::parse("int add(int a, int b)").unwrap();
    assert_eq!(sig.name, "add");
    assert_eq!(sig.return_type, FfiType::I32);
    assert_eq!(sig.params.len(), 2);
    assert_eq!(sig.params[0], FfiType::I32);
    assert_eq!(sig.params[1], FfiType::I32);
    assert!(!sig.variadic);

    // Void return
    let sig = FfiSignature::parse("void print(cstr msg)").unwrap();
    assert_eq!(sig.name, "print");
    assert_eq!(sig.return_type, FfiType::Void);
    assert_eq!(sig.params.len(), 1);
    assert_eq!(sig.params[0], FfiType::CStr);

    // No parameters
    let sig = FfiSignature::parse("u64 get_time()").unwrap();
    assert_eq!(sig.name, "get_time");
    assert_eq!(sig.return_type, FfiType::U64);
    assert_eq!(sig.params.len(), 0);
}

#[test]
fn test_signature_display() {
    let sig = FfiSignature::new("add", vec![FfiType::I32, FfiType::I32], FfiType::I32);
    assert_eq!(sig.to_string(), "i32 add(i32, i32)");

    let sig = FfiSignature::variadic("printf", vec![FfiType::CStr], FfiType::I32);
    assert_eq!(sig.to_string(), "i32 printf(cstr, ...)");
}

#[test]
fn test_signature_validation() {
    let sig = FfiSignature::new("add", vec![FfiType::I32, FfiType::I32], FfiType::I32);
    assert!(sig.validate_args(2));
    assert!(!sig.validate_args(1));
    assert!(!sig.validate_args(3));

    let sig = FfiSignature::variadic("printf", vec![FfiType::CStr], FfiType::I32);
    assert!(sig.validate_args(1));
    assert!(sig.validate_args(2));
    assert!(sig.validate_args(10));
    assert!(!sig.validate_args(0));
}

#[test]
fn test_registry_creation() {
    let registry = FfiRegistry::new();
    assert!(registry.list_functions().is_empty());
    assert!(registry.list_libraries().is_empty());
}

#[test]
fn test_function_info_creation() {
    let sig = FfiSignature::new("add", vec![FfiType::I32, FfiType::I32], FfiType::I32);
    let info = FfiFunctionInfo::new("mylib", sig, "Add two integers").with_keywords(vec![
        "add".to_string(),
        "sum".to_string(),
        "plus".to_string(),
    ]);

    assert_eq!(info.library, "mylib");
    assert_eq!(info.signature.name, "add");
    assert_eq!(info.description, "Add two integers");
    assert_eq!(info.keywords.len(), 3);
}

#[test]
fn test_ffi_error_display() {
    let err = FfiError::LoadError("test".to_string());
    assert!(err.to_string().contains("Load error"));

    let err = FfiError::InvalidArgCount {
        expected: 2,
        got: 3,
    };
    assert!(err.to_string().contains("2"));
    assert!(err.to_string().contains("3"));
}

#[cfg(target_os = "linux")]
#[test]
fn test_libc_loading() {
    // Try to load libc (should always be available on Linux)
    let mut registry = FfiRegistry::new();

    // libc.so.6 should be in the default search paths
    if let Ok(()) = registry.load_library("c", Some("libc.so.6")) {
        // Register getpid function
        let sig = FfiSignature::new("getpid", vec![], FfiType::I32);
        let info = FfiFunctionInfo::new("c", sig, "Get process ID")
            .with_keywords(vec!["pid".to_string(), "process".to_string()]);

        registry
            .register_function(info)
            .expect("Failed to register getpid");

        // Call getpid
        let result = registry
            .call("c:getpid", &[])
            .expect("Failed to call getpid");

        // PID should be positive
        assert!(result > 0);

        // Should match std::process::id()
        assert_eq!(result as u32, std::process::id());
    }
}
