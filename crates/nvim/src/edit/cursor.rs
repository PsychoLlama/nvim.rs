//! Moving the cursor one line or one character, from anywhere.
//!
//! These live in edit.c for historical reasons and are called from all over
//! the tree; they are not Insert-mode specific.  What they have in common is
//! that each is the *legal* version of an obvious operation: [`oneright`] and
//! [`oneleft`] refuse to step onto the NUL past the end of a line unless
//! 'virtualedit' or 'whichwrap' allow it, and know about composing
//! characters; [`cursor_up`]/[`cursor_down`] treat a closed fold as one line
//! and skip concealed lines; [`beginline`] is the "go to the start of the
//! line" whose meaning 'startofline' changes.
//!
//! Every one of them ends in `adjust_skipcol` or `coladvance`, because the
//! cursor is not allowed to come to rest on a column that is not there.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::*;
use crate::types::{FAIL, NUL, OK};

/// Move the cursor to the start of the current line.
///
/// `flags` is a set of `BL_*`:
/// - `BL_WHITE` -- stop at the first non-white character.
/// - `BL_SOL` -- do that only when 'startofline' is set; otherwise keep the
///   column the user wants (`w_curswant`) and do not move horizontally at
///   all.
/// - `BL_FIX` -- do not leave the cursor on the NUL, i.e. on an all-white
///   line stop on the last blank rather than past it.
///
/// # Safety
/// Must run with a live `curwin` whose cursor line exists.
pub(crate) unsafe fn beginline(flags: c_int) {
    unsafe {
        let win = curwin.get();
        if flags & BL_SOL != 0 && p_sol.get() == 0 {
            coladvance(win, (*win).w_curswant);
        } else {
            (*win).w_cursor.col = 0;
            (*win).w_cursor.coladd = 0;

            if flags & (BL_WHITE | BL_SOL) != 0 {
                let mut ptr = get_cursor_line_ptr();
                // `ptr[1] == NUL` under BL_FIX is what keeps an all-white
                // line from ending with the cursor on the NUL.
                while ascii_iswhite(*ptr as c_int)
                    && !(flags & BL_FIX != 0 && *ptr.offset(1) as c_int == NUL)
                {
                    (*win).w_cursor.col += 1;
                    ptr = ptr.offset(1);
                }
            }
            (*win).w_set_curswant = true_0;
        }
        adjust_skipcol();
    }
}

/// Move one character right, answering `OK` or `FAIL` at the end of the line.
///
/// # Safety
/// Must run with a live `curwin` whose cursor is on a valid position.
pub(crate) unsafe fn oneright() -> c_int {
    unsafe {
        let win = curwin.get();

        if virtual_active(win) {
            // In 'virtualedit' the step is a *screen* column, so a wide
            // character has to be stepped over whole -- except a TAB, whose
            // width the cursor is allowed to sit inside.
            let prevpos = (*win).w_cursor;
            let ptr = get_cursor_pos_ptr();
            let width = if *ptr as c_int != TAB && vim_isprintc(utf_ptr2char(ptr)) {
                ptr2cells(ptr)
            } else {
                1
            };
            coladvance(win, getviscol() + width);
            (*win).w_set_curswant = true_0;
            // OK if the cursor moved, FAIL otherwise (at the window edge).
            return if prevpos.col != (*win).w_cursor.col || prevpos.coladd != (*win).w_cursor.coladd
            {
                OK
            } else {
                FAIL
            };
        }

        let ptr = get_cursor_pos_ptr();
        if *ptr as c_int == NUL {
            return FAIL; // already at the very end
        }

        // Move "l" bytes right, but do not end up on the NUL unless
        // 'virtualedit' contains "onemore".
        let l = utfc_ptr2len(ptr);
        if *ptr.offset(l as isize) as c_int == NUL
            && get_ve_flags(win) & kOptVeFlagOnemore as c_int as ::core::ffi::c_uint == 0
        {
            return FAIL;
        }
        (*win).w_cursor.col += l;

        (*win).w_set_curswant = true_0;
        adjust_skipcol();
        OK
    }
}

/// Move one character left, answering `OK` or `FAIL` at column 0.
///
/// # Safety
/// Must run with a live `curwin` whose cursor is on a valid position.
pub(crate) unsafe fn oneleft() -> c_int {
    unsafe {
        let win = curwin.get();

        if virtual_active(win) {
            let v = getviscol();
            if v == 0 {
                return FAIL;
            }

            // One screen column left may land on the same virtual column --
            // 'showbreak' and 'breakindent' both insert columns the cursor
            // cannot occupy -- so widen the step until it actually moves.
            let mut width = 1;
            loop {
                coladvance(win, v as colnr_T - width as colnr_T);
                if getviscol() < v {
                    break;
                }
                width += 1;
            }

            if (*win).w_cursor.coladd == 1 {
                // Landed one cell inside a character: legal for a TAB, not
                // for a wide one.
                let ptr = get_cursor_pos_ptr();
                if *ptr as c_int != TAB && vim_isprintc(utf_ptr2char(ptr)) && ptr2cells(ptr) > 1 {
                    (*win).w_cursor.coladd = 0;
                }
            }

            (*win).w_set_curswant = true_0;
            adjust_skipcol();
            return OK;
        }

        if (*win).w_cursor.col == 0 {
            return FAIL;
        }

        (*win).w_set_curswant = true_0;
        (*win).w_cursor.col -= 1;
        // The byte to the left may be the tail of a multi-byte character.
        mb_adjust_cursor();
        adjust_skipcol();
        OK
    }
}

/// Move `wp`'s cursor up `n` lines, counting a closed fold as one line.
///
/// With `skip_conceal`, a line hidden by a `conceal_lines` decoration does
/// not count either -- which is spelled by giving `n` its step back, so the
/// loop runs one more time.
///
/// # Safety
/// `wp` must point to a live window.
pub(crate) unsafe fn cursor_up_inner(wp: *mut win_T, mut n: linenr_T, skip_conceal: bool) {
    unsafe {
        let mut lnum = (*wp).w_cursor.lnum;

        if n >= lnum {
            lnum = 1;
        } else if win_lines_concealed(wp) {
            // Count each sequence of folded lines as one logical line: go to
            // the start of the fold the cursor is in first.
            hasFolding(wp, lnum, &raw mut lnum, ::core::ptr::null_mut());

            while n != 0 {
                n -= 1;
                lnum -= 1;
                if lnum <= 1 {
                    break;
                }
                n += (skip_conceal && decor_conceal_line(wp, lnum as c_int - 1, true)) as linenr_T;
                // On entering a fold, move to its beginning -- unless this is
                // the last step and the fold is about to open anyway.
                if n > 0
                    || !(State.get() & MODE_INSERT != 0
                        || fdo_flags.get() & kOptFdoFlagAll as ::core::ffi::c_uint != 0)
                {
                    hasFolding(wp, lnum, &raw mut lnum, ::core::ptr::null_mut());
                }
            }
            lnum = lnum.max(1);
        } else {
            lnum -= n;
        }

        (*wp).w_cursor.lnum = lnum;
    }
}

/// `k`: move the cursor up `n` lines and back to the wanted column.
///
/// `FAIL` when the cursor is already on line 1.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn cursor_up(n: linenr_T, upd_topline: bool) -> c_int {
    unsafe {
        let win = curwin.get();
        if n > 0 && (*win).w_cursor.lnum <= 1 {
            return FAIL;
        }
        cursor_up_inner(win, n, false);

        // Try to advance to the column we want to be at.
        coladvance(win, (*win).w_curswant);

        if upd_topline {
            update_topline(win); // make sure w_topline is valid
        }
        OK
    }
}

/// Move `wp`'s cursor down `n` lines, counting a closed fold as one line.
///
/// The mirror of [`cursor_up_inner`], including the `skip_conceal` step-back.
///
/// # Safety
/// `wp` must point to a live window.
pub(crate) unsafe fn cursor_down_inner(wp: *mut win_T, mut n: c_int, skip_conceal: bool) {
    unsafe {
        let mut lnum = (*wp).w_cursor.lnum;
        let line_count = (*(*wp).w_buffer).b_ml.ml_line_count;

        if lnum + n as linenr_T >= line_count {
            lnum = line_count;
        } else if win_lines_concealed(wp) {
            let mut last: linenr_T = 0;
            while n != 0 {
                n -= 1;
                if hasFoldingWin(
                    wp,
                    lnum,
                    ::core::ptr::null_mut(),
                    &raw mut last,
                    true,
                    ::core::ptr::null_mut(),
                ) {
                    lnum = last + 1;
                } else {
                    lnum += 1;
                }
                if lnum >= line_count {
                    break;
                }
                n += (skip_conceal && decor_conceal_line(wp, lnum as c_int - 1, true)) as c_int;
            }
            lnum = lnum.min(line_count);
        } else {
            lnum += n as linenr_T;
        }

        (*wp).w_cursor.lnum = lnum;
    }
}

/// `j`: move the cursor down `n` lines and back to the wanted column.
///
/// `FAIL` when the cursor is already in the last line -- or in the fold that
/// ends on it, which is why the bound is measured from the fold's end.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn cursor_down(n: c_int, upd_topline: bool) -> c_int {
    unsafe {
        let win = curwin.get();
        let mut lnum = (*win).w_cursor.lnum;
        hasFoldingWin(
            win,
            lnum,
            ::core::ptr::null_mut(),
            &raw mut lnum,
            true,
            ::core::ptr::null_mut(),
        );
        if n > 0 && lnum >= (*(*win).w_buffer).b_ml.ml_line_count {
            return FAIL;
        }
        cursor_down_inner(win, n, false);

        // Try to advance to the column we want to be at.
        coladvance(win, (*win).w_curswant);

        if upd_topline {
            update_topline(win); // make sure w_topline is valid
        }
        OK
    }
}
