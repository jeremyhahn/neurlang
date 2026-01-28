; @name: HTTP Status Code Validator
; @description: Checks if HTTP status code indicates success (2xx)
; @category: network/http
; @difficulty: 1
;
; @prompt: check if HTTP status is success
; @prompt: validate HTTP response code
; @prompt: is status code 2xx
; @prompt: check HTTP success status
;
; @param: r0 "HTTP status code (200, 201, 400, 404, 500, etc.)"
;
; @test: r0=200 -> r0=1
; @test: r0=201 -> r0=1
; @test: r0=204 -> r0=1
; @test: r0=299 -> r0=1
; @test: r0=199 -> r0=0
; @test: r0=300 -> r0=0
; @test: r0=400 -> r0=0
; @test: r0=404 -> r0=0
; @test: r0=500 -> r0=0
;
; HTTP Status Code Validator
; ==========================
; Checks if status code is in success range (200-299).
; Returns 1 for success, 0 for failure/redirect.

.entry main

.section .text

main:
    ; r0 = HTTP status code
    ; Success if 200 <= code <= 299

    ; Check if code < 200
    mov r1, 200
    blt r0, r1, not_success

    ; Check if code >= 300
    mov r1, 300
    bge r0, r1, not_success

    ; Success (2xx)
    mov r0, 1
    halt

not_success:
    mov r0, 0
    halt
