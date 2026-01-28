; @name: Webhook Processor
; @description: Secure webhook handler with signature verification
; @category: advanced/integration
; @difficulty: 4
;
; @prompt: implement webhook handler
; @prompt: webhook signature verification
; @prompt: secure webhook processing
; @prompt: handle incoming webhooks
; @prompt: webhook payload validation
; @prompt: process webhook events
; @prompt: webhook with hmac verification
; @prompt: webhook dispatcher
; @prompt: github webhook handler
; @prompt: stripe webhook processor
;
; @param: sig_valid=r0 "Signature is valid (0/1)"
; @param: event_type=r1 "Event type (1=create, 2=update, 3=delete)"
;
; @test: r0=1, r1=1 -> r0=200
; @test: r0=1, r1=2 -> r0=200
; @test: r0=0, r1=1 -> r0=401
; @test: r0=1, r1=0 -> r0=400
;
; @note: Returns HTTP status code
; @note: Verifies HMAC-SHA256 signature before processing
; @note: Testable control flow - extension calls are in helper functions
;
; Webhook Processor Pattern
; =========================
; Verify signature -> Parse payload -> Dispatch -> Acknowledge

.entry main

.section .data

; Webhook secret for signature verification
webhook_secret:     .asciz "whsec_your_webhook_secret_here"
secret_len:         .word 32

; Request data
payload_buf:        .space 8192, 0
signature_buf:      .space 64, 0
computed_sig:       .space 32, 0

; Event handlers
handler_table:      .space 256, 0   ; Function pointers

; Response
response_buf:       .space 256, 0

.section .text

main:
    ; r0 = signature_valid
    ; r1 = event_type
    mov r10, r0
    mov r11, r1

    ; Verify signature first
    beq r10, zero, unauthorized

    ; Validate event type
    beq r11, zero, bad_request
    mov r0, 4
    bge r11, r0, bad_request

    ; Dispatch event
    mov r0, r11
    call dispatch_event

    ; Acknowledge receipt
    mov r0, 200
    halt

unauthorized:
    mov r0, 401
    halt

bad_request:
    mov r0, 400
    halt

; Full webhook handler
handle_webhook:
    ; r0 = raw request ptr
    ; r1 = request length
    mov r10, r0
    mov r11, r1

    ; Extract signature header
    mov r0, r10
    call extract_signature_header
    beq r0, zero, missing_signature
    mov r12, r0                     ; signature

    ; Extract timestamp header (replay protection)
    mov r0, r10
    call extract_timestamp_header
    beq r0, zero, missing_timestamp
    mov r13, r0                     ; timestamp

    ; Check timestamp freshness (prevent replay)
    mov r0, r13
    call validate_timestamp
    beq r0, zero, timestamp_expired

    ; Extract payload
    mov r0, r10
    call extract_body
    mov r14, r0                     ; payload ptr
    mov r15, r1                     ; payload len

    ; Compute expected signature
    ; HMAC-SHA256(secret, timestamp + "." + payload)
    mov r0, r13                     ; timestamp
    mov r1, r14                     ; payload
    mov r2, r15                     ; payload len
    call compute_signature

    ; Compare signatures (constant time)
    mov r1, computed_sig            ; computed
    ext.call r0, constant_time_eq, r12, r1  ; constant_time_compare: r0=result
    beq r0, zero, invalid_signature

    ; Parse payload JSON
    ext.call r10, json_parse, r14, r15      ; json_parse: r10=handle, r14=data, r15=len

    ; Extract event type
    mov r1, event_type_key
    ext.call r11, json_get, r10, r1  ; json_get: r11=value, r10=handle, r1=key

    ; Dispatch to handler
    mov r0, r11
    mov r1, r10
    call dispatch_by_type

    ; Acknowledge (HTTP 200)
    mov r0, 200
    ret

missing_signature:
missing_timestamp:
    mov r0, 400
    ret

timestamp_expired:
    mov r0, 400
    ret

invalid_signature:
    mov r0, 401
    ret

extract_signature_header:
    ; Extract X-Webhook-Signature or similar header
    mov r0, signature_buf           ; Stub
    ret

extract_timestamp_header:
    ; Extract X-Webhook-Timestamp header
    mov r0, 1234567890              ; Stub timestamp
    ret

validate_timestamp:
    ; r0 = timestamp
    ; Check within acceptable window (e.g., 5 minutes)
    mov r2, r0                      ; save timestamp
    ext.call r0, datetime_now       ; datetime_now: r0=current time
    mov r1, r0                      ; current time

    ; Stub - would check |current - timestamp| < 300
    mov r0, 1
    ret

extract_body:
    ; Extract HTTP body from request
    mov r0, payload_buf
    mov r1, 0                       ; Stub length
    ret

compute_signature:
    ; r0 = timestamp, r1 = payload ptr, r2 = payload len
    ; Compute HMAC-SHA256(secret, timestamp + "." + payload)

    ; Build signed payload string
    ; (would concatenate timestamp + "." + payload)

    ; HMAC-SHA256: result in computed_sig buffer
    mov r4, payload_buf
    mov r5, webhook_secret
    ext.call r0, hmac_sha256, r4, r5  ; hmac_sha256: r0=output, r4=data, r5=key
    ret

dispatch_by_type:
    ; r0 = event type string
    ; r1 = JSON payload handle
    ; Route to appropriate handler
    ret

dispatch_event:
    ; r0 = event type (1-3)
    mov r10, r0

    mov r0, 1
    beq r10, r0, handle_create
    mov r0, 2
    beq r10, r0, handle_update
    mov r0, 3
    beq r10, r0, handle_delete

    ret

handle_create:
    ; Process create event
    ret

handle_update:
    ; Process update event
    ret

handle_delete:
    ; Process delete event
    ret

; Retry logic for handler failures
retry_handler:
    ; r0 = handler function
    ; r1 = payload
    ; Retry with exponential backoff
    mov r10, r0
    mov r11, r1
    mov r12, 0                      ; attempt

retry_loop:
    mov r0, 3
    bge r12, r0, retry_exhausted

    ; Try handler
    mov r0, r11
    ; (would call handler)
    bne r0, zero, handler_success

    ; Backoff delay
    mov r0, 1
    add r0, r12, r0
    ; (would sleep)

    addi r12, r12, 1
    b retry_loop

handler_success:
    mov r0, 1
    ret

retry_exhausted:
    ; Send to dead letter queue
    mov r0, r11
    call send_to_dlq
    mov r0, 0
    ret

send_to_dlq:
    ; Store failed webhook for later processing
    ret

; Idempotency handling
check_idempotency:
    ; r0 = event ID
    ; Check if already processed
    mov r0, 0                       ; Not seen stub
    ret

mark_processed:
    ; r0 = event ID
    ; Mark as processed to prevent duplicates
    ret

.section .data
event_type_key:     .asciz "type"
