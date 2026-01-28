; @name: Input Sanitization
; @description: Sanitize input to prevent SQL injection
; @category: patterns/validation
; @difficulty: 3
;
; @prompt: sanitize input for sql injection prevention
; @prompt: escape sql special characters
; @prompt: prevent sql injection attacks
; @prompt: sanitize user input for database
; @prompt: escape single quotes in input
; @prompt: sql injection prevention filter
; @prompt: clean input for safe database query
; @prompt: escape dangerous sql characters
; @prompt: sanitize string for sql query
; @prompt: input validation for sql safety
;
; @param: char=r0 "Character to check"
;
; @test: r0=0x41, r1=0 -> r0=1
; @test: r0=0x27, r1=0 -> r0=2
; @test: r0=0x3B, r1=0 -> r0=2
; @test: r0=0x2D, r1=0 -> r0=2
; @test: r0=0x5C, r1=0 -> r0=2
;
; @note: Checks if character needs SQL escaping
; @note: Returns 2 if needs escape, 1 if safe

.entry main

.section .text

main:
    ; r0 = test character (for unit testing)
    mov r10, r0                     ; r10 = input char

    ; Check for dangerous characters
    mov r0, r10

    ; Check for single quote (')
    mov r1, 0x27
    beq r0, r1, needs_escape

    ; Check for semicolon (;)
    mov r1, 0x3B
    beq r0, r1, needs_escape

    ; Check for double dash start (-)
    mov r1, 0x2D
    beq r0, r1, needs_escape

    ; Check for backslash (\)
    mov r1, 0x5C
    beq r0, r1, needs_escape

    ; Safe character
    mov r0, 1                       ; Output length = 1 (unchanged)
    halt

needs_escape:
    mov r0, 2                       ; Output length = 2 (escaped)
    halt
