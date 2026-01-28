; @name: F64 To Bits
; @description: Convert f64 bits to u64 (for bit manipulation).
; @category: float/conversion
; @difficulty: 1
;
; @prompt: convert float {x} to bits
; @prompt: f64_to_bits({x})
; @prompt: float to bits {x}
; @prompt: get bit pattern of {x}
; @prompt: reinterpret {x} as u64
; @prompt: float bits of {x}
; @prompt: raw bits of float {x}
; @prompt: extract bits from float {x}
; @prompt: to_bits({x})
; @prompt: float to integer bits {x}
;
; @param: x=r0 "The floating-point value"
;
; @test: r0=0 -> r0=0
;
; @export: f64_to_bits
; Generated from Rust by nl stdlib build

.entry:
    nop
    halt
