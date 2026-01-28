//! Regex Extensions
//!
//! Regular expression operations using the regex crate.

use std::sync::Arc;

use regex::Regex;

use super::buffer::{HandleManager, OwnedBuffer};
use super::{ext_ids, ExtCategory, ExtError, ExtensionRegistry};

// =============================================================================
// Pattern Matching
// =============================================================================

/// Check if pattern matches anywhere in input
fn is_match_impl(pattern: &str, input: &str) -> Result<bool, ExtError> {
    let re = Regex::new(pattern)
        .map_err(|e| ExtError::ExtensionError(format!("Invalid regex pattern: {}", e)))?;
    Ok(re.is_match(input))
}

/// Find first match position (start, end)
#[allow(dead_code)]
fn find_impl(pattern: &str, input: &str) -> Result<Option<(usize, usize)>, ExtError> {
    let re = Regex::new(pattern)
        .map_err(|e| ExtError::ExtensionError(format!("Invalid regex pattern: {}", e)))?;
    Ok(re.find(input).map(|m| (m.start(), m.end())))
}

/// Find first match text
fn find_text_impl(pattern: &str, input: &str) -> Result<Option<String>, ExtError> {
    let re = Regex::new(pattern)
        .map_err(|e| ExtError::ExtensionError(format!("Invalid regex pattern: {}", e)))?;
    Ok(re.find(input).map(|m| m.as_str().to_string()))
}

// =============================================================================
// Replacement
// =============================================================================

/// Replace first match
fn replace_impl(pattern: &str, input: &str, replacement: &str) -> Result<String, ExtError> {
    let re = Regex::new(pattern)
        .map_err(|e| ExtError::ExtensionError(format!("Invalid regex pattern: {}", e)))?;
    Ok(re.replace(input, replacement).into_owned())
}

/// Replace all matches
#[allow(dead_code)]
fn replace_all_impl(pattern: &str, input: &str, replacement: &str) -> Result<String, ExtError> {
    let re = Regex::new(pattern)
        .map_err(|e| ExtError::ExtensionError(format!("Invalid regex pattern: {}", e)))?;
    Ok(re.replace_all(input, replacement).into_owned())
}

// =============================================================================
// Splitting
// =============================================================================

/// Split input by pattern
fn split_impl(pattern: &str, input: &str) -> Result<Vec<String>, ExtError> {
    let re = Regex::new(pattern)
        .map_err(|e| ExtError::ExtensionError(format!("Invalid regex pattern: {}", e)))?;
    Ok(re.split(input).map(|s| s.to_string()).collect())
}

// =============================================================================
// Extension Registration
// =============================================================================

pub fn register_regex(registry: &mut ExtensionRegistry) {
    // regex_match: Check if pattern matches
    registry.register_with_id(
        ext_ids::REGEX_MATCH,
        "regex_match",
        "Check if regex pattern matches input. Args: pattern_handle, input_handle. Returns 1 if match, 0 otherwise.",
        2,
        true,
        ExtCategory::Regex,
        Arc::new(|args, outputs| {
            let pattern_handle = args[0];
            let input_handle = args[1];

            let pattern_buf = HandleManager::get(pattern_handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid pattern handle".to_string()))?;
            let input_buf = HandleManager::get(input_handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid input handle".to_string()))?;

            let pattern = pattern_buf.as_str()
                .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8 in pattern: {}", e)))?;
            let input = input_buf.as_str()
                .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8 in input: {}", e)))?;

            let matched = is_match_impl(pattern, input)?;
            let result = if matched { 1i64 } else { 0i64 };
            outputs[0] = result as u64;
            Ok(result)
        }),
    );

    // regex_find: Find first match
    registry.register_with_id(
        ext_ids::REGEX_FIND,
        "regex_find",
        "Find first regex match. Args: pattern_handle, input_handle. Returns buffer_handle with matched text.",
        2,
        true,
        ExtCategory::Regex,
        Arc::new(|args, outputs| {
            let pattern_handle = args[0];
            let input_handle = args[1];

            let pattern_buf = HandleManager::get(pattern_handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid pattern handle".to_string()))?;
            let input_buf = HandleManager::get(input_handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid input handle".to_string()))?;

            let pattern = pattern_buf.as_str()
                .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8 in pattern: {}", e)))?;
            let input = input_buf.as_str()
                .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8 in input: {}", e)))?;

            match find_text_impl(pattern, input)? {
                Some(text) => {
                    let result_handle = HandleManager::store(OwnedBuffer::from_string(text));
                    outputs[0] = result_handle;
                    Ok(result_handle as i64)
                }
                None => {
                    outputs[0] = 0;
                    Ok(0)
                }
            }
        }),
    );

    // regex_replace: Replace first match
    registry.register_with_id(
        ext_ids::REGEX_REPLACE,
        "regex_replace",
        "Replace first regex match. Args: pattern_handle, input_handle, replacement_handle. Returns buffer_handle.",
        3,
        true,
        ExtCategory::Regex,
        Arc::new(|args, outputs| {
            let pattern_handle = args[0];
            let input_handle = args[1];
            let replacement_handle = args[2];

            let pattern_buf = HandleManager::get(pattern_handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid pattern handle".to_string()))?;
            let input_buf = HandleManager::get(input_handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid input handle".to_string()))?;
            let replacement_buf = HandleManager::get(replacement_handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid replacement handle".to_string()))?;

            let pattern = pattern_buf.as_str()
                .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8 in pattern: {}", e)))?;
            let input = input_buf.as_str()
                .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8 in input: {}", e)))?;
            let replacement = replacement_buf.as_str()
                .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8 in replacement: {}", e)))?;

            let result = replace_impl(pattern, input, replacement)?;
            let result_handle = HandleManager::store(OwnedBuffer::from_string(result));
            outputs[0] = result_handle;
            Ok(result_handle as i64)
        }),
    );

    // regex_split: Split by pattern
    registry.register_with_id(
        ext_ids::REGEX_SPLIT,
        "regex_split",
        "Split input by regex pattern. Args: pattern_handle, input_handle. Returns vec_handle of string handles.",
        2,
        true,
        ExtCategory::Regex,
        Arc::new(|args, outputs| {
            let pattern_handle = args[0];
            let input_handle = args[1];

            let pattern_buf = HandleManager::get(pattern_handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid pattern handle".to_string()))?;
            let input_buf = HandleManager::get(input_handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid input handle".to_string()))?;

            let pattern = pattern_buf.as_str()
                .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8 in pattern: {}", e)))?;
            let input = input_buf.as_str()
                .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8 in input: {}", e)))?;

            let parts = split_impl(pattern, input)?;

            // Store each part as a string handle
            let mut handles = Vec::with_capacity(parts.len());
            for part in parts {
                let h = super::strings::next_handle();
                super::strings::STRING_STORAGE.write().unwrap().insert(h, part);
                handles.push(h);
            }

            // Store the vec of handles
            let vec_handle = super::collections::next_handle();
            super::collections::VEC_STORAGE.write().unwrap().insert(vec_handle, handles);

            outputs[0] = vec_handle;
            Ok(vec_handle as i64)
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_match() {
        assert!(is_match_impl(r"\d+", "The answer is 42").unwrap());
        assert!(!is_match_impl(r"\d+", "No numbers here").unwrap());
    }

    #[test]
    fn test_find() {
        let result = find_impl(r"\d+", "Answer is 42!").unwrap();
        assert_eq!(result, Some((10, 12)));
    }

    #[test]
    fn test_find_text() {
        let result = find_text_impl(r"\d+", "Answer is 42!").unwrap();
        assert_eq!(result, Some("42".to_string()));
    }

    #[test]
    fn test_replace() {
        let result = replace_impl(r"\d+", "Item 1 and Item 2", "X").unwrap();
        assert_eq!(result, "Item X and Item 2");
    }

    #[test]
    fn test_replace_all() {
        let result = replace_all_impl(r"\d+", "Item 1 and Item 2", "X").unwrap();
        assert_eq!(result, "Item X and Item X");
    }

    #[test]
    fn test_split() {
        let parts = split_impl(r"\s+", "hello   world\tfoo").unwrap();
        assert_eq!(parts, vec!["hello", "world", "foo"]);
    }

    #[test]
    fn test_invalid_pattern() {
        let result = is_match_impl(r"[invalid", "test");
        assert!(result.is_err());
    }
}
