; @name: Hashtable Init
; @description: Initialize a hash table.
; @category: collections/hashtable
; @difficulty: 2
;
; @prompt: initialize a hash table at {ptr} with capacity {capacity}
; @prompt: create a new hashtable with {capacity} buckets at memory address {ptr}
; @prompt: set up empty hash table structure at {ptr} holding up to {capacity} entries
; @prompt: allocate hash table of size {capacity} at location {ptr}
; @prompt: init hashtable buffer at {ptr} with max size {capacity}
; @prompt: prepare hash table data structure at {ptr} for {capacity} items
; @prompt: create hash map at {ptr} with {capacity} capacity
; @prompt: initialize empty hashtable at memory {ptr} with room for {capacity} key-value pairs
; @prompt: set up hash table at address {ptr} supporting {capacity} entries
; @prompt: construct hashtable at {ptr} with maximum capacity {capacity}
; @prompt: make a new hash table at {ptr} that can hold {capacity} elements
; @prompt: initialize dictionary at {ptr} with size {capacity}
; @prompt: create key-value store at {ptr} with {capacity} bucket capacity
;
; @param: ptr=r0 "Memory address where hash table will be initialized"
; @param: capacity=r1 "Maximum number of key-value pairs the hash table can hold"
;
; @test: r0=0, r1=0 -> r0=0
;
; @export: hashtable_init
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; ptr
    mov r14, r1  ; capacity
    store.Double r14, [r15]
    mov r15, r0  ; ptr
    mov r14, 1  ; 1
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, 0  ; 0
    store.Double r14, [r15]
    mov r2, 0  ; 0
.while_0:
    nop
    mov r15, r2  ; i
    mov r14, r1  ; capacity
    blt r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endwhile_1
    mov r15, r0  ; ptr
    mov r15, r2  ; i
    mov r14, 2  ; 2
    alu.Add r14, r14, r15
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, 0  ; 0
    store.Double r14, [r15]
    mov r15, 1  ; 1
    alu.Add r2, r2, r15
    b .while_0
.endwhile_1:
    nop
    halt
