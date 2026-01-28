# nl generate

Generate code from natural language using the AI model.

## Usage

```
nl generate [OPTIONS] <PROMPT>
```

## Options

| Option | Description |
|--------|-------------|
| `<PROMPT>` | Natural language description (required) |
| `--model <FILE>` | Model path (default: `./model.onnx`) |
| `--engine <ENGINE>` | Inference engine: `ort`, `tract`, `candle`, `burn`, `auto` |
| `--show-asm` | Display generated assembly code |
| `--max-retries <N>` | Maximum retry attempts (default: 3) |
| `-o, --output <FILE>` | Save generated binary to file |
| `-v, --verbose` | Show debug info and each attempt |

## Examples

```bash
# Simple arithmetic
nl generate "add 5 and 3"

# Show generated assembly
nl generate "calculate factorial of 10" --show-asm

# Use specific inference engine
nl generate "multiply 7 by 8" --engine tract

# Save to file
nl generate "fibonacci of 15" -o fib.nlb

# Custom model
nl generate "add 5 and 3" --model models/custom.onnx
```

## Inference Engines

| Engine | Binary Size | GPU | Best For |
|--------|-------------|-----|----------|
| `ort` | +15 MB | CUDA, TensorRT, CoreML | Production, GPU |
| `tract` | +2 MB | None | Minimal binary, embedded |
| `candle` | +10 MB | CUDA | HuggingFace ecosystem |
| `burn` | +45 MB | wgpu, CUDA | Native training |

## Output

```
Generating code from: "add 5 and 3"
Loading model: model.onnx

Generated program (3 instructions):
Result: 8
Latency: 127us
```

With `--show-asm`:

```
Generating code from: "add 5 and 3"
Loading model: model.onnx

Assembly:
  mov r0, 5
  mov r1, 3
  add r0, r0, r1
  halt

Result: 8
```

## Supported Prompts

| Category | Example Prompts |
|----------|-----------------|
| Arithmetic | "add 5 and 3", "5 + 3", "subtract 10 from 20" |
| Math | "factorial of 5", "5!", "fibonacci 10" |
| Power | "5 squared", "2^8", "power of 3 by 4" |
| Bitwise | "5 AND 3", "bitwise or of 7 and 2" |
| Comparison | "max of 5 and 8", "min(3, 7)" |

## See Also

- [chat](chat.md) - Interactive generation mode
- [train](train.md) - Train the model
- [datagen](datagen.md) - Generate training data
