; @name: Fsub
; @description: Subtract two floating-point numbers.
; @category: float
; @difficulty: 1
;
; @prompt: subtract {b} from {a} float
; @prompt: float subtract {a} - {b}
; @prompt: fsub({a}, {b})
; @prompt: subtract floating point {a} minus {b}
; @prompt: compute {a} - {b} as floats
; @prompt: difference of two floats {a} and {b}
; @prompt: floating point subtraction {a} {b}
; @prompt: subtract f64 values {a} {b}
; @prompt: {a} minus {b} float
; @prompt: compute float difference of {a} and {b}
;
; @param: a=r0 "First floating-point number (minuend)"
; @param: b=r1 "Second floating-point number (subtrahend)"
;
; @test: r0=0, r1=0 -> r0=0
;
; @export: fsub
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r1  ; b
    fpu.Fsub r0, r0, r15
    halt
