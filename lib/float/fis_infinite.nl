; @name: Fis Infinite
; @description: Check if float is infinite.
; @category: float
; @difficulty: 2
;
; @prompt: is {x} infinite
; @prompt: fis_infinite({x})
; @prompt: check if {x} is infinity
; @prompt: is_infinite({x})
; @prompt: test if {x} is infinite
; @prompt: is {x} plus or minus infinity
; @prompt: check infinity {x}
; @prompt: detect infinity in {x}
; @prompt: is {x} unbounded
; @prompt: {x} == inf check
;
; @param: x=r0 "The floating-point value to check"
;
; @test: r0=0 -> r0=0
;
; @export: fis_infinite
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; x
    mov r15, 0  ; TODO: is_infinite
    beq r15, zero, .else_0
    mov r0, 1  ; 1
    b .endif_1
.else_0:
    nop
    mov r0, 0  ; 0
.endif_1:
    nop
    halt
