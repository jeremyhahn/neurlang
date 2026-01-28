# Creating Extensions

This guide explains how to create and publish Neurlang extensions.

## Overview

Extensions in Neurlang follow a Go-style pattern where the **import path IS the source URL**. No central registry is needed.

```bash
# Install from any git host
nl extension --add github.com/user/csv-parser
nl extension --add gitlab.com/team/utilities
nl extension --add github.com/user/csv-parser@v1.2.0  # pinned version
```

## Quick Start

### Create a Local Extension

```bash
# Create a new extension
nl extension --new my-utils

# This creates:
# ~/.neurlang/extensions/local/my-utils/
# ├── neurlang.json     # Manifest
# └── main.nl           # Entry point
```

### Write Your Extension Code

```asm
; ~/.neurlang/extensions/local/my-utils/main.nl

; Export: add two numbers
add:
    add r0, r0, r1
    ret

; Export: multiply two numbers
multiply:
    muldiv.mul r0, r0, r1
    ret

; Export: factorial
factorial:
    mov r1, 1           ; result = 1
.loop:
    beq r0, zero, .done
    muldiv.mul r1, r1, r0
    alui.sub r0, r0, 1
    b .loop
.done:
    mov r0, r1
    ret
```

### Update the Manifest

```json
{
  "name": "my-utils",
  "version": "0.1.0",
  "description": "My utility functions",
  "entry": "main.nl",
  "exports": [
    {
      "name": "add",
      "description": "Add two numbers together",
      "inputs": ["int", "int"],
      "output": "int"
    },
    {
      "name": "multiply",
      "description": "Multiply two numbers",
      "inputs": ["int", "int"],
      "output": "int"
    },
    {
      "name": "factorial",
      "description": "Calculate factorial of a number",
      "inputs": ["int"],
      "output": "int"
    }
  ]
}
```

### Test Your Extension

```bash
# Load and test
nl extension --load ~/.neurlang/extensions/local/my-utils/

# The extension is now available in the agent
nl agent --new "use my-utils to calculate factorial of 5"
```

### Publish to Git

```bash
cd ~/.neurlang/extensions/local/my-utils

# Initialize git repo
git init
git add .
git commit -m "Initial release"

# Push to GitHub
git remote add origin https://github.com/you/my-utils
git push -u origin main

# Tag a release
git tag v1.0.0
git push --tags
```

Now anyone can install your extension:

```bash
nl extension --add github.com/you/my-utils@v1.0.0
```

---

## Extension Types

### Pure Neurlang Extensions

Written in Neurlang assembly, compiled by the loader:

```
my-extension/
├── neurlang.json
├── main.nl           # Entry point
├── utils.nl          # Additional files
└── data/
    └── config.json   # Static data
```

### Rust FFI Extensions

Complex operations implemented in Rust:

```
my-extension/
├── neurlang.json
├── Cargo.toml
└── src/
    └── lib.rs
```

```rust
// src/lib.rs
use neurlang_ext::*;

#[neurlang_export]
pub fn parse_csv(input: String) -> Vec<Vec<String>> {
    // Implementation
}

#[neurlang_export(description = "Format data as CSV with custom delimiter")]
pub fn format_csv(data: Vec<Vec<String>>, delimiter: char) -> String {
    // Implementation
}
```

When installed, Rust extensions are compiled with `cargo build --release`.

---

## Manifest Reference

### Required Fields

```json
{
  "name": "my-extension",
  "version": "1.0.0"
}
```

### All Fields

```json
{
  "name": "my-extension",
  "version": "1.0.0",
  "description": "What this extension does",
  "entry": "main.nl",
  "exports": [
    {
      "name": "function_name",
      "description": "What this function does",
      "inputs": ["int", "string"],
      "output": "buffer"
    }
  ],
  "dependencies": [
    "github.com/other/lib@v1.0.0"
  ],
  "neurlang_version": ">=0.1.0",
  "authors": ["Your Name <you@example.com>"],
  "license": "MIT",
  "repository": "https://github.com/you/my-extension",
  "keywords": ["csv", "parsing", "data"]
}
```

### Field Descriptions

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Extension name (no path prefix) |
| `version` | Yes | Semantic version (x.y.z) |
| `description` | No | Brief description for RAG indexing |
| `entry` | No | Entry point file (default: main.nl) |
| `exports` | No | Exported functions with signatures |
| `dependencies` | No | Other extensions this depends on |
| `neurlang_version` | No | Required Neurlang version constraint |
| `authors` | No | List of author strings |
| `license` | No | SPDX license identifier |
| `repository` | No | Source repository URL |
| `keywords` | No | Search keywords |

### Export Types

| Type | Description |
|------|-------------|
| `int` | 64-bit signed integer |
| `uint` | 64-bit unsigned integer |
| `float` | 64-bit floating point |
| `bool` | Boolean (0 or 1) |
| `string` | UTF-8 string (handle) |
| `buffer` | Byte buffer (handle) |
| `array` | Array (handle) |
| `json` | JSON value (handle) |
| `handle` | Opaque handle |

---

## Best Practices

### Write Good Descriptions

The description is used for RAG indexing. Good descriptions help the model find your extension:

```json
{
  "name": "parse_csv",
  "description": "Parse CSV text into rows and columns with header support"
}
```

Bad:
```json
{
  "description": "CSV parser"  // Too vague
}
```

### Include Multiple Keywords

Add variations to help RAG matching:

```json
{
  "description": "Parse CSV/comma-separated values text into structured data",
  "keywords": ["csv", "comma separated", "spreadsheet", "table", "rows", "columns"]
}
```

### Version Carefully

Follow semantic versioning:
- MAJOR: Breaking changes
- MINOR: New features, backward compatible
- PATCH: Bug fixes, backward compatible

```json
{
  "version": "1.2.3"
}
```

### Test Before Publishing

```bash
# Test loading
nl extension --load ./my-extension

# Test in agent
nl agent --new "test my extension"

# Run examples
nl run examples/demo.nl
```

### Document Usage

Include a README.md:

```markdown
# My Extension

## Installation

```bash
nl extension --add github.com/you/my-extension
```

## Usage

```asm
ext.call r0, @"parse CSV data", r1, r0
```

## Examples

See the `examples/` directory.
```

---

## Directory Structure

```
~/.neurlang/extensions/
├── local/                    # User-created extensions
│   └── my-utils/
│       ├── neurlang.json
│       └── main.nl
├── cache/                    # Git-installed extensions
│   └── github.com/
│       └── user/
│           ├── csv-parser/           # Latest version
│           │   ├── neurlang.json
│           │   └── main.nl
│           └── csv-parser@v1.2.0/   # Pinned version
│               ├── neurlang.json
│               └── main.nl
└── extensions.lock           # Version lock file
```

---

## Dependencies

### Declaring Dependencies

```json
{
  "dependencies": [
    "github.com/stdlib/strings@v1.0.0",
    "github.com/user/json-utils@^2.0.0"
  ]
}
```

### Version Constraints

| Syntax | Meaning |
|--------|---------|
| `1.2.3` | Exact version |
| `^1.2.3` | Compatible with 1.x.x (>=1.2.3, <2.0.0) |
| `~1.2.3` | Approximately 1.2.x (>=1.2.3, <1.3.0) |
| `>=1.2.3` | Minimum version |
| `<2.0.0` | Maximum version |
| `>=1.0.0,<2.0.0` | Range |

### Dependency Resolution

When installing, Neurlang resolves the dependency tree:

```bash
$ nl extension --add github.com/user/my-app

Installing github.com/user/my-app@v1.0.0
  → Requires: github.com/stdlib/strings@^1.0.0
  → Requires: github.com/user/json-utils@^2.0.0
    → Requires: github.com/stdlib/strings@^1.2.0

Resolved:
  github.com/stdlib/strings@v1.2.0
  github.com/user/json-utils@v2.1.0
  github.com/user/my-app@v1.0.0

Installing 3 extensions...
Done.
```

---

## Rust FFI Extensions

For complex operations, implement in Rust:

### Cargo.toml

```toml
[package]
name = "my-extension"
version = "1.0.0"

[lib]
crate-type = ["cdylib"]

[dependencies]
neurlang-ext = "0.1"  # Extension SDK
```

### lib.rs

```rust
use neurlang_ext::prelude::*;

/// Parse CSV text into a 2D array
#[neurlang_export]
pub fn parse_csv(input: String) -> Result<Vec<Vec<String>>, ExtError> {
    let mut result = Vec::new();
    for line in input.lines() {
        let row: Vec<String> = line.split(',')
            .map(|s| s.trim().to_string())
            .collect();
        result.push(row);
    }
    Ok(result)
}

/// Format 2D array as CSV
#[neurlang_export]
pub fn format_csv(data: Vec<Vec<String>>) -> Result<String, ExtError> {
    let lines: Vec<String> = data.iter()
        .map(|row| row.join(","))
        .collect();
    Ok(lines.join("\n"))
}
```

### Manifest for Rust Extension

```json
{
  "name": "csv-parser",
  "version": "1.0.0",
  "type": "rust",
  "exports": [
    {
      "name": "parse_csv",
      "description": "Parse CSV text into rows and columns",
      "inputs": ["string"],
      "output": "array"
    },
    {
      "name": "format_csv",
      "description": "Format 2D array as CSV text",
      "inputs": ["array"],
      "output": "string"
    }
  ]
}
```

---

## Publishing Workflow

```bash
# 1. Create extension
nl extension --new csv-parser
cd ~/.neurlang/extensions/local/csv-parser

# 2. Write code and manifest
vim main.nl
vim neurlang.json

# 3. Test locally
nl extension --load .
nl agent --new "parse this CSV: a,b,c"

# 4. Initialize git
git init
git add .
git commit -m "Initial release v1.0.0"

# 5. Create GitHub repo and push
gh repo create csv-parser --public
git remote add origin https://github.com/you/csv-parser
git push -u origin main

# 6. Tag release
git tag v1.0.0
git push --tags

# 7. Test installation
nl extension --add github.com/you/csv-parser@v1.0.0
```

---

## See Also

- [Bundled Extensions](./bundled.md) - Built-in extension reference
- [Three-Layer Architecture](../architecture/three-layers.md) - Extension system design
- [RAG-Based Extension Resolution](../architecture/rag-extensions.md) - How discovery works
- [Extension CLI](../cli/extension.md) - CLI reference
