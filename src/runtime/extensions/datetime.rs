//! DateTime Extensions
//!
//! Date and time operations using chrono.

use std::sync::Arc;

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

use super::{ext_ids, ExtCategory, ExtError, ExtensionRegistry};

// =============================================================================
// Current Time
// =============================================================================

/// Get current UTC timestamp (milliseconds since epoch)
fn now() -> i64 {
    Utc::now().timestamp_millis()
}

/// Get current UTC timestamp (seconds since epoch)
#[allow(dead_code)]
fn now_secs() -> i64 {
    Utc::now().timestamp()
}

/// Get current time as ISO 8601 string
#[allow(dead_code)]
fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

// =============================================================================
// Parsing
// =============================================================================

/// Parse ISO 8601 datetime string to timestamp (milliseconds)
fn parse_iso(input: &str) -> Result<i64, ExtError> {
    let dt = DateTime::parse_from_rfc3339(input.trim())
        .map_err(|e| ExtError::ExtensionError(format!("Invalid ISO 8601: {}", e)))?;
    Ok(dt.timestamp_millis())
}

/// Parse datetime with custom format
#[allow(dead_code)]
fn parse_format(input: &str, format: &str) -> Result<i64, ExtError> {
    let dt = NaiveDateTime::parse_from_str(input.trim(), format.trim())
        .map_err(|e| ExtError::ExtensionError(format!("Parse error: {}", e)))?;

    Ok(Utc.from_utc_datetime(&dt).timestamp_millis())
}

// =============================================================================
// Formatting
// =============================================================================

/// Format timestamp (ms) as ISO 8601
fn format_iso(timestamp_ms: i64) -> String {
    let dt = timestamp_to_datetime(timestamp_ms);
    dt.to_rfc3339()
}

/// Format timestamp (ms) with custom format
#[allow(dead_code)]
fn format_custom(timestamp_ms: i64, fmt: &str) -> String {
    let dt = timestamp_to_datetime(timestamp_ms);
    dt.format(fmt).to_string()
}

// =============================================================================
// Arithmetic
// =============================================================================

/// Add days to timestamp (ms)
fn add_days(timestamp_ms: i64, days: i64) -> i64 {
    timestamp_ms + (days * 24 * 60 * 60 * 1000)
}

/// Get difference in seconds between two timestamps (ms)
fn diff_seconds(a: i64, b: i64) -> i64 {
    (a - b) / 1000
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
// Extension Registration
// =============================================================================

pub fn register_datetime(registry: &mut ExtensionRegistry) {
    // datetime_now
    registry.register_with_id(
        ext_ids::DATETIME_NOW,
        "datetime_now",
        "Get current UTC timestamp in milliseconds. Returns i64.",
        0,
        true,
        ExtCategory::DateTime,
        Arc::new(|_args, outputs| {
            let ts = now();
            outputs[0] = ts as u64;
            Ok(ts)
        }),
    );

    // datetime_parse
    registry.register_with_id(
        ext_ids::DATETIME_PARSE,
        "datetime_parse",
        "Parse ISO 8601 string to timestamp. Args: string_handle. Returns timestamp ms.",
        1,
        true,
        ExtCategory::DateTime,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let storage = super::strings::STRING_STORAGE.read().unwrap();
            let s = storage
                .get(&handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid string handle".to_string()))?;
            let ts = parse_iso(s)?;
            outputs[0] = ts as u64;
            Ok(ts)
        }),
    );

    // datetime_format
    registry.register_with_id(
        ext_ids::DATETIME_FORMAT,
        "datetime_format",
        "Format timestamp as ISO 8601. Args: timestamp_ms. Returns string_handle.",
        1,
        true,
        ExtCategory::DateTime,
        Arc::new(|args, outputs| {
            let ts = args[0] as i64;
            let formatted = format_iso(ts);
            let handle = super::strings::next_handle();
            super::strings::STRING_STORAGE
                .write()
                .unwrap()
                .insert(handle, formatted);
            outputs[0] = handle;
            Ok(handle as i64)
        }),
    );

    // datetime_add_days
    registry.register_with_id(
        ext_ids::DATETIME_ADD_DAYS,
        "datetime_add_days",
        "Add days to timestamp. Args: timestamp_ms, days. Returns timestamp_ms.",
        2,
        true,
        ExtCategory::DateTime,
        Arc::new(|args, outputs| {
            let ts = args[0] as i64;
            let days = args[1] as i64;
            let result = add_days(ts, days);
            outputs[0] = result as u64;
            Ok(result)
        }),
    );

    // datetime_diff
    registry.register_with_id(
        ext_ids::DATETIME_DIFF,
        "datetime_diff",
        "Get difference in seconds between timestamps. Args: ts1, ts2. Returns seconds.",
        2,
        true,
        ExtCategory::DateTime,
        Arc::new(|args, outputs| {
            let ts1 = args[0] as i64;
            let ts2 = args[1] as i64;
            let diff = diff_seconds(ts1, ts2);
            outputs[0] = diff as u64;
            Ok(diff)
        }),
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
        let ts = parse_iso("2024-01-25T12:00:00Z").unwrap();
        assert!(ts > 0);
    }

    #[test]
    fn test_format_iso() {
        let ts = 1706180400000i64;
        let formatted = format_iso(ts);
        assert!(formatted.contains("2024"));
    }

    #[test]
    fn test_add_days() {
        let ts = now();
        let tomorrow = add_days(ts, 1);
        assert_eq!(tomorrow - ts, 24 * 60 * 60 * 1000);
    }
}
