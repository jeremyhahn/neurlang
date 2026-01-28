; @name: Fclamp
; @description: Clamp a value between min and max.
; @category: float
; @difficulty: 1
;
; @prompt: clamp {x} between {min_val} and {max_val}
; @prompt: fclamp({x}, {min_val}, {max_val})
; @prompt: constrain {x} to range {min_val} to {max_val}
; @prompt: limit {x} to {min_val}-{max_val}
; @prompt: clamp float {x} in range
; @prompt: bound {x} between {min_val} and {max_val}
; @prompt: restrict {x} to interval {min_val} {max_val}
; @prompt: ensure {x} is between {min_val} and {max_val}
; @prompt: saturate {x} to range
; @prompt: clamp value {x} min {min_val} max {max_val}
;
; @param: x=r0 "The value to clamp"
; @param: min_val=r1 "Minimum bound"
; @param: max_val=r2 "Maximum bound"
;
; @test: r0=0x4014000000000000, r1=0, r2=0x4024000000000000 -> r0=0x4014000000000000
; @test: r0=0xC000000000000000, r1=0, r2=0x4024000000000000 -> r0=0
; @test: r0=0x4034000000000000, r1=0, r2=0x4024000000000000 -> r0=0x4024000000000000
; @note: f64 bit patterns. 5.0 in [0,10] -> 5.0; -2.0 in [0,10] -> 0.0; 20.0 in [0,10] -> 10.0
;
; @export: fclamp
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; x
    mov r14, r1  ; min_val
    fpu.Fcmplt r15, r15, r14
    beq r15, zero, .else_0
    mov r0, r1  ; min_val
    b .endif_1
.else_0:
    nop
    mov r15, r0  ; x
    mov r14, r2  ; max_val
    fpu.Fcmpgt r15, r15, r14
    beq r15, zero, .else_2
    mov r0, r2  ; max_val
    b .endif_3
.else_2:
    nop
.endif_3:
    nop
.endif_1:
    nop
    halt
