; @name: Factorial
; @description: Calculate factorial of n iteratively
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
;
; @param: n=r0 "The number to compute factorial of"
;
; @test: r0=0 -> r0=1
; @test: r0=1 -> r0=1
; @test: r0=5 -> r0=120
; @test: r0=10 -> r0=3628800

.entry main

main:
    ; result = 1
    mov r1, 1

    ; i = n (count down)
    mov r2, r0

.loop:
    ; while i > 0
    beq r2, zero, .done

    ; result = result * i
    muldiv.mul r1, r1, r2

    ; i--
    alui.sub r2, r2, 1
    b .loop

.done:
    ; return result
    mov r0, r1
    halt
