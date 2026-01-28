; @name: Notification Service - Send
; @description: Sends notification via HTTP POST
; @category: application/notification-service
; @difficulty: 2
;
; @prompt: send notification
; @prompt: notification service send
; @prompt: dispatch notification
; @prompt: send alert notification
; @prompt: notification delivery
; @prompt: send user notification
; @prompt: notification dispatch service
; @prompt: deliver notification
; @prompt: send system notification
; @prompt: notification sender
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
; @note: Simple notification pattern

.entry main

.section .data
    notify_url: .asciz "http://127.0.0.1:8081/notify"
    key_event:  .asciz "event"
    key_data:   .asciz "data"

.section .text

main:
    ; r0 = event name, r1 = event data string
    mov r14, r0                      ; event name
    mov r15, r1                      ; data

    ; Build notification payload
    ext.call r10, json_new_object

    mov r2, key_event
    ext.call r0, json_set, r10, r2, r14

    mov r2, key_data
    ext.call r0, json_set, r10, r2, r15

    ; Stringify payload
    ext.call r11, json_stringify, r10
    ext.call r0, json_free, r10

    ; POST to notification URL
    mov r0, notify_url
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
