//! Stencil module for copy-and-patch compilation
//!
//! Contains pre-compiled code stencils and runtime patching logic.

pub mod concurrency;
pub mod io;
pub mod security;
pub mod table;

pub use concurrency::{
    atomics, ChannelError, ChannelId, ConcurrencyError, ConcurrencyRuntime, TaskId, TaskState,
};
pub use io::{IOError, IOPermissions, IORuntime};
pub use security::{
    check_capability, check_exec, check_read, check_write, CapCheckResult, SecurityContext,
    TaintLevel, TaintTracker,
};
pub use table::{patch_stencil, PatchInfo, PatchKind, StencilEntry, StencilTable};
