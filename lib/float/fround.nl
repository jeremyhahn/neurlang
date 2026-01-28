; @name: Fround
; @description: Round to nearest integer.
; @category: float
; @difficulty: 2
;
; @prompt: round {x} to nearest integer
; @prompt: fround({x})
; @prompt: round({x})
; @prompt: round float {x}
; @prompt: nearest integer to {x}
; @prompt: round {x} to whole number
; @prompt: compute round of {x}
; @prompt: round floating point {x}
; @prompt: banker's rounding {x}
; @prompt: round {x} half to even
;
; @param: x=r0 "The floating-point value to round"
;
; @test: r0=0x400B333333333333 -> r0=0x4008000000000000
; @test: r0=0x400CCCCCCCCCCCCD -> r0=0x4010000000000000
; @note: Input/output are f64 bit patterns. round(3.4) = 3.0, round(3.6) = 4.0
;
; @export: fround
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r1, 1071644672             ; 0x3FE00000 (high 32 bits of 0.5)
    alui.Shl r1, r1, 32            ; shift to get 0x3FE0000000000000 = 0.5
    fpu.Fadd r0, r0, r1            ; x + 0.5
    fpu.Ffloor r0, r0              ; floor(x + 0.5)
    halt
