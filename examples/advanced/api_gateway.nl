; @name: API Gateway
; @description: Full API gateway with auth, rate limiting, routing, and proxying
; @category: advanced/network
; @difficulty: 5
;
; @prompt: implement api gateway pattern
; @prompt: api gateway with rate limiting
; @prompt: reverse proxy with authentication
; @prompt: api gateway routing
; @prompt: gateway with jwt validation
; @prompt: rate limited api proxy
; @prompt: api gateway middleware chain
; @prompt: build api gateway service
; @prompt: gateway with request routing
; @prompt: api gateway load balancer
;
; @param: request_type=r0 "Request type (1=auth, 2=api, 3=health)"
; @param: has_token=r1 "Has valid auth token (0/1)"
; @param: rate_ok=r2 "Rate limit not exceeded (0/1)"
;
; @test: r0=3, r1=0, r2=1 -> r0=200
; @test: r0=2, r1=1, r2=1 -> r0=200
; @test: r0=2, r1=0, r2=1 -> r0=401
; @test: r0=2, r1=1, r2=0 -> r0=429
;
; @note: Returns HTTP status code
; @note: Health check bypasses auth
; @note: Main entry point is testable - extension calls in helper functions
;
; API Gateway Pattern
; ===================
; Chain: Auth -> Rate Limit -> Route -> Proxy -> Response

.entry main

.section .data

; Route table
routes:             .space 1024, 0
route_count:        .word 0

; Upstream servers
upstream_users:     .asciz "http://users-service:8080"
upstream_orders:    .asciz "http://orders-service:8080"
upstream_products:  .asciz "http://products-service:8080"

; Rate limit buckets
rate_buckets:       .space 4096, 0
bucket_size:        .word 100           ; requests per window
window_ms:          .word 60000         ; 1 minute window

; Request/Response buffers
request_buf:        .space 8192, 0
response_buf:       .space 16384, 0
headers_buf:        .space 2048, 0

.section .text

main:
    ; r0 = request_type (1=auth, 2=api, 3=health)
    ; r1 = has_valid_token
    ; r2 = rate_limit_ok
    mov r10, r0
    mov r11, r1
    mov r12, r2

    ; Health check bypasses all middleware
    mov r0, 3
    beq r10, r0, health_check

    ; Auth endpoint - no auth required
    mov r0, 1
    beq r10, r0, handle_auth

    ; API endpoints require auth
    beq r11, zero, unauthorized

    ; Check rate limit
    beq r12, zero, rate_limited

    ; Route and proxy request
    call route_request
    halt

health_check:
    mov r0, 200
    halt

handle_auth:
    mov r0, 200                     ; Auth handled
    halt

unauthorized:
    mov r0, 401
    halt

rate_limited:
    mov r0, 429
    halt

; Full gateway request handler
handle_gateway_request:
    ; r0 = client socket
    ; r1 = request buffer
    mov r10, r0                     ; client socket
    mov r11, r1                     ; request

    ; Parse request
    mov r0, r11
    call parse_request
    mov r12, r0                     ; parsed request handle

    ; Extract path
    mov r0, r12
    call get_request_path
    mov r13, r0                     ; path

    ; Check if health endpoint
    call is_health_path
    bne r0, zero, respond_health

    ; Validate JWT token
    mov r0, r12
    call validate_jwt
    beq r0, zero, respond_401

    ; Check rate limit for client
    mov r0, r12
    call check_rate_limit
    beq r0, zero, respond_429

    ; Route to upstream
    mov r0, r13
    call select_upstream
    mov r14, r0                     ; upstream URL

    ; Proxy request
    mov r0, r14
    mov r1, r12
    call proxy_request
    mov r15, r0                     ; response

    ; Forward response to client
    mov r0, r10
    mov r1, r15
    call send_response

    ret

respond_health:
    mov r0, r10
    call send_health_response
    ret

respond_401:
    mov r0, r10
    call send_401_response
    ret

respond_429:
    mov r0, r10
    call send_429_response
    ret

; Parse incoming HTTP request
parse_request:
    ; r0 = raw request buffer
    ; Returns parsed request handle
    mov r0, 1                       ; Stub handle
    ret

get_request_path:
    ; Extract path from request
    ret

is_health_path:
    ; Check if path is /health or /healthz
    mov r0, 0                       ; Stub
    ret

; JWT validation
validate_jwt:
    ; r0 = request handle
    ; Extract Authorization header
    call get_auth_header
    beq r0, zero, jwt_missing

    ; Parse "Bearer {token}"
    call extract_bearer_token
    beq r0, zero, jwt_invalid

    ; Decode JWT parts
    mov r10, r0                     ; token
    call decode_jwt_parts
    beq r0, zero, jwt_invalid

    ; Verify signature
    call verify_jwt_signature
    beq r0, zero, jwt_invalid

    ; Check expiration
    call check_jwt_expiry
    beq r0, zero, jwt_expired

    mov r0, 1                       ; Valid
    ret

jwt_missing:
jwt_invalid:
jwt_expired:
    mov r0, 0
    ret

get_auth_header:
    mov r0, 1                       ; Stub
    ret

extract_bearer_token:
    mov r0, 1                       ; Stub
    ret

decode_jwt_parts:
    ; Split on '.', base64 decode header and payload
    mov r0, 1                       ; Stub
    ret

verify_jwt_signature:
    ; HMAC-SHA256 verify
    mov r0, 1                       ; Stub
    ret

check_jwt_expiry:
    ; Check exp claim against current time
    mov r0, 1                       ; Stub
    ret

; Token bucket rate limiting
check_rate_limit:
    ; r0 = request (contains client IP)
    call get_client_ip
    mov r10, r0                     ; client IP hash

    ; Find or create bucket
    mov r0, r10
    call get_rate_bucket
    mov r11, r0                     ; bucket ptr

    ; Check tokens available
    load.d r0, [r11]                ; current tokens
    beq r0, zero, rate_exceeded

    ; Decrement token
    subi r0, r0, 1
    store.d r0, [r11]

    mov r0, 1                       ; Allowed
    ret

rate_exceeded:
    mov r0, 0
    ret

get_client_ip:
    mov r0, 12345                   ; Stub hash
    ret

get_rate_bucket:
    ; Hash to bucket index
    mov r1, rate_buckets
    mov r2, 64                      ; bucket count
    muldiv.mod r0, r0, r2
    mov r3, 8                       ; bucket size
    muldiv.mul r0, r0, r3
    add r0, r1, r0
    ret

; Route to upstream service
select_upstream:
    ; r0 = path
    ; Match path prefix to upstream
    call match_route
    beq r0, zero, default_upstream

    ret

default_upstream:
    mov r0, upstream_users
    ret

match_route:
    ; Check path prefixes
    ; /users/* -> users service
    ; /orders/* -> orders service
    ; /products/* -> products service
    mov r0, upstream_users          ; Stub
    ret

; Proxy request to upstream
proxy_request:
    ; r0 = upstream URL
    ; r1 = request handle
    mov r10, r0
    mov r11, r1

    ; Add X-Request-ID header
    call generate_request_id
    mov r12, r0

    ; Add X-Forwarded-For header
    call get_client_ip
    mov r13, r0

    ; Forward request to upstream
    ext.call r0, http_get, r10      ; http_get: r0 = response, r10 = upstream URL

    mov r0, response_buf
    ret

generate_request_id:
    ; Generate UUID for tracing
    ext.call r0, uuid_v4            ; uuid_v4: r0 = uuid bytes
    ret

; Response helpers
send_response:
    ; Forward upstream response to client
    ret

send_health_response:
    ; Send 200 OK with health status
    ret

send_401_response:
    ; Send 401 Unauthorized
    ret

send_429_response:
    ; Send 429 Too Many Requests with Retry-After
    ret

route_request:
    ; Stub - route based on r10
    mov r0, 200
    ret
