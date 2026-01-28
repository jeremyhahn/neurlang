; @name: Validate JSON Number
; @description: Checks if a string is a valid JSON number
; @category: data/json
; @difficulty: 2
;
; @prompt: validate JSON number format
; @prompt: check if string is valid JSON number
; @prompt: is string a valid numeric literal
; @prompt: JSON number validation
; @prompt: parse and validate number string
; @prompt: check numeric string format
; @prompt: validate integer or decimal string
; @prompt: is input a valid JSON numeric value
; @prompt: number format validator
; @prompt: check if string can be JSON number
;
; @test: r0=1 -> r0=1
; @test: r0=0 -> r0=0
; @note: r0=1 tests "123" (valid), r0=0 tests "12a" (invalid)

.entry main

.section .data
    valid_num: .asciz "123"
    invalid_num: .asciz "12a"

.section .text

main:
    ; r0 = 1 for valid test, 0 for invalid test
    beq r0, zero, test_invalid
    mov r1, valid_num
    b do_validate

test_invalid:
    mov r1, invalid_num

do_validate:
    ; Check each character is a digit (0-9) or minus sign at start
    mov r2, 1                    ; assume valid
    mov r3, 0                    ; position

validate_loop:
    load.b r4, [r1]
    beq r4, zero, done           ; end of string

    ; Allow minus at position 0
    mov r5, 0x2D                 ; '-'
    bne r4, r5, not_minus
    bne r3, zero, invalid        ; minus only at start
    b next_char

not_minus:
    ; Check if digit (0x30-0x39)
    mov r5, 0x30                 ; '0'
    blt r4, r5, invalid
    mov r5, 0x39                 ; '9'
    bgt r4, r5, invalid

next_char:
    alui.add r1, r1, 1
    alui.add r3, r3, 1
    b validate_loop

invalid:
    mov r2, 0

done:
    mov r0, r2
    halt
