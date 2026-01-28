# Embeddings System

Neurlang uses real ML embeddings for semantic search and pattern matching in the two-tier orchestration system. No fallback/hash-based embeddings are used in production.

## Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    EMBEDDINGS ARCHITECTURE                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  User Query: "implement fibonacci function"                     │
│                           │                                     │
│                           ▼                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              EMBEDDER (Ollama or ONNX)                   │   │
│  │                                                         │   │
│  │  Input: "implement fibonacci function"                   │   │
│  │  Output: [0.123, -0.456, 0.789, ...] (768 dimensions)   │   │
│  └─────────────────────────────────────────────────────────┘   │
│                           │                                     │
│          ┌────────────────┴────────────────┐                   │
│          ▼                                 ▼                   │
│  ┌───────────────────┐          ┌─────────────────────┐       │
│  │  PATTERN MATCHING │          │   SEMANTIC SEARCH    │       │
│  │  (Tier Decision)  │          │   (Context Retrieval)│       │
│  │                   │          │                      │       │
│  │  Compare to known │          │  Find relevant       │       │
│  │  training patterns│          │  requirements/docs   │       │
│  └───────────────────┘          └─────────────────────┘       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Supported Backends

### Ollama (Recommended)

Uses Ollama's `/api/embeddings` endpoint with embedding models:

| Model | Dimensions | Quality | Speed |
|-------|------------|---------|-------|
| `nomic-embed-text` | 768 | Good | Fast |
| `mxbai-embed-large` | 1024 | Best | Slower |
| `all-minilm` | 384 | Basic | Fastest |

```bash
# Start Ollama
ollama serve

# Or via Docker
docker run -d -p 11434:11434 ollama/ollama
```

### ONNX (requires `--features ort-backend`)

Uses ONNX Runtime for local inference:

- Supports any ONNX embedding model
- Good for air-gapped deployments
- Place model at `~/.neurlang/models/embeddings.onnx`

## Configuration

### Environment Variables

```bash
# Ollama settings
export OLLAMA_HOST="http://localhost:11434"
export NEURLANG_EMBED_MODEL="nomic-embed-text"
```

### Programmatic Configuration

```rust
use neurlang::{EmbedderConfig, create_embedder};

// Explicit Ollama configuration
let config = EmbedderConfig::ollama("http://localhost:11434", "nomic-embed-text");
let embedder = create_embedder(config)?;

// Or auto-detect available backend
let embedder = create_embedder_auto()?;
```

## Usage in Neurlang

### VectorIndex (Context Retrieval)

```rust
use neurlang::{VectorIndex, IndexConfig, EmbedderConfig, create_embedder};
use std::sync::Arc;

// Create embedder
let config = EmbedderConfig::ollama_default("nomic-embed-text");
let embedder = Arc::new(create_embedder(config)?);

// Create index with embedder
let mut index = VectorIndex::with_embedder(IndexConfig::default(), embedder);

// Add documents
index.add_document("User authentication requirements...");
index.add_document("API rate limiting specs...");
index.build()?;  // Computes embeddings

// Search
let results = index.search("authentication", 5);
```

### PatternClassifier (Tier Decision)

```rust
use neurlang::orchestration::{PatternClassifier, TierDecision};

// Create classifier with embedder
let classifier = PatternClassifier::with_embedder(0.85, embedder);
classifier.add_pattern("compute factorial of a number", "arithmetic");

// Classify request
match classifier.classify("calculate factorial") {
    TierDecision::Tier1 { pattern, confidence } => {
        println!("Tier 1 can handle: {} ({:.2}% confidence)", pattern, confidence * 100.0);
    }
    TierDecision::Tier2 { reason } => {
        println!("Escalate to LLM: {}", reason);
    }
}
```

## Model Auto-Pull

When using Ollama, models are automatically pulled on first use:

```
$ nl agent --new "build REST API"
Pulling embedding model 'nomic-embed-text'...
Model 'nomic-embed-text' pulled successfully.
[generating...]
```

## Dimension Handling

Different models produce different embedding dimensions:

| Constant | Value | Models |
|----------|-------|--------|
| `EMBEDDING_DIM` | 384 | all-minilm, MiniLM-L6 |
| `NOMIC_EMBED_DIM` | 768 | nomic-embed-text |
| `MXBAI_EMBED_DIM` | 1024 | mxbai-embed-large |

The system automatically handles dimension differences:
- VectorIndex resizes its query buffer as needed
- PatternClassifier compares embeddings of the same dimension

## FastEmbedder (Zero-Allocation)

For hot-path inference, `FastEmbedder` provides zero-allocation embedding:

```rust
use neurlang::inference::FastEmbedder;

// Wrap any embedder
let fast = FastEmbedder::new(Box::new(ollama_embedder));

// Zero-allocation: returns fixed-size array
let embedding: [f32; 384] = fast.embed_384("add 5 and 3")?;

// Or embed into a pre-allocated buffer
let mut buffer = [0f32; 384];
fast.embed_into("add 5 and 3", &mut buffer)?;
```

**Benefits:**
- No heap allocation per query
- Pre-sized output buffer
- Optimal for tight loops

## Performance

| Operation | Latency |
|-----------|---------|
| Ollama embed (nomic) | ~10-20ms |
| Ollama embed (mxbai) | ~30-50ms |
| ONNX embed (MiniLM) | ~5-10ms |
| **FastEmbedder (cached)** | **~0.05ms** |
| Cosine similarity | <0.01ms |
| Vector search (10K docs) | ~1ms |
| Intent classification (54) | ~0.02ms |

## Error Handling

```rust
use neurlang::{create_embedder_auto, EmbedderError};

match create_embedder_auto() {
    Ok(embedder) => {
        // Ready to use
    }
    Err(EmbedderError::OllamaNotAvailable { host, message }) => {
        eprintln!("Ollama not running at {}: {}", host, message);
        eprintln!("Start with: ollama serve");
    }
    Err(EmbedderError::NoBackendConfigured) => {
        eprintln!("No embedding backend available");
        eprintln!("Install Ollama or compile with --features ort-backend");
    }
    Err(e) => {
        eprintln!("Embedder error: {}", e);
    }
}
```

## Testing Without Backend

For unit tests, a `HashEmbedder` is available under `#[cfg(test)]` only:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity() {
        let embedder = HashEmbedder::new();  // Test-only, not for production
        let e1 = embedder.embed("hello world").unwrap();
        let e2 = embedder.embed("hello there").unwrap();
        assert!(cosine_similarity(&e1, &e2) > 0.5);
    }
}
```
