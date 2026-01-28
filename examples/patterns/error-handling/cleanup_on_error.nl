; @name: Cleanup on Error
; @description: Ensure resource cleanup in all error paths
; @category: patterns/error-handling
; @difficulty: 3
;
; @prompt: cleanup resources on error
; @prompt: ensure cleanup in all error paths
; @prompt: release resources on failure
; @prompt: implement cleanup-on-error pattern
; @prompt: resource cleanup with error handling
; @prompt: free resources when operation fails
; @prompt: guarantee cleanup regardless of outcome
; @prompt: cleanup pattern for error safety
; @prompt: resource management with error cleanup
; @prompt: ensure no resource leaks on error
;
; @param: fail_at_step=r0 "Step to fail at (0=none, 1-3=step)"
;
; @test: r0=0 -> r0=0
; @test: r0=1 -> r0=1
; @test: r0=2 -> r0=2
; @test: r0=3 -> r0=3
;
; @note: Resources are always cleaned up regardless of which step fails
; @note: Cleanup order is reverse of acquisition order
;
; Cleanup on Error Pattern
; ========================
; Acquire resources, perform work, cleanup on any error.
; Uses structured cleanup to prevent resource leaks.

.entry main

.section .data

fail_step:          .dword 0
resource_a:         .dword 0        ; First resource
resource_b:         .dword 0        ; Second resource
resource_c:         .dword 0        ; Third resource
cleanup_count:      .dword 0        ; Track cleanups for testing

.section .text

main:
    ; r0 = step to fail at (0=success, 1/2/3=fail at step)
    mov r1, fail_step
    store.d r0, [r1]

    ; Acquire resource A
    call acquire_a
    bne r0, zero, fail_no_cleanup   ; Nothing to cleanup yet

    ; Acquire resource B
    call acquire_b
    bne r0, zero, cleanup_a         ; Cleanup A only

    ; Acquire resource C
    call acquire_c
    bne r0, zero, cleanup_ab        ; Cleanup A and B

    ; All acquisitions succeeded - do work
    call do_work

    ; Cleanup all resources (success path)
    call release_c
    call release_b
    call release_a

    mov r0, 0                       ; Success
    halt

fail_no_cleanup:
    ; Failed before acquiring anything
    mov r0, 1
    halt

cleanup_a:
    call release_a
    mov r0, 2
    halt

cleanup_ab:
    call release_b
    call release_a
    mov r0, 3
    halt

acquire_a:
    mov r1, fail_step
    load.d r1, [r1]
    mov r2, 1
    beq r1, r2, acquire_a_fail

    mov r1, resource_a
    mov r2, 1                       ; Acquired
    store.d r2, [r1]
    mov r0, 0
    ret

acquire_a_fail:
    mov r0, 1
    ret

acquire_b:
    mov r1, fail_step
    load.d r1, [r1]
    mov r2, 2
    beq r1, r2, acquire_b_fail

    mov r1, resource_b
    mov r2, 1
    store.d r2, [r1]
    mov r0, 0
    ret

acquire_b_fail:
    mov r0, 1
    ret

acquire_c:
    mov r1, fail_step
    load.d r1, [r1]
    mov r2, 3
    beq r1, r2, acquire_c_fail

    mov r1, resource_c
    mov r2, 1
    store.d r2, [r1]
    mov r0, 0
    ret

acquire_c_fail:
    mov r0, 1
    ret

release_a:
    mov r1, resource_a
    store.d zero, [r1]
    call increment_cleanup
    ret

release_b:
    mov r1, resource_b
    store.d zero, [r1]
    call increment_cleanup
    ret

release_c:
    mov r1, resource_c
    store.d zero, [r1]
    call increment_cleanup
    ret

increment_cleanup:
    mov r1, cleanup_count
    load.d r2, [r1]
    addi r2, r2, 1
    store.d r2, [r1]
    ret

do_work:
    ; Simulate doing actual work with resources
    ret
