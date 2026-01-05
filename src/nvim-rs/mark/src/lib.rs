//! Mark utilities for Neovim
//!
//! This crate provides functions for working with marks and positions.

use std::ffi::c_int;

/// Number of possible named marks (a-z)
pub const NMARKS: c_int = 26;

/// pos_T structure matching Neovim's pos_defs.h
/// Position in file or buffer
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PosT {
    /// line number
    pub lnum: i32,
    /// column number
    pub col: i32,
    /// column add (for virtual columns)
    pub coladd: i32,
}

/// Check if a character is an ASCII uppercase letter (A-Z).
#[inline]
const fn ascii_isupper(c: u8) -> bool {
    c >= b'A' && c <= b'Z'
}

/// Check if a character is an ASCII lowercase letter (a-z).
#[inline]
const fn ascii_islower(c: u8) -> bool {
    c >= b'a' && c <= b'z'
}

/// Check if a character is an ASCII digit (0-9).
#[inline]
const fn ascii_isdigit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

/// Convert mark name to the global mark index.
///
/// Returns the offset for uppercase marks (A-Z) or digit marks (0-9),
/// or -1 if the name is not a valid global mark.
#[no_mangle]
pub extern "C" fn rs_mark_global_index(name: c_int) -> c_int {
    let Ok(c) = u8::try_from(name) else {
        return -1;
    };
    if ascii_isupper(c) {
        c_int::from(c - b'A')
    } else if ascii_isdigit(c) {
        NMARKS + c_int::from(c - b'0')
    } else {
        -1
    }
}

/// Convert local mark name to the offset.
///
/// Returns the offset for lowercase marks (a-z) or special marks (", ^, .),
/// or -1 if the name is not a valid local mark.
#[no_mangle]
pub extern "C" fn rs_mark_local_index(name: c_int) -> c_int {
    let Ok(c) = u8::try_from(name) else {
        return -1;
    };
    if ascii_islower(c) {
        c_int::from(c - b'a')
    } else if c == b'"' {
        NMARKS
    } else if c == b'^' {
        NMARKS + 1
    } else if c == b'.' {
        NMARKS + 2
    } else {
        -1
    }
}

/// Return true if position a is before (less than) position b.
#[no_mangle]
pub extern "C" fn rs_lt(a: PosT, b: PosT) -> c_int {
    let result = if a.lnum != b.lnum {
        a.lnum < b.lnum
    } else if a.col != b.col {
        a.col < b.col
    } else {
        a.coladd < b.coladd
    };
    c_int::from(result)
}

/// Return true if position a equals position b.
#[no_mangle]
pub extern "C" fn rs_equalpos(a: PosT, b: PosT) -> c_int {
    c_int::from(a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd)
}

/// Return true if position a is less than or equal to position b.
#[no_mangle]
pub extern "C" fn rs_ltoreq(a: PosT, b: PosT) -> c_int {
    c_int::from(rs_lt(a, b) != 0 || rs_equalpos(a, b) != 0)
}

/// Return true if position is empty (all fields are 0).
///
/// Matches the C macro: `EMPTY_POS(a) ((a).lnum == 0 && (a).col == 0 && (a).coladd == 0)`
#[no_mangle]
pub extern "C" fn rs_empty_pos(a: PosT) -> c_int {
    c_int::from(a.lnum == 0 && a.col == 0 && a.coladd == 0)
}

/// Clear a position by setting all fields to 0.
///
/// # Safety
///
/// `a` must be a valid, non-null pointer to a PosT struct.
#[no_mangle]
pub unsafe extern "C" fn rs_clearpos(a: *mut PosT) {
    if a.is_null() {
        return;
    }
    (*a).lnum = 0;
    (*a).col = 0;
    (*a).coladd = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_global_index() {
        // Uppercase letters A-Z map to 0-25
        assert_eq!(rs_mark_global_index(c_int::from(b'A')), 0);
        assert_eq!(rs_mark_global_index(c_int::from(b'Z')), 25);
        assert_eq!(rs_mark_global_index(c_int::from(b'M')), 12);

        // Digits 0-9 map to NMARKS + 0..9 (26-35)
        assert_eq!(rs_mark_global_index(c_int::from(b'0')), NMARKS);
        assert_eq!(rs_mark_global_index(c_int::from(b'9')), NMARKS + 9);
        assert_eq!(rs_mark_global_index(c_int::from(b'5')), NMARKS + 5);

        // Invalid marks return -1
        assert_eq!(rs_mark_global_index(c_int::from(b'a')), -1);
        assert_eq!(rs_mark_global_index(c_int::from(b'!')), -1);
        assert_eq!(rs_mark_global_index(-1), -1);
        assert_eq!(rs_mark_global_index(256), -1);
    }

    #[test]
    fn test_mark_local_index() {
        // Lowercase letters a-z map to 0-25
        assert_eq!(rs_mark_local_index(c_int::from(b'a')), 0);
        assert_eq!(rs_mark_local_index(c_int::from(b'z')), 25);
        assert_eq!(rs_mark_local_index(c_int::from(b'm')), 12);

        // Special marks
        assert_eq!(rs_mark_local_index(c_int::from(b'"')), NMARKS);
        assert_eq!(rs_mark_local_index(c_int::from(b'^')), NMARKS + 1);
        assert_eq!(rs_mark_local_index(c_int::from(b'.')), NMARKS + 2);

        // Invalid marks return -1
        assert_eq!(rs_mark_local_index(c_int::from(b'A')), -1);
        assert_eq!(rs_mark_local_index(c_int::from(b'0')), -1);
        assert_eq!(rs_mark_local_index(c_int::from(b'!')), -1);
        assert_eq!(rs_mark_local_index(-1), -1);
    }

    #[test]
    fn test_lt() {
        let pos1 = PosT {
            lnum: 1,
            col: 5,
            coladd: 0,
        };
        let pos2 = PosT {
            lnum: 2,
            col: 3,
            coladd: 0,
        };
        let pos3 = PosT {
            lnum: 1,
            col: 10,
            coladd: 0,
        };
        let pos4 = PosT {
            lnum: 1,
            col: 5,
            coladd: 1,
        };
        let pos5 = PosT {
            lnum: 1,
            col: 5,
            coladd: 0,
        };

        // Different lines
        assert_ne!(rs_lt(pos1, pos2), 0); // pos1 < pos2 (line 1 < line 2)
        assert_eq!(rs_lt(pos2, pos1), 0); // pos2 > pos1

        // Same line, different columns
        assert_ne!(rs_lt(pos1, pos3), 0); // pos1 < pos3 (col 5 < col 10)
        assert_eq!(rs_lt(pos3, pos1), 0); // pos3 > pos1

        // Same line and column, different coladd
        assert_ne!(rs_lt(pos1, pos4), 0); // pos1 < pos4 (coladd 0 < coladd 1)
        assert_eq!(rs_lt(pos4, pos1), 0); // pos4 > pos1

        // Equal positions
        assert_eq!(rs_lt(pos1, pos5), 0); // not less than
    }

    #[test]
    fn test_equalpos() {
        let pos1 = PosT {
            lnum: 1,
            col: 5,
            coladd: 0,
        };
        let pos2 = PosT {
            lnum: 1,
            col: 5,
            coladd: 0,
        };
        let pos3 = PosT {
            lnum: 1,
            col: 5,
            coladd: 1,
        };
        let pos4 = PosT {
            lnum: 1,
            col: 6,
            coladd: 0,
        };
        let pos5 = PosT {
            lnum: 2,
            col: 5,
            coladd: 0,
        };

        assert_ne!(rs_equalpos(pos1, pos2), 0); // equal
        assert_eq!(rs_equalpos(pos1, pos3), 0); // different coladd
        assert_eq!(rs_equalpos(pos1, pos4), 0); // different col
        assert_eq!(rs_equalpos(pos1, pos5), 0); // different lnum
    }

    #[test]
    fn test_ltoreq() {
        let pos1 = PosT {
            lnum: 1,
            col: 5,
            coladd: 0,
        };
        let pos2 = PosT {
            lnum: 1,
            col: 5,
            coladd: 0,
        };
        let pos3 = PosT {
            lnum: 1,
            col: 10,
            coladd: 0,
        };
        let pos4 = PosT {
            lnum: 2,
            col: 1,
            coladd: 0,
        };

        // Equal positions
        assert_ne!(rs_ltoreq(pos1, pos2), 0);

        // Less than
        assert_ne!(rs_ltoreq(pos1, pos3), 0);
        assert_ne!(rs_ltoreq(pos1, pos4), 0);

        // Greater than
        assert_eq!(rs_ltoreq(pos3, pos1), 0);
        assert_eq!(rs_ltoreq(pos4, pos1), 0);
    }

    #[test]
    fn test_empty_pos() {
        // Empty position (all zeros)
        let empty = PosT {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        assert_ne!(rs_empty_pos(empty), 0);

        // Non-empty positions (at least one field non-zero)
        let non_empty1 = PosT {
            lnum: 1,
            col: 0,
            coladd: 0,
        };
        assert_eq!(rs_empty_pos(non_empty1), 0);

        let non_empty2 = PosT {
            lnum: 0,
            col: 1,
            coladd: 0,
        };
        assert_eq!(rs_empty_pos(non_empty2), 0);

        let non_empty3 = PosT {
            lnum: 0,
            col: 0,
            coladd: 1,
        };
        assert_eq!(rs_empty_pos(non_empty3), 0);

        let non_empty4 = PosT {
            lnum: 1,
            col: 5,
            coladd: 2,
        };
        assert_eq!(rs_empty_pos(non_empty4), 0);
    }

    #[test]
    fn test_clearpos() {
        // Clear a non-empty position
        let mut pos = PosT {
            lnum: 10,
            col: 5,
            coladd: 2,
        };
        unsafe { rs_clearpos(&mut pos) };
        assert_eq!(pos.lnum, 0);
        assert_eq!(pos.col, 0);
        assert_eq!(pos.coladd, 0);

        // Should be empty after clearing
        assert_ne!(rs_empty_pos(pos), 0);

        // Null pointer should be handled gracefully
        unsafe { rs_clearpos(std::ptr::null_mut()) };
    }

    #[test]
    fn test_nmarks_constant() {
        // Verify NMARKS matches C definition (26 named marks a-z)
        assert_eq!(NMARKS, 26);
    }

    #[test]
    fn test_ascii_helpers() {
        // Test ascii_isupper
        assert!(ascii_isupper(b'A'));
        assert!(ascii_isupper(b'Z'));
        assert!(ascii_isupper(b'M'));
        assert!(!ascii_isupper(b'a'));
        assert!(!ascii_isupper(b'z'));
        assert!(!ascii_isupper(b'0'));
        assert!(!ascii_isupper(b'@')); // before A
        assert!(!ascii_isupper(b'[')); // after Z

        // Test ascii_islower
        assert!(ascii_islower(b'a'));
        assert!(ascii_islower(b'z'));
        assert!(ascii_islower(b'm'));
        assert!(!ascii_islower(b'A'));
        assert!(!ascii_islower(b'Z'));
        assert!(!ascii_islower(b'0'));
        assert!(!ascii_islower(b'`')); // before a
        assert!(!ascii_islower(b'{')); // after z

        // Test ascii_isdigit
        assert!(ascii_isdigit(b'0'));
        assert!(ascii_isdigit(b'9'));
        assert!(ascii_isdigit(b'5'));
        assert!(!ascii_isdigit(b'a'));
        assert!(!ascii_isdigit(b'A'));
        assert!(!ascii_isdigit(b'/')); // before 0
        assert!(!ascii_isdigit(b':')); // after 9
    }

    #[test]
    fn test_pos_t_default() {
        // Default should be an empty position
        let pos = PosT::default();
        assert_eq!(pos.lnum, 0);
        assert_eq!(pos.col, 0);
        assert_eq!(pos.coladd, 0);
        assert_ne!(rs_empty_pos(pos), 0);
    }

    #[test]
    fn test_pos_t_clone_and_eq() {
        let pos1 = PosT {
            lnum: 10,
            col: 5,
            coladd: 2,
        };
        let pos2 = pos1;
        assert_eq!(pos1, pos2);

        let pos3 = PosT {
            lnum: 10,
            col: 5,
            coladd: 3,
        };
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_pos_t_debug() {
        let pos = PosT {
            lnum: 10,
            col: 5,
            coladd: 2,
        };
        let debug_str = format!("{pos:?}");
        assert!(debug_str.contains("lnum: 10"));
        assert!(debug_str.contains("col: 5"));
        assert!(debug_str.contains("coladd: 2"));
    }

    #[test]
    fn test_mark_global_index_all_uppercase() {
        // Test all uppercase letters map correctly
        for (i, c) in (b'A'..=b'Z').enumerate() {
            assert_eq!(
                rs_mark_global_index(c_int::from(c)),
                i as c_int,
                "Failed for {}",
                c as char
            );
        }
    }

    #[test]
    fn test_mark_global_index_all_digits() {
        // Test all digits map correctly
        for (i, c) in (b'0'..=b'9').enumerate() {
            assert_eq!(
                rs_mark_global_index(c_int::from(c)),
                NMARKS + i as c_int,
                "Failed for {}",
                c as char
            );
        }
    }

    #[test]
    fn test_mark_local_index_all_lowercase() {
        // Test all lowercase letters map correctly
        for (i, c) in (b'a'..=b'z').enumerate() {
            assert_eq!(
                rs_mark_local_index(c_int::from(c)),
                i as c_int,
                "Failed for {}",
                c as char
            );
        }
    }

    #[test]
    fn test_lt_negative_values() {
        // Test with negative values
        let pos1 = PosT {
            lnum: -1,
            col: 0,
            coladd: 0,
        };
        let pos2 = PosT {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        assert_ne!(rs_lt(pos1, pos2), 0); // -1 < 0
        assert_eq!(rs_lt(pos2, pos1), 0); // 0 > -1
    }

    #[test]
    fn test_position_comparison_transitivity() {
        // Test transitivity: if a < b and b < c, then a < c
        let a = PosT {
            lnum: 1,
            col: 0,
            coladd: 0,
        };
        let b = PosT {
            lnum: 2,
            col: 0,
            coladd: 0,
        };
        let c = PosT {
            lnum: 3,
            col: 0,
            coladd: 0,
        };

        assert_ne!(rs_lt(a, b), 0); // a < b
        assert_ne!(rs_lt(b, c), 0); // b < c
        assert_ne!(rs_lt(a, c), 0); // a < c (transitivity)
    }
}

// =============================================================================
// Phase 5: Mark System Foundation - Additional Functions
// =============================================================================

/// Number of file marks (A-Z + 0-9)
pub const NGLOBALMARKS: c_int = NMARKS + 10; // 36

/// Check if a character is a valid named mark (a-z).
#[no_mangle]
pub extern "C" fn rs_mark_is_valid_named(name: c_int) -> bool {
    let Ok(c) = u8::try_from(name) else {
        return false;
    };
    ascii_islower(c)
}

/// Check if a character is a valid file mark (A-Z or 0-9).
#[no_mangle]
pub extern "C" fn rs_mark_is_file_mark(name: c_int) -> bool {
    let Ok(c) = u8::try_from(name) else {
        return false;
    };
    ascii_isupper(c) || ascii_isdigit(c)
}

/// Check if a mark name is a jump mark (' or `).
#[no_mangle]
pub extern "C" fn rs_mark_is_jump_mark(name: c_int) -> bool {
    let Ok(c) = u8::try_from(name) else {
        return false;
    };
    c == b'\'' || c == b'`'
}

/// Check if a mark name is a special mark.
#[no_mangle]
pub extern "C" fn rs_mark_is_special(name: c_int) -> bool {
    let Ok(c) = u8::try_from(name) else {
        return false;
    };
    matches!(c, b'"' | b'^' | b'.' | b'[' | b']' | b'<' | b'>' | b'\'' | b'`')
}

/// Check if a mark name is a visual mark (< or >).
#[no_mangle]
pub extern "C" fn rs_mark_is_visual(name: c_int) -> bool {
    let Ok(c) = u8::try_from(name) else {
        return false;
    };
    c == b'<' || c == b'>'
}

/// Check if a mark name is the last cursor position mark (").
#[no_mangle]
pub extern "C" fn rs_mark_is_last_cursor(name: c_int) -> bool {
    name == c_int::from(b'"')
}

/// Check if a mark name is the last insert position mark (^).
#[no_mangle]
pub extern "C" fn rs_mark_is_last_insert(name: c_int) -> bool {
    name == c_int::from(b'^')
}

/// Check if a mark name is the last change position mark (.).
#[no_mangle]
pub extern "C" fn rs_mark_is_last_change(name: c_int) -> bool {
    name == c_int::from(b'.')
}

/// Check if a mark name is a sentence boundary mark ([ or ]).
#[no_mangle]
pub extern "C" fn rs_mark_is_sentence(name: c_int) -> bool {
    let Ok(c) = u8::try_from(name) else {
        return false;
    };
    c == b'[' || c == b']'
}

/// Check if a position is valid (non-zero line number).
#[no_mangle]
pub extern "C" fn rs_pos_is_valid(pos: PosT) -> c_int {
    c_int::from(pos.lnum > 0)
}

/// Check if a position line is in range for a given buffer line count.
#[no_mangle]
pub extern "C" fn rs_pos_in_range(pos: PosT, line_count: i32) -> c_int {
    c_int::from(pos.lnum > 0 && pos.lnum <= line_count)
}

/// Compare two positions and return -1, 0, or 1.
#[no_mangle]
pub extern "C" fn rs_pos_compare(a: PosT, b: PosT) -> c_int {
    if rs_lt(a, b) != 0 {
        -1
    } else if rs_equalpos(a, b) != 0 {
        0
    } else {
        1
    }
}

/// Copy position from source to destination.
///
/// # Safety
///
/// Both pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_pos_copy(dst: *mut PosT, src: *const PosT) {
    if !dst.is_null() && !src.is_null() {
        *dst = *src;
    }
}

/// Get the line number from a position.
#[no_mangle]
pub extern "C" fn rs_pos_get_lnum(pos: PosT) -> i32 {
    pos.lnum
}

/// Get the column number from a position.
#[no_mangle]
pub extern "C" fn rs_pos_get_col(pos: PosT) -> i32 {
    pos.col
}

/// Get the virtual column add from a position.
#[no_mangle]
pub extern "C" fn rs_pos_get_coladd(pos: PosT) -> i32 {
    pos.coladd
}

/// Set the line number in a position.
///
/// # Safety
///
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_pos_set_lnum(pos: *mut PosT, lnum: i32) {
    if !pos.is_null() {
        (*pos).lnum = lnum;
    }
}

/// Set the column number in a position.
///
/// # Safety
///
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_pos_set_col(pos: *mut PosT, col: i32) {
    if !pos.is_null() {
        (*pos).col = col;
    }
}

/// Set the virtual column add in a position.
///
/// # Safety
///
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_pos_set_coladd(pos: *mut PosT, coladd: i32) {
    if !pos.is_null() {
        (*pos).coladd = coladd;
    }
}

// =============================================================================
// Phase 6: Mark Operations - Additional Functions
// =============================================================================

/// Get the display name for a mark character.
///
/// # Safety
///
/// The `buf` pointer must be valid and point to at least `buf_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_get_name(name: c_int, buf: *mut u8, buf_len: usize) {
    if buf.is_null() || buf_len < 2 {
        return;
    }

    let buf_slice = std::slice::from_raw_parts_mut(buf, buf_len);

    if name == -1 {
        // No mark
        buf_slice[0] = b'-';
        buf_slice[1] = 0;
    } else if let Ok(c) = u8::try_from(name) {
        buf_slice[0] = c;
        buf_slice[1] = 0;
    } else {
        buf_slice[0] = b'?';
        buf_slice[1] = 0;
    }
}

/// Get a category string for a mark.
/// Returns a static string identifying the mark category.
#[no_mangle]
pub extern "C" fn rs_mark_get_category(name: c_int) -> *const std::ffi::c_char {
    let Ok(c) = u8::try_from(name) else {
        return c"unknown".as_ptr();
    };

    if ascii_islower(c) {
        c"local".as_ptr()
    } else if ascii_isupper(c) {
        c"file".as_ptr()
    } else if ascii_isdigit(c) {
        c"numbered".as_ptr()
    } else if c == b'"' {
        c"cursor".as_ptr()
    } else if c == b'^' || c == b'.' {
        c"change".as_ptr()
    } else if c == b'[' || c == b']' {
        c"text".as_ptr()
    } else if c == b'<' || c == b'>' {
        c"visual".as_ptr()
    } else if c == b'\'' || c == b'`' {
        c"jump".as_ptr()
    } else {
        c"special".as_ptr()
    }
}

/// Check if mark name is user-settable (not automatic).
#[no_mangle]
pub extern "C" fn rs_mark_is_user_settable(name: c_int) -> bool {
    let Ok(c) = u8::try_from(name) else {
        return false;
    };
    // User can set named marks (a-z, A-Z) and some special marks
    ascii_islower(c) || ascii_isupper(c) || c == b'\'' || c == b'`' || c == b'<' || c == b'>'
}

/// Check if mark should be persisted to shada.
#[no_mangle]
pub extern "C" fn rs_mark_is_persistent(name: c_int) -> bool {
    let Ok(c) = u8::try_from(name) else {
        return false;
    };
    // Named marks (a-z, A-Z), numbered marks (0-9), and special marks (", ^, .)
    ascii_islower(c)
        || ascii_isupper(c)
        || ascii_isdigit(c)
        || c == b'"'
        || c == b'^'
        || c == b'.'
}

/// Create a new position with given values.
#[no_mangle]
pub extern "C" fn rs_pos_new(lnum: i32, col: i32, coladd: i32) -> PosT {
    PosT { lnum, col, coladd }
}

/// Create a zero position.
#[no_mangle]
pub extern "C" fn rs_pos_zero() -> PosT {
    PosT {
        lnum: 0,
        col: 0,
        coladd: 0,
    }
}

/// Adjust position line number by delta.
///
/// # Safety
///
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_pos_adjust_line(pos: *mut PosT, delta: i32) {
    if !pos.is_null() {
        (*pos).lnum += delta;
    }
}

/// Adjust position column by delta.
///
/// # Safety
///
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_pos_adjust_col(pos: *mut PosT, delta: i32) {
    if !pos.is_null() {
        (*pos).col += delta;
    }
}

/// Clamp a position to valid buffer bounds.
///
/// # Safety
///
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_pos_clamp(pos: *mut PosT, max_lnum: i32, max_col: i32) {
    if pos.is_null() {
        return;
    }

    if (*pos).lnum < 1 {
        (*pos).lnum = 1;
    } else if (*pos).lnum > max_lnum {
        (*pos).lnum = max_lnum;
    }

    if (*pos).col < 0 {
        (*pos).col = 0;
    } else if (*pos).col > max_col {
        (*pos).col = max_col;
    }

    if (*pos).coladd < 0 {
        (*pos).coladd = 0;
    }
}

/// Get the distance (in lines) between two positions.
#[no_mangle]
pub extern "C" fn rs_pos_line_distance(a: PosT, b: PosT) -> i32 {
    (b.lnum - a.lnum).abs()
}

/// Check if two positions are on the same line.
#[no_mangle]
pub extern "C" fn rs_pos_same_line(a: PosT, b: PosT) -> c_int {
    c_int::from(a.lnum == b.lnum)
}

/// Swap two positions if a > b (ensure a <= b).
///
/// # Safety
///
/// Both pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_pos_order(a: *mut PosT, b: *mut PosT) {
    if a.is_null() || b.is_null() {
        return;
    }
    if rs_lt(*b, *a) != 0 {
        std::ptr::swap(a, b);
    }
}

// =============================================================================
// Phase 5 & 6 Tests
// =============================================================================

#[cfg(test)]
mod phase56_tests {
    use super::*;

    #[test]
    fn test_mark_validation() {
        // Named mark validation
        assert!(rs_mark_is_valid_named(c_int::from(b'a')));
        assert!(rs_mark_is_valid_named(c_int::from(b'z')));
        assert!(!rs_mark_is_valid_named(c_int::from(b'A')));
        assert!(!rs_mark_is_valid_named(c_int::from(b'0')));

        // File mark validation
        assert!(rs_mark_is_file_mark(c_int::from(b'A')));
        assert!(rs_mark_is_file_mark(c_int::from(b'Z')));
        assert!(rs_mark_is_file_mark(c_int::from(b'0')));
        assert!(!rs_mark_is_file_mark(c_int::from(b'a')));

        // Jump mark validation
        assert!(rs_mark_is_jump_mark(c_int::from(b'\'')));
        assert!(rs_mark_is_jump_mark(c_int::from(b'`')));
        assert!(!rs_mark_is_jump_mark(c_int::from(b'a')));
    }

    #[test]
    fn test_mark_type_categorization() {
        // Special marks
        assert!(rs_mark_is_special(c_int::from(b'"')));
        assert!(rs_mark_is_special(c_int::from(b'^')));
        assert!(rs_mark_is_special(c_int::from(b'.')));
        assert!(rs_mark_is_special(c_int::from(b'[')));
        assert!(rs_mark_is_special(c_int::from(b']')));
        assert!(rs_mark_is_special(c_int::from(b'<')));
        assert!(rs_mark_is_special(c_int::from(b'>')));
        assert!(!rs_mark_is_special(c_int::from(b'a')));

        // Visual marks
        assert!(rs_mark_is_visual(c_int::from(b'<')));
        assert!(rs_mark_is_visual(c_int::from(b'>')));
        assert!(!rs_mark_is_visual(c_int::from(b'a')));

        // Sentence marks
        assert!(rs_mark_is_sentence(c_int::from(b'[')));
        assert!(rs_mark_is_sentence(c_int::from(b']')));
        assert!(!rs_mark_is_sentence(c_int::from(b'a')));
    }

    #[test]
    fn test_pos_constructors() {
        let pos = rs_pos_new(10, 5, 2);
        assert_eq!(pos.lnum, 10);
        assert_eq!(pos.col, 5);
        assert_eq!(pos.coladd, 2);

        let zero = rs_pos_zero();
        assert_eq!(zero.lnum, 0);
        assert_eq!(zero.col, 0);
        assert_eq!(zero.coladd, 0);
    }

    #[test]
    fn test_pos_getters() {
        let pos = rs_pos_new(10, 5, 2);
        assert_eq!(rs_pos_get_lnum(pos), 10);
        assert_eq!(rs_pos_get_col(pos), 5);
        assert_eq!(rs_pos_get_coladd(pos), 2);
    }

    #[test]
    fn test_pos_validity() {
        let valid = rs_pos_new(1, 0, 0);
        assert_ne!(rs_pos_is_valid(valid), 0);

        let invalid = rs_pos_new(0, 0, 0);
        assert_eq!(rs_pos_is_valid(invalid), 0);

        let negative = rs_pos_new(-1, 0, 0);
        assert_eq!(rs_pos_is_valid(negative), 0);
    }

    #[test]
    fn test_pos_in_range() {
        let pos = rs_pos_new(5, 0, 0);
        assert_ne!(rs_pos_in_range(pos, 10), 0);
        assert_eq!(rs_pos_in_range(pos, 4), 0);

        let zero = rs_pos_zero();
        assert_eq!(rs_pos_in_range(zero, 10), 0);
    }

    #[test]
    fn test_pos_compare() {
        let a = rs_pos_new(1, 0, 0);
        let b = rs_pos_new(2, 0, 0);
        let c = rs_pos_new(1, 0, 0);

        assert_eq!(rs_pos_compare(a, b), -1);
        assert_eq!(rs_pos_compare(b, a), 1);
        assert_eq!(rs_pos_compare(a, c), 0);
    }

    #[test]
    fn test_pos_same_line() {
        let a = rs_pos_new(1, 0, 0);
        let b = rs_pos_new(1, 5, 0);
        let c = rs_pos_new(2, 0, 0);

        assert_ne!(rs_pos_same_line(a, b), 0);
        assert_eq!(rs_pos_same_line(a, c), 0);
    }

    #[test]
    fn test_pos_line_distance() {
        let a = rs_pos_new(1, 0, 0);
        let b = rs_pos_new(5, 0, 0);
        assert_eq!(rs_pos_line_distance(a, b), 4);
        assert_eq!(rs_pos_line_distance(b, a), 4);
    }

    #[test]
    fn test_mark_persistence() {
        // Named marks (a-z) are persistent
        assert!(rs_mark_is_persistent(c_int::from(b'a')));
        assert!(rs_mark_is_persistent(c_int::from(b'z')));

        // File marks (A-Z) are persistent
        assert!(rs_mark_is_persistent(c_int::from(b'A')));
        assert!(rs_mark_is_persistent(c_int::from(b'Z')));

        // Numbered marks (0-9) are persistent
        assert!(rs_mark_is_persistent(c_int::from(b'0')));
        assert!(rs_mark_is_persistent(c_int::from(b'9')));

        // Special persistent marks
        assert!(rs_mark_is_persistent(c_int::from(b'"')));
        assert!(rs_mark_is_persistent(c_int::from(b'^')));
        assert!(rs_mark_is_persistent(c_int::from(b'.')));

        // Non-persistent marks
        assert!(!rs_mark_is_persistent(c_int::from(b'<')));
        assert!(!rs_mark_is_persistent(c_int::from(b'>')));
    }

    #[test]
    fn test_mark_user_settable() {
        // Named marks are user-settable
        assert!(rs_mark_is_user_settable(c_int::from(b'a')));
        assert!(rs_mark_is_user_settable(c_int::from(b'A')));

        // Jump marks are user-settable
        assert!(rs_mark_is_user_settable(c_int::from(b'\'')));
        assert!(rs_mark_is_user_settable(c_int::from(b'`')));

        // Visual marks are user-settable
        assert!(rs_mark_is_user_settable(c_int::from(b'<')));
        assert!(rs_mark_is_user_settable(c_int::from(b'>')));

        // Automatic marks are not user-settable
        assert!(!rs_mark_is_user_settable(c_int::from(b'"')));
        assert!(!rs_mark_is_user_settable(c_int::from(b'^')));
        assert!(!rs_mark_is_user_settable(c_int::from(b'.')));
    }
}
