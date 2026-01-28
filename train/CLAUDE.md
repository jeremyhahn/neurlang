# Training Data Generation

This directory contains the training data generation pipeline and model for Neurlang.

## Quick Start

```bash
# Download pre-trained model (recommended)
make download-model

# Or train from scratch (requires GPU)
make generate-data
make train
```

## Pre-trained Model

The pre-trained model is available via GitHub Releases:

```bash
# Download and verify
make download-model
make verify-model

# Test inference
python train/parallel/test_inference.py --pytorch train/models/best_model.pt
```

**Model Stats:**
- Parameters: 5,754,637 (5.75M)
- Accuracy: 99.86% opcode prediction
- Training: 70,150 samples, 20 epochs (~10 minutes on RTX 6000 Pro)
- Size: 67MB (best_model.pt), 23MB (model.onnx)

## Directory Structure

```
train/
├── models/              # Trained models (gitignored)
│   ├── best_model.pt    # Best checkpoint
│   └── model.config.json
├── generators/          # Training data generators
│   ├── extension_patterns.py
│   ├── http_patterns.py
│   └── diverse_patterns.py
├── parallel/            # Model architecture & training
│   ├── model.py         # ParallelInstructionModel
│   ├── train.py         # Training script
│   └── test_inference.py
└── generate_training_data.py  # Unified data generator
```

## Training Data Generation

The `generate_training_data.py` script produces training data from verified sources.

### Basic Usage

```bash
# Generate with default settings (via Makefile)
make generate-data

# Or directly
python train/generate_training_data.py train/training_data.jsonl
```

### Training Data Sources

| Source | Description | Sample Count |
|--------|-------------|--------------|
| `lib/*.nl` | Verified stdlib functions | ~17,000 |
| `examples/*.nl` | Hand-written examples | ~13,000 |
| Extension patterns | Crypto, JSON, HTTP, TLS | ~30,000 |
| HTTP patterns | Protocol details, headers | ~10,000 |

**Total: ~70,000 samples**

## Training (GPU Required)

Training requires a GPU with PyTorch CUDA support.

### Remote Training (Recommended)

See `PROFILES.md` for remote GPU instance setup.

```bash
# Provision and train on remote GPU
./scripts/provision-gpu-instance.sh root@<GPU-IP>
```

### Local Training

```bash
# Generate data first
make generate-data

# Train (requires CUDA)
make train
```

### Training Configuration

Default settings in Makefile:
- Epochs: 100 (early stopping with patience=5)
- Batch size: 512
- Learning rate: 3e-4
- Mixed precision: enabled

## Conventions

**NEVER** train on CPU - it will take forever.

**ALWAYS** use GPU instances for training. See `PROFILES.md` for setup scripts.

## Testing

```bash
# Verify examples pass before training
timeout 60s ./target/release/nl test -p examples

# Test model inference
python train/parallel/test_inference.py --pytorch train/models/best_model.pt
```
