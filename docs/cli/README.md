# CLI Reference

Command-line interface documentation for Neurlang.

## nl Commands

The `nl` binary provides all functionality for training, inference, and program execution.

### AI Code Generation

| Command | Description | Documentation |
|---------|-------------|---------------|
| `nl generate` | Generate code from natural language | [generate.md](generate.md) |
| `nl chat` | Interactive code generation | [chat.md](chat.md) |
| `neurlang agent` | Autonomous agent with session management | [agent.md](agent.md) |

### Training

| Command | Description | Documentation |
|---------|-------------|---------------|
| `nl datagen` | Generate synthetic training data | [datagen.md](datagen.md) |
| `nl train` | Train the AI model (local or remote) | [nl-train.md](nl-train.md) |
| `nl test-accuracy` | Test model accuracy | [test-accuracy.md](test-accuracy.md) |
| `nl export-onnx` | Export PyTorch model to ONNX | [export-onnx.md](export-onnx.md) |

### Program Execution

| Command | Description | Documentation |
|---------|-------------|---------------|
| `nl run` | Execute a program (JIT or interpreter) | [run.md](run.md) |
| `nl repl` | Interactive REPL | [repl.md](repl.md) |

### Compilation

| Command | Description | Documentation |
|---------|-------------|---------------|
| `nl asm` | Assemble text to binary IR | [asm.md](asm.md) |
| `nl disasm` | Disassemble binary to text | [disasm.md](disasm.md) |
| `nl compile` | Compile to standalone native code | [compile.md](compile.md) |

### Utilities

| Command | Description | Documentation |
|---------|-------------|---------------|
| `nl bench` | Run performance benchmarks | [bench.md](bench.md) |
| `nl spec` | Show IR specification | [spec.md](spec.md) |

## Quick Start

```bash
# Generate training data
nl datagen -o training_data.jsonl -n 100000

# Train the model
nl train --data training_data.jsonl --epochs 20 --output model.onnx

# Test model accuracy
nl test-accuracy --model model.onnx --benchmark

# Generate code
nl generate "add 5 and 3"
# Result: 8

# Interactive mode
nl chat
```

## Python Training Scripts

Located in `train/multihead/` for Docker backend:

| Script | Description | Documentation |
|--------|-------------|---------------|
| `generate_data.py` | Generate training data | [generate_data.md](generate_data.md) |
| `train.py` | Train multi-head model | [train.md](train.md) |
| `evaluate.py` | Evaluate model accuracy | [evaluate.md](evaluate.md) |
| `export_onnx.py` | Export model to ONNX | [export_onnx.md](export_onnx.md) |

## Global Options

```
-h, --help       Show help message
-V, --version    Show version
```

## File Extensions

| Extension | Description |
|-----------|-------------|
| `.asm` | Text assembly source |
| `.nlb` | Binary IR (Neurlang binary) |
| `.elf` | Linux executable (AOT) |
| `.jsonl` | Training data (JSON Lines) |
| `.pt` | PyTorch checkpoint |
| `.onnx` | ONNX model |

## See Also

- [Quick Start](quickstart.md)
- [Commands Reference](commands.md)
- [Examples](examples.md)
