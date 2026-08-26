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

use core::ffi::{c_char, c_int};

use super::*;
use crate::types::{FAIL, NUL, OK};
use crate::winlayer::Win;

crate::flag_set! {
    /// What [`beginline`] should do about the column -- upstream's `BL_*`.
    pub(crate) struct BeginlineOpts;

    /// Stop at the first non-white character.
    const WHITE = 1;
    /// Do that only when `'startofline'` is set; otherwise keep the column
    /// the user wants (`w_curswant`) and do not move horizontally at all.
    const SOL = 2;
    /// Do not leave the cursor on the NUL: on an all-white line stop on the
    /// last blank rather than past it.
    const FIX = 4;
}

/// Move the cursor to the start of the current line.
///
/// # Safety
/// Must run with a live `curwin` whose cursor line exists.
pub(crate) unsafe fn beginline(flags: BeginlineOpts) {
    let mut win = cur_win();
    if flags.has(BeginlineOpts::SOL) && p_sol.get() == 0 {
        // SAFETY: `curwin` is live for the whole session.
        let want = win.w_curswant;
        coladvance_win(win, want);
    } else {
        win.w_cursor.col = 0;
        win.w_cursor.coladd = 0;

        if flags.has(BeginlineOpts::WHITE | BeginlineOpts::SOL) {
            let mut ptr = unsafe { get_cursor_line_ptr() };
            // `ptr[1] == NUL` under `FIX` is what keeps an all-white
            // line from ending with the cursor on the NUL.
            while ascii_iswhite(unsafe { *ptr } as c_int)
                && !(flags.has(BeginlineOpts::FIX) && unsafe { *ptr.offset(1) } as c_int == NUL)
            {
                win.w_cursor.col += 1;
                ptr = unsafe { ptr.offset(1) };
            }
        }
        win.w_set_curswant = true;
    }
    adjust_skipcol_now();
}

/// Move one character right, answering `OK` or `FAIL` at the end of the line.
///
/// # Safety
/// Must run with a live `curwin` whose cursor is on a valid position.
pub(crate) unsafe fn oneright() -> c_int {
    // SAFETY: every `unsafe` call below is an editor-wide routine whose only
    // precondition is the live `curwin`/`curbuf` this mode runs with.
    // The strings walked below are NUL-terminated lines of that buffer, and
    // every step stops at the NUL.
    let mut win = cur_win();

    if virtual_edit(win) {
        // In 'virtualedit' the step is a *screen* column, so a wide
        // character has to be stepped over whole -- except a TAB, whose
        // width the cursor is allowed to sit inside.
        let prevpos = win.w_cursor;
        let ptr = cursor_pos_ptr();
        let width = if unsafe { *ptr } as c_int != TAB && unsafe { vim_isprintc(utf_ptr2char(ptr)) }
        {
            unsafe { ptr2cells(ptr) }
        } else {
            1
        };
        coladvance_win(win, viscol() + width);
        win.w_set_curswant = true;
        // OK if the cursor moved, FAIL otherwise (at the window edge).
        return if prevpos.col != win.w_cursor.col || prevpos.coladd != win.w_cursor.coladd {
            OK
        } else {
            FAIL
        };
    }

    let ptr = cursor_pos_ptr();
    if unsafe { *ptr } as c_int == NUL {
        return FAIL; // already at the very end
    }

    // Move "l" bytes right, but do not end up on the NUL unless
    // 'virtualedit' contains "onemore".
    let l = unsafe { utfc_ptr2len(ptr) };
    if unsafe { *ptr.offset(l as isize) } as c_int == NUL
        && unsafe { get_ve_flags(win.raw()) } & kOptVeFlagOnemore as c_int as ::core::ffi::c_uint
            == 0
    {
        return FAIL;
    }
    win.w_cursor.col += l;

    win.w_set_curswant = true;
    adjust_skipcol_now();
    OK
}

/// Move one character left, answering `OK` or `FAIL` at column 0.
///
/// # Safety
/// Must run with a live `curwin` whose cursor is on a valid position.
pub(crate) unsafe fn oneleft() -> c_int {
    // SAFETY: every `unsafe` call below is an editor-wide routine whose only
    // precondition is the live `curwin`/`curbuf` this mode runs with.
    // The strings walked below are NUL-terminated lines of that buffer, and
    // every step stops at the NUL.
    let mut win = cur_win();

    if virtual_edit(win) {
        let v = viscol();
        if v == 0 {
            return FAIL;
        }

        // One screen column left may land on the same virtual column --
        // 'showbreak' and 'breakindent' both insert columns the cursor
        // cannot occupy -- so widen the step until it actually moves.
        let mut width = 1;
        loop {
            coladvance_win(win, v as colnr_T - width as colnr_T);
            if viscol() < v {
                break;
            }
            width += 1;
        }

        if win.w_cursor.coladd == 1 {
            // Landed one cell inside a character: legal for a TAB, not
            // for a wide one.
            let ptr = cursor_pos_ptr();
            if unsafe { *ptr } as c_int != TAB
                && unsafe { vim_isprintc(utf_ptr2char(ptr)) }
                && unsafe { ptr2cells(ptr) } > 1
            {
                win.w_cursor.coladd = 0;
            }
        }

        win.w_set_curswant = true;
        adjust_skipcol_now();
        return OK;
    }

    if win.w_cursor.col == 0 {
        return FAIL;
    }

    win.w_set_curswant = true;
    win.w_cursor.col -= 1;
    // The byte to the left may be the tail of a multi-byte character.
    unsafe { mb_adjust_cursor() };
    adjust_skipcol_now();
    OK
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
    // SAFETY: the caller promises `wp` is a live window.
    let mut win = unsafe { Win::new(wp) };
    let mut lnum = win.w_cursor.lnum;

    if n >= lnum {
        lnum = 1;
    } else if lines_concealed(win) {
        // Count each sequence of folded lines as one logical line: go to
        // the start of the fold the cursor is in first.
        fold_start(win, lnum, &mut lnum);

        while n != 0 {
            n -= 1;
            lnum -= 1;
            if lnum <= 1 {
                break;
            }
            n += (skip_conceal && line_concealed(win, lnum)) as linenr_T;
            // On entering a fold, move to its beginning -- unless this is
            // the last step and the fold is about to open anyway.
            if n > 0
                || !(State.get() & MODE_INSERT != 0
                    || fdo_flags.get() & kOptFdoFlagAll as ::core::ffi::c_uint != 0)
            {
                fold_start(win, lnum, &mut lnum);
            }
        }
        lnum = lnum.max(1);
    } else {
        lnum -= n;
    }

    win.w_cursor.lnum = lnum;
}

/// `k`: move the cursor up `n` lines and back to the wanted column.
///
/// `FAIL` when the cursor is already on line 1.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn cursor_up(n: linenr_T, upd_topline: bool) -> c_int {
    let win = cur_win();
    if n > 0 && win.w_cursor.lnum <= 1 {
        return FAIL;
    }
    // SAFETY: `curwin` is live for the whole session.
    unsafe { cursor_up_inner(win.raw(), n, false) };

    // Try to advance to the column we want to be at.
    let want = win.w_curswant;
    coladvance_win(win, want);

    if upd_topline {
        unsafe { update_topline(win.raw()) }; // make sure w_topline is valid
    }
    OK
}

/// Move `wp`'s cursor down `n` lines, counting a closed fold as one line.
///
/// The mirror of [`cursor_up_inner`], including the `skip_conceal` step-back.
///
/// # Safety
/// `wp` must point to a live window.
pub(crate) unsafe fn cursor_down_inner(wp: *mut win_T, mut n: c_int, skip_conceal: bool) {
    // SAFETY: the caller promises `wp` is a live window.
    let mut win = unsafe { Win::new(wp) };
    let mut lnum = win.w_cursor.lnum;
    let line_count = win.buffer().b_ml.ml_line_count;

    if lnum + n as linenr_T >= line_count {
        lnum = line_count;
    } else if lines_concealed(win) {
        let mut last: linenr_T = 0;
        while n != 0 {
            n -= 1;
            if fold_end(win, lnum, &mut last) {
                lnum = last + 1;
            } else {
                lnum += 1;
            }
            if lnum >= line_count {
                break;
            }
            n += (skip_conceal && line_concealed(win, lnum)) as c_int;
        }
        lnum = lnum.min(line_count);
    } else {
        lnum += n as linenr_T;
    }

    win.w_cursor.lnum = lnum;
}

/// `j`: move the cursor down `n` lines and back to the wanted column.
///
/// `FAIL` when the cursor is already in the last line -- or in the fold that
/// ends on it, which is why the bound is measured from the fold's end.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn cursor_down(n: c_int, upd_topline: bool) -> c_int {
    let win = cur_win();
    let mut lnum = win.w_cursor.lnum;
    fold_end(win, lnum, &mut lnum);
    if n > 0 && lnum >= win.buffer().b_ml.ml_line_count {
        return FAIL;
    }
    // SAFETY: `curwin` is live for the whole session.
    unsafe { cursor_down_inner(win.raw(), n, false) };

    // Try to advance to the column we want to be at.
    let want = win.w_curswant;
    coladvance_win(win, want);

    if upd_topline {
        unsafe { update_topline(win.raw()) }; // make sure w_topline is valid
    }
    OK
}

/// Keep 'smoothscroll''s skipped column in range after a cursor move.
#[inline(always)]
fn adjust_skipcol_now() {
    // SAFETY: `curwin` is live for the whole session.
    unsafe { adjust_skipcol() }
}

/// Move `win`'s cursor to virtual column `vcol` of its line.
#[inline(always)]
fn coladvance_win(win: Win, vcol: colnr_T) {
    // SAFETY: a live window, whose cursor line exists.
    unsafe { coladvance(win.raw(), vcol) };
}

/// The cursor's line, from the cursor's own column on.
#[inline(always)]
fn cursor_pos_ptr() -> *mut c_char {
    // SAFETY: `curwin`/`curbuf` are live for the whole session.
    unsafe { get_cursor_pos_ptr() }
}

/// The cursor's virtual column.
#[inline(always)]
fn viscol() -> colnr_T {
    // SAFETY: `curwin` is live for the whole session.
    unsafe { getviscol() }
}

/// Is 'virtualedit' letting `win`'s cursor stand where no character is?
#[inline(always)]
fn virtual_edit(win: Win) -> bool {
    // SAFETY: a live window.
    unsafe { virtual_active(win.raw()) }
}

/// Does `win` hide any of its lines, by a fold or a decoration?
#[inline(always)]
fn lines_concealed(win: Win) -> bool {
    // SAFETY: a live window.
    unsafe { win_lines_concealed(win.raw()) }
}

/// Is the line *before* `lnum` hidden by a `conceal_lines` decoration?
#[inline(always)]
fn line_concealed(win: Win, lnum: linenr_T) -> bool {
    // SAFETY: a live window and a line number of its buffer.
    unsafe { decor_conceal_line(win.raw(), lnum as c_int - 1, true) }
}

/// Is `lnum` inside a closed fold of `win`?  `first` is left holding that
/// fold's first line when it is.
#[inline(always)]
fn fold_start(win: Win, lnum: linenr_T, first: &mut linenr_T) -> bool {
    // SAFETY: a live window and a line number of its buffer.
    unsafe { has_folding(win.raw(), lnum, first, ::core::ptr::null_mut()) }
}

/// [`fold_start`], leaving the fold's *last* line in `last` instead.
#[inline(always)]
fn fold_end(win: Win, lnum: linenr_T, last: &mut linenr_T) -> bool {
    // SAFETY: a live window and a line number of its buffer.
    unsafe {
        has_folding_win(
            win.raw(),
            lnum,
            ::core::ptr::null_mut(),
            last,
            true,
            ::core::ptr::null_mut(),
        )
    }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
