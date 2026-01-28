; @name: Htoi
; @description: Parse hexadecimal string to integer.
; @category: string
; @difficulty: 2
;
; @prompt: parse hex string {str}
; @prompt: convert hex {str} to integer
; @prompt: htoi of {str}
; @prompt: hexadecimal {str} to number
; @prompt: read hex value from {str}
; @prompt: decode hex string {str}
; @prompt: hex to int for {str}
; @prompt: parse hexadecimal from {str}
; @prompt: convert {str} from hex to decimal
; @prompt: extract hex number from {str}
; @prompt: interpret {str} as hexadecimal
; @prompt: hex string {str} to integer
; @prompt: parse 0x prefixed string {str}
;
; @param: str=r0 "Pointer to null-terminated hex string"
;
; @test: r0=0x1000 [0x1000]="ff" -> r0=255
; @test: r0=0x1000 [0x1000]="10" -> r0=16
; @test: r0=0x1000 [0x1000]="0" -> r0=0
;
; @export: htoi
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r1, 0  ; 0
    mov r2, r0  ; ptr
.loop_0:
    nop
    mov r3, r2  ; p
    load.Byte r3, [r3]
    mov r14, r3  ; c
    mov r15, 13  ; 13
    bne r14, r15, .set_4
    mov r14, 0
    b .cmp_end_5
.set_4:
    nop
    mov r14, 1
.cmp_end_5:
    nop
    mov r14, r3  ; c
    mov r15, 10  ; 10
    bne r14, r15, .set_6
    mov r14, 0
    b .cmp_end_7
.set_6:
    nop
    mov r14, 1
.cmp_end_7:
    nop
    mov r14, r3  ; c
    mov r15, 9  ; 9
    bne r14, r15, .set_8
    mov r14, 0
    b .cmp_end_9
.set_8:
    nop
    mov r14, 1
.cmp_end_9:
    nop
    mov r15, r3  ; c
    mov r14, 32  ; 32
    bne r15, r14, .set_10
    mov r15, 0
    b .cmp_end_11
.set_10:
    nop
    mov r15, 1
.cmp_end_11:
    nop
    bne r15, zero, .set_12
    mov r15, 0
    b .cmp_end_13
.set_12:
    nop
    mov r15, 1
.cmp_end_13:
    nop
    mov r13, r14
    bne r13, zero, .set_14
    mov r13, 0
    b .cmp_end_15
.set_14:
    nop
    mov r13, 1
.cmp_end_15:
    nop
    alu.And r15, r15, r13
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
    bne r15, zero, .set_20
    mov r15, 0
    b .cmp_end_21
.set_20:
    nop
    mov r15, 1
.cmp_end_21:
    nop
    mov r13, r14
    bne r13, zero, .set_22
    mov r13, 0
    b .cmp_end_23
.set_22:
    nop
    mov r13, 1
.cmp_end_23:
    nop
    alu.And r15, r15, r13
    beq r15, zero, .endif_3
    b .endloop_1
.endif_3:
    nop
    mov r15, 1  ; 1
    alu.Add r2, r2, r15
    b .loop_0
.endloop_1:
    nop
    mov r15, r2  ; p
    mov r14, 1  ; 1
    alu.Add r15, r15, r14
    load.Byte r15, [r15]
    mov r14, 88  ; 88
    beq r15, r14, .set_26
    mov r15, 0
    b .cmp_end_27
.set_26:
    nop
    mov r15, 1
.cmp_end_27:
    nop
    mov r14, r2  ; p
    mov r15, 1  ; 1
    alu.Add r14, r14, r15
    load.Byte r14, [r14]
    mov r15, 120  ; 120
    beq r14, r15, .set_28
    mov r14, 0
    b .cmp_end_29
.set_28:
    nop
    mov r14, 1
.cmp_end_29:
    nop
    alu.Or r14, r14, r15
    bne r14, zero, .set_30
    mov r14, 0
    b .cmp_end_31
.set_30:
    nop
    mov r14, 1
.cmp_end_31:
    nop
    mov r15, r2  ; p
    load.Byte r15, [r15]
    mov r14, 48  ; 48
    beq r15, r14, .set_32
    mov r15, 0
    b .cmp_end_33
.set_32:
    nop
    mov r15, 1
.cmp_end_33:
    nop
    bne r15, zero, .set_34
    mov r15, 0
    b .cmp_end_35
.set_34:
    nop
    mov r15, 1
.cmp_end_35:
    nop
    mov r13, r14
    bne r13, zero, .set_36
    mov r13, 0
    b .cmp_end_37
.set_36:
    nop
    mov r13, 1
.cmp_end_37:
    nop
    alu.And r15, r15, r13
    beq r15, zero, .endif_25
    mov r15, 2  ; 2
    alu.Add r2, r2, r15
.endif_25:
    nop
.loop_38:
    nop
    mov r4, r2  ; p
    load.Byte r4, [r4]
    ; Check if c is digit '0'-'9' (48 <= c <= 57)
    mov r15, r4  ; c
    mov r14, 48  ; '0'
    blt r15, r14, .try_lower_hex  ; c < '0', not a digit
    mov r15, r4  ; c
    mov r14, 57  ; '9'
    bgt r15, r14, .try_lower_hex  ; c > '9', not a digit
    ; c is a decimal digit
    b .is_dec_digit
.try_lower_hex:
    nop
    b .else_40
.is_dec_digit:
    nop
    ; Digit 0-9: digit = c - 48
    mov r5, r4  ; c
    mov r15, 48  ; 48
    alu.Sub r5, r5, r15  ; r5 = digit value
    mov r15, 16  ; 16
    muldiv.Mul r1, r1, r15  ; r1 = r1 * 16
    alu.Add r1, r1, r5  ; r1 = r1 + digit
    b .endif_41
.else_40:
    nop
    mov r14, r4  ; c
    mov r15, 102  ; 102
    ble r14, r15, .set_52
    mov r14, 0
    b .cmp_end_53
.set_52:
    nop
    mov r14, 1
.cmp_end_53:
    nop
    mov r15, r4  ; c
    mov r14, 97  ; 97
    bge r15, r14, .set_54
    mov r15, 0
    b .cmp_end_55
.set_54:
    nop
    mov r15, 1
.cmp_end_55:
    nop
    bne r15, zero, .set_56
    mov r15, 0
    b .cmp_end_57
.set_56:
    nop
    mov r15, 1
.cmp_end_57:
    nop
    mov r13, r14
    bne r13, zero, .set_58
    mov r13, 0
    b .cmp_end_59
.set_58:
    nop
    mov r13, 1
.cmp_end_59:
    nop
    alu.And r15, r15, r13
    beq r15, zero, .else_50
    ; Lowercase a-f: digit = c - 97 + 10
    mov r5, r4  ; c
    mov r15, 97  ; 'a'
    alu.Sub r5, r5, r15  ; r5 = c - 'a'
    mov r15, 10  ; 10
    alu.Add r5, r5, r15  ; r5 = digit value (10-15)
    mov r15, 16  ; 16
    muldiv.Mul r1, r1, r15  ; r1 = r1 * 16
    alu.Add r1, r1, r5  ; r1 = r1 + digit
    b .endif_51
.else_50:
    nop
    mov r14, r4  ; c
    mov r15, 70  ; 70
    ble r14, r15, .set_62
    mov r14, 0
    b .cmp_end_63
.set_62:
    nop
    mov r14, 1
.cmp_end_63:
    nop
    mov r15, r4  ; c
    mov r14, 65  ; 65
    bge r15, r14, .set_64
    mov r15, 0
    b .cmp_end_65
.set_64:
    nop
    mov r15, 1
.cmp_end_65:
    nop
    bne r15, zero, .set_66
    mov r15, 0
    b .cmp_end_67
.set_66:
    nop
    mov r15, 1
.cmp_end_67:
    nop
    mov r13, r14
    bne r13, zero, .set_68
    mov r13, 0
    b .cmp_end_69
.set_68:
    nop
    mov r13, 1
.cmp_end_69:
    nop
    alu.And r15, r15, r13
    beq r15, zero, .else_60
    ; Uppercase A-F: digit = c - 65 + 10
    mov r5, r4  ; c
    mov r15, 65  ; 'A'
    alu.Sub r5, r5, r15  ; r5 = c - 'A'
    mov r15, 10  ; 10
    alu.Add r5, r5, r15  ; r5 = digit value (10-15)
    mov r15, 16  ; 16
    muldiv.Mul r1, r1, r15  ; r1 = r1 * 16
    alu.Add r1, r1, r5  ; r1 = r1 + digit
    b .endif_61
.else_60:
    nop
    b .endloop_39
.endif_61:
    nop
.endif_51:
    nop
.endif_41:
    nop
    mov r15, 1  ; 1
    alu.Add r2, r2, r15
    b .loop_38
.endloop_39:
    nop
    mov r0, r1  ; result
    halt
