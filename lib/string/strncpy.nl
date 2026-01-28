; @name: Strncpy
; @description: Copy at most n characters from src to dst.
; @category: string
; @difficulty: 2
;
; @prompt: copy {n} characters from {src} to {dst}
; @prompt: strncpy {n} bytes from {src} to {dst}
; @prompt: copy at most {n} chars from {src} to {dst}
; @prompt: limited copy of {src} to {dst} with max {n}
; @prompt: copy up to {n} characters from {src} to {dst}
; @prompt: bounded string copy from {src} to {dst} limit {n}
; @prompt: copy string {src} to {dst} max length {n}
; @prompt: safe copy {n} chars from {src} to {dst}
; @prompt: copy first {n} characters of {src} to {dst}
; @prompt: truncated copy from {src} to {dst} at {n}
; @prompt: copy {src} to {dst} with limit {n}
; @prompt: transfer up to {n} bytes from {src} to {dst}
; @prompt: partial string copy {src} to {dst} max {n}
;
; @param: dst=r0 "Pointer to destination buffer"
; @param: src=r1 "Pointer to source string"
; @param: n=r2 "Maximum number of characters to copy"
;
; @test: r0=0x2000 r1=0x1000 r2=5 [0x1000]="hello" -> r0=5
; @test: r0=0x2000 r1=0x1000 r2=3 [0x1000]="hello" -> r0=3
; @test: r0=0x2000 r1=0x1000 r2=10 [0x1000]="hi" -> r0=2
;
; @export: strncpy
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r3, 0  ; 0
.while_0:
    nop
    mov r15, r3  ; i
    mov r14, r2  ; n
    blt r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endwhile_1
    mov r4, r1  ; src
    mov r15, r3  ; i
    alu.Add r4, r4, r15
    load.Byte r4, [r4]
    mov r15, r0  ; dst
    mov r14, r3  ; i
    alu.Add r15, r15, r14
    mov r14, r4  ; c
    store.Byte r14, [r15]
    mov r15, r4  ; c
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
    mov r0, r3  ; i
    halt
.endif_5:
    nop
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .while_0
.endwhile_1:
    nop
    mov r15, r2  ; n
    mov r14, 0  ; 0
    bgt r15, r14, .set_10
    mov r15, 0
    b .cmp_end_11
.set_10:
    nop
    mov r15, 1
.cmp_end_11:
    nop
    beq r15, zero, .endif_9
    mov r15, r0  ; dst
    mov r14, r2  ; n
    mov r13, 1  ; 1
    alu.Sub r14, r14, r13
    alu.Add r15, r15, r14
    mov r14, 0  ; 0
    store.Byte r14, [r15]
.endif_9:
    nop
    mov r0, r3  ; i
    halt
