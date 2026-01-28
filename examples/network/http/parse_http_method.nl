; @name: HTTP Method Parser
; @description: Identifies HTTP method from first character code
; @category: network/http
; @difficulty: 1
;
; @prompt: parse HTTP method from request
; @prompt: identify GET POST PUT DELETE from first byte
; @prompt: determine HTTP method type
; @prompt: check if request is GET
; @prompt: detect HTTP request method
;
; @param: r0 "ASCII code of first character (G=71, P=80, D=68)"
;
; @test: r0=71 -> r0=1
; @test: r0=80 -> r0=2
; @test: r0=68 -> r0=3
; @test: r0=72 -> r0=0
;
; @note: Returns: 1=GET, 2=POST/PUT, 3=DELETE, 0=unknown
; @note: POST and PUT both start with P - need second byte to distinguish
;
; HTTP Method Parser
; ==================
; First-byte method detection for HTTP request routing.
; Returns method category for fast routing.
;
; Method codes:
;   'G' (0x47 = 71) -> GET     -> returns 1
;   'P' (0x50 = 80) -> POST/PUT -> returns 2
;   'D' (0x44 = 68) -> DELETE  -> returns 3
;   Other          -> Unknown  -> returns 0

.entry main

.section .text

main:
    ; r0 = first byte of HTTP request

    ; Check for 'G' (GET)
    mov r1, 71
    beq r0, r1, is_get

    ; Check for 'P' (POST/PUT)
    mov r1, 80
    beq r0, r1, is_post_put

    ; Check for 'D' (DELETE)
    mov r1, 68
    beq r0, r1, is_delete

    ; Unknown method
    mov r0, 0
    halt

is_get:
    mov r0, 1
    halt

is_post_put:
    mov r0, 2
    halt

is_delete:
    mov r0, 3
    halt
