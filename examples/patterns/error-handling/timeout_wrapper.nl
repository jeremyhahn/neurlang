; @name: Timeout Wrapper
; @description: Wrap an operation with timeout and cleanup on failure
; @category: patterns/error-handling
; @difficulty: 3
;
; @prompt: wrap operation with timeout
; @prompt: implement operation timeout handler
; @prompt: add timeout to blocking operation
; @prompt: timeout wrapper with cleanup
; @prompt: create timeout guard for operation
; @prompt: handle operation timeout gracefully
; @prompt: implement timeout with resource cleanup
; @prompt: timeout operation and release resources
; @prompt: add deadline to operation
; @prompt: cancel operation after timeout
;
; @param: timeout_ms=r0 "Timeout in milliseconds"
; @param: operation_duration=r1 "Operation duration (for testing)"
;
; @test: r0=1000, r1=500 -> r0=0
; @test: r0=500, r1=1000 -> r0=1
; @test: r0=100, r1=100 -> r0=0
;
; @note: Returns 0 if completed within timeout, 1 if timed out
; @note: Always calls cleanup regardless of success/timeout
;
; Timeout Wrapper Pattern
; =======================
; Executes operation with timeout, ensures cleanup on both paths.

.entry main

.section .data

resource_handle:    .word 0         ; Simulated resource handle
cleanup_called:     .word 0         ; Track cleanup for testing

.section .text

main:
    ; r0 = timeout_ms
    ; r1 = operation_duration (simulated)
    mov r10, r0                     ; r10 = timeout
    mov r11, r1                     ; r11 = duration

    ; Acquire resource before operation
    call acquire_resource

    ; Check if operation would exceed timeout
    bgt r11, r10, operation_timeout

    ; Operation completes in time
    call execute_operation
    call cleanup_resource
    mov r0, 0                       ; Success
    halt

operation_timeout:
    ; Timeout occurred - cancel and cleanup
    call cancel_operation
    call cleanup_resource
    mov r0, 1                       ; Timeout error
    halt

acquire_resource:
    ; Simulate acquiring a resource (e.g., file handle, connection)
    mov r0, resource_handle
    mov r1, 12345                   ; Fake handle value
    store.d r1, [r0]
    ret

execute_operation:
    ; Simulate operation execution
    ; In real code, this would be the actual work
    ret

cancel_operation:
    ; Simulate canceling in-progress operation
    ret

cleanup_resource:
    ; Always cleanup regardless of success/timeout
    mov r0, cleanup_called
    mov r1, 1
    store.d r1, [r0]

    ; Release the resource
    mov r0, resource_handle
    store.d zero, [r0]
    ret
