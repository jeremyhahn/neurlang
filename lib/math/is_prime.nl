; @name: Is Prime
; @description: Check if a number is prime.
; @category: algorithm/math
; @difficulty: 2
;
; @prompt: check if {n} is prime
; @prompt: is {n} a prime number
; @prompt: test primality of {n}
; @prompt: is_prime({n})
; @prompt: determine if {n} is prime
; @prompt: check primality of {n}
; @prompt: is {n} prime?
; @prompt: prime test for {n}
; @prompt: verify {n} is prime
; @prompt: check whether {n} has only two factors
; @prompt: is {n} divisible only by 1 and itself
; @prompt: primality check for {n}
;
; @param: n=r0 "The number to check for primality"
;
; @test: r0=2 -> r0=1
; @test: r0=3 -> r0=1
; @test: r0=4 -> r0=0
; @test: r0=17 -> r0=1
; @test: r0=100 -> r0=0
; @test: r0=101 -> r0=1
;
; @export: is_prime
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; n
    mov r14, 2  ; 2
    blt r15, r14, .set_2
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
    mov r14, 2  ; 2
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
    mov r15, r0  ; n
    mov r14, 1  ; 1
    alu.And r15, r15, r14
    mov r14, 0  ; 0
    beq r15, r14, .set_10
    mov r15, 0
    b .cmp_end_11
.set_10:
    nop
    mov r15, 1
.cmp_end_11:
    nop
    beq r15, zero, .endif_9
    mov r0, 0  ; 0
    halt
.endif_9:
    nop
    mov r1, 3  ; 3
.while_12:
    nop
    mov r15, r1  ; i
    mov r14, r1  ; i
    muldiv.Mul r15, r15, r14
    mov r14, r0  ; n
    ble r15, r14, .set_14
    mov r15, 0
    b .cmp_end_15
.set_14:
    nop
    mov r15, 1
.cmp_end_15:
    nop
    beq r15, zero, .endwhile_13
    mov r15, r0  ; n
    mov r14, r1  ; i
    muldiv.Mod r15, r15, r14
    mov r14, 0  ; 0
    beq r15, r14, .set_18
    mov r15, 0
    b .cmp_end_19
.set_18:
    nop
    mov r15, 1
.cmp_end_19:
    nop
    beq r15, zero, .endif_17
    mov r0, 0  ; 0
    halt
.endif_17:
    nop
    mov r15, 2  ; 2
    alu.Add r1, r1, r15
    b .while_12
.endwhile_13:
    nop
    mov r0, 1  ; 1
    halt
