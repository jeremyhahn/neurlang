//! Test the multi-head prediction pipeline
//!
//! Run with: cargo run --release --example test_pipeline

use neurlang::inference::pipeline::FastPipeline;

fn main() {
    println!("=== Multi-Head Direct Prediction Pipeline Test ===\n");

    let mut pipeline = FastPipeline::new();

    // Test cases to run
    let test_cases = vec![
        // Arithmetic
        ("5 + 3", "ADD"),
        ("10 - 7", "SUB"),
        ("6 * 7", "MUL"),
        ("20 / 4", "DIV"),
        ("17 % 5", "MOD"),
        // Math functions
        ("5!", "FACTORIAL"),
        ("factorial(10)", "FACTORIAL"),
        ("fibonacci(10)", "FIBONACCI"),
        ("gcd(48, 18)", "GCD"),
        ("lcm(12, 8)", "LCM"),
        // Comparisons
        ("max(15, 23)", "MAX"),
        ("min(15, 23)", "MIN"),
        // Natural language
        ("add 100 and 200", "ADD"),
        ("multiply 7 by 8", "MUL"),
        ("what is the factorial of 6", "FACTORIAL"),
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (prompt, expected_intent) in &test_cases {
        match pipeline.run(prompt) {
            Ok(result) => {
                let status = if result.intent_name == *expected_intent {
                    passed += 1;
                    "✓"
                } else {
                    failed += 1;
                    "✗"
                };

                println!("{} \"{}\"", status, prompt);
                println!(
                    "   Intent: {} (expected: {})",
                    result.intent_name, expected_intent
                );
                println!("   Operands: {:?}", result.operands);
                println!("   Result: {}", result.result);
                println!(
                    "   Latency: {:?} (compile: {:?}, exec: {:?})",
                    result.total_latency, result.compilation_latency, result.execution_latency
                );
                println!();
            }
            Err(e) => {
                failed += 1;
                println!("✗ \"{}\": Error - {}\n", prompt, e);
            }
        }
    }

    println!("=== Summary ===");
    println!("Passed: {}/{}", passed, passed + failed);
    println!("Failed: {}/{}", failed, passed + failed);

    // Benchmark latency
    println!("\n=== Latency Benchmark ===");
    let iterations = 1000;

    // Warm up
    for _ in 0..100 {
        let _ = pipeline.run("5 + 3");
    }

    // Measure
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = pipeline.run("5 + 3");
    }
    let elapsed = start.elapsed();

    let avg_ns = elapsed.as_nanos() / iterations as u128;
    let avg_us = avg_ns as f64 / 1000.0;

    println!("Iterations: {}", iterations);
    println!("Total time: {:?}", elapsed);
    println!("Average latency: {:.2} μs ({} ns)", avg_us, avg_ns);

    if avg_us < 500.0 {
        println!("✓ Target met: <500μs per inference");
    } else {
        println!("✗ Target missed: expected <500μs");
    }
}
