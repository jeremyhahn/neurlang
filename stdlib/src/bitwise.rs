//! Bitwise functions for Neurlang stdlib
//!
//! These functions compile to bits.* opcodes where possible.

/// Count the number of set bits (population count).
///
/// # Neurlang Export
/// - Category: bitwise
/// - Opcode: bits.popcount
///
/// # Prompts
/// - count the number of set bits in {n}
/// - get the population count of {n}
/// - how many 1 bits are in {n}
/// - count ones in {n}
/// - popcount of {n}
/// - compute the hamming weight of {n}
/// - count how many bits are set in {n}
/// - number of 1s in binary representation of {n}
/// - bit count for {n}
/// - get set bit count of {n}
/// - count all set bits in value {n}
/// - find the population count for {n}
/// - calculate the number of ones in {n}
///
/// # Parameters
/// - n=r0 "The value to count set bits in"
///
/// # Test Cases
/// - popcount(0) = 0
/// - popcount(1) = 1
/// - popcount(0xFF) = 8
/// - popcount(0xFFFFFFFFFFFFFFFF) = 64
#[inline(never)]
pub fn popcount(mut n: u64) -> u64 {
    // This implementation can be replaced by a single bits.popcount instruction
    let mut count: u64 = 0;
    while n != 0 {
        count = count + (n & 1);
        n = n >> 1;
    }
    count
}

/// Count leading zeros.
///
/// # Neurlang Export
/// - Category: bitwise
/// - Opcode: bits.clz
///
/// # Prompts
/// - count leading zeros in {n}
/// - how many leading zero bits in {n}
/// - clz of {n}
/// - get the number of leading zeros in {n}
/// - find leading zero count of {n}
/// - count zeros from the most significant bit in {n}
/// - leading zero count for {n}
/// - number of zero bits at the start of {n}
/// - count leading 0s in {n}
/// - get clz for value {n}
/// - find how many zeros lead {n}
/// - compute leading zero count of {n}
/// - count zeros before first set bit in {n}
///
/// # Parameters
/// - n=r0 "The value to count leading zeros in"
///
/// # Test Cases
/// - clz(0) = 64
/// - clz(1) = 63
/// - clz(0x8000000000000000) = 0
#[inline(never)]
pub fn clz(n: u64) -> u64 {
    if n == 0 {
        return 64;
    }

    let mut count: u64 = 0;
    let mut val = n;

    // Use computed masks instead of large literals
    // Check if top 32 bits are zero
    if (val >> 32) == 0 {
        count = count + 32;
        val = val << 32;
    }
    // Check if top 16 bits are zero
    if (val >> 48) == 0 {
        count = count + 16;
        val = val << 16;
    }
    // Check if top 8 bits are zero
    if (val >> 56) == 0 {
        count = count + 8;
        val = val << 8;
    }
    // Check if top 4 bits are zero
    if (val >> 60) == 0 {
        count = count + 4;
        val = val << 4;
    }
    // Check if top 2 bits are zero
    if (val >> 62) == 0 {
        count = count + 2;
        val = val << 2;
    }
    // Check if top bit is zero
    if (val >> 63) == 0 {
        count = count + 1;
    }

    count
}

/// Count trailing zeros.
///
/// # Neurlang Export
/// - Category: bitwise
/// - Opcode: bits.ctz
///
/// # Prompts
/// - count trailing zeros in {n}
/// - how many trailing zero bits in {n}
/// - ctz of {n}
/// - get the number of trailing zeros in {n}
/// - find trailing zero count of {n}
/// - count zeros from the least significant bit in {n}
/// - trailing zero count for {n}
/// - number of zero bits at the end of {n}
/// - count trailing 0s in {n}
/// - get ctz for value {n}
/// - find how many zeros trail {n}
/// - compute trailing zero count of {n}
/// - count zeros after last set bit in {n}
///
/// # Parameters
/// - n=r0 "The value to count trailing zeros in"
///
/// # Test Cases
/// - ctz(0) = 64
/// - ctz(1) = 0
/// - ctz(8) = 3
/// - ctz(0x8000000000000000) = 63
#[inline(never)]
pub fn ctz(n: u64) -> u64 {
    if n == 0 {
        return 64;
    }

    let mut count: u64 = 0;
    let mut val = n;

    // Compute masks dynamically to avoid sign extension issues
    // Check if bottom 32 bits are zero: shift left then right to isolate
    let mask32 = (1u64 << 32) - 1;
    let low32 = val & mask32;
    if low32 == 0 {
        count = count + 32;
        val = val >> 32;
    }
    // Check if bottom 16 bits are zero
    let mask16 = (1u64 << 16) - 1;
    let low16 = val & mask16;
    if low16 == 0 {
        count = count + 16;
        val = val >> 16;
    }
    // Check if bottom 8 bits are zero
    let mask8 = (1u64 << 8) - 1;
    let low8 = val & mask8;
    if low8 == 0 {
        count = count + 8;
        val = val >> 8;
    }
    // Check if bottom 4 bits are zero
    let mask4 = (1u64 << 4) - 1;
    let low4 = val & mask4;
    if low4 == 0 {
        count = count + 4;
        val = val >> 4;
    }
    // Check if bottom 2 bits are zero
    let mask2 = (1u64 << 2) - 1;
    let low2 = val & mask2;
    if low2 == 0 {
        count = count + 2;
        val = val >> 2;
    }
    // Check if bottom bit is zero
    let low1 = val & 1;
    if low1 == 0 {
        count = count + 1;
    }

    count
}

/// Byte swap (reverse byte order for endian conversion).
///
/// # Neurlang Export
/// - Category: bitwise
/// - Opcode: bits.bswap
///
/// # Prompts
/// - swap bytes in {n}
/// - reverse byte order of {n}
/// - convert endianness of {n}
/// - byte swap {n}
/// - bswap {n}
/// - flip byte order in {n}
/// - convert {n} from big endian to little endian
/// - convert {n} from little endian to big endian
/// - reverse the bytes of {n}
/// - swap byte order for {n}
/// - change endianness of {n}
/// - perform byte reversal on {n}
/// - endian swap {n}
///
/// # Parameters
/// - n=r0 "The value to byte swap"
///
/// # Test Cases
/// - bswap(0x0102030405060708) = 0x0807060504030201
#[inline(never)]
pub fn bswap(n: u64) -> u64 {
    let b0 = (n & 0xFF) << 56;
    let b1 = ((n >> 8) & 0xFF) << 48;
    let b2 = ((n >> 16) & 0xFF) << 40;
    let b3 = ((n >> 24) & 0xFF) << 32;
    let b4 = ((n >> 32) & 0xFF) << 24;
    let b5 = ((n >> 40) & 0xFF) << 16;
    let b6 = ((n >> 48) & 0xFF) << 8;
    let b7 = (n >> 56) & 0xFF;

    b0 | b1 | b2 | b3 | b4 | b5 | b6 | b7
}

/// Rotate left.
///
/// # Neurlang Export
/// - Category: bitwise
/// - Difficulty: 1
///
/// # Prompts
/// - rotate {n} left by {shift} bits
/// - left rotate {n} by {shift}
/// - circular shift {n} left by {shift}
/// - rotl {n} by {shift}
/// - rotate bits of {n} left {shift} positions
/// - perform left rotation on {n} by {shift} bits
/// - bit rotate {n} leftward by {shift}
/// - cyclically shift {n} left by {shift}
/// - left bit rotation of {n} by {shift}
/// - rotate {n} leftward {shift} times
/// - circular left shift {n} by {shift}
/// - roll bits of {n} left by {shift}
///
/// # Parameters
/// - n=r0 "The value to rotate"
/// - shift=r1 "Number of bits to rotate left"
///
/// # Test Cases
/// - rotl(0x8000000000000001, 1) = 0x0000000000000003
#[inline(never)]
pub fn rotl(n: u64, shift: u64) -> u64 {
    let s = shift & 63; // Mask to valid range
    if s == 0 {
        return n;
    }
    let left_part = n << s;
    let right_shift = 64 - s;
    let right_part = n >> right_shift;
    left_part | right_part
}

/// Rotate right.
///
/// # Neurlang Export
/// - Category: bitwise
/// - Difficulty: 1
///
/// # Prompts
/// - rotate {n} right by {shift} bits
/// - right rotate {n} by {shift}
/// - circular shift {n} right by {shift}
/// - rotr {n} by {shift}
/// - rotate bits of {n} right {shift} positions
/// - perform right rotation on {n} by {shift} bits
/// - bit rotate {n} rightward by {shift}
/// - cyclically shift {n} right by {shift}
/// - right bit rotation of {n} by {shift}
/// - rotate {n} rightward {shift} times
/// - circular right shift {n} by {shift}
/// - roll bits of {n} right by {shift}
///
/// # Parameters
/// - n=r0 "The value to rotate"
/// - shift=r1 "Number of bits to rotate right"
///
/// # Test Cases
/// - rotr(0x0000000000000003, 1) = 0x8000000000000001
#[inline(never)]
pub fn rotr(n: u64, shift: u64) -> u64 {
    let s = shift & 63;
    if s == 0 {
        return n;
    }
    let right_part = n >> s;
    let left_shift = 64 - s;
    let left_part = n << left_shift;
    right_part | left_part
}

/// Extract a bit field.
///
/// # Neurlang Export
/// - Category: bitwise
/// - Difficulty: 2
///
/// # Prompts
/// - extract {len} bits from {n} starting at position {start}
/// - get bit field from {n} at offset {start} with length {len}
/// - extract bits {start} to {start}+{len} from {n}
/// - read {len} bits from {n} at bit {start}
/// - get bits from position {start} of width {len} in {n}
/// - extract a {len}-bit field from {n} starting at {start}
/// - pull {len} bits out of {n} at offset {start}
/// - slice {len} bits from {n} beginning at {start}
/// - get bit range from {n} starting at {start} for {len} bits
/// - extract bit field at position {start} width {len} from {n}
/// - read bit slice from {n} at {start} with size {len}
/// - get {len} bits from {n} offset by {start}
///
/// # Parameters
/// - n=r0 "Source value to extract bits from"
/// - start=r1 "Starting bit position (0-indexed from LSB)"
/// - len=r2 "Number of bits to extract"
#[inline(never)]
pub fn extract_bits(n: u64, start: u64, len: u64) -> u64 {
    if len == 0 || start >= 64 {
        return 0;
    }
    let mask = if len >= 64 { u64::MAX } else { (1u64 << len) - 1 };
    (n >> start) & mask
}

/// Insert bits into a value.
///
/// # Neurlang Export
/// - Category: bitwise
/// - Difficulty: 2
///
/// # Prompts
/// - insert {len} bits of {src} into {dest} at position {start}
/// - set bit field in {dest} with {src} at offset {start} length {len}
/// - write {len} bits from {src} to {dest} at bit {start}
/// - place {src} bits into {dest} at position {start} for {len} bits
/// - insert a {len}-bit field from {src} into {dest} at {start}
/// - put {len} bits of {src} into {dest} starting at {start}
/// - overwrite {len} bits in {dest} with {src} at offset {start}
/// - set bits {start} to {start}+{len} of {dest} with {src}
/// - insert bit field {src} into {dest} at {start} width {len}
/// - write bit slice from {src} to {dest} at {start} with size {len}
/// - embed {len} bits from {src} into {dest} at position {start}
/// - modify {dest} by inserting {src} at bit {start} for {len} bits
///
/// # Parameters
/// - dest=r0 "Destination value to insert bits into"
/// - src=r1 "Source bits to insert"
/// - start=r2 "Starting bit position (0-indexed from LSB)"
/// - len=r3 "Number of bits to insert"
#[inline(never)]
pub fn insert_bits(dest: u64, src: u64, start: u64, len: u64) -> u64 {
    if len == 0 || start >= 64 {
        return dest;
    }
    let mask = if len >= 64 { u64::MAX } else { (1u64 << len) - 1 };
    let clear_mask = !(mask << start);
    (dest & clear_mask) | ((src & mask) << start)
}

/// Check if exactly one bit is set (power of 2 check).
///
/// # Neurlang Export
/// - Category: bitwise
/// - Difficulty: 1
///
/// # Prompts
/// - check if {n} is a power of 2
/// - is {n} a power of two
/// - test if {n} has exactly one bit set
/// - determine if {n} is a power of 2
/// - check whether {n} is power of 2
/// - is {n} an exact power of 2
/// - verify {n} is a power of two
/// - test power of 2 for {n}
/// - check if only one bit is set in {n}
/// - is {n} a binary power
/// - determine whether {n} is 2^k for some k
/// - check if {n} equals 2 to some power
/// - validate {n} as power of 2
///
/// # Parameters
/// - n=r0 "The value to check"
///
/// # Test Cases
/// - is_power_of_2(0) = 0
/// - is_power_of_2(1) = 1
/// - is_power_of_2(16) = 1
/// - is_power_of_2(17) = 0
#[inline(never)]
pub fn is_power_of_2(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    if (n & (n - 1)) == 0 { 1 } else { 0 }
}

/// Round up to next power of 2.
///
/// # Neurlang Export
/// - Category: bitwise
/// - Difficulty: 2
///
/// # Prompts
/// - round {n} up to the next power of 2
/// - get the next power of 2 greater than or equal to {n}
/// - find smallest power of 2 >= {n}
/// - round {n} to next higher power of two
/// - ceiling power of 2 for {n}
/// - next power of 2 for {n}
/// - round up {n} to power of 2
/// - find the smallest power of 2 not less than {n}
/// - get next highest power of 2 from {n}
/// - compute ceiling power of two for {n}
/// - round {n} up to nearest power of 2
/// - next greater or equal power of 2 for {n}
/// - find power of 2 ceiling of {n}
///
/// # Parameters
/// - n=r0 "The value to round up"
///
/// # Test Cases
/// - next_power_of_2(5) = 8
/// - next_power_of_2(8) = 8
/// - next_power_of_2(1) = 1
#[inline(never)]
pub fn next_power_of_2(n: u64) -> u64 {
    if n == 0 {
        return 1;
    }

    let mut v = n - 1;
    v = v | (v >> 1);
    v = v | (v >> 2);
    v = v | (v >> 4);
    v = v | (v >> 8);
    v = v | (v >> 16);
    v = v | (v >> 32);
    v + 1
}

/// Reverse bits.
///
/// # Neurlang Export
/// - Category: bitwise
/// - Difficulty: 2
///
/// # Prompts
/// - reverse the bits in {n}
/// - flip bit order of {n}
/// - reverse bit order in {n}
/// - mirror the bits of {n}
/// - reflect bits in {n}
/// - reverse all 64 bits of {n}
/// - bit reversal of {n}
/// - swap bit positions in {n} (MSB becomes LSB)
/// - invert the bit order of {n}
/// - reverse binary representation of {n}
/// - flip {n} bit by bit
/// - get bit-reversed value of {n}
/// - compute bit reversal for {n}
///
/// # Parameters
/// - n=r0 "The value to reverse bits in"
#[inline(never)]
pub fn reverse_bits(mut n: u64) -> u64 {
    let mut result: u64 = 0;
    let mut i: u64 = 0;

    while i < 64 {
        result = result << 1;
        result = result | (n & 1);
        n = n >> 1;
        i = i + 1;
    }

    result
}

/// Get the highest set bit position (0-indexed from LSB).
///
/// # Neurlang Export
/// - Category: bitwise
/// - Difficulty: 1
///
/// # Prompts
/// - find the position of the highest set bit in {n}
/// - get the most significant set bit position in {n}
/// - find MSB position of {n}
/// - get index of highest 1 bit in {n}
/// - find the topmost set bit in {n}
/// - position of leading 1 in {n}
/// - get the bit index of the highest set bit in {n}
/// - find the most significant 1 bit position in {n}
/// - locate the highest set bit in {n}
/// - which bit position is the highest set in {n}
/// - get floor log2 of {n}
/// - find bit width minus 1 of {n}
/// - position of the leftmost 1 in {n}
///
/// # Parameters
/// - n=r0 "The value to find highest set bit in"
///
/// Returns: bit position, or 64 if n == 0
#[inline(never)]
pub fn highest_set_bit(n: u64) -> u64 {
    if n == 0 {
        return 64;
    }
    // Inline clz using shift-based checks
    let mut count: u64 = 0;
    let mut val = n;
    if (val >> 32) == 0 {
        count = count + 32;
        val = val << 32;
    }
    if (val >> 48) == 0 {
        count = count + 16;
        val = val << 16;
    }
    if (val >> 56) == 0 {
        count = count + 8;
        val = val << 8;
    }
    if (val >> 60) == 0 {
        count = count + 4;
        val = val << 4;
    }
    if (val >> 62) == 0 {
        count = count + 2;
        val = val << 2;
    }
    if (val >> 63) == 0 {
        count = count + 1;
    }
    63 - count
}

/// Get the lowest set bit position (0-indexed from LSB).
///
/// # Neurlang Export
/// - Category: bitwise
/// - Difficulty: 1
///
/// # Prompts
/// - find the position of the lowest set bit in {n}
/// - get the least significant set bit position in {n}
/// - find LSB position of {n}
/// - get index of lowest 1 bit in {n}
/// - find the bottommost set bit in {n}
/// - position of trailing 1 in {n}
/// - get the bit index of the lowest set bit in {n}
/// - find the least significant 1 bit position in {n}
/// - locate the lowest set bit in {n}
/// - which bit position is the lowest set in {n}
/// - find the rightmost 1 bit position in {n}
/// - get trailing zero count of {n}
/// - position of the first 1 from the right in {n}
///
/// # Parameters
/// - n=r0 "The value to find lowest set bit in"
///
/// Returns: bit position, or 64 if n == 0
#[inline(never)]
pub fn lowest_set_bit(n: u64) -> u64 {
    if n == 0 {
        return 64;
    }
    // Inline ctz using computed masks
    let mut count: u64 = 0;
    let mut val = n;
    let mask32 = (1u64 << 32) - 1;
    let low32 = val & mask32;
    if low32 == 0 {
        count = count + 32;
        val = val >> 32;
    }
    let mask16 = (1u64 << 16) - 1;
    let low16 = val & mask16;
    if low16 == 0 {
        count = count + 16;
        val = val >> 16;
    }
    let mask8 = (1u64 << 8) - 1;
    let low8 = val & mask8;
    if low8 == 0 {
        count = count + 8;
        val = val >> 8;
    }
    let mask4 = (1u64 << 4) - 1;
    let low4 = val & mask4;
    if low4 == 0 {
        count = count + 4;
        val = val >> 4;
    }
    let mask2 = (1u64 << 2) - 1;
    let low2 = val & mask2;
    if low2 == 0 {
        count = count + 2;
        val = val >> 2;
    }
    let low1 = val & 1;
    if low1 == 0 {
        count = count + 1;
    }
    count
}

/// Clear the lowest set bit.
///
/// # Neurlang Export
/// - Category: bitwise
/// - Difficulty: 1
///
/// # Prompts
/// - clear the lowest set bit in {n}
/// - turn off the rightmost 1 bit in {n}
/// - unset the least significant set bit in {n}
/// - remove the lowest 1 bit from {n}
/// - clear the first set bit from the right in {n}
/// - zero out the lowest set bit in {n}
/// - reset the rightmost 1 in {n}
/// - turn off LSB of {n}
/// - clear lowest 1 bit in {n}
/// - unset first set bit from right in {n}
/// - remove the bottommost set bit from {n}
/// - mask off the lowest set bit in {n}
/// - compute {n} AND ({n} - 1)
///
/// # Parameters
/// - n=r0 "The value to clear lowest set bit in"
#[inline(never)]
pub fn clear_lowest_bit(n: u64) -> u64 {
    n & (n - 1)
}

/// Isolate the lowest set bit.
///
/// # Neurlang Export
/// - Category: bitwise
/// - Difficulty: 1
///
/// # Prompts
/// - isolate the lowest set bit in {n}
/// - extract only the rightmost 1 bit from {n}
/// - get the least significant set bit of {n}
/// - return only the lowest 1 bit of {n}
/// - mask all but the lowest set bit in {n}
/// - keep only the first set bit from the right in {n}
/// - isolate LSB of {n}
/// - get the lowest set bit value from {n}
/// - extract the bottommost set bit of {n}
/// - find the value of the lowest 1 bit in {n}
/// - get only the rightmost 1 in {n}
/// - compute {n} AND (NOT {n} + 1)
/// - isolate least significant 1 bit of {n}
///
/// # Parameters
/// - n=r0 "The value to isolate lowest set bit from"
#[inline(never)]
pub fn isolate_lowest_bit(n: u64) -> u64 {
    n & (!n + 1)
}

/// Parity (1 if odd number of set bits, 0 otherwise).
///
/// # Neurlang Export
/// - Category: bitwise
/// - Difficulty: 1
///
/// # Prompts
/// - compute the parity of {n}
/// - get the parity bit for {n}
/// - check if {n} has an odd number of set bits
/// - calculate parity of {n}
/// - XOR all bits of {n} together
/// - is the bit count of {n} odd or even
/// - compute single-bit parity for {n}
/// - parity check for {n}
/// - determine if {n} has odd parity
/// - get the parity of the bits in {n}
/// - find if popcount of {n} is odd
/// - return 1 if {n} has odd number of 1s, else 0
/// - calculate the parity bit of {n}
///
/// # Parameters
/// - n=r0 "The value to compute parity for"
#[inline(never)]
pub fn parity(mut n: u64) -> u64 {
    // Inline popcount
    let mut count: u64 = 0;
    while n != 0 {
        count = count + (n & 1);
        n = n >> 1;
    }
    count & 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popcount() {
        assert_eq!(popcount(0), 0);
        assert_eq!(popcount(1), 1);
        assert_eq!(popcount(0xFF), 8);
        assert_eq!(popcount(0xFFFFFFFFFFFFFFFF), 64);
    }

    #[test]
    fn test_clz() {
        assert_eq!(clz(0), 64);
        assert_eq!(clz(1), 63);
        assert_eq!(clz(0x8000000000000000), 0);
    }

    #[test]
    fn test_ctz() {
        assert_eq!(ctz(0), 64);
        assert_eq!(ctz(1), 0);
        assert_eq!(ctz(8), 3);
    }

    #[test]
    fn test_bswap() {
        assert_eq!(bswap(0x0102030405060708), 0x0807060504030201);
    }

    #[test]
    fn test_rotl_rotr() {
        let n = 0x8000000000000001u64;
        assert_eq!(rotl(n, 1), 0x0000000000000003);
        assert_eq!(rotr(0x0000000000000003, 1), n);
    }

    #[test]
    fn test_is_power_of_2() {
        assert_eq!(is_power_of_2(0), 0);
        assert_eq!(is_power_of_2(1), 1);
        assert_eq!(is_power_of_2(16), 1);
        assert_eq!(is_power_of_2(17), 0);
    }

    #[test]
    fn test_next_power_of_2() {
        assert_eq!(next_power_of_2(5), 8);
        assert_eq!(next_power_of_2(8), 8);
        assert_eq!(next_power_of_2(1), 1);
    }
}
