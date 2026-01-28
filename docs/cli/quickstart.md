# CLI Quick Start

Get started with Neurlang in 5 minutes.

## Installation

```bash
# Clone and build
git clone https://github.com/your-repo/nl.git
cd nl
cargo build --release

# Add to PATH (optional)
export PATH="$PATH:$(pwd)/target/release"
```

## Your First Program

### 1. Create a Source File

```bash
cat > hello.asm << 'EOF'
; My first Neurlang program
; Returns 42

main:
    mov r0, 42      ; Set return value
    halt            ; Stop execution
EOF
```

### 2. Run It

```bash
./target/release/nl run -i hello.asm
```

Output:
```
Loaded 2 instructions
Program halted
R0 = 42
```

### 3. Run with Statistics

```bash
./target/release/nl run -i hello.asm --stats
```

Output:
```
Loaded 2 instructions
Program halted
R0 = 42

Statistics:
  Code size: 24 bytes
  Compile time: 0.00ms (1.23μs)
  Execution time: 45ns
```

## Interactive REPL

```bash
./target/release/nl repl
```

```
Neurlang Interactive REPL
Type 'help' for commands, 'quit' to exit

nl> mov r0, 100
=> 0
nl> addi r0, r0, 23
=> 0
nl> regs
Registers:
  r 0 = 0x000000000000007b (123)
  r 1 = 0x0000000000000000 (0)
  ...
nl> halt
Halted. r0 = 123
nl> quit
Goodbye!
```

## Common Workflows

### Assemble and Run Separately

```bash
# Assemble to binary
./nl asm -i program.asm -o program.nlb

# Run binary
./nl run -i program.nlb
```

### Use Interpreter (for Debugging)

```bash
./nl run -i program.asm --interp --stats
```

### Compile to Standalone Binary

```bash
# Compile to raw machine code
./nl compile -i program.asm -o program.bin

# Compile to ELF (Linux)
./nl compile -i program.asm -o program --format elf
chmod +x program
./program
```

### Check Disassembly

```bash
./nl asm -i program.asm --disasm
```

Output:
```
Assembled 5 instructions (28 bytes)

Disassembly:
0000:  mov r0, 10
0008:  mov r1, 1
0010:  add.Add r0, r0, r1
0014:  halt
```

## Example Programs

### Fibonacci

```bash
cat > fib.asm << 'EOF'
; Calculate fib(10)
    mov r0, 10      ; n = 10
    mov r1, 0       ; fib(n-2)
    mov r2, 1       ; fib(n-1)
loop:
    beq r0, zero, done
    add r3, r1, r2
    mov r1, r2
    mov r2, r3
    subi r0, r0, 1
    b loop
done:
    mov r0, r1
    halt
EOF

./nl run -i fib.asm
# Output: R0 = 55
```

### Sum 1 to 100

```bash
cat > sum.asm << 'EOF'
    mov r0, 100     ; n = 100
    mov r1, 0       ; sum = 0
loop:
    add r1, r1, r0
    subi r0, r0, 1
    bne r0, zero, loop
    mov r0, r1
    halt
EOF

./nl run -i sum.asm
# Output: R0 = 5050
```

## Next Steps

1. Read the [Assembly Guide](../ir/assembly.md)
2. Explore [Commands Reference](./commands.md)
3. Try the [Examples](./examples.md)
4. Run [Benchmarks](./commands.md#bench)
