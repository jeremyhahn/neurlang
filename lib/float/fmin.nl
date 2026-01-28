; @name: Fmin
; @description: Calculate minimum of two floats.
; @category: float
; @difficulty: 1
;
; @prompt: minimum of {a} and {b} floats
; @prompt: fmin({a}, {b})
; @prompt: smaller float {a} or {b}
; @prompt: float min of {a} {b}
; @prompt: minimum floating point {a} {b}
; @prompt: lesser of {a} and {b} as floats
; @prompt: compute float minimum {a} {b}
; @prompt: min({a}, {b}) float
; @prompt: which is smaller float {a} {b}
; @prompt: find minimum float between {a} and {b}
;
; @param: a=r0 "First floating-point value"
; @param: b=r1 "Second floating-point value"
;
; @test: r0=5, r1=3 -> r0=3
;
; @export: fmin
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; a
    mov r14, r1  ; b
    fpu.Fcmplt r15, r15, r14
    beq r15, zero, .else_0
    b .endif_1
.else_0:
    nop
    mov r0, r1  ; b
.endif_1:
    nop
    halt
