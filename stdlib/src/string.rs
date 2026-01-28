//! String functions for Neurlang stdlib
//!
//! These functions operate on null-terminated strings in memory.
//! Pointers are represented as u64 values.

/// Calculate string length (null-terminated).
///
/// # Safety
/// Assumes ptr points to valid null-terminated string.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 1
///
/// # Prompts
/// - get the length of {str}
/// - count characters in {str}
/// - how many characters in {str}
/// - find string length of {str}
/// - calculate the size of {str}
/// - measure the length of string {str}
/// - get character count of {str}
/// - return length of {str}
/// - compute string size for {str}
/// - determine how long {str} is
/// - what is the length of {str}
/// - count bytes in string {str}
/// - strlen of {str}
///
/// # Parameters
/// - str=r0 "Pointer to null-terminated string"
///
/// # Test Cases
/// - strlen("hello") = 5
/// - strlen("") = 0
/// - strlen("a") = 1
#[inline(never)]
pub unsafe fn strlen(ptr: *const u8) -> u64 {
    let mut len: u64 = 0;
    let mut p = ptr;

    while *p != 0 {
        len = len + 1;
        p = p.add(1);
    }

    len
}

/// Compare two strings for equality.
///
/// # Safety
/// Assumes both pointers are valid null-terminated strings.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 1
///
/// # Prompts
/// - compare {str1} and {str2}
/// - check if {str1} equals {str2}
/// - are strings {str1} and {str2} the same
/// - test string equality of {str1} and {str2}
/// - do {str1} and {str2} match
/// - is {str1} equal to {str2}
/// - compare strings {str1} with {str2}
/// - check string equality between {str1} and {str2}
/// - determine if {str1} is identical to {str2}
/// - verify {str1} matches {str2}
/// - strcmp {str1} and {str2}
/// - are {str1} and {str2} equal strings
/// - test if {str1} == {str2}
///
/// # Parameters
/// - str1=r0 "Pointer to first null-terminated string"
/// - str2=r1 "Pointer to second null-terminated string"
///
/// # Test Cases
/// - strcmp("abc", "abc") = 1
/// - strcmp("abc", "abd") = 0
/// - strcmp("", "") = 1
#[inline(never)]
pub unsafe fn strcmp(a: *const u8, b: *const u8) -> u64 {
    let mut pa = a;
    let mut pb = b;

    loop {
        let ca = *pa;
        let cb = *pb;

        if ca != cb {
            return 0;
        }

        if ca == 0 {
            return 1; // Both reached null terminator
        }

        pa = pa.add(1);
        pb = pb.add(1);
    }
}

/// Copy string from src to dst.
///
/// # Safety
/// Assumes dst has enough space for src.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 1
///
/// # Prompts
/// - copy string from {src} to {dst}
/// - duplicate string {src} into {dst}
/// - copy {src} to destination {dst}
/// - clone string {src} to {dst}
/// - transfer string from {src} to {dst}
/// - copy contents of {src} into {dst}
/// - strcpy from {src} to {dst}
/// - replicate string {src} at {dst}
/// - write string {src} to buffer {dst}
/// - copy text from {src} to {dst}
/// - duplicate {src} into {dst} buffer
/// - move string {src} to {dst}
/// - copy string data from {src} to {dst}
///
/// # Parameters
/// - dst=r0 "Pointer to destination buffer"
/// - src=r1 "Pointer to source null-terminated string"
///
/// Returns: length of copied string
#[inline(never)]
pub unsafe fn strcpy(dst: *mut u8, src: *const u8) -> u64 {
    let mut pd = dst;
    let mut ps = src;
    let mut len: u64 = 0;

    loop {
        let c = *ps;
        *pd = c;

        if c == 0 {
            break;
        }

        len = len + 1;
        pd = pd.add(1);
        ps = ps.add(1);
    }

    len
}

/// Convert ASCII digit character to integer value.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 1
///
/// # Prompts
/// - convert character {c} to digit
/// - get numeric value of char {c}
/// - parse digit character {c}
/// - char {c} to integer
/// - convert {c} from ascii digit to number
/// - get digit value of {c}
/// - transform character {c} to integer
/// - extract numeric value from {c}
/// - ascii digit {c} to number
/// - convert ascii {c} to its numeric value
/// - parse character {c} as digit
/// - get the number value of {c}
/// - char to digit for {c}
///
/// # Parameters
/// - c=r0 "ASCII character to convert"
///
/// # Test Cases
/// - char_to_digit('0') = 0
/// - char_to_digit('5') = 5
/// - char_to_digit('9') = 9
/// - char_to_digit('a') = 0xFFFFFFFF (invalid)
#[inline(never)]
pub fn char_to_digit(c: u8) -> u64 {
    if c >= b'0' && c <= b'9' {
        (c - b'0') as u64
    } else {
        u64::MAX // Invalid
    }
}

/// Check if character is a digit.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 1
///
/// # Prompts
/// - is {c} a digit
/// - check if {c} is numeric
/// - is character {c} a number
/// - test if {c} is a digit character
/// - determine if {c} is 0-9
/// - is {c} a numeric character
/// - check whether {c} is a digit
/// - is char {c} in range 0 to 9
/// - verify {c} is a digit
/// - does {c} represent a number
/// - isdigit for {c}
/// - is {c} between 0 and 9
/// - check {c} is numeric digit
///
/// # Parameters
/// - c=r0 "ASCII character to check"
#[inline(never)]
pub fn is_digit(c: u8) -> u64 {
    if c < b'0' { return 0; }
    if c > b'9' { return 0; }
    1
}

/// Check if character is alphabetic.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 1
///
/// # Prompts
/// - is {c} a letter
/// - check if {c} is alphabetic
/// - is character {c} a letter
/// - test if {c} is alpha
/// - determine if {c} is a-z or A-Z
/// - is {c} an alphabetic character
/// - check whether {c} is a letter
/// - is char {c} alphabetic
/// - verify {c} is a letter
/// - does {c} represent a letter
/// - isalpha for {c}
/// - is {c} in the alphabet
/// - check {c} is a letter character
///
/// # Parameters
/// - c=r0 "ASCII character to check"
#[inline(never)]
pub fn is_alpha(c: u8) -> u64 {
    // Check lowercase
    if c >= b'a' && c <= b'z' { return 1; }
    // Check uppercase
    if c >= b'A' && c <= b'Z' { return 1; }
    0
}

/// Check if character is alphanumeric.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 1
///
/// # Prompts
/// - is {c} alphanumeric
/// - check if {c} is letter or digit
/// - is character {c} alphanumeric
/// - test if {c} is alnum
/// - determine if {c} is a-z, A-Z, or 0-9
/// - is {c} a letter or number
/// - check whether {c} is alphanumeric
/// - is char {c} letter or digit
/// - verify {c} is alphanumeric
/// - does {c} represent letter or number
/// - isalnum for {c}
/// - is {c} word character
/// - check {c} is alphanumeric character
///
/// # Parameters
/// - c=r0 "ASCII character to check"
#[inline(never)]
pub fn is_alnum(c: u8) -> u64 {
    // Check digit (0-9)
    if c >= b'0' && c <= b'9' { return 1; }
    // Check lowercase (a-z)
    if c >= b'a' && c <= b'z' { return 1; }
    // Check uppercase (A-Z)
    if c >= b'A' && c <= b'Z' { return 1; }
    0
}

/// Check if character is whitespace.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 1
///
/// # Prompts
/// - is {c} whitespace
/// - check if {c} is a space character
/// - is character {c} whitespace
/// - test if {c} is space or tab
/// - determine if {c} is whitespace
/// - is {c} a blank character
/// - check whether {c} is whitespace
/// - is char {c} space, tab, or newline
/// - verify {c} is whitespace
/// - does {c} represent whitespace
/// - isspace for {c}
/// - is {c} a spacing character
/// - check {c} is whitespace character
///
/// # Parameters
/// - c=r0 "ASCII character to check"
#[inline(never)]
pub fn is_space(c: u8) -> u64 {
    if c == b' ' { return 1; }
    if c == b'\t' { return 1; }
    if c == b'\n' { return 1; }
    if c == b'\r' { return 1; }
    0
}

/// Convert lowercase letter to uppercase.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 1
///
/// # Prompts
/// - convert {c} to uppercase
/// - make {c} uppercase
/// - uppercase character {c}
/// - transform {c} to upper case
/// - capitalize character {c}
/// - change {c} to uppercase
/// - toupper for {c}
/// - get uppercase of {c}
/// - convert lowercase {c} to upper
/// - make letter {c} uppercase
/// - shift {c} to uppercase
/// - upper case conversion of {c}
/// - convert {c} from lower to upper
///
/// # Parameters
/// - c=r0 "ASCII character to convert"
#[inline(never)]
pub fn to_upper(c: u8) -> u8 {
    if c < b'a' { return c; }
    if c > b'z' { return c; }
    c - 32
}

/// Convert uppercase letter to lowercase.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 1
///
/// # Prompts
/// - convert {c} to lowercase
/// - make {c} lowercase
/// - lowercase character {c}
/// - transform {c} to lower case
/// - uncapitalize character {c}
/// - change {c} to lowercase
/// - tolower for {c}
/// - get lowercase of {c}
/// - convert uppercase {c} to lower
/// - make letter {c} lowercase
/// - shift {c} to lowercase
/// - lower case conversion of {c}
/// - convert {c} from upper to lower
///
/// # Parameters
/// - c=r0 "ASCII character to convert"
#[inline(never)]
pub fn to_lower(c: u8) -> u8 {
    if c < b'A' { return c; }
    if c > b'Z' { return c; }
    c + 32
}

/// Parse unsigned integer from string (atoi).
///
/// # Safety
/// Assumes ptr points to valid string.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 2
///
/// # Prompts
/// - parse integer from {str}
/// - convert string {str} to number
/// - atoi of {str}
/// - string {str} to integer
/// - extract number from {str}
/// - parse number from string {str}
/// - convert {str} to unsigned integer
/// - read integer from {str}
/// - get numeric value of {str}
/// - transform {str} to integer
/// - decode integer from {str}
/// - string to int for {str}
/// - parse unsigned number from {str}
///
/// # Parameters
/// - str=r0 "Pointer to null-terminated string containing digits"
///
/// # Test Cases
/// - atoi("123") = 123
/// - atoi("0") = 0
/// - atoi("42abc") = 42
#[inline(never)]
pub unsafe fn atoi(ptr: *const u8) -> u64 {
    let mut result: u64 = 0;
    let mut p = ptr;

    // Skip leading whitespace (inline is_space)
    loop {
        let c = *p;
        if c != b' ' && c != b'\t' && c != b'\n' && c != b'\r' {
            break;
        }
        p = p.add(1);
    }

    // Parse digits (inline is_digit)
    loop {
        let c = *p;
        if c < b'0' || c > b'9' {
            break;
        }
        let digit = (c - b'0') as u64;
        result = result * 10 + digit;
        p = p.add(1);
    }

    result
}

/// Convert unsigned integer to string (itoa).
///
/// # Safety
/// Assumes dst has enough space (at least 21 bytes for u64).
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 2
///
/// # Prompts
/// - convert integer {n} to string at {dst}
/// - itoa of {n} into {dst}
/// - write number {n} as string to {dst}
/// - integer {n} to string at {dst}
/// - format number {n} to buffer {dst}
/// - convert {n} to decimal string in {dst}
/// - stringify integer {n} at {dst}
/// - render number {n} as text to {dst}
/// - int to string for {n} into {dst}
/// - write integer {n} to string buffer {dst}
/// - transform {n} to string at {dst}
/// - encode integer {n} as string in {dst}
/// - number to text conversion of {n} to {dst}
///
/// # Parameters
/// - n=r0 "Unsigned integer to convert"
/// - dst=r1 "Pointer to destination buffer"
///
/// Returns: length of string written
#[inline(never)]
pub unsafe fn itoa(mut n: u64, dst: *mut u8) -> u64 {
    if n == 0 {
        *dst = b'0';
        *dst.add(1) = 0;
        return 1;
    }

    // Count digits
    let mut temp = n;
    let mut len: u64 = 0;
    while temp > 0 {
        len = len + 1;
        temp = temp / 10;
    }

    // Write digits in reverse
    let mut i = len;
    while n > 0 {
        i = i - 1;
        *dst.add(i as usize) = b'0' + (n % 10) as u8;
        n = n / 10;
    }

    // Null terminate
    *dst.add(len as usize) = 0;

    len
}

/// Find first occurrence of character in string.
///
/// # Safety
/// Assumes ptr points to valid null-terminated string.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 2
///
/// # Prompts
/// - find character {c} in {str}
/// - locate {c} in string {str}
/// - search for {c} in {str}
/// - get index of {c} in {str}
/// - position of {c} in {str}
/// - where is {c} in {str}
/// - find first occurrence of {c} in {str}
/// - strchr for {c} in {str}
/// - index of character {c} in {str}
/// - locate first {c} in {str}
/// - search string {str} for {c}
/// - find position of {c} in {str}
/// - get offset of {c} in {str}
///
/// # Parameters
/// - str=r0 "Pointer to null-terminated string to search"
/// - c=r1 "Character to find"
///
/// Returns: index of character, or u64::MAX if not found
#[inline(never)]
pub unsafe fn strchr(ptr: *const u8, c: u8) -> u64 {
    let mut p = ptr;
    let mut idx: u64 = 0;

    while *p != 0 {
        if *p == c {
            return idx;
        }
        p = p.add(1);
        idx = idx + 1;
    }

    u64::MAX // Not found
}

/// Find last occurrence of character in string.
///
/// # Safety
/// Assumes ptr points to valid null-terminated string.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 2
///
/// # Prompts
/// - find last {c} in {str}
/// - locate last occurrence of {c} in {str}
/// - search for last {c} in {str}
/// - get index of last {c} in {str}
/// - position of final {c} in {str}
/// - where is last {c} in {str}
/// - find rightmost {c} in {str}
/// - strrchr for {c} in {str}
/// - index of last character {c} in {str}
/// - locate final {c} in {str}
/// - search string {str} for last {c}
/// - find position of last {c} in {str}
/// - get offset of rightmost {c} in {str}
///
/// # Parameters
/// - str=r0 "Pointer to null-terminated string to search"
/// - c=r1 "Character to find"
///
/// Returns: index of character, or u64::MAX if not found
#[inline(never)]
pub unsafe fn strrchr(ptr: *const u8, c: u8) -> u64 {
    let mut p = ptr;
    let mut idx: u64 = 0;
    let mut last: u64 = u64::MAX;

    while *p != 0 {
        if *p == c {
            last = idx;
        }
        p = p.add(1);
        idx = idx + 1;
    }

    last
}

/// Check if string starts with prefix.
///
/// # Safety
/// Assumes both pointers are valid null-terminated strings.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 2
///
/// # Prompts
/// - does {str} start with {prefix}
/// - check if {str} begins with {prefix}
/// - test if {str} has prefix {prefix}
/// - verify {str} starts with {prefix}
/// - is {prefix} a prefix of {str}
/// - check string {str} starts with {prefix}
/// - does {str} begin with {prefix}
/// - has prefix {prefix} in {str}
/// - starts with check for {prefix} in {str}
/// - determine if {str} starts with {prefix}
/// - test prefix {prefix} against {str}
/// - is {str} prefixed by {prefix}
/// - check beginning of {str} for {prefix}
///
/// # Parameters
/// - str=r0 "Pointer to string to check"
/// - prefix=r1 "Pointer to prefix string"
#[inline(never)]
pub unsafe fn starts_with(str_ptr: *const u8, prefix_ptr: *const u8) -> u64 {
    let mut ps = str_ptr;
    let mut pp = prefix_ptr;

    loop {
        let cp = *pp;
        if cp == 0 {
            return 1; // Prefix exhausted = match
        }

        let cs = *ps;
        if cs != cp {
            return 0;
        }

        ps = ps.add(1);
        pp = pp.add(1);
    }
}

/// Count occurrences of character in string.
///
/// # Safety
/// Assumes ptr points to valid null-terminated string.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 1
///
/// # Prompts
/// - count {c} in {str}
/// - how many {c} in {str}
/// - count occurrences of {c} in {str}
/// - number of {c} in string {str}
/// - tally {c} in {str}
/// - get count of {c} in {str}
/// - count character {c} in {str}
/// - occurrences of {c} in {str}
/// - how many times does {c} appear in {str}
/// - frequency of {c} in {str}
/// - count instances of {c} in {str}
/// - total {c} characters in {str}
/// - count all {c} in string {str}
///
/// # Parameters
/// - str=r0 "Pointer to null-terminated string"
/// - c=r1 "Character to count"
#[inline(never)]
pub unsafe fn count_char(ptr: *const u8, c: u8) -> u64 {
    let mut p = ptr;
    let mut count: u64 = 0;

    while *p != 0 {
        if *p == c {
            count = count + 1;
        }
        p = p.add(1);
    }

    count
}

/// Check if string ends with suffix.
///
/// # Safety
/// Assumes both pointers are valid null-terminated strings.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 2
///
/// # Prompts
/// - does {str} end with {suffix}
/// - check if {str} ends with {suffix}
/// - test if {str} has suffix {suffix}
/// - verify {str} ends with {suffix}
/// - is {suffix} a suffix of {str}
/// - check string {str} ends with {suffix}
/// - does {str} finish with {suffix}
/// - has suffix {suffix} in {str}
/// - ends with check for {suffix} in {str}
/// - determine if {str} ends with {suffix}
/// - test suffix {suffix} against {str}
/// - is {str} suffixed by {suffix}
/// - check ending of {str} for {suffix}
///
/// # Parameters
/// - str=r0 "Pointer to string to check"
/// - suffix=r1 "Pointer to suffix string"
#[inline(never)]
pub unsafe fn ends_with(str_ptr: *const u8, suffix_ptr: *const u8) -> u64 {
    // Inline strlen for str_ptr
    let mut str_len: u64 = 0;
    let mut p = str_ptr;
    while *p != 0 {
        str_len = str_len + 1;
        p = p.add(1);
    }

    // Inline strlen for suffix_ptr
    let mut suffix_len: u64 = 0;
    p = suffix_ptr;
    while *p != 0 {
        suffix_len = suffix_len + 1;
        p = p.add(1);
    }

    if suffix_len > str_len {
        return 0;
    }

    // Compare from the end
    let start = str_len - suffix_len;
    let mut i: u64 = 0;

    while i < suffix_len {
        if *str_ptr.add((start + i) as usize) != *suffix_ptr.add(i as usize) {
            return 0;
        }
        i = i + 1;
    }

    1
}

/// Find substring in string (naive algorithm).
///
/// # Safety
/// Assumes both pointers are valid null-terminated strings.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 3
///
/// # Prompts
/// - find {needle} in {haystack}
/// - locate substring {needle} in {haystack}
/// - search for {needle} in {haystack}
/// - get index of {needle} in {haystack}
/// - position of substring {needle} in {haystack}
/// - where is {needle} in {haystack}
/// - find first occurrence of {needle} in {haystack}
/// - strstr for {needle} in {haystack}
/// - index of substring {needle} in {haystack}
/// - search string {haystack} for {needle}
/// - find position of {needle} in {haystack}
/// - locate string {needle} within {haystack}
/// - find pattern {needle} in text {haystack}
///
/// # Parameters
/// - haystack=r0 "Pointer to string to search in"
/// - needle=r1 "Pointer to substring to find"
///
/// Returns: index of first occurrence, or u64::MAX if not found
#[inline(never)]
pub unsafe fn strstr(haystack: *const u8, needle: *const u8) -> u64 {
    // Inline strlen for needle
    let mut needle_len: u64 = 0;
    let mut p = needle;
    while *p != 0 {
        needle_len = needle_len + 1;
        p = p.add(1);
    }

    // Empty needle matches at position 0
    if needle_len == 0 {
        return 0;
    }

    // Inline strlen for haystack
    let mut haystack_len: u64 = 0;
    p = haystack;
    while *p != 0 {
        haystack_len = haystack_len + 1;
        p = p.add(1);
    }
    if needle_len > haystack_len {
        return u64::MAX;
    }

    let mut i: u64 = 0;
    let max_start = haystack_len - needle_len;

    while i <= max_start {
        let mut j: u64 = 0;
        let mut matched = 1u64;

        while j < needle_len {
            if *haystack.add((i + j) as usize) != *needle.add(j as usize) {
                matched = 0;
                break;
            }
            j = j + 1;
        }

        if matched == 1 {
            return i;
        }
        i = i + 1;
    }

    u64::MAX
}

/// Concatenate src string to end of dst.
///
/// # Safety
/// Assumes dst has enough space for both strings.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 2
///
/// # Prompts
/// - concatenate {src} to {dst}
/// - append {src} to {dst}
/// - join {src} onto {dst}
/// - add {src} to end of {dst}
/// - strcat {src} to {dst}
/// - combine {dst} and {src}
/// - append string {src} to buffer {dst}
/// - concatenate strings {dst} and {src}
/// - extend {dst} with {src}
/// - add string {src} after {dst}
/// - join strings {dst} and {src}
/// - merge {src} into {dst}
/// - append text {src} to {dst}
///
/// # Parameters
/// - dst=r0 "Pointer to destination buffer (existing string)"
/// - src=r1 "Pointer to source string to append"
///
/// Returns: total length of resulting string
#[inline(never)]
pub unsafe fn strcat(dst: *mut u8, src: *const u8) -> u64 {
    // Inline strlen for dst
    let mut dst_len: u64 = 0;
    let mut p = dst as *const u8;
    while *p != 0 {
        dst_len = dst_len + 1;
        p = p.add(1);
    }

    // Inline strlen for src
    let mut src_len: u64 = 0;
    p = src;
    while *p != 0 {
        src_len = src_len + 1;
        p = p.add(1);
    }

    let mut i: u64 = 0;
    while i <= src_len {  // Include null terminator
        *dst.add((dst_len + i) as usize) = *src.add(i as usize);
        i = i + 1;
    }

    dst_len + src_len
}

/// Copy at most n characters from src to dst.
///
/// # Safety
/// Assumes dst has at least n bytes available.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 2
///
/// # Prompts
/// - copy {n} characters from {src} to {dst}
/// - strncpy {n} bytes from {src} to {dst}
/// - copy at most {n} chars from {src} to {dst}
/// - limited copy of {src} to {dst} with max {n}
/// - copy up to {n} characters from {src} to {dst}
/// - bounded string copy from {src} to {dst} limit {n}
/// - copy string {src} to {dst} max length {n}
/// - safe copy {n} chars from {src} to {dst}
/// - copy first {n} characters of {src} to {dst}
/// - truncated copy from {src} to {dst} at {n}
/// - copy {src} to {dst} with limit {n}
/// - transfer up to {n} bytes from {src} to {dst}
/// - partial string copy {src} to {dst} max {n}
///
/// # Parameters
/// - dst=r0 "Pointer to destination buffer"
/// - src=r1 "Pointer to source string"
/// - n=r2 "Maximum number of characters to copy"
///
/// Returns: number of characters copied (excluding null)
#[inline(never)]
pub unsafe fn strncpy(dst: *mut u8, src: *const u8, n: u64) -> u64 {
    let mut i: u64 = 0;

    while i < n {
        let c = *src.add(i as usize);
        *dst.add(i as usize) = c;
        if c == 0 {
            return i;
        }
        i = i + 1;
    }

    // Null-terminate if we hit the limit
    if n > 0 {
        *dst.add((n - 1) as usize) = 0;
    }

    i
}

/// Trim leading whitespace, returns pointer offset.
///
/// # Safety
/// Assumes ptr points to valid null-terminated string.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 2
///
/// # Prompts
/// - trim leading whitespace from {str}
/// - skip whitespace at start of {str}
/// - remove leading spaces from {str}
/// - ltrim {str}
/// - strip leading whitespace from {str}
/// - get offset past leading spaces in {str}
/// - find first non-whitespace in {str}
/// - trim left side of {str}
/// - skip spaces at beginning of {str}
/// - remove spaces from start of {str}
/// - left trim whitespace in {str}
/// - trim start of string {str}
/// - strip spaces from left of {str}
///
/// # Parameters
/// - str=r0 "Pointer to null-terminated string"
///
/// Returns: offset to first non-whitespace character
#[inline(never)]
pub unsafe fn trim_left(ptr: *const u8) -> u64 {
    let mut offset: u64 = 0;

    while *ptr.add(offset as usize) != 0 {
        let c = *ptr.add(offset as usize);
        // Inline is_space check
        if c != b' ' && c != b'\t' && c != b'\n' && c != b'\r' {
            break;
        }
        offset = offset + 1;
    }

    offset
}

/// Find length of string excluding trailing whitespace.
///
/// # Safety
/// Assumes ptr points to valid null-terminated string.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 2
///
/// # Prompts
/// - trim trailing whitespace length from {str}
/// - get length excluding trailing spaces of {str}
/// - find trimmed length of {str}
/// - rtrim length of {str}
/// - length without trailing whitespace for {str}
/// - get non-whitespace length of {str}
/// - trim right length of {str}
/// - skip trailing spaces length in {str}
/// - remove trailing whitespace length from {str}
/// - right trimmed length of string {str}
/// - strip trailing spaces length from {str}
/// - effective length of {str} without trailing space
/// - trim end length for {str}
///
/// # Parameters
/// - str=r0 "Pointer to null-terminated string"
///
/// Returns: length excluding trailing whitespace
#[inline(never)]
pub unsafe fn trim_right_len(ptr: *const u8) -> u64 {
    // Inline strlen
    let mut len: u64 = 0;
    let mut p = ptr;
    while *p != 0 {
        len = len + 1;
        p = p.add(1);
    }

    if len == 0 {
        return 0;
    }

    let mut end = len;
    while end > 0 {
        let c = *ptr.add((end - 1) as usize);
        // Inline is_space check
        if c != b' ' && c != b'\t' && c != b'\n' && c != b'\r' {
            break;
        }
        end = end - 1;
    }

    end
}

/// Parse hexadecimal string to integer.
///
/// # Safety
/// Assumes ptr points to valid string.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 2
///
/// # Prompts
/// - parse hex string {str}
/// - convert hex {str} to integer
/// - htoi of {str}
/// - hexadecimal {str} to number
/// - read hex value from {str}
/// - decode hex string {str}
/// - hex to int for {str}
/// - parse hexadecimal from {str}
/// - convert {str} from hex to decimal
/// - extract hex number from {str}
/// - interpret {str} as hexadecimal
/// - hex string {str} to integer
/// - parse 0x prefixed string {str}
///
/// # Parameters
/// - str=r0 "Pointer to null-terminated hex string"
///
/// # Test Cases
/// - htoi("ff") = 255
/// - htoi("0x10") = 16
/// - htoi("DEADBEEF") = 3735928559
#[inline(never)]
pub unsafe fn htoi(ptr: *const u8) -> u64 {
    let mut result: u64 = 0;
    let mut p = ptr;

    // Skip leading whitespace (inline is_space)
    loop {
        let c = *p;
        if c != b' ' && c != b'\t' && c != b'\n' && c != b'\r' {
            break;
        }
        p = p.add(1);
    }

    // Skip optional "0x" or "0X" prefix
    if *p == b'0' && (*p.add(1) == b'x' || *p.add(1) == b'X') {
        p = p.add(2);
    }

    // Parse hex digits
    loop {
        let c = *p;

        // Parse digit value, break if not hex
        if c >= b'0' && c <= b'9' {
            result = result * 16 + (c - b'0') as u64;
        } else if c >= b'a' && c <= b'f' {
            result = result * 16 + (c - b'a' + 10) as u64;
        } else if c >= b'A' && c <= b'F' {
            result = result * 16 + (c - b'A' + 10) as u64;
        } else {
            break;
        }

        p = p.add(1);
    }

    result
}

/// Parse signed integer from string.
///
/// # Safety
/// Assumes ptr points to valid string.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 2
///
/// # Prompts
/// - parse signed integer from {str}
/// - convert string {str} to signed number
/// - atoi signed of {str}
/// - string {str} to signed int
/// - extract signed number from {str}
/// - parse negative number from {str}
/// - convert {str} to signed integer
/// - read signed integer from {str}
/// - get signed numeric value of {str}
/// - parse integer with sign from {str}
/// - decode signed integer from {str}
/// - string to signed int for {str}
/// - parse positive or negative from {str}
///
/// # Parameters
/// - str=r0 "Pointer to null-terminated string with optional sign"
///
/// # Test Cases
/// - atoi_signed("-123") = -123 (as two's complement u64)
/// - atoi_signed("456") = 456
/// - atoi_signed("-0") = 0
#[inline(never)]
pub unsafe fn atoi_signed(ptr: *const u8) -> u64 {
    let mut p = ptr;
    let mut negative = false;

    // Skip leading whitespace (inline is_space)
    loop {
        let c = *p;
        if c != b' ' && c != b'\t' && c != b'\n' && c != b'\r' {
            break;
        }
        p = p.add(1);
    }

    // Check for sign
    if *p == b'-' {
        negative = true;
        p = p.add(1);
    } else if *p == b'+' {
        p = p.add(1);
    }

    // Parse digits (inline is_digit)
    let mut result: u64 = 0;
    loop {
        let c = *p;
        if c < b'0' || c > b'9' {
            break;
        }
        let digit = (c - b'0') as u64;
        result = result * 10 + digit;
        p = p.add(1);
    }

    if negative {
        // Two's complement negation
        (!result).wrapping_add(1)
    } else {
        result
    }
}

/// Convert integer to hexadecimal string.
///
/// # Safety
/// Assumes dst has at least 17 bytes available.
///
/// # Neurlang Export
/// - Category: string
/// - Difficulty: 2
///
/// # Prompts
/// - convert {n} to hex string at {dst}
/// - itoa hex of {n} into {dst}
/// - write number {n} as hex to {dst}
/// - integer {n} to hex string at {dst}
/// - format number {n} as hexadecimal to {dst}
/// - convert {n} to hexadecimal string in {dst}
/// - stringify integer {n} as hex at {dst}
/// - render number {n} as hex text to {dst}
/// - int to hex string for {n} into {dst}
/// - write integer {n} as hex to buffer {dst}
/// - transform {n} to hex string at {dst}
/// - encode integer {n} as hex in {dst}
/// - number to hex conversion of {n} to {dst}
///
/// # Parameters
/// - n=r0 "Unsigned integer to convert"
/// - dst=r1 "Pointer to destination buffer"
///
/// Returns: length of hex string
#[inline(never)]
pub unsafe fn itoa_hex(mut n: u64, dst: *mut u8) -> u64 {
    if n == 0 {
        *dst = b'0';
        *dst.add(1) = 0;
        return 1;
    }

    // Count hex digits
    let mut temp = n;
    let mut len: u64 = 0;
    while temp > 0 {
        len = len + 1;
        temp = temp >> 4;
    }

    // Write digits in reverse
    let mut i = len;
    while n > 0 {
        i = i - 1;
        let digit = (n & 0xF) as u8;
        let c = if digit < 10 { b'0' + digit } else { b'a' + digit - 10 };
        *dst.add(i as usize) = c;
        n = n >> 4;
    }

    // Null terminate
    *dst.add(len as usize) = 0;

    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strlen() {
        unsafe {
            assert_eq!(strlen(b"hello\0".as_ptr()), 5);
            assert_eq!(strlen(b"\0".as_ptr()), 0);
            assert_eq!(strlen(b"a\0".as_ptr()), 1);
        }
    }

    #[test]
    fn test_strcmp() {
        unsafe {
            assert_eq!(strcmp(b"abc\0".as_ptr(), b"abc\0".as_ptr()), 1);
            assert_eq!(strcmp(b"abc\0".as_ptr(), b"abd\0".as_ptr()), 0);
            assert_eq!(strcmp(b"\0".as_ptr(), b"\0".as_ptr()), 1);
        }
    }

    #[test]
    fn test_char_classification() {
        assert_eq!(is_digit(b'5'), 1);
        assert_eq!(is_digit(b'a'), 0);
        assert_eq!(is_alpha(b'A'), 1);
        assert_eq!(is_space(b' '), 1);
    }

    #[test]
    fn test_case_conversion() {
        assert_eq!(to_upper(b'a'), b'A');
        assert_eq!(to_lower(b'Z'), b'z');
    }

    #[test]
    fn test_atoi() {
        unsafe {
            assert_eq!(atoi(b"123\0".as_ptr()), 123);
            assert_eq!(atoi(b"0\0".as_ptr()), 0);
            assert_eq!(atoi(b"42abc\0".as_ptr()), 42);
        }
    }

    #[test]
    fn test_itoa() {
        unsafe {
            let mut buf = [0u8; 32];
            let len = itoa(123, buf.as_mut_ptr());
            assert_eq!(len, 3);
            assert_eq!(&buf[0..3], b"123");
        }
    }

    #[test]
    fn test_ends_with() {
        unsafe {
            assert_eq!(ends_with(b"hello.txt\0".as_ptr(), b".txt\0".as_ptr()), 1);
            assert_eq!(ends_with(b"hello.txt\0".as_ptr(), b".rs\0".as_ptr()), 0);
            assert_eq!(ends_with(b"a\0".as_ptr(), b"abc\0".as_ptr()), 0);
        }
    }

    #[test]
    fn test_strstr() {
        unsafe {
            assert_eq!(strstr(b"hello world\0".as_ptr(), b"world\0".as_ptr()), 6);
            assert_eq!(strstr(b"hello world\0".as_ptr(), b"xyz\0".as_ptr()), u64::MAX);
            assert_eq!(strstr(b"hello\0".as_ptr(), b"\0".as_ptr()), 0);
        }
    }

    #[test]
    fn test_htoi() {
        unsafe {
            assert_eq!(htoi(b"ff\0".as_ptr()), 255);
            assert_eq!(htoi(b"0x10\0".as_ptr()), 16);
            assert_eq!(htoi(b"DEADBEEF\0".as_ptr()), 0xDEADBEEF);
        }
    }

    #[test]
    fn test_atoi_signed() {
        unsafe {
            assert_eq!(atoi_signed(b"123\0".as_ptr()), 123);
            assert_eq!(atoi_signed(b"-123\0".as_ptr()) as i64, -123);
            assert_eq!(atoi_signed(b"-0\0".as_ptr()), 0);
        }
    }
}
