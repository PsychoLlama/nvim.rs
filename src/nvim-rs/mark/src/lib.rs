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
