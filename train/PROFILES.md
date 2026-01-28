# GPU Profiles for Neurlang Training

This document contains GPU specifications and pricing for training the Neurlang model.
Prices are spot prices from cloud GPU providers (as of January 2025).

## Quick Recommendation

For the **25M parameter Neurlang model**, we recommend:

| Budget | GPU | Why |
|--------|-----|-----|
| **Best Value** | L40S | $0.59/hr, 48GB VRAM, plenty for batch 1024+ |
| **Fastest** | H100 SXM | $1.89/hr, 80GB VRAM, fastest training |
| **Budget** | RTX 4090 | $0.34/hr, 24GB VRAM, good for batch 256 |

The model only needs ~10-18GB VRAM. Don't overpay for H200/B200 unless scaling to larger models.

---

## GPU Pricing Table (Spot Prices)

| GPU | VRAM | Spot Price | On-Demand | Architecture | Notes |
|-----|------|------------|-----------|--------------|-------|
| **H200 SXM** | 141GB | $2.49/hr | $4.49/hr | Hopper | Overkill for 25M model |
| **H100 SXM** | 80GB | $1.89/hr | $3.39/hr | Hopper | Fast, good for larger models |
| **H100 NVL** | 94GB | $2.19/hr | $3.89/hr | Hopper | Dual-GPU config |
| **H100 PCIe** | 80GB | $1.79/hr | $2.99/hr | Hopper | Slightly slower than SXM |
| **A100 SXM** | 80GB | $1.19/hr | $1.99/hr | Ampere | Great price/performance |
| **A100 PCIe** | 80GB | $1.09/hr | $1.79/hr | Ampere | Slightly slower than SXM |
| **A100 PCIe** | 40GB | $0.79/hr | $1.29/hr | Ampere | Budget A100 option |
| **L40S** | 48GB | $0.59/hr | $0.99/hr | Ada Lovelace | **Best value for 25M model** |
| **A6000** | 48GB | $0.39/hr | $0.69/hr | Ampere | Good budget option |
| **RTX 4090** | 24GB | $0.34/hr | $0.59/hr | Ada Lovelace | Consumer GPU, limited VRAM |
| **L4** | 24GB | $0.19/hr | $0.34/hr | Ada Lovelace | Inference-optimized |
| **RTX 4000 Ada** | 20GB | $0.19/hr | $0.34/hr | Ada Lovelace | Entry-level |
| **A4000** | 16GB | $0.14/hr | $0.24/hr | Ampere | Very limited VRAM |
| **RTX 3090** | 24GB | $0.19/hr | $0.34/hr | Ampere | Older consumer GPU |

---

## Memory Requirements (25M Model)

The Neurlang model with default config (6 layers, 384 embedding) uses approximately:

| Batch Size | Estimated VRAM (BF16) | Recommended GPU |
|------------|----------------------|-----------------|
| 32 | ~1.5GB | Any GPU |
| 64 | ~2GB | Any GPU |
| 128 | ~3GB | Any GPU |
| 256 | ~6GB | RTX 4090, A4000+ |
| 512 | ~10GB | L40S, A6000, A100 |
| 1024 | ~18GB | L40S, A100 |
| 2048 | ~35GB | L40S, A100 80GB |
| 4096 | ~68GB | A100 80GB, H100 |

---

## GPU Profiles in train/model.py

Use `--gpu-profile` to automatically configure batch size and optimizations:

```bash
# List all available profiles
python train/model.py --list-profiles

# Use a specific profile
python train/model.py --data training_data.jsonl --gpu-profile l40s
```

### Available Profiles

| Profile | GPU | Batch | BF16 | Compile | Description |
|---------|-----|-------|------|---------|-------------|
| `cpu` | CPU | 8 | No | No | Testing only |
| `rtx4090` | RTX 4090 | 256 | Yes | Yes | Consumer GPU |
| `l40s` | L40S | 256 (×4) | Yes | No* | **Recommended** |
| `a100-40` | A100 40GB | 1024 | Yes | Yes | Good value |
| `a100-80` | A100 80GB | 2048 | Yes | Yes | Fast training |
| `h100` | H100 | 2048 | Yes | Yes | Maximum speed |
| `h200` | H200 | 4096 | Yes | Yes | For larger models |
| `b200` | B200 | 4096 | Yes | No* | Blackwell arch |
| `b300` | B300 | 4096 | Yes | No* | Blackwell arch |

*torch.compile disabled: L40S CUDA graph memory exceeds 48GB with batch 1024; Blackwell has cuDNN compatibility issues

---

## Training Time Estimates

For 50,000 training examples, 10,000 iterations:

| GPU | Batch Size | Est. Time | Est. Cost |
|-----|------------|-----------|-----------|
| RTX 4090 | 256 | ~45 min | ~$0.25 |
| L40S | 1024 | ~20 min | ~$0.20 |
| A100 80GB | 2048 | ~12 min | ~$0.25 |
| H100 | 2048 | ~8 min | ~$0.25 |

All GPUs end up costing roughly the same for this small model. **L40S is the sweet spot.**

---

## Usage Examples

### Generate Training Data

```bash
# Generate 50,000 examples (default)
cargo run --release --bin nl-datagen -- --output training_data.jsonl

# Generate more examples with higher complexity
cargo run --release --bin nl-datagen -- \
    --num-examples 100000 \
    --curriculum-level 5 \
    --output training_data.jsonl
```

### Train Model

```bash
# Recommended: Use L40S profile
python train/model.py --data training_data.jsonl --gpu-profile l40s

# Budget: Use RTX 4090
python train/model.py --data training_data.jsonl --gpu-profile rtx4090

# Maximum speed: Use H100
python train/model.py --data training_data.jsonl --gpu-profile h100

# Override batch size
python train/model.py --data training_data.jsonl --gpu-profile l40s --batch_size 512
```

### Export to ONNX

```bash
python scripts/export_onnx.py model.pt model.onnx
```

---

## Cost Comparison: Complete Training Run

For training the 25M Neurlang model end-to-end:

| GPU | Spot Price | Training Time | **Total Cost** |
|-----|------------|---------------|----------------|
| L4 | $0.19/hr | ~90 min | ~$0.29 |
| RTX 4090 | $0.34/hr | ~45 min | ~$0.26 |
| A6000 | $0.39/hr | ~35 min | ~$0.23 |
| **L40S** | $0.59/hr | ~20 min | **~$0.20** |
| A100 80GB | $1.19/hr | ~12 min | ~$0.24 |
| H100 | $1.89/hr | ~8 min | ~$0.25 |

**Conclusion:** All options cost roughly $0.20-$0.30 for complete training.
Choose based on your time preference:
- **Fastest:** H100 (~8 min)
- **Best value:** L40S (~20 min)
- **Budget:** L4/RTX 4090 (~45-90 min)
