; @name: HTTP Client DELETE Request
; @description: Makes HTTP DELETE request to remove resource
; @category: extension/http
; @difficulty: 1
;
; @prompt: make http delete request
; @prompt: delete resource via api
; @prompt: http client delete
; @prompt: remove via endpoint
; @prompt: delete request
; @prompt: http delete call
; @prompt: remove resource via api
; @prompt: make api delete call
; @prompt: delete from service
; @prompt: delete at external api
;
; @mock: http_delete=1             ; http_delete returns handle 1
; @mock: http_response_status=200  ; http_response_status returns 200 OK
; @mock: http_free=0               ; http_free returns 0 (success)
;
; @test: r0=0 -> r0=1              ; verify we get success on 200 response
;
; @note: Uses extensions: http_delete(223), http_response_status(226), http_free(231)
; @note: Returns 1 on success (200/204), 0 on error

.entry main

.section .data
    api_url: .asciz "http://127.0.0.1:8080/users/123"

.section .text

main:
    ; Make DELETE request
    mov r0, api_url
    ext.call r1, http_delete, r0     ; r1 = response handle
    beq r1, zero, error

    ; Check status code (accept 200 OK or 204 No Content)
    ext.call r2, http_response_status, r1
    ext.call r0, http_free, r1

    ; Check for 200
    mov r3, 200
    beq r2, r3, success

    ; Check for 204
    mov r3, 204
    beq r2, r3, success

    ; Not a success code
    b error

success:
    mov r0, 1
    halt

error:
    mov r0, 0
    halt
