//! Math functions for Neurlang stdlib
//!
//! These functions compile to pure Neurlang IR without extension calls.

/// Calculate factorial of n iteratively.
///
/// # Neurlang Export
/// - Category: algorithm/math
/// - Difficulty: 2
///
/// # Prompts
/// - compute factorial of {n}
/// - {n}!
/// - calculate {n} factorial
/// - what is {n}!
/// - factorial({n})
/// - multiply 1 * 2 * ... * {n}
/// - product of integers from 1 to {n}
/// - n! where n={n}
/// - iterative factorial for {n}
/// - compute {n} factorial using a loop
/// - find the factorial of {n}
/// - calculate {n}! iteratively
///
/// # Parameters
/// - n=r0 "The number to compute factorial of"
///
/// # Test Cases
/// - factorial(0) = 1
/// - factorial(1) = 1
/// - factorial(5) = 120
/// - factorial(10) = 3628800
/// - factorial(20) = 2432902008176640000
#[inline(never)]
pub fn factorial(n: u64) -> u64 {
    let mut result: u64 = 1;
    let mut i: u64 = n;
    while i > 0 {
        result = result * i;
        i = i - 1;
    }
    result
}

/// Calculate the nth Fibonacci number iteratively.
///
/// # Neurlang Export
/// - Category: algorithm/math
/// - Difficulty: 2
///
/// # Prompts
/// - compute fibonacci({n})
/// - calculate fib({n})
/// - find the {n}th fibonacci number
/// - fibonacci sequence element {n}
/// - what is fib({n})
/// - {n}th fibonacci
/// - fibonacci of {n}
/// - iterative fibonacci for {n}
/// - compute fib sequence at position {n}
/// - calculate fibonacci number {n}
/// - find fibonacci({n})
/// - get the {n}th term of fibonacci sequence
///
/// # Parameters
/// - n=r0 "The position in the Fibonacci sequence (0-indexed)"
///
/// # Test Cases
/// - fibonacci(0) = 0
/// - fibonacci(1) = 1
/// - fibonacci(10) = 55
/// - fibonacci(20) = 6765
/// - fibonacci(40) = 102334155
#[inline(never)]
pub fn fibonacci(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }

    let mut a: u64 = 0;
    let mut b: u64 = 1;
    let mut i: u64 = 2;

    while i <= n {
        let temp = a + b;
        a = b;
        b = temp;
        i = i + 1;
    }

    b
}

/// Calculate greatest common divisor using Euclidean algorithm.
///
/// # Neurlang Export
/// - Category: algorithm/math
/// - Difficulty: 2
///
/// # Prompts
/// - find GCD of {a} and {b}
/// - gcd({a}, {b})
/// - greatest common divisor of {a} and {b}
/// - euclidean algorithm for {a}, {b}
/// - compute gcd of {a} {b}
/// - what is the GCD of {a} and {b}
/// - calculate greatest common divisor {a} {b}
/// - find common divisor of {a} and {b}
/// - highest common factor of {a} and {b}
/// - HCF of {a} and {b}
/// - largest divisor common to {a} and {b}
/// - compute euclidean gcd({a}, {b})
///
/// # Parameters
/// - a=r0 "First number"
/// - b=r1 "Second number"
///
/// # Test Cases
/// - gcd(48, 18) = 6
/// - gcd(100, 35) = 5
/// - gcd(17, 13) = 1
/// - gcd(0, 5) = 5
/// - gcd(12, 0) = 12
#[inline(never)]
pub fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

/// Calculate least common multiple.
///
/// # Neurlang Export
/// - Category: algorithm/math
/// - Difficulty: 2
///
/// # Prompts
/// - find LCM of {a} and {b}
/// - lcm({a}, {b})
/// - least common multiple of {a} and {b}
/// - compute lcm of {a} {b}
/// - what is the LCM of {a} and {b}
/// - smallest common multiple of {a} and {b}
/// - calculate least common multiple {a} {b}
/// - lowest common multiple {a} {b}
/// - find smallest number divisible by {a} and {b}
/// - compute lowest common multiple({a}, {b})
///
/// # Parameters
/// - a=r0 "First number"
/// - b=r1 "Second number"
///
/// # Test Cases
/// - lcm(4, 6) = 12
/// - lcm(3, 5) = 15
/// - lcm(12, 18) = 36
#[inline(never)]
pub fn lcm(a: u64, b: u64) -> u64 {
    if a == 0 || b == 0 {
        return 0;
    }
    // Inline gcd algorithm
    let mut x = a;
    let mut y = b;
    while y != 0 {
        let temp = y;
        y = x % y;
        x = temp;
    }
    // x is now gcd(a, b)
    (a / x) * b
}

/// Calculate a^n (power) iteratively using binary exponentiation.
///
/// # Neurlang Export
/// - Category: algorithm/math
/// - Difficulty: 2
///
/// # Prompts
/// - compute {base} to the power of {exp}
/// - {base}^{exp}
/// - calculate {base} raised to {exp}
/// - power({base}, {exp})
/// - exponentiate {base} by {exp}
/// - {base} ** {exp}
/// - compute {base} pow {exp}
/// - raise {base} to the {exp} power
/// - binary exponentiation {base}^{exp}
/// - fast power {base} to {exp}
/// - calculate {base}^{exp} iteratively
/// - compute integer power of {base} and {exp}
///
/// # Parameters
/// - base=r0 "The base number"
/// - exp=r1 "The exponent"
///
/// # Test Cases
/// - power(2, 0) = 1
/// - power(2, 10) = 1024
/// - power(3, 5) = 243
/// - power(10, 3) = 1000
#[inline(never)]
pub fn power(base: u64, exp: u64) -> u64 {
    let mut result: u64 = 1;
    let mut b = base;
    let mut e = exp;

    while e > 0 {
        if (e & 1) == 1 {
            result = result * b;
        }
        b = b * b;
        e = e >> 1;
    }

    result
}

/// Check if a number is prime.
///
/// # Neurlang Export
/// - Category: algorithm/math
/// - Difficulty: 2
///
/// # Prompts
/// - check if {n} is prime
/// - is {n} a prime number
/// - test primality of {n}
/// - is_prime({n})
/// - determine if {n} is prime
/// - check primality of {n}
/// - is {n} prime?
/// - prime test for {n}
/// - verify {n} is prime
/// - check whether {n} has only two factors
/// - is {n} divisible only by 1 and itself
/// - primality check for {n}
///
/// # Parameters
/// - n=r0 "The number to check for primality"
///
/// # Test Cases
/// - is_prime(2) = 1
/// - is_prime(3) = 1
/// - is_prime(4) = 0
/// - is_prime(17) = 1
/// - is_prime(100) = 0
/// - is_prime(101) = 1
#[inline(never)]
pub fn is_prime(n: u64) -> u64 {
    if n < 2 {
        return 0;
    }
    if n == 2 {
        return 1;
    }
    if (n & 1) == 0 {
        return 0;
    }

    let mut i: u64 = 3;
    while i * i <= n {
        if n % i == 0 {
            return 0;
        }
        i = i + 2;
    }

    1
}

/// Calculate absolute value (for signed integers represented as u64).
///
/// # Neurlang Export
/// - Category: algorithm/math
/// - Difficulty: 1
///
/// # Prompts
/// - absolute value of {n}
/// - abs({n})
/// - compute |{n}|
/// - get absolute value of {n}
/// - make {n} positive
/// - magnitude of {n}
/// - unsigned value of {n}
/// - remove sign from {n}
/// - convert {n} to positive
/// - calculate absolute value {n}
///
/// # Parameters
/// - n=r0 "The signed integer value"
///
/// # Test Cases
/// - abs_i64(5) = 5
/// - abs_i64(0) = 0
#[inline(never)]
pub fn abs_i64(n: u64) -> u64 {
    // Check if negative (sign bit set)
    if (n >> 63) == 1 {
        // Two's complement negation
        (!n).wrapping_add(1)
    } else {
        n
    }
}

/// Calculate minimum of two values.
///
/// # Neurlang Export
/// - Category: algorithm/math
/// - Difficulty: 1
///
/// # Prompts
/// - minimum of {a} and {b}
/// - min({a}, {b})
/// - smaller of {a} and {b}
/// - find minimum between {a} {b}
/// - which is smaller {a} or {b}
/// - get the lesser of {a} and {b}
/// - compute min of {a} {b}
/// - return smaller value {a} {b}
/// - find the smallest of {a} {b}
/// - lesser value between {a} and {b}
///
/// # Parameters
/// - a=r0 "First value"
/// - b=r1 "Second value"
///
/// # Test Cases
/// - min(5, 3) = 3
/// - min(3, 5) = 3
/// - min(7, 7) = 7
#[inline(never)]
pub fn min(a: u64, b: u64) -> u64 {
    if a < b { a } else { b }
}

/// Calculate maximum of two values.
///
/// # Neurlang Export
/// - Category: algorithm/math
/// - Difficulty: 1
///
/// # Prompts
/// - maximum of {a} and {b}
/// - max({a}, {b})
/// - larger of {a} and {b}
/// - find maximum between {a} {b}
/// - which is larger {a} or {b}
/// - get the greater of {a} and {b}
/// - compute max of {a} {b}
/// - return larger value {a} {b}
/// - find the largest of {a} {b}
/// - greater value between {a} and {b}
///
/// # Parameters
/// - a=r0 "First value"
/// - b=r1 "Second value"
///
/// # Test Cases
/// - max(5, 3) = 5
/// - max(3, 5) = 5
/// - max(7, 7) = 7
#[inline(never)]
pub fn max(a: u64, b: u64) -> u64 {
    if a > b { a } else { b }
}

/// Integer division with rounding up (ceiling).
///
/// # Neurlang Export
/// - Category: algorithm/math
/// - Difficulty: 1
///
/// # Prompts
/// - ceiling division {a} / {b}
/// - divide {a} by {b} rounding up
/// - div_ceil({a}, {b})
/// - integer division ceiling {a} {b}
/// - round up {a} / {b}
/// - compute {a} / {b} rounded up
/// - ceiling of {a} divided by {b}
/// - divide and round up {a} {b}
/// - integer ceiling division {a} by {b}
/// - {a} divided by {b} round to ceiling
///
/// # Parameters
/// - a=r0 "Dividend"
/// - b=r1 "Divisor"
///
/// # Test Cases
/// - div_ceil(10, 3) = 4
/// - div_ceil(9, 3) = 3
/// - div_ceil(1, 10) = 1
#[inline(never)]
pub fn div_ceil(a: u64, b: u64) -> u64 {
    if b == 0 {
        return 0; // Avoid division by zero
    }
    (a + b - 1) / b
}

/// Calculate the sum of numbers from 1 to n (triangle number).
///
/// # Neurlang Export
/// - Category: algorithm/math
/// - Difficulty: 1
///
/// # Prompts
/// - sum of 1 to {n}
/// - triangle number {n}
/// - sum 1 + 2 + ... + {n}
/// - compute 1+2+3+...+{n}
/// - triangular number for {n}
/// - sum first {n} natural numbers
/// - what is 1+2+...+{n}
/// - gauss sum to {n}
/// - arithmetic sum 1 to {n}
/// - calculate sum of integers from 1 to {n}
/// - add numbers 1 through {n}
/// - sum of first {n} positive integers
///
/// # Parameters
/// - n=r0 "The upper bound of the sum"
///
/// # Test Cases
/// - triangle_number(1) = 1
/// - triangle_number(5) = 15
/// - triangle_number(10) = 55
/// - triangle_number(100) = 5050
#[inline(never)]
pub fn triangle_number(n: u64) -> u64 {
    n * (n + 1) / 2
}

/// Integer square root (floor).
///
/// # Neurlang Export
/// - Category: algorithm/math
/// - Difficulty: 2
///
/// # Prompts
/// - integer square root of {n}
/// - isqrt({n})
/// - floor of sqrt({n})
/// - compute integer sqrt of {n}
/// - find largest x where x*x <= {n}
/// - square root rounded down {n}
/// - integer sqrt {n}
/// - floor sqrt of {n}
/// - newton's method sqrt {n}
/// - approximate square root {n}
/// - calculate floor(sqrt({n}))
/// - find integer part of sqrt({n})
///
/// # Parameters
/// - n=r0 "The number to find square root of"
///
/// # Test Cases
/// - isqrt(0) = 0
/// - isqrt(1) = 1
/// - isqrt(4) = 2
/// - isqrt(10) = 3
/// - isqrt(100) = 10
/// - isqrt(101) = 10
#[inline(never)]
pub fn isqrt(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }

    let mut x = n;
    let mut y = (x + 1) / 2;

    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }

    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(5), 120);
        assert_eq!(factorial(10), 3628800);
        assert_eq!(factorial(20), 2432902008176640000);
    }

    #[test]
    fn test_fibonacci() {
        assert_eq!(fibonacci(0), 0);
        assert_eq!(fibonacci(1), 1);
        assert_eq!(fibonacci(10), 55);
        assert_eq!(fibonacci(20), 6765);
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(48, 18), 6);
        assert_eq!(gcd(100, 35), 5);
        assert_eq!(gcd(17, 13), 1);
        assert_eq!(gcd(0, 5), 5);
        assert_eq!(gcd(12, 0), 12);
    }

    #[test]
    fn test_lcm() {
        assert_eq!(lcm(4, 6), 12);
        assert_eq!(lcm(3, 5), 15);
        assert_eq!(lcm(12, 18), 36);
    }

    #[test]
    fn test_power() {
        assert_eq!(power(2, 0), 1);
        assert_eq!(power(2, 10), 1024);
        assert_eq!(power(3, 5), 243);
    }

    #[test]
    fn test_is_prime() {
        assert_eq!(is_prime(2), 1);
        assert_eq!(is_prime(3), 1);
        assert_eq!(is_prime(4), 0);
        assert_eq!(is_prime(17), 1);
        assert_eq!(is_prime(100), 0);
        assert_eq!(is_prime(101), 1);
    }

    #[test]
    fn test_min_max() {
        assert_eq!(min(5, 3), 3);
        assert_eq!(max(5, 3), 5);
    }

    #[test]
    fn test_isqrt() {
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(1), 1);
        assert_eq!(isqrt(4), 2);
        assert_eq!(isqrt(10), 3);
        assert_eq!(isqrt(100), 10);
    }
}
