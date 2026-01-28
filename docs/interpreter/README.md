# Interpreter Documentation

Fast interpreter for program execution and debugging.

## Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Execution Strategy                               │
└─────────────────────────────────────────────────────────────────────┘

  Program Size Decision (INTERP_THRESHOLD = 1000):

  if program.len < 1000 instructions:
      ┌─────────────────────────────────────────────────────────────┐
      │  Use INTERPRETER                                             │
      │  • 0 compile time                                            │
      │  • Simple execution, good for debugging                      │
      │  • Supports all 32 opcodes including I/O                     │
      └─────────────────────────────────────────────────────────────┘
  else:
      ┌─────────────────────────────────────────────────────────────┐
      │  Use JIT (copy-and-patch)                                    │
      │  • ~5μs compile time                                         │
      │  • Native execution speed                                    │
      │  • Better for larger programs                                │
      └─────────────────────────────────────────────────────────────┘

  NOTE: The high threshold (1000) is currently used while JIT branch
  handling is being stabilized. Most programs use the interpreter.
```

## Interpreter Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Interpreter State                                │
└─────────────────────────────────────────────────────────────────────┘

  ┌────────────────────────────────────────────────────────────────┐
  │  Registers: [u64; 32]    Program Counter: usize                 │
  │  ┌────┬────┬────┬────┬────┬────┬────┬────┐                     │
  │  │ r0 │ r1 │ r2 │ r3 │...│r30 │r31 │ PC │                     │
  │  └────┴────┴────┴────┴────┴────┴────┴────┘                     │
  └────────────────────────────────────────────────────────────────┘

  ┌────────────────────────────────────────────────────────────────┐
  │  Memory: Vec<u8>                                                │
  │  ┌────────────────────────────────────────────────────────┐    │
  │  │ 0x0000 │ 0x0008 │ 0x0010 │ ...                         │    │
  │  └────────────────────────────────────────────────────────┘    │
  └────────────────────────────────────────────────────────────────┘

  ┌────────────────────────────────────────────────────────────────┐
  │  Flags: halted, trap_code                                       │
  └────────────────────────────────────────────────────────────────┘
```

## Dispatch Loop

```rust
pub struct Interpreter {
    registers: [u64; 32],
    memory: Vec<u8>,
    pc: usize,
    halted: bool,
}

impl Interpreter {
    pub fn execute(&mut self, program: &Program) -> InterpResult {
        while !self.halted && self.pc < program.instructions.len() {
            let instr = &program.instructions[self.pc];
            self.pc += 1;

            match self.dispatch(instr) {
                Ok(()) => continue,
                Err(trap) => return InterpResult::Trap(trap),
            }
        }

        InterpResult::Halted(self.registers[0])
    }
}
```

## Dispatch Table

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Opcode Dispatch                                  │
└─────────────────────────────────────────────────────────────────────┘

  Computed Goto Table (conceptual):

  ┌────────┬────────────────────────────────────────────────────────┐
  │ Opcode │ Handler                                                 │
  ├────────┼────────────────────────────────────────────────────────┤
  │ 0x00   │ handle_alu(mode, rd, rs1, rs2)                         │
  │ 0x01   │ handle_alui(mode, rd, rs1, imm)                        │
  │ 0x02   │ handle_muldiv(mode, rd, rs1, rs2)                      │
  │ 0x03   │ handle_load(width, rd, rs1, offset)                    │
  │ 0x04   │ handle_store(width, rs, rs1, offset)                   │
  │ 0x05   │ handle_atomic(mode, rd, rs1, rs2)                      │
  │ 0x06   │ handle_branch(cond, rs1, rs2, target)                  │
  │ ...    │ ...                                                     │
  │ 0x17   │ handle_halt()                                          │
  └────────┴────────────────────────────────────────────────────────┘
```

## Handler Examples

### ALU Operations

```rust
fn handle_alu(&mut self, mode: AluOp, rd: u8, rs1: u8, rs2: u8) {
    let a = self.registers[rs1 as usize];
    let b = self.registers[rs2 as usize];

    let result = match mode {
        AluOp::Add => a.wrapping_add(b),
        AluOp::Sub => a.wrapping_sub(b),
        AluOp::And => a & b,
        AluOp::Or  => a | b,
        AluOp::Xor => a ^ b,
        AluOp::Shl => a << (b & 63),
        AluOp::Shr => a >> (b & 63),
        AluOp::Sar => (a as i64 >> (b & 63)) as u64,
    };

    self.registers[rd as usize] = result;
}
```

### Memory Operations

```rust
fn handle_load(&mut self, width: MemWidth, rd: u8, base: u8, offset: i16) {
    let addr = self.registers[base as usize].wrapping_add(offset as u64);

    // Bounds check (implicit security)
    if addr as usize + width.size() > self.memory.len() {
        return Err(Trap::OutOfBounds);
    }

    let value = match width {
        MemWidth::Byte => self.memory[addr as usize] as u64,
        MemWidth::Half => u16::from_le_bytes(...) as u64,
        MemWidth::Word => u32::from_le_bytes(...) as u64,
        MemWidth::Double => u64::from_le_bytes(...),
    };

    self.registers[rd as usize] = value;
}
```

### Branch Operations

```rust
fn handle_branch(&mut self, cond: BranchCond, rs1: u8, rs2: u8, target: i16) {
    let a = self.registers[rs1 as usize];
    let b = self.registers[rs2 as usize];

    let take = match cond {
        BranchCond::Eq  => a == b,
        BranchCond::Ne  => a != b,
        BranchCond::Lt  => (a as i64) < (b as i64),
        BranchCond::Le  => (a as i64) <= (b as i64),
        BranchCond::Gt  => (a as i64) > (b as i64),
        BranchCond::Ge  => (a as i64) >= (b as i64),
        BranchCond::Always => true,
    };

    if take {
        self.pc = (self.pc as i64 + target as i64) as usize;
    }
}
```

## Performance Characteristics

```
┌─────────────────────────────────────────────────────────────────────┐
│                   Interpreter vs JIT                                 │
└─────────────────────────────────────────────────────────────────────┘

  Startup Cost:
  ┌────────────────┬────────────────┬────────────────┐
  │                │  Interpreter   │      JIT       │
  ├────────────────┼────────────────┼────────────────┤
  │ Compile        │      0         │    5-10 μs     │
  │ First instr    │   ~100 ns      │    ~10 ns      │
  └────────────────┴────────────────┴────────────────┘

  Per-Instruction:
  ┌────────────────┬────────────────┬────────────────┐
  │                │  Interpreter   │      JIT       │
  ├────────────────┼────────────────┼────────────────┤
  │ Dispatch       │    ~50 ns      │      0         │
  │ Execute        │    ~20 ns      │    ~2 ns       │
  │ Total          │    ~70 ns      │    ~2 ns       │
  └────────────────┴────────────────┴────────────────┘

  Break-even Point:
  ┌─────────────────────────────────────────────────────────────────┐
  │ JIT compile time / (interp_time - jit_time) per instruction    │
  │ = 5000 ns / 68 ns ≈ 73 instructions                            │
  │                                                                  │
  │ Currently using threshold of 1000 (interpreter preferred        │
  │ while JIT branch handling is being stabilized)                  │
  └─────────────────────────────────────────────────────────────────┘
```

## Usage

### Direct Interpretation

```rust
use neurlang::interp::Interpreter;
use neurlang::ir::Program;

let program: Program = /* ... */;
let mut interp = Interpreter::new(64 * 1024);  // 64KB memory

// Set initial registers
interp.set_register(0, 42);
interp.set_register(1, 10);

// Execute
let result = interp.execute(&program);

match result {
    InterpResult::Halted(value) => println!("Result: {}", value),
    InterpResult::Trap(trap) => println!("Trap: {:?}", trap),
    InterpResult::MaxInstructions => println!("Hit limit"),
}
```

### With Instruction Limit

```rust
// Prevent infinite loops
let result = interp.execute_limited(&program, 1_000_000);
```

### Step Debugging

```rust
// Single-step execution
while !interp.is_halted() {
    println!("PC: {}, R0: {}", interp.pc(), interp.get_register(0));
    interp.step(&program)?;
}
```

## Result Types

```rust
pub enum InterpResult {
    /// Program halted normally, returning value in r0
    Halted(u64),

    /// Security or runtime trap
    Trap(TrapCode),

    /// Hit maximum instruction limit
    MaxInstructions,
}

pub enum TrapCode {
    OutOfBounds,
    InvalidTag,
    PermissionDenied,
    TaintedData,
    DivideByZero,
    InvalidOpcode,
    StackOverflow,
}
```

## CLI Usage

```bash
# Force interpreter mode
nl run -i program.asm --interp

# With instruction limit
nl run -i program.asm --interp --max-instr 10000

# Compare performance
nl run -i benchmark.asm --stats       # JIT
nl run -i benchmark.asm --interp --stats  # Interpreter
```

## Security in Interpreter

```
┌─────────────────────────────────────────────────────────────────────┐
│                  Interpreter Security Checks                         │
└─────────────────────────────────────────────────────────────────────┘

  Same security model as JIT:

  1. Every LOAD/STORE:
     ┌─────────────────────────────────────────────────────────────┐
     │ • Bounds check against memory size                          │
     │ • Permission check (if capability mode)                     │
     │ • Taint propagation                                         │
     └─────────────────────────────────────────────────────────────┘

  2. Every branch:
     ┌─────────────────────────────────────────────────────────────┐
     │ • Target validation                                         │
     │ • Stack depth check                                         │
     └─────────────────────────────────────────────────────────────┘
```
