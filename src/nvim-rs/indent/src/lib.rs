//! Indentation utilities for Neovim.
//!
//! This crate provides FFI-compatible indentation calculation functions.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]

use std::ffi::c_char;
use std::ffi::c_int;

// Type aliases matching C types
// colnr_T = int (i32)
// OptInt = int64_t (i64)

const TAB: c_char = b'\t' as c_char;
const SPACE: c_char = b' ' as c_char;

/// Calculate the number of screen spaces a tab will occupy.
///
/// If `vts` is set (non-null and vts[0] > 0) then the tab widths are taken
/// from that array, otherwise the value of `ts` is used.
///
/// The `vts` array format: vts[0] = count, vts[1..count+1] = tabstop widths
///
/// # Safety
/// If `vts` is non-null, it must point to a valid array where vts[0] contains
/// the count and vts[1..count+1] contains valid tabstop values.
#[no_mangle]
pub unsafe extern "C" fn rs_tabstop_padding(col: c_int, ts_arg: i64, vts: *const c_int) -> c_int {
    let ts: i64 = if ts_arg == 0 { 8 } else { ts_arg };

    // If no variable tabstops, use fixed width
    if vts.is_null() || *vts == 0 {
        return (ts - (i64::from(col) % ts)) as c_int;
    }

    let tabcount = *vts;
    let mut tabcol: c_int = 0;
    let mut padding: c_int = 0;
    let mut t: c_int = 1;

    while t <= tabcount {
        tabcol += *vts.offset(t as isize);
        if tabcol > col {
            padding = tabcol - col;
            break;
        }
        t += 1;
    }

    if t > tabcount {
        // Past all defined tabstops, use the last one repeatedly
        let last_ts = *vts.offset(tabcount as isize);
        padding = last_ts - ((col - tabcol) % last_ts);
    }

    padding
}

/// Compute the size of the indent (in window cells) in line `ptr`,
/// using tabstops.
///
/// # Safety
/// - `ptr` must point to a valid null-terminated C string.
/// - If `vts` is non-null, it must point to a valid tabstop array.
#[no_mangle]
pub unsafe extern "C" fn rs_indent_size_ts(
    ptr: *const c_char,
    ts: i64,
    vts: *const c_int,
) -> c_int {
    if ptr.is_null() {
        return 0;
    }

    let mut p = ptr;
    let mut vcol: c_int = 0;
    let tabstop_width: c_int;
    let mut next_tab_vcol: c_int;

    if vts.is_null() || *vts < 1 {
        // Tab has fixed width
        tabstop_width = if ts == 0 { 8 } else { ts as c_int };
        next_tab_vcol = tabstop_width;
    } else {
        // Tab has variable width
        let tabcount = *vts;
        let mut cur_tabstop_idx: c_int = 1;
        let last_tabstop_idx = tabcount;

        // Process variable-width tabstops
        while cur_tabstop_idx != last_tabstop_idx {
            let mut cur_vcol = vcol;
            vcol += *vts.offset(cur_tabstop_idx as isize);
            cur_tabstop_idx += 1;

            loop {
                let c = *p;
                p = p.add(1);
                if c == SPACE {
                    cur_vcol += 1;
                } else if c == TAB {
                    // Tab found, break to next tabstop
                    break;
                } else {
                    // Non-whitespace found
                    return cur_vcol;
                }
                if cur_vcol == vcol {
                    break;
                }
            }
        }

        tabstop_width = *vts.offset(last_tabstop_idx as isize);
        next_tab_vcol = vcol + tabstop_width;
    }

    // Process remaining characters with fixed tabstop width
    loop {
        let c = *p;
        p = p.add(1);
        if c == SPACE {
            vcol += 1;
            if vcol == next_tab_vcol {
                next_tab_vcol += tabstop_width;
            }
        } else if c == TAB {
            vcol = next_tab_vcol;
            next_tab_vcol += tabstop_width;
        } else {
            return vcol;
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_tabstop_padding_fixed() {
        unsafe {
            // With null vts, use fixed tabstops
            // col=0, ts=8 -> padding = 8 - (0 % 8) = 8
            assert_eq!(rs_tabstop_padding(0, 8, std::ptr::null()), 8);
            // col=1, ts=8 -> padding = 8 - (1 % 8) = 7
            assert_eq!(rs_tabstop_padding(1, 8, std::ptr::null()), 7);
            // col=7, ts=8 -> padding = 8 - (7 % 8) = 1
            assert_eq!(rs_tabstop_padding(7, 8, std::ptr::null()), 1);
            // col=8, ts=8 -> padding = 8 - (8 % 8) = 8
            assert_eq!(rs_tabstop_padding(8, 8, std::ptr::null()), 8);
            // col=9, ts=8 -> padding = 8 - (9 % 8) = 7
            assert_eq!(rs_tabstop_padding(9, 8, std::ptr::null()), 7);

            // With ts=4
            assert_eq!(rs_tabstop_padding(0, 4, std::ptr::null()), 4);
            assert_eq!(rs_tabstop_padding(1, 4, std::ptr::null()), 3);
            assert_eq!(rs_tabstop_padding(3, 4, std::ptr::null()), 1);
            assert_eq!(rs_tabstop_padding(4, 4, std::ptr::null()), 4);

            // With ts=0 (should default to 8)
            assert_eq!(rs_tabstop_padding(0, 0, std::ptr::null()), 8);
            assert_eq!(rs_tabstop_padding(4, 0, std::ptr::null()), 4);
        }
    }

    #[test]
    fn test_tabstop_padding_variable() {
        unsafe {
            // Variable tabstops: [count, ts1, ts2, ...]
            // vts = [2, 4, 8] means first tab at 4, second at 4+8=12, then every 8 after
            let vts: [c_int; 3] = [2, 4, 8];

            // col=0 -> next tab at 4, padding = 4
            assert_eq!(rs_tabstop_padding(0, 8, vts.as_ptr()), 4);
            // col=3 -> next tab at 4, padding = 1
            assert_eq!(rs_tabstop_padding(3, 8, vts.as_ptr()), 1);
            // col=4 -> next tab at 12, padding = 8
            assert_eq!(rs_tabstop_padding(4, 8, vts.as_ptr()), 8);
            // col=10 -> next tab at 12, padding = 2
            assert_eq!(rs_tabstop_padding(10, 8, vts.as_ptr()), 2);
            // col=12 -> past all defined, use last (8), next at 20, padding = 8
            assert_eq!(rs_tabstop_padding(12, 8, vts.as_ptr()), 8);
            // col=15 -> past all defined, next at 20, padding = 5
            assert_eq!(rs_tabstop_padding(15, 8, vts.as_ptr()), 5);
        }
    }

    #[test]
    fn test_indent_size_ts_fixed() {
        unsafe {
            // No indent
            let s = CString::new("hello").unwrap();
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, std::ptr::null()), 0);

            // Spaces only
            let s = CString::new("    hello").unwrap();
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, std::ptr::null()), 4);

            // Tab only (at col 0, tab goes to col 8)
            let s = CString::new("\thello").unwrap();
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, std::ptr::null()), 8);

            // Tab with ts=4
            let s = CString::new("\thello").unwrap();
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 4, std::ptr::null()), 4);

            // Spaces then tab
            let s = CString::new("  \thello").unwrap();
            // 2 spaces + tab -> tab takes us to 8
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, std::ptr::null()), 8);

            // Tab then spaces
            let s = CString::new("\t  hello").unwrap();
            // tab to 8, then 2 spaces = 10
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, std::ptr::null()), 10);

            // Multiple tabs
            let s = CString::new("\t\thello").unwrap();
            // tab to 8, tab to 16
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, std::ptr::null()), 16);

            // Empty string
            let s = CString::new("").unwrap();
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, std::ptr::null()), 0);

            // All whitespace
            let s = CString::new("   ").unwrap();
            // Should return 3 when hitting NUL
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, std::ptr::null()), 3);
        }
    }

    #[test]
    fn test_indent_size_ts_variable() {
        unsafe {
            // vts = [2, 4, 8] means first tab at 4, second at 12, then every 8 after
            let vts: [c_int; 3] = [2, 4, 8];

            // Single tab: goes to position 4
            let s = CString::new("\thello").unwrap();
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, vts.as_ptr()), 4);

            // Two tabs: first to 4, second to 12
            let s = CString::new("\t\thello").unwrap();
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, vts.as_ptr()), 12);

            // Three tabs: 4, 12, 20
            let s = CString::new("\t\t\thello").unwrap();
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, vts.as_ptr()), 20);

            // Spaces then tab
            let s = CString::new("  \thello").unwrap();
            // 2 spaces, then tab to 4
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, vts.as_ptr()), 4);

            // 4 spaces then tab
            let s = CString::new("    \thello").unwrap();
            // 4 spaces = at position 4, tab to 12
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, vts.as_ptr()), 12);
        }
    }
}
