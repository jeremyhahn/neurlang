; @name: Random Number
; @description: Generates cryptographically secure random numbers
; @category: crypto/random
; @difficulty: 1
;
; @prompt: generate random number
; @prompt: random u64
; @prompt: secure random value
; @prompt: get random bytes
; @prompt: crypto random number
; @prompt: generate random integer
;
; @server: true
; @nondeterministic: true
; @note: Output is non-deterministic, cannot be tested with fixed values

.entry main

main:
    ; Generate a random 64-bit value
    rand.u64 r0                    ; r0 = random u64

    ; Generate another random value
    rand.u64 r1                    ; r1 = another random u64

    ; XOR them together for demonstration
    alu.xor r2, r0, r1

    ; Result in r0
    mov r0, r2

    halt
