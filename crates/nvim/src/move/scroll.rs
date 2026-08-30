//! Scrolling the window by a count -- `scrolldown()`, `scrollup()` and the
//! clamped forms.
//!
//! These move `w_topline` (and, under `'smoothscroll'`, `w_skipcol`) by a given
//! number of lines without regard to where the cursor is, leaving the cursor
//! correction to the caller.  The `_clamp` pair stops before the cursor would
//! leave the window at all, which is what CTRL-E/CTRL-Y need; [`topline_back`]
//! and [`botline_forw`] step one [`lineoff_T`] at a time over folds and diff
//! filler, and are the shared primitive the `scroll_cursor_*` family walks
//! with.
//!
//! Original: `src/nvim/move.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::*;
use crate::cursor::coladvance;
use crate::drawscreen::{UPD_NOT_VALID, UPD_VALID};
use crate::edit::{cursor_down, cursor_up};
use crate::pos::MAXCOL;
use crate::types::{colnr_T, int64_t, linenr_T};

impl Win {
    /// Put the cursor at virtual column `wcol`, or as close as the line
    /// allows. Answers whether it got there.
    pub(crate) fn coladvance(self, wcol: colnr_T) -> bool {
        // SAFETY: a live window.
        coladvance(self, wcol)
    }
}

/// The window's first and later screen lines' text widths, which every
/// `'smoothscroll'` arm below works in.
pub(super) fn sms_widths(win: Win) -> (c_int, c_int) {
    win.text_widths()
}

/// Whether this window scrolls by screen line rather than by buffer line.
fn do_sms(win: Win) -> bool {
    win.w_onebuf_opt.wo_wrap != 0 && win.w_onebuf_opt.wo_sms != 0
}

impl Win {
    /// Make sure the cursor is in the visible part of the top line after
    /// scrolling with 'smoothscroll'.
    pub(super) fn cursor_correct_sms(mut self) {
        if !do_sms(self) || self.w_cursor.lnum != self.w_topline {
            return;
        }

        let so = self.scrolloff();
        let (width1, width2) = sms_widths(self);
        let mut so_cols = arith::scrolloff_cols(so, width1, width2);
        let space_cols = (self.w_view_height - 1) * width2;
        let size = if so == 0 {
            0
        } else {
            self.line_display_width(self.w_topline)
        };

        if self.w_topline == 1 && self.w_skipcol == 0 {
            // Ignore 'scrolloff' at the top of the buffer.
            so_cols = 0;
        } else if so_cols > (space_cols / 2) as int64_t {
            // Not enough room: put the cursor in the middle.
            so_cols = (space_cols / 2) as int64_t;
        }
        so_cols = arith::fit_scrolloff_cols(so_cols, size, width1, width2);

        let overlap = if self.w_skipcol == 0 {
            0
        } else {
            self.marker_overlap(self.w_view_width - width2)
        };
        // With a non-zero 'scrolloff' the marker overlap does not matter.
        let top = self.w_skipcol as int64_t
            + if so_cols != 0 {
                so_cols
            } else {
                overlap as int64_t
            };
        let bot =
            (self.w_skipcol + width1 + (self.w_view_height - 1) * width2) as int64_t - so_cols;

        self.validate_virtcol();
        let col = arith::visible_sms_col(self.w_virtcol, top, bot, width1, width2);
        if col == self.w_virtcol {
            return;
        }

        self.w_curswant = col;
        let reached = self.coladvance(self.w_curswant);
        // `validate_virtcol()` marked various things valid; moving the cursor
        // has just invalidated them again.
        self.w_valid.clear(
            WinValid::WROW
                | WinValid::WCOL
                | WinValid::CHEIGHT
                | WinValid::CROW
                | WinValid::VIRTCOL,
        );
        if reached || self.w_skipcol == 0 || self.w_cursor.lnum >= self.buffer().line_count() {
            return;
        }
        self.validate_virtcol();
        if self.w_virtcol < self.w_skipcol + overlap {
            // Cursor still not visible: move it to the next line instead.
            self.w_cursor.lnum += 1;
            self.w_cursor.col = 0;
            self.w_cursor.coladd = 0;
            self.w_curswant = 0;
            self.w_valid.clear(WinValid::VIRTCOL);
        }
    }
}

/// [`scroll_redraw_cur`], for the callers still holding a raw count.
///
/// # Safety
/// The current window must be valid.
pub unsafe fn scroll_redraw(up: c_int, count: linenr_T) {
    // SAFETY: `curwin` is set from startup to exit.
    scroll_redraw_cur(unsafe { Win::current() }, up != 0, count);
}

/// Scroll `count` lines up or down, and redraw.
///
/// `win` must be the current window: the cursor corrections below move
/// `curwin`'s cursor.
pub(super) fn scroll_redraw_cur(mut win: Win, up: bool, count: linenr_T) {
    let prev_topline = win.w_topline;
    let prev_skipcol = win.w_skipcol;
    let prev_topfill = win.w_topfill;
    let prev_lnum = win.w_cursor.lnum;

    let moved = if up {
        win.scrollup(count, true)
    } else {
        win.scrolldown(count, true)
    };

    if win.scrolloff() > 0 {
        // Adjust the cursor position for 'scrolloff'. Mark `w_topline` valid,
        // otherwise the screen jumps back at the end of the file.
        win.cursor_correct();
        win.check_cursor_moved();
        win.w_valid |= WinValid::TOPLINE;

        // If we ended up back where we were, at least move the cursor, or we
        // get stuck at one position. Don't move it up when the first line of
        // the buffer is already on screen.
        while win.w_topline == prev_topline
            && win.w_skipcol == prev_skipcol
            && win.w_topfill == prev_topfill
        {
            if up {
                // SAFETY: the caller's promise -- `win` is `curwin`.
                if win.w_cursor.lnum > prev_lnum || unsafe { cursor_down(1, false) }.is_err() {
                    break;
                }
            } else if win.w_cursor.lnum < prev_lnum || prev_topline == 1 {
                break;
            } else {
                // SAFETY: the caller's promise -- `win` is `curwin`.
                if unsafe { cursor_up(1, false) }.is_err() {
                    break;
                }
            }
            win.check_cursor_moved();
            win.w_valid |= WinValid::TOPLINE;
        }
    }

    if moved {
        win.w_viewport_invalid = true;
    }
    win.cursor_correct_sms();
    if win.w_cursor.lnum != prev_lnum {
        win.coladvance(win.w_curswant);
    }
    win.redraw_later(UPD_VALID);
}

/// [`Win::scrolldown`], for the callers still holding a raw window.
pub fn scrolldown(wp: Win, line_count: linenr_T, byfold: bool) -> bool {
    wp.scrolldown(line_count, byfold)
}

/// [`Win::scrollup`], for the callers still holding a raw window.
pub fn scrollup(wp: Win, line_count: linenr_T, byfold: bool) -> bool {
    wp.scrollup(line_count, byfold)
}

impl Win {
    /// Scroll the window down by `line_count` logical lines -- CTRL-Y.
    /// `byfold` counts a closed fold as one line. Answers whether the cursor
    /// had to be moved.
    pub(super) fn scrolldown(mut self, line_count: linenr_T, byfold: bool) -> bool {
        // Total screen lines scrolled, which is what the cursor row moves by.
        let mut done = 0;
        let do_sms = do_sms(self);
        let (width1, width2) = if do_sms { sms_widths(self) } else { (0, 0) };

        // Make sure `w_topline` is at the first of a sequence of folded lines.
        self.w_topline = self.fold_first(self.w_topline).unwrap_or(self.w_topline);
        // `w_wrow` has to be valid.
        self.validate_cursor();

        let mut todo = line_count as c_int;
        while todo > 0 {
            let can_fill = self.w_topfill < self.w_view_height - 1
                && self.w_topfill < self.fill_above(self.w_topline);
            // At the very top there is nothing left to scroll to.
            if self.w_topline == 1 && !can_fill && (!do_sms || self.w_skipcol < width1) {
                break;
            }
            if do_sms && self.w_skipcol >= width1 {
                // Scroll one screen line down.
                self.w_skipcol = arith::skipcol_line_back(self.w_skipcol, width1, width2);
                self.redraw_later(UPD_NOT_VALID);
                done += 1;
            } else if can_fill {
                self.w_topfill += 1;
                done += 1;
            } else {
                // Scroll one text line down.
                self.w_topline -= 1;
                self.w_skipcol = 0;
                self.w_topfill = 0;
                if let Some(first) = self.fold_first(self.w_topline) {
                    // A sequence of folded lines counts as one logical line.
                    done += !self.conceals_line(first - 1, false) as c_int;
                    let span = self.w_topline - first;
                    if !byfold {
                        todo -= span - 1;
                    }
                    self.w_botline -= span;
                    self.w_topline = first;
                } else if self.conceals_line(self.w_topline - 1, false) {
                    todo += 1;
                } else if do_sms {
                    let size = self.line_display_width(self.w_topline);
                    // Upstream redraws for any line taller than one screen
                    // line, which is not quite "`w_skipcol` ended up nonzero"
                    // in a window with no room for text at all.
                    if size > width1 {
                        self.redraw_later(UPD_NOT_VALID);
                    }
                    self.w_skipcol = arith::skipcol_showing_last(size, width1, width2);
                    done += 1;
                } else {
                    done += self.plines_nofill(self.w_topline, true);
                }
            }
            // Approximate `w_botline`.
            self.w_botline -= 1;
            self.invalidate_botline();
            todo -= 1;
        }

        // Adjust for concealed lines above `w_topline`.
        while self.w_topline > 1 && self.conceals_line(self.w_topline - 2, false) {
            self.w_topline -= 1;
            self.w_topline = self.fold_first(self.w_topline).unwrap_or(self.w_topline);
        }

        // Keep `w_wrow` and `w_cline_row` up to date.
        self.w_wrow += done;
        self.w_cline_row += done;
        if self.w_cursor.lnum == self.w_topline {
            self.w_cline_row = 0;
        }
        self.check_topfill(true);

        // Compute the row of the last screen line of the cursor line, and move
        // the cursor onto the displayed part of the window.
        let mut wrow = self.w_wrow;
        if self.w_onebuf_opt.wo_wrap != 0 && self.w_view_width != 0 {
            self.validate_virtcol();
            self.validate_cheight();
            wrow += self.w_cline_height - 1 - self.w_virtcol / self.w_view_width;
        }
        let mut moved = false;
        while wrow >= self.w_view_height && self.w_cursor.lnum > 1 {
            if let Some(first) = self.fold_first(self.w_cursor.lnum) {
                wrow -= !self.conceals_line(self.w_cursor.lnum - 1, false) as c_int;
                self.w_cursor.lnum = (first - 1).max(1);
            } else {
                let lnum = self.w_cursor.lnum;
                self.w_cursor.lnum = lnum - 1;
                wrow -= self.plines(lnum, true);
            }
            self.w_valid.clear(
                WinValid::WROW
                    | WinValid::WCOL
                    | WinValid::CHEIGHT
                    | WinValid::CROW
                    | WinValid::VIRTCOL,
            );
            moved = true;
        }
        if moved {
            // Move the cursor to the first line of a closed fold.
            self.fold_adjust_cursor();
            self.coladvance(self.w_curswant);
        }
        let topline = self.w_topline;
        self.w_cursor.lnum = self.w_cursor.lnum.max(topline);

        moved
    }

    /// Scroll the window up by `line_count` logical lines -- CTRL-E.
    /// `byfold` counts a closed fold as one line. Answers whether the visible
    /// range changed.
    pub(super) fn scrollup(mut self, line_count: linenr_T, byfold: bool) -> bool {
        let topline = self.w_topline;
        let botline = self.w_botline;
        let do_sms = do_sms(self);

        if do_sms || (byfold && self.lines_concealed()) || self.may_fill() {
            let (width1, width2) = sms_widths(self);
            let mut size = if do_sms {
                self.line_display_width(self.w_topline)
            } else {
                0
            };
            let prev_skipcol = self.w_skipcol;

            // 'diff': consume `w_topfill` first. 'smoothscroll': raise
            // `w_skipcol` until it goes past the end of the line, then advance
            // to the next one. Folding: each sequence of folded lines counts
            // as one logical line.
            let mut todo = line_count as c_int;
            while todo > 0 {
                todo += self.conceals_line(self.w_topline - 1, false) as c_int;
                if self.w_topfill > 0 {
                    self.w_topfill -= 1;
                } else {
                    // For a closed fold, go to the last line in the fold.
                    let mut lnum = if byfold {
                        self.fold_last(self.w_topline)
                    } else {
                        self.w_topline
                    };
                    if lnum == self.w_topline && do_sms {
                        let add = if self.w_skipcol > 0 { width2 } else { width1 };
                        self.w_skipcol += add;
                        if self.w_skipcol >= size {
                            if lnum == self.buffer().line_count() {
                                // At the last screen line: can't scroll on.
                                self.w_skipcol -= add;
                                break;
                            }
                            lnum += 1;
                        }
                    } else {
                        if lnum >= self.buffer().line_count() {
                            break;
                        }
                        lnum += 1;
                    }

                    if lnum > self.w_topline {
                        // Approximate `w_botline`.
                        self.w_botline += lnum - self.w_topline;
                        self.w_topline = lnum;
                        self.w_topfill = self.fill_above(lnum);
                        self.w_skipcol = 0;
                        if todo > 1 && do_sms {
                            size = self.line_display_width(self.w_topline);
                        }
                    }
                }
                todo -= 1;
            }

            if prev_skipcol > 0 || self.w_skipcol > 0 {
                // More has to be redrawn: the new top line's remembered
                // `wl_size` may now be wrong.
                self.redraw_later(UPD_NOT_VALID);
            }
        } else {
            self.w_topline += line_count;
            // Approximate `w_botline`.
            self.w_botline += line_count;
        }

        let last = self.buffer().line_count();
        self.w_topline = self.w_topline.min(last);
        self.w_botline = self.w_botline.min(last + 1);

        self.check_topfill(false);

        // Make sure `w_topline` is at the first of a sequence of folded lines.
        self.w_topline = self.fold_first(self.w_topline).unwrap_or(self.w_topline);

        self.w_valid = self
            .w_valid
            .without(WinValid::WROW | WinValid::CROW | WinValid::BOTLINE);
        if self.w_cursor.lnum < self.w_topline {
            self.w_cursor.lnum = self.w_topline;
            self.w_valid.clear(
                WinValid::WROW
                    | WinValid::WCOL
                    | WinValid::CHEIGHT
                    | WinValid::CROW
                    | WinValid::VIRTCOL,
            );
            self.coladvance(self.w_curswant);
        }

        topline != self.w_topline || botline != self.w_botline
    }
}

/// Called after changing the cursor column: make sure `w_skipcol` is right for
/// 'smoothscroll'.
///
/// # Safety
/// The current window must be valid.
pub unsafe fn adjust_skipcol() {
    // SAFETY: `curwin` is set from startup to exit.
    let mut win = unsafe { Win::current() };
    if !do_sms(win) || win.w_cursor.lnum != win.w_topline {
        return;
    }

    let (width1, width2) = sms_widths(win);
    if width1 <= 0 {
        // No text will be displayed.
        return;
    }
    let scrolloff_cols = arith::scrolloff_cols(win.scrolloff(), width1, width2);

    win.validate_cheight();
    // `w_cline_height` may be capped at the window height, so check there
    // aren't actually more lines.
    if win.w_cline_height == win.w_view_height
        && win.plines(win.w_cursor.lnum, false) <= win.w_view_height
    {
        // The line just fits in the window: don't scroll.
        win.reset_skipcol();
        return;
    }

    win.validate_virtcol();
    let overlap = win.marker_overlap(win.w_view_width - width2);
    let mut scrolled = false;
    while win.w_skipcol > 0
        && (win.w_virtcol as int64_t) < (win.w_skipcol + overlap) as int64_t + scrolloff_cols
    {
        // Scroll one screen line down.
        win.w_skipcol = arith::skipcol_line_back(win.w_skipcol, width1, width2);
        scrolled = true;
    }
    if scrolled {
        win.validate_virtcol();
        win.redraw_later(UPD_NOT_VALID);
        // Don't scroll in the other direction now.
        return;
    }

    // The line's width is only needed to wind the 'scrolloff' columns back.
    let size = if scrolloff_cols > 0 {
        win.line_display_width(win.w_topline)
    } else {
        0
    };
    let row = arith::sms_cursor_row(
        win.w_virtcol,
        scrolloff_cols,
        win.w_skipcol,
        size,
        width1,
        width2,
    );
    if row >= win.w_view_height {
        let mut row = row;
        if win.w_skipcol == 0 {
            win.w_skipcol += width1;
            row -= 1;
        }
        if row >= win.w_view_height {
            win.w_skipcol += (row - win.w_view_height) * width2;
        }
        win.redraw_later(UPD_NOT_VALID);
    }
}

/// [`Win::check_topfill`], for the callers still holding a raw window.
pub fn check_topfill(wp: Win, down: bool) {
    wp.check_topfill(down);
}

impl Win {
    /// Don't end up with more filler lines in the window than fit. `down`
    /// scrolls down when there is not enough space.
    pub(crate) fn check_topfill(mut self, down: bool) {
        if self.w_topfill > 0 {
            let n = self.plines_nofill(self.w_topline, true);
            if self.w_topfill + n > self.w_view_height {
                if down && self.w_topline > 1 {
                    self.w_topline -= 1;
                    self.w_topfill = 0;
                } else {
                    self.w_topfill = (self.w_view_height - n).max(0);
                }
            }
        }
        self.check_anchored_floats();
    }
}

/// Scroll the screen one line down, unless that would move the cursor off it.
///
/// # Safety
/// The current window must be valid.
pub unsafe fn scrolldown_clamp() {
    // SAFETY: `curwin` is set from startup to exit.
    let mut win = unsafe { Win::current() };
    let can_fill = win.w_topfill < win.fill_above(win.w_topline);
    if win.w_topline <= 1 && !can_fill {
        return;
    }

    // `w_wrow` has to be valid.
    win.validate_cursor();

    // Compute the row of the last screen line of the cursor line and make sure
    // it does not go off the screen, nor past 'scrolloff' lines from its end.
    let mut end_row = win.w_wrow;
    if can_fill {
        end_row += 1;
    } else {
        end_row += win.plines_nofill(win.w_topline - 1, true);
    }
    if win.w_onebuf_opt.wo_wrap != 0 && win.w_view_width != 0 {
        win.validate_cheight();
        win.validate_virtcol();
        end_row += win.w_cline_height - 1 - win.w_virtcol / win.w_view_width;
    }
    if (end_row as int64_t) < win.w_view_height as int64_t - win.scrolloff() {
        if can_fill {
            win.w_topfill += 1;
            win.check_topfill(true);
        } else {
            win.w_topline -= 1;
            win.w_topfill = 0;
        }
        win.w_topline = win.fold_first(win.w_topline).unwrap_or(win.w_topline);
        // Approximate `w_botline`.
        win.w_botline -= 1;
        win.w_valid = win
            .w_valid
            .without(WinValid::WROW | WinValid::CROW | WinValid::BOTLINE);
    }
}

/// Scroll the screen one line up, unless that would move the cursor off it.
///
/// # Safety
/// The current window must be valid.
pub unsafe fn scrollup_clamp() {
    // SAFETY: `curwin` is set from startup to exit.
    let mut win = unsafe { Win::current() };
    if win.w_topline == win.buffer().line_count() && win.w_topfill == 0 {
        return;
    }

    // `w_wrow` has to be valid.
    win.validate_cursor();

    // Compute the row of the first screen line of the cursor line and make
    // sure it does not go off the screen, nor before 'scrolloff' lines from
    // its start.
    let mut start_row = win.w_wrow - win.plines_nofill(win.w_topline, true) - win.w_topfill;
    if win.w_onebuf_opt.wo_wrap != 0 && win.w_view_width != 0 {
        win.validate_virtcol();
        start_row -= win.w_virtcol / win.w_view_width;
    }
    if start_row as int64_t >= win.scrolloff() {
        if win.w_topfill > 0 {
            win.w_topfill -= 1;
        } else {
            win.w_topline = win.fold_last(win.w_topline);
            win.w_topline += 1;
        }
        // Approximate `w_botline`.
        win.w_botline += 1;
        win.w_valid = win
            .w_valid
            .without(WinValid::WROW | WinValid::CROW | WinValid::BOTLINE);
    }
}

/// Add one line above `lp.lnum`: a filler line, a closed fold or a (wrapped)
/// text line. Uses and sets `lp.fill`, and answers the height of the added
/// line in `lp.height`. Lines above the first one are incredibly high --
/// `MAXCOL`. `winheight` limits a line's height to the window's.
pub(super) fn topline_back_winheight(win: Win, lp: &mut lineoff_T, winheight: bool) {
    if lp.fill < win.fill_above(lp.lnum) {
        // Add a filler line.
        lp.fill += 1;
        lp.height = 1;
    } else {
        lp.lnum -= 1;
        lp.fill = 0;
        if lp.lnum < 1 {
            lp.height = MAXCOL as c_int;
        } else if let Some(first) = win.fold_first(lp.lnum) {
            // Add a closed fold, unless it is concealed.
            lp.lnum = first;
            lp.height = !win.conceals_line(lp.lnum - 1, false) as c_int;
        } else {
            lp.height = win.plines_nofill(lp.lnum, winheight);
        }
    }
}

/// [`topline_back_winheight`], capping a line's height at the window's.
pub(super) fn topline_back(win: Win, lp: &mut lineoff_T) {
    topline_back_winheight(win, lp, true);
}

/// Add one line below `lp.lnum`, as [`topline_back_winheight`] adds one above.
/// Lines below the last one are incredibly high.
pub(super) fn botline_forw(win: Win, lp: &mut lineoff_T) {
    if lp.fill < win.fill_above(lp.lnum + 1) {
        // Add a filler line.
        lp.fill += 1;
        lp.height = 1;
    } else {
        lp.lnum += 1;
        lp.fill = 0;
        debug_assert!(!win.buffer().raw().is_null(), "wp->w_buffer != 0");
        if lp.lnum > win.buffer().line_count() {
            lp.height = MAXCOL as c_int;
            return;
        }
        let (folded, _, last) = win.fold_span(lp.lnum);
        if folded {
            // Add a closed fold, unless it is concealed.
            lp.lnum = last;
            lp.height = !win.conceals_line(lp.lnum - 1, false) as c_int;
        } else {
            lp.height = win.plines_nofill(lp.lnum, true);
        }
    }
}
