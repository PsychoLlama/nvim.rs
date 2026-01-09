//! Text manipulation functions for VimL.
//!
//! This module implements text-related functions from `src/nvim/eval/funcs.c`:
//! - Line range operations
//! - Text insertion/deletion helpers
//! - Indent calculation
//!
//! ## Note
//!
//! These are helper functions for text operations.
//! Actual buffer modifications require C FFI calls that access buffer state.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]

use std::ffi::c_int;

// =============================================================================
// Line Range Types
// =============================================================================

/// Line range specification for operations like getline/setline.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineRange {
    /// Start line (1-based, inclusive)
    pub start: i64,
    /// End line (1-based, inclusive), 0 means same as start
    pub end: i64,
}

impl LineRange {
    /// Create a single-line range.
    pub const fn single(lnum: i64) -> Self {
        Self {
            start: lnum,
            end: 0,
        }
    }

    /// Create a multi-line range.
    pub const fn range(start: i64, end: i64) -> Self {
        Self { start, end }
    }

    /// Get effective end line (handles 0 meaning single line).
    pub const fn effective_end(&self) -> i64 {
        if self.end == 0 {
            self.start
        } else {
            self.end
        }
    }

    /// Check if this is a single-line range.
    pub const fn is_single(&self) -> bool {
        self.end == 0 || self.end == self.start
    }

    /// Get number of lines in range.
    pub const fn count(&self) -> i64 {
        let end = self.effective_end();
        if end >= self.start {
            end - self.start + 1
        } else {
            0
        }
    }

    /// Check if range is valid (positive lines, start <= end).
    pub const fn is_valid(&self) -> bool {
        self.start >= 1 && (self.end == 0 || self.end >= self.start)
    }
}

/// Append position for append() function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum AppendPos {
    /// Append after specified line
    After = 0,
    /// Insert before specified line
    Before = 1,
}

impl AppendPos {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Before,
            _ => Self::After,
        }
    }
}

// =============================================================================
// Text Operation Helpers
// =============================================================================

/// Validate line range against buffer line count.
pub const fn validate_range(range: &LineRange, line_count: i64) -> bool {
    if !range.is_valid() {
        return false;
    }
    let end = range.effective_end();
    range.start <= line_count && end <= line_count
}

/// Clamp line range to buffer bounds.
pub const fn clamp_range(range: &LineRange, line_count: i64) -> LineRange {
    let start = if range.start < 1 {
        1
    } else if range.start > line_count {
        line_count
    } else {
        range.start
    };

    let end = range.effective_end();
    let end = if end < start {
        start
    } else if end > line_count {
        line_count
    } else {
        end
    };

    LineRange {
        start,
        end: if end == start { 0 } else { end },
    }
}

/// FFI export: create single-line range.
#[no_mangle]
pub extern "C" fn rs_text_line_range_single(lnum: i64) -> LineRange {
    LineRange::single(lnum)
}

/// FFI export: create multi-line range.
#[no_mangle]
pub extern "C" fn rs_text_line_range(start: i64, end: i64) -> LineRange {
    LineRange::range(start, end)
}

/// FFI export: validate line range.
#[no_mangle]
pub extern "C" fn rs_text_validate_range(start: i64, end: i64, line_count: i64) -> bool {
    let range = LineRange::range(start, end);
    validate_range(&range, line_count)
}

// =============================================================================
// Indent Helpers
// =============================================================================

/// Count leading whitespace characters in a line.
pub fn count_indent_chars(line: &[u8]) -> usize {
    line.iter().take_while(|&&c| c == b' ' || c == b'\t').count()
}

/// Count leading whitespace as virtual columns.
///
/// Tabs expand to `tabstop` columns (aligned to tabstop boundary).
pub fn count_indent_width(line: &[u8], tabstop: usize) -> usize {
    let mut width = 0;
    for &c in line {
        match c {
            b' ' => width += 1,
            b'\t' => width = (width / tabstop + 1) * tabstop,
            _ => break,
        }
    }
    width
}

/// Create indent string of given width.
///
/// Uses tabs and spaces to create indent.
pub fn make_indent(width: usize, tabstop: usize, use_tabs: bool) -> Vec<u8> {
    if !use_tabs || tabstop == 0 {
        return vec![b' '; width];
    }

    let tabs = width / tabstop;
    let spaces = width % tabstop;
    let mut result = Vec::with_capacity(tabs + spaces);
    result.extend(std::iter::repeat_n(b'\t', tabs));
    result.extend(std::iter::repeat_n(b' ', spaces));
    result
}

/// FFI export: count indent width.
///
/// # Safety
/// - `line` must be valid pointer to `len` bytes, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_text_count_indent(line: *const u8, len: c_int, tabstop: c_int) -> c_int {
    if line.is_null() || len < 0 {
        return 0;
    }

    let slice = unsafe { std::slice::from_raw_parts(line, len as usize) };
    let tabstop = if tabstop <= 0 { 8 } else { tabstop as usize };
    count_indent_width(slice, tabstop) as c_int
}

// =============================================================================
// Text Content Helpers
// =============================================================================

/// Check if line is blank (empty or only whitespace).
pub fn is_blank_line(line: &[u8]) -> bool {
    line.iter().all(|&c| c.is_ascii_whitespace())
}

/// Check if line contains only whitespace before given column.
pub fn is_blank_before(line: &[u8], col: usize) -> bool {
    line.get(..col)
        .is_none_or(|s| s.iter().all(|&c| c.is_ascii_whitespace()))
}

/// Find first non-blank character position.
pub fn first_nonblank(line: &[u8]) -> Option<usize> {
    line.iter().position(|&c| !c.is_ascii_whitespace())
}

/// Find last non-blank character position.
pub fn last_nonblank(line: &[u8]) -> Option<usize> {
    line.iter().rposition(|&c| !c.is_ascii_whitespace())
}

/// FFI export: check if line is blank.
///
/// # Safety
/// - `line` must be valid pointer to `len` bytes, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_text_is_blank(line: *const u8, len: c_int) -> bool {
    if line.is_null() || len < 0 {
        return true;
    }

    let slice = unsafe { std::slice::from_raw_parts(line, len as usize) };
    is_blank_line(slice)
}

/// FFI export: find first non-blank position.
///
/// Returns -1 if line is all blank.
///
/// # Safety
/// - `line` must be valid pointer to `len` bytes, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_text_first_nonblank(line: *const u8, len: c_int) -> c_int {
    if line.is_null() || len < 0 {
        return -1;
    }

    let slice = unsafe { std::slice::from_raw_parts(line, len as usize) };
    first_nonblank(slice).map_or(-1, |p| p as c_int)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_range() {
        let single = LineRange::single(5);
        assert!(single.is_single());
        assert_eq!(single.effective_end(), 5);
        assert_eq!(single.count(), 1);
        assert!(single.is_valid());

        let range = LineRange::range(5, 10);
        assert!(!range.is_single());
        assert_eq!(range.effective_end(), 10);
        assert_eq!(range.count(), 6);
        assert!(range.is_valid());

        let invalid = LineRange::range(0, 5);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_validate_range() {
        let range = LineRange::range(1, 10);
        assert!(validate_range(&range, 100));
        assert!(validate_range(&range, 10));
        assert!(!validate_range(&range, 5));
    }

    #[test]
    fn test_clamp_range() {
        let range = LineRange::range(5, 15);
        let clamped = clamp_range(&range, 10);
        assert_eq!(clamped.start, 5);
        assert_eq!(clamped.effective_end(), 10);
    }

    #[test]
    fn test_count_indent() {
        assert_eq!(count_indent_chars(b"  hello"), 2);
        assert_eq!(count_indent_chars(b"\thello"), 1);
        assert_eq!(count_indent_chars(b"hello"), 0);

        assert_eq!(count_indent_width(b"    hello", 8), 4);
        assert_eq!(count_indent_width(b"\thello", 8), 8);
        assert_eq!(count_indent_width(b"  \thello", 8), 8);
    }

    #[test]
    fn test_make_indent() {
        assert_eq!(make_indent(4, 8, false), vec![b' '; 4]);
        assert_eq!(make_indent(8, 8, true), vec![b'\t']);
        assert_eq!(make_indent(10, 8, true), vec![b'\t', b' ', b' ']);
    }

    #[test]
    fn test_blank_line() {
        assert!(is_blank_line(b""));
        assert!(is_blank_line(b"   "));
        assert!(is_blank_line(b"\t\t"));
        assert!(!is_blank_line(b"  x"));
    }

    #[test]
    fn test_nonblank() {
        assert_eq!(first_nonblank(b"  hello"), Some(2));
        assert_eq!(first_nonblank(b"   "), None);
        assert_eq!(last_nonblank(b"hello  "), Some(4));
        assert_eq!(last_nonblank(b"   "), None);
    }
}
