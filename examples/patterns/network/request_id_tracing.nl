; @name: Request ID Tracing
; @description: Generate and propagate request ID for tracing
; @category: patterns/network
; @difficulty: 3
;
; @prompt: generate request id for tracing
; @prompt: propagate trace id through request
; @prompt: distributed tracing with request id
; @prompt: add x-request-id header
; @prompt: request correlation id pattern
; @prompt: trace id generation and propagation
; @prompt: request id for logging
; @prompt: correlation id middleware
; @prompt: distributed request tracing
; @prompt: generate and return trace id
;
; @param: has_incoming_id=r0 "Request has incoming ID (0/1)"
;
; @test: r0=0 -> r0=1
; @test: r0=1 -> r0=1
;
; @note: Returns 1 (ID set successfully)
; @note: Uses incoming ID if present, generates if not
;
; Request ID Tracing Pattern
; ==========================
; Ensure every request has a unique ID for correlation.

.entry main

.section .data

current_request_id: .space 64, 0    ; Current request's ID
id_counter:         .word 0         ; Simple counter for unique IDs

.section .text

main:
    ; r0 = has_incoming_id
    mov r10, r0

    bne r10, zero, use_incoming

    ; No incoming ID - generate one
    call generate_request_id
    b set_response_header

use_incoming:
    ; Use the incoming X-Request-ID
    call extract_incoming_id

set_response_header:
    ; Add ID to response headers
    call add_id_to_response

    mov r0, 1                       ; Success
    halt

generate_request_id:
    ; Generate unique ID (would use UUID in real impl)
    ; ext.call 330, output_buf  ; uuid_v4

    ; Simple counter-based ID for demo
    mov r0, id_counter
    load.d r1, [r0]
    addi r1, r1, 1
    store.d r1, [r0]

    ; Store in current_request_id
    mov r0, current_request_id
    store.d r1, [r0]

    ret

extract_incoming_id:
    ; Would parse X-Request-ID header from request
    ; Copy to current_request_id
    ret

add_id_to_response:
    ; Add X-Request-ID header to response
    ; Would append "X-Request-ID: {id}\r\n" to headers
    ret

; Log with request ID
log_with_id:
    ; r0 = message ptr
    ; Would prefix log with current request ID
    ret

; Propagate ID to downstream calls
propagate_to_downstream:
    ; When making outgoing HTTP calls, include the ID
    ; Set X-Request-ID header on outgoing request
    ret

; Full middleware implementation
tracing_middleware:
    ; r0 = request ptr

    ; Check for incoming X-Request-ID
    mov r1, r0
    call has_request_id_header
    beq r0, zero, generate_new_id

    ; Extract incoming ID
    mov r0, r1
    call extract_request_id_header
    b store_id

generate_new_id:
    call generate_request_id

store_id:
    ; Store in thread-local or context
    mov r0, current_request_id

    ret

has_request_id_header:
    ; Check if X-Request-ID header exists
    mov r0, 0                       ; Stub
    ret

extract_request_id_header:
    ; Extract and return ID value
    ret
