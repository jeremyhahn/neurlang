; @name: Fmax
; @description: Calculate maximum of two floats.
; @category: float
; @difficulty: 1
;
; @prompt: maximum of {a} and {b} floats
; @prompt: fmax({a}, {b})
; @prompt: larger float {a} or {b}
; @prompt: float max of {a} {b}
; @prompt: maximum floating point {a} {b}
; @prompt: greater of {a} and {b} as floats
; @prompt: compute float maximum {a} {b}
; @prompt: max({a}, {b}) float
; @prompt: which is larger float {a} {b}
; @prompt: find maximum float between {a} and {b}
;
; @param: a=r0 "First floating-point value"
; @param: b=r1 "Second floating-point value"
;
; @test: r0=5, r1=3 -> r0=5
;
; @export: fmax
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; a
    mov r14, r1  ; b
    fpu.Fcmpgt r15, r15, r14
    beq r15, zero, .else_0
    b .endif_1
.else_0:
    nop
    mov r0, r1  ; b
.endif_1:
    nop
    halt
