# RAG-Enhanced Intent Classification

Neurlang's inference pipeline includes an in-memory RAG-based intent classification layer that provides 3x faster intent detection with confidence-based routing.

## Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         STARTUP (once)                                  │
│  Load intent_index.bin (54 intents × 384 dims = ~82KB)                  │
│  Load example_index.bin (optional, 100K × 384 = ~147MB)                 │
│  ~50MB RAM, ~100ms load time                                            │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                       HOT PATH (per query)                              │
│                                                                         │
│  Query "add 5 and 3"                                                    │
│         │                                                               │
│         ▼                                                               │
│  ┌─────────────────┐                                                    │
│  │ FastEmbedder    │  ~0.05ms (in-process, zero allocation)             │
│  │ (MiniLM-L6)     │  Reuses pre-allocated 384-dim buffer               │
│  └────────┬────────┘                                                    │
│           │ 384-dim vector                                              │
│           ▼                                                             │
│  ┌─────────────────┐                                                    │
│  │ IntentIndex     │  ~0.02ms (54 dot products, SIMD-optimized)         │
│  │ classify()      │  O(n) where n=54 intents                           │
│  └────────┬────────┘                                                    │
│           │ (intent_id=0, confidence=0.92)                              │
│           ▼                                                             │
│      confidence routing                                                 │
│         /        \                                                      │
│       HIGH        LOW                                                   │
│      (>0.7)      (<0.7)                                                 │
│        │           │                                                    │
│        ▼           ▼                                                    │
│  ┌──────────┐  ┌──────────────┐                                         │
│  │ Direct   │  │ 25M ONNX     │  Only ~10% of queries                   │
│  │ Generator│  │ Model        │  hit fallback path                      │
│  │ ~0.01ms  │  │ ~0.3ms       │                                         │
│  └────┬─────┘  └──────┬───────┘                                         │
│       │               │                                                 │
│       └───────┬───────┘                                                 │
│               ▼                                                         │
│        IR Generation → Compile → Execute                                │
│                                                                         │
│  FAST PATH: ~0.1ms total                                                │
│  FALLBACK PATH: ~0.4ms total                                            │
└─────────────────────────────────────────────────────────────────────────┘
```

## Performance Targets

| Path | Latency | When Used |
|------|---------|-----------|
| Fast path | ~0.1ms | confidence > 0.7 (~90% of queries) |
| With hints | ~0.15ms | 0.5 < confidence < 0.7 (~5%) |
| Fallback | ~0.4ms | confidence < 0.5 (~5%) |

**Comparison to previous system:**

| Metric | Before RAG | With RAG | Improvement |
|--------|------------|----------|-------------|
| Intent classification | ~0.3ms | ~0.05ms | **6x faster** |
| Full pipeline (avg) | ~0.35ms | ~0.12ms | **3x faster** |
| Memory overhead | 0 | ~150MB | Tradeoff |

## Components

### IntentIndex

Pre-computed embeddings for all 54 intent types:

```rust
use neurlang::inference::{IntentIndex, INTENT_DESCRIPTIONS};

// Build index from canonical descriptions
let index = IntentIndex::build_from_descriptions(&embedder, &INTENT_DESCRIPTIONS)?;

// Classify a query embedding
let (intent_id, confidence) = index.classify(&query_embedding);

// Save/load for persistence
index.save("~/.neurlang/intent_index.bin")?;
let loaded = IntentIndex::load("~/.neurlang/intent_index.bin")?;
```

**File format:** Binary with NLII magic, 54 × 384 × 4 bytes = ~82KB

### FastEmbedder

Zero-allocation wrapper for hot-path embedding:

```rust
use neurlang::inference::FastEmbedder;

let embedder = FastEmbedder::new(Box::new(ollama_embedder));

// Zero-allocation embedding into fixed buffer
let embedding: [f32; 384] = embedder.embed_384("add 5 and 3")?;

// Or embed into a provided slice
embedder.embed_into("add 5 and 3", &mut output_buffer)?;
```

### ExampleIndex (Optional)

For borderline confidence (0.5-0.7), retrieve similar training examples:

```rust
use neurlang::inference::ExampleIndex;

// Load pre-built example index
let mut examples = ExampleIndex::load("example_index.bin", "example_meta.bin")?;

// Search for similar examples
let results = examples.search(&query_embedding, 3);

// Aggregate votes by intent
let votes = examples.search_intent_votes(&query_embedding, 5);
// Returns: [(intent_id, count, avg_score), ...]
```

### RagPipeline

Confidence-based routing pipeline:

```rust
use neurlang::inference::{RagPipeline, RagPipelineConfig, confidence};

let config = RagPipelineConfig {
    use_intent_index: true,
    use_example_index: true,
    high_confidence_threshold: confidence::HIGH,  // 0.7
    medium_confidence_threshold: confidence::MEDIUM,  // 0.5
    num_example_hints: 3,
};

let mut pipeline = RagPipeline::new(config)?;
let result = pipeline.run("compute factorial of 5")?;

// Check which path was taken
match result.path {
    InferencePath::Fast => println!("High confidence, direct generation"),
    InferencePath::WithHints => println!("Used example hints"),
    InferencePath::Fallback => println!("Used full model"),
    InferencePath::Legacy => println!("RAG disabled"),
}
```

## CLI Commands

### Building Indices

```bash
# Build intent index from canonical descriptions
nl index --build

# Build example index (optional, for borderline queries)
nl index --build-examples

# Show index status
nl index --info

# Verify index accuracy
nl index --verify
```

### Configuration

```bash
# Use specific embedder
nl index --build --embedder onnx

# Use Ollama with specific model
nl index --build --ollama nomic-embed-text
```

## Confidence Thresholds

| Threshold | Value | Meaning |
|-----------|-------|---------|
| HIGH | 0.7 | Direct generation without model |
| MEDIUM | 0.5 | Use example hints for disambiguation |
| Below MEDIUM | <0.5 | Full model inference required |

These thresholds are tuned based on validation accuracy:
- 95%+ accuracy when confidence > 0.7
- 85%+ accuracy when confidence > 0.5
- Full model provides 98%+ accuracy as fallback

## Intent Descriptions

The 54 canonical intent descriptions map prompts to operations:

| ID | Intent | Description |
|----|--------|-------------|
| 0 | ADD | add two numbers together |
| 1 | SUB | subtract one number from another |
| 2 | MUL | multiply two numbers |
| 3 | DIV | divide one number by another |
| 4 | MOD | compute remainder/modulo |
| 5 | NEG | negate a number |
| ... | ... | ... |
| 53 | HALT | stop program execution |

Full list in `src/inference/intent_index.rs`.

## Architecture Benefits

1. **Zero disk I/O on hot path** - All indices loaded at startup
2. **No external services** - No Ollama/API calls during fast path
3. **Pre-computed embeddings** - Intent descriptions embedded once at build time
4. **Graceful degradation** - Falls back to full model when confidence is low
5. **Extensibility** - Add new intents by adding one embedding vector

## Memory Footprint

| Component | Size |
|-----------|------|
| IntentIndex (54 intents) | ~82KB |
| ExampleIndex (100K examples) | ~147MB |
| FastEmbedder buffer | 1.5KB |
| **Total (with examples)** | **~150MB** |
| **Total (without examples)** | **~100KB** |

## See Also

- [Embeddings System](./embeddings.md) - Backend configuration
- [Two-Tier Orchestration](./two-tier-orchestration.md) - Generation architecture
- [Performance Benchmarks](../PERFORMANCE.md) - Latency measurements
- [CLI Commands](../cli/commands.md) - `nl index` reference
