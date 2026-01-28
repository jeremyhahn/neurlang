; @name: Migration Pattern
; @description: Database schema migration with version tracking
; @category: patterns/database
; @difficulty: 4
;
; @prompt: implement database migration pattern
; @prompt: track schema version in database
; @prompt: run database migrations
; @prompt: schema migration with versioning
; @prompt: upgrade database schema
; @prompt: migration version check and apply
; @prompt: database schema upgrade pattern
; @prompt: apply pending migrations
; @prompt: version-controlled schema changes
; @prompt: migrate database to latest version
;
; @param: current_version=r0 "Current schema version"
; @param: target_version=r1 "Target schema version"
;
; @test: r0=1, r1=3 -> r0=3
; @test: r0=3, r1=3 -> r0=3
; @test: r0=3, r1=1 -> r0=0
;
; @note: Returns new version on success, 0 if downgrade attempted
; @note: Applies migrations sequentially from current to target
;
; Migration Pattern
; =================
; Track schema version, apply migrations in order.

.entry main

.section .data

schema_version:     .word 1         ; Current schema version
migration_count:    .word 5         ; Total available migrations

.section .text

main:
    ; r0 = current version
    ; r1 = target version
    mov r10, r0                     ; r10 = current
    mov r11, r1                     ; r11 = target

    ; Check for downgrade (not supported)
    bgt r10, r11, downgrade_error

    ; Already at target?
    beq r10, r11, already_current

    ; Apply migrations
migration_loop:
    bge r10, r11, migration_done

    ; Apply next migration
    addi r10, r10, 1
    mov r0, r10
    call apply_migration
    bne r0, zero, migration_error

    b migration_loop

migration_done:
    ; Update stored version
    mov r0, schema_version
    store.d r10, [r0]
    mov r0, r10                     ; Return new version
    halt

already_current:
    mov r0, r10                     ; Already at target
    halt

downgrade_error:
    mov r0, 0                       ; Downgrade not supported
    halt

migration_error:
    ; Migration failed - version unchanged
    mov r0, 0
    halt

apply_migration:
    ; r0 = migration version to apply
    mov r5, r0

    ; Dispatch to appropriate migration
    mov r1, 1
    beq r5, r1, migration_v1
    mov r1, 2
    beq r5, r1, migration_v2
    mov r1, 3
    beq r5, r1, migration_v3

    ; Unknown migration
    mov r0, 1                       ; Error
    ret

migration_v1:
    ; CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)
    mov r0, 0                       ; Success
    ret

migration_v2:
    ; ALTER TABLE users ADD COLUMN email TEXT
    mov r0, 0
    ret

migration_v3:
    ; CREATE INDEX idx_users_email ON users(email)
    mov r0, 0
    ret
