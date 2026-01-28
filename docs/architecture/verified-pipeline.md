# Verified Pipeline Architecture

This document describes the implementation plan for a **mathematically verified** inference pipeline that either produces correct code OR returns an explicit error.

## Overview

The verified pipeline guarantees: **Correct output OR explicit error. Never silent wrong code.**

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    VERIFIED INFERENCE PIPELINE                          │
│                                                                         │
│  "add 5 and 3"                                                          │
│       │                                                                 │
│       ▼                                                                 │
│  ┌─────────────────┐                                                    │
│  │ Intent Classify │  VERIFIED: Exhaustive testing over all 54 intents │
│  │ (confidence)    │  Returns Err(UnknownIntent) if conf < threshold   │
│  └────────┬────────┘                                                    │
│           │ intent_id = 0 (ADD), confidence = 0.95                      │
│           ▼                                                             │
│  ┌─────────────────┐                                                    │
│  │ Operand Extract │  VERIFIED: Regex-based, deterministic             │
│  │ (parse numbers) │  Returns Err(ParseFailed) if extraction fails     │
│  └────────┬────────┘                                                    │
│           │ operands = [5, 3]                                           │
│           ▼                                                             │
│  ┌─────────────────┐                                                    │
│  │ IR Generator    │  VERIFIED: Each of 54 generators proven correct   │
│  │ (deterministic) │  Returns Err(GeneratorFailed) on invalid input    │
│  └────────┬────────┘                                                    │
│           │ IR: mov r0, 5 / mov r1, 3 / add r0, r0, r1                  │
│           ▼                                                             │
│  ┌─────────────────┐                                                    │
│  │ Compiler        │  VERIFIED: Stencils verified against x86-64 spec  │
│  │ (copy-patch)    │  Returns Err(CompileFailed) on invalid IR         │
│  └────────┬────────┘                                                    │
│           │ x86-64 machine code                                         │
│           ▼                                                             │
│  ┌─────────────────┐                                                    │
│  │ Test Verify     │  VERIFIED: Rust is oracle, tests are complete     │
│  │ (optional)      │  Returns Err(TestFailed) if any test fails        │
│  └────────┬────────┘                                                    │
│           │                                                             │
│           ▼                                                             │
│       SUCCESS: Verified correct program                                 │
│          OR                                                             │
│       ERROR: Explicit failure with reason                               │
└─────────────────────────────────────────────────────────────────────────┘
```

## Verification Modes

The pipeline supports three verification modes via a `--verify` flag:

| Mode | Description | Latency Overhead | Use Case |
|------|-------------|------------------|----------|
| `none` | No verification, fastest | 0 | Production, speed-critical |
| `quick` | Confidence threshold only | ~0.01ms | Default, catches obvious errors |
| `full` | Run test suite after generation | ~1-10ms | Development, high-assurance |

### CLI Flag

```bash
# No verification (fastest, "let er rip")
nl prompt "add 5 and 3" --verify=none

# Quick verification (default)
nl prompt "add 5 and 3" --verify=quick

# Full verification with test execution
nl prompt "add 5 and 3" --verify=full
```

## Latency Analysis

### Overhead by Mode

| Stage | none | quick | full |
|-------|------|-------|------|
| Intent classification | 0.05ms | 0.05ms | 0.05ms |
| Confidence check | - | 0.001ms | 0.001ms |
| Operand extraction | 0.01ms | 0.01ms | 0.01ms |
| IR generation | 0.01ms | 0.01ms | 0.01ms |
| Compilation | 0.005ms | 0.005ms | 0.005ms |
| Test execution | - | - | 1-10ms |
| **Total** | **~0.08ms** | **~0.08ms** | **~1-10ms** |

### When to Use Each Mode

| Scenario | Recommended Mode |
|----------|------------------|
| Real-time inference (< 1ms budget) | `none` or `quick` |
| API server (latency-sensitive) | `quick` |
| Batch processing | `full` |
| Development/debugging | `full` |
| Safety-critical applications | `full` |

## Implementation Plan

### Phase 1: Confidence-Based Rejection (Week 1)

Add confidence threshold checking to reject low-confidence classifications.

```rust
pub struct VerifiedPipeline {
    inner: RagPipeline,
    mode: VerifyMode,
    confidence_threshold: f32,
}

pub enum VerifyMode {
    None,   // No verification
    Quick,  // Confidence check only
    Full,   // Run tests after generation
}

pub enum VerifiedResult {
    Success {
        program: Program,
        confidence: f32,
        tests_passed: Option<usize>,
    },
    Error(VerifyError),
}

pub enum VerifyError {
    UnknownIntent { query: String, best_confidence: f32 },
    LowConfidence { intent: String, confidence: f32, threshold: f32 },
    OperandParseFailed { reason: String },
    GeneratorFailed { intent: usize, reason: String },
    CompileFailed { reason: String },
    TestFailed { passed: usize, failed: usize, errors: Vec<String> },
    MissingExtension { name: String },
}
```

**Files to modify:**
- `src/inference/pipeline.rs` - Add VerifiedPipeline wrapper
- `src/main.rs` - Add `--verify` flag

**Latency added:** ~0.001ms (one comparison)

### Phase 2: Test Suite Integration (Week 2)

Integrate the existing `Verifier` with the pipeline for `--verify=full` mode.

```rust
impl VerifiedPipeline {
    pub fn run_verified(&mut self, prompt: &str) -> VerifiedResult {
        // 1. Run normal pipeline
        let result = self.inner.run(prompt)?;

        // 2. Skip tests if mode is none/quick
        if self.mode == VerifyMode::None || self.mode == VerifyMode::Quick {
            return VerifiedResult::Success {
                program: result.program,
                confidence: result.confidence,
                tests_passed: None,
            };
        }

        // 3. Run tests for full verification
        let suite = self.get_test_suite(result.intent_id);
        let verification = self.verifier.verify(&result.program, &suite);

        if verification.all_passed {
            VerifiedResult::Success {
                program: result.program,
                confidence: result.confidence,
                tests_passed: Some(verification.passed_count),
            }
        } else {
            VerifiedResult::Error(VerifyError::TestFailed {
                passed: verification.passed_count,
                failed: verification.failed_count,
                errors: verification.format_errors().lines().map(String::from).collect(),
            })
        }
    }
}
```

**Files to modify:**
- `src/inference/pipeline.rs` - Add test suite lookup
- `src/inference/verify.rs` - Already exists, add intent→suite mapping

**Latency added:** ~1-10ms (test execution)

### Phase 3: Generator Verification (Week 3)

Add property-based testing to prove generators correct.

```rust
// Property: For all valid operands, generator produces IR that computes correct result
#[cfg(test)]
mod generator_proofs {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn add_generator_correct(a in 0i64..1000, b in 0i64..1000) {
            let gen = AddGenerator;
            let program = gen.generate(&[a, b]).unwrap();

            let mut regs = [0u64; 32];
            execute(&program, &mut regs).unwrap();

            assert_eq!(regs[0] as i64, a + b);
        }

        #[test]
        fn factorial_generator_correct(n in 0u64..13) {
            let gen = FactorialGenerator;
            let program = gen.generate(&[n as i64]).unwrap();

            let mut regs = [0u64; 32];
            execute(&program, &mut regs).unwrap();

            assert_eq!(regs[0], factorial_rust(n));
        }
    }
}
```

**Files to modify:**
- `src/inference/generators.rs` - Add property tests for all 54 generators

**Latency added:** 0 (tests run at build time)

### Phase 4: Intent Classifier Exhaustive Testing (Week 4)

Prove that for all training examples, classification is correct above threshold.

```rust
#[test]
fn intent_classifier_exhaustive() {
    let index = IntentIndex::load("intent_index.bin").unwrap();
    let embedder = create_embedder_auto().unwrap();

    // Test all 54 canonical descriptions
    for (id, desc) in INTENT_DESCRIPTIONS.iter().enumerate() {
        let emb = embedder.embed(desc).unwrap();
        let (pred_id, conf) = index.classify(&emb);

        assert_eq!(pred_id, id, "Canonical description misclassified");
        assert!(conf > 0.7, "Low confidence for canonical");
    }

    // Test all training examples
    for example in load_training_examples() {
        let emb = embedder.embed(&example.prompt).unwrap();
        let (pred_id, conf) = index.classify(&emb);

        if conf > 0.7 {
            assert_eq!(pred_id, example.intent_id,
                "High-confidence misclassification: {}", example.prompt);
        }
    }
}
```

**Files to modify:**
- `src/inference/intent_index.rs` - Add exhaustive tests

**Latency added:** 0 (tests run at build time)

## API Design

### Rust API

```rust
use neurlang::inference::{VerifiedPipeline, VerifyMode, VerifiedResult};

// Create pipeline with verification mode
let mut pipeline = VerifiedPipeline::new(VerifyMode::Quick)?;

// Run with verification
match pipeline.run("compute factorial of 5") {
    VerifiedResult::Success { program, confidence, tests_passed } => {
        println!("Generated program with {:.1}% confidence", confidence * 100.0);
        if let Some(n) = tests_passed {
            println!("Passed {} tests", n);
        }
        // Execute program...
    }
    VerifiedResult::Error(e) => {
        eprintln!("Verification failed: {}", e);
        // Handle error...
    }
}
```

### CLI API

```bash
# Default (quick verification)
nl prompt "add 5 and 3"
# Output: Result: 8 (confidence: 95.2%)

# No verification (fastest)
nl prompt "add 5 and 3" --verify=none
# Output: Result: 8

# Full verification
nl prompt "add 5 and 3" --verify=full
# Output: Result: 8 (confidence: 95.2%, 6/6 tests passed)

# Error example
nl prompt "frozzle the widgets" --verify=quick
# Error: Unknown intent (best match: "process list" at 32.1% confidence)
```

## Error Handling Matrix

| Error Type | Cause | Recovery |
|------------|-------|----------|
| `UnknownIntent` | Query doesn't match any known intent | Return error, suggest similar intents |
| `LowConfidence` | Confidence below threshold | Return error OR fallback to full model |
| `OperandParseFailed` | Couldn't extract numbers from query | Return error with parse failure reason |
| `GeneratorFailed` | Generator received invalid input | Return error (should be rare) |
| `CompileFailed` | IR invalid for compiler | Return error (indicates bug) |
| `TestFailed` | Generated code doesn't pass tests | Return error with test failures |
| `MissingExtension` | RAG couldn't resolve extension | Return error with extension name |

## Testing Strategy

### Unit Tests

| Component | Test Method | Coverage |
|-----------|-------------|----------|
| Intent classifier | Exhaustive over all 54 | 100% of intents |
| Generators | Property-based (proptest) | All valid inputs |
| Compiler | Stencil verification | All 32 opcodes |
| Verifier | Known test suites | All common operations |

### Integration Tests

```bash
# Run all verification tests
cargo test --test verified_pipeline

# Run property tests (slow but thorough)
cargo test --test generator_proofs --release

# Run exhaustive intent tests
cargo test --test intent_exhaustive --release
```

### Benchmarks

```bash
# Benchmark verification overhead
nl bench --type verify --iterations 10000

# Expected output:
# Verify Mode   | Mean Latency | P99 Latency
# ------------- | ------------ | -----------
# none          | 0.08ms       | 0.12ms
# quick         | 0.08ms       | 0.12ms
# full          | 2.3ms        | 8.1ms
```

## Configuration

### Environment Variables

```bash
# Default verification mode
export NEURLANG_VERIFY_MODE=quick

# Confidence threshold (0.0-1.0)
export NEURLANG_CONFIDENCE_THRESHOLD=0.7

# Enable verbose verification logging
export NEURLANG_VERIFY_VERBOSE=1
```

### Config File

```toml
# ~/.neurlang/config.toml

[verification]
default_mode = "quick"      # none, quick, full
confidence_threshold = 0.7
verbose = false

[verification.full]
timeout_ms = 10000          # Max time for test execution
max_test_cases = 100        # Limit test cases per run
```

## Summary

| Verification Mode | Latency | Guarantees |
|-------------------|---------|------------|
| `none` | ~0.08ms | None (fastest) |
| `quick` | ~0.08ms | Rejects low-confidence (<0.7) |
| `full` | ~1-10ms | Tests pass OR error returned |

**Key insight:** The `--verify=none` flag lets users who want maximum speed disable verification entirely, while `--verify=full` provides mathematical certainty for safety-critical applications.

## See Also

- [RAG Intent Index](./rag-intent-index.md) - Intent classification system
- [Performance Benchmarks](../PERFORMANCE.md) - Latency measurements
- [Generators](../inference/generators.md) - IR generator reference
