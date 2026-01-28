# nl disasm

Disassemble binary IR to text.

## Usage

```
nl disasm [OPTIONS] -i <INPUT>
```

## Options

| Option | Description |
|--------|-------------|
| `-i, --input <FILE>` | Input binary file (required) |
| `--offsets` | Show byte offsets |
| `--bytes` | Show raw bytes |

## Examples

```bash
# Basic disassembly
nl disasm -i program.nlb

# Show offsets
nl disasm -i program.nlb --offsets

# Show raw bytes with offsets
nl disasm -i program.nlb --bytes --offsets
```

## Output

Basic output:

```
mov r0, 10
mov r1, 1
add r2, r0, r1
halt
```

With `--offsets`:

```
0000:  mov r0, 10
0008:  mov r1, 1
0010:  add r2, r0, r1
0014:  halt
```

With `--bytes --offsets`:

```
0000:  50 80 00 00 0a 00 00 00  mov r0, 10
0008:  50 80 08 00 01 00 00 00  mov r1, 1
0010:  01 00 10 00 00 08 00 00  add r2, r0, r1
0018:  1f 00 00 00 00 00 00 00  halt
```

## See Also

- [asm](asm.md) - Assemble text to binary
- [spec](spec.md) - Show IR specification
