; @name: Atoi Signed
; @description: Parse signed integer from string.
; @category: string
; @difficulty: 2
;
; @prompt: parse signed integer from {str}
; @prompt: convert string {str} to signed number
; @prompt: atoi signed of {str}
; @prompt: string {str} to signed int
; @prompt: extract signed number from {str}
; @prompt: parse negative number from {str}
; @prompt: convert {str} to signed integer
; @prompt: read signed integer from {str}
; @prompt: get signed numeric value of {str}
; @prompt: parse integer with sign from {str}
; @prompt: decode signed integer from {str}
; @prompt: string to signed int for {str}
; @prompt: parse positive or negative from {str}
;
; @param: str=r0 "Pointer to null-terminated string with optional sign"
;
; @test: r0=0x1000 [0x1000]="456" -> r0=456
; @test: r0=0x1000 [0x1000]="0" -> r0=0
; @test: r0=0x1000 [0x1000]="-123" -> r0=-123
;
; @export: atoi_signed
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r1, r0  ; ptr
    mov r2, 0
.loop_0:
    nop
    mov r3, r1  ; p
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
    alu.Add r1, r1, r15
    b .loop_0
.endloop_1:
    nop
    mov r15, r1  ; p
    load.Byte r15, [r15]
    mov r14, 45  ; 45
    beq r15, r14, .set_26
    mov r15, 0
    b .cmp_end_27
.set_26:
    nop
    mov r15, 1
.cmp_end_27:
    nop
    beq r15, zero, .else_24
    mov r2, 1
    mov r15, 1  ; 1
    alu.Add r1, r1, r15
    b .endif_25
.else_24:
    nop
    mov r15, r1  ; p
    load.Byte r15, [r15]
    mov r14, 43  ; 43
    beq r15, r14, .set_30
    mov r15, 0
    b .cmp_end_31
.set_30:
    nop
    mov r15, 1
.cmp_end_31:
    nop
    beq r15, zero, .endif_29
    mov r15, 1  ; 1
    alu.Add r1, r1, r15
.endif_29:
    nop
.endif_25:
    nop
    mov r4, 0  ; 0
.loop_32:
    nop
    mov r5, r1  ; p
    load.Byte r5, [r5]
    ; Check if c > 57 (not a digit)
    mov r14, r5  ; c
    mov r15, 57  ; '9'
    bgt r14, r15, .not_digit_s
    ; Check if c < 48 (not a digit)
    mov r15, r5  ; c
    mov r14, 48  ; '0'
    blt r15, r14, .not_digit_s
    ; c is a digit (48 <= c <= 57)
    b .is_digit_s
.not_digit_s:
    nop
    b .endloop_33
.is_digit_s:
    nop
    mov r6, r5  ; c
    mov r15, 48  ; 48
    alu.Sub r6, r6, r15
    mov r15, 10  ; 10
    muldiv.Mul r4, r4, r15
    mov r15, r6  ; digit
    alu.Add r4, r4, r15
    mov r15, 1  ; 1
    alu.Add r1, r1, r15
    b .loop_32
.endloop_33:
    nop
    mov r15, r2  ; negative
    beq r15, zero, .else_42
    mov r0, r4  ; result
    alui.Xor r0, r0, -1
    mov r15, 1  ; 1
    alu.Add r0, r0, r15
    b .endif_43
.else_42:
    nop
    mov r0, r4  ; result
.endif_43:
    nop
    halt
