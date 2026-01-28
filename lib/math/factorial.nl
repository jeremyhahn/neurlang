; @name: Factorial
; @description: Calculate factorial of n iteratively.
; @category: algorithm/math
; @difficulty: 2
;
; @prompt: compute factorial of {n}
; @prompt: {n}!
; @prompt: calculate {n} factorial
; @prompt: what is {n}!
; @prompt: factorial({n})
; @prompt: multiply 1 * 2 * ... * {n}
; @prompt: product of integers from 1 to {n}
; @prompt: n! where n={n}
; @prompt: iterative factorial for {n}
; @prompt: compute {n} factorial using a loop
; @prompt: find the factorial of {n}
; @prompt: calculate {n}! iteratively
;
; @param: n=r0 "The number to compute factorial of"
;
; @test: r0=0 -> r0=1
; @test: r0=1 -> r0=1
; @test: r0=5 -> r0=120
; @test: r0=10 -> r0=3628800
; @test: r0=20 -> r0=2432902008176640000
;
; @export: factorial
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r1, 1  ; 1
    mov r2, r0  ; n
.while_0:
    nop
    mov r15, r2  ; i
    mov r14, 0  ; 0
    bgt r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endwhile_1
    mov r15, r2  ; i
    muldiv.Mul r1, r1, r15
    mov r15, 1  ; 1
    alu.Sub r2, r2, r15
    b .while_0
.endwhile_1:
    nop
    mov r0, r1  ; result
    halt
