# Neurlang Templates (Skeletons)

Templates provide **application-level patterns** - the complete structure for servers and clients. They define the control flow, state machines, and slot placeholders that the model fills.

## Architecture: Templates vs Stdlib vs Extensions

| Layer | Purpose | Examples |
|-------|---------|----------|
| **Templates** | Application flow patterns | `tcp_server.skeleton`, `websocket_server.skeleton` |
| **Stdlib** | Reusable primitives (model calls these) | `strlen`, `json_parse`, `ws_frame_decode` |
| **Extensions** | Complex libraries via FFI | TLS, HTTP/2, Protobuf, QUIC |

**Templates** = WHERE code goes (structure)
**Stdlib** = WHAT code does (building blocks)
**Extensions** = HEAVY lifting (crypto, compression)

## Available Templates

### Network Servers

| Template | Transport | Use Case |
|----------|-----------|----------|
| `tcp_server.skeleton` | TCP | Line-based protocols (SMTP, FTP, Redis) |
| `udp_server.skeleton` | UDP | Datagram protocols (DNS, DHCP, game servers) |
| `unix_socket_server.skeleton` | Unix Domain | IPC, local services |
| `http_server.skeleton` | HTTP/1.1 | Web servers, webhooks |
| `rest_api.skeleton` | HTTP REST | JSON APIs with CRUD |
| `websocket_server.skeleton` | WebSocket | Real-time bidirectional |
| `json_rpc_server.skeleton` | JSON-RPC 2.0 | RPC over HTTP or TCP |
| `grpc_server.skeleton` | gRPC | High-performance RPC (requires extensions) |

### Clients

| Template | Transport | Use Case |
|----------|-----------|----------|
| `http_client.skeleton` | HTTP/1.1 | REST API consumption, webhooks |
| `grpc_client.skeleton` | gRPC | High-performance RPC client |
| `mqtt_client.skeleton` | MQTT 3.1.1 | IoT pub/sub messaging |

### Applications

| Template | Type | Use Case |
|----------|------|----------|
| `cli_application.skeleton` | CLI | Command-line tools with args/flags |
| `database_crud.skeleton` | Database | CRUD operations (SQLite, Postgres, MySQL) |
| `worker_service.skeleton` | Worker | Background job processing, queue consumers |

## Stdlib Modules Required

For these templates to work effectively, the model needs stdlib primitives:

### Currently Available (lib/)
- `math/` - Arithmetic, factorial, gcd
- `float/` - FPU operations
- `string/` - strlen, strcmp, strcpy, atoi, itoa
- `array/` - Sorting, searching
- `bitwise/` - Bit manipulation
- `collections/` - Stack, queue, hashtable

### Needed (TO BE ADDED)

```
lib/
├── net/                 # Network primitives
│   ├── socket_helpers.nl    # Socket utility functions
│   └── address_parse.nl     # IP/port parsing
│
├── json/                # JSON handling
│   ├── json_parse.nl        # Parse JSON to tokens
│   ├── json_get_string.nl   # Extract string field
│   ├── json_get_number.nl   # Extract number field
│   ├── json_get_bool.nl     # Extract boolean field
│   ├── json_get_array.nl    # Extract array
│   ├── json_build.nl        # Build JSON string
│   └── json_escape.nl       # Escape special chars
│
├── http/                # HTTP helpers
│   ├── parse_request_line.nl   # Parse "GET /path HTTP/1.1"
│   ├── parse_headers.nl        # Parse headers into pairs
│   ├── find_header.nl          # Find specific header
│   ├── parse_content_length.nl # Extract Content-Length
│   └── url_decode.nl           # URL decoding
│
├── websocket/           # WebSocket framing
│   ├── ws_frame_decode.nl   # Decode incoming frame
│   ├── ws_frame_encode.nl   # Encode outgoing frame
│   ├── ws_unmask.nl         # XOR unmask payload
│   └── ws_accept_key.nl     # Compute Sec-WebSocket-Accept
│
├── base64/              # Base64 encoding
│   ├── base64_encode.nl
│   └── base64_decode.nl
│
├── mqtt/                # MQTT packet helpers
│   ├── mqtt_remaining_length.nl  # Variable-length encoding
│   ├── mqtt_parse_string.nl      # Parse MQTT string
│   └── mqtt_build_string.nl      # Build MQTT string
│
└── rpc/                 # RPC helpers
    ├── jsonrpc_parse.nl     # Parse JSON-RPC request
    └── jsonrpc_build.nl     # Build JSON-RPC response
```

## Extension Requirements

Some templates require extensions for complex functionality:

| Template | Required Extensions |
|----------|---------------------|
| `websocket_server.skeleton` | `sha1` (for handshake) |
| `grpc_server.skeleton` | `http2`, `protobuf`, optionally `tls` |
| `grpc_client.skeleton` | `http2`, `protobuf`, optionally `tls` |
| `http_client.skeleton` | `http_get`, `http_post`, etc. (IDs 190-198) |
| `database_crud.skeleton` | `sqlite_*` or `pg_*` or `mysql_*` |
| `worker_service.skeleton` | Queue-specific extensions (Redis, SQS, etc.) |

### Why Not Pure Neurlang?

Certain protocols are too complex for slot-based generation:

1. **QUIC** - Requires TLS 1.3 crypto, congestion control, packet encryption
2. **HTTP/2** - HPACK compression, complex framing, flow control
3. **TLS** - Cryptographic operations, certificate handling
4. **Protobuf** - Schema-based encoding requires codegen

These should be **extensions** that the model calls via `ext.call`, not Neurlang assembly.

## Template Slot Conventions

Templates use placeholder markers for slots:

```asm
; Slot that reads a command
{{SLOT_READ_COMMAND}}

; Slot that dispatches to handlers
{{SLOT_COMMAND_DISPATCH}}

; Data items from SlotSpec
{{DATA_ITEMS}}

; Generated handler code
{{COMMAND_HANDLERS}}
```

The slot filler model generates assembly for each `{{SLOT_*}}` based on:
1. Slot type (PatternMatch, ResponseBuilder, etc.)
2. Context (available registers, labels, data refs)
3. Protocol specification

## Adding New Templates

1. Create `templates/<name>.skeleton`
2. Define register conventions in header comments
3. Add data section with protocol constants
4. Mark slots with `{{SLOT_*}}` placeholders
5. Document required stdlib/extensions

## Usage

```bash
# Generate from protocol spec (uses appropriate template)
nl generate "SMTP server" --dry-run   # Shows which template

# Explicit template selection
nl generate --template tcp_server --spec specs/protocols/smtp.yaml

# List available templates
ls templates/*.skeleton
```
