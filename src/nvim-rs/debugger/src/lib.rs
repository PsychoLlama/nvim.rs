//! Debugger infrastructure
//!
//! This crate provides Rust implementations for debugging support,
//! including DAP (Debug Adapter Protocol), breakpoints, and stepping.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::use_self)]

pub mod breakpoint;
pub mod dap;
pub mod state;
pub mod stepping;

use std::ffi::c_int;

// Re-export key types
pub use breakpoint::{Breakpoint, BreakpointFlags, BreakpointType};
pub use dap::{DapMessage, DapMessageType, DapState};
pub use state::{DebuggerState, DebugStatus};
pub use stepping::{StepMode, StepResult};

// =============================================================================
// Constants
// =============================================================================

/// Maximum number of breakpoints
pub const MAX_BREAKPOINTS: usize = 1000;

/// Maximum stack depth for debugging
pub const MAX_STACK_DEPTH: c_int = 256;

// =============================================================================
// Debug Level
// =============================================================================

/// Debug output level
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DebugLevel {
    /// No debug output
    #[default]
    None = 0,
    /// Errors only
    Error = 1,
    /// Errors and warnings
    Warning = 2,
    /// Errors, warnings, and info
    Info = 3,
    /// All messages including debug
    Debug = 4,
    /// Trace level (very verbose)
    Trace = 5,
}

impl DebugLevel {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Error),
            2 => Some(Self::Warning),
            3 => Some(Self::Info),
            4 => Some(Self::Debug),
            5 => Some(Self::Trace),
            _ => None,
        }
    }

    /// Convert to raw value
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Check if level should show message
    pub const fn should_log(self, msg_level: DebugLevel) -> bool {
        (msg_level as c_int) <= (self as c_int)
    }
}

// =============================================================================
// Debug Event Type
// =============================================================================

/// Types of debugger events
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugEventType {
    /// Breakpoint hit
    BreakpointHit = 0,
    /// Step completed
    StepComplete = 1,
    /// Exception occurred
    Exception = 2,
    /// Program started
    Started = 3,
    /// Program stopped
    Stopped = 4,
    /// Program exited
    Exited = 5,
    /// Thread created
    ThreadCreated = 6,
    /// Thread exited
    ThreadExited = 7,
    /// Output produced
    Output = 8,
    /// Module loaded
    ModuleLoaded = 9,
}

impl DebugEventType {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::BreakpointHit),
            1 => Some(Self::StepComplete),
            2 => Some(Self::Exception),
            3 => Some(Self::Started),
            4 => Some(Self::Stopped),
            5 => Some(Self::Exited),
            6 => Some(Self::ThreadCreated),
            7 => Some(Self::ThreadExited),
            8 => Some(Self::Output),
            9 => Some(Self::ModuleLoaded),
            _ => None,
        }
    }

    /// Check if this is a stopping event
    pub const fn is_stopping(self) -> bool {
        matches!(
            self,
            Self::BreakpointHit | Self::StepComplete | Self::Exception | Self::Stopped
        )
    }

    /// Check if this terminates debugging
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Exited)
    }
}

// =============================================================================
// Debug Capability Flags
// =============================================================================

/// Debugger capability flags
pub const CAP_SUPPORTS_BREAKPOINTS: u32 = 0x0001;
pub const CAP_SUPPORTS_CONDITIONAL: u32 = 0x0002;
pub const CAP_SUPPORTS_HITCOUNT: u32 = 0x0004;
pub const CAP_SUPPORTS_LOGPOINTS: u32 = 0x0008;
pub const CAP_SUPPORTS_STEP_IN: u32 = 0x0010;
pub const CAP_SUPPORTS_STEP_OUT: u32 = 0x0020;
pub const CAP_SUPPORTS_STEP_BACK: u32 = 0x0040;
pub const CAP_SUPPORTS_GOTO: u32 = 0x0080;
pub const CAP_SUPPORTS_EVALUATE: u32 = 0x0100;
pub const CAP_SUPPORTS_MODULES: u32 = 0x0200;
pub const CAP_SUPPORTS_EXCEPTION: u32 = 0x0400;
pub const CAP_SUPPORTS_TERMINATE: u32 = 0x0800;

/// Debugger capabilities wrapper
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DebugCapabilities {
    flags: u32,
}

impl DebugCapabilities {
    /// Create with no capabilities
    pub const fn none() -> Self {
        Self { flags: 0 }
    }

    /// Create from raw flags
    pub const fn from_raw(flags: u32) -> Self {
        Self { flags }
    }

    /// Get raw flags
    pub const fn as_raw(self) -> u32 {
        self.flags
    }

    /// Check if breakpoints are supported
    pub const fn supports_breakpoints(self) -> bool {
        (self.flags & CAP_SUPPORTS_BREAKPOINTS) != 0
    }

    /// Check if conditional breakpoints are supported
    pub const fn supports_conditional(self) -> bool {
        (self.flags & CAP_SUPPORTS_CONDITIONAL) != 0
    }

    /// Check if hit count is supported
    pub const fn supports_hitcount(self) -> bool {
        (self.flags & CAP_SUPPORTS_HITCOUNT) != 0
    }

    /// Check if step in is supported
    pub const fn supports_step_in(self) -> bool {
        (self.flags & CAP_SUPPORTS_STEP_IN) != 0
    }

    /// Check if step out is supported
    pub const fn supports_step_out(self) -> bool {
        (self.flags & CAP_SUPPORTS_STEP_OUT) != 0
    }

    /// Check if step back is supported
    pub const fn supports_step_back(self) -> bool {
        (self.flags & CAP_SUPPORTS_STEP_BACK) != 0
    }

    /// Check if evaluate is supported
    pub const fn supports_evaluate(self) -> bool {
        (self.flags & CAP_SUPPORTS_EVALUATE) != 0
    }

    /// Check if terminate is supported
    pub const fn supports_terminate(self) -> bool {
        (self.flags & CAP_SUPPORTS_TERMINATE) != 0
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if debug level is valid
#[no_mangle]
pub extern "C" fn rs_debug_level_valid(level: c_int) -> c_int {
    c_int::from(DebugLevel::from_raw(level).is_some())
}

/// FFI export: Check if level should log
#[no_mangle]
pub extern "C" fn rs_debug_level_should_log(current: c_int, msg_level: c_int) -> c_int {
    let current = DebugLevel::from_raw(current).unwrap_or(DebugLevel::None);
    let msg = DebugLevel::from_raw(msg_level).unwrap_or(DebugLevel::Error);
    c_int::from(current.should_log(msg))
}

/// FFI export: Check if event type is stopping
#[no_mangle]
pub extern "C" fn rs_debug_event_is_stopping(event: c_int) -> c_int {
    DebugEventType::from_raw(event).map_or(0, |e| c_int::from(e.is_stopping()))
}

/// FFI export: Check if event type is terminal
#[no_mangle]
pub extern "C" fn rs_debug_event_is_terminal(event: c_int) -> c_int {
    DebugEventType::from_raw(event).map_or(0, |e| c_int::from(e.is_terminal()))
}

/// FFI export: Check if capabilities support breakpoints
#[no_mangle]
pub extern "C" fn rs_debug_caps_breakpoints(flags: u32) -> c_int {
    c_int::from(DebugCapabilities::from_raw(flags).supports_breakpoints())
}

/// FFI export: Check if capabilities support step in
#[no_mangle]
pub extern "C" fn rs_debug_caps_step_in(flags: u32) -> c_int {
    c_int::from(DebugCapabilities::from_raw(flags).supports_step_in())
}

/// FFI export: Check if capabilities support evaluate
#[no_mangle]
pub extern "C" fn rs_debug_caps_evaluate(flags: u32) -> c_int {
    c_int::from(DebugCapabilities::from_raw(flags).supports_evaluate())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_level() {
        assert_eq!(DebugLevel::from_raw(0), Some(DebugLevel::None));
        assert_eq!(DebugLevel::from_raw(4), Some(DebugLevel::Debug));
        assert_eq!(DebugLevel::from_raw(100), None);

        let level = DebugLevel::Warning;
        assert!(level.should_log(DebugLevel::Error));
        assert!(level.should_log(DebugLevel::Warning));
        assert!(!level.should_log(DebugLevel::Info));
    }

    #[test]
    fn test_debug_event_type() {
        assert!(DebugEventType::BreakpointHit.is_stopping());
        assert!(DebugEventType::StepComplete.is_stopping());
        assert!(!DebugEventType::Started.is_stopping());

        assert!(DebugEventType::Exited.is_terminal());
        assert!(!DebugEventType::Stopped.is_terminal());
    }

    #[test]
    fn test_debug_capabilities() {
        let caps = DebugCapabilities::none();
        assert!(!caps.supports_breakpoints());

        let caps = DebugCapabilities::from_raw(CAP_SUPPORTS_BREAKPOINTS | CAP_SUPPORTS_STEP_IN);
        assert!(caps.supports_breakpoints());
        assert!(caps.supports_step_in());
        assert!(!caps.supports_evaluate());
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_debug_level_valid(0), 1);
        assert_eq!(rs_debug_level_valid(100), 0);

        assert_eq!(rs_debug_level_should_log(2, 1), 1); // Warning level, Error msg
        assert_eq!(rs_debug_level_should_log(2, 3), 0); // Warning level, Info msg

        assert_eq!(rs_debug_event_is_stopping(0), 1);
        assert_eq!(rs_debug_event_is_stopping(5), 0);
    }
}
