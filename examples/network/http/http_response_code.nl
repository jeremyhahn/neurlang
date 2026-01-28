; @name: HTTP Response Code Builder
; @description: Returns appropriate HTTP status code for different scenarios
; @category: network/http
; @difficulty: 2
;
; @prompt: get HTTP status code for operation result
; @prompt: return 200 for success, 404 for not found
; @prompt: map operation result to HTTP status
; @prompt: HTTP status code selector
; @prompt: choose HTTP response code
; @prompt: return appropriate HTTP status
; @prompt: HTTP status for CRUD operations
; @prompt: map result to 200/201/204/400/404/500
; @prompt: get status code for REST response
; @prompt: HTTP response status selector
;
; @param: op=r0 "Operation type: 0=OK, 1=CREATED, 2=NO_CONTENT, 3=BAD_REQUEST, 4=NOT_FOUND, 5=ERROR"
;
; @test: r0=0 -> r0=200
; @test: r0=1 -> r0=201
; @test: r0=2 -> r0=204
; @test: r0=3 -> r0=400
; @test: r0=4 -> r0=404
; @test: r0=5 -> r0=500

.entry main

main:
    ; r0 = operation result code
    ; Return corresponding HTTP status

    mov r1, 0
    beq r0, r1, status_200

    mov r1, 1
    beq r0, r1, status_201

    mov r1, 2
    beq r0, r1, status_204

    mov r1, 3
    beq r0, r1, status_400

    mov r1, 4
    beq r0, r1, status_404

    ; Default: 500 Internal Server Error
    mov r0, 500
    halt

status_200:
    mov r0, 200
    halt

status_201:
    mov r0, 201
    halt

status_204:
    mov r0, 204
    halt

status_400:
    mov r0, 400
    halt

status_404:
    mov r0, 404
    halt
