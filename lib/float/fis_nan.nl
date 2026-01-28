; @name: Fis Nan
; @description: Check if float is NaN.
; @category: float
; @difficulty: 2
;
; @prompt: is {x} NaN
; @prompt: fis_nan({x})
; @prompt: check if {x} is not a number
; @prompt: is_nan({x})
; @prompt: test if {x} is NaN
; @prompt: {x} == NaN check
; @prompt: is float {x} undefined
; @prompt: check NaN {x}
; @prompt: detect NaN in {x}
; @prompt: is {x} a valid number
;
; @param: x=r0 "The floating-point value to check"
;
; @test: r0=0 -> r0=0
;
; @export: fis_nan
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; x
    mov r15, 0  ; TODO: is_nan
    beq r15, zero, .else_0
    mov r0, 1  ; 1
    b .endif_1
.else_0:
    nop
    mov r0, 0  ; 0
.endif_1:
    nop
    halt
