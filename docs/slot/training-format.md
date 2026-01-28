# Slot Training Data Format

This document describes the training data format for slot-based code generation models.

## Overview

Slot training data teaches the model to fill individual slots based on:
- Slot type (one of 20 types)
- Input parameters (context-specific)
- Expected output (Neurlang assembly)

The model learns patterns for each slot type independently, enabling parallel generation.

## File Format

Training data is stored in JSONL format (one JSON object per line):

```
train/
  slot_training.jsonl           # Combined training data
  slot_training_patternmatch.jsonl    # Per-type files (optional)
  slot_training_responsebuilder.jsonl
  ...
```

## Example Records

### PatternMatch

```json
{
  "slot_type": "PatternMatch",
  "input": {
    "pattern": "HELO {domain}",
    "input_reg": "r0",
    "captures": [
      {
        "name": "domain",
        "output_reg": "r3",
        "capture_type": "Word"
      }
    ],
    "match_label": "helo_match",
    "no_match_label": "try_next"
  },
  "output": "load.b r1, [r0]\nmov r2, 0x48\nbne r1, r2, try_next\nload.b r1, [r0 + 1]\nmov r2, 0x45\nbne r1, r2, try_next\nload.b r1, [r0 + 2]\nmov r2, 0x4C\nbne r1, r2, try_next\nload.b r1, [r0 + 3]\nmov r2, 0x4F\nbne r1, r2, try_next\nload.b r1, [r0 + 4]\nmov r2, 0x20\nbne r1, r2, try_next\naddi r3, r0, 5\nb helo_match"
}
```

### ResponseBuilder

```json
{
  "slot_type": "ResponseBuilder",
  "input": {
    "template": "250 Hello {domain}\r\n",
    "variables": {
      "domain": "r3"
    },
    "output_reg": "r6",
    "length_reg": "r7"
  },
  "output": "mov r0, r6\nmov r1, 0x32\nstore.b r1, [r0]\nmov r1, 0x35\nstore.b r1, [r0 + 1]\nmov r1, 0x30\nstore.b r1, [r0 + 2]\nmov r1, 0x20\nstore.b r1, [r0 + 3]\nmov r1, 0x48\nstore.b r1, [r0 + 4]\nmov r1, 0x65\nstore.b r1, [r0 + 5]\nmov r1, 0x6C\nstore.b r1, [r0 + 6]\nmov r1, 0x6C\nstore.b r1, [r0 + 7]\nmov r1, 0x6F\nstore.b r1, [r0 + 8]\nmov r1, 0x20\nstore.b r1, [r0 + 9]\nmov r8, r0\naddi r8, r8, 10\nmov r9, r3\ncopy_domain:\nload.b r1, [r9]\nbeqz r1, copy_done\nstore.b r1, [r8]\naddi r8, r8, 1\naddi r9, r9, 1\nb copy_domain\ncopy_done:\nmov r1, 0x0D\nstore.b r1, [r8]\naddi r8, r8, 1\nmov r1, 0x0A\nstore.b r1, [r8]\naddi r8, r8, 1\nsub r7, r8, r0"
}
```

### StateCheck

```json
{
  "slot_type": "StateCheck",
  "input": {
    "state_reg": "r20",
    "valid_states": ["STATE_MAIL_FROM", "STATE_RCPT_TO"],
    "ok_label": "state_ok",
    "error_label": "state_error"
  },
  "output": "mov r1, STATE_MAIL_FROM\nbeq r20, r1, state_ok\nmov r1, STATE_RCPT_TO\nbeq r20, r1, state_ok\nb state_error\nstate_ok:"
}
```

### SendResponse

```json
{
  "slot_type": "SendResponse",
  "input": {
    "socket_reg": "r10",
    "buffer_reg": "r6",
    "length_reg": "r7"
  },
  "output": "mov r0, r10\nmov r1, r6\nmov r2, r7\nmov r3, 0\next.call @\"socket_send\", r0, r1, r2, r3"
}
```

### ExtensionCall

```json
{
  "slot_type": "ExtensionCall",
  "input": {
    "extension": "parse JSON string",
    "args": ["r0", "r1"],
    "result_reg": "r2"
  },
  "output": "ext.call @\"parse JSON string\", r0, r1\nmov r2, r0"
}
```

## Schema Definition

### Common Fields

All records have:

| Field | Type | Description |
|-------|------|-------------|
| `slot_type` | string | One of 20 slot type names |
| `input` | object | Type-specific input parameters |
| `output` | string | Neurlang assembly (newline-separated) |

### Optional Fields

| Field | Type | Description |
|-------|------|-------------|
| `metadata.source` | string | Source file/example |
| `metadata.difficulty` | int | Complexity (1-5) |
| `metadata.instruction_count` | int | Number of instructions |
| `unit_test` | object | Per-slot verification test |

### Unit Test Format

```json
{
  "slot_type": "PatternMatch",
  "input": { ... },
  "output": "...",
  "unit_test": {
    "setup": "mov r0, test_buffer\nmov r1, 0",
    "input_values": {
      "test_buffer": "HELO example.com\r\n"
    },
    "expected": {
      "branch_taken": "helo_match",
      "registers": {
        "r3": "test_buffer + 5"
      }
    }
  }
}
```

## Generation Sources

Training data comes from verified sources only:

### 1. Extracted from lib/*.nl (Generated from Rust)

```bash
nl-datagen slot-extract --lib-dir lib --output train/slot_training_lib.jsonl
```

Extracts patterns from generated stdlib functions.

### 2. Extracted from examples/*.nl (Hand-Written)

```bash
nl-datagen slot-extract --examples-dir examples --output train/slot_training_examples.jsonl
```

Extracts patterns from verified example programs.

### 3. Protocol Spec Expansion

```bash
nl-datagen slot-expand --specs-dir specs/protocols --output train/slot_training_protocols.jsonl
```

Generates all slot combinations from protocol specifications.

## Training Data Statistics

Target: 5,000+ examples with good coverage:

| Slot Type | Target Examples | Notes |
|-----------|-----------------|-------|
| PatternMatch | 800+ | High variation in patterns |
| PatternSwitch | 200+ | Multi-case dispatch |
| ResponseBuilder | 600+ | Template variations |
| StringCompare | 150+ | Common utility |
| StringCopy | 150+ | Common utility |
| IntToString | 100+ | Number formatting |
| StringToInt | 100+ | Number parsing |
| RangeCheck | 100+ | Validation |
| StateCheck | 300+ | State machines |
| StateTransition | 150+ | Simple but frequent |
| LoopUntil | 200+ | Loop patterns |
| SendResponse | 300+ | I/O operations |
| ReadUntil | 250+ | I/O operations |
| ReadNBytes | 150+ | I/O operations |
| ExtensionCall | 400+ | Extension usage |
| ValidationHook | 200+ | Validation patterns |
| ErrorResponse | 250+ | Error handling |
| BufferWrite | 200+ | Data manipulation |
| BufferRead | 200+ | Data manipulation |
| **Total** | **5,000+** | |

## Quality Requirements

### Correctness

All training examples must:
1. Compile without errors
2. Pass unit tests (if provided)
3. Use correct register conventions

### Diversity

For each slot type:
1. Vary register assignments
2. Vary label names
3. Vary parameter values
4. Include edge cases

### Consistency

1. Follow Neurlang assembly conventions
2. Use consistent whitespace
3. Include comments for complex sequences

## Validation

### Schema Validation

```bash
nl-datagen validate --input train/slot_training.jsonl
```

Checks:
- Required fields present
- Valid slot_type
- Input matches slot type schema
- Output is valid assembly

### Compilation Test

```bash
nl-datagen compile-test --input train/slot_training.jsonl
```

Attempts to assemble each output.

### Unit Test Execution

```bash
nl-datagen unit-test --input train/slot_training.jsonl
```

Runs unit tests where provided.

## Deduplication

Training data should be deduplicated to avoid overfitting:

```bash
nl-datagen dedupe --input train/slot_training.jsonl --output train/slot_training_deduped.jsonl
```

Deduplication rules:
1. Exact output match -> keep one
2. Same input parameters -> keep one (prefer verified source)
3. Same pattern structure with different literals -> keep both

## Data Augmentation

Optional augmentation for more variety:

```bash
nl-datagen augment --input train/slot_training.jsonl --output train/slot_training_augmented.jsonl
```

Augmentation strategies:
1. Register renaming (r3 -> r4, etc.)
2. Label renaming (match -> found, etc.)
3. Literal variation (ASCII codes, addresses)

## See Also

- [Slot Architecture Overview](./README.md)
- [Slot Types Reference](./slot-types.md)
- [CLI: nl-datagen](../cli/datagen.md)
