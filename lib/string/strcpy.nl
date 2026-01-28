; @name: Strcpy
; @description: Copy string from src to dst.
; @category: string
; @difficulty: 1
;
; @prompt: copy string from {src} to {dst}
; @prompt: duplicate string {src} into {dst}
; @prompt: copy {src} to destination {dst}
; @prompt: clone string {src} to {dst}
; @prompt: transfer string from {src} to {dst}
; @prompt: copy contents of {src} into {dst}
; @prompt: strcpy from {src} to {dst}
; @prompt: replicate string {src} at {dst}
; @prompt: write string {src} to buffer {dst}
; @prompt: copy text from {src} to {dst}
; @prompt: duplicate {src} into {dst} buffer
; @prompt: move string {src} to {dst}
; @prompt: copy string data from {src} to {dst}
;
; @param: dst=r0 "Pointer to destination buffer"
; @param: src=r1 "Pointer to source null-terminated string"
;
; @test: r0=0x2000 r1=0x1000 [0x1000]="hello" -> r0=5
;
; @export: strcpy
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r2, r0  ; dst
    mov r3, r1  ; src
    mov r4, 0  ; 0
.loop_0:
    nop
    mov r5, r3  ; ps
    load.Byte r5, [r5]
    mov r15, r2  ; pd
    mov r14, r5  ; c
    store.Byte r14, [r15]
    mov r15, r5  ; c
    mov r14, 0  ; 0
    beq r15, r14, .set_4
    mov r15, 0
    b .cmp_end_5
.set_4:
    nop
    mov r15, 1
.cmp_end_5:
    nop
    beq r15, zero, .endif_3
    b .endloop_1
.endif_3:
    nop
    mov r15, 1  ; 1
    alu.Add r4, r4, r15
    mov r15, 1  ; 1
    alu.Add r2, r2, r15
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .loop_0
.endloop_1:
    nop
    mov r0, r4  ; len
    halt
