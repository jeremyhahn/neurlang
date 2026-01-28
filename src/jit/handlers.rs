//! JIT Opcode Handlers
//!
//! Native implementations of all 32 opcodes. These are called from the JIT executor.

use crate::ir::{
    AluOp, AtomicOp, BitsOp, BranchCond, FileOp, FpuOp, Instruction, IoOp, MulDivOp, NetOp,
    NetOption, Opcode, RandOp, TimeOp, TrapType,
};
use crate::jit::context::JitContext;
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

/// Control flow result from instruction execution
#[derive(Debug, Clone, Copy)]
pub enum ControlFlow {
    /// Continue to next instruction
    Continue,
    /// Jump to relative offset
    Jump(i32),
    /// Jump to absolute instruction index
    AbsoluteJump(usize),
    /// Halt execution
    Halt,
    /// Error occurred
    Error,
}

/// Execute a single instruction
#[inline(always)]
pub fn execute_instruction(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    match instr.opcode {
        Opcode::Alu => execute_alu(ctx, instr),
        Opcode::AluI => execute_alui(ctx, instr),
        Opcode::MulDiv => execute_muldiv(ctx, instr),
        Opcode::Load => execute_load(ctx, instr),
        Opcode::Store => execute_store(ctx, instr),
        Opcode::Atomic => execute_atomic(ctx, instr),
        Opcode::Branch => execute_branch(ctx, instr),
        Opcode::Call => execute_call(ctx, instr),
        Opcode::Ret => execute_ret(ctx, instr),
        Opcode::Jump => execute_jump(ctx, instr),
        Opcode::CapNew | Opcode::CapRestrict | Opcode::CapQuery => execute_capability(ctx, instr),
        Opcode::Spawn | Opcode::Join | Opcode::Chan => execute_concurrency(ctx, instr),
        Opcode::Fence => execute_fence(ctx, instr),
        Opcode::Yield => execute_yield(ctx, instr),
        Opcode::Taint | Opcode::Sanitize => execute_taint(ctx, instr),
        Opcode::File => execute_file(ctx, instr),
        Opcode::Net => execute_net(ctx, instr),
        Opcode::NetSetopt => execute_net_setopt(ctx, instr),
        Opcode::Io => execute_io(ctx, instr),
        Opcode::Time => execute_time(ctx, instr),
        Opcode::Fpu => execute_fpu(ctx, instr),
        Opcode::Rand => execute_rand(ctx, instr),
        Opcode::Bits => execute_bits(ctx, instr),
        Opcode::Mov => execute_mov(ctx, instr),
        Opcode::Trap => execute_trap(ctx, instr),
        Opcode::ExtCall => execute_extcall(ctx, instr),
        Opcode::Nop => ControlFlow::Continue,
        Opcode::Halt => ControlFlow::Halt,
    }
}

// ============ Arithmetic Operations ============

#[inline(always)]
fn execute_alu(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    let src1 = ctx.get_reg(instr.rs1 as usize);
    let src2 = ctx.get_reg(instr.rs2 as usize);
    let result = match AluOp::from_u8(instr.mode) {
        Some(AluOp::Add) => src1.wrapping_add(src2),
        Some(AluOp::Sub) => src1.wrapping_sub(src2),
        Some(AluOp::And) => src1 & src2,
        Some(AluOp::Or) => src1 | src2,
        Some(AluOp::Xor) => src1 ^ src2,
        Some(AluOp::Shl) => src1 << (src2 & 63),
        Some(AluOp::Shr) => src1 >> (src2 & 63),
        Some(AluOp::Sar) => (src1 as i64 >> (src2 & 63)) as u64,
        None => 0,
    };
    ctx.set_reg(instr.rd as usize, result);
    ControlFlow::Continue
}

#[inline(always)]
fn execute_alui(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    let src1 = ctx.get_reg(instr.rs1 as usize);
    let imm = instr.imm.unwrap_or(0) as i64 as u64;
    let result = match AluOp::from_u8(instr.mode) {
        Some(AluOp::Add) => src1.wrapping_add(imm),
        Some(AluOp::Sub) => src1.wrapping_sub(imm),
        Some(AluOp::And) => src1 & imm,
        Some(AluOp::Or) => src1 | imm,
        Some(AluOp::Xor) => src1 ^ imm,
        Some(AluOp::Shl) => src1 << (imm & 63),
        Some(AluOp::Shr) => src1 >> (imm & 63),
        Some(AluOp::Sar) => (src1 as i64 >> (imm & 63)) as u64,
        None => 0,
    };
    ctx.set_reg(instr.rd as usize, result);
    ControlFlow::Continue
}

#[inline(always)]
fn execute_muldiv(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    let src1 = ctx.get_reg(instr.rs1 as usize);
    let src2 = ctx.get_reg(instr.rs2 as usize);

    if src2 == 0
        && matches!(
            MulDivOp::from_u8(instr.mode),
            Some(MulDivOp::Div) | Some(MulDivOp::Mod)
        )
    {
        ctx.error = Some("Division by zero".into());
        return ControlFlow::Error;
    }

    let result = match MulDivOp::from_u8(instr.mode) {
        Some(MulDivOp::Mul) => src1.wrapping_mul(src2),
        Some(MulDivOp::MulH) => {
            let full = (src1 as u128) * (src2 as u128);
            (full >> 64) as u64
        }
        Some(MulDivOp::Div) => src1 / src2,
        Some(MulDivOp::Mod) => src1 % src2,
        None => 0,
    };
    ctx.set_reg(instr.rd as usize, result);
    ControlFlow::Continue
}

// ============ Memory Operations ============

#[inline(always)]
fn execute_load(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    let base = ctx.get_reg(instr.rs1 as usize);
    let offset = instr.imm.unwrap_or(0) as i64;
    let addr = (base as i64 + offset) as u64;
    let width = instr.mode & 0x03;

    match ctx.read_mem(addr, width) {
        Ok(value) => {
            ctx.set_reg(instr.rd as usize, value);
            ControlFlow::Continue
        }
        Err(msg) => {
            ctx.error = Some(msg.into());
            ControlFlow::Error
        }
    }
}

#[inline(always)]
fn execute_store(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    let base = ctx.get_reg(instr.rs1 as usize);
    let offset = instr.imm.unwrap_or(0) as i64;
    let addr = (base as i64 + offset) as u64;
    let value = ctx.get_reg(instr.rd as usize);
    let width = instr.mode & 0x03;

    match ctx.write_mem(addr, value, width) {
        Ok(()) => ControlFlow::Continue,
        Err(msg) => {
            ctx.error = Some(msg.into());
            ControlFlow::Error
        }
    }
}

#[inline(always)]
fn execute_atomic(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    let addr = ctx.get_reg(instr.rs1 as usize);
    let value = ctx.get_reg(instr.rs2 as usize);

    match ctx.read_mem_u64(addr) {
        Ok(current) => {
            let (result, store) = match AtomicOp::from_u8(instr.mode) {
                Some(AtomicOp::Cas) => {
                    let expected = ctx.get_reg(instr.rd as usize);
                    if current == expected {
                        (current, value)
                    } else {
                        (current, current)
                    }
                }
                Some(AtomicOp::Xchg) => (current, value),
                Some(AtomicOp::Add) => (current, current.wrapping_add(value)),
                Some(AtomicOp::And) => (current, current & value),
                Some(AtomicOp::Or) => (current, current | value),
                Some(AtomicOp::Xor) => (current, current ^ value),
                Some(AtomicOp::Min) => (current, current.min(value)),
                Some(AtomicOp::Max) => (current, current.max(value)),
                None => (current, current),
            };

            if let Err(msg) = ctx.write_mem_u64(addr, store) {
                ctx.error = Some(msg.into());
                return ControlFlow::Error;
            }
            ctx.set_reg(instr.rd as usize, result);
            ControlFlow::Continue
        }
        Err(msg) => {
            ctx.error = Some(msg.into());
            ControlFlow::Error
        }
    }
}

// ============ Control Flow ============

#[inline(always)]
fn execute_branch(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    let cond = BranchCond::from_u8(instr.mode).unwrap_or(BranchCond::Always);
    let src1 = ctx.get_reg(instr.rs1 as usize) as i64;
    let src2 = ctx.get_reg(instr.rs2 as usize) as i64;

    let take_branch = match cond {
        BranchCond::Always => true,
        BranchCond::Eq => src1 == src2,
        BranchCond::Ne => src1 != src2,
        BranchCond::Lt => src1 < src2,
        BranchCond::Le => src1 <= src2,
        BranchCond::Gt => src1 > src2,
        BranchCond::Ge => src1 >= src2,
        BranchCond::Ltu => (src1 as u64) < (src2 as u64),
    };

    if take_branch {
        ControlFlow::Jump(instr.imm.unwrap_or(0))
    } else {
        ControlFlow::Continue
    }
}

#[inline(always)]
fn execute_call(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    // Push return address onto call stack for nested calls
    let return_addr = ctx.pc + 1;
    ctx.call_stack.push(return_addr);
    // Also set LR register for compatibility with code that reads it
    ctx.set_reg(26, return_addr as u64);
    ControlFlow::Jump(instr.imm.unwrap_or(0))
}

#[inline(always)]
fn execute_ret(ctx: &mut JitContext, _instr: &Instruction) -> ControlFlow {
    // Pop return address from call stack
    let return_addr = ctx.call_stack.pop().unwrap_or_else(|| {
        // Fallback to LR if stack is empty (for backward compatibility)
        ctx.get_reg(26) as usize
    });
    ControlFlow::AbsoluteJump(return_addr)
}

#[inline(always)]
fn execute_jump(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    if instr.mode == 1 {
        // Indirect - register holds absolute instruction index
        let target = ctx.get_reg(instr.rs1 as usize) as usize;
        ControlFlow::AbsoluteJump(target)
    } else {
        // Direct - immediate is relative offset
        ControlFlow::Jump(instr.imm.unwrap_or(0))
    }
}

// ============ Capability Operations ============

#[inline(always)]
fn execute_capability(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    // Simplified - just pass through values
    let value = ctx.get_reg(instr.rs1 as usize);
    ctx.set_reg(instr.rd as usize, value);
    ControlFlow::Continue
}

// ============ Concurrency Operations ============

#[inline(always)]
fn execute_concurrency(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    // Simplified - no actual threading in single-threaded JIT
    ctx.set_reg(instr.rd as usize, 0);
    ControlFlow::Continue
}

#[inline(always)]
fn execute_fence(_ctx: &mut JitContext, _instr: &Instruction) -> ControlFlow {
    std::sync::atomic::fence(Ordering::SeqCst);
    ControlFlow::Continue
}

#[inline(always)]
fn execute_yield(_ctx: &mut JitContext, _instr: &Instruction) -> ControlFlow {
    std::thread::yield_now();
    ControlFlow::Continue
}

// ============ Taint Operations ============

#[inline(always)]
fn execute_taint(_ctx: &mut JitContext, _instr: &Instruction) -> ControlFlow {
    // Taint tracking is a no-op in JIT mode (would need shadow registers)
    ControlFlow::Continue
}

// ============ File Operations ============

fn execute_file(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    let result = match FileOp::from_u8(instr.mode) {
        Some(FileOp::Open) => {
            let path_addr = ctx.get_reg(instr.rs1 as usize) as usize;
            let path_len = ctx.get_reg(instr.rs2 as usize) as usize;
            let flags = instr.imm.unwrap_or(0) as u32;

            // Copy path to avoid borrow conflict
            let path_owned = ctx.get_string(path_addr, path_len).map(|s| s.to_string());
            if let Some(path) = path_owned {
                ctx.io_runtime.file_open(&path, flags).unwrap_or(u64::MAX)
            } else {
                u64::MAX
            }
        }
        Some(FileOp::Read) => {
            let fd = ctx.get_reg(instr.rs1 as usize);
            let buf_addr = ctx.get_reg(instr.rs2 as usize) as usize;
            let imm_len = instr.imm.unwrap_or(0) as usize;
            let len = if imm_len == 0 {
                ctx.get_reg(instr.rd as usize) as usize
            } else {
                imm_len
            };

            // Use pre-allocated I/O buffer to avoid allocation
            if buf_addr + len <= ctx.memory_size {
                // Ensure io_buffer is large enough (usually it is)
                if ctx.io_buffer.len() < len {
                    ctx.io_buffer.resize(len, 0);
                }
                match ctx.io_runtime.file_read(fd, &mut ctx.io_buffer[..len]) {
                    Ok(n) => {
                        ctx.memory[buf_addr..buf_addr + n].copy_from_slice(&ctx.io_buffer[..n]);
                        n as u64
                    }
                    Err(_) => u64::MAX,
                }
            } else {
                u64::MAX
            }
        }
        Some(FileOp::Write) => {
            let fd = ctx.get_reg(instr.rs1 as usize);
            let buf_addr = ctx.get_reg(instr.rs2 as usize) as usize;
            let imm_len = instr.imm.unwrap_or(0) as usize;
            let len = if imm_len == 0 {
                ctx.get_reg(instr.rd as usize) as usize
            } else {
                imm_len
            };

            // Copy data to avoid borrow conflict
            if buf_addr + len <= ctx.memory_size {
                let data = ctx.memory[buf_addr..buf_addr + len].to_vec();
                match ctx.io_runtime.file_write(fd, &data) {
                    Ok(n) => n as u64,
                    Err(_) => u64::MAX,
                }
            } else {
                u64::MAX
            }
        }
        Some(FileOp::Close) => {
            let fd = ctx.get_reg(instr.rs1 as usize);
            match ctx.io_runtime.file_close(fd) {
                Ok(()) => 0,
                Err(_) => u64::MAX,
            }
        }
        Some(FileOp::Seek) => {
            let fd = ctx.get_reg(instr.rs1 as usize);
            let offset = ctx.get_reg(instr.rs2 as usize) as i64;
            let whence = instr.imm.unwrap_or(0) as u32;
            ctx.io_runtime
                .file_seek(fd, offset, whence)
                .unwrap_or(u64::MAX)
        }
        Some(FileOp::Stat) => {
            let path_addr = ctx.get_reg(instr.rs1 as usize) as usize;
            let path_len = ctx.get_reg(instr.rs2 as usize) as usize;

            // Copy path to avoid borrow conflict
            let path_owned = ctx.get_string(path_addr, path_len).map(|s| s.to_string());
            if let Some(path) = path_owned {
                match ctx.io_runtime.file_stat(&path) {
                    Ok((size, mtime)) => {
                        ctx.set_reg(1, mtime); // Store mtime in r1
                        size
                    }
                    Err(_) => u64::MAX,
                }
            } else {
                u64::MAX
            }
        }
        Some(FileOp::Mkdir) => {
            let path_addr = ctx.get_reg(instr.rs1 as usize) as usize;
            let path_len = ctx.get_reg(instr.rs2 as usize) as usize;

            // Copy path to avoid borrow conflict
            let path_owned = ctx.get_string(path_addr, path_len).map(|s| s.to_string());
            if let Some(path) = path_owned {
                match ctx.io_runtime.file_mkdir(&path) {
                    Ok(()) => 0,
                    Err(_) => u64::MAX,
                }
            } else {
                u64::MAX
            }
        }
        Some(FileOp::Delete) => {
            let path_addr = ctx.get_reg(instr.rs1 as usize) as usize;
            let path_len = ctx.get_reg(instr.rs2 as usize) as usize;

            // Copy path to avoid borrow conflict
            let path_owned = ctx.get_string(path_addr, path_len).map(|s| s.to_string());
            if let Some(path) = path_owned {
                match ctx.io_runtime.file_delete(&path) {
                    Ok(()) => 0,
                    Err(_) => u64::MAX,
                }
            } else {
                u64::MAX
            }
        }
        None => u64::MAX,
    };
    ctx.set_reg(instr.rd as usize, result);
    ControlFlow::Continue
}

// ============ Network Operations ============

fn execute_net(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    let result = match NetOp::from_u8(instr.mode) {
        Some(NetOp::Socket) => {
            let domain = ctx.get_reg(instr.rs1 as usize) as u32;
            let socket_type = ctx.get_reg(instr.rs2 as usize) as u32;
            ctx.io_runtime
                .net_socket(domain, socket_type)
                .unwrap_or(u64::MAX)
        }
        Some(NetOp::Connect) => {
            let fd = ctx.get_reg(instr.rs1 as usize);
            let addr_ptr = ctx.get_reg(instr.rs2 as usize) as usize;
            let port = instr.imm.unwrap_or(0) as u16;

            // Copy address to avoid borrow conflict
            let addr_owned = ctx.get_string(addr_ptr, 256).map(|s| s.to_string());
            if let Some(addr) = addr_owned {
                match ctx.io_runtime.net_connect(fd, &addr, port) {
                    Ok(()) => 0,
                    Err(_) => u64::MAX,
                }
            } else {
                u64::MAX
            }
        }
        Some(NetOp::Bind) => {
            let fd = ctx.get_reg(instr.rs1 as usize);
            let addr_ptr = ctx.get_reg(instr.rs2 as usize) as usize;
            let port = instr.imm.unwrap_or(0) as u16;

            // Copy address to avoid borrow conflict
            let addr_owned = ctx.get_string(addr_ptr, 256).map(|s| s.to_string());
            if let Some(addr) = addr_owned {
                match ctx.io_runtime.net_bind(fd, &addr, port) {
                    Ok(()) => 0,
                    Err(_) => u64::MAX,
                }
            } else {
                u64::MAX
            }
        }
        Some(NetOp::Listen) => {
            let fd = ctx.get_reg(instr.rs1 as usize);
            let backlog = ctx.get_reg(instr.rs2 as usize) as u32;
            match ctx.io_runtime.net_listen(fd, backlog) {
                Ok(()) => 0,
                Err(_) => u64::MAX,
            }
        }
        Some(NetOp::Accept) => {
            let fd = ctx.get_reg(instr.rs1 as usize);
            ctx.io_runtime.net_accept(fd).unwrap_or(u64::MAX)
        }
        Some(NetOp::Send) => {
            let fd = ctx.get_reg(instr.rs1 as usize);
            let buf_addr = ctx.get_reg(instr.rs2 as usize) as usize;
            let imm_len = instr.imm.unwrap_or(0) as usize;
            let len = if imm_len == 0 {
                ctx.get_reg(instr.rd as usize) as usize
            } else {
                imm_len
            };

            // Copy data to avoid borrow conflict
            if buf_addr + len <= ctx.memory_size {
                let data = ctx.memory[buf_addr..buf_addr + len].to_vec();
                match ctx.io_runtime.net_send(fd, &data) {
                    Ok(n) => n as u64,
                    Err(_) => u64::MAX,
                }
            } else {
                u64::MAX
            }
        }
        Some(NetOp::Recv) => {
            let fd = ctx.get_reg(instr.rs1 as usize);
            let buf_addr = ctx.get_reg(instr.rs2 as usize) as usize;
            let imm_len = instr.imm.unwrap_or(0) as usize;
            let len = if imm_len == 0 {
                ctx.get_reg(instr.rd as usize) as usize
            } else {
                imm_len
            };

            // Use pre-allocated I/O buffer to avoid allocation
            if buf_addr + len <= ctx.memory_size {
                // Ensure io_buffer is large enough (usually it is)
                if ctx.io_buffer.len() < len {
                    ctx.io_buffer.resize(len, 0);
                }
                match ctx.io_runtime.net_recv(fd, &mut ctx.io_buffer[..len]) {
                    Ok(n) => {
                        ctx.memory[buf_addr..buf_addr + n].copy_from_slice(&ctx.io_buffer[..n]);
                        n as u64
                    }
                    Err(_) => u64::MAX,
                }
            } else {
                u64::MAX
            }
        }
        Some(NetOp::Close) => {
            let fd = ctx.get_reg(instr.rs1 as usize);
            match ctx.io_runtime.net_close(fd) {
                Ok(()) => 0,
                Err(_) => u64::MAX,
            }
        }
        None => u64::MAX,
    };
    ctx.set_reg(instr.rd as usize, result);
    ControlFlow::Continue
}

fn execute_net_setopt(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    let fd = ctx.get_reg(instr.rs1 as usize);
    let value = ctx.get_reg(instr.rs2 as usize);
    let result = match NetOption::from_u8(instr.mode) {
        Some(option) => match ctx.io_runtime.net_setopt(fd, option, value) {
            Ok(()) => 0,
            Err(_) => u64::MAX,
        },
        None => u64::MAX,
    };
    ctx.set_reg(instr.rd as usize, result);
    ControlFlow::Continue
}

// ============ Console I/O Operations ============

fn execute_io(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    match IoOp::from_u8(instr.mode) {
        Some(IoOp::Print) => {
            let addr = ctx.get_reg(instr.rs1 as usize) as usize;
            let len = ctx.get_reg(instr.rs2 as usize) as usize;
            if let Some(bytes) = ctx.memory_slice(addr, len) {
                if let Ok(s) = std::str::from_utf8(bytes) {
                    print!("{}", s);
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                    ctx.set_reg(instr.rd as usize, len as u64);
                } else {
                    ctx.set_reg(instr.rd as usize, u64::MAX);
                }
            } else {
                ctx.set_reg(instr.rd as usize, u64::MAX);
            }
        }
        Some(IoOp::ReadLine) => {
            let addr = ctx.get_reg(instr.rs1 as usize) as usize;
            let max_len = ctx.get_reg(instr.rs2 as usize) as usize;
            if let Some(buf) = ctx.memory_slice_mut(addr, max_len) {
                let mut input = String::new();
                if std::io::stdin().read_line(&mut input).is_ok() {
                    let bytes = input.as_bytes();
                    let copy_len = bytes.len().min(max_len);
                    buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
                    ctx.set_reg(instr.rd as usize, copy_len as u64);
                } else {
                    ctx.set_reg(instr.rd as usize, u64::MAX);
                }
            } else {
                ctx.set_reg(instr.rd as usize, u64::MAX);
            }
        }
        Some(IoOp::GetArgs) => {
            ctx.set_reg(instr.rd as usize, 0);
        }
        Some(IoOp::GetEnv) => {
            ctx.set_reg(instr.rd as usize, 0);
        }
        None => {
            ctx.set_reg(instr.rd as usize, u64::MAX);
        }
    }
    ControlFlow::Continue
}

// ============ Time Operations ============

fn execute_time(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    match TimeOp::from_u8(instr.mode) {
        Some(TimeOp::Now) => {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            ctx.set_reg(instr.rd as usize, now);
        }
        Some(TimeOp::Sleep) => {
            let ms = ctx.get_reg(instr.rs1 as usize);
            std::thread::sleep(std::time::Duration::from_millis(ms));
            ctx.set_reg(instr.rd as usize, 0);
        }
        Some(TimeOp::Monotonic) => {
            let ns = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            ctx.set_reg(instr.rd as usize, ns);
        }
        Some(TimeOp::Reserved) | None => {
            ctx.set_reg(instr.rd as usize, 0);
        }
    }
    ControlFlow::Continue
}

// ============ FPU Operations ============

fn execute_fpu(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    let src1 = f64::from_bits(ctx.get_reg(instr.rs1 as usize));
    let src2 = f64::from_bits(ctx.get_reg(instr.rs2 as usize));

    match FpuOp::from_u8(instr.mode) {
        // Arithmetic operations return f64 bit pattern
        Some(FpuOp::Fadd) => {
            ctx.set_reg(instr.rd as usize, (src1 + src2).to_bits());
        }
        Some(FpuOp::Fsub) => {
            ctx.set_reg(instr.rd as usize, (src1 - src2).to_bits());
        }
        Some(FpuOp::Fmul) => {
            ctx.set_reg(instr.rd as usize, (src1 * src2).to_bits());
        }
        Some(FpuOp::Fdiv) => {
            ctx.set_reg(instr.rd as usize, (src1 / src2).to_bits());
        }
        Some(FpuOp::Fsqrt) => {
            ctx.set_reg(instr.rd as usize, src1.sqrt().to_bits());
        }
        Some(FpuOp::Fabs) => {
            ctx.set_reg(instr.rd as usize, src1.abs().to_bits());
        }
        Some(FpuOp::Ffloor) => {
            ctx.set_reg(instr.rd as usize, src1.floor().to_bits());
        }
        Some(FpuOp::Fceil) => {
            ctx.set_reg(instr.rd as usize, src1.ceil().to_bits());
        }
        // Comparison operations return integer 1 or 0
        Some(FpuOp::Fcmpeq) => {
            ctx.set_reg(instr.rd as usize, if src1 == src2 { 1 } else { 0 });
        }
        Some(FpuOp::Fcmpne) => {
            ctx.set_reg(instr.rd as usize, if src1 != src2 { 1 } else { 0 });
        }
        Some(FpuOp::Fcmplt) => {
            ctx.set_reg(instr.rd as usize, if src1 < src2 { 1 } else { 0 });
        }
        Some(FpuOp::Fcmple) => {
            ctx.set_reg(instr.rd as usize, if src1 <= src2 { 1 } else { 0 });
        }
        Some(FpuOp::Fcmpgt) => {
            ctx.set_reg(instr.rd as usize, if src1 > src2 { 1 } else { 0 });
        }
        Some(FpuOp::Fcmpge) => {
            ctx.set_reg(instr.rd as usize, if src1 >= src2 { 1 } else { 0 });
        }
        None => {
            ctx.set_reg(instr.rd as usize, 0);
        }
    }
    ControlFlow::Continue
}

// ============ Random Operations ============

fn execute_rand(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    match RandOp::from_u8(instr.mode) {
        Some(RandOp::RandBytes) => {
            let addr = ctx.get_reg(instr.rs1 as usize) as usize;
            let len = ctx.get_reg(instr.rs2 as usize) as usize;
            if let Some(buf) = ctx.memory_slice_mut(addr, len) {
                // Use a simple PRNG (for actual crypto, use proper CSPRNG)
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64;
                let mut state = seed;
                for byte in buf.iter_mut() {
                    state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                    *byte = (state >> 33) as u8;
                }
                ctx.set_reg(instr.rd as usize, len as u64);
            } else {
                ctx.set_reg(instr.rd as usize, u64::MAX);
            }
        }
        Some(RandOp::RandU64) => {
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            let rand = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            ctx.set_reg(instr.rd as usize, rand);
        }
        None => {
            ctx.set_reg(instr.rd as usize, 0);
        }
    }
    ControlFlow::Continue
}

// ============ Bit Operations ============

fn execute_bits(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    let src = ctx.get_reg(instr.rs1 as usize);
    let result = match BitsOp::from_u8(instr.mode) {
        Some(BitsOp::Popcount) => src.count_ones() as u64,
        Some(BitsOp::Clz) => src.leading_zeros() as u64,
        Some(BitsOp::Ctz) => src.trailing_zeros() as u64,
        Some(BitsOp::Bswap) => src.swap_bytes(),
        None => 0,
    };
    ctx.set_reg(instr.rd as usize, result);
    ControlFlow::Continue
}

// ============ Move Operations ============

fn execute_mov(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    if instr.rs1 as usize != 31 {
        // Register-to-register move: rd = rs1
        let value = ctx.get_reg(instr.rs1 as usize);
        ctx.set_reg(instr.rd as usize, value);
    } else if let Some(imm) = instr.imm {
        // Load immediate: rd = imm (sign-extended)
        ctx.set_reg(instr.rd as usize, imm as i64 as u64);
    }
    ControlFlow::Continue
}

// ============ Trap Operations ============

fn execute_trap(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    let trap_type = TrapType::from_u8(instr.mode).unwrap_or(TrapType::Syscall);
    ctx.error = Some(format!("Trap: {:?}", trap_type));
    ControlFlow::Error
}

fn execute_extcall(ctx: &mut JitContext, instr: &Instruction) -> ControlFlow {
    // Extension ID comes from the immediate field
    let ext_id = instr.imm.unwrap_or(0) as u32;

    // Collect arguments from registers (rs1, rs2, and r3, r4 for additional args)
    let args = [
        ctx.get_reg(instr.rs1 as usize),
        ctx.get_reg(instr.rs2 as usize),
        ctx.get_reg(3), // r3 for additional arguments
        ctx.get_reg(4), // r4 for additional arguments
    ];
    let mut outputs = [0u64; 4];

    // Call the extension registry
    match ctx.extensions.call(ext_id, &args, &mut outputs) {
        Ok(result) => {
            ctx.set_reg(instr.rd as usize, result as u64);
            ControlFlow::Continue
        }
        Err(e) => {
            ctx.error = Some(format!("ExtCall {} failed: {}", ext_id, e));
            ControlFlow::Error
        }
    }
}
