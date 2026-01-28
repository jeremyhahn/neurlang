//! DateTime Wrappers
//!
//! Date and time operations using chrono.

use chrono::{DateTime, Datelike, Local, NaiveDateTime, TimeZone, Timelike, Utc};

use super::{OwnedBuffer, WrapperCategory, WrapperError, WrapperRegistry, WrapperResult};

// =============================================================================
// Current Time
// =============================================================================

/// Get current UTC timestamp (milliseconds since epoch)
pub fn now() -> i64 {
    Utc::now().timestamp_millis()
}

/// Get current UTC timestamp (seconds since epoch)
pub fn now_secs() -> i64 {
    Utc::now().timestamp()
}

/// Get current local timestamp (milliseconds)
pub fn now_local() -> i64 {
    Local::now().timestamp_millis()
}

/// Get current time as ISO 8601 string
pub fn now_iso() -> OwnedBuffer {
    OwnedBuffer::from_string(Utc::now().to_rfc3339())
}

// =============================================================================
// Parsing
// =============================================================================

/// Parse ISO 8601 datetime string to timestamp (milliseconds)
pub fn parse_iso(input: &OwnedBuffer) -> WrapperResult<i64> {
    let input_str = input.as_str()?;
    let dt = DateTime::parse_from_rfc3339(input_str.trim())
        .map_err(|e| WrapperError::DateTimeError(format!("Invalid ISO 8601: {}", e)))?;
    Ok(dt.timestamp_millis())
}

/// Parse datetime with custom format
pub fn parse_format(input: &OwnedBuffer, format: &OwnedBuffer) -> WrapperResult<i64> {
    let input_str = input.as_str()?;
    let format_str = format.as_str()?;

    let dt = NaiveDateTime::parse_from_str(input_str.trim(), format_str.trim())
        .map_err(|e| WrapperError::DateTimeError(format!("Parse error: {}", e)))?;

    Ok(Utc.from_utc_datetime(&dt).timestamp_millis())
}

/// Parse RFC 2822 datetime (email format)
pub fn parse_rfc2822(input: &OwnedBuffer) -> WrapperResult<i64> {
    let input_str = input.as_str()?;
    let dt = DateTime::parse_from_rfc2822(input_str.trim())
        .map_err(|e| WrapperError::DateTimeError(format!("Invalid RFC 2822: {}", e)))?;
    Ok(dt.timestamp_millis())
}

// =============================================================================
// Formatting
// =============================================================================

/// Format timestamp (ms) as ISO 8601
pub fn format_iso(timestamp_ms: i64) -> OwnedBuffer {
    let dt = timestamp_to_datetime(timestamp_ms);
    OwnedBuffer::from_string(dt.to_rfc3339())
}

/// Format timestamp (ms) with custom format
pub fn format(timestamp_ms: i64, fmt: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    let fmt_str = fmt.as_str()?;
    let dt = timestamp_to_datetime(timestamp_ms);
    Ok(OwnedBuffer::from_string(dt.format(fmt_str).to_string()))
}

/// Format timestamp (ms) as RFC 2822
pub fn format_rfc2822(timestamp_ms: i64) -> OwnedBuffer {
    let dt = timestamp_to_datetime(timestamp_ms);
    OwnedBuffer::from_string(dt.to_rfc2822())
}

// =============================================================================
// Arithmetic
// =============================================================================

/// Add days to timestamp (ms)
pub fn add_days(timestamp_ms: i64, days: i64) -> i64 {
    timestamp_ms + (days * 24 * 60 * 60 * 1000)
}

/// Add hours to timestamp (ms)
pub fn add_hours(timestamp_ms: i64, hours: i64) -> i64 {
    timestamp_ms + (hours * 60 * 60 * 1000)
}

/// Add minutes to timestamp (ms)
pub fn add_minutes(timestamp_ms: i64, minutes: i64) -> i64 {
    timestamp_ms + (minutes * 60 * 1000)
}

/// Add seconds to timestamp (ms)
pub fn add_seconds(timestamp_ms: i64, seconds: i64) -> i64 {
    timestamp_ms + (seconds * 1000)
}

/// Get difference in seconds between two timestamps (ms)
pub fn diff_seconds(a: i64, b: i64) -> i64 {
    (a - b) / 1000
}

/// Get difference in days between two timestamps (ms)
pub fn diff_days(a: i64, b: i64) -> i64 {
    (a - b) / (24 * 60 * 60 * 1000)
}

// =============================================================================
// Component Extraction
// =============================================================================

/// Get year from timestamp (ms)
pub fn year(timestamp_ms: i64) -> i32 {
    timestamp_to_datetime(timestamp_ms).year()
}

/// Get month (1-12) from timestamp (ms)
pub fn month(timestamp_ms: i64) -> u32 {
    timestamp_to_datetime(timestamp_ms).month()
}

/// Get day of month (1-31) from timestamp (ms)
pub fn day(timestamp_ms: i64) -> u32 {
    timestamp_to_datetime(timestamp_ms).day()
}

/// Get hour (0-23) from timestamp (ms)
pub fn hour(timestamp_ms: i64) -> u32 {
    timestamp_to_datetime(timestamp_ms).hour()
}

/// Get minute (0-59) from timestamp (ms)
pub fn minute(timestamp_ms: i64) -> u32 {
    timestamp_to_datetime(timestamp_ms).minute()
}

/// Get second (0-59) from timestamp (ms)
pub fn second(timestamp_ms: i64) -> u32 {
    timestamp_to_datetime(timestamp_ms).second()
}

/// Get weekday (0=Monday, 6=Sunday in chrono)
pub fn weekday(timestamp_ms: i64) -> u32 {
    timestamp_to_datetime(timestamp_ms)
        .weekday()
        .num_days_from_monday()
}

/// Get day of year (1-366)
pub fn day_of_year(timestamp_ms: i64) -> u32 {
    timestamp_to_datetime(timestamp_ms).ordinal()
}

// =============================================================================
// Validation
// =============================================================================

/// Check if year is a leap year
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Get number of days in month
pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

// =============================================================================
// Helper
// =============================================================================

fn timestamp_to_datetime(timestamp_ms: i64) -> DateTime<Utc> {
    let secs = timestamp_ms / 1000;
    let nsecs = ((timestamp_ms % 1000) * 1_000_000) as u32;
    DateTime::from_timestamp(secs, nsecs).unwrap_or_else(Utc::now)
}

// =============================================================================
// Registration
// =============================================================================

/// Register all datetime wrappers with the registry
pub fn register(registry: &mut WrapperRegistry) {
    registry.register_wrapper(
        "datetime_now",
        "Get current UTC timestamp (milliseconds)",
        WrapperCategory::DateTime,
        0,
        &["now", "current time", "timestamp"],
        |_args| {
            let ts = now();
            Ok(OwnedBuffer::from_vec(ts.to_le_bytes().to_vec()))
        },
    );

    registry.register_wrapper(
        "datetime_now_iso",
        "Get current time as ISO 8601 string",
        WrapperCategory::DateTime,
        0,
        &["now iso", "current iso", "iso timestamp"],
        |_args| Ok(now_iso()),
    );

    registry.register_wrapper(
        "datetime_parse_iso",
        "Parse ISO 8601 datetime string to timestamp",
        WrapperCategory::DateTime,
        1,
        &["parse iso", "parse datetime", "from iso"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No input provided".to_string()));
            }
            let ts = parse_iso(&args[0])?;
            Ok(OwnedBuffer::from_vec(ts.to_le_bytes().to_vec()))
        },
    );

    registry.register_wrapper(
        "datetime_format_iso",
        "Format timestamp as ISO 8601",
        WrapperCategory::DateTime,
        1,
        &["format iso", "to iso", "iso format"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg(
                    "No timestamp provided".to_string(),
                ));
            }
            let ts = i64::from_le_bytes(
                args[0]
                    .as_slice()
                    .try_into()
                    .map_err(|_| WrapperError::InvalidArg("Invalid timestamp".to_string()))?,
            );
            Ok(format_iso(ts))
        },
    );

    registry.register_wrapper(
        "datetime_format",
        "Format timestamp with custom format string",
        WrapperCategory::DateTime,
        2,
        &["format", "strftime", "to string"],
        |args| {
            if args.len() < 2 {
                return Err(WrapperError::InvalidArg(
                    "Need timestamp and format".to_string(),
                ));
            }
            let ts = i64::from_le_bytes(
                args[0]
                    .as_slice()
                    .try_into()
                    .map_err(|_| WrapperError::InvalidArg("Invalid timestamp".to_string()))?,
            );
            format(ts, &args[1])
        },
    );

    registry.register_wrapper(
        "datetime_add_days",
        "Add days to timestamp",
        WrapperCategory::DateTime,
        2,
        &["add days", "plus days"],
        |args| {
            if args.len() < 2 {
                return Err(WrapperError::InvalidArg(
                    "Need timestamp and days".to_string(),
                ));
            }
            let ts = i64::from_le_bytes(
                args[0]
                    .as_slice()
                    .try_into()
                    .map_err(|_| WrapperError::InvalidArg("Invalid timestamp".to_string()))?,
            );
            let days = i64::from_le_bytes(
                args[1]
                    .as_slice()
                    .try_into()
                    .map_err(|_| WrapperError::InvalidArg("Invalid days".to_string()))?,
            );
            let result = add_days(ts, days);
            Ok(OwnedBuffer::from_vec(result.to_le_bytes().to_vec()))
        },
    );

    registry.register_wrapper(
        "datetime_year",
        "Get year from timestamp",
        WrapperCategory::DateTime,
        1,
        &["year", "get year", "extract year"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg(
                    "No timestamp provided".to_string(),
                ));
            }
            let ts = i64::from_le_bytes(
                args[0]
                    .as_slice()
                    .try_into()
                    .map_err(|_| WrapperError::InvalidArg("Invalid timestamp".to_string()))?,
            );
            let y = year(ts);
            Ok(OwnedBuffer::from_vec((y as i64).to_le_bytes().to_vec()))
        },
    );

    registry.register_wrapper(
        "datetime_month",
        "Get month (1-12) from timestamp",
        WrapperCategory::DateTime,
        1,
        &["month", "get month"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg(
                    "No timestamp provided".to_string(),
                ));
            }
            let ts = i64::from_le_bytes(
                args[0]
                    .as_slice()
                    .try_into()
                    .map_err(|_| WrapperError::InvalidArg("Invalid timestamp".to_string()))?,
            );
            let m = month(ts);
            Ok(OwnedBuffer::from_vec((m as i64).to_le_bytes().to_vec()))
        },
    );

    registry.register_wrapper(
        "datetime_day",
        "Get day of month from timestamp",
        WrapperCategory::DateTime,
        1,
        &["day", "get day"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg(
                    "No timestamp provided".to_string(),
                ));
            }
            let ts = i64::from_le_bytes(
                args[0]
                    .as_slice()
                    .try_into()
                    .map_err(|_| WrapperError::InvalidArg("Invalid timestamp".to_string()))?,
            );
            let d = day(ts);
            Ok(OwnedBuffer::from_vec((d as i64).to_le_bytes().to_vec()))
        },
    );

    registry.register_wrapper(
        "datetime_weekday",
        "Get weekday from timestamp",
        WrapperCategory::DateTime,
        1,
        &["weekday", "day of week"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg(
                    "No timestamp provided".to_string(),
                ));
            }
            let ts = i64::from_le_bytes(
                args[0]
                    .as_slice()
                    .try_into()
                    .map_err(|_| WrapperError::InvalidArg("Invalid timestamp".to_string()))?,
            );
            let w = weekday(ts);
            Ok(OwnedBuffer::from_vec((w as i64).to_le_bytes().to_vec()))
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now() {
        let ts = now();
        assert!(ts > 0);
        assert!(ts > 1700000000000); // After Nov 2023
    }

    #[test]
    fn test_parse_iso() {
        let input = OwnedBuffer::from_str("2024-01-25T12:00:00Z");
        let ts = parse_iso(&input).unwrap();
        assert!(ts > 0);
    }

    #[test]
    fn test_format_iso() {
        let ts = 1706180400000i64; // 2024-01-25T11:00:00Z
        let formatted = format_iso(ts);
        let s = formatted.as_str().unwrap();
        assert!(s.contains("2024"));
        assert!(s.contains("01"));
        assert!(s.contains("25"));
    }

    #[test]
    fn test_parse_format_roundtrip() {
        let ts = now();
        let iso = format_iso(ts);
        let parsed = parse_iso(&iso).unwrap();
        // Allow 1 second difference due to formatting precision
        assert!((ts - parsed).abs() < 1000);
    }

    #[test]
    fn test_add_days() {
        let ts = now();
        let tomorrow = add_days(ts, 1);
        let yesterday = add_days(ts, -1);

        assert_eq!(tomorrow - ts, 24 * 60 * 60 * 1000);
        assert_eq!(ts - yesterday, 24 * 60 * 60 * 1000);
    }

    #[test]
    fn test_diff() {
        let a = 1706220000000i64;
        let b = 1706133600000i64;

        assert_eq!(diff_days(a, b), 1);
        assert_eq!(diff_seconds(a, b), 86400);
    }

    #[test]
    fn test_components() {
        let ts = 1706180400000i64; // 2024-01-25T11:00:00Z

        assert_eq!(year(ts), 2024);
        assert_eq!(month(ts), 1);
        assert_eq!(day(ts), 25);
        assert_eq!(hour(ts), 11);
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2023));
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2024, 2), 29); // Leap year
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2024, 1), 31);
        assert_eq!(days_in_month(2024, 4), 30);
    }
}
