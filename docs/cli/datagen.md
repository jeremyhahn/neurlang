# nl datagen

Generate synthetic training data for the AI model.

## Usage

```
nl datagen [OPTIONS]
```

## Options

| Option | Description |
|--------|-------------|
| `-o, --output <FILE>` | Output file path (default: `training_data.jsonl`) |
| `-n, --num-examples <N>` | Number of examples to generate (default: 100000) |
| `-l, --level <N>` | Curriculum level 1-5 (default: 5) |
| `--seed <N>` | Random seed for reproducibility (default: 42) |
| `--include-examples` | Include examples from `examples/` directory |

## Curriculum Levels

| Level | Description | Categories |
|-------|-------------|------------|
| 1 | Basic | Arithmetic, move, immediate values |
| 2 | Control Flow | Conditionals, loops |
| 3 | Memory | Load/store, arrays |
| 4 | Functions | Call, return, stack |
| 5 | Advanced | Algorithms, concurrency, security, crypto |

## Examples

```bash
# Generate 100k examples at all levels
nl datagen -o training_data.jsonl -n 100000

# Generate only basic arithmetic examples
nl datagen -o basic.jsonl -n 10000 --level 1

# Include real examples from examples/ directory
nl datagen -o data.jsonl --include-examples

# Reproducible generation
nl datagen -o data.jsonl --seed 12345
```

## Output Format

Each line is a JSON object:

```json
{
  "prompt": "add 5 and 3",
  "binary_ir": [80, 128, 0, 0, 5, ...],
  "assembly": "mov r0, 5\nmov r1, 3\nadd r0, r0, r1\nhalt",
  "expected_output": 8,
  "level": 1,
  "category": "arithmetic"
}
```

## Output

```
Generating 100000 training examples...
  Level 1 (basic): 20000 examples
  Level 2 (control): 20000 examples
  Level 3 (memory): 20000 examples
  Level 4 (functions): 20000 examples
  Level 5 (advanced): 20000 examples

Wrote 100000 examples to training_data.jsonl
File size: 45.2 MB
```

## Categories Generated

- Arithmetic: add, sub, mul, div, mod
- Bitwise: and, or, xor, shl, shr
- Math: factorial, fibonacci, power, sqrt, gcd
- Comparisons: max, min, sign, is_even, is_prime
- Memory: load, store, memcpy, array_sum
- Loops: for, while, countdown
- Functions: call, return, recursion
- Algorithms: sort, search
- Crypto: sha256, aes, hmac

## See Also

- [train](train.md) - Train the model
- [test-accuracy](test-accuracy.md) - Test model accuracy
