# Training Cost Breakdown

This document provides a detailed breakdown of Neurlang's training costs and infrastructure options.

## Actual Training Results

The Neurlang model was trained on **70,150 samples** achieving **99.86% accuracy** in approximately **10 minutes** on an RTX 6000 Pro GPU.

### Model Statistics

| Metric | Value |
|--------|-------|
| Parameters | 5,754,637 (5.75M) |
| Training Samples | 70,150 |
| Best Accuracy | 99.86% |
| Training Time | ~10 minutes |
| GPU Used | RTX 6000 Pro |
| Training Cost | ~$0.08 |

## GPU Pricing (Verda)

Current spot prices as of January 29, 2026:

| GPU | Hourly Rate |
|-----|-------------|
| B300 | $1.73/hr |
| B200 | $1.38/hr |
| H200 | $1.04/hr |
| H100 | $0.80/hr |
| A100-80 | $0.45/hr |
| A100-40 | $0.25/hr |
| L40S | $0.32/hr |
| RTX 6000 Pro | $0.48/hr |
| RTX 6000 Ada | $0.28/hr |
| RTX A6000 | $0.17/hr |
| Tesla V100 | $0.04/hr |

## Training Cost Estimates

Based on actual training time (~10 minutes) on RTX 6000 Pro:

| GPU | Estimated Time | Cost |
|-----|----------------|------|
| B300 | ~5 min | ~$0.14 |
| B200 | ~6 min | ~$0.14 |
| H200 | ~7 min | ~$0.12 |
| H100 | ~8 min | ~$0.11 |
| A100-80 | ~10 min | ~$0.08 |
| **RTX 6000 Pro** | **~10 min** | **~$0.08** |
| A100-40 | ~12 min | ~$0.05 |
| L40S | ~12 min | ~$0.06 |
| RTX 6000 Ada | ~12 min | ~$0.06 |
| RTX A6000 | ~15 min | ~$0.04 |
| Tesla V100 | ~30 min | ~$0.02 |

**Note**: Training time scales inversely with GPU memory bandwidth and tensor core performance. The estimates above are approximations based on relative GPU performance.

## Cost Breakdown by Component

### Training Data Sources

| Source | Samples | Purpose |
|--------|---------|---------|
| lib/*.nl | ~17,000 | Verified stdlib functions |
| examples/*.nl | ~13,000 | Hand-written examples |
| Extension patterns | ~30,000 | Crypto, JSON, HTTP compositions |
| HTTP patterns | ~10,000 | Protocol details, headers |
| **Total** | **70,150** | |

### What's NOT Trained (RAG-Resolved)

These extensions are resolved via RAG at **zero training cost**:

| Category | Extensions |
|----------|------------|
| JSON | parse, stringify, get, set, array ops |
| HTTP | get, post, put, delete, headers |
| Crypto | sha256, hmac, aes, sign, verify |
| File I/O | read, write, exists, mkdir |
| SQLite | open, query, execute, prepare |
| Regex | match, find_all, replace, split |
| DateTime | now, parse, format, add, diff |
| UUID | v4, v7, parse |
| Base64/URL | encode, decode |
| **Any user extension** | anything |

## Training Configuration

Default settings used:

```bash
cd train && PYTHONPATH=. python parallel/train.py \
    --data training_data.jsonl \
    --output models/model.pt \
    --epochs 100 \
    --batch-size 512 \
    --mixed-precision \
    --device cuda
```

- **Epochs**: 100 (early stopping with patience=5)
- **Batch size**: 512
- **Learning rate**: 3e-4
- **Mixed precision**: Enabled (BF16 where available)
- **Early stopping**: Best model saved at epoch 20

## Recommended Setup

For training from scratch:

1. **Budget option**: RTX A6000 or Tesla V100 (~$0.02-0.04)
2. **Balanced option**: RTX 6000 Pro or L40S (~$0.06-0.08)
3. **Fast option**: A100-80 or H100 (~$0.08-0.11)

For inference development:

- RTX 6000 Pro provides good balance of cost and performance
- A100 recommended for production inference benchmarking

## Cost Comparison

### Traditional vs RAG-Based Approach

| Approach | Initial Cost | Per-Extension Cost |
|----------|--------------|-------------------|
| **Neurlang (RAG)** | **~$0.08** | **$0** |
| Train per extension | $50-100+ | $5-10+ each |
| Large LLM API | $0 | $0.01-0.10/call |

### Long-Term Projection

| Period | Cost |
|--------|------|
| Initial training | ~$0.08 |
| Fine-tuning (quarterly) | ~$0.02-0.04 |
| **Year 1 total** | **~$0.16-0.24** |
| **Year 2+ (fine-tuning only)** | **~$0.08-0.16** |

## See Also

- [Training Guide](./README.md) - How to train the model
- [PROFILES.md](../PROFILES.md) - GPU instance setup scripts
