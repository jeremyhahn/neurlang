; @name: Strstr
; @description: Find substring in string (naive algorithm).
; @category: string
; @difficulty: 3
;
; @prompt: find {needle} in {haystack}
; @prompt: locate substring {needle} in {haystack}
; @prompt: search for {needle} in {haystack}
; @prompt: get index of {needle} in {haystack}
; @prompt: position of substring {needle} in {haystack}
; @prompt: where is {needle} in {haystack}
; @prompt: find first occurrence of {needle} in {haystack}
; @prompt: strstr for {needle} in {haystack}
; @prompt: index of substring {needle} in {haystack}
; @prompt: search string {haystack} for {needle}
; @prompt: find position of {needle} in {haystack}
; @prompt: locate string {needle} within {haystack}
; @prompt: find pattern {needle} in text {haystack}
;
; @param: haystack=r0 "Pointer to string to search in"
; @param: needle=r1 "Pointer to substring to find"
;
; @test: r0=0x1000 r1=0x1100 [0x1000]="hello world" [0x1100]="world" -> r0=6
; @test: r0=0x1000 r1=0x1100 [0x1000]="hello" [0x1100]="xyz" -> r0=-1
; @test: r0=0x1000 r1=0x1100 [0x1000]="hello" [0x1100]="" -> r0=0
;
; @export: strstr
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r2, 0  ; 0
    mov r3, r1  ; needle
.while_0:
    nop
    mov r15, r3  ; p
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
    alu.Add r2, r2, r15
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .while_0
.endwhile_1:
    nop
    mov r15, r2  ; needle_len
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
    mov r4, 0  ; 0
    mov r3, r0  ; haystack
.while_8:
    nop
    mov r15, r3  ; p
    load.Byte r15, [r15]
    mov r14, 0  ; 0
    bne r15, r14, .set_10
    mov r15, 0
    b .cmp_end_11
.set_10:
    nop
    mov r15, 1
.cmp_end_11:
    nop
    beq r15, zero, .endwhile_9
    mov r15, 1  ; 1
    alu.Add r4, r4, r15
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .while_8
.endwhile_9:
    nop
    mov r15, r2  ; needle_len
    mov r14, r4  ; haystack_len
    bgt r15, r14, .set_14
    mov r15, 0
    b .cmp_end_15
.set_14:
    nop
    mov r15, 1
.cmp_end_15:
    nop
    beq r15, zero, .endif_13
    mov r0, -1  ; 18446744073709551615
    halt
.endif_13:
    nop
    mov r5, 0  ; 0
    mov r6, r4  ; haystack_len
    mov r15, r2  ; needle_len
    alu.Sub r6, r6, r15
.while_16:
    nop
    mov r15, r5  ; i
    mov r14, r6  ; max_start
    ble r15, r14, .set_18
    mov r15, 0
    b .cmp_end_19
.set_18:
    nop
    mov r15, 1
.cmp_end_19:
    nop
    beq r15, zero, .endwhile_17
    mov r7, 0  ; 0
    mov r8, 1  ; 1
.while_20:
    nop
    mov r15, r7  ; j
    mov r14, r2  ; needle_len
    blt r15, r14, .set_22
    mov r15, 0
    b .cmp_end_23
.set_22:
    nop
    mov r15, 1
.cmp_end_23:
    nop
    beq r15, zero, .endwhile_21
    ; Load needle[j]
    mov r14, r1  ; needle
    alu.Add r14, r14, r7  ; needle + j
    load.Byte r14, [r14]  ; r14 = needle[j]
    ; Load haystack[i + j]
    mov r15, r0  ; haystack
    mov r13, r5  ; i
    alu.Add r13, r13, r7  ; i + j
    alu.Add r15, r15, r13  ; haystack + i + j
    load.Byte r15, [r15]  ; r15 = haystack[i + j]
    ; Compare
    bne r15, r14, .set_26
    mov r15, 0
    b .cmp_end_27
.set_26:
    nop
    mov r15, 1
.cmp_end_27:
    nop
    beq r15, zero, .endif_25
    mov r8, 0  ; 0
    b .endwhile_21
.endif_25:
    nop
    mov r15, 1  ; 1
    alu.Add r7, r7, r15
    b .while_20
.endwhile_21:
    nop
    mov r15, r8  ; matched
    mov r14, 1  ; 1
    beq r15, r14, .set_30
    mov r15, 0
    b .cmp_end_31
.set_30:
    nop
    mov r15, 1
.cmp_end_31:
    nop
    beq r15, zero, .endif_29
    mov r0, r5  ; i
    halt
.endif_29:
    nop
    mov r15, 1  ; 1
    alu.Add r5, r5, r15
    b .while_16
.endwhile_17:
    nop
    mov r0, -1  ; 18446744073709551615
    halt
