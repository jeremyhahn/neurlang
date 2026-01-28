; @name: F64 From Bits
; @description: Convert u64 bits to f64.
; @category: float/conversion
; @difficulty: 1
;
; @prompt: convert bits {bits} to float
; @prompt: f64_from_bits({bits})
; @prompt: bits to float {bits}
; @prompt: reinterpret {bits} as f64
; @prompt: make float from bits {bits}
; @prompt: from_bits({bits})
; @prompt: integer bits to float {bits}
; @prompt: create float from bit pattern {bits}
; @prompt: u64 to f64 bits {bits}
; @prompt: construct float from {bits}
;
; @param: bits=r0 "The bit pattern to interpret as float"
;
; @test: r0=0 -> r0=0
;
; @export: f64_from_bits
; Generated from Rust by nl stdlib build

.entry:
    nop
    halt
