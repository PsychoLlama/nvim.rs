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

#![forbid(unsafe_code)]

use core::ffi::c_int;

use super::*;
use crate::charset::ptr2cells_at;
use crate::cstr::byte_at;
use crate::mbyte::{c_cluster_len, char_at};
use crate::types::{Failed, NUL};
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
/// Safe: the only promise is that the editor exists, which `cur_win()`
/// carries.
pub(crate) fn beginline(flags: BeginlineOpts) {
    let mut win = Win::current();
    if flags.has(BeginlineOpts::SOL) && !p_sol() {
        let want = win.w_curswant;
        coladvance_win(win, want);
    } else {
        win.w_cursor.col = 0;
        win.w_cursor.coladd = 0;

        if flags.has(BeginlineOpts::WHITE | BeginlineOpts::SOL) {
            let mut lines = Lines::current();
            let line = lines.line(win.w_cursor.lnum);
            let mut at = 0;
            // `line[at + 1] == NUL` under `FIX` is what keeps an all-white
            // line from ending with the cursor on the NUL.
            while ascii_iswhite(c_int::from(byte_at(line, at)))
                && !(flags.has(BeginlineOpts::FIX) && byte_at(line, at + 1) == NUL as u8)
            {
                win.w_cursor.col += 1;
                at += 1;
            }
        }
        win.w_set_curswant = true;
    }
    adjust_skipcol_now();
}

/// Move one character right; answers `Err` at the end of the line.
pub(crate) fn oneright() -> Result<(), Failed> {
    let mut win = Win::current();

    if virtual_edit(win) {
        // In 'virtualedit' the step is a *screen* column, so a wide
        // character has to be stepped over whole -- except a TAB, whose
        // width the cursor is allowed to sit inside.
        let prevpos = win.w_cursor;
        let width = with_cursor_tail(|tail| {
            if printable_not_tab(tail) {
                ptr2cells_at(tail)
            } else {
                1
            }
        });
        coladvance_win(win, viscol() + width);
        win.w_set_curswant = true;
        // OK if the cursor moved, FAIL otherwise (at the window edge).
        return if prevpos.col != win.w_cursor.col || prevpos.coladd != win.w_cursor.coladd {
            Ok(())
        } else {
            Err(Failed)
        };
    }

    // How far the next character is, and whether it is the last one.
    let Some((l, last)) = with_cursor_tail(|tail| {
        let l = c_cluster_len(tail);
        (l != 0).then(|| (l, byte_at(tail, l) == NUL as u8))
    }) else {
        return Err(Failed); // already at the very end
    };

    // Move "l" bytes right, but do not end up on the NUL unless
    // 'virtualedit' contains "onemore".
    if last && get_ve_flags(win) & kOptVeFlagOnemore as c_int as ::core::ffi::c_uint == 0 {
        return Err(Failed);
    }
    win.w_cursor.col += l as ColNr;

    win.w_set_curswant = true;
    adjust_skipcol_now();
    Ok(())
}

/// Move one character left; answers `Err` at column 0.
pub(crate) fn oneleft() -> Result<(), Failed> {
    let mut win = Win::current();

    if virtual_edit(win) {
        let v = viscol();
        if v == 0 {
            return Err(Failed);
        }

        // One screen column left may land on the same virtual column --
        // 'showbreak' and 'breakindent' both insert columns the cursor
        // cannot occupy -- so widen the step until it actually moves.
        let mut width = 1;
        loop {
            coladvance_win(win, v as ColNr - width as ColNr);
            if viscol() < v {
                break;
            }
            width += 1;
        }

        if win.w_cursor.coladd == 1 {
            // Landed one cell inside a character: legal for a TAB, not
            // for a wide one.
            if with_cursor_tail(|tail| printable_not_tab(tail) && ptr2cells_at(tail) > 1) {
                win.w_cursor.coladd = 0;
            }
        }

        win.w_set_curswant = true;
        adjust_skipcol_now();
        return Ok(());
    }

    if win.w_cursor.col == 0 {
        return Err(Failed);
    }

    win.w_set_curswant = true;
    win.w_cursor.col -= 1;
    // The byte to the left may be the tail of a multi-byte character.
    mb_adjust_cursor();
    adjust_skipcol_now();
    Ok(())
}

/// Move `wp`'s cursor up `n` lines, counting a closed fold as one line.
///
/// With `skip_conceal`, a line hidden by a `conceal_lines` decoration does
/// not count either -- which is spelled by giving `n` its step back, so the
/// loop runs one more time.
///
/// Safe: a [`Win`] carries the whole of the promise this needs.
pub(crate) fn cursor_up_inner(mut win: Win, mut n: LineNr, skip_conceal: bool) {
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
            n += (skip_conceal && line_concealed(win, lnum)) as LineNr;
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
/// `Err` when the cursor is already on line 1.
pub(crate) fn cursor_up(n: LineNr, upd_topline: bool) -> Result<(), Failed> {
    let win = Win::current();
    if n > 0 && win.w_cursor.lnum <= 1 {
        return Err(Failed);
    }
    cursor_up_inner(win, n, false);

    // Try to advance to the column we want to be at.
    let want = win.w_curswant;
    coladvance_win(win, want);

    if upd_topline {
        update_topline(win); // make sure w_topline is valid
    }
    Ok(())
}

/// Move `wp`'s cursor down `n` lines, counting a closed fold as one line.
///
/// The mirror of [`cursor_up_inner`], including the `skip_conceal` step-back.
///
/// Safe: a [`Win`] carries the whole of the promise this needs.
pub(crate) fn cursor_down_inner(mut win: Win, mut n: c_int, skip_conceal: bool) {
    let mut lnum = win.w_cursor.lnum;
    let line_count = win.buffer().b_ml.ml_line_count;

    if lnum + n as LineNr >= line_count {
        lnum = line_count;
    } else if lines_concealed(win) {
        let mut last: LineNr = 0;
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
        lnum += n as LineNr;
    }

    win.w_cursor.lnum = lnum;
}

/// `j`: move the cursor down `n` lines and back to the wanted column.
///
/// `Err` when the cursor is already in the last line -- or in the fold that
/// ends on it, which is why the bound is measured from the fold's end.
pub(crate) fn cursor_down(n: c_int, upd_topline: bool) -> Result<(), Failed> {
    let win = Win::current();
    let mut lnum = win.w_cursor.lnum;
    fold_end(win, lnum, &mut lnum);
    if n > 0 && lnum >= win.buffer().b_ml.ml_line_count {
        return Err(Failed);
    }
    cursor_down_inner(win, n, false);

    // Try to advance to the column we want to be at.
    let want = win.w_curswant;
    coladvance_win(win, want);

    if upd_topline {
        update_topline(win); // make sure w_topline is valid
    }
    Ok(())
}

/// Keep 'smoothscroll''s skipped column in range after a cursor move.
#[inline(always)]
fn adjust_skipcol_now() {
    adjust_skipcol()
}

/// Move `win`'s cursor to virtual column `vcol` of its line.
#[inline(always)]
fn coladvance_win(win: Win, vcol: ColNr) {
    coladvance(win, vcol);
}

/// `f` of the cursor's line from the cursor's own column on.
#[inline(always)]
fn with_cursor_tail<R>(f: impl FnOnce(&[u8]) -> R) -> R {
    let cursor = Win::current().w_cursor;
    let mut lines = Lines::current();
    let line = lines.line(cursor.lnum);
    f(&line[(cursor.col as usize).min(line.len())..])
}

/// Whether the character `tail` starts with is printable and not a TAB --
/// the one a 'virtualedit' cursor may not sit inside.
#[inline(always)]
fn printable_not_tab(tail: &[u8]) -> bool {
    c_int::from(byte_at(tail, 0)) != TAB && vim_isprintc(char_at(tail))
}

/// The cursor's virtual column.
#[inline(always)]
fn viscol() -> ColNr {
    getviscol()
}

/// Is 'virtualedit' letting `win`'s cursor stand where no character is?
#[inline(always)]
fn virtual_edit(win: Win) -> bool {
    virtual_active(win)
}

/// Does `win` hide any of its lines, by a fold or a decoration?
#[inline(always)]
fn lines_concealed(win: Win) -> bool {
    win_lines_concealed(win)
}

/// Is the line *before* `lnum` hidden by a `conceal_lines` decoration?
#[inline(always)]
fn line_concealed(win: Win, lnum: LineNr) -> bool {
    decor_conceal_line(win, lnum as c_int - 1, true)
}

/// Is `lnum` inside a closed fold of `win`?  `first` is left holding that
/// fold's first line when it is.
#[inline(always)]
fn fold_start(win: Win, lnum: LineNr, first: &mut LineNr) -> bool {
    has_folding(win, lnum, Some(first), None)
}

/// [`fold_start`], leaving the fold's *last* line in `last` instead.
#[inline(always)]
fn fold_end(win: Win, lnum: LineNr, last: &mut LineNr) -> bool {
    has_folding_win(win, lnum, None, Some(last), true, None)
}
