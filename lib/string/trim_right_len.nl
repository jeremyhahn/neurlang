; @name: Trim Right Len
; @description: Find length of string excluding trailing whitespace.
; @category: string
; @difficulty: 2
;
; @prompt: trim trailing whitespace length from {str}
; @prompt: get length excluding trailing spaces of {str}
; @prompt: find trimmed length of {str}
; @prompt: rtrim length of {str}
; @prompt: length without trailing whitespace for {str}
; @prompt: get non-whitespace length of {str}
; @prompt: trim right length of {str}
; @prompt: skip trailing spaces length in {str}
; @prompt: remove trailing whitespace length from {str}
; @prompt: right trimmed length of string {str}
; @prompt: strip trailing spaces length from {str}
; @prompt: effective length of {str} without trailing space
; @prompt: trim end length for {str}
;
; @param: str=r0 "Pointer to null-terminated string"
;
; @test: r0=0x1000 [0x1000]="hello  " -> r0=5
;
; @export: trim_right_len
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r1, 0  ; 0
    mov r2, r0  ; ptr
.while_0:
    nop
    mov r15, r2  ; p
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
    mov r15, 1  ; 1
    alu.Add r1, r1, r15
    mov r15, 1  ; 1
    alu.Add r2, r2, r15
    b .while_0
.endwhile_1:
    nop
    mov r15, r1  ; len
    mov r14, 0  ; 0
    beq r15, r14, .set_6
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
    mov r3, r1  ; len
.while_8:
    nop
    mov r15, r3  ; end
    mov r14, 0  ; 0
    bgt r15, r14, .set_10
    mov r15, 0
    b .cmp_end_11
.set_10:
    nop
    mov r15, 1
.cmp_end_11:
    nop
    beq r15, zero, .endwhile_9
    mov r4, r0  ; ptr
    mov r15, r3  ; end
    mov r14, 1  ; 1
    alu.Sub r15, r15, r14
    alu.Add r4, r4, r15
    load.Byte r4, [r4]
    mov r14, r4  ; c
    mov r15, 13  ; 13
    bne r14, r15, .set_14
    mov r14, 0
    b .cmp_end_15
.set_14:
    nop
    mov r14, 1
.cmp_end_15:
    nop
    mov r14, r4  ; c
    mov r15, 10  ; 10
    bne r14, r15, .set_16
    mov r14, 0
    b .cmp_end_17
.set_16:
    nop
    mov r14, 1
.cmp_end_17:
    nop
    mov r14, r4  ; c
    mov r15, 9  ; 9
    bne r14, r15, .set_18
    mov r14, 0
    b .cmp_end_19
.set_18:
    nop
    mov r14, 1
.cmp_end_19:
    nop
    mov r15, r4  ; c
    mov r14, 32  ; 32
    bne r15, r14, .set_20
    mov r15, 0
    b .cmp_end_21
.set_20:
    nop
    mov r15, 1
.cmp_end_21:
    nop
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
    bne r15, zero, .set_30
    mov r15, 0
    b .cmp_end_31
.set_30:
    nop
    mov r15, 1
.cmp_end_31:
    nop
    mov r13, r14
    bne r13, zero, .set_32
    mov r13, 0
    b .cmp_end_33
.set_32:
    nop
    mov r13, 1
.cmp_end_33:
    nop
    alu.And r15, r15, r13
    beq r15, zero, .endif_13
    b .endwhile_9
.endif_13:
    nop
    mov r15, 1  ; 1
    alu.Sub r3, r3, r15
    b .while_8
.endwhile_9:
    nop
    mov r0, r3  ; end
    halt
