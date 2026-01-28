; @name: Triangle Number
; @description: Calculate the sum of numbers from 1 to n (triangle number).
; @category: algorithm/math
; @difficulty: 1
;
; @prompt: sum of 1 to {n}
; @prompt: triangle number {n}
; @prompt: sum 1 + 2 + ... + {n}
; @prompt: compute 1+2+3+...+{n}
; @prompt: triangular number for {n}
; @prompt: sum first {n} natural numbers
; @prompt: what is 1+2+...+{n}
; @prompt: gauss sum to {n}
; @prompt: arithmetic sum 1 to {n}
; @prompt: calculate sum of integers from 1 to {n}
; @prompt: add numbers 1 through {n}
; @prompt: sum of first {n} positive integers
;
; @param: n=r0 "The upper bound of the sum"
;
; @test: r0=1 -> r0=1
; @test: r0=5 -> r0=15
; @test: r0=10 -> r0=55
; @test: r0=100 -> r0=5050
;
; @export: triangle_number
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; n
    mov r14, 1  ; 1
    alu.Add r15, r15, r14
    muldiv.Mul r0, r0, r15
    mov r15, 2  ; 2
    muldiv.Div r0, r0, r15
    halt
