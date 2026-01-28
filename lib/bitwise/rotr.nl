; @name: Rotr
; @description: Rotate right.
; @category: bitwise
; @difficulty: 1
;
; @prompt: rotate {n} right by {shift} bits
; @prompt: right rotate {n} by {shift}
; @prompt: circular shift {n} right by {shift}
; @prompt: rotr {n} by {shift}
; @prompt: rotate bits of {n} right {shift} positions
; @prompt: perform right rotation on {n} by {shift} bits
; @prompt: bit rotate {n} rightward by {shift}
; @prompt: cyclically shift {n} right by {shift}
; @prompt: right bit rotation of {n} by {shift}
; @prompt: rotate {n} rightward {shift} times
; @prompt: circular right shift {n} by {shift}
; @prompt: roll bits of {n} right by {shift}
;
; @param: n=r0 "The value to rotate"
; @param: shift=r1 "Number of bits to rotate right"
;
; @test: r0=1, r1=0 -> r0=1
; @test: r0=2, r1=1 -> r0=1
; @test: r0=1, r1=1 -> r0=0x8000000000000000
; @note: Rotating by 0 returns the same value
;
; @export: rotr
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r2, r1  ; shift
    mov r15, 63  ; 63
    alu.And r2, r2, r15
    mov r15, r2  ; s
    mov r14, 0  ; 0
    beq r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endif_1
    halt
.endif_1:
    nop
    mov r3, r0  ; n
    mov r15, r2  ; s
    alu.Shr r3, r3, r15
    mov r4, 64  ; 64
    mov r15, r2  ; s
    alu.Sub r4, r4, r15
    mov r5, r0  ; n
    mov r15, r4  ; left_shift
    alu.Shl r5, r5, r15
    mov r0, r3  ; right_part
    mov r15, r5  ; left_part
    alu.Or r0, r0, r15
    halt
