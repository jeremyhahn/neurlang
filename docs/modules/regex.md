# Regex Module

Regular expression operations.

## Overview

The regex module provides safe regular expression matching, searching, and replacement using the `regex` crate. Patterns are compiled once and can be reused.

## API Reference

### Pattern Matching

```rust
/// Check if pattern matches anywhere in input
pub fn is_match(pattern: &OwnedBuffer, input: &OwnedBuffer) -> WrapperResult<bool>;

/// Check if pattern matches entire input
pub fn is_full_match(pattern: &OwnedBuffer, input: &OwnedBuffer) -> WrapperResult<bool>;
```

### Finding Matches

```rust
/// Find first match, returns (start, end) or None
pub fn find(pattern: &OwnedBuffer, input: &OwnedBuffer) -> WrapperResult<Option<(usize, usize)>>;

/// Find first match and return the matched text
pub fn find_text(pattern: &OwnedBuffer, input: &OwnedBuffer) -> WrapperResult<Option<OwnedBuffer>>;

/// Find all matches, returns vec of (start, end)
pub fn find_all(pattern: &OwnedBuffer, input: &OwnedBuffer) -> WrapperResult<Vec<(usize, usize)>>;

/// Find all matches and return matched texts
pub fn find_all_text(pattern: &OwnedBuffer, input: &OwnedBuffer) -> WrapperResult<Vec<OwnedBuffer>>;
```

### Capture Groups

```rust
/// Get capture groups from first match
pub fn captures(pattern: &OwnedBuffer, input: &OwnedBuffer) -> WrapperResult<Vec<Option<OwnedBuffer>>>;

/// Get named capture groups as key-value pairs
pub fn captures_named(pattern: &OwnedBuffer, input: &OwnedBuffer) -> WrapperResult<Vec<(String, OwnedBuffer)>>;
```

### Replacement

```rust
/// Replace first match
pub fn replace(pattern: &OwnedBuffer, input: &OwnedBuffer, replacement: &OwnedBuffer) -> WrapperResult<OwnedBuffer>;

/// Replace all matches
pub fn replace_all(pattern: &OwnedBuffer, input: &OwnedBuffer, replacement: &OwnedBuffer) -> WrapperResult<OwnedBuffer>;
```

### Splitting

```rust
/// Split input by pattern
pub fn split(pattern: &OwnedBuffer, input: &OwnedBuffer) -> WrapperResult<Vec<OwnedBuffer>>;

/// Split with limit on number of parts
pub fn splitn(pattern: &OwnedBuffer, input: &OwnedBuffer, limit: usize) -> WrapperResult<Vec<OwnedBuffer>>;
```

## Usage Examples

### Basic Matching

```rust
use neurlang::wrappers::{OwnedBuffer, regex};

let pattern = OwnedBuffer::from_str(r"\d+");
let input = OwnedBuffer::from_str("The answer is 42");

// Check if matches
if regex::is_match(&pattern, &input)? {
    println!("Contains a number!");
}

// Find the match
if let Some((start, end)) = regex::find(&pattern, &input)? {
    println!("Found at {}..{}", start, end);
}

// Get matched text
if let Some(matched) = regex::find_text(&pattern, &input)? {
    println!("Matched: {}", matched.as_str().unwrap());  // "42"
}
```

### Finding All Matches

```rust
let pattern = OwnedBuffer::from_str(r"\b\w{4}\b");  // 4-letter words
let input = OwnedBuffer::from_str("The quick brown fox jumps over the lazy dog");

let matches = regex::find_all_text(&pattern, &input)?;
for m in matches {
    println!("Found: {}", m.as_str().unwrap());
}
// Output: "over", "lazy"
```

### Capture Groups

```rust
// Named captures
let pattern = OwnedBuffer::from_str(r"(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2})");
let input = OwnedBuffer::from_str("Date: 2024-01-25");

let captures = regex::captures_named(&pattern, &input)?;
for (name, value) in captures {
    println!("{}: {}", name, value.as_str().unwrap());
}
// Output:
// year: 2024
// month: 01
// day: 25

// Positional captures
let pattern = OwnedBuffer::from_str(r"(\w+)@(\w+)\.(\w+)");
let input = OwnedBuffer::from_str("Email: user@example.com");

let captures = regex::captures(&pattern, &input)?;
// captures[0] = full match "user@example.com"
// captures[1] = "user"
// captures[2] = "example"
// captures[3] = "com"
```

### Replacement

```rust
// Replace first
let pattern = OwnedBuffer::from_str(r"\d+");
let input = OwnedBuffer::from_str("Item 1 and Item 2");
let replacement = OwnedBuffer::from_str("X");

let result = regex::replace(&pattern, &input, &replacement)?;
assert_eq!(result.as_str().unwrap(), "Item X and Item 2");

// Replace all
let result = regex::replace_all(&pattern, &input, &replacement)?;
assert_eq!(result.as_str().unwrap(), "Item X and Item X");

// With capture groups
let pattern = OwnedBuffer::from_str(r"(\w+)@(\w+)");
let replacement = OwnedBuffer::from_str("$1 at $2");
let input = OwnedBuffer::from_str("Contact: admin@server");

let result = regex::replace(&pattern, &input, &replacement)?;
assert_eq!(result.as_str().unwrap(), "Contact: admin at server");
```

### Splitting

```rust
// Split by whitespace
let pattern = OwnedBuffer::from_str(r"\s+");
let input = OwnedBuffer::from_str("hello   world\tfoo");

let parts = regex::split(&pattern, &input)?;
// ["hello", "world", "foo"]

// Split with limit
let parts = regex::splitn(&pattern, &input, 2)?;
// ["hello", "world\tfoo"]
```

## IR Assembly Usage

```asm
; Check match
mov r1, pattern_ptr
mov r2, input_ptr
ext.call r0, @"regex match", r1, r2
; r0 = 1 if match, 0 if not

; Find first match
mov r1, pattern_ptr
mov r2, input_ptr
ext.call r0, @"regex find", r1, r2
; r0 = handle to match info

; Replace all
mov r1, pattern_ptr
mov r2, input_ptr
mov r3, replacement_ptr
ext.call r0, @"regex replace all", r1, r2, r3

; Split
mov r1, pattern_ptr
mov r2, input_ptr
ext.call r0, @"regex split", r1, r2
; r0 = handle to vec of parts
```

## RAG Keywords

| Intent | Resolves To |
|--------|-------------|
| "regex match", "pattern match", "matches" | `is_match` |
| "regex find", "search pattern" | `find` |
| "find all", "findall", "global match" | `find_all` |
| "regex replace", "substitute" | `replace` |
| "replace all", "gsub", "global replace" | `replace_all` |
| "regex split", "split pattern" | `split` |
| "capture groups", "captures" | `captures` |

## Regex Syntax Reference

### Character Classes

| Pattern | Matches |
|---------|---------|
| `.` | Any character except newline |
| `\d` | Digit (0-9) |
| `\D` | Non-digit |
| `\w` | Word character (a-z, A-Z, 0-9, _) |
| `\W` | Non-word character |
| `\s` | Whitespace |
| `\S` | Non-whitespace |
| `[abc]` | a, b, or c |
| `[^abc]` | Not a, b, or c |
| `[a-z]` | a through z |

### Quantifiers

| Pattern | Matches |
|---------|---------|
| `*` | 0 or more |
| `+` | 1 or more |
| `?` | 0 or 1 |
| `{n}` | Exactly n |
| `{n,}` | n or more |
| `{n,m}` | Between n and m |

### Anchors

| Pattern | Matches |
|---------|---------|
| `^` | Start of string |
| `$` | End of string |
| `\b` | Word boundary |
| `\B` | Non-word boundary |

### Groups

| Pattern | Description |
|---------|-------------|
| `(...)` | Capture group |
| `(?:...)` | Non-capturing group |
| `(?P<name>...)` | Named capture group |
| `\1`, `\2` | Backreference |

## Error Handling

```rust
use neurlang::wrappers::{WrapperError, regex};

match regex::is_match(&pattern, &input) {
    Ok(matched) => {
        println!("Matched: {}", matched);
    }
    Err(WrapperError::RegexError(msg)) => {
        // Invalid pattern syntax
        eprintln!("Regex error: {}", msg);
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    }
}
```

Common errors:
- `RegexError("invalid syntax")` - Pattern has syntax error
- `RegexError("unclosed group")` - Missing closing parenthesis
- `RegexError("invalid escape")` - Unknown escape sequence

## Performance Considerations

### Pattern Compilation

The `regex` crate compiles patterns, which has overhead. For repeated use:

```rust
// Inefficient: compiles pattern each time
for line in lines {
    if regex::is_match(&pattern, &line)? {
        // ...
    }
}

// Future API: compiled patterns
// let compiled = regex::compile(&pattern)?;
// for line in lines {
//     if compiled.is_match(&line) {
//         // ...
//     }
// }
```

### Avoid Catastrophic Backtracking

Some patterns can be slow:

```rust
// Dangerous: exponential backtracking
let bad = OwnedBuffer::from_str(r"(a+)+$");

// Better: use possessive quantifiers or atomic groups
// (not available in regex crate, but pattern can be restructured)
```

## Dependencies

```toml
[dependencies]
regex = "1.10"
```

## See Also

- [Buffer Module](buffer.md) - OwnedBuffer type
- [Strings Module](../runtime/stdlib/strings.md) - Simple string operations
