# Neurlang Test Coverage Design

## Current State

- 222 total .nl files (109 examples + 113 lib)
- 180 files have @test annotations
- 42 are server patterns (untestable via unit tests)
- Only 7 testable files are missing tests
- 445 total test cases

## Coverage Levels

### Level 1: File Coverage (DONE)
Track which files have @test annotations.
- Tool: `train/analyze_coverage.py`
- Current: 100% of testable files covered

### Level 2: Instruction Coverage (PROPOSED)

Add instrumentation to track which instructions execute during tests.

```rust
// Add to src/interp/dispatch.rs

pub struct CoverageTracker {
    /// Bitmap of executed instruction indices
    executed: BitVec,
    /// Count per instruction (for hot path analysis)
    counts: Vec<u64>,
    /// Branch targets taken (source_pc -> target_pc)
    branches: HashMap<usize, HashSet<usize>>,
    /// Total branches possible vs taken
    branch_coverage: (usize, usize),
}

impl Interpreter {
    pub fn with_coverage(mut self) -> Self {
        self.coverage = Some(CoverageTracker::new());
        self
    }

    fn execute_instruction(&mut self, instr: &Instruction) -> Result<ControlFlow, InterpResult> {
        // Track coverage
        if let Some(ref mut cov) = self.coverage {
            cov.mark_executed(self.pc);
        }
        // ... existing dispatch ...
    }
}
```

### Level 3: Branch Coverage (PROPOSED)

Track which branch paths are taken:

```rust
// In execute_instruction for branch opcodes
Opcode::Beq | Opcode::Bne | ... => {
    if let Some(ref mut cov) = self.coverage {
        cov.mark_branch(self.pc, condition, target);
    }
}
```

### Level 4: Source Line Mapping (FUTURE)

Map instruction addresses back to source lines:

```rust
pub struct SourceMap {
    /// instruction_index -> (file, line_number)
    mappings: Vec<(PathBuf, usize)>,
}
```

## CLI Integration

```bash
# Run tests with coverage
nl test -p examples --coverage

# Generate coverage report
nl coverage --format html --output coverage/

# Check coverage threshold
nl test -p examples --coverage --min-coverage 80
```

## Report Format

```
COVERAGE REPORT: examples/patterns/caching/lru_cache.nl

Instructions: 45/52 (86.5%)
Branches: 8/10 (80.0%)
Lines: 34/40 (85.0%)

Uncovered:
  Line 45: mov r0, 0  ; error path not tested
  Line 67-70: ; cleanup block not reached

Branch Details:
  Line 23: beq r0, zero, cache_miss
    ✓ Taken: 2 times
    ✓ Not taken: 3 times

  Line 45: bgt r1, r2, evict_oldest
    ✓ Taken: 1 time
    ✗ Not taken: never tested
```

## Implementation Plan

### Phase 1: Basic Instruction Coverage
1. Add `CoverageTracker` struct
2. Instrument interpreter's `execute_instruction`
3. Add `--coverage` flag to `nl test`
4. Generate simple text report

### Phase 2: Branch Coverage
1. Track conditional branch outcomes
2. Identify untested branch paths
3. Add branch coverage to report

### Phase 3: Source Mapping
1. Store line info during assembly
2. Map coverage data to source lines
3. Generate HTML reports with highlighted source

### Phase 4: Integration
1. CI integration (fail if coverage drops)
2. Coverage badges
3. Historical tracking

## Quick Win: Coverage Script

Before full implementation, extend `analyze_coverage.py` to:
1. Run each test
2. Count executed instructions via `--trace` flag
3. Compare to total instructions in file

```python
def measure_runtime_coverage(nl_file: Path) -> float:
    """Run tests and measure instruction coverage."""
    # Run with tracing
    result = subprocess.run(
        ['./target/release/nl', 'run', '-i', str(nl_file), '--trace'],
        capture_output=True
    )
    # Parse trace output for instruction count
    executed = parse_trace(result.stdout)
    total = count_instructions(nl_file)
    return executed / total
```

## Rust Test Coverage (Existing)

The Rust runtime itself can use llvm-cov:

```bash
# Install
cargo install cargo-llvm-cov

# Run coverage for Rust code
cargo llvm-cov --html

# View report
open target/llvm-cov/html/index.html
```

This covers the compiler, assembler, and runtime - but not the Neurlang programs themselves.
