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
//! keeps that arm's integer widths: the C mixes `int`, `colnr_T` and
//! `int64_t` deliberately, and where a subtraction is done narrow before it
//! is widened, so is it here.
//!
//! Original: `src/nvim/move.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use crate::src::nvim::types::{OptInt, colnr_T, int64_t};

/// Screen lines of the top line that `w_skipcol` scrolls out of sight.
///
/// `width1` is the text width of a line's first screen line and `width2` that
/// of its later ones. From `adjust_plines_for_skipcol()`.
pub fn skipped_plines(skipcol: colnr_T, width1: c_int, width2: c_int) -> c_int {
    if skipcol == 0 {
        return 0;
    }
    if skipcol >= width1 && width2 > 0 {
        (skipcol - width1) / width2 + 1
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

/// Lines a scroll moves at a minimum: 'scrolljump' as a count when positive,
/// as a percentage of the window height when negative.
pub fn scrolljump_lines(scrolljump: OptInt, view_height: c_int) -> c_int {
    if scrolljump >= 0 {
        scrolljump as c_int
    } else {
        view_height * (-scrolljump) as c_int / 100
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
pub fn wrap_rowoff(col: colnr_T, view_width: c_int, width: c_int) -> c_int {
    (col - view_width) / width + 1
}

/// The window column the cursor's virtual column shows at: `w_wcol` as
/// `validate_cursor_col()` computes it.
///
/// `off` is the non-text width on the left, `width` the text width of a
/// wrapped line's later screen lines.
pub fn cursor_screen_col(
    virtcol: colnr_T,
    off: colnr_T,
    view_width: c_int,
    width: c_int,
    wrap: bool,
    leftcol: colnr_T,
) -> c_int {
    let mut col = virtcol + off;
    // Long line wrapping: bring the column back onto its own screen line.
    if wrap && col >= view_width && width > 0 {
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
    skipcol: colnr_T,
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
    if at_topline && skipcol > 0 && wcol >= skipcol {
        if skipcol <= width1 {
            wcol -= width2;
        } else {
            wcol -= width2 * ((skipcol - width1) / width2 + 1);
        }
        did_sub_skipcol = true;
    }
    if wcol >= view_width {
        let n = wrap_rowoff(wcol, view_width, width2);
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
    startcol: colnr_T,
    endcol: colnr_T,
    leftcol: colnr_T,
    wcol: c_int,
    extra: c_int,
    view_width: c_int,
    width1: c_int,
    siso: int64_t,
    sidescroll: OptInt,
) -> Option<c_int> {
    let off_left = (startcol - leftcol) as int64_t - siso;
    let off_right = (endcol - leftcol) as int64_t - (view_width as int64_t - siso) + 1;
    if off_left >= 0 && off_right <= 0 {
        return None;
    }
    let mut diff = if off_left < 0 { -off_left } else { off_right };
    // Far off, or not enough room on either side: put the cursor in the
    // middle of the window.
    let new_leftcol = if sidescroll == 0 || diff >= (width1 / 2) as int64_t || off_right >= off_left
    {
        wcol - extra - width1 / 2
    } else {
        if diff < sidescroll {
            debug_assert!(sidescroll <= c_int::MAX as OptInt, "p_ss <= INT_MAX");
            diff = sidescroll;
        }
        if off_left < 0 {
            leftcol - diff as c_int
        } else {
            leftcol + diff as c_int
        }
    };
    Some(if new_leftcol > 0 { new_leftcol } else { 0 })
}

/// The `w_skipcol` that shows the text around the cursor when a single line
/// is taller than the window. From `curs_columns()`'s tall-line arm.
///
/// `plines` is the line's height in screen lines, *before* the C's `plines--`.
#[allow(clippy::too_many_arguments)]
pub fn skipcol_for_tall_line(
    skipcol: colnr_T,
    virtcol: colnr_T,
    so: int64_t,
    width1: c_int,
    width2: c_int,
    view_height: c_int,
    wrow: c_int,
    plines: c_int,
) -> colnr_T {
    // 1: less than 'scrolloff' lines above; 2: less than 'scrolloff' below;
    // 3: both.
    let mut extra = 0;
    if skipcol as int64_t + so * width2 as int64_t > virtcol as int64_t {
        extra = 1;
    }
    let plines = plines - 1;
    // The last display line of the buffer line we want at the bottom.
    let n = if plines as int64_t > wrow as int64_t + so {
        debug_assert!(
            wrow as int64_t + so <= c_int::MAX as int64_t,
            "wp->w_wrow + so <= INT_MAX"
        );
        (wrow as int64_t + so) as c_int
    } else {
        plines
    };
    if n as int64_t >= (view_height + skipcol / width2) as int64_t - so {
        extra += 2;
    }

    if extra == 3 || view_height as int64_t <= so * 2 {
        // Not enough room for 'scrolloff' either way: centre the cursor.
        let mut n = virtcol / width2;
        if n > view_height / 2 {
            n -= view_height / 2;
        } else {
            n = 0;
        }
        // Don't skip more than necessary.
        if n > plines - view_height + 1 {
            n = plines - view_height + 1;
        }
        if n > 0 { width1 + (n - 1) * width2 } else { 0 }
    } else if extra == 1 {
        debug_assert!(so <= c_int::MAX as int64_t, "so <= INT_MAX");
        let mut back = ((skipcol as int64_t + so * width2 as int64_t - virtcol as int64_t
            + width2 as int64_t
            - 1)
            / width2 as int64_t) as c_int;
        if back > 0 {
            if back * width2 > skipcol {
                back = skipcol / width2;
            }
            skipcol - back * width2
        } else {
            skipcol
        }
    } else if extra == 2 {
        let mut endcol = (n - view_height + 1) * width2;
        while endcol > virtcol {
            endcol -= width2;
        }
        if skipcol > endcol { skipcol } else { endcol }
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
    skipcol: colnr_T,
    prev_skipcol: colnr_T,
    wrow: c_int,
    did_sub_skipcol: bool,
    width2: c_int,
    view_height: c_int,
) -> (colnr_T, c_int, c_int) {
    let mut skipcol = skipcol;
    let mut wrow = wrow;
    if did_sub_skipcol {
        wrow -= (skipcol - prev_skipcol) / width2;
    } else {
        wrow -= skipcol / width2;
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
