; @name: Fibonacci
; @description: Calculate the nth Fibonacci number iteratively
; @category: algorithm/math
; @difficulty: 2
;
; @prompt: compute fibonacci({n})
; @prompt: calculate fib({n})
; @prompt: find the {n}th fibonacci number
; @prompt: fibonacci sequence element {n}
; @prompt: what is fib({n})
; @prompt: {n}th fibonacci
; @prompt: fibonacci of {n}
; @prompt: iterative fibonacci for {n}
; @prompt: compute fib sequence at position {n}
; @prompt: calculate fibonacci number {n}
;
; @param: n=r0 "The position in the Fibonacci sequence (0-indexed)"
;
; @test: r0=0 -> r0=0
; @test: r0=1 -> r0=1
; @test: r0=10 -> r0=55
; @test: r0=20 -> r0=6765

.entry main

main:
    ; Handle base cases
    ; if n == 0, return 0
    beq r0, zero, .return_zero

    ; if n == 1, return 1
    mov r15, 1
    beq r0, r15, .return_one

    ; Iterative Fibonacci: fib(n) for n >= 2
    mov r1, 0           ; a = fib(0) = 0
    mov r2, 1           ; b = fib(1) = 1
    mov r3, 2           ; i = 2

.loop:
    ; while i <= n
    bgt r3, r0, .done

    ; temp = a + b
    alu.add r4, r1, r2

    ; a = b
    mov r1, r2

    ; b = temp
    mov r2, r4

    ; i++
    alui.add r3, r3, 1
    b .loop

.done:
    ; return b
    mov r0, r2
    halt

.return_zero:
    mov r0, 0
    halt

.return_one:
    mov r0, 1
    halt
