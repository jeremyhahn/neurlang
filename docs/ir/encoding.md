# Binary Encoding Reference

Detailed specification of the Neurlang binary instruction format.

## Instruction Formats

### Fixed 32-bit Base Format

```
┌───────────────────────────────────────────────────────────────────┐
│                    Base Instruction (32 bits)                     │
└───────────────────────────────────────────────────────────────────┘

  ┌──────────┬──────────┬──────────┬──────────┬─────────────────────┐
  │  Opcode  │   Mode   │    Rd    │   Rs1    │     Rs2/Imm16       │
  │  5 bits  │  3 bits  │  5 bits  │  5 bits  │      14 bits        │
  └──────────┴──────────┴──────────┴──────────┴─────────────────────┘
   31     27  26     24  23     19  18     14  13                   0

  Bit layout:
  [31:27] = opcode (0x00-0x1F, 32 opcodes)
  [26:24] = mode (operation variant)
  [23:19] = rd (destination register)
  [18:14] = rs1 (source register 1)
  [13:0]  = rs2/imm (source register 2 OR immediate)
```

### Extended 64-bit Format (with 32-bit immediate)

```
┌───────────────────────────────────────────────────────────────────┐
│                  Extended Instruction (64 bits)                   │
└───────────────────────────────────────────────────────────────────┘

  First Word (32 bits):
  ┌──────────┬──────────┬──────────┬──────────┬────────┬────────────┐
  │  Opcode  │   Mode   │    Rd    │   Rs1    │ Ext=1  │   Flags    │
  │  5 bits  │  3 bits  │  5 bits  │  5 bits  │ 1 bit  │  13 bits   │
  └──────────┴──────────┴──────────┴──────────┴────────┴────────────┘

  Second Word (32 bits):
  ┌───────────────────────────────────────────────────────────────────┐
  │                       32-bit Immediate                            │
  └───────────────────────────────────────────────────────────────────┘

  The Ext bit (bit 13) indicates presence of immediate word.
```

## Opcode Encoding

### Opcode Table

| Opcode | Binary | Hex | Name | Description |
|--------|--------|-----|------|-------------|
| 0 | 00000 | 0x00 | ALU | Arithmetic/logic (reg-reg) |
| 1 | 00001 | 0x01 | ALUI | Arithmetic/logic (reg-imm) |
| 2 | 00010 | 0x02 | MULDIV | Multiply/divide |
| 3 | 00011 | 0x03 | LOAD | Memory load |
| 4 | 00100 | 0x04 | STORE | Memory store |
| 5 | 00101 | 0x05 | ATOMIC | Atomic operations |
| 6 | 00110 | 0x06 | BRANCH | Conditional branch |
| 7 | 00111 | 0x07 | CALL | Function call |
| 8 | 01000 | 0x08 | RET | Return |
| 9 | 01001 | 0x09 | JUMP | Unconditional jump |
| 10 | 01010 | 0x0A | CAP_NEW | Create capability |
| 11 | 01011 | 0x0B | CAP_RESTRICT | Restrict capability |
| 12 | 01100 | 0x0C | CAP_QUERY | Query capability |
| 13 | 01101 | 0x0D | SPAWN | Create task |
| 14 | 01110 | 0x0E | JOIN | Wait for task |
| 15 | 01111 | 0x0F | CHAN | Channel operations |
| 16 | 10000 | 0x10 | FENCE | Memory fence |
| 17 | 10001 | 0x11 | YIELD | Cooperative yield |
| 18 | 10010 | 0x12 | TAINT | Mark tainted |
| 19 | 10011 | 0x13 | SANITIZE | Remove taint |
| 20 | 10100 | 0x14 | MOV | Move/load immediate |
| 21 | 10101 | 0x15 | TRAP | System trap |
| 22 | 10110 | 0x16 | NOP | No operation |
| 23 | 10111 | 0x17 | HALT | Stop execution |

## Mode Encoding

### ALU Mode (3 bits)

| Mode | Binary | Operation |
|------|--------|-----------|
| 0 | 000 | ADD |
| 1 | 001 | SUB |
| 2 | 010 | AND |
| 3 | 011 | OR |
| 4 | 100 | XOR |
| 5 | 101 | SHL |
| 6 | 110 | SHR |
| 7 | 111 | SAR |

### MULDIV Mode (2 bits)

| Mode | Binary | Operation |
|------|--------|-----------|
| 0 | 00 | MUL |
| 1 | 01 | DIV |
| 2 | 10 | MOD |
| 3 | 11 | MULH |

### Memory Width (2 bits)

| Mode | Binary | Width |
|------|--------|-------|
| 0 | 00 | Byte (8-bit) |
| 1 | 01 | Half (16-bit) |
| 2 | 10 | Word (32-bit) |
| 3 | 11 | Double (64-bit) |

### Branch Condition (3 bits)

| Mode | Binary | Condition |
|------|--------|-----------|
| 0 | 000 | EQ (equal) |
| 1 | 001 | NE (not equal) |
| 2 | 010 | LT (less than, signed) |
| 3 | 011 | LE (less or equal, signed) |
| 4 | 100 | GT (greater than, signed) |
| 5 | 101 | GE (greater or equal, signed) |
| 6 | 110 | Reserved |
| 7 | 111 | Always (unconditional) |

### Atomic Mode (3 bits)

| Mode | Binary | Operation |
|------|--------|-----------|
| 0 | 000 | CAS |
| 1 | 001 | XCHG |
| 2 | 010 | ADD |
| 3 | 011 | AND |
| 4 | 100 | OR |
| 5 | 101 | XOR |
| 6 | 110 | MIN |
| 7 | 111 | MAX |

### Channel Mode (2 bits)

| Mode | Binary | Operation |
|------|--------|-----------|
| 0 | 00 | CREATE |
| 1 | 01 | SEND |
| 2 | 10 | RECV |
| 3 | 11 | CLOSE |

### Fence Mode (2 bits)

| Mode | Binary | Ordering |
|------|--------|----------|
| 0 | 00 | Acquire |
| 1 | 01 | Release |
| 2 | 10 | SeqCst |
| 3 | 11 | Reserved |

## Register Encoding

### General Purpose Registers

| Register | Encoding | Convention |
|----------|----------|------------|
| r0 | 00000 | Return value / Arg 0 |
| r1 | 00001 | Arg 1 |
| r2-r7 | 00010-00111 | Args 2-7 |
| r8-r15 | 01000-01111 | Temporaries |
| r16-r23 | 10000-10111 | Saved |
| r24-r29 | 11000-11101 | Saved |
| r30 | 11110 | Frame pointer |
| r31 | 11111 | Stack pointer |

### Special Register: Zero

Register `r0` when used as source reads as zero (like RISC-V x0).
When used as destination, the value is written normally.

## Encoding Examples

### ADD r5, r3, r7

```
Opcode: ALU (00000)
Mode:   ADD (000)
Rd:     r5  (00101)
Rs1:    r3  (00011)
Rs2:    r7  (00111)

Binary: 00000 000 00101 00011 00000000000111
        [opc][mod][rd  ][rs1 ][    rs2      ]

Hex: 0x00 0x28 0x60 0x07
```

### ADDI r0, r1, 42

```
Opcode: ALUI (00001)
Mode:   ADD  (000)
Rd:     r0   (00000)
Rs1:    r1   (00001)
Imm:    42   (0x2A)

Extended format (64 bits):
Word 1: 00001 000 00000 00001 1 0000000000000
Word 2: 00000000 00000000 00000000 00101010

Hex: 0x08 0x00 0x22 0x00 0x00 0x00 0x00 0x2A
```

### LOAD.D r2, [r5 + 16]

```
Opcode: LOAD (00011)
Mode:   Double (11)
Rd:     r2 (00010)
Rs1:    r5 (00101)
Offset: 16

Binary: 00011 011 00010 00101 00000000010000

Hex: 0x1B 0x22 0x80 0x10
```

### BEQ r3, r4, -8

```
Opcode: BRANCH (00110)
Mode:   EQ (000)
Rs1:    r3 (00011)
Rs2:    r4 (00100)
Offset: -8 (signed, 14-bit)

Binary: 00110 000 xxxxx 00011 11111111111000
        (rd unused for branch)

Hex: 0x30 0x00 0x63 0xF8
```

### HALT

```
Opcode: HALT (10111)
Mode:   0 (000)
Rd:     0 (00000)
Rs1:    0 (00000)
Rs2:    0 (00000000000000)

Binary: 10111 000 00000 00000 00000000000000

Hex: 0xB8 0x00 0x00 0x00
```

## Byte Order

All multi-byte values use **little-endian** encoding:

```
Value: 0x12345678
Memory: [78] [56] [34] [12]
        low              high
```

## Decoding Algorithm

```rust
pub fn decode(bytes: &[u8]) -> Instruction {
    let word = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

    let opcode = (word >> 27) & 0x1F;
    let mode = (word >> 24) & 0x07;
    let rd = (word >> 19) & 0x1F;
    let rs1 = (word >> 14) & 0x1F;
    let ext = (word >> 13) & 0x01;

    if ext == 1 {
        // Extended format with 32-bit immediate
        let imm = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        Instruction::new_imm(opcode, mode, rd, rs1, imm)
    } else {
        let rs2_imm = word & 0x3FFF;
        Instruction::new(opcode, mode, rd, rs1, rs2_imm)
    }
}
```
