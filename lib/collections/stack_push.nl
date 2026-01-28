; @name: Stack Push
; @description: Push a value onto the stack.
; @category: collections/stack
; @difficulty: 1
;
; @prompt: push {value} onto stack at {ptr}
; @prompt: add {value} to top of stack {ptr}
; @prompt: put {value} on stack at address {ptr}
; @prompt: push value {value} to stack {ptr}
; @prompt: add element {value} to stack at {ptr}
; @prompt: insert {value} at top of stack {ptr}
; @prompt: stack push {value} to {ptr}
; @prompt: append {value} to stack at memory {ptr}
; @prompt: place {value} on top of stack {ptr}
; @prompt: push item {value} onto LIFO stack at {ptr}
; @prompt: add {value} to the stack located at {ptr}
; @prompt: store {value} on stack {ptr}
; @prompt: push {value} to stack structure at {ptr}
;
; @param: ptr=r0 "Memory address of the stack"
; @param: value=r1 "Value to push onto the stack"
;
; @test: r0=0, r1=0 -> r0=0
;
; @export: stack_push
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r2, r0  ; ptr
    load.Double r2, [r2]
    mov r3, r0  ; ptr
    mov r15, 1  ; 1
    alui.Shl r15, r15, 3
    alu.Add r3, r3, r15
    load.Double r3, [r3]
    mov r15, r3  ; size
    mov r14, r2  ; capacity
    bge r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endif_1
    mov r0, 0  ; 0
    halt
.endif_1:
    nop
    mov r15, r0  ; ptr
    mov r15, r3  ; size
    mov r14, 2  ; 2
    alu.Add r14, r14, r15
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r1  ; value
    store.Double r14, [r15]
    mov r15, r0  ; ptr
    mov r14, 1  ; 1
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r3  ; size
    mov r15, 1  ; 1
    alu.Add r14, r14, r15
    store.Double r14, [r15]
    mov r0, 1  ; 1
    halt
