//! Window functions for VimL.
//!
//! This module implements window-related functions from `src/nvim/eval/funcs.c`:
//! - Window identification helpers
//! - Window number/id conversion utilities
//!
//! ## Note
//!
//! These are helper functions that work with window identifiers.
//! The actual window operations require C FFI calls that access window state.

#![allow(clippy::must_use_candidate)]

use std::ffi::c_int;

// =============================================================================
// Window Identifier Types
// =============================================================================

/// Window identifier types in VimL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum WinIdType {
    /// Window number (1-based in tab)
    Number = 0,
    /// Window ID (global unique)
    Id = 1,
    /// Current window
    Current = 2,
    /// Invalid window
    Invalid = -1,
}

impl WinIdType {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Number,
            1 => Self::Id,
            2 => Self::Current,
            _ => Self::Invalid,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Window Number Helpers
// =============================================================================

/// Special window number values.
pub mod special_winnr {
    /// Current window
    pub const CURRENT: i64 = 0;
    /// Previous window (#)
    pub const PREVIOUS: i64 = -1;
}

/// Check if a window number is valid (positive).
pub const fn is_valid_winnr(winnr: i64) -> bool {
    winnr > 0
}

/// Check if a window ID is valid (positive).
pub const fn is_valid_winid(winid: i64) -> bool {
    winid > 0
}

/// FFI export: check if window number is valid.
#[no_mangle]
pub extern "C" fn rs_win_is_valid_winnr(winnr: i64) -> bool {
    is_valid_winnr(winnr)
}

/// FFI export: check if window ID is valid.
#[no_mangle]
pub extern "C" fn rs_win_is_valid_winid(winid: i64) -> bool {
    is_valid_winid(winid)
}

// =============================================================================
// Window Position Helpers
// =============================================================================

/// Window position relative to viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum WinRelPos {
    /// Absolute position
    Absolute = 0,
    /// Relative to cursor
    Cursor = 1,
    /// Relative to window
    Window = 2,
    /// Relative to editor grid
    Editor = 3,
}

impl WinRelPos {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Cursor,
            2 => Self::Window,
            3 => Self::Editor,
            _ => Self::Absolute,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Window Layout Helpers
// =============================================================================

/// Window split direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum WinSplitDir {
    /// Split horizontally (above/below)
    Horizontal = 0,
    /// Split vertically (left/right)
    Vertical = 1,
}

impl WinSplitDir {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Vertical,
            _ => Self::Horizontal,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_win_id_type() {
        assert_eq!(WinIdType::from_c_int(0), WinIdType::Number);
        assert_eq!(WinIdType::from_c_int(-1), WinIdType::Invalid);
    }

    #[test]
    fn test_is_valid_winnr() {
        assert!(is_valid_winnr(1));
        assert!(!is_valid_winnr(0));
        assert!(!is_valid_winnr(-1));
    }

    #[test]
    fn test_is_valid_winid() {
        assert!(is_valid_winid(1000));
        assert!(!is_valid_winid(0));
        assert!(!is_valid_winid(-1));
    }
}
