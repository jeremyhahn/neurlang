; @name: Reverse Bits
; @description: Reverse bits.
; @category: bitwise
; @difficulty: 2
;
; @prompt: reverse the bits in {n}
; @prompt: flip bit order of {n}
; @prompt: reverse bit order in {n}
; @prompt: mirror the bits of {n}
; @prompt: reflect bits in {n}
; @prompt: reverse all 64 bits of {n}
; @prompt: bit reversal of {n}
; @prompt: swap bit positions in {n} (MSB becomes LSB)
; @prompt: invert the bit order of {n}
; @prompt: reverse binary representation of {n}
; @prompt: flip {n} bit by bit
; @prompt: get bit-reversed value of {n}
; @prompt: compute bit reversal for {n}
;
; @param: n=r0 "The value to reverse bits in"
;
; @test: r0=0 -> r0=0
;
; @export: reverse_bits
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r1, 0  ; 0
    mov r2, 0  ; 0
.while_0:
    nop
    mov r15, r2  ; i
    mov r14, 64  ; 64
    blt r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endwhile_1
    mov r15, 1  ; 1
    alu.Shl r1, r1, r15
    mov r15, r0  ; n
    mov r14, 1  ; 1
    alu.And r15, r15, r14
    alu.Or r1, r1, r15
    mov r15, 1  ; 1
    alu.Shr r0, r0, r15
    mov r15, 1  ; 1
    alu.Add r2, r2, r15
    b .while_0
.endwhile_1:
    nop
    mov r0, r1  ; result
    halt
