# Three-Layer Extension Architecture

This document explains Neurlang's three-layer architecture for code reuse and extension management.

## Overview

Neurlang uses a three-layer architecture to balance training costs, flexibility, and performance:

```
+-------------------------------------------------------------------------+
|                         RUNTIME                                          |
|  (The execution engine - not trainable, just code)                       |
+-------------------------------------------------------------------------+
|  - Copy-and-patch compiler (5us compilation)                             |
|  - Register machine (32 registers)                                       |
|  - Extension dispatcher (ID -> function lookup)                          |
|  - Memory management                                                     |
|  - RAG index for extension resolution                                    |
+-------------------------------------------------------------------------+

+-------------------------------------------------------------------------+
|                         STDLIB                                           |
|  (Trained into model - keep MINIMAL)                                     |
+-------------------------------------------------------------------------+
|  Only what you can't build real programs without:                        |
|                                                                          |
|  - Vec (dynamic arrays)          ~13 operations                          |
|  - HashMap (key-value)           ~10 operations                          |
|  - String (text manipulation)    ~19 operations                          |
|                                                                          |
|  Total: ~42 operations to train                                          |
|  Training cost: ~$50-100 additional                                      |
+-------------------------------------------------------------------------+

+-------------------------------------------------------------------------+
|                      BUNDLED EXTENSIONS                                  |
|  (Ships with neurlang, discovered via RAG, NOT trained)                  |
+-------------------------------------------------------------------------+
|  Common enough to include, but resolved dynamically:                     |
|                                                                          |
|  - JSON parsing/stringify                                                |
|  - HTTP client (GET/POST/PUT/DELETE)                                     |
|  - Crypto (SHA256, HMAC, AES, signatures)                                |
|  - File I/O (read/write/exists)                                          |
|  - SQLite (local database)                                               |
|  - Regex (pattern matching)                                              |
|  - Date/Time (parsing, formatting)                                       |
|  - Environment variables                                                 |
|  - UUID generation                                                       |
|  - Base64 encoding/decoding                                              |
|                                                                          |
|  Training cost: $0 (RAG resolves them)                                   |
+-------------------------------------------------------------------------+

+-------------------------------------------------------------------------+
|                      USER EXTENSIONS                                     |
|  (Installed by user, discovered via RAG)                                 |
+-------------------------------------------------------------------------+
|  - Domain-specific APIs (Stripe, AWS, etc.)                              |
|  - Custom business logic                                                 |
|  - Industry-specific formats (HL7, FIX, etc.)                            |
|  - User's own libraries                                                  |
|                                                                          |
|  Training cost: $0                                                       |
+-------------------------------------------------------------------------+
```

## Why This Split?

### Stdlib (Trained)

The stdlib contains operations used in **literally every program**:

- **Vec**: Dynamic arrays are fundamental to any non-trivial program
- **HashMap**: Key-value storage is ubiquitous
- **String**: Text manipulation is required for any I/O

The model should emit these "instinctively" without RAG overhead. Since there are only ~42 operations, training is cheap (~$50-100).

### Bundled Extensions (RAG)

These are common but not universal. Instead of training the model on specific extension IDs, we train it to **describe what it needs**:

```
TRAINED APPROACH (requires retraining per extension):
  Model outputs: EXT.CALL 190, r0, r1    ; Must know 190 = http_get
  Cost: +$30-50 per extension

RAG APPROACH (no retraining):
  Model outputs: EXT.CALL @"fetch URL content", r0, r1
                          |
  RAG matches "fetch URL content" -> http_get extension -> ID 190
                          |
  Assembler emits: EXT.CALL 190, r0, r1
  Cost: $0
```

### Training Cost Comparison

| Approach | Base Training | Per Extension | 50 Extensions |
|----------|---------------|---------------|---------------|
| Train on each extension | $200 | +$30-50 | $1,700-2,700 |
| RAG-based intent | $200 | $0 | $200 |

With RAG-based intent resolution, the model learns ONE skill: "describe what you need". The cost is fixed regardless of how many extensions exist.

## Extension ID Ranges

| Category | ID Range | Resolution |
|----------|----------|------------|
| **Crypto** | 1-99 | Hard-coded (security-critical) |
| **Stdlib (Vec)** | 100-112 | Trained |
| **Stdlib (HashMap)** | 120-129 | Trained |
| **Stdlib (String)** | 140-158 | Trained |
| **Stdlib (JSON)** | 170-181 | Trained or RAG |
| **Bundled** | 200-499 | RAG |
| **User** | 500+ | RAG |

## Stdlib Operations

### Vec Operations (100-112)

| ID | Name | Description |
|----|------|-------------|
| 100 | vec_new | Create empty vector |
| 101 | vec_with_cap | Create with capacity |
| 102 | vec_push | Add element to end |
| 103 | vec_pop | Remove and return last |
| 104 | vec_get | Get element by index |
| 105 | vec_set | Set element by index |
| 106 | vec_len | Get length |
| 107 | vec_capacity | Get capacity |
| 108 | vec_clear | Remove all elements |
| 109 | vec_free | Deallocate vector |
| 111 | vec_insert | Insert at index |
| 112 | vec_remove | Remove at index |

### HashMap Operations (120-129)

| ID | Name | Description |
|----|------|-------------|
| 120 | hashmap_new | Create empty map |
| 121 | hashmap_insert | Insert key-value pair |
| 122 | hashmap_get | Get value by key |
| 123 | hashmap_remove | Remove key-value pair |
| 124 | hashmap_contains | Check if key exists |
| 125 | hashmap_len | Get number of entries |
| 126 | hashmap_clear | Remove all entries |
| 127 | hashmap_free | Deallocate map |
| 128 | hashmap_keys | Get all keys as Vec |
| 129 | hashmap_values | Get all values as Vec |

### String Operations (140-158)

| ID | Name | Description |
|----|------|-------------|
| 140 | string_new | Create empty string |
| 141 | string_from_bytes | Create from byte buffer |
| 142 | string_len | Get byte length |
| 143 | string_concat | Concatenate two strings |
| 144 | string_substr | Extract substring |
| 145 | string_find | Find substring index |
| 146 | string_replace | Replace substring |
| 147 | string_split | Split by delimiter |
| 148 | string_trim | Remove whitespace |
| 149 | string_to_upper | Convert to uppercase |
| 150 | string_to_lower | Convert to lowercase |
| 151 | string_starts_with | Check prefix |
| 152 | string_ends_with | Check suffix |
| 153 | string_to_bytes | Convert to byte Vec |
| 154 | string_free | Deallocate string |
| 155 | string_parse_int | Parse as integer |
| 156 | string_parse_float | Parse as float |
| 157 | string_from_int | Convert int to string |
| 158 | string_from_float | Convert float to string |

## Bundled Extensions

| Extension | Why | Key Operations |
|-----------|-----|----------------|
| **json** | Universal data format | parse, stringify, get, set |
| **http** | Web is everywhere | get, post, put, delete |
| **crypto** | Security critical | sha256, hmac, aes, sign, verify |
| **fs** | Basic system access | read, write, exists, mkdir |
| **sqlite** | Local persistence | open, query, execute |
| **regex** | Text processing | match, find_all, replace |
| **datetime** | Common need | now, parse, format, add |
| **env** | Configuration | get, set, all |
| **uuid** | Unique IDs | v4, v7, parse |
| **base64** | Encoding | encode, decode |
| **url** | Web basics | parse, encode, decode |
| **log** | Debugging | debug, info, warn, error |

## User Extensions

Users can install extensions from any git repository:

```bash
# Install from GitHub
nl extension --add github.com/user/csv-parser
nl extension --add github.com/user/csv-parser@v1.2.0

# Create local extension
nl extension --new my-utils
```

Once installed, extensions are indexed for RAG resolution:

1. Extension manifest is parsed for function descriptions
2. Descriptions are embedded using the embedding model
3. Embeddings are stored in `~/.neurlang/extensions/index.bin`
4. When the model emits `@"description"`, RAG finds the best match

## Implementation

### Extension Registration Flow

```
Extension Installation (one-time):
+-------------------------------------------------------------------------+
|  $ neurlang extension add github.com/user/csv-parser                    |
|                                                                          |
|  1. Download source from git                                             |
|  2. If Rust extension:                                                   |
|     -> cargo build --release                                             |
|     -> Produces: libcsv_parser.so                                        |
|  3. If Neurlang extension:                                               |
|     -> Store .nl files                                                   |
|  4. Register in ~/.neurlang/extensions/registry.json:                    |
|     { "csv-parser": { "id": 500, "path": "...", ... } }                  |
|  5. Index description for RAG:                                           |
|     embed("parse CSV files with headers") -> vector                      |
+-------------------------------------------------------------------------+

Runtime Startup:
+-------------------------------------------------------------------------+
|  neurlang run my_program.nl                                              |
|                                                                          |
|  1. Load stdlib extensions (built-in)                                    |
|     -> ID 100-199: Vec, HashMap, String                                  |
|  2. Load bundled extensions                                              |
|     -> ID 200-499: JSON, HTTP, Crypto, etc.                              |
|  3. Load user extensions from registry.json                              |
|     -> dlopen("libcsv_parser.so")                                        |
|     -> Register functions at ID 500+                                     |
|  4. Now runtime has all extensions ready                                 |
|  5. Execute program - EXT.CALL dispatches to loaded functions            |
+-------------------------------------------------------------------------+
```

### RAG Resolution at Assembly Time

```
Model output:       EXT.CALL @"parse CSV", r0
                         |
                         v
RAG lookup:         search("parse CSV")
                         |
                         v
Match:              csv-parser.parse (ID: 500)
                         |
                         v
Resolved IR:        EXT.CALL 500, r0  -------->  Copy-and-patch   ----> Native code
                                                 emits call stub
```

## Final Training Breakdown

```
Model learns:
+-- 32 core opcodes                    (base, ~$100-150)
+-- ~42 stdlib operations              (trained, ~$50-100)
+-- Intent emission skill              (one pattern, ~$20-30)
+-- Total: ~$170-280 one-time cost

Model does NOT learn:
+-- JSON operations                    (RAG resolves)
+-- HTTP operations                    (RAG resolves)
+-- Any other extension                (RAG resolves)
+-- Future extensions                  (RAG resolves)
```

## See Also

- [RAG-Based Extension Resolution](./rag-extensions.md) - How dynamic resolution works
- [Bundled Extensions](../extensions/bundled.md) - Full API reference
- [Creating Extensions](../extensions/creating.md) - How to publish your own
- [Training Costs](../training/costs.md) - Detailed cost breakdown
