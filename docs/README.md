# Neurlang Documentation

Complete documentation for the AI-Optimized Binary Programming Language.

**Neurlang achieves 1000x faster code generation** by combining a small local model (~50-100M params), a 32-opcode binary IR, and copy-and-patch compilation. The system iterates 100-1000 times in seconds to produce verified correct output.

## Documentation Index

| Section | Description |
|---------|-------------|
| [Architecture](./architecture/) | System design, two-tier orchestration, three-layer extensions |
| [Slot-Based Generation](./slot/) | 1000x faster code generation via template+slot architecture |
| [IR Specification](./ir/) | 32-opcode binary format, encoding, fat pointers |
| [Compiler](./compiler/) | Copy-and-patch + Rust→IR compilation |
| [Stdlib](./stdlib/) | Standard library development guide |
| [Stencils](./stencil/) | Pre-compiled code templates |
| [Code Generation](./codegen/) | Transpile IR to C, Go, Rust, pseudocode |
| [Runtime](./runtime/) | Buffer pool, async I/O, memory management |
| [Extensions](./extensions/) | Bundled extensions, creating extensions |
| [Interpreter](./interpreter/) | Fallback execution engine |
| [Security](./security/) | Capabilities, bounds checking, taint tracking |
| [Concurrency](./concurrency/) | Tasks, channels, atomics |
| [Training](./training/) | Data generation, model training, cost breakdown |
| [CLI Reference](./cli/) | Command-line interface usage |

## Quick Navigation

### For Users
- [CLI Quick Start](./cli/quickstart.md)
- [CLI Command Reference](./cli/commands.md)
- [CLI Examples](./cli/examples.md)
- [nl prompt](./cli/prompt.md) - Generate from natural language
- [nl generate](./cli/generate.md) - Generate programs from descriptions
- [nl agent](./cli/agent.md) - Interactive AI agent
- [nl extension](./cli/extension.md) - Package management
- [Assembly Language Guide](./ir/assembly.md)
- [Opcode Reference](./ir/opcodes.md)

### For Developers
- [How It Works](./architecture/how-it-works.md) - Technical deep dive
- [Architecture Overview](./architecture/overview.md)
- [Slot-Based Generation](./slot/README.md) - 1000x faster code generation
  - [Slot Types Reference](./slot/slot-types.md) - 20 slot types for any protocol
  - [Protocol Specifications](./slot/protocol-specs.md) - JSON format for protocols
  - [Verification System](./slot/verification.md) - How correctness is ensured
  - [Training Data Format](./slot/training-format.md) - Slot training data
- [Three-Layer Architecture](./architecture/three-layers.md) - Runtime, Stdlib, Extensions
- [Two-Tier Orchestration](./architecture/two-tier-orchestration.md) - LLM as project manager
- [RAG-Based Extension Resolution](./architecture/rag-extensions.md) - Dynamic extension lookup
- [RAG Intent Index](./architecture/rag-intent-index.md) - In-memory intent classification (3x faster)
- [Verified Pipeline](./architecture/verified-pipeline.md) - Mathematically verified inference
- [Stdlib Development](./stdlib/README.md) - Writing stdlib in Rust
- [Rust→IR Compiler](./compiler/README.md#rustir-compiler) - Compiling Rust to Neurlang
- [Multi-Architecture Support](./architecture/multiarch.md) - x86-64, ARM64, RISC-V
- [Design Decisions](./DESIGN-DECISIONS.md) - Why we made key choices
- [Project Roadmap](./ROADMAP.md) - Implementation status and checklist

### For ML Engineers
- [Training Overview](./training/README.md) - Multi-backend training pipeline
- [Training Costs](./training/costs.md) - Cost breakdown and optimization
- [Data Generation](./cli/datagen.md) - Synthetic training data
- [Training Commands](./cli/train.md) - Local and remote training
- [Model Accuracy](./cli/accuracy.md) - Accuracy testing
- [ONNX Export](./cli/export_onnx.md) - Model export for inference

**Supported Inference Engines:**
| Engine | Format | GPU | Binary Size | Best For |
|--------|--------|-----|-------------|----------|
| ort | .onnx | CUDA, TensorRT | +15 MB | Production |
| tract | .onnx | None | +2 MB | Minimal binary |
| candle | .onnx | CUDA | +10 MB | HuggingFace |
| burn | .mpk | wgpu | +45 MB | Native training |

### Extension Development
- [Bundled Extensions](./extensions/bundled.md) - Built-in extension API reference
- [Creating Extensions](./extensions/creating.md) - How to build and publish extensions
- [Extension CLI](./cli/extension.md) - Package management commands

### Component Documentation
- [Compiler](./compiler/README.md) - Copy-and-patch engine
- [Stencil System](./stencil/README.md) - Pre-compiled templates
- [Runtime](./runtime/README.md) - Buffer pool, async I/O
- [Interpreter](./interpreter/README.md) - Fallback execution
- [Security](./security/README.md) - Capabilities, taint tracking
- [Concurrency](./concurrency/README.md) - Tasks, channels, atomics

## System Architecture

```
+---------------------------------------------------------------------------+
|                         USER                                               |
|  "build a REST API with user authentication and order system"             |
+---------------------------------------------------------------------------+
                              |
                              v
+---------------------------------------------------------------------------+
|                    PATTERN CLASSIFIER                                      |
|  If simple task -> Tier 1 (small model, 30ms)                              |
|  If complex task -> Tier 2 (LLM decomposes into subtasks)                  |
+---------------------------------------------------------------------------+
                              |
                              v
+---------------------------------------------------------------------------+
|                    REQUIREMENTS INDEX (In-Memory)                          |
|  Semantic search on large specs: ~1ms retrieval                            |
+---------------------------------------------------------------------------+
                              |
                              v
+---------------------------------------------------------------------------+
|                    RAG EXTENSION RESOLUTION                                |
|  @"parse JSON" -> semantic search -> json_parse (ID 170)                   |
+---------------------------------------------------------------------------+
                              |
                              v
+---------------------------------------------------------------------------+
|                 COMPILE + EXECUTE + VERIFY                                 |
|  Copy-and-patch (5us) -> Native execution -> Test verification            |
+---------------------------------------------------------------------------+
```

## System Requirements

- **Rust**: 1.70+ (for building)
- **Python**: 3.8+ (optional, for PyTorch training)
- **PyTorch**: 2.0+ (optional, for Docker backend)
- **Docker/Podman**: Optional, for containerized training
- **OS**: Linux (primary), macOS, Windows
- **Architecture**: x86-64 (primary), ARM64 (supported)

### Build Variants

```bash
# Minimal (tract only, ~7 MB)
cargo build --release --features "tract"

# Standard (ort, GPU support, ~20 MB)
cargo build --release --features "ort-backend"

# Full (all engines + training, ~80 MB)
cargo build --release --features "ort-backend,tract,candle,train"
```

## Performance Targets

| Metric | Target | Typical |
|--------|--------|---------|
| Compile time (32 instr) | <5us | 2-3us |
| Interpreter IPS | >10M | 15M |
| Buffer acquire | <1us | 200ns |
| Model inference | <50ms | 30ms |
| RAG lookup | <2ms | 1ms |
| End-to-end code gen | <10us | 3-5us |
| Binary size (minimal) | <10MB | 7MB |
| Binary size (full) | <100MB | 80MB |

## Value Proposition

| Aspect | Claude Code / Copilot | Neurlang |
|--------|----------------------|----------|
| Model location | Cloud API | Local (50-100M params) |
| Language | Python/JS/etc (for humans) | 32-opcode binary IR (for AI) |
| Generation speed | 2-30 sec/iteration | 30ms/iteration |
| Compilation | 100ms-10s | 5us (copy-and-patch) |
| Iterations | 3-5 visible to user | 100-1000 automated |
| Result | Might have bugs | Verified correct |
| Total time | Minutes | Seconds |
| Offline | No | Yes |

## Reference

- **[Glossary](./GLOSSARY.md)** - All terms, concepts, and jargon explained
- [Project Roadmap](./ROADMAP.md) - Implementation status and upcoming work
- [Design Decisions](./DESIGN-DECISIONS.md) - Architectural rationale
- [GitHub Repository](https://github.com/neurlang/neurlang)
