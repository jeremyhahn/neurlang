; @name: Copy
; @description: Copy array from src to dst.
; @category: array
; @difficulty: 1
;
; @prompt: copy {len} elements from {src} to {dst}
; @prompt: duplicate array {src} into {dst} for {len} items
; @prompt: memcpy {len} u64 values from {src} to {dst}
; @prompt: copy array {src} to destination {dst} with {len} elements
; @prompt: transfer {len} elements from {src} array to {dst}
; @prompt: clone {src} into {dst} for {len} values
; @prompt: copy {len} entries from source {src} to dest {dst}
; @prompt: replicate {src} array to {dst} with {len} items
; @prompt: copy {len} element buffer from {src} to {dst}
; @prompt: array copy from {src} to {dst} of size {len}
; @prompt: move {len} values from {src} into {dst} array
; @prompt: duplicate {len} u64 elements from {src} to {dst}
;
; @param: dst=r0 "Pointer to destination array (mutable)"
; @param: src=r1 "Pointer to source array"
; @param: len=r2 "Number of elements to copy"
;
; @test: r0=0, r1=0, r2=0 -> r0=0
; @note: Copies len elements from src to dst
;
; @export: copy
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r3, 0  ; 0
.while_0:
    nop
    mov r15, r3  ; i
    mov r14, r2  ; len
    blt r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endwhile_1
    mov r15, r0  ; dst
    mov r14, r3  ; i
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r1  ; src
    mov r15, r3  ; i
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    store.Double r14, [r15]
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .while_0
.endwhile_1:
    nop
    halt
