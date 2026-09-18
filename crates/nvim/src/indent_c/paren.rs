//! Finding the enclosing bracket, and the matching keyword.
//!
//! [`find_match_paren`]/[`find_match_char`] search backwards for an unclosed
//! `(`/`[`, [`find_start_brace`] for an unclosed `{` that is not inside a
//! comment or a paren, both bounded by 'cinoptions' `)N`
//! (`b_ind_maxparen`).  [`find_last_paren`] puts the cursor on the rightmost
//! unmatched bracket of a line first, which is what makes the backwards
//! search start in the right place.  [`find_match`] is the other kind of
//! matching: the `if` an `else` belongs to, or the `do` a `while` closes.
//!
//! Every answer here is a `Pos` by value: the searches call `findmatch`
//! more than once and each one used to overwrite the last answer's storage.
//!
//! | C | here |
//! | --- | --- |
//! | `cin_skip2pos` | [`first_code_col`] |

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

/// The column `trypos.col` sits at once comments and strings before it are
/// stepped over -- so a `{` inside a comment answers a column past its own,
/// which is how the searches below reject it.
pub(crate) fn first_code_col(trypos: Pos) -> c_int {
    let mut lines = Lines::current();
    let line = lines.line(trypos.lnum);
    let limit = usize::try_from(trypos.col).expect("a paren column is never negative");
    let mut at = 0usize;
    while byte_at(line, at) != 0 && at < limit {
        if starts_comment(line, at) {
            at = code_at(line, at);
        } else {
            let past = string_end_at(line, at);
            at = if past == at { at + 1 } else { past };
        }
    }
    c_int::try_from(at).expect("a column within a line fits an int")
}

/// The `{` opening the block the cursor is in, or null.
///
/// A `{` inside a `//` or `/* */` comment is ignored -- which is what makes
/// the three lines of `foo()\n{\n}` indent -- and the search resumes from
/// the start of whatever comment or raw string swallowed it.
pub(crate) fn find_start_brace() -> Option<Pos> {
    let cursor_save = Win::current().w_cursor;
    let mut trypos;
    loop {
        // SAFETY: on the main thread, with a current window and buffer.
        trypos = findmatchlimit(None, c_int::from(b'{'), FM_BLOCKSTOP, 0);
        let Some(brace) = trypos else {
            break;
        };
        Win::current().w_cursor = brace;

        // The comment search runs only when the `{` really sits at
        // `brace.col`, and the `&&` chain is left whole so that it keeps
        // doing so.
        let mut pos = None;
        let uncommented = first_code_col(brace) == brace.col && {
            pos = ind_find_start_comment_or_raw_string(None);
            pos.is_none()
        };
        if uncommented {
            break;
        }
        if let Some(pos) = pos {
            Win::current().w_cursor = pos;
        }
    }
    Win::current().w_cursor = cursor_save;
    trypos
}

/// The unclosed `(` above the cursor, or null.
pub(crate) fn find_match_paren(ind_maxparen: c_int) -> Option<Pos> {
    find_match_char(b'(', ind_maxparen)
}

/// The unclosed `c` above the cursor, or null, ignoring one inside a comment
/// or a raw string.
///
/// When the match turns out to be inside one, the search restarts from the
/// *start* of that comment with the remaining budget -- `ind_maxparen` less
/// the lines already walked -- so the total distance searched stays bounded
/// however many comments are in the way.
pub(crate) fn find_match_char(c: u8, ind_maxparen: c_int) -> Option<Pos> {
    let cursor_save = Win::current().w_cursor;
    let mut ind_maxp_wk = ind_maxparen;

    let found = loop {
        let limit = int64_t::from(ind_maxp_wk);
        // SAFETY: on the main thread, with a current window and buffer.
        let found = findmatchlimit(None, c_int::from(c), 0, limit);
        let Some(trypos) = found else {
            break None;
        };

        // Is the match inside a `//` comment?  `trypos` is a position
        // `findmatchlimit` found in the current buffer, so the cache answers
        // with the line it sits on.
        if first_code_col(trypos) > trypos.col {
            ind_maxp_wk = ind_maxparen - (cursor_save.lnum - trypos.lnum);
            if ind_maxp_wk <= 0 {
                break None;
            }
            Win::current().w_cursor = trypos.with_col(0);
            continue;
        }

        Win::current().w_cursor = trypos;

        let enclosing = ind_find_start_comment_or_raw_string(None);
        let Some(trypos_wk) = enclosing else {
            break Some(trypos);
        };
        ind_maxp_wk = ind_maxparen - (cursor_save.lnum - trypos_wk.lnum);
        if ind_maxp_wk <= 0 {
            break None;
        }
        Win::current().w_cursor = trypos_wk;
    };

    Win::current().w_cursor = cursor_save;
    found
}

/// [`find_match_paren`], but null when an unmatched `{` is closer.
pub(crate) fn find_match_paren_after_brace(ind_maxparen: c_int) -> Option<Pos> {
    let trypos = find_match_paren(ind_maxparen)?;
    let brace_is_further_down = find_start_brace().is_some_and(|brace| {
        if trypos.lnum != brace.lnum {
            trypos.lnum < brace.lnum
        } else {
            trypos.col < brace.col
        }
    });
    (!brace_is_further_down).then_some(trypos)
}

/// 'cinoptions' `)N` corrected for how far *below* the cursor `startpos` is.
///
/// Searching for a match above the cursor from a position below it would
/// otherwise get a longer reach than the option allows, and could find a
/// paren the option was meant to exclude.  Only a `startpos` below the cursor
/// and within half the budget shortens it.
pub(crate) fn corr_ind_maxparen(startpos: &Pos) -> c_int {
    let maxparen = Buf::current().b_ind_maxparen;
    let n = startpos.lnum - Win::current().w_cursor.lnum;
    if n > 0 && n < maxparen / 2 {
        maxparen - n
    } else {
        maxparen
    }
}

/// Put `w_cursor.col` on the last unmatched `end` in `line`, answering
/// whether there was one.
///
/// Brackets inside comments and strings do not count, which is what the two
/// skips at the top of the loop are for.
pub(crate) fn find_last_paren(line: &[u8], start: u8, end: u8) -> bool {
    let mut retval = false;
    let mut open_count = 0;
    Win::current().w_cursor.col = 0; // default is start of line

    let mut at = 0usize;
    while byte_at(line, at) != 0 {
        at = code_at(line, at); // brackets in comments
        at = string_end_at(line, at); // ... and in quotes
        let c = byte_at(line, at);
        if c == start {
            open_count += 1;
        } else if c == end {
            if open_count > 0 {
                open_count -= 1;
            } else {
                Win::current().w_cursor.col =
                    ColNr::try_from(at).expect("a column within a line fits a ColNr");
                retval = true;
            }
        }
        at += 1;
    }
    retval
}

/// Search back from the cursor for the `if` an `else` belongs to
/// (`LOOKFOR_IF`) or the `do` a `while` closes, stopping at `ourscope`.
///
/// Both directions are one walk with two counters: an `else` that is not an
/// `else if` needs one more `if`, a `do`-`while` needs one more `do`, and a
/// line whose enclosing brace is not `ourscope`'s is in a different scope and
/// is skipped whole.
pub(crate) fn find_match(lookfor: c_int, ourscope: LineNr) -> bool {
    let (mut elselevel, mut whilelevel) = if lookfor == LOOKFOR_IF {
        (1, 0)
    } else {
        (0, 1)
    };

    Win::current().w_cursor.col = 0;

    while Win::current().w_cursor.lnum > ourscope + 1 {
        Win::current().w_cursor.lnum -= 1;
        Win::current().w_cursor.col = 0;

        // Upstream tests the four in this order and stops at the first that
        // answers; the `while`-of-`do` half that re-enters is asked last, so
        // the borrow the other three take is dropped before it runs.
        let (starts_while_of_do, interesting) = {
            let mut lines = Lines::current();
            let line = lines.line(Win::current().w_cursor.lnum);
            let look = code_at(line, 0);
            (
                starts_while(line, look),
                is_else(line, look) || is_if(line, look) || is_do(line, look),
            )
        };
        let interesting =
            interesting || (starts_while_of_do && while_closes_do(Win::current().w_cursor.lnum));
        if !interesting {
            continue;
        }

        // Outside the braces entirely, or enclosed by a brace further
        // back than ours: out of scope either way.
        let Some(theirscope) = find_start_brace() else {
            return false;
        };
        if theirscope.lnum < ourscope {
            return false;
        }
        // Enclosed by a *deeper* brace: a different scope, ignore it.
        if theirscope.lnum > ourscope {
            continue;
        }

        // `find_start_brace` may have unlocked the line, so it is read
        // again -- and dropped again before the `while`-of-`do` search.
        let (starts_while_of_do, is_else_line, plain_else, is_if_line, is_do_line) = {
            let mut lines = Lines::current();
            let line = lines.line(Win::current().w_cursor.lnum);
            let look = code_at(line, 0);
            (
                starts_while(line, look),
                is_else(line, look),
                // An `else` that is not an `else if` needs one more `if`.
                // Upstream reads four bytes on from `look` itself, which for
                // `} else` is the middle of the word; reproduced.
                !is_if(line, code_at(line, look + 4)),
                is_if(line, look),
                is_do(line, look),
            )
        };

        // Looking for an `if`, ignore the `if`s and `else`s of a deeper
        // do-while loop.
        if !(lookfor == LOOKFOR_IF && whilelevel != 0) {
            if is_else_line {
                if plain_else {
                    elselevel += 1;
                }
                continue;
            }
            if is_if_line {
                elselevel -= 1;
                // Once the `if` is found, `while`s stop getting in the way.
                if elselevel == 0 && lookfor == LOOKFOR_IF {
                    whilelevel = 0;
                }
            }
        }

        if starts_while_of_do && while_closes_do(Win::current().w_cursor.lnum) {
            whilelevel += 1;
            continue;
        }
        if is_do_line {
            whilelevel -= 1;
        }

        // All the `else`s used up: this is the one.
        if elselevel <= 0 && whilelevel <= 0 {
            return true;
        }
    }
    false
}
