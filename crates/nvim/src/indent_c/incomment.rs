//! Indenting a line that is *inside* a comment.
//!
//! Two shapes.  A `//` comment lines up with the nearest `//` comment above
//! it ([`align_with_line_comment`]).  A `/* */` comment is the interesting
//! one: 'comments' describes the three-part leader (`s:/*`, `m:*`, `e:*/`)
//! and [`align_in_comment`] uses it to decide whether this line is a middle
//! or an end part and what it should line up with -- plus 'cinoptions' `c`
//! (how far from the opener, when there is nothing after it) and `C` (use `c`
//! even when there *is* something after it).

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};

/// How long a 'comments' leader part may be.
const LEN: usize = COM_MAX_LEN as usize;

/// Whether `a` and `b` agree for `n` bytes -- `strncmp(a, b, n) == 0` over
/// two NUL-terminated strings held as slices.
///
/// Not `starts_with`: a comparison that reaches the NUL in *both* stops there
/// and succeeds, which is how an empty line matches an as-yet-unset comment
/// leader.  That case is reachable -- a blank line inside a comment, before
/// any `s:`/`m:` item has been seen -- and `starts_with` answers differently.
fn ncmp_eq(a: &[u8], b: &[u8], n: usize) -> bool {
    for i in 0..n {
        let (x, y) = (byte_at(a, i), byte_at(b, i));
        if x != y {
            return false;
        }
        if x == 0 {
            return true;
        }
    }
    true
}

/// The column of the nearest `//` comment above the cursor, or `None`.
///
/// Blank lines are skipped; if there is no such comment on its own line, the
/// line directly above is searched for one starting *after* code.
pub(crate) fn align_with_line_comment() -> Option<c_int> {
    let mut trypos = find_line_comment();
    if trypos.is_none() && Win::current().w_cursor.lnum > 1 {
        // There may be a statement before the comment; search from the
        // end of the line above for a comment start.
        // SAFETY: the test in front says `lnum - 1` is at least 1, so it is a
        // line of the buffer, and `ml_get` hands back a NUL-terminated one.
        let lnum = Win::current().w_cursor.lnum - 1;
        let col = check_linecomment(Lines::current().line(lnum));
        if col != MAXCOL {
            let lnum = Win::current().w_cursor.lnum - 1;
            trypos = Some(Pos {
                lnum,
                col,
                coladd: 0,
            });
        }
    }
    trypos.map(|pos| line_vcol(pos.lnum, pos.col))
}

/// The indent for a line inside a `/* */` comment whose opener is at
/// `comment`.
pub(crate) fn align_in_comment(line: &Line, comment: &mut Pos) -> c_int {
    // Start from how indented the line that opens the comment is.
    let mut amount = line_vcol(comment.lnum, comment.col);

    if align_with_comment_leader(line, comment, &mut amount) {
        return amount;
    }

    // A line starting with an asterisk lines up with the asterisk in the
    // opener; anything else with the first character of the comment text.
    if line.starts_with(b'*') {
        return amount + 1;
    }

    // More than one line below the opener: take the indent of the
    // previous non-empty line.
    for lnum in (comment.lnum + 1..line.cur_curpos.lnum).rev() {
        // SAFETY: `lnum` sits between the comment opener and the cursor, so
        // it is a line of the current buffer.
        if !linewhite(lnum) {
            // SAFETY: the same line number.
            return get_indent_lnum(lnum);
        }
    }

    // Directly below the opener.  With 'cinoptions' `CO` -- or with
    // nothing after the opener -- add `c`; otherwise line up with the
    // text that follows the opener.
    let mut nothing_after_opener = true;
    if Buf::current().b_ind_in_comment2 == 0 {
        // `comment` is the position of a `/*` in this buffer, so `col + 2`
        // is the byte after the `*` -- at worst one past the line's end,
        // which reads as the terminator.
        let mut lines = Lines::current();
        let start = lines.line(comment.lnum);
        let look = usize::try_from(comment.col).unwrap_or(0) + 2; // skip / and *
        nothing_after_opener = byte_at(start, look) == 0;
        if !nothing_after_opener {
            let at = look + skip::white(&start[look.min(start.len())..]);
            comment.col = ColNr::try_from(at).expect("a column within a line fits a ColNr");
        }
    }
    amount = line_vcol(comment.lnum, comment.col);
    if Buf::current().b_ind_in_comment2 != 0 || nothing_after_opener {
        amount += Buf::current().b_ind_in_comment;
    }
    amount
}

/// Walk 'comments' looking for an item whose *end* part describes this line,
/// writing the amount it implies into `amount`.
///
/// Answers upstream's `done` -- whether the walk decided.  That flag is
/// deliberately set *before* the "this item's start leader does not match the
/// opener, skip it" `continue`, so an abandoned item still suppresses the
/// fallbacks in [`align_in_comment`].
fn align_with_comment_leader(line: &Line, comment: &Pos, amount: &mut c_int) -> bool {
    // A closure, not a free function: two call sites in one body.
    // SAFETY: the leaders are `LEN`-byte buffers `copy_option_part` filled,
    // and it always NUL-terminates what it writes.
    let strsize = |buf: &[u8; LEN]| unsafe { vim_strsize(buf.as_ptr().cast::<c_char>()) };
    let theline = line.theline();

    // The three parts of a leader.  The initial lengths are upstream's,
    // and they matter: until an `s:`/`m:` item has been seen the buffers
    // are empty and a comparison against them still runs for that many
    // bytes.
    let mut lead_start = [0u8; LEN];
    let mut lead_start_len = 2usize;
    let mut lead_middle = [0u8; LEN];
    let mut lead_middle_len = 1usize;
    let mut start_off = 0;
    let mut start_align = 0;
    let mut done = false;

    let mut p = Buf::current().b_p_com;
    // SAFETY: 'comments' is a NUL-terminated option string, and nothing below
    // steps `p` past that NUL: each `add(1)` follows a byte just read and
    // found non-NUL, `getdigits_int` stops at the first non-digit, and
    // `copy_option_part` stops at the NUL or the separator.
    while unsafe { *p != 0 } {
        // The flag letters in front of this item's ':'.
        let mut align = 0;
        let mut off = 0;
        let mut what = 0;
        // SAFETY: as above -- `p` is inside the option string throughout.
        while unsafe { *p != 0 && (*p).cast_unsigned() != b':' } {
            let c = c_int::from(unsafe { *p }.cast_unsigned());
            if c == COM_START || c == COM_END || c == COM_MIDDLE {
                what = c;
                p = unsafe { p.add(1) };
            } else if c == COM_LEFT || c == COM_RIGHT {
                align = c;
                p = unsafe { p.add(1) };
            } else if ascii_isdigit(c) || c == c_int::from(b'-') {
                off = unsafe { getdigits_int(&raw mut p, true, 0) };
            } else {
                p = unsafe { p.add(1) };
            }
        }
        // SAFETY: the loop above left `p` on the NUL or on the ':'.
        if unsafe { *p }.cast_unsigned() == b':' {
            p = unsafe { p.add(1) };
        }

        let mut lead_end = [0u8; LEN];
        let dst = lead_end.as_mut_ptr().cast::<c_char>();
        let comma = c",".as_ptr().cast_mut();
        // SAFETY: `p` points into the NUL-terminated option, `dst` has `LEN`
        // bytes -- which is the cap handed over -- and `comma` is a literal.
        let lead_end_len = unsafe { copy_option_part(&raw mut p, dst, LEN, comma) };

        if what == COM_START {
            lead_start = lead_end;
            lead_start_len = lead_end_len;
            start_off = off;
            start_align = align;
            continue;
        }
        if what == COM_MIDDLE {
            lead_middle = lead_end;
            lead_middle_len = lead_end_len;
            continue;
        }
        if what != COM_END {
            continue;
        }

        // Our line starts with the *middle* leader: line it up with the
        // comment opener, per this item.
        if ncmp_eq(theline, &lead_middle, lead_middle_len)
            && !ncmp_eq(theline, &lead_end, lead_end_len)
        {
            done = true;
            if Win::current().w_cursor.lnum > 1 {
                let prev = Win::current().w_cursor.lnum - 1;
                // The line above starting with the start leader: its
                // indent plus the offset.  With the middle leader: its
                // indent and nothing more.
                let mut lines = Lines::current();
                let above = lines.line(prev);
                let above = &above[skip::white(above)..];
                if ncmp_eq(above, &lead_start, lead_start_len) {
                    // SAFETY: the same line number.
                    *amount = get_indent_lnum(prev);
                } else if ncmp_eq(above, &lead_middle, lead_middle_len) {
                    // SAFETY: the same.
                    *amount = get_indent_lnum(prev);
                    break;
                } else {
                    // The opener does not match this item's start leader:
                    // skip the item -- but `done` stays set. `comment` is the
                    // position of a `/*` in this buffer, so its column is
                    // inside its own line; a column past the end answers
                    // nothing rather than reading on.
                    let line = lines.line(comment.lnum);
                    let at = usize::try_from(comment.col).unwrap_or(0);
                    let opener = line.get(at..).unwrap_or_default();
                    if !ncmp_eq(opener, &lead_start, lead_start_len) {
                        continue;
                    }
                }
            }
            if start_off != 0 {
                *amount += start_off;
            } else if start_align == COM_RIGHT {
                *amount += strsize(&lead_start) - strsize(&lead_middle);
            }
            break;
        }

        // Our line starts with the *end* leader: line it up with the
        // middle one.
        if !ncmp_eq(theline, &lead_middle, lead_middle_len)
            && ncmp_eq(theline, &lead_end, lead_end_len)
        {
            // SAFETY: on the main thread with a current buffer; a bad line
            // number is `ml_get`'s own to report, as upstream leaves it.
            *amount = get_indent_lnum(Win::current().w_cursor.lnum - 1);
            if off != 0 {
                *amount += off;
            } else if align == COM_RIGHT {
                *amount += strsize(&lead_start) - strsize(&lead_middle);
            }
            done = true;
            break;
        }
    }
    done
}

#[cfg(test)]
mod tests {
    use super::ncmp_eq;

    #[test]
    fn ncmp_eq_is_strncmp() {
        assert!(ncmp_eq(b"*/rest", b"*/", 2));
        assert!(!ncmp_eq(b"*/rest", b"*", 2));
        // Both reach their NUL inside the count: equal.
        assert!(ncmp_eq(b"", b"", 1));
        assert!(ncmp_eq(b"", b"", 50));
        // The reachable case `starts_with` gets wrong: an empty line against
        // an unset one-byte leader.
        assert!(!ncmp_eq(b"x", b"", 1));
        assert!(ncmp_eq(b"ab", b"abc", 2));
        assert!(!ncmp_eq(b"ab", b"abc", 3));
    }
}
