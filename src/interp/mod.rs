//! Interpreter module
//!
//! Provides a fallback interpreter for small programs where compile overhead exceeds execution time.

pub mod coverage;
pub mod dispatch;

pub use crate::stencil::io::IOPermissions;
pub use coverage::{CoverageReport, CoverageTracker};
pub use dispatch::{
    execute_fast, execute_fast_with_extensions, execute_fast_with_permissions,
    execute_with_coverage, execute_with_mocks, ExtensionMock, InterpResult, Interpreter,
};
