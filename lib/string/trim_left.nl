; @name: Trim Left
; @description: Trim leading whitespace, returns pointer offset.
; @category: string
; @difficulty: 2
;
; @prompt: trim leading whitespace from {str}
; @prompt: skip whitespace at start of {str}
; @prompt: remove leading spaces from {str}
; @prompt: ltrim {str}
; @prompt: strip leading whitespace from {str}
; @prompt: get offset past leading spaces in {str}
; @prompt: find first non-whitespace in {str}
; @prompt: trim left side of {str}
; @prompt: skip spaces at beginning of {str}
; @prompt: remove spaces from start of {str}
; @prompt: left trim whitespace in {str}
; @prompt: trim start of string {str}
; @prompt: strip spaces from left of {str}
;
; @param: str=r0 "Pointer to null-terminated string"
;
; @test: r0=0x1000 [0x1000]="  hello" -> r0=2
;
; @export: trim_left
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r1, 0  ; 0
.while_0:
    nop
    mov r15, r0  ; ptr
    mov r14, r1  ; offset
    alu.Add r15, r15, r14
    load.Byte r15, [r15]
    mov r14, 0  ; 0
    bne r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endwhile_1
    mov r2, r0  ; ptr
    mov r15, r1  ; offset
    alu.Add r2, r2, r15
    load.Byte r2, [r2]
    mov r14, r2  ; c
    mov r15, 13  ; 13
    bne r14, r15, .set_6
    mov r14, 0
    b .cmp_end_7
.set_6:
    nop
    mov r14, 1
.cmp_end_7:
    nop
    mov r14, r2  ; c
    mov r15, 10  ; 10
    bne r14, r15, .set_8
    mov r14, 0
    b .cmp_end_9
.set_8:
    nop
    mov r14, 1
.cmp_end_9:
    nop
    mov r14, r2  ; c
    mov r15, 9  ; 9
    bne r14, r15, .set_10
    mov r14, 0
    b .cmp_end_11
.set_10:
    nop
    mov r14, 1
.cmp_end_11:
    nop
    mov r15, r2  ; c
    mov r14, 32  ; 32
    bne r15, r14, .set_12
    mov r15, 0
    b .cmp_end_13
.set_12:
    nop
    mov r15, 1
.cmp_end_13:
    nop
    bne r15, zero, .set_14
    mov r15, 0
    b .cmp_end_15
.set_14:
    nop
    mov r15, 1
.cmp_end_15:
    nop
    mov r13, r14
    bne r13, zero, .set_16
    mov r13, 0
    b .cmp_end_17
.set_16:
    nop
    mov r13, 1
.cmp_end_17:
    nop
    alu.And r15, r15, r13
    bne r15, zero, .set_18
    mov r15, 0
    b .cmp_end_19
.set_18:
    nop
    mov r15, 1
.cmp_end_19:
    nop
    mov r13, r14
    bne r13, zero, .set_20
    mov r13, 0
    b .cmp_end_21
.set_20:
    nop
    mov r13, 1
.cmp_end_21:
    nop
    alu.And r15, r15, r13
    bne r15, zero, .set_22
    mov r15, 0
    b .cmp_end_23
.set_22:
    nop
    mov r15, 1
.cmp_end_23:
    nop
    mov r13, r14
    bne r13, zero, .set_24
    mov r13, 0
    b .cmp_end_25
.set_24:
    nop
    mov r13, 1
.cmp_end_25:
    nop
    alu.And r15, r15, r13
    beq r15, zero, .endif_5
    b .endwhile_1
.endif_5:
    nop
    mov r15, 1  ; 1
    alu.Add r1, r1, r15
    b .while_0
.endwhile_1:
    nop
    mov r0, r1  ; offset
    halt
