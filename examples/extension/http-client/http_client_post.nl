; @name: HTTP Client POST Request
; @description: Makes HTTP POST request with JSON body
; @category: extension/http
; @difficulty: 2
;
; @prompt: make http post request
; @prompt: post data to api
; @prompt: http client post
; @prompt: send json to endpoint
; @prompt: post request with body
; @prompt: http post json
; @prompt: create resource via api
; @prompt: make api post call
; @prompt: send data to service
; @prompt: post to external api
;
; @mock: http_post=1               ; http_post returns handle 1
; @mock: http_response_status=201  ; http_response_status returns 201 Created
; @mock: http_free=0               ; http_free returns 0 (success)
;
; @test: r0=0 -> r0=1              ; verify we get success on 201 response
;
; @note: Uses extensions: http_post(221), http_response_status(226), http_free(231)
; @note: Returns 1 on success (201 Created), 0 on error

.entry main

.section .data
    api_url:   .asciz "http://127.0.0.1:8080/users"
    post_body: .asciz "{\"name\":\"Bob\",\"email\":\"bob@example.com\"}"

.section .text

main:
    ; Make POST request with body
    mov r0, api_url
    mov r1, post_body
    ext.call r2, http_post, r0, r1   ; r2 = response handle
    beq r2, zero, error

    ; Check status code (expect 201 Created)
    ext.call r3, http_response_status, r2
    mov r4, 201
    bne r3, r4, http_error

    ; Free response handle
    ext.call r0, http_free, r2

    ; Return success
    mov r0, 1
    halt

http_error:
    ext.call r0, http_free, r2

error:
    mov r0, 0
    halt
