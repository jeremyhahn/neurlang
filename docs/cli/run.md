# nl run

Execute a program using JIT compilation or interpreter.

## Usage

```
nl run [OPTIONS] -i <INPUT>
```

## Options

| Option | Description |
|--------|-------------|
| `-i, --input <FILE>` | Input file (assembly `.asm` or binary `.nlb`) |
| `--interp` | Use interpreter instead of JIT |
| `-s, --stats` | Show execution statistics |
| `--max-instr <N>` | Max instructions (default: 1000000) |
| `-w, --workers <N>` | Number of worker threads (default: 0 = single-threaded) |
| `--strategy <TYPE>` | Worker strategy: `auto`, `reuseport`, `shared` |

## Examples

```bash
# Run with JIT (default)
nl run -i program.asm

# Run with interpreter
nl run -i program.asm --interp

# Show execution statistics
nl run -i program.asm --stats

# Limit instruction count (for safety)
nl run -i program.asm --interp --max-instr 10000

# Run server with 4 workers
nl run -i server.nl --workers 4

# Explicit worker strategy
nl run -i server.nl --workers 4 --strategy reuseport
```

## Multi-Worker Mode

For server workloads, use `--workers` to spawn multiple worker threads:

```bash
nl run -i server.nl --workers 4
```

### Worker Strategies

| Strategy | Description | Platform |
|----------|-------------|----------|
| `auto` | Auto-detect best strategy | All |
| `reuseport` | SO_REUSEPORT kernel load-balancing | Linux, macOS, FreeBSD |
| `shared` | Shared listener (workers compete) | All platforms |

## Output

JIT execution:

```
Loaded 8 instructions
Program halted
R0 = 55

Statistics:
  Code size: 48 bytes
  Compile time: 0.00ms (2.15us)
  Execution time: 123ns
```

Interpreter execution:

```
Loaded 8 instructions
Program halted
R0 = 55

Statistics:
  Instructions: 47
  Time: 1.234us
  IPS: 38.09M
```

## See Also

- [asm](asm.md) - Assemble text to binary
- [compile](compile.md) - Compile to standalone native code
- [repl](repl.md) - Interactive REPL
