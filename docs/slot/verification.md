# Slot Verification System

This document describes how Neurlang verifies that generated code is correct at every level.

## Verification Layers

```
+===========================================================================+
|                        VERIFICATION ARCHITECTURE                           |
+===========================================================================+
|                                                                            |
|  LAYER 1: SOURCE VERIFICATION                                              |
|  ============================                                              |
|                                                                            |
|  lib/ (Stdlib)                      examples/                              |
|  +------------------+               +------------------+                   |
|  | Rust Source      |               | Hand-written .nl |                   |
|  | (oracle)         |               | + @test cases    |                   |
|  +--------+---------+               +--------+---------+                   |
|           |                                  |                             |
|           v                                  |                             |
|  +------------------+                        |                             |
|  | nl stdlib build  |                        |                             |
|  | (Rust->IR)       |                        |                             |
|  +--------+---------+                        |                             |
|           |                                  |                             |
|           v                                  v                             |
|  +------------------+               +------------------+                   |
|  | lib/*.nl         |               | nl test -p X     |                   |
|  | (@test from Rust)|               | (runs @test)     |                   |
|  +--------+---------+               +------------------+                   |
|           |                                                                |
|           v                                                                |
|  +------------------+                                                      |
|  | nl stdlib verify |                                                      |
|  | Rust == .nl      |                                                      |
|  +------------------+                                                      |
|                                                                            |
+===========================================================================+
|                                                                            |
|  LAYER 2: SPEC VALIDATION                                                  |
|  ========================                                                  |
|                                                                            |
|  specs/protocols/*.json                                                    |
|  +------------------+                                                      |
|  | Protocol Spec    |                                                      |
|  +--------+---------+                                                      |
|           |                                                                |
|           v                                                                |
|  +------------------+                                                      |
|  | nl spec validate |                                                      |
|  +--------+---------+                                                      |
|           |                                                                |
|           v                                                                |
|  +-----------------------------------------------+                         |
|  | CHECKS:                                       |                         |
|  | [x] Exactly one initial state                 |                         |
|  | [x] All states reachable from initial         |                         |
|  | [x] No undefined state references             |                         |
|  | [x] Valid pattern syntax (balanced braces)    |                         |
|  | [x] State transitions lead to defined states  |                         |
|  | [x] No duplicate state/command names          |                         |
|  |                                               |                         |
|  | WARNINGS:                                     |                         |
|  | [!] Unreachable states                        |                         |
|  | [!] Commands without test coverage            |                         |
|  | [!] Missing error handlers                    |                         |
|  | [!] Dead-end states (no outgoing transitions) |                         |
|  +-----------------------------------------------+                         |
|                                                                            |
+===========================================================================+
|                                                                            |
|  LAYER 3: SLOT VERIFICATION                                                |
|  ==========================                                                |
|                                                                            |
|  For each slot in SlotSpec:                                                |
|  +------------------+                                                      |
|  | Slot Definition  |                                                      |
|  | + SlotTest       |                                                      |
|  +--------+---------+                                                      |
|           |                                                                |
|           v                                                                |
|  +------------------+                                                      |
|  | SlotFiller       |                                                      |
|  | (model/template) |                                                      |
|  +--------+---------+                                                      |
|           |                                                                |
|           v                                                                |
|  +------------------+                                                      |
|  | Generated Code   |                                                      |
|  +--------+---------+                                                      |
|           |                                                                |
|           v                                                                |
|  +------------------+                                                      |
|  | SlotVerifier     |                                                      |
|  +--------+---------+                                                      |
|           |                                                                |
|           v                                                                |
|  +-----------------------------------------------+                         |
|  | Build test program:                           |                         |
|  |   1. Setup code (from SlotTest.setup)         |                         |
|  |   2. Slot code under test                     |                         |
|  |   3. Verification code (check registers/mem)  |                         |
|  |                                               |                         |
|  | Execute and verify:                           |                         |
|  |   - Expected registers match                  |                         |
|  |   - Expected memory contents match            |                         |
|  |   - Expected branch taken                     |                         |
|  +-----------------------------------------------+                         |
|           |                                                                |
|       +---+---+                                                            |
|       |       |                                                            |
|    PASS     FAIL                                                           |
|       |       |                                                            |
|       v       v                                                            |
|    Cache    Retry (up to N times)                                          |
|                                                                            |
+===========================================================================+
|                                                                            |
|  LAYER 4: INTEGRATION VERIFICATION                                         |
|  =================================                                         |
|                                                                            |
|  After all slots assembled:                                                |
|  +------------------+                                                      |
|  | Complete .nl     |                                                      |
|  +--------+---------+                                                      |
|           |                                                                |
|           v                                                                |
|  +------------------+                                                      |
|  | nl spec test     |                                                      |
|  +--------+---------+                                                      |
|           |                                                                |
|           v                                                                |
|  +-----------------------------------------------+                         |
|  | For each TestCase in spec.tests:              |                         |
|  |   1. Start server                             |                         |
|  |   2. Connect as client                        |                         |
|  |   3. Execute send/expect steps:               |                         |
|  |      - expect: "220"          (check greeting)|                         |
|  |      - send: "HELO test.com"                  |                         |
|  |      - expect: "250 Hello"                    |                         |
|  |   4. Verify all steps pass                    |                         |
|  +-----------------------------------------------+                         |
|                                                                            |
+===========================================================================+
```

## Layer 1: Source Verification

### Stdlib (lib/)

The stdlib is **generated from Rust**, making Rust the oracle:

```bash
# Generate .nl from Rust source
nl stdlib build

# Verify Rust output == .nl output
nl stdlib verify
```

**How it works:**

1. Rust function runs with test inputs
2. Generated .nl runs with same inputs
3. Outputs must match exactly

```rust
// stdlib/src/math.rs
pub fn factorial(n: u64) -> u64 {
    let mut result = 1;
    for i in 1..=n { result *= i; }
    result
}
// Rust says: factorial(5) = 120

// lib/math/factorial.nl
; @test: r0=5 -> r0=120
// .nl must also produce 120
```

### Examples (examples/)

Examples are hand-written with `@test` annotations:

```asm
; examples/algorithm/gcd.nl
; @test: r0=48, r1=18 -> r0=6
; @test: r0=100, r1=25 -> r0=25

.entry:
    ; ... implementation ...
```

Run tests:
```bash
nl test -p examples
```

## Layer 2: Spec Validation

Protocol specs are validated before use:

```bash
nl protocol -i specs/protocols/smtp.json --validate
```

### Errors (Spec is Invalid)

| Error | Description |
|-------|-------------|
| `NoInitialState` | No state has `initial: true` |
| `MultipleInitialStates` | More than one initial state |
| `UndefinedState` | Reference to non-existent state |
| `InvalidTransition` | `next_state` references undefined state |
| `DuplicateState` | Two states with same name |
| `DuplicateCommand` | Two commands with same name |
| `InvalidPattern` | Malformed pattern (unclosed `{`) |

### Warnings (Spec is Valid but May Have Issues)

| Warning | Description |
|---------|-------------|
| `UnreachableState` | State can't be reached from initial |
| `NoTerminalState` | No state has `terminal: true` |
| `UntestedCommand` | Command not covered by any test |
| `DeadEndState` | Non-terminal state with no outgoing transitions |
| `MissingErrorHandler` | No handler for common error type |

### Example Output

```
$ nl protocol -i specs/protocols/smtp.json --validate

Protocol: smtp v1.0

Validating spec...
  ✓ Spec is VALID
    6 states, 8 commands, 4 tests

  Warnings (7):
    ⚠ UnreachableState("QUIT")
    ⚠ UntestedCommand("RSET")
    ⚠ UntestedCommand("NOOP")
```

## Layer 3: Slot Verification

Each slot has an optional unit test:

```rust
pub struct SlotTest {
    pub setup: String,           // Assembly to set up test state
    pub input: TestInput,        // Input registers/memory
    pub expected: TestExpected,  // Expected output
}

pub struct TestExpected {
    pub registers: HashMap<String, u64>,  // r0=120
    pub memory: HashMap<String, Vec<u8>>, // buffer contents
    pub branch_taken: Option<String>,     // which label reached
}
```

### Verification Process

```rust
// SlotVerifier builds a test program:

; Test setup (from SlotTest.setup)
mov r0, 5
mov r13, STATE_INIT

; Slot code under test
{{generated_slot_code}}

; Verification
mov r30, 120
beq r0, r30, .test_pass_r0
trap 1  ; Test failed
.test_pass_r0:
halt
```

### Handling Failures

```
Slot PATTERN_MATCH_HELO: FAILED
  Expected: r3 = pointer to captured domain
  Actual: r3 = 0

  Retrying... (attempt 2/10)
  Slot PATTERN_MATCH_HELO: PASS
```

## Layer 4: Integration Verification

After assembly, run protocol-level tests:

```bash
nl protocol -i specs/protocols/smtp.json --test --program smtp_server.nl
```

### Test Format

From the spec's `tests` section:

```json
{
  "tests": [
    {
      "name": "basic_session",
      "steps": [
        { "expect": "220" },
        { "send": "HELO test.com\r\n", "expect": "250 Hello test.com" },
        { "send": "QUIT\r\n", "expect": "221 Bye" }
      ]
    },
    {
      "name": "wrong_sequence",
      "steps": [
        { "expect": "220" },
        { "send": "MAIL FROM:<x@y.com>\r\n", "expect": "503 Bad sequence" }
      ]
    }
  ]
}
```

### Test Execution

```
$ nl protocol -i specs/protocols/smtp.json --test --program smtp_server.nl

Protocol: smtp v1.0

Running integration tests against: smtp_server.nl

  Tests to run (4):
    - basic_session
    - wrong_sequence
    - unknown_command
    - ehlo_capabilities

Running 4 integration tests...

  basic_session: PASS (23ms)
  wrong_sequence: PASS (12ms)
  unknown_command: PASS (11ms)
  ehlo_capabilities: PASS (15ms)

All 4 tests passed in 61ms
```

## Verification Commands Summary

| Command | What It Verifies |
|---------|------------------|
| `nl stdlib build` | Generate .nl from Rust |
| `nl stdlib verify` | Rust output == .nl output |
| `nl test -p lib` | Run @test cases in lib/ |
| `nl test -p examples` | Run @test cases in examples/ |
| `nl protocol -i X.json --validate` | Validate spec structure |
| `nl protocol -i X.json --test --program P.nl` | Run spec's integration tests |

## Writing Good Tests

### For Stdlib Functions

Tests are auto-generated from Rust:

```rust
// stdlib/src/math.rs
/// @test: factorial(0) = 1
/// @test: factorial(5) = 120
/// @test: factorial(10) = 3628800
pub fn factorial(n: u64) -> u64 { ... }
```

### For Examples

Include edge cases:

```asm
; @test: r0=0 -> r0=1              ; base case
; @test: r0=1 -> r0=1              ; trivial
; @test: r0=5 -> r0=120            ; normal case
; @test: r0=20 -> r0=2432902008176640000  ; large value
```

### For Protocol Specs

Test happy path AND error cases:

```json
{
  "tests": [
    { "name": "happy_path", "steps": [...] },
    { "name": "wrong_sequence", "steps": [...] },
    { "name": "unknown_command", "steps": [...] },
    { "name": "invalid_argument", "steps": [...] }
  ]
}
```

## See Also

- [Slot Types Reference](./slot-types.md)
- [Protocol Specification Format](./protocol-specs.md)
- [Training Data Format](./training-format.md)
- [Glossary](../GLOSSARY.md)
