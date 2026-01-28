; @name: Unique
; @description: Remove duplicates from a sorted array.
; @category: array
; @difficulty: 2
;
; @prompt: remove duplicates from sorted {arr} with {len} elements
; @prompt: deduplicate sorted array {arr} of size {len}
; @prompt: unique elements in sorted {arr} containing {len} items
; @prompt: eliminate duplicates from {arr} of length {len}
; @prompt: remove duplicate values from sorted {arr} with {len} entries
; @prompt: in-place dedup of sorted {arr} with {len} elements
; @prompt: keep only unique values in sorted {arr} of {len} items
; @prompt: remove repeated elements from sorted {arr} with {len} values
; @prompt: dedupe {len} element sorted array {arr}
; @prompt: filter duplicates from sorted {arr} containing {len} entries
; @prompt: compress {arr} by removing duplicates over {len} elements
; @prompt: return unique count after deduping sorted {arr} of {len}
;
; @param: arr=r0 "Pointer to sorted array of u64 elements (mutable)"
; @param: len=r1 "Number of elements in the array"
;
; @test: r0=0, r1=0 -> r0=0
;
; @export: unique
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r1  ; len
    mov r14, 2  ; 2
    blt r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endif_1
    mov r0, r1  ; len
    halt
.endif_1:
    nop
    mov r2, 1  ; 1
    mov r3, 1  ; 1
.while_4:
    nop
    mov r15, r3  ; read_idx
    mov r14, r1  ; len
    blt r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endwhile_5
    mov r14, r0  ; ptr
    mov r15, r2  ; write_idx
    mov r14, 1  ; 1
    alu.Sub r15, r15, r14
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    mov r15, r0  ; ptr
    mov r14, r3  ; read_idx
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    load.Double r15, [r15]
    bne r15, r14, .set_10
    mov r15, 0
    b .cmp_end_11
.set_10:
    nop
    mov r15, 1
.cmp_end_11:
    nop
    beq r15, zero, .endif_9
    mov r15, r0  ; ptr
    mov r14, r2  ; write_idx
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r0  ; ptr
    mov r15, r3  ; read_idx
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    store.Double r14, [r15]
    mov r15, 1  ; 1
    alu.Add r2, r2, r15
.endif_9:
    nop
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .while_4
.endwhile_5:
    nop
    mov r0, r2  ; write_idx
    halt
