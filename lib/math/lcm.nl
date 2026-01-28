; @name: Lcm
; @description: Calculate least common multiple.
; @category: algorithm/math
; @difficulty: 2
;
; @prompt: find LCM of {a} and {b}
; @prompt: lcm({a}, {b})
; @prompt: least common multiple of {a} and {b}
; @prompt: compute lcm of {a} {b}
; @prompt: what is the LCM of {a} and {b}
; @prompt: smallest common multiple of {a} and {b}
; @prompt: calculate least common multiple {a} {b}
; @prompt: lowest common multiple {a} {b}
; @prompt: find smallest number divisible by {a} and {b}
; @prompt: compute lowest common multiple({a}, {b})
;
; @param: a=r0 "First number"
; @param: b=r1 "Second number"
;
; @test: r0=4 r1=6 -> r0=12
; @test: r0=3 r1=5 -> r0=15
; @test: r0=12 r1=18 -> r0=36
;
; @export: lcm
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r14, r1  ; b
    mov r15, 0  ; 0
    beq r14, r15, .set_2
    mov r14, 0
    b .cmp_end_3
.set_2:
    nop
    mov r14, 1
.cmp_end_3:
    nop
    mov r15, r0  ; a
    mov r14, 0  ; 0
    beq r15, r14, .set_4
    mov r15, 0
    b .cmp_end_5
.set_4:
    nop
    mov r15, 1
.cmp_end_5:
    nop
    alu.Or r15, r15, r14
    bne r15, zero, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endif_1
    mov r0, 0  ; 0
    halt
.endif_1:
    nop
    mov r2, r0  ; a
    mov r3, r1  ; b
.while_8:
    nop
    mov r15, r3  ; y
    mov r14, 0  ; 0
    bne r15, r14, .set_10
    mov r15, 0
    b .cmp_end_11
.set_10:
    nop
    mov r15, 1
.cmp_end_11:
    nop
    beq r15, zero, .endwhile_9
    mov r4, r3  ; y
    mov r15, r3  ; y
    mov r3, r2  ; x
    muldiv.Mod r3, r3, r15
    mov r2, r4  ; temp
    b .while_8
.endwhile_9:
    nop
    mov r15, r2  ; x
    muldiv.Div r0, r0, r15
    mov r15, r1  ; b
    muldiv.Mul r0, r0, r15
    halt
