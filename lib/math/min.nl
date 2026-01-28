; @name: Min
; @description: Calculate minimum of two values.
; @category: algorithm/math
; @difficulty: 1
;
; @prompt: minimum of {a} and {b}
; @prompt: min({a}, {b})
; @prompt: smaller of {a} and {b}
; @prompt: find minimum between {a} {b}
; @prompt: which is smaller {a} or {b}
; @prompt: get the lesser of {a} and {b}
; @prompt: compute min of {a} {b}
; @prompt: return smaller value {a} {b}
; @prompt: find the smallest of {a} {b}
; @prompt: lesser value between {a} and {b}
;
; @param: a=r0 "First value"
; @param: b=r1 "Second value"
;
; @test: r0=5 r1=3 -> r0=3
; @test: r0=3 r1=5 -> r0=3
; @test: r0=7 r1=7 -> r0=7
;
; @export: min
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; a
    mov r14, r1  ; b
    blt r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .else_0
    b .endif_1
.else_0:
    nop
    mov r0, r1  ; b
.endif_1:
    nop
    halt
