# Training Documentation

Data generation and model training for Neurlang AI code synthesis.

## Overview

Neurlang supports a flexible ML backend architecture with multiple training and inference options:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Training Pipeline Architecture                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│   Data Generation              Training Backends           Inference Engines │
│   ───────────────             ─────────────────           ───────────────── │
│                                                                               │
│   ┌─────────────┐     ┌───────────────────────────┐     ┌─────────────────┐ │
│   │   Datagen   │────▶│  Native Rust (burn)       │────▶│  ort (ONNX RT)  │ │
│   │  Generator  │     │  - Zero dependencies      │     │  - GPU support  │ │
│   │             │     │  - Single binary          │     │  - Fastest      │ │
│   └─────────────┘     │  - wgpu/CUDA backends     │     └─────────────────┘ │
│         │             └───────────────────────────┘              │           │
│         │                       OR                               │           │
│         │             ┌───────────────────────────┐     ┌───────┴─────────┐ │
│         └────────────▶│  Docker/Podman (PyTorch)  │────▶│  tract (pure    │ │
│                       │  - Containerized          │     │  Rust, smallest)│ │
│                       │  - GPU-optimized          │     └─────────────────┘ │
│                       │  - Auto-builds image      │              │           │
│                       └───────────────────────────┘     ┌───────┴─────────┐ │
│                                                         │  candle (HF)    │ │
│                                                         └─────────────────┘ │
│                                                                  │           │
│                                                         ┌───────┴─────────┐ │
│                                                         │  burn (native)  │ │
│                                                         └─────────────────┘ │
│                                                                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Quick Start

```bash
# 1. Build stdlib from Rust sources
nl stdlib --build

# 2. Run tests to verify generated code
nl test -p lib

# 3. Generate training data (from stdlib + examples)
nl datagen -o training_data.jsonl --include-examples --stdlib-dir lib

# 4. Train the model (auto-detects best backend)
nl train --data training_data.jsonl --epochs 20

# 5. Test the model
nl accuracy --benchmark

# 6. Use the model
nl prompt "add 5 and 3"
```

## Training Data Sources

Neurlang training data comes from multiple verified sources:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Training Data Pipeline                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│   stdlib/src/*.rs          examples/*.nl           datagen (synthetic)       │
│   (Rust source)            (Hand-written)          (Generated patterns)      │
│         │                        │                        │                   │
│         ▼                        │                        │                   │
│   nl stdlib --build              │                        │                   │
│         │                        │                        │                   │
│         ▼                        │                        │                   │
│   lib/*.nl                       │                        │                   │
│   (Generated, verified)          │                        │                   │
│         │                        │                        │                   │
│         └────────────────────────┼────────────────────────┘                   │
│                                  │                                            │
│                                  ▼                                            │
│                    nl datagen --include-examples                              │
│                                  │                                            │
│                                  ▼                                            │
│                       training_data.jsonl                                     │
│                                  │                                            │
│                                  ▼                                            │
│                       nl train --data ...                                     │
│                                                                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Source Priority and Deduplication

| Source | Quality | Volume | Use Case |
|--------|---------|--------|----------|
| **Stdlib (lib/)** | Highest | ~113 functions | Core patterns, verified implementations |
| **Examples** | High | ~50 files | Composition patterns, application code |
| **Synthetic** | Medium | 10K-100K | Fill gaps, edge cases |

**Deduplication Logic:**
The training generator processes `lib/` first as the source of truth. Functions in `examples/` that duplicate stdlib functions are skipped. This ensures:
1. Only verified Rust implementations are trained on
2. Examples focus on composition patterns, not basic algorithms
3. No conflicting implementations in training data

```python
# In generate_training_data.py
lib_functions = set()  # Track seen function names

# Process lib/ first (source of truth)
for file in lib_files:
    spec = parse_nl_file(file)
    if spec and spec.get('prompts'):
        lib_functions.add(spec['name'])
        # Add to training data...

# Process examples/ with deduplication
for file in example_files:
    spec = parse_nl_file(file)
    if spec['name'] not in lib_functions:
        # Only include if not in stdlib
```

## Training Backends

### Native Rust Training (burn)

The default training backend uses the [burn](https://burn.dev/) framework for pure Rust training. No Python or external dependencies required.

**Advantages:**
- Zero external dependencies
- Single static binary
- Cross-platform (wgpu backend)
- CUDA support via burn-cuda feature

**Usage:**
```bash
# Train with native backend (default)
nl train --data training_data.jsonl --backend pytorch  # Uses local Python
nl train --data training_data.jsonl --profile cpu      # Force CPU training

# Output: model.onnx (exported from PyTorch)
```

**Model Architecture (burn):**
```rust
MultiHeadModel:
├── Embedding (261 × 64)
├── Encoder (CNN)
│   ├── Conv1d(64, 64, k=3) + BatchNorm + ReLU
│   ├── Conv1d(64, 128, k=3) + BatchNorm + ReLU
│   ├── Conv1d(128, 256, k=3) + ReLU
│   └── AdaptiveMaxPool1d(1)
├── Intent Head (256 → 128 → 54 intents)
├── Count Head (256 → 64 → 5 counts)
├── Operand Heads × 4 (256 → 128 → 256 bins)
└── Sign Heads × 4 (256 → 2 classes)
```

### Docker/Podman Training (PyTorch)

For maximum compatibility with existing ML workflows, use the containerized backend:

**Advantages:**
- Familiar PyTorch ecosystem
- NVIDIA GPU acceleration
- Reproducible environment
- Auto-builds container on first run

**Usage:**
```bash
# Ensure Docker or Podman is installed
which docker || which podman

# Train with Docker backend (auto-builds image)
nl train --data training_data.jsonl --backend docker
# First run: builds neurlang-train image (~2 min)
# Output: model.onnx
```

**Container Setup:**
```dockerfile
# Auto-generated at docker/Dockerfile.train
FROM nvidia/cuda:12.1-runtime-ubuntu22.04

RUN pip3 install torch==2.1.0 onnx==1.15.0 onnxruntime==1.16.0
COPY train/multihead /app/train
ENTRYPOINT ["python3", "-m", "train"]
```

### Remote Training

Train on a remote GPU server via SSH:

```bash
# Train on remote H100 server
nl train --data training_data.jsonl \
    --remote user@gpu-server \
    --profile h100 \
    --epochs 50

# The CLI will:
# 1. Sync training data to remote
# 2. Run training with GPU profile
# 3. Download trained model
```

## Data Format

### JSONL Structure

```json
{
  "text": "add 5 and 3",
  "intent": 0,
  "operand_count": 2,
  "operands": [5, 3, 0, 0],
  "program": "mov r0, 5\nmov r1, 3\nadd r0, r0, r1\nhalt"
}
```

### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `text` | string | Natural language input |
| `intent` | int | Intent class ID (0-53) |
| `operand_count` | int | Number of operands (0-4) |
| `operands` | int[4] | Operand values (padded to 4) |
| `program` | string | Optional: corresponding assembly |

## Data Generation

### Stdlib-Based Training Data

The primary source of high-quality training data is the stdlib, which is compiled from verified Rust:

```bash
# Build stdlib from Rust sources
nl stdlib --build --verbose

# Verify all tests pass
nl test -p lib

# Generate training data including stdlib
nl datagen -o training_data.jsonl --include-examples --stdlib-dir lib
```

**Why stdlib is preferred:**

1. **Verified correctness** - Rust implementations are tested
2. **Real patterns** - Actual loop, conditional, arithmetic patterns
3. **Consistent quality** - Generated by the same compiler
4. **Auto-generated tests** - @test annotations from Rust execution

### Training Data from Stdlib

Each stdlib function generates multiple training examples by expanding `@prompt` annotations with parameter values:

**Source .nl file (lib/math/factorial.nl):**
```asm
; @prompt: compute factorial of {n}
; @prompt: {n}!
; @prompt: calculate {n} factorial
; @param: n=r0 "The number to compute factorial of"
```

**Generated training examples:**
```json
{
  "prompt": "compute factorial of 5",
  "context": "factorial",
  "assembly": "mov r1, 1\nmov r2, r0\n...",
  "test_cases": [
    {"inputs": {"r0": 5}, "outputs": {"r0": 120}}
  ],
  "source": "lib/math/factorial.nl"
}
{
  "prompt": "10!",
  "context": "factorial",
  "assembly": "mov r1, 1\nmov r2, r0\n...",
  "test_cases": [
    {"inputs": {"r0": 10}, "outputs": {"r0": 3628800}}
  ],
  "source": "lib/math/factorial.nl"
}
```

### Curriculum Levels

```
Level 1 (Basic):
├── Arithmetic: add, sub, mul, div
├── Move: register moves, immediates
└── Simple operations with constants

Level 2 (Control Flow):
├── Conditionals: if-else patterns
├── Loops: for, while, do-while
└── Branch conditions

Level 3 (Memory):
├── Load/Store operations
├── Array access and traversal
└── Pointer arithmetic

Level 4 (Functions):
├── Call/Return patterns
├── Stack management
└── Recursion

Level 5 (Advanced):
├── Algorithms: sort, search, factorial
├── Concurrency: spawn, channels
└── Security: capabilities, taint
```

### Generator Usage

```bash
# Generate balanced dataset
nl datagen -o training_data.jsonl -n 100000 --level 5

# Basic operations only
nl datagen -o basic.jsonl -n 10000 --level 1

# Include real examples from examples/ directory
nl datagen -o data.jsonl --include-examples

# Reproducible generation
nl datagen -o data.jsonl --seed 12345
```

## GPU Profiles

Pre-configured profiles for common GPU hardware:

| Profile | GPU | Batch Size | Memory | Notes |
|---------|-----|------------|--------|-------|
| `h100` | NVIDIA H100 | 256 | 80 GB | Production, fastest |
| `h200` | NVIDIA H200 | 256 | 141 GB | Latest datacenter |
| `b300` | NVIDIA Blackwell | 512 | 192 GB | Next-gen |
| `l40s` | NVIDIA L40S | 256 | 48 GB | Cloud inference |
| `a100` | NVIDIA A100 | 256 | 40/80 GB | Common datacenter |
| `generic` | Any CUDA | 128 | Auto | Unknown hardware |
| `cpu` | None | 32 | System | No GPU fallback |

```bash
# List available profiles
nl train --list-profiles

# Use specific profile
nl train --data data.jsonl --profile a100
```

## Inference Engines

After training, the model can be served by multiple inference backends:

| Engine | Format | Binary Size | GPU | Latency | Best For |
|--------|--------|-------------|-----|---------|----------|
| **ort** | .onnx | +15 MB | CUDA, TensorRT, CoreML | ~0.3μs | Production, GPU |
| **tract** | .onnx | +2 MB | None | ~0.5μs | Minimal binary, embedded |
| **candle** | .onnx, .safetensors | +10 MB | CUDA | ~0.8μs | HuggingFace ecosystem |
| **burn** | .mpk | +45 MB | wgpu, CUDA | ~1-2μs | Native training workflow |

### Engine Selection

```bash
# Auto-detect best engine
nl generate "add 5 and 3"  # Uses ort if available

# Explicit engine selection
nl generate "add 5 and 3" --engine tract  # Pure Rust, smallest
nl generate "add 5 and 3" --engine ort    # ONNX Runtime, GPU
nl generate "add 5 and 3" --engine candle # HuggingFace
```

### Model Format Compatibility

| Format | ort | tract | candle | burn |
|--------|-----|-------|--------|------|
| .onnx | ✓ | ✓ | ✓ | ✗ |
| .mpk | ✗ | ✗ | ✗ | ✓ |
| .safetensors | ✗ | ✗ | ✓ | ✓ |

## Training Configuration

### Default Settings

```rust
TrainConfig {
    epochs: 20,
    batch_size: 256,
    learning_rate: 0.001,
    weight_decay: 0.01,
    train_ratio: 0.9,      // 90% train, 10% validation
    patience: 5,           // Early stopping patience
    seed: 42,
}
```

### Loss Weights

Multi-head training uses weighted cross-entropy loss:

```rust
LossWeights {
    intent: 1.0,    // Primary classification
    count: 0.5,     // Operand count
    operand: 0.5,   // Operand values
    sign: 0.3,      // Operand signs
}
```

### Training Output

```
Training Configuration:
  Data: training_data.jsonl (100000 examples)
  Train samples: 90000
  Val samples: 10000
  Epochs: 20
  Batch size: 256
  Learning rate: 0.001

Starting training...
Epoch 1/20:
  Train loss: 2.345  Intent acc: 45.2%
  Val loss: 2.123    Intent acc: 52.1%

Epoch 20/20:
  Train loss: 0.234  Intent acc: 96.8%
  Val loss: 0.289    Intent acc: 94.2%

Training complete!
Model saved to: model.onnx
Best validation accuracy: 94.2%
Total time: 12 minutes
```

## Cross-Validation

For robust model evaluation:

```bash
# 5-fold cross-validation
nl train --data data.jsonl --cross-validate --folds 5
```

Output:
```
Cross-Validation Results (5 folds):
  Fold 1: 93.2% accuracy
  Fold 2: 94.1% accuracy
  Fold 3: 93.8% accuracy
  Fold 4: 94.5% accuracy
  Fold 5: 93.9% accuracy

Mean accuracy: 93.9% ± 0.4%
```

## Model Export

### PyTorch to ONNX

```bash
# Export model for inference
nl export-onnx -i model.pt -o model.onnx

# Verify export
nl test-accuracy --model model.onnx --benchmark
```

### ONNX Optimization

The exported model is automatically optimized:
- Graph optimization
- Constant folding
- Operator fusion
- Float16 quantization (optional)

## End-to-End Pipeline

```
User Prompt                              Execution
     │                                       │
     ▼                                       ▼
┌─────────────┐                       ┌─────────────┐
│  Tokenize   │                       │   Execute   │
│  (bytes)    │                       │  (<5μs JIT) │
└──────┬──────┘                       └──────▲──────┘
       │                                     │
       ▼                                     │
┌─────────────┐     Binary IR         ┌──────┴──────┐
│   Model     │────────────────────▶  │   Compile   │
│  (~0.3μs)   │                       │ (copy-patch)│
└─────────────┘                       └─────────────┘

Total latency: ~0.3μs inference + ~3μs compile = ~3.3μs end-to-end
```

## Troubleshooting

### Common Issues

**Model file not found:**
```
Error: Model file not found: model.onnx
```
Solution: Train a model first with `nl train` or specify path with `--model`.

**No inference engine available:**
```
Error: No inference engine available for this model format
```
Solution: Rebuild with appropriate features:
```bash
cargo build --release --features "ort-backend"  # For .onnx
cargo build --release --features "train"        # For .mpk
```

**Docker not found:**
```
Error: Docker/Podman not found
```
Solution: Install Docker or Podman, or use local training.

**CUDA out of memory:**
```
Error: CUDA out of memory
```
Solution: Use a smaller GPU profile or reduce batch size:
```bash
nl train --data data.jsonl --profile cpu  # CPU fallback
```

## Build Features

Configure inference engines at compile time:

```bash
# Minimal binary (tract only, ~7 MB)
cargo build --release --features "tract"

# Standard (ort, ~20 MB)
cargo build --release --features "ort-backend"

# Full (all engines + training, ~80 MB)
cargo build --release --features "ort-backend,tract,candle,train"
```
