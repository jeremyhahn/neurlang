; @name: Fadd
; @description: Add two floating-point numbers.
; @category: float
; @difficulty: 1
;
; @prompt: add {a} and {b} floats
; @prompt: float add {a} + {b}
; @prompt: fadd({a}, {b})
; @prompt: add floating point numbers {a} {b}
; @prompt: compute {a} + {b} as floats
; @prompt: sum two floats {a} and {b}
; @prompt: floating point addition {a} {b}
; @prompt: add f64 values {a} {b}
; @prompt: {a} plus {b} float
; @prompt: compute float sum of {a} and {b}
;
; @param: a=r0 "First floating-point number (as bits)"
; @param: b=r1 "Second floating-point number (as bits)"
;
; @test: r0=0, r1=0 -> r0=0
;
; @export: fadd
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r1  ; b
    fpu.Fadd r0, r0, r15
    halt
