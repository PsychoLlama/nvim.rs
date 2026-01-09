//! Text editing primitives for character and byte insertion/deletion.
//!
//! This module provides functions for inserting and deleting bytes and
//! characters at the cursor position.

use std::ffi::{c_char, c_int};

use crate::{ColnrT, LinenrT, WinHandle, FAIL, OK};

// =============================================================================
// C Accessor Functions (extern declarations)
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Window/cursor accessors
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_win_get_cursor_lnum(win: WinHandle) -> LinenrT;
    fn nvim_win_get_cursor_col(win: WinHandle) -> ColnrT;
    fn nvim_win_set_cursor_col(win: WinHandle, col: ColnrT);
    fn nvim_win_get_cursor_coladd(win: WinHandle) -> ColnrT;
    fn nvim_win_set_cursor_coladd(win: WinHandle, coladd: ColnrT);

    // Line access
    fn nvim_ml_get(lnum: LinenrT) -> *mut c_char;
    fn nvim_ml_get_len(lnum: LinenrT) -> ColnrT;
    fn nvim_ml_replace(lnum: LinenrT, line: *mut c_char, copy: bool) -> c_int;
    fn nvim_ml_line_alloced() -> bool;
    fn nvim_ml_add_deleted_len(ptr: *mut c_char, len: ColnrT);

    // Memory allocation
    fn nvim_xmalloc(size: usize) -> *mut c_char;

    // State checks
    fn nvim_virtual_active(win: WinHandle) -> bool;
    fn nvim_coladvance_force(vcol: ColnrT);
    fn nvim_getviscol() -> ColnrT;
    fn nvim_get_state() -> c_int;
    fn nvim_get_restart_edit() -> c_int;
    fn nvim_get_ve_flags(win: WinHandle) -> c_int;

    // Multi-byte functions
    fn nvim_mb_adjust_cursor();
    fn nvim_get_cursor_pos_ptr() -> *mut c_char;
    fn nvim_utfc_ptr2len(ptr: *const c_char) -> c_int;
    fn nvim_utfc_ptr2len_len(ptr: *const c_char, maxlen: c_int) -> c_int;
    fn nvim_utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    fn nvim_utf_ptr2len(ptr: *const c_char) -> c_int;
    fn nvim_utf_head_off(base: *const c_char, ptr: *const c_char) -> c_int;
    fn nvim_utf_composinglike(p0: *const c_char, p1: *const c_char, state: *mut u64) -> bool;

    // Replace mode functions
    fn nvim_replace_push_nul();
    fn nvim_replace_push(ptr: *const c_char, len: usize);

    // Showmatch
    fn nvim_p_sm() -> bool;
    fn nvim_msg_silent() -> c_int;
    fn nvim_ins_compl_active() -> bool;
    fn nvim_showmatch(c: c_int);
    fn nvim_utf_ptr2char(ptr: *const c_char) -> c_int;

    // Right-to-left
    fn nvim_p_ri() -> bool;

    // VREPLACE mode
    fn nvim_getvcol(
        win: WinHandle,
        pos: *const LinenrT,
        start: *mut ColnrT,
        cursor: *mut ColnrT,
        end: *mut ColnrT,
    );
    fn nvim_win_chartabsize(win: WinHandle, ptr: *const c_char, vcol: ColnrT) -> ColnrT;
    fn nvim_win_get_p_list(win: WinHandle) -> bool;
    fn nvim_win_set_p_list(win: WinHandle, val: bool);
    fn nvim_vim_strchr_cpo_listwm() -> bool;

    // Delcombine option
    fn nvim_p_deco() -> bool;

    // Error message
    fn nvim_siemsg(s: *const c_char, arg: i64);

    // Changed notification
    fn rs_inserted_bytes(lnum: LinenrT, start_col: ColnrT, old_col: c_int, new_col: c_int);

    // Curbuf memline accessor
    fn nvim_curbuf_get_ml_line_len() -> ColnrT;
    fn nvim_curbuf_set_ml_line_len(len: ColnrT);
    fn nvim_curbuf_get_ml_line_ptr() -> *mut c_char;
}

/// State mode flags.
const MODE_INSERT: c_int = 0x10;
const REPLACE_FLAG: c_int = 0x40;
const VREPLACE_FLAG: c_int = 0x80;

/// Maximum bytes for a multibyte character.
const MB_MAXCHAR: usize = 6;

/// TAB character.
const TAB: c_char = 9;

/// NUL character.
const NUL: c_char = 0;

/// Virtual edit onemore flag.
const K_OPT_VE_FLAG_ONEMORE: c_int = 0x04;

/// Grapheme state initial value.
const GRAPHEME_STATE_INIT: u64 = 0;

// =============================================================================
// Insert Functions
// =============================================================================

/// Insert string at the cursor position. Stops at a NUL byte.
///
/// Handles Replace mode and multi-byte characters.
fn ins_bytes_impl(p: *const c_char) {
    // SAFETY: We trust the caller to provide a valid C string
    unsafe {
        let len = libc::strlen(p);
        ins_bytes_len_impl(p, len);
    }
}

/// FFI wrapper for `ins_bytes`.
#[no_mangle]
pub extern "C" fn rs_ins_bytes(p: *const c_char) {
    ins_bytes_impl(p);
}

/// Insert string with length at the cursor position.
///
/// Handles Replace mode and multi-byte characters.
fn ins_bytes_len_impl(p: *const c_char, len: usize) {
    // SAFETY: We trust the caller to provide a valid pointer and length
    unsafe {
        let mut i = 0usize;
        while i < len {
            // avoid reading past p[len]
            let remaining = (len - i) as c_int;
            let n = nvim_utfc_ptr2len_len(p.add(i), remaining) as usize;
            ins_char_bytes_impl(p.add(i) as *mut c_char, n);
            i += n;
        }
    }
}

/// FFI wrapper for `ins_bytes_len`.
#[no_mangle]
pub extern "C" fn rs_ins_bytes_len(p: *const c_char, len: usize) {
    ins_bytes_len_impl(p, len);
}

/// Insert or replace a single character at the cursor position.
///
/// When in REPLACE or VREPLACE state, replace any existing character.
/// Caller must have prepared for undo.
fn ins_char_impl(c: c_int) {
    // SAFETY: All operations are safe FFI calls
    unsafe {
        let mut buf = [0i8; MB_MAXCHAR + 1];
        let n = nvim_utf_char2bytes(c, buf.as_mut_ptr()) as usize;

        // When "c" is 0x100, 0x200, etc. we don't want to insert a NUL byte.
        if buf[0] == 0 {
            buf[0] = b'\n' as i8;
        }
        ins_char_bytes_impl(buf.as_ptr() as *mut c_char, n);
    }
}

/// FFI wrapper for `ins_char`.
#[no_mangle]
pub extern "C" fn rs_ins_char(c: c_int) {
    ins_char_impl(c);
}

/// Insert or replace character bytes at the cursor position.
fn ins_char_bytes_impl(buf: *mut c_char, charlen: usize) {
    // SAFETY: All operations are safe FFI calls
    unsafe {
        let curwin = nvim_get_curwin();

        // Break tabs if needed.
        if nvim_virtual_active(curwin) && nvim_win_get_cursor_coladd(curwin) > 0 {
            nvim_coladvance_force(nvim_getviscol());
        }

        let col = nvim_win_get_cursor_col(curwin) as usize;
        let lnum = nvim_win_get_cursor_lnum(curwin);
        let oldp = nvim_ml_get(lnum);
        let linelen = nvim_ml_get_len(lnum) as usize + 1; // length including NUL

        // The lengths default to the values for when not replacing.
        let mut oldlen = 0usize;
        let mut newlen = charlen;

        let state = nvim_get_state();
        if (state & REPLACE_FLAG) != 0 {
            if (state & VREPLACE_FLAG) != 0 {
                // VREPLACE mode - complex handling
                let old_list = nvim_win_get_p_list(curwin);
                if old_list && !nvim_vim_strchr_cpo_listwm() {
                    nvim_win_set_p_list(curwin, false);
                }

                let mut vcol: ColnrT = 0;
                nvim_getvcol(
                    curwin,
                    std::ptr::null(),
                    std::ptr::null_mut(),
                    &mut vcol,
                    std::ptr::null_mut(),
                );
                let new_vcol = vcol + nvim_win_chartabsize(curwin, buf, vcol);

                while *oldp.add(col + oldlen) != NUL && vcol < new_vcol {
                    vcol += nvim_win_chartabsize(curwin, oldp.add(col + oldlen), vcol);
                    if vcol > new_vcol && *oldp.add(col + oldlen) == TAB {
                        break;
                    }
                    oldlen += nvim_utfc_ptr2len(oldp.add(col + oldlen)) as usize;
                    if vcol > new_vcol {
                        newlen += (vcol - new_vcol) as usize;
                    }
                }
                nvim_win_set_p_list(curwin, old_list);
            } else if *oldp.add(col) != NUL {
                // normal replace
                oldlen = nvim_utfc_ptr2len(oldp.add(col)) as usize;
            }

            // Push replaced bytes onto the replace stack
            nvim_replace_push_nul();
            nvim_replace_push(oldp.add(col), oldlen);
        }

        let newp = nvim_xmalloc(linelen + newlen - oldlen);

        // Copy bytes before the cursor.
        if col > 0 {
            std::ptr::copy_nonoverlapping(oldp, newp, col);
        }

        // Copy bytes after the changed character(s).
        let p = newp.add(col);
        if linelen > col + oldlen {
            std::ptr::copy_nonoverlapping(oldp.add(col + oldlen), p.add(newlen), linelen - col - oldlen);
        }

        // Insert or overwrite the new character.
        std::ptr::copy_nonoverlapping(buf, p, charlen);

        // Fill with spaces when necessary.
        for i in charlen..newlen {
            *p.add(i) = b' ' as c_char;
        }

        // Replace the line in the buffer.
        nvim_ml_replace(lnum, newp, false);

        // Mark the buffer as changed
        rs_inserted_bytes(lnum, col as ColnrT, oldlen as c_int, newlen as c_int);

        // Showmatch for parens/braces
        if nvim_p_sm() && (state & MODE_INSERT) != 0 && nvim_msg_silent() == 0 && !nvim_ins_compl_active()
        {
            nvim_showmatch(nvim_utf_ptr2char(buf));
        }

        if !nvim_p_ri() || (state & REPLACE_FLAG) != 0 {
            // Normal insert: move cursor right
            nvim_win_set_cursor_col(curwin, nvim_win_get_cursor_col(curwin) + charlen as ColnrT);
        }
    }
}

/// FFI wrapper for `ins_char_bytes`.
#[no_mangle]
pub extern "C" fn rs_ins_char_bytes(buf: *mut c_char, charlen: usize) {
    ins_char_bytes_impl(buf, charlen);
}

/// Insert a string at the cursor position.
///
/// Note: Does NOT handle Replace mode.
/// Caller must have prepared for undo.
fn ins_str_impl(s: *const c_char, slen: usize) {
    // SAFETY: All operations are safe FFI calls
    unsafe {
        let curwin = nvim_get_curwin();
        let lnum = nvim_win_get_cursor_lnum(curwin);

        if nvim_virtual_active(curwin) && nvim_win_get_cursor_coladd(curwin) > 0 {
            nvim_coladvance_force(nvim_getviscol());
        }

        let col = nvim_win_get_cursor_col(curwin) as usize;
        let oldp = nvim_ml_get(lnum);
        let oldlen = nvim_ml_get_len(lnum) as usize;

        let newp = nvim_xmalloc(oldlen + slen + 1);
        if col > 0 {
            std::ptr::copy_nonoverlapping(oldp, newp, col);
        }
        std::ptr::copy_nonoverlapping(s, newp.add(col), slen);
        let bytes = oldlen - col + 1;
        std::ptr::copy_nonoverlapping(oldp.add(col), newp.add(col + slen), bytes);

        nvim_ml_replace(lnum, newp, false);
        rs_inserted_bytes(lnum, col as ColnrT, 0, slen as c_int);
        nvim_win_set_cursor_col(curwin, nvim_win_get_cursor_col(curwin) + slen as ColnrT);
    }
}

/// FFI wrapper for `ins_str`.
#[no_mangle]
pub extern "C" fn rs_ins_str(s: *const c_char, slen: usize) {
    ins_str_impl(s, slen);
}

// =============================================================================
// Delete Functions
// =============================================================================

/// Delete one character under the cursor.
///
/// If "fixpos" is true, don't leave the cursor on the NUL after the line.
/// Caller must have prepared for undo.
///
/// Returns FAIL for failure, OK otherwise.
fn del_char_impl(fixpos: bool) -> c_int {
    // SAFETY: All operations are safe FFI calls
    unsafe {
        // Make sure the cursor is at the start of a character.
        nvim_mb_adjust_cursor();
        if *nvim_get_cursor_pos_ptr() == NUL {
            return FAIL;
        }
        del_chars_impl(1, fixpos as c_int)
    }
}

/// FFI wrapper for `del_char`.
#[no_mangle]
pub extern "C" fn rs_del_char(fixpos: bool) -> c_int {
    del_char_impl(fixpos)
}

/// Like del_bytes(), but delete characters instead of bytes.
fn del_chars_impl(count: c_int, fixpos: c_int) -> c_int {
    // SAFETY: All operations are safe FFI calls
    unsafe {
        let mut bytes = 0;
        let mut p = nvim_get_cursor_pos_ptr();
        for _ in 0..count {
            if *p == NUL {
                break;
            }
            let l = nvim_utfc_ptr2len(p);
            bytes += l;
            p = p.add(l as usize);
        }
        del_bytes_impl(bytes as ColnrT, fixpos != 0, true)
    }
}

/// FFI wrapper for `del_chars`.
#[no_mangle]
pub extern "C" fn rs_del_chars(count: c_int, fixpos: c_int) -> c_int {
    del_chars_impl(count, fixpos)
}

/// Delete "count" bytes under the cursor.
///
/// If "fixpos" is true, don't leave the cursor on the NUL after the line.
/// Caller must have prepared for undo.
///
/// Returns FAIL for failure, OK otherwise.
fn del_bytes_impl(count: ColnrT, fixpos_arg: bool, use_delcombine: bool) -> c_int {
    // SAFETY: All operations are safe FFI calls
    unsafe {
        let curwin = nvim_get_curwin();
        let lnum = nvim_win_get_cursor_lnum(curwin);
        let mut col = nvim_win_get_cursor_col(curwin);
        let mut fixpos = fixpos_arg;
        let oldp = nvim_ml_get(lnum);
        let oldlen = nvim_ml_get_len(lnum);
        let mut count = count;

        // Can't do anything when the cursor is on the NUL after the line.
        if col >= oldlen {
            return FAIL;
        }
        // If "count" is zero there is nothing to do.
        if count == 0 {
            return OK;
        }
        // If "count" is negative the caller must be doing something wrong.
        if count < 1 {
            nvim_siemsg(
                c"E292: Invalid count for del_bytes(): %ld".as_ptr(),
                count as i64,
            );
            return FAIL;
        }

        // If 'delcombine' is set and deleting (less than) one character, only
        // delete the last combining character.
        if nvim_p_deco() && use_delcombine && nvim_utfc_ptr2len(oldp.add(col as usize)) >= count {
            let p0 = oldp.add(col as usize);
            let mut state: u64 = GRAPHEME_STATE_INIT;
            if nvim_utf_composinglike(p0, p0.add(nvim_utf_ptr2len(p0) as usize), &mut state) {
                // Find the last composing char, there can be several.
                let mut n = col;
                loop {
                    col = n;
                    count = nvim_utf_ptr2len(oldp.add(n as usize));
                    n += count;
                    if !nvim_utf_composinglike(
                        oldp.add(col as usize),
                        oldp.add(n as usize),
                        &mut state,
                    ) {
                        break;
                    }
                }
                fixpos = false;
            }
        }

        // When count is too big, reduce it.
        let mut movelen = oldlen - col - count + 1; // includes trailing NUL
        if movelen <= 1 {
            // If we just took off the last character of a non-blank line, and
            // fixpos is true, we don't want to end up positioned at the NUL.
            if col > 0
                && fixpos
                && nvim_get_restart_edit() == 0
                && (nvim_get_ve_flags(curwin) & K_OPT_VE_FLAG_ONEMORE) == 0
            {
                let mut cur_col = nvim_win_get_cursor_col(curwin) - 1;
                nvim_win_set_cursor_coladd(curwin, 0);
                cur_col -= nvim_utf_head_off(oldp, oldp.add(cur_col as usize));
                nvim_win_set_cursor_col(curwin, cur_col);
            }
            count = oldlen - col;
            movelen = 1;
        }
        let newlen = oldlen - count;

        // Check if the old line was allocated
        let alloc_newp = !nvim_ml_line_alloced();
        let newp;
        if !alloc_newp {
            nvim_ml_add_deleted_len(nvim_curbuf_get_ml_line_ptr(), oldlen);
            newp = oldp; // use same allocated memory
        } else {
            newp = nvim_xmalloc((newlen + 1) as usize);
            std::ptr::copy_nonoverlapping(oldp, newp, col as usize);
        }
        std::ptr::copy_nonoverlapping(
            oldp.add((col + count) as usize),
            newp.add(col as usize),
            movelen as usize,
        );
        if alloc_newp {
            nvim_ml_replace(lnum, newp, false);
        } else {
            nvim_curbuf_set_ml_line_len(nvim_curbuf_get_ml_line_len() - count);
        }

        // mark the buffer as changed and prepare for displaying
        rs_inserted_bytes(lnum, col, count, 0);

        OK
    }
}

/// FFI wrapper for `del_bytes`.
#[no_mangle]
pub extern "C" fn rs_del_bytes(count: ColnrT, fixpos_arg: bool, use_delcombine: bool) -> c_int {
    del_bytes_impl(count, fixpos_arg, use_delcombine)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(MODE_INSERT, 0x10);
        assert_eq!(REPLACE_FLAG, 0x40);
        assert_eq!(VREPLACE_FLAG, 0x80);
        assert_eq!(MB_MAXCHAR, 6);
    }
}
