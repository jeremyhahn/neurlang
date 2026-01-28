; @name: Health Check Endpoint
; @description: Service health check that verifies database connectivity
; @category: application/pattern
; @difficulty: 2
;
; @prompt: health check endpoint
; @prompt: service health status
; @prompt: health check api
; @prompt: liveness check endpoint
; @prompt: readiness probe
; @prompt: service status check
; @prompt: health check response
; @prompt: kubernetes health endpoint
; @prompt: service health api
; @prompt: health status endpoint
;
; Mock SQLite and JSON extensions for testing
; @mock: sqlite_open=1
; @mock: sqlite_prepare=1
; @mock: sqlite_step=1
; @mock: sqlite_finalize=0
; @mock: sqlite_close=0
; @mock: json_new_object=1
; @mock: json_set=0
; @mock: json_stringify=65536
; @mock: json_free=0
;
; @test: -> r0=65536
;
; @note: Composes: sqlite_query (ping), json_build
; @note: Standard health check pattern for microservices

.entry main

.section .data
    db_path:       .asciz "/tmp/app.db"
    ping_sql:      .asciz "SELECT 1"
    key_status:    .asciz "status"
    key_database:  .asciz "database"
    status_ok:     .asciz "ok"
    status_error:  .asciz "error"
    val_healthy:   .asciz "healthy"
    val_unhealthy: .asciz "unhealthy"

.section .text

main:
    mov r14, 1                       ; overall healthy flag

    ; Check database connectivity
    mov r0, db_path
    ext.call r10, sqlite_open, r0
    beq r10, zero, db_unhealthy

    mov r0, ping_sql
    ext.call r11, sqlite_prepare, r10, r0
    beq r11, zero, db_unhealthy_close

    ext.call r0, sqlite_step, r11
    ext.call r0, sqlite_finalize, r11
    ext.call r0, sqlite_close, r10

    mov r15, val_healthy
    b build_response

db_unhealthy_close:
    ext.call r0, sqlite_close, r10

db_unhealthy:
    mov r15, val_unhealthy
    mov r14, 0                       ; mark unhealthy

build_response:
    ; Build response JSON
    ext.call r1, json_new_object

    ; Set overall status
    mov r2, key_status
    beq r14, zero, set_error_status
    mov r3, status_ok
    b set_status
set_error_status:
    mov r3, status_error
set_status:
    ext.call r0, json_set, r1, r2, r3

    ; Set database status
    mov r2, key_database
    ext.call r0, json_set, r1, r2, r15

    ext.call r0, json_stringify, r1
    ext.call r4, json_free, r1
    halt
