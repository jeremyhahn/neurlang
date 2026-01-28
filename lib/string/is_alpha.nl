; @name: Is Alpha
; @description: Check if character is alphabetic.
; @category: string
; @difficulty: 1
;
; @prompt: is {c} a letter
; @prompt: check if {c} is alphabetic
; @prompt: is character {c} a letter
; @prompt: test if {c} is alpha
; @prompt: determine if {c} is a-z or A-Z
; @prompt: is {c} an alphabetic character
; @prompt: check whether {c} is a letter
; @prompt: is char {c} alphabetic
; @prompt: verify {c} is a letter
; @prompt: does {c} represent a letter
; @prompt: isalpha for {c}
; @prompt: is {c} in the alphabet
; @prompt: check {c} is a letter character
;
; @param: c=r0 "ASCII character to check"
;
; @test: r0=65 -> r0=1
; @test: r0=90 -> r0=1
; @test: r0=97 -> r0=1
; @test: r0=48 -> r0=0
;
; @export: is_alpha
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r14, r0  ; c
    mov r15, 122  ; 122
    ble r14, r15, .set_2
    mov r14, 0
    b .cmp_end_3
.set_2:
    nop
    mov r14, 1
.cmp_end_3:
    nop
    mov r15, r0  ; c
    mov r14, 97  ; 97
    bge r15, r14, .set_4
    mov r15, 0
    b .cmp_end_5
.set_4:
    nop
    mov r15, 1
.cmp_end_5:
    nop
    bne r15, zero, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    mov r13, r14
    bne r13, zero, .set_8
    mov r13, 0
    b .cmp_end_9
.set_8:
    nop
    mov r13, 1
.cmp_end_9:
    nop
    alu.And r15, r15, r13
    beq r15, zero, .endif_1
    mov r0, 1  ; 1
    halt
.endif_1:
    nop
    mov r14, r0  ; c
    mov r15, 90  ; 90
    ble r14, r15, .set_12
    mov r14, 0
    b .cmp_end_13
.set_12:
    nop
    mov r14, 1
.cmp_end_13:
    nop
    mov r15, r0  ; c
    mov r14, 65  ; 65
    bge r15, r14, .set_14
    mov r15, 0
    b .cmp_end_15
.set_14:
    nop
    mov r15, 1
.cmp_end_15:
    nop
    bne r15, zero, .set_16
    mov r15, 0
    b .cmp_end_17
.set_16:
    nop
    mov r15, 1
.cmp_end_17:
    nop
    mov r13, r14
    bne r13, zero, .set_18
    mov r13, 0
    b .cmp_end_19
.set_18:
    nop
    mov r13, 1
.cmp_end_19:
    nop
    alu.And r15, r15, r13
    beq r15, zero, .endif_11
    mov r0, 1  ; 1
    halt
.endif_11:
    nop
    mov r0, 0  ; 0
    halt
