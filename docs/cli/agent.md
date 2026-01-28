# neurlang agent

Autonomous AI agent for iterative code generation with session management.

## Usage

```
neurlang agent [OPTIONS] [PROMPT]
```

## Options

| Option | Description |
|--------|-------------|
| `--new <PROMPT>` | Start a new agent session with the given task |
| `--continue <SESSION-ID>` | Continue an existing session with a new request |
| `--resume <SESSION-ID>` | Resume a session after crash or interruption |
| `--list` | List all available sessions |
| `--interactive` | Start interactive REPL mode |
| `--context <FILE>` | Provide context file (e.g., requirements.txt) |
| `--tests <FILE>` | Custom test cases in JSON format |
| `--max-iter <N>` | Maximum iteration limit (default: 100) |
| `--verbose` | Show iteration progress during generation |

## Commands

### Start a New Session

Create a new agent session with a task description:

```bash
# Basic usage
neurlang agent --new "task description"

# With context file
neurlang agent --new "implement API endpoints" --context requirements.txt

# With custom iteration limit
neurlang agent --new "build sorting algorithm" --max-iter 500
```

### Continue an Existing Session

Add enhancements or modifications to a previous session:

```bash
neurlang agent --continue a1b2c3d4 "add input validation"
neurlang agent --continue a1b2c3d4 "optimize for performance"
```

### Resume After Crash

Recover a session from the last checkpoint:

```bash
neurlang agent --resume a1b2c3d4
```

### List Sessions

View all available sessions:

```bash
neurlang agent --list
```

Output:

```
Session ID   Created              Status      Task
─────────────────────────────────────────────────────────────────────
a1b2c3d4     2026-01-24 10:30    complete    build a REST API with user auth
e5f6g7h8     2026-01-23 15:45    active      implement caching layer
i9j0k1l2     2026-01-22 09:15    crashed     parse JSON config files
```

### Interactive REPL Mode

Enter an interactive session for continuous development:

```bash
neurlang agent --interactive
```

## Session Storage

Sessions are persisted to disk for durability and recovery.

### Storage Location

```
~/.neurlang/sessions/{uuid}/
```

### Session Files

| File | Description |
|------|-------------|
| `session.json` | Session metadata and configuration |
| `index.bin` | Generation index and state |
| `functions/*.nl` | Generated function definitions |
| `current.nl` | Current working program |

### Checkpointing

- Automatic checkpoint after each generation cycle
- Resume from last checkpoint on crash or interruption
- Manual checkpoint available in interactive mode

## Example Workflow

### Building a REST API

```bash
$ neurlang agent --new "build a REST API with user auth"
[session: a1b2c3d4]
[generating... 147 iterations, 5.8 seconds]
[checkpointing... done]
✓ Generated: api_server

$ neurlang agent --continue a1b2c3d4 "add rate limiting"
[loading session...]
[generating... 23 iterations, 0.9 seconds]
✓ Updated: api_server

$ neurlang agent --resume a1b2c3d4
[loaded at iteration 170]
Ready for next request.
```

### Using Context Files

```bash
$ cat requirements.txt
- RESTful endpoints for user CRUD
- JWT authentication
- PostgreSQL database
- Input validation

$ neurlang agent --new "implement the API" --context requirements.txt
[session: m3n4o5p6]
[loading context: requirements.txt]
[generating... 312 iterations, 12.4 seconds]
✓ Generated: user_api
```

### Custom Test Cases

```bash
$ cat tests.json
{
  "test_cases": [
    {"input": "GET /users", "expected": 200},
    {"input": "POST /users {}", "expected": 400},
    {"input": "GET /users/1", "expected": 200}
  ]
}

$ neurlang agent --new "REST API" --tests tests.json
[session: q7r8s9t0]
[loaded 3 test cases]
[generating... 89 iterations, 3.2 seconds]
[running tests... 3/3 passed]
✓ Generated: rest_api
```

### Verbose Mode

```bash
$ neurlang agent --new "factorial function" --verbose
[session: u1v2w3x4]
[iteration 1] exploring: recursive approach
[iteration 2] exploring: iterative approach
[iteration 3] refining: iterative loop
[iteration 4] optimizing: register usage
[iteration 5] validating: edge cases
[generating... 5 iterations, 0.2 seconds]
✓ Generated: factorial
```

## Output

### Successful Generation

```
[session: a1b2c3d4]
[generating... 147 iterations, 5.8 seconds]
[checkpointing... done]
✓ Generated: api_server

Files created:
  ~/.neurlang/sessions/a1b2c3d4/functions/api_server.nl
  ~/.neurlang/sessions/a1b2c3d4/current.nl
```

### Session Continuation

```
[loading session...]
[previous iterations: 147]
[generating... 23 iterations, 0.9 seconds]
[total iterations: 170]
✓ Updated: api_server
```

### Error Recovery

```
[session: i9j0k1l2]
[loading checkpoint at iteration 45]
[recovered state: functions/parser.nl]
Ready for next request.
```

## Exit Codes

| Code | Description |
|------|-------------|
| `0` | Success |
| `1` | General error |
| `2` | Invalid session ID |
| `3` | Session not found |
| `4` | Checkpoint corrupted |
| `5` | Max iterations exceeded |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `NEURLANG_SESSIONS_DIR` | Override default session storage location |
| `NEURLANG_MAX_ITER` | Default maximum iterations |
| `NEURLANG_VERBOSE` | Enable verbose output by default |

## See Also

- [generate](generate.md) - Single-shot code generation
- [chat](chat.md) - Interactive chat mode
- [run](run.md) - Execute generated programs
- [repl](repl.md) - Interactive REPL
