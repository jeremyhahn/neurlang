; @name: Is Space
; @description: Check if character is whitespace.
; @category: string
; @difficulty: 1
;
; @prompt: is {c} whitespace
; @prompt: check if {c} is a space character
; @prompt: is character {c} whitespace
; @prompt: test if {c} is space or tab
; @prompt: determine if {c} is whitespace
; @prompt: is {c} a blank character
; @prompt: check whether {c} is whitespace
; @prompt: is char {c} space, tab, or newline
; @prompt: verify {c} is whitespace
; @prompt: does {c} represent whitespace
; @prompt: isspace for {c}
; @prompt: is {c} a spacing character
; @prompt: check {c} is whitespace character
;
; @param: c=r0 "ASCII character to check"
;
; @test: r0=32 -> r0=1
; @test: r0=9 -> r0=1
; @test: r0=65 -> r0=0
;
; @export: is_space
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; c
    mov r14, 32  ; 32
    beq r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endif_1
    mov r0, 1  ; 1
    halt
.endif_1:
    nop
    mov r15, r0  ; c
    mov r14, 9  ; 9
    beq r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endif_5
    mov r0, 1  ; 1
    halt
.endif_5:
    nop
    mov r15, r0  ; c
    mov r14, 10  ; 10
    beq r15, r14, .set_10
    mov r15, 0
    b .cmp_end_11
.set_10:
    nop
    mov r15, 1
.cmp_end_11:
    nop
    beq r15, zero, .endif_9
    mov r0, 1  ; 1
    halt
.endif_9:
    nop
    mov r15, r0  ; c
    mov r14, 13  ; 13
    beq r15, r14, .set_14
    mov r15, 0
    b .cmp_end_15
.set_14:
    nop
    mov r15, 1
.cmp_end_15:
    nop
    beq r15, zero, .endif_13
    mov r0, 1  ; 1
    halt
.endif_13:
    nop
    mov r0, 0  ; 0
    halt
