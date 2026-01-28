; @name: Fdiv
; @description: Divide two floating-point numbers.
; @category: float
; @difficulty: 1
;
; @prompt: divide {a} by {b} float
; @prompt: float divide {a} / {b}
; @prompt: fdiv({a}, {b})
; @prompt: divide floating point {a} by {b}
; @prompt: compute {a} / {b} as floats
; @prompt: quotient of two floats {a} and {b}
; @prompt: floating point division {a} {b}
; @prompt: divide f64 values {a} {b}
; @prompt: {a} divided by {b} float
; @prompt: compute float quotient of {a} and {b}
;
; @param: a=r0 "Dividend (numerator)"
; @param: b=r1 "Divisor (denominator)"
;
; @test: r0=0x4024000000000000, r1=0x4000000000000000 -> r0=0x4014000000000000
; @note: Input/output are f64 bit patterns. 10.0 / 2.0 = 5.0
;
; @export: fdiv
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r1  ; b
    fpu.Fdiv r0, r0, r15
    halt
