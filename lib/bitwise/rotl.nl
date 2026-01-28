; @name: Rotl
; @description: Rotate left.
; @category: bitwise
; @difficulty: 1
;
; @prompt: rotate {n} left by {shift} bits
; @prompt: left rotate {n} by {shift}
; @prompt: circular shift {n} left by {shift}
; @prompt: rotl {n} by {shift}
; @prompt: rotate bits of {n} left {shift} positions
; @prompt: perform left rotation on {n} by {shift} bits
; @prompt: bit rotate {n} leftward by {shift}
; @prompt: cyclically shift {n} left by {shift}
; @prompt: left bit rotation of {n} by {shift}
; @prompt: rotate {n} leftward {shift} times
; @prompt: circular left shift {n} by {shift}
; @prompt: roll bits of {n} left by {shift}
;
; @param: n=r0 "The value to rotate"
; @param: shift=r1 "Number of bits to rotate left"
;
; @test: r0=1, r1=0 -> r0=1
; @test: r0=1, r1=1 -> r0=2
; @test: r0=0x8000000000000000, r1=1 -> r0=1
; @note: Rotating by 0 returns the same value
;
; @export: rotl
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
    alu.Shl r3, r3, r15
    mov r4, 64  ; 64
    mov r15, r2  ; s
    alu.Sub r4, r4, r15
    mov r5, r0  ; n
    mov r15, r4  ; right_shift
    alu.Shr r5, r5, r15
    mov r0, r3  ; left_part
    mov r15, r5  ; right_part
    alu.Or r0, r0, r15
    halt
