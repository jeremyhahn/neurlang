; @name: HTTP Client GET Request
; @description: Makes HTTP GET request using client extensions
; @category: extension/http
; @difficulty: 1
;
; @prompt: make http get request
; @prompt: fetch data from url
; @prompt: http client get
; @prompt: call external api
; @prompt: get request to service
; @prompt: http fetch url
; @prompt: retrieve data from api
; @prompt: make api get call
; @prompt: http client request
; @prompt: fetch from endpoint
;
; @mock: http_get=1                ; http_get returns handle 1
; @mock: http_response_status=200  ; http_response_status returns 200 OK
; @mock: http_response_body=12345  ; http_response_body returns mock body pointer
; @mock: http_free=0               ; http_free returns 0 (success)
;
; @test: r0=0 -> r0=12345          ; verify we get the body pointer on success
;
; @note: Uses extensions: http_get(220), http_response_status(226), http_response_body(227), http_free(231)
; @note: Returns body pointer in r0, or 0 on error

.entry main

.section .data
    api_url: .asciz "http://127.0.0.1:8080/users/1"

.section .text

main:
    ; Make GET request
    mov r0, api_url
    ext.call r1, http_get, r0        ; r1 = response handle
    beq r1, zero, error

    ; Check status code
    ext.call r2, http_response_status, r1  ; r2 = status code
    mov r3, 200
    bne r2, r3, http_error

    ; Get response body
    ext.call r4, http_response_body, r1    ; r4 = body pointer

    ; Store body pointer before freeing
    mov r5, r4

    ; Free response handle
    ext.call r0, http_free, r1

    ; Return body
    mov r0, r5
    halt

http_error:
    ext.call r0, http_free, r1

error:
    mov r0, 0
    halt
