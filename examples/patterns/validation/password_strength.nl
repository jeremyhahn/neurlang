; @name: Password Strength
; @description: Validate password complexity requirements
; @category: patterns/validation
; @difficulty: 2
;
; @prompt: validate password strength
; @prompt: check password complexity
; @prompt: enforce password requirements
; @prompt: password strength validator
; @prompt: check password has uppercase lowercase digit
; @prompt: validate password meets policy
; @prompt: password complexity checker
; @prompt: check password minimum requirements
; @prompt: validate strong password
; @prompt: password policy enforcement
;
; @param: length=r0 "Password length"
; @param: has_flags=r1 "Flags: bit0=upper, bit1=lower, bit2=digit, bit3=special"
;
; @test: r0=8, r1=15 -> r0=1
; @test: r0=7, r1=15 -> r0=0
; @test: r0=8, r1=7 -> r0=0
; @test: r0=12, r1=15 -> r0=1
;
; @note: Requirements: min 8 chars, upper, lower, digit, special
; @note: Returns 1 if strong, 0 if weak

.entry main

.section .data

min_length:         .dword 8        ; Minimum password length
required_flags:     .dword 15       ; All 4 classes required

.section .text

main:
    ; r0 = length
    ; r1 = has_flags (bit0=upper, bit1=lower, bit2=digit, bit3=special)
    mov r10, r0                     ; r10 = length
    mov r11, r1                     ; r11 = flags

    ; Check minimum length
    mov r0, min_length
    load.d r0, [r0]
    blt r10, r0, weak_password

    ; Check all required flags are present
    mov r0, required_flags
    load.d r0, [r0]
    alu.And r1, r11, r0             ; Check if all required flags set
    bne r1, r0, weak_password

    ; All checks passed
    mov r0, 1                       ; Strong
    halt

weak_password:
    mov r0, 0                       ; Weak
    halt
