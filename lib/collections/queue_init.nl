; @name: Queue Init
; @description: Initialize a queue at the given memory location.
; @category: collections/queue
; @difficulty: 1
;
; @prompt: initialize a queue at {ptr} with capacity {capacity}
; @prompt: create a new queue with {capacity} slots at memory address {ptr}
; @prompt: set up empty queue structure at {ptr} holding up to {capacity} elements
; @prompt: allocate queue of size {capacity} at location {ptr}
; @prompt: init queue buffer at {ptr} with max size {capacity}
; @prompt: prepare queue data structure at {ptr} for {capacity} items
; @prompt: create FIFO queue at {ptr} with {capacity} capacity
; @prompt: initialize empty queue at memory {ptr} with room for {capacity} values
; @prompt: set up ring buffer queue at address {ptr} supporting {capacity} entries
; @prompt: construct queue at {ptr} with maximum capacity {capacity}
; @prompt: make a new queue at {ptr} that can hold {capacity} elements
; @prompt: initialize FIFO data structure at {ptr} with size {capacity}
; @prompt: create circular queue buffer at {ptr} with {capacity} element capacity
;
; @param: ptr=r0 "Memory address where queue will be initialized"
; @param: capacity=r1 "Maximum number of elements the queue can hold"
;
; @test: r0=0, r1=0 -> r0=0
;
; @export: queue_init
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
    mov r15, r0  ; ptr
    mov r14, 2  ; 2
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, 0  ; 0
    store.Double r14, [r15]
    mov r15, r0  ; ptr
    mov r14, 3  ; 3
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, 0  ; 0
    store.Double r14, [r15]
    halt
