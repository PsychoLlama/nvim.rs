//! Cursor and position functions for VimL.
//!
//! This module implements cursor/position-related functions from `src/nvim/eval/funcs.c`:
//! - Position types and helpers
//! - Cursor movement validation
//! - Mark position handling
//!
//! ## Note
//!
//! These are helper functions that work with positions.
//! The actual cursor operations require C FFI calls that access editor state.

#![allow(clippy::must_use_candidate)]

use std::ffi::c_int;

// =============================================================================
// Position Types
// =============================================================================

/// Position in a buffer (line, column).
///
/// VimL positions are 1-based for both line and column.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    /// Line number (1-based)
    pub lnum: i64,
    /// Column number (1-based, byte offset)
    pub col: i64,
    /// Column offset for virtual columns
    pub coladd: i64,
}

impl Position {
    /// Create a new position.
    pub const fn new(lnum: i64, col: i64) -> Self {
        Self {
            lnum,
            col,
            coladd: 0,
        }
    }

    /// Create a position with column offset.
    pub const fn with_coladd(lnum: i64, col: i64, coladd: i64) -> Self {
        Self { lnum, col, coladd }
    }

    /// Check if position is valid (positive line and column).
    pub const fn is_valid(&self) -> bool {
        self.lnum >= 1 && self.col >= 0
    }

    /// Create an invalid position marker.
    pub const fn invalid() -> Self {
        Self {
            lnum: 0,
            col: 0,
            coladd: 0,
        }
    }
}

/// Position type (for getpos/setpos).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum PosType {
    /// Cursor position (.)
    Cursor = 0,
    /// Mark position (')
    Mark = 1,
    /// Visual start (<)
    VisualStart = 2,
    /// Visual end (>)
    VisualEnd = 3,
    /// Insert start ([)
    InsertStart = 4,
    /// Insert end (])
    InsertEnd = 5,
    /// Last change start (')
    ChangeStart = 6,
    /// Last change end (')
    ChangeEnd = 7,
}

impl PosType {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Cursor),
            1 => Some(Self::Mark),
            2 => Some(Self::VisualStart),
            3 => Some(Self::VisualEnd),
            4 => Some(Self::InsertStart),
            5 => Some(Self::InsertEnd),
            6 => Some(Self::ChangeStart),
            7 => Some(Self::ChangeEnd),
            _ => None,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Position Validation
// =============================================================================

/// Validate a line number for buffer with given line count.
pub const fn validate_lnum(lnum: i64, line_count: i64) -> bool {
    lnum >= 1 && lnum <= line_count
}

/// Validate a column number for line with given byte length.
pub const fn validate_col(col: i64, line_len: i64) -> bool {
    col >= 0 && col <= line_len
}

/// Validate a position for buffer.
pub const fn validate_position(pos: &Position, line_count: i64, line_len: i64) -> bool {
    validate_lnum(pos.lnum, line_count) && validate_col(pos.col, line_len)
}

/// FFI export: create a cursor position.
#[no_mangle]
pub extern "C" fn rs_cursor_pos_new(lnum: i64, col: i64, coladd: i64) -> Position {
    Position::with_coladd(lnum, col, coladd)
}

/// FFI export: check if cursor position is valid.
///
/// # Safety
/// - `pos` must be a valid pointer to a Position struct, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_pos_is_valid(pos: *const Position) -> bool {
    if pos.is_null() {
        return false;
    }
    unsafe { (*pos).is_valid() }
}

// =============================================================================
// Line/Column Helpers
// =============================================================================

/// Special line values for line() function.
pub mod special_line {
    /// Current line (.)
    pub const CURRENT: i64 = -1;
    /// Last line ($)
    pub const LAST: i64 = -2;
}

/// Get special line number meaning.
pub const fn resolve_special_line(spec: i64, current: i64, last: i64) -> i64 {
    match spec {
        x if x == special_line::CURRENT => current,
        x if x == special_line::LAST => last,
        _ => spec,
    }
}

/// Clamp line number to valid range.
pub const fn clamp_lnum(lnum: i64, line_count: i64) -> i64 {
    if lnum < 1 {
        1
    } else if lnum > line_count {
        line_count
    } else {
        lnum
    }
}

/// Clamp column to valid range.
pub const fn clamp_col(col: i64, line_len: i64) -> i64 {
    if col < 0 {
        0
    } else if col > line_len {
        line_len
    } else {
        col
    }
}

// =============================================================================
// Mark Helpers
// =============================================================================

/// Check if a character is a valid mark name.
pub fn is_valid_mark(c: u8) -> bool {
    // Valid marks: a-z, A-Z, 0-9, and special marks
    c.is_ascii_alphanumeric()
        || matches!(
            c,
            b'\'' | b'`' | b'<' | b'>' | b'[' | b']' | b'^' | b'.' | b'"'
        )
}

/// Check if mark is a lowercase (buffer-local) mark.
pub const fn is_local_mark(c: u8) -> bool {
    c.is_ascii_lowercase()
}

/// Check if mark is an uppercase (global) mark.
pub const fn is_global_mark(c: u8) -> bool {
    c.is_ascii_uppercase()
}

/// Check if mark is a numbered mark.
pub const fn is_numbered_mark(c: u8) -> bool {
    c.is_ascii_digit()
}

/// FFI export: check valid mark character.
#[no_mangle]
pub extern "C" fn rs_is_valid_mark(c: u8) -> bool {
    is_valid_mark(c)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position() {
        let pos = Position::new(1, 0);
        assert!(pos.is_valid());

        let pos = Position::new(0, 0);
        assert!(!pos.is_valid());

        let pos = Position::with_coladd(10, 5, 2);
        assert_eq!(pos.lnum, 10);
        assert_eq!(pos.col, 5);
        assert_eq!(pos.coladd, 2);
    }

    #[test]
    fn test_pos_type() {
        assert_eq!(PosType::from_c_int(0), Some(PosType::Cursor));
        assert_eq!(PosType::from_c_int(99), None);
    }

    #[test]
    fn test_validate() {
        // 10 lines in buffer, current line has 20 bytes
        assert!(validate_lnum(1, 10));
        assert!(validate_lnum(10, 10));
        assert!(!validate_lnum(0, 10));
        assert!(!validate_lnum(11, 10));

        assert!(validate_col(0, 20));
        assert!(validate_col(20, 20));
        assert!(!validate_col(-1, 20));
        assert!(!validate_col(21, 20));
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp_lnum(0, 10), 1);
        assert_eq!(clamp_lnum(5, 10), 5);
        assert_eq!(clamp_lnum(15, 10), 10);

        assert_eq!(clamp_col(-1, 20), 0);
        assert_eq!(clamp_col(10, 20), 10);
        assert_eq!(clamp_col(25, 20), 20);
    }

    #[test]
    fn test_marks() {
        assert!(is_valid_mark(b'a'));
        assert!(is_valid_mark(b'Z'));
        assert!(is_valid_mark(b'0'));
        assert!(is_valid_mark(b'\''));
        assert!(!is_valid_mark(b'@'));

        assert!(is_local_mark(b'a'));
        assert!(!is_local_mark(b'A'));

        assert!(is_global_mark(b'A'));
        assert!(!is_global_mark(b'a'));

        assert!(is_numbered_mark(b'5'));
        assert!(!is_numbered_mark(b'a'));
    }
}
