; @name: Fabs
; @description: Calculate absolute value of float.
; @category: float
; @difficulty: 1
;
; @prompt: absolute value of {x} float
; @prompt: fabs({x})
; @prompt: float abs {x}
; @prompt: |{x}| as float
; @prompt: absolute float {x}
; @prompt: make {x} positive float
; @prompt: magnitude of float {x}
; @prompt: compute float absolute value {x}
; @prompt: floating point abs of {x}
; @prompt: remove sign from float {x}
;
; @param: x=r0 "The floating-point value"
;
; @test: r0=0xC014000000000000 -> r0=0x4014000000000000
; @test: r0=0x4014000000000000 -> r0=0x4014000000000000
; @test: r0=0 -> r0=0
; @note: Input/output are f64 bit patterns. -5.0 -> 5.0, 5.0 -> 5.0, 0.0 -> 0.0
;
; @export: fabs
; Generated from Rust by nl stdlib build

.entry:
    nop
    fpu.Fabs r0, r0
    halt
