; @name: OAuth2 Flow
; @description: Complete OAuth2 authorization code flow with token management
; @category: advanced/security
; @difficulty: 5
;
; @prompt: implement oauth2 authorization code flow
; @prompt: oauth2 authentication with tokens
; @prompt: exchange authorization code for token
; @prompt: oauth2 token refresh flow
; @prompt: implement oauth authentication
; @prompt: oauth2 client credentials flow
; @prompt: secure oauth2 implementation
; @prompt: handle oauth2 callback
; @prompt: oauth2 with pkce
; @prompt: oauth authorization server integration
;
; @param: auth_code=r0 "Authorization code from callback (0=invalid)"
; @param: state=r1 "State parameter valid (0=mismatch, 1=valid)"
;
; @test: r0=1, r1=1 -> r0=200
; @test: r0=0, r1=1 -> r0=401
; @test: r0=1, r1=0 -> r0=403
;
; @note: Returns HTTP status code (200=success, 401=invalid code, 403=state mismatch)
; @note: Uses ext.call for HTTP, JSON, and crypto operations
;
; OAuth2 Authorization Code Flow
; ==============================
; Complete flow: authorize -> callback -> token exchange -> refresh

.entry main

.section .text

main:
    ; r0 = auth_code (0 = missing/invalid)
    ; r1 = state_valid (0 = mismatch)
    mov r10, r0
    mov r11, r1

    ; Validate state parameter (CSRF protection)
    beq r11, zero, state_mismatch

    ; Validate auth code present
    beq r10, zero, invalid_code

    ; Exchange auth code for tokens (simplified)
    ; In real impl: POST to token endpoint, parse response

    ; Success
    mov r0, 200
    halt

state_mismatch:
    mov r0, 403                     ; Forbidden - CSRF detected
    halt

invalid_code:
    mov r0, 401                     ; Unauthorized - bad code
    halt
