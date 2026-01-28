# nl spec

Show IR specification (opcodes, registers).

## Usage

```
nl spec [OPTIONS]
```

## Options

| Option | Description |
|--------|-------------|
| `--opcodes` | Show opcode list |
| `--registers` | Show register list |
| `--all` | Show everything |

## Examples

```bash
# Show opcodes
nl spec --opcodes

# Show registers
nl spec --registers

# Show all
nl spec --all
```

## Output

```
Neurlang Opcodes (24 total):
=========================

Arithmetic/Logic:
  0x00  ALU        Add, Sub, And, Or, Xor, Shl, Shr, Sar
  0x01  ALUI       Same with immediate
  0x02  MULDIV     Mul, Div, Mod, MulH

Memory:
  0x03  LOAD       Load 8/16/32/64 (auto bounds-check)
  0x04  STORE      Store 8/16/32/64 (auto bounds-check)
  0x05  ATOMIC     CAS, Xchg, Add, And, Or, Xor, Min, Max

Control Flow:
  0x06  BRANCH     Conditional branches
  0x07  JUMP       Unconditional jump
  0x08  CALL       Function call
  0x09  RET        Return from function

...

Registers (32):
  r0-r27: General purpose
  r28:    Frame pointer (fp)
  r29:    Stack pointer (sp)
  r30:    Link register (lr)
  r31:    Zero register (always 0)
```

## See Also

- [asm](asm.md) - Assemble programs
- [disasm](disasm.md) - Disassemble binaries
