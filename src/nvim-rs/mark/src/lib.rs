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
    matches!(
        c,
        b'"' | b'^' | b'.' | b'[' | b']' | b'<' | b'>' | b'\'' | b'`'
    )
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
    ascii_islower(c) || ascii_isupper(c) || ascii_isdigit(c) || c == b'"' || c == b'^' || c == b'.'
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

// =============================================================================
// Phase 1: Mark View and Memory Operations
// =============================================================================

/// linenr_T equivalent from Neovim
pub type LinenrT = i32;

/// MAXLNUM value - represents no view
pub const MAXLNUM: LinenrT = 0x7fffffff;

/// fmarkv_T structure matching Neovim's mark_defs.h
/// Represents view in which the mark was created
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FmarkvT {
    /// Amount of lines from the mark lnum to the top of the window.
    /// Use MAXLNUM to indicate that the mark does not have a view.
    pub topline_offset: LinenrT,
}

/// Create a new fmarkv_T with MAXLNUM (no view).
#[no_mangle]
pub extern "C" fn rs_fmarkv_init() -> FmarkvT {
    FmarkvT {
        topline_offset: MAXLNUM,
    }
}

/// Create an fmarkv_T from topline and position.
///
/// This stores the offset between the mark's line number and the window's
/// topline, allowing the view to be restored later.
///
/// # Arguments
/// * `topline` - The window's current topline
/// * `pos_lnum` - The mark's line number
///
/// # Returns
/// An fmarkv_T with the calculated topline offset
#[no_mangle]
pub extern "C" fn rs_mark_view_make(topline: LinenrT, pos_lnum: LinenrT) -> FmarkvT {
    FmarkvT {
        topline_offset: pos_lnum - topline,
    }
}

/// Calculate the topline to restore from a mark view.
///
/// This computes the topline based on the mark's line number and the stored
/// topline offset. Returns -1 if the view should not be restored (offset >= MAXLNUM
/// or calculated topline < 1).
///
/// # Arguments
/// * `mark_lnum` - The mark's line number
/// * `topline_offset` - The stored topline offset from fmarkv_T
///
/// # Returns
/// The topline to set, or -1 if view should not be restored
#[no_mangle]
pub extern "C" fn rs_mark_view_calc_topline(
    mark_lnum: LinenrT,
    topline_offset: LinenrT,
) -> LinenrT {
    // If topline_offset is MAXLNUM (no view) or negative, don't restore view
    if !(0..MAXLNUM).contains(&topline_offset) {
        return -1;
    }

    let topline = mark_lnum - topline_offset;
    if topline >= 1 {
        topline
    } else {
        -1
    }
}

/// Check if an fmarkv_T has a valid view.
#[no_mangle]
pub extern "C" fn rs_fmarkv_has_view(view: FmarkvT) -> c_int {
    c_int::from((0..MAXLNUM).contains(&view.topline_offset))
}

// =============================================================================
// Phase 1 Tests
// =============================================================================

#[cfg(test)]
mod phase1_tests {
    use super::*;

    #[test]
    fn test_fmarkv_init() {
        let view = rs_fmarkv_init();
        assert_eq!(view.topline_offset, MAXLNUM);
    }

    #[test]
    fn test_mark_view_make() {
        // Normal case: mark at line 10, topline at line 5
        let view = rs_mark_view_make(5, 10);
        assert_eq!(view.topline_offset, 5); // 10 - 5 = 5

        // Mark at topline
        let view = rs_mark_view_make(10, 10);
        assert_eq!(view.topline_offset, 0);

        // Mark above topline (shouldn't happen in practice, but handle it)
        let view = rs_mark_view_make(10, 5);
        assert_eq!(view.topline_offset, -5);
    }

    #[test]
    fn test_mark_view_calc_topline() {
        // Normal case: mark at line 10, offset 5 -> topline should be 5
        let topline = rs_mark_view_calc_topline(10, 5);
        assert_eq!(topline, 5);

        // Mark at line 10, offset 0 -> topline should be 10
        let topline = rs_mark_view_calc_topline(10, 0);
        assert_eq!(topline, 10);

        // MAXLNUM offset (no view) -> should return -1
        let topline = rs_mark_view_calc_topline(10, MAXLNUM);
        assert_eq!(topline, -1);

        // Negative offset -> should return -1
        let topline = rs_mark_view_calc_topline(10, -1);
        assert_eq!(topline, -1);

        // Calculated topline would be < 1 -> should return -1
        let topline = rs_mark_view_calc_topline(1, 5);
        assert_eq!(topline, -1); // 1 - 5 = -4, which is < 1
    }

    #[test]
    fn test_fmarkv_has_view() {
        // Valid view with offset 0
        let view = FmarkvT { topline_offset: 0 };
        assert_ne!(rs_fmarkv_has_view(view), 0);

        // Valid view with positive offset
        let view = FmarkvT { topline_offset: 10 };
        assert_ne!(rs_fmarkv_has_view(view), 0);

        // No view (MAXLNUM)
        let view = FmarkvT {
            topline_offset: MAXLNUM,
        };
        assert_eq!(rs_fmarkv_has_view(view), 0);

        // Invalid view (negative)
        let view = FmarkvT { topline_offset: -1 };
        assert_eq!(rs_fmarkv_has_view(view), 0);
    }

    #[test]
    fn test_view_roundtrip() {
        // Create a view at mark line 100, topline 50
        let view = rs_mark_view_make(50, 100);
        assert_eq!(view.topline_offset, 50);

        // Restore the view - should get topline 50
        let restored_topline = rs_mark_view_calc_topline(100, view.topline_offset);
        assert_eq!(restored_topline, 50);
    }
}

// =============================================================================
// Phase 2: Mark Structures and Validation
// =============================================================================

/// Timestamp type matching Neovim's time_defs.h
pub type Timestamp = u64;

/// colnr_T equivalent from Neovim
pub type ColnrT = i32;

/// MAXCOL value - represents maximum column
pub const MAXCOL: ColnrT = 0x7fffffff;

/// Opaque pointer to AdditionalData from C
#[repr(C)]
pub struct AdditionalData {
    _private: [u8; 0],
}

/// fmark_T structure matching Neovim's mark_defs.h
/// Structure defining single local mark
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FmarkT {
    /// Cursor position
    pub mark: PosT,
    /// File number
    pub fnum: c_int,
    /// Time when this mark was last set
    pub timestamp: Timestamp,
    /// View the mark was created on
    pub view: FmarkvT,
    /// Additional data from ShaDa file (opaque pointer)
    pub additional_data: *mut AdditionalData,
}

impl Default for FmarkT {
    fn default() -> Self {
        FmarkT {
            mark: PosT::default(),
            fnum: 0,
            timestamp: 0,
            view: FmarkvT {
                topline_offset: MAXLNUM,
            },
            additional_data: std::ptr::null_mut(),
        }
    }
}

/// xfmark_T structure matching Neovim's mark_defs.h
/// Structure defining extended mark (mark with file name attached)
#[repr(C)]
pub struct XfmarkT {
    /// Actual mark
    pub fmark: FmarkT,
    /// File name, used when fnum == 0
    pub fname: *mut std::ffi::c_char,
}

/// Mark validation result codes
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkValidation {
    /// Mark is valid
    Valid = 0,
    /// Mark pointer is NULL (unknown mark)
    NullMark = 1,
    /// Mark line number is 0 (mark not set)
    NotSet = 2,
    /// Mark line number is negative (invalid)
    Negative = 3,
    /// Mark line number exceeds buffer line count
    OutOfBounds = 4,
}

/// Validate a mark's position.
///
/// Checks for:
/// - Line number <= 0 (mark not set or invalid)
///
/// # Arguments
/// * `mark_lnum` - The mark's line number
///
/// # Returns
/// MarkValidation indicating the result
#[no_mangle]
pub extern "C" fn rs_mark_validate_lnum(mark_lnum: LinenrT) -> MarkValidation {
    if mark_lnum == 0 {
        MarkValidation::NotSet
    } else if mark_lnum < 0 {
        MarkValidation::Negative
    } else {
        MarkValidation::Valid
    }
}

/// Validate a mark's line number against buffer bounds.
///
/// # Arguments
/// * `mark_lnum` - The mark's line number
/// * `buf_line_count` - The buffer's line count
///
/// # Returns
/// MarkValidation indicating the result
#[no_mangle]
pub extern "C" fn rs_mark_validate_bounds(
    mark_lnum: LinenrT,
    buf_line_count: LinenrT,
) -> MarkValidation {
    let lnum_valid = rs_mark_validate_lnum(mark_lnum);
    if lnum_valid != MarkValidation::Valid {
        return lnum_valid;
    }
    if mark_lnum > buf_line_count {
        MarkValidation::OutOfBounds
    } else {
        MarkValidation::Valid
    }
}

/// Check if a mark line number is valid (> 0).
#[no_mangle]
pub extern "C" fn rs_mark_lnum_is_valid(mark_lnum: LinenrT) -> c_int {
    c_int::from(mark_lnum > 0)
}

/// Check if a mark line number is within buffer bounds.
#[no_mangle]
pub extern "C" fn rs_mark_lnum_in_bounds(mark_lnum: LinenrT, buf_line_count: LinenrT) -> c_int {
    c_int::from(mark_lnum > 0 && mark_lnum <= buf_line_count)
}

/// Initialize an fmark_T with default values.
///
/// # Safety
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_fmark_init(fm: *mut FmarkT) {
    if fm.is_null() {
        return;
    }
    (*fm).mark = PosT::default();
    (*fm).fnum = 0;
    (*fm).timestamp = 0;
    (*fm).view.topline_offset = MAXLNUM;
    (*fm).additional_data = std::ptr::null_mut();
}

/// Check if an fmark_T has a valid mark position (lnum > 0).
#[no_mangle]
pub extern "C" fn rs_fmark_is_set(fm: FmarkT) -> c_int {
    c_int::from(fm.mark.lnum > 0)
}

/// Get the line number from an fmark_T.
#[no_mangle]
pub extern "C" fn rs_fmark_get_lnum(fm: FmarkT) -> LinenrT {
    fm.mark.lnum
}

/// Get the column from an fmark_T.
#[no_mangle]
pub extern "C" fn rs_fmark_get_col(fm: FmarkT) -> ColnrT {
    fm.mark.col
}

/// Get the file number from an fmark_T.
#[no_mangle]
pub extern "C" fn rs_fmark_get_fnum(fm: FmarkT) -> c_int {
    fm.fnum
}

/// Get the timestamp from an fmark_T.
#[no_mangle]
pub extern "C" fn rs_fmark_get_timestamp(fm: FmarkT) -> Timestamp {
    fm.timestamp
}

/// Set the mark position in an fmark_T.
///
/// # Safety
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_fmark_set_pos(fm: *mut FmarkT, lnum: LinenrT, col: ColnrT) {
    if fm.is_null() {
        return;
    }
    (*fm).mark.lnum = lnum;
    (*fm).mark.col = col;
}

/// Set the file number in an fmark_T.
///
/// # Safety
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_fmark_set_fnum(fm: *mut FmarkT, fnum: c_int) {
    if fm.is_null() {
        return;
    }
    (*fm).fnum = fnum;
}

/// Set the timestamp in an fmark_T.
///
/// # Safety
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_fmark_set_timestamp(fm: *mut FmarkT, timestamp: Timestamp) {
    if fm.is_null() {
        return;
    }
    (*fm).timestamp = timestamp;
}

/// Copy an fmark_T from source to destination.
///
/// # Safety
/// Both pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_fmark_copy(dst: *mut FmarkT, src: *const FmarkT) {
    if dst.is_null() || src.is_null() {
        return;
    }
    // Don't copy additional_data pointer - that needs special handling
    (*dst).mark = (*src).mark;
    (*dst).fnum = (*src).fnum;
    (*dst).timestamp = (*src).timestamp;
    (*dst).view = (*src).view;
}

/// Compare two positions and determine visual order.
/// Returns which position should be considered "start" vs "end" for visual selection.
///
/// This implements the logic: if name == '<', return the lesser position;
/// if name == '>', return the greater position.
///
/// # Arguments
/// * `start_lnum`, `start_col` - First position (vi_start)
/// * `end_lnum`, `end_col` - Second position (vi_end)
/// * `name` - Mark name ('<' or '>')
///
/// # Returns
/// 0 to use start position, 1 to use end position
#[no_mangle]
pub extern "C" fn rs_visual_mark_select(
    start_lnum: LinenrT,
    start_col: ColnrT,
    end_lnum: LinenrT,
    end_col: ColnrT,
    name: c_int,
) -> c_int {
    let start = PosT {
        lnum: start_lnum,
        col: start_col,
        coladd: 0,
    };
    let end = PosT {
        lnum: end_lnum,
        col: end_col,
        coladd: 0,
    };

    let start_is_less = rs_lt(start, end) != 0;

    // '<' wants the lesser position, '>' wants the greater
    // But also handle edge cases: if end.lnum == 0 or start.lnum == 0
    let name_is_less = name == c_int::from(b'<');

    if end_lnum == 0 && start_lnum != 0 {
        // End is invalid, use start
        return 0;
    }

    if (name_is_less == start_is_less || end_lnum == 0) && start_lnum != 0 {
        0 // use start
    } else {
        1 // use end
    }
}

// =============================================================================
// Phase 2 Tests
// =============================================================================

#[cfg(test)]
mod phase2_tests {
    use super::*;

    #[test]
    fn test_mark_validate_lnum() {
        assert_eq!(rs_mark_validate_lnum(1), MarkValidation::Valid);
        assert_eq!(rs_mark_validate_lnum(100), MarkValidation::Valid);
        assert_eq!(rs_mark_validate_lnum(0), MarkValidation::NotSet);
        assert_eq!(rs_mark_validate_lnum(-1), MarkValidation::Negative);
    }

    #[test]
    fn test_mark_validate_bounds() {
        // Valid cases
        assert_eq!(rs_mark_validate_bounds(1, 100), MarkValidation::Valid);
        assert_eq!(rs_mark_validate_bounds(100, 100), MarkValidation::Valid);

        // Out of bounds
        assert_eq!(
            rs_mark_validate_bounds(101, 100),
            MarkValidation::OutOfBounds
        );

        // Invalid lnum
        assert_eq!(rs_mark_validate_bounds(0, 100), MarkValidation::NotSet);
        assert_eq!(rs_mark_validate_bounds(-1, 100), MarkValidation::Negative);
    }

    #[test]
    fn test_mark_lnum_checks() {
        assert_ne!(rs_mark_lnum_is_valid(1), 0);
        assert_eq!(rs_mark_lnum_is_valid(0), 0);
        assert_eq!(rs_mark_lnum_is_valid(-1), 0);

        assert_ne!(rs_mark_lnum_in_bounds(1, 100), 0);
        assert_ne!(rs_mark_lnum_in_bounds(100, 100), 0);
        assert_eq!(rs_mark_lnum_in_bounds(101, 100), 0);
        assert_eq!(rs_mark_lnum_in_bounds(0, 100), 0);
    }

    #[test]
    fn test_fmark_default() {
        let fm = FmarkT::default();
        assert_eq!(fm.mark.lnum, 0);
        assert_eq!(fm.mark.col, 0);
        assert_eq!(fm.fnum, 0);
        assert_eq!(fm.timestamp, 0);
        assert_eq!(fm.view.topline_offset, MAXLNUM);
        assert!(fm.additional_data.is_null());
    }

    #[test]
    fn test_fmark_init() {
        let mut fm = FmarkT {
            mark: PosT {
                lnum: 10,
                col: 5,
                coladd: 2,
            },
            fnum: 1,
            timestamp: 12345,
            view: FmarkvT { topline_offset: 3 },
            additional_data: std::ptr::null_mut(),
        };

        unsafe { rs_fmark_init(&mut fm) };

        assert_eq!(fm.mark.lnum, 0);
        assert_eq!(fm.mark.col, 0);
        assert_eq!(fm.fnum, 0);
        assert_eq!(fm.timestamp, 0);
        assert_eq!(fm.view.topline_offset, MAXLNUM);
    }

    #[test]
    fn test_fmark_is_set() {
        let mut fm = FmarkT::default();
        assert_eq!(rs_fmark_is_set(fm), 0);

        fm.mark.lnum = 1;
        assert_ne!(rs_fmark_is_set(fm), 0);

        fm.mark.lnum = -1;
        assert_eq!(rs_fmark_is_set(fm), 0);
    }

    #[test]
    fn test_fmark_getters() {
        let fm = FmarkT {
            mark: PosT {
                lnum: 10,
                col: 5,
                coladd: 2,
            },
            fnum: 3,
            timestamp: 12345,
            view: FmarkvT { topline_offset: 3 },
            additional_data: std::ptr::null_mut(),
        };

        assert_eq!(rs_fmark_get_lnum(fm), 10);
        assert_eq!(rs_fmark_get_col(fm), 5);
        assert_eq!(rs_fmark_get_fnum(fm), 3);
        assert_eq!(rs_fmark_get_timestamp(fm), 12345);
    }

    #[test]
    fn test_fmark_setters() {
        let mut fm = FmarkT::default();

        unsafe {
            rs_fmark_set_pos(&mut fm, 10, 5);
            rs_fmark_set_fnum(&mut fm, 3);
            rs_fmark_set_timestamp(&mut fm, 12345);
        }

        assert_eq!(fm.mark.lnum, 10);
        assert_eq!(fm.mark.col, 5);
        assert_eq!(fm.fnum, 3);
        assert_eq!(fm.timestamp, 12345);
    }

    #[test]
    fn test_fmark_copy() {
        let src = FmarkT {
            mark: PosT {
                lnum: 10,
                col: 5,
                coladd: 2,
            },
            fnum: 3,
            timestamp: 12345,
            view: FmarkvT { topline_offset: 7 },
            additional_data: std::ptr::null_mut(),
        };
        let mut dst = FmarkT::default();

        unsafe { rs_fmark_copy(&mut dst, &src) };

        assert_eq!(dst.mark.lnum, 10);
        assert_eq!(dst.mark.col, 5);
        assert_eq!(dst.mark.coladd, 2);
        assert_eq!(dst.fnum, 3);
        assert_eq!(dst.timestamp, 12345);
        assert_eq!(dst.view.topline_offset, 7);
    }

    #[test]
    fn test_visual_mark_select() {
        // '<' should select lesser position
        // start < end, name = '<' -> use start (0)
        assert_eq!(rs_visual_mark_select(1, 0, 10, 0, c_int::from(b'<')), 0);

        // start > end, name = '<' -> use end (1)
        assert_eq!(rs_visual_mark_select(10, 0, 1, 0, c_int::from(b'<')), 1);

        // '>' should select greater position
        // start < end, name = '>' -> use end (1)
        assert_eq!(rs_visual_mark_select(1, 0, 10, 0, c_int::from(b'>')), 1);

        // start > end, name = '>' -> use start (0)
        assert_eq!(rs_visual_mark_select(10, 0, 1, 0, c_int::from(b'>')), 0);

        // Edge case: end.lnum == 0, start.lnum != 0 -> use start
        assert_eq!(rs_visual_mark_select(5, 0, 0, 0, c_int::from(b'<')), 0);
        assert_eq!(rs_visual_mark_select(5, 0, 0, 0, c_int::from(b'>')), 0);
    }
}

// =============================================================================
// Phase 3 & 5: Jumplist and Changelist Operations
// =============================================================================

/// Maximum number of marks in jump list
pub const JUMPLISTSIZE: c_int = 100;

/// Maximum number of marks in change list
pub const GETMARKLIST_MAXCHANGES: c_int = 100;

/// Calculate the new jumplist length after incrementing.
///
/// Implements the logic: if ++len > JUMPLISTSIZE, len = JUMPLISTSIZE
///
/// # Arguments
/// * `current_len` - Current jumplist length
///
/// # Returns
/// New jumplist length (clamped to JUMPLISTSIZE)
#[no_mangle]
pub extern "C" fn rs_jumplist_new_len(current_len: c_int) -> c_int {
    let new_len = current_len + 1;
    if new_len > JUMPLISTSIZE {
        JUMPLISTSIZE
    } else {
        new_len
    }
}

/// Check if jumplist is full and needs oldest entry removed.
///
/// # Arguments
/// * `current_len` - Current jumplist length before increment
///
/// # Returns
/// 1 if full (oldest entry should be removed), 0 otherwise
#[no_mangle]
pub extern "C" fn rs_jumplist_is_full(current_len: c_int) -> c_int {
    c_int::from(current_len >= JUMPLISTSIZE)
}

/// Calculate jumplist trim length for stack mode.
///
/// When jumpoptions=stack, discard everything after current index.
///
/// # Arguments
/// * `idx` - Current jumplist index
/// * `len` - Current jumplist length
///
/// # Returns
/// New length if trim needed, or -1 if no trim needed
#[no_mangle]
pub extern "C" fn rs_jumplist_stack_trim(idx: c_int, len: c_int) -> c_int {
    if idx < len - 1 {
        idx + 1
    } else {
        -1 // No trim needed
    }
}

/// Calculate new jumplist index after a jump.
///
/// # Arguments
/// * `current_idx` - Current jumplist index
/// * `current_len` - Current jumplist length
/// * `count` - Jump count (negative for backward, positive for forward)
///
/// # Returns
/// New index, or -1 if out of bounds
#[no_mangle]
pub extern "C" fn rs_jumplist_calc_idx(
    current_idx: c_int,
    current_len: c_int,
    count: c_int,
) -> c_int {
    let new_idx = current_idx + count;
    if new_idx < 0 || new_idx >= current_len {
        -1
    } else {
        new_idx
    }
}

/// Calculate new changelist index after navigation.
///
/// # Arguments
/// * `current_idx` - Current changelist index
/// * `changelist_len` - Changelist length
/// * `count` - Navigation count (negative for backward, positive for forward)
///
/// # Returns
/// (new_idx, clamped) - new_idx is the calculated index, clamped indicates if the
/// value was clamped to bounds. Returns (-1, 0) if navigation not possible.
#[no_mangle]
pub extern "C" fn rs_changelist_calc_idx(
    current_idx: c_int,
    changelist_len: c_int,
    count: c_int,
) -> c_int {
    let n = current_idx;
    if n + count < 0 {
        if n == 0 {
            return -1; // Can't navigate further back
        }
        return 0; // Clamp to start
    } else if n + count >= changelist_len {
        if n == changelist_len - 1 {
            return -1; // Can't navigate further forward
        }
        return changelist_len - 1; // Clamp to end
    }
    n + count
}

/// Determine the target mark based on mark name.
///
/// Returns a code indicating which mark storage should be used:
/// - 0: Invalid/not handled
/// - 1: Global mark (A-Z, 0-9)
/// - 2: Local named mark (a-z)
/// - 3: Jump mark (' or `)
/// - 4: Last cursor mark (")
/// - 5: Sentence start ([)
/// - 6: Sentence end (])
/// - 7: Visual start (<)
/// - 8: Visual end (>)
/// - 9: Last insert (^)
/// - 10: Last change (.)
/// - 11: Prompt mark (:)
#[no_mangle]
pub extern "C" fn rs_mark_target_type(name: c_int) -> c_int {
    let Ok(c) = u8::try_from(name) else {
        return 0;
    };

    if ascii_isupper(c) || ascii_isdigit(c) {
        1 // Global mark
    } else if ascii_islower(c) {
        2 // Local named mark
    } else {
        match c {
            b'\'' | b'`' => 3, // Jump mark
            b'"' => 4,         // Last cursor
            b'[' => 5,         // Sentence start
            b']' => 6,         // Sentence end
            b'<' => 7,         // Visual start
            b'>' => 8,         // Visual end
            b'^' => 9,         // Last insert
            b'.' => 10,        // Last change
            b':' => 11,        // Prompt mark
            _ => 0,            // Not handled
        }
    }
}

/// Position clamp operation for mark setting.
///
/// Ensures lnum is at least 1 (valid for Vim positions).
#[no_mangle]
pub extern "C" fn rs_pos_clamp_lnum_min(lnum: LinenrT) -> LinenrT {
    if lnum < 1 {
        1
    } else {
        lnum
    }
}

// =============================================================================
// Phase 3 & 5 Tests
// =============================================================================

#[cfg(test)]
mod phase35_tests {
    use super::*;

    #[test]
    fn test_jumplist_new_len() {
        // Normal increment
        assert_eq!(rs_jumplist_new_len(0), 1);
        assert_eq!(rs_jumplist_new_len(50), 51);
        assert_eq!(rs_jumplist_new_len(99), 100);

        // At max, should stay at max
        assert_eq!(rs_jumplist_new_len(100), 100);
        assert_eq!(rs_jumplist_new_len(200), 100);
    }

    #[test]
    fn test_jumplist_is_full() {
        assert_eq!(rs_jumplist_is_full(99), 0);
        assert_ne!(rs_jumplist_is_full(100), 0);
        assert_ne!(rs_jumplist_is_full(150), 0);
    }

    #[test]
    fn test_jumplist_stack_trim() {
        // idx < len - 1: should trim
        assert_eq!(rs_jumplist_stack_trim(5, 10), 6);
        assert_eq!(rs_jumplist_stack_trim(0, 10), 1);

        // idx >= len - 1: no trim needed
        assert_eq!(rs_jumplist_stack_trim(9, 10), -1);
        assert_eq!(rs_jumplist_stack_trim(10, 10), -1);
    }

    #[test]
    fn test_jumplist_calc_idx() {
        // Valid jumps
        assert_eq!(rs_jumplist_calc_idx(5, 10, -2), 3);
        assert_eq!(rs_jumplist_calc_idx(5, 10, 2), 7);
        assert_eq!(rs_jumplist_calc_idx(0, 10, 0), 0);

        // Out of bounds
        assert_eq!(rs_jumplist_calc_idx(0, 10, -1), -1);
        assert_eq!(rs_jumplist_calc_idx(9, 10, 1), -1);
    }

    #[test]
    fn test_changelist_calc_idx() {
        // Valid navigation
        assert_eq!(rs_changelist_calc_idx(5, 10, -2), 3);
        assert_eq!(rs_changelist_calc_idx(5, 10, 2), 7);

        // Clamp to start
        assert_eq!(rs_changelist_calc_idx(2, 10, -5), 0);

        // Clamp to end
        assert_eq!(rs_changelist_calc_idx(7, 10, 5), 9);

        // Already at boundary, can't navigate
        assert_eq!(rs_changelist_calc_idx(0, 10, -1), -1);
        assert_eq!(rs_changelist_calc_idx(9, 10, 1), -1);
    }

    #[test]
    fn test_mark_target_type() {
        // Global marks
        assert_eq!(rs_mark_target_type(c_int::from(b'A')), 1);
        assert_eq!(rs_mark_target_type(c_int::from(b'Z')), 1);
        assert_eq!(rs_mark_target_type(c_int::from(b'0')), 1);

        // Local named marks
        assert_eq!(rs_mark_target_type(c_int::from(b'a')), 2);
        assert_eq!(rs_mark_target_type(c_int::from(b'z')), 2);

        // Special marks
        assert_eq!(rs_mark_target_type(c_int::from(b'\'')), 3);
        assert_eq!(rs_mark_target_type(c_int::from(b'`')), 3);
        assert_eq!(rs_mark_target_type(c_int::from(b'"')), 4);
        assert_eq!(rs_mark_target_type(c_int::from(b'[')), 5);
        assert_eq!(rs_mark_target_type(c_int::from(b']')), 6);
        assert_eq!(rs_mark_target_type(c_int::from(b'<')), 7);
        assert_eq!(rs_mark_target_type(c_int::from(b'>')), 8);
        assert_eq!(rs_mark_target_type(c_int::from(b'^')), 9);
        assert_eq!(rs_mark_target_type(c_int::from(b'.')), 10);
        assert_eq!(rs_mark_target_type(c_int::from(b':')), 11);

        // Invalid
        assert_eq!(rs_mark_target_type(c_int::from(b'@')), 0);
        assert_eq!(rs_mark_target_type(-1), 0);
    }

    #[test]
    fn test_pos_clamp_lnum_min() {
        assert_eq!(rs_pos_clamp_lnum_min(5), 5);
        assert_eq!(rs_pos_clamp_lnum_min(1), 1);
        assert_eq!(rs_pos_clamp_lnum_min(0), 1);
        assert_eq!(rs_pos_clamp_lnum_min(-1), 1);
    }
}

// =============================================================================
// Phase 4: Mark Movement Functions
// =============================================================================

/// Flags for outcomes when moving to a mark.
/// These match MarkMoveRes in mark_defs.h
pub mod mark_move_res {
    pub const SUCCESS: i32 = 1;
    pub const FAILED: i32 = 2;
    pub const SWITCHED_BUF: i32 = 4;
    pub const CHANGED_COL: i32 = 8;
    pub const CHANGED_LINE: i32 = 16;
    pub const CHANGED_CURSOR: i32 = 32;
    pub const CHANGED_VIEW: i32 = 64;
}

/// Flags to configure the movement to a mark.
/// These match MarkMove in mark_defs.h
pub mod mark_move_flags {
    pub const BEGIN_LINE: i32 = 1;
    pub const CONTEXT: i32 = 2;
    pub const NO_CONTEXT: i32 = 4;
    pub const SET_VIEW: i32 = 8;
    pub const JUMP_LIST: i32 = 16;
}

/// Direction constants for mark searching
pub const FORWARD: c_int = 1;
pub const BACKWARD: c_int = -1;

/// Calculate MarkMoveRes flags based on position changes.
///
/// # Arguments
/// * `prev_lnum`, `prev_col` - Previous cursor position
/// * `new_lnum`, `new_col` - New cursor position
/// * `initial_res` - Initial result flags
///
/// # Returns
/// Updated result flags with CHANGED_LINE, CHANGED_COL, CHANGED_CURSOR set appropriately
#[no_mangle]
pub extern "C" fn rs_mark_move_calc_result(
    prev_lnum: LinenrT,
    prev_col: ColnrT,
    new_lnum: LinenrT,
    new_col: ColnrT,
    initial_res: c_int,
) -> c_int {
    let mut res = initial_res;
    if prev_lnum != new_lnum {
        res |= mark_move_res::CHANGED_LINE | mark_move_res::CHANGED_CURSOR;
    }
    if prev_col != new_col {
        res |= mark_move_res::CHANGED_COL | mark_move_res::CHANGED_CURSOR;
    }
    res
}

/// Check if mark_move_to should do additional cursor checking.
///
/// # Arguments
/// * `res` - Current result flags
///
/// # Returns
/// Non-zero if cursor check should be performed
#[no_mangle]
pub extern "C" fn rs_mark_move_needs_cursor_check(res: c_int) -> c_int {
    c_int::from(
        (res & mark_move_res::SWITCHED_BUF) != 0 || (res & mark_move_res::CHANGED_CURSOR) != 0,
    )
}

/// Prepare column for getnextmark search based on direction and begin_line.
///
/// # Arguments
/// * `col` - Current column
/// * `dir` - Direction (FORWARD or BACKWARD)
/// * `begin_line` - Whether to search from beginning of line
///
/// # Returns
/// Adjusted column value for the search
#[no_mangle]
pub extern "C" fn rs_getnextmark_adjust_col(col: ColnrT, dir: c_int, begin_line: c_int) -> ColnrT {
    if begin_line != 0 {
        if dir == BACKWARD {
            0
        } else {
            MAXCOL
        }
    } else {
        col
    }
}

/// Compare positions for getnextmark search.
///
/// Implements the logic for finding the next/previous mark relative to a position.
///
/// # Arguments
/// * `candidate_lnum`, `candidate_col` - Position of the candidate mark
/// * `current_best_lnum`, `current_best_col` - Position of the current best match (use 0,0 if none)
/// * `start_lnum`, `start_col` - Position to search from
/// * `dir` - Direction (FORWARD or BACKWARD)
///
/// # Returns
/// Non-zero if candidate is better than current_best
#[no_mangle]
pub extern "C" fn rs_getnextmark_is_better(
    candidate_lnum: LinenrT,
    candidate_col: ColnrT,
    current_best_lnum: LinenrT,
    current_best_col: ColnrT,
    start_lnum: LinenrT,
    start_col: ColnrT,
    dir: c_int,
) -> c_int {
    // Skip invalid candidates
    if candidate_lnum <= 0 {
        return 0;
    }

    let candidate = PosT {
        lnum: candidate_lnum,
        col: candidate_col,
        coladd: 0,
    };
    let start = PosT {
        lnum: start_lnum,
        col: start_col,
        coladd: 0,
    };
    let no_best = current_best_lnum == 0;

    if dir == FORWARD {
        // For forward: candidate must be after start, and closer than current best
        let after_start = rs_lt(start, candidate) != 0;
        if !after_start {
            return 0;
        }
        if no_best {
            return 1;
        }
        let best = PosT {
            lnum: current_best_lnum,
            col: current_best_col,
            coladd: 0,
        };
        c_int::from(rs_lt(candidate, best) != 0)
    } else {
        // For backward: candidate must be before start, and closer than current best
        let before_start = rs_lt(candidate, start) != 0;
        if !before_start {
            return 0;
        }
        if no_best {
            return 1;
        }
        let best = PosT {
            lnum: current_best_lnum,
            col: current_best_col,
            coladd: 0,
        };
        c_int::from(rs_lt(best, candidate) != 0)
    }
}

// =============================================================================
// Phase 4 Tests
// =============================================================================

#[cfg(test)]
mod phase4_tests {
    use super::*;

    #[test]
    fn test_mark_move_calc_result() {
        // No change
        let res = rs_mark_move_calc_result(10, 5, 10, 5, mark_move_res::SUCCESS);
        assert_eq!(res, mark_move_res::SUCCESS);

        // Line changed
        let res = rs_mark_move_calc_result(10, 5, 20, 5, mark_move_res::SUCCESS);
        assert_ne!(res & mark_move_res::CHANGED_LINE, 0);
        assert_ne!(res & mark_move_res::CHANGED_CURSOR, 0);
        assert_eq!(res & mark_move_res::CHANGED_COL, 0);

        // Column changed
        let res = rs_mark_move_calc_result(10, 5, 10, 15, mark_move_res::SUCCESS);
        assert_ne!(res & mark_move_res::CHANGED_COL, 0);
        assert_ne!(res & mark_move_res::CHANGED_CURSOR, 0);
        assert_eq!(res & mark_move_res::CHANGED_LINE, 0);

        // Both changed
        let res = rs_mark_move_calc_result(10, 5, 20, 15, mark_move_res::SUCCESS);
        assert_ne!(res & mark_move_res::CHANGED_LINE, 0);
        assert_ne!(res & mark_move_res::CHANGED_COL, 0);
        assert_ne!(res & mark_move_res::CHANGED_CURSOR, 0);
    }

    #[test]
    fn test_mark_move_needs_cursor_check() {
        assert_eq!(rs_mark_move_needs_cursor_check(mark_move_res::SUCCESS), 0);
        assert_ne!(
            rs_mark_move_needs_cursor_check(mark_move_res::SWITCHED_BUF),
            0
        );
        assert_ne!(
            rs_mark_move_needs_cursor_check(mark_move_res::CHANGED_CURSOR),
            0
        );
        assert_ne!(
            rs_mark_move_needs_cursor_check(
                mark_move_res::SWITCHED_BUF | mark_move_res::CHANGED_CURSOR
            ),
            0
        );
    }

    #[test]
    fn test_getnextmark_adjust_col() {
        // No begin_line adjustment
        assert_eq!(rs_getnextmark_adjust_col(5, FORWARD, 0), 5);
        assert_eq!(rs_getnextmark_adjust_col(5, BACKWARD, 0), 5);

        // begin_line adjustment
        assert_eq!(rs_getnextmark_adjust_col(5, FORWARD, 1), MAXCOL);
        assert_eq!(rs_getnextmark_adjust_col(5, BACKWARD, 1), 0);
    }

    #[test]
    fn test_getnextmark_is_better_forward() {
        // Forward from (10, 5): looking for marks after this position
        // No current best (0, 0), candidate at (20, 5) - should be better
        assert_ne!(rs_getnextmark_is_better(20, 5, 0, 0, 10, 5, FORWARD), 0);

        // Candidate before start - not better
        assert_eq!(rs_getnextmark_is_better(5, 5, 0, 0, 10, 5, FORWARD), 0);

        // Candidate closer than current best
        assert_ne!(rs_getnextmark_is_better(15, 5, 20, 5, 10, 5, FORWARD), 0);

        // Candidate farther than current best
        assert_eq!(rs_getnextmark_is_better(25, 5, 20, 5, 10, 5, FORWARD), 0);

        // Invalid candidate (lnum <= 0)
        assert_eq!(rs_getnextmark_is_better(0, 5, 0, 0, 10, 5, FORWARD), 0);
    }

    #[test]
    fn test_getnextmark_is_better_backward() {
        // Backward from (20, 5): looking for marks before this position
        // No current best, candidate at (10, 5) - should be better
        assert_ne!(rs_getnextmark_is_better(10, 5, 0, 0, 20, 5, BACKWARD), 0);

        // Candidate after start - not better
        assert_eq!(rs_getnextmark_is_better(25, 5, 0, 0, 20, 5, BACKWARD), 0);

        // Candidate closer than current best (closer means higher for backward)
        assert_ne!(rs_getnextmark_is_better(15, 5, 10, 5, 20, 5, BACKWARD), 0);

        // Candidate farther than current best
        assert_eq!(rs_getnextmark_is_better(5, 5, 10, 5, 20, 5, BACKWARD), 0);
    }
}

// =============================================================================
// Phase 6: Mark Adjustment Functions
// =============================================================================

/// Result of a line number adjustment.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineAdjustResult {
    /// New line number after adjustment
    pub new_lnum: LinenrT,
    /// Whether the line was modified
    pub modified: c_int,
}

/// Adjust a line number based on line deletion/insertion.
///
/// Implements ONE_ADJUST logic:
/// - If lnum in [line1, line2]: add amount (or set to 0 if amount is MAXLNUM)
/// - If lnum > line2: add amount_after
///
/// # Arguments
/// * `lnum` - The line number to adjust
/// * `line1` - Start of affected range
/// * `line2` - End of affected range
/// * `amount` - Amount to add for lines in range (MAXLNUM means delete)
/// * `amount_after` - Amount to add for lines after range
///
/// # Returns
/// LineAdjustResult with the new line number and modification flag
#[no_mangle]
pub extern "C" fn rs_mark_adjust_lnum(
    lnum: LinenrT,
    line1: LinenrT,
    line2: LinenrT,
    amount: LinenrT,
    amount_after: LinenrT,
) -> LineAdjustResult {
    if lnum >= line1 && lnum <= line2 {
        // Line is in the affected range
        if amount == MAXLNUM {
            // Deletion: set to 0
            LineAdjustResult {
                new_lnum: 0,
                modified: 1,
            }
        } else {
            LineAdjustResult {
                new_lnum: lnum + amount,
                modified: 1,
            }
        }
    } else if amount_after != 0 && lnum > line2 {
        // Line is after the range
        LineAdjustResult {
            new_lnum: lnum + amount_after,
            modified: 1,
        }
    } else {
        // No change
        LineAdjustResult {
            new_lnum: lnum,
            modified: 0,
        }
    }
}

/// Adjust a line number with no-delete behavior.
///
/// Implements ONE_ADJUST_NODEL logic:
/// - If lnum in [line1, line2]: add amount (or set to line1 if amount is MAXLNUM)
/// - If lnum > line2: add amount_after
///
/// # Arguments
/// * `lnum` - The line number to adjust
/// * `line1` - Start of affected range
/// * `line2` - End of affected range
/// * `amount` - Amount to add for lines in range (MAXLNUM means set to line1)
/// * `amount_after` - Amount to add for lines after range
///
/// # Returns
/// LineAdjustResult with the new line number and modification flag
#[no_mangle]
pub extern "C" fn rs_mark_adjust_lnum_nodel(
    lnum: LinenrT,
    line1: LinenrT,
    line2: LinenrT,
    amount: LinenrT,
    amount_after: LinenrT,
) -> LineAdjustResult {
    if lnum >= line1 && lnum <= line2 {
        // Line is in the affected range
        if amount == MAXLNUM {
            // No delete: set to line1
            LineAdjustResult {
                new_lnum: line1,
                modified: 1,
            }
        } else {
            LineAdjustResult {
                new_lnum: lnum + amount,
                modified: 1,
            }
        }
    } else if amount_after != 0 && lnum > line2 {
        // Line is after the range
        LineAdjustResult {
            new_lnum: lnum + amount_after,
            modified: 1,
        }
    } else {
        // No change
        LineAdjustResult {
            new_lnum: lnum,
            modified: 0,
        }
    }
}

/// Result of a cursor position adjustment.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorAdjustResult {
    /// New line number after adjustment
    pub new_lnum: LinenrT,
    /// New column after adjustment
    pub new_col: ColnrT,
    /// Whether the position was modified
    pub modified: c_int,
}

/// Adjust a cursor position based on line deletion/insertion.
///
/// Implements ONE_ADJUST_CURSOR logic:
/// - If lnum in [line1, line2] and amount is MAXLNUM: move to max(line1-1, 1), col 0
/// - If lnum in [line1, line2]: add amount to lnum
/// - If lnum > line2: add amount_after
///
/// # Arguments
/// * `lnum` - The line number to adjust
/// * `col` - The column to adjust
/// * `line1` - Start of affected range
/// * `line2` - End of affected range
/// * `amount` - Amount to add for lines in range (MAXLNUM means delete)
/// * `amount_after` - Amount to add for lines after range
///
/// # Returns
/// CursorAdjustResult with the new position and modification flag
#[no_mangle]
pub extern "C" fn rs_mark_adjust_cursor(
    lnum: LinenrT,
    col: ColnrT,
    line1: LinenrT,
    line2: LinenrT,
    amount: LinenrT,
    amount_after: LinenrT,
) -> CursorAdjustResult {
    if lnum >= line1 && lnum <= line2 {
        // Cursor is in the affected range
        if amount == MAXLNUM {
            // Line with cursor is deleted
            let new_lnum = std::cmp::max(line1 - 1, 1);
            CursorAdjustResult {
                new_lnum,
                new_col: 0,
                modified: 1,
            }
        } else {
            // Keep cursor on the same line
            CursorAdjustResult {
                new_lnum: lnum + amount,
                new_col: col,
                modified: 1,
            }
        }
    } else if amount_after != 0 && lnum > line2 {
        // Cursor is after the range
        CursorAdjustResult {
            new_lnum: lnum + amount_after,
            new_col: col,
            modified: 1,
        }
    } else {
        // No change
        CursorAdjustResult {
            new_lnum: lnum,
            new_col: col,
            modified: 0,
        }
    }
}

/// Result of a column adjustment.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColAdjustResult {
    /// New line number after adjustment
    pub new_lnum: LinenrT,
    /// New column after adjustment
    pub new_col: ColnrT,
    /// Whether the position was modified
    pub modified: c_int,
}

/// Adjust a position's column based on column changes.
///
/// Implements COL_ADJUST logic for mark_col_adjust.
///
/// # Arguments
/// * `pos_lnum` - Position's line number
/// * `pos_col` - Position's column
/// * `lnum` - Line being modified
/// * `mincol` - Minimum column affected
/// * `lnum_amount` - Amount to add to line number
/// * `col_amount` - Amount to add to column
/// * `spaces_removed` - Number of spaces removed
///
/// # Returns
/// ColAdjustResult with the new position and modification flag
#[no_mangle]
pub extern "C" fn rs_mark_col_adjust(
    pos_lnum: LinenrT,
    pos_col: ColnrT,
    lnum: LinenrT,
    mincol: ColnrT,
    lnum_amount: LinenrT,
    col_amount: ColnrT,
    spaces_removed: c_int,
) -> ColAdjustResult {
    if pos_lnum != lnum || pos_col < mincol {
        // Position not affected
        return ColAdjustResult {
            new_lnum: pos_lnum,
            new_col: pos_col,
            modified: 0,
        };
    }

    let new_lnum = pos_lnum + lnum_amount;
    let new_col = if col_amount < 0 && pos_col <= -col_amount {
        0
    } else if pos_col < spaces_removed {
        col_amount + spaces_removed
    } else {
        pos_col + col_amount
    };

    ColAdjustResult {
        new_lnum,
        new_col,
        modified: 1,
    }
}

/// Check if mark adjustment should be skipped.
///
/// # Arguments
/// * `line1` - Start of range
/// * `line2` - End of range
/// * `amount_after` - Amount for lines after range
///
/// # Returns
/// Non-zero if adjustment should be skipped
#[no_mangle]
pub extern "C" fn rs_mark_adjust_should_skip(
    line1: LinenrT,
    line2: LinenrT,
    amount_after: LinenrT,
) -> c_int {
    c_int::from(line2 < line1 && amount_after == 0)
}

// =============================================================================
// Phase 6 Tests
// =============================================================================

#[cfg(test)]
mod phase6_tests {
    use super::*;

    #[test]
    fn test_mark_adjust_lnum_in_range() {
        // Line in range, add amount
        let result = rs_mark_adjust_lnum(5, 3, 7, 2, 0);
        assert_eq!(result.new_lnum, 7); // 5 + 2
        assert_ne!(result.modified, 0);

        // Line in range, MAXLNUM (delete)
        let result = rs_mark_adjust_lnum(5, 3, 7, MAXLNUM, 0);
        assert_eq!(result.new_lnum, 0);
        assert_ne!(result.modified, 0);
    }

    #[test]
    fn test_mark_adjust_lnum_after_range() {
        // Line after range
        let result = rs_mark_adjust_lnum(10, 3, 7, 2, 3);
        assert_eq!(result.new_lnum, 13); // 10 + 3
        assert_ne!(result.modified, 0);
    }

    #[test]
    fn test_mark_adjust_lnum_no_change() {
        // Line before range
        let result = rs_mark_adjust_lnum(2, 3, 7, 2, 3);
        assert_eq!(result.new_lnum, 2);
        assert_eq!(result.modified, 0);

        // Line after range but amount_after is 0
        let result = rs_mark_adjust_lnum(10, 3, 7, 2, 0);
        assert_eq!(result.new_lnum, 10);
        assert_eq!(result.modified, 0);
    }

    #[test]
    fn test_mark_adjust_lnum_nodel() {
        // Line in range, MAXLNUM (no delete - set to line1)
        let result = rs_mark_adjust_lnum_nodel(5, 3, 7, MAXLNUM, 0);
        assert_eq!(result.new_lnum, 3);
        assert_ne!(result.modified, 0);
    }

    #[test]
    fn test_mark_adjust_cursor() {
        // Cursor in range, deleted
        let result = rs_mark_adjust_cursor(5, 10, 3, 7, MAXLNUM, 0);
        assert_eq!(result.new_lnum, 2); // max(3-1, 1) = 2
        assert_eq!(result.new_col, 0);
        assert_ne!(result.modified, 0);

        // Edge case: line1 is 1
        let result = rs_mark_adjust_cursor(5, 10, 1, 7, MAXLNUM, 0);
        assert_eq!(result.new_lnum, 1); // max(1-1, 1) = 1
        assert_eq!(result.new_col, 0);
    }

    #[test]
    fn test_mark_col_adjust() {
        // Position on affected line, col >= mincol
        let result = rs_mark_col_adjust(5, 10, 5, 5, 0, 3, 0);
        assert_eq!(result.new_lnum, 5);
        assert_eq!(result.new_col, 13); // 10 + 3
        assert_ne!(result.modified, 0);

        // Position on different line - no change
        let result = rs_mark_col_adjust(4, 10, 5, 5, 0, 3, 0);
        assert_eq!(result.new_lnum, 4);
        assert_eq!(result.new_col, 10);
        assert_eq!(result.modified, 0);

        // Position col < mincol - no change
        let result = rs_mark_col_adjust(5, 3, 5, 5, 0, 3, 0);
        assert_eq!(result.new_lnum, 5);
        assert_eq!(result.new_col, 3);
        assert_eq!(result.modified, 0);

        // Negative col_amount, col would go negative
        let result = rs_mark_col_adjust(5, 3, 5, 0, 0, -5, 0);
        assert_eq!(result.new_col, 0);

        // spaces_removed case
        let result = rs_mark_col_adjust(5, 2, 5, 0, 0, 5, 4);
        assert_eq!(result.new_col, 9); // col_amount + spaces_removed = 5 + 4
    }

    #[test]
    fn test_mark_adjust_should_skip() {
        assert_ne!(rs_mark_adjust_should_skip(5, 3, 0), 0); // line2 < line1, amount_after == 0
        assert_eq!(rs_mark_adjust_should_skip(3, 5, 0), 0); // line2 >= line1
        assert_eq!(rs_mark_adjust_should_skip(5, 3, 1), 0); // amount_after != 0
    }
}
