//! Buffer functions for VimL.
//!
//! This module implements buffer-related functions from `src/nvim/eval/funcs.c`:
//! - Buffer identification helpers
//! - Buffer name/number conversion utilities
//!
//! ## Note
//!
//! These are helper functions that work with buffer identifiers.
//! The actual buffer operations require C FFI calls that access buffer state.

#![allow(clippy::must_use_candidate)]

use std::ffi::c_int;

// =============================================================================
// Buffer Identifier Types
// =============================================================================

/// Buffer identifier types in VimL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum BufIdType {
    /// Buffer number (integer)
    Number = 0,
    /// Buffer name (string pattern)
    Name = 1,
    /// Current buffer ($)
    Current = 2,
    /// Alternate buffer (#)
    Alternate = 3,
    /// Last buffer (%)
    Last = 4,
    /// Invalid buffer id
    Invalid = -1,
}

impl BufIdType {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Number,
            1 => Self::Name,
            2 => Self::Current,
            3 => Self::Alternate,
            4 => Self::Last,
            _ => Self::Invalid,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Buffer Existence Check Helpers
// =============================================================================

/// Special buffer number values.
pub mod special_bufnr {
    /// Current buffer
    pub const CURRENT: i64 = 0;
    /// Alternate buffer
    pub const ALTERNATE: i64 = -1;
    /// No buffer / invalid
    pub const NONE: i64 = -2;
}

/// Check if a buffer number is valid (positive).
pub const fn is_valid_bufnr(bufnr: i64) -> bool {
    bufnr > 0
}

/// Check if a buffer number is a special value.
pub const fn is_special_bufnr(bufnr: i64) -> bool {
    bufnr <= 0
}

/// FFI export: check if buffer number is valid.
#[no_mangle]
pub extern "C" fn rs_buf_is_valid_bufnr(bufnr: i64) -> bool {
    is_valid_bufnr(bufnr)
}

/// FFI export: check if buffer number is special.
#[no_mangle]
pub extern "C" fn rs_buf_is_special_bufnr(bufnr: i64) -> bool {
    is_special_bufnr(bufnr)
}

// =============================================================================
// Buffer Line Operations Helpers
// =============================================================================

/// Validate line number for a buffer.
///
/// VimL line numbers are 1-based.
/// Returns true if the line number is valid for a buffer with `line_count` lines.
pub const fn is_valid_lnum(lnum: i64, line_count: i64) -> bool {
    lnum >= 1 && lnum <= line_count
}

/// Normalize line number (handle special values).
///
/// VimL special line values:
/// - 0: means line before first line (invalid in most contexts)
/// - $: means last line (represented as large negative or special value)
/// - .: means current line
pub const fn normalize_lnum(lnum: i64, line_count: i64, _current_line: i64) -> Option<i64> {
    if lnum == 0 {
        // Line 0 is usually invalid
        return None;
    }
    if lnum < 0 {
        // Negative could mean from end, but VimL doesn't support this for lines
        return None;
    }
    if lnum > line_count {
        // Beyond end of buffer
        return None;
    }
    Some(lnum)
}

/// FFI export: validate line number.
#[no_mangle]
pub extern "C" fn rs_buf_is_valid_lnum(lnum: i64, line_count: i64) -> bool {
    is_valid_lnum(lnum, line_count)
}

// =============================================================================
// Buffer Option Helpers
// =============================================================================

/// Buffer-local option scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum BufOptScope {
    /// Buffer-local value
    Local = 0,
    /// Global default
    Global = 1,
}

impl BufOptScope {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Global,
            _ => Self::Local,
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
    fn test_buf_id_type() {
        assert_eq!(BufIdType::from_c_int(0), BufIdType::Number);
        assert_eq!(BufIdType::from_c_int(-1), BufIdType::Invalid);
    }

    #[test]
    fn test_is_valid_bufnr() {
        assert!(is_valid_bufnr(1));
        assert!(is_valid_bufnr(100));
        assert!(!is_valid_bufnr(0));
        assert!(!is_valid_bufnr(-1));
    }

    #[test]
    fn test_is_special_bufnr() {
        assert!(is_special_bufnr(0));
        assert!(is_special_bufnr(-1));
        assert!(!is_special_bufnr(1));
    }

    #[test]
    fn test_is_valid_lnum() {
        // Buffer with 10 lines
        assert!(is_valid_lnum(1, 10));
        assert!(is_valid_lnum(10, 10));
        assert!(!is_valid_lnum(0, 10));
        assert!(!is_valid_lnum(11, 10));
        assert!(!is_valid_lnum(-1, 10));
    }

    #[test]
    fn test_normalize_lnum() {
        // Buffer with 10 lines, current line 5
        assert_eq!(normalize_lnum(1, 10, 5), Some(1));
        assert_eq!(normalize_lnum(10, 10, 5), Some(10));
        assert_eq!(normalize_lnum(0, 10, 5), None);
        assert_eq!(normalize_lnum(11, 10, 5), None);
        assert_eq!(normalize_lnum(-1, 10, 5), None);
    }
}
