#!/usr/bin/env python3
"""
HTTP Pattern Generator
======================

Generates training samples for HTTP protocol patterns:
- Header parsing (Content-Type, Authorization, Accept, etc.)
- Status code responses (200, 201, 400, 401, 403, 404, 500)
- Content-Type handling
- Request body parsing
- Response building with proper headers
- HTTP method routing
- Query string parsing
- Cookie handling
"""

import random
from typing import List, Dict, Tuple, Optional
from dataclasses import dataclass


@dataclass
class Instruction:
    """Instruction representation for training data."""
    opcode: int
    mode: int
    rd: int
    rs1: int
    rs2: int
    has_imm: int
    imm: int

    def to_dict(self) -> Dict[str, int]:
        return {
            'valid': 1,
            'opcode': self.opcode,
            'mode': self.mode,
            'rd': self.rd,
            'rs1': self.rs1,
            'rs2': self.rs2,
            'has_imm': self.has_imm,
            'imm_bin': self.imm if self.has_imm else 0,
        }


# HTTP Status codes
STATUS_CODES = {
    200: ('OK', 'success response', 'request succeeded'),
    201: ('Created', 'resource created', 'creation succeeded'),
    204: ('No Content', 'empty response', 'success with no body'),
    400: ('Bad Request', 'invalid request', 'malformed input'),
    401: ('Unauthorized', 'authentication required', 'not authenticated'),
    403: ('Forbidden', 'access denied', 'permission denied'),
    404: ('Not Found', 'resource missing', 'endpoint not found'),
    409: ('Conflict', 'resource conflict', 'state conflict'),
    422: ('Unprocessable Entity', 'validation failed', 'invalid entity'),
    429: ('Too Many Requests', 'rate limited', 'quota exceeded'),
    500: ('Internal Server Error', 'server error', 'internal failure'),
    502: ('Bad Gateway', 'gateway error', 'upstream failed'),
    503: ('Service Unavailable', 'service down', 'temporarily unavailable'),
}

# Common HTTP headers
HEADERS = {
    'request': [
        ('Content-Type', ['application/json', 'application/x-www-form-urlencoded', 'text/plain', 'multipart/form-data']),
        ('Accept', ['application/json', '*/*', 'text/html', 'application/xml']),
        ('Authorization', ['Bearer token123', 'Basic base64creds', 'API-Key key123']),
        ('Content-Length', ['0', '128', '256', '1024', '4096']),
        ('User-Agent', ['Mozilla/5.0', 'curl/7.68.0', 'PostmanRuntime/7.28.0']),
        ('Host', ['api.example.com', 'localhost:8080', 'service.internal']),
        ('Accept-Encoding', ['gzip, deflate', 'gzip', 'br']),
        ('Cookie', ['session=abc123', 'token=xyz789', 'user_id=42']),
        ('X-Request-ID', ['req-123', 'trace-abc-def', 'uuid-here']),
        ('If-None-Match', ['"etag123"', '"version-1"', 'W/"weak-etag"']),
        ('If-Match', ['"etag123"', '"version-1"']),
    ],
    'response': [
        ('Content-Type', ['application/json; charset=utf-8', 'text/html', 'text/plain']),
        ('Content-Length', ['0', '64', '256', '1024']),
        ('Cache-Control', ['no-cache', 'max-age=3600', 'public, max-age=86400']),
        ('ETag', ['"abc123"', '"v1"', 'W/"weak"']),
        ('X-Request-ID', ['req-123', 'trace-abc']),
        ('Set-Cookie', ['session=xyz; HttpOnly', 'token=abc; Secure']),
        ('Location', ['/users/123', '/api/items/new-id']),
        ('WWW-Authenticate', ['Bearer realm="api"', 'Basic realm="secure"']),
        ('Retry-After', ['30', '60', '120']),
        ('X-RateLimit-Remaining', ['99', '50', '0']),
    ],
}

# HTTP Methods
HTTP_METHODS = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS']


class HTTPPatternGenerator:
    """Generate HTTP protocol pattern samples."""

    # Opcodes
    OP_MOV = 0x1C
    OP_LOAD = 0x03
    OP_STORE = 0x04
    OP_BRANCH = 0x06
    OP_ALU = 0x00
    OP_ALUI = 0x01
    OP_CALL = 0x07
    OP_RET = 0x08
    OP_HALT = 0x1D
    OP_NET = 0x15
    OP_IO = 0x17

    # ALU modes
    ALU_ADD = 0
    ALU_SUB = 1
    ALU_AND = 2
    ALU_OR = 3
    ALU_XOR = 4

    # Branch modes
    BR_EQ = 0
    BR_NE = 1
    BR_LT = 2
    BR_GE = 3

    # Net modes
    NET_SEND = 5
    NET_RECV = 4

    def __init__(self):
        """Initialize generator with pattern definitions."""
        self.patterns = [
            # Header parsing patterns (high weight)
            (15, self.gen_parse_content_type),
            (12, self.gen_parse_authorization),
            (10, self.gen_parse_content_length),
            (8, self.gen_parse_accept_header),
            (8, self.gen_parse_cookie_header),
            (6, self.gen_parse_if_match),

            # Status code patterns
            (12, self.gen_send_success_response),
            (10, self.gen_send_error_response),
            (8, self.gen_check_status_code),

            # Request handling patterns
            (12, self.gen_parse_http_method),
            (10, self.gen_parse_request_path),
            (8, self.gen_parse_query_string),
            (8, self.gen_parse_request_body),

            # Response building patterns
            (10, self.gen_build_json_response),
            (8, self.gen_build_response_headers),
            (6, self.gen_send_redirect),

            # Method routing patterns
            (10, self.gen_method_router),
            (6, self.gen_cors_preflight),
        ]

        total = sum(p[0] for p in self.patterns)
        self.weights = [p[0] / total for p in self.patterns]

    def generate(self) -> Tuple[str, List[Instruction]]:
        """Generate a single training sample."""
        idx = random.choices(range(len(self.patterns)), weights=self.weights)[0]
        return self.patterns[idx][1]()

    def generate_samples(self, count: int) -> List[Dict]:
        """Generate multiple training samples."""
        samples = []
        for _ in range(count):
            prompt, instructions = self.generate()
            samples.append({
                'context': prompt,
                'instructions': self._pad_instructions(instructions),
                'metadata': {
                    'category': 'http-protocol',
                    'source': 'synthetic-http',
                }
            })
        return samples

    def _pad_instructions(self, instructions: List[Instruction], slots: int = 64) -> List[Dict]:
        """Pad instruction list to fixed slots."""
        result = [instr.to_dict() for instr in instructions]
        while len(result) < slots:
            result.append({
                'valid': 0, 'opcode': 0, 'mode': 0, 'rd': 0,
                'rs1': 0, 'rs2': 0, 'has_imm': 0, 'imm_bin': 0
            })
        return result[:slots]

    def _mov(self, rd: int, imm: int) -> Instruction:
        return Instruction(self.OP_MOV, 0, rd, 0, 0, 1, imm)

    def _mov_reg(self, rd: int, rs: int) -> Instruction:
        return Instruction(self.OP_MOV, 0, rd, rs, 0, 0, 0)

    def _load_byte(self, rd: int, rs: int, offset: int = 0) -> Instruction:
        return Instruction(self.OP_LOAD, 0, rd, rs, 0, 1, offset)

    def _load_word(self, rd: int, rs: int, offset: int = 0) -> Instruction:
        return Instruction(self.OP_LOAD, 3, rd, rs, 0, 1, offset)

    def _store_byte(self, rs: int, rd: int, offset: int = 0) -> Instruction:
        return Instruction(self.OP_STORE, 0, rd, rs, 0, 1, offset)

    def _store_word(self, rs: int, rd: int, offset: int = 0) -> Instruction:
        return Instruction(self.OP_STORE, 3, rd, rs, 0, 1, offset)

    def _branch(self, mode: int, rs1: int, rs2: int, offset: int) -> Instruction:
        return Instruction(self.OP_BRANCH, mode, 0, rs1, rs2, 1, offset)

    def _alu(self, mode: int, rd: int, rs1: int, rs2: int) -> Instruction:
        return Instruction(self.OP_ALU, mode, rd, rs1, rs2, 0, 0)

    def _alui(self, mode: int, rd: int, rs: int, imm: int) -> Instruction:
        return Instruction(self.OP_ALUI, mode, rd, rs, 0, 1, imm)

    def _call(self, offset: int) -> Instruction:
        return Instruction(self.OP_CALL, 0, 0, 0, 0, 1, offset)

    def _ret(self) -> Instruction:
        return Instruction(self.OP_RET, 0, 0, 0, 0, 0, 0)

    def _halt(self) -> Instruction:
        return Instruction(self.OP_HALT, 0, 0, 0, 0, 0, 0)

    def _net_send(self, rd: int, fd: int, buf: int, flags: int = 0) -> Instruction:
        return Instruction(self.OP_NET, self.NET_SEND, rd, fd, buf, 1, flags)

    def _net_recv(self, rd: int, fd: int, buf: int, flags: int = 0) -> Instruction:
        return Instruction(self.OP_NET, self.NET_RECV, rd, fd, buf, 1, flags)

    # ========================================================================
    # HEADER PARSING PATTERNS
    # ========================================================================

    def gen_parse_content_type(self) -> Tuple[str, List[Instruction]]:
        """Generate Content-Type header parsing pattern."""
        content_types = [
            ('application/json', 'json'),
            ('application/x-www-form-urlencoded', 'form'),
            ('text/plain', 'text'),
            ('multipart/form-data', 'multipart'),
        ]
        ct, short = random.choice(content_types)

        prompts = [
            "parse content-type header from request",
            f"check if content-type is {ct}",
            f"detect {short} content type",
            "extract content-type from http headers",
            "validate request content type",
        ]

        # Pattern: search for "Content-Type:" then compare value
        instructions = [
            self._mov(0, 0x1000),           # r0 = header buffer ptr
            self._mov(1, 0x2000),           # r1 = "Content-Type:" pattern
            self._call(10),                  # call find_header
            self._branch(self.BR_EQ, 0, 0, 7),  # if not found, return 0
            self._mov(1, 0x3000),           # r1 = expected content type
            self._call(15),                  # call strcmp
            self._mov_reg(5, 0),            # r5 = result
            self._ret(),
            # find_header: scan for header name
            self._load_byte(2, 0, 0),       # load char
            self._branch(self.BR_EQ, 2, 0, 3),  # if null, not found
            self._alui(self.ALU_ADD, 0, 0, 1),  # advance ptr
            self._branch(0, 0, 0, -3),      # loop
            self._mov(0, 0),                # not found
            self._ret(),
            # strcmp:
            self._load_byte(2, 0, 0),
            self._load_byte(3, 1, 0),
            self._branch(self.BR_NE, 2, 3, 4),  # if different, return 0
            self._branch(self.BR_EQ, 2, 0, 3),  # if both null, return 1
            self._alui(self.ALU_ADD, 0, 0, 1),
            self._alui(self.ALU_ADD, 1, 1, 1),
            self._branch(0, 0, 0, -6),
            self._mov(0, 1),
            self._ret(),
            self._mov(0, 0),
            self._ret(),
        ]

        return random.choice(prompts), instructions

    def gen_parse_authorization(self) -> Tuple[str, List[Instruction]]:
        """Generate Authorization header parsing pattern."""
        auth_types = ['Bearer', 'Basic', 'API-Key']
        auth_type = random.choice(auth_types)

        prompts = [
            "parse authorization header from request",
            f"extract {auth_type} token from authorization",
            "get bearer token from authorization header",
            "validate authorization header format",
            "extract api key from authorization",
        ]

        instructions = [
            self._mov(0, 0x1000),           # r0 = header buffer
            self._mov(1, 0x2000),           # r1 = "Authorization:" pattern
            self._call(8),                   # find_header
            self._branch(self.BR_EQ, 0, 0, 5),  # if not found, error
            # Skip "Bearer " prefix (7 chars)
            self._alui(self.ALU_ADD, 0, 0, 7),
            # Copy token to output
            self._mov(1, 0x3000),           # r1 = output buffer
            self._call(12),                  # copy_until_space
            self._ret(),
            # error:
            self._mov(0, 0),
            self._ret(),
            # find_header stub
            self._mov(0, 0x1100),           # mock found position
            self._ret(),
            # copy_until_space
            self._load_byte(2, 0, 0),
            self._mov(3, 0x20),             # space
            self._branch(self.BR_EQ, 2, 3, 4),
            self._branch(self.BR_EQ, 2, 0, 3),  # null terminator
            self._store_byte(2, 1, 0),
            self._alui(self.ALU_ADD, 0, 0, 1),
            self._alui(self.ALU_ADD, 1, 1, 1),
            self._branch(0, 0, 0, -6),
            self._store_byte(0, 1, 0),      # null terminate
            self._ret(),
        ]

        return random.choice(prompts), instructions

    def gen_parse_content_length(self) -> Tuple[str, List[Instruction]]:
        """Generate Content-Length header parsing pattern."""
        prompts = [
            "parse content-length header",
            "extract content length value",
            "get request body size from content-length",
            "read content-length as integer",
            "parse http content length header",
        ]

        instructions = [
            self._mov(0, 0x1000),           # r0 = headers buffer
            self._mov(1, 0x2000),           # r1 = "Content-Length:" pattern
            self._call(8),                   # find_header
            self._branch(self.BR_EQ, 0, 0, 5),  # if not found, return 0
            # Parse digits to integer
            self._call(12),                  # atoi
            self._ret(),
            # error:
            self._mov(0, 0),
            self._ret(),
            # find_header (stub)
            self._mov(0, 0x1100),
            self._ret(),
            # atoi: parse decimal string to int
            self._mov(1, 0),                # result = 0
            self._load_byte(2, 0, 0),       # load char
            self._mov(3, 0x30),             # '0'
            self._alu(self.ALU_SUB, 4, 2, 3),  # digit = char - '0'
            self._mov(5, 10),
            self._branch(self.BR_GE, 4, 5, 4),  # if >= 10, done
            self._branch(self.BR_LT, 4, 0, 3),  # if < 0, done (using zero reg)
            # result = result * 10 + digit
            self._mov(5, 10),
            # (multiply by 10 using shifts and adds: x*10 = x*8 + x*2)
            self._alui(self.ALU_ADD, 1, 1, 0),  # simplified: just accumulate
            self._alui(self.ALU_ADD, 0, 0, 1),
            self._branch(0, 0, 0, -9),
            self._mov_reg(0, 1),
            self._ret(),
        ]

        return random.choice(prompts), instructions

    def gen_parse_accept_header(self) -> Tuple[str, List[Instruction]]:
        """Generate Accept header parsing pattern."""
        prompts = [
            "parse accept header from request",
            "check if client accepts json",
            "extract accepted content types",
            "validate accept header",
            "determine response content type from accept",
        ]

        instructions = [
            self._mov(0, 0x1000),           # r0 = headers
            self._mov(1, 0x2000),           # r1 = "Accept:" pattern
            self._call(6),                   # find_header
            self._branch(self.BR_EQ, 0, 0, 3),  # if not found, use default
            self._mov(1, 0x3000),           # r1 = "application/json"
            self._call(10),                  # str_contains
            self._ret(),
            # default:
            self._mov(0, 1),                # assume json accepted
            self._ret(),
            # find_header stub
            self._mov(0, 0x1100),
            self._ret(),
            # str_contains stub
            self._mov(0, 1),
            self._ret(),
        ]

        return random.choice(prompts), instructions

    def gen_parse_cookie_header(self) -> Tuple[str, List[Instruction]]:
        """Generate Cookie header parsing pattern."""
        cookie_names = ['session', 'token', 'user_id', 'auth']
        cookie = random.choice(cookie_names)

        prompts = [
            f"extract {cookie} cookie from request",
            "parse cookie header values",
            f"get {cookie} value from cookies",
            "read session cookie from request",
            "parse http cookie header",
        ]

        instructions = [
            self._mov(0, 0x1000),           # r0 = headers
            self._mov(1, 0x2000),           # r1 = "Cookie:" pattern
            self._call(8),                   # find_header
            self._branch(self.BR_EQ, 0, 0, 5),
            # Find specific cookie name
            self._mov(1, 0x3000),           # r1 = cookie name
            self._call(12),                  # find_cookie_value
            self._ret(),
            # not found:
            self._mov(0, 0),
            self._ret(),
            # find_header stub
            self._mov(0, 0x1100),
            self._ret(),
            # find_cookie_value stub
            self._mov(0, 0x4000),           # return value ptr
            self._ret(),
        ]

        return random.choice(prompts), instructions

    def gen_parse_if_match(self) -> Tuple[str, List[Instruction]]:
        """Generate If-Match/If-None-Match header parsing pattern."""
        if random.random() > 0.5:
            header = 'If-Match'
            prompts = [
                "parse if-match header for conditional update",
                "extract etag from if-match header",
                "check if-match precondition",
                "validate etag for update request",
            ]
        else:
            header = 'If-None-Match'
            prompts = [
                "parse if-none-match header for caching",
                "extract etag from if-none-match",
                "check cache validation header",
                "validate if-none-match condition",
            ]

        instructions = [
            self._mov(0, 0x1000),           # r0 = headers
            self._mov(1, 0x2000),           # r1 = header pattern
            self._call(6),                   # find_header
            self._branch(self.BR_EQ, 0, 0, 3),
            self._call(10),                  # parse_etag
            self._ret(),
            # not found:
            self._mov(0, 0),
            self._ret(),
            # find_header stub
            self._mov(0, 0x1100),
            self._ret(),
            # parse_etag stub
            self._mov(0, 0x3000),
            self._ret(),
        ]

        return random.choice(prompts), instructions

    # ========================================================================
    # STATUS CODE PATTERNS
    # ========================================================================

    def gen_send_success_response(self) -> Tuple[str, List[Instruction]]:
        """Generate success response sending pattern."""
        success_codes = [(200, 'OK'), (201, 'Created'), (204, 'No Content')]
        code, text = random.choice(success_codes)

        prompts = [
            f"send http {code} {text} response",
            f"return {code} success response",
            f"build and send {code} response",
            f"respond with http {code}",
            f"send {text.lower()} status code",
        ]

        instructions = [
            self._mov(0, 0x1000),           # r0 = response buffer
            # Build "HTTP/1.1 {code} {text}\r\n"
            self._mov(1, 0x2000),           # r1 = status line template
            self._call(10),                  # strcpy
            # Add headers
            self._mov(1, 0x3000),           # r1 = "Content-Type: application/json\r\n"
            self._call(10),                  # strcat
            self._mov(1, 0x4000),           # r1 = "Content-Length: "
            self._call(10),
            # Add body length
            self._mov(1, 0),                # content length
            self._call(14),                  # itoa
            # CRLF CRLF
            self._mov(1, 0x5000),           # r1 = "\r\n\r\n"
            self._call(10),
            # Send
            self._mov(1, 11),               # r1 = socket fd (r11)
            self._mov_reg(2, 0),            # r2 = response buffer
            self._net_send(0, 1, 2),
            self._ret(),
            # strcpy stub
            self._ret(),
            # itoa stub
            self._ret(),
        ]

        return random.choice(prompts), instructions

    def gen_send_error_response(self) -> Tuple[str, List[Instruction]]:
        """Generate error response sending pattern."""
        error_codes = [
            (400, 'Bad Request'),
            (401, 'Unauthorized'),
            (403, 'Forbidden'),
            (404, 'Not Found'),
            (500, 'Internal Server Error'),
        ]
        code, text = random.choice(error_codes)

        prompts = [
            f"send http {code} {text} error response",
            f"return {code} error",
            f"respond with {code} status",
            f"build {text.lower()} error response",
            f"send {code} with error body",
        ]

        instructions = [
            self._mov(0, 0x1000),           # r0 = response buffer
            # Build status line
            self._mov(1, 0x2000),           # r1 = "HTTP/1.1 {code} {text}\r\n"
            self._call(12),                  # strcpy
            # Headers
            self._mov(1, 0x3000),           # r1 = headers
            self._call(12),                  # strcat
            # Error body JSON
            self._mov(1, 0x4000),           # r1 = '{"error":"..."}'
            self._call(12),
            # Send response
            self._mov(1, 11),               # socket fd
            self._mov_reg(2, 0),
            self._mov(3, 256),              # length estimate
            self._net_send(0, 1, 2),
            self._ret(),
            # strcpy stub
            self._ret(),
        ]

        return random.choice(prompts), instructions

    def gen_check_status_code(self) -> Tuple[str, List[Instruction]]:
        """Generate status code checking pattern."""
        prompts = [
            "check if http status indicates success",
            "validate response status code",
            "check for 2xx success status",
            "determine if request succeeded from status",
            "check http response status",
        ]

        instructions = [
            self._mov(1, 200),              # r1 = 200
            self._branch(self.BR_LT, 0, 1, 6),  # if < 200, error
            self._mov(1, 300),              # r1 = 300
            self._branch(self.BR_GE, 0, 1, 4),  # if >= 300, error
            # Success (2xx)
            self._mov(0, 1),
            self._ret(),
            # Error
            self._mov(0, 0),
            self._ret(),
        ]

        return random.choice(prompts), instructions

    # ========================================================================
    # REQUEST HANDLING PATTERNS
    # ========================================================================

    def gen_parse_http_method(self) -> Tuple[str, List[Instruction]]:
        """Generate HTTP method parsing pattern."""
        prompts = [
            "parse http method from request line",
            "extract request method GET POST PUT DELETE",
            "identify http method from request",
            "check http request method",
            "determine http method from first line",
        ]

        instructions = [
            self._mov(0, 0x1000),           # r0 = request buffer
            self._load_byte(1, 0, 0),       # first char
            # Check 'G' for GET
            self._mov(2, 0x47),             # 'G'
            self._branch(self.BR_EQ, 1, 2, 8),  # goto is_get
            # Check 'P' for POST/PUT/PATCH
            self._mov(2, 0x50),             # 'P'
            self._branch(self.BR_EQ, 1, 2, 8),  # goto is_p_method
            # Check 'D' for DELETE
            self._mov(2, 0x44),             # 'D'
            self._branch(self.BR_EQ, 1, 2, 8),  # goto is_delete
            # Check 'H' for HEAD
            self._mov(2, 0x48),             # 'H'
            self._branch(self.BR_EQ, 1, 2, 8),  # goto is_head
            # Unknown method
            self._mov(0, 0),
            self._ret(),
            # is_get:
            self._mov(0, 1),                # GET = 1
            self._ret(),
            # is_p_method: check second char for POST vs PUT vs PATCH
            self._load_byte(1, 0, 1),
            self._mov(2, 0x4F),             # 'O' for POST
            self._branch(self.BR_EQ, 1, 2, 3),
            self._mov(0, 3),                # PUT = 3
            self._ret(),
            self._mov(0, 2),                # POST = 2
            self._ret(),
            # is_delete:
            self._mov(0, 4),                # DELETE = 4
            self._ret(),
            # is_head:
            self._mov(0, 5),                # HEAD = 5
            self._ret(),
        ]

        return random.choice(prompts), instructions

    def gen_parse_request_path(self) -> Tuple[str, List[Instruction]]:
        """Generate request path parsing pattern."""
        prompts = [
            "extract url path from http request",
            "parse request path from request line",
            "get path component from http request",
            "extract uri path from request",
            "parse path between method and http version",
        ]

        instructions = [
            self._mov(0, 0x1000),           # r0 = request buffer
            self._mov(1, 0x2000),           # r1 = path output buffer
            # Skip method (find first space)
            self._load_byte(2, 0, 0),
            self._mov(3, 0x20),             # space
            self._branch(self.BR_EQ, 2, 3, 2),
            self._alui(self.ALU_ADD, 0, 0, 1),
            self._branch(0, 0, 0, -4),
            # Skip the space
            self._alui(self.ALU_ADD, 0, 0, 1),
            # Copy path until next space or ?
            self._load_byte(2, 0, 0),
            self._mov(3, 0x20),             # space
            self._branch(self.BR_EQ, 2, 3, 6),
            self._mov(3, 0x3F),             # '?'
            self._branch(self.BR_EQ, 2, 3, 4),
            self._store_byte(2, 1, 0),
            self._alui(self.ALU_ADD, 0, 0, 1),
            self._alui(self.ALU_ADD, 1, 1, 1),
            self._branch(0, 0, 0, -8),
            # Null terminate
            self._store_byte(0, 1, 0),
            self._ret(),
        ]

        return random.choice(prompts), instructions

    def gen_parse_query_string(self) -> Tuple[str, List[Instruction]]:
        """Generate query string parsing pattern."""
        params = ['id', 'page', 'limit', 'offset', 'search', 'filter']
        param = random.choice(params)

        prompts = [
            f"parse query string parameter {param}",
            "extract query parameters from url",
            f"get {param} value from query string",
            "parse url query string",
            "decode query parameters from request",
        ]

        instructions = [
            self._mov(0, 0x1000),           # r0 = url/path
            self._mov(1, 0x2000),           # r1 = param name to find
            # Find '?' in url
            self._load_byte(2, 0, 0),
            self._mov(3, 0x3F),             # '?'
            self._branch(self.BR_EQ, 2, 3, 3),
            self._branch(self.BR_EQ, 2, 0, 5),  # end of string
            self._alui(self.ALU_ADD, 0, 0, 1),
            self._branch(0, 0, 0, -5),
            # Found '?', skip it
            self._alui(self.ALU_ADD, 0, 0, 1),
            # Search for param name (stub)
            self._call(8),                   # find_param
            self._ret(),
            # not found:
            self._mov(0, 0),
            self._ret(),
            # find_param stub:
            self._mov(0, 0x3000),           # return value ptr
            self._ret(),
        ]

        return random.choice(prompts), instructions

    def gen_parse_request_body(self) -> Tuple[str, List[Instruction]]:
        """Generate request body parsing pattern."""
        prompts = [
            "extract body from http request",
            "find request body after headers",
            "parse http request body",
            "get post body from request",
            "extract payload from http request",
        ]

        instructions = [
            self._mov(0, 0x1000),           # r0 = request buffer
            # Find "\r\n\r\n" (end of headers)
            self._load_byte(1, 0, 0),
            self._mov(2, 0x0D),             # '\r'
            self._branch(self.BR_NE, 1, 2, 3),
            self._load_byte(1, 0, 1),
            self._mov(2, 0x0A),             # '\n'
            self._branch(self.BR_NE, 1, 2, 6),
            self._load_byte(1, 0, 2),
            self._mov(2, 0x0D),
            self._branch(self.BR_NE, 1, 2, 4),
            self._load_byte(1, 0, 3),
            self._mov(2, 0x0A),
            self._branch(self.BR_EQ, 1, 2, 3),  # found!
            self._alui(self.ALU_ADD, 0, 0, 1),
            self._branch(0, 0, 0, -13),
            # Found body start
            self._alui(self.ALU_ADD, 0, 0, 4),  # skip \r\n\r\n
            self._ret(),
        ]

        return random.choice(prompts), instructions

    # ========================================================================
    # RESPONSE BUILDING PATTERNS
    # ========================================================================

    def gen_build_json_response(self) -> Tuple[str, List[Instruction]]:
        """Generate JSON response building pattern."""
        prompts = [
            "build json http response",
            "create json response with headers",
            "format json response body",
            "construct http json response",
            "build rest api json response",
        ]

        instructions = [
            self._mov(0, 0x1000),           # r0 = response buffer
            self._mov(1, 0x2000),           # r1 = "HTTP/1.1 200 OK\r\n"
            self._call(16),                  # strcpy
            self._mov_reg(5, 0),            # save ptr
            self._mov(1, 0x3000),           # r1 = "Content-Type: application/json\r\n"
            self._call(16),
            self._mov(1, 0x4000),           # r1 = "Content-Length: "
            self._call(16),
            # Calculate and add body length
            self._mov(0, 0x5000),           # r0 = json body
            self._call(20),                  # strlen
            self._mov_reg(6, 0),            # save length
            self._mov_reg(0, 5),            # restore ptr
            self._mov_reg(1, 6),            # length
            self._call(24),                  # itoa
            self._mov(1, 0x6000),           # r1 = "\r\n\r\n"
            self._call(16),
            self._mov(1, 0x5000),           # r1 = json body
            self._call(16),
            self._ret(),
            # strcpy stub
            self._ret(),
            # strlen stub
            self._mov(0, 64),
            self._ret(),
            # itoa stub
            self._ret(),
        ]

        return random.choice(prompts), instructions

    def gen_build_response_headers(self) -> Tuple[str, List[Instruction]]:
        """Generate response header building pattern."""
        headers = [
            ('Cache-Control', 'no-cache'),
            ('ETag', '"abc123"'),
            ('X-Request-ID', 'req-123'),
        ]
        header, value = random.choice(headers)

        prompts = [
            f"add {header} header to response",
            "build http response headers",
            f"set {header} response header",
            "construct response header section",
            "append header to http response",
        ]

        instructions = [
            self._mov(0, 0x1000),           # r0 = response buffer (append position)
            self._mov(1, 0x2000),           # r1 = header name
            self._call(8),                   # strcpy
            self._mov(1, 0x3000),           # r1 = ": "
            self._call(8),
            self._mov(1, 0x4000),           # r1 = header value
            self._call(8),
            self._mov(1, 0x5000),           # r1 = "\r\n"
            self._call(8),
            self._ret(),
            # strcpy stub
            self._ret(),
        ]

        return random.choice(prompts), instructions

    def gen_send_redirect(self) -> Tuple[str, List[Instruction]]:
        """Generate redirect response pattern."""
        codes = [(301, 'Moved Permanently'), (302, 'Found'), (307, 'Temporary Redirect')]
        code, text = random.choice(codes)

        prompts = [
            f"send http {code} redirect response",
            f"redirect client with {code} status",
            "send redirect with location header",
            f"return {text.lower()} redirect",
            "build redirect response with location",
        ]

        instructions = [
            self._mov(0, 0x1000),           # r0 = response buffer
            self._mov(1, 0x2000),           # r1 = "HTTP/1.1 {code} {text}\r\n"
            self._call(10),                  # strcpy
            self._mov(1, 0x3000),           # r1 = "Location: "
            self._call(10),
            self._mov(1, 0x4000),           # r1 = redirect url
            self._call(10),
            self._mov(1, 0x5000),           # r1 = "\r\n\r\n"
            self._call(10),
            # Send response
            self._mov(1, 11),               # socket fd
            self._mov(2, 0x1000),
            self._mov(3, 128),
            self._net_send(0, 1, 2),
            self._ret(),
            # strcpy stub
            self._ret(),
        ]

        return random.choice(prompts), instructions

    # ========================================================================
    # METHOD ROUTING PATTERNS
    # ========================================================================

    def gen_method_router(self) -> Tuple[str, List[Instruction]]:
        """Generate HTTP method routing pattern."""
        prompts = [
            "route request by http method",
            "dispatch handler based on method",
            "create method router for http requests",
            "switch on http method type",
            "route to handler by request method",
        ]

        instructions = [
            self._mov(0, 0x1000),           # r0 = request buffer
            self._load_byte(1, 0, 0),       # first char of method
            # GET
            self._mov(2, 0x47),             # 'G'
            self._branch(self.BR_EQ, 1, 2, 10),  # handle_get
            # POST
            self._mov(2, 0x50),             # 'P'
            self._branch(self.BR_EQ, 1, 2, 10),  # handle_post (needs more check)
            # DELETE
            self._mov(2, 0x44),             # 'D'
            self._branch(self.BR_EQ, 1, 2, 10),  # handle_delete
            # Method not allowed
            self._mov(0, 405),
            self._call(20),                  # send_error
            self._ret(),
            # handle_get:
            self._call(22),
            self._ret(),
            # handle_post:
            self._call(24),
            self._ret(),
            # handle_delete:
            self._call(26),
            self._ret(),
            # stubs
            self._ret(),
            self._ret(),
            self._ret(),
            self._ret(),
        ]

        return random.choice(prompts), instructions

    def gen_cors_preflight(self) -> Tuple[str, List[Instruction]]:
        """Generate CORS preflight response pattern."""
        prompts = [
            "handle cors preflight options request",
            "respond to cors preflight with headers",
            "build cors preflight response",
            "handle options request for cors",
            "send access-control-allow headers",
        ]

        instructions = [
            self._mov(0, 0x1000),           # r0 = response buffer
            self._mov(1, 0x2000),           # r1 = "HTTP/1.1 204 No Content\r\n"
            self._call(14),                  # strcpy
            self._mov(1, 0x3000),           # "Access-Control-Allow-Origin: *\r\n"
            self._call(14),
            self._mov(1, 0x4000),           # "Access-Control-Allow-Methods: GET, POST, PUT, DELETE\r\n"
            self._call(14),
            self._mov(1, 0x5000),           # "Access-Control-Allow-Headers: Content-Type, Authorization\r\n"
            self._call(14),
            self._mov(1, 0x6000),           # "\r\n"
            self._call(14),
            # Send response
            self._mov(1, 11),
            self._mov(2, 0x1000),
            self._mov(3, 256),
            self._net_send(0, 1, 2),
            self._ret(),
            # strcpy stub
            self._ret(),
        ]

        return random.choice(prompts), instructions


def main():
    """Test the generator."""
    import json

    gen = HTTPPatternGenerator()
    samples = gen.generate_samples(100)

    print(f"Generated {len(samples)} samples")
    print("\nSample prompts:")
    for s in samples[:10]:
        print(f"  - {s['context']}")


if __name__ == '__main__':
    main()
