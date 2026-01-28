; @name: Optimistic Locking
; @description: Version-based conflict detection for concurrent updates
; @category: patterns/database
; @difficulty: 4
;
; @prompt: implement optimistic locking with version
; @prompt: detect concurrent update conflicts
; @prompt: version-based optimistic concurrency
; @prompt: optimistic locking pattern
; @prompt: conflict detection on update
; @prompt: check version before update
; @prompt: prevent lost updates with versioning
; @prompt: optimistic concurrency control
; @prompt: version check and update atomically
; @prompt: detect stale data on update
;
; @param: expected_version=r0 "Version client expects"
; @param: current_version=r1 "Actual version in database"
;
; @test: r0=1, r1=1 -> r0=1
; @test: r0=1, r1=2 -> r0=0
; @test: r0=5, r1=5 -> r0=1
;
; @note: Returns 1 if update allowed, 0 if version conflict
; @note: Updates should increment version on success
;
; Optimistic Locking Pattern
; ==========================
; Check version matches before update, detect conflicts.

.entry main

.section .data

record_version:     .word 1         ; Current version in "database"

.section .text

main:
    ; r0 = expected version (from client)
    ; r1 = current version (from database)
    mov r10, r0                     ; r10 = expected
    mov r11, r1                     ; r11 = current

    ; Compare versions
    bne r10, r11, version_conflict

    ; Versions match - update is allowed
    ; Increment version
    addi r11, r11, 1
    mov r0, record_version
    store.d r11, [r0]

    mov r0, 1                       ; Update succeeded
    halt

version_conflict:
    mov r0, 0                       ; Conflict detected
    halt

; Full optimistic update implementation
update_with_version:
    ; r0 = record id
    ; r1 = expected version
    ; r2 = new data ptr
    mov r10, r0
    mov r11, r1
    mov r12, r2

    ; Start transaction
    call begin_transaction

    ; Read current version from database
    mov r0, r10
    call get_record_version
    mov r13, r0                     ; r13 = current version

    ; Compare versions
    bne r11, r13, conflict

    ; Versions match - perform update
    mov r0, r10                     ; record id
    mov r1, r12                     ; new data
    addi r2, r13, 1                 ; new version
    call perform_update

    ; Commit
    call commit_transaction
    mov r0, 1                       ; Success
    ret

conflict:
    ; Version mismatch - rollback and report
    call rollback_transaction
    mov r0, 0                       ; Conflict
    ret

begin_transaction:
    ; ext.call sqlite_begin
    ret

commit_transaction:
    ; ext.call sqlite_commit
    ret

rollback_transaction:
    ; ext.call sqlite_rollback
    ret

get_record_version:
    ; Would SELECT version FROM table WHERE id = ?
    mov r0, 1                       ; Mock version
    ret

perform_update:
    ; Would UPDATE table SET data=?, version=? WHERE id=?
    ret
