//! The viewport arithmetic that touches no pointer: where a virtual column
//! lands on the screen, how far a wrapped line is skipped, and which of
//! 'scrolljump', 'sidescroll' and 'scrolloff' decides a scroll.
//!
//! These are the decisions [`super`]'s validation functions make between
//! their window reads, lifted out of the raw-pointer code so they can be
//! stated — and tested — on their own. `tests/unit/move.rs` drives them
//! directly, which is also how Miri sees this half of the module.
//!
//! Every function here mirrors one arm of `v0.12.4`'s `src/nvim/move.c`, and
//! keeps that arm's integer widths: the C mixes `int`, `ColNr` and
//! `int64_t` deliberately, and where a subtraction is done narrow before it
//! is widened, so is it here.
//!
//! Original: `src/nvim/move.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use crate::narrow::number_as_int;
use crate::os::libc::int64_t;
use crate::types::{ColNr, OptInt};

/// Screen lines of the top line that `w_skipcol` scrolls out of sight.
///
/// `width1` is the text width of a line's first screen line and `width2` that
/// of its later ones. From `adjust_plines_for_skipcol()`.
pub fn skipped_plines(skipcol: ColNr, width1: c_int, width2: c_int) -> c_int {
    if skipcol == ColNr::ZERO {
        return 0;
    }
    if skipcol >= ColNr::new(width1) && width2 > 0 {
        (skipcol - width1).get() / width2 + 1
    } else {
        0
    }
}

/// The inverse: the `w_skipcol` that hides `plines_off` screen lines of the
/// top line. From `skipcol_from_plines()`.
pub fn skipcol_from_plines(plines_off: c_int, width1: c_int, width2: c_int) -> c_int {
    let mut skipcol = 0;
    if plines_off > 0 {
        skipcol += width1;
    }
    if plines_off > 1 {
        skipcol += width2 * (plines_off - 1);
    }
    skipcol
}

/// Columns of buffer text the 'smoothscroll' `<<<` marker sits on top of.
///
/// `extra2` is the padding on a wrapped line's *second* screen line. From
/// `sms_marker_overlap()`, whose two early answers are the option states its
/// caller has already read.
pub fn marker_overlap(extra2: c_int, showbreak: bool, list_precedes: bool) -> c_int {
    // In 'showbreak' mode the marker is not drawn over text at all; see
    // wlv_put_linebuf().
    if showbreak {
        return 0;
    }
    if list_precedes {
        return 1;
    }
    if extra2 > 3 { 0 } else { 3 - extra2 }
}

/// Screen lines of the top line that `w_skipcol` scrolls out of sight, as
/// `scroll_cursor_bot()` counts them.
///
/// Not [`skipped_plines`]: that one stops answering below `width1`, this one
/// counts any non-zero `w_skipcol` as a whole screen line. The C calls it "a
/// similar formula" to `curs_columns()`'s and the difference is deliberate --
/// here the partly-hidden first screen line still has to be scrolled past.
pub fn top_skipped_plines(skipcol: ColNr, width1: c_int, width2: c_int) -> c_int {
    if skipcol > ColNr::new(width1) {
        (skipcol - width1).get() / width2 + 1
    } else {
        c_int::from(skipcol > ColNr::ZERO)
    }
}

/// The columns of context 'scrolloff' asks for on a `'smoothscroll'` line:
/// the first screen line plus `so - 1` later ones. From `cursor_correct_sms()`
/// and `adjust_skipcol()`.
pub fn scrolloff_cols(so: int64_t, width1: c_int, width2: c_int) -> int64_t {
    if so == 0 {
        0
    } else {
        int64_t::from(width1) + (so - 1) * int64_t::from(width2)
    }
}

/// Drop `so_cols` to what a line only `size` cells wide can actually offer.
/// From `cursor_correct_sms()`.
pub fn fit_scrolloff_cols(so_cols: int64_t, size: c_int, width1: c_int, width2: c_int) -> int64_t {
    let mut so_cols = so_cols;
    // Not enough screen lines in the top line: ignore 'scrolloff'.
    while so_cols > int64_t::from(size)
        && so_cols - int64_t::from(width2) >= int64_t::from(width1)
        && width1 > 0
    {
        so_cols -= int64_t::from(width2);
    }
    if so_cols >= int64_t::from(width1) && so_cols > int64_t::from(size) {
        so_cols -= int64_t::from(width1);
    }
    so_cols
}

/// The virtual column nearest `col` that lies within the `top..bot` band of a
/// `'smoothscroll'` top line -- where `cursor_correct_sms()` puts the cursor.
///
/// Stepping by whole screen lines is what keeps the cursor's column intact.
pub fn visible_sms_col(
    col: ColNr,
    top: int64_t,
    bot: int64_t,
    width1: c_int,
    width2: c_int,
) -> ColNr {
    let mut col = col;
    if i64::from(col) < top {
        if col < ColNr::new(width1) {
            col += width1;
        }
        while width2 > 0 && i64::from(col) < top {
            col += width2;
        }
    } else {
        while width2 > 0 && i64::from(col) >= bot {
            col -= width2;
        }
    }
    col
}

/// The `w_skipcol` one screen line further down from `skipcol` -- the first
/// screen line is `width1` wide, every later one `width2`. From
/// `scrolldown()` and `adjust_skipcol()`.
pub fn skipcol_line_back(skipcol: ColNr, width1: c_int, width2: c_int) -> ColNr {
    if skipcol >= ColNr::new(width1 + width2) {
        skipcol - width2
    } else {
        skipcol - width1
    }
}

/// The `w_skipcol` that puts the *last* screen line of a `size`-wide line at
/// the top of the window. From `scrolldown()`'s 'smoothscroll' arm.
pub fn skipcol_showing_last(size: c_int, width1: c_int, width2: c_int) -> ColNr {
    if size <= width1 {
        return ColNr::ZERO;
    }
    let mut skipcol = width1;
    let mut left = size - width1;
    while left > width2 {
        skipcol += width2;
        left -= width2;
    }
    ColNr::new(skipcol)
}

/// The screen row the cursor sits on once 'scrolloff' has been added to its
/// virtual column and `w_skipcol` taken off. From `adjust_skipcol()`.
///
/// `size` is the line's display width; the 'scrolloff' columns are wound back
/// to it so a short line does not push the cursor off the bottom.
pub fn sms_cursor_row(
    virtcol: ColNr,
    scrolloff_cols: int64_t,
    skipcol: ColNr,
    size: c_int,
    width1: c_int,
    width2: c_int,
) -> c_int {
    let mut row = 0;
    let mut col = i64::from(virtcol) + scrolloff_cols;
    // Avoid adjusting for 'scrolloff' beyond the text line height.
    if scrolloff_cols > 0 {
        // The width of the line rounded up to a whole screen line.
        let size = width1 + width2 * ((size - width1 + width2 - 1) / width2);
        while col > int64_t::from(size) {
            col -= int64_t::from(width2);
        }
    }
    col -= i64::from(skipcol);
    if col >= int64_t::from(width1) {
        col -= int64_t::from(width1);
        row += 1;
    }
    // Upstream's second test is `>` where the first is `>=`, so a column
    // landing exactly on `width2` stays on this row.
    if col > int64_t::from(width2) {
        row += number_as_int(col / int64_t::from(width2));
    }
    row
}

/// Screen lines `scroll_with_sms()` has to scroll *backwards* to bring
/// `w_skipcol` back to zero: the screen lines of the top line already
/// scrolled past.
pub fn sms_fixup_count_back(skipcol: ColNr, width1: c_int, width2: c_int) -> c_int {
    1 + (skipcol - width1 - 1).get() / width2
}

/// As [`sms_fixup_count_back`], *forwards*: the screen lines of a
/// `size`-wide top line still to come.
pub fn sms_fixup_count_forw(skipcol: ColNr, size: c_int, width1: c_int, width2: c_int) -> c_int {
    1 + (size - skipcol.get() - width1 + width2 - 1) / width2
}

/// Lines a scroll moves at a minimum: 'scrolljump' as a count when positive,
/// as a percentage of the window height when negative.
pub fn scrolljump_lines(scrolljump: OptInt, view_height: c_int) -> c_int {
    if scrolljump >= 0 {
        number_as_int(scrolljump)
    } else {
        view_height * number_as_int(-scrolljump) / 100
    }
}

/// How far out of the window the cursor has to be before `update_topline()`
/// re-centres it instead of scrolling to put it near the edge. Half a window,
/// but never less than two lines.
pub fn recentre_threshold(view_height: c_int) -> c_int {
    let halfheight = view_height / 2 - 1;
    if halfheight < 2 { 2 } else { halfheight }
}

/// Screen lines down from the start of a wrapped line that virtual column
/// `col` falls on, where `width` is the text width of the later screen lines.
/// From the shared formula in `curs_columns()`, `validate_cursor_col()` and
/// `textpos2screenpos()`.
pub fn wrap_rowoff(col: ColNr, view_width: c_int, width: c_int) -> c_int {
    (col - view_width).get() / width + 1
}

/// The window column the cursor's virtual column shows at: `w_wcol` as
/// `validate_cursor_col()` computes it.
///
/// `off` is the non-text width on the left, `width` the text width of a
/// wrapped line's later screen lines.
pub fn cursor_screen_col(
    virtcol: ColNr,
    off: i32,
    view_width: c_int,
    width: c_int,
    wrap: bool,
    leftcol: ColNr,
) -> c_int {
    let mut col = virtcol + off;
    // Long line wrapping: bring the column back onto its own screen line.
    if wrap && col >= ColNr::new(view_width) && width > 0 {
        col -= wrap_rowoff(col, view_width, width) * width;
    }
    if col > leftcol { col - leftcol } else { 0 }
}

/// Where a wrapped cursor sits once `w_skipcol` and the line's own wrapping
/// are taken off: the new `(wcol, wrow)` and whether `w_skipcol` was one of
/// the terms — which the later `w_skipcol` fixup has to know.
///
/// From `curs_columns()`'s 'wrap' arm.
pub fn wrap_cursor_cell(
    wcol: c_int,
    wrow: c_int,
    skipcol: ColNr,
    at_topline: bool,
    width1: c_int,
    width2: c_int,
    view_width: c_int,
) -> (c_int, c_int, bool) {
    let (mut wcol, mut wrow) = (wcol, wrow);
    let mut did_sub_skipcol = false;
    // Skip the columns 'smoothscroll' has scrolled past. Deducting whole
    // multiples of width2 is what lets the wrapping formula below still give
    // the right answer.
    if at_topline && skipcol > ColNr::ZERO && wcol >= skipcol.get() {
        if skipcol <= ColNr::new(width1) {
            wcol -= width2;
        } else {
            wcol -= width2 * ((skipcol - width1).get() / width2 + 1);
        }
        did_sub_skipcol = true;
    }
    if wcol >= view_width {
        let n = wrap_rowoff(ColNr::new(wcol), view_width, width2);
        wcol -= n * width2;
        wrow += n;
    }
    (wcol, wrow, did_sub_skipcol)
}

/// The `w_leftcol` a horizontal scroll should move to, or `None` when the
/// cursor is far enough from both edges. From `curs_columns()`'s
/// no-wrap-may-scroll arm; the caller compares against the current value
/// before writing it.
///
/// `startcol`/`endcol` span the cursor's character, `wcol` is its screen
/// column with `extra` (the non-text width) still included.
#[allow(clippy::too_many_arguments)]
pub fn sidescroll_leftcol(
    startcol: ColNr,
    endcol: ColNr,
    leftcol: ColNr,
    wcol: c_int,
    extra: c_int,
    view_width: c_int,
    width1: c_int,
    siso: int64_t,
    sidescroll: OptInt,
) -> Option<c_int> {
    let off_left = int64_t::from(startcol - leftcol) - siso;
    let off_right = int64_t::from(endcol - leftcol) - (int64_t::from(view_width) - siso) + 1;
    if off_left >= 0 && off_right <= 0 {
        return None;
    }
    let mut diff = if off_left < 0 { -off_left } else { off_right };
    // Far off, or not enough room on either side: put the cursor in the
    // middle of the window.
    let new_leftcol =
        if sidescroll == 0 || diff >= int64_t::from(width1 / 2) || off_right >= off_left {
            wcol - extra - width1 / 2
        } else {
            if diff < sidescroll {
                debug_assert!(sidescroll <= OptInt::from(c_int::MAX), "p_ss <= INT_MAX");
                diff = sidescroll;
            }
            (if off_left < 0 {
                leftcol - number_as_int(diff)
            } else {
                leftcol + number_as_int(diff)
            })
            .get()
        };
    Some(if new_leftcol > 0 { new_leftcol } else { 0 })
}

/// The `w_skipcol` that shows the text around the cursor when a single line
/// is taller than the window. From `curs_columns()`'s tall-line arm.
///
/// `plines` is the line's height in screen lines, *before* the C's `plines--`.
#[allow(clippy::too_many_arguments)]
pub fn skipcol_for_tall_line(
    skipcol: ColNr,
    virtcol: ColNr,
    so: int64_t,
    width1: c_int,
    width2: c_int,
    view_height: c_int,
    wrow: c_int,
    plines: c_int,
) -> ColNr {
    // 1: less than 'scrolloff' lines above; 2: less than 'scrolloff' below;
    // 3: both.
    let mut extra = 0;
    if i64::from(skipcol) + so * int64_t::from(width2) > i64::from(virtcol) {
        extra = 1;
    }
    let plines = plines - 1;
    // The last display line of the buffer line we want at the bottom.
    let n = if int64_t::from(plines) > int64_t::from(wrow) + so {
        debug_assert!(
            int64_t::from(wrow) + so <= int64_t::from(c_int::MAX),
            "wp->w_wrow + so <= INT_MAX"
        );
        number_as_int(int64_t::from(wrow) + so)
    } else {
        plines
    };
    if int64_t::from(n) >= int64_t::from(view_height + skipcol.get() / width2) - so {
        extra += 2;
    }

    if extra == 3 || int64_t::from(view_height) <= so * 2 {
        // Not enough room for 'scrolloff' either way: centre the cursor.
        let mut n = virtcol.get() / width2;
        if n > view_height / 2 {
            n -= view_height / 2;
        } else {
            n = 0;
        }
        // Don't skip more than necessary.
        if n > plines - view_height + 1 {
            n = plines - view_height + 1;
        }
        if n > 0 {
            ColNr::new(width1 + (n - 1) * width2)
        } else {
            ColNr::ZERO
        }
    } else if extra == 1 {
        debug_assert!(so <= int64_t::from(c_int::MAX), "so <= INT_MAX");
        let mut back = number_as_int(
            (i64::from(skipcol) + so * int64_t::from(width2) - i64::from(virtcol)
                + int64_t::from(width2)
                - 1)
                / int64_t::from(width2),
        );
        if back > 0 {
            if back * width2 > skipcol.get() {
                back = skipcol.get() / width2;
            }
            skipcol - back * width2
        } else {
            skipcol
        }
    } else if extra == 2 {
        let mut endcol = (n - view_height + 1) * width2;
        while endcol > virtcol.get() {
            endcol -= width2;
        }
        if skipcol > ColNr::new(endcol) {
            skipcol
        } else {
            ColNr::new(endcol)
        }
    } else {
        skipcol
    }
}

/// Pull the cursor's screen row back into the window after `w_skipcol`
/// changed, raising `w_skipcol` further if the window is too small to hold it.
/// Answers the final `(skipcol, wrow)` and the number of screen lines the
/// window's contents moved by, which `win_scroll_lines()` wants.
///
/// From the tail of `curs_columns()`'s tall-line arm.
pub fn fit_skipcol_to_window(
    skipcol: ColNr,
    prev_skipcol: ColNr,
    wrow: c_int,
    did_sub_skipcol: bool,
    width2: c_int,
    view_height: c_int,
) -> (ColNr, c_int, c_int) {
    let mut skipcol = skipcol;
    let mut wrow = wrow;
    if did_sub_skipcol {
        wrow -= (skipcol - prev_skipcol) / width2;
    } else {
        wrow -= skipcol.get() / width2;
    }
    // A small window: make sure the cursor is still in it.
    if wrow >= view_height {
        let over = wrow - view_height + 1;
        skipcol += over * width2;
        wrow -= over;
    }
    // Either sign: the window may have scrolled back as well as forward.
    (skipcol, wrow, (prev_skipcol - skipcol) / width2)
}
