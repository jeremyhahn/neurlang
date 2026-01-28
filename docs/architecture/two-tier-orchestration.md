# Two-Tier Orchestration: LLM as Project Manager

This document explains Neurlang's two-tier orchestration system where an LLM acts as a "project manager" to decompose complex tasks, while a small local model handles verified code generation.

## The Insight

**The LLM does NOT write IR directly.** It breaks complex tasks into subtasks that the small model can handle. This ensures ALL output goes through the verification loop.

## Architecture

```
+-------------------------------------------------------------------------+
|                 LLM AS PROJECT MANAGER                                   |
+-------------------------------------------------------------------------+
|                                                                          |
|  User: "build REST API with auth"                                        |
|                    |                                                     |
|                    v                                                     |
|  +---------------------------------------------------------------+      |
|  |           PATTERN CLASSIFIER (Rust code, not a model)         |      |
|  |  - Embed request using existing embedder                       |      |
|  |  - Compare to known patterns via cosine similarity             |      |
|  |  - If similarity > threshold -> small model handles it         |      |
|  |  - If similarity < threshold -> LLM decomposes task            |      |
|  +---------------------------------------------------------------+      |
|                    |                                                     |
|        +-----------+-----------+                                         |
|        v                       v                                         |
|  [Simple Task]           [Complex Task]                                  |
|        |                       |                                         |
|        v                       v                                         |
|  +-----------+    +---------------------------------------------+       |
|  |SMALL MODEL|    |  LLM (Claude/Ollama) - PLANNING ONLY        |       |
|  |Direct gen |    |  "Break into subtasks small model can       |       |
|  |           |    |   handle"                                   |       |
|  +-----------+    +---------------------------------------------+       |
|        |                       |                                         |
|        |                       v                                         |
|        |            Subtasks:                                            |
|        |            1. "create HTTP server on port 8080"                 |
|        |            2. "parse JSON from request body"                    |
|        |            3. "hash password with SHA256"                       |
|        |            4. "store user in hashmap"                           |
|        |            5. "return JSON response"                            |
|        |                       |                                         |
|        |                       v                                         |
|        |            +---------------------------------------------+      |
|        |            |  SMALL MODEL (for each subtask)             |      |
|        +----------->|  Generate IR -> Verify -> Iterate           |      |
|                     +---------------------------------------------+      |
|                              |                                           |
|                              v                                           |
|  +---------------------------------------------------------------+      |
|  |                 VERIFIER (100% of output)                      |      |
|  |  - ALL code comes from small model                             |      |
|  |  - ALL code goes through verification loop                     |      |
|  |  - 100% guaranteed correct (or fails with clear error)         |      |
|  +---------------------------------------------------------------+      |
|                              |                                           |
|                              v                                           |
|  +---------------------------------------------------------------+      |
|  |              TRAINING DATA COLLECTOR                           |      |
|  |  - Capture successful subtask->IR mappings                     |      |
|  |  - Over time, small model learns more patterns                 |      |
|  |  - LLM needed less and less                                    |      |
|  +---------------------------------------------------------------+      |
|                                                                          |
+-------------------------------------------------------------------------+
```

## Why LLM Should NOT Write IR

| LLM Writes IR Directly | LLM as Project Manager |
|------------------------|------------------------|
| No verification loop | 100% verified output |
| LLM might make mistakes | Mistakes caught by tests |
| Wastes LLM strength (reasoning) | Uses LLM for planning |
| Small model not utilized | Small model does all codegen |
| Slow (LLM inference) | Fast (local model) |

## Pattern Classifier

The pattern classifier is pure Rust code, not a trained model. It reuses the existing embedder:

```rust
// src/orchestration/classifier.rs
pub struct PatternClassifier {
    known_patterns: Vec<(Embedding, PatternInfo)>,  // Pre-computed from training data
    threshold: f32,  // e.g., 0.85
}

impl PatternClassifier {
    pub fn classify(&self, request: &str, embedder: &Embedder) -> Decision {
        let embedding = embedder.embed(request);
        let (best_match, similarity) = self.find_nearest(&embedding);

        if similarity >= self.threshold {
            Decision::Tier1 {
                pattern: best_match.clone(),
                confidence: similarity,
            }
        } else {
            Decision::Tier2 {
                reason: "Novel pattern, not in training data",
            }
        }
    }
}

pub enum Decision {
    Tier1 { pattern: PatternInfo, confidence: f32 },
    Tier2 { reason: &'static str },
}
```

## LLM Backend Abstraction

Support multiple LLM backends with a common interface:

```rust
// src/orchestration/backends/mod.rs

/// Trait for LLM backends (Tier 2)
pub trait LlmBackend: Send + Sync {
    fn name(&self) -> &str;
    fn decompose_task(&self, task: &str) -> Result<Vec<Subtask>>;
    fn supports_streaming(&self) -> bool;
}

/// A subtask that the small model can handle
pub struct Subtask {
    pub description: String,
    pub test_cases: Vec<TestCase>,
    pub depends_on: Vec<usize>,  // Indices of subtasks this depends on
}

/// Claude API backend
pub struct ClaudeBackend {
    api_key: String,
    model: String,  // claude-sonnet-4-20250514, etc.
}

/// Ollama local backend
pub struct OllamaBackend {
    host: String,
    model: String,  // llama3, codellama, etc.
}

/// OpenAI-compatible backend (for OpenRouter, vLLM, etc.)
pub struct OpenAiBackend {
    base_url: String,
    api_key: String,
    model: String,
}

/// Backend registry
pub struct BackendRegistry {
    backends: HashMap<String, Box<dyn LlmBackend>>,
    default: String,
}

impl BackendRegistry {
    pub fn get(&self, name: &str) -> Option<&dyn LlmBackend>;
    pub fn set_default(&mut self, name: &str);
}
```

## Two-Tier Orchestrator

```rust
// src/orchestration/mod.rs
pub struct TwoTierOrchestrator {
    // Tier 1: Fast local model
    tier1_model: OnnxModel,

    // Tier 2: LLM backends
    backends: BackendRegistry,

    // Decision making
    classifier: PatternClassifier,

    // Learning
    collector: TrainingDataCollector,

    // Compilation & execution
    compiler: CopyPatchCompiler,
    runtime: Runtime,
}

impl TwoTierOrchestrator {
    pub fn generate(&mut self, request: &str, tests: &[TestCase]) -> Result<Program> {
        match self.classifier.classify(request) {
            Decision::Tier1 { confidence, .. } => {
                // Try fast path first
                match self.try_tier1(request, tests) {
                    Ok(program) => return Ok(program),
                    Err(_) => {
                        // Escalate to Tier 2 on failure
                        self.try_tier2(request, tests)
                    }
                }
            }
            Decision::Tier2 { .. } => {
                // Go directly to LLM
                self.try_tier2(request, tests)
            }
        }
    }

    fn try_tier1(&mut self, task: &str, tests: &[TestCase]) -> Result<Program> {
        let mut errors = None;

        for _ in 0..100 {  // Max 100 fast iterations
            let ir = self.tier1_model.generate(task, errors.as_deref())?;
            let code = self.compiler.compile(&ir)?;

            match self.run_tests(&code, tests) {
                Ok(()) => return Ok(Program::new(ir)),
                Err(e) => errors = Some(e.to_string()),
            }
        }

        Err(OrchestratorError::Tier1Failed)
    }

    fn try_tier2(&mut self, task: &str, tests: &[TestCase]) -> Result<Program> {
        // Get default LLM backend
        let backend = self.backends.get_default();

        // LLM decomposes the task
        let subtasks = backend.decompose_task(task)?;

        let mut partial_program = Vec::new();

        for subtask in subtasks {
            // Small model handles each subtask
            let subtask_ir = self.try_tier1(&subtask.description, &subtask.test_cases)?;
            partial_program.extend(subtask_ir);

            // Record for future training
            self.collector.record_success(&subtask.description, &subtask_ir);
        }

        // Final verification
        let code = self.compiler.compile(&partial_program)?;
        self.run_tests(&code, tests)?;

        Ok(Program::new(partial_program))
    }
}
```

## Training Data Collection

Over time, the small model learns more patterns, reducing LLM usage:

```rust
// src/orchestration/collector.rs

/// Captures successful Tier 2 generations for training
pub struct TrainingDataCollector {
    output_path: PathBuf,
    buffer: Vec<TrainingExample>,
}

impl TrainingDataCollector {
    /// Called when Tier 2 produces verified IR
    pub fn record_success(&mut self, prompt: &str, ir: &[Instruction]) {
        self.buffer.push(TrainingExample {
            context: prompt.to_string(),
            partial_ir: vec![],
            error_feedback: None,
            expected_ir: ir.to_vec(),
        });

        // Flush periodically
        if self.buffer.len() >= 100 {
            self.flush();
        }
    }

    pub fn flush(&mut self) {
        // Write to JSONL file for next training run
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.output_path)
            .unwrap();
        for example in self.buffer.drain(..) {
            serde_json::to_writer(&file, &example).unwrap();
            writeln!(&file).unwrap();
        }
    }
}
```

## CLI Usage

```bash
# Use default backend (Tier 1 with auto-escalation)
$ nl agent --new "build REST API"

# Force Tier 2 with specific backend
$ nl agent --new "complex pattern" --backend claude
$ nl agent --new "offline mode" --backend ollama

# Configure backends
$ nl config set backends.claude.api_key "sk-..."
$ nl config set backends.ollama.host "http://localhost:11434"
$ nl config set backends.default "claude"

# List available backends
$ nl backends list
  claude   (API)     claude-sonnet-4-20250514
  ollama   (local)   llama3:latest
  openai   (API)     gpt-4
```

## LLM Prompting

The LLM receives a system prompt explaining its role:

```
You are a project manager for Neurlang, an AI coding system.
Your job is to break complex tasks into simple subtasks that
a small local model can handle.

Rules:
1. Each subtask should be simple enough to implement in <64 instructions
2. Each subtask should be independently testable
3. Provide test cases for each subtask
4. Use @"description" syntax for extension calls

Example:
User: "build REST API with user auth"

Your output:
{
  "subtasks": [
    {
      "description": "create HTTP server listening on port 8080",
      "tests": [{"input": "GET /health", "expected": "200 OK"}]
    },
    {
      "description": "parse JSON body from HTTP request",
      "tests": [{"input": "{\"name\":\"test\"}", "expected": "parse succeeds"}]
    },
    {
      "description": "hash password using SHA256",
      "tests": [{"input": "password123", "expected": "32-byte hash"}]
    },
    {
      "description": "store user in hashmap with email as key",
      "tests": [{"input": ["user@test.com", "hash"], "expected": "stored"}]
    },
    {
      "description": "return JSON response with status code",
      "tests": [{"input": {"status": 200, "body": {"ok": true}}, "expected": "valid response"}]
    }
  ]
}
```

## Benefits

1. **Verified Output**: All code goes through the verification loop
2. **Fast Iteration**: Local model runs at 30ms, LLM only for planning
3. **Continuous Learning**: Successful patterns are captured for training
4. **Offline Capability**: Ollama backend works without internet
5. **Cost Efficiency**: LLM used only when necessary

## Implementation Files

| File | Description | Priority |
|------|-------------|----------|
| `src/orchestration/mod.rs` | Two-tier orchestrator module | P0 |
| `src/orchestration/classifier.rs` | Pattern classifier for tier decision | P1 |
| `src/orchestration/backends/mod.rs` | LLM backend trait and registry | P0 |
| `src/orchestration/backends/claude.rs` | Claude API backend | P0 |
| `src/orchestration/backends/ollama.rs` | Ollama local backend | P1 |
| `src/orchestration/backends/openai.rs` | OpenAI-compatible backend | P2 |
| `src/orchestration/collector.rs` | Training data collection | P1 |

## See Also

- [Three-Layer Architecture](./three-layers.md) - Extension system design
- [RAG-Based Extension Resolution](./rag-extensions.md) - Dynamic extension lookup
- [Architecture Overview](./overview.md) - System design overview
