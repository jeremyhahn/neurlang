# train.py

Train the multi-head prediction model for intent classification and operand prediction.

## Synopsis

```bash
python train.py --data <file> [OPTIONS]
```

## Description

Trains a multi-head neural network that predicts:
1. **Intent ID** (0-53): Which operation to perform
2. **Operand Count** (0-4): How many operands the operation requires
3. **Operand Values**: The actual numeric values (extracted from text)

The model uses a CNN-based encoder with separate prediction heads for each output.

## Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--data` | string | *required* | Training data JSONL file |
| `--epochs` | int | `50` | Number of training epochs |
| `--batch-size` | int | `256` | Batch size for training |
| `--lr` | float | `0.001` | Learning rate |
| `--val-split` | float | `0.1` | Validation split ratio (0.0-1.0) |
| `--save-dir` | string | `checkpoints` | Directory to save checkpoints |
| `--light` | flag | false | Use lightweight model (~50K params) |
| `--device` | string | `cuda`/`cpu` | Training device (auto-detects CUDA) |
| `--max-samples` | int | none | Limit training samples (for testing) |

## Examples

### Basic Training

```bash
# Train with default settings
python train.py --data data/train.jsonl

# Train for fewer epochs (faster)
python train.py --data data/train.jsonl --epochs 10

# Train with larger batch size (if GPU memory allows)
python train.py --data data/train.jsonl --batch-size 512
```

### Custom Configuration

```bash
# Full configuration
python train.py \
    --data data/train.jsonl \
    --epochs 50 \
    --batch-size 256 \
    --lr 0.001 \
    --val-split 0.1 \
    --save-dir checkpoints/

# Train lightweight model
python train.py --data data/train.jsonl --light --epochs 20

# Limit samples for quick testing
python train.py --data data/train.jsonl --max-samples 1000 --epochs 5
```

### GPU/CPU Selection

```bash
# Force CPU training
python train.py --data data/train.jsonl --device cpu

# Use specific GPU
CUDA_VISIBLE_DEVICES=0 python train.py --data data/train.jsonl
```

## Output

The training script saves:

### Checkpoints Directory Structure

```
checkpoints/
├── best_model.pt    # Best model by validation intent accuracy
└── final_model.pt   # Final model after all epochs
```

### Checkpoint Contents

```python
{
    'epoch': int,                    # Training epoch
    'model_state_dict': dict,        # Model weights
    'optimizer_state_dict': dict,    # Optimizer state
    'val_loss': float,               # Best validation loss
    'intent_acc': float,             # Best intent accuracy
}
```

## Training Metrics

The script reports these metrics each epoch:

| Metric | Description | Target |
|--------|-------------|--------|
| Train Loss | Combined training loss | Decreasing |
| Train Intent | Intent classification loss | < 0.01 |
| Train Count | Operand count loss | < 0.01 |
| Val Intent Acc | Validation intent accuracy | > 0.98 |
| Val Count Acc | Validation operand count accuracy | > 0.99 |

## Example Output

```
Using device: cuda
Loaded 50000 samples from data/train.jsonl
Train samples: 45000, Val samples: 5000
Model parameters: 475,075

Epoch 1/10
Train - Loss: 2.2222, Intent: 0.2686, Count: 0.0406
Val - Loss: 1.8542, Intent Acc: 1.0000, Count Acc: 1.0000
Saved best model (intent_acc: 1.0000)

Epoch 2/10
Train - Loss: 1.7536, Intent: 0.0054, Count: 0.0013
Val - Loss: 1.5935, Intent Acc: 0.9998, Count Acc: 1.0000
...

Training complete. Best intent accuracy: 1.0000
```

## Model Architecture

### Full Model (~475K parameters)

```
MultiHeadPredictor:
├── Embedding (261 × 64)
├── Encoder (CNN)
│   ├── Conv1d(64, 64, k=3)
│   ├── Conv1d(64, 128, k=3)
│   ├── Conv1d(128, 256, k=3)
│   └── AdaptiveMaxPool1d(1)
├── Intent Head (256 → 128 → 54)
├── Count Head (256 → 64 → 5)
├── Operand Heads (256 → 128 → 256) × 4
└── Sign Heads (256 → 2) × 4
```

### Lightweight Model (~50K parameters)

```
LightMultiHeadPredictor:
├── Embedding (261 × 32)
├── Encoder (CNN)
│   ├── Conv1d(32, 64, k=3)
│   ├── Conv1d(64, 64, k=3)
│   └── AdaptiveMaxPool1d(1)
├── Intent Head (64 → 54)
├── Count Head (64 → 5)
└── Operand Heads (64 → 256) × 4
```

## Tips

### Memory Issues

If you run out of GPU memory:
```bash
# Reduce batch size
python train.py --data data/train.jsonl --batch-size 64

# Use lightweight model
python train.py --data data/train.jsonl --light

# Use CPU (slower but unlimited memory)
python train.py --data data/train.jsonl --device cpu
```

### Faster Training

```bash
# Use larger batch size if GPU allows
python train.py --data data/train.jsonl --batch-size 512

# Reduce epochs (usually converges fast)
python train.py --data data/train.jsonl --epochs 10
```

### Debugging

```bash
# Quick test with limited data
python train.py --data data/train.jsonl --max-samples 100 --epochs 2
```

## Requirements

- Python 3.8+
- PyTorch 2.0+
- CUDA (optional, for GPU training)

## See Also

- [generate_data.py](generate_data.md) - Generate training data
- [evaluate.py](evaluate.md) - Evaluate trained model
- [export_onnx.py](export_onnx.md) - Export to ONNX format
