; @name: Fsign
; @description: Calculate the sign of a number: -1.0, 0.0, or 1.0.
; @category: float
; @difficulty: 1
;
; @prompt: sign of {x}
; @prompt: fsign({x})
; @prompt: signum of {x}
; @prompt: get sign of float {x}
; @prompt: is {x} positive negative or zero
; @prompt: sign function {x}
; @prompt: compute sign of {x}
; @prompt: determine sign of {x}
; @prompt: float sign {x}
; @prompt: return -1 0 or 1 for {x}
;
; @param: x=r0 "The floating-point value"
;
; @test: r0=0x4014000000000000 -> r0=0x3FF0000000000000
; @test: r0=0 -> r0=0
; @test: r0=0xC014000000000000 -> r0=0xBFF0000000000000
; @note: f64 bit patterns. sign(5.0) = 1.0, sign(0.0) = 0.0, sign(-5.0) = -1.0
;
; @export: fsign
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; x
    mov r14, 0  ; 0.0
    fpu.Fcmpgt r15, r15, r14
    beq r15, zero, .else_0
    mov r0, 1072693248            ; 0x3FF00000 (high 32 bits of 1.0)
    alui.Shl r0, r0, 32           ; shift to get 1.0
    b .endif_1
.else_0:
    nop
    mov r15, r0  ; x
    mov r14, 0  ; 0.0
    fpu.Fcmplt r15, r15, r14
    beq r15, zero, .else_2
    mov r0, 3220176896            ; 0xBFF00000 (high 32 bits of -1.0)
    alui.Shl r0, r0, 32           ; shift to get -1.0
    b .endif_3
.else_2:
    nop
    mov r0, 0  ; 0.0
.endif_3:
    nop
.endif_1:
    nop
    halt
