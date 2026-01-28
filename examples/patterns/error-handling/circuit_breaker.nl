; @name: Circuit Breaker
; @description: Circuit breaker pattern with CLOSED, OPEN, HALF_OPEN states
; @category: patterns/error-handling
; @difficulty: 4
;
; @prompt: implement circuit breaker pattern
; @prompt: create circuit breaker with failure threshold
; @prompt: handle cascading failures with circuit breaker
; @prompt: implement circuit breaker state machine
; @prompt: prevent repeated failures with circuit breaker
; @prompt: create fail-fast circuit breaker
; @prompt: circuit breaker with half-open state
; @prompt: implement fault isolation with circuit breaker
; @prompt: circuit breaker for external service calls
; @prompt: build resilient circuit breaker
;
; @param: threshold=r0 "Failure threshold to open circuit"
;
; States: 0=CLOSED (normal), 1=OPEN (reject calls), 2=HALF_OPEN (testing)
;
; @test: r0=3, r1=0 -> r0=0
; @test: r0=3, r1=1 -> r0=1
; @test: r0=1, r1=3 -> r0=1
;
; @note: State transitions: CLOSED->OPEN after threshold failures
; @note: OPEN->HALF_OPEN after cooldown, HALF_OPEN->CLOSED on success
;
; Circuit Breaker Pattern
; =======================
; Prevents cascading failures by failing fast when error rate is high.
; Three states: CLOSED (normal), OPEN (rejecting), HALF_OPEN (probing)

.entry main

.section .data

state:              .word 0         ; 0=CLOSED, 1=OPEN, 2=HALF_OPEN
failure_count:      .word 0         ; Current consecutive failures
cooldown_timer:     .word 0         ; Timer for OPEN->HALF_OPEN transition
cooldown_period:    .word 1000      ; 1 second cooldown

.section .text

main:
    ; r0 = failure_threshold
    ; r1 = test_input (0=success, 1+=failures to simulate)
    mov r10, r0                     ; r10 = threshold
    mov r11, r1                     ; r11 = simulated result

    ; Load current state
    mov r0, state
    load.d r12, [r0]                ; r12 = current state

    ; State dispatch
    beq r12, zero, state_closed     ; 0 = CLOSED
    mov r1, 1
    beq r12, r1, state_open         ; 1 = OPEN
    b state_half_open               ; 2 = HALF_OPEN

state_closed:
    ; Normal operation - call the service
    call attempt_call
    beq r0, zero, closed_success

    ; Failure - increment failure count
    mov r1, failure_count
    load.d r2, [r1]
    addi r2, r2, 1
    store.d r2, [r1]

    ; Check threshold
    blt r2, r10, closed_failure_below_threshold

    ; Threshold exceeded - trip to OPEN
    call trip_open
    mov r0, 1                       ; Return failure
    halt

closed_failure_below_threshold:
    mov r0, 1                       ; Return failure but stay closed
    halt

closed_success:
    ; Reset failure count on success
    mov r1, failure_count
    store.d zero, [r1]
    mov r0, 0                       ; Return success
    halt

state_open:
    ; Circuit is open - check cooldown
    mov r0, cooldown_timer
    load.d r1, [r0]
    mov r2, cooldown_period
    load.d r2, [r2]
    blt r1, r2, reject_call

    ; Cooldown expired - transition to HALF_OPEN
    mov r1, state
    mov r2, 2                       ; HALF_OPEN
    store.d r2, [r1]
    b state_half_open

reject_call:
    ; Increment timer (simulated)
    addi r1, r1, 100
    mov r2, cooldown_timer
    store.d r1, [r2]
    mov r0, 1                       ; Reject - return failure immediately
    halt

state_half_open:
    ; Try a single test call
    call attempt_call
    beq r0, zero, half_open_success

    ; Failed - trip back to OPEN
    call trip_open
    mov r0, 1
    halt

half_open_success:
    ; Success - close the circuit
    mov r1, state
    store.d zero, [r1]              ; CLOSED = 0
    mov r1, failure_count
    store.d zero, [r1]              ; Reset failures
    mov r0, 0
    halt

trip_open:
    mov r1, state
    mov r2, 1                       ; OPEN
    store.d r2, [r1]
    mov r1, cooldown_timer
    store.d zero, [r1]              ; Reset timer
    ret

; Simulated service call
attempt_call:
    beq r11, zero, call_success
    subi r11, r11, 1
    mov r0, 1                       ; Failure
    ret
call_success:
    mov r0, 0                       ; Success
    ret
