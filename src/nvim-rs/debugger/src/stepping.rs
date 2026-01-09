//! Stepping control for debugging
//!
//! This module provides types for controlling stepping through code,
//! including step modes, granularity, and step results.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::c_int;

/// Line number type
type LinenrT = i32;

// =============================================================================
// Step Mode
// =============================================================================

/// Stepping mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StepMode {
    /// No stepping (continue)
    #[default]
    None = 0,
    /// Step into functions
    Into = 1,
    /// Step over functions
    Over = 2,
    /// Step out of current function
    Out = 3,
    /// Step backward
    Back = 4,
    /// Step to specific line
    Goto = 5,
    /// Run to cursor
    RunToCursor = 6,
}

impl StepMode {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Into),
            2 => Some(Self::Over),
            3 => Some(Self::Out),
            4 => Some(Self::Back),
            5 => Some(Self::Goto),
            6 => Some(Self::RunToCursor),
            _ => None,
        }
    }

    /// Convert to raw value
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Get DAP command for this mode
    pub const fn dap_command(self) -> &'static str {
        match self {
            Self::None => "continue",
            Self::Into => "stepIn",
            Self::Over => "next",
            Self::Out => "stepOut",
            Self::Back => "stepBack",
            Self::Goto => "goto",
            Self::RunToCursor => "continue",
        }
    }

    /// Check if this is a forward step
    pub const fn is_forward(self) -> bool {
        matches!(self, Self::Into | Self::Over | Self::Out)
    }

    /// Check if this is a backward step
    pub const fn is_backward(self) -> bool {
        matches!(self, Self::Back)
    }

    /// Check if this requires a target
    pub const fn needs_target(self) -> bool {
        matches!(self, Self::Goto | Self::RunToCursor)
    }
}

// =============================================================================
// Step Granularity
// =============================================================================

/// Step granularity (how much to step)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StepGranularity {
    /// Step by statement (default)
    #[default]
    Statement = 0,
    /// Step by line
    Line = 1,
    /// Step by instruction
    Instruction = 2,
}

impl StepGranularity {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Statement),
            1 => Some(Self::Line),
            2 => Some(Self::Instruction),
            _ => None,
        }
    }

    /// Get DAP granularity string
    pub const fn dap_string(self) -> &'static str {
        match self {
            Self::Statement => "statement",
            Self::Line => "line",
            Self::Instruction => "instruction",
        }
    }
}

// =============================================================================
// Step Request
// =============================================================================

/// Step request configuration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StepRequest {
    /// Step mode
    pub mode: StepMode,
    /// Step granularity
    pub granularity: StepGranularity,
    /// Thread to step
    pub thread_id: c_int,
    /// Target line (for Goto/RunToCursor)
    pub target_line: LinenrT,
    /// Whether to step single thread only
    pub single_thread: bool,
}

impl Default for StepRequest {
    fn default() -> Self {
        Self {
            mode: StepMode::Over,
            granularity: StepGranularity::Statement,
            thread_id: 0,
            target_line: 0,
            single_thread: false,
        }
    }
}

impl StepRequest {
    /// Create a step over request
    pub const fn step_over(thread_id: c_int) -> Self {
        Self {
            mode: StepMode::Over,
            granularity: StepGranularity::Statement,
            thread_id,
            target_line: 0,
            single_thread: false,
        }
    }

    /// Create a step into request
    pub const fn step_into(thread_id: c_int) -> Self {
        Self {
            mode: StepMode::Into,
            granularity: StepGranularity::Statement,
            thread_id,
            target_line: 0,
            single_thread: false,
        }
    }

    /// Create a step out request
    pub const fn step_out(thread_id: c_int) -> Self {
        Self {
            mode: StepMode::Out,
            granularity: StepGranularity::Statement,
            thread_id,
            target_line: 0,
            single_thread: false,
        }
    }

    /// Create a goto request
    pub const fn goto(thread_id: c_int, target_line: LinenrT) -> Self {
        Self {
            mode: StepMode::Goto,
            granularity: StepGranularity::Line,
            thread_id,
            target_line,
            single_thread: false,
        }
    }

    /// Check if request is valid
    pub const fn is_valid(&self) -> bool {
        if self.mode.needs_target() && self.target_line <= 0 {
            return false;
        }
        true
    }
}

// =============================================================================
// Step Result
// =============================================================================

/// Result of a step operation
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StepResult {
    /// Step completed successfully
    #[default]
    Completed = 0,
    /// Step is in progress
    InProgress = 1,
    /// Step was cancelled
    Cancelled = 2,
    /// Error during step
    Error = 3,
    /// No valid thread
    NoThread = 4,
    /// Breakpoint hit during step
    BreakpointHit = 5,
    /// Exception during step
    Exception = 6,
}

impl StepResult {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Completed),
            1 => Some(Self::InProgress),
            2 => Some(Self::Cancelled),
            3 => Some(Self::Error),
            4 => Some(Self::NoThread),
            5 => Some(Self::BreakpointHit),
            6 => Some(Self::Exception),
            _ => None,
        }
    }

    /// Convert to raw value
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Check if step finished (not in progress)
    pub const fn is_finished(self) -> bool {
        !matches!(self, Self::InProgress)
    }

    /// Check if step was successful
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Completed)
    }

    /// Check if step had an error
    pub const fn is_error(self) -> bool {
        matches!(self, Self::Error | Self::NoThread | Self::Exception)
    }
}

// =============================================================================
// Step State
// =============================================================================

/// Current stepping state
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StepState {
    /// Active step mode
    pub mode: StepMode,
    /// Thread being stepped
    pub thread_id: c_int,
    /// Starting line
    pub start_line: LinenrT,
    /// Target line (for Goto)
    pub target_line: LinenrT,
    /// Number of steps taken
    pub step_count: c_int,
    /// Whether stepping is active
    pub active: bool,
}

impl Default for StepState {
    fn default() -> Self {
        Self {
            mode: StepMode::None,
            thread_id: 0,
            start_line: 0,
            target_line: 0,
            step_count: 0,
            active: false,
        }
    }
}

impl StepState {
    /// Create inactive state
    pub const fn inactive() -> Self {
        Self {
            mode: StepMode::None,
            thread_id: 0,
            start_line: 0,
            target_line: 0,
            step_count: 0,
            active: false,
        }
    }

    /// Create active stepping state
    pub const fn active(mode: StepMode, thread_id: c_int, start_line: LinenrT) -> Self {
        Self {
            mode,
            thread_id,
            start_line,
            target_line: 0,
            step_count: 0,
            active: true,
        }
    }

    /// Check if stepping is active
    pub const fn is_active(&self) -> bool {
        self.active
    }

    /// Increment step count
    pub fn increment(&mut self) {
        self.step_count += 1;
    }

    /// Reset state
    pub fn reset(&mut self) {
        *self = Self::inactive();
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if step mode is valid
#[no_mangle]
pub extern "C" fn rs_step_mode_valid(mode: c_int) -> c_int {
    c_int::from(StepMode::from_raw(mode).is_some())
}

/// FFI export: Check if step mode is forward
#[no_mangle]
pub extern "C" fn rs_step_mode_is_forward(mode: c_int) -> c_int {
    StepMode::from_raw(mode).map_or(0, |m| c_int::from(m.is_forward()))
}

/// FFI export: Check if step mode needs target
#[no_mangle]
pub extern "C" fn rs_step_mode_needs_target(mode: c_int) -> c_int {
    StepMode::from_raw(mode).map_or(0, |m| c_int::from(m.needs_target()))
}

/// FFI export: Create step over request
#[no_mangle]
pub extern "C" fn rs_step_request_over(thread_id: c_int) -> StepRequest {
    StepRequest::step_over(thread_id)
}

/// FFI export: Create step into request
#[no_mangle]
pub extern "C" fn rs_step_request_into(thread_id: c_int) -> StepRequest {
    StepRequest::step_into(thread_id)
}

/// FFI export: Create step out request
#[no_mangle]
pub extern "C" fn rs_step_request_out(thread_id: c_int) -> StepRequest {
    StepRequest::step_out(thread_id)
}

/// FFI export: Check if step request is valid
#[no_mangle]
pub extern "C" fn rs_step_request_is_valid(req: *const StepRequest) -> c_int {
    if req.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*req).is_valid() })
}

/// FFI export: Check if step result is finished
#[no_mangle]
pub extern "C" fn rs_step_result_is_finished(result: c_int) -> c_int {
    StepResult::from_raw(result).map_or(0, |r| c_int::from(r.is_finished()))
}

/// FFI export: Check if step result is success
#[no_mangle]
pub extern "C" fn rs_step_result_is_success(result: c_int) -> c_int {
    StepResult::from_raw(result).map_or(0, |r| c_int::from(r.is_success()))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_mode() {
        assert_eq!(StepMode::from_raw(0), Some(StepMode::None));
        assert_eq!(StepMode::from_raw(100), None);

        assert!(StepMode::Into.is_forward());
        assert!(StepMode::Over.is_forward());
        assert!(!StepMode::Back.is_forward());

        assert!(StepMode::Back.is_backward());
        assert!(!StepMode::Into.is_backward());

        assert!(StepMode::Goto.needs_target());
        assert!(!StepMode::Into.needs_target());
    }

    #[test]
    fn test_step_granularity() {
        assert_eq!(StepGranularity::Statement.dap_string(), "statement");
        assert_eq!(StepGranularity::Line.dap_string(), "line");
    }

    #[test]
    fn test_step_request() {
        let over = StepRequest::step_over(1);
        assert!(over.is_valid());
        assert_eq!(over.mode, StepMode::Over);

        let goto = StepRequest::goto(1, 100);
        assert!(goto.is_valid());
        assert_eq!(goto.target_line, 100);

        let invalid_goto = StepRequest {
            mode: StepMode::Goto,
            target_line: 0,
            ..Default::default()
        };
        assert!(!invalid_goto.is_valid());
    }

    #[test]
    fn test_step_result() {
        assert!(StepResult::Completed.is_success());
        assert!(StepResult::Completed.is_finished());

        assert!(!StepResult::InProgress.is_finished());

        assert!(StepResult::Error.is_error());
        assert!(StepResult::Exception.is_error());
        assert!(!StepResult::Completed.is_error());
    }

    #[test]
    fn test_step_state() {
        let inactive = StepState::inactive();
        assert!(!inactive.is_active());

        let active = StepState::active(StepMode::Over, 1, 10);
        assert!(active.is_active());
        assert_eq!(active.start_line, 10);

        let mut state = StepState::active(StepMode::Into, 1, 1);
        state.increment();
        assert_eq!(state.step_count, 1);

        state.reset();
        assert!(!state.is_active());
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_step_mode_valid(1), 1);
        assert_eq!(rs_step_mode_valid(100), 0);

        assert_eq!(rs_step_mode_is_forward(1), 1); // Into
        assert_eq!(rs_step_mode_is_forward(4), 0); // Back

        assert_eq!(rs_step_result_is_finished(0), 1); // Completed
        assert_eq!(rs_step_result_is_finished(1), 0); // InProgress
    }
}
