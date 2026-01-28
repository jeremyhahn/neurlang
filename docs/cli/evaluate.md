# evaluate.py

Evaluate a trained multi-head model on test data.

## Synopsis

```bash
python evaluate.py --checkpoint <file> --data <file> [OPTIONS]
```

## Description

Evaluates model performance on a test dataset and provides detailed metrics including:
- Overall accuracy for intent classification and operand count
- Per-intent accuracy breakdown
- Confusion matrix for error analysis

## Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--checkpoint` | string | *required* | Path to model checkpoint (.pt file) |
| `--data` | string | *required* | Test data JSONL file |
| `--batch-size` | int | `256` | Batch size for evaluation |
| `--light` | flag | false | Use lightweight model architecture |
| `--device` | string | `cuda`/`cpu` | Evaluation device (auto-detects CUDA) |
| `--output` | string | none | Save metrics to JSON file |

## Examples

### Basic Evaluation

```bash
# Evaluate on test set
python evaluate.py --checkpoint checkpoints/best_model.pt --data data/test.jsonl

# Save metrics to file
python evaluate.py --checkpoint checkpoints/best_model.pt --data data/test.jsonl --output results/metrics.json
```

### Different Configurations

```bash
# Evaluate lightweight model
python evaluate.py --checkpoint checkpoints/best_model.pt --data data/test.jsonl --light

# Use CPU for evaluation
python evaluate.py --checkpoint checkpoints/best_model.pt --data data/test.jsonl --device cpu

# Larger batch size for faster evaluation
python evaluate.py --checkpoint checkpoints/best_model.pt --data data/test.jsonl --batch-size 512
```

## Output

### Console Output

```
============================================================
EVALUATION RESULTS
============================================================

Overall Metrics (5000 samples):
  Intent Accuracy:  1.0000 (100.00%)
  Count Accuracy:   1.0000 (100.00%)
  Operand Accuracy: 0.0692 (6.92%)

Per-Intent Accuracy:
----------------------------------------
  ADD             ████████████████████ 100.00% (213/213)
  SUB             ████████████████████ 100.00% (154/154)
  MUL             ████████████████████ 100.00% (165/165)
  ...

Most Common Confusions:
----------------------------------------
  (none - 100% accuracy)
```

### JSON Output (--output)

```json
{
  "total": 5000,
  "intent_accuracy": 1.0,
  "count_accuracy": 1.0,
  "operand_accuracy": 0.0692,
  "per_intent": {
    "ADD": {"correct": 213, "total": 213, "accuracy": 1.0},
    "SUB": {"correct": 154, "total": 154, "accuracy": 1.0},
    ...
  },
  "confusion": {
    "0": {"0": 213},
    "1": {"1": 154},
    ...
  }
}
```

## Metrics Explained

### Intent Accuracy

Percentage of samples where the predicted intent matches the true intent.

**Target**: > 98%

### Count Accuracy

Percentage of samples where the predicted operand count matches the true count.

**Target**: > 99%

### Operand Accuracy

Percentage of samples where all predicted operand values match the true values. This is typically low because:
- Operand values are extracted from text, not predicted by the model
- The model focuses on intent and count prediction

**Note**: In the inference pipeline, operands are extracted from the input text using regex patterns, not predicted by the model.

### Per-Intent Accuracy

Breakdown of accuracy for each of the 54 intents. Useful for identifying:
- Underperforming intents
- Intents that need more training data
- Confusion between similar intents (e.g., ADD vs AND)

### Confusion Matrix

Shows which intents are confused with each other. Format:
```
confusion[true_intent][predicted_intent] = count
```

## Interpreting Results

### Perfect Results (100% accuracy)

```
Intent Accuracy: 1.0000 (100.00%)
```

This means the model correctly classifies all intents. Achieved with sufficient training data and epochs.

### Good Results (>98% accuracy)

```
Intent Accuracy: 0.9850 (98.50%)
```

Some confusion between similar intents. Check:
- Which intents are confused (confusion matrix)
- Whether training data has enough variation

### Poor Results (<95% accuracy)

```
Intent Accuracy: 0.9200 (92.00%)
```

Likely causes:
- Insufficient training data
- Too few epochs
- Model too small (try without `--light`)
- Overlapping prompt patterns

## Common Issues

### Low Intent Accuracy on Specific Intents

Check if:
1. Training data has enough examples for that intent
2. Prompt templates are distinct from similar intents
3. Intent keywords don't overlap (e.g., "and" in ADD vs AND)

### Out of Memory

```bash
# Reduce batch size
python evaluate.py --checkpoint checkpoints/best_model.pt --data data/test.jsonl --batch-size 64

# Use CPU
python evaluate.py --checkpoint checkpoints/best_model.pt --data data/test.jsonl --device cpu
```

### Wrong Model Architecture

If you get shape mismatch errors, ensure you use the same architecture:
```bash
# If trained with --light, evaluate with --light
python evaluate.py --checkpoint checkpoints/best_model.pt --data data/test.jsonl --light
```

## See Also

- [train.py](train.md) - Train the model
- [generate_data.py](generate_data.md) - Generate test data
- [export_onnx.py](export_onnx.md) - Export for production
