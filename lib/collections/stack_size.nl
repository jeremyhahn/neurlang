; @name: Stack Size
; @description: Get the current size of the stack.
; @category: collections/stack
; @difficulty: 1
;
; @prompt: get size of stack at {ptr}
; @prompt: how many elements in stack {ptr}
; @prompt: return stack size for {ptr}
; @prompt: count elements in stack at {ptr}
; @prompt: get number of items on stack {ptr}
; @prompt: stack length at address {ptr}
; @prompt: find stack size at {ptr}
; @prompt: get element count for stack {ptr}
; @prompt: how full is stack at {ptr}
; @prompt: return number of elements in stack {ptr}
; @prompt: query stack size at memory {ptr}
; @prompt: get current size of LIFO stack {ptr}
; @prompt: count items on stack at {ptr}
;
; @param: ptr=r0 "Memory address of the stack"
;
; @test: r0=0 -> r0=0
;
; @export: stack_size
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, 1  ; 1
    alui.Shl r15, r15, 3
    alu.Add r0, r0, r15
    load.Double r0, [r0]
    halt
