//! Script sourcing operations
//!
//! This module handles sourcing Vim and Lua scripts.

use std::ffi::c_int;

use crate::{doso, LinenrT, ScidT};

// =============================================================================
// Source Flags
// =============================================================================

/// Check if sourcing a vimrc file.
pub fn rs_sourcing_vimrc(flags: c_int) -> bool {
    flags == doso::VIMRC
}

/// Check if this is a regular source (not vimrc).
pub fn rs_sourcing_regular(flags: c_int) -> bool {
    flags == doso::NONE
}

// =============================================================================
// Source State
// =============================================================================

/// Source file state for tracking line-by-line execution.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SourceState {
    /// Current line number being executed
    pub current_lnum: LinenrT,
    /// Total lines in the file
    pub total_lines: LinenrT,
    /// Script ID assigned to this source
    pub script_id: ScidT,
    /// Whether we're in breakpoint debug mode
    pub breakpoint: bool,
    /// Whether this is a Lua file
    pub is_lua: bool,
}

impl Default for SourceState {
    fn default() -> Self {
        Self {
            current_lnum: 1,
            total_lines: 0,
            script_id: 0,
            breakpoint: false,
            is_lua: false,
        }
    }
}

/// Create default source state.
pub fn rs_source_state_default() -> SourceState {
    SourceState::default()
}

/// Initialize source state for a script.
pub fn rs_source_state_init(script_id: ScidT, total_lines: LinenrT, is_lua: bool) -> SourceState {
    SourceState {
        current_lnum: 1,
        total_lines,
        script_id,
        breakpoint: false,
        is_lua,
    }
}

/// Advance to the next line.
pub fn rs_source_state_next_line(state: &mut SourceState) {
    state.current_lnum += 1;
}

/// Check if we're at the end of the file.
pub fn rs_source_state_at_end(state: &SourceState) -> bool {
    state.current_lnum > state.total_lines
}

/// Get progress percentage (0-100).
pub fn rs_source_state_progress(state: &SourceState) -> c_int {
    if state.total_lines <= 0 {
        return 100;
    }
    ((state.current_lnum * 100) / state.total_lines) as c_int
}

// =============================================================================
// Source Result
// =============================================================================

/// Result codes from do_source()
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceResult {
    /// Script sourced successfully
    Ok = 0,
    /// File not found
    FileNotFound = 1,
    /// Permission denied
    PermissionDenied = 2,
    /// Read error
    ReadError = 3,
    /// Script aborted (user interrupt)
    Aborted = 4,
    /// Error in script
    ScriptError = 5,
}

impl SourceResult {
    /// Convert to integer
    pub const fn as_int(self) -> c_int {
        self as c_int
    }

    /// Create from integer
    pub const fn from_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Ok),
            1 => Some(Self::FileNotFound),
            2 => Some(Self::PermissionDenied),
            3 => Some(Self::ReadError),
            4 => Some(Self::Aborted),
            5 => Some(Self::ScriptError),
            _ => None,
        }
    }

    /// Check if result is success
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }
}

/// Check if source result indicates success.
pub fn rs_source_result_ok(result: c_int) -> bool {
    result == SourceResult::Ok as c_int
}

/// Check if source result indicates file not found.
pub fn rs_source_result_not_found(result: c_int) -> bool {
    result == SourceResult::FileNotFound as c_int
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_flags() {
        assert!(!rs_sourcing_vimrc(doso::NONE));
        assert!(rs_sourcing_vimrc(doso::VIMRC));
        assert!(rs_sourcing_regular(doso::NONE));
        assert!(!rs_sourcing_regular(doso::VIMRC));
    }

    #[test]
    fn test_source_state() {
        let state = rs_source_state_init(1, 100, false);
        assert_eq!(state.current_lnum, 1);
        assert_eq!(state.total_lines, 100);
        assert_eq!(state.script_id, 1);
        assert!(!state.is_lua);
        assert!(!rs_source_state_at_end(&state));
    }

    #[test]
    fn test_source_state_progress() {
        let mut state = rs_source_state_init(1, 100, false);
        assert_eq!(rs_source_state_progress(&state), 1);

        state.current_lnum = 50;
        assert_eq!(rs_source_state_progress(&state), 50);

        state.current_lnum = 100;
        assert_eq!(rs_source_state_progress(&state), 100);

        // Empty file
        let empty = rs_source_state_init(1, 0, false);
        assert_eq!(rs_source_state_progress(&empty), 100);
    }

    #[test]
    fn test_source_result() {
        assert!(rs_source_result_ok(SourceResult::Ok as c_int));
        assert!(!rs_source_result_ok(SourceResult::FileNotFound as c_int));
        assert!(rs_source_result_not_found(
            SourceResult::FileNotFound as c_int
        ));
    }
}
