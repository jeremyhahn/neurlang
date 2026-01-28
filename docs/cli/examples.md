# CLI Examples

Practical examples for common tasks.

## Basic Programs

### Hello World

```bash
# Create program
cat > hello.asm << 'EOF'
mov r0, 42
halt
EOF

# Run
nl run -i hello.asm
# Output: R0 = 42
```

### Calculator

```bash
# Add two numbers
cat > add.asm << 'EOF'
mov r0, 15      ; First number
mov r1, 27      ; Second number
add r2, r0, r1  ; Result in r2
mov r0, r2      ; Move to r0 for return
halt
EOF

nl run -i add.asm
# Output: R0 = 42
```

## Loops

### Count to 10

```bash
cat > count.asm << 'EOF'
    mov r0, 0       ; counter
    mov r1, 10      ; target
loop:
    addi r0, r0, 1
    blt r0, r1, loop
    halt
EOF

nl run -i count.asm --stats
```

### Factorial

```bash
cat > factorial.asm << 'EOF'
; Calculate 5! = 120
    mov r0, 5       ; n
    mov r1, 1       ; result
loop:
    beq r0, zero, done
    mul r1, r1, r0
    subi r0, r0, 1
    b loop
done:
    mov r0, r1
    halt
EOF

nl run -i factorial.asm
# Output: R0 = 120
```

## Memory Operations

### Store and Load

```bash
cat > memory.asm << 'EOF'
    mov r0, 0x1234  ; Value to store
    mov r1, 0       ; Address
    store.d r0, [r1]
    load.d r2, [r1]
    mov r0, r2      ; Should be 0x1234
    halt
EOF

nl run -i memory.asm
# Output: R0 = 4660 (0x1234)
```

### Array Sum

```bash
cat > array.asm << 'EOF'
; Sum 5 elements starting at address 0
; First, initialize array in memory

    mov r0, 10      ; array[0] = 10
    mov r1, 0
    store.d r0, [r1]

    mov r0, 20      ; array[1] = 20
    addi r1, r1, 8
    store.d r0, [r1]

    mov r0, 30      ; array[2] = 30
    addi r1, r1, 8
    store.d r0, [r1]

; Now sum them
    mov r2, 0       ; sum = 0
    mov r3, 0       ; address
    mov r4, 3       ; count

loop:
    beq r4, zero, done
    load.d r5, [r3]
    add r2, r2, r5
    addi r3, r3, 8
    subi r4, r4, 1
    b loop

done:
    mov r0, r2      ; Return sum
    halt
EOF

nl run -i array.asm --interp
# Output: R0 = 60
```

## Batch Processing

### Assemble Multiple Files

```bash
for f in *.asm; do
    nl asm -i "$f" -o "${f%.asm}.nlb"
done
```

### Benchmark Multiple Programs

```bash
for f in *.nlb; do
    echo "=== $f ==="
    nl run -i "$f" --stats
done
```

### Generate Test Data

```bash
# Generate 1000 training examples
cargo run --release --bin nl-datagen -- \
    --output training.jsonl \
    --num-examples 1000 \
    --curriculum-level 3

# Check output
head -3 training.jsonl | jq .
```

## Pipeline Examples

### Compile and Execute

```bash
# One-liner: assemble, compile to ELF, run
nl asm -i prog.asm -o /tmp/prog.nlb && \
nl compile -i /tmp/prog.nlb -o /tmp/prog --format elf && \
chmod +x /tmp/prog && \
/tmp/prog
```

### Debug with Interpreter

```bash
# Run with interpreter for detailed execution
nl run -i buggy.asm --interp --max-instr 100

# Check instruction count
nl run -i program.asm --interp --stats 2>&1 | grep Instructions
```

### Performance Comparison

```bash
echo "JIT:"
time nl run -i benchmark.asm --stats

echo "Interpreter:"
time nl run -i benchmark.asm --interp --stats
```

## REPL Sessions

### Interactive Debugging

```bash
nl repl << 'EOF'
mov r0, 100
mov r1, 1
sub r0, r0, r1
regs
quit
EOF
```

### Test Assembly Syntax

```bash
nl repl << 'EOF'
; Test different syntaxes
add r0, r1, r2
addi r0, r1, 42
load.d r0, [r1]
load.d r0, [r1 + 8]
ld r0, 16(r1)
quit
EOF
```

## Integration Examples

### From Shell Script

```bash
#!/bin/bash
# run_program.sh

PROGRAM="$1"
INPUT="$2"

# Prepare registers (r0 = input)
nl repl << EOF
mov r0, $INPUT
$(cat "$PROGRAM")
regs
quit
EOF
```

### Generate and Run

```bash
# Generate Fibonacci for N
N=15
cat > /tmp/fib.asm << EOF
    mov r0, $N
    mov r1, 0
    mov r2, 1
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

nl run -i /tmp/fib.asm
# Output: R0 = 610 (fib(15))
```
