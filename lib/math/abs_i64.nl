; @name: Abs I64
; @description: Calculate absolute value (for signed integers represented as u64).
; @category: algorithm/math
; @difficulty: 1
;
; @prompt: absolute value of {n}
; @prompt: abs({n})
; @prompt: compute |{n}|
; @prompt: get absolute value of {n}
; @prompt: make {n} positive
; @prompt: magnitude of {n}
; @prompt: unsigned value of {n}
; @prompt: remove sign from {n}
; @prompt: convert {n} to positive
; @prompt: calculate absolute value {n}
;
; @param: n=r0 "The signed integer value"
;
; @test: r0=5 -> r0=5
; @test: r0=0 -> r0=0
;
; @export: abs_i64
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; n
    mov r14, 63  ; 63
    alu.Shr r15, r15, r14
    mov r14, 1  ; 1
    beq r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .else_0
    alui.Xor r0, r0, -1
    mov r15, 1  ; 1
    alu.Add r0, r0, r15
    b .endif_1
.else_0:
    nop
.endif_1:
    nop
    halt
