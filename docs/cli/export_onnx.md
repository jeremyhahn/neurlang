# export_onnx.py

Export a trained multi-head model to ONNX format for production deployment.

## Synopsis

```bash
python export_onnx.py --checkpoint <file> [OPTIONS]
```

## Description

Exports a trained PyTorch model to ONNX (Open Neural Network Exchange) format, which can be loaded by the Rust inference pipeline using the `ort` crate.

The exported model includes:
- Intent classification head (54 classes)
- Operand count head (5 classes: 0-4)
- Operand value heads (4 heads, 256 bins each)
- Sign heads (4 heads, 2 classes each)

## Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--checkpoint` | string | *required* | Path to model checkpoint (.pt file) |
| `--output` | string | `multihead_model.onnx` | Output ONNX file path |
| `--light` | flag | false | Export lightweight model |
| `--quantize` | flag | false | Create quantized (int8) version |
| `--verify` | flag | false | Verify ONNX model after export |
| `--opset` | int | `14` | ONNX opset version |

## Examples

### Basic Export

```bash
# Export with default settings
python export_onnx.py --checkpoint checkpoints/best_model.pt

# Export to specific location
python export_onnx.py --checkpoint checkpoints/best_model.pt --output models/multihead_model.onnx
```

### With Verification

```bash
# Export and verify
python export_onnx.py --checkpoint checkpoints/best_model.pt --output models/model.onnx --verify
```

### Quantized Export

```bash
# Export with int8 quantization (smaller, faster)
python export_onnx.py --checkpoint checkpoints/best_model.pt --output models/model.onnx --quantize
```

### Production Export

```bash
# Full production export with verification
python export_onnx.py \
    --checkpoint checkpoints/best_model.pt \
    --output models/multihead_model.onnx \
    --verify \
    --opset 14

# Copy to project root
cp models/multihead_model.onnx ../../models/
```

## Output

### ONNX Model Structure

```
Model Inputs:
  - input_ids: int64[batch_size, 128]  # Tokenized input

Model Outputs:
  - intent_logits: float32[batch_size, 54]       # Intent classification
  - count_logits: float32[batch_size, 5]         # Operand count (0-4)
  - operand_0_logits: float32[batch_size, 256]   # Operand 0 value
  - operand_1_logits: float32[batch_size, 256]   # Operand 1 value
  - operand_2_logits: float32[batch_size, 256]   # Operand 2 value
  - operand_3_logits: float32[batch_size, 256]   # Operand 3 value
  - sign_0_logits: float32[batch_size, 2]        # Operand 0 sign
  - sign_1_logits: float32[batch_size, 2]        # Operand 1 sign
  - sign_2_logits: float32[batch_size, 2]        # Operand 2 sign
  - sign_3_logits: float32[batch_size, 2]        # Operand 3 sign
```

### Verification Output

```
Exporting model to models/multihead_model.onnx...
Exported to models/multihead_model.onnx
ONNX model verified successfully

Model inputs: ['input_ids']
Model outputs: ['intent_logits', 'count_logits', 'operand_0_logits', ...]

ONNX inference test:
  Input shape: (1, 128)
  Number of outputs: 10
  Output 0 shape: (1, 54)
  Output 1 shape: (1, 5)
  ...

  Predicted intent: 0
  Confidence: 0.9823
```

### File Sizes

| Model | Size |
|-------|------|
| Full model | ~1.9 MB |
| Lightweight model | ~200 KB |
| Quantized (int8) | ~500 KB |

## Using with Rust

### Building with ONNX Support

```bash
# Build with ONNX feature
cargo build --release --features onnx
```

### Loading the Model

```rust
use neurlang::inference::MultiHeadInference;
use std::path::Path;

let inference = MultiHeadInference::load(
    Path::new("models/multihead_model.onnx")
)?;

let prediction = inference.predict("add 5 and 3")?;
println!("Intent: {}", prediction.intent_id);
```

## ONNX Opset Versions

| Version | Compatibility |
|---------|---------------|
| 14 | Recommended, good compatibility |
| 15-18 | Newer features, some runtimes may not support |
| 11-13 | Older runtimes, limited features |

## Quantization

The `--quantize` option creates an additional int8 quantized model:

```bash
python export_onnx.py --checkpoint checkpoints/best_model.pt --output model.onnx --quantize
# Creates: model.onnx and model_int8.onnx
```

Benefits:
- Smaller file size (~4x reduction)
- Faster inference on CPU
- Lower memory usage

Trade-offs:
- Slight accuracy reduction (usually <1%)
- Not supported by all runtimes

## Troubleshooting

### Export Fails with "No ONNX function found"

Some PyTorch operations aren't directly supported. The script uses the legacy TorchScript-based exporter which has better compatibility.

### Verification Fails

If verification fails:
```bash
# Try different opset version
python export_onnx.py --checkpoint checkpoints/best_model.pt --opset 15

# Skip verification and test manually
python export_onnx.py --checkpoint checkpoints/best_model.pt
```

### Size Too Large

Use the lightweight model:
```bash
python export_onnx.py --checkpoint checkpoints/best_model.pt --light
```

Or quantize:
```bash
python export_onnx.py --checkpoint checkpoints/best_model.pt --quantize
```

## Requirements

- Python 3.8+
- PyTorch 2.0+
- onnx
- onnxruntime (for verification)
- onnxscript (for PyTorch 2.x export)

## See Also

- [train.py](train.md) - Train the model
- [evaluate.py](evaluate.md) - Evaluate before export
- [test_pipeline.md](test_pipeline.md) - Test with Rust pipeline
