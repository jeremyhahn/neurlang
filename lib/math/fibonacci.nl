; @name: Fibonacci
; @description: Calculate the nth Fibonacci number iteratively.
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
; @prompt: find fibonacci({n})
; @prompt: get the {n}th term of fibonacci sequence
;
; @param: n=r0 "The position in the Fibonacci sequence (0-indexed)"
;
; @test: r0=0 -> r0=0
; @test: r0=1 -> r0=1
; @test: r0=10 -> r0=55
; @test: r0=20 -> r0=6765
; @test: r0=40 -> r0=102334155
;
; @export: fibonacci
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; n
    mov r14, 0  ; 0
    beq r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endif_1
    mov r0, 0  ; 0
    halt
.endif_1:
    nop
    mov r15, r0  ; n
    mov r14, 1  ; 1
    beq r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endif_5
    mov r0, 1  ; 1
    halt
.endif_5:
    nop
    mov r1, 0  ; 0
    mov r2, 1  ; 1
    mov r3, 2  ; 2
.while_8:
    nop
    mov r15, r3  ; i
    mov r14, r0  ; n
    ble r15, r14, .set_10
    mov r15, 0
    b .cmp_end_11
.set_10:
    nop
    mov r15, 1
.cmp_end_11:
    nop
    beq r15, zero, .endwhile_9
    mov r4, r1  ; a
    mov r15, r2  ; b
    alu.Add r4, r4, r15
    mov r1, r2  ; b
    mov r2, r4  ; temp
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .while_8
.endwhile_9:
    nop
    mov r0, r2  ; b
    halt
