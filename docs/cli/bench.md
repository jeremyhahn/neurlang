# nl bench

Run performance benchmarks.

## Usage

```
nl bench [OPTIONS]
```

## Options

| Option | Description |
|--------|-------------|
| `-b, --bench-type <TYPE>` | Benchmark type: `compile`, `fib`, `all` |
| `-i, --iterations <N>` | Number of iterations (default: 1000) |

## Examples

```bash
# Run all benchmarks
nl bench

# Run compile benchmark only
nl bench -b compile

# More iterations for accuracy
nl bench -b compile -i 10000
```

## Benchmark Types

| Type | Description |
|------|-------------|
| `compile` | Measure copy-and-patch compilation time |
| `fib` | Fibonacci computation (interpreter vs JIT) |
| `all` | Run all benchmarks |

## Output

```
Neurlang Benchmarks
================
Iterations: 1000

Compile Time Benchmark:
-----------------------
  Instructions: 9
  Iterations: 1000
  Total time: 3.456ms
  Average: 3456 ns (3.46 us)
  Target: <5000 ns (<5 us)

Fibonacci Benchmark:
--------------------
  Interpreter:
    Iterations: 100
    Avg time: 12.34us
    Avg instructions: 142
  JIT:
    Compile time: 2.15 us
    Iterations: 1000
    Avg exec time: 89ns
```

## See Also

- [run](run.md) - Execute programs
- [spec](spec.md) - IR specification
