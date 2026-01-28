; @name: Itoa Hex
; @description: Convert integer to hexadecimal string.
; @category: string
; @difficulty: 2
;
; @prompt: convert {n} to hex string at {dst}
; @prompt: itoa hex of {n} into {dst}
; @prompt: write number {n} as hex to {dst}
; @prompt: integer {n} to hex string at {dst}
; @prompt: format number {n} as hexadecimal to {dst}
; @prompt: convert {n} to hexadecimal string in {dst}
; @prompt: stringify integer {n} as hex at {dst}
; @prompt: render number {n} as hex text to {dst}
; @prompt: int to hex string for {n} into {dst}
; @prompt: write integer {n} as hex to buffer {dst}
; @prompt: transform {n} to hex string at {dst}
; @prompt: encode integer {n} as hex in {dst}
; @prompt: number to hex conversion of {n} to {dst}
;
; @param: n=r0 "Unsigned integer to convert"
; @param: dst=r1 "Pointer to destination buffer"
;
; @test: r0=0 r1=0x2000 -> r0=1
; @test: r0=255 r1=0x2000 -> r0=2
; @test: r0=16 r1=0x2000 -> r0=2
;
; @export: itoa_hex
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; n
    mov r14, 0  ; 0
    beq r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endif_1
    mov r15, r1  ; dst
    mov r14, 48  ; 48
    store.Byte r14, [r15]
    mov r15, r1  ; dst
    mov r14, 1  ; 1
    alu.Add r15, r15, r14
    mov r14, 0  ; 0
    store.Byte r14, [r15]
    mov r0, 1  ; 1
    halt
.endif_1:
    nop
    mov r2, r0  ; n
    mov r3, 0  ; 0
.while_4:
    nop
    mov r15, r2  ; temp
    mov r14, 0  ; 0
    bgt r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endwhile_5
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    mov r15, 4  ; 4
    alu.Shr r2, r2, r15
    b .while_4
.endwhile_5:
    nop
    mov r4, r3  ; len
.while_8:
    nop
    mov r15, r0  ; n
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
    mov r15, 1  ; 1
    alu.Sub r4, r4, r15
    mov r5, r0  ; n
    mov r15, 15  ; 15
    alu.And r5, r5, r15
    mov r15, r5  ; digit
    mov r14, 10  ; 10
    blt r15, r14, .set_14
    mov r15, 0
    b .cmp_end_15
.set_14:
    nop
    mov r15, 1
.cmp_end_15:
    nop
    beq r15, zero, .cond_else_12
    mov r6, 48  ; 48
    mov r15, r5  ; digit
    alu.Add r6, r6, r15
    b .cond_end_13
.cond_else_12:
    nop
    mov r6, 97  ; 97
    mov r15, r5  ; digit
    alu.Add r6, r6, r15
    mov r15, 10  ; 10
    alu.Sub r6, r6, r15
.cond_end_13:
    nop
    mov r15, r1  ; dst
    mov r14, r4  ; i
    alu.Add r15, r15, r14
    mov r14, r6  ; c
    store.Byte r14, [r15]
    mov r15, 4  ; 4
    alu.Shr r0, r0, r15
    b .while_8
.endwhile_9:
    nop
    mov r15, r1  ; dst
    mov r14, r3  ; len
    alu.Add r15, r15, r14
    mov r14, 0  ; 0
    store.Byte r14, [r15]
    mov r0, r3  ; len
    halt
