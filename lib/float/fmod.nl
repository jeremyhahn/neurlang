; @name: Fmod
; @description: Calculate x modulo y for floating point.
; @category: float
; @difficulty: 2
;
; @prompt: {x} modulo {y} float
; @prompt: fmod({x}, {y})
; @prompt: float remainder {x} % {y}
; @prompt: compute {x} mod {y} as float
; @prompt: floating point modulo {x} {y}
; @prompt: remainder of {x} / {y}
; @prompt: {x} mod {y} floating point
; @prompt: calculate float remainder {x} {y}
; @prompt: modulus of {x} and {y}
; @prompt: {x} % {y} for floats
;
; @param: x=r0 "Dividend"
; @param: y=r1 "Divisor"
;
; @test: r0=0x4024000000000000, r1=0x4008000000000000 -> r0=0x3FF0000000000000
; @test: r0=0x401C000000000000, r1=0x4010000000000000 -> r0=0x4008000000000000
; @note: f64 bit patterns. 10.0 % 3.0 = 1.0; 7.0 % 4.0 = 3.0
;
; @export: fmod
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; x
    mov r14, r1  ; y
    fpu.Fdiv r15, r15, r14
    fpu.Ffloor r15, r15
    mov r14, r1  ; y
    fpu.Fmul r15, r15, r14
    fpu.Fsub r0, r0, r15
    halt
