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

use super::*;
use core::ffi::{CStr, c_char, c_int};

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
///
/// # Safety
/// Reads the cursor and the buffer; may unlock the current line.
pub(crate) unsafe fn align_with_line_comment() -> Option<c_int> {
    unsafe {
        let mut trypos = find_line_comment();
        if trypos.is_none() && (*curwin.get()).w_cursor.lnum > 1 {
            // There may be a statement before the comment; search from the
            // end of the line above for a comment start.
            let col = check_linecomment(ml_get((*curwin.get()).w_cursor.lnum - 1));
            if col != MAXCOL {
                let lnum = (*curwin.get()).w_cursor.lnum - 1;
                trypos = Some(pos_T {
                    lnum,
                    col,
                    coladd: 0,
                });
            }
        }
        trypos.map(|pos| line_vcol(pos.lnum, pos.col))
    }
}

/// The indent for a line inside a `/* */` comment whose opener is at
/// `comment`.
///
/// # Safety
/// Reads the cursor and the buffer; may unlock the current line.  `comment`
/// is a copy the caller owns, and is moved onto the comment's *text* when
/// 'cinoptions' `C` is off and there is text after the opener.
pub(crate) unsafe fn align_in_comment(line: &Line, comment: &mut pos_T) -> c_int {
    unsafe {
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
            if !linewhite(lnum) {
                return get_indent_lnum(lnum);
            }
        }

        // Directly below the opener.  With 'cinoptions' `CO` -- or with
        // nothing after the opener -- add `c`; otherwise line up with the
        // text that follows the opener.
        let mut nothing_after_opener = true;
        if (*curbuf.get()).b_ind_in_comment2 == 0 {
            let start = ml_get(comment.lnum);
            let look = start.offset(comment.col as isize).add(2); // skip / and *
            nothing_after_opener = *look == 0;
            if !nothing_after_opener {
                comment.col = skipwhite(look).offset_from(start) as colnr_T;
            }
        }
        amount = line_vcol(comment.lnum, comment.col);
        if (*curbuf.get()).b_ind_in_comment2 != 0 || nothing_after_opener {
            amount += (*curbuf.get()).b_ind_in_comment;
        }
        amount
    }
}

/// Walk 'comments' looking for an item whose *end* part describes this line,
/// writing the amount it implies into `amount`.
///
/// Answers upstream's `done` -- whether the walk decided.  That flag is
/// deliberately set *before* the "this item's start leader does not match the
/// opener, skip it" `continue`, so an abandoned item still suppresses the
/// fallbacks in [`align_in_comment`].
///
/// # Safety
/// Reads the buffer; may unlock the current line.
unsafe fn align_with_comment_leader(line: &Line, comment: &pos_T, amount: &mut c_int) -> bool {
    unsafe {
        // A closure, not a free function: two call sites in one body.
        let strsize = |buf: &[u8; LEN]| vim_strsize(buf.as_ptr().cast::<c_char>());
        let theline = CStr::from_ptr(line.theline).to_bytes();

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

        let mut p = (*curbuf.get()).b_p_com;
        while *p != 0 {
            // The flag letters in front of this item's ':'.
            let mut align = 0;
            let mut off = 0;
            let mut what = 0;
            while *p != 0 && *p as u8 != b':' {
                let c = c_int::from(*p as u8);
                if c == COM_START || c == COM_END || c == COM_MIDDLE {
                    what = c;
                    p = p.add(1);
                } else if c == COM_LEFT || c == COM_RIGHT {
                    align = c;
                    p = p.add(1);
                } else if ascii_isdigit(c) || c == c_int::from(b'-') {
                    off = getdigits_int(&raw mut p, true, 0);
                } else {
                    p = p.add(1);
                }
            }
            if *p as u8 == b':' {
                p = p.add(1);
            }

            let mut lead_end = [0u8; LEN];
            let lead_end_len = copy_option_part(
                &raw mut p,
                lead_end.as_mut_ptr().cast::<c_char>(),
                LEN,
                c",".as_ptr().cast_mut(),
            );

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
                if (*curwin.get()).w_cursor.lnum > 1 {
                    let prev = (*curwin.get()).w_cursor.lnum - 1;
                    // The line above starting with the start leader: its
                    // indent plus the offset.  With the middle leader: its
                    // indent and nothing more.
                    let above = CStr::from_ptr(skipwhite(ml_get(prev))).to_bytes();
                    if ncmp_eq(above, &lead_start, lead_start_len) {
                        *amount = get_indent_lnum(prev);
                    } else if ncmp_eq(above, &lead_middle, lead_middle_len) {
                        *amount = get_indent_lnum(prev);
                        break;
                    } else {
                        // The opener does not match this item's start leader:
                        // skip the item -- but `done` stays set.
                        let opener = ml_get(comment.lnum).offset(comment.col as isize);
                        let opener = CStr::from_ptr(opener).to_bytes();
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
                *amount = get_indent_lnum((*curwin.get()).w_cursor.lnum - 1);
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
