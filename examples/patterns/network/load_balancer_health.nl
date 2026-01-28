; @name: Load Balancer Health Check
; @description: Health check endpoint for load balancer
; @category: patterns/network
; @difficulty: 2
;
; @prompt: implement health check endpoint
; @prompt: create load balancer health check
; @prompt: health status endpoint
; @prompt: kubernetes liveness probe endpoint
; @prompt: implement /health endpoint
; @prompt: health check for load balancer
; @prompt: ready check endpoint
; @prompt: service health endpoint
; @prompt: liveness and readiness checks
; @prompt: health check response pattern
;
; @param: is_healthy=r0 "Service health status (0=unhealthy, 1=healthy)"
;
; @test: r0=1 -> r0=200
; @test: r0=0 -> r0=503
;
; @note: Returns HTTP status code (200=healthy, 503=unhealthy)

.entry main

.section .data

status_200:         .dword 200
status_503:         .dword 503

.section .text

main:
    ; r0 = is_healthy
    mov r10, r0

    beq r10, zero, unhealthy

    ; Healthy
    mov r0, status_200
    load.d r0, [r0]
    halt

unhealthy:
    mov r0, status_503
    load.d r0, [r0]
    halt
