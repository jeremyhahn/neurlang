; @name: HTTP Client PUT Request
; @description: Makes HTTP PUT request to update resource
; @category: extension/http
; @difficulty: 2
;
; @prompt: make http put request
; @prompt: update resource via api
; @prompt: http client put
; @prompt: put json to endpoint
; @prompt: update request with body
; @prompt: http put json
; @prompt: modify resource via api
; @prompt: make api put call
; @prompt: update data at service
; @prompt: put to external api
;
; @mock: http_put=1                ; http_put returns handle 1
; @mock: http_response_status=200  ; http_response_status returns 200 OK
; @mock: http_free=0               ; http_free returns 0 (success)
;
; @test: r0=0 -> r0=1              ; verify we get success on 200 response
;
; @note: Uses extensions: http_put(222), http_response_status(226), http_free(231)
; @note: Returns 1 on success (200 OK), 0 on error

.entry main

.section .data
    api_url:  .asciz "http://127.0.0.1:8080/users/123"
    put_body: .asciz "{\"name\":\"Updated Name\",\"email\":\"new@example.com\"}"

.section .text

main:
    ; Make PUT request with body
    mov r0, api_url
    mov r1, put_body
    ext.call r2, http_put, r0, r1    ; r2 = response handle
    beq r2, zero, error

    ; Check status code (expect 200 OK)
    ext.call r3, http_response_status, r2
    ext.call r0, http_free, r2

    mov r4, 200
    bne r3, r4, error

    ; Return success
    mov r0, 1
    halt

error:
    mov r0, 0
    halt
