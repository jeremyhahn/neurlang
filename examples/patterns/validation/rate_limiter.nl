; @name: Rate Limiter
; @description: Token bucket rate limiting implementation
; @category: patterns/validation
; @difficulty: 4
;
; @prompt: implement token bucket rate limiter
; @prompt: rate limit requests using token bucket
; @prompt: create rate limiter with token bucket
; @prompt: implement request rate limiting
; @prompt: token bucket algorithm for rate limiting
; @prompt: rate limit api calls
; @prompt: throttle requests with token bucket
; @prompt: implement api rate limiter
; @prompt: rate limiting with bucket refill
; @prompt: control request rate with tokens
;
; @param: request_count=r0 "Number of requests to check"
; @param: bucket_tokens=r1 "Current tokens in bucket"
;
; @test: r0=1, r1=10 -> r0=1
; @test: r0=10, r1=10 -> r0=1
; @test: r0=11, r1=10 -> r0=0
; @test: r0=5, r1=3 -> r0=0
;
; @note: Returns 1 if allowed, 0 if rate limited
; @note: Each request consumes 1 token
;
; Rate Limiter Pattern (Token Bucket)
; ===================================
; Allow requests if tokens available, refill over time.

.entry main

.section .data

bucket_capacity:    .word 100       ; Max tokens in bucket
tokens:             .word 100       ; Current token count
refill_rate:        .word 10        ; Tokens added per interval
last_refill:        .word 0         ; Last refill timestamp

.section .text

main:
    ; r0 = requests to check
    ; r1 = current tokens (for testing)
    mov r10, r0                     ; r10 = requests
    mov r11, r1                     ; r11 = tokens

    ; Check if we have enough tokens
    bgt r10, r11, rate_limited

    ; Have tokens - allow request
    ; (In real impl, would decrement tokens here)
    mov r0, 1                       ; Allowed
    halt

rate_limited:
    mov r0, 0                       ; Denied
    halt

; Full rate limiter with time-based refill
check_rate_limit:
    ; r0 = current_time, r1 = tokens_needed
    mov r2, r1                      ; r2 = tokens needed

    ; Refill tokens based on elapsed time
    call refill_tokens

    ; Check if enough tokens
    mov r0, tokens
    load.d r3, [r0]
    blt r3, r2, limit_exceeded

    ; Consume tokens
    sub r3, r3, r2
    store.d r3, [r0]
    mov r0, 1                       ; Allowed
    ret

limit_exceeded:
    mov r0, 0                       ; Denied
    ret

refill_tokens:
    ; Calculate tokens to add based on time elapsed
    ; (Simplified - just add refill_rate)
    mov r0, tokens
    load.d r1, [r0]
    mov r2, refill_rate
    load.d r2, [r2]
    add r1, r1, r2

    ; Cap at capacity
    mov r3, bucket_capacity
    load.d r3, [r3]
    blt r1, r3, store_tokens
    mov r1, r3                      ; Cap at max

store_tokens:
    store.d r1, [r0]
    ret
