//! Build script for Neurlang stencil generation
//!
//! This script generates pre-compiled code stencils at build time using the host C compiler.
//! Stencils are machine code templates with placeholder bytes that get patched at runtime.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap_or_else(|_| "x86_64-unknown-linux-gnu".to_string());

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=stencils/");

    // Determine architecture
    let arch = if target.contains("x86_64") {
        "x86_64"
    } else if target.contains("aarch64") {
        "aarch64"
    } else if target.contains("riscv64") {
        "riscv64"
    } else {
        eprintln!(
            "Warning: Unknown architecture {}, using interpreter fallback",
            target
        );
        generate_interp_only(&out_dir);
        return;
    };

    // Generate stencils for the target architecture
    match arch {
        "x86_64" => generate_x86_64_stencils(&out_dir),
        "aarch64" => generate_aarch64_stencils(&out_dir),
        "riscv64" => generate_riscv64_stencils(&out_dir),
        _ => generate_interp_only(&out_dir),
    }
}

/// Generate x86-64 stencils using inline assembly
fn generate_x86_64_stencils(out_dir: &Path) {
    let stencil_c = out_dir.join("stencils.c");
    let stencil_o = out_dir.join("stencils.o");
    let stencil_rs = out_dir.join("stencils_generated.rs");

    // Generate C source with inline assembly
    let c_source = generate_x86_64_c_source();
    fs::write(&stencil_c, c_source).expect("Failed to write stencils.c");

    // Compile to object file
    let cc = env::var("CC").unwrap_or_else(|_| "cc".to_string());
    let status = Command::new(&cc)
        .args([
            "-c",
            "-O2",
            "-fno-stack-protector",
            "-fno-asynchronous-unwind-tables",
            "-fno-exceptions",
            "-fPIC",
            "-o",
        ])
        .arg(&stencil_o)
        .arg(&stencil_c)
        .status();

    match status {
        Ok(s) if s.success() => {
            // Extract machine code from object file and generate Rust module
            extract_stencils_from_object(&stencil_o, &stencil_rs, "x86_64");
        }
        _ => {
            eprintln!("Warning: Failed to compile stencils, using fallback");
            generate_fallback_stencils(&stencil_rs, "x86_64");
        }
    }
}

/// Generate ARM64 stencils
fn generate_aarch64_stencils(out_dir: &Path) {
    let stencil_rs = out_dir.join("stencils_generated.rs");
    generate_fallback_stencils(&stencil_rs, "aarch64");
}

/// Generate RISC-V stencils
fn generate_riscv64_stencils(out_dir: &Path) {
    let stencil_rs = out_dir.join("stencils_generated.rs");
    generate_fallback_stencils(&stencil_rs, "riscv64");
}

/// Generate interpreter-only mode
fn generate_interp_only(out_dir: &Path) {
    let stencil_rs = out_dir.join("stencils_generated.rs");
    let content = r#"
// Generated stencils (interpreter-only mode)

pub const ARCH: &str = "interp";
pub const STENCILS_AVAILABLE: bool = false;

pub struct Stencil {
    pub code: &'static [u8],
    pub patches: &'static [(usize, u8)],
}

pub static STENCIL_TABLE: [Option<Stencil>; 24] = [None; 24];
"#;
    fs::write(&stencil_rs, content).expect("Failed to write stencils");
}

/// Generate x86-64 C source with inline assembly stencils
fn generate_x86_64_c_source() -> String {
    String::from(
        r#"
// Generated stencil functions for x86-64
// Each function represents a single IR opcode implementation

#include <stdint.h>

// Placeholder values that get patched at runtime
#define REG_DST     0xDEADBEEF11111111ULL
#define REG_SRC1    0xDEADBEEF22222222ULL
#define REG_SRC2    0xDEADBEEF33333333ULL
#define IMM_VAL     0xDEADBEEF44444444ULL

// Register file pointer (passed in RDI)
typedef uint64_t* regfile_t;

// ALU operations (using placeholder values that get patched)
__attribute__((naked, noinline))
void stencil_alu_add(void) {
    __asm__ volatile(
        "movq $0xDEADBEEF22222222, %%rax\n"  // Load src1 reg index
        "movq (%%rdi,%%rax,8), %%rax\n"       // Load src1 value
        "movq $0xDEADBEEF33333333, %%rcx\n"  // Load src2 reg index
        "movq (%%rdi,%%rcx,8), %%rcx\n"       // Load src2 value
        "addq %%rcx, %%rax\n"                 // Add
        "movq $0xDEADBEEF11111111, %%rcx\n"  // Load dst reg index
        "movq %%rax, (%%rdi,%%rcx,8)\n"       // Store result
        "ret\n"
        ::: "rax", "rcx", "memory"
    );
}

__attribute__((naked, noinline))
void stencil_alu_sub(void) {
    __asm__ volatile(
        "movq $0xDEADBEEF22222222, %%rax\n"
        "movq (%%rdi,%%rax,8), %%rax\n"
        "movq $0xDEADBEEF33333333, %%rcx\n"
        "movq (%%rdi,%%rcx,8), %%rcx\n"
        "subq %%rcx, %%rax\n"
        "movq $0xDEADBEEF11111111, %%rcx\n"
        "movq %%rax, (%%rdi,%%rcx,8)\n"
        "ret\n"
        ::: "rax", "rcx", "memory"
    );
}

__attribute__((naked, noinline))
void stencil_alu_and(void) {
    __asm__ volatile(
        "movq $0xDEADBEEF22222222, %%rax\n"
        "movq (%%rdi,%%rax,8), %%rax\n"
        "movq $0xDEADBEEF33333333, %%rcx\n"
        "movq (%%rdi,%%rcx,8), %%rcx\n"
        "andq %%rcx, %%rax\n"
        "movq $0xDEADBEEF11111111, %%rcx\n"
        "movq %%rax, (%%rdi,%%rcx,8)\n"
        "ret\n"
        ::: "rax", "rcx", "memory"
    );
}

__attribute__((naked, noinline))
void stencil_alu_or(void) {
    __asm__ volatile(
        "movq $0xDEADBEEF22222222, %%rax\n"
        "movq (%%rdi,%%rax,8), %%rax\n"
        "movq $0xDEADBEEF33333333, %%rcx\n"
        "movq (%%rdi,%%rcx,8), %%rcx\n"
        "orq %%rcx, %%rax\n"
        "movq $0xDEADBEEF11111111, %%rcx\n"
        "movq %%rax, (%%rdi,%%rcx,8)\n"
        "ret\n"
        ::: "rax", "rcx", "memory"
    );
}

__attribute__((naked, noinline))
void stencil_alu_xor(void) {
    __asm__ volatile(
        "movq $0xDEADBEEF22222222, %%rax\n"
        "movq (%%rdi,%%rax,8), %%rax\n"
        "movq $0xDEADBEEF33333333, %%rcx\n"
        "movq (%%rdi,%%rcx,8), %%rcx\n"
        "xorq %%rcx, %%rax\n"
        "movq $0xDEADBEEF11111111, %%rcx\n"
        "movq %%rax, (%%rdi,%%rcx,8)\n"
        "ret\n"
        ::: "rax", "rcx", "memory"
    );
}

__attribute__((naked, noinline))
void stencil_alu_shl(void) {
    __asm__ volatile(
        "movq $0xDEADBEEF22222222, %%rax\n"
        "movq (%%rdi,%%rax,8), %%rax\n"
        "movq $0xDEADBEEF33333333, %%rcx\n"
        "movq (%%rdi,%%rcx,8), %%rcx\n"
        "shlq %%cl, %%rax\n"
        "movq $0xDEADBEEF11111111, %%rcx\n"
        "movq %%rax, (%%rdi,%%rcx,8)\n"
        "ret\n"
        ::: "rax", "rcx", "memory"
    );
}

__attribute__((naked, noinline))
void stencil_alu_shr(void) {
    __asm__ volatile(
        "movq $0xDEADBEEF22222222, %%rax\n"
        "movq (%%rdi,%%rax,8), %%rax\n"
        "movq $0xDEADBEEF33333333, %%rcx\n"
        "movq (%%rdi,%%rcx,8), %%rcx\n"
        "shrq %%cl, %%rax\n"
        "movq $0xDEADBEEF11111111, %%rcx\n"
        "movq %%rax, (%%rdi,%%rcx,8)\n"
        "ret\n"
        ::: "rax", "rcx", "memory"
    );
}

__attribute__((naked, noinline))
void stencil_alu_sar(void) {
    __asm__ volatile(
        "movq $0xDEADBEEF22222222, %%rax\n"
        "movq (%%rdi,%%rax,8), %%rax\n"
        "movq $0xDEADBEEF33333333, %%rcx\n"
        "movq (%%rdi,%%rcx,8), %%rcx\n"
        "sarq %%cl, %%rax\n"
        "movq $0xDEADBEEF11111111, %%rcx\n"
        "movq %%rax, (%%rdi,%%rcx,8)\n"
        "ret\n"
        ::: "rax", "rcx", "memory"
    );
}

// ALU with immediate
__attribute__((naked, noinline))
void stencil_alui_add(void) {
    __asm__ volatile(
        "movq $0xDEADBEEF22222222, %%rax\n"
        "movq (%%rdi,%%rax,8), %%rax\n"
        "movl $0xDEADBEEF, %%ecx\n"           // Immediate (32-bit)
        "movsx %%ecx, %%rcx\n"
        "addq %%rcx, %%rax\n"
        "movq $0xDEADBEEF11111111, %%rcx\n"
        "movq %%rax, (%%rdi,%%rcx,8)\n"
        "ret\n"
        ::: "rax", "rcx", "memory"
    );
}

// MulDiv operations
__attribute__((naked, noinline))
void stencil_mul(void) {
    __asm__ volatile(
        "movq $0xDEADBEEF22222222, %%rax\n"
        "movq (%%rdi,%%rax,8), %%rax\n"
        "movq $0xDEADBEEF33333333, %%rcx\n"
        "movq (%%rdi,%%rcx,8), %%rcx\n"
        "imulq %%rcx, %%rax\n"
        "movq $0xDEADBEEF11111111, %%rcx\n"
        "movq %%rax, (%%rdi,%%rcx,8)\n"
        "ret\n"
        ::: "rax", "rcx", "rdx", "memory"
    );
}

__attribute__((naked, noinline))
void stencil_div(void) {
    __asm__ volatile(
        "movq $0xDEADBEEF22222222, %%rax\n"
        "movq (%%rdi,%%rax,8), %%rax\n"
        "movq $0xDEADBEEF33333333, %%rcx\n"
        "movq (%%rdi,%%rcx,8), %%rcx\n"
        "xorq %%rdx, %%rdx\n"
        "divq %%rcx\n"
        "movq $0xDEADBEEF11111111, %%rcx\n"
        "movq %%rax, (%%rdi,%%rcx,8)\n"
        "ret\n"
        ::: "rax", "rcx", "rdx", "memory"
    );
}

// Load operations (64-bit)
__attribute__((naked, noinline))
void stencil_load64(void) {
    __asm__ volatile(
        "movq $0xDEADBEEF22222222, %%rax\n"   // Base register
        "movq (%%rdi,%%rax,8), %%rax\n"        // Load base address
        "movl $0xDEADBEEF, %%ecx\n"            // Offset
        "movsx %%ecx, %%rcx\n"
        "movq (%%rax,%%rcx,1), %%rax\n"        // Load from memory
        "movq $0xDEADBEEF11111111, %%rcx\n"   // Dest register
        "movq %%rax, (%%rdi,%%rcx,8)\n"        // Store to register
        "ret\n"
        ::: "rax", "rcx", "memory"
    );
}

// Store operations (64-bit)
__attribute__((naked, noinline))
void stencil_store64(void) {
    __asm__ volatile(
        "movq $0xDEADBEEF22222222, %%rax\n"   // Base register
        "movq (%%rdi,%%rax,8), %%rax\n"        // Load base address
        "movl $0xDEADBEEF, %%ecx\n"            // Offset
        "movsx %%ecx, %%rcx\n"
        "addq %%rcx, %%rax\n"                   // Calculate address
        "movq $0xDEADBEEF11111111, %%rcx\n"   // Source register
        "movq (%%rdi,%%rcx,8), %%rcx\n"        // Load value
        "movq %%rcx, (%%rax)\n"                 // Store to memory
        "ret\n"
        ::: "rax", "rcx", "memory"
    );
}

// MOV immediate
__attribute__((naked, noinline))
void stencil_mov_imm(void) {
    __asm__ volatile(
        "movl $0xDEADBEEF, %%eax\n"            // Load immediate
        "movsx %%eax, %%rax\n"                  // Sign extend
        "movq $0xDEADBEEF11111111, %%rcx\n"   // Dest register
        "movq %%rax, (%%rdi,%%rcx,8)\n"        // Store to register
        "ret\n"
        ::: "rax", "rcx", "memory"
    );
}

// MOV register
__attribute__((naked, noinline))
void stencil_mov_reg(void) {
    __asm__ volatile(
        "movq $0xDEADBEEF22222222, %%rax\n"   // Source register
        "movq (%%rdi,%%rax,8), %%rax\n"        // Load value
        "movq $0xDEADBEEF11111111, %%rcx\n"   // Dest register
        "movq %%rax, (%%rdi,%%rcx,8)\n"        // Store to register
        "ret\n"
        ::: "rax", "rcx", "memory"
    );
}

// NOP
__attribute__((naked, noinline))
void stencil_nop(void) {
    __asm__ volatile(
        "ret\n"
    );
}

// HALT (returns special value)
__attribute__((naked, noinline))
void stencil_halt(void) {
    __asm__ volatile(
        "movq $0xFFFFFFFFFFFFFFFF, %%rax\n"   // Return halt sentinel
        "ret\n"
        ::: "rax"
    );
}

// Branch (conditional jump placeholder)
__attribute__((naked, noinline))
void stencil_branch_eq(void) {
    __asm__ volatile(
        "movq $0xDEADBEEF22222222, %%rax\n"
        "movq (%%rdi,%%rax,8), %%rax\n"
        "movq $0xDEADBEEF33333333, %%rcx\n"
        "movq (%%rdi,%%rcx,8), %%rcx\n"
        "cmpq %%rcx, %%rax\n"
        "movl $0xDEADBEEF, %%eax\n"           // Target offset
        "movsx %%eax, %%rax\n"
        "cmovneq %%rsi, %%rax\n"              // If not equal, use fallthrough (RSI = next)
        "ret\n"
        ::: "rax", "rcx"
    );
}

// Atomic CAS
__attribute__((naked, noinline))
void stencil_atomic_cas(void) {
    __asm__ volatile(
        "movq $0xDEADBEEF22222222, %%rax\n"   // Address register
        "movq (%%rdi,%%rax,8), %%r8\n"         // Load address
        "movq $0xDEADBEEF33333333, %%rax\n"   // Expected value register
        "movq (%%rdi,%%rax,8), %%rax\n"        // Load expected
        "movq $0xDEADBEEF11111111, %%rcx\n"   // New value register
        "movq (%%rdi,%%rcx,8), %%rcx\n"        // Load new value
        "lock cmpxchgq %%rcx, (%%r8)\n"        // CAS
        "movq $0xDEADBEEF11111111, %%rcx\n"   // Result register
        "movq %%rax, (%%rdi,%%rcx,8)\n"        // Store old value
        "ret\n"
        ::: "rax", "rcx", "r8", "memory"
    );
}

// Fence (memory barrier)
__attribute__((naked, noinline))
void stencil_fence_seqcst(void) {
    __asm__ volatile(
        "mfence\n"
        "ret\n"
        ::: "memory"
    );
}
"#,
    )
}

/// Extract stencil machine code from object file
fn extract_stencils_from_object(_obj_path: &Path, out_path: &Path, arch: &str) {
    // For now, always use the hand-coded fallback stencils
    // The objdump extraction is complex and the fallback is more reliable
    generate_fallback_stencils(out_path, arch);
}

/// Generate fallback stencils when compilation fails
fn generate_fallback_stencils(out_path: &Path, arch: &str) {
    let content = format!(
        r#"// Fallback stencils for {} (hand-coded)
// Used when build-time stencil generation fails

pub const ARCH: &str = "{}";
pub const STENCILS_AVAILABLE: bool = true;

/// A code stencil with patchable locations
#[derive(Clone, Copy)]
pub struct Stencil {{
    /// Raw machine code bytes
    pub code: &'static [u8],
    /// Patch locations: (offset, patch_type)
    pub patches: &'static [(usize, u8)],
}}

/// Placeholder values
pub const PLACEHOLDER_DST: u64 = 0xDEADBEEF11111111;
pub const PLACEHOLDER_SRC1: u64 = 0xDEADBEEF22222222;
pub const PLACEHOLDER_SRC2: u64 = 0xDEADBEEF33333333;
pub const PLACEHOLDER_IMM: u32 = 0xDEADBEEF;

// x86-64 stencil implementations
// These are minimal hand-coded stencils

/// ADD rd, rs1, rs2 - adds two registers
/// movabs rax, PLACEHOLDER_SRC1    ; 48 b8 [8 bytes]
/// mov rax, [rdi + rax*8]          ; 48 8b 04 c7
/// movabs rcx, PLACEHOLDER_SRC2    ; 48 b9 [8 bytes]
/// mov rcx, [rdi + rcx*8]          ; 48 8b 0c cf
/// add rax, rcx                     ; 48 01 c8
/// movabs rcx, PLACEHOLDER_DST     ; 48 b9 [8 bytes]
/// mov [rdi + rcx*8], rax          ; 48 89 04 cf
/// ret                              ; c3
pub static ALU_ADD_CODE: [u8; 46] = [
    0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,  // movabs rax, src1
    0x48, 0x8b, 0x04, 0xc7,                                      // mov rax, [rdi + rax*8]
    0x48, 0xb9, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33,  // movabs rcx, src2
    0x48, 0x8b, 0x0c, 0xcf,                                      // mov rcx, [rdi + rcx*8]
    0x48, 0x01, 0xc8,                                            // add rax, rcx
    0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,  // movabs rcx, dst
    0x48, 0x89, 0x04, 0xcf,                                      // mov [rdi + rcx*8], rax
    0xc3,                                                        // ret
];
pub static ALU_ADD_PATCHES: [(usize, u8); 3] = [(2, 2), (16, 3), (33, 1)];

/// SUB rd, rs1, rs2
pub static ALU_SUB_CODE: [u8; 46] = [
    0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
    0x48, 0x8b, 0x04, 0xc7,
    0x48, 0xb9, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33,
    0x48, 0x8b, 0x0c, 0xcf,
    0x48, 0x29, 0xc8,  // sub instead of add
    0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x48, 0x89, 0x04, 0xcf,
    0xc3,
];
pub static ALU_SUB_PATCHES: [(usize, u8); 3] = [(2, 2), (16, 3), (33, 1)];

/// MUL rd, rs1, rs2
pub static MUL_CODE: [u8; 47] = [
    0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
    0x48, 0x8b, 0x04, 0xc7,
    0x48, 0xb9, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33,
    0x48, 0x8b, 0x0c, 0xcf,
    0x48, 0x0f, 0xaf, 0xc1,  // imul rax, rcx
    0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x48, 0x89, 0x04, 0xcf,
    0xc3,
];
pub static MUL_PATCHES: [(usize, u8); 3] = [(2, 2), (16, 3), (34, 1)];

/// MOV rd, imm32
/// movabs rax, imm                  ; 48 b8 [8 bytes - but we use 4-byte imm]
/// mov eax, imm                     ; b8 [4 bytes]
/// cdqe                             ; 48 98 (sign extend)
/// movabs rcx, PLACEHOLDER_DST     ; 48 b9 [8 bytes]
/// mov [rdi + rcx*8], rax          ; 48 89 04 cf
/// ret                              ; c3
pub static MOV_IMM_CODE: [u8; 22] = [
    0xb8, 0xef, 0xbe, 0xad, 0xde,  // mov eax, imm32
    0x48, 0x98,                     // cdqe (sign extend eax to rax)
    0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,  // movabs rcx, dst
    0x48, 0x89, 0x04, 0xcf,         // mov [rdi + rcx*8], rax
    0xc3,
];
pub static MOV_IMM_PATCHES: [(usize, u8); 2] = [(1, 4), (9, 1)];

/// MOV rd, rs
pub static MOV_REG_CODE: [u8; 29] = [
    0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,  // movabs rax, src1
    0x48, 0x8b, 0x04, 0xc7,                                      // mov rax, [rdi + rax*8]
    0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,  // movabs rcx, dst
    0x48, 0x89, 0x04, 0xcf,                                      // mov [rdi + rcx*8], rax
    0xc3,
];
pub static MOV_REG_PATCHES: [(usize, u8); 2] = [(2, 2), (16, 1)];

/// LOAD.D rd, [rs1 + offset]
pub static LOAD64_CODE: [u8; 41] = [
    0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,  // movabs rax, base_reg
    0x48, 0x8b, 0x04, 0xc7,                                      // mov rax, [rdi + rax*8]
    0xb9, 0xef, 0xbe, 0xad, 0xde,                                // mov ecx, offset
    0x48, 0x63, 0xc9,                                            // movsxd rcx, ecx
    0x48, 0x8b, 0x04, 0x08,                                      // mov rax, [rax + rcx]
    0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,  // movabs rcx, dst_reg
    0x48, 0x89, 0x04, 0xcf,                                      // mov [rdi + rcx*8], rax
    0xc3,
];
pub static LOAD64_PATCHES: [(usize, u8); 3] = [(2, 2), (15, 4), (28, 1)];

/// STORE.D rs, [rs1 + offset]
pub static STORE64_CODE: [u8; 43] = [
    0x48, 0xb8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,  // movabs rax, base_reg
    0x48, 0x8b, 0x04, 0xc7,                                      // mov rax, [rdi + rax*8]
    0xb9, 0xef, 0xbe, 0xad, 0xde,                                // mov ecx, offset
    0x48, 0x63, 0xc9,                                            // movsxd rcx, ecx
    0x48, 0x01, 0xc8,                                            // add rax, rcx
    0x48, 0xb9, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,  // movabs rcx, src_reg
    0x48, 0x8b, 0x0c, 0xcf,                                      // mov rcx, [rdi + rcx*8]
    0x48, 0x89, 0x08,                                            // mov [rax], rcx
    0xc3,
];
pub static STORE64_PATCHES: [(usize, u8); 3] = [(2, 2), (15, 4), (28, 1)];

/// NOP
pub static NOP_CODE: [u8; 1] = [0xc3];  // Just ret
pub static NOP_PATCHES: [(usize, u8); 0] = [];

/// HALT - Returns sentinel value
pub static HALT_CODE: [u8; 12] = [
    0x48, 0xb8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,  // movabs rax, -1
    0xc3,                                                        // ret
    0x00,                                                        // padding
];
pub static HALT_PATCHES: [(usize, u8); 0] = [];

/// FENCE (memory barrier)
pub static FENCE_CODE: [u8; 4] = [
    0x0f, 0xae, 0xf0,  // mfence
    0xc3,              // ret
];
pub static FENCE_PATCHES: [(usize, u8); 0] = [];

/// RUNTIME_CALL - Call into runtime library function
/// This is a placeholder that calls a runtime function through RSI
/// The runtime sets up RSI to point to the appropriate handler
/// mov rax, rsi       ; load runtime handler address
/// call rax           ; call runtime
/// ret
pub static RUNTIME_CALL_CODE: [u8; 26] = [
    // Save registers (RDI contains register file pointer)
    0x48, 0xb8, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,  // movabs rax, dst_reg (for result)
    0x48, 0xb9, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,  // movabs rcx, src1_reg
    // Call runtime (through RSI - set by JIT engine before execution)
    0xff, 0xd6,  // call rsi
    // Result is in RAX, store to dst reg
    0xc3,        // ret
    0x90, 0x90, 0x90,  // padding
];
pub static RUNTIME_CALL_PATCHES: [(usize, u8); 2] = [(2, 1), (12, 2)];

/// Stencil references for each opcode
pub static ALU_ADD: Stencil = Stencil {{ code: &ALU_ADD_CODE, patches: &ALU_ADD_PATCHES }};
pub static ALU_SUB: Stencil = Stencil {{ code: &ALU_SUB_CODE, patches: &ALU_SUB_PATCHES }};
pub static MUL: Stencil = Stencil {{ code: &MUL_CODE, patches: &MUL_PATCHES }};
pub static MOV_IMM: Stencil = Stencil {{ code: &MOV_IMM_CODE, patches: &MOV_IMM_PATCHES }};
pub static MOV_REG: Stencil = Stencil {{ code: &MOV_REG_CODE, patches: &MOV_REG_PATCHES }};
pub static LOAD64: Stencil = Stencil {{ code: &LOAD64_CODE, patches: &LOAD64_PATCHES }};
pub static STORE64: Stencil = Stencil {{ code: &STORE64_CODE, patches: &STORE64_PATCHES }};
pub static NOP: Stencil = Stencil {{ code: &NOP_CODE, patches: &NOP_PATCHES }};
pub static HALT: Stencil = Stencil {{ code: &HALT_CODE, patches: &HALT_PATCHES }};
pub static FENCE: Stencil = Stencil {{ code: &FENCE_CODE, patches: &FENCE_PATCHES }};
pub static RUNTIME_CALL: Stencil = Stencil {{ code: &RUNTIME_CALL_CODE, patches: &RUNTIME_CALL_PATCHES }};
"#,
        arch, arch
    );

    fs::write(out_path, content).expect("Failed to write fallback stencils");
}
