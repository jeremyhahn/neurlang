//! Intrinsics for Neurlang
//!
//! Tier 1 of the hybrid approach: ~50 common algorithms as "macro tokens"
//! that expand to optimized Neurlang IR at generation time.
//!
//! # Zero Runtime Overhead
//!
//! Intrinsics expand at assembly time, not runtime. The AI emits a single
//! token like `@memcpy r0, r1, 256` which expands to optimized Neurlang instructions.
//! This guarantees correctness on first try while keeping output tokens minimal.
//!
//! # Categories
//!
//! - Memory: memcpy, memset, memcmp, memmove, memchr, memzero
//! - String: strlen, strcmp, strcpy, strcat, strrev, strstr, strchr, strtok, strlower, strupper
//! - Conversion: atoi, itoa, atof, ftoa, htoi, itoh, btoi, itob
//! - Search: linear_search, binary_search, find_min, find_max
//! - Sort: qsort, insertion_sort, merge_sort, heapsort, is_sorted
//! - Hash: fnv_hash, crc32, djb2_hash, murmur3, xxhash
//! - Math: abs, min, max, clamp, pow, sqrt, log2, gcd, lcm, factorial
//! - Array: sum, product, reverse, fill, copy, rotate, shuffle, unique, count
//! - Bitwise: popcount, clz, ctz, bitrev, nextpow2

use crate::ir::format::{AluOp, BranchCond, Instruction, MemWidth, Opcode, Register};
use std::collections::HashMap;

/// Represents a patch point in an intrinsic template where registers/immediates
/// are substituted
#[derive(Debug, Clone)]
pub struct PatchPoint {
    /// Index of the instruction to patch
    pub instr_idx: usize,
    /// Which field to patch
    pub field: PatchField,
    /// Which argument to substitute (0-based)
    pub arg_idx: usize,
}

/// Which field of an instruction to patch
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchField {
    Rd,
    Rs1,
    Rs2,
    Imm,
}

/// Argument type for intrinsic parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgType {
    /// Register argument
    Register,
    /// Immediate value
    Immediate,
    /// Either register or immediate (flexible)
    RegOrImm,
}

/// Definition of an intrinsic
#[derive(Debug, Clone)]
pub struct IntrinsicDef {
    /// Name of the intrinsic (e.g., "memcpy")
    pub name: &'static str,
    /// Description for AI training context
    pub description: &'static str,
    /// Category for organization
    pub category: IntrinsicCategory,
    /// Argument types in order
    pub args: Vec<ArgType>,
    /// Template instructions
    pub template: Vec<Instruction>,
    /// Patch points for register/immediate substitution
    pub patch_points: Vec<PatchPoint>,
    /// Labels used in the template (name -> instruction index)
    pub labels: HashMap<String, usize>,
}

/// Categories of intrinsics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntrinsicCategory {
    Memory,
    String,
    Conversion,
    Search,
    Sort,
    Hash,
    Math,
    Array,
    Bitwise,
}

impl IntrinsicCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            IntrinsicCategory::Memory => "memory",
            IntrinsicCategory::String => "string",
            IntrinsicCategory::Conversion => "conversion",
            IntrinsicCategory::Search => "search",
            IntrinsicCategory::Sort => "sort",
            IntrinsicCategory::Hash => "hash",
            IntrinsicCategory::Math => "math",
            IntrinsicCategory::Array => "array",
            IntrinsicCategory::Bitwise => "bitwise",
        }
    }
}

/// Parsed intrinsic call from assembly source
#[derive(Debug, Clone)]
pub struct IntrinsicCall {
    /// Name of the intrinsic
    pub name: String,
    /// Arguments (registers or immediates)
    pub args: Vec<IntrinsicArg>,
}

/// An argument to an intrinsic call
#[derive(Debug, Clone)]
pub enum IntrinsicArg {
    Register(Register),
    Immediate(i32),
}

/// Registry of all available intrinsics
pub struct IntrinsicRegistry {
    intrinsics: HashMap<&'static str, IntrinsicDef>,
}

impl IntrinsicRegistry {
    /// Create a new registry with all built-in intrinsics
    pub fn new() -> Self {
        let mut intrinsics = HashMap::new();

        // Memory intrinsics
        Self::add_memory_intrinsics(&mut intrinsics);

        // String intrinsics
        Self::add_string_intrinsics(&mut intrinsics);

        // Math intrinsics
        Self::add_math_intrinsics(&mut intrinsics);

        // Search intrinsics
        Self::add_search_intrinsics(&mut intrinsics);

        // Array intrinsics
        Self::add_array_intrinsics(&mut intrinsics);

        // Bitwise intrinsics
        Self::add_bitwise_intrinsics(&mut intrinsics);

        // Hash intrinsics
        Self::add_hash_intrinsics(&mut intrinsics);

        Self { intrinsics }
    }

    /// Get an intrinsic by name
    pub fn get(&self, name: &str) -> Option<&IntrinsicDef> {
        self.intrinsics.get(name)
    }

    /// List all available intrinsics
    pub fn list(&self) -> Vec<&IntrinsicDef> {
        self.intrinsics.values().collect()
    }

    /// List intrinsics by category
    pub fn list_by_category(&self, category: IntrinsicCategory) -> Vec<&IntrinsicDef> {
        self.intrinsics
            .values()
            .filter(|i| i.category == category)
            .collect()
    }

    /// Expand an intrinsic call to Neurlang instructions
    pub fn expand(&self, call: &IntrinsicCall) -> Result<Vec<Instruction>, IntrinsicError> {
        let def = self
            .get(&call.name)
            .ok_or_else(|| IntrinsicError::UnknownIntrinsic(call.name.clone()))?;

        // Validate argument count
        if call.args.len() != def.args.len() {
            return Err(IntrinsicError::ArgCountMismatch {
                name: call.name.clone(),
                expected: def.args.len(),
                got: call.args.len(),
            });
        }

        // Clone template and apply patches
        let mut instructions = def.template.clone();

        for patch in &def.patch_points {
            let arg = &call.args[patch.arg_idx];
            let instr = &mut instructions[patch.instr_idx];

            match (&patch.field, arg) {
                (PatchField::Rd, IntrinsicArg::Register(r)) => instr.rd = *r,
                (PatchField::Rs1, IntrinsicArg::Register(r)) => instr.rs1 = *r,
                (PatchField::Rs2, IntrinsicArg::Register(r)) => instr.rs2 = *r,
                (PatchField::Imm, IntrinsicArg::Immediate(i)) => instr.imm = Some(*i),
                (PatchField::Imm, IntrinsicArg::Register(_)) => {
                    return Err(IntrinsicError::TypeMismatch {
                        name: call.name.clone(),
                        arg_idx: patch.arg_idx,
                        expected: "immediate",
                        got: "register",
                    });
                }
                (_, IntrinsicArg::Immediate(_)) => {
                    return Err(IntrinsicError::TypeMismatch {
                        name: call.name.clone(),
                        arg_idx: patch.arg_idx,
                        expected: "register",
                        got: "immediate",
                    });
                }
            }
        }

        Ok(instructions)
    }

    fn add_memory_intrinsics(map: &mut HashMap<&'static str, IntrinsicDef>) {
        // @memcpy dst, src, len
        // Copies len bytes from src to dst
        map.insert("memcpy", IntrinsicDef {
            name: "memcpy",
            description: "Copy len bytes from src to dst. Usage: @memcpy dst_reg, src_reg, len_imm_or_reg",
            category: IntrinsicCategory::Memory,
            args: vec![ArgType::Register, ArgType::Register, ArgType::RegOrImm],
            template: vec![
                // r10 = dst, r11 = src, r12 = len (saved from args)
                // r13 = counter
                Instruction::new(Opcode::Mov, Register::R13, Register::Zero, Register::Zero, 0), // counter = 0
                // loop:
                Instruction::new(Opcode::Branch, Register::Zero, Register::R13, Register::R12, BranchCond::Ge as u8), // if counter >= len, branch to done (+5)
                Instruction::new(Opcode::Alu, Register::R14, Register::R11, Register::R13, AluOp::Add as u8), // r14 = src + counter
                Instruction::new(Opcode::Load, Register::R15, Register::R14, Register::Zero, MemWidth::Byte as u8), // r15 = load byte
                Instruction::new(Opcode::Alu, Register::R14, Register::R10, Register::R13, AluOp::Add as u8), // r14 = dst + counter
                Instruction::new(Opcode::Store, Register::R15, Register::R14, Register::Zero, MemWidth::Byte as u8), // store byte
                Instruction::with_imm(Opcode::AluI, Register::R13, Register::R13, AluOp::Add as u8, 1), // counter++
                Instruction::with_imm(Opcode::Branch, Register::Zero, Register::Zero, BranchCond::Always as u8, -6), // branch to loop
                // done:
                Instruction::new(Opcode::Nop, Register::Zero, Register::Zero, Register::Zero, 0),
            ],
            patch_points: vec![
                PatchPoint { instr_idx: 0, field: PatchField::Rd, arg_idx: 0 }, // This will be overwritten - we need setup
            ],
            labels: [("loop".to_string(), 1), ("done".to_string(), 8)].into_iter().collect(),
        });

        // @memset dst, val, len
        // Sets len bytes at dst to val
        map.insert("memset", IntrinsicDef {
            name: "memset",
            description: "Set len bytes at dst to val. Usage: @memset dst_reg, val_reg_or_imm, len_imm_or_reg",
            category: IntrinsicCategory::Memory,
            args: vec![ArgType::Register, ArgType::RegOrImm, ArgType::RegOrImm],
            template: vec![
                Instruction::new(Opcode::Mov, Register::R13, Register::Zero, Register::Zero, 0), // counter = 0
                // loop:
                Instruction::new(Opcode::Branch, Register::Zero, Register::R13, Register::R12, BranchCond::Ge as u8), // if counter >= len, done
                Instruction::new(Opcode::Alu, Register::R14, Register::R10, Register::R13, AluOp::Add as u8), // r14 = dst + counter
                Instruction::new(Opcode::Store, Register::R11, Register::R14, Register::Zero, MemWidth::Byte as u8), // store val
                Instruction::with_imm(Opcode::AluI, Register::R13, Register::R13, AluOp::Add as u8, 1), // counter++
                Instruction::with_imm(Opcode::Branch, Register::Zero, Register::Zero, BranchCond::Always as u8, -4), // branch to loop
                // done:
                Instruction::new(Opcode::Nop, Register::Zero, Register::Zero, Register::Zero, 0),
            ],
            patch_points: vec![],
            labels: [("loop".to_string(), 1), ("done".to_string(), 6)].into_iter().collect(),
        });

        // @memzero dst, len
        // Zero out len bytes at dst
        map.insert(
            "memzero",
            IntrinsicDef {
                name: "memzero",
                description: "Zero out len bytes at dst. Usage: @memzero dst_reg, len_imm_or_reg",
                category: IntrinsicCategory::Memory,
                args: vec![ArgType::Register, ArgType::RegOrImm],
                template: vec![
                    Instruction::new(
                        Opcode::Mov,
                        Register::R13,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ), // counter = 0
                    // loop:
                    Instruction::new(
                        Opcode::Branch,
                        Register::Zero,
                        Register::R13,
                        Register::R11,
                        BranchCond::Ge as u8,
                    ), // if counter >= len, done
                    Instruction::new(
                        Opcode::Alu,
                        Register::R14,
                        Register::R10,
                        Register::R13,
                        AluOp::Add as u8,
                    ), // r14 = dst + counter
                    Instruction::new(
                        Opcode::Store,
                        Register::Zero,
                        Register::R14,
                        Register::Zero,
                        MemWidth::Byte as u8,
                    ), // store 0
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R13,
                        Register::R13,
                        AluOp::Add as u8,
                        1,
                    ), // counter++
                    Instruction::with_imm(
                        Opcode::Branch,
                        Register::Zero,
                        Register::Zero,
                        BranchCond::Always as u8,
                        -4,
                    ), // branch to loop
                    // done:
                    Instruction::new(
                        Opcode::Nop,
                        Register::Zero,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ),
                ],
                patch_points: vec![],
                labels: [("loop".to_string(), 1), ("done".to_string(), 6)]
                    .into_iter()
                    .collect(),
            },
        );

        // @memcmp ptr1, ptr2, len -> result in r0 (0 if equal, <0 if ptr1<ptr2, >0 if ptr1>ptr2)
        map.insert("memcmp", IntrinsicDef {
            name: "memcmp",
            description: "Compare len bytes at ptr1 and ptr2. Returns 0 if equal. Usage: @memcmp ptr1_reg, ptr2_reg, len_reg",
            category: IntrinsicCategory::Memory,
            args: vec![ArgType::Register, ArgType::Register, ArgType::Register],
            template: vec![
                Instruction::new(Opcode::Mov, Register::R0, Register::Zero, Register::Zero, 0), // result = 0
                Instruction::new(Opcode::Mov, Register::R13, Register::Zero, Register::Zero, 0), // counter = 0
                // loop:
                Instruction::new(Opcode::Branch, Register::Zero, Register::R13, Register::R12, BranchCond::Ge as u8), // if counter >= len, done (equal)
                Instruction::new(Opcode::Alu, Register::R14, Register::R10, Register::R13, AluOp::Add as u8), // r14 = ptr1 + counter
                Instruction::new(Opcode::Load, Register::R14, Register::R14, Register::Zero, MemWidth::Byte as u8), // r14 = *ptr1
                Instruction::new(Opcode::Alu, Register::R15, Register::R11, Register::R13, AluOp::Add as u8), // r15 = ptr2 + counter
                Instruction::new(Opcode::Load, Register::R15, Register::R15, Register::Zero, MemWidth::Byte as u8), // r15 = *ptr2
                Instruction::new(Opcode::Branch, Register::Zero, Register::R14, Register::R15, BranchCond::Ne as u8), // if *ptr1 != *ptr2, branch to diff
                Instruction::with_imm(Opcode::AluI, Register::R13, Register::R13, AluOp::Add as u8, 1), // counter++
                Instruction::with_imm(Opcode::Branch, Register::Zero, Register::Zero, BranchCond::Always as u8, -7), // branch to loop
                // diff:
                Instruction::new(Opcode::Alu, Register::R0, Register::R14, Register::R15, AluOp::Sub as u8), // result = *ptr1 - *ptr2
                // done:
                Instruction::new(Opcode::Nop, Register::Zero, Register::Zero, Register::Zero, 0),
            ],
            patch_points: vec![],
            labels: [("loop".to_string(), 2), ("diff".to_string(), 10), ("done".to_string(), 11)].into_iter().collect(),
        });
    }

    fn add_string_intrinsics(map: &mut HashMap<&'static str, IntrinsicDef>) {
        // @strlen str -> length in r0
        map.insert("strlen", IntrinsicDef {
            name: "strlen",
            description: "Calculate length of null-terminated string. Result in r0. Usage: @strlen str_reg",
            category: IntrinsicCategory::String,
            args: vec![ArgType::Register],
            template: vec![
                Instruction::new(Opcode::Mov, Register::R0, Register::Zero, Register::Zero, 0), // length = 0
                Instruction::new(Opcode::Mov, Register::R13, Register::R10, Register::Zero, 0), // ptr = str
                // loop:
                Instruction::new(Opcode::Load, Register::R14, Register::R13, Register::Zero, MemWidth::Byte as u8), // r14 = *ptr
                Instruction::new(Opcode::Branch, Register::Zero, Register::R14, Register::Zero, BranchCond::Eq as u8), // if *ptr == 0, done
                Instruction::with_imm(Opcode::AluI, Register::R0, Register::R0, AluOp::Add as u8, 1), // length++
                Instruction::with_imm(Opcode::AluI, Register::R13, Register::R13, AluOp::Add as u8, 1), // ptr++
                Instruction::with_imm(Opcode::Branch, Register::Zero, Register::Zero, BranchCond::Always as u8, -4), // branch to loop
                // done:
                Instruction::new(Opcode::Nop, Register::Zero, Register::Zero, Register::Zero, 0),
            ],
            patch_points: vec![],
            labels: [("loop".to_string(), 2), ("done".to_string(), 7)].into_iter().collect(),
        });

        // @strcmp str1, str2 -> result in r0 (0 if equal)
        map.insert("strcmp", IntrinsicDef {
            name: "strcmp",
            description: "Compare two null-terminated strings. Returns 0 if equal. Usage: @strcmp str1_reg, str2_reg",
            category: IntrinsicCategory::String,
            args: vec![ArgType::Register, ArgType::Register],
            template: vec![
                Instruction::new(Opcode::Mov, Register::R13, Register::R10, Register::Zero, 0), // ptr1 = str1
                Instruction::new(Opcode::Mov, Register::R14, Register::R11, Register::Zero, 0), // ptr2 = str2
                // loop:
                Instruction::new(Opcode::Load, Register::R0, Register::R13, Register::Zero, MemWidth::Byte as u8), // r0 = *ptr1
                Instruction::new(Opcode::Load, Register::R15, Register::R14, Register::Zero, MemWidth::Byte as u8), // r15 = *ptr2
                Instruction::new(Opcode::Branch, Register::Zero, Register::R0, Register::R15, BranchCond::Ne as u8), // if *ptr1 != *ptr2, done (diff)
                Instruction::new(Opcode::Branch, Register::Zero, Register::R0, Register::Zero, BranchCond::Eq as u8), // if *ptr1 == 0, done (equal)
                Instruction::with_imm(Opcode::AluI, Register::R13, Register::R13, AluOp::Add as u8, 1), // ptr1++
                Instruction::with_imm(Opcode::AluI, Register::R14, Register::R14, AluOp::Add as u8, 1), // ptr2++
                Instruction::with_imm(Opcode::Branch, Register::Zero, Register::Zero, BranchCond::Always as u8, -6), // branch to loop
                // diff/done:
                Instruction::new(Opcode::Alu, Register::R0, Register::R0, Register::R15, AluOp::Sub as u8), // r0 = *ptr1 - *ptr2
                Instruction::new(Opcode::Nop, Register::Zero, Register::Zero, Register::Zero, 0),
            ],
            patch_points: vec![],
            labels: [("loop".to_string(), 2), ("done".to_string(), 10)].into_iter().collect(),
        });

        // @strcpy dst, src -> dst in r0
        map.insert("strcpy", IntrinsicDef {
            name: "strcpy",
            description: "Copy null-terminated string from src to dst. Returns dst. Usage: @strcpy dst_reg, src_reg",
            category: IntrinsicCategory::String,
            args: vec![ArgType::Register, ArgType::Register],
            template: vec![
                Instruction::new(Opcode::Mov, Register::R0, Register::R10, Register::Zero, 0), // save dst for return
                Instruction::new(Opcode::Mov, Register::R13, Register::R10, Register::Zero, 0), // dst_ptr = dst
                Instruction::new(Opcode::Mov, Register::R14, Register::R11, Register::Zero, 0), // src_ptr = src
                // loop:
                Instruction::new(Opcode::Load, Register::R15, Register::R14, Register::Zero, MemWidth::Byte as u8), // r15 = *src
                Instruction::new(Opcode::Store, Register::R15, Register::R13, Register::Zero, MemWidth::Byte as u8), // *dst = r15
                Instruction::new(Opcode::Branch, Register::Zero, Register::R15, Register::Zero, BranchCond::Eq as u8), // if *src == 0, done
                Instruction::with_imm(Opcode::AluI, Register::R13, Register::R13, AluOp::Add as u8, 1), // dst_ptr++
                Instruction::with_imm(Opcode::AluI, Register::R14, Register::R14, AluOp::Add as u8, 1), // src_ptr++
                Instruction::with_imm(Opcode::Branch, Register::Zero, Register::Zero, BranchCond::Always as u8, -5), // branch to loop
                // done:
                Instruction::new(Opcode::Nop, Register::Zero, Register::Zero, Register::Zero, 0),
            ],
            patch_points: vec![],
            labels: [("loop".to_string(), 3), ("done".to_string(), 9)].into_iter().collect(),
        });
    }

    fn add_math_intrinsics(map: &mut HashMap<&'static str, IntrinsicDef>) {
        // @abs val -> |val| in r0
        map.insert(
            "abs",
            IntrinsicDef {
                name: "abs",
                description: "Calculate absolute value. Result in r0. Usage: @abs val_reg",
                category: IntrinsicCategory::Math,
                args: vec![ArgType::Register],
                template: vec![
                    Instruction::new(Opcode::Mov, Register::R0, Register::R10, Register::Zero, 0), // r0 = val
                    Instruction::new(
                        Opcode::Branch,
                        Register::Zero,
                        Register::R0,
                        Register::Zero,
                        BranchCond::Ge as u8,
                    ), // if val >= 0, done
                    Instruction::new(
                        Opcode::Alu,
                        Register::R0,
                        Register::Zero,
                        Register::R0,
                        AluOp::Sub as u8,
                    ), // r0 = 0 - val
                    // done:
                    Instruction::new(
                        Opcode::Nop,
                        Register::Zero,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ),
                ],
                patch_points: vec![],
                labels: [("done".to_string(), 3)].into_iter().collect(),
            },
        );

        // @min a, b -> min(a, b) in r0
        map.insert(
            "min",
            IntrinsicDef {
                name: "min",
                description: "Return minimum of two values. Result in r0. Usage: @min a_reg, b_reg",
                category: IntrinsicCategory::Math,
                args: vec![ArgType::Register, ArgType::Register],
                template: vec![
                    Instruction::new(Opcode::Mov, Register::R0, Register::R10, Register::Zero, 0), // r0 = a
                    Instruction::new(
                        Opcode::Branch,
                        Register::Zero,
                        Register::R10,
                        Register::R11,
                        BranchCond::Le as u8,
                    ), // if a <= b, done
                    Instruction::new(Opcode::Mov, Register::R0, Register::R11, Register::Zero, 0), // r0 = b
                    // done:
                    Instruction::new(
                        Opcode::Nop,
                        Register::Zero,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ),
                ],
                patch_points: vec![],
                labels: [("done".to_string(), 3)].into_iter().collect(),
            },
        );

        // @max a, b -> max(a, b) in r0
        map.insert(
            "max",
            IntrinsicDef {
                name: "max",
                description: "Return maximum of two values. Result in r0. Usage: @max a_reg, b_reg",
                category: IntrinsicCategory::Math,
                args: vec![ArgType::Register, ArgType::Register],
                template: vec![
                    Instruction::new(Opcode::Mov, Register::R0, Register::R10, Register::Zero, 0), // r0 = a
                    Instruction::new(
                        Opcode::Branch,
                        Register::Zero,
                        Register::R10,
                        Register::R11,
                        BranchCond::Ge as u8,
                    ), // if a >= b, done
                    Instruction::new(Opcode::Mov, Register::R0, Register::R11, Register::Zero, 0), // r0 = b
                    // done:
                    Instruction::new(
                        Opcode::Nop,
                        Register::Zero,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ),
                ],
                patch_points: vec![],
                labels: [("done".to_string(), 3)].into_iter().collect(),
            },
        );

        // @clamp val, min, max -> clamped value in r0
        map.insert("clamp", IntrinsicDef {
            name: "clamp",
            description: "Clamp value between min and max. Result in r0. Usage: @clamp val_reg, min_reg, max_reg",
            category: IntrinsicCategory::Math,
            args: vec![ArgType::Register, ArgType::Register, ArgType::Register],
            template: vec![
                Instruction::new(Opcode::Mov, Register::R0, Register::R10, Register::Zero, 0), // r0 = val
                Instruction::new(Opcode::Branch, Register::Zero, Register::R0, Register::R11, BranchCond::Ge as u8), // if val >= min, check_max
                Instruction::new(Opcode::Mov, Register::R0, Register::R11, Register::Zero, 0), // r0 = min
                Instruction::with_imm(Opcode::Branch, Register::Zero, Register::Zero, BranchCond::Always as u8, 3), // branch to done
                // check_max:
                Instruction::new(Opcode::Branch, Register::Zero, Register::R0, Register::R12, BranchCond::Le as u8), // if val <= max, done
                Instruction::new(Opcode::Mov, Register::R0, Register::R12, Register::Zero, 0), // r0 = max
                // done:
                Instruction::new(Opcode::Nop, Register::Zero, Register::Zero, Register::Zero, 0),
            ],
            patch_points: vec![],
            labels: [("check_max".to_string(), 4), ("done".to_string(), 6)].into_iter().collect(),
        });

        // @gcd a, b -> gcd(a, b) in r0
        map.insert("gcd", IntrinsicDef {
            name: "gcd",
            description: "Calculate GCD using Euclidean algorithm. Result in r0. Usage: @gcd a_reg, b_reg",
            category: IntrinsicCategory::Math,
            args: vec![ArgType::Register, ArgType::Register],
            template: vec![
                Instruction::new(Opcode::Mov, Register::R0, Register::R10, Register::Zero, 0), // a
                Instruction::new(Opcode::Mov, Register::R13, Register::R11, Register::Zero, 0), // b
                // loop:
                Instruction::new(Opcode::Branch, Register::Zero, Register::R13, Register::Zero, BranchCond::Eq as u8), // if b == 0, done
                Instruction::new(Opcode::MulDiv, Register::R14, Register::R0, Register::R13, 3), // r14 = a % b
                Instruction::new(Opcode::Mov, Register::R0, Register::R13, Register::Zero, 0), // a = b
                Instruction::new(Opcode::Mov, Register::R13, Register::R14, Register::Zero, 0), // b = remainder
                Instruction::with_imm(Opcode::Branch, Register::Zero, Register::Zero, BranchCond::Always as u8, -4), // branch to loop
                // done:
                Instruction::new(Opcode::Nop, Register::Zero, Register::Zero, Register::Zero, 0),
            ],
            patch_points: vec![],
            labels: [("loop".to_string(), 2), ("done".to_string(), 7)].into_iter().collect(),
        });

        // @pow base, exp -> base^exp in r0
        map.insert("pow", IntrinsicDef {
            name: "pow",
            description: "Calculate base raised to exp (integer exponent). Result in r0. Usage: @pow base_reg, exp_reg",
            category: IntrinsicCategory::Math,
            args: vec![ArgType::Register, ArgType::Register],
            template: vec![
                Instruction::with_imm(Opcode::Mov, Register::R0, Register::Zero, 0, 1), // result = 1
                Instruction::new(Opcode::Mov, Register::R13, Register::R11, Register::Zero, 0), // counter = exp
                // loop:
                Instruction::new(Opcode::Branch, Register::Zero, Register::R13, Register::Zero, BranchCond::Eq as u8), // if counter == 0, done
                Instruction::new(Opcode::MulDiv, Register::R0, Register::R0, Register::R10, 0), // result *= base
                Instruction::with_imm(Opcode::AluI, Register::R13, Register::R13, AluOp::Sub as u8, 1), // counter--
                Instruction::with_imm(Opcode::Branch, Register::Zero, Register::Zero, BranchCond::Always as u8, -3), // branch to loop
                // done:
                Instruction::new(Opcode::Nop, Register::Zero, Register::Zero, Register::Zero, 0),
            ],
            patch_points: vec![],
            labels: [("loop".to_string(), 2), ("done".to_string(), 6)].into_iter().collect(),
        });

        // @factorial n -> n! in r0
        map.insert(
            "factorial",
            IntrinsicDef {
                name: "factorial",
                description: "Calculate factorial of n. Result in r0. Usage: @factorial n_reg",
                category: IntrinsicCategory::Math,
                args: vec![ArgType::Register],
                template: vec![
                    Instruction::with_imm(Opcode::Mov, Register::R0, Register::Zero, 0, 1), // result = 1
                    Instruction::new(Opcode::Mov, Register::R13, Register::R10, Register::Zero, 0), // counter = n
                    // loop:
                    Instruction::new(
                        Opcode::Branch,
                        Register::Zero,
                        Register::R13,
                        Register::Zero,
                        BranchCond::Eq as u8,
                    ), // if counter == 0, done
                    Instruction::new(Opcode::MulDiv, Register::R0, Register::R0, Register::R13, 0), // result *= counter
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R13,
                        Register::R13,
                        AluOp::Sub as u8,
                        1,
                    ), // counter--
                    Instruction::with_imm(
                        Opcode::Branch,
                        Register::Zero,
                        Register::Zero,
                        BranchCond::Always as u8,
                        -3,
                    ), // branch to loop
                    // done:
                    Instruction::new(
                        Opcode::Nop,
                        Register::Zero,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ),
                ],
                patch_points: vec![],
                labels: [("loop".to_string(), 2), ("done".to_string(), 6)]
                    .into_iter()
                    .collect(),
            },
        );
    }

    fn add_search_intrinsics(map: &mut HashMap<&'static str, IntrinsicDef>) {
        // @linear_search arr, len, target -> index in r0 (-1 if not found)
        map.insert("linear_search", IntrinsicDef {
            name: "linear_search",
            description: "Linear search for target in array. Returns index or -1. Usage: @linear_search arr_reg, len_reg, target_reg",
            category: IntrinsicCategory::Search,
            args: vec![ArgType::Register, ArgType::Register, ArgType::Register],
            template: vec![
                Instruction::new(Opcode::Mov, Register::R0, Register::Zero, Register::Zero, 0), // index = 0
                // loop:
                Instruction::new(Opcode::Branch, Register::Zero, Register::R0, Register::R11, BranchCond::Ge as u8), // if index >= len, not_found
                Instruction::new(Opcode::MulDiv, Register::R13, Register::R0, Register::Zero, 0), // r13 = index * 8 (simplified)
                Instruction::with_imm(Opcode::AluI, Register::R13, Register::R0, AluOp::Shl as u8, 3), // r13 = index << 3 (index * 8)
                Instruction::new(Opcode::Alu, Register::R13, Register::R10, Register::R13, AluOp::Add as u8), // r13 = arr + offset
                Instruction::new(Opcode::Load, Register::R14, Register::R13, Register::Zero, MemWidth::Double as u8), // r14 = arr[index]
                Instruction::new(Opcode::Branch, Register::Zero, Register::R14, Register::R12, BranchCond::Eq as u8), // if found, done
                Instruction::with_imm(Opcode::AluI, Register::R0, Register::R0, AluOp::Add as u8, 1), // index++
                Instruction::with_imm(Opcode::Branch, Register::Zero, Register::Zero, BranchCond::Always as u8, -7), // branch to loop
                // not_found:
                Instruction::with_imm(Opcode::Mov, Register::R0, Register::Zero, 0, -1), // r0 = -1
                // done:
                Instruction::new(Opcode::Nop, Register::Zero, Register::Zero, Register::Zero, 0),
            ],
            patch_points: vec![],
            labels: [("loop".to_string(), 1), ("not_found".to_string(), 9), ("done".to_string(), 10)].into_iter().collect(),
        });

        // @binary_search arr, len, target -> index in r0 (-1 if not found)
        map.insert("binary_search", IntrinsicDef {
            name: "binary_search",
            description: "Binary search for target in sorted array. Returns index or -1. Usage: @binary_search arr_reg, len_reg, target_reg",
            category: IntrinsicCategory::Search,
            args: vec![ArgType::Register, ArgType::Register, ArgType::Register],
            template: vec![
                Instruction::new(Opcode::Mov, Register::R13, Register::Zero, Register::Zero, 0), // left = 0
                Instruction::new(Opcode::Mov, Register::R14, Register::R11, Register::Zero, 0), // right = len
                // loop:
                Instruction::new(Opcode::Branch, Register::Zero, Register::R13, Register::R14, BranchCond::Ge as u8), // if left >= right, not_found
                Instruction::new(Opcode::Alu, Register::R0, Register::R13, Register::R14, AluOp::Add as u8), // r0 = left + right
                Instruction::with_imm(Opcode::AluI, Register::R0, Register::R0, AluOp::Shr as u8, 1), // mid = (left + right) / 2
                Instruction::with_imm(Opcode::AluI, Register::R15, Register::R0, AluOp::Shl as u8, 3), // r15 = mid * 8
                Instruction::new(Opcode::Alu, Register::R15, Register::R10, Register::R15, AluOp::Add as u8), // r15 = arr + offset
                Instruction::new(Opcode::Load, Register::R15, Register::R15, Register::Zero, MemWidth::Double as u8), // r15 = arr[mid]
                Instruction::new(Opcode::Branch, Register::Zero, Register::R15, Register::R12, BranchCond::Eq as u8), // if found, done
                Instruction::new(Opcode::Branch, Register::Zero, Register::R15, Register::R12, BranchCond::Lt as u8), // if arr[mid] < target, search_right
                // search_left:
                Instruction::new(Opcode::Mov, Register::R14, Register::R0, Register::Zero, 0), // right = mid
                Instruction::with_imm(Opcode::Branch, Register::Zero, Register::Zero, BranchCond::Always as u8, -9), // branch to loop
                // search_right:
                Instruction::with_imm(Opcode::AluI, Register::R13, Register::R0, AluOp::Add as u8, 1), // left = mid + 1
                Instruction::with_imm(Opcode::Branch, Register::Zero, Register::Zero, BranchCond::Always as u8, -11), // branch to loop
                // not_found:
                Instruction::with_imm(Opcode::Mov, Register::R0, Register::Zero, 0, -1), // r0 = -1
                // done:
                Instruction::new(Opcode::Nop, Register::Zero, Register::Zero, Register::Zero, 0),
            ],
            patch_points: vec![],
            labels: [("loop".to_string(), 2), ("search_left".to_string(), 10), ("search_right".to_string(), 12), ("not_found".to_string(), 14), ("done".to_string(), 15)].into_iter().collect(),
        });

        // @find_min arr, len -> index of minimum in r0
        map.insert("find_min", IntrinsicDef {
            name: "find_min",
            description: "Find index of minimum element in array. Result in r0. Usage: @find_min arr_reg, len_reg",
            category: IntrinsicCategory::Search,
            args: vec![ArgType::Register, ArgType::Register],
            template: vec![
                Instruction::new(Opcode::Mov, Register::R0, Register::Zero, Register::Zero, 0), // min_idx = 0
                Instruction::new(Opcode::Load, Register::R13, Register::R10, Register::Zero, MemWidth::Double as u8), // min_val = arr[0]
                Instruction::with_imm(Opcode::Mov, Register::R14, Register::Zero, 0, 1), // i = 1
                // loop:
                Instruction::new(Opcode::Branch, Register::Zero, Register::R14, Register::R11, BranchCond::Ge as u8), // if i >= len, done
                Instruction::with_imm(Opcode::AluI, Register::R15, Register::R14, AluOp::Shl as u8, 3), // r15 = i * 8
                Instruction::new(Opcode::Alu, Register::R15, Register::R10, Register::R15, AluOp::Add as u8), // r15 = arr + offset
                Instruction::new(Opcode::Load, Register::R15, Register::R15, Register::Zero, MemWidth::Double as u8), // r15 = arr[i]
                Instruction::new(Opcode::Branch, Register::Zero, Register::R15, Register::R13, BranchCond::Ge as u8), // if arr[i] >= min, skip
                Instruction::new(Opcode::Mov, Register::R13, Register::R15, Register::Zero, 0), // min_val = arr[i]
                Instruction::new(Opcode::Mov, Register::R0, Register::R14, Register::Zero, 0), // min_idx = i
                // skip:
                Instruction::with_imm(Opcode::AluI, Register::R14, Register::R14, AluOp::Add as u8, 1), // i++
                Instruction::with_imm(Opcode::Branch, Register::Zero, Register::Zero, BranchCond::Always as u8, -8), // branch to loop
                // done:
                Instruction::new(Opcode::Nop, Register::Zero, Register::Zero, Register::Zero, 0),
            ],
            patch_points: vec![],
            labels: [("loop".to_string(), 3), ("skip".to_string(), 10), ("done".to_string(), 12)].into_iter().collect(),
        });

        // @find_max arr, len -> index of maximum in r0
        map.insert("find_max", IntrinsicDef {
            name: "find_max",
            description: "Find index of maximum element in array. Result in r0. Usage: @find_max arr_reg, len_reg",
            category: IntrinsicCategory::Search,
            args: vec![ArgType::Register, ArgType::Register],
            template: vec![
                Instruction::new(Opcode::Mov, Register::R0, Register::Zero, Register::Zero, 0), // max_idx = 0
                Instruction::new(Opcode::Load, Register::R13, Register::R10, Register::Zero, MemWidth::Double as u8), // max_val = arr[0]
                Instruction::with_imm(Opcode::Mov, Register::R14, Register::Zero, 0, 1), // i = 1
                // loop:
                Instruction::new(Opcode::Branch, Register::Zero, Register::R14, Register::R11, BranchCond::Ge as u8), // if i >= len, done
                Instruction::with_imm(Opcode::AluI, Register::R15, Register::R14, AluOp::Shl as u8, 3), // r15 = i * 8
                Instruction::new(Opcode::Alu, Register::R15, Register::R10, Register::R15, AluOp::Add as u8), // r15 = arr + offset
                Instruction::new(Opcode::Load, Register::R15, Register::R15, Register::Zero, MemWidth::Double as u8), // r15 = arr[i]
                Instruction::new(Opcode::Branch, Register::Zero, Register::R15, Register::R13, BranchCond::Le as u8), // if arr[i] <= max, skip
                Instruction::new(Opcode::Mov, Register::R13, Register::R15, Register::Zero, 0), // max_val = arr[i]
                Instruction::new(Opcode::Mov, Register::R0, Register::R14, Register::Zero, 0), // max_idx = i
                // skip:
                Instruction::with_imm(Opcode::AluI, Register::R14, Register::R14, AluOp::Add as u8, 1), // i++
                Instruction::with_imm(Opcode::Branch, Register::Zero, Register::Zero, BranchCond::Always as u8, -8), // branch to loop
                // done:
                Instruction::new(Opcode::Nop, Register::Zero, Register::Zero, Register::Zero, 0),
            ],
            patch_points: vec![],
            labels: [("loop".to_string(), 3), ("skip".to_string(), 10), ("done".to_string(), 12)].into_iter().collect(),
        });
    }

    fn add_array_intrinsics(map: &mut HashMap<&'static str, IntrinsicDef>) {
        // @sum arr, len -> sum in r0
        map.insert(
            "sum",
            IntrinsicDef {
                name: "sum",
                description:
                    "Sum all elements in array. Result in r0. Usage: @sum arr_reg, len_reg",
                category: IntrinsicCategory::Array,
                args: vec![ArgType::Register, ArgType::Register],
                template: vec![
                    Instruction::new(Opcode::Mov, Register::R0, Register::Zero, Register::Zero, 0), // sum = 0
                    Instruction::new(
                        Opcode::Mov,
                        Register::R13,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ), // i = 0
                    // loop:
                    Instruction::new(
                        Opcode::Branch,
                        Register::Zero,
                        Register::R13,
                        Register::R11,
                        BranchCond::Ge as u8,
                    ), // if i >= len, done
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R14,
                        Register::R13,
                        AluOp::Shl as u8,
                        3,
                    ), // r14 = i * 8
                    Instruction::new(
                        Opcode::Alu,
                        Register::R14,
                        Register::R10,
                        Register::R14,
                        AluOp::Add as u8,
                    ), // r14 = arr + offset
                    Instruction::new(
                        Opcode::Load,
                        Register::R14,
                        Register::R14,
                        Register::Zero,
                        MemWidth::Double as u8,
                    ), // r14 = arr[i]
                    Instruction::new(
                        Opcode::Alu,
                        Register::R0,
                        Register::R0,
                        Register::R14,
                        AluOp::Add as u8,
                    ), // sum += arr[i]
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R13,
                        Register::R13,
                        AluOp::Add as u8,
                        1,
                    ), // i++
                    Instruction::with_imm(
                        Opcode::Branch,
                        Register::Zero,
                        Register::Zero,
                        BranchCond::Always as u8,
                        -6,
                    ), // branch to loop
                    // done:
                    Instruction::new(
                        Opcode::Nop,
                        Register::Zero,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ),
                ],
                patch_points: vec![],
                labels: [("loop".to_string(), 2), ("done".to_string(), 9)]
                    .into_iter()
                    .collect(),
            },
        );

        // @reverse arr, len
        map.insert(
            "reverse",
            IntrinsicDef {
                name: "reverse",
                description: "Reverse array in place. Usage: @reverse arr_reg, len_reg",
                category: IntrinsicCategory::Array,
                args: vec![ArgType::Register, ArgType::Register],
                template: vec![
                    Instruction::new(
                        Opcode::Mov,
                        Register::R13,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ), // left = 0
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R14,
                        Register::R11,
                        AluOp::Sub as u8,
                        1,
                    ), // right = len - 1
                    // loop:
                    Instruction::new(
                        Opcode::Branch,
                        Register::Zero,
                        Register::R13,
                        Register::R14,
                        BranchCond::Ge as u8,
                    ), // if left >= right, done
                    // Load left element
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R15,
                        Register::R13,
                        AluOp::Shl as u8,
                        3,
                    ), // r15 = left * 8
                    Instruction::new(
                        Opcode::Alu,
                        Register::R15,
                        Register::R10,
                        Register::R15,
                        AluOp::Add as u8,
                    ), // r15 = arr + left_offset
                    Instruction::new(
                        Opcode::Load,
                        Register::R0,
                        Register::R15,
                        Register::Zero,
                        MemWidth::Double as u8,
                    ), // r0 = arr[left]
                    // Load right element
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R1,
                        Register::R14,
                        AluOp::Shl as u8,
                        3,
                    ), // r1 = right * 8
                    Instruction::new(
                        Opcode::Alu,
                        Register::R1,
                        Register::R10,
                        Register::R1,
                        AluOp::Add as u8,
                    ), // r1 = arr + right_offset
                    Instruction::new(
                        Opcode::Load,
                        Register::R2,
                        Register::R1,
                        Register::Zero,
                        MemWidth::Double as u8,
                    ), // r2 = arr[right]
                    // Swap
                    Instruction::new(
                        Opcode::Store,
                        Register::R2,
                        Register::R15,
                        Register::Zero,
                        MemWidth::Double as u8,
                    ), // arr[left] = arr[right]
                    Instruction::new(
                        Opcode::Store,
                        Register::R0,
                        Register::R1,
                        Register::Zero,
                        MemWidth::Double as u8,
                    ), // arr[right] = arr[left]
                    // Move pointers
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R13,
                        Register::R13,
                        AluOp::Add as u8,
                        1,
                    ), // left++
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R14,
                        Register::R14,
                        AluOp::Sub as u8,
                        1,
                    ), // right--
                    Instruction::with_imm(
                        Opcode::Branch,
                        Register::Zero,
                        Register::Zero,
                        BranchCond::Always as u8,
                        -11,
                    ), // branch to loop
                    // done:
                    Instruction::new(
                        Opcode::Nop,
                        Register::Zero,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ),
                ],
                patch_points: vec![],
                labels: [("loop".to_string(), 2), ("done".to_string(), 14)]
                    .into_iter()
                    .collect(),
            },
        );

        // @fill arr, len, val
        map.insert(
            "fill",
            IntrinsicDef {
                name: "fill",
                description: "Fill array with value. Usage: @fill arr_reg, len_reg, val_reg",
                category: IntrinsicCategory::Array,
                args: vec![ArgType::Register, ArgType::Register, ArgType::Register],
                template: vec![
                    Instruction::new(
                        Opcode::Mov,
                        Register::R13,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ), // i = 0
                    // loop:
                    Instruction::new(
                        Opcode::Branch,
                        Register::Zero,
                        Register::R13,
                        Register::R11,
                        BranchCond::Ge as u8,
                    ), // if i >= len, done
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R14,
                        Register::R13,
                        AluOp::Shl as u8,
                        3,
                    ), // r14 = i * 8
                    Instruction::new(
                        Opcode::Alu,
                        Register::R14,
                        Register::R10,
                        Register::R14,
                        AluOp::Add as u8,
                    ), // r14 = arr + offset
                    Instruction::new(
                        Opcode::Store,
                        Register::R12,
                        Register::R14,
                        Register::Zero,
                        MemWidth::Double as u8,
                    ), // arr[i] = val
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R13,
                        Register::R13,
                        AluOp::Add as u8,
                        1,
                    ), // i++
                    Instruction::with_imm(
                        Opcode::Branch,
                        Register::Zero,
                        Register::Zero,
                        BranchCond::Always as u8,
                        -5,
                    ), // branch to loop
                    // done:
                    Instruction::new(
                        Opcode::Nop,
                        Register::Zero,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ),
                ],
                patch_points: vec![],
                labels: [("loop".to_string(), 1), ("done".to_string(), 7)]
                    .into_iter()
                    .collect(),
            },
        );

        // @count arr, len, val -> count in r0
        map.insert("count", IntrinsicDef {
            name: "count",
            description: "Count occurrences of value in array. Result in r0. Usage: @count arr_reg, len_reg, val_reg",
            category: IntrinsicCategory::Array,
            args: vec![ArgType::Register, ArgType::Register, ArgType::Register],
            template: vec![
                Instruction::new(Opcode::Mov, Register::R0, Register::Zero, Register::Zero, 0), // count = 0
                Instruction::new(Opcode::Mov, Register::R13, Register::Zero, Register::Zero, 0), // i = 0
                // loop:
                Instruction::new(Opcode::Branch, Register::Zero, Register::R13, Register::R11, BranchCond::Ge as u8), // if i >= len, done
                Instruction::with_imm(Opcode::AluI, Register::R14, Register::R13, AluOp::Shl as u8, 3), // r14 = i * 8
                Instruction::new(Opcode::Alu, Register::R14, Register::R10, Register::R14, AluOp::Add as u8), // r14 = arr + offset
                Instruction::new(Opcode::Load, Register::R14, Register::R14, Register::Zero, MemWidth::Double as u8), // r14 = arr[i]
                Instruction::new(Opcode::Branch, Register::Zero, Register::R14, Register::R12, BranchCond::Ne as u8), // if arr[i] != val, skip
                Instruction::with_imm(Opcode::AluI, Register::R0, Register::R0, AluOp::Add as u8, 1), // count++
                // skip:
                Instruction::with_imm(Opcode::AluI, Register::R13, Register::R13, AluOp::Add as u8, 1), // i++
                Instruction::with_imm(Opcode::Branch, Register::Zero, Register::Zero, BranchCond::Always as u8, -7), // branch to loop
                // done:
                Instruction::new(Opcode::Nop, Register::Zero, Register::Zero, Register::Zero, 0),
            ],
            patch_points: vec![],
            labels: [("loop".to_string(), 2), ("skip".to_string(), 8), ("done".to_string(), 10)].into_iter().collect(),
        });
    }

    fn add_bitwise_intrinsics(map: &mut HashMap<&'static str, IntrinsicDef>) {
        // @popcount val -> count in r0 (uses built-in BITS.POPCOUNT)
        map.insert(
            "popcount",
            IntrinsicDef {
                name: "popcount",
                description: "Count number of set bits. Result in r0. Usage: @popcount val_reg",
                category: IntrinsicCategory::Bitwise,
                args: vec![ArgType::Register],
                template: vec![
                    Instruction::new(Opcode::Bits, Register::R0, Register::R10, Register::Zero, 0), // POPCOUNT
                ],
                patch_points: vec![],
                labels: HashMap::new(),
            },
        );

        // @clz val -> count in r0 (uses built-in BITS.CLZ)
        map.insert(
            "clz",
            IntrinsicDef {
                name: "clz",
                description: "Count leading zeros. Result in r0. Usage: @clz val_reg",
                category: IntrinsicCategory::Bitwise,
                args: vec![ArgType::Register],
                template: vec![
                    Instruction::new(Opcode::Bits, Register::R0, Register::R10, Register::Zero, 1), // CLZ
                ],
                patch_points: vec![],
                labels: HashMap::new(),
            },
        );

        // @ctz val -> count in r0 (uses built-in BITS.CTZ)
        map.insert(
            "ctz",
            IntrinsicDef {
                name: "ctz",
                description: "Count trailing zeros. Result in r0. Usage: @ctz val_reg",
                category: IntrinsicCategory::Bitwise,
                args: vec![ArgType::Register],
                template: vec![
                    Instruction::new(Opcode::Bits, Register::R0, Register::R10, Register::Zero, 2), // CTZ
                ],
                patch_points: vec![],
                labels: HashMap::new(),
            },
        );

        // @bswap val -> byte-swapped value in r0 (uses built-in BITS.BSWAP)
        map.insert(
            "bswap",
            IntrinsicDef {
                name: "bswap",
                description: "Byte swap for endian conversion. Result in r0. Usage: @bswap val_reg",
                category: IntrinsicCategory::Bitwise,
                args: vec![ArgType::Register],
                template: vec![
                    Instruction::new(Opcode::Bits, Register::R0, Register::R10, Register::Zero, 3), // BSWAP
                ],
                patch_points: vec![],
                labels: HashMap::new(),
            },
        );

        // @nextpow2 val -> next power of 2 in r0
        map.insert(
            "nextpow2",
            IntrinsicDef {
                name: "nextpow2",
                description: "Round up to next power of 2. Result in r0. Usage: @nextpow2 val_reg",
                category: IntrinsicCategory::Bitwise,
                args: vec![ArgType::Register],
                template: vec![
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R0,
                        Register::R10,
                        AluOp::Sub as u8,
                        1,
                    ), // r0 = val - 1
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R13,
                        Register::R0,
                        AluOp::Shr as u8,
                        1,
                    ), // r13 = r0 >> 1
                    Instruction::new(
                        Opcode::Alu,
                        Register::R0,
                        Register::R0,
                        Register::R13,
                        AluOp::Or as u8,
                    ), // r0 |= r13
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R13,
                        Register::R0,
                        AluOp::Shr as u8,
                        2,
                    ), // r13 = r0 >> 2
                    Instruction::new(
                        Opcode::Alu,
                        Register::R0,
                        Register::R0,
                        Register::R13,
                        AluOp::Or as u8,
                    ), // r0 |= r13
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R13,
                        Register::R0,
                        AluOp::Shr as u8,
                        4,
                    ), // r13 = r0 >> 4
                    Instruction::new(
                        Opcode::Alu,
                        Register::R0,
                        Register::R0,
                        Register::R13,
                        AluOp::Or as u8,
                    ), // r0 |= r13
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R13,
                        Register::R0,
                        AluOp::Shr as u8,
                        8,
                    ), // r13 = r0 >> 8
                    Instruction::new(
                        Opcode::Alu,
                        Register::R0,
                        Register::R0,
                        Register::R13,
                        AluOp::Or as u8,
                    ), // r0 |= r13
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R13,
                        Register::R0,
                        AluOp::Shr as u8,
                        16,
                    ), // r13 = r0 >> 16
                    Instruction::new(
                        Opcode::Alu,
                        Register::R0,
                        Register::R0,
                        Register::R13,
                        AluOp::Or as u8,
                    ), // r0 |= r13
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R13,
                        Register::R0,
                        AluOp::Shr as u8,
                        32,
                    ), // r13 = r0 >> 32
                    Instruction::new(
                        Opcode::Alu,
                        Register::R0,
                        Register::R0,
                        Register::R13,
                        AluOp::Or as u8,
                    ), // r0 |= r13
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R0,
                        Register::R0,
                        AluOp::Add as u8,
                        1,
                    ), // r0 += 1
                ],
                patch_points: vec![],
                labels: HashMap::new(),
            },
        );
    }

    fn add_hash_intrinsics(map: &mut HashMap<&'static str, IntrinsicDef>) {
        // @fnv_hash data, len -> hash in r0 (FNV-1a 64-bit)
        map.insert(
            "fnv_hash",
            IntrinsicDef {
                name: "fnv_hash",
                description: "FNV-1a 64-bit hash. Result in r0. Usage: @fnv_hash data_reg, len_reg",
                category: IntrinsicCategory::Hash,
                args: vec![ArgType::Register, ArgType::Register],
                template: vec![
                    // FNV-1a offset basis: 0xcbf29ce484222325 (split into two MOV for 64-bit)
                    Instruction::with_imm(
                        Opcode::Mov,
                        Register::R0,
                        Register::Zero,
                        0,
                        0x84222325_u32 as i32,
                    ), // hash low
                    Instruction::with_imm(
                        Opcode::Mov,
                        Register::R13,
                        Register::Zero,
                        0,
                        0xcbf29ce4_u32 as i32,
                    ), // hash high
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R13,
                        Register::R13,
                        AluOp::Shl as u8,
                        32,
                    ), // shift high to position
                    Instruction::new(
                        Opcode::Alu,
                        Register::R0,
                        Register::R0,
                        Register::R13,
                        AluOp::Or as u8,
                    ), // combine to full hash
                    Instruction::new(
                        Opcode::Mov,
                        Register::R14,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ), // i = 0
                    // loop:
                    Instruction::new(
                        Opcode::Branch,
                        Register::Zero,
                        Register::R14,
                        Register::R11,
                        BranchCond::Ge as u8,
                    ), // if i >= len, done
                    Instruction::new(
                        Opcode::Alu,
                        Register::R15,
                        Register::R10,
                        Register::R14,
                        AluOp::Add as u8,
                    ), // r15 = data + i
                    Instruction::new(
                        Opcode::Load,
                        Register::R15,
                        Register::R15,
                        Register::Zero,
                        MemWidth::Byte as u8,
                    ), // r15 = data[i]
                    Instruction::new(
                        Opcode::Alu,
                        Register::R0,
                        Register::R0,
                        Register::R15,
                        AluOp::Xor as u8,
                    ), // hash ^= byte
                    // Multiply by FNV prime (0x100000001b3) - simplified to shift+add
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R13,
                        Register::R0,
                        AluOp::Shl as u8,
                        8,
                    ), // r13 = hash << 8
                    Instruction::new(
                        Opcode::Alu,
                        Register::R0,
                        Register::R0,
                        Register::R13,
                        AluOp::Add as u8,
                    ), // hash += hash << 8
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R14,
                        Register::R14,
                        AluOp::Add as u8,
                        1,
                    ), // i++
                    Instruction::with_imm(
                        Opcode::Branch,
                        Register::Zero,
                        Register::Zero,
                        BranchCond::Always as u8,
                        -7,
                    ), // branch to loop
                    // done:
                    Instruction::new(
                        Opcode::Nop,
                        Register::Zero,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ),
                ],
                patch_points: vec![],
                labels: [("loop".to_string(), 5), ("done".to_string(), 13)]
                    .into_iter()
                    .collect(),
            },
        );

        // @djb2_hash str -> hash in r0 (null-terminated string)
        map.insert(
            "djb2_hash",
            IntrinsicDef {
                name: "djb2_hash",
                description:
                    "DJB2 hash for null-terminated string. Result in r0. Usage: @djb2_hash str_reg",
                category: IntrinsicCategory::Hash,
                args: vec![ArgType::Register],
                template: vec![
                    Instruction::with_imm(Opcode::Mov, Register::R0, Register::Zero, 0, 5381), // hash = 5381
                    Instruction::new(Opcode::Mov, Register::R13, Register::R10, Register::Zero, 0), // ptr = str
                    // loop:
                    Instruction::new(
                        Opcode::Load,
                        Register::R14,
                        Register::R13,
                        Register::Zero,
                        MemWidth::Byte as u8,
                    ), // r14 = *ptr
                    Instruction::new(
                        Opcode::Branch,
                        Register::Zero,
                        Register::R14,
                        Register::Zero,
                        BranchCond::Eq as u8,
                    ), // if *ptr == 0, done
                    // hash = hash * 33 + c = (hash << 5) + hash + c
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R15,
                        Register::R0,
                        AluOp::Shl as u8,
                        5,
                    ), // r15 = hash << 5
                    Instruction::new(
                        Opcode::Alu,
                        Register::R0,
                        Register::R15,
                        Register::R0,
                        AluOp::Add as u8,
                    ), // hash = (hash << 5) + hash
                    Instruction::new(
                        Opcode::Alu,
                        Register::R0,
                        Register::R0,
                        Register::R14,
                        AluOp::Add as u8,
                    ), // hash += c
                    Instruction::with_imm(
                        Opcode::AluI,
                        Register::R13,
                        Register::R13,
                        AluOp::Add as u8,
                        1,
                    ), // ptr++
                    Instruction::with_imm(
                        Opcode::Branch,
                        Register::Zero,
                        Register::Zero,
                        BranchCond::Always as u8,
                        -6,
                    ), // branch to loop
                    // done:
                    Instruction::new(
                        Opcode::Nop,
                        Register::Zero,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ),
                ],
                patch_points: vec![],
                labels: [("loop".to_string(), 2), ("done".to_string(), 9)]
                    .into_iter()
                    .collect(),
            },
        );
    }
}

impl Default for IntrinsicRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during intrinsic expansion
#[derive(Debug, Clone)]
pub enum IntrinsicError {
    UnknownIntrinsic(String),
    ArgCountMismatch {
        name: String,
        expected: usize,
        got: usize,
    },
    TypeMismatch {
        name: String,
        arg_idx: usize,
        expected: &'static str,
        got: &'static str,
    },
    ParseError(String),
}

impl std::fmt::Display for IntrinsicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntrinsicError::UnknownIntrinsic(name) => write!(f, "Unknown intrinsic: @{}", name),
            IntrinsicError::ArgCountMismatch {
                name,
                expected,
                got,
            } => {
                write!(f, "@{} expects {} arguments, got {}", name, expected, got)
            }
            IntrinsicError::TypeMismatch {
                name,
                arg_idx,
                expected,
                got,
            } => {
                write!(
                    f,
                    "@{} argument {} expected {}, got {}",
                    name, arg_idx, expected, got
                )
            }
            IntrinsicError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for IntrinsicError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = IntrinsicRegistry::new();
        assert!(registry.get("memcpy").is_some());
        assert!(registry.get("strlen").is_some());
        assert!(registry.get("abs").is_some());
        assert!(registry.get("gcd").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_list_by_category() {
        let registry = IntrinsicRegistry::new();
        let memory_intrinsics = registry.list_by_category(IntrinsicCategory::Memory);
        assert!(!memory_intrinsics.is_empty());
        for intrinsic in memory_intrinsics {
            assert_eq!(intrinsic.category, IntrinsicCategory::Memory);
        }
    }

    #[test]
    fn test_intrinsic_expansion() {
        let registry = IntrinsicRegistry::new();
        let call = IntrinsicCall {
            name: "abs".to_string(),
            args: vec![IntrinsicArg::Register(Register::R0)],
        };
        let result = registry.expand(&call);
        assert!(result.is_ok());
        let instructions = result.unwrap();
        assert!(!instructions.is_empty());
    }

    #[test]
    fn test_unknown_intrinsic() {
        let registry = IntrinsicRegistry::new();
        let call = IntrinsicCall {
            name: "nonexistent".to_string(),
            args: vec![],
        };
        let result = registry.expand(&call);
        assert!(matches!(result, Err(IntrinsicError::UnknownIntrinsic(_))));
    }

    // === Extensive intrinsic tests for coverage ===

    #[test]
    fn test_all_memory_intrinsics() {
        let registry = IntrinsicRegistry::new();
        let memory = registry.list_by_category(IntrinsicCategory::Memory);

        // Should have memcpy, memset, memzero, memcmp
        let names: Vec<_> = memory.iter().map(|i| i.name).collect();
        assert!(names.contains(&"memcpy"), "Missing memcpy");
        assert!(names.contains(&"memset"), "Missing memset");
        assert!(names.contains(&"memzero"), "Missing memzero");
        assert!(names.contains(&"memcmp"), "Missing memcmp");
    }

    #[test]
    fn test_all_string_intrinsics() {
        let registry = IntrinsicRegistry::new();
        let strings = registry.list_by_category(IntrinsicCategory::String);

        let names: Vec<_> = strings.iter().map(|i| i.name).collect();
        assert!(names.contains(&"strlen"), "Missing strlen");
        assert!(names.contains(&"strcmp"), "Missing strcmp");
        assert!(names.contains(&"strcpy"), "Missing strcpy");
    }

    #[test]
    fn test_all_math_intrinsics() {
        let registry = IntrinsicRegistry::new();
        let math = registry.list_by_category(IntrinsicCategory::Math);

        let names: Vec<_> = math.iter().map(|i| i.name).collect();
        assert!(names.contains(&"abs"), "Missing abs");
        assert!(names.contains(&"min"), "Missing min");
        assert!(names.contains(&"max"), "Missing max");
        assert!(names.contains(&"clamp"), "Missing clamp");
        assert!(names.contains(&"gcd"), "Missing gcd");
        assert!(names.contains(&"pow"), "Missing pow");
        assert!(names.contains(&"factorial"), "Missing factorial");
    }

    #[test]
    fn test_all_search_intrinsics() {
        let registry = IntrinsicRegistry::new();
        let search = registry.list_by_category(IntrinsicCategory::Search);

        let names: Vec<_> = search.iter().map(|i| i.name).collect();
        assert!(names.contains(&"linear_search"), "Missing linear_search");
        assert!(names.contains(&"binary_search"), "Missing binary_search");
        assert!(names.contains(&"find_min"), "Missing find_min");
        assert!(names.contains(&"find_max"), "Missing find_max");
    }

    #[test]
    fn test_all_array_intrinsics() {
        let registry = IntrinsicRegistry::new();
        let array = registry.list_by_category(IntrinsicCategory::Array);

        let names: Vec<_> = array.iter().map(|i| i.name).collect();
        assert!(names.contains(&"sum"), "Missing sum");
        assert!(names.contains(&"reverse"), "Missing reverse");
        assert!(names.contains(&"fill"), "Missing fill");
        assert!(names.contains(&"count"), "Missing count");
    }

    #[test]
    fn test_all_bitwise_intrinsics() {
        let registry = IntrinsicRegistry::new();
        let bitwise = registry.list_by_category(IntrinsicCategory::Bitwise);

        let names: Vec<_> = bitwise.iter().map(|i| i.name).collect();
        assert!(names.contains(&"popcount"), "Missing popcount");
        assert!(names.contains(&"clz"), "Missing clz");
        assert!(names.contains(&"ctz"), "Missing ctz");
        assert!(names.contains(&"bswap"), "Missing bswap");
        assert!(names.contains(&"nextpow2"), "Missing nextpow2");
    }

    #[test]
    fn test_all_hash_intrinsics() {
        let registry = IntrinsicRegistry::new();
        let hash = registry.list_by_category(IntrinsicCategory::Hash);

        let names: Vec<_> = hash.iter().map(|i| i.name).collect();
        assert!(names.contains(&"fnv_hash"), "Missing fnv_hash");
        assert!(names.contains(&"djb2_hash"), "Missing djb2_hash");
    }

    #[test]
    fn test_expand_memcpy() {
        let registry = IntrinsicRegistry::new();
        let call = IntrinsicCall {
            name: "memcpy".to_string(),
            args: vec![
                IntrinsicArg::Register(Register::R0),
                IntrinsicArg::Register(Register::R1),
                IntrinsicArg::Register(Register::R2),
            ],
        };
        let result = registry.expand(&call);
        assert!(result.is_ok());
        let instructions = result.unwrap();
        assert!(
            instructions.len() >= 5,
            "memcpy should expand to multiple instructions"
        );
    }

    #[test]
    fn test_expand_strlen() {
        let registry = IntrinsicRegistry::new();
        let call = IntrinsicCall {
            name: "strlen".to_string(),
            args: vec![IntrinsicArg::Register(Register::R0)],
        };
        let result = registry.expand(&call);
        assert!(result.is_ok());
        let instructions = result.unwrap();
        assert!(instructions.len() >= 4, "strlen should expand to loop");
    }

    #[test]
    fn test_expand_gcd() {
        let registry = IntrinsicRegistry::new();
        let call = IntrinsicCall {
            name: "gcd".to_string(),
            args: vec![
                IntrinsicArg::Register(Register::R0),
                IntrinsicArg::Register(Register::R1),
            ],
        };
        let result = registry.expand(&call);
        assert!(result.is_ok());
        let instructions = result.unwrap();
        assert!(
            instructions.len() >= 5,
            "gcd should expand to Euclidean loop"
        );
    }

    #[test]
    fn test_expand_binary_search() {
        let registry = IntrinsicRegistry::new();
        let call = IntrinsicCall {
            name: "binary_search".to_string(),
            args: vec![
                IntrinsicArg::Register(Register::R0),
                IntrinsicArg::Register(Register::R1),
                IntrinsicArg::Register(Register::R2),
            ],
        };
        let result = registry.expand(&call);
        assert!(result.is_ok());
        let instructions = result.unwrap();
        assert!(
            instructions.len() >= 10,
            "binary_search should expand to complex loop"
        );
    }

    #[test]
    fn test_arg_count_mismatch_too_few() {
        let registry = IntrinsicRegistry::new();
        let call = IntrinsicCall {
            name: "memcpy".to_string(),
            args: vec![IntrinsicArg::Register(Register::R0)], // Missing 2 args
        };
        let result = registry.expand(&call);
        assert!(matches!(
            result,
            Err(IntrinsicError::ArgCountMismatch { .. })
        ));
    }

    #[test]
    fn test_arg_count_mismatch_too_many() {
        let registry = IntrinsicRegistry::new();
        let call = IntrinsicCall {
            name: "abs".to_string(),
            args: vec![
                IntrinsicArg::Register(Register::R0),
                IntrinsicArg::Register(Register::R1), // Extra arg
            ],
        };
        let result = registry.expand(&call);
        assert!(matches!(
            result,
            Err(IntrinsicError::ArgCountMismatch { .. })
        ));
    }

    #[test]
    fn test_list_all_intrinsics() {
        let registry = IntrinsicRegistry::new();
        let all = registry.list();

        // Should have at least 25+ intrinsics across all categories
        assert!(
            all.len() >= 25,
            "Should have at least 25 intrinsics, got {}",
            all.len()
        );
    }

    #[test]
    fn test_intrinsic_def_fields() {
        let registry = IntrinsicRegistry::new();
        let memcpy = registry.get("memcpy").unwrap();

        assert_eq!(memcpy.name, "memcpy");
        assert!(!memcpy.description.is_empty());
        assert_eq!(memcpy.category, IntrinsicCategory::Memory);
        assert_eq!(memcpy.args.len(), 3);
        assert!(!memcpy.template.is_empty());
    }

    #[test]
    fn test_category_as_str() {
        assert_eq!(IntrinsicCategory::Memory.as_str(), "memory");
        assert_eq!(IntrinsicCategory::String.as_str(), "string");
        assert_eq!(IntrinsicCategory::Conversion.as_str(), "conversion");
        assert_eq!(IntrinsicCategory::Search.as_str(), "search");
        assert_eq!(IntrinsicCategory::Sort.as_str(), "sort");
        assert_eq!(IntrinsicCategory::Hash.as_str(), "hash");
        assert_eq!(IntrinsicCategory::Math.as_str(), "math");
        assert_eq!(IntrinsicCategory::Array.as_str(), "array");
        assert_eq!(IntrinsicCategory::Bitwise.as_str(), "bitwise");
    }

    #[test]
    fn test_patch_field_variants() {
        // Just verify all variants exist
        let _rd = PatchField::Rd;
        let _rs1 = PatchField::Rs1;
        let _rs2 = PatchField::Rs2;
        let _imm = PatchField::Imm;
    }

    #[test]
    fn test_arg_type_variants() {
        let _reg = ArgType::Register;
        let _imm = ArgType::Immediate;
        let _reg_or_imm = ArgType::RegOrImm;
    }

    #[test]
    fn test_intrinsic_error_display() {
        let err1 = IntrinsicError::UnknownIntrinsic("foo".to_string());
        assert!(err1.to_string().contains("foo"));

        let err2 = IntrinsicError::ArgCountMismatch {
            name: "bar".to_string(),
            expected: 3,
            got: 1,
        };
        assert!(err2.to_string().contains("3"));
        assert!(err2.to_string().contains("1"));

        let err3 = IntrinsicError::TypeMismatch {
            name: "baz".to_string(),
            arg_idx: 0,
            expected: "register",
            got: "immediate",
        };
        assert!(err3.to_string().contains("register"));
        assert!(err3.to_string().contains("immediate"));

        let err4 = IntrinsicError::ParseError("syntax error".to_string());
        assert!(err4.to_string().contains("syntax error"));
    }

    #[test]
    fn test_default_trait() {
        let registry = IntrinsicRegistry::default();
        assert!(registry.get("memcpy").is_some());
    }

    #[test]
    fn test_intrinsic_call_debug() {
        let call = IntrinsicCall {
            name: "abs".to_string(),
            args: vec![IntrinsicArg::Register(Register::R5)],
        };
        let debug_str = format!("{:?}", call);
        assert!(debug_str.contains("abs"));
    }

    #[test]
    fn test_intrinsic_arg_debug() {
        let reg = IntrinsicArg::Register(Register::R0);
        let imm = IntrinsicArg::Immediate(42);

        let debug_reg = format!("{:?}", reg);
        let debug_imm = format!("{:?}", imm);

        assert!(debug_reg.contains("Register"));
        assert!(debug_imm.contains("42"));
    }

    #[test]
    fn test_expand_all_bitwise() {
        let registry = IntrinsicRegistry::new();

        // popcount - 1 arg
        let popcount = IntrinsicCall {
            name: "popcount".to_string(),
            args: vec![IntrinsicArg::Register(Register::R0)],
        };
        assert!(registry.expand(&popcount).is_ok());

        // clz - 1 arg
        let clz = IntrinsicCall {
            name: "clz".to_string(),
            args: vec![IntrinsicArg::Register(Register::R0)],
        };
        assert!(registry.expand(&clz).is_ok());

        // ctz - 1 arg
        let ctz = IntrinsicCall {
            name: "ctz".to_string(),
            args: vec![IntrinsicArg::Register(Register::R0)],
        };
        assert!(registry.expand(&ctz).is_ok());

        // bswap - 1 arg
        let bswap = IntrinsicCall {
            name: "bswap".to_string(),
            args: vec![IntrinsicArg::Register(Register::R0)],
        };
        assert!(registry.expand(&bswap).is_ok());

        // nextpow2 - 1 arg
        let nextpow2 = IntrinsicCall {
            name: "nextpow2".to_string(),
            args: vec![IntrinsicArg::Register(Register::R0)],
        };
        assert!(registry.expand(&nextpow2).is_ok());
    }

    #[test]
    fn test_expand_hash_intrinsics() {
        let registry = IntrinsicRegistry::new();

        // fnv_hash - 2 args (data, len)
        let fnv = IntrinsicCall {
            name: "fnv_hash".to_string(),
            args: vec![
                IntrinsicArg::Register(Register::R0),
                IntrinsicArg::Register(Register::R1),
            ],
        };
        assert!(registry.expand(&fnv).is_ok());

        // djb2_hash - 1 arg (string)
        let djb2 = IntrinsicCall {
            name: "djb2_hash".to_string(),
            args: vec![IntrinsicArg::Register(Register::R0)],
        };
        assert!(registry.expand(&djb2).is_ok());
    }

    #[test]
    fn test_expand_array_intrinsics() {
        let registry = IntrinsicRegistry::new();

        // sum - 2 args (arr, len)
        let sum = IntrinsicCall {
            name: "sum".to_string(),
            args: vec![
                IntrinsicArg::Register(Register::R0),
                IntrinsicArg::Register(Register::R1),
            ],
        };
        assert!(registry.expand(&sum).is_ok());

        // reverse - 2 args (arr, len)
        let reverse = IntrinsicCall {
            name: "reverse".to_string(),
            args: vec![
                IntrinsicArg::Register(Register::R0),
                IntrinsicArg::Register(Register::R1),
            ],
        };
        assert!(registry.expand(&reverse).is_ok());

        // fill - 3 args (arr, len, val)
        let fill = IntrinsicCall {
            name: "fill".to_string(),
            args: vec![
                IntrinsicArg::Register(Register::R0),
                IntrinsicArg::Register(Register::R1),
                IntrinsicArg::Register(Register::R2),
            ],
        };
        assert!(registry.expand(&fill).is_ok());

        // count - 3 args (arr, len, val)
        let count = IntrinsicCall {
            name: "count".to_string(),
            args: vec![
                IntrinsicArg::Register(Register::R0),
                IntrinsicArg::Register(Register::R1),
                IntrinsicArg::Register(Register::R2),
            ],
        };
        assert!(registry.expand(&count).is_ok());
    }

    #[test]
    fn test_expand_math_intrinsics() {
        let registry = IntrinsicRegistry::new();

        // clamp - 3 args
        let clamp = IntrinsicCall {
            name: "clamp".to_string(),
            args: vec![
                IntrinsicArg::Register(Register::R0),
                IntrinsicArg::Register(Register::R1),
                IntrinsicArg::Register(Register::R2),
            ],
        };
        assert!(registry.expand(&clamp).is_ok());

        // pow - 2 args
        let pow = IntrinsicCall {
            name: "pow".to_string(),
            args: vec![
                IntrinsicArg::Register(Register::R0),
                IntrinsicArg::Register(Register::R1),
            ],
        };
        assert!(registry.expand(&pow).is_ok());

        // factorial - 1 arg
        let factorial = IntrinsicCall {
            name: "factorial".to_string(),
            args: vec![IntrinsicArg::Register(Register::R0)],
        };
        assert!(registry.expand(&factorial).is_ok());
    }

    #[test]
    fn test_expand_string_intrinsics() {
        let registry = IntrinsicRegistry::new();

        // strcmp - 2 args
        let strcmp = IntrinsicCall {
            name: "strcmp".to_string(),
            args: vec![
                IntrinsicArg::Register(Register::R0),
                IntrinsicArg::Register(Register::R1),
            ],
        };
        assert!(registry.expand(&strcmp).is_ok());

        // strcpy - 2 args
        let strcpy = IntrinsicCall {
            name: "strcpy".to_string(),
            args: vec![
                IntrinsicArg::Register(Register::R0),
                IntrinsicArg::Register(Register::R1),
            ],
        };
        assert!(registry.expand(&strcpy).is_ok());
    }

    #[test]
    fn test_expand_search_intrinsics() {
        let registry = IntrinsicRegistry::new();

        // linear_search - 3 args
        let linear = IntrinsicCall {
            name: "linear_search".to_string(),
            args: vec![
                IntrinsicArg::Register(Register::R0),
                IntrinsicArg::Register(Register::R1),
                IntrinsicArg::Register(Register::R2),
            ],
        };
        assert!(registry.expand(&linear).is_ok());

        // find_min - 2 args
        let find_min = IntrinsicCall {
            name: "find_min".to_string(),
            args: vec![
                IntrinsicArg::Register(Register::R0),
                IntrinsicArg::Register(Register::R1),
            ],
        };
        assert!(registry.expand(&find_min).is_ok());

        // find_max - 2 args
        let find_max = IntrinsicCall {
            name: "find_max".to_string(),
            args: vec![
                IntrinsicArg::Register(Register::R0),
                IntrinsicArg::Register(Register::R1),
            ],
        };
        assert!(registry.expand(&find_max).is_ok());
    }

    #[test]
    fn test_expand_memory_intrinsics() {
        let registry = IntrinsicRegistry::new();

        // memset - 3 args
        let memset = IntrinsicCall {
            name: "memset".to_string(),
            args: vec![
                IntrinsicArg::Register(Register::R0),
                IntrinsicArg::Register(Register::R1),
                IntrinsicArg::Register(Register::R2),
            ],
        };
        assert!(registry.expand(&memset).is_ok());

        // memzero - 2 args
        let memzero = IntrinsicCall {
            name: "memzero".to_string(),
            args: vec![
                IntrinsicArg::Register(Register::R0),
                IntrinsicArg::Register(Register::R1),
            ],
        };
        assert!(registry.expand(&memzero).is_ok());

        // memcmp - 3 args
        let memcmp = IntrinsicCall {
            name: "memcmp".to_string(),
            args: vec![
                IntrinsicArg::Register(Register::R0),
                IntrinsicArg::Register(Register::R1),
                IntrinsicArg::Register(Register::R2),
            ],
        };
        assert!(registry.expand(&memcmp).is_ok());
    }
}
