; @name: Fsqrt
; @description: Calculate square root.
; @category: float
; @difficulty: 1
;
; @prompt: square root of {x}
; @prompt: sqrt({x})
; @prompt: fsqrt({x})
; @prompt: compute square root of {x}
; @prompt: float square root {x}
; @prompt: calculate sqrt of {x}
; @prompt: find square root {x}
; @prompt: root of {x}
; @prompt: compute sqrt {x}
; @prompt: floating point square root of {x}
;
; @param: x=r0 "The value to compute square root of"
;
; @test: r0=0 -> r0=0
;
; @export: fsqrt
; Generated from Rust by nl stdlib build

.entry:
    nop
    fpu.Fsqrt r0, r0
    halt
