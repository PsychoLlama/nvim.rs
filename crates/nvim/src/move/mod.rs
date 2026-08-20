//! Where the cursor is on the screen, and which part of the buffer the window
//! shows.
//!
//! Carved by the question each part answers:
//!
//! | child | what |
//! | --- | --- |
//! | [`topline`] | `update_topline()` -- has the cursor left the visible range? |
//! | [`columns`] | `curs_columns()`, `screenpos()`, `virtcol2col()` -- the horizontal half |
//! | [`scroll`] | `scrolldown()`/`scrollup()` and the clamped forms |
//! | [`scrollcur`] | the `scroll_cursor_*` family and `cursor_correct()` |
//! | [`page`] | `pagescroll()` and `'cursorbind'` |
//! | [`arith`] | the pointer-free viewport arithmetic they share |
//!
//! What stays here is the `w_valid` flag alphabet the five share, the
//! `lineoff_T` cursor those flags guard, the small predicates that read or
//! invalidate them (`validate_cursor`, `validate_virtcol`,
//! `validate_cursor_col`, `changed_cline_bef_curs` and friends), `curs_rows`,
//! and the two `win_col_off` helpers that say how much of a window is not
//! text.
//!
//! The windows stay raw `*mut win_T` at the module boundary -- callers hold
//! several at once and re-enter through autocommands -- but the dereferences
//! do not spread: [`Win`] wraps one and makes its *construction* the unsafe
//! step, and the `impl Win` block below adds one thin wrapper per call into a
//! neighbouring module. Everything above that layer, this family's arithmetic
//! included, is ordinary safe code.
//!
//! Original: `src/nvim/move.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uint};

pub mod arith;

// The carve of the transpiled module; see each child's docs.
mod columns;
mod page;
mod scroll;
mod scrollcur;
mod topline;

pub use self::columns::*;
pub use self::page::*;
pub use self::scroll::*;
pub use self::scrollcur::*;
pub use self::topline::*;

use crate::cursor::check_cursor_lnum;
use crate::decoration::{SIGN_WIDTH, decor_conceal_line, win_lines_concealed};
use crate::drawscreen::{
    UPD_INVERTED, UPD_SOME_VALID, UPD_VALID, conceal_cursor_line, number_width, redraw_buf_later,
    redraw_win_line, redrawing, win_cursorline_standout,
};
use crate::fold::foldAdjustCursor;
use crate::main::{VIsual_active, cmdwin_win, curbuf};
use crate::option::{cpo_has, get_scrolloff_value, get_showbreak_value, get_sidescrolloff_value};
use crate::options::kOptCuloptFlagScreenline;
use crate::plines::{
    linetabsize_eol, plines_m_win, plines_win, plines_win_full, plines_win_nofill, win_get_fill,
    win_may_fill,
};
use crate::types::{CpoFlag, MotionType, NUL, colnr_T, int64_t, linenr_T, win_T, wline_T};
use crate::window::win_fdccol_count;
use crate::winfloat::win_check_anchored_floats;
use crate::winlayer::Win;

pub type C2Rust_Unnamed_15 = c_uint;
pub const kMTCharWise: MotionType = 0;

/// One buffer line as the vertical scrolling walks it: the line, the filler
/// lines drawn above it, and the screen lines it takes.
#[derive(Copy, Clone)]
pub struct lineoff_T {
    pub lnum: linenr_T,
    pub fill: c_int,
    pub height: c_int,
}

crate::flag_set! {
    /// Which of a window's cached cursor and scroll positions are still
    /// right -- upstream's `VALID_*`, the bits `win_T::w_valid` carries.
    /// A field whose bit is clear must be recomputed before it is read.
    pub struct WinValid;

    /// `w_wrow` is right.
    const WROW = 0x1;
    /// `w_wcol` is right.
    const WCOL = 0x2;
    /// `w_virtcol` is right.
    const VIRTCOL = 0x4;
    /// `w_cline_height` and `w_cline_folded` are right.
    const CHEIGHT = 0x8;
    /// `w_cline_row` is right.
    const CROW = 0x10;
    /// `w_botline` is right.
    const BOTLINE = 0x20;
    /// `w_botline` is at worst approximately right.
    const BOTLINE_AP = 0x40;
    /// `w_topline` shows the cursor with 'scrolloff' context.
    const TOPLINE = 0x80;
}

// ---------------------------------------------------------------------------
// The window, as this family reads it
//
// Each method below is one call into a neighbouring module or one projection
// off the window pointer, resting on the promise `Win`'s constructor took.
// Everything else in the family is safe code written on top of them.

impl Win {
    /// Screen columns to the left of the text: the 'number'/'statuscolumn'
    /// column, the command-line window's marker, the fold column and the sign
    /// column. None of them move when the window scrolls horizontally.
    pub(super) fn col_off(self) -> c_int {
        self.number_col()
            + (self.raw() == cmdwin_win.get()) as c_int
            + self.fdccol_count()
            + self.w_scwidth * SIGN_WIDTH
    }

    /// The extra offset the *second* and later screen lines of a wrapped line
    /// get. Positive only with 'number'/'relativenumber' and `n` in
    /// 'cpoptions'.
    pub(super) fn col_off2(self) -> c_int {
        let indents = cpo_has(CpoFlag::NUMCOL);
        if indents { self.number_col() } else { 0 }
    }

    /// Both text widths at once: a line's first screen line, then its later
    /// ones. Upstream reads the two offsets once each and so does this --
    /// asking [`Win::col_off`] twice would ask [`number_width`] twice.
    pub(super) fn text_widths(self) -> (c_int, c_int) {
        let width1 = self.w_view_width - self.col_off();
        (width1, width1 + self.col_off2())
    }

    /// Text width of a line's first screen line.
    pub(super) fn text_width(self) -> c_int {
        self.w_view_width - self.col_off()
    }

    /// The width the 'number'/'statuscolumn' column takes: 'numberwidth'
    /// digits plus one separating space, and nothing at all when neither
    /// option is on.
    ///
    /// The early return is not just a shortcut. [`number_width`] *memoises*
    /// into `w_nrwidth_width`, and this is on the per-line draw path, so
    /// asking it where upstream would not both costs and writes.
    fn number_col(self) -> c_int {
        // SAFETY: an option string is NUL-terminated, never null.
        let stc_empty = unsafe { *self.w_onebuf_opt.wo_stc == NUL as c_char };
        if self.w_onebuf_opt.wo_nu == 0 && self.w_onebuf_opt.wo_rnu == 0 && stc_empty {
            return 0;
        }
        // SAFETY: a live window.
        unsafe { number_width(self.raw()) + stc_empty as c_int }
    }

    pub(super) fn fdccol_count(self) -> c_int {
        // SAFETY: a live window.
        unsafe { win_fdccol_count(self.raw()) }
    }

    /// Whether 'showbreak' is unset for this window.
    pub(super) fn showbreak_empty(self) -> bool {
        // SAFETY: a live window; the answer is a NUL-terminated string.
        unsafe { *get_showbreak_value(self.raw()) == NUL as c_char }
    }

    /// Screen lines line `lnum` takes with 'wrap' and folds accounted for but
    /// filler lines left out, optionally capped at the window height.
    pub(super) fn plines_nofill(self, lnum: linenr_T, limit_winheight: bool) -> c_int {
        // SAFETY: a live window.
        unsafe { plines_win_nofill(self.raw(), lnum, limit_winheight) }
    }

    /// As [`Win::plines_nofill`], filler lines included.
    pub(super) fn plines(self, lnum: linenr_T, limit_winheight: bool) -> c_int {
        // SAFETY: a live window.
        unsafe { plines_win(self.raw(), lnum, limit_winheight) }
    }

    /// Screen lines the range `first..=last` takes, capped at `max`.
    pub(super) fn plines_range(self, first: linenr_T, last: linenr_T, max: c_int) -> c_int {
        // SAFETY: a live window.
        unsafe { plines_m_win(self.raw(), first, last, max) }
    }

    /// Screen cells line `lnum` takes, the cell past its end included -- the
    /// width 'smoothscroll' measures a line by.
    pub(super) fn line_display_width(self, lnum: linenr_T) -> c_int {
        // SAFETY: a live window.
        unsafe { linetabsize_eol(self.raw(), lnum) }
    }

    /// Move the cursor to the first line of the fold it landed in.
    pub(super) fn fold_adjust_cursor(self) {
        // SAFETY: a live window.
        unsafe { foldAdjustCursor(self.raw()) };
    }

    /// Screen lines line `lnum` takes, folds, filler lines and 'wrap' all
    /// counted. Answers the height, the last line of a fold starting at
    /// `lnum` (`lnum` itself when there is none), and whether it is folded.
    pub(super) fn plines_full(
        self,
        lnum: linenr_T,
        cache: bool,
        limit_winheight: bool,
    ) -> (c_int, linenr_T, bool) {
        let mut next = lnum;
        let mut folded = false;
        let (n, f) = (&raw mut next, &raw mut folded);
        // SAFETY: a live window. `next` is written only for a folded line,
        // which is why it is seeded with `lnum`.
        let height = unsafe { plines_win_full(self.raw(), lnum, n, f, cache, limit_winheight) };
        (height, next, folded)
    }

    /// Whether the window has filler lines above its top line ('diff').
    pub(super) fn may_fill(self) -> bool {
        // SAFETY: a live window.
        unsafe { win_may_fill(self.raw()) }
    }

    /// Filler lines drawn above line `lnum`.
    pub(super) fn fill_above(self, lnum: linenr_T) -> c_int {
        // SAFETY: a live window.
        unsafe { win_get_fill(self.raw(), lnum) }
    }

    /// Whether any line of the window is hidden outright by a decoration.
    pub(super) fn lines_concealed(self) -> bool {
        // SAFETY: a live window.
        unsafe { win_lines_concealed(self.raw()) }
    }

    /// Whether line `lnum` (zero-based, as the decoration layer counts) is
    /// hidden outright.
    pub(super) fn conceals_line(self, lnum: c_int, include_cursor: bool) -> bool {
        // SAFETY: a live window.
        unsafe { decor_conceal_line(self.raw(), lnum, include_cursor) }
    }

    /// Whether the cursor line is drawn concealed in the current mode.
    pub(super) fn conceal_cursor_line(self) -> bool {
        // SAFETY: a live window.
        unsafe { conceal_cursor_line(self.raw()) }
    }

    /// Whether 'cursorline' would draw this window's cursor line differently.
    fn cursorline_standout(self) -> bool {
        // SAFETY: a live window.
        unsafe { win_cursorline_standout(self.raw()) }
    }

    /// 'scrolloff' for this window, its window-local value preferred.
    pub(super) fn scrolloff(self) -> int64_t {
        // SAFETY: a live window.
        unsafe { get_scrolloff_value(self.raw()) }
    }

    /// 'sidescrolloff' for this window, its window-local value preferred.
    pub(super) fn sidescrolloff(self) -> int64_t {
        // SAFETY: a live window.
        unsafe { get_sidescrolloff_value(self.raw()) }
    }

    /// Redraw just the cursor's line.
    pub(super) fn redraw_cursor_line(self) {
        // SAFETY: a live window.
        unsafe { redraw_win_line(self.raw(), self.w_cursor.lnum) };
    }

    /// Let any float anchored to this window follow it.
    pub(super) fn check_anchored_floats(self) {
        // SAFETY: a live window.
        unsafe { win_check_anchored_floats(self.raw()) };
    }

    /// Clamp the cursor's line number to the buffer.
    pub(super) fn check_cursor_lnum(self) {
        // SAFETY: a live window.
        unsafe { check_cursor_lnum(self.raw()) };
    }

    /// One of the screen lines the window remembers drawing, copied out.
    ///
    /// Copied rather than borrowed because the walk that reads these also
    /// calls back into the fold and decoration layers between reads.
    pub(super) fn remembered_line(self, i: c_int) -> wline_T {
        debug_assert!(i >= 0 && i < self.w_lines_valid, "i < wp->w_lines_valid");
        // SAFETY: `w_lines` holds at least `w_lines_valid` live entries.
        unsafe { *self.w_lines.offset(i as isize) }
    }
}

/// Screen lines of the top line that `w_skipcol` scrolls out of sight.
fn adjust_plines_for_skipcol(win: Win) -> c_int {
    let (width1, width2) = win.text_widths();
    arith::skipped_plines(win.w_skipcol, width1, width2)
}

impl Win {
    /// Screen lines line `lnum` takes, corrected for it being the top line
    /// (where `w_skipcol` may hide part of it) and optionally capped at the
    /// window height. Also answers the last line of a fold starting at
    /// `lnum`, and whether there is one.
    pub(super) fn corrected_plines(
        self,
        lnum: linenr_T,
        limit_winheight: bool,
    ) -> (c_int, linenr_T, bool) {
        let (mut n, next, folded) = self.plines_full(lnum, true, false);
        if lnum == self.w_topline {
            n -= adjust_plines_for_skipcol(self);
        }
        if limit_winheight && n > self.w_view_height {
            n = self.w_view_height;
        }
        (n, next, folded)
    }
}

/// [`Win::corrected_plines`], for the callers still holding a raw window.
///
/// Answers the height and the last line of a fold starting at `lnum`.
/// Upstream passes both back through `linenr_T *`/`bool *` out-params; no
/// caller ever asked for the third.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn plines_correct_topline(
    wp: *mut win_T,
    lnum: linenr_T,
    limit_winheight: bool,
) -> (c_int, linenr_T) {
    // SAFETY: the caller's promise.
    let (n, next, _) = unsafe { Win::new(wp) }.corrected_plines(lnum, limit_winheight);
    (n, next)
}

/// Recompute `w_botline` for the current `w_topline`.
fn comp_botline(mut win: Win) {
    win.check_cursor_moved();
    // If `w_cline_row` is valid start there, otherwise at the top line.
    let (mut lnum, mut done) = if win.w_valid.has(WinValid::CROW) {
        (win.w_cursor.lnum, win.w_cline_row)
    } else {
        (win.w_topline, 0)
    };
    while lnum <= win.buffer().line_count() {
        let (n, last, folded) = win.corrected_plines(lnum, true);
        if lnum <= win.w_cursor.lnum && last >= win.w_cursor.lnum {
            win.w_cline_row = done;
            win.w_cline_height = n;
            win.w_cline_folded = folded;
            redraw_for_cursorline(win);
            win.w_valid |= WinValid::CROW | WinValid::CHEIGHT;
        }
        if done + n > win.w_view_height {
            break;
        }
        done += n;
        lnum = last + 1;
    }
    // `w_botline` is the line just below the window.
    win.w_botline = lnum;
    win.w_valid |= WinValid::BOTLINE | WinValid::BOTLINE_AP;
    win.w_viewport_invalid = true;
    win.set_empty_rows(done);
    win.check_anchored_floats();
}

/// Redraw when `w_cline_row` changed and 'relativenumber' or 'cursorline' is
/// set, or when concealing is on and 'concealcursor' is not active.
fn redraw_for_cursorline(win: Win) {
    if win.w_valid.has(WinValid::CROW) {
        return;
    }
    if win.w_onebuf_opt.wo_rnu != 0 || win.cursorline_standout() {
        // `win_line()` will redraw the number column and cursorline only.
        win.redraw_later(UPD_VALID);
    }
}

/// Redraw when 'concealcursor' is active, or when `w_virtcol` changed and
/// 'cursorcolumn' is set, 'cursorlineopt' contains "screenline", or Visual
/// mode is active.
fn redraw_for_cursorcolumn(win: Win) {
    // Moving horizontally under 'concealcursor' changes what the line looks
    // like, so it has to be drawn again to place the cursor.
    if win.is_current() && win.w_onebuf_opt.wo_cole > 0 && win.conceal_cursor_line() {
        win.redraw_cursor_line();
    }
    if win.w_valid.has(WinValid::VIRTCOL) {
        return;
    }
    if win.w_onebuf_opt.wo_cuc != 0 {
        win.redraw_later(UPD_SOME_VALID);
    } else if win.w_onebuf_opt.wo_cul != 0
        && win.w_p_culopt_flags as c_int & kOptCuloptFlagScreenline as c_int != 0
    {
        win.redraw_later(UPD_VALID);
    }
    // The current buffer's cursor moving in Visual mode changes the highlight.
    if VIsual_active.get() && win.w_buffer == curbuf.get() {
        // SAFETY: `curbuf` is set from startup to exit.
        unsafe { redraw_buf_later(curbuf.get(), UPD_INVERTED) };
    }
}

/// Record a `w_virtcol` the caller has already computed, redrawing if it was
/// invalid before.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn set_valid_virtcol(wp: *mut win_T, vcol: colnr_T) {
    // SAFETY: the caller's promise.
    let mut win = unsafe { Win::new(wp) };
    win.w_virtcol = vcol;
    redraw_for_cursorcolumn(win);
    win.w_valid |= WinValid::VIRTCOL;
}

/// [`Win::marker_overlap`], for the callers still holding a raw window.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn sms_marker_overlap(wp: *mut win_T, extra2: c_int) -> c_int {
    // SAFETY: the caller's promise.
    unsafe { Win::new(wp) }.marker_overlap(extra2)
}

impl Win {
    /// Columns of buffer text the 'listchars' "precedes" or 'smoothscroll'
    /// `<<<` marker covers. `extra2` is the padding on a wrapped line's second
    /// screen line, or -1 to compute it.
    pub(super) fn marker_overlap(self, extra2: c_int) -> c_int {
        let extra2 = if extra2 == -1 {
            self.col_off() - self.col_off2()
        } else {
            extra2
        };
        arith::marker_overlap(
            extra2,
            !self.showbreak_empty(),
            self.w_onebuf_opt.wo_list != 0 && self.w_p_lcs_chars.prec != 0,
        )
    }

    /// The `w_skipcol` that hides `plines_off` screen lines of the top line.
    pub(super) fn skipcol_from_plines(self, plines_off: c_int) -> colnr_T {
        let (width1, width2) = self.text_widths();
        arith::skipcol_from_plines(plines_off, width1, width2)
    }

    /// Set `w_skipcol` back to zero, redrawing if it was not already.
    pub(super) fn reset_skipcol(mut self) {
        if self.w_skipcol == 0 {
            return;
        }
        self.w_skipcol = 0;
        // The cheapest redraw that shows everything that changed:
        // UPD_NOT_VALID is too expensive and UPD_REDRAW_TOP redraws too
        // little when the top line gains a screen line.
        self.redraw_later(UPD_SOME_VALID);
    }
}

/// The length of the cursor line changed *before* the cursor, so its screen
/// height -- and with it `w_topline` and `w_crow` -- may have changed.
/// `w_botline` is the caller's problem.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn changed_cline_bef_curs(wp: *mut win_T) {
    // SAFETY: the caller's promise.
    unsafe { Win::new(wp) }.invalidate_above_cursor();
}

/// As [`changed_cline_bef_curs`], for a line *above* the cursor in the
/// current window.
///
/// # Safety
/// The current window must be valid.
pub unsafe fn changed_line_abv_curs() {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }.invalidate_above_cursor();
}

/// As [`changed_line_abv_curs`], for a given window.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn changed_line_abv_curs_win(wp: *mut win_T) {
    // SAFETY: the caller's promise.
    unsafe { Win::new(wp) }.invalidate_above_cursor();
}

impl Win {
    /// Forget everything that depends on the cursor line's screen height.
    fn invalidate_above_cursor(mut self) {
        self.w_valid.clear(
            WinValid::WROW
                | WinValid::WCOL
                | WinValid::VIRTCOL
                | WinValid::CROW
                | WinValid::CHEIGHT
                | WinValid::TOPLINE,
        );
    }
}

impl Win {
    /// Make sure `w_botline` is right.
    pub(super) fn validate_botline(self) {
        if !self.w_valid.has(WinValid::BOTLINE) {
            comp_botline(self);
        }
    }

    /// Make sure `w_wrow` and `w_wcol` are right.
    pub(super) fn validate_cursor(self) {
        self.check_cursor_lnum();
        self.check_cursor_moved();
        if !self.w_valid.has_all(WinValid::WCOL | WinValid::WROW) {
            self.curs_columns(true);
        }
    }

    /// Make sure `w_virtcol` is right.
    pub(super) fn validate_virtcol(mut self) {
        self.check_cursor_moved();
        if self.w_valid.has(WinValid::VIRTCOL) {
            return;
        }
        self.w_virtcol = self.virtual_cursor_vcol(self.cursor());
        redraw_for_cursorcolumn(self);
        self.w_valid |= WinValid::VIRTCOL;
    }
}

/// [`Win::validate_botline`], for the callers still holding a raw window.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn validate_botline_win(wp: *mut win_T) {
    // SAFETY: the caller's promise.
    unsafe { Win::new(wp) }.validate_botline();
}

/// [`Win::invalidate_botline`], for the callers still holding a raw window.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn invalidate_botline_win(wp: *mut win_T) {
    // SAFETY: the caller's promise.
    unsafe { Win::new(wp) }.invalidate_botline();
}

impl Win {
    /// Mark `w_botline` invalid, because the buffer changed.
    pub(super) fn invalidate_botline(mut self) {
        self.w_valid = self
            .w_valid
            .without(WinValid::BOTLINE | WinValid::BOTLINE_AP);
    }
}

/// Mark `w_botline` as only approximately right.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn approximate_botline_win(wp: *mut win_T) {
    // SAFETY: the caller's promise.
    let mut win = unsafe { Win::new(wp) };
    win.w_valid.clear(WinValid::BOTLINE);
}

/// Whether `w_wrow` and `w_wcol` are both right.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn cursor_valid(wp: *mut win_T) -> c_int {
    // SAFETY: the caller's promise.
    let win = unsafe { Win::new(wp) };
    win.check_cursor_moved();
    win.w_valid.has_all(WinValid::WROW | WinValid::WCOL) as c_int
}

/// Make sure `w_wrow` and `w_wcol` are right. `w_topline` must already be --
/// callers usually want `update_topline()` first.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn validate_cursor(wp: *mut win_T) {
    // SAFETY: the caller's promise.
    unsafe { Win::new(wp) }.validate_cursor();
}

/// Compute `w_cline_row` and `w_cline_height` from the current `w_topline`.
fn curs_rows(mut win: Win) {
    // Are the remembered `w_lines[].wl_size` usable at all?
    // SAFETY: `redrawing` reads editor state, not a pointer of ours.
    let all_invalid = !unsafe { redrawing() }
        || win.w_lines_valid == 0
        || win.remembered_line(0).wl_lnum > win.w_topline;
    let mut i: c_int = 0;
    win.w_cline_row = 0;
    let mut lnum = win.w_topline;
    while lnum < win.w_cursor.lnum {
        let mut valid = false;
        let mut skipped = false;
        if !all_invalid && i < win.w_lines_valid {
            let wl = win.remembered_line(i);
            if wl.wl_lnum < lnum || !wl.wl_valid {
                // A changed or deleted line; move on to the next entry.
                skipped = true;
            } else if wl.wl_lnum == lnum {
                // Newly inserted lines below this row mean the folds have to
                // be looked at again.
                if !win.buffer().b_mod_set
                    || wl.wl_lastlnum < win.w_cursor.lnum
                    || win.buffer().b_mod_top > wl.wl_lastlnum + 1
                {
                    valid = true;
                }
            } else if wl.wl_lnum > lnum {
                // Hold at inserted lines: the `i += 1` below undoes this.
                i -= 1;
            }
        }
        if !skipped {
            if valid && (lnum != win.w_topline || (win.w_skipcol == 0 && !win.may_fill())) {
                let wl = win.remembered_line(i);
                lnum = wl.wl_lastlnum + 1;
                // The cursor is inside folded or concealed lines; this row
                // does not count.
                if lnum > win.w_cursor.lnum {
                    break;
                }
                win.w_cline_row += wl.wl_size as c_int;
            } else {
                let (n, last, _) = win.corrected_plines(lnum, true);
                lnum = last + 1;
                if lnum + win.conceals_line(lnum - 1, false) as linenr_T > win.w_cursor.lnum {
                    break;
                }
                win.w_cline_row += n;
            }
        }
        i += 1;
    }

    win.check_cursor_moved();
    if !win.w_valid.has(WinValid::CHEIGHT) {
        // The remembered entry is unusable when it names another line or was
        // marked stale.
        let stale = i < win.w_lines_valid && {
            let wl = win.remembered_line(i);
            !wl.wl_valid || wl.wl_lnum != win.w_cursor.lnum
        };
        if all_invalid || i == win.w_lines_valid || stale {
            let (height, _, folded) = win.plines_full(win.w_cursor.lnum, true, true);
            win.w_cline_height = height;
            win.w_cline_folded = folded;
        } else if i > win.w_lines_valid {
            // A line too long to fit on the last screen line.
            win.w_cline_height = 0;
            win.w_cline_folded = win.fold_span(win.w_cursor.lnum).0;
        } else {
            let wl = win.remembered_line(i);
            win.w_cline_height = wl.wl_size as c_int;
            win.w_cline_folded = wl.wl_folded;
        }
    }
    redraw_for_cursorline(win);
    win.w_valid |= WinValid::CROW | WinValid::CHEIGHT;
}

/// Make sure `w_virtcol` is right, and nothing else.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn validate_virtcol(wp: *mut win_T) {
    // SAFETY: the caller's promise.
    unsafe { Win::new(wp) }.validate_virtcol();
}

/// [`Win::validate_cheight`], for the callers still holding a raw window.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn validate_cheight(wp: *mut win_T) {
    // SAFETY: the caller's promise.
    unsafe { Win::new(wp) }.validate_cheight();
}

impl Win {
    /// Make sure `w_cline_height` is right, and nothing else.
    pub(super) fn validate_cheight(mut self) {
        self.check_cursor_moved();
        if self.w_valid.has(WinValid::CHEIGHT) {
            return;
        }
        let (height, _, folded) = self.plines_full(self.w_cursor.lnum, true, true);
        self.w_cline_height = height;
        self.w_cline_folded = folded;
        self.w_valid |= WinValid::CHEIGHT;
    }
}

/// Make sure `w_wcol` and `w_virtcol` are right, and nothing else.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn validate_cursor_col(wp: *mut win_T) {
    // SAFETY: the caller's promise.
    let mut win = unsafe { Win::new(wp) };
    win.validate_virtcol();
    if win.w_valid.has(WinValid::WCOL) {
        return;
    }
    let off = win.col_off();
    win.w_wcol = arith::cursor_screen_col(
        win.w_virtcol,
        off,
        win.w_view_width,
        win.w_view_width - off + win.col_off2(),
        win.w_onebuf_opt.wo_wrap != 0,
        win.w_leftcol,
    );
    win.w_valid |= WinValid::WCOL;
}

/// Columns of a window that are not text: the 'number'/'statuscolumn'
/// column, the fold column and the sign column. They do not move when the
/// window scrolls horizontally.
///
/// # Safety
/// `wp` must be a valid window.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn win_col_off(wp: *mut win_T) -> c_int {
    // SAFETY: the caller's promise.
    unsafe { Win::new(wp) }.col_off()
}

/// The extra column offset a wrapped line's later screen lines get.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn win_col_off2(wp: *mut win_T) -> c_int {
    // SAFETY: the caller's promise.
    unsafe { Win::new(wp) }.col_off2()
}
