; @name: Power of Two
; @description: Computes 2^n using bit shift
; @category: algorithm/math
; @difficulty: 1
;
; @prompt: compute 2 to the power of {n}
; @prompt: 2^{n}
; @prompt: calculate 2**{n}
; @prompt: power of two for exponent {n}
; @prompt: two raised to {n}
; @prompt: bit shift to compute 2^{n}
; @prompt: 1 << {n}
; @prompt: compute 2 raised to power {n}
;
; @param: n=r0 "The exponent"
;
; @test: r0=0 -> r0=1
; @test: r0=1 -> r0=2
; @test: r0=10 -> r0=1024
; @test: r0=8 -> r0=256

.entry main

main:
    ; 2^n = 1 << n
    mov r1, 1
    alu.shl r0, r1, r0
    halt
