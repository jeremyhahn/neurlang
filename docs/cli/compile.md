# nl compile

Compile to standalone native code.

## Usage

```
nl compile [OPTIONS] -i <INPUT> -o <OUTPUT>
```

## Options

| Option | Description |
|--------|-------------|
| `-i, --input <FILE>` | Input file (required) |
| `-o, --output <FILE>` | Output file (required) |
| `--format <FMT>` | Output format: `raw` or `elf` (default: raw) |

## Examples

```bash
# Compile to raw machine code
nl compile -i program.asm -o program.bin

# Compile to ELF executable (Linux)
nl compile -i program.asm -o program --format elf
chmod +x program
./program
```

## Output Formats

| Format | Description | Executable |
|--------|-------------|------------|
| `raw` | Raw x86-64 machine code | No |
| `elf` | Linux ELF executable | Yes (Linux only) |

## Output

```
Compiled 156 bytes to program.bin
```

## See Also

- [asm](asm.md) - Assemble text to binary
- [run](run.md) - Execute with JIT
