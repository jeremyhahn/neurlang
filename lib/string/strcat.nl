; @name: Strcat
; @description: Concatenate src string to end of dst.
; @category: string
; @difficulty: 2
;
; @prompt: concatenate {src} to {dst}
; @prompt: append {src} to {dst}
; @prompt: join {src} onto {dst}
; @prompt: add {src} to end of {dst}
; @prompt: strcat {src} to {dst}
; @prompt: combine {dst} and {src}
; @prompt: append string {src} to buffer {dst}
; @prompt: concatenate strings {dst} and {src}
; @prompt: extend {dst} with {src}
; @prompt: add string {src} after {dst}
; @prompt: join strings {dst} and {src}
; @prompt: merge {src} into {dst}
; @prompt: append text {src} to {dst}
;
; @param: dst=r0 "Pointer to destination buffer (existing string)"
; @param: src=r1 "Pointer to source string to append"
;
; @test: r0=0x2000 r1=0x1000 [0x2000]="hello" [0x1000]=" world" -> r0=11
;
; @export: strcat
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r2, 0  ; 0
    mov r3, r0  ; dst
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
    mov r4, 0  ; 0
    mov r3, r1  ; src
.while_4:
    nop
    mov r15, r3  ; p
    load.Byte r15, [r15]
    mov r14, 0  ; 0
    bne r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endwhile_5
    mov r15, 1  ; 1
    alu.Add r4, r4, r15
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .while_4
.endwhile_5:
    nop
    mov r5, 0  ; 0
.while_8:
    nop
    mov r15, r5  ; i
    mov r14, r4  ; src_len
    ble r15, r14, .set_10
    mov r15, 0
    b .cmp_end_11
.set_10:
    nop
    mov r15, 1
.cmp_end_11:
    nop
    beq r15, zero, .endwhile_9
    mov r15, r0  ; dst
    mov r14, r2  ; dst_len
    alu.Add r14, r14, r5  ; dst_len + i
    alu.Add r15, r15, r14 ; dst + dst_len + i
    mov r14, r1  ; src
    alu.Add r14, r14, r5  ; src + i
    load.Byte r14, [r14]
    store.Byte r14, [r15]
    mov r15, 1  ; 1
    alu.Add r5, r5, r15
    b .while_8
.endwhile_9:
    nop
    mov r0, r2  ; dst_len
    mov r15, r4  ; src_len
    alu.Add r0, r0, r15
    halt
