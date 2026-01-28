//! Regex Wrappers
//!
//! Regular expression operations using the regex crate.

use regex::Regex;

use super::{OwnedBuffer, WrapperCategory, WrapperError, WrapperRegistry, WrapperResult};

// =============================================================================
// Pattern Matching
// =============================================================================

/// Check if pattern matches anywhere in input
pub fn is_match(pattern: &OwnedBuffer, input: &OwnedBuffer) -> WrapperResult<bool> {
    let pattern_str = pattern.as_str()?;
    let input_str = input.as_str()?;

    let re = Regex::new(pattern_str)
        .map_err(|e| WrapperError::RegexError(format!("Invalid pattern: {}", e)))?;

    Ok(re.is_match(input_str))
}

/// Check if pattern matches entire input
pub fn is_full_match(pattern: &OwnedBuffer, input: &OwnedBuffer) -> WrapperResult<bool> {
    let pattern_str = pattern.as_str()?;
    let input_str = input.as_str()?;

    // Anchor the pattern to match the entire string
    let anchored = format!("^(?:{})$", pattern_str);
    let re = Regex::new(&anchored)
        .map_err(|e| WrapperError::RegexError(format!("Invalid pattern: {}", e)))?;

    Ok(re.is_match(input_str))
}

// =============================================================================
// Finding Matches
// =============================================================================

/// Find first match, returns (start, end) or None
pub fn find(pattern: &OwnedBuffer, input: &OwnedBuffer) -> WrapperResult<Option<(usize, usize)>> {
    let pattern_str = pattern.as_str()?;
    let input_str = input.as_str()?;

    let re = Regex::new(pattern_str)
        .map_err(|e| WrapperError::RegexError(format!("Invalid pattern: {}", e)))?;

    Ok(re.find(input_str).map(|m| (m.start(), m.end())))
}

/// Find first match and return the matched text
pub fn find_text(pattern: &OwnedBuffer, input: &OwnedBuffer) -> WrapperResult<Option<OwnedBuffer>> {
    let pattern_str = pattern.as_str()?;
    let input_str = input.as_str()?;

    let re = Regex::new(pattern_str)
        .map_err(|e| WrapperError::RegexError(format!("Invalid pattern: {}", e)))?;

    Ok(re
        .find(input_str)
        .map(|m| OwnedBuffer::from_str(m.as_str())))
}

/// Find all matches, returns vec of (start, end)
pub fn find_all(pattern: &OwnedBuffer, input: &OwnedBuffer) -> WrapperResult<Vec<(usize, usize)>> {
    let pattern_str = pattern.as_str()?;
    let input_str = input.as_str()?;

    let re = Regex::new(pattern_str)
        .map_err(|e| WrapperError::RegexError(format!("Invalid pattern: {}", e)))?;

    Ok(re
        .find_iter(input_str)
        .map(|m| (m.start(), m.end()))
        .collect())
}

/// Find all matches and return matched texts
pub fn find_all_text(
    pattern: &OwnedBuffer,
    input: &OwnedBuffer,
) -> WrapperResult<Vec<OwnedBuffer>> {
    let pattern_str = pattern.as_str()?;
    let input_str = input.as_str()?;

    let re = Regex::new(pattern_str)
        .map_err(|e| WrapperError::RegexError(format!("Invalid pattern: {}", e)))?;

    Ok(re
        .find_iter(input_str)
        .map(|m| OwnedBuffer::from_str(m.as_str()))
        .collect())
}

// =============================================================================
// Capture Groups
// =============================================================================

/// Get capture groups from first match
pub fn captures(
    pattern: &OwnedBuffer,
    input: &OwnedBuffer,
) -> WrapperResult<Vec<Option<OwnedBuffer>>> {
    let pattern_str = pattern.as_str()?;
    let input_str = input.as_str()?;

    let re = Regex::new(pattern_str)
        .map_err(|e| WrapperError::RegexError(format!("Invalid pattern: {}", e)))?;

    match re.captures(input_str) {
        Some(caps) => Ok(caps
            .iter()
            .map(|m| m.map(|m| OwnedBuffer::from_str(m.as_str())))
            .collect()),
        None => Ok(Vec::new()),
    }
}

/// Get named capture groups as key-value pairs
pub fn captures_named(
    pattern: &OwnedBuffer,
    input: &OwnedBuffer,
) -> WrapperResult<Vec<(String, OwnedBuffer)>> {
    let pattern_str = pattern.as_str()?;
    let input_str = input.as_str()?;

    let re = Regex::new(pattern_str)
        .map_err(|e| WrapperError::RegexError(format!("Invalid pattern: {}", e)))?;

    let mut result = Vec::new();

    if let Some(caps) = re.captures(input_str) {
        for name in re.capture_names().flatten() {
            if let Some(m) = caps.name(name) {
                result.push((name.to_string(), OwnedBuffer::from_str(m.as_str())));
            }
        }
    }

    Ok(result)
}

// =============================================================================
// Replacement
// =============================================================================

/// Replace first match
pub fn replace(
    pattern: &OwnedBuffer,
    input: &OwnedBuffer,
    replacement: &OwnedBuffer,
) -> WrapperResult<OwnedBuffer> {
    let pattern_str = pattern.as_str()?;
    let input_str = input.as_str()?;
    let replacement_str = replacement.as_str()?;

    let re = Regex::new(pattern_str)
        .map_err(|e| WrapperError::RegexError(format!("Invalid pattern: {}", e)))?;

    Ok(OwnedBuffer::from_string(
        re.replace(input_str, replacement_str).into_owned(),
    ))
}

/// Replace all matches
pub fn replace_all(
    pattern: &OwnedBuffer,
    input: &OwnedBuffer,
    replacement: &OwnedBuffer,
) -> WrapperResult<OwnedBuffer> {
    let pattern_str = pattern.as_str()?;
    let input_str = input.as_str()?;
    let replacement_str = replacement.as_str()?;

    let re = Regex::new(pattern_str)
        .map_err(|e| WrapperError::RegexError(format!("Invalid pattern: {}", e)))?;

    Ok(OwnedBuffer::from_string(
        re.replace_all(input_str, replacement_str).into_owned(),
    ))
}

// =============================================================================
// Splitting
// =============================================================================

/// Split input by pattern
pub fn split(pattern: &OwnedBuffer, input: &OwnedBuffer) -> WrapperResult<Vec<OwnedBuffer>> {
    let pattern_str = pattern.as_str()?;
    let input_str = input.as_str()?;

    let re = Regex::new(pattern_str)
        .map_err(|e| WrapperError::RegexError(format!("Invalid pattern: {}", e)))?;

    Ok(re.split(input_str).map(OwnedBuffer::from_str).collect())
}

/// Split with limit on number of parts
pub fn splitn(
    pattern: &OwnedBuffer,
    input: &OwnedBuffer,
    limit: usize,
) -> WrapperResult<Vec<OwnedBuffer>> {
    let pattern_str = pattern.as_str()?;
    let input_str = input.as_str()?;

    let re = Regex::new(pattern_str)
        .map_err(|e| WrapperError::RegexError(format!("Invalid pattern: {}", e)))?;

    Ok(re
        .splitn(input_str, limit)
        .map(OwnedBuffer::from_str)
        .collect())
}

// =============================================================================
// Registration
// =============================================================================

/// Register all regex wrappers with the registry
pub fn register(registry: &mut WrapperRegistry) {
    registry.register_wrapper(
        "regex_match",
        "Check if pattern matches anywhere in input",
        WrapperCategory::Regex,
        2,
        &["regex", "match", "test", "pattern match"],
        |args| {
            if args.len() < 2 {
                return Err(WrapperError::InvalidArg(
                    "Need pattern and input".to_string(),
                ));
            }
            let matched = is_match(&args[0], &args[1])?;
            Ok(OwnedBuffer::from_slice(&[if matched { 1 } else { 0 }]))
        },
    );

    registry.register_wrapper(
        "regex_find",
        "Find first match and return matched text",
        WrapperCategory::Regex,
        2,
        &["regex find", "search", "locate pattern"],
        |args| {
            if args.len() < 2 {
                return Err(WrapperError::InvalidArg(
                    "Need pattern and input".to_string(),
                ));
            }
            match find_text(&args[0], &args[1])? {
                Some(buf) => Ok(buf),
                None => Ok(OwnedBuffer::new()),
            }
        },
    );

    registry.register_wrapper(
        "regex_find_all",
        "Find all matches and return matched texts",
        WrapperCategory::Regex,
        2,
        &["regex find all", "findall", "all matches"],
        |args| {
            if args.len() < 2 {
                return Err(WrapperError::InvalidArg(
                    "Need pattern and input".to_string(),
                ));
            }
            let matches = find_all_text(&args[0], &args[1])?;
            // Return as newline-separated
            let result: String = matches
                .iter()
                .filter_map(|m| m.as_str().ok())
                .collect::<Vec<_>>()
                .join("\n");
            Ok(OwnedBuffer::from_string(result))
        },
    );

    registry.register_wrapper(
        "regex_replace",
        "Replace first match",
        WrapperCategory::Regex,
        3,
        &["regex replace", "substitute", "sub"],
        |args| {
            if args.len() < 3 {
                return Err(WrapperError::InvalidArg(
                    "Need pattern, input, and replacement".to_string(),
                ));
            }
            replace(&args[0], &args[1], &args[2])
        },
    );

    registry.register_wrapper(
        "regex_replace_all",
        "Replace all matches",
        WrapperCategory::Regex,
        3,
        &["regex replace all", "gsub", "global replace"],
        |args| {
            if args.len() < 3 {
                return Err(WrapperError::InvalidArg(
                    "Need pattern, input, and replacement".to_string(),
                ));
            }
            replace_all(&args[0], &args[1], &args[2])
        },
    );

    registry.register_wrapper(
        "regex_split",
        "Split input by pattern",
        WrapperCategory::Regex,
        2,
        &["regex split", "tokenize"],
        |args| {
            if args.len() < 2 {
                return Err(WrapperError::InvalidArg(
                    "Need pattern and input".to_string(),
                ));
            }
            let parts = split(&args[0], &args[1])?;
            // Return as newline-separated
            let result: String = parts
                .iter()
                .filter_map(|p| p.as_str().ok())
                .collect::<Vec<_>>()
                .join("\n");
            Ok(OwnedBuffer::from_string(result))
        },
    );

    registry.register_wrapper(
        "regex_captures",
        "Get capture groups from first match",
        WrapperCategory::Regex,
        2,
        &["regex captures", "capture groups", "groups"],
        |args| {
            if args.len() < 2 {
                return Err(WrapperError::InvalidArg(
                    "Need pattern and input".to_string(),
                ));
            }
            let caps = captures(&args[0], &args[1])?;
            // Return as newline-separated (empty for None groups)
            let result: String = caps
                .iter()
                .map(|c| c.as_ref().and_then(|b| b.as_str().ok()).unwrap_or(""))
                .collect::<Vec<_>>()
                .join("\n");
            Ok(OwnedBuffer::from_string(result))
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_match() {
        let pattern = OwnedBuffer::from_str(r"\d+");
        let input = OwnedBuffer::from_str("The answer is 42");
        assert!(is_match(&pattern, &input).unwrap());

        let no_match = OwnedBuffer::from_str("No numbers here");
        assert!(!is_match(&pattern, &no_match).unwrap());
    }

    #[test]
    fn test_is_full_match() {
        let pattern = OwnedBuffer::from_str(r"\d+");
        let full = OwnedBuffer::from_str("42");
        let partial = OwnedBuffer::from_str("Answer: 42");

        assert!(is_full_match(&pattern, &full).unwrap());
        assert!(!is_full_match(&pattern, &partial).unwrap());
    }

    #[test]
    fn test_find() {
        let pattern = OwnedBuffer::from_str(r"\d+");
        let input = OwnedBuffer::from_str("Answer is 42!");

        let (start, end) = find(&pattern, &input).unwrap().unwrap();
        assert_eq!(&input.as_str().unwrap()[start..end], "42");
    }

    #[test]
    fn test_find_all() {
        let pattern = OwnedBuffer::from_str(r"\d+");
        let input = OwnedBuffer::from_str("1 + 2 = 3");

        let matches = find_all_text(&pattern, &input).unwrap();
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].as_str().unwrap(), "1");
        assert_eq!(matches[1].as_str().unwrap(), "2");
        assert_eq!(matches[2].as_str().unwrap(), "3");
    }

    #[test]
    fn test_captures() {
        let pattern = OwnedBuffer::from_str(r"(\w+)@(\w+)\.(\w+)");
        let input = OwnedBuffer::from_str("Email: user@example.com");

        let caps = captures(&pattern, &input).unwrap();
        assert_eq!(caps.len(), 4); // Full match + 3 groups
        assert_eq!(caps[1].as_ref().unwrap().as_str().unwrap(), "user");
        assert_eq!(caps[2].as_ref().unwrap().as_str().unwrap(), "example");
        assert_eq!(caps[3].as_ref().unwrap().as_str().unwrap(), "com");
    }

    #[test]
    fn test_named_captures() {
        let pattern = OwnedBuffer::from_str(r"(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2})");
        let input = OwnedBuffer::from_str("Date: 2024-01-25");

        let caps = captures_named(&pattern, &input).unwrap();
        assert_eq!(caps.len(), 3);

        let year = caps.iter().find(|(k, _)| k == "year").unwrap();
        assert_eq!(year.1.as_str().unwrap(), "2024");
    }

    #[test]
    fn test_replace() {
        let pattern = OwnedBuffer::from_str(r"\d+");
        let input = OwnedBuffer::from_str("Item 1 and Item 2");
        let replacement = OwnedBuffer::from_str("X");

        let result = replace(&pattern, &input, &replacement).unwrap();
        assert_eq!(result.as_str().unwrap(), "Item X and Item 2");
    }

    #[test]
    fn test_replace_all() {
        let pattern = OwnedBuffer::from_str(r"\d+");
        let input = OwnedBuffer::from_str("Item 1 and Item 2");
        let replacement = OwnedBuffer::from_str("X");

        let result = replace_all(&pattern, &input, &replacement).unwrap();
        assert_eq!(result.as_str().unwrap(), "Item X and Item X");
    }

    #[test]
    fn test_replace_with_groups() {
        let pattern = OwnedBuffer::from_str(r"(\w+)@(\w+)");
        let input = OwnedBuffer::from_str("Contact: admin@server");
        let replacement = OwnedBuffer::from_str("$1 at $2");

        let result = replace(&pattern, &input, &replacement).unwrap();
        assert_eq!(result.as_str().unwrap(), "Contact: admin at server");
    }

    #[test]
    fn test_split() {
        let pattern = OwnedBuffer::from_str(r"\s+");
        let input = OwnedBuffer::from_str("hello   world\tfoo");

        let parts = split(&pattern, &input).unwrap();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].as_str().unwrap(), "hello");
        assert_eq!(parts[1].as_str().unwrap(), "world");
        assert_eq!(parts[2].as_str().unwrap(), "foo");
    }

    #[test]
    fn test_splitn() {
        let pattern = OwnedBuffer::from_str(r"\s+");
        let input = OwnedBuffer::from_str("a b c d");

        let parts = splitn(&pattern, &input, 2).unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].as_str().unwrap(), "a");
        assert_eq!(parts[1].as_str().unwrap(), "b c d");
    }

    #[test]
    fn test_invalid_pattern() {
        let pattern = OwnedBuffer::from_str(r"[invalid");
        let input = OwnedBuffer::from_str("test");

        let result = is_match(&pattern, &input);
        assert!(result.is_err());
    }
}
