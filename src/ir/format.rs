//! IR Format Specification for Neurlang
//!
//! 32-opcode binary IR with implicit security, designed for AI code generation.
//!
//! # Instruction Encoding
//!
//! Base format (4 bytes):
//! ```text
//! ┌──────────┬──────────┬──────────┬──────────┐
//! │  Opcode  │  RegDst  │  RegSrc1 │  RegSrc2 │
//! │  6 bits  │  5 bits  │  5 bits  │  16 bits │
//! └──────────┴──────────┴──────────┴──────────┘
//! ```
//!
//! Extended format (8 bytes, with immediate):
//! ```text
//! ┌──────────┬──────────┬──────────┬──────────┬──────────────────────┐
//! │  Opcode  │  RegDst  │  RegSrc1 │  Flags   │  32-bit Immediate    │
//! │  6 bits  │  5 bits  │  5 bits  │  16 bits │  32 bits             │
//! └──────────┴──────────┴──────────┴──────────┴──────────────────────┘
//! ```

use std::fmt;

/// Opcode definitions (32 total, fits in 5 bits)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Opcode {
    // Arithmetic/Logic (3 opcodes → 16+ ops via mode bits)
    /// Arithmetic/Logic Unit operations: ADD, SUB, AND, OR, XOR, SHL, SHR, SAR
    Alu = 0x00,
    /// ALU with immediate operand
    AluI = 0x01,
    /// Multiply/Divide: MUL, DIV, MOD, MULH
    MulDiv = 0x02,

    // Memory (3 opcodes, implicit bounds checking)
    /// Load from memory (8/16/32/64-bit, auto bounds-checked)
    Load = 0x03,
    /// Store to memory (8/16/32/64-bit, auto bounds-checked)
    Store = 0x04,
    /// Atomic operations: CAS, XCHG, ADD, AND, OR, XOR, MIN, MAX
    Atomic = 0x05,

    // Control Flow (4 opcodes)
    /// Conditional branch: EQ, NE, LT, LE, GT, GE, unconditional
    Branch = 0x06,
    /// Function call (direct/indirect)
    Call = 0x07,
    /// Return from function
    Ret = 0x08,
    /// Unconditional jump (direct/indirect)
    Jump = 0x09,

    // Capabilities (3 opcodes - security)
    /// Create new capability (privileged)
    CapNew = 0x0A,
    /// Restrict capability (can only shrink bounds/perms)
    CapRestrict = 0x0B,
    /// Query capability (get base, length, permissions)
    CapQuery = 0x0C,

    // Concurrency (5 opcodes)
    /// Spawn new thread/task
    Spawn = 0x0D,
    /// Wait for task completion
    Join = 0x0E,
    /// Channel operations: create, send, recv, close
    Chan = 0x0F,
    /// Memory fence: acquire, release, seq_cst
    Fence = 0x10,
    /// Cooperative yield
    Yield = 0x11,

    // Taint tracking (2 opcodes - info flow security)
    /// Mark value as tainted
    Taint = 0x12,
    /// Remove taint after validation
    Sanitize = 0x13,

    // I/O (5 opcodes - file, network, console, time)
    /// File operations: open, read, write, close, seek, stat, mkdir, delete
    File = 0x14,
    /// Network operations: socket, connect, bind, listen, accept, send, recv, close
    Net = 0x15,
    /// Socket options: setopt(fd, option, value)
    NetSetopt = 0x16,
    /// Console I/O: print, read_line, get_args, get_env
    Io = 0x17,
    /// Time operations: now, sleep, monotonic
    Time = 0x18,

    // Math Extensions (3 opcodes)
    /// Floating-point: fadd, fsub, fmul, fdiv, fsqrt, fabs, ffloor, fceil
    Fpu = 0x19,
    /// Random: rand_bytes, rand_u64
    Rand = 0x1A,
    /// Bit manipulation: popcount, clz, ctz, bswap
    Bits = 0x1B,

    // System (4 opcodes)
    /// Move: reg-reg or load immediate
    Mov = 0x1C,
    /// System trap: syscall, breakpoint, fault
    Trap = 0x1D,
    /// No operation
    Nop = 0x1E,
    /// Halt execution
    Halt = 0x1F,

    // Extension opcodes (Tier 2 FFI)
    /// Call a registered Rust extension
    /// Format: ext.call rd, ext_id, rs1, rs2, rs3
    /// rd = result, ext_id = extension ID (immediate), rs1-rs3 = arguments
    ExtCall = 0x20,
}

impl Opcode {
    /// Convert from u8, returning None for invalid opcodes
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 0x20 {
            Some(unsafe { std::mem::transmute(val) })
        } else {
            None
        }
    }

    /// Check if this opcode uses an extended (8-byte) encoding
    pub fn is_extended(&self) -> bool {
        matches!(
            self,
            Opcode::AluI
                | Opcode::Load
                | Opcode::Store
                | Opcode::Branch
                | Opcode::Call
                | Opcode::Jump
                | Opcode::Mov
                | Opcode::File    // path pointer or buffer
                | Opcode::Net     // address/port info
                | Opcode::NetSetopt // option value
                | Opcode::Io      // buffer pointer
                | Opcode::Time    // duration value
                | Opcode::ExtCall // extension ID
        )
    }

    /// Get the mnemonic for this opcode
    pub fn mnemonic(&self) -> &'static str {
        match self {
            Opcode::Alu => "alu",
            Opcode::AluI => "alui",
            Opcode::MulDiv => "muldiv",
            Opcode::Load => "load",
            Opcode::Store => "store",
            Opcode::Atomic => "atomic",
            Opcode::Branch => "branch",
            Opcode::Call => "call",
            Opcode::Ret => "ret",
            Opcode::Jump => "jump",
            Opcode::CapNew => "cap.new",
            Opcode::CapRestrict => "cap.restrict",
            Opcode::CapQuery => "cap.query",
            Opcode::Spawn => "spawn",
            Opcode::Join => "join",
            Opcode::Chan => "chan",
            Opcode::Fence => "fence",
            Opcode::Yield => "yield",
            Opcode::Taint => "taint",
            Opcode::Sanitize => "sanitize",
            Opcode::File => "file",
            Opcode::Net => "net",
            Opcode::NetSetopt => "net.setopt",
            Opcode::Io => "io",
            Opcode::Time => "time",
            Opcode::Fpu => "fpu",
            Opcode::Rand => "rand",
            Opcode::Bits => "bits",
            Opcode::Mov => "mov",
            Opcode::Trap => "trap",
            Opcode::Nop => "nop",
            Opcode::Halt => "halt",
            Opcode::ExtCall => "ext.call",
        }
    }
}

/// ALU operation modes (3 bits → 8 operations)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AluOp {
    Add = 0,
    Sub = 1,
    And = 2,
    Or = 3,
    Xor = 4,
    Shl = 5,
    Shr = 6, // Logical shift right
    Sar = 7, // Arithmetic shift right
}

impl AluOp {
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 7 {
            Some(unsafe { std::mem::transmute(val) })
        } else {
            None
        }
    }
}

/// MulDiv operation modes (2 bits → 4 operations)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MulDivOp {
    Mul = 0,  // Low bits of multiplication
    MulH = 1, // High bits of multiplication (signed)
    Div = 2,
    Mod = 3,
}

impl MulDivOp {
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 3 {
            Some(unsafe { std::mem::transmute(val) })
        } else {
            None
        }
    }
}

/// Memory width modes (2 bits → 4 widths)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MemWidth {
    Byte = 0,   // 8-bit
    Half = 1,   // 16-bit
    Word = 2,   // 32-bit
    Double = 3, // 64-bit
}

impl MemWidth {
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 3 {
            Some(unsafe { std::mem::transmute(val) })
        } else {
            None
        }
    }

    pub fn byte_size(&self) -> usize {
        match self {
            MemWidth::Byte => 1,
            MemWidth::Half => 2,
            MemWidth::Word => 4,
            MemWidth::Double => 8,
        }
    }
}

/// Atomic operation modes (3 bits → 8 operations)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AtomicOp {
    Cas = 0,  // Compare-and-swap
    Xchg = 1, // Exchange
    Add = 2,
    And = 3,
    Or = 4,
    Xor = 5,
    Min = 6,
    Max = 7,
}

impl AtomicOp {
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 7 {
            Some(unsafe { std::mem::transmute(val) })
        } else {
            None
        }
    }
}

/// Branch condition modes (3 bits → 7 conditions + unconditional)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BranchCond {
    Always = 0,
    Eq = 1,
    Ne = 2,
    Lt = 3,  // Signed less than
    Le = 4,  // Signed less or equal
    Gt = 5,  // Signed greater than
    Ge = 6,  // Signed greater or equal
    Ltu = 7, // Unsigned less than (uses bit 7)
}

impl BranchCond {
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 7 {
            Some(unsafe { std::mem::transmute(val) })
        } else {
            None
        }
    }
}

/// Channel operation modes (2 bits)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ChanOp {
    Create = 0,
    Send = 1,
    Recv = 2,
    Close = 3,
}

impl ChanOp {
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 3 {
            Some(unsafe { std::mem::transmute(val) })
        } else {
            None
        }
    }
}

/// Memory fence modes (2 bits)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FenceMode {
    Acquire = 0,
    Release = 1,
    AcqRel = 2,
    SeqCst = 3,
}

impl FenceMode {
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 3 {
            Some(unsafe { std::mem::transmute(val) })
        } else {
            None
        }
    }
}

/// Trap types (3 bits)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TrapType {
    Syscall = 0,
    Breakpoint = 1,
    BoundsViolation = 2,
    CapabilityViolation = 3,
    TaintViolation = 4,
    DivByZero = 5,
    InvalidOp = 6,
    User = 7,
}

impl TrapType {
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 7 {
            Some(unsafe { std::mem::transmute(val) })
        } else {
            None
        }
    }
}

/// File operation modes (3 bits → 8 operations)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FileOp {
    Open = 0,   // open(path, flags) → fd
    Read = 1,   // read(fd, buf, len) → n
    Write = 2,  // write(fd, buf, len) → n
    Close = 3,  // close(fd)
    Seek = 4,   // seek(fd, offset, whence)
    Stat = 5,   // stat(path) → size, mtime
    Mkdir = 6,  // mkdir(path)
    Delete = 7, // delete(path)
}

impl FileOp {
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 7 {
            Some(unsafe { std::mem::transmute(val) })
        } else {
            None
        }
    }
}

/// Network operation modes (3 bits → 8 operations)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NetOp {
    Socket = 0,  // socket(domain, type) → fd
    Connect = 1, // connect(fd, addr, port)
    Bind = 2,    // bind(fd, addr, port)
    Listen = 3,  // listen(fd, backlog)
    Accept = 4,  // accept(fd) → client_fd
    Send = 5,    // send(fd, buf, len) → n
    Recv = 6,    // recv(fd, buf, len) → n
    Close = 7,   // close(fd)
}

impl NetOp {
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 7 {
            Some(unsafe { std::mem::transmute(val) })
        } else {
            None
        }
    }
}

/// Socket option types for NET.SETOPT
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NetOption {
    Nonblock = 0,    // 0=blocking (default), 1=non-blocking
    TimeoutMs = 1,   // recv/send timeout in milliseconds (0=infinite)
    Keepalive = 2,   // 0=off, 1=on
    ReuseAddr = 3,   // 0=off, 1=on
    NoDelay = 4,     // 0=off (Nagle on), 1=on (Nagle off)
    RecvBufSize = 5, // receive buffer size
    SendBufSize = 6, // send buffer size
    Linger = 7,      // linger time on close
}

impl NetOption {
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 7 {
            Some(unsafe { std::mem::transmute(val) })
        } else {
            None
        }
    }
}

/// Console I/O operation modes (2 bits → 4 operations)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IoOp {
    Print = 0,    // print(buf, len)
    ReadLine = 1, // read_line(buf, max) → n
    GetArgs = 2,  // get_args() → argc, argv_ptr
    GetEnv = 3,   // get_env(name) → value_ptr
}

impl IoOp {
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 3 {
            Some(unsafe { std::mem::transmute(val) })
        } else {
            None
        }
    }
}

/// Time operation modes (2 bits → 4 operations)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TimeOp {
    Now = 0,       // now() → unix_timestamp
    Sleep = 1,     // sleep(milliseconds)
    Monotonic = 2, // monotonic() → nanoseconds
    Reserved = 3,  // reserved for future use
}

impl TimeOp {
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 3 {
            Some(unsafe { std::mem::transmute(val) })
        } else {
            None
        }
    }
}

/// Floating-point operation modes (4 bits → 14 operations)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FpuOp {
    Fadd = 0,   // rd = rs1 + rs2 (f64)
    Fsub = 1,   // rd = rs1 - rs2 (f64)
    Fmul = 2,   // rd = rs1 * rs2 (f64)
    Fdiv = 3,   // rd = rs1 / rs2 (f64)
    Fsqrt = 4,  // rd = sqrt(rs1)
    Fabs = 5,   // rd = abs(rs1)
    Ffloor = 6, // rd = floor(rs1)
    Fceil = 7,  // rd = ceil(rs1)
    // Comparison operations (modes 8-13) - return 1 if true, 0 if false
    Fcmpeq = 8,  // rd = (rs1 == rs2) ? 1 : 0
    Fcmpne = 9,  // rd = (rs1 != rs2) ? 1 : 0
    Fcmplt = 10, // rd = (rs1 < rs2) ? 1 : 0
    Fcmple = 11, // rd = (rs1 <= rs2) ? 1 : 0
    Fcmpgt = 12, // rd = (rs1 > rs2) ? 1 : 0
    Fcmpge = 13, // rd = (rs1 >= rs2) ? 1 : 0
}

impl FpuOp {
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 13 {
            Some(unsafe { std::mem::transmute(val) })
        } else {
            None
        }
    }
}

/// Random number operation modes (1 bit → 2 operations)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RandOp {
    RandBytes = 0, // rand_bytes(buf, len)
    RandU64 = 1,   // rand_u64() → r0
}

impl RandOp {
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 1 {
            Some(unsafe { std::mem::transmute(val) })
        } else {
            None
        }
    }
}

/// Bit manipulation operation modes (2 bits → 4 operations)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BitsOp {
    Popcount = 0, // count set bits
    Clz = 1,      // count leading zeros
    Ctz = 2,      // count trailing zeros
    Bswap = 3,    // byte swap (endian conversion)
}

impl BitsOp {
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 3 {
            Some(unsafe { std::mem::transmute(val) })
        } else {
            None
        }
    }
}

/// Capability permissions (8 bits)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CapPerms(pub u8);

impl CapPerms {
    pub const READ: u8 = 0b0000_0001;
    pub const WRITE: u8 = 0b0000_0010;
    pub const EXEC: u8 = 0b0000_0100;
    pub const CAP: u8 = 0b0000_1000; // Can store/load capabilities
    pub const SEAL: u8 = 0b0001_0000;
    pub const UNSEAL: u8 = 0b0010_0000;

    pub fn new(bits: u8) -> Self {
        Self(bits)
    }

    pub fn can_read(&self) -> bool {
        self.0 & Self::READ != 0
    }
    pub fn can_write(&self) -> bool {
        self.0 & Self::WRITE != 0
    }
    pub fn can_exec(&self) -> bool {
        self.0 & Self::EXEC != 0
    }

    /// Check if `other` is a subset of `self` (restriction is valid)
    pub fn can_restrict_to(&self, other: CapPerms) -> bool {
        (self.0 & other.0) == other.0
    }
}

/// Fat pointer format (128 bits total for full capability)
///
/// Compact representation for register storage:
/// ```text
/// [TAG:4][TAINT:4][PERMS:8][LENGTH:16][BASE:32] = 64 bits
/// [ADDRESS:64] = 64 bits
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FatPointer {
    /// Tag indicating this is a valid capability (magic value)
    pub tag: u8,
    /// Taint level (0 = untainted, higher = more tainted)
    pub taint: u8,
    /// Permission bits
    pub perms: CapPerms,
    /// Length of accessible region in bytes
    pub length: u32,
    /// Base address of region
    pub base: u64,
    /// Current address (must be within [base, base+length))
    pub address: u64,
}

impl FatPointer {
    /// Magic tag value indicating a valid capability
    pub const VALID_TAG: u8 = 0xCA;

    /// Create a new fat pointer with full access to a region
    pub fn new(base: u64, length: u32, perms: CapPerms) -> Self {
        Self {
            tag: Self::VALID_TAG,
            taint: 0,
            perms,
            length,
            base,
            address: base,
        }
    }

    /// Check if this is a valid capability
    pub fn is_valid(&self) -> bool {
        self.tag == Self::VALID_TAG
    }

    /// Check if an access of `size` bytes at current address is in bounds
    pub fn check_bounds(&self, size: usize) -> bool {
        if !self.is_valid() {
            return false;
        }
        let end = self.address.saturating_add(size as u64);
        self.address >= self.base && end <= self.base.saturating_add(self.length as u64)
    }

    /// Restrict to a narrower region (can only shrink)
    pub fn restrict(&self, new_base: u64, new_length: u32, new_perms: CapPerms) -> Option<Self> {
        // Can only shrink bounds
        if new_base < self.base {
            return None;
        }
        if new_base + new_length as u64 > self.base + self.length as u64 {
            return None;
        }
        // Can only remove permissions
        if !self.perms.can_restrict_to(new_perms) {
            return None;
        }

        Some(Self {
            tag: Self::VALID_TAG,
            taint: self.taint, // Taint propagates
            perms: new_perms,
            length: new_length,
            base: new_base,
            address: new_base,
        })
    }

    /// Encode to 128 bits (two u64 values)
    pub fn encode(&self) -> (u64, u64) {
        let metadata = ((self.tag as u64) << 56)
            | ((self.taint as u64) << 48)
            | ((self.perms.0 as u64) << 40)
            | ((self.length as u64) << 8)
            | (self.base & 0xFF); // Low 8 bits of base

        let address = self.address | ((self.base >> 8) << 56); // Pack remaining base bits

        (metadata, address)
    }

    /// Decode from 128 bits
    pub fn decode(metadata: u64, address_packed: u64) -> Self {
        let tag = ((metadata >> 56) & 0xFF) as u8;
        let taint = ((metadata >> 48) & 0xFF) as u8;
        let perms = CapPerms(((metadata >> 40) & 0xFF) as u8);
        let length = ((metadata >> 8) & 0xFFFFFFFF) as u32;
        let base_low = metadata & 0xFF;
        let base_high = (address_packed >> 56) << 8;
        let base = base_high | base_low;
        let address = address_packed & 0x00FF_FFFF_FFFF_FFFF;

        Self {
            tag,
            taint,
            perms,
            length,
            base,
            address,
        }
    }
}

/// Register identifiers (5 bits → 32 registers)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Register {
    // General purpose (R0-R15)
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13,
    R14 = 14,
    R15 = 15,
    // Special purpose
    Sp = 16,  // Stack pointer
    Fp = 17,  // Frame pointer
    Lr = 18,  // Link register
    Pc = 19,  // Program counter (read-only)
    Csp = 20, // Capability stack pointer
    Cfp = 21, // Capability frame pointer
    // Reserved for future
    Reserved22 = 22,
    Reserved23 = 23,
    // Argument/return registers (aliases)
    // A0-A5 = R0-R5, Ret = R0
    Reserved24 = 24,
    Reserved25 = 25,
    Reserved26 = 26,
    Reserved27 = 27,
    Reserved28 = 28,
    Reserved29 = 29,
    Reserved30 = 30,
    Zero = 31, // Always zero
}

impl Register {
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 31 {
            Some(unsafe { std::mem::transmute(val) })
        } else {
            None
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Register::R0 => "r0",
            Register::R1 => "r1",
            Register::R2 => "r2",
            Register::R3 => "r3",
            Register::R4 => "r4",
            Register::R5 => "r5",
            Register::R6 => "r6",
            Register::R7 => "r7",
            Register::R8 => "r8",
            Register::R9 => "r9",
            Register::R10 => "r10",
            Register::R11 => "r11",
            Register::R12 => "r12",
            Register::R13 => "r13",
            Register::R14 => "r14",
            Register::R15 => "r15",
            Register::Sp => "sp",
            Register::Fp => "fp",
            Register::Lr => "lr",
            Register::Pc => "pc",
            Register::Csp => "csp",
            Register::Cfp => "cfp",
            Register::Zero => "zero",
            _ => "reserved",
        }
    }

    /// Check if this register is writable
    pub fn is_writable(&self) -> bool {
        !matches!(self, Register::Pc | Register::Zero)
    }
}

/// Decoded instruction representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub rd: Register,     // Destination register
    pub rs1: Register,    // Source register 1
    pub rs2: Register,    // Source register 2 (or mode bits encoded)
    pub mode: u8,         // Mode bits (ALU op, mem width, etc.)
    pub imm: Option<i32>, // Optional immediate value
}

impl Instruction {
    /// Create a new instruction without immediate
    pub fn new(opcode: Opcode, rd: Register, rs1: Register, rs2: Register, mode: u8) -> Self {
        Self {
            opcode,
            rd,
            rs1,
            rs2,
            mode,
            imm: None,
        }
    }

    /// Create a new instruction with immediate
    pub fn with_imm(opcode: Opcode, rd: Register, rs1: Register, mode: u8, imm: i32) -> Self {
        Self {
            opcode,
            rd,
            rs1,
            rs2: Register::Zero,
            mode,
            imm: Some(imm),
        }
    }

    /// Create a branch instruction with two registers to compare
    pub fn branch(cond: BranchCond, rs1: Register, rs2: Register, offset: i32) -> Self {
        Self {
            opcode: Opcode::Branch,
            rd: Register::Zero,
            rs1,
            rs2,
            mode: cond as u8,
            imm: Some(offset),
        }
    }

    /// Encode to bytes (4 or 8 bytes)
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8);

        // First 4 bytes: opcode(6) | rd(5) | rs1(5) | rs2_or_mode(16)
        let word1 = ((self.opcode as u32) << 26)
            | ((self.rd as u32) << 21)
            | ((self.rs1 as u32) << 16)
            | ((self.rs2 as u32) << 11)
            | ((self.mode as u32) << 8);

        bytes.extend_from_slice(&word1.to_le_bytes());

        // Second word for immediate (always include for extended opcodes)
        if self.imm.is_some() || self.opcode.is_extended() {
            let imm = self.imm.unwrap_or(0);
            bytes.extend_from_slice(&imm.to_le_bytes());
        }

        bytes
    }

    /// Decode from bytes
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 4 {
            return None;
        }

        let word1 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let opcode = Opcode::from_u8(((word1 >> 26) & 0x3F) as u8)?;
        let rd = Register::from_u8(((word1 >> 21) & 0x1F) as u8)?;
        let rs1 = Register::from_u8(((word1 >> 16) & 0x1F) as u8)?;
        let rs2 = Register::from_u8(((word1 >> 11) & 0x1F) as u8)?;
        let mode = ((word1 >> 8) & 0x07) as u8;

        let imm = if opcode.is_extended() && bytes.len() >= 8 {
            Some(i32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]))
        } else {
            None
        };

        Some(Self {
            opcode,
            rd,
            rs1,
            rs2,
            mode,
            imm,
        })
    }

    /// Get the size of this instruction in bytes
    pub fn size(&self) -> usize {
        if self.imm.is_some() || self.opcode.is_extended() {
            8
        } else {
            4
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.opcode.mnemonic())?;

        match self.opcode {
            Opcode::Alu | Opcode::AluI => {
                let op = AluOp::from_u8(self.mode).unwrap_or(AluOp::Add);
                write!(f, ".{:?} {}, {}, ", op, self.rd.name(), self.rs1.name())?;
                if let Some(imm) = self.imm {
                    write!(f, "{}", imm)?;
                } else {
                    write!(f, "{}", self.rs2.name())?;
                }
            }
            Opcode::MulDiv => {
                let op = MulDivOp::from_u8(self.mode).unwrap_or(MulDivOp::Mul);
                write!(
                    f,
                    ".{:?} {}, {}, {}",
                    op,
                    self.rd.name(),
                    self.rs1.name(),
                    self.rs2.name()
                )?;
            }
            Opcode::Load | Opcode::Store => {
                let width = MemWidth::from_u8(self.mode).unwrap_or(MemWidth::Double);
                write!(f, ".{:?} {}, [{}", width, self.rd.name(), self.rs1.name())?;
                if let Some(imm) = self.imm {
                    if imm != 0 {
                        write!(f, " + {}", imm)?;
                    }
                }
                write!(f, "]")?;
            }
            Opcode::Branch => {
                let cond = BranchCond::from_u8(self.mode).unwrap_or(BranchCond::Always);
                if matches!(cond, BranchCond::Always) {
                    write!(f, " ")?;
                } else {
                    write!(f, ".{:?} {}, {}, ", cond, self.rs1.name(), self.rs2.name())?;
                }
                if let Some(imm) = self.imm {
                    write!(f, "{}", imm)?;
                }
            }
            Opcode::Mov => {
                write!(f, " {}, ", self.rd.name())?;
                // Register-to-register move: rs1 is not Zero
                if self.rs1 != Register::Zero {
                    write!(f, "{}", self.rs1.name())?;
                } else if let Some(imm) = self.imm {
                    write!(f, "{}", imm)?;
                } else {
                    write!(f, "0")?;
                }
            }
            Opcode::Ret | Opcode::Nop | Opcode::Halt | Opcode::Yield => {
                // No operands
            }
            Opcode::File => {
                let op = FileOp::from_u8(self.mode).unwrap_or(FileOp::Open);
                write!(f, ".{:?} {}, {}", op, self.rd.name(), self.rs1.name())?;
                if self.rs2 != Register::Zero {
                    write!(f, ", {}", self.rs2.name())?;
                }
                if let Some(imm) = self.imm {
                    write!(f, ", {}", imm)?;
                }
            }
            Opcode::Net => {
                let op = NetOp::from_u8(self.mode).unwrap_or(NetOp::Socket);
                write!(f, ".{:?} {}, {}", op, self.rd.name(), self.rs1.name())?;
                if self.rs2 != Register::Zero {
                    write!(f, ", {}", self.rs2.name())?;
                }
                if let Some(imm) = self.imm {
                    write!(f, ", {}", imm)?;
                }
            }
            Opcode::NetSetopt => {
                let opt = NetOption::from_u8(self.mode).unwrap_or(NetOption::Nonblock);
                write!(f, ".{:?} {}", opt, self.rs1.name())?;
                if let Some(imm) = self.imm {
                    write!(f, ", {}", imm)?;
                }
            }
            Opcode::Io => {
                let op = IoOp::from_u8(self.mode).unwrap_or(IoOp::Print);
                write!(f, ".{:?} {}, {}", op, self.rd.name(), self.rs1.name())?;
                if self.rs2 != Register::Zero {
                    write!(f, ", {}", self.rs2.name())?;
                }
                if let Some(imm) = self.imm {
                    write!(f, ", {}", imm)?;
                }
            }
            Opcode::Time => {
                let op = TimeOp::from_u8(self.mode).unwrap_or(TimeOp::Now);
                match op {
                    TimeOp::Now | TimeOp::Monotonic => write!(f, ".{:?} {}", op, self.rd.name())?,
                    TimeOp::Sleep => {
                        write!(f, ".{:?}", op)?;
                        if let Some(imm) = self.imm {
                            write!(f, " {}", imm)?;
                        }
                    }
                    TimeOp::Reserved => write!(f, ".reserved")?,
                }
            }
            Opcode::Fpu => {
                let op = FpuOp::from_u8(self.mode).unwrap_or(FpuOp::Fadd);
                match op {
                    FpuOp::Fadd | FpuOp::Fsub | FpuOp::Fmul | FpuOp::Fdiv => {
                        write!(
                            f,
                            ".{:?} {}, {}, {}",
                            op,
                            self.rd.name(),
                            self.rs1.name(),
                            self.rs2.name()
                        )?;
                    }
                    FpuOp::Fsqrt | FpuOp::Fabs | FpuOp::Ffloor | FpuOp::Fceil => {
                        write!(f, ".{:?} {}, {}", op, self.rd.name(), self.rs1.name())?;
                    }
                    // Comparison operations return 1 or 0
                    FpuOp::Fcmpeq
                    | FpuOp::Fcmpne
                    | FpuOp::Fcmplt
                    | FpuOp::Fcmple
                    | FpuOp::Fcmpgt
                    | FpuOp::Fcmpge => {
                        write!(
                            f,
                            ".{:?} {}, {}, {}",
                            op,
                            self.rd.name(),
                            self.rs1.name(),
                            self.rs2.name()
                        )?;
                    }
                }
            }
            Opcode::Rand => {
                let op = RandOp::from_u8(self.mode).unwrap_or(RandOp::RandU64);
                match op {
                    RandOp::RandBytes => {
                        write!(f, ".bytes {}, {}", self.rd.name(), self.rs1.name())?
                    }
                    RandOp::RandU64 => write!(f, ".u64 {}", self.rd.name())?,
                }
            }
            Opcode::Bits => {
                let op = BitsOp::from_u8(self.mode).unwrap_or(BitsOp::Popcount);
                write!(f, ".{:?} {}, {}", op, self.rd.name(), self.rs1.name())?;
            }
            Opcode::ExtCall => {
                // ext.call rd, ext_id, rs1, rs2
                write!(f, " {}", self.rd.name())?;
                if let Some(ext_id) = self.imm {
                    write!(f, ", {}", ext_id)?;
                }
                if self.rs1 != Register::Zero {
                    write!(f, ", {}", self.rs1.name())?;
                }
                if self.rs2 != Register::Zero {
                    write!(f, ", {}", self.rs2.name())?;
                }
            }
            _ => {
                // Generic format
                write!(
                    f,
                    " {}, {}, {}",
                    self.rd.name(),
                    self.rs1.name(),
                    self.rs2.name()
                )?;
                if let Some(imm) = self.imm {
                    write!(f, ", {}", imm)?;
                }
            }
        }

        Ok(())
    }
}

/// Program representation
#[derive(Debug, Clone)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub entry_point: usize,
    pub data_section: Vec<u8>,
    /// Entry point label (used by assembler to resolve entry_point)
    pub entry_label: Option<String>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            entry_point: 0,
            data_section: Vec::new(),
            entry_label: None,
        }
    }

    /// Create a program from a list of instructions
    pub fn from_instructions(instructions: Vec<Instruction>) -> Self {
        Self {
            instructions,
            entry_point: 0,
            data_section: Vec::new(),
            entry_label: None,
        }
    }

    /// Encode the entire program to bytes
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Header: magic(4) + entry(4) + code_len(4) + data_len(4)
        bytes.extend_from_slice(b"NRLG"); // Magic (Neurlang)
        bytes.extend_from_slice(&(self.entry_point as u32).to_le_bytes());

        let code: Vec<u8> = self.instructions.iter().flat_map(|i| i.encode()).collect();
        bytes.extend_from_slice(&(code.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&(self.data_section.len() as u32).to_le_bytes());

        // Code section
        bytes.extend(code);

        // Data section
        bytes.extend_from_slice(&self.data_section);

        bytes
    }

    /// Decode a program from bytes
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 16 || &bytes[0..4] != b"NRLG" {
            return None;
        }

        let entry_point = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
        let code_len = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
        let data_len = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;

        if bytes.len() < 16 + code_len + data_len {
            return None;
        }

        let code_bytes = &bytes[16..16 + code_len];
        let data_section = bytes[16 + code_len..16 + code_len + data_len].to_vec();

        let mut instructions = Vec::new();
        let mut offset = 0;

        while offset < code_bytes.len() {
            let instr = Instruction::decode(&code_bytes[offset..])?;
            let size = instr.size();
            instructions.push(instr);
            offset += size;
        }

        Some(Self {
            instructions,
            entry_point,
            data_section,
            entry_label: None,
        })
    }

    /// Get total code size in bytes
    pub fn code_size(&self) -> usize {
        self.instructions.iter().map(|i| i.size()).sum()
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_encoding() {
        assert_eq!(Opcode::from_u8(0x00), Some(Opcode::Alu));
        assert_eq!(Opcode::from_u8(0x1F), Some(Opcode::Halt));
        assert_eq!(Opcode::from_u8(0x20), Some(Opcode::ExtCall));
        assert_eq!(Opcode::from_u8(0x21), None); // Next invalid opcode
                                                 // Test new I/O opcodes
        assert_eq!(Opcode::from_u8(0x14), Some(Opcode::File));
        assert_eq!(Opcode::from_u8(0x15), Some(Opcode::Net));
        assert_eq!(Opcode::from_u8(0x19), Some(Opcode::Fpu));
    }

    #[test]
    fn test_instruction_encode_decode() {
        let instr = Instruction::new(
            Opcode::Alu,
            Register::R0,
            Register::R1,
            Register::R2,
            AluOp::Add as u8,
        );
        let bytes = instr.encode();
        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(instr, decoded);
    }

    #[test]
    fn test_instruction_with_immediate() {
        let instr = Instruction::with_imm(
            Opcode::AluI,
            Register::R0,
            Register::R1,
            AluOp::Add as u8,
            42,
        );
        let bytes = instr.encode();
        assert_eq!(bytes.len(), 8);
        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(instr.imm, decoded.imm);
    }

    #[test]
    fn test_fat_pointer_bounds() {
        let ptr = FatPointer::new(0x1000, 256, CapPerms::new(CapPerms::READ | CapPerms::WRITE));
        assert!(ptr.check_bounds(1));
        assert!(ptr.check_bounds(256));
        assert!(!ptr.check_bounds(257));
    }

    #[test]
    fn test_fat_pointer_restrict() {
        let ptr = FatPointer::new(0x1000, 256, CapPerms::new(CapPerms::READ | CapPerms::WRITE));

        // Valid restriction
        let restricted = ptr
            .restrict(0x1010, 100, CapPerms::new(CapPerms::READ))
            .unwrap();
        assert_eq!(restricted.base, 0x1010);
        assert_eq!(restricted.length, 100);
        assert!(restricted.perms.can_read());
        assert!(!restricted.perms.can_write());

        // Invalid: expanding base
        assert!(ptr
            .restrict(0x0F00, 256, CapPerms::new(CapPerms::READ))
            .is_none());

        // Invalid: adding permissions
        assert!(ptr
            .restrict(0x1000, 256, CapPerms::new(CapPerms::READ | CapPerms::EXEC))
            .is_none());
    }

    #[test]
    fn test_program_encode_decode() {
        let mut prog = Program::new();
        prog.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            42,
        ));
        prog.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        let bytes = prog.encode();
        let decoded = Program::decode(&bytes).unwrap();

        assert_eq!(decoded.instructions.len(), 2);
        assert_eq!(decoded.instructions[0].imm, Some(42));
    }
}
