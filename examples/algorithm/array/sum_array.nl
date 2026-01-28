; @name: Sum Array
; @description: Sums a hardcoded array of values
; @category: algorithm/array
; @difficulty: 1
;
; @prompt: sum array elements
; @prompt: add all values in array
; @prompt: calculate array sum
; @prompt: total of array elements
; @prompt: accumulate array values
; @prompt: sum hardcoded array
; @prompt: compute total of array
; @prompt: add up array
;
; @test: -> r0=150
; @note: Sums [10, 20, 30, 40, 50] = 150

.entry main

main:
    ; Sum hardcoded values: 10 + 20 + 30 + 40 + 50 = 150
    ; Using register-based approach for simplicity

    mov r0, 0           ; sum = 0

    ; Add 10
    mov r1, 10
    alu.add r0, r0, r1

    ; Add 20
    mov r1, 20
    alu.add r0, r0, r1

    ; Add 30
    mov r1, 30
    alu.add r0, r0, r1

    ; Add 40
    mov r1, 40
    alu.add r0, r0, r1

    ; Add 50
    mov r1, 50
    alu.add r0, r0, r1

    ; r0 = 150
    halt
