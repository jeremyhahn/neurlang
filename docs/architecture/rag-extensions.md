# RAG-Based Extension Resolution

This document explains how Neurlang uses Retrieval-Augmented Generation (RAG) to dynamically resolve extension calls without requiring model retraining.

## The Problem

Traditional approaches require the model to know exact extension IDs:

```asm
; Model must memorize that 190 = http_get
ext.call r0, 190, r1, r2
```

This creates problems:
1. **Training cost**: +$30-50 per extension to train
2. **Inflexibility**: Adding extensions requires retraining
3. **User extensions**: Can't use custom extensions without retraining

## The Solution: Intent-Based Resolution

Instead of training the model on specific IDs, train it to **describe what it needs**:

```asm
; Model describes intent in natural language
ext.call r0, @"fetch URL content", r1, r2
             |
             | RAG lookup at assembly time
             v
; Resolved to actual extension ID
ext.call r0, 190, r1, r2
```

## How It Works

### Assembly-Time Resolution

```
                    ASSEMBLY TIME                    COMPILE TIME              RUNTIME
                    -------------                    ------------              -------

Model output:       EXT.CALL @"parse CSV", r0
                         |
                         v
RAG lookup:         search("parse CSV")
                         |
                         v
Match:              csv-parser.parse (ID: 500)
                         |
                         v
Resolved IR:        EXT.CALL 500, r0  ---------->  Copy-and-patch   ----> call ext_dispatch(500)
                                                   emits call stub          |
                                                                            v
                                                                       REGISTRY.get(500)
                                                                            |
                                                                            v
                                                                       csv_parser::parse(args)
                                                                       (pre-loaded .so)
```

### The RAG Pipeline

```
+-------------------------------------------------------------------------+
|                        RAG Resolution Pipeline                           |
+-------------------------------------------------------------------------+
|                                                                          |
|  1. INDEXING (one-time, at extension installation):                      |
|     +-----------------------------------------------------------+        |
|     | Extension: csv-parser                                      |        |
|     | Functions:                                                 |        |
|     |   - parse: "Parse CSV string into records"                 |        |
|     |   - format: "Convert records to CSV string"                |        |
|     +-----------------------------------------------------------+        |
|                              |                                           |
|                              v                                           |
|     +-----------------------------------------------------------+        |
|     | Embed descriptions using embedding model                   |        |
|     | "Parse CSV string into records" -> [0.12, -0.34, ...]     |        |
|     +-----------------------------------------------------------+        |
|                              |                                           |
|                              v                                           |
|     +-----------------------------------------------------------+        |
|     | Store in vector index: ~/.neurlang/extensions/index.bin    |        |
|     +-----------------------------------------------------------+        |
|                                                                          |
|  2. RESOLUTION (at assembly time):                                       |
|     +-----------------------------------------------------------+        |
|     | Model emits: @"parse CSV data"                             |        |
|     +-----------------------------------------------------------+        |
|                              |                                           |
|                              v                                           |
|     +-----------------------------------------------------------+        |
|     | Embed query: "parse CSV data" -> [0.11, -0.35, ...]        |        |
|     +-----------------------------------------------------------+        |
|                              |                                           |
|                              v                                           |
|     +-----------------------------------------------------------+        |
|     | Cosine similarity search in vector index                   |        |
|     | Top match: "Parse CSV string into records" (sim=0.94)      |        |
|     +-----------------------------------------------------------+        |
|                              |                                           |
|                              v                                           |
|     +-----------------------------------------------------------+        |
|     | Return: csv-parser.parse, ID=500                           |        |
|     +-----------------------------------------------------------+        |
|                                                                          |
+-------------------------------------------------------------------------+
```

## Syntax

### Symbolic Names (Identifier Syntax)

For stdlib and well-known extensions, use `@name`:

```asm
ext.call r0, @vec_new, r0, r0       ; Vec operations
ext.call r0, @hashmap_insert, r1, r2 ; HashMap operations
ext.call r0, @string_concat, r1, r2  ; String operations
```

These are resolved via hardcoded lookup table (no RAG needed):

```rust
// src/ir/assembler.rs
fn resolve_symbolic_name(name: &str) -> Option<u32> {
    match name {
        "vec_new" => Some(100),
        "vec_push" => Some(102),
        "hashmap_new" => Some(120),
        "string_concat" => Some(143),
        // ...
        _ => None  // Fall back to RAG
    }
}
```

### Intent Descriptions (Quoted String Syntax)

For bundled and user extensions, use `@"description"`:

```asm
ext.call r0, @"parse JSON string", r1, r2
ext.call r0, @"make HTTP GET request", r1, r2
ext.call r0, @"calculate SHA256 hash", r1, r2
ext.call r0, @"read file contents", r1, r2
```

These are resolved via RAG at assembly time.

## Implementation

### RAG Resolver Module

The actual implementation uses keyword-based similarity search with `ext_ids` as the single source of truth for extension IDs:

```rust
// src/ir/rag_resolver.rs
use crate::runtime::extensions::ext_ids;

pub struct RagResolver {
    /// Extension database: description keywords -> extension info
    extensions: Vec<ExtensionEntry>,
    /// Name lookup: exact name -> extension ID
    name_lookup: HashMap<String, u32>,
}

impl RagResolver {
    pub fn new() -> Self {
        let mut resolver = Self {
            extensions: Vec::new(),
            name_lookup: HashMap::new(),
        };
        resolver.register_bundled_extensions();
        resolver
    }

    /// Register bundled extensions using ext_ids as single source of truth
    fn register_bundled_extensions(&mut self) {
        // All IDs come from ext_ids - no hardcoded values!
        self.register(ext_ids::JSON_PARSE, "json_parse", "parse JSON string",
                      &["json", "parse", "decode"], 1);
        self.register(ext_ids::HTTP_GET, "http_get", "make HTTP GET request",
                      &["http", "get", "fetch", "url"], 1);
        self.register(ext_ids::SQLITE_OPEN, "sqlite_open", "open SQLite database",
                      &["sqlite", "open", "database"], 1);
        // ... all extensions registered with ext_ids constants
    }

    /// Resolve an intent description to an extension
    pub fn resolve(&self, intent: &str) -> Option<ResolvedExtension> {
        // 1. Try exact name lookup first
        if let Some(&id) = self.name_lookup.get(&intent.to_lowercase()) {
            return self.get_by_id(id);
        }

        // 2. Score by keyword overlap
        let intent_words: Vec<&str> = intent.to_lowercase().split_whitespace().collect();
        let mut best_score = 0.0;
        let mut best_match: Option<&ExtensionEntry> = None;

        for ext in &self.extensions {
            let score = self.compute_similarity(&intent_words, ext);
            if score > best_score {
                best_score = score;
                best_match = Some(ext);
            }
        }

        // 3. Require minimum threshold (0.3)
        if best_score >= 0.3 {
            best_match.map(|ext| ResolvedExtension { ... })
        } else {
            None
        }
    }
}

pub struct ResolvedExtension {
    pub id: u32,
    pub name: String,
    pub input_count: usize,
    pub description: String,
}
```

### Single Source of Truth

**Critical**: All extension IDs are defined in `src/runtime/extensions/mod.rs` in the `ext_ids` module:

```rust
// src/runtime/extensions/mod.rs
pub mod ext_ids {
    // Crypto (1-99)
    pub const SHA256: u32 = 1;
    pub const HMAC_SHA256: u32 = 2;

    // JSON (170-189)
    pub const JSON_PARSE: u32 = 170;
    pub const JSON_STRINGIFY: u32 = 171;

    // HTTP (190-209)
    pub const HTTP_GET: u32 = 190;

    // SQLite (260-279)
    pub const SQLITE_OPEN: u32 = 260;
    pub const SQLITE_PREPARE: u32 = 264;

    // UUID (330-339)
    pub const UUID_V4: u32 = 330;

    // ... all other extensions
}
```

This ensures consistency between:
- **Assembly-time resolution** (RagResolver imports ext_ids)
- **Runtime execution** (ExtensionRegistry uses ext_ids)
- **Test mocking** (`@mock: extension_name=value` resolved via RagResolver)

### Assembler Integration

```rust
// src/ir/assembler.rs
fn parse_ext_call(&mut self) -> Result<Instruction> {
    // Parse: ext.call rd, <target>, rs1, rs2
    let rd = self.parse_register()?;
    self.expect_comma()?;

    let ext_id = if self.peek_char() == '@' {
        self.advance(); // consume '@'

        if self.peek_char() == '"' {
            // Intent syntax: @"description"
            let intent = self.parse_string()?;
            self.rag_resolver.resolve(&intent)?.extension_id
        } else {
            // Symbolic syntax: @name
            let name = self.parse_identifier()?;
            self.resolve_symbolic_name(&name)?
        }
    } else {
        // Numeric ID
        self.parse_u32()?
    };

    let rs1 = self.parse_register()?;
    let rs2 = self.parse_register()?;

    Ok(Instruction::ExtCall { ext_id, rd, rs1, rs2 })
}
```

### Extension Index Format

```rust
// ~/.neurlang/extensions/index.bin
pub struct ExtensionIndex {
    pub version: u32,
    pub embedding_dim: u32,
    pub entries: Vec<ExtensionEntry>,
}

pub struct ExtensionEntry {
    pub extension_id: u32,
    pub function_name: String,
    pub description: String,
    pub embedding: Vec<f32>,  // 384-dim for all-MiniLM-L6-v2
    pub signature: FunctionSignature,
}
```

## Type Checking

RAG resolution includes type checking to catch mismatches early:

```rust
// Resolution returns the function signature
let resolved = rag_resolver.resolve("parse JSON string")?;
// resolved.signature = { inputs: [String], output: JsonHandle }

// Assembler can verify argument types if known
if let Some(rs1_type) = self.register_types.get(&rs1) {
    if rs1_type != &resolved.signature.inputs[0] {
        return Err(AssemblerError::TypeMismatch {
            expected: resolved.signature.inputs[0].clone(),
            found: rs1_type.clone(),
        });
    }
}
```

## Fallback Chain

Resolution follows a fallback chain:

```
1. Check hardcoded stdlib names (@vec_new, @hashmap_insert, etc.)
   |
   | Not found
   v
2. Check RAG index for bundled extensions
   |
   | Not found or low confidence
   v
3. Check RAG index for user extensions
   |
   | Not found
   v
4. Return error with suggestions

Error: Could not resolve "@parse Excel file"
  Did you mean:
    - @"parse CSV file" (csv-parser, ID: 500)
    - @"parse JSON string" (json, ID: 170)
  Or install an Excel parser:
    nl extension --add github.com/user/excel-parser
```

## Benefits

### Zero Training Cost for Extensions

| Approach | Training Cost |
|----------|---------------|
| Train on each extension | +$30-50 per extension |
| RAG-based resolution | $0 |

### Self-Documenting Code

```asm
; Which is clearer?
ext.call r0, 500, r1, r2              ; What does 500 do?
ext.call r0, @"parse CSV data", r1, r2 ; Self-documenting
```

### Future-Proof

New extensions can be added without any model changes:

1. User installs extension: `nl extension --add github.com/user/new-parser`
2. Extension descriptions are indexed
3. Model can immediately use it via intent description

## Performance

| Operation | Latency |
|-----------|---------|
| Embed intent | ~1ms |
| Vector search | ~0.5ms |
| Total RAG resolution | ~1.5ms |

This happens once per assembly, not in the hot path. The compiled code uses numeric IDs directly.

## See Also

- [Three-Layer Architecture](./three-layers.md) - Overall extension architecture
- [Bundled Extensions](../extensions/bundled.md) - Built-in extension reference
- [Creating Extensions](../extensions/creating.md) - How to publish extensions
