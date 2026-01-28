; @name: Atoi
; @description: Parse unsigned integer from string (atoi).
; @category: string
; @difficulty: 2
;
; @prompt: parse integer from {str}
; @prompt: convert string {str} to number
; @prompt: atoi of {str}
; @prompt: string {str} to integer
; @prompt: extract number from {str}
; @prompt: parse number from string {str}
; @prompt: convert {str} to unsigned integer
; @prompt: read integer from {str}
; @prompt: get numeric value of {str}
; @prompt: transform {str} to integer
; @prompt: decode integer from {str}
; @prompt: string to int for {str}
; @prompt: parse unsigned number from {str}
;
; @param: str=r0 "Pointer to null-terminated string containing digits"
;
; @test: r0=0x1000 [0x1000]="5" -> r0=5
; @test: r0=0x1000 [0x1000]="123" -> r0=123
; @test: r0=0x1000 [0x1000]="0" -> r0=0
;
; @export: atoi
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
.loop_24:
    nop
    mov r4, r2  ; p
    load.Byte r4, [r4]
    ; Check if c > 57 (not a digit)
    mov r14, r4  ; c
    mov r15, 57  ; '9'
    bgt r14, r15, .not_digit
    ; Check if c < 48 (not a digit)
    mov r15, r4  ; c
    mov r14, 48  ; '0'
    blt r15, r14, .not_digit
    ; c is a digit (48 <= c <= 57)
    b .is_digit
.not_digit:
    nop
    b .endloop_25
.is_digit:
    nop
    mov r5, r4  ; c
    mov r15, 48  ; 48
    alu.Sub r5, r5, r15
    mov r15, 10  ; 10
    muldiv.Mul r1, r1, r15
    mov r15, r5  ; digit
    alu.Add r1, r1, r15
    mov r15, 1  ; 1
    alu.Add r2, r2, r15
    b .loop_24
.endloop_25:
    nop
    mov r0, r1  ; result
    halt
