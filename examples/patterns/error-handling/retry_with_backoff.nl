; @name: Retry with Exponential Backoff
; @description: Retry an operation up to 3 times with exponential backoff delay
; @category: patterns/error-handling
; @difficulty: 4
;
; @prompt: retry operation with exponential backoff
; @prompt: implement retry logic with 3 attempts
; @prompt: create retry mechanism with increasing delays
; @prompt: handle transient failures with exponential retry
; @prompt: retry failed operation up to {n} times
; @prompt: implement backoff retry pattern
; @prompt: create fault-tolerant retry with delays
; @prompt: retry with 1s, 2s, 4s delays between attempts
; @prompt: exponential backoff retry handler
; @prompt: wrap operation in retry with backoff
;
; @param: max_retries=r0 "Maximum number of retry attempts"
; @param: operation_ptr=r1 "Pointer to operation function"
;
; @test: r0=3, r1=0 -> r0=0
; @test: r0=3, r1=1 -> r0=0
; @test: r0=3, r1=4 -> r0=1
; @test: r0=1, r1=0 -> r0=0
;
; @note: Returns 0 on success (any retry succeeds), 1 on all retries failed
; @note: Delay doubles each retry: 100ms, 200ms, 400ms...
;
; Retry with Exponential Backoff Pattern
; ======================================
; Attempts operation up to max_retries times.
; Each retry waits delay_ms * 2^attempt before retrying.
; Returns 0 if any attempt succeeds, 1 if all fail.

.entry main

.section .data

initial_delay:      .dword 100      ; 100ms initial delay

.section .text

main:
    ; r0 = max_retries
    ; r1 = simulated_failures (for testing: 0=success, 1+=failures)
    mov r10, r0                     ; r10 = max_retries
    mov r11, r1                     ; r11 = failures_to_simulate
    mov r12, 0                      ; r12 = attempt counter
    mov r13, initial_delay
    load.d r13, [r13]               ; r13 = current_delay_ms

retry_loop:
    ; Check if we've exhausted retries
    bge r12, r10, all_failed

    ; Try the operation
    call try_operation
    beq r0, zero, success           ; If result == 0, success!

    ; Operation failed - increment attempt counter
    addi r12, r12, 1

    ; Sleep for current_delay_ms (simulated)
    mov r0, r13
    call delay_ms

    ; Double the delay for next attempt (exponential backoff)
    add r13, r13, r13               ; delay *= 2

    ; Retry
    b retry_loop

success:
    mov r0, 0                       ; Return success
    halt

all_failed:
    mov r0, 1                       ; Return failure
    halt

; Simulated operation - fails based on r11 counter
try_operation:
    ; Decrement failures counter
    beq r11, zero, op_success
    subi r11, r11, 1
    mov r0, 1                       ; Return failure
    ret

op_success:
    mov r0, 0                       ; Return success
    ret

; Simulated delay (in real impl would use time.sleep)
delay_ms:
    ; Just return for testing - real impl would sleep
    ret
