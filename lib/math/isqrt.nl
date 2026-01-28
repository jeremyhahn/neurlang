; @name: Isqrt
; @description: Integer square root (floor).
; @category: algorithm/math
; @difficulty: 2
;
; @prompt: integer square root of {n}
; @prompt: isqrt({n})
; @prompt: floor of sqrt({n})
; @prompt: compute integer sqrt of {n}
; @prompt: find largest x where x*x <= {n}
; @prompt: square root rounded down {n}
; @prompt: integer sqrt {n}
; @prompt: floor sqrt of {n}
; @prompt: newton's method sqrt {n}
; @prompt: approximate square root {n}
; @prompt: calculate floor(sqrt({n}))
; @prompt: find integer part of sqrt({n})
;
; @param: n=r0 "The number to find square root of"
;
; @test: r0=0 -> r0=0
; @test: r0=1 -> r0=1
; @test: r0=4 -> r0=2
; @test: r0=10 -> r0=3
; @test: r0=100 -> r0=10
; @test: r0=101 -> r0=10
;
; @export: isqrt
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
    mov r1, r0  ; n
    mov r2, r1  ; x
    mov r15, 1  ; 1
    alu.Add r2, r2, r15
    mov r15, 2  ; 2
    muldiv.Div r2, r2, r15
.while_4:
    nop
    mov r15, r2  ; y
    mov r14, r1  ; x
    blt r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endwhile_5
    mov r1, r2  ; y
    mov r15, r0  ; n
    mov r14, r1  ; x
    muldiv.Div r15, r15, r14
    mov r2, r1  ; x
    alu.Add r2, r2, r15
    mov r15, 2  ; 2
    muldiv.Div r2, r2, r15
    b .while_4
.endwhile_5:
    nop
    mov r0, r1  ; x
    halt
