# Neurlang Project Roadmap

**Last Updated:** January 25, 2026

---

## Executive Summary

Neurlang is **Claude Code with a 1000x faster iteration loop** - an AI coding agent with a custom programming language designed for rapid automated iteration. Give it a task (even 10GB of requirements), and it generates, tests, and fixes code in a tight internal loop until it produces a verified working solution - all in seconds, running entirely on your local machine.

### Current Status: **Phase 5 In Progress** (Slot-Based Code Generation)

| Metric | Current | Target |
|--------|---------|--------|
| Parallel Prediction | 64 slots/pass | 64 slots |
| Compile Time | <5us | <5us |
| Inference Time | ~30ms | <50ms |
| Session Persistence | Done | Done |
| Interactive Agent | Done | Done |
| Extension System | Done | Done |
| Async I/O | Done | Done |
| RAG Extension Resolution | Done | Done |
| Two-Tier Orchestration | Done | Done |
| Real Embeddings | Done | Done |
| **Slot-Based Generation** | **In Progress** | **<500ms** |
| **Protocol Specs** | **0/5** | **5 (SMTP, HTTP, etc.)** |
| **Slot Training Examples** | **0** | **5,000+** |

---

## Architecture Overview

```
+---------------------------------------------------------------------------+
|                         USER                                               |
|  "build a REST API with user authentication and order system"             |
+---------------------------------------------------------------------------+
                              |
                              v
+---------------------------------------------------------------------------+
|                    PATTERN CLASSIFIER (Rust code)                          |
|                                                                            |
|  Embed request, compare to known patterns via cosine similarity            |
|  - If similarity > threshold -> Tier 1 (small model)                       |
|  - If similarity < threshold -> Tier 2 (LLM decomposes task)               |
+---------------------------------------------------------------------------+
                              |
              +---------------+---------------+
              |                               |
              v                               v
+--------------------------+    +--------------------------------+
|     TIER 1: SMALL MODEL  |    |   TIER 2: LLM PROJECT MANAGER  |
|                          |    |                                |
|  50-100M params, ONNX    |    |  Claude/Ollama/OpenAI          |
|  64 parallel slots       |    |  Decomposes complex tasks      |
|  ~30ms inference         |    |  Returns subtasks for Tier 1   |
+--------------------------+    +--------------------------------+
              |                               |
              +---------------+---------------+
                              |
                              v
+---------------------------------------------------------------------------+
|                    REQUIREMENTS INDEX (In-Memory)                          |
|                                                                            |
|  For large specs (up to 10GB+):                                            |
|  - Chunk into ~1000 token segments                                         |
|  - Generate embeddings (small embedding model)                             |
|  - Store in-memory (hora crate or Vec + brute-force)                       |
|  - Retrieval: ~0.5-1ms semantic search                                     |
+---------------------------------------------------------------------------+
                              |
                              v
+---------------------------------------------------------------------------+
|                    RAG EXTENSION RESOLUTION                                |
|                                                                            |
|  Model emits: EXT.CALL @"parse JSON", r0, r1                               |
|  RAG matches "parse JSON" -> json_parse extension -> ID 170                |
|  Assembler emits: EXT.CALL 170, r0, r1                                     |
+---------------------------------------------------------------------------+
                              |
                              v
+---------------------------------------------------------------------------+
|                 COMPILE + EXECUTE                                          |
|                                                                            |
|  Compiler: Copy-and-patch (5us to native code)                             |
|  Runtime: Native execution with Rust extensions                            |
|  Extensions: Three layers (Runtime, Stdlib, Bundled/User)                  |
+---------------------------------------------------------------------------+
                              |
                              v
+---------------------------------------------------------------------------+
|                      VERIFIER                                              |
|                                                                            |
|  - Run generated code with test inputs                                     |
|  - Compare output against expected results                                 |
|  - If pass: mark complete, move to next subtask                            |
|  - If fail: extract error, feed back to model                              |
+---------------------------------------------------------------------------+
```

---

## Three-Layer Extension Architecture

```
+-------------------------------------------------------------------------+
|                         RUNTIME                                          |
|  (The execution engine - not trainable, just code)                       |
+-------------------------------------------------------------------------+
|  - Copy-and-patch compiler (5us compilation)                             |
|  - Register machine (32 registers)                                       |
|  - Extension dispatcher (ID -> function lookup)                          |
|  - RAG index for extension resolution                                    |
+-------------------------------------------------------------------------+

+-------------------------------------------------------------------------+
|                         STDLIB                                           |
|  (Trained into model - keep MINIMAL)                                     |
+-------------------------------------------------------------------------+
|  Only what you can't build real programs without:                        |
|  - Vec (dynamic arrays)          ~13 operations                          |
|  - HashMap (key-value)           ~10 operations                          |
|  - String (text manipulation)    ~19 operations                          |
|  Total: ~42 operations to train                                          |
+-------------------------------------------------------------------------+

+-------------------------------------------------------------------------+
|                      BUNDLED EXTENSIONS                                  |
|  (Ships with neurlang, discovered via RAG, NOT trained)                  |
+-------------------------------------------------------------------------+
|  - JSON parsing/stringify                                                |
|  - HTTP client (GET/POST/PUT/DELETE)                                     |
|  - Crypto (SHA256, HMAC, AES, signatures)                                |
|  - File I/O, SQLite, Regex, DateTime, UUID, Base64                       |
|  Training cost: $0 (RAG resolves them)                                   |
+-------------------------------------------------------------------------+

+-------------------------------------------------------------------------+
|                      USER EXTENSIONS                                     |
|  (Installed by user, discovered via RAG)                                 |
+-------------------------------------------------------------------------+
|  - Domain-specific APIs (Stripe, AWS, etc.)                              |
|  - Custom business logic                                                 |
|  - Training cost: $0                                                     |
+-------------------------------------------------------------------------+
```

---

## Implementation Status

### Phase 1: Complete Language + Compiler - COMPLETE

| Task | Status | Location |
|------|--------|----------|
| IR format (32 opcodes) | Done | `src/ir/format.rs` |
| Assembler/Disassembler | Done | `src/ir/assembler.rs` |
| Stencil build system | Done | `build.rs` |
| Copy-and-patch compiler | Done | `src/compile/engine.rs` |
| Buffer pool (RWX) | Done | `src/runtime/buffer_pool.rs` |
| Security stencils | Done | `src/stencil/security.rs` |
| Concurrency stencils | Done | `src/stencil/concurrency.rs` |
| I/O stencils | Done | `src/stencil/io.rs` |
| CLI (asm, run, agent) | Done | `src/main.rs` |
| ONNX export script | Done | `scripts/export_onnx.py` |
| Inference engine | Done | `src/inference/engine.rs` |
| CLI prompt command | Done | `nl prompt "task"` |
| Training data generator | Done | `datagen/src/main.rs` |
| Train model | Done | Multi-head prediction |

### Phase 2: Agent Architecture - COMPLETE

| Task | Status | Location |
|------|--------|----------|
| Parallel slot prediction (64) | Done | `src/train/model.rs` |
| Cross-attention decoder | Done | `src/train/model.rs` |
| Instruction dataset format | Done | `src/train/dataset.rs` |
| Multi-head loss function | Done | `src/train/trainer.rs` |
| Session persistence | Done | `src/inference/session.rs` |
| Agent with hot/cold path | Done | `src/inference/agent.rs` |
| Test verification | Done | `src/inference/verify.rs` |
| In-memory vector index | Done | `src/inference/index.rs` |
| Agent CLI subcommand | Done | `nl agent --new/--continue/--resume` |
| Extension manifest | Done | `src/extensions/manifest.rs` |
| Extension registry | Done | `src/extensions/registry.rs` |
| Extension loader | Done | `src/extensions/loader.rs` |
| Extension CLI | Done | `nl extension --add/--new/--list` |
| Async I/O runtime | Done | `src/runtime/async_io/` |
| Intrinsic expansion (@name) | Done | `src/ir/assembler.rs` |
| Hard-coded symbolic names | Done | `src/ir/assembler.rs:798-813` |
| Vec, HashMap, String stdlib | Done | `src/runtime/stdlib/` |
| JSON, HTTP stdlib | Done | `src/runtime/stdlib/` |
| Crypto extensions | Done | `src/runtime/extensions.rs` |

### Phase 3: RAG + Two-Tier Orchestration - COMPLETE

All Phase 3 tasks are complete. Only 3B.5 (OpenAI-compatible backend) remains as a P2 future enhancement.

#### Phase 3A: RAG-Based Extension Resolution

**Goal**: Dynamic `@"description"` -> semantic search -> extension ID

- [x] **3A.1: Update assembler for intent syntax** ✅ DONE
  - File: `src/ir/assembler.rs`
  - Parse `EXT.CALL @"description", args` (quoted string = intent)
  - Now handles both `@ext_name` (identifier) and `@"description"` (intent) syntax
  - Priority: P0

- [x] **3A.2: Create RAG resolver module** ✅ DONE
  - File: `src/ir/rag_resolver.rs` (NEW)
  - Query with intent string via keyword/cosine similarity
  - Returns matching extension ID + type signature
  - All bundled extensions registered (JSON, HTTP, Crypto, FS, SQLite, etc.)
  - Priority: P0

- [x] **3A.3: Extension description indexing** ✅ DONE
  - File: `src/extensions/registry.rs`
  - Added `register_with_rag()` method to register all extensions
  - Added `get_by_rag_id()` method to lookup by resolved ID
  - User extensions get IDs starting from 500
  - Priority: P0

- [x] **3A.4: Wire RAG resolver into assembler** ✅ DONE
  - File: `src/ir/assembler.rs`
  - Call RAG resolver when encountering `@"..."` syntax
  - Fall back to RAG name lookup for `@name` syntax (replaces hardcoded)
  - Priority: P0

#### Phase 3B: LLM as Project Manager (Two-Tier Orchestration)

**Goal**: LLM decomposes complex tasks, small model generates verified IR

- [x] **3B.1: Create orchestration module** ✅ DONE
  - File: `src/orchestration/mod.rs` (NEW)
  - OrchestratorConfig, OrchestrationResult, OrchestratorError structs
  - Priority: P0

- [x] **3B.2: Define LLM backend trait** ✅ DONE
  - File: `src/orchestration/backends/mod.rs` (NEW)
  - Trait: `LlmBackend { fn decompose_task(&self, task: &str) -> DecomposeResult }`
  - BackendRegistry for managing multiple backends
  - Priority: P0

- [x] **3B.3: Implement Claude backend** ✅ DONE
  - File: `src/orchestration/backends/claude.rs` (NEW)
  - Anthropic API integration with mock support when feature disabled
  - Priority: P0

- [x] **3B.4: Implement Ollama backend** ✅ DONE
  - File: `src/orchestration/backends/ollama.rs` (NEW)
  - Local LLM support via Ollama API
  - Priority: P1

- [ ] **3B.5: Implement OpenAI-compatible backend**
  - File: `src/orchestration/backends/openai.rs` (NEW)
  - For OpenRouter, local vLLM, etc.
  - Priority: P2

- [x] **3B.6: Implement pattern classifier** ✅ DONE
  - File: `src/orchestration/classifier.rs` (NEW)
  - Bag-of-words embedding with TF-IDF-like weighting
  - Cosine similarity to known patterns
  - TierDecision enum for routing
  - Priority: P1

- [x] **3B.7: Integrate orchestrator with agent** ✅ DONE
  - File: `src/inference/agent.rs`
  - Added `enable_orchestration` config flag
  - Pattern classifier routes to Tier 1 or Tier 2
  - Tier 2 decomposes via LLM, processes subtasks
  - Training data collector records successes
  - Priority: P0

- [x] **3B.8: Training data collector** ✅ DONE
  - File: `src/orchestration/collector.rs` (NEW)
  - TrainingDataCollector with batch writing
  - record_success(), record_error_recovery() methods
  - JSONL format for training data
  - Priority: P1

- [x] **3B.9: Backend CLI configuration** ✅ DONE
  - `nl config --set/--get/--list/--unset` commands
  - `nl backends --list/--status/--set-default/--test` commands
  - Config stored in ~/.neurlang/config.json
  - Priority: P1

#### Phase 3C: Real Embeddings ✓

**Goal**: Require real embedding backends (Ollama or ONNX) - no fallback

- [x] **3C.1: Create embedder module with real backends**
  - New file: `src/inference/embedder.rs`
  - `Embedder` trait with `embed()`, `embed_batch()`, `embedding_dim()`, `is_available()`, `is_ml_based()`
  - `OllamaEmbedder` - embeddings via Ollama (nomic-embed-text, mxbai-embed-large, all-minilm)
  - `OnnxEmbedder` - embeddings via ONNX (requires ort-backend feature)
  - `EmbedderConfig` enum for configuration (Ollama or ONNX)
  - `create_embedder()` - create from config
  - `create_embedder_auto()` - auto-detect available backend
  - `cosine_similarity()` utility
  - Auto-pull Ollama models via `ensure_model()`
  - Dimension constants: `EMBEDDING_DIM` (384), `NOMIC_EMBED_DIM` (768), `MXBAI_EMBED_DIM` (1024)
  - 5 unit tests (HashEmbedder only for tests)
  - Priority: P0

- [x] **3C.2: Integrate embedder with VectorIndex**
  - Updated: `src/inference/index.rs`
  - Added `embedder` field to `VectorIndex`
  - `build()` requires embedder or pre-computed embeddings (no fallback)
  - `search()` requires embedder (or use `search_embedding()` with pre-computed)
  - Added `with_embedder()`, `load_with_embedder()` constructors
  - Added `set_embedder()`, `embedder_name()`, `uses_ml_embeddings()` methods
  - Priority: P0

- [x] **3C.3: Integrate embedder with PatternClassifier**
  - Updated: `src/orchestration/classifier.rs`
  - Added optional `embedder` field
  - Added `with_embedder()` constructor
  - Added `set_embedder()`, `uses_ml_embeddings()` methods
  - Falls back to legacy bag-of-words only when no embedder configured
  - Priority: P0

- [x] **3C.4: Export embedder types in lib.rs**
  - `Embedder`, `EmbedderError`, `EmbedderConfig`, `OllamaEmbedder`
  - `OnnxEmbedder` (with ort-backend feature)
  - `create_embedder()`, `create_embedder_auto()`, `cosine_similarity()`
  - Dimension constants
  - Priority: P0

- [x] **3C.5: Ollama embeddings integration**
  - Uses Ollama's `/api/embeddings` endpoint
  - Auto-pull models with `ensure_model()`
  - Supports: nomic-embed-text (768d), mxbai-embed-large (1024d), all-minilm (384d)
  - Environment variables: `OLLAMA_HOST`, `NEURLANG_EMBED_MODEL`
  - Priority: P0

#### Phase 3D: Documentation

- [x] **3D.1: Create docs/architecture/three-layers.md**
  - Runtime vs Stdlib vs Extensions explanation
  - Priority: P0

- [x] **3D.2: Create docs/architecture/rag-extensions.md**
  - How RAG-based extension resolution works
  - Priority: P0

- [x] **3D.3: Create docs/architecture/two-tier-orchestration.md**
  - Tier 1 (small model) + Tier 2 (LLM) system
  - Priority: P0

- [x] **3D.4: Create docs/extensions/bundled.md**
  - List of bundled extensions and their APIs
  - Priority: P1

- [x] **3D.5: Create docs/extensions/creating.md**
  - How to create and publish extensions
  - Priority: P1

- [x] **3D.6: Create docs/training/costs.md**
  - Training cost breakdown and optimization
  - Priority: P1

- [x] **3D.7: Update docs/ROADMAP.md**
  - This file - project management checklist
  - Priority: P0

#### Phase 3E: Training and Testing Infrastructure ✓

**Goal**: Enable end-to-end training and testing of the parallel prediction model

- [x] **3E.1: Update datagen for parallel training format**
  - Updated: `datagen/src/main.rs`
  - Added `--parallel` flag for instruction-level output format
  - New `ParallelTrainingExample` struct with `InstructionData` fields
  - Converts assembly → Program → instruction-level data
  - Test cases included in output for verification
  - Priority: P0

- [x] **3E.2: RAG seed bank initialization utilities**
  - Updated: `src/extensions/mod.rs`
  - Added `create_configured_assembler()` - creates Assembler with all extensions (bundled + user)
  - Added `list_all_extensions()` - get all registered extensions
  - Added `extension_count()` - count (bundled, user) extensions
  - Exported in `lib.rs` for easy access
  - Priority: P0

---

### Phase 4: Rust→IR Compiler - COMPLETE

**Goal**: Build a Rust→Neurlang IR compiler that generates stdlib from verified Rust implementations.

#### Phase 4A: Compiler Infrastructure

- [x] **4A.1: Create compiler module** ✅ DONE
  - File: `src/compiler/mod.rs`
  - Parser using syn crate
  - Analyzer for type checking and scoping
  - Code generator for IR emission
  - Priority: P0

- [x] **4A.2: Implement parser** ✅ DONE
  - File: `src/compiler/parser.rs`
  - Parse Rust functions via syn
  - Extract types, parameters, expressions
  - Support subset: integers, floats, control flow
  - Priority: P0

- [x] **4A.3: Implement analyzer** ✅ DONE
  - File: `src/compiler/analyzer.rs`
  - Type checking (u64, i64, f64, bool)
  - Variable scoping and shadowing
  - Register pre-allocation
  - Priority: P0

- [x] **4A.4: Implement code generator** ✅ DONE
  - File: `src/compiler/codegen.rs`
  - Emit Neurlang IR instructions
  - Handle arithmetic, comparisons, control flow
  - Generate branch labels
  - FPU operations for f64
  - Priority: P0

- [x] **4A.5: Implement test generator** ✅ DONE
  - File: `src/compiler/test_gen.rs`
  - Generate test inputs based on types
  - Execute Rust to get expected outputs
  - Emit @test annotations
  - Priority: P0

#### Phase 4B: Configuration System

- [x] **4B.1: Create neurlang.toml schema** ✅ DONE
  - File: `neurlang.toml`
  - Package metadata
  - Stdlib module enable/disable flags
  - Extension configuration (crates, go, c, local)
  - Build settings
  - Priority: P0

- [x] **4B.2: Implement config parser** ✅ DONE
  - File: `src/config.rs`
  - Parse neurlang.toml with serde
  - NeurlangConfig, StdlibConfig, BuildConfig structs
  - Load from cwd or specified path
  - Priority: P0

#### Phase 4C: CLI Integration

- [x] **4C.1: Add stdlib command** ✅ DONE
  - `nl stdlib --build` - Build all modules
  - `nl stdlib --verify` - Verify Rust == Neurlang
  - `--verbose` flag for progress
  - `--config` flag for custom config path
  - Priority: P0

- [x] **4C.2: Add resolve command** ✅ DONE
  - `nl resolve "intent"` - RAG resolution
  - Shows top matches with similarity scores
  - Priority: P1

- [x] **4C.3: Add crate command** ✅ DONE
  - `nl crate --add/--remove/--list`
  - Integrates with neurlang.toml
  - Priority: P1

#### Phase 4D: Stdlib Implementation

- [x] **4D.1: Math module** ✅ DONE (93% pass rate)
  - factorial, fibonacci, gcd, power, is_prime, etc.
  - 46/49 tests passing
  - Priority: P0

- [x] **4D.2: String module** ✅ DONE
  - strlen, strcmp, atoi, itoa
  - Priority: P0

- [x] **4D.3: Array module** ✅ DONE
  - sum, min, max, reverse, search
  - Priority: P0

- [x] **4D.4: Bitwise module** ✅ DONE
  - popcount, clz, ctz, rotl, rotr
  - Priority: P0

- [x] **4D.5: Collections module** ✅ DONE
  - Basic stack, queue operations
  - Priority: P1

- [ ] **4D.6: Float module** - PARTIAL
  - FPU operations implemented
  - Float comparisons not yet supported
  - Priority: P1

#### Phase 4E: Documentation

- [x] **4E.1: Update compiler docs** ✅ DONE
  - `docs/compiler/README.md` - Added Rust→IR section
  - Priority: P0

- [x] **4E.2: Create stdlib guide** ✅ DONE
  - `docs/stdlib/README.md` - Development guide
  - Priority: P0

- [x] **4E.3: Update CLI docs** ✅ DONE
  - `docs/cli/commands.md` - Added stdlib, resolve, crate, test
  - Priority: P0

---

## Phase 5: Slot-Based Code Generation - IN PROGRESS

**Goal**: Enable 1000x faster code generation via template+slot architecture with parallel slot filling.

### Architecture Overview

```
USER REQUEST → DECOMPOSITION → SLOT FILLING → ASSEMBLY → VERIFICATION
                   │               │
           Rule-based OR LLM   Parallel batch
           produces SlotSpec   ~50ms total
```

**Key Performance Gains**:
- Parallel slot generation: All slots in single forward pass (~50ms total)
- Slot caching: 60-80% hit rate after warmup
- Adaptive slot sizing: 50-200 instructions based on complexity
- Total generation time: <500ms cold, <100ms cached

### Phase 5A: Core Infrastructure - COMPLETE ✅

| Task | Status | Files |
|------|--------|-------|
| 5A.1 Create slot module structure | [x] | `src/slot/mod.rs` |
| 5A.2 Implement SlotType enum (20 types) | [x] | `src/slot/types.rs` |
| 5A.3 Implement SlotSpec struct | [x] | `src/slot/spec.rs` |
| 5A.4 Implement JSON spec parser | [x] | `src/slot/parser.rs` |
| 5A.5 Create SMTP protocol spec | [x] | `specs/protocols/smtp.json` |
| 5A.6 Create HTTP protocol spec | [x] | `specs/protocols/http.json` |
| 5A.7 Create Redis protocol spec | [x] | `specs/protocols/redis.json` |
| 5A.8 Create FTP protocol spec | [x] | `specs/protocols/ftp.json` |
| 5A.9 Create DNS protocol spec | [x] | `specs/protocols/dns.json` |

### Phase 5B: Template Engine - COMPLETE ✅

| Task | Status | Files |
|------|--------|-------|
| 5B.1 Implement template expander | [x] | `src/slot/template.rs` |
| 5B.2 Implement intent parser | [x] | `src/slot/intent.rs` |
| 5B.3 Implement router (rule vs LLM) | [x] | `src/slot/router.rs` |
| 5B.4 Create TCP server skeleton | [x] | `templates/tcp_server.skeleton` |
| 5B.5 Create HTTP server skeleton | [x] | `templates/http_server.skeleton` |
| 5B.6 Create REST API skeleton | [x] | `templates/rest_api.skeleton` |
| 5B.7 Wire up `nl generate` command | [x] | `src/main.rs` |

### Phase 5C: Slot Filler & Verification - IN PROGRESS

| Task | Status | Files |
|------|--------|-------|
| 5C.1 Define slot training data format | [x] | `docs/slot/training-format.md` |
| 5C.2 Extract slots from protocol specs | [x] | `src/slot/training.rs` |
| 5C.3 Generate slot examples (~300) | [x] | `train/slot_training.jsonl` |
| 5C.4 Implement parallel slot filler | [x] | `src/slot/filler.rs` |
| 5C.5 Implement slot cache | [x] | `src/slot/cache.rs` |
| 5C.6 Implement per-slot verifier | [x] | `src/slot/verifier.rs` |
| 5C.7 Implement skeleton+slot assembler | [x] | `src/slot/assembler.rs` |
| 5C.8 Implement spec validator | [x] | `src/slot/validator.rs` |
| 5C.9 Wire up CLI commands | [x] | `nl protocol`, `nl slotdata` |
| 5C.10 End-to-end tests (mock filler) | [x] | `src/slot/router.rs` tests |
| 5C.11 Train slot-filling model | [ ] | Remote GPU - NEXT |

### SlotType Categories (20 Types)

| Category | Types | Instructions/Slot |
|----------|-------|-------------------|
| String/Pattern | PatternMatch, PatternSwitch, ResponseBuilder, StringCompare, StringCopy | 50-150 |
| Numeric | IntToString, StringToInt, RangeCheck | 20-50 |
| Control Flow | StateCheck, StateTransition, LoopUntil | 20-50 |
| I/O | SendResponse, ReadUntil, ReadNBytes | 30-80 |
| Extension | ExtensionCall, ValidationHook | 10-30 |
| Error | ErrorResponse | 20-40 |
| Data | BufferWrite, BufferRead | 10-30 |

### Success Criteria

| Metric | Target |
|--------|--------|
| `nl generate "SMTP server" --offline` | Produces compilable .nl |
| Parallel slot generation | All slots in single batch |
| Generation time (cold) | <500ms |
| Generation time (cached) | <100ms |
| First-attempt slot accuracy | >85% |
| Retry success rate | >95% |
| Protocol specs | 5+ working (SMTP, HTTP, Redis, FTP, DNS) |

### Future Phases (After 5 Complete)

| Phase | Description | Status |
|-------|-------------|--------|
| Phase 5D | Debug & iteration tools | Deferred |
| Phase 5E | Transpilation (Go/Rust/Python) | Deferred |
| Phase 5F | Deployment & packaging | Deferred |
| Phase 5G | Documentation generation | Deferred |

---

## Phase 6: Optimization (Future)

| Task | Priority | Notes |
|------|----------|-------|
| Mamba architecture | P2 | 5x faster inference |
| ARM64 stencils | P2 | Cross-platform support |
| INT4 quantization | P3 | Smaller model |
| RISC-V stencils | P3 | Embedded/IoT support |

---

## Phase 7: Production Tooling (Future)

| Task | Priority | Notes |
|------|----------|-------|
| LSP server | P2 | IDE integration |
| VS Code extension | P2 | Syntax highlighting |
| Debugger | P3 | Step-through execution |
| Extended math stdlib | P2 | sin, cos, tan, log, exp, pow |
| SQLite extension | P2 | Local persistence |
| Logging extension | P2 | Structured output |
| Testing framework | P2 | assert, expect, test runner |

---

## CLI Commands Reference

| Command | Description | Status |
|---------|-------------|--------|
| `nl asm` | Assemble text to binary IR | Done |
| `nl run` | Execute a program | Done |
| `nl prompt` | Generate code from natural language | Done |
| `nl datagen` | Generate training data | Done |
| `nl train` | Train the AI model (local or remote) | Done |
| `nl accuracy` | Test model accuracy | Done |
| `nl export-onnx` | Export PyTorch model to ONNX | Done |
| `nl spec` | Show IR specification | Done |
| `nl agent` | Interactive AI agent with sessions | Done |
| `nl extension` | Manage extensions (Go-style packages) | Done |
| `nl config` | Configure backends and settings | Done |
| `nl backends` | List and manage LLM backends | Done |
| `nl stdlib` | Build stdlib from Rust sources | Done |
| `nl resolve` | Resolve intent to extension ID | Done |
| `nl crate` | Manage crate extensions | Done |
| `nl test` | Run tests on .nl files | Done |

---

## Agent Commands

```bash
# Start a new session
nl agent --new "build a REST API with auth"

# Continue existing session
nl agent --cont abc123 "add rate limiting"

# Resume after crash
nl agent --resume

# List all sessions
nl agent --list

# Interactive mode
nl agent --interactive

# Force specific tier
nl agent --new "complex pattern" --backend claude
nl agent --new "offline mode" --backend ollama
```

---

## Files Reference

| File | Purpose |
|------|---------|
| `src/inference/agent.rs` | Interactive agent with hot/cold path |
| `src/inference/session.rs` | Session persistence and checkpointing |
| `src/inference/verify.rs` | Test verification module |
| `src/inference/index.rs` | In-memory vector index |
| `src/inference/embedder.rs` | Embeddings module (Ollama/ONNX backends) |
| `src/extensions/manifest.rs` | Extension manifest (neurlang.json) |
| `src/extensions/registry.rs` | Extension discovery and management |
| `src/extensions/loader.rs` | Extension loading and compilation |
| `src/train/model.rs` | Parallel prediction model |
| `src/train/dataset.rs` | Instruction sequence dataset |
| `src/train/trainer.rs` | Training with multi-head loss |
| `src/runtime/extensions.rs` | Rust FFI extensions (crypto, etc.) |
| `src/runtime/async_io/mod.rs` | Async I/O runtime module |
| `src/orchestration/mod.rs` | Two-tier orchestrator config and types |
| `src/orchestration/classifier.rs` | Pattern classifier for tier decision |
| `src/orchestration/collector.rs` | Training data collector |
| `src/orchestration/backends/mod.rs` | LlmBackend trait and BackendRegistry |
| `src/orchestration/backends/claude.rs` | Claude API backend |
| `src/orchestration/backends/ollama.rs` | Ollama local backend |
| `src/ir/rag_resolver.rs` | RAG-based extension resolution |
| `src/compiler/mod.rs` | Rust→IR compiler module |
| `src/compiler/parser.rs` | Rust AST parser (via syn) |
| `src/compiler/analyzer.rs` | Type checking and scoping |
| `src/compiler/codegen.rs` | IR code generation |
| `src/compiler/test_gen.rs` | Test annotation generator |
| `src/config.rs` | neurlang.toml configuration |
| `neurlang.toml` | Project configuration file |
| `stdlib/src/*.rs` | Stdlib Rust source files |
| `lib/*.nl` | Generated Neurlang assembly |
| **Slot-Based Generation (Phase 5)** | |
| `src/slot/mod.rs` | Slot module exports |
| `src/slot/types.rs` | SlotType enum (20 types) |
| `src/slot/spec.rs` | SlotSpec struct |
| `src/slot/parser.rs` | YAML protocol spec parser |
| `src/slot/template.rs` | Template expander |
| `src/slot/intent.rs` | Intent parser |
| `src/slot/router.rs` | Rule vs LLM router |
| `src/slot/filler.rs` | Parallel slot filling |
| `src/slot/cache.rs` | Slot cache |
| `src/slot/verifier.rs` | Per-slot verification |
| `src/slot/assembler.rs` | Skeleton + slot combiner |
| `src/slot/feedback.rs` | Error feedback loop |
| `specs/protocols/*.yaml` | Protocol specifications |
| `templates/*.skeleton` | Code skeleton templates |
| `datagen/src/slot_extractor.rs` | Extract slots from examples |

---

## Performance Targets

| Metric | Current | Phase 3 Target | Production |
|--------|---------|----------------|------------|
| Parallel Prediction | 64 slots | 64 slots | 64 slots |
| Compile Time | <5us | <5us | <5us |
| Inference Time | ~30ms | <50ms | <30ms |
| Hot Loop (zero I/O) | Yes | Yes | Yes |
| Session Persistence | Yes | Yes | Yes |
| Extension System | Yes | Yes | Yes |
| RAG Resolution | Yes | Yes | Yes |
| Two-Tier Orchestration | Yes | Yes | Yes |
| Real Embeddings | Yes | Yes | Yes |

---

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

---

## Training Cost Breakdown

| Component | Cost | Notes |
|-----------|------|-------|
| 32 core opcodes | $100-150 | Base training |
| ~42 stdlib operations | $50-100 | Trained (Vec, HashMap, String) |
| Intent emission skill | $20-30 | One pattern for RAG |
| **Total one-time cost** | **$170-280** | |
| JSON, HTTP, Crypto | $0 | RAG resolves |
| Future extensions | $0 | RAG resolves |

---

## Verification Checklist

### Unit Tests

- [ ] Model forward pass produces correct shapes: (batch, 64, num_classes)
- [ ] Prediction decoding produces valid Instructions
- [ ] All 32 opcodes can be predicted and decoded
- [ ] RAG resolver returns correct extension IDs
- [ ] Pattern classifier correctly routes to Tier 1 vs Tier 2

### Integration Tests

- [ ] Generated IR compiles without errors
- [ ] Simple prompts produce working programs (add, factorial)
- [ ] Error feedback improves output in subsequent iterations
- [ ] RAG-based extension calls work end-to-end
- [ ] Two-tier orchestration correctly decomposes complex tasks

### Performance Tests

- [ ] Forward pass latency < 50ms on GPU
- [ ] Compile + execute latency < 1ms
- [ ] 100 iterations complete in < 5 seconds
- [ ] Hot loop has zero disk I/O (verify with strace)
- [ ] RAG lookup < 2ms

### Session Persistence Tests

- [ ] Session saves correctly after generation completes
- [ ] Session loads correctly with all state intact
- [ ] `--continue` has full conversation history
- [ ] `--resume` after simulated crash recovers last checkpoint
- [ ] Vector index caches by requirements hash

### End-to-End Tests

- [ ] `neurlang agent --new "compute factorial of 10"` returns 3628800
- [ ] `neurlang agent --continue {id} "now compute for 12"` returns 479001600
- [ ] Kill process mid-generation, `--resume` recovers
- [ ] Complex task correctly escalates to LLM and decomposes

---

## See Also

- [Architecture Overview](./architecture/overview.md)
- [Three-Layer Architecture](./architecture/three-layers.md)
- [RAG-Based Extension Resolution](./architecture/rag-extensions.md)
- [Two-Tier Orchestration](./architecture/two-tier-orchestration.md)
- [Embeddings System](./architecture/embeddings.md)
- [Design Decisions](./DESIGN-DECISIONS.md)
- [CLI Reference](./cli/commands.md)
