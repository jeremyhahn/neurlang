; @name: Parity
; @description: Parity (1 if odd number of set bits, 0 otherwise).
; @category: bitwise
; @difficulty: 1
;
; @prompt: compute the parity of {n}
; @prompt: get the parity bit for {n}
; @prompt: check if {n} has an odd number of set bits
; @prompt: calculate parity of {n}
; @prompt: XOR all bits of {n} together
; @prompt: is the bit count of {n} odd or even
; @prompt: compute single-bit parity for {n}
; @prompt: parity check for {n}
; @prompt: determine if {n} has odd parity
; @prompt: get the parity of the bits in {n}
; @prompt: find if popcount of {n} is odd
; @prompt: return 1 if {n} has odd number of 1s, else 0
; @prompt: calculate the parity bit of {n}
;
; @param: n=r0 "The value to compute parity for"
;
; @test: r0=0 -> r0=0
;
; @export: parity
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r1, 0  ; 0
.while_0:
    nop
    mov r15, r0  ; n
    mov r14, 0  ; 0
    bne r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endwhile_1
    mov r15, r0  ; n
    mov r14, 1  ; 1
    alu.And r15, r15, r14
    alu.Add r1, r1, r15
    mov r15, 1  ; 1
    alu.Shr r0, r0, r15
    b .while_0
.endwhile_1:
    nop
    mov r0, r1  ; count
    mov r15, 1  ; 1
    alu.And r0, r0, r15
    halt
