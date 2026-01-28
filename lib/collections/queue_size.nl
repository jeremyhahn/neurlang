; @name: Queue Size
; @description: Get the current size of the queue.
; @category: collections/queue
; @difficulty: 1
;
; @prompt: get size of queue at {ptr}
; @prompt: how many elements in queue {ptr}
; @prompt: return queue size for {ptr}
; @prompt: count elements in queue at {ptr}
; @prompt: get number of items in queue {ptr}
; @prompt: queue length at address {ptr}
; @prompt: find queue size at {ptr}
; @prompt: get element count for queue {ptr}
; @prompt: how full is queue at {ptr}
; @prompt: return number of elements in queue {ptr}
; @prompt: query queue size at memory {ptr}
; @prompt: get current size of FIFO queue {ptr}
; @prompt: count items in queue at {ptr}
;
; @param: ptr=r0 "Memory address of the queue"
;
; @test: r0=0 -> r0=0
;
; @export: queue_size
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, 3  ; 3
    alui.Shl r15, r15, 3
    alu.Add r0, r0, r15
    load.Double r0, [r0]
    halt
