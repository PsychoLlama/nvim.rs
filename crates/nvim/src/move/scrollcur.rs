//! Choosing a topline for a cursor that has moved -- the `scroll_cursor_*`
//! family and `cursor_correct()`.
//!
//! [`scroll_cursor_top`], [`scroll_cursor_bot`] and
//! [`scroll_cursor_halfway`] are the three answers `update_topline` picks
//! between when the cursor has left the visible range: put its line at the top
//! (honouring `'scrolloff'`), at the bottom (`scroll_cursor_bot` also decides
//! whether scrolling or redrawing is cheaper), or in the middle.
//! [`cursor_correct`] is the reverse -- the window stays put and the cursor
//! moves to satisfy `'scrolloff'`.
//!
//! Original: `src/nvim/move.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::*;
use crate::drawscreen::UPD_NOT_VALID;
use crate::main::mouse_dragging;
use crate::pos::MAXCOL;
use crate::search::{BACKWARD, FORWARD};
use crate::types::{Direction, colnr_T, int64_t, linenr_T};

/// The 'scrolloff' the `scroll_cursor_*` family works with: a mouse drag
/// slows scrolling down by standing in for the option.
fn scrolloff_or_drag(win: Win) -> int64_t {
    if mouse_dragging.get() > 0 {
        (mouse_dragging.get() - 1) as int64_t
    } else {
        win.scrolloff()
    }
}

/// An empty [`lineoff_T`] at `lnum`, which the walks below fill in.
fn lineoff_at(lnum: linenr_T) -> lineoff_T {
    lineoff_T {
        lnum,
        fill: 0,
        height: 0,
    }
}

/// [`Win::scroll_cursor_top`], for the callers still holding a raw window.
pub fn scroll_cursor_top(wp: Win, min_scroll: c_int, always: c_int) {
    wp.scroll_cursor_top(min_scroll, always != 0);
}

impl Win {
    /// Recompute `w_topline` to put the cursor at the top of the window,
    /// scrolling at least `min_scroll` lines. `always` sets `w_topline` even
    /// when that scrolls the other way, which is what `zt` wants.
    pub(super) fn scroll_cursor_top(mut self, min_scroll: c_int, always: bool) {
        let old_topline = self.w_topline;
        let old_skipcol = self.w_skipcol;
        let old_topfill = self.w_topfill;
        let off = scrolloff_or_drag(self);

        // Decrease `w_topline` until it has become 1, or (part of) the cursor
        // line is moved off the screen, or we have moved at least 'scrolljump'
        // lines with at least 'scrolloff' lines above and below the cursor.
        self.validate_cheight();
        let mut scrolled = 0;
        // Includes the filler lines above.
        let mut used = self.w_cline_height;
        if self.w_cursor.lnum < self.w_topline {
            scrolled = used;
        }

        // Just above and just below the displayed lines.
        let (folded, first, last) = self.fold_span(self.w_cursor.lnum);
        let (mut top, mut bot) = if folded {
            (first - 1, last + 1)
        } else {
            (self.w_cursor.lnum - 1, self.w_cursor.lnum + 1)
        };
        let mut new_topline = top + 1;

        // `used` already counts the filler lines above, so hide them from it
        // by adding them to `extra` instead.
        let mut extra = self.fill_above(self.w_cursor.lnum);

        // Do the lines from `top` to `bot` fit in the window? If they do, set
        // `new_topline` and take in more lines.
        while top > 0 {
            let i = self.plines_nofill(top, true);
            top = self.fold_first(top).unwrap_or(top);
            if top < self.w_topline {
                scrolled += i;
            }

            // If scrolling is needed, scroll at least 'scrolljump' lines.
            if (new_topline >= self.w_topline || scrolled > min_scroll) && extra as int64_t >= off {
                break;
            }

            used += i;
            if (extra + i) as int64_t <= off && bot < self.buffer().line_count() {
                let (height, next, _) = self.plines_full(bot, true, true);
                bot = next;
                used += height;
            }
            if used > self.w_view_height {
                break;
            }

            extra += i;
            new_topline = top;
            top -= 1;
            bot += 1;
        }

        // Without enough space, put the cursor in the middle instead; that way
        // "k" and "j" land in the same place in a small window.
        if used > self.w_view_height {
            self.scroll_cursor_halfway(false, false);
            return;
        }

        // Unless `always`, only lower `w_topline`: a higher value can happen
        // with wrapping lines.
        if new_topline < self.w_topline || always {
            self.w_topline = new_topline;
        }
        let cursor_lnum = self.w_cursor.lnum;
        self.w_topline = self.w_topline.min(cursor_lnum);
        self.w_topfill = self.fill_above(self.w_topline);
        if self.w_topfill > 0 && extra as int64_t > off {
            self.w_topfill -= extra - off as c_int;
            self.w_topfill = self.w_topfill.max(0);
        }
        self.check_topfill(false);
        if self.w_topline != old_topline {
            self.reset_skipcol();
        } else if self.w_topline == self.w_cursor.lnum {
            self.validate_virtcol();
            if self.w_skipcol >= self.w_virtcol {
                // TODO(vim): when the line doesn't fit, optimise `w_skipcol`
                // rather than zeroing it.
                self.reset_skipcol();
            }
        }
        if self.w_topline != old_topline
            || self.w_skipcol != old_skipcol
            || self.w_topfill != old_topfill
        {
            self.w_valid
                .clear(WinValid::WROW | WinValid::CROW | WinValid::BOTLINE | WinValid::BOTLINE_AP);
        }
        self.w_valid |= WinValid::TOPLINE;
        self.w_viewport_invalid = true;
    }
}

/// [`Win::set_empty_rows`], for the callers still holding a raw window.
pub fn set_empty_rows(wp: Win, used: c_int) {
    wp.set_empty_rows(used);
}

impl Win {
    /// Record how much of the window is left over once `used` screen lines of
    /// text have been drawn: `w_empty_rows` below the last line, and
    /// `w_filler_rows` of them claimed by 'diff' filler.
    pub(super) fn set_empty_rows(mut self, used: c_int) {
        self.w_filler_rows = 0;
        if used == 0 {
            // A single line that does not fit.
            self.w_empty_rows = 0;
            return;
        }
        self.w_empty_rows = self.w_view_height - used;
        if self.w_botline <= self.buffer().line_count() {
            self.w_filler_rows = self.fill_above(self.w_botline);
            if self.w_empty_rows > self.w_filler_rows {
                self.w_empty_rows -= self.w_filler_rows;
            } else {
                self.w_filler_rows = self.w_empty_rows;
                self.w_empty_rows = 0;
            }
        }
    }
}

/// [`Win::scroll_cursor_bot`], for the callers still holding a raw window.
pub fn scroll_cursor_bot(wp: Win, min_scroll: c_int, set_topbot: bool) {
    wp.scroll_cursor_bot(min_scroll, set_topbot);
}

impl Win {
    /// Recompute `w_topline` to put the cursor at the bottom of the window,
    /// scrolling at least `min_scroll` lines. `set_topbot` sets `w_topline`
    /// and `w_botline` from the cursor line first, which is what `zb` wants.
    pub(super) fn scroll_cursor_bot(mut self, min_scroll: c_int, set_topbot: bool) {
        let old_topline = self.w_topline;
        let old_skipcol = self.w_skipcol;
        let old_topfill = self.w_topfill;
        let old_botline = self.w_botline;
        let old_valid = self.w_valid;
        let old_empty_rows = self.w_empty_rows;
        let cursor_lnum = self.w_cursor.lnum;
        let do_sms = self.w_onebuf_opt.wo_wrap != 0 && self.w_onebuf_opt.wo_sms != 0;

        if set_topbot {
            self.fill_from_bottom(cursor_lnum, do_sms);
            if self.w_topline != old_topline
                || self.w_topfill != old_topfill
                || self.w_skipcol != old_skipcol
                || self.w_skipcol != 0
            {
                self.w_valid.clear(WinValid::WROW | WinValid::CROW);
                if self.w_skipcol != old_skipcol {
                    self.redraw_later(UPD_NOT_VALID);
                } else {
                    self.reset_skipcol();
                }
            }
        } else {
            self.validate_botline();
        }

        let (used, scrolled) = self.count_below_window(cursor_lnum, min_scroll, do_sms);
        let line_count = self.lines_to_scroll(used, scrolled);

        // Scroll up when the cursor is a little off the bottom of the screen;
        // otherwise put it at half the screen.
        if line_count >= self.w_view_height as linenr_T && line_count > min_scroll as linenr_T {
            self.scroll_cursor_halfway(false, true);
        } else if line_count > 0 {
            if do_sms {
                // TODO(vim):
                self.scrollup(scrolled as linenr_T, true);
            } else {
                self.scrollup(line_count, true);
            }
        }

        // When `w_topline` did not change, restore the `w_botline` and
        // `w_empty_rows` we changed; when it did, `update_screen()` sets them.
        if self.w_topline == old_topline && self.w_skipcol == old_skipcol && set_topbot {
            self.w_botline = old_botline;
            self.w_empty_rows = old_empty_rows;
            self.w_valid = old_valid;
        }
        self.w_valid |= WinValid::TOPLINE;
        self.w_viewport_invalid = true;

        // Make sure the cursor is still visible after `zb` adjusted `w_skipcol`.
        if set_topbot {
            self.cursor_correct_sms();
        }
    }

    /// `zb`'s first half: fill the window upwards from the cursor line, so
    /// that it is the last one shown.
    fn fill_from_bottom(mut self, cursor_lnum: linenr_T, do_sms: bool) {
        let mut used = 0;
        let last = self.fold_last(cursor_lnum);
        self.w_botline = last + 1;
        let mut loff = lineoff_at(last + 1);
        loop {
            topline_back_winheight(self, &mut loff, false);
            if loff.height == MAXCOL as c_int {
                break;
            }
            if used + loff.height > self.w_view_height {
                // With 'smoothscroll' and 'wrap' the line above is too long to
                // show whole, so show just a part of it.
                if do_sms && used < self.w_view_height {
                    let plines_offset = used + loff.height - self.w_view_height;
                    used = self.w_view_height;
                    self.w_topfill = loff.fill;
                    self.w_topline = loff.lnum;
                    self.w_skipcol = self.skipcol_from_plines(plines_offset);
                }
                break;
            }
            self.w_topfill = loff.fill;
            self.w_topline = loff.lnum;
            used += loff.height;
        }

        self.set_empty_rows(used);
        self.w_valid |= WinValid::BOTLINE | WinValid::BOTLINE_AP;
    }

    /// Walk outwards from the cursor line until enough context is found, and
    /// answer the screen lines used and the ones that are below the window and
    /// so would have to be scrolled into view.
    fn count_below_window(
        self,
        cursor_lnum: linenr_T,
        min_scroll: c_int,
        do_sms: bool,
    ) -> (c_int, c_int) {
        // The cursor line's own screen lines are always used.
        let mut used = self.plines_nofill(cursor_lnum, true);

        // On or below `w_botline` we scroll by at least the cursor line's
        // height. Correct for the empty rows, which really belong to it.
        let mut scrolled = 0;
        if cursor_lnum >= self.w_botline {
            scrolled = used;
            if cursor_lnum == self.w_botline {
                scrolled -= self.w_empty_rows;
            }
            if do_sms {
                // Screen lines the top line occupies. When that is more than
                // the whole window, the clipped ones have to be scrolled past
                // before any other line can be.
                let mut top_plines = self.plines_nofill(self.w_topline, false);
                let (width1, width2) = sms_widths(self);
                if width1 > 0 {
                    top_plines -= arith::top_skipped_plines(self.w_skipcol, width1, width2);
                    if top_plines > self.w_view_height {
                        scrolled += top_plines - self.w_view_height;
                    }
                }
            }
        }

        // Stop counting lines to scroll when we hit the start of the file, or
        // scrolled nothing or at least 'scrolljump' lines, and found
        // 'scrolloff' lines below the cursor, and counted the lines between
        // `w_botline` and the cursor.
        let (folded, first, last) = self.fold_span(cursor_lnum);
        let (mut loff, mut boff) = if folded {
            (lineoff_at(first), lineoff_at(last))
        } else {
            (lineoff_at(cursor_lnum), lineoff_at(cursor_lnum))
        };
        let fill_below_window = self.fill_above(self.w_botline) - self.w_filler_rows;

        let mut extra = 0;
        let so = scrolloff_or_drag(self);
        while loff.lnum > 1 {
            if ((scrolled <= 0 || scrolled >= min_scroll) && extra as int64_t >= so
                || boff.lnum + 1 > self.buffer().line_count())
                && loff.lnum <= self.w_botline
                && (loff.lnum < self.w_botline || loff.fill >= fill_below_window)
            {
                break;
            }

            // Add one line above.
            topline_back(self, &mut loff);
            if loff.height == MAXCOL as c_int {
                used = MAXCOL as c_int;
            } else {
                used += loff.height;
            }
            if used > self.w_view_height {
                break;
            }
            if loff.lnum >= self.w_botline
                && (loff.lnum > self.w_botline || loff.fill <= fill_below_window)
            {
                // Count the screen lines that are below the window.
                scrolled += loff.height;
                if loff.lnum == self.w_botline && loff.fill == 0 {
                    scrolled -= self.w_empty_rows;
                }
            }

            if boff.lnum >= self.buffer().line_count() {
                continue;
            }
            // Add one line below.
            botline_forw(self, &mut boff);
            debug_assert!(boff.height != MAXCOL as c_int, "boff.height != MAXCOL");
            used += boff.height;
            if used > self.w_view_height {
                break;
            }
            if (extra as int64_t) < so || scrolled < min_scroll {
                extra += boff.height;
                if boff.lnum >= self.w_botline
                    || boff.lnum + 1 == self.w_botline && boff.fill > self.w_filler_rows
                {
                    // Count the screen lines that are below the window.
                    scrolled += boff.height;
                    if boff.lnum == self.w_botline && boff.fill == 0 {
                        scrolled -= self.w_empty_rows;
                    }
                }
            }
        }
        (used, scrolled)
    }

    /// Turn the screen lines below the window into the number of logical lines
    /// to scroll by -- 0 for none, 9999 for "more than the window holds".
    fn lines_to_scroll(self, used: c_int, scrolled: c_int) -> linenr_T {
        if scrolled <= 0 {
            // `w_empty_rows` is larger: no need to scroll.
            return 0;
        }
        if used > self.w_view_height {
            // More than a screenful: don't scroll, redraw.
            return used as linenr_T;
        }
        // Scroll the minimal number of lines.
        let mut line_count = 0;
        let mut boff = lineoff_T {
            lnum: self.w_topline - 1,
            fill: self.w_topfill,
            height: 0,
        };
        let mut i = 0;
        while i < scrolled && boff.lnum < self.w_botline {
            botline_forw(self, &mut boff);
            i += boff.height;
            line_count += 1;
        }
        if i < scrolled {
            // Below `w_botline`: don't scroll.
            return 9999;
        }
        line_count
    }
}

/// [`Win::scroll_cursor_halfway`], for the callers still holding a raw window.
pub fn scroll_cursor_halfway(wp: Win, atend: bool, prefer_above: bool) {
    wp.scroll_cursor_halfway(atend, prefer_above);
}

impl Win {
    /// Recompute `w_topline` to put the cursor halfway across the window.
    /// `atend` also puts it halfway to the end of the file.
    pub(super) fn scroll_cursor_halfway(mut self, atend: bool, prefer_above: bool) {
        let old_topline = self.w_topline;
        let (folded, first, last) = self.fold_span(self.w_cursor.lnum);
        let (mut loff, mut boff) = if folded {
            (lineoff_at(first), lineoff_at(last))
        } else {
            (
                lineoff_at(self.w_cursor.lnum),
                lineoff_at(self.w_cursor.lnum),
            )
        };
        let mut used = self.plines_nofill(loff.lnum, true);
        let mut topline = loff.lnum;
        let mut skipcol: colnr_T = 0;

        let do_sms = self.w_onebuf_opt.wo_wrap != 0 && self.w_onebuf_opt.wo_sms != 0;
        // Only read under `do_sms`, which is also the only arm that sets it.
        let mut want_height = 0;
        if do_sms {
            if atend {
                want_height = (self.w_view_height - used) / 2;
                used = 0;
            } else {
                want_height = self.w_view_height;
            }
        }

        let mut topfill = 0;
        while topline > 1 {
            if do_sms {
                // With 'smoothscroll' we can scroll to the exact point where
                // the cursor is halfway down the screen.
                topline_back_winheight(self, &mut loff, false);
                if loff.height == MAXCOL as c_int {
                    break;
                }
                used += loff.height;
                if !atend && boff.lnum < self.buffer().line_count() {
                    botline_forw(self, &mut boff);
                    used += boff.height;
                }
                if used > want_height {
                    if used - loff.height < want_height {
                        topline = loff.lnum;
                        topfill = loff.fill;
                        skipcol = self.skipcol_from_plines(used - want_height);
                    }
                    break;
                }
                topline = loff.lnum;
                topfill = loff.fill;
                continue;
            }

            // Without 'smoothscroll' we have to find how many lines to scroll
            // down to roughly fit the cursor, which may not be exactly in the
            // middle when a line is taller than one screen line.
            //
            // `prefer_above` decides whether a line above or below goes in
            // first; the two rounds exist only to avoid duplicating the code.
            let mut done = false;
            let mut above = 0;
            let mut below = 0;
            for round in 1..=2 {
                let add_below = if prefer_above {
                    round == 2 && below < above
                } else {
                    round == 1 && below <= above
                };
                if add_below {
                    if boff.lnum < self.buffer().line_count() {
                        botline_forw(self, &mut boff);
                        used += boff.height;
                        if used > self.w_view_height {
                            done = true;
                            break;
                        }
                        below += boff.height;
                    } else {
                        // Count a `~` line.
                        below += 1;
                        if atend {
                            used += 1;
                        }
                    }
                }

                // Upstream only ever adds a line above in round 1, whichever
                // way `prefer_above` points; only the comparison differs.
                let add_above = round == 1
                    && if prefer_above {
                        below >= above
                    } else {
                        below > above
                    };
                if add_above {
                    topline_back(self, &mut loff);
                    if loff.height == MAXCOL as c_int {
                        used = MAXCOL as c_int;
                    } else {
                        used += loff.height;
                    }
                    if used > self.w_view_height {
                        done = true;
                        break;
                    }
                    above += loff.height;
                    topline = loff.lnum;
                    topfill = loff.fill;
                }
            }
            if done {
                break;
            }
        }

        // A folded top line writes `w_topline` itself; only an unfolded one
        // goes through the `skipcol` bookkeeping below.
        if let Some(first) = self.fold_first(topline) {
            self.w_topline = first;
        } else if self.w_topline != topline || skipcol != 0 || self.w_skipcol != 0 {
            self.w_topline = topline;
            if skipcol != 0 {
                self.w_skipcol = skipcol;
                self.redraw_later(UPD_NOT_VALID);
            } else if do_sms {
                self.reset_skipcol();
            }
        }
        self.w_topfill = topfill;
        if old_topline > self.w_topline + self.w_view_height as linenr_T {
            self.w_botfill = false;
        }
        self.check_topfill(false);
        self.w_valid = self
            .w_valid
            .without(WinValid::WROW | WinValid::CROW | WinValid::BOTLINE | WinValid::BOTLINE_AP);
        self.w_valid |= WinValid::TOPLINE;
    }
}

/// [`Win::cursor_correct`], for the callers still holding a raw window.
pub fn cursor_correct(wp: Win) {
    wp.cursor_correct();
}

impl Win {
    /// Move the cursor so that it is at least 'scrolloff' lines from the top
    /// and bottom of the window where possible, and where not, where
    /// [`Win::scroll_cursor_halfway`] would have put it. `w_topline` must be
    /// valid.
    pub(super) fn cursor_correct(mut self) {
        // How much context we would like above and below the cursor depends on
        // whether the first and last line of the file are on screen.
        let mut above_wanted = scrolloff_or_drag(self);
        let mut below_wanted = above_wanted;
        if self.w_topline == 1 {
            above_wanted = 0;
            below_wanted = below_wanted.min((self.w_view_height / 2) as int64_t);
        }
        self.validate_botline();
        if self.w_botline == self.buffer().line_count() + 1 && mouse_dragging.get() == 0 {
            below_wanted = 0;
            above_wanted = above_wanted.min(((self.w_view_height - 1) / 2) as int64_t);
        }

        // Enough file lines above and below the cursor: nothing to do.
        let cursor_lnum = self.w_cursor.lnum;
        if cursor_lnum as int64_t >= self.w_topline as int64_t + above_wanted
            && (cursor_lnum as int64_t) < self.w_botline as int64_t - below_wanted
            && !self.lines_concealed()
        {
            return;
        }

        // 'smoothscroll' is active. TODO(vim): when the cursor line does not
        // fit in the window, adjust `w_skipcol` instead.
        if self.w_onebuf_opt.wo_sms != 0
            && self.w_onebuf_opt.wo_wrap == 0
            && self.w_cline_height == self.w_view_height
        {
            // The cursor line just fits in the window: don't scroll.
            self.reset_skipcol();
            return;
        }

        // Narrow down the area the cursor may be put in by taking lines off
        // the top and the bottom until the wanted context is found, or the
        // lines from the top have passed the lines from the bottom.
        let mut topline = self.w_topline;
        let mut botline = self.w_botline - 1;
        // Filler lines count as context.
        let mut above = self.w_topfill;
        let mut below = self.w_filler_rows;
        while ((above as int64_t) < above_wanted || (below as int64_t) < below_wanted)
            && topline < botline
        {
            if (below as int64_t) < below_wanted
                && (below <= above || above as int64_t >= above_wanted)
            {
                below += self.plines_full(botline, true, true).0;
                botline = self.fold_first(botline).unwrap_or(botline);
                botline -= 1;
            }
            if (above as int64_t) < above_wanted
                && (above < below || below as int64_t >= below_wanted)
            {
                above += self.plines_nofill(topline, true);
                topline = self.fold_last(topline);
                // Count the filler lines below this line as context.
                if topline < botline {
                    above += self.fill_above(topline + 1);
                }
                topline += 1;
            }
        }
        if topline == botline || botline == 0 {
            self.w_cursor.lnum = topline;
        } else if topline > botline {
            self.w_cursor.lnum = botline;
        } else {
            if cursor_lnum < topline && self.w_topline > 1 {
                self.w_cursor.lnum = topline;
                self.w_valid = self
                    .w_valid
                    .without(WinValid::WROW | WinValid::WCOL | WinValid::CHEIGHT | WinValid::CROW);
            }
            if cursor_lnum > botline && self.w_botline <= self.buffer().line_count() {
                self.w_cursor.lnum = botline;
                self.w_valid = self
                    .w_valid
                    .without(WinValid::WROW | WinValid::WCOL | WinValid::CHEIGHT | WinValid::CROW);
            }
        }
        self.check_cursor_moved();
        self.w_valid |= WinValid::TOPLINE;
        self.w_viewport_invalid = true;
    }
}

/// How much overlap to use for a page-up or page-down scroll.
///
/// Symmetric, so that doing both keeps the same lines displayed. Three lines
/// are examined:
///
/// ```text
///  before CTRL-F          after CTRL-F / before CTRL-B
///     etc.                    l1
///  l1 last but one line       ------------
///  l2 last text line          l2 top text line
///  -------------              l3 second text line
///  l3                            etc.
/// ```
pub(super) fn get_scroll_overlap(win: Win, dir: Direction) -> c_int {
    let min_height = win.w_view_height - 2;
    let forward = dir == FORWARD;
    let backward = dir == BACKWARD;

    win.validate_botline();
    if backward && win.w_topline == 1 || forward && win.w_botline > win.buffer().line_count() {
        // No overlap; still handle 'smoothscroll'.
        return min_height + 2;
    }

    let lnum = if forward {
        win.w_botline
    } else {
        win.w_topline - 1
    };
    let mut loff = lineoff_T {
        lnum,
        // Paging backwards, the filler lines that matter are the ones above
        // the line *below* this one.
        fill: win.fill_above(lnum + backward as linenr_T)
            - if forward {
                win.w_filler_rows
            } else {
                win.w_topfill
            },
        height: 0,
    };
    loff.height = if loff.fill > 0 {
        1
    } else {
        win.plines_nofill(loff.lnum, true)
    };

    // One step outwards per examined line, against the direction the page
    // moves: paging forward, the overlap is measured upwards from `w_botline`.
    let step = |lp: &mut lineoff_T| {
        if forward {
            topline_back(win, lp);
        } else {
            botline_forw(win, lp);
        }
    };

    let h1 = loff.height;
    if h1 > min_height {
        // No overlap.
        return min_height + 2;
    }
    step(&mut loff);
    let h2 = loff.height;
    if h2 == MAXCOL as c_int || h2 + h1 > min_height {
        return min_height + 2;
    }
    step(&mut loff);
    let h3 = loff.height;
    if h3 == MAXCOL as c_int || h3 + h2 > min_height {
        return min_height + 2;
    }
    step(&mut loff);
    let h4 = loff.height;
    if h4 == MAXCOL as c_int || h4 + h3 + h2 > min_height || h3 + h2 + h1 > min_height {
        // One line of overlap.
        min_height + 1
    } else {
        // Two lines of overlap.
        min_height
    }
}

/// Scroll `count` lines with 'smoothscroll' in direction `dir`, answering
/// whether anything moved. `curscount` is corrected for scrolling a different
/// number of lines when 'smoothscroll' is off.
///
/// `win` must be the current window: the scroll itself goes through
/// [`scroll_redraw_cur`].
pub(super) fn scroll_with_sms(
    mut win: Win,
    dir: Direction,
    count: c_int,
    curscount: &mut c_int,
) -> bool {
    let prev_sms = win.w_onebuf_opt.wo_sms;
    let prev_skipcol = win.w_skipcol;
    let prev_topline = win.w_topline;
    let prev_topfill = win.w_topfill;

    win.w_onebuf_opt.wo_sms = 1;
    scroll_redraw_cur(win, dir == FORWARD, count as linenr_T);

    // Not actually smoothscrolling, but we ended up with a partly visible
    // line. Keep scrolling until `w_skipcol` is zero again.
    if prev_sms == 0 && win.w_skipcol > 0 {
        // Reverse the scroll direction when `w_topline` already changed. One
        // line extra scrolling backward, so that consuming `w_skipcol` is
        // symmetric.
        let fixdir = if (win.w_topline - prev_topline).abs() > (dir == BACKWARD) as linenr_T {
            -dir
        } else {
            dir
        };

        let (width1, width2) = sms_widths(win);
        let count = if fixdir == FORWARD {
            let size = win.line_display_width(win.w_topline);
            arith::sms_fixup_count_forw(win.w_skipcol, size, width1, width2)
        } else {
            arith::sms_fixup_count_back(win.w_skipcol, width1, width2)
        };

        scroll_redraw_cur(win, fixdir == FORWARD, count as linenr_T);
        *curscount += count * if fixdir == dir { 1 } else { -1 };
    }
    win.w_onebuf_opt.wo_sms = prev_sms;

    win.w_topline != prev_topline || win.w_topfill != prev_topfill || win.w_skipcol != prev_skipcol
}
