; @name: Bit Count (Popcount)
; @description: Count the number of set bits (population count)
; @category: algorithm/bitwise
; @difficulty: 2
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
;
; @param: n=r0 "The value to count set bits in"
;
; @test: r0=0 -> r0=0
; @test: r0=1 -> r0=1
; @test: r0=0xFF00FF -> r0=16
; @test: r0=255 -> r0=8

.entry main

main:
    ; count = 0
    mov r1, 0

.loop:
    ; while n != 0
    beq r0, zero, .done

    ; count += n & 1
    alui.and r2, r0, 1
    alu.add r1, r1, r2

    ; n >>= 1
    alui.shr r0, r0, 1
    b .loop

.done:
    ; return count
    mov r0, r1
    halt
