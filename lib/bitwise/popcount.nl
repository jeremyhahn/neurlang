; @name: Popcount
; @description: Count the number of set bits (population count).
; @category: bitwise
;
; @prompt: count the number of set bits in {n}
; @prompt: get the population count of {n}
; @prompt: how many 1 bits are in {n}
; @prompt: count ones in {n}
; @prompt: popcount of {n}
; @prompt: compute the hamming weight of {n}
; @prompt: count how many bits are set in {n}
; @prompt: number of 1s in binary representation of {n}
; @prompt: bit count for {n}
; @prompt: get set bit count of {n}
; @prompt: count all set bits in value {n}
; @prompt: find the population count for {n}
; @prompt: calculate the number of ones in {n}
;
; @param: n=r0 "The value to count set bits in"
;
; @test: r0=0 -> r0=0
; @test: r0=1 -> r0=1
; @test: r0=0xFF -> r0=8
; @test: r0=0xFFFFFFFFFFFFFFFF -> r0=64
;
; @export: popcount
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
    halt
