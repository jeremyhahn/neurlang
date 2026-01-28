# DateTime Module

Date and time operations using chrono.

## Overview

The datetime module provides safe date/time handling using the `chrono` crate. All operations work with timestamps (Unix epoch milliseconds) or formatted strings.

## Concepts

### Time Representation

| Type | Description | Example |
|------|-------------|---------|
| Timestamp | Unix epoch milliseconds (i64) | `1706198400000` |
| ISO 8601 | Standard string format | `2024-01-25T12:00:00Z` |
| Custom format | strftime-style | `Jan 25, 2024` |

### Timezones

- **UTC**: Universal Coordinated Time (default)
- **Local**: System's local timezone
- **Fixed offset**: e.g., `+05:30`

## API Reference

### Current Time

```rust
/// Get current UTC timestamp (milliseconds since epoch)
pub fn now() -> i64;

/// Get current UTC timestamp (seconds since epoch)
pub fn now_secs() -> i64;

/// Get current local timestamp (milliseconds)
pub fn now_local() -> i64;

/// Get current time as ISO 8601 string
pub fn now_iso() -> OwnedBuffer;
```

### Parsing

```rust
/// Parse ISO 8601 datetime string to timestamp
pub fn parse_iso(input: &OwnedBuffer) -> WrapperResult<i64>;

/// Parse datetime with custom format
pub fn parse_format(input: &OwnedBuffer, format: &OwnedBuffer) -> WrapperResult<i64>;

/// Parse RFC 2822 datetime (email format)
pub fn parse_rfc2822(input: &OwnedBuffer) -> WrapperResult<i64>;
```

### Formatting

```rust
/// Format timestamp as ISO 8601
pub fn format_iso(timestamp: i64) -> OwnedBuffer;

/// Format timestamp with custom format
pub fn format(timestamp: i64, format: &OwnedBuffer) -> OwnedBuffer;

/// Format timestamp as RFC 2822
pub fn format_rfc2822(timestamp: i64) -> OwnedBuffer;
```

### Arithmetic

```rust
/// Add days to timestamp
pub fn add_days(timestamp: i64, days: i64) -> i64;

/// Add hours to timestamp
pub fn add_hours(timestamp: i64, hours: i64) -> i64;

/// Add minutes to timestamp
pub fn add_minutes(timestamp: i64, minutes: i64) -> i64;

/// Add seconds to timestamp
pub fn add_seconds(timestamp: i64, seconds: i64) -> i64;

/// Get difference in seconds between two timestamps
pub fn diff_seconds(a: i64, b: i64) -> i64;

/// Get difference in days between two timestamps
pub fn diff_days(a: i64, b: i64) -> i64;
```

### Component Extraction

```rust
/// Get year from timestamp
pub fn year(timestamp: i64) -> i32;

/// Get month (1-12) from timestamp
pub fn month(timestamp: i64) -> u32;

/// Get day of month (1-31) from timestamp
pub fn day(timestamp: i64) -> u32;

/// Get hour (0-23) from timestamp
pub fn hour(timestamp: i64) -> u32;

/// Get minute (0-59) from timestamp
pub fn minute(timestamp: i64) -> u32;

/// Get second (0-59) from timestamp
pub fn second(timestamp: i64) -> u32;

/// Get weekday (0=Sunday, 6=Saturday)
pub fn weekday(timestamp: i64) -> u32;

/// Get day of year (1-366)
pub fn day_of_year(timestamp: i64) -> u32;
```

### Validation

```rust
/// Check if year is a leap year
pub fn is_leap_year(year: i32) -> bool;

/// Get number of days in month
pub fn days_in_month(year: i32, month: u32) -> u32;
```

## Usage Examples

### Getting Current Time

```rust
use neurlang::wrappers::datetime;

// Milliseconds since epoch
let now_ms = datetime::now();
println!("Timestamp: {}", now_ms);

// ISO 8601 string
let iso = datetime::now_iso();
println!("ISO: {}", iso.as_str().unwrap());
// Output: 2024-01-25T15:30:45.123Z
```

### Parsing Dates

```rust
// ISO 8601
let ts = datetime::parse_iso(&OwnedBuffer::from_str("2024-01-25T12:00:00Z"))?;

// Custom format
let format = OwnedBuffer::from_str("%Y-%m-%d");
let ts = datetime::parse_format(
    &OwnedBuffer::from_str("2024-01-25"),
    &format
)?;

// Email format (RFC 2822)
let ts = datetime::parse_rfc2822(
    &OwnedBuffer::from_str("Thu, 25 Jan 2024 12:00:00 +0000")
)?;
```

### Formatting Dates

```rust
let ts = datetime::now();

// ISO 8601
let iso = datetime::format_iso(ts);
// "2024-01-25T15:30:45.123Z"

// Custom format
let custom = datetime::format(ts, &OwnedBuffer::from_str("%B %d, %Y"));
// "January 25, 2024"

// Short date
let short = datetime::format(ts, &OwnedBuffer::from_str("%Y-%m-%d"));
// "2024-01-25"
```

### Date Arithmetic

```rust
let ts = datetime::now();

// Add time
let tomorrow = datetime::add_days(ts, 1);
let next_week = datetime::add_days(ts, 7);
let in_two_hours = datetime::add_hours(ts, 2);

// Subtract (use negative values)
let yesterday = datetime::add_days(ts, -1);

// Calculate difference
let start = datetime::parse_iso(&OwnedBuffer::from_str("2024-01-01T00:00:00Z"))?;
let end = datetime::parse_iso(&OwnedBuffer::from_str("2024-01-25T00:00:00Z"))?;
let days = datetime::diff_days(end, start);
println!("{} days between dates", days);  // 24
```

### Extracting Components

```rust
let ts = datetime::parse_iso(&OwnedBuffer::from_str("2024-01-25T15:30:45Z"))?;

println!("Year: {}", datetime::year(ts));      // 2024
println!("Month: {}", datetime::month(ts));    // 1
println!("Day: {}", datetime::day(ts));        // 25
println!("Hour: {}", datetime::hour(ts));      // 15
println!("Minute: {}", datetime::minute(ts));  // 30
println!("Second: {}", datetime::second(ts));  // 45
println!("Weekday: {}", datetime::weekday(ts)); // 4 (Thursday)
```

## Format Specifiers

Common strftime specifiers:

| Specifier | Description | Example |
|-----------|-------------|---------|
| `%Y` | Year (4 digits) | 2024 |
| `%m` | Month (01-12) | 01 |
| `%d` | Day of month (01-31) | 25 |
| `%H` | Hour (00-23) | 15 |
| `%M` | Minute (00-59) | 30 |
| `%S` | Second (00-59) | 45 |
| `%B` | Full month name | January |
| `%b` | Abbreviated month | Jan |
| `%A` | Full weekday name | Thursday |
| `%a` | Abbreviated weekday | Thu |
| `%j` | Day of year (001-366) | 025 |
| `%Z` | Timezone name | UTC |
| `%z` | Timezone offset | +0000 |

## IR Assembly Usage

```asm
; Get current time
ext.call r0, @"datetime now"

; Parse ISO string
mov r1, iso_string_ptr
ext.call r0, @"datetime parse", r1

; Format timestamp
mov r1, timestamp
mov r2, format_ptr
ext.call r0, @"datetime format", r1, r2

; Add days
mov r1, timestamp
mov r2, 7          ; days to add
ext.call r0, @"datetime add days", r1, r2

; Extract year
mov r1, timestamp
ext.call r0, @"datetime year", r1
```

## RAG Keywords

| Intent | Resolves To |
|--------|-------------|
| "now", "current time", "timestamp" | `now` |
| "parse date", "parse datetime", "strptime" | `parse_iso` / `parse_format` |
| "format date", "format datetime", "strftime" | `format` |
| "add days", "date add" | `add_days` |
| "diff", "difference", "between dates" | `diff_seconds` / `diff_days` |
| "year", "get year", "extract year" | `year` |
| "month", "get month" | `month` |
| "weekday", "day of week" | `weekday` |

## Error Handling

```rust
use neurlang::wrappers::{WrapperError, datetime};

match datetime::parse_iso(&input) {
    Ok(ts) => {
        println!("Parsed: {}", ts);
    }
    Err(WrapperError::DateTimeError(msg)) => {
        // Invalid format
        eprintln!("Parse error: {}", msg);
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    }
}
```

Common errors:
- `DateTimeError("invalid format")` - String doesn't match expected format
- `DateTimeError("out of range")` - Invalid date (e.g., Feb 30)

## Timezone Considerations

### UTC vs Local

```rust
// UTC - consistent across servers
let utc = datetime::now();

// Local - depends on system timezone
let local = datetime::now_local();

// Always store UTC, convert to local for display
let stored = utc;
let display = datetime::format(stored, &OwnedBuffer::from_str("%Y-%m-%d %H:%M %Z"));
```

### Best Practices

1. **Store timestamps in UTC** - Avoids DST issues
2. **Convert to local only for display** - User sees their timezone
3. **Use ISO 8601 for interchange** - Unambiguous format
4. **Include timezone in parsed strings** - Avoid ambiguity

## Dependencies

```toml
[dependencies]
chrono = "0.4"
```

## See Also

- [Buffer Module](buffer.md) - OwnedBuffer type
- [Encoding Module](encoding.md) - For timestamp encoding
