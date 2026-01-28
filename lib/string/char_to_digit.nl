; @name: Char To Digit
; @description: Convert ASCII digit character to integer value.
; @category: string
; @difficulty: 1
;
; @prompt: convert character {c} to digit
; @prompt: get numeric value of char {c}
; @prompt: parse digit character {c}
; @prompt: char {c} to integer
; @prompt: convert {c} from ascii digit to number
; @prompt: get digit value of {c}
; @prompt: transform character {c} to integer
; @prompt: extract numeric value from {c}
; @prompt: ascii digit {c} to number
; @prompt: convert ascii {c} to its numeric value
; @prompt: parse character {c} as digit
; @prompt: get the number value of {c}
; @prompt: char to digit for {c}
;
; @param: c=r0 "ASCII character to convert"
;
; @test: r0=48 -> r0=0
; @test: r0=53 -> r0=5
; @test: r0=57 -> r0=9
;
; @export: char_to_digit
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
    beq r15, zero, .else_0
    mov r15, 48  ; 48
    alu.Sub r0, r0, r15
    b .endif_1
.else_0:
    nop
    mov r0, -1  ; 18446744073709551615
.endif_1:
    nop
    halt
