# Execution Model

Neurlang supports two execution modes: JIT compilation and interpretation.

## Execution Decision Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Execution Mode Selection                          │
└─────────────────────────────────────────────────────────────────────┘

                    ┌─────────────────┐
                    │  Load Program   │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │ Instructions    │
                    │   < 1000 ?      │
                    └────────┬────────┘
                             │
              ┌──────────────┴──────────────┐
              │ Yes                         │ No
              ▼                             ▼
     ┌─────────────────┐          ┌─────────────────┐
     │  Interpreter    │          │  JIT Compiler   │
     │                 │          │                 │
     │ • Zero compile  │          │ • <5μs compile  │
     │ • All 32 opcodes│          │ • Native speed  │
     │ • Good for debug│          │ • Direct call   │
     └─────────────────┘          └─────────────────┘
```

## Register File

```
┌─────────────────────────────────────────────────────────────────────┐
│                      Register File (32 x 64-bit)                     │
└─────────────────────────────────────────────────────────────────────┘

  ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐
  │ r0  │ r1  │ r2  │ r3  │ r4  │ r5  │ r6  │ r7  │  General Purpose
  │ a0  │ a1  │ a2  │ a3  │ a4  │ a5  │     │     │  (Arguments)
  └─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘

  ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐
  │ r8  │ r9  │ r10 │ r11 │ r12 │ r13 │ r14 │ r15 │  General Purpose
  └─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘

  ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐
  │ sp  │ fp  │ lr  │ pc  │ csp │ cfp │ --- │ zero│  Special Purpose
  └─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘
    │     │     │     │     │     │           │
    │     │     │     │     │     │           └── Always 0 (read-only)
    │     │     │     │     │     └── Capability frame pointer
    │     │     │     │     └── Capability stack pointer
    │     │     │     └── Program counter (read-only)
    │     │     └── Link register (return address)
    │     └── Frame pointer
    └── Stack pointer
```

## JIT Calling Convention

```
┌─────────────────────────────────────────────────────────────────────┐
│                     JIT Function ABI                                 │
└─────────────────────────────────────────────────────────────────────┘

  Compiled function signature:
    fn(regfile: *mut u64) -> u64

  Register mapping (x86-64):
  ┌───────────────┬────────────────────────────────────────────────┐
  │ CPU Register  │ Purpose                                        │
  ├───────────────┼────────────────────────────────────────────────┤
  │ RDI           │ Pointer to register file (32 x u64)            │
  │ RAX, RCX      │ Scratch registers for operations               │
  │ RSI           │ Next instruction pointer (for branches)        │
  │ Return (RAX)  │ Execution result / halt sentinel               │
  └───────────────┴────────────────────────────────────────────────┘

  Memory layout during execution:
  ┌────────────────────────────────────────────────────────────────┐
  │ RDI points here:                                               │
  │ ┌──────────────────────────────────────────────────────────┐  │
  │ │ r0 │ r1 │ r2 │ ... │ r15 │ sp │ fp │ lr │ pc │...│ zero │  │
  │ │ +0 │ +8 │+16 │     │+120 │+128│+136│+144│+152│   │ +248 │  │
  │ └──────────────────────────────────────────────────────────┘  │
  └────────────────────────────────────────────────────────────────┘
```

## Interpreter Dispatch

```
┌─────────────────────────────────────────────────────────────────────┐
│                   Interpreter Main Loop                              │
└─────────────────────────────────────────────────────────────────────┘

  loop {
      ┌─────────────────┐
      │ Fetch instr[PC] │
      └────────┬────────┘
               │
               ▼
      ┌─────────────────┐
      │ Decode opcode   │
      └────────┬────────┘
               │
               ▼
      ┌─────────────────────────────────────────┐
      │            Match opcode                  │
      ├───────┬───────┬───────┬───────┬────────┤
      │  ALU  │  MEM  │ BRANCH│ SYSTEM│  ...   │
      └───┬───┴───┬───┴───┬───┴───┬───┴────────┘
          │       │       │       │
          ▼       ▼       ▼       ▼
      ┌───────────────────────────────────────┐
      │         Execute operation              │
      └───────────────────────────────────────┘
               │
               ▼
      ┌─────────────────┐
      │ Update PC       │────▶ Continue or Halt
      └─────────────────┘
  }
```

## Control Flow Operations

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Branch Execution                                 │
└─────────────────────────────────────────────────────────────────────┘

  Instruction: BRANCH.cond rs1, rs2, offset

  ┌─────────────────────────────────────────────────────────────────┐
  │                                                                 │
  │   1. Load rs1 value                                             │
  │   2. Load rs2 value                                             │
  │   3. Compare based on condition (EQ, NE, LT, LE, GT, GE)       │
  │   4. If condition true:                                         │
  │        PC = PC + offset (relative jump)                         │
  │      Else:                                                       │
  │        PC = PC + 1 (fall through)                               │
  │                                                                 │
  └─────────────────────────────────────────────────────────────────┘

  Example:
  ┌──────┬────────┬────────┬────────┬────────────────────────────────┐
  │ Addr │ Instr  │ Before │ After  │ Notes                          │
  ├──────┼────────┼────────┼────────┼────────────────────────────────┤
  │ 0x00 │ mov r0,5│ --    │ r0=5   │                                │
  │ 0x08 │ mov r1,0│ --    │ r1=0   │                                │
  │ 0x10 │ beq r0,r1,done│ --│ --   │ 5 != 0, fall through          │
  │ 0x18 │ sub r0,r0,1│ r0=5│ r0=4  │                                │
  │ 0x20 │ b loop │  --    │ PC=0x10│ Jump back                      │
  └──────┴────────┴────────┴────────┴────────────────────────────────┘
```

## Memory Access

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Memory Operation Flow                            │
└─────────────────────────────────────────────────────────────────────┘

  LOAD.width rd, [rs1 + offset]

       ┌──────────────────────────────────────────┐
       │ 1. Calculate address = rs1 + offset      │
       └─────────────────┬────────────────────────┘
                         │
                         ▼
       ┌──────────────────────────────────────────┐
       │ 2. Check capability (implicit)           │
       │    - Valid tag?                          │
       │    - Within bounds?                      │
       │    - Has read permission?                │
       └─────────────────┬────────────────────────┘
                         │
              ┌──────────┴──────────┐
              │ Pass               │ Fail
              ▼                    ▼
       ┌─────────────┐      ┌─────────────┐
       │ Load value  │      │ Trap with   │
       │ Store to rd │      │ violation   │
       └─────────────┘      └─────────────┘
```

## Halt and Return Values

```
Execution terminates when:
  1. HALT instruction executed
  2. RET with empty call stack
  3. Trap (error condition)
  4. Max instructions exceeded (interpreter)

Return value:
  - Located in r0 (a0)
  - JIT: Returns from function call
  - Interpreter: Accessible via registers[0]
```
