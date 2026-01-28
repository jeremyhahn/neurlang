; @name: Div Ceil
; @description: Integer division with rounding up (ceiling).
; @category: algorithm/math
; @difficulty: 1
;
; @prompt: ceiling division {a} / {b}
; @prompt: divide {a} by {b} rounding up
; @prompt: div_ceil({a}, {b})
; @prompt: integer division ceiling {a} {b}
; @prompt: round up {a} / {b}
; @prompt: compute {a} / {b} rounded up
; @prompt: ceiling of {a} divided by {b}
; @prompt: divide and round up {a} {b}
; @prompt: integer ceiling division {a} by {b}
; @prompt: {a} divided by {b} round to ceiling
;
; @param: a=r0 "Dividend"
; @param: b=r1 "Divisor"
;
; @test: r0=10 r1=3 -> r0=4
; @test: r0=9 r1=3 -> r0=3
; @test: r0=1 r1=10 -> r0=1
;
; @export: div_ceil
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r1  ; b
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
    mov r15, r1  ; b
    alu.Add r0, r0, r15
    mov r15, 1  ; 1
    alu.Sub r0, r0, r15
    mov r15, r1  ; b
    muldiv.Div r0, r0, r15
    halt
