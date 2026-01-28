# Protocol Specification Format

This document describes the YAML format for defining protocol specifications that drive slot-based code generation.

## Overview

Protocol specifications define:
- Transport layer (TCP, UDP, Unix socket)
- State machine (states and transitions)
- Commands (patterns, handlers, responses)
- Error handling
- Test cases for verification

## File Location

Protocol specs are stored in `specs/protocols/`:

```
specs/
  protocols/
    smtp.yaml
    http.yaml
    redis.yaml
    ftp.yaml
    dns.yaml
    custom_protocol.yaml
```

## Schema Reference

### Top-Level Fields

```yaml
# Required metadata
name: smtp                    # Protocol identifier
description: "Simple Mail Transfer Protocol (RFC 5321)"
version: "1.0"                # Spec version (not protocol version)

# Transport configuration
transport: tcp                # tcp, udp, unix
port: 25                      # Default port
line_ending: "\r\n"           # Line delimiter (optional)

# Connection handling
greeting:                     # Sent on connect (optional)
  format: "220 {hostname} SMTP Ready\r\n"

# State machine
states:
  - name: INIT
    initial: true             # Starting state
  - name: GREETED
  - name: QUIT
    terminal: true            # Connection closes after

# Command definitions
commands:
  - name: HELO
    pattern: "HELO {domain:word}"
    valid_states: [INIT]
    handler:
      type: simple_response
      response: "250 Hello {domain}\r\n"
      next_state: GREETED

# Error responses
errors:
  syntax: "500 Syntax error\r\n"
  sequence: "503 Bad sequence of commands\r\n"

# Test cases
tests:
  - name: basic_session
    steps:
      - send: "HELO test.com\r\n"
        expect: "250 Hello test.com\r\n"
```

---

## State Machine

### State Definition

```yaml
states:
  - name: STATE_NAME          # Uppercase convention
    initial: true             # Only one state can be initial
    terminal: false           # If true, connection closes after
    description: "Optional description"
```

### State Transition Rules

States are referenced in command handlers:

```yaml
commands:
  - name: COMMAND
    valid_states: [STATE_A, STATE_B]  # Allowed current states
    handler:
      next_state: STATE_C             # State after handler
```

Special state names:
- `ANY` - Command valid in any state
- `SAME` - Stay in current state (default if not specified)

---

## Commands

### Command Pattern

Patterns use placeholders with optional type hints:

```yaml
pattern: "COMMAND {arg1} {arg2:word}"
```

**Capture Types:**

| Type | Syntax | Description |
|------|--------|-------------|
| word | `{name:word}` or `{name}` | Until whitespace (default) |
| until | `{name:until:X}` | Until character X |
| quoted | `{name:quoted}` | Between quotes |
| rest | `{name:rest}` | Rest of line |
| int | `{name:int}` | Parse as integer |

**Examples:**
```yaml
# Simple word capture
pattern: "HELO {domain}"

# Capture until specific char
pattern: "MAIL FROM:<{sender:until:>}>"

# Multiple captures
pattern: "USER {username} PASS {password:rest}"

# Quoted string
pattern: "SET {key} {value:quoted}"
```

### Handler Types

#### simple_response

Send a fixed response with optional variable substitution.

```yaml
handler:
  type: simple_response
  response: "250 Hello {domain}\r\n"
  next_state: GREETED
```

#### multi_line_response

Send multiple response lines.

```yaml
handler:
  type: multi_line_response
  lines:
    - "250-{hostname}"
    - "250-SIZE 10240000"
    - "250-PIPELINING"
    - "250 OK"
  next_state: GREETED
```

#### validated_response

Validate input before responding.

```yaml
handler:
  type: validated_response
  validation:
    type: db_lookup           # db_lookup, regex, range, extension
    query: "SELECT 1 FROM users WHERE email = ?"
    param: recipient
  response_ok: "250 OK\r\n"
  response_err: "550 User not found\r\n"
  next_state: RCPT_TO
```

**Validation Types:**

| Type | Description | Parameters |
|------|-------------|------------|
| db_lookup | Database query | query, param |
| regex | Regular expression | pattern, param |
| range | Numeric range | min, max, param |
| extension | Custom extension | name, param |

#### multiline_reader

Read multiple lines until terminator.

```yaml
handler:
  type: multiline_reader
  response: "354 Start mail input\r\n"
  terminator: ".\r\n"         # Read until this line
  max_size: 10485760          # 10MB limit
  on_complete:
    response: "250 OK: Message accepted\r\n"
    next_state: GREETED
```

#### close_connection

Send response and close.

```yaml
handler:
  type: close_connection
  response: "221 Bye\r\n"
```

#### custom

Execute custom slot sequence.

```yaml
handler:
  type: custom
  slots:
    - type: ExtensionCall
      extension: "authenticate user"
      args: [username, password]
      result_reg: r5
    - type: RangeCheck
      value_reg: r5
      min: 1
      max: 1
      ok_label: auth_ok
      error_label: auth_fail
```

---

## Error Handling

### Error Definitions

```yaml
errors:
  # Named error types
  syntax: "500 Syntax error\r\n"
  sequence: "503 Bad sequence of commands\r\n"
  not_found: "550 User not found\r\n"
  auth_failed: "535 Authentication failed\r\n"

  # Default for unmatched commands
  unknown: "502 Command not recognized\r\n"
```

### Error Triggers

Errors are triggered by:
1. **State mismatch** - Command not valid in current state -> `sequence`
2. **Pattern mismatch** - No command pattern matches -> `unknown`
3. **Validation failure** - Handler validation fails -> custom error
4. **Parse failure** - Invalid syntax -> `syntax`

---

## Test Cases

### Basic Tests

```yaml
tests:
  - name: test_name
    description: "Optional description"
    steps:
      - send: "INPUT\r\n"
        expect: "EXPECTED OUTPUT\r\n"
        timeout_ms: 1000      # Optional, default 5000
```

### Multi-Step Tests

```yaml
tests:
  - name: full_smtp_session
    steps:
      - expect: "220"                    # Initial greeting
      - send: "HELO test.com\r\n"
        expect: "250 Hello test.com\r\n"
      - send: "MAIL FROM:<sender@test.com>\r\n"
        expect: "250 OK\r\n"
      - send: "RCPT TO:<user@example.com>\r\n"
        expect: "250 OK\r\n"
      - send: "QUIT\r\n"
        expect: "221 Bye\r\n"
```

### Pattern Matching in Tests

```yaml
tests:
  - name: test_with_patterns
    steps:
      - send: "COMMAND\r\n"
        expect_pattern: "^200 OK.*$"     # Regex pattern
      - send: "COMMAND\r\n"
        expect_contains: "success"       # Substring match
      - send: "COMMAND\r\n"
        expect_not_contains: "error"
```

### Test Setup

```yaml
tests:
  - name: test_with_setup
    setup:
      # Pre-populate database
      sql: |
        INSERT INTO users (email) VALUES ('test@example.com');
      # Or set initial state
      state: GREETED
    steps:
      - send: "RCPT TO:<test@example.com>\r\n"
        expect: "250 OK\r\n"
```

---

## Complete Example: SMTP

```yaml
name: smtp
description: "Simple Mail Transfer Protocol (RFC 5321)"
version: "1.0"

transport: tcp
port: 25
line_ending: "\r\n"

greeting:
  format: "220 {hostname} SMTP Ready\r\n"

states:
  - name: INIT
    initial: true
    description: "Initial state, waiting for HELO/EHLO"
  - name: GREETED
    description: "Client has identified itself"
  - name: MAIL_FROM
    description: "Sender specified"
  - name: RCPT_TO
    description: "At least one recipient specified"
  - name: DATA
    description: "Reading message body"
  - name: QUIT
    terminal: true

commands:
  - name: HELO
    pattern: "HELO {domain:word}"
    valid_states: [INIT]
    handler:
      type: simple_response
      response: "250 Hello {domain}\r\n"
      next_state: GREETED

  - name: EHLO
    pattern: "EHLO {domain:word}"
    valid_states: [INIT]
    handler:
      type: multi_line_response
      lines:
        - "250-{hostname}"
        - "250-SIZE 10240000"
        - "250-PIPELINING"
        - "250 OK"
      next_state: GREETED

  - name: MAIL_FROM
    pattern: "MAIL FROM:<{sender:until:>}>"
    valid_states: [GREETED]
    handler:
      type: simple_response
      response: "250 OK\r\n"
      next_state: MAIL_FROM

  - name: RCPT_TO
    pattern: "RCPT TO:<{recipient:until:>}>"
    valid_states: [MAIL_FROM, RCPT_TO]
    handler:
      type: validated_response
      validation:
        type: db_lookup
        query: "SELECT 1 FROM users WHERE email = ?"
        param: recipient
      response_ok: "250 OK\r\n"
      response_err: "550 User not found\r\n"
      next_state: RCPT_TO

  - name: DATA
    pattern: "DATA"
    valid_states: [RCPT_TO]
    handler:
      type: multiline_reader
      response: "354 Start mail input\r\n"
      terminator: ".\r\n"
      max_size: 10485760
      on_complete:
        response: "250 OK: Message accepted\r\n"
        next_state: GREETED

  - name: RSET
    pattern: "RSET"
    valid_states: [ANY]
    handler:
      type: simple_response
      response: "250 OK\r\n"
      next_state: GREETED

  - name: NOOP
    pattern: "NOOP"
    valid_states: [ANY]
    handler:
      type: simple_response
      response: "250 OK\r\n"
      next_state: SAME

  - name: QUIT
    pattern: "QUIT"
    valid_states: [ANY]
    handler:
      type: close_connection
      response: "221 Bye\r\n"

errors:
  syntax: "500 Syntax error\r\n"
  sequence: "503 Bad sequence of commands\r\n"
  not_found: "550 User not found\r\n"
  unknown: "502 Command not recognized\r\n"

tests:
  - name: basic_session
    steps:
      - expect: "220"
      - send: "HELO test.com\r\n"
        expect: "250 Hello test.com\r\n"
      - send: "MAIL FROM:<sender@test.com>\r\n"
        expect: "250 OK\r\n"
      - send: "RCPT TO:<user@example.com>\r\n"
        expect: "250 OK\r\n"
      - send: "QUIT\r\n"
        expect: "221 Bye\r\n"

  - name: wrong_sequence
    steps:
      - expect: "220"
      - send: "MAIL FROM:<sender@test.com>\r\n"
        expect: "503 Bad sequence of commands\r\n"

  - name: unknown_command
    steps:
      - expect: "220"
      - send: "INVALID\r\n"
        expect: "502 Command not recognized\r\n"
```

---

## Protocol Variables

Built-in variables available in templates:

| Variable | Description |
|----------|-------------|
| `{hostname}` | Server hostname |
| `{timestamp}` | Current ISO timestamp |
| `{client_ip}` | Client IP address |
| `{connection_id}` | Unique connection ID |

Custom variables from command captures are also available.

---

## See Also

- [Slot Architecture Overview](./README.md)
- [Slot Types Reference](./slot-types.md)
- [CLI: nl generate](../cli/generate.md)
