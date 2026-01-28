; @name: Stack Init
; @description: Initialize a stack at the given memory location.
; @category: collections/stack
; @difficulty: 1
;
; @prompt: initialize a stack at {ptr} with capacity {capacity}
; @prompt: create a new stack with {capacity} slots at memory address {ptr}
; @prompt: set up empty stack structure at {ptr} holding up to {capacity} elements
; @prompt: allocate stack of size {capacity} at location {ptr}
; @prompt: init stack buffer at {ptr} with max size {capacity}
; @prompt: prepare stack data structure at {ptr} for {capacity} items
; @prompt: create LIFO stack at {ptr} with {capacity} capacity
; @prompt: initialize empty stack at memory {ptr} with room for {capacity} values
; @prompt: set up stack at address {ptr} supporting {capacity} entries
; @prompt: construct stack at {ptr} with maximum capacity {capacity}
; @prompt: make a new stack at {ptr} that can hold {capacity} elements
; @prompt: initialize LIFO data structure at {ptr} with size {capacity}
; @prompt: create stack buffer at {ptr} with {capacity} element capacity
;
; @param: ptr=r0 "Memory address where stack will be initialized"
; @param: capacity=r1 "Maximum number of elements the stack can hold"
;
; @test: r0=0, r1=0 -> r0=0
;
; @export: stack_init
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
    halt
