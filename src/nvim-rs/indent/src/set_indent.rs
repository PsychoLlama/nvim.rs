//! Implementation of `set_indent()` — sets the indent of the current line.

use std::ffi::{c_char, c_int};

use nvim_buffer::BufHandle;
use nvim_memory::{xfree, xmalloc};

use crate::{rs_tabstop_padding, TAB};

// Constants matching C definitions (verified via _Static_assert in indent_ffi.c)
pub const SIN_CHANGED: c_int = 1;
const SIN_INSERT: c_int = 2;
const SIN_UNDO: c_int = 4;
const SIN_NOMARK: c_int = 8;

const SPACE: c_char = b' ' as c_char;
// C accessor functions
extern "C" {
    fn nvim_get_cursor_line_ptr() -> *mut c_char;
    fn nvim_get_cursor_line_len() -> c_int;

    fn nvim_curbuf_get_p_et() -> bool;
    fn nvim_curbuf_get_b_p_pi() -> bool;
    fn nvim_curbuf_get_b_p_ts() -> i64;
    fn nvim_curbuf_get_b_p_vts_array() -> *const c_int;

    fn skipwhite(s: *const c_char) -> *mut c_char;

    fn nvim_u_savesub_curline() -> c_int;

    fn nvim_ml_replace(lnum: i32, line: *mut c_char, copy: bool) -> c_int;
    fn nvim_extmark_splice_cols(
        buf: BufHandle,
        start_row: c_int,
        start_col: i32,
        old_col: i32,
        new_col: i32,
        undo: c_int,
    );
    fn changed_bytes(lnum: i32, col: i32);

    fn nvim_get_saved_cursor_lnum() -> i32;
    fn nvim_get_saved_cursor_col() -> i32;
    fn nvim_set_saved_cursor_col(val: i32);

    fn nvim_set_curwin_cursor_col(col: i32);

    fn nvim_get_curbuf() -> BufHandle;

    // Get curwin->w_cursor.lnum
    fn nvim_get_curwin_cursor_lnum() -> i32;
}

/// Helper: check if a char is ASCII whitespace (space or tab).
#[inline]
unsafe fn ascii_iswhite(c: c_char) -> bool {
    c == SPACE || c == TAB
}

/// Set the indent of the current line.
///
/// Leaves the cursor on the first non-blank in the line.
/// Caller must take care of undo.
///
/// # Flags
/// - `SIN_CHANGED`: call `changed_bytes()` if the line was changed
/// - `SIN_INSERT`: insert the indent in front of the line
/// - `SIN_UNDO`: save line for undo before changing it
/// - `SIN_NOMARK`: don't move extmarks
///
/// # Safety
/// Accesses global editor state (current buffer, window, cursor).
#[must_use]
#[export_name = "set_indent"]
pub unsafe extern "C" fn rs_set_indent(size: c_int, flags: c_int) -> bool {
    let mut doit = false;
    let mut ind_done: c_int = 0;
    let mut tab_pad: c_int;
    let mut retval = false;

    // Number of initial whitespace chars when 'et' and 'pi' are both set.
    let mut orig_char_len: c_int = -1;

    // First check if there is anything to do and compute the number of
    // characters needed for the indent.
    let mut todo = size;
    let mut ind_len: c_int = 0; // Measured in characters
    let oldline = nvim_get_cursor_line_ptr();
    let mut p = oldline;
    let mut line_len = nvim_get_cursor_line_len() + 1; // includes NUL

    let b_p_et = nvim_curbuf_get_p_et();
    let b_p_pi = nvim_curbuf_get_b_p_pi();
    let b_p_ts = nvim_curbuf_get_b_p_ts();
    let b_p_vts = nvim_curbuf_get_b_p_vts_array();

    // Calculate the buffer size for the new indent, and check to see if it
    // isn't already set.
    if !b_p_et || (flags & SIN_INSERT == 0 && b_p_pi) {
        let mut ind_col: c_int = 0;

        // If 'preserveindent' is set then reuse as much as possible of
        // the existing indent structure for the new indent.
        if flags & SIN_INSERT == 0 && b_p_pi {
            ind_done = 0;

            // Count as many characters as we can use.
            while todo > 0 && ascii_iswhite(*p) {
                if *p == TAB {
                    tab_pad = rs_tabstop_padding(ind_done, b_p_ts, b_p_vts);
                    if todo < tab_pad {
                        break;
                    }
                    todo -= tab_pad;
                    ind_len += 1;
                    ind_done += tab_pad;
                } else {
                    todo -= 1;
                    ind_len += 1;
                    ind_done += 1;
                }
                p = p.add(1);
            }

            ind_col = ind_done;

            // Set initial number of whitespace chars to copy if we are
            // preserving indent but expandtab is set.
            if b_p_et {
                orig_char_len = ind_len;
            }

            // Fill to next tabstop with a tab, if possible.
            tab_pad = rs_tabstop_padding(ind_done, b_p_ts, b_p_vts);
            if todo >= tab_pad && orig_char_len == -1 {
                doit = true;
                todo -= tab_pad;
                ind_len += 1;
                ind_col += tab_pad;
            }
        }

        // Count tabs required for indent.
        loop {
            tab_pad = rs_tabstop_padding(ind_col, b_p_ts, b_p_vts);
            if todo < tab_pad {
                break;
            }
            if *p != TAB {
                doit = true;
            } else {
                p = p.add(1);
            }
            todo -= tab_pad;
            ind_len += 1;
            ind_col += tab_pad;
        }
    }

    // Count spaces required for indent.
    while todo > 0 {
        if *p != SPACE {
            doit = true;
        } else {
            p = p.add(1);
        }
        todo -= 1;
        ind_len += 1;
    }

    // Return if the indent is OK already.
    if !doit && !ascii_iswhite(*p) && (flags & SIN_INSERT == 0) {
        return false;
    }

    // Allocate memory for the new line.
    if flags & SIN_INSERT != 0 {
        p = oldline;
    } else {
        p = skipwhite(p);
        line_len -= p.offset_from(oldline) as c_int;
    }

    // If 'preserveindent' and 'expandtab' are both set keep the original
    // characters and allocate accordingly.
    let mut skipcols: c_int = 0;
    let newline: *mut c_char;
    let mut s: *mut c_char;

    if orig_char_len != -1 {
        // newline_size = orig_char_len + size - ind_done + line_len
        let newline_size = orig_char_len
            .checked_add(size)
            .and_then(|v| v.checked_sub(ind_done))
            .and_then(|v| v.checked_add(line_len))
            .unwrap_or(0);
        assert!(newline_size >= 0);
        newline = xmalloc(newline_size as usize).cast();
        todo = size - ind_done;

        // Set total length of indent in characters.
        ind_len = orig_char_len + todo;
        p = oldline;
        s = newline;
        skipcols = orig_char_len;

        let mut remaining = orig_char_len;
        while remaining > 0 {
            *s = *p;
            s = s.add(1);
            p = p.add(1);
            remaining -= 1;
        }

        // Skip over any additional white space.
        while ascii_iswhite(*p) {
            p = p.add(1);
        }
    } else {
        todo = size;
        assert!(ind_len + line_len >= 0);
        let newline_size = (ind_len as usize)
            .checked_add(line_len as usize)
            .unwrap_or(0);
        newline = xmalloc(newline_size).cast();
        s = newline;
    }

    // Put the characters in the new line.
    // if 'expandtab' isn't set: use TABs
    if !b_p_et {
        // If 'preserveindent' is set then reuse as much as possible of
        // the existing indent structure for the new indent.
        if flags & SIN_INSERT == 0 && b_p_pi {
            p = oldline;
            ind_done = 0;

            while todo > 0 && ascii_iswhite(*p) {
                if *p == TAB {
                    tab_pad = rs_tabstop_padding(ind_done, b_p_ts, b_p_vts);
                    if todo < tab_pad {
                        break;
                    }
                    todo -= tab_pad;
                    ind_done += tab_pad;
                } else {
                    todo -= 1;
                    ind_done += 1;
                }
                *s = *p;
                s = s.add(1);
                p = p.add(1);
                skipcols += 1;
            }

            // Fill to next tabstop with a tab, if possible.
            tab_pad = rs_tabstop_padding(ind_done, b_p_ts, b_p_vts);
            if todo >= tab_pad {
                *s = TAB;
                s = s.add(1);
                todo -= tab_pad;
                ind_done += tab_pad;
            }
            p = skipwhite(p);
        }

        loop {
            tab_pad = rs_tabstop_padding(ind_done, b_p_ts, b_p_vts);
            if todo < tab_pad {
                break;
            }
            *s = TAB;
            s = s.add(1);
            todo -= tab_pad;
            ind_done += tab_pad;
        }
    }

    while todo > 0 {
        *s = SPACE;
        s = s.add(1);
        todo -= 1;
    }

    std::ptr::copy(p, s, line_len as usize);

    // Replace the line (unless undo fails).
    let curwin_lnum = nvim_get_curwin_cursor_lnum();
    if flags & SIN_UNDO == 0 || nvim_u_savesub_curline() == 1 {
        let old_offset = p.offset_from(oldline) as i32;
        let new_offset = s.offset_from(newline) as i32;

        // this may free "newline"
        nvim_ml_replace(curwin_lnum, newline, false);

        if flags & SIN_NOMARK == 0 {
            let buf = nvim_get_curbuf();
            nvim_extmark_splice_cols(
                buf,
                curwin_lnum - 1,
                skipcols,
                old_offset - skipcols,
                new_offset - skipcols,
                1, // kExtmarkUndo
            );
        }

        if flags & SIN_CHANGED != 0 {
            changed_bytes(curwin_lnum, 0);
        }

        // Correct saved cursor position if it is in this line.
        if nvim_get_saved_cursor_lnum() == curwin_lnum {
            let saved_col = nvim_get_saved_cursor_col();
            if saved_col >= old_offset {
                nvim_set_saved_cursor_col(saved_col + ind_len - old_offset);
            } else if saved_col >= new_offset {
                nvim_set_saved_cursor_col(new_offset);
            }
        }
        retval = true;
    } else {
        xfree(newline.cast());
    }

    nvim_set_curwin_cursor_col(ind_len);
    retval
}
