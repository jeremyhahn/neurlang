; @name: Ffloor
; @description: Calculate floor (round down).
; @category: float
; @difficulty: 1
;
; @prompt: floor of {x}
; @prompt: ffloor({x})
; @prompt: round {x} down
; @prompt: floor function {x}
; @prompt: largest integer <= {x}
; @prompt: round down {x}
; @prompt: compute floor of {x}
; @prompt: integer part rounded down {x}
; @prompt: floating point floor {x}
; @prompt: floor({x})
;
; @param: x=r0 "The floating-point value to floor"
;
; @test: r0=0x400B333333333333 -> r0=0x4008000000000000
; @test: r0=0x4014000000000000 -> r0=0x4014000000000000
; @note: Input/output are f64 bit patterns. 3.4 -> 3.0, 5.0 -> 5.0
;
; @export: ffloor
; Generated from Rust by nl stdlib build

.entry:
    nop
    fpu.Ffloor r0, r0
    halt
