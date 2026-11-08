//! The `i"`/`a'` objects, and the quote scan they are built on.
//!
//! [`find_next_quote`] and [`find_prev_quote`] walk one line looking for an
//! unescaped `quotechar`, where "escaped" is decided by 'quoteescape'.
//! [`current_quote`] is the bookkeeping around them: which of the two quotes
//! the cursor is nearest, whether the current Visual selection already sits
//! inside a quoted string, and how 'selection' shifts both ends.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr::byte_at;
use crate::memline::Lines;
use crate::narrow::len_as_int;
use crate::option::vars::P_SEL;
use crate::regexp::low_byte;
use crate::strings::find_char;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::ascii::ascii_iswhite;
use crate::cursor::{dec_cursor, inc_cursor};
use crate::drawscreen::state::redraw_cmdline;
use crate::drawscreen::{UPD_INVERTED, redraw_curbuf_later};
use crate::mbyte::{c_cluster_len, head_off};
use crate::memline::{Moved, dec};
use crate::normal::defs::OpArg;
use crate::normal::{
    VisualMode, set_visual_anchor, set_visual_mode, visual_active, visual_anchor, visual_mode,
    with_visual_anchor,
};
use crate::optionstr::OptString;
use crate::pos::{equalpos, lt};
use crate::types::{ColNr, NUL};

/// The byte at column `col` of `line`; NUL past its end and before its
/// start, which every walk below stops at.
fn byte_of(line: &[u8], col: c_int) -> u8 {
    usize::try_from(col).map_or(0, |i| byte_at(line, i))
}

/// Column `col` as an index: the walks never step before column zero.
fn index(col: c_int) -> usize {
    usize::try_from(col).expect("a column on the line")
}

/// The column of the next `quotechar` at or after `col`; `None` when there
/// is none before the end of the line.
///
/// A character named in `escape` ('quoteescape') takes the one after it out
/// of consideration, whatever it is.
fn find_next_quote(
    line: &[u8],
    mut col: c_int,
    quotechar: c_int,
    escape: Option<&[u8]>,
) -> Option<c_int> {
    let at = |col: c_int| byte_of(line, col);
    loop {
        let c = c_int::from(at(col));
        if c == NUL {
            return None;
        }
        if escape.is_some_and(|escape| find_char(escape, c).is_some()) {
            col += 1;
            if at(col) == 0 {
                return None;
            }
        } else if c == quotechar {
            return Some(col);
        }
        col += len_as_int(c_cluster_len(&line[index(col)..]));
    }
}

/// The column of the last `quotechar` before `col_start`, or zero when there
/// is none.
///
/// An *odd* run of 'quoteescape' characters in front of a quote escapes it;
/// an even one is escaped escapes, and the quote counts.
fn find_prev_quote(
    line: &[u8],
    mut col_start: c_int,
    quotechar: c_int,
    escape: Option<&[u8]>,
) -> c_int {
    let at = |col: c_int| c_int::from(byte_of(line, col));
    while col_start > 0 {
        col_start -= 1;
        col_start -= len_as_int(head_off(line, index(col_start)));
        let mut n = 0;
        if let Some(escape) = escape {
            while col_start - n > 0 && find_char(escape, at(col_start - n - 1)).is_some() {
                n += 1;
            }
        }
        if n & 1 != 0 {
            col_start -= n; // an odd number of escapes: skip the quote
        } else if at(col_start) == quotechar {
            break;
        }
    }
    col_start
}

/// Where the quoted string enclosing the cursor begins and ends, as byte
/// columns of `line`, the cursor's line. `None` is upstream's `abort_search`.
///
/// Which of the four ways this is asked depends on what is already selected
/// and on whether the cursor sits on a quote -- the cursor alone cannot say
/// whether a quote it is on opens or closes a string, which is why the third
/// arm rescans from column zero.
fn quoted_span(
    line: &[u8],
    mut col_start: c_int,
    quotechar: c_int,
    vis_empty: bool,
    vis_bef_curs: bool,
) -> Option<(c_int, c_int)> {
    // 'quoteescape', copied: the walks below read it per byte.
    let qe = Buf::current().b_p_qe.get();
    let qe = Some(&qe[..]);
    let next = |col| find_next_quote(line, col, quotechar, None);
    let next_esc = |col| find_next_quote(line, col, quotechar, qe);
    let prev = |col| find_prev_quote(line, col, quotechar, None);
    let prev_esc = |col| find_prev_quote(line, col, quotechar, qe);
    let at = |col: c_int| c_int::from(byte_of(line, col));

    let on_quote = at(col_start) == quotechar;
    let col_end;

    if !vis_empty && on_quote {
        // Something is already selected and the cursor is on a quote:
        // find the *next* quoted string.
        if vis_bef_curs {
            // Assume this is a closing quote: move past the next opening one.
            col_start = next(col_start + 1)?;
            col_end = next_esc(col_start + 1).unwrap_or_else(|| {
                // It was a starting quote after all.
                let end = col_start;
                col_start = Win::current().w_cursor.col.get();
                end
            });
        } else {
            let mut end = prev(col_start);
            if at(end) != quotechar {
                return None;
            }
            col_start = prev_esc(end);
            if at(col_start) != quotechar {
                // It was an ending quote after all.
                col_start = end;
                end = Win::current().w_cursor.col.get();
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
            // No quote after the selection leaves no string to find.
            first_col = if vis_bef_curs {
                next(col_start)?
            } else {
                prev(col_start)
            };
        }
        col_start = 0;
        loop {
            col_start = next(col_start)?;
            if col_start > first_col {
                return None;
            }
            let end = next_esc(col_start + 1)?;
            if col_start <= first_col && first_col <= end {
                col_end = end;
                break;
            }
            col_start = end + 1;
        }
    } else {
        // Search backwards for an opening quote.
        col_start = prev_esc(col_start);
        if at(col_start) != quotechar {
            // None before the cursor: look after it.
            col_start = next(col_start)?;
        }
        col_end = next_esc(col_start + 1)?;
    }
    Some((col_start, col_end))
}

/// Swap the cursor and the Visual anchor, so the anchor is the earlier end.
fn swap_cursor_and_anchor() {
    let cursor = core::mem::replace(&mut *Win::current().cursor(), visual_anchor());
    set_visual_anchor(cursor);
}

/// `i"` / `a'` / ... : the text inside the quoted string under the cursor,
/// cursor left at the end.
pub(crate) fn current_quote(
    op: &mut OpArg,
    count: c_int,
    include: bool,
    quotechar: c_int,
) -> Result<(), NoObject> {
    // A copy of the cursor line: the cursor moves to the next line below
    // when the string ends the line, and the line is still read after that.
    let line = Lines::current().line_copy(Win::current().w_cursor.lnum);
    // Past the end reads as the terminator, which no quote character is.
    let at = |col: c_int| byte_of(&line, col);
    let mut col_start = Win::current().w_cursor.col.get();
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
        if visual_anchor().lnum != Win::current().w_cursor.lnum {
            return Err(NoObject);
        }
        vis_bef_curs = lt(visual_anchor(), Win::current().w_cursor);
        vis_empty = equalpos(visual_anchor(), Win::current().w_cursor);
        if P_SEL.first_byte() == b'e' {
            if vis_bef_curs {
                dec_cursor();
                did_exclusive_adj = true;
            } else if !vis_empty {
                with_visual_anchor(dec);
                did_exclusive_adj = true;
            }
            vis_empty = equalpos(visual_anchor(), Win::current().w_cursor);
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
            inside_quotes = visual_anchor().col > ColNr::ZERO
                && at((visual_anchor().col - 1).get()) == low_byte(quotechar)
                && at(Win::current().w_cursor.col.get()) != 0
                && at((Win::current().w_cursor.col + 1).get()) == low_byte(quotechar);
            i = visual_anchor().col.get();
            sel_end = Win::current().w_cursor.col.get();
        } else {
            inside_quotes = Win::current().w_cursor.col > ColNr::ZERO
                && at((Win::current().w_cursor.col - 1).get()) == low_byte(quotechar)
                && at(visual_anchor().col.get()) != 0
                && at((visual_anchor().col + 1).get()) == low_byte(quotechar);
            i = Win::current().w_cursor.col.get();
            sel_end = visual_anchor().col.get();
        }
        // Is there a quote in the selection at all?
        while i <= sel_end {
            // The line may have been changed since the Visual area was
            // selected, so this can run off the end of it.
            if at(i) == 0 {
                break;
            }
            let c = c_int::from(at(i));
            i += 1;
            if c == quotechar {
                selected_quote = true;
                break;
            }
        }
    }

    let Some((start, mut col_end)) =
        quoted_span(&line, col_start, quotechar, vis_empty, vis_bef_curs)
    else {
        // `abort_search`: undo the 'selection' adjustment made above.
        if visual_active() && P_SEL.first_byte() == b'e' {
            if did_exclusive_adj {
                inc_cursor();
            }
            if restore_vis_bef {
                swap_cursor_and_anchor();
            }
        }
        return Err(NoObject);
    };
    col_start = start;

    // With `include`, take the white space after the closing quote, or --
    // when there is none there -- the white space before the opening one.
    if include {
        if ascii_iswhite(c_int::from(at(col_end + 1))) {
            while ascii_iswhite(c_int::from(at(col_end + 1))) {
                col_end += 1;
            }
        } else {
            while col_start > 0 && ascii_iswhite(c_int::from(at(col_start - 1))) {
                col_start -= 1;
            }
        }
    }

    // The start position. After a `vi"`, another `i"` must take the quotes
    // in; so must `v2i"`.
    if !include && count < 2 && (vis_empty || !inside_quotes) {
        col_start += 1;
    }
    Win::current().w_cursor.col = ColNr::new(col_start);
    if visual_active() {
        // Set the start of the Visual area when it was empty, when we
        // were just inside quotes, or when it neither started at a quote
        // nor contained one.
        let anchor_col = visual_anchor().col;
        if vis_empty
            || (vis_bef_curs
                && !selected_quote
                && (inside_quotes
                    || (at(anchor_col.get()) != low_byte(quotechar)
                        && (anchor_col == ColNr::ZERO
                            || at((anchor_col - 1).get()) != low_byte(quotechar)))))
        {
            set_visual_anchor(Win::current().w_cursor);
            redraw_curbuf_later(UPD_INVERTED);
        }
    } else {
        op.start = Win::current().w_cursor;
        op.motion_type = kMTCharWise;
    }

    // The end position.
    Win::current().w_cursor.col = ColNr::new(col_end);
    if (include || count > 1 || (!vis_empty && inside_quotes)) && inc_cursor() == Moved::OntoNul {
        inclusive = true;
    }
    if visual_active() {
        if vis_empty || vis_bef_curs {
            // Step the cursor back when 'selection' is not exclusive.
            if P_SEL.first_byte() != b'e' {
                dec_cursor();
            }
        } else {
            // The cursor is at the start of the Visual area. Set its end
            // when we were just inside quotes, or when it did not end at
            // a quote.
            if inside_quotes
                || (!selected_quote
                    && at(visual_anchor().col.get()) != low_byte(quotechar)
                    && (at(visual_anchor().col.get()) == 0
                        || at((visual_anchor().col + 1).get()) != low_byte(quotechar)))
            {
                dec_cursor();
                set_visual_anchor(Win::current().w_cursor);
            }
            Win::current().w_cursor.col = ColNr::new(col_start);
        }
        if visual_mode().is_line() {
            set_visual_mode(VisualMode::CHAR);
            redraw_cmdline.set(true); // show the mode later
        }
    } else {
        op.inclusive = inclusive;
    }
    Ok(())
}
