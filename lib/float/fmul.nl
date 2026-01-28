; @name: Fmul
; @description: Multiply two floating-point numbers.
; @category: float
; @difficulty: 1
;
; @prompt: multiply {a} and {b} floats
; @prompt: float multiply {a} * {b}
; @prompt: fmul({a}, {b})
; @prompt: multiply floating point {a} times {b}
; @prompt: compute {a} * {b} as floats
; @prompt: product of two floats {a} and {b}
; @prompt: floating point multiplication {a} {b}
; @prompt: multiply f64 values {a} {b}
; @prompt: {a} times {b} float
; @prompt: compute float product of {a} and {b}
;
; @param: a=r0 "First floating-point number"
; @param: b=r1 "Second floating-point number"
;
; @test: r0=0, r1=0 -> r0=0
;
; @export: fmul
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r1  ; b
    fpu.Fmul r0, r0, r15
    halt
