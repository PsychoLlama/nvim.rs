//! Mark-related Ex commands (:marks, :delmarks, :jumps, :changes)
//!
//! This module implements Ex commands for working with marks, jump lists,
//! and change lists.

use std::ffi::{c_char, c_int};

// =============================================================================
// Mark Types
// =============================================================================

/// Type of mark being operated on.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarkType {
    /// Named marks (a-z, A-Z)
    #[default]
    Named = 0,
    /// Numbered marks (0-9)
    Numbered = 1,
    /// Special marks (', `, [, ], <, >, ^, .)
    Special = 2,
    /// File marks (only relevant for global marks)
    File = 3,
}

impl MarkType {
    /// Create from mark character.
    #[must_use]
    pub const fn from_char(c: u8) -> Self {
        match c {
            b'a'..=b'z' | b'A'..=b'Z' => Self::Named,
            b'0'..=b'9' => Self::Numbered,
            b'\'' | b'`' | b'[' | b']' | b'<' | b'>' | b'^' | b'.' | b'"' => Self::Special,
            _ => Self::Named,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Mark Validation
// =============================================================================

/// Check if a character is a valid mark name.
#[must_use]
pub const fn is_valid_mark(c: u8) -> bool {
    matches!(
        c,
        b'a'..=b'z'
            | b'A'..=b'Z'
            | b'0'..=b'9'
            | b'\''
            | b'`'
            | b'['
            | b']'
            | b'<'
            | b'>'
            | b'^'
            | b'.'
            | b'"'
    )
}

/// Check if a character is a lowercase mark (local to buffer).
#[must_use]
pub const fn is_local_mark(c: u8) -> bool {
    c.is_ascii_lowercase()
}

/// Check if a character is an uppercase mark (global).
#[must_use]
pub const fn is_global_mark(c: u8) -> bool {
    c.is_ascii_uppercase()
}

/// Check if a character is a numbered mark.
#[must_use]
pub const fn is_numbered_mark(c: u8) -> bool {
    c.is_ascii_digit()
}

/// Check if a character is a special mark.
#[must_use]
pub const fn is_special_mark(c: u8) -> bool {
    matches!(
        c,
        b'\'' | b'`' | b'[' | b']' | b'<' | b'>' | b'^' | b'.' | b'"'
    )
}

// =============================================================================
// Delmarks Parsing
// =============================================================================

/// Result of parsing :delmarks arguments.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DelmarksResult {
    /// Whether parsing was successful
    pub valid: bool,
    /// Whether to delete all marks (!)
    pub delete_all: bool,
    /// Number of individual marks to delete
    pub mark_count: c_int,
    /// Error code (0 = no error)
    pub error: c_int,
}

/// Parse a mark range like a-f or A-Z.
///
/// # Arguments
/// * `start` - Starting mark character
/// * `end` - Ending mark character
///
/// # Returns
/// `(valid, count)` - Whether range is valid and how many marks
#[must_use]
pub const fn parse_mark_range(start: u8, end: u8) -> (bool, c_int) {
    // Range must be same type (both lowercase or both uppercase)
    let valid = (is_local_mark(start) && is_local_mark(end))
        || (is_global_mark(start) && is_global_mark(end))
        || (is_numbered_mark(start) && is_numbered_mark(end));

    if !valid || end < start {
        return (false, 0);
    }

    (true, (end - start + 1) as c_int)
}

/// Count marks in a delmarks argument string.
///
/// Handles individual marks and ranges (e.g., "a-f A B").
#[must_use]
pub fn count_marks_in_args(args: &[u8]) -> c_int {
    let mut count = 0;
    let mut i = 0;

    while i < args.len() {
        let c = args[i];

        // Skip whitespace
        if c == b' ' || c == b'\t' {
            i += 1;
            continue;
        }

        if !is_valid_mark(c) {
            i += 1;
            continue;
        }

        // Check for range (mark-mark)
        if i + 2 < args.len() && args[i + 1] == b'-' {
            let end = args[i + 2];
            let (valid, range_count) = parse_mark_range(c, end);
            if valid {
                count += range_count;
                i += 3;
                continue;
            }
        }

        // Individual mark
        count += 1;
        i += 1;
    }

    count
}

// =============================================================================
// Jump List Operations
// =============================================================================

/// Maximum jump list size.
pub const JUMPLISTSIZE: c_int = 100;

/// Check if jump list index is valid.
#[must_use]
pub const fn is_valid_jumplist_idx(idx: c_int, len: c_int) -> bool {
    idx >= 0 && idx < len && idx < JUMPLISTSIZE
}

/// Calculate index for :jumps display (newest first).
///
/// Jump list displays with newest at bottom (index 0 is oldest).
#[must_use]
pub const fn jumplist_display_idx(actual_idx: c_int, len: c_int) -> c_int {
    if len <= 0 {
        return 0;
    }
    len - 1 - actual_idx
}

// =============================================================================
// Change List Operations
// =============================================================================

/// Maximum change list size.
pub const CHANGELISTSIZE: c_int = 50;

/// Check if change list index is valid.
#[must_use]
pub const fn is_valid_changelist_idx(idx: c_int, len: c_int) -> bool {
    idx >= 0 && idx < len && idx < CHANGELISTSIZE
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Check if character is valid mark name.
#[no_mangle]
pub extern "C" fn rs_is_valid_mark(c: c_int) -> c_int {
    c_int::from(is_valid_mark(c as u8))
}

/// FFI: Check if character is local mark.
#[no_mangle]
pub extern "C" fn rs_is_local_mark(c: c_int) -> c_int {
    c_int::from(is_local_mark(c as u8))
}

/// FFI: Check if character is global mark.
#[no_mangle]
pub extern "C" fn rs_is_global_mark(c: c_int) -> c_int {
    c_int::from(is_global_mark(c as u8))
}

/// FFI: Check if character is numbered mark.
#[no_mangle]
pub extern "C" fn rs_is_numbered_mark(c: c_int) -> c_int {
    c_int::from(is_numbered_mark(c as u8))
}

/// FFI: Check if character is special mark.
#[no_mangle]
pub extern "C" fn rs_is_special_mark(c: c_int) -> c_int {
    c_int::from(is_special_mark(c as u8))
}

/// FFI: Get mark type from character.
#[no_mangle]
pub extern "C" fn rs_mark_type_from_char(c: c_int) -> c_int {
    MarkType::from_char(c as u8).to_raw()
}

/// FFI: Parse mark range.
///
/// # Safety
/// `count_out` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_mark_range(
    start: c_int,
    end: c_int,
    count_out: *mut c_int,
) -> c_int {
    let (valid, count) = parse_mark_range(start as u8, end as u8);
    if !count_out.is_null() {
        *count_out = count;
    }
    c_int::from(valid)
}

/// FFI: Count marks in argument string.
///
/// # Safety
/// `args` must be a valid null-terminated string or null.
#[no_mangle]
pub unsafe extern "C" fn rs_count_marks_in_args(args: *const c_char) -> c_int {
    if args.is_null() {
        return 0;
    }

    // Convert to slice
    let mut len = 0;
    let mut p = args;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }

    if len == 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(args as *const u8, len);
    count_marks_in_args(slice)
}

/// FFI: Check if jump list index is valid.
#[no_mangle]
pub extern "C" fn rs_is_valid_jumplist_idx(idx: c_int, len: c_int) -> c_int {
    c_int::from(is_valid_jumplist_idx(idx, len))
}

/// FFI: Get jump list display index.
#[no_mangle]
pub extern "C" fn rs_jumplist_display_idx(actual_idx: c_int, len: c_int) -> c_int {
    jumplist_display_idx(actual_idx, len)
}

/// FFI: Check if change list index is valid.
#[no_mangle]
pub extern "C" fn rs_is_valid_changelist_idx(idx: c_int, len: c_int) -> c_int {
    c_int::from(is_valid_changelist_idx(idx, len))
}

/// FFI: Get JUMPLISTSIZE constant.
#[no_mangle]
pub extern "C" fn rs_get_jumplist_size() -> c_int {
    JUMPLISTSIZE
}

/// FFI: Get CHANGELISTSIZE constant.
#[no_mangle]
pub extern "C" fn rs_get_changelist_size() -> c_int {
    CHANGELISTSIZE
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_mark() {
        assert!(is_valid_mark(b'a'));
        assert!(is_valid_mark(b'z'));
        assert!(is_valid_mark(b'A'));
        assert!(is_valid_mark(b'Z'));
        assert!(is_valid_mark(b'0'));
        assert!(is_valid_mark(b'9'));
        assert!(is_valid_mark(b'\''));
        assert!(is_valid_mark(b'`'));
        assert!(!is_valid_mark(b'@'));
        assert!(!is_valid_mark(b'\n'));
    }

    #[test]
    fn test_mark_types() {
        assert!(is_local_mark(b'a'));
        assert!(is_local_mark(b'z'));
        assert!(!is_local_mark(b'A'));

        assert!(is_global_mark(b'A'));
        assert!(is_global_mark(b'Z'));
        assert!(!is_global_mark(b'a'));

        assert!(is_numbered_mark(b'0'));
        assert!(is_numbered_mark(b'9'));
        assert!(!is_numbered_mark(b'a'));

        assert!(is_special_mark(b'\''));
        assert!(is_special_mark(b'.'));
        assert!(!is_special_mark(b'a'));
    }

    #[test]
    fn test_mark_type_from_char() {
        assert_eq!(MarkType::from_char(b'a'), MarkType::Named);
        assert_eq!(MarkType::from_char(b'Z'), MarkType::Named);
        assert_eq!(MarkType::from_char(b'5'), MarkType::Numbered);
        assert_eq!(MarkType::from_char(b'\''), MarkType::Special);
    }

    #[test]
    fn test_parse_mark_range() {
        // Valid lowercase range
        let (valid, count) = parse_mark_range(b'a', b'f');
        assert!(valid);
        assert_eq!(count, 6);

        // Valid uppercase range
        let (valid, count) = parse_mark_range(b'A', b'C');
        assert!(valid);
        assert_eq!(count, 3);

        // Invalid: mixed case
        let (valid, _) = parse_mark_range(b'a', b'Z');
        assert!(!valid);

        // Invalid: end before start
        let (valid, _) = parse_mark_range(b'z', b'a');
        assert!(!valid);
    }

    #[test]
    fn test_count_marks_in_args() {
        // Individual marks
        assert_eq!(count_marks_in_args(b"a b c"), 3);

        // Range
        assert_eq!(count_marks_in_args(b"a-f"), 6);

        // Mixed
        assert_eq!(count_marks_in_args(b"a-c X Y"), 5);

        // Empty
        assert_eq!(count_marks_in_args(b""), 0);
    }

    #[test]
    fn test_jumplist_indices() {
        assert!(is_valid_jumplist_idx(0, 10));
        assert!(is_valid_jumplist_idx(9, 10));
        assert!(!is_valid_jumplist_idx(10, 10));
        assert!(!is_valid_jumplist_idx(-1, 10));
        assert!(!is_valid_jumplist_idx(0, 0));

        // Display index (inverted for newest-first display)
        assert_eq!(jumplist_display_idx(0, 10), 9);
        assert_eq!(jumplist_display_idx(9, 10), 0);
    }

    #[test]
    fn test_changelist_indices() {
        assert!(is_valid_changelist_idx(0, 50));
        assert!(is_valid_changelist_idx(49, 50));
        assert!(!is_valid_changelist_idx(50, 50));
    }

    #[test]
    fn test_ffi_wrappers() {
        assert_eq!(rs_is_valid_mark(b'a' as c_int), 1);
        assert_eq!(rs_is_valid_mark(b'@' as c_int), 0);
        assert_eq!(rs_is_local_mark(b'a' as c_int), 1);
        assert_eq!(rs_is_global_mark(b'A' as c_int), 1);
    }
}
