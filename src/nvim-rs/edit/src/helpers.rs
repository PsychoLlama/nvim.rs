//! Self-contained helper functions migrated from edit.c
//!
//! These are small, mostly-independent functions used in insert mode:
//! dollar display, trailing space truncation, backspace helpers,
//! abbreviation checks, and last-insert text management.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

/// Column number type (matches `colnr_T` in Neovim).
type ColnrT = i32;

/// Line number type (matches `linenr_T` in Neovim).
type LinenrT = i32;

/// Opaque handle to a window (`win_T *`).
type WinHandle = *mut c_void;

// ============================================================================
// C accessor functions
// ============================================================================

extern "C" {
    // Dollar display
    fn nvim_get_dollar_vcol() -> ColnrT;
    fn nvim_set_dollar_vcol(val: ColnrT);
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_curwin_get_cursor_lnum() -> LinenrT;
    fn redrawWinline(wp: WinHandle, lnum: LinenrT);

    // Cursor position
    fn nvim_curwin_get_cursor_col() -> ColnrT;
    fn nvim_curwin_set_cursor_col(col: ColnrT);

    // State
    fn nvim_get_State() -> c_int;

    // Replace stack (still in C for Phase 1, migrated in Phase 2)
    fn replace_join(off: c_int);
    fn nvim_replace_do_bs(limit_col: c_int);

    // Character operations
    fn mb_adjust_cursor();
    fn nvim_get_cursor_pos_ptr() -> *const c_char;
    fn utf_ptr2len(p: *const c_char) -> c_int;
    fn del_bytes(count: ColnrT, fixpos: c_int, use_delcombine: c_int) -> c_int;
    fn del_char(fixpos: c_int) -> c_int;

    // Abbreviation check
    fn nvim_get_p_paste() -> c_int;
    fn nvim_get_no_abbr() -> c_int;
    fn nvim_get_arrow_used() -> c_int;
    fn check_abbr(c: c_int, ptr: *mut c_char, col: c_int, mincol: c_int) -> c_int;
    fn get_cursor_line_ptr() -> *mut c_char;
    fn nvim_get_Insstart_lnum() -> LinenrT;
    fn nvim_get_Insstart_col() -> ColnrT;

    // `get_nolist_virtcol` dependencies
    fn nvim_curwin_buf_valid() -> c_int;
    fn nvim_curwin_buf_line_count() -> LinenrT;
    fn nvim_curwin_w_p_list() -> c_int;
    fn nvim_p_cpo_has_listwm() -> c_int;
    fn nvim_getvcol_nolist() -> ColnrT;
    fn nvim_validate_virtcol_curwin();
    fn nvim_curwin_get_w_virtcol() -> ColnrT;

    // Last-insert management
    fn nvim_get_last_insert_data() -> *mut c_char;
    fn nvim_get_last_insert_size() -> usize;
    fn nvim_set_last_insert(data: *mut c_char, size: usize);
    fn nvim_clear_last_insert();
    fn nvim_get_last_insert_skip() -> c_int;
    fn nvim_set_last_insert_skip(val: c_int);
    fn xfree(ptr: *mut c_void);
    fn xmalloc(size: usize) -> *mut c_void;
    fn xmemdupz(data: *const c_void, len: usize) -> *mut c_char;
    fn add_char2buf(c: c_int, s: *mut c_char) -> *mut c_char;
}

// ============================================================================
// Constants (verified against C headers with `_Static_assert` in `edit.c`)
// ============================================================================

/// `REPLACE_FLAG` from `state_defs.h`
const REPLACE_FLAG: c_int = 0x100;

/// NUL byte
const NUL: c_char = 0;

/// `Ctrl_V` from `ascii_defs.h`
const CTRL_V: c_char = 22;

/// ESC from `ascii_defs.h`
const ESC: c_char = 0x1b; // '\033'

/// `DEL` from `ascii_defs.h`
const DEL: c_int = 0x7f;

/// `MB_MAXBYTES` from `mbyte_defs.h`
const MB_MAXBYTES: usize = 21;

// ============================================================================
// undisplay_dollar
// ============================================================================

/// Reset the dollar display marker and redraw the current line.
///
/// Called before moving the cursor from the normal insert position
/// in insert mode. If `dollar_vcol` is already -1 (no dollar displayed),
/// this is a no-op.
unsafe fn undisplay_dollar_impl() {
    if nvim_get_dollar_vcol() < 0 {
        return;
    }
    nvim_set_dollar_vcol(-1);
    let curwin = nvim_get_curwin();
    let lnum = nvim_curwin_get_cursor_lnum();
    redrawWinline(curwin, lnum);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_undisplay_dollar() {
    undisplay_dollar_impl();
}

// ============================================================================
// get_nolist_virtcol
// ============================================================================

/// Get the value that `w_virtcol` would have when 'list' is off,
/// unless 'cpo' contains the 'L' flag.
///
/// Returns 0 if the current buffer is invalid.
unsafe fn get_nolist_virtcol_impl() -> ColnrT {
    // Check validity of cursor in current buffer
    if nvim_curwin_buf_valid() == 0 || nvim_curwin_get_cursor_lnum() > nvim_curwin_buf_line_count()
    {
        return 0;
    }
    if nvim_curwin_w_p_list() != 0 && nvim_p_cpo_has_listwm() == 0 {
        return nvim_getvcol_nolist();
    }
    nvim_validate_virtcol_curwin();
    nvim_curwin_get_w_virtcol()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_get_nolist_virtcol() -> ColnrT {
    get_nolist_virtcol_impl()
}

// ============================================================================
// echeck_abbr
// ============================================================================

/// Check for abbreviation before the cursor.
///
/// Returns false in paste mode, when abbreviations are disabled,
/// or right after using cursor keys.
unsafe fn echeck_abbr_impl(c: c_int) -> bool {
    if nvim_get_p_paste() != 0 || nvim_get_no_abbr() != 0 || nvim_get_arrow_used() != 0 {
        return false;
    }

    let cursor_col = nvim_curwin_get_cursor_col();
    let cursor_lnum = nvim_curwin_get_cursor_lnum();
    let insstart_lnum = nvim_get_Insstart_lnum();
    let insstart_col = nvim_get_Insstart_col();
    let mincol = if cursor_lnum == insstart_lnum {
        insstart_col
    } else {
        0
    };

    check_abbr(c, get_cursor_line_ptr(), cursor_col, mincol) != 0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_echeck_abbr(c: c_int) -> c_int {
    c_int::from(echeck_abbr_impl(c))
}

// ============================================================================
// truncate_spaces
// ============================================================================

/// Truncate trailing whitespace from a line.
///
/// In replace mode, also removes NUL separators from the replace stack
/// for each removed space.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_truncate_spaces(line: *mut c_char, len: usize) {
    if line.is_null() {
        return;
    }

    let state = nvim_get_State();
    let mut i = len as isize - 1;

    // Find start of trailing whitespace
    while i >= 0 {
        let ch = *line.offset(i) as u8;
        if ch != b' ' && ch != b'\t' {
            break;
        }
        if state & REPLACE_FLAG != 0 {
            replace_join(0);
        }
        i -= 1;
    }
    *line.offset(i + 1) = NUL;
}

// ============================================================================
// del_char_after_col
// ============================================================================

/// Delete character after column, respecting composing characters.
///
/// Like `del_char()`, but ensures we don't go before `limit_col`.
/// Returns true when something was deleted.
unsafe fn del_char_after_col_impl(limit_col: c_int) -> bool {
    if limit_col >= 0 {
        let ecol: ColnrT = nvim_curwin_get_cursor_col() + 1;

        // Make sure the cursor is at the start of a character, but
        // skip forward again when going too far back because of a
        // composing character.
        mb_adjust_cursor();
        while nvim_curwin_get_cursor_col() < limit_col as ColnrT {
            let p = nvim_get_cursor_pos_ptr();
            let l = utf_ptr2len(p);
            if l == 0 {
                break;
            }
            nvim_curwin_set_cursor_col(nvim_curwin_get_cursor_col() + l as ColnrT);
        }
        let p = nvim_get_cursor_pos_ptr();
        if *p == NUL || nvim_curwin_get_cursor_col() == ecol {
            return false;
        }
        del_bytes(ecol - nvim_curwin_get_cursor_col(), 0, 1);
    } else {
        del_char(0);
    }
    true
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_del_char_after_col(limit_col: c_int) -> c_int {
    c_int::from(del_char_after_col_impl(limit_col))
}

// ============================================================================
// backspace_until_column
// ============================================================================

/// Backspace the cursor until the given column.
///
/// Handles replace mode correctly by using `replace_do_bs`.
/// In normal insert mode, uses `del_char_after_col` to avoid
/// deleting before composing characters.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_backspace_until_column(col: c_int) {
    while nvim_curwin_get_cursor_col() as c_int > col {
        nvim_curwin_set_cursor_col(nvim_curwin_get_cursor_col() - 1);
        if nvim_get_State() & REPLACE_FLAG != 0 {
            nvim_replace_do_bs(col);
        } else if !del_char_after_col_impl(col) {
            break;
        }
    }
}

// ============================================================================
// set_last_insert
// ============================================================================

/// Set the last inserted text to a single character.
///
/// Used for the replace command. Allocates a buffer, optionally
/// prefixes with CTRL-V for special characters, and appends ESC.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_set_last_insert(c: c_int) {
    // Free existing last_insert data
    let old_data = nvim_get_last_insert_data();
    if !old_data.is_null() {
        xfree(old_data.cast());
    }

    // Allocate new buffer: MB_MAXBYTES * 3 + 5 (same as C)
    let buf_size = MB_MAXBYTES * 3 + 5;
    let new_data: *mut c_char = xmalloc(buf_size).cast();
    let mut s = new_data;

    // Use CTRL-V only when entering a special char
    if c < c_int::from(b' ') || c == DEL {
        *s = CTRL_V;
        s = s.add(1);
    }
    s = add_char2buf(c, s);
    *s = ESC;
    s = s.add(1);
    *s = NUL;

    let size = s.offset_from(new_data) as usize;
    nvim_set_last_insert(new_data, size);
    nvim_set_last_insert_skip(0);
}

// ============================================================================
// free_last_insert
// ============================================================================

/// Free the last-insert text (used at exit).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_free_last_insert() {
    nvim_clear_last_insert();
}

// ============================================================================
// get_last_insert
// ============================================================================

/// FFI-compatible String type matching Neovim's `String` (`{char *data; size_t size}`).
#[repr(C)]
pub struct NvimString {
    pub data: *mut c_char,
    pub size: usize,
}

/// Get the last inserted text, skipping the initial `last_insert_skip` bytes.
///
/// Returns a string with `data = NULL` if no last insert exists.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_get_last_insert() -> NvimString {
    let data = nvim_get_last_insert_data();
    if data.is_null() {
        return NvimString {
            data: ptr::null_mut(),
            size: 0,
        };
    }
    let size = nvim_get_last_insert_size();
    let skip = nvim_get_last_insert_skip() as usize;
    NvimString {
        data: data.add(skip),
        size: size - skip,
    }
}

// ============================================================================
// get_last_insert_save
// ============================================================================

/// Get last inserted string with trailing ESC removed.
///
/// Returns a newly allocated copy (caller must free), or NULL if none.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_get_last_insert_save() -> *mut c_char {
    let insert = rs_get_last_insert();
    if insert.data.is_null() {
        return ptr::null_mut();
    }

    let s = xmemdupz(insert.data.cast(), insert.size);
    if insert.size > 0 && *s.add(insert.size - 1) == ESC {
        let new_size = insert.size - 1;
        *s.add(new_size) = NUL;
    }
    s
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(REPLACE_FLAG, 0x100);
        assert_eq!(CTRL_V as u8, 22);
        assert_eq!(ESC as u8, 0x1b);
        assert_eq!(DEL, 0x7f);
        assert_eq!(MB_MAXBYTES, 21);
    }

    #[test]
    fn test_nvim_string_layout() {
        // NvimString must match C's String layout: pointer + size
        assert_eq!(
            std::mem::size_of::<NvimString>(),
            std::mem::size_of::<*mut c_char>() + std::mem::size_of::<usize>()
        );
    }

    #[test]
    fn test_null_nvim_string() {
        let s = NvimString {
            data: ptr::null_mut(),
            size: 0,
        };
        assert!(s.data.is_null());
        assert_eq!(s.size, 0);
    }
}
