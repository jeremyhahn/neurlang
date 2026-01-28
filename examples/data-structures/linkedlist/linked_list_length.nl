; @name: Linked List Length
; @description: Counts nodes in a linked list
; @category: datastructure/linkedlist
; @difficulty: 2
;
; @prompt: count nodes in linked list
; @prompt: get linked list length
; @prompt: traverse linked list and count
; @prompt: find size of linked list
; @prompt: count elements in linked list
; @prompt: linked list node count
; @prompt: traverse list to get length
; @prompt: how many nodes in linked list
; @prompt: linked list size calculation
; @prompt: iterate linked list for count
;
; @test: r0=3 -> r0=3
; @note: List has 3 nodes: [10] -> [20] -> [30] -> null

.entry main

.section .data
    ; Linked list: each node is (value:4, next:4) = 8 bytes
    ; Node 0: value=10, next=node1
    node0: .word 10
    node0_next: .word 0          ; will point to node1 (set at runtime equivalent)
    ; Node 1: value=20, next=node2
    node1: .word 20
    node1_next: .word 0
    ; Node 2: value=30, next=null
    node2: .word 30
    node2_next: .word 0          ; null

.section .text

main:
    ; Setup linked list pointers
    ; node0.next = &node1
    mov r1, node1
    mov r2, node0_next
    store.w r1, [r2]

    ; node1.next = &node2
    mov r1, node2
    mov r2, node1_next
    store.w r1, [r2]

    ; node2.next = 0 (null) - already set

    ; Count nodes
    mov r0, 0                    ; count
    mov r1, node0                ; current = head

count_loop:
    beq r1, zero, done           ; if current == null, done
    alui.add r0, r0, 1           ; count++

    ; current = current->next (offset 4 from node start)
    alui.add r2, r1, 4           ; &current->next
    load.w r1, [r2]              ; current = current->next

    b count_loop

done:
    halt
