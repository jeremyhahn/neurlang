//! JIT Execution Context
//!
//! Holds all state needed for JIT execution including registers, memory, and I/O runtime.

use crate::ir::{Program, DATA_BASE};
use crate::runtime::extensions::ExtensionRegistry;
use crate::stencil::io::{IOPermissions, IORuntime};

/// Default I/O buffer size (64KB - covers most HTTP requests/responses)
const IO_BUFFER_SIZE: usize = 64 * 1024;

/// JIT execution context - passed to all JIT handlers
#[repr(C)]
pub struct JitContext {
    /// Register file (32 64-bit registers)
    pub registers: [u64; 32],
    /// Program counter (instruction index)
    pub pc: usize,
    /// Memory buffer for program use
    pub memory: Vec<u8>,
    /// Memory base pointer (for fast access)
    pub memory_ptr: *mut u8,
    /// Memory size
    pub memory_size: usize,
    /// I/O Runtime for file, network, console operations
    pub io_runtime: IORuntime,
    /// Extension registry for ext.call opcode
    pub extensions: ExtensionRegistry,
    /// Call stack for nested function calls
    pub call_stack: Vec<usize>,
    /// Halt flag
    pub halted: bool,
    /// Error message if any
    pub error: Option<String>,
    /// Instruction count (for debugging)
    pub instruction_count: u64,
    /// Pre-allocated I/O buffer (avoids heap allocations on every I/O call)
    pub io_buffer: Vec<u8>,
    /// Flag to track if data section has been loaded (for test memory setup)
    pub data_section_loaded: bool,
}

impl JitContext {
    /// Create a new JIT context with specified memory size
    pub fn new(memory_size: usize) -> Self {
        Self::with_permissions(memory_size, IOPermissions::allow_all())
    }

    /// Create a new JIT context with custom I/O permissions
    pub fn with_permissions(memory_size: usize, permissions: IOPermissions) -> Self {
        let mut memory = vec![0u8; memory_size];
        let memory_ptr = memory.as_mut_ptr();
        let memory_size = memory.len();

        Self {
            registers: [0; 32],
            pc: 0,
            memory,
            memory_ptr,
            memory_size,
            io_runtime: IORuntime::new(permissions),
            extensions: ExtensionRegistry::new(),
            call_stack: Vec::with_capacity(256), // Support up to 256 nested calls
            halted: false,
            error: None,
            instruction_count: 0,
            io_buffer: vec![0u8; IO_BUFFER_SIZE],
            data_section_loaded: false,
        }
    }

    /// Get the pre-allocated I/O buffer (resizes if needed)
    #[inline]
    pub fn get_io_buffer(&mut self, min_size: usize) -> &mut Vec<u8> {
        if self.io_buffer.len() < min_size {
            self.io_buffer.resize(min_size, 0);
        }
        &mut self.io_buffer
    }

    /// Load a program's data section into memory
    /// Only loads once - subsequent calls are no-ops (for test memory setup)
    pub fn load_data_section(&mut self, program: &Program) {
        // Skip if already loaded (allows test memory setup to persist)
        if self.data_section_loaded {
            return;
        }

        if !program.data_section.is_empty() {
            let data_start = DATA_BASE as usize;
            let data_end = data_start + program.data_section.len();

            // Ensure memory is large enough
            if data_end > self.memory.len() {
                self.memory.resize(data_end, 0);
                self.memory_ptr = self.memory.as_mut_ptr();
                self.memory_size = self.memory.len();
            }

            // Copy data section to memory
            self.memory[data_start..data_end].copy_from_slice(&program.data_section);
        }

        self.data_section_loaded = true;
    }

    /// Get a register value
    #[inline(always)]
    pub fn get_reg(&self, reg: usize) -> u64 {
        if reg == 31 {
            // Zero register
            0
        } else {
            self.registers[reg]
        }
    }

    /// Set a register value
    #[inline(always)]
    pub fn set_reg(&mut self, reg: usize, value: u64) {
        if reg < 31 {
            // Can't write to zero register (index 31)
            self.registers[reg] = value;
        }
    }

    /// Read from memory (64-bit)
    #[inline(always)]
    pub fn read_mem_u64(&self, addr: u64) -> Result<u64, &'static str> {
        let addr = addr as usize;
        if addr + 8 > self.memory_size {
            return Err("Memory read out of bounds");
        }
        unsafe {
            Ok(std::ptr::read_unaligned(
                self.memory_ptr.add(addr) as *const u64
            ))
        }
    }

    /// Write to memory (64-bit)
    #[inline(always)]
    pub fn write_mem_u64(&mut self, addr: u64, value: u64) -> Result<(), &'static str> {
        let addr = addr as usize;
        if addr + 8 > self.memory_size {
            return Err("Memory write out of bounds");
        }
        unsafe {
            std::ptr::write_unaligned(self.memory_ptr.add(addr) as *mut u64, value);
        }
        Ok(())
    }

    /// Read from memory with specified width
    #[inline]
    pub fn read_mem(&self, addr: u64, width: u8) -> Result<u64, &'static str> {
        let addr = addr as usize;
        let size = 1usize << width; // 0=1, 1=2, 2=4, 3=8
        if addr + size > self.memory_size {
            return Err("Memory read out of bounds");
        }
        unsafe {
            let ptr = self.memory_ptr.add(addr);
            Ok(match width {
                0 => *ptr as u64,
                1 => std::ptr::read_unaligned(ptr as *const u16) as u64,
                2 => std::ptr::read_unaligned(ptr as *const u32) as u64,
                3 => std::ptr::read_unaligned(ptr as *const u64),
                _ => 0,
            })
        }
    }

    /// Write to memory with specified width
    #[inline]
    pub fn write_mem(&mut self, addr: u64, value: u64, width: u8) -> Result<(), &'static str> {
        let addr = addr as usize;
        let size = 1usize << width;
        if addr + size > self.memory_size {
            return Err("Memory write out of bounds");
        }
        unsafe {
            let ptr = self.memory_ptr.add(addr);
            match width {
                0 => *ptr = value as u8,
                1 => std::ptr::write_unaligned(ptr as *mut u16, value as u16),
                2 => std::ptr::write_unaligned(ptr as *mut u32, value as u32),
                3 => std::ptr::write_unaligned(ptr as *mut u64, value),
                _ => {}
            }
        }
        Ok(())
    }

    /// Get a memory slice for I/O operations
    #[inline]
    pub fn memory_slice(&self, addr: usize, len: usize) -> Option<&[u8]> {
        if addr + len <= self.memory_size {
            Some(&self.memory[addr..addr + len])
        } else {
            None
        }
    }

    /// Get a mutable memory slice for I/O operations
    #[inline]
    pub fn memory_slice_mut(&mut self, addr: usize, len: usize) -> Option<&mut [u8]> {
        if addr + len <= self.memory_size {
            Some(&mut self.memory[addr..addr + len])
        } else {
            None
        }
    }

    /// Get string from memory (null-terminated or with length)
    pub fn get_string(&self, addr: usize, max_len: usize) -> Option<&str> {
        if addr >= self.memory_size {
            return None;
        }
        let end = (addr + max_len).min(self.memory_size);
        let bytes = &self.memory[addr..end];

        // Find null terminator
        let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        std::str::from_utf8(&bytes[..len]).ok()
    }
}

impl Default for JitContext {
    fn default() -> Self {
        // Default to 1MB memory with full I/O permissions
        Self::new(1024 * 1024)
    }
}
