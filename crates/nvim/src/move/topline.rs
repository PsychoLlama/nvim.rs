//! Deciding which line the window starts at -- `update_topline()` and the
//! validity bookkeeping around it.
//!
//! [`update_topline`] is the entry point every redraw goes through: it decides
//! whether the cursor has left the visible range and, if so, hands off to the
//! `scroll_cursor_*` family to pick a new `w_topline`.  Around it sit
//! `'scrolljump'`, the `'scrolloff'` margin test, the "did the cursor move?"
//! memo that lets a redraw skip the work entirely, and `set_topline`, the
//! explicit form used when something else has already chosen the line.
//!
//! Original: `src/nvim/move.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::*;
use crate::buffer::buf_is_empty;
use crate::drawscreen::{UPD_NOT_VALID, UPD_SOME_VALID, UPD_VALID};
use crate::main::{
    curtab, default_grid, dollar_vcol, first_tabpage, firstwin, mouse_dragging, p_sj, p_so,
    skip_update_topline,
};
use crate::types::{OptInt, int64_t, linenr_T, win_T};
use crate::winlayer::Win;

/// The 'scrolloff' `update_topline()` works with: the window-local value when
/// it is set, the global one otherwise.
///
/// C reaches this through an `OptInt *` because the mouse-drag arm *writes*
/// through it and restores the old value on the way out, which is what the
/// two variants are here to reproduce.
#[derive(Clone, Copy)]
enum ScrollOff {
    Window(Win),
    Global,
}

impl ScrollOff {
    fn of(win: Win) -> Self {
        if win.w_onebuf_opt.wo_so >= 0 {
            Self::Window(win)
        } else {
            Self::Global
        }
    }

    fn get(self) -> OptInt {
        match self {
            Self::Window(win) => win.w_onebuf_opt.wo_so,
            Self::Global => p_so.get(),
        }
    }

    fn set(self, value: OptInt) {
        match self {
            Self::Window(mut win) => win.w_onebuf_opt.wo_so = value,
            Self::Global => p_so.set(value),
        }
    }
}

/// [`Win::update_topline`], for the callers still holding a raw window.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn update_topline(wp: *mut win_T) {
    // SAFETY: the caller's promise.
    unsafe { Win::new(wp) }.update_topline();
}

impl Win {
    /// Move `w_topline` so that the cursor is on the screen, with 'scrolloff'
    /// lines of context above and below it where the buffer allows.
    pub(crate) fn update_topline(self) {
        update_topline_win(self);
    }
}

fn update_topline_win(mut win: Win) {
    let wp = win.raw();
    let mut check_botline = false;
    let so = ScrollOff::of(win);
    let save_so = so.get();

    // With 'splitkeep' the cursor is moved instead.
    if skip_update_topline.get() {
        return;
    }

    // No screen yet, or a window with no room: just show the cursor line.
    if default_grid.with(|grid| grid.chars.is_null()) || win.w_view_height == 0 {
        win.check_cursor_lnum();
        win.w_topline = win.w_cursor.lnum;
        win.w_botline = win.w_topline;
        win.w_viewport_invalid = true;
        win.w_scbind_pos = 1;
        return;
    }

    win.check_cursor_moved();
    if win.w_valid & VALID_TOPLINE != 0 {
        return;
    }

    // Dragging with the mouse should not scroll that quickly. This writes
    // the option value itself and restores it on the way out, so a window
    // reading `&l:scrolloff` from inside a drag sees the slowed-down one.
    if mouse_dragging.get() > 0 {
        so.set((mouse_dragging.get() - 1) as OptInt);
    }

    let old_topline = win.w_topline;
    let old_topfill = win.w_topfill;

    // SAFETY: a live buffer.
    if unsafe { buf_is_empty(win.buffer().raw()) } {
        // Special case: an empty file always starts at line 1.
        if win.w_topline != 1 {
            win.redraw_later(UPD_NOT_VALID);
        }
        win.w_topline = 1;
        win.w_botline = 2;
        win.w_skipcol = 0;
        win.w_valid |= VALID_BOTLINE | VALID_BOTLINE_AP;
        win.w_viewport_invalid = true;
        win.w_scbind_pos = 1;
    } else if check_topline(win) {
        let halfheight = arith::recentre_threshold(win.w_view_height);
        // How far the cursor is above the top of the window, give or take:
        // an approximation of how much would have to be scrolled.
        let n = if win.lines_concealed() {
            unconcealed_above(win, halfheight, so.get())
        } else {
            (win.w_topline as OptInt + so.get() - win.w_cursor.lnum as OptInt) as int64_t
        };
        // Far out to begin with: put the cursor in the middle of the window.
        // Close: put it near the top.
        if n >= halfheight as int64_t {
            // SAFETY: a live window.
            unsafe { scroll_cursor_halfway(wp, false, false) };
        } else {
            // SAFETY: a live window.
            unsafe {
                scroll_cursor_top(
                    wp,
                    arith::scrolljump_lines(p_sj.get(), win.w_view_height),
                    false_0,
                )
            };
            check_botline = true;
        }
    } else {
        // Make sure the top line is the first line of a fold.
        win.w_topline = win.fold_first(win.w_topline).unwrap_or(win.w_topline);
        check_botline = true;
    }

    // The cursor below the bottom of the window: scroll it into view.
    // Recompute `w_botline` first when it is invalid, to avoid a later
    // redraw; when it was only approximated a redraw may still be needed in
    // a few cases, but recomputing it for every small change costs more.
    if check_botline {
        if win.w_valid & VALID_BOTLINE_AP == 0 {
            win.validate_botline();
        }
        if win.w_botline <= win.buffer().line_count() {
            if win.w_cursor.lnum < win.w_botline {
                check_botline = !enough_below(win, so.get());
            }
            if check_botline {
                let n = if win.lines_concealed() {
                    unconcealed_below(win, so.get())
                } else {
                    ((win.w_cursor.lnum - win.w_botline + 1) as OptInt + so.get()) as int64_t
                };
                if n <= (win.w_view_height + 1) as int64_t {
                    // SAFETY: a live window.
                    unsafe {
                        scroll_cursor_bot(
                            wp,
                            arith::scrolljump_lines(p_sj.get(), win.w_view_height),
                            false,
                        )
                    };
                } else {
                    // SAFETY: a live window.
                    unsafe { scroll_cursor_halfway(wp, false, false) };
                }
            }
        }
    }

    win.w_valid |= VALID_TOPLINE;
    win.w_viewport_invalid = true;
    win.check_anchored_floats();

    // The top line moved, so the window has to be redrawn.
    if win.w_topline != old_topline || win.w_topfill != old_topfill {
        dollar_vcol.set(-1);
        win.redraw_later(UPD_VALID);

        // Without 'smoothscroll' there is nothing for `w_skipcol` to mean.
        if win.w_onebuf_opt.wo_sms == 0 {
            win.reset_skipcol();
        } else if win.w_skipcol != 0 {
            win.redraw_later(UPD_SOME_VALID);
        }

        // `w_skipcol` may have to be set when the cursor is on the top line.
        if win.w_cursor.lnum == win.w_topline {
            win.validate_cursor();
        }
    }

    so.set(save_so);
}

/// Whether the cursor is above the window, or too close to its top for
/// 'scrolloff', or the window shows more filler lines than there is room for.
fn check_topline(win: Win) -> bool {
    if win.w_topline > 1 || win.w_skipcol > 0 {
        // Above the top line: scrolling is always needed. Far below it and
        // with no folding: scrolling down never is.
        if win.w_cursor.lnum < win.w_topline {
            return true;
        }
        if check_top_offset(win) {
            return true;
        }
        if win.w_skipcol > 0 && win.w_cursor.lnum == win.w_topline {
            // Is the cursor's own column visible? Add the columns the
            // top-left marker covers.
            let vcol = win.virtual_vcol(win.cursor());
            // SAFETY: a live window.
            let overlap = unsafe { sms_marker_overlap(win.raw(), -1) };
            if win.w_skipcol + overlap > vcol {
                return true;
            }
        }
    }
    // More filler lines than there is room for.
    win.w_topfill > win.fill_above(win.w_topline)
}

/// Logical lines between the cursor and `w_topline + 'scrolloff'`, counting
/// only lines a decoration does not hide, and stopping once the answer can no
/// longer matter.
fn unconcealed_above(win: Win, halfheight: c_int, so: OptInt) -> int64_t {
    let mut n: int64_t = 0;
    let mut lnum = win.w_cursor.lnum;
    while (lnum as OptInt) < win.w_topline as OptInt + so {
        // Stop at the end of the file, or once we know we are far off.
        if lnum >= win.buffer().line_count() || {
            n += !win.conceals_line(lnum, false) as int64_t;
            n >= halfheight as int64_t
        } {
            break;
        }
        lnum = win.fold_last(lnum) + 1;
    }
    n
}

/// As [`unconcealed_above`], downwards from the cursor to
/// `w_botline - 'scrolloff'`. Upstream stops this one at *more* than the
/// window height rather than at or above it, so the two walks are not
/// symmetric; kept.
fn unconcealed_below(win: Win, so: OptInt) -> int64_t {
    let mut n: int64_t = 0;
    let mut lnum = win.w_cursor.lnum;
    while (lnum as OptInt) >= win.w_botline as OptInt - so {
        if lnum <= 0 || {
            n += !win.conceals_line(lnum - 1, false) as int64_t;
            n > (win.w_view_height + 1) as int64_t
        } {
            break;
        }
        lnum = win.fold_first(lnum).unwrap_or(lnum) - 1;
    }
    n
}

/// Whether there are already 'scrolloff' window lines below the cursor, so
/// that nothing has to be scrolled.
fn enough_below(win: Win, so: OptInt) -> bool {
    if (win.w_cursor.lnum as OptInt) < win.w_botline as OptInt - so && !win.lines_concealed() {
        return true;
    }
    let mut loff = lineoff_T {
        // In a fold, count from its last line.
        lnum: win.fold_last(win.w_cursor.lnum),
        fill: 0,
        height: 0,
    };
    let mut n = win.w_empty_rows + win.w_filler_rows;
    while loff.lnum < win.w_botline && (loff.lnum + 1 < win.w_botline || loff.fill == 0) {
        n += loff.height;
        if n as OptInt >= so {
            break;
        }
        botline_forw(win, &mut loff);
    }
    n as OptInt >= so
}

/// Whether there are fewer than 'scrolloff' visible screen lines above the
/// cursor.
///
/// This asks `get_scrolloff_value()` rather than the [`ScrollOff`] its caller
/// is holding, so during a mouse drag the question "is the cursor too close to
/// the top?" uses the real 'scrolloff' while the scroll it triggers uses the
/// slowed-down one. That is upstream's shape, kept.
fn check_top_offset(win: Win) -> bool {
    let so = win.scrolloff();
    if (win.w_cursor.lnum as int64_t) < win.w_topline as int64_t + so || win.lines_concealed() {
        let mut loff = lineoff_T {
            lnum: win.w_cursor.lnum,
            fill: 0,
            height: 0,
        };
        // The filler lines above the top line are always context.
        let mut n = win.w_topfill;
        while (n as int64_t) < so {
            topline_back(win, &mut loff);
            // Stop once a line above the window has been counted.
            if loff.lnum < win.w_topline || (loff.lnum == win.w_topline && loff.fill > 0) {
                break;
            }
            n += loff.height;
        }
        if (n as int64_t) < so {
            return true;
        }
    }
    false
}

/// Recompute `w_curswant` from the cursor's virtual column.
///
/// # Safety
/// The current window must be valid.
pub unsafe fn update_curswant_force() {
    // SAFETY: `curwin` is set from startup to exit.
    let mut win = unsafe { Win::current() };
    win.validate_virtcol();
    win.w_curswant = win.w_virtcol;
    win.w_set_curswant = false_0;
}

/// [`update_curswant_force`], but only when something asked for it.
///
/// # Safety
/// The current window must be valid.
pub unsafe fn update_curswant() {
    // SAFETY: `curwin` is set from startup to exit.
    if unsafe { Win::current() }.w_set_curswant != 0 {
        // SAFETY: the caller's promise.
        unsafe { update_curswant_force() };
    }
}

/// [`Win::check_cursor_moved`], for the callers still holding a raw window.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn check_cursor_moved(wp: *mut win_T) {
    // SAFETY: the caller's promise.
    unsafe { Win::new(wp) }.check_cursor_moved();
}

impl Win {
    /// Notice that the cursor moved since the last check, and drop the
    /// `w_valid` flags that no longer hold.
    pub(super) fn check_cursor_moved(self) {
        check_cursor_moved_win(self);
    }

    /// A window setting changed in a way that needs the cursor position,
    /// `w_botline` and `w_topline` recomputed and the window redrawn --
    /// 'wrap' or folding, for instance.
    pub(super) fn changed_window_setting(mut self) {
        self.w_lines_valid = 0;
        self.invalidate_above_cursor();
        self.w_valid &= !(VALID_BOTLINE | VALID_BOTLINE_AP | VALID_TOPLINE);
        self.redraw_later(UPD_NOT_VALID);
    }
}

fn check_cursor_moved_win(mut win: Win) {
    if win.w_cursor.lnum != win.w_valid_cursor.lnum {
        win.w_valid &=
            !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL | VALID_CHEIGHT | VALID_CROW | VALID_TOPLINE);
        // Concealed-line visibility toggled.
        if win.is_current()
            && win.w_valid_cursor.lnum > 0
            && win.w_onebuf_opt.wo_cole >= 2
            && !win.conceal_cursor_line()
            && (win.conceals_line(win.w_cursor.lnum - 1, true)
                || win.conceals_line(win.w_valid_cursor.lnum - 1, true))
        {
            win.changed_window_setting();
        }
        win.w_valid_cursor = win.w_cursor;
        win.w_valid_leftcol = win.w_leftcol;
        win.w_valid_skipcol = win.w_skipcol;
        win.w_viewport_invalid = true;
    } else if win.w_skipcol != win.w_valid_skipcol {
        win.w_valid &= !(VALID_WROW
            | VALID_WCOL
            | VALID_VIRTCOL
            | VALID_CHEIGHT
            | VALID_CROW
            | VALID_BOTLINE
            | VALID_BOTLINE_AP);
        win.w_valid_cursor = win.w_cursor;
        win.w_valid_leftcol = win.w_leftcol;
        win.w_valid_skipcol = win.w_skipcol;
    } else if win.w_cursor.col != win.w_valid_cursor.col
        || win.w_leftcol != win.w_valid_leftcol
        || win.w_cursor.coladd != win.w_valid_cursor.coladd
    {
        win.w_valid &= !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL);
        win.w_valid_cursor.col = win.w_cursor.col;
        win.w_valid_leftcol = win.w_leftcol;
        win.w_valid_cursor.coladd = win.w_cursor.coladd;
        win.w_viewport_invalid = true;
    }
}

/// [`Win::changed_window_setting`], for the callers still holding a raw
/// window.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn changed_window_setting(wp: *mut win_T) {
    // SAFETY: the caller's promise.
    unsafe { Win::new(wp) }.changed_window_setting();
}

/// [`changed_window_setting`] for every window of every tab page.
///
/// # Safety
/// The editor's window list must be valid.
pub unsafe fn changed_window_setting_all() {
    let mut tp = first_tabpage.get();
    while !tp.is_null() {
        // The current tab page's windows hang off `firstwin`; a tab page's
        // own list is only filled in when it is left.
        let first = if tp == curtab.get() {
            firstwin.get()
        } else {
            // SAFETY: a live tab page.
            unsafe { (*tp).tp_firstwin }
        };
        // SAFETY: the editor's window list holds live windows.
        let mut win = (!first.is_null()).then(|| unsafe { Win::new(first) });
        while let Some(w) = win {
            w.changed_window_setting();
            win = w.next();
        }
        // SAFETY: a live tab page.
        tp = unsafe { (*tp).tp_next };
    }
}

/// Put the window's top line at `lnum`, approximating `w_botline` rather than
/// recomputing it.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn set_topline(wp: *mut win_T, lnum: linenr_T) {
    // SAFETY: the caller's promise.
    let mut win = unsafe { Win::new(wp) };
    let prev_topline = win.w_topline;
    // Go to the first line of a closed fold.
    let lnum = win.fold_first(lnum).unwrap_or(lnum);
    let shift = lnum - win.w_topline;
    win.w_botline += shift;
    let last = win.buffer().line_count() + 1;
    if win.w_botline > last {
        win.w_botline = last;
    }
    win.w_topline = lnum;
    win.w_topline_was_set = true_0 as ::core::ffi::c_char;
    if lnum != prev_topline {
        // The filler lines are kept when the top line did not change.
        win.w_topfill = 0;
    }
    win.w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_TOPLINE);
    // Not VALID_TOPLINE: 'scrolloff' still has to be checked.
    win.redraw_later(UPD_VALID);
}
