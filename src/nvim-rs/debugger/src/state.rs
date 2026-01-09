//! Debugger state management
//!
//! This module provides types for managing debugger state,
//! including session state, execution status, and debugging context.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::c_int;

/// Line number type
type LinenrT = i32;

// =============================================================================
// Debug Status
// =============================================================================

/// Current debug execution status
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DebugStatus {
    /// Not debugging
    #[default]
    Idle = 0,
    /// Preparing to debug
    Starting = 1,
    /// Running (executing code)
    Running = 2,
    /// Stopped (at breakpoint/step)
    Stopped = 3,
    /// Pausing
    Pausing = 4,
    /// Stepping
    Stepping = 5,
    /// Terminating
    Terminating = 6,
    /// Error state
    Error = 7,
}

impl DebugStatus {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Idle),
            1 => Some(Self::Starting),
            2 => Some(Self::Running),
            3 => Some(Self::Stopped),
            4 => Some(Self::Pausing),
            5 => Some(Self::Stepping),
            6 => Some(Self::Terminating),
            7 => Some(Self::Error),
            _ => None,
        }
    }

    /// Convert to raw value
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Check if actively debugging
    pub const fn is_active(self) -> bool {
        matches!(
            self,
            Self::Running | Self::Stopped | Self::Pausing | Self::Stepping
        )
    }

    /// Check if can issue control commands
    pub const fn can_control(self) -> bool {
        matches!(self, Self::Running | Self::Stopped)
    }

    /// Check if in a transitional state
    pub const fn is_transitional(self) -> bool {
        matches!(
            self,
            Self::Starting | Self::Pausing | Self::Stepping | Self::Terminating
        )
    }

    /// Check if stopped
    pub const fn is_stopped(self) -> bool {
        matches!(self, Self::Stopped)
    }
}

// =============================================================================
// Stop Reason
// =============================================================================

/// Reason for debugging to stop
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StopReason {
    /// Unknown reason
    #[default]
    Unknown = 0,
    /// Hit a breakpoint
    Breakpoint = 1,
    /// Step completed
    Step = 2,
    /// User paused execution
    Pause = 3,
    /// Exception occurred
    Exception = 4,
    /// Reached entry point
    Entry = 5,
    /// Goto target reached
    Goto = 6,
    /// Data breakpoint hit
    DataBreakpoint = 7,
    /// Instruction breakpoint hit
    InstructionBreakpoint = 8,
}

impl StopReason {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Unknown),
            1 => Some(Self::Breakpoint),
            2 => Some(Self::Step),
            3 => Some(Self::Pause),
            4 => Some(Self::Exception),
            5 => Some(Self::Entry),
            6 => Some(Self::Goto),
            7 => Some(Self::DataBreakpoint),
            8 => Some(Self::InstructionBreakpoint),
            _ => None,
        }
    }

    /// Get reason string
    pub const fn reason_string(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Breakpoint => "breakpoint",
            Self::Step => "step",
            Self::Pause => "pause",
            Self::Exception => "exception",
            Self::Entry => "entry",
            Self::Goto => "goto",
            Self::DataBreakpoint => "data breakpoint",
            Self::InstructionBreakpoint => "instruction breakpoint",
        }
    }

    /// Check if this is an error-like stop
    pub const fn is_error(self) -> bool {
        matches!(self, Self::Exception)
    }
}

// =============================================================================
// Stop Location
// =============================================================================

/// Location where debugger stopped
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StopLocation {
    /// Line number (1-based)
    pub line: LinenrT,
    /// Column number (0 = unknown)
    pub column: c_int,
    /// End line (for range)
    pub end_line: LinenrT,
    /// End column
    pub end_column: c_int,
}

impl Default for StopLocation {
    fn default() -> Self {
        Self {
            line: 0,
            column: 0,
            end_line: 0,
            end_column: 0,
        }
    }
}

impl StopLocation {
    /// Create a new location
    pub const fn new(line: LinenrT) -> Self {
        Self {
            line,
            column: 0,
            end_line: 0,
            end_column: 0,
        }
    }

    /// Create with column
    pub const fn with_column(line: LinenrT, column: c_int) -> Self {
        Self {
            line,
            column,
            end_line: 0,
            end_column: 0,
        }
    }

    /// Check if location is valid
    pub const fn is_valid(&self) -> bool {
        self.line > 0
    }
}

// =============================================================================
// Debugger State
// =============================================================================

/// Complete debugger state
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DebuggerState {
    /// Current status
    pub status: DebugStatus,
    /// Stop reason (when stopped)
    pub stop_reason: StopReason,
    /// Current location
    pub location: StopLocation,
    /// Current thread ID
    pub thread_id: c_int,
    /// Current frame ID
    pub frame_id: c_int,
    /// Number of active breakpoints
    pub breakpoint_count: c_int,
    /// Whether all threads are stopped
    pub all_threads_stopped: bool,
}

impl Default for DebuggerState {
    fn default() -> Self {
        Self {
            status: DebugStatus::Idle,
            stop_reason: StopReason::Unknown,
            location: StopLocation::default(),
            thread_id: 0,
            frame_id: 0,
            breakpoint_count: 0,
            all_threads_stopped: true,
        }
    }
}

impl DebuggerState {
    /// Create a new idle state
    pub const fn idle() -> Self {
        Self {
            status: DebugStatus::Idle,
            stop_reason: StopReason::Unknown,
            location: StopLocation::new(0),
            thread_id: 0,
            frame_id: 0,
            breakpoint_count: 0,
            all_threads_stopped: true,
        }
    }

    /// Create a stopped state
    pub const fn stopped(reason: StopReason, line: LinenrT, thread_id: c_int) -> Self {
        Self {
            status: DebugStatus::Stopped,
            stop_reason: reason,
            location: StopLocation::new(line),
            thread_id,
            frame_id: 0,
            breakpoint_count: 0,
            all_threads_stopped: true,
        }
    }

    /// Check if actively debugging
    pub const fn is_active(&self) -> bool {
        self.status.is_active()
    }

    /// Check if stopped
    pub const fn is_stopped(&self) -> bool {
        self.status.is_stopped()
    }

    /// Check if can continue
    pub const fn can_continue(&self) -> bool {
        self.status.is_stopped()
    }

    /// Check if can step
    pub const fn can_step(&self) -> bool {
        self.status.is_stopped()
    }

    /// Check if can pause
    pub const fn can_pause(&self) -> bool {
        matches!(self.status, DebugStatus::Running)
    }
}

// =============================================================================
// Thread State
// =============================================================================

/// Thread state during debugging
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    /// Running
    Running = 0,
    /// Stopped
    Stopped = 1,
    /// Exited
    Exited = 2,
}

impl ThreadState {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Running),
            1 => Some(Self::Stopped),
            2 => Some(Self::Exited),
            _ => None,
        }
    }
}

impl Default for ThreadState {
    fn default() -> Self {
        Self::Stopped
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if debug status is valid
#[no_mangle]
pub extern "C" fn rs_debug_status_valid(status: c_int) -> c_int {
    c_int::from(DebugStatus::from_raw(status).is_some())
}

/// FFI export: Check if debug status is active
#[no_mangle]
pub extern "C" fn rs_debug_status_is_active(status: c_int) -> c_int {
    DebugStatus::from_raw(status).map_or(0, |s| c_int::from(s.is_active()))
}

/// FFI export: Check if debug status can control
#[no_mangle]
pub extern "C" fn rs_debug_status_can_control(status: c_int) -> c_int {
    DebugStatus::from_raw(status).map_or(0, |s| c_int::from(s.can_control()))
}

/// FFI export: Check if stop reason is error
#[no_mangle]
pub extern "C" fn rs_debug_stop_reason_is_error(reason: c_int) -> c_int {
    StopReason::from_raw(reason).map_or(0, |r| c_int::from(r.is_error()))
}

/// FFI export: Create idle state
#[no_mangle]
pub extern "C" fn rs_debugger_state_idle() -> DebuggerState {
    DebuggerState::idle()
}

/// FFI export: Create stopped state
#[no_mangle]
pub extern "C" fn rs_debugger_state_stopped(
    reason: c_int,
    line: LinenrT,
    thread_id: c_int,
) -> DebuggerState {
    let reason = StopReason::from_raw(reason).unwrap_or(StopReason::Unknown);
    DebuggerState::stopped(reason, line, thread_id)
}

/// FFI export: Check if state can continue
#[no_mangle]
pub extern "C" fn rs_debugger_state_can_continue(state: *const DebuggerState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*state).can_continue() })
}

/// FFI export: Check if state can step
#[no_mangle]
pub extern "C" fn rs_debugger_state_can_step(state: *const DebuggerState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*state).can_step() })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_status() {
        assert_eq!(DebugStatus::from_raw(0), Some(DebugStatus::Idle));
        assert_eq!(DebugStatus::from_raw(100), None);

        assert!(!DebugStatus::Idle.is_active());
        assert!(DebugStatus::Running.is_active());
        assert!(DebugStatus::Stopped.is_active());

        assert!(!DebugStatus::Idle.can_control());
        assert!(DebugStatus::Running.can_control());
        assert!(DebugStatus::Stopped.can_control());
    }

    #[test]
    fn test_stop_reason() {
        assert_eq!(StopReason::Breakpoint.reason_string(), "breakpoint");
        assert!(StopReason::Exception.is_error());
        assert!(!StopReason::Breakpoint.is_error());
    }

    #[test]
    fn test_stop_location() {
        let loc = StopLocation::new(10);
        assert!(loc.is_valid());

        let invalid = StopLocation::default();
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_debugger_state() {
        let idle = DebuggerState::idle();
        assert!(!idle.is_active());
        assert!(!idle.can_continue());

        let stopped = DebuggerState::stopped(StopReason::Breakpoint, 10, 1);
        assert!(stopped.is_active());
        assert!(stopped.is_stopped());
        assert!(stopped.can_continue());
        assert!(stopped.can_step());
        assert!(!stopped.can_pause());
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_debug_status_valid(0), 1);
        assert_eq!(rs_debug_status_valid(100), 0);

        assert_eq!(rs_debug_status_is_active(2), 1); // Running
        assert_eq!(rs_debug_status_is_active(0), 0); // Idle

        assert_eq!(rs_debug_stop_reason_is_error(4), 1); // Exception
        assert_eq!(rs_debug_stop_reason_is_error(1), 0); // Breakpoint
    }
}
