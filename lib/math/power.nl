; @name: Power
; @description: Calculate a^n (power) iteratively using binary exponentiation.
; @category: algorithm/math
; @difficulty: 2
;
; @prompt: compute {base} to the power of {exp}
; @prompt: {base}^{exp}
; @prompt: calculate {base} raised to {exp}
; @prompt: power({base}, {exp})
; @prompt: exponentiate {base} by {exp}
; @prompt: {base} ** {exp}
; @prompt: compute {base} pow {exp}
; @prompt: raise {base} to the {exp} power
; @prompt: binary exponentiation {base}^{exp}
; @prompt: fast power {base} to {exp}
; @prompt: calculate {base}^{exp} iteratively
; @prompt: compute integer power of {base} and {exp}
;
; @param: base=r0 "The base number"
; @param: exp=r1 "The exponent"
;
; @test: r0=2 r1=0 -> r0=1
; @test: r0=2 r1=10 -> r0=1024
; @test: r0=3 r1=5 -> r0=243
; @test: r0=10 r1=3 -> r0=1000
;
; @export: power
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r2, 1  ; 1
    mov r3, r0  ; base
    mov r4, r1  ; exp
.while_0:
    nop
    mov r15, r4  ; e
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
    mov r15, r4  ; e
    mov r14, 1  ; 1
    alu.And r15, r15, r14
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
    mov r15, r3  ; b
    muldiv.Mul r2, r2, r15
.endif_5:
    nop
    mov r15, r3  ; b
    muldiv.Mul r3, r3, r15
    mov r15, 1  ; 1
    alu.Shr r4, r4, r15
    b .while_0
.endwhile_1:
    nop
    mov r0, r2  ; result
    halt
