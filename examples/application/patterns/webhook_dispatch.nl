; @name: Webhook Dispatcher
; @description: Dispatches webhook notification to registered URL
; @category: application/pattern
; @difficulty: 2
;
; @prompt: send webhook notification
; @prompt: dispatch webhook event
; @prompt: post to webhook url
; @prompt: webhook notification sender
; @prompt: trigger webhook
; @prompt: send event to webhook
; @prompt: webhook dispatch service
; @prompt: notify via webhook
; @prompt: http webhook call
; @prompt: webhook event dispatcher
;
; Mock all extensions for testing
; @mock: json_new_object=1
; @mock: json_set=0
; @mock: json_stringify=12345
; @mock: json_free=0
; @mock: http_post=1
; @mock: http_response_status=200
; @mock: http_free=0
;
; @test: r0=1, r1=1 -> r0=1
;
; @note: Composes: json_build, http_post
; @note: Standard webhook pattern for event notifications

.entry main

.section .data
    webhook_url: .asciz "http://example.com/webhooks/events"
    key_event:   .asciz "event"
    key_data:    .asciz "data"

.section .text

main:
    ; r0 = event name, r1 = event data (string)
    mov r14, r0                      ; event name
    mov r15, r1                      ; data

    ; Build webhook payload
    ext.call r10, json_new_object

    mov r2, key_event
    ext.call r0, json_set, r10, r2, r14

    mov r2, key_data
    ext.call r0, json_set, r10, r2, r15

    ; Stringify payload
    ext.call r11, json_stringify, r10
    ext.call r0, json_free, r10

    ; POST to webhook URL
    mov r0, webhook_url
    ext.call r12, http_post, r0, r11
    beq r12, zero, error

    ; Check response
    ext.call r13, http_response_status, r12
    ext.call r0, http_free, r12

    ; Return 1 if 2xx
    mov r1, 200
    blt r13, r1, error
    mov r1, 300
    bge r13, r1, error

    mov r0, 1
    halt

error:
    mov r0, 0
    halt
