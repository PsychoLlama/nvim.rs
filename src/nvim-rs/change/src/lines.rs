//! Line-level editing operations.
//!
//! This module provides functions for line operations like truncation
//! and deletion of multiple lines.

use std::ffi::{c_char, c_int};

use crate::{buf_ref, win_mut, win_ref, BufHandle, ColnrT, LinenrT, WinHandle, FAIL};

// =============================================================================
// C Accessor Functions (extern declarations)
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Window/cursor accessors
    fn nvim_get_curwin() -> WinHandle;

    // Line access
    fn nvim_ml_get(lnum: LinenrT) -> *mut c_char;
    fn nvim_ml_get_len(lnum: LinenrT) -> ColnrT;
    fn nvim_ml_replace(lnum: LinenrT, line: *mut c_char, copy: bool) -> c_int;
    #[link_name = "ml_delete_flags"]
    fn nvim_ml_delete_flags(lnum: LinenrT, flags: c_int) -> c_int;

    fn nvim_get_curbuf() -> BufHandle;

    // Memory allocation
    fn nvim_xmalloc(size: usize) -> *mut c_char;
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;
    fn nvim_xstrnsave(s: *const c_char, len: usize) -> *mut c_char;

    // Undo
    fn u_savedel(lnum: LinenrT, count: LinenrT) -> c_int;

    // Cursor check
    fn check_cursor_lnum(win: WinHandle);

    // Changed notification
    #[link_name = "inserted_bytes"]
    fn rs_inserted_bytes(lnum: LinenrT, start_col: ColnrT, old_col: c_int, new_col: c_int);
    #[link_name = "deleted_lines_mark"]
    fn rs_deleted_lines_mark(lnum: LinenrT, count: c_int);
}

/// ML_EMPTY flag - buffer has no lines.
#[allow(dead_code)]
const ML_EMPTY: c_int = 0x01;

/// ML_DEL_MESSAGE flag - show deletion message.
const ML_DEL_MESSAGE: c_int = 0x04;

// =============================================================================
// Line Operations
// =============================================================================

/// Delete from cursor to end of line.
///
/// Caller must have prepared for undo.
/// If "fixpos" is true, fix the cursor position when done.
fn truncate_line_impl(fixpos: c_int) {
    // SAFETY: All operations are safe FFI calls
    unsafe {
        let curwin = nvim_get_curwin();
        let lnum = win_ref(curwin).w_cursor.lnum;
        let col = win_ref(curwin).w_cursor.col;
        let old_line = nvim_ml_get(lnum);
        let deleted = nvim_ml_get_len(lnum) - col;

        // Create new line - either empty string or truncated copy
        let newp = if col == 0 {
            nvim_xstrdup(c"".as_ptr())
        } else {
            nvim_xstrnsave(old_line, col as usize)
        };

        nvim_ml_replace(lnum, newp, false);

        // mark the buffer as changed and prepare for displaying
        rs_inserted_bytes(lnum, col, deleted, 0);

        // If "fixpos" is true we don't want to end up positioned at the NUL.
        if fixpos != 0 && col > 0 {
            win_mut(curwin).w_cursor.col = col - 1;
        }
    }
}

/// FFI wrapper for `truncate_line`.
///
/// Delete from cursor to end of line.
/// Caller must have prepared for undo.
/// If "fixpos" is true, fix the cursor position when done.
#[export_name = "truncate_line"]
pub extern "C" fn rs_truncate_line(fixpos: c_int) {
    truncate_line_impl(fixpos);
}

/// Delete "nlines" lines at the cursor.
///
/// Saves the lines for undo first if "undo" is true.
fn del_lines_impl(nlines: LinenrT, undo: bool) {
    // SAFETY: All operations are safe FFI calls
    unsafe {
        if nlines <= 0 {
            return;
        }

        let curwin = nvim_get_curwin();
        let curbuf = nvim_get_curbuf();
        let first = win_ref(curwin).w_cursor.lnum;

        // save the deleted lines for undo
        if undo && u_savedel(first, nlines) == FAIL {
            return;
        }

        let mut n = 0;
        while n < nlines {
            if buf_ref(curbuf).ml_is_empty() {
                // nothing to delete
                break;
            }

            nvim_ml_delete_flags(first, ML_DEL_MESSAGE);
            n += 1;

            // If we delete the last line in the file, stop
            if first > buf_ref(curbuf).ml_line_count {
                break;
            }
        }

        // Correct the cursor position before calling deleted_lines_mark(), it may
        // trigger a callback to display the cursor.
        win_mut(curwin).w_cursor.col = 0;
        check_cursor_lnum(curwin);

        // adjust marks, mark the buffer as changed and prepare for displaying
        rs_deleted_lines_mark(first, n as c_int);
    }
}

/// FFI wrapper for `del_lines`.
///
/// Delete "nlines" lines at the cursor.
/// Saves the lines for undo first if "undo" is true.
#[export_name = "del_lines"]
pub extern "C" fn rs_del_lines(nlines: LinenrT, undo: bool) {
    del_lines_impl(nlines, undo);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(ML_EMPTY, 0x01);
        assert_eq!(ML_DEL_MESSAGE, 0x04);
    }
}
