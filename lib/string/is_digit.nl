; @name: Is Digit
; @description: Check if character is a digit.
; @category: string
; @difficulty: 1
;
; @prompt: is {c} a digit
; @prompt: check if {c} is numeric
; @prompt: is character {c} a number
; @prompt: test if {c} is a digit character
; @prompt: determine if {c} is 0-9
; @prompt: is {c} a numeric character
; @prompt: check whether {c} is a digit
; @prompt: is char {c} in range 0 to 9
; @prompt: verify {c} is a digit
; @prompt: does {c} represent a number
; @prompt: isdigit for {c}
; @prompt: is {c} between 0 and 9
; @prompt: check {c} is numeric digit
;
; @param: c=r0 "ASCII character to check"
;
; @test: r0=48 -> r0=1
; @test: r0=57 -> r0=1
; @test: r0=65 -> r0=0
; @test: r0=32 -> r0=0
;
; @export: is_digit
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; c
    mov r14, 48  ; 48
    blt r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endif_1
    mov r0, 0  ; 0
    halt
.endif_1:
    nop
    mov r15, r0  ; c
    mov r14, 57  ; 57
    bgt r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endif_5
    mov r0, 0  ; 0
    halt
.endif_5:
    nop
    mov r0, 1  ; 1
    halt
