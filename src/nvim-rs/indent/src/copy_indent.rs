//! Implementation of `copy_indent()` — copies indent from a source line.

use std::ffi::{c_char, c_int};

use nvim_memory::xmalloc;

use crate::{rs_tabstop_padding, TAB};

const SPACE: c_char = b' ' as c_char;

// C accessor functions
extern "C" {
    fn nvim_get_cursor_line_ptr() -> *mut c_char;
    fn nvim_get_cursor_line_len() -> c_int;

    fn nvim_curbuf_get_p_et() -> bool;
    fn nvim_curbuf_get_b_p_ts() -> i64;
    fn nvim_curbuf_get_b_p_vts_array() -> *const c_int;

    fn nvim_ml_replace(lnum: i32, line: *mut c_char, copy: bool) -> c_int;
    fn nvim_set_curwin_cursor_col(col: i32);
    fn nvim_get_curwin_cursor_lnum() -> i32;
}

/// Helper: check if a char is ASCII whitespace (space or tab).
#[inline]
unsafe fn ascii_iswhite(c: c_char) -> bool {
    c == SPACE || c == TAB
}

/// Copy the indent from `src` to the current line (and fill to `size`).
///
/// Leaves the cursor on the first non-blank in the line.
///
/// # Safety
/// - `src` must be a valid null-terminated C string.
/// - Accesses global editor state (current buffer, window, cursor).
#[must_use]
#[export_name = "copy_indent"]
pub unsafe extern "C" fn rs_copy_indent(size: c_int, src: *const c_char) -> bool {
    let b_p_et = nvim_curbuf_get_p_et();
    let b_p_ts = nvim_curbuf_get_b_p_ts();
    let b_p_vts = nvim_curbuf_get_b_p_vts_array();

    let mut p: *mut c_char = std::ptr::null_mut();
    let mut line: *mut c_char = std::ptr::null_mut();
    let mut ind_len: c_int;
    let mut line_len: c_int = 0;
    let mut tab_pad: c_int;

    // Round 1: compute the number of characters needed for the indent
    // Round 2: copy the characters.
    for round in 1..=2 {
        let mut todo = size;
        ind_len = 0;
        let mut ind_done: c_int = 0;
        let mut ind_col: c_int = 0;
        let mut s = src;

        // Count/copy the usable portion of the source line.
        while todo > 0 && ascii_iswhite(*s) {
            if *s == TAB {
                tab_pad = rs_tabstop_padding(ind_done, b_p_ts, b_p_vts);
                if todo < tab_pad {
                    break;
                }
                todo -= tab_pad;
                ind_done += tab_pad;
                ind_col += tab_pad;
            } else {
                todo -= 1;
                ind_done += 1;
                ind_col += 1;
            }
            ind_len += 1;

            if !p.is_null() {
                *p = *s;
                p = p.add(1);
            }
            s = s.add(1);
        }

        // Fill to next tabstop with a tab, if possible.
        tab_pad = rs_tabstop_padding(ind_done, b_p_ts, b_p_vts);
        if todo >= tab_pad && !b_p_et {
            todo -= tab_pad;
            ind_len += 1;
            ind_col += tab_pad;

            if !p.is_null() {
                *p = TAB;
                p = p.add(1);
            }
        }

        // Add tabs required for indent.
        if !b_p_et {
            loop {
                tab_pad = rs_tabstop_padding(ind_col, b_p_ts, b_p_vts);
                if todo < tab_pad {
                    break;
                }
                todo -= tab_pad;
                ind_len += 1;
                ind_col += tab_pad;
                if !p.is_null() {
                    *p = TAB;
                    p = p.add(1);
                }
            }
        }

        // Count/add spaces required for indent.
        while todo > 0 {
            todo -= 1;
            ind_len += 1;

            if !p.is_null() {
                *p = SPACE;
                p = p.add(1);
            }
        }

        if round == 1 {
            // After round 1, allocate memory for the result.
            line_len = nvim_get_cursor_line_len() + 1;
            assert!(ind_len + line_len >= 0);
            let line_size = (ind_len as usize)
                .checked_add(line_len as usize)
                .unwrap_or(0);
            line = xmalloc(line_size).cast();
            p = line;
        }
    }

    // Append the original line
    std::ptr::copy(nvim_get_cursor_line_ptr(), p, line_len as usize);

    // Replace the line
    let curwin_lnum = nvim_get_curwin_cursor_lnum();
    nvim_ml_replace(curwin_lnum, line, false);

    // Put the cursor after the indent.
    // ind_len from last round is the final value we need.
    // We need to recalculate it since we lost it in the loop scope.
    // Actually, p - line gives us the indent length.
    let final_ind_len = p.offset_from(line) as i32;
    nvim_set_curwin_cursor_col(final_ind_len);
    true
}
