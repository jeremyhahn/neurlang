; @name: Selection Sort
; @description: Sorts an array using selection sort algorithm
; @category: algorithm/sorting
; @difficulty: 3
;
; @prompt: sort array using selection sort
; @prompt: selection sort {size} elements
; @prompt: implement selection sort algorithm
; @prompt: find minimum and swap sort
; @prompt: O(n^2) in-place sort
; @prompt: selection sort array of integers
; @prompt: sort by selecting minimum element
; @prompt: simple comparison sort
; @prompt: minimize swaps sorting algorithm
; @prompt: sort by repeatedly finding minimum
;
; @param: size=r2 "Number of elements to sort"
;
; @test: r2=5 -> r0=1
; @test: r2=3 -> r0=1
; @test: r2=1 -> r0=1
; @note: Returns 1 on success, array is sorted in place
; @note: Test array: [64, 34, 25, 12, 22] -> [12, 22, 25, 34, 64]

.entry main

.section .data
    ; Test array: 64, 34, 25, 12, 22
    array: .word 64, 34, 25, 12, 22

.section .text

main:
    ; r0 = array pointer
    mov r0, array
    ; r1 = size
    mov r1, 5

    ; Outer loop: i from 0 to n-1
    mov r2, 0                    ; i = 0

outer_loop:
    mov r3, r1
    alui.sub r3, r3, 1           ; n-1
    bge r2, r3, sort_done        ; if i >= n-1, done

    ; min_idx = i
    mov r4, r2                   ; min_idx = i

    ; Inner loop: j from i+1 to n
    mov r5, r2
    alui.add r5, r5, 1           ; j = i + 1

find_min:
    bge r5, r1, do_swap          ; if j >= n, swap

    ; Load arr[j] and arr[min_idx]
    mov r6, r5
    alui.shl r6, r6, 2           ; j * 4
    alu.add r6, r0, r6           ; &arr[j]
    load.w r7, [r6]              ; arr[j]

    mov r8, r4
    alui.shl r8, r8, 2           ; min_idx * 4
    alu.add r8, r0, r8           ; &arr[min_idx]
    load.w r9, [r8]              ; arr[min_idx]

    ; if arr[j] < arr[min_idx], update min_idx
    bge r7, r9, no_update
    mov r4, r5                   ; min_idx = j

no_update:
    alui.add r5, r5, 1           ; j++
    b find_min

do_swap:
    ; Swap arr[i] and arr[min_idx] if different
    beq r2, r4, next_i           ; if i == min_idx, skip swap

    ; Load arr[i] and arr[min_idx]
    mov r6, r2
    alui.shl r6, r6, 2           ; i * 4
    alu.add r6, r0, r6           ; &arr[i]
    load.w r7, [r6]              ; arr[i]

    mov r8, r4
    alui.shl r8, r8, 2           ; min_idx * 4
    alu.add r8, r0, r8           ; &arr[min_idx]
    load.w r9, [r8]              ; arr[min_idx]

    ; Swap
    store.w r9, [r6]             ; arr[i] = arr[min_idx]
    store.w r7, [r8]             ; arr[min_idx] = arr[i]

next_i:
    alui.add r2, r2, 1           ; i++
    b outer_loop

sort_done:
    mov r0, 1                    ; return success
    halt
