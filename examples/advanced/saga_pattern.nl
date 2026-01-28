; @name: Saga Pattern
; @description: Distributed transaction saga with compensating actions
; @category: advanced/distributed
; @difficulty: 5
;
; @prompt: implement saga pattern for distributed transactions
; @prompt: saga with compensating transactions
; @prompt: orchestrated saga pattern
; @prompt: distributed transaction coordination
; @prompt: saga rollback on failure
; @prompt: microservice saga implementation
; @prompt: saga compensation logic
; @prompt: transaction saga orchestrator
; @prompt: saga pattern with rollback
; @prompt: distributed saga workflow
;
; @param: step1_success=r0 "Step 1 succeeds (0/1)"
; @param: step2_success=r1 "Step 2 succeeds (0/1)"
; @param: step3_success=r2 "Step 3 succeeds (0/1)"
;
; @test: r0=1, r1=1, r2=1 -> r0=1
; @test: r0=1, r1=1, r2=0 -> r0=0
; @test: r0=1, r1=0, r2=1 -> r0=0
; @test: r0=0, r1=1, r2=1 -> r0=0
;
; @note: Returns 1 if saga completes, 0 if compensated
; @note: Compensations run in reverse order on failure
;
; Saga Pattern
; ============
; Execute steps forward, compensate backward on failure.

.entry main

.section .data

step_status:        .space 8, 0     ; Track completed steps

.section .text

main:
    ; r0 = step1_success
    ; r1 = step2_success
    ; r2 = step3_success
    mov r10, r0
    mov r11, r1
    mov r12, r2

    ; Step 1: Reserve Inventory
    beq r10, zero, saga_failed

    ; Step 2: Process Payment
    beq r11, zero, compensate_step1

    ; Step 3: Ship Order
    beq r12, zero, compensate_step2

    ; All steps succeeded
    mov r0, 1
    halt

compensate_step2:
    ; Rollback step 2 (refund payment)
    ; Then fall through to compensate step 1

compensate_step1:
    ; Rollback step 1 (release inventory)

saga_failed:
    mov r0, 0
    halt
