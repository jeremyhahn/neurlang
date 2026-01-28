# nl asm

Assemble text source to binary IR.

## Usage

```
nl asm [OPTIONS] -i <INPUT>
```

## Options

| Option | Description |
|--------|-------------|
| `-i, --input <FILE>` | Input assembly file (required) |
| `-o, --output <FILE>` | Output binary file (default: `<input>.nlb`) |
| `-d, --disasm` | Show disassembly after assembly |

## Examples

```bash
# Assemble a file
nl asm -i program.asm

# Specify output path
nl asm -i program.asm -o program.nlb

# Verify assembly with disassembly output
nl asm -i program.asm --disasm
```

## Output

```
Assembled 12 instructions (64 bytes)
Wrote 80 bytes to program.nlb
```

With `--disasm`:

```
Assembled 12 instructions (64 bytes)
Wrote 80 bytes to program.nlb

Disassembly:
  0000:  mov r0, 10
  0008:  mov r1, 1
  0010:  add r2, r0, r1
  0014:  halt
```

## Assembly Syntax

```asm
; Comments start with semicolon
mov r0, 10        ; Load immediate value
mov r1, 5
add r2, r0, r1    ; r2 = r0 + r1
halt              ; Stop execution

; Labels
loop:
  sub r0, r0, r1
  bne r0, zero, loop
```

## See Also

- [disasm](disasm.md) - Disassemble binary to text
- [run](run.md) - Execute a program
- [compile](compile.md) - Compile to native code
