//! The `i"`/`a'` objects, and the quote scan they are built on.
//!
//! [`find_next_quote`] and [`find_prev_quote`] walk one line looking for an
//! unescaped `quotechar`, where "escaped" is decided by 'quoteescape'.
//! [`current_quote`] is the bookkeeping around them: which of the two quotes
//! the cursor is nearest, whether the current Visual selection already sits
//! inside a quoted string, and how 'selection' shifts both ends.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::*;
use crate::ascii::ascii_iswhite;
use crate::cursor::{dec_cursor, get_cursor_line_ptr, inc_cursor};
use crate::drawscreen::{UPD_INVERTED, redraw_curbuf_later};
use crate::main::{curbuf, curwin, p_sel, redraw_cmdline};
use crate::mbyte::{utf_head_off, utfc_ptr2len};
use crate::memline::dec;
use crate::normal::{
    VisualMode, set_visual_anchor, set_visual_mode, visual_active, visual_anchor, visual_mode,
    with_visual_anchor,
};
use crate::pos::{equalpos, lt};
use crate::strings::vim_strchr;
use crate::types::{NUL, colnr_T, oparg_T};

/// The column of the next `quotechar` at or after `col`, or -1 when there is
/// none before the end of the line.
///
/// A character named in `escape` ('quoteescape') takes the one after it out
/// of consideration, whatever it is.
///
/// # Safety
/// `line` must be NUL-terminated; `escape` must be null or NUL-terminated.
unsafe fn find_next_quote(
    line: *mut c_char,
    mut col: c_int,
    quotechar: c_int,
    escape: *mut c_char,
) -> c_int {
    unsafe {
        loop {
            let c = *line.offset(col as isize) as u8 as c_int;
            if c == NUL {
                return -1;
            }
            if !escape.is_null() && !vim_strchr(escape, c).is_null() {
                col += 1;
                if *line.offset(col as isize) as c_int == NUL {
                    return -1;
                }
            } else if c == quotechar {
                return col;
            }
            col += utfc_ptr2len(line.offset(col as isize));
        }
    }
}

/// The column of the last `quotechar` before `col_start`, or zero when there
/// is none.
///
/// An *odd* run of 'quoteescape' characters in front of a quote escapes it;
/// an even one is escaped escapes, and the quote counts.
///
/// # Safety
/// `line` must be NUL-terminated; `escape` must be null or NUL-terminated.
unsafe fn find_prev_quote(
    line: *mut c_char,
    mut col_start: c_int,
    quotechar: c_int,
    escape: *mut c_char,
) -> c_int {
    unsafe {
        while col_start > 0 {
            col_start -= 1;
            col_start -= utf_head_off(line, line.offset(col_start as isize));
            let mut n = 0;
            if !escape.is_null() {
                while col_start - n > 0
                    && !vim_strchr(
                        escape,
                        *line.offset((col_start - n - 1) as isize) as u8 as c_int,
                    )
                    .is_null()
                {
                    n += 1;
                }
            }
            if n & 1 != 0 {
                col_start -= n; // an odd number of escapes: skip the quote
            } else if *line.offset(col_start as isize) as u8 as c_int == quotechar {
                break;
            }
        }
        col_start
    }
}

/// Where the quoted string enclosing the cursor begins and ends, as byte
/// columns of `line`. `None` is upstream's `abort_search`.
///
/// Which of the four ways this is asked depends on what is already selected
/// and on whether the cursor sits on a quote -- the cursor alone cannot say
/// whether a quote it is on opens or closes a string, which is why the third
/// arm rescans from column zero.
///
/// # Safety
/// `line` must be the current line, NUL-terminated.
unsafe fn quoted_span(
    line: *mut c_char,
    mut col_start: c_int,
    quotechar: c_int,
    vis_empty: bool,
    vis_bef_curs: bool,
) -> Option<(c_int, c_int)> {
    unsafe {
        let qe = (*curbuf.get()).b_p_qe;
        let on_quote = *line.offset(col_start as isize) as u8 as c_int == quotechar;
        let col_end;

        if !vis_empty && on_quote {
            // Something is already selected and the cursor is on a quote:
            // find the *next* quoted string.
            if vis_bef_curs {
                // Assume this is a closing quote: move past the next opening
                // one.
                col_start = find_next_quote(
                    line,
                    col_start + 1,
                    quotechar,
                    ::core::ptr::null_mut::<c_char>(),
                );
                if col_start < 0 {
                    return None;
                }
                let mut end = find_next_quote(line, col_start + 1, quotechar, qe);
                if end < 0 {
                    // It was a starting quote after all.
                    end = col_start;
                    col_start = (*curwin.get()).w_cursor.col as c_int;
                }
                col_end = end;
            } else {
                let mut end = find_prev_quote(
                    line,
                    col_start,
                    quotechar,
                    ::core::ptr::null_mut::<c_char>(),
                );
                if *line.offset(end as isize) as u8 as c_int != quotechar {
                    return None;
                }
                col_start = find_prev_quote(line, end, quotechar, qe);
                if *line.offset(col_start as isize) as u8 as c_int != quotechar {
                    // It was an ending quote after all.
                    col_start = end;
                    end = (*curwin.get()).w_cursor.col as c_int;
                }
                col_end = end;
            }
        } else if on_quote || !vis_empty {
            // The cursor is on a quote and there is no telling whether it
            // opens or closes a string, so rescan from the start of the line.
            // Also done with a Visual area, since `a'` can leave the cursor
            // between two strings.
            let mut first_col = col_start;
            if !vis_empty {
                first_col = if vis_bef_curs {
                    find_next_quote(
                        line,
                        col_start,
                        quotechar,
                        ::core::ptr::null_mut::<c_char>(),
                    )
                } else {
                    find_prev_quote(
                        line,
                        col_start,
                        quotechar,
                        ::core::ptr::null_mut::<c_char>(),
                    )
                };
            }
            col_start = 0;
            loop {
                col_start = find_next_quote(
                    line,
                    col_start,
                    quotechar,
                    ::core::ptr::null_mut::<c_char>(),
                );
                if col_start < 0 || col_start > first_col {
                    return None;
                }
                let end = find_next_quote(line, col_start + 1, quotechar, qe);
                if end < 0 {
                    return None;
                }
                if col_start <= first_col && first_col <= end {
                    col_end = end;
                    break;
                }
                col_start = end + 1;
            }
        } else {
            // Search backwards for an opening quote.
            col_start = find_prev_quote(line, col_start, quotechar, qe);
            if *line.offset(col_start as isize) as u8 as c_int != quotechar {
                // None before the cursor: look after it.
                col_start = find_next_quote(
                    line,
                    col_start,
                    quotechar,
                    ::core::ptr::null_mut::<c_char>(),
                );
                if col_start < 0 {
                    return None;
                }
            }
            let end = find_next_quote(line, col_start + 1, quotechar, qe);
            if end < 0 {
                return None;
            }
            col_end = end;
        }
        Some((col_start, col_end))
    }
}

/// Swap the cursor and the Visual anchor, so the anchor is the earlier end.
///
/// # Safety
/// The current window must be live.
unsafe fn swap_cursor_and_anchor() {
    // SAFETY: the caller's promise; `visual_anchor` reads no window state.
    let cursor = unsafe { core::mem::replace(&mut (*curwin.get()).w_cursor, visual_anchor()) };
    set_visual_anchor(cursor);
}

/// `i"` / `a'` / ... : the text inside the quoted string under the cursor,
/// cursor left at the end. Answers whether one was found.
///
/// # Safety
/// `oap` must be a live operator argument, and there must be a current line.
pub unsafe fn current_quote(
    oap: *mut oparg_T,
    count: c_int,
    include: bool,
    quotechar: c_int,
) -> bool {
    unsafe {
        let line = get_cursor_line_ptr();
        let mut col_start = (*curwin.get()).w_cursor.col as c_int;
        let mut inclusive = false;
        let mut vis_empty = true; // the Visual selection is one character or less
        let mut vis_bef_curs = false; // the Visual area starts before the cursor
        let mut did_exclusive_adj = false; // the position was adjusted for 'selection'
        let mut inside_quotes = false; // looks like an `i'` was done before
        let mut selected_quote = false; // a quote is inside the selection
        let mut restore_vis_bef = false; // put `VIsual` back if this aborts

        // With 'selection' "exclusive", move the cursor to where it would be
        // with "inclusive" so that the rest of this is written once; it is
        // moved forward again after the area has been adjusted.
        if visual_active() {
            // This only works within one line.
            if visual_anchor().lnum != (*curwin.get()).w_cursor.lnum {
                return false;
            }
            vis_bef_curs = lt(visual_anchor(), (*curwin.get()).w_cursor);
            vis_empty = equalpos(visual_anchor(), (*curwin.get()).w_cursor);
            if *p_sel.get() as c_int == 'e' as c_int {
                if vis_bef_curs {
                    dec_cursor();
                    did_exclusive_adj = true;
                } else if !vis_empty {
                    with_visual_anchor(|anchor| dec(anchor));
                    did_exclusive_adj = true;
                }
                vis_empty = equalpos(visual_anchor(), (*curwin.get()).w_cursor);
                if !vis_bef_curs && !vis_empty {
                    // `VIsual` has to be the start of the selection.
                    swap_cursor_and_anchor();
                    vis_bef_curs = true;
                    restore_vis_bef = true;
                }
            }
        }

        if !vis_empty {
            // Does the existing selection span exactly the text inside a pair
            // of quotes?
            let mut i;
            let sel_end;
            if vis_bef_curs {
                inside_quotes = visual_anchor().col > 0
                    && *line.offset(visual_anchor().col as isize - 1) as u8 as c_int == quotechar
                    && *line.offset((*curwin.get()).w_cursor.col as isize) as c_int != NUL
                    && *line.offset((*curwin.get()).w_cursor.col as isize + 1) as u8 as c_int
                        == quotechar;
                i = visual_anchor().col as c_int;
                sel_end = (*curwin.get()).w_cursor.col as c_int;
            } else {
                inside_quotes = (*curwin.get()).w_cursor.col > 0
                    && *line.offset((*curwin.get()).w_cursor.col as isize - 1) as u8 as c_int
                        == quotechar
                    && *line.offset(visual_anchor().col as isize) as c_int != NUL
                    && *line.offset(visual_anchor().col as isize + 1) as u8 as c_int == quotechar;
                i = (*curwin.get()).w_cursor.col as c_int;
                sel_end = visual_anchor().col as c_int;
            }
            // Is there a quote in the selection at all?
            while i <= sel_end {
                // The line may have been changed since the Visual area was
                // selected, so this can run off the end of it.
                if *line.offset(i as isize) as c_int == NUL {
                    break;
                }
                let c = *line.offset(i as isize) as u8 as c_int;
                i += 1;
                if c == quotechar {
                    selected_quote = true;
                    break;
                }
            }
        }

        let Some((start, mut col_end)) =
            quoted_span(line, col_start, quotechar, vis_empty, vis_bef_curs)
        else {
            // `abort_search`: undo the 'selection' adjustment made above.
            if visual_active() && *p_sel.get() as c_int == 'e' as c_int {
                if did_exclusive_adj {
                    inc_cursor();
                }
                if restore_vis_bef {
                    swap_cursor_and_anchor();
                }
            }
            return false;
        };
        col_start = start;

        // With `include`, take the white space after the closing quote, or --
        // when there is none there -- the white space before the opening one.
        if include {
            if ascii_iswhite(*line.offset(col_end as isize + 1) as c_int) {
                while ascii_iswhite(*line.offset(col_end as isize + 1) as c_int) {
                    col_end += 1;
                }
            } else {
                while col_start > 0 && ascii_iswhite(*line.offset(col_start as isize - 1) as c_int)
                {
                    col_start -= 1;
                }
            }
        }

        // The start position. After a `vi"`, another `i"` must take the quotes
        // in; so must `v2i"`.
        if !include && count < 2 && (vis_empty || !inside_quotes) {
            col_start += 1;
        }
        (*curwin.get()).w_cursor.col = col_start as colnr_T;
        if visual_active() {
            // Set the start of the Visual area when it was empty, when we
            // were just inside quotes, or when it neither started at a quote
            // nor contained one.
            let anchor_col = visual_anchor().col;
            if vis_empty
                || (vis_bef_curs
                    && !selected_quote
                    && (inside_quotes
                        || (*line.offset(anchor_col as isize) as u8 as c_int != quotechar
                            && (anchor_col == 0
                                || *line.offset(anchor_col as isize - 1) as u8 as c_int
                                    != quotechar))))
            {
                set_visual_anchor((*curwin.get()).w_cursor);
                redraw_curbuf_later(UPD_INVERTED);
            }
        } else {
            (*oap).start = (*curwin.get()).w_cursor;
            (*oap).motion_type = kMTCharWise;
        }

        // The end position.
        (*curwin.get()).w_cursor.col = col_end as colnr_T;
        if (include || count > 1 || (!vis_empty && inside_quotes)) && inc_cursor() == 2 {
            inclusive = true;
        }
        if visual_active() {
            if vis_empty || vis_bef_curs {
                // Step the cursor back when 'selection' is not exclusive.
                if *p_sel.get() as c_int != 'e' as c_int {
                    dec_cursor();
                }
            } else {
                // The cursor is at the start of the Visual area. Set its end
                // when we were just inside quotes, or when it did not end at
                // a quote.
                if inside_quotes
                    || (!selected_quote
                        && *line.offset(visual_anchor().col as isize) as u8 as c_int != quotechar
                        && (*line.offset(visual_anchor().col as isize) as c_int == NUL
                            || *line.offset(visual_anchor().col as isize + 1) as u8 as c_int
                                != quotechar))
                {
                    dec_cursor();
                    set_visual_anchor((*curwin.get()).w_cursor);
                }
                (*curwin.get()).w_cursor.col = col_start as colnr_T;
            }
            if visual_mode().is_line() {
                set_visual_mode(VisualMode::CHAR);
                redraw_cmdline.set(true); // show the mode later
            }
        } else {
            (*oap).inclusive = inclusive;
        }
        true
    }
}
