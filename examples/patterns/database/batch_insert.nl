; @name: Batch Insert
; @description: Bulk insert with transaction for atomicity
; @category: patterns/database
; @difficulty: 3
;
; @prompt: batch insert records in transaction
; @prompt: bulk insert with transaction
; @prompt: insert multiple records atomically
; @prompt: batch database insert
; @prompt: insert batch with rollback on failure
; @prompt: bulk insert pattern
; @prompt: multi-row insert in transaction
; @prompt: atomic batch insert
; @prompt: insert array of records
; @prompt: batch write to database
;
; @param: count=r0 "Number of records to insert"
; @param: fail_at=r1 "Which record fails (0=none)"
;
; @test: r0=3, r1=0 -> r0=3
; @test: r0=3, r1=2 -> r0=0
; @test: r0=0, r1=0 -> r0=0
;
; @note: Returns count of inserted records (0 if rolled back)
; @note: All-or-nothing insertion
;
; Batch Insert Pattern
; ====================
; Insert multiple records atomically with transaction.

.entry main

.section .data

db_path:            .asciz "test.db"
insert_sql:         .asciz "INSERT INTO items (name) VALUES (?)"

.section .text

main:
    ; r0 = count to insert
    ; r1 = which index fails (0 = none fail)
    mov r10, r0                     ; r10 = total count
    mov r11, r1                     ; r11 = fail_at
    mov r12, 0                      ; r12 = current index
    mov r13, 0                      ; r13 = success count

    ; Handle empty batch
    beq r10, zero, batch_empty

    ; Begin transaction
    call begin_transaction

insert_loop:
    bge r12, r10, batch_success

    ; Check if this insert should fail
    beq r11, zero, do_insert
    mov r0, r12
    addi r0, r0, 1                  ; 1-based index
    beq r0, r11, insert_failed

do_insert:
    ; Insert record
    call insert_record
    addi r13, r13, 1
    addi r12, r12, 1
    b insert_loop

batch_success:
    ; All inserts succeeded - commit
    call commit_transaction
    mov r0, r13
    halt

insert_failed:
    ; One insert failed - rollback all
    call rollback_transaction
    mov r0, 0
    halt

batch_empty:
    mov r0, 0
    halt

begin_transaction:
    ; ext.call 250, db_handle, zero, zero  ; sqlite_begin
    ret

commit_transaction:
    ; ext.call 251, db_handle, zero, zero  ; sqlite_commit
    ret

rollback_transaction:
    ; ext.call 252, db_handle, zero, zero  ; sqlite_rollback
    ret

insert_record:
    ; Prepare and execute insert
    ; ext.call 247 (prepare), 248 (bind), 249 (step)
    ret
