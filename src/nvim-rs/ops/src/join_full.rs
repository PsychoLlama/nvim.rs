//! Full `do_join` migration (Phase 2)
//!
//! Migrated from `do_join()` in ops.c.
//! Handles J (join lines) and gJ (join without spaces).

#![allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::missing_panics_doc,
    clippy::too_many_lines
)]

use std::ffi::{c_char, c_int, c_void};

// -----------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------

const OK: c_int = 1;
const FAIL: c_int = 0;
const NUL: c_int = 0;
const TAB: c_int = b'\t' as c_int;
/// FO_REMOVE_COMS = 'j'
const FO_REMOVE_COMS: c_int = b'j' as c_int;
/// FO_MBYTE_JOIN = 'M'
const FO_MBYTE_JOIN: c_int = b'M' as c_int;
/// FO_MBYTE_JOIN2 = 'B'
const FO_MBYTE_JOIN2: c_int = b'B' as c_int;
/// CPO_JOINCOL = 'q'
const CPO_JOINCOL: c_int = b'q' as c_int;
/// kExtmarkUndo = 1
const K_EXTMARK_UNDO: c_int = 1;

// -----------------------------------------------------------------------
// FFI declarations
// -----------------------------------------------------------------------

extern "C" {
    // Undo
    fn u_save(top: c_int, bot: c_int) -> c_int;

    // Line content
    fn ml_get(lnum: c_int) -> *mut c_char;
    fn ml_replace_len(lnum: c_int, line: *mut c_char, len: usize, copy: bool) -> c_int;

    // Text processing
    fn skip_comment(
        line: *mut c_char,
        process: bool,
        include_space: bool,
        is_comment: *mut bool,
    ) -> *mut c_char;
    fn skipwhite(s: *const c_char) -> *mut c_char;
    fn has_format_option(opt: c_int) -> bool;
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn utf_eat_space(cc: c_int) -> bool;

    // Extmarks
    fn extmark_splice(
        buf: *mut c_void,
        start_row: c_int,
        start_col: c_int,
        old_row: c_int,
        old_col: c_int,
        old_byte: c_int,
        new_row: c_int,
        new_col: c_int,
        new_byte: c_int,
        undo: c_int,
    );

    // Mark adjustment
    #[link_name = "mark_col_adjust"]
    fn nvim_mark_col_adjust(
        lnum: c_int,
        col: c_int,
        amount_lnum: c_int,
        amount_col: c_int,
        spaces_removed: c_int,
    );

    // Change notifications
    fn changed_lines(
        buf: *mut c_void,
        lnum: c_int,
        col: c_int,
        lnum_end: c_int,
        added: c_int,
        do_buf_event: bool,
    );

    // Deletion
    fn del_lines(nlines: c_int, undo: bool);

    // Cursor state
    fn nvim_curwin_get_cursor_lnum() -> c_int;
    fn nvim_curwin_set_cursor_lnum(lnum: c_int);
    fn nvim_curwin_set_cursor_col(col: c_int);
    fn nvim_check_cursor_col_curwin();
    /// Sets curwin->w_cursor.coladd = 0 and curwin->w_set_curswant = true
    fn nvim_ecmd_curwin_set_coladd_curswant();

    // Marks
    fn nvim_cmdmod_has_lockmarks() -> c_int;
    fn nvim_curbuf_set_op_start(lnum: c_int, col: c_int);
    fn nvim_curbuf_set_op_end(lnum: c_int, col: c_int);
    fn nvim_curbuf_set_deleted_bytes2(val: c_int);

    // Memory
    fn xcalloc(count: usize, size: usize) -> *mut c_void;
    fn xmallocz(size: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_void);

    // Input/interrupt
    fn line_breakcheck();

    // String
    fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;

    // Globals
    static mut got_int: bool;
    static p_js: c_int;
    static p_cpo: *const c_char;
    static mut curbuf: *mut c_void;
    static mut curbuf_splice_pending: c_int;
}

// -----------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------

/// Implement MB_PTR_BACK(base, p): back up pointer p by one character.
/// Returns the new value of p.
/// Equivalent to: p -= utf_head_off(base, p-1) + 1
///
/// # Safety
/// `base` and `p` must be valid pointers with `p > base`.
#[inline]
unsafe fn mb_ptr_back(base: *const c_char, p: *mut c_char) -> *mut c_char {
    let prev = p.sub(1);
    let off = utf_head_off(base, prev) as usize;
    p.sub(off + 1)
}

// -----------------------------------------------------------------------
// `do_join` -- full migration
// -----------------------------------------------------------------------

/// Full port of `do_join()` from ops.c (line 1200).
///
/// # Safety
/// - Accesses global state via C functions
/// - `count >= 1` (asserted)
#[unsafe(export_name = "do_join")]
pub unsafe extern "C" fn rs_do_join(
    count: usize,
    insert_space: bool,
    save_undo: bool,
    use_formatoptions: bool,
    setmark: bool,
) -> c_int {
    assert!(count >= 1);

    let cursor_lnum = nvim_curwin_get_cursor_lnum();

    if save_undo {
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let bot = cursor_lnum + count as c_int;
        if u_save(cursor_lnum - 1, bot) == FAIL {
            return FAIL;
        }
    }

    // Allocate spaces array (number of spaces inserted before each line)
    let spaces: *mut c_char = xcalloc(count, 1).cast();
    let remove_comments = use_formatoptions && has_format_option(FO_REMOVE_COMS);
    let comments: *mut c_int = if remove_comments {
        xcalloc(count, std::mem::size_of::<c_int>()).cast()
    } else {
        std::ptr::null_mut()
    };

    let mut curr_start: *mut c_char = std::ptr::null_mut();
    let mut curr: *mut c_char = std::ptr::null_mut();
    let mut endcurr1 = NUL;
    let mut endcurr2 = NUL;
    let mut currsize: c_int = 0;
    let mut sumsize: c_int = 0;
    let mut ret = OK;
    let mut prev_was_comment = false;

    // Forward pass: compute final line length and spaces array
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let count_i = count as c_int;

    for t in 0..count_i {
        curr_start = ml_get(cursor_lnum + t);
        curr = curr_start;

        if t == 0 && setmark && nvim_cmdmod_has_lockmarks() == 0 {
            // Set the '[ mark
            let col = libc_strlen(curr);
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            nvim_curbuf_set_op_start(cursor_lnum, col as c_int);
        }

        if remove_comments {
            if t > 0 && prev_was_comment {
                let new_curr = skip_comment(curr, true, insert_space, &raw mut prev_was_comment);
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                {
                    *comments.add(t as usize) = new_curr.offset_from(curr) as c_int;
                }
                curr = new_curr;
            } else {
                curr = skip_comment(curr, false, insert_space, &raw mut prev_was_comment);
            }
        }

        if insert_space && t > 0 {
            curr = skipwhite(curr);
            let first_char = utf_ptr2char(curr);
            if *curr != 0 // not NUL
                && *curr != b')' as c_char
                && sumsize != 0
                && endcurr1 != TAB
                && (!has_format_option(FO_MBYTE_JOIN)
                    || (first_char < 0x100 && endcurr1 < 0x100))
                && (!has_format_option(FO_MBYTE_JOIN2)
                    || (first_char < 0x100 && !utf_eat_space(endcurr1))
                    || (endcurr1 < 0x100 && !utf_eat_space(first_char)))
            {
                if endcurr1 == b' ' as c_int {
                    endcurr1 = endcurr2;
                } else {
                    #[allow(clippy::cast_sign_loss)]
                    {
                        *spaces.add(t as usize) += 1;
                    }
                }
                // Extra space for joinspaces after '.', '?', '!'
                if p_js != 0
                    && (endcurr1 == b'.' as c_int
                        || endcurr1 == b'?' as c_int
                        || endcurr1 == b'!' as c_int)
                {
                    #[allow(clippy::cast_sign_loss)]
                    {
                        *spaces.add(t as usize) += 1;
                    }
                }
            }
        }

        if t > 0 && curbuf_splice_pending == 0 {
            #[allow(clippy::cast_sign_loss)]
            let removed = curr.offset_from(curr_start) as c_int;
            #[allow(clippy::cast_sign_loss)]
            let spaces_t = *spaces.add(t as usize) as c_int;
            extmark_splice(
                curbuf,
                cursor_lnum - 1,
                sumsize,
                1,
                removed,
                removed + 1,
                0,
                spaces_t,
                spaces_t,
                K_EXTMARK_UNDO,
            );
        }

        currsize = libc_strlen(curr) as c_int;
        #[allow(clippy::cast_sign_loss)]
        let spaces_t = *spaces.add(t as usize) as c_int;
        sumsize += currsize + spaces_t;
        endcurr1 = NUL;
        endcurr2 = NUL;
        if insert_space && currsize > 0 {
            let mut cend = curr.add(currsize as usize);
            cend = mb_ptr_back(curr, cend);
            endcurr1 = utf_ptr2char(cend);
            if cend > curr {
                cend = mb_ptr_back(curr, cend);
                endcurr2 = utf_ptr2char(cend);
            }
        }

        line_breakcheck();
        if got_int {
            ret = FAIL;
            // goto theend
            xfree(spaces.cast());
            if remove_comments {
                xfree(comments.cast());
            }
            return ret;
        }
    }

    // Column of start of last joined line
    #[allow(clippy::cast_sign_loss)]
    let col = {
        let spaces_last = *spaces.add(count - 1) as c_int;
        sumsize - currsize - spaces_last
    };

    // Allocate new combined line (sumsize bytes + NUL via xmallocz)
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let newp_len = sumsize as usize;
    let newp: *mut c_char = xmallocz(newp_len);
    let mut cend: *mut c_char = newp.add(newp_len);

    // Increment splice pending for the backward pass
    curbuf_splice_pending += 1;

    // Backward pass: fill in the new line from back to front
    let mut t = count_i - 1;
    loop {
        #[allow(clippy::cast_sign_loss)]
        {
            cend = cend.sub(currsize as usize);
        }
        std::ptr::copy_nonoverlapping(curr, cend, currsize as usize);

        #[allow(clippy::cast_sign_loss)]
        let spaces_t = *spaces.add(t as usize) as c_int;
        if spaces_t > 0 {
            #[allow(clippy::cast_sign_loss)]
            {
                cend = cend.sub(spaces_t as usize);
                std::ptr::write_bytes(cend, b' ', spaces_t as usize);
            }
        }

        // Mark adjustment
        let spaces_removed = curr.offset_from(curr_start) as c_int - spaces_t;
        let lnum = cursor_lnum + t;
        let lnum_amount = -t;
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let col_amount = cend.offset_from(newp) as c_int - spaces_removed;

        nvim_mark_col_adjust(lnum, 0, lnum_amount, col_amount, spaces_removed);

        if t == 0 {
            break;
        }

        curr_start = ml_get(cursor_lnum + t - 1);
        curr = curr_start;
        if remove_comments {
            #[allow(clippy::cast_sign_loss)]
            {
                curr = curr.add(*comments.add((t - 1) as usize) as usize);
            }
        }
        if insert_space && t > 1 {
            curr = skipwhite(curr);
        }
        currsize = libc_strlen(curr) as c_int;

        t -= 1;
    }

    ml_replace_len(cursor_lnum, newp, newp_len, false);

    if setmark && nvim_cmdmod_has_lockmarks() == 0 {
        // Set the '] mark
        nvim_curbuf_set_op_end(cursor_lnum, sumsize);
    }

    // Only report change in first line; del_lines will report the rest
    changed_lines(curbuf, cursor_lnum, currsize, cursor_lnum + 1, 0, true);

    // Delete the following lines (temporarily move cursor)
    let t = cursor_lnum;
    nvim_curwin_set_cursor_lnum(cursor_lnum + 1);
    del_lines(count_i - 1, false);
    nvim_curwin_set_cursor_lnum(t);
    curbuf_splice_pending -= 1;
    nvim_curbuf_set_deleted_bytes2(0);

    // Set cursor column
    let new_col = if vim_strchr(p_cpo, CPO_JOINCOL).is_null() {
        col
    } else {
        currsize
    };
    nvim_curwin_set_cursor_col(new_col);
    nvim_check_cursor_col_curwin();
    nvim_ecmd_curwin_set_coladd_curswant();

    xfree(spaces.cast());
    if remove_comments {
        xfree(comments.cast());
    }
    ret
}

// -----------------------------------------------------------------------
// Helper: strlen via pointer iteration (avoids import of libc)
// -----------------------------------------------------------------------

/// Compute strlen of a C string. Returns byte count (not including NUL).
///
/// # Safety
/// `s` must be a valid NUL-terminated C string.
#[inline]
unsafe fn libc_strlen(s: *const c_char) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut p = s;
    while *p != 0 {
        p = p.add(1);
    }
    #[allow(clippy::cast_sign_loss)]
    {
        p.offset_from(s) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
        assert_eq!(FO_REMOVE_COMS, c_int::from(b'j'));
        assert_eq!(FO_MBYTE_JOIN, c_int::from(b'M'));
        assert_eq!(FO_MBYTE_JOIN2, c_int::from(b'B'));
        assert_eq!(CPO_JOINCOL, c_int::from(b'q'));
    }

    #[test]
    fn test_libc_strlen_null_safe() {
        // SAFETY: null pointer test is handled
        unsafe { assert_eq!(libc_strlen(std::ptr::null()), 0) };
    }
}
