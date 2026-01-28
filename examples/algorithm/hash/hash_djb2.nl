; @name: DJB2 Hash Function
; @description: Computes DJB2 hash of a string
; @category: algorithm/hash
; @difficulty: 2
;
; @prompt: compute djb2 hash of string
; @prompt: hash string using djb2 algorithm
; @prompt: string hashing function
; @prompt: djb2 hash implementation
; @prompt: compute hash value for string
; @prompt: hash function for strings
; @prompt: bernstein hash algorithm
; @prompt: calculate string hash
; @prompt: djb2 string hash
; @prompt: hash key for hash table
;
; @test: r0=5381 -> r0=5863208
; @note: Hash of "ab" using djb2: ((5381*33)+'a')*33+'b'

.entry main

.section .data
    str: .asciz "ab"

.section .text

main:
    mov r0, 5381                 ; hash = 5381 (djb2 initial value)
    mov r1, str

hash_loop:
    load.b r2, [r1]              ; c = *str
    beq r2, zero, done           ; if null terminator, done

    ; hash = hash * 33 + c
    ; hash * 33 = hash * 32 + hash = (hash << 5) + hash
    mov r3, r0
    alui.shl r3, r3, 5           ; hash << 5
    alu.add r0, r3, r0           ; hash * 33
    alu.add r0, r0, r2           ; + c

    alui.add r1, r1, 1           ; str++
    b hash_loop

done:
    halt
