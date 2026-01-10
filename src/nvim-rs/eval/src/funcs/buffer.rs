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
// Buffer Info Types
// =============================================================================

/// Buffer type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum BufType {
    /// Normal buffer
    Normal = 0,
    /// Help buffer
    Help = 1,
    /// Quickfix buffer
    Quickfix = 2,
    /// Terminal buffer
    Terminal = 3,
    /// Prompt buffer
    Prompt = 4,
    /// Popup buffer
    Popup = 5,
    /// Scratch buffer (nofile, noswap)
    Scratch = 6,
}

impl BufType {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Help,
            2 => Self::Quickfix,
            3 => Self::Terminal,
            4 => Self::Prompt,
            5 => Self::Popup,
            6 => Self::Scratch,
            _ => Self::Normal,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if buffer type is special (non-file).
    pub const fn is_special(self) -> bool {
        !matches!(self, Self::Normal)
    }
}

/// FFI: Check if buffer type is special.
#[no_mangle]
pub extern "C" fn rs_buf_type_is_special(buf_type: c_int) -> c_int {
    c_int::from(BufType::from_c_int(buf_type).is_special())
}

// =============================================================================
// Buffer State Flags
// =============================================================================

/// Buffer state flags.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BufStateFlags {
    /// Buffer is modified
    pub modified: bool,
    /// Buffer is read-only
    pub readonly: bool,
    /// Buffer is listed
    pub listed: bool,
    /// Buffer is loaded
    pub loaded: bool,
    /// Buffer has a file name
    pub named: bool,
    /// Buffer has a swap file
    pub swapfile: bool,
}

impl BufStateFlags {
    /// Create new flags with all false.
    pub const fn new() -> Self {
        Self {
            modified: false,
            readonly: false,
            listed: true,
            loaded: false,
            named: false,
            swapfile: true,
        }
    }
}

/// FFI: Create default buffer state flags.
#[no_mangle]
pub extern "C" fn rs_buf_state_flags_new() -> BufStateFlags {
    BufStateFlags::new()
}

// =============================================================================
// Line Range Operations
// =============================================================================

/// Clamp a line number to valid range.
///
/// # Arguments
/// * `lnum` - Line number to clamp
/// * `line_count` - Total lines in buffer
///
/// # Returns
/// Clamped line number (1 <= result <= line_count)
pub const fn clamp_lnum(lnum: i64, line_count: i64) -> i64 {
    if lnum < 1 {
        1
    } else if lnum > line_count {
        line_count
    } else {
        lnum
    }
}

/// FFI: Clamp line number.
#[no_mangle]
pub extern "C" fn rs_buf_clamp_lnum(lnum: i64, line_count: i64) -> i64 {
    clamp_lnum(lnum, line_count)
}

/// Calculate line range for operations like deletebufline().
///
/// # Arguments
/// * `first` - First line (1-based)
/// * `last` - Last line (1-based, or 0 for same as first)
/// * `line_count` - Total lines in buffer
///
/// # Returns
/// (start, end) tuple with validated range, or None if invalid
pub const fn calc_line_range(first: i64, last: i64, line_count: i64) -> Option<(i64, i64)> {
    if first < 1 || first > line_count {
        return None;
    }

    let end = if last == 0 { first } else { last };

    if end < first || end > line_count {
        return None;
    }

    Some((first, end))
}

/// FFI: Calculate line range start.
#[no_mangle]
pub extern "C" fn rs_buf_calc_range_start(first: i64, last: i64, line_count: i64) -> i64 {
    calc_line_range(first, last, line_count).map_or(-1, |(s, _)| s)
}

/// FFI: Calculate line range end.
#[no_mangle]
pub extern "C" fn rs_buf_calc_range_end(first: i64, last: i64, line_count: i64) -> i64 {
    calc_line_range(first, last, line_count).map_or(-1, |(_, e)| e)
}

// =============================================================================
// Buffer Variable Helpers
// =============================================================================

/// Check if a variable name is valid for buffer variables.
///
/// Buffer variable names must start with a letter and contain only
/// alphanumeric characters and underscores.
pub fn is_valid_bufvar_name(name: &[u8]) -> bool {
    if name.is_empty() {
        return false;
    }

    // First character must be a letter
    if !name[0].is_ascii_alphabetic() {
        return false;
    }

    // Rest must be alphanumeric or underscore
    name[1..]
        .iter()
        .all(|&c| c.is_ascii_alphanumeric() || c == b'_')
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

    #[test]
    fn test_buf_type() {
        assert_eq!(BufType::from_c_int(0), BufType::Normal);
        assert_eq!(BufType::from_c_int(3), BufType::Terminal);
        assert!(!BufType::Normal.is_special());
        assert!(BufType::Help.is_special());
        assert!(BufType::Terminal.is_special());
    }

    #[test]
    fn test_clamp_lnum() {
        assert_eq!(clamp_lnum(5, 10), 5);
        assert_eq!(clamp_lnum(0, 10), 1);
        assert_eq!(clamp_lnum(-5, 10), 1);
        assert_eq!(clamp_lnum(15, 10), 10);
    }

    #[test]
    fn test_calc_line_range() {
        // Valid ranges
        assert_eq!(calc_line_range(1, 5, 10), Some((1, 5)));
        assert_eq!(calc_line_range(5, 0, 10), Some((5, 5))); // last=0 means same as first
        assert_eq!(calc_line_range(1, 10, 10), Some((1, 10)));

        // Invalid ranges
        assert_eq!(calc_line_range(0, 5, 10), None); // first < 1
        assert_eq!(calc_line_range(11, 15, 10), None); // first > line_count
        assert_eq!(calc_line_range(5, 3, 10), None); // end < first
        assert_eq!(calc_line_range(5, 15, 10), None); // end > line_count
    }

    #[test]
    fn test_is_valid_bufvar_name() {
        assert!(is_valid_bufvar_name(b"foo"));
        assert!(is_valid_bufvar_name(b"myVar123"));
        assert!(is_valid_bufvar_name(b"my_var"));
        assert!(!is_valid_bufvar_name(b""));
        assert!(!is_valid_bufvar_name(b"123var")); // starts with digit
        assert!(!is_valid_bufvar_name(b"_var")); // starts with underscore
    }
}
