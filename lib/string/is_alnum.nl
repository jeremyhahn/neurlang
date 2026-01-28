; @name: Is Alnum
; @description: Check if character is alphanumeric.
; @category: string
; @difficulty: 1
;
; @prompt: is {c} alphanumeric
; @prompt: check if {c} is letter or digit
; @prompt: is character {c} alphanumeric
; @prompt: test if {c} is alnum
; @prompt: determine if {c} is a-z, A-Z, or 0-9
; @prompt: is {c} a letter or number
; @prompt: check whether {c} is alphanumeric
; @prompt: is char {c} letter or digit
; @prompt: verify {c} is alphanumeric
; @prompt: does {c} represent letter or number
; @prompt: isalnum for {c}
; @prompt: is {c} word character
; @prompt: check {c} is alphanumeric character
;
; @param: c=r0 "ASCII character to check"
;
; @test: r0=65 -> r0=1
; @test: r0=48 -> r0=1
; @test: r0=32 -> r0=0
;
; @export: is_alnum
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r14, r0  ; c
    mov r15, 57  ; 57
    ble r14, r15, .set_2
    mov r14, 0
    b .cmp_end_3
.set_2:
    nop
    mov r14, 1
.cmp_end_3:
    nop
    mov r15, r0  ; c
    mov r14, 48  ; 48
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
    mov r15, 122  ; 122
    ble r14, r15, .set_12
    mov r14, 0
    b .cmp_end_13
.set_12:
    nop
    mov r14, 1
.cmp_end_13:
    nop
    mov r15, r0  ; c
    mov r14, 97  ; 97
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
    mov r14, r0  ; c
    mov r15, 90  ; 90
    ble r14, r15, .set_22
    mov r14, 0
    b .cmp_end_23
.set_22:
    nop
    mov r14, 1
.cmp_end_23:
    nop
    mov r15, r0  ; c
    mov r14, 65  ; 65
    bge r15, r14, .set_24
    mov r15, 0
    b .cmp_end_25
.set_24:
    nop
    mov r15, 1
.cmp_end_25:
    nop
    bne r15, zero, .set_26
    mov r15, 0
    b .cmp_end_27
.set_26:
    nop
    mov r15, 1
.cmp_end_27:
    nop
    mov r13, r14
    bne r13, zero, .set_28
    mov r13, 0
    b .cmp_end_29
.set_28:
    nop
    mov r13, 1
.cmp_end_29:
    nop
    alu.And r15, r15, r13
    beq r15, zero, .endif_21
    mov r0, 1  ; 1
    halt
.endif_21:
    nop
    mov r0, 0  ; 0
    halt
