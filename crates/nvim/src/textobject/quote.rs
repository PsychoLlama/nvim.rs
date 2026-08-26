//! The `i"`/`a'` objects, and the quote scan they are built on.
//!
//! [`find_next_quote`] and [`find_prev_quote`] walk one line looking for an
//! unescaped `quotechar`, where "escaped" is decided by 'quoteescape'.
//! [`current_quote`] is the bookkeeping around them: which of the two quotes
//! the cursor is nearest, whether the current Visual selection already sits
//! inside a quoted string, and how 'selection' shifts both ends.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};

use super::*;
use crate::ascii::ascii_iswhite;
use crate::cursor::{dec_cursor, get_cursor_line_ptr, inc_cursor};
use crate::drawscreen::{UPD_INVERTED, redraw_curbuf_later};
use crate::main::{p_sel, redraw_cmdline};
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
    loop {
        // SAFETY: `line` is NUL-terminated and `col` is a column of it: the
        // walk stops at the first NUL, and every step past a byte is the
        // `utfc_ptr2len` of the character just read there.
        let c = unsafe { *line.offset(col as isize) } as u8 as c_int;
        if c == NUL {
            return -1;
        }
        // SAFETY: `escape` is null or NUL-terminated; the `&&` guards the
        // call and is left whole.
        if !escape.is_null() && unsafe { !vim_strchr(escape, c).is_null() } {
            col += 1;
            // SAFETY: `c` was not the NUL, so `col` is still within the line.
            if unsafe { *line.offset(col as isize) } as c_int == NUL {
                return -1;
            }
        } else if c == quotechar {
            return col;
        }
        // SAFETY: as above.
        col += unsafe { utfc_ptr2len(line.offset(col as isize)) };
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
    while col_start > 0 {
        col_start -= 1;
        // SAFETY: `line` is NUL-terminated and `col_start` is a column of it,
        // so `utf_head_off` only ever walks back towards `line`.
        col_start -= unsafe { utf_head_off(line, line.offset(col_start as isize)) };
        let mut n = 0;
        if !escape.is_null() {
            // SAFETY: `escape` is NUL-terminated, and the byte read is at a
            // column kept at or above zero by the first half of the `&&` --
            // the chain is the proof and is left whole.
            while unsafe {
                col_start - n > 0
                    && !vim_strchr(
                        escape,
                        *line.offset((col_start - n - 1) as isize) as u8 as c_int,
                    )
                    .is_null()
            } {
                n += 1;
            }
        }
        if n & 1 != 0 {
            col_start -= n; // an odd number of escapes: skip the quote
        // SAFETY: `col_start` is still a column of `line`.
        } else if unsafe { *line.offset(col_start as isize) } as u8 as c_int == quotechar {
            break;
        }
    }
    col_start
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
    let qe = cur_buf().b_p_qe;
    // SAFETY, for all five: the caller guarantees `line` is the current line
    // and so NUL-terminated, and 'quoteescape' is a NUL-terminated option
    // string -- between them that is everything the two searches ask of their
    // arguments.  Every column handed to them below either came out of one of
    // them or is the cursor's, so it is a column of `line`.
    let next = |col| unsafe { find_next_quote(line, col, quotechar, ::core::ptr::null_mut()) };
    let next_esc = |col| unsafe { find_next_quote(line, col, quotechar, qe) };
    let prev = |col| unsafe { find_prev_quote(line, col, quotechar, ::core::ptr::null_mut()) };
    let prev_esc = |col| unsafe { find_prev_quote(line, col, quotechar, qe) };
    let at = |col: c_int| unsafe { *line.offset(col as isize) };

    let on_quote = at(col_start) as u8 as c_int == quotechar;
    let col_end;

    if !vis_empty && on_quote {
        // Something is already selected and the cursor is on a quote:
        // find the *next* quoted string.
        if vis_bef_curs {
            // Assume this is a closing quote: move past the next opening one.
            col_start = next(col_start + 1);
            if col_start < 0 {
                return None;
            }
            let mut end = next_esc(col_start + 1);
            if end < 0 {
                // It was a starting quote after all.
                end = col_start;
                col_start = cur_win().w_cursor.col as c_int;
            }
            col_end = end;
        } else {
            let mut end = prev(col_start);
            if at(end) as u8 as c_int != quotechar {
                return None;
            }
            col_start = prev_esc(end);
            if at(col_start) as u8 as c_int != quotechar {
                // It was an ending quote after all.
                col_start = end;
                end = cur_win().w_cursor.col as c_int;
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
                next(col_start)
            } else {
                prev(col_start)
            };
        }
        col_start = 0;
        loop {
            col_start = next(col_start);
            if col_start < 0 || col_start > first_col {
                return None;
            }
            let end = next_esc(col_start + 1);
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
        col_start = prev_esc(col_start);
        if at(col_start) as u8 as c_int != quotechar {
            // None before the cursor: look after it.
            col_start = next(col_start);
            if col_start < 0 {
                return None;
            }
        }
        let end = next_esc(col_start + 1);
        if end < 0 {
            return None;
        }
        col_end = end;
    }
    Some((col_start, col_end))
}

/// Swap the cursor and the Visual anchor, so the anchor is the earlier end.
fn swap_cursor_and_anchor() {
    let cursor = core::mem::replace(&mut *cur_win().cursor(), visual_anchor());
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
    // SAFETY: the caller guarantees a current line; `get_cursor_line_ptr`
    // hands back the cursor's line, NUL-terminated.
    let line = unsafe { get_cursor_line_ptr() };
    // SAFETY: `line` is NUL-terminated.  Every column read through this is
    // either one of `line`'s own or one just past a byte already found
    // non-NUL, so the reads stay inside it -- the `&&` chains that prove it
    // are left whole below.
    let at = |col: c_int| unsafe { *line.offset(col as isize) };
    let mut col_start = cur_win().w_cursor.col as c_int;
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
        if visual_anchor().lnum != cur_win().w_cursor.lnum {
            return false;
        }
        vis_bef_curs = lt(visual_anchor(), cur_win().w_cursor);
        vis_empty = equalpos(visual_anchor(), cur_win().w_cursor);
        // SAFETY: 'selection' is a NUL-terminated option string.
        if unsafe { *p_sel.get() } as c_int == 'e' as c_int {
            if vis_bef_curs {
                // SAFETY: the cursor is on a line of the current buffer.
                unsafe { dec_cursor() };
                did_exclusive_adj = true;
            } else if !vis_empty {
                // SAFETY: the anchor is a position of the current buffer.
                unsafe { with_visual_anchor(|anchor| dec(anchor)) };
                did_exclusive_adj = true;
            }
            vis_empty = equalpos(visual_anchor(), cur_win().w_cursor);
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
                && at(visual_anchor().col - 1) as u8 as c_int == quotechar
                && at(cur_win().w_cursor.col) as c_int != NUL
                && at(cur_win().w_cursor.col + 1) as u8 as c_int == quotechar;
            i = visual_anchor().col as c_int;
            sel_end = cur_win().w_cursor.col as c_int;
        } else {
            inside_quotes = cur_win().w_cursor.col > 0
                && at(cur_win().w_cursor.col - 1) as u8 as c_int == quotechar
                && at(visual_anchor().col) as c_int != NUL
                && at(visual_anchor().col + 1) as u8 as c_int == quotechar;
            i = cur_win().w_cursor.col as c_int;
            sel_end = visual_anchor().col as c_int;
        }
        // Is there a quote in the selection at all?
        while i <= sel_end {
            // The line may have been changed since the Visual area was
            // selected, so this can run off the end of it.
            if at(i) as c_int == NUL {
                break;
            }
            let c = at(i) as u8 as c_int;
            i += 1;
            if c == quotechar {
                selected_quote = true;
                break;
            }
        }
    }

    // SAFETY: `line` is the current line, NUL-terminated.
    let Some((start, mut col_end)) =
        (unsafe { quoted_span(line, col_start, quotechar, vis_empty, vis_bef_curs) })
    else {
        // `abort_search`: undo the 'selection' adjustment made above.
        // SAFETY: 'selection' is a NUL-terminated option string.
        if visual_active() && unsafe { *p_sel.get() } as c_int == 'e' as c_int {
            if did_exclusive_adj {
                // SAFETY: the cursor is on a line of the current buffer.
                unsafe { inc_cursor() };
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
        if ascii_iswhite(at(col_end + 1) as c_int) {
            while ascii_iswhite(at(col_end + 1) as c_int) {
                col_end += 1;
            }
        } else {
            while col_start > 0 && ascii_iswhite(at(col_start - 1) as c_int) {
                col_start -= 1;
            }
        }
    }

    // The start position. After a `vi"`, another `i"` must take the quotes
    // in; so must `v2i"`.
    if !include && count < 2 && (vis_empty || !inside_quotes) {
        col_start += 1;
    }
    cur_win().w_cursor.col = col_start as colnr_T;
    if visual_active() {
        // Set the start of the Visual area when it was empty, when we
        // were just inside quotes, or when it neither started at a quote
        // nor contained one.
        let anchor_col = visual_anchor().col;
        if vis_empty
            || (vis_bef_curs
                && !selected_quote
                && (inside_quotes
                    || (at(anchor_col) as u8 as c_int != quotechar
                        && (anchor_col == 0 || at(anchor_col - 1) as u8 as c_int != quotechar))))
        {
            set_visual_anchor(cur_win().w_cursor);
            // SAFETY: on the main thread with a current buffer.
            unsafe { redraw_curbuf_later(UPD_INVERTED) };
        }
    } else {
        // SAFETY: the caller guarantees `oap` is a live operator argument.
        let oap = unsafe { &mut *oap };
        oap.start = cur_win().w_cursor;
        oap.motion_type = kMTCharWise;
    }

    // The end position.
    cur_win().w_cursor.col = col_end as colnr_T;
    // SAFETY: the cursor is on a line of the current buffer; the `&&` keeps
    // `inc_cursor`'s side effect behind the same test it had.
    if (include || count > 1 || (!vis_empty && inside_quotes)) && unsafe { inc_cursor() } == 2 {
        inclusive = true;
    }
    if visual_active() {
        if vis_empty || vis_bef_curs {
            // Step the cursor back when 'selection' is not exclusive.
            // SAFETY: 'selection' is a NUL-terminated option string, and the
            // cursor is on a line of the current buffer.
            if unsafe { *p_sel.get() } as c_int != 'e' as c_int {
                unsafe { dec_cursor() };
            }
        } else {
            // The cursor is at the start of the Visual area. Set its end
            // when we were just inside quotes, or when it did not end at
            // a quote.
            if inside_quotes
                || (!selected_quote
                    && at(visual_anchor().col) as u8 as c_int != quotechar
                    && (at(visual_anchor().col) as c_int == NUL
                        || at(visual_anchor().col + 1) as u8 as c_int != quotechar))
            {
                // SAFETY: the cursor is on a line of the current buffer.
                unsafe { dec_cursor() };
                set_visual_anchor(cur_win().w_cursor);
            }
            cur_win().w_cursor.col = col_start as colnr_T;
        }
        if visual_mode().is_line() {
            set_visual_mode(VisualMode::CHAR);
            redraw_cmdline.set(true); // show the mode later
        }
    } else {
        // SAFETY: the caller guarantees `oap` is a live operator argument.
        unsafe { (*oap).inclusive = inclusive };
    }
    true
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
