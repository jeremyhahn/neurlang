//! Floating-point functions for Neurlang stdlib
//!
//! These functions compile to FPU opcodes (fpu.fadd, fpu.fsub, etc.)
//! All f64 values are stored as bit patterns in u64 registers.

/// Add two floating-point numbers.
///
/// # Neurlang Export
/// - Category: float
/// - Difficulty: 1
///
/// # Prompts
/// - add {a} and {b} floats
/// - float add {a} + {b}
/// - fadd({a}, {b})
/// - add floating point numbers {a} {b}
/// - compute {a} + {b} as floats
/// - sum two floats {a} and {b}
/// - floating point addition {a} {b}
/// - add f64 values {a} {b}
/// - {a} plus {b} float
/// - compute float sum of {a} and {b}
///
/// # Parameters
/// - a=r0 "First floating-point number (as bits)"
/// - b=r1 "Second floating-point number (as bits)"
#[inline(never)]
pub fn fadd(a: f64, b: f64) -> f64 {
    a + b
}

/// Subtract two floating-point numbers.
///
/// # Neurlang Export
/// - Category: float
/// - Difficulty: 1
///
/// # Prompts
/// - subtract {b} from {a} float
/// - float subtract {a} - {b}
/// - fsub({a}, {b})
/// - subtract floating point {a} minus {b}
/// - compute {a} - {b} as floats
/// - difference of two floats {a} and {b}
/// - floating point subtraction {a} {b}
/// - subtract f64 values {a} {b}
/// - {a} minus {b} float
/// - compute float difference of {a} and {b}
///
/// # Parameters
/// - a=r0 "First floating-point number (minuend)"
/// - b=r1 "Second floating-point number (subtrahend)"
#[inline(never)]
pub fn fsub(a: f64, b: f64) -> f64 {
    a - b
}

/// Multiply two floating-point numbers.
///
/// # Neurlang Export
/// - Category: float
/// - Difficulty: 1
///
/// # Prompts
/// - multiply {a} and {b} floats
/// - float multiply {a} * {b}
/// - fmul({a}, {b})
/// - multiply floating point {a} times {b}
/// - compute {a} * {b} as floats
/// - product of two floats {a} and {b}
/// - floating point multiplication {a} {b}
/// - multiply f64 values {a} {b}
/// - {a} times {b} float
/// - compute float product of {a} and {b}
///
/// # Parameters
/// - a=r0 "First floating-point number"
/// - b=r1 "Second floating-point number"
#[inline(never)]
pub fn fmul(a: f64, b: f64) -> f64 {
    a * b
}

/// Divide two floating-point numbers.
///
/// # Neurlang Export
/// - Category: float
/// - Difficulty: 1
///
/// # Prompts
/// - divide {a} by {b} float
/// - float divide {a} / {b}
/// - fdiv({a}, {b})
/// - divide floating point {a} by {b}
/// - compute {a} / {b} as floats
/// - quotient of two floats {a} and {b}
/// - floating point division {a} {b}
/// - divide f64 values {a} {b}
/// - {a} divided by {b} float
/// - compute float quotient of {a} and {b}
///
/// # Parameters
/// - a=r0 "Dividend (numerator)"
/// - b=r1 "Divisor (denominator)"
#[inline(never)]
pub fn fdiv(a: f64, b: f64) -> f64 {
    a / b
}

/// Calculate square root.
///
/// # Neurlang Export
/// - Category: float
/// - Difficulty: 1
///
/// # Prompts
/// - square root of {x}
/// - sqrt({x})
/// - fsqrt({x})
/// - compute square root of {x}
/// - float square root {x}
/// - calculate sqrt of {x}
/// - find square root {x}
/// - root of {x}
/// - compute sqrt {x}
/// - floating point square root of {x}
///
/// # Parameters
/// - x=r0 "The value to compute square root of"
///
/// # Test Cases
/// - sqrt(4.0) = 2.0
/// - sqrt(9.0) = 3.0
/// - sqrt(2.0) ≈ 1.414...
#[inline(never)]
pub fn fsqrt(x: f64) -> f64 {
    x.sqrt()
}

/// Calculate absolute value of float.
///
/// # Neurlang Export
/// - Category: float
/// - Difficulty: 1
///
/// # Prompts
/// - absolute value of {x} float
/// - fabs({x})
/// - float abs {x}
/// - |{x}| as float
/// - absolute float {x}
/// - make {x} positive float
/// - magnitude of float {x}
/// - compute float absolute value {x}
/// - floating point abs of {x}
/// - remove sign from float {x}
///
/// # Parameters
/// - x=r0 "The floating-point value"
///
/// # Test Cases
/// - fabs(-5.0) = 5.0
/// - fabs(5.0) = 5.0
/// - fabs(0.0) = 0.0
#[inline(never)]
pub fn fabs(x: f64) -> f64 {
    x.abs()
}

/// Calculate floor (round down).
///
/// # Neurlang Export
/// - Category: float
/// - Difficulty: 1
///
/// # Prompts
/// - floor of {x}
/// - ffloor({x})
/// - round {x} down
/// - floor function {x}
/// - largest integer <= {x}
/// - round down {x}
/// - compute floor of {x}
/// - integer part rounded down {x}
/// - floating point floor {x}
/// - floor({x})
///
/// # Parameters
/// - x=r0 "The floating-point value to floor"
///
/// # Test Cases
/// - ffloor(3.7) = 3.0
/// - ffloor(-2.3) = -3.0
/// - ffloor(5.0) = 5.0
#[inline(never)]
pub fn ffloor(x: f64) -> f64 {
    x.floor()
}

/// Calculate ceiling (round up).
///
/// # Neurlang Export
/// - Category: float
/// - Difficulty: 1
///
/// # Prompts
/// - ceiling of {x}
/// - fceil({x})
/// - round {x} up
/// - ceiling function {x}
/// - smallest integer >= {x}
/// - round up {x}
/// - compute ceiling of {x}
/// - integer part rounded up {x}
/// - floating point ceiling {x}
/// - ceil({x})
///
/// # Parameters
/// - x=r0 "The floating-point value to ceil"
///
/// # Test Cases
/// - fceil(3.2) = 4.0
/// - fceil(-2.7) = -2.0
/// - fceil(5.0) = 5.0
#[inline(never)]
pub fn fceil(x: f64) -> f64 {
    x.ceil()
}

/// Round to nearest integer.
///
/// # Neurlang Export
/// - Category: float
/// - Difficulty: 2
///
/// # Prompts
/// - round {x} to nearest integer
/// - fround({x})
/// - round({x})
/// - round float {x}
/// - nearest integer to {x}
/// - round {x} to whole number
/// - compute round of {x}
/// - round floating point {x}
/// - banker's rounding {x}
/// - round {x} half to even
///
/// # Parameters
/// - x=r0 "The floating-point value to round"
///
/// # Test Cases
/// - fround(3.4) = 3.0
/// - fround(3.5) = 4.0
/// - fround(-2.5) = -2.0 (round half to even)
#[inline(never)]
pub fn fround(x: f64) -> f64 {
    x.round()
}

/// Calculate the sign of a number: -1.0, 0.0, or 1.0.
///
/// # Neurlang Export
/// - Category: float
/// - Difficulty: 1
///
/// # Prompts
/// - sign of {x}
/// - fsign({x})
/// - signum of {x}
/// - get sign of float {x}
/// - is {x} positive negative or zero
/// - sign function {x}
/// - compute sign of {x}
/// - determine sign of {x}
/// - float sign {x}
/// - return -1 0 or 1 for {x}
///
/// # Parameters
/// - x=r0 "The floating-point value"
///
/// # Test Cases
/// - fsign(5.0) = 1.0
/// - fsign(-3.0) = -1.0
/// - fsign(0.0) = 0.0
#[inline(never)]
pub fn fsign(x: f64) -> f64 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

/// Calculate minimum of two floats.
///
/// # Neurlang Export
/// - Category: float
/// - Difficulty: 1
///
/// # Prompts
/// - minimum of {a} and {b} floats
/// - fmin({a}, {b})
/// - smaller float {a} or {b}
/// - float min of {a} {b}
/// - minimum floating point {a} {b}
/// - lesser of {a} and {b} as floats
/// - compute float minimum {a} {b}
/// - min({a}, {b}) float
/// - which is smaller float {a} {b}
/// - find minimum float between {a} and {b}
///
/// # Parameters
/// - a=r0 "First floating-point value"
/// - b=r1 "Second floating-point value"
#[inline(never)]
pub fn fmin(a: f64, b: f64) -> f64 {
    if a < b { a } else { b }
}

/// Calculate maximum of two floats.
///
/// # Neurlang Export
/// - Category: float
/// - Difficulty: 1
///
/// # Prompts
/// - maximum of {a} and {b} floats
/// - fmax({a}, {b})
/// - larger float {a} or {b}
/// - float max of {a} {b}
/// - maximum floating point {a} {b}
/// - greater of {a} and {b} as floats
/// - compute float maximum {a} {b}
/// - max({a}, {b}) float
/// - which is larger float {a} {b}
/// - find maximum float between {a} and {b}
///
/// # Parameters
/// - a=r0 "First floating-point value"
/// - b=r1 "Second floating-point value"
#[inline(never)]
pub fn fmax(a: f64, b: f64) -> f64 {
    if a > b { a } else { b }
}

/// Clamp a value between min and max.
///
/// # Neurlang Export
/// - Category: float
/// - Difficulty: 1
///
/// # Prompts
/// - clamp {x} between {min_val} and {max_val}
/// - fclamp({x}, {min_val}, {max_val})
/// - constrain {x} to range {min_val} to {max_val}
/// - limit {x} to {min_val}-{max_val}
/// - clamp float {x} in range
/// - bound {x} between {min_val} and {max_val}
/// - restrict {x} to interval {min_val} {max_val}
/// - ensure {x} is between {min_val} and {max_val}
/// - saturate {x} to range
/// - clamp value {x} min {min_val} max {max_val}
///
/// # Parameters
/// - x=r0 "The value to clamp"
/// - min_val=r1 "Minimum bound"
/// - max_val=r2 "Maximum bound"
///
/// # Test Cases
/// - fclamp(5.0, 0.0, 10.0) = 5.0
/// - fclamp(-1.0, 0.0, 10.0) = 0.0
/// - fclamp(15.0, 0.0, 10.0) = 10.0
#[inline(never)]
pub fn fclamp(x: f64, min_val: f64, max_val: f64) -> f64 {
    if x < min_val {
        min_val
    } else if x > max_val {
        max_val
    } else {
        x
    }
}

/// Linear interpolation between two values.
///
/// # Neurlang Export
/// - Category: float
/// - Difficulty: 2
///
/// # Prompts
/// - lerp from {a} to {b} at {t}
/// - linear interpolation {a} {b} {t}
/// - lerp({a}, {b}, {t})
/// - interpolate between {a} and {b} by {t}
/// - blend {a} and {b} with factor {t}
/// - mix {a} {b} at position {t}
/// - compute lerp {a} to {b} t={t}
/// - linear blend {a} {b} {t}
/// - interpolate {t} between {a} and {b}
/// - weighted average {a} {b} weight {t}
///
/// # Parameters
/// - a=r0 "Start value"
/// - b=r1 "End value"
/// - t=r2 "Interpolation factor (0.0 to 1.0)"
///
/// # Test Cases
/// - lerp(0.0, 10.0, 0.0) = 0.0
/// - lerp(0.0, 10.0, 0.5) = 5.0
/// - lerp(0.0, 10.0, 1.0) = 10.0
#[inline(never)]
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Calculate the reciprocal (1/x).
///
/// # Neurlang Export
/// - Category: float
/// - Difficulty: 1
///
/// # Prompts
/// - reciprocal of {x}
/// - frecip({x})
/// - 1 / {x}
/// - one over {x}
/// - inverse of {x}
/// - compute reciprocal {x}
/// - calculate 1/{x}
/// - multiplicative inverse of {x}
/// - float reciprocal {x}
/// - divide 1 by {x}
///
/// # Parameters
/// - x=r0 "The value to compute reciprocal of"
#[inline(never)]
pub fn frecip(x: f64) -> f64 {
    1.0 / x
}

/// Calculate x modulo y for floating point.
///
/// # Neurlang Export
/// - Category: float
/// - Difficulty: 2
///
/// # Prompts
/// - {x} modulo {y} float
/// - fmod({x}, {y})
/// - float remainder {x} % {y}
/// - compute {x} mod {y} as float
/// - floating point modulo {x} {y}
/// - remainder of {x} / {y}
/// - {x} mod {y} floating point
/// - calculate float remainder {x} {y}
/// - modulus of {x} and {y}
/// - {x} % {y} for floats
///
/// # Parameters
/// - x=r0 "Dividend"
/// - y=r1 "Divisor"
///
/// # Test Cases
/// - fmod(5.5, 2.0) = 1.5
/// - fmod(10.0, 3.0) = 1.0
#[inline(never)]
pub fn fmod(x: f64, y: f64) -> f64 {
    x - (x / y).floor() * y
}

/// Convert f64 bits to u64 (for bit manipulation).
///
/// # Neurlang Export
/// - Category: float/conversion
/// - Difficulty: 1
///
/// # Prompts
/// - convert float {x} to bits
/// - f64_to_bits({x})
/// - float to bits {x}
/// - get bit pattern of {x}
/// - reinterpret {x} as u64
/// - float bits of {x}
/// - raw bits of float {x}
/// - extract bits from float {x}
/// - to_bits({x})
/// - float to integer bits {x}
///
/// # Parameters
/// - x=r0 "The floating-point value"
#[inline(never)]
pub fn f64_to_bits(x: f64) -> u64 {
    x.to_bits()
}

/// Convert u64 bits to f64.
///
/// # Neurlang Export
/// - Category: float/conversion
/// - Difficulty: 1
///
/// # Prompts
/// - convert bits {bits} to float
/// - f64_from_bits({bits})
/// - bits to float {bits}
/// - reinterpret {bits} as f64
/// - make float from bits {bits}
/// - from_bits({bits})
/// - integer bits to float {bits}
/// - create float from bit pattern {bits}
/// - u64 to f64 bits {bits}
/// - construct float from {bits}
///
/// # Parameters
/// - bits=r0 "The bit pattern to interpret as float"
#[inline(never)]
pub fn f64_from_bits(bits: u64) -> f64 {
    f64::from_bits(bits)
}

/// Check if float is NaN.
///
/// # Neurlang Export
/// - Category: float
/// - Difficulty: 2
///
/// # Prompts
/// - is {x} NaN
/// - fis_nan({x})
/// - check if {x} is not a number
/// - is_nan({x})
/// - test if {x} is NaN
/// - {x} == NaN check
/// - is float {x} undefined
/// - check NaN {x}
/// - detect NaN in {x}
/// - is {x} a valid number
///
/// # Parameters
/// - x=r0 "The floating-point value to check"
#[inline(never)]
pub fn fis_nan(x: f64) -> u64 {
    if x.is_nan() { 1 } else { 0 }
}

/// Check if float is infinite.
///
/// # Neurlang Export
/// - Category: float
/// - Difficulty: 2
///
/// # Prompts
/// - is {x} infinite
/// - fis_infinite({x})
/// - check if {x} is infinity
/// - is_infinite({x})
/// - test if {x} is infinite
/// - is {x} plus or minus infinity
/// - check infinity {x}
/// - detect infinity in {x}
/// - is {x} unbounded
/// - {x} == inf check
///
/// # Parameters
/// - x=r0 "The floating-point value to check"
#[inline(never)]
pub fn fis_infinite(x: f64) -> u64 {
    if x.is_infinite() { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ops() {
        assert_eq!(fadd(1.0, 2.0), 3.0);
        assert_eq!(fsub(5.0, 3.0), 2.0);
        assert_eq!(fmul(3.0, 4.0), 12.0);
        assert_eq!(fdiv(10.0, 2.0), 5.0);
    }

    #[test]
    fn test_sqrt() {
        assert_eq!(fsqrt(4.0), 2.0);
        assert_eq!(fsqrt(9.0), 3.0);
    }

    #[test]
    fn test_abs() {
        assert_eq!(fabs(-5.0), 5.0);
        assert_eq!(fabs(5.0), 5.0);
    }

    #[test]
    fn test_floor_ceil() {
        assert_eq!(ffloor(3.7), 3.0);
        assert_eq!(fceil(3.2), 4.0);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(fclamp(5.0, 0.0, 10.0), 5.0);
        assert_eq!(fclamp(-1.0, 0.0, 10.0), 0.0);
        assert_eq!(fclamp(15.0, 0.0, 10.0), 10.0);
    }

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
    }
}
