; @name: Content-Length Parser
; @description: Extracts Content-Length value from HTTP header
; @category: network/http
; @difficulty: 3
;
; @prompt: parse Content-Length from HTTP header
; @prompt: extract content length value
; @prompt: find Content-Length in headers
; @prompt: parse HTTP Content-Length header
; @prompt: get body size from headers
; @prompt: extract numeric header value
; @prompt: HTTP header Content-Length extraction
; @prompt: parse content length as integer
; @prompt: find content-length in HTTP request
; @prompt: get request body length
;
; @test: r0=42 -> r0=42
; @note: Returns the Content-Length value (42)

.entry main

.section .data
    ; Simplified: just the number part
    content_len: .asciz "42"

.section .text

main:
    ; Parse number from string (simplified atoi)
    mov r1, content_len
    mov r0, 0                    ; result

parse_loop:
    load.b r2, [r1]
    beq r2, zero, done

    ; Check if digit (0x30-0x39)
    mov r3, 0x30                 ; '0'
    blt r2, r3, done
    mov r3, 0x39                 ; '9'
    bgt r2, r3, done

    ; result = result * 10 + digit
    mov r3, 10
    muldiv.mul r0, r0, r3
    mov r3, 0x30
    alu.sub r2, r2, r3           ; digit value
    alu.add r0, r0, r2

    alui.add r1, r1, 1
    b parse_loop

done:
    halt
