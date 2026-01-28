; @name: Error Propagation
; @description: Propagate error codes up the call chain
; @category: patterns/error-handling
; @difficulty: 3
;
; @prompt: propagate error codes through call stack
; @prompt: bubble errors up call chain
; @prompt: implement error propagation pattern
; @prompt: pass error codes to caller
; @prompt: chain error handling through functions
; @prompt: propagate failure through layers
; @prompt: error code bubbling pattern
; @prompt: return error from nested calls
; @prompt: handle errors at each level
; @prompt: implement error result chaining
;
; @param: inject_error_level=r0 "Level to inject error (0=none, 1-3=level)"
;
; @test: r0=0 -> r0=0
; @test: r0=1 -> r0=1
; @test: r0=2 -> r0=2
; @test: r0=3 -> r0=3
;
; @note: Error codes: 0=success, 1=level1_error, 2=level2_error, 3=level3_error
; @note: Each level checks nested result and propagates if non-zero
;
; Error Propagation Pattern
; =========================
; Each function checks result from callee and propagates errors upward.

.entry main

.section .data

error_at_level:     .word 0         ; Which level to inject error

.section .text

main:
    ; r0 = level to inject error (0=no error)
    mov r1, error_at_level
    store.d r0, [r1]

    ; Start the call chain
    call level1
    ; r0 now contains the result (0=success, or error code)
    halt

level1:
    ; Call level2
    call level2
    bne r0, zero, level1_propagate  ; If error, propagate it

    ; Level2 succeeded - check if we should fail at level1
    mov r1, error_at_level
    load.d r1, [r1]
    mov r2, 1
    beq r1, r2, level1_fail

    ; All good at level1
    mov r0, 0
    ret

level1_fail:
    mov r0, 1                       ; Error code 1 = level1 error
    ret

level1_propagate:
    ; Error from lower level - just return it
    ret

level2:
    ; Call level3
    call level3
    bne r0, zero, level2_propagate

    ; Level3 succeeded - check if we should fail at level2
    mov r1, error_at_level
    load.d r1, [r1]
    mov r2, 2
    beq r1, r2, level2_fail

    mov r0, 0
    ret

level2_fail:
    mov r0, 2                       ; Error code 2 = level2 error
    ret

level2_propagate:
    ret

level3:
    ; Innermost level - check if we should fail
    mov r1, error_at_level
    load.d r1, [r1]
    mov r2, 3
    beq r1, r2, level3_fail

    mov r0, 0
    ret

level3_fail:
    mov r0, 3                       ; Error code 3 = level3 error
    ret
