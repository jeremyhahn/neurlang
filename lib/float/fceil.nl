; @name: Fceil
; @description: Calculate ceiling (round up).
; @category: float
; @difficulty: 1
;
; @prompt: ceiling of {x}
; @prompt: fceil({x})
; @prompt: round {x} up
; @prompt: ceiling function {x}
; @prompt: smallest integer >= {x}
; @prompt: round up {x}
; @prompt: compute ceiling of {x}
; @prompt: integer part rounded up {x}
; @prompt: floating point ceiling {x}
; @prompt: ceil({x})
;
; @param: x=r0 "The floating-point value to ceil"
;
; @test: r0=0x400B333333333333 -> r0=0x4010000000000000
; @test: r0=0x4014000000000000 -> r0=0x4014000000000000
; @note: Input/output are f64 bit patterns. 3.4 -> 4.0, 5.0 -> 5.0
;
; @export: fceil
; Generated from Rust by nl stdlib build

.entry:
    nop
    fpu.Fceil r0, r0
    halt
