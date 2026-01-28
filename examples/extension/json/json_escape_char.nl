; @name: JSON Escape Character
; @description: Checks if a character needs JSON escaping
; @category: data/json
; @difficulty: 2
;
; @prompt: check if character needs JSON escaping
; @prompt: is character a JSON special char
; @prompt: detect JSON escape characters
; @prompt: check for quote backslash newline
; @prompt: JSON string escape detection
; @prompt: needs escaping for JSON string
; @prompt: identify JSON control characters
; @prompt: check if char requires backslash escape
; @prompt: JSON special character test
; @prompt: validate JSON string character
;
; @param: char=r0 "ASCII character code"
;
; @test: r0=34 -> r0=1
; @test: r0=92 -> r0=1
; @test: r0=10 -> r0=1
; @test: r0=13 -> r0=1
; @test: r0=9 -> r0=1
; @test: r0=65 -> r0=0
; @test: r0=97 -> r0=0
; @test: r0=1 -> r0=1
; @note: Returns 1 if needs escape, 0 otherwise
; @note: 34=", 92=\, 10=\n, 13=\r, 9=\t

.entry main

main:
    ; Check for characters that need escaping in JSON
    ; " (34), \ (92), newline (10), carriage return (13), tab (9)

    mov r1, 34                   ; double quote
    beq r0, r1, needs_escape

    mov r1, 92                   ; backslash
    beq r0, r1, needs_escape

    mov r1, 10                   ; newline
    beq r0, r1, needs_escape

    mov r1, 13                   ; carriage return
    beq r0, r1, needs_escape

    mov r1, 9                    ; tab
    beq r0, r1, needs_escape

    ; Control characters (0-31) also need escaping
    mov r1, 32
    blt r0, r1, needs_escape

    ; No escape needed
    mov r0, 0
    halt

needs_escape:
    mov r0, 1
    halt
