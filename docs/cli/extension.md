# neurlang extension

Go-style package management for Neurlang extensions.

## Overview

The extension system provides Go-style package management where the import path IS the source URL. Extensions can be installed from any git repository or created locally.

## Usage

```
nl extension [OPTIONS]
```

## Options

| Option | Description |
|--------|-------------|
| `--add <URL>` | Install extension from git URL |
| `--new <NAME>` | Create a new local extension |
| `--remove <PATH>` | Remove an installed extension |
| `--list` | List all installed extensions |
| `--load <FILE>` | Load extension from file for testing |
| `--info <PATH>` | Show extension details |

## Commands

### Install from Git

Install extensions directly from any git repository:

```bash
# Install latest version
nl extension --add github.com/user/csv-parser

# Install specific version
nl extension --add github.com/user/csv-parser@v1.2.0

# Install from other hosts
nl extension --add gitlab.com/team/utilities
```

### Create Local Extension

Create a new extension in your local development directory:

```bash
nl extension --new my-utils
```

This creates:
```
~/.neurlang/extensions/local/my-utils/
├── neurlang.json     # Extension manifest
└── main.nl           # Entry point
```

### List Installed Extensions

View all installed extensions:

```bash
nl extension --list
```

Output:
```
Installed Extensions
====================

IMPORT PATH                              VERSION    SOURCE
---------------------------------------- ---------- ----------
local/my-utils                           0.1.0      local
github.com/user/csv-parser@v1.2.0       1.2.0      git
```

### Remove Extension

Uninstall an extension:

```bash
nl extension --remove local/my-utils
nl extension --remove github.com/user/csv-parser@v1.2.0
```

### Load for Testing

Test an extension without installing it:

```bash
nl extension --load ./my-extension/

# Or load a single file
nl extension --load ./utils.nl
```

### Show Extension Info

View detailed information about an extension:

```bash
nl extension --info local/my-utils
```

Output:
```
Extension: my-utils
  Version: 0.1.0
  Path: /home/user/.neurlang/extensions/local/my-utils
  Entry: main.nl
  Source: local
  Exports:
    - parse_csv - Parse CSV data into records
    - format_json - Format data as JSON
```

## Directory Structure

Extensions are stored in `~/.neurlang/extensions/`:

```
~/.neurlang/extensions/
├── local/                    # User-created extensions
│   ├── my-utils/
│   │   ├── neurlang.json     # Manifest
│   │   └── main.nl           # Entry point
│   └── parser/
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

## Extension Manifest

Each extension must have a `neurlang.json` manifest:

```json
{
  "name": "csv-parser",
  "version": "1.2.0",
  "description": "Parse and generate CSV files",
  "entry": "main.nl",
  "exports": [
    {
      "name": "parse",
      "inputs": ["string"],
      "output": "buffer",
      "description": "Parse CSV string to records"
    },
    {
      "name": "format",
      "inputs": ["buffer"],
      "output": "string",
      "description": "Format records as CSV"
    }
  ],
  "dependencies": [
    "github.com/other/string-utils@v1.0.0"
  ],
  "neurlang_version": ">=0.1.0",
  "authors": ["Author Name <author@example.com>"],
  "license": "MIT",
  "repository": "https://github.com/user/csv-parser"
}
```

### Manifest Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Extension name (without path prefix) |
| `version` | Yes | Semantic version (e.g., "1.2.0") |
| `description` | No | Brief description |
| `entry` | No | Entry point file (default: "main.nl") |
| `exports` | No | Exported functions |
| `dependencies` | No | Other extensions this depends on |
| `neurlang_version` | No | Required Neurlang version |
| `authors` | No | List of authors |
| `license` | No | SPDX license identifier |
| `repository` | No | Source repository URL |

### Export Definition

Each export describes a function that can be called from other code:

```json
{
  "name": "function_name",
  "inputs": ["int", "string", "buffer"],
  "output": "int",
  "description": "What this function does"
}
```

**Supported Types:**
- `int` - 64-bit integer
- `float` - Floating point
- `string` - String (pointer + length)
- `buffer` - Byte buffer (pointer + length)
- `bool` - Boolean
- `array` - Array of a type

## Writing Extensions

### Basic Extension

```asm
; main.nl - Simple math utilities

; Export: add two numbers
add:
    add r0, r0, r1
    ret

; Export: multiply two numbers
multiply:
    mul r0, r0, r1
    ret

; Export: factorial
factorial:
    mov r1, 1           ; result = 1
.loop:
    beq r0, zero, .done
    mul r1, r1, r0
    subi r0, r0, 1
    b .loop
.done:
    mov r0, r1
    ret
```

### Using Extensions

Extensions can be imported in your code:

```asm
; Import extension
@import "github.com/user/math-utils"

; Use exported function
main:
    mov r0, 5
    mov r1, 3
    call add          ; From imported extension
    halt
```

## Versioning

### Semantic Versioning

Extensions use semantic versioning (semver):
- `1.0.0` - Exact version
- `^1.0.0` - Compatible with 1.x.x
- `>=1.2.0` - Minimum version
- `~1.2.3` - Approximately 1.2.x

### Version Pinning

Pin specific versions for reproducibility:

```bash
# Install specific version
nl extension --add github.com/user/lib@v1.2.3

# Lock file tracks installed versions
cat ~/.neurlang/extensions/extensions.lock
```

## Examples

### Create and Publish

```bash
# Create local extension
nl extension --new string-utils
cd ~/.neurlang/extensions/local/string-utils

# Edit main.nl with your code
vim main.nl

# Edit manifest
vim neurlang.json

# Test it
nl extension --load .

# Initialize git and publish
git init
git add .
git commit -m "Initial release"
git remote add origin https://github.com/you/string-utils
git push -u origin main
git tag v1.0.0
git push --tags
```

### Install in Another Project

```bash
# Install your published extension
nl extension --add github.com/you/string-utils@v1.0.0

# Use it
nl generate "use string-utils to parse input"
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `NEURLANG_EXTENSIONS_DIR` | Override default extensions directory |

## Exit Codes

| Code | Description |
|------|-------------|
| `0` | Success |
| `1` | General error |
| `2` | Extension not found |
| `3` | Git clone failed |
| `4` | Invalid manifest |
| `5` | Already exists |

## See Also

- [agent](agent.md) - Interactive AI agent
- [generate](generate.md) - Code generation
- [run](run.md) - Execute programs
