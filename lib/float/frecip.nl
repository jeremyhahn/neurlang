; @name: Frecip
; @description: Calculate the reciprocal (1/x).
; @category: float
; @difficulty: 1
;
; @prompt: reciprocal of {x}
; @prompt: frecip({x})
; @prompt: 1 / {x}
; @prompt: one over {x}
; @prompt: inverse of {x}
; @prompt: compute reciprocal {x}
; @prompt: calculate 1/{x}
; @prompt: multiplicative inverse of {x}
; @prompt: float reciprocal {x}
; @prompt: divide 1 by {x}
;
; @param: x=r0 "The value to compute reciprocal of"
;
; @test: r0=0x4000000000000000 -> r0=0x3FE0000000000000
; @test: r0=0x4010000000000000 -> r0=0x3FD0000000000000
; @note: Input/output are f64 bit patterns. 1/2.0 = 0.5, 1/4.0 = 0.25
;
; @export: frecip
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0                    ; x
    mov r0, 1072693248             ; 0x3FF00000 (high 32 bits of 1.0)
    alui.Shl r0, r0, 32            ; shift to get 0x3FF0000000000000 = 1.0
    fpu.Fdiv r0, r0, r15           ; 1.0 / x
    halt
