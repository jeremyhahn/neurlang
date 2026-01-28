# nl test-accuracy

Test model accuracy on benchmark or custom test data.

## Usage

```
nl test-accuracy [OPTIONS]
```

## Options

| Option | Description |
|--------|-------------|
| `--model <FILE>` | Model path (default: `./model.onnx`) |
| `--benchmark` | Use built-in benchmark test suite |
| `--test-data <FILE>` | Custom test data path (JSONL) |
| `-v, --verbose` | Show each test case result |

## Examples

```bash
# Test with built-in benchmarks
nl test-accuracy --benchmark

# Test with custom data
nl test-accuracy --test-data test.jsonl

# Verbose output (show each test)
nl test-accuracy --benchmark --verbose

# Custom model
nl test-accuracy --model models/custom.onnx --benchmark
```

## Output

```
Testing model: model.onnx
Using benchmark test suite (500 cases)

Running tests...
  Arithmetic: 100/100 (100.0%)
  Loops: 98/100 (98.0%)
  Memory: 95/100 (95.0%)
  Functions: 92/100 (92.0%)
  Advanced: 88/100 (88.0%)

Overall Accuracy: 94.6% (473/500)
Average Latency: 0.12ms
```

With `--verbose`:

```
Testing model: model.onnx
Using benchmark test suite

[PASS] "add 5 and 3" -> 8 (expected: 8)
[PASS] "multiply 7 by 8" -> 56 (expected: 56)
[FAIL] "fibonacci of 12" -> 89 (expected: 144)
...

Overall Accuracy: 94.6% (473/500)
```

## Benchmark Categories

| Category | Tests | Description |
|----------|-------|-------------|
| Arithmetic | 100 | add, sub, mul, div, mod |
| Loops | 100 | for, while, countdown |
| Memory | 100 | load, store, arrays |
| Functions | 100 | call, return, recursion |
| Advanced | 100 | algorithms, crypto |

## See Also

- [train](train.md) - Train the model
- [generate](generate.md) - Generate code from prompts
