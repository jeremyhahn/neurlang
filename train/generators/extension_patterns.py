#!/usr/bin/env python3
"""
Extension Pattern Generator
===========================

Generates training samples for extension composition patterns covering:
- Crypto patterns (hash, sign, encrypt)
- JSON patterns (parse, extract, build)
- HTTP client patterns (get, post, headers)
- Database patterns (query, transaction)
- TLS patterns (secure connect)
- Compression patterns (gzip)
- Encoding patterns (base64, url)

Extension ID Reference:
  1-99: Crypto (sha256=1, hmac=2, aes=3/4, ed25519=8/9, argon2id=35, sha1=38)
  100-119: Vec operations
  120-139: HashMap operations
  140-169: String operations
  170-189: JSON operations (parse=170, stringify=171, get=172-177, free=179, new=180, set=181)
  190-209: HTTP client (get=190, post=191, put=192, delete=193, headers=194-198)
  240-259: File system
  260-279: SQLite (open=245, prepare=247, bind=248, step=249, begin=250, commit=251, rollback=252)
  280-299: Regex
  300-319: DateTime
  330-339: UUID (v4=330)
  400-407: Compression (gzip_compress=402, gzip_decompress=403)
  420-429: Encoding (base64_encode=420, base64_decode=421, url_encode=424, url_decode=425)
  500-509: TLS (connect=500, write=501, read=502, close=503)
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


# Extension IDs
EXT_SHA256 = 1
EXT_HMAC_SHA256 = 2
EXT_AES_ENCRYPT = 3
EXT_AES_DECRYPT = 4
EXT_CONST_TIME_CMP = 5
EXT_ED25519_SIGN = 8
EXT_ED25519_VERIFY = 9
EXT_SHA384 = 15
EXT_SHA512 = 16
EXT_SHA3_256 = 17
EXT_SHA3_512 = 18
EXT_HMAC_SHA384 = 21
EXT_HMAC_SHA512 = 22
EXT_ARGON2ID = 35
EXT_SHA1 = 38

EXT_JSON_PARSE = 170
EXT_JSON_STRINGIFY = 171
EXT_JSON_GET_STRING = 172
EXT_JSON_GET_INT = 173
EXT_JSON_GET_BOOL = 174
EXT_JSON_ARRAY_LEN = 175
EXT_JSON_ARRAY_GET = 176
EXT_JSON_HAS_KEY = 177
EXT_JSON_FREE = 179
EXT_JSON_NEW = 180
EXT_JSON_SET = 181

EXT_HTTP_GET = 190
EXT_HTTP_POST = 191
EXT_HTTP_PUT = 192
EXT_HTTP_DELETE = 193
EXT_HTTP_STATUS = 194
EXT_HTTP_HEADER = 195
EXT_HTTP_BODY = 196
EXT_HTTP_FREE = 197
EXT_HTTP_SET_HEADER = 198

EXT_SQLITE_OPEN = 245
EXT_SQLITE_PREPARE = 247
EXT_SQLITE_BIND = 248
EXT_SQLITE_STEP = 249
EXT_SQLITE_BEGIN = 250
EXT_SQLITE_COMMIT = 251
EXT_SQLITE_ROLLBACK = 252

EXT_GZIP_COMPRESS = 402
EXT_GZIP_DECOMPRESS = 403

EXT_BASE64_ENCODE = 420
EXT_BASE64_DECODE = 421
EXT_URL_ENCODE = 424
EXT_URL_DECODE = 425

EXT_TLS_CONNECT = 500
EXT_TLS_WRITE = 501
EXT_TLS_READ = 502
EXT_TLS_CLOSE = 503

EXT_UUID_V4 = 330


class ExtensionPatternGenerator:
    """Generate extension composition pattern samples."""

    # Opcodes
    OP_MOV = 0x1C
    OP_EXT_CALL = 0x20
    OP_HALT = 0x1D
    OP_LOAD = 0x03
    OP_STORE = 0x04
    OP_BRANCH = 0x06
    OP_ALU = 0x00
    OP_ALUI = 0x01
    OP_CALL = 0x07
    OP_RET = 0x08

    # Branch modes
    BR_EQ = 0
    BR_NE = 1
    BR_LT = 2
    BR_GE = 3

    def __init__(self):
        """Initialize generator with pattern definitions."""
        # Pattern definitions: (weight, generator_function)
        self.patterns = [
            # Crypto patterns (high weight - common operations)
            (15, self.gen_hash_pattern),
            (10, self.gen_hmac_pattern),
            (8, self.gen_sign_verify_pattern),
            (8, self.gen_encrypt_decrypt_pattern),
            (5, self.gen_password_hash_pattern),

            # JSON patterns (high weight - very common)
            (15, self.gen_json_parse_extract_pattern),
            (12, self.gen_json_build_pattern),
            (8, self.gen_json_array_iterate_pattern),

            # HTTP client patterns
            (12, self.gen_http_get_pattern),
            (10, self.gen_http_post_pattern),
            (6, self.gen_http_with_headers_pattern),

            # Database patterns
            (8, self.gen_sqlite_query_pattern),
            (6, self.gen_sqlite_transaction_pattern),

            # Encoding patterns
            (8, self.gen_base64_roundtrip_pattern),
            (6, self.gen_url_encode_pattern),

            # Compression patterns
            (5, self.gen_compression_pattern),

            # TLS patterns
            (6, self.gen_tls_secure_pattern),

            # UUID patterns
            (4, self.gen_uuid_pattern),
        ]

        # Compute normalized weights
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
                    'category': 'extension-patterns',
                    'source': 'synthetic-extension',
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
        """mov rd, imm"""
        return Instruction(self.OP_MOV, 0, rd, 0, 0, 1, imm)

    def _mov_reg(self, rd: int, rs: int) -> Instruction:
        """mov rd, rs"""
        return Instruction(self.OP_MOV, 0, rd, rs, 0, 0, 0)

    def _ext_call(self, ext_id: int, r0: int = 0, r1: int = 1, r2: int = 2) -> Instruction:
        """ext.call ext_id, r0, r1, r2"""
        return Instruction(self.OP_EXT_CALL, 0, r0, r1, r2, 1, ext_id)

    def _halt(self) -> Instruction:
        """halt"""
        return Instruction(self.OP_HALT, 0, 0, 0, 0, 0, 0)

    def _load(self, rd: int, rs: int, offset: int = 0) -> Instruction:
        """load rd, [rs + offset]"""
        return Instruction(self.OP_LOAD, 3, rd, rs, 0, 1, offset)

    def _store(self, rs: int, rd: int, offset: int = 0) -> Instruction:
        """store rs, [rd + offset]"""
        return Instruction(self.OP_STORE, 3, rd, rs, 0, 1, offset)

    def _branch(self, mode: int, rs1: int, rs2: int, offset: int) -> Instruction:
        """branch.mode rs1, rs2, offset"""
        return Instruction(self.OP_BRANCH, mode, 0, rs1, rs2, 1, offset)

    def _alu_add(self, rd: int, rs1: int, rs2: int) -> Instruction:
        """add rd, rs1, rs2"""
        return Instruction(self.OP_ALU, 0, rd, rs1, rs2, 0, 0)

    def _alui_add(self, rd: int, rs: int, imm: int) -> Instruction:
        """addi rd, rs, imm"""
        return Instruction(self.OP_ALUI, 0, rd, rs, 0, 1, imm)

    def _call(self, offset: int) -> Instruction:
        """call offset"""
        return Instruction(self.OP_CALL, 0, 0, 0, 0, 1, offset)

    def _ret(self) -> Instruction:
        """ret"""
        return Instruction(self.OP_RET, 0, 0, 0, 0, 0, 0)

    # ========================================================================
    # CRYPTO PATTERNS
    # ========================================================================

    def gen_hash_pattern(self) -> Tuple[str, List[Instruction]]:
        """Generate hash computation pattern."""
        hash_types = [
            (EXT_SHA256, 'sha256', 'SHA-256', 32),
            (EXT_SHA384, 'sha384', 'SHA-384', 48),
            (EXT_SHA512, 'sha512', 'SHA-512', 64),
            (EXT_SHA3_256, 'sha3-256', 'SHA3-256', 32),
            (EXT_SHA1, 'sha1', 'SHA-1', 20),
        ]
        ext_id, name, full_name, output_len = random.choice(hash_types)

        prompts = [
            f"compute {name} hash of input buffer",
            f"hash data using {full_name}",
            f"calculate {name} digest of message",
            f"create {name} hash of input",
            f"generate {full_name} checksum",
            f"apply {name} to input data",
        ]

        instructions = [
            self._mov(0, 0x1000),          # r0 = input ptr
            self._mov(1, 256),             # r1 = input length
            self._mov(2, 0x2000),          # r2 = output buffer
            self._ext_call(ext_id, 0, 1, 2),  # ext.call hash
            self._halt(),
        ]

        return random.choice(prompts), instructions

    def gen_hmac_pattern(self) -> Tuple[str, List[Instruction]]:
        """Generate HMAC computation pattern."""
        hmac_types = [
            (EXT_HMAC_SHA256, 'hmac-sha256', 'HMAC-SHA256'),
            (EXT_HMAC_SHA384, 'hmac-sha384', 'HMAC-SHA384'),
            (EXT_HMAC_SHA512, 'hmac-sha512', 'HMAC-SHA512'),
        ]
        ext_id, name, full_name = random.choice(hmac_types)

        prompts = [
            f"compute {name} of message with key",
            f"generate {full_name} authentication code",
            f"create {name} signature for data",
            f"calculate {name} MAC",
            f"authenticate message using {name}",
        ]

        instructions = [
            self._mov(0, 0x1000),          # r0 = message ptr
            self._mov(1, 128),             # r1 = message length
            self._mov(2, 0x2000),          # r2 = key ptr
            self._mov(3, 32),              # r3 = key length
            self._mov(4, 0x3000),          # r4 = output buffer
            self._ext_call(ext_id, 0, 1, 2),  # ext.call hmac (key info in r3, r4)
            self._halt(),
        ]

        return random.choice(prompts), instructions

    def gen_sign_verify_pattern(self) -> Tuple[str, List[Instruction]]:
        """Generate sign/verify pattern (Ed25519)."""
        if random.random() > 0.5:
            # Sign
            prompts = [
                "sign message with ed25519 private key",
                "create ed25519 signature for data",
                "generate digital signature using ed25519",
                "sign data with private key",
            ]
            instructions = [
                self._mov(0, 0x1000),          # r0 = message ptr
                self._mov(1, 64),              # r1 = message length
                self._mov(2, 0x2000),          # r2 = private key ptr
                self._mov(3, 0x3000),          # r3 = signature output
                self._ext_call(EXT_ED25519_SIGN, 0, 1, 2),
                self._halt(),
            ]
        else:
            # Verify
            prompts = [
                "verify ed25519 signature on message",
                "check digital signature validity",
                "validate ed25519 signed data",
                "verify signature with public key",
            ]
            instructions = [
                self._mov(0, 0x1000),          # r0 = message ptr
                self._mov(1, 64),              # r1 = message length
                self._mov(2, 0x2000),          # r2 = signature ptr
                self._mov(3, 0x3000),          # r3 = public key ptr
                self._ext_call(EXT_ED25519_VERIFY, 0, 1, 2),
                # r0 = 1 if valid, 0 if invalid
                self._halt(),
            ]

        return random.choice(prompts), instructions

    def gen_encrypt_decrypt_pattern(self) -> Tuple[str, List[Instruction]]:
        """Generate encrypt/decrypt pattern (AES-GCM)."""
        if random.random() > 0.5:
            # Encrypt
            prompts = [
                "encrypt data with aes-256-gcm",
                "apply aes gcm encryption to plaintext",
                "encrypt message using aes-256",
                "secure data with aes-gcm encryption",
            ]
            ext_id = EXT_AES_ENCRYPT
        else:
            # Decrypt
            prompts = [
                "decrypt aes-256-gcm ciphertext",
                "apply aes gcm decryption",
                "decrypt message using aes-256",
                "recover plaintext from aes-gcm",
            ]
            ext_id = EXT_AES_DECRYPT

        instructions = [
            self._mov(0, 0x1000),          # r0 = input ptr
            self._mov(1, 256),             # r1 = input length
            self._mov(2, 0x2000),          # r2 = key ptr (32 bytes)
            self._mov(3, 0x3000),          # r3 = nonce ptr (12 bytes)
            self._mov(4, 0x4000),          # r4 = output ptr
            self._ext_call(ext_id, 0, 1, 2),
            self._halt(),
        ]

        return random.choice(prompts), instructions

    def gen_password_hash_pattern(self) -> Tuple[str, List[Instruction]]:
        """Generate password hashing pattern (Argon2id)."""
        prompts = [
            "hash password with argon2id",
            "securely hash user password",
            "create password hash using argon2",
            "generate password-based key derivation",
            "hash password for storage",
        ]

        instructions = [
            self._mov(0, 0x1000),          # r0 = password ptr
            self._mov(1, 16),              # r1 = password length
            self._mov(2, 0x2000),          # r2 = salt ptr (16 bytes)
            self._mov(3, 0x3000),          # r3 = output ptr
            self._ext_call(EXT_ARGON2ID, 0, 1, 2),
            self._halt(),
        ]

        return random.choice(prompts), instructions

    # ========================================================================
    # JSON PATTERNS
    # ========================================================================

    def gen_json_parse_extract_pattern(self) -> Tuple[str, List[Instruction]]:
        """Generate JSON parse + extract pattern."""
        field_types = [
            (EXT_JSON_GET_STRING, 'string', 'name', 'username', 'email'),
            (EXT_JSON_GET_INT, 'integer', 'id', 'count', 'age'),
            (EXT_JSON_GET_BOOL, 'boolean', 'active', 'enabled', 'valid'),
        ]
        ext_get, type_name, *fields = random.choice(field_types)
        field = random.choice(fields)

        prompts = [
            f"parse json and extract {field} field",
            f"get {type_name} value from json key {field}",
            f"parse json object and read {field}",
            f"extract {field} from json string",
            f"decode json and get {field} value",
        ]

        instructions = [
            self._mov(0, 0x1000),          # r0 = json string ptr
            self._mov(1, 256),             # r1 = json length
            self._ext_call(EXT_JSON_PARSE, 0, 1, 0),  # parse -> r0 = handle
            self._mov_reg(5, 0),           # r5 = json handle
            self._mov(1, 0x2000),          # r1 = field name ptr
            self._mov(2, 0x3000),          # r2 = output buffer
            self._ext_call(ext_get, 5, 1, 2),  # get field
            self._mov_reg(6, 0),           # r6 = result
            self._mov_reg(0, 5),           # r0 = handle
            self._ext_call(EXT_JSON_FREE, 0, 0, 0),  # free json
            self._mov_reg(0, 6),           # return result
            self._halt(),
        ]

        return random.choice(prompts), instructions

    def gen_json_build_pattern(self) -> Tuple[str, List[Instruction]]:
        """Generate JSON object building pattern."""
        prompts = [
            "build json object with fields",
            "create json object and set properties",
            "construct json response object",
            "build json with id and name fields",
            "create json object and stringify",
        ]

        instructions = [
            self._ext_call(EXT_JSON_NEW, 0, 0, 0),  # new json object -> r0
            self._mov_reg(5, 0),           # r5 = json handle
            self._mov(1, 0x1000),          # r1 = "id" key
            self._mov(2, 123),             # r2 = id value
            self._ext_call(EXT_JSON_SET, 5, 1, 2),  # set "id": 123
            self._mov(1, 0x1100),          # r1 = "name" key
            self._mov(2, 0x2000),          # r2 = name value ptr
            self._ext_call(EXT_JSON_SET, 5, 1, 2),  # set "name": "..."
            self._mov_reg(0, 5),           # r0 = handle
            self._mov(1, 0x3000),          # r1 = output buffer
            self._mov(2, 4096),            # r2 = buffer size
            self._ext_call(EXT_JSON_STRINGIFY, 0, 1, 2),  # stringify
            self._mov_reg(0, 5),           # r0 = handle
            self._ext_call(EXT_JSON_FREE, 0, 0, 0),  # free
            self._halt(),
        ]

        return random.choice(prompts), instructions

    def gen_json_array_iterate_pattern(self) -> Tuple[str, List[Instruction]]:
        """Generate JSON array iteration pattern."""
        prompts = [
            "iterate over json array elements",
            "loop through json array",
            "process each element in json array",
            "parse json array and iterate items",
            "read all items from json array",
        ]

        instructions = [
            self._mov(0, 0x1000),          # r0 = json string ptr
            self._mov(1, 256),             # r1 = json length
            self._ext_call(EXT_JSON_PARSE, 0, 1, 0),  # parse
            self._mov_reg(5, 0),           # r5 = json handle
            self._mov(1, 0x2000),          # r1 = "items" key
            self._ext_call(EXT_JSON_ARRAY_LEN, 5, 1, 0),  # get length
            self._mov_reg(6, 0),           # r6 = array length
            self._mov(7, 0),               # r7 = index = 0
            # loop:
            self._branch(self.BR_GE, 7, 6, 6),  # if index >= len, exit
            self._mov(1, 0x2000),          # r1 = "items" key
            self._mov_reg(2, 7),           # r2 = index
            self._ext_call(EXT_JSON_ARRAY_GET, 5, 1, 2),  # get element
            self._alui_add(7, 7, 1),       # index++
            self._branch(self.BR_LT, 7, 6, -5),  # loop
            # end:
            self._mov_reg(0, 5),           # r0 = handle
            self._ext_call(EXT_JSON_FREE, 0, 0, 0),  # free
            self._halt(),
        ]

        return random.choice(prompts), instructions

    # ========================================================================
    # HTTP CLIENT PATTERNS
    # ========================================================================

    def gen_http_get_pattern(self) -> Tuple[str, List[Instruction]]:
        """Generate HTTP GET request pattern."""
        resources = ['users', 'items', 'products', 'orders', 'data', 'config']
        resource = random.choice(resources)

        prompts = [
            f"make http get request to /{resource}",
            f"fetch {resource} from api",
            f"send get request and parse response",
            f"http client get /{resource} endpoint",
            f"retrieve {resource} via http get",
        ]

        instructions = [
            self._mov(0, 0x1000),          # r0 = URL ptr
            self._ext_call(EXT_HTTP_GET, 0, 0, 0),  # http_get -> r0 = response handle
            self._mov_reg(5, 0),           # r5 = response handle
            self._ext_call(EXT_HTTP_STATUS, 5, 0, 0),  # get status -> r0
            self._mov_reg(6, 0),           # r6 = status code
            self._mov(1, 0x2000),          # r1 = body buffer
            self._mov(2, 4096),            # r2 = buffer size
            self._ext_call(EXT_HTTP_BODY, 5, 1, 2),  # get body -> r0 = length
            self._mov_reg(0, 5),           # r0 = handle
            self._ext_call(EXT_HTTP_FREE, 0, 0, 0),  # free response
            self._mov_reg(0, 6),           # return status
            self._halt(),
        ]

        return random.choice(prompts), instructions

    def gen_http_post_pattern(self) -> Tuple[str, List[Instruction]]:
        """Generate HTTP POST request pattern."""
        resources = ['users', 'items', 'orders', 'messages', 'events']
        resource = random.choice(resources)

        prompts = [
            f"make http post request to create {resource}",
            f"post json data to /{resource} endpoint",
            f"send post request with body",
            f"http client post to /{resource}",
            f"create {resource} via http post",
        ]

        instructions = [
            self._mov(0, 0x1000),          # r0 = URL ptr
            self._mov(1, 0x2000),          # r1 = body ptr
            self._mov(2, 256),             # r2 = body length
            self._ext_call(EXT_HTTP_POST, 0, 1, 2),  # http_post -> r0 = response handle
            self._mov_reg(5, 0),           # r5 = response handle
            self._ext_call(EXT_HTTP_STATUS, 5, 0, 0),  # get status
            self._mov_reg(6, 0),           # r6 = status
            self._mov_reg(0, 5),           # r0 = handle
            self._ext_call(EXT_HTTP_FREE, 0, 0, 0),  # free
            self._mov_reg(0, 6),           # return status
            self._halt(),
        ]

        return random.choice(prompts), instructions

    def gen_http_with_headers_pattern(self) -> Tuple[str, List[Instruction]]:
        """Generate HTTP request with custom headers."""
        prompts = [
            "make http request with authorization header",
            "send http request with custom headers",
            "http client with bearer token",
            "add content-type header to http request",
            "http request with accept header",
        ]

        instructions = [
            self._mov(0, 0x1000),          # r0 = URL ptr
            self._mov(1, 0x2000),          # r1 = "Authorization" header name
            self._mov(2, 0x3000),          # r2 = header value
            self._ext_call(EXT_HTTP_SET_HEADER, 0, 1, 2),  # set header
            self._mov(1, 0x4000),          # r1 = "Content-Type" header name
            self._mov(2, 0x5000),          # r2 = "application/json"
            self._ext_call(EXT_HTTP_SET_HEADER, 0, 1, 2),  # set header
            self._ext_call(EXT_HTTP_GET, 0, 0, 0),  # make request
            self._mov_reg(5, 0),           # r5 = response handle
            self._ext_call(EXT_HTTP_STATUS, 5, 0, 0),  # get status
            self._mov_reg(0, 5),
            self._ext_call(EXT_HTTP_FREE, 0, 0, 0),  # free
            self._halt(),
        ]

        return random.choice(prompts), instructions

    # ========================================================================
    # DATABASE PATTERNS
    # ========================================================================

    def gen_sqlite_query_pattern(self) -> Tuple[str, List[Instruction]]:
        """Generate SQLite query pattern."""
        tables = ['users', 'items', 'orders', 'products', 'accounts']
        table = random.choice(tables)

        prompts = [
            f"query {table} from sqlite database",
            f"execute select on {table} table",
            f"run sqlite query on {table}",
            f"fetch records from {table}",
            f"sqlite prepared statement for {table}",
        ]

        instructions = [
            self._mov(0, 0x1000),          # r0 = db path ptr
            self._ext_call(EXT_SQLITE_OPEN, 0, 0, 0),  # open db -> r0 = handle
            self._mov_reg(5, 0),           # r5 = db handle
            self._mov(1, 0x2000),          # r1 = SQL query ptr
            self._ext_call(EXT_SQLITE_PREPARE, 5, 1, 0),  # prepare -> r0 = stmt
            self._mov_reg(6, 0),           # r6 = statement handle
            self._mov(1, 1),               # r1 = param index
            self._mov(2, 0x3000),          # r2 = param value ptr
            self._ext_call(EXT_SQLITE_BIND, 6, 1, 2),  # bind parameter
            self._ext_call(EXT_SQLITE_STEP, 6, 0, 0),  # step -> r0 = result
            self._halt(),
        ]

        return random.choice(prompts), instructions

    def gen_sqlite_transaction_pattern(self) -> Tuple[str, List[Instruction]]:
        """Generate SQLite transaction pattern."""
        prompts = [
            "begin sqlite transaction and commit",
            "wrap database operations in transaction",
            "sqlite transaction with rollback on error",
            "atomic database update with transaction",
            "begin commit transaction pattern",
        ]

        instructions = [
            self._mov(0, 0x1000),          # r0 = db path ptr
            self._ext_call(EXT_SQLITE_OPEN, 0, 0, 0),  # open db
            self._mov_reg(5, 0),           # r5 = db handle
            self._ext_call(EXT_SQLITE_BEGIN, 5, 0, 0),  # begin transaction
            self._mov(1, 0x2000),          # r1 = SQL insert ptr
            self._ext_call(EXT_SQLITE_PREPARE, 5, 1, 0),  # prepare
            self._mov_reg(6, 0),           # r6 = stmt handle
            self._ext_call(EXT_SQLITE_STEP, 6, 0, 0),  # execute
            self._branch(self.BR_NE, 0, 0, 2),  # if error, rollback
            self._ext_call(EXT_SQLITE_COMMIT, 5, 0, 0),  # commit
            self._halt(),
            # rollback path:
            self._ext_call(EXT_SQLITE_ROLLBACK, 5, 0, 0),  # rollback
            self._halt(),
        ]

        return random.choice(prompts), instructions

    # ========================================================================
    # ENCODING PATTERNS
    # ========================================================================

    def gen_base64_roundtrip_pattern(self) -> Tuple[str, List[Instruction]]:
        """Generate base64 encode/decode pattern."""
        if random.random() > 0.5:
            prompts = [
                "encode binary data to base64",
                "base64 encode input buffer",
                "convert bytes to base64 string",
                "apply base64 encoding",
            ]
            ext_id = EXT_BASE64_ENCODE
        else:
            prompts = [
                "decode base64 to binary",
                "base64 decode string to bytes",
                "convert base64 to raw data",
                "apply base64 decoding",
            ]
            ext_id = EXT_BASE64_DECODE

        instructions = [
            self._mov(0, 0x1000),          # r0 = input ptr
            self._mov(1, 256),             # r1 = input length
            self._mov(2, 0x2000),          # r2 = output buffer
            self._ext_call(ext_id, 0, 1, 2),  # encode/decode
            self._halt(),
        ]

        return random.choice(prompts), instructions

    def gen_url_encode_pattern(self) -> Tuple[str, List[Instruction]]:
        """Generate URL encode/decode pattern."""
        if random.random() > 0.5:
            prompts = [
                "url encode query parameter",
                "percent encode string for url",
                "encode special chars for url",
                "apply url encoding",
            ]
            ext_id = EXT_URL_ENCODE
        else:
            prompts = [
                "url decode percent-encoded string",
                "decode url query parameter",
                "convert percent encoding to chars",
                "apply url decoding",
            ]
            ext_id = EXT_URL_DECODE

        instructions = [
            self._mov(0, 0x1000),          # r0 = input ptr
            self._mov(1, 128),             # r1 = input length
            self._mov(2, 0x2000),          # r2 = output buffer
            self._ext_call(ext_id, 0, 1, 2),
            self._halt(),
        ]

        return random.choice(prompts), instructions

    # ========================================================================
    # COMPRESSION PATTERNS
    # ========================================================================

    def gen_compression_pattern(self) -> Tuple[str, List[Instruction]]:
        """Generate gzip compression/decompression pattern."""
        if random.random() > 0.5:
            prompts = [
                "gzip compress data buffer",
                "apply gzip compression",
                "compress response with gzip",
                "deflate data using gzip",
            ]
            ext_id = EXT_GZIP_COMPRESS
        else:
            prompts = [
                "gzip decompress data buffer",
                "apply gzip decompression",
                "inflate gzip compressed data",
                "decompress gzip response",
            ]
            ext_id = EXT_GZIP_DECOMPRESS

        instructions = [
            self._mov(0, 0x1000),          # r0 = input ptr
            self._mov(1, 4096),            # r1 = input length
            self._mov(2, 0x2000),          # r2 = output buffer
            self._mov(3, 8192),            # r3 = output buffer size
            self._ext_call(ext_id, 0, 1, 2),  # compress/decompress
            self._halt(),
        ]

        return random.choice(prompts), instructions

    # ========================================================================
    # TLS PATTERNS
    # ========================================================================

    def gen_tls_secure_pattern(self) -> Tuple[str, List[Instruction]]:
        """Generate TLS secure connection pattern."""
        prompts = [
            "establish tls connection to server",
            "connect securely using tls",
            "tls handshake and send data",
            "create secure socket connection",
            "https connection using tls",
        ]

        instructions = [
            self._mov(0, 0x1000),          # r0 = hostname ptr
            self._mov(1, 443),             # r1 = port
            self._ext_call(EXT_TLS_CONNECT, 0, 1, 0),  # connect -> r0 = handle
            self._mov_reg(5, 0),           # r5 = tls handle
            self._mov(1, 0x2000),          # r1 = data to send
            self._mov(2, 256),             # r2 = data length
            self._ext_call(EXT_TLS_WRITE, 5, 1, 2),  # write
            self._mov(1, 0x3000),          # r1 = receive buffer
            self._mov(2, 4096),            # r2 = buffer size
            self._ext_call(EXT_TLS_READ, 5, 1, 2),  # read -> r0 = bytes read
            self._mov_reg(6, 0),           # r6 = bytes read
            self._mov_reg(0, 5),           # r0 = handle
            self._ext_call(EXT_TLS_CLOSE, 0, 0, 0),  # close
            self._mov_reg(0, 6),           # return bytes read
            self._halt(),
        ]

        return random.choice(prompts), instructions

    # ========================================================================
    # UUID PATTERNS
    # ========================================================================

    def gen_uuid_pattern(self) -> Tuple[str, List[Instruction]]:
        """Generate UUID generation pattern."""
        prompts = [
            "generate uuid v4",
            "create random uuid",
            "generate unique identifier",
            "create uuid for new record",
            "generate random uuid v4 string",
        ]

        instructions = [
            self._mov(0, 0x1000),          # r0 = output buffer (36 bytes)
            self._ext_call(EXT_UUID_V4, 0, 0, 0),  # generate uuid
            self._halt(),
        ]

        return random.choice(prompts), instructions


def main():
    """Test the generator."""
    import json

    gen = ExtensionPatternGenerator()
    samples = gen.generate_samples(100)

    print(f"Generated {len(samples)} samples")
    print("\nSample prompts:")
    for s in samples[:10]:
        print(f"  - {s['context']}")

    # Count valid instructions
    valid_counts = [sum(1 for i in s['instructions'] if i['valid']) for s in samples]
    avg_valid = sum(valid_counts) / len(valid_counts)
    print(f"\nAverage valid instructions: {avg_valid:.1f}")


if __name__ == '__main__':
    main()
