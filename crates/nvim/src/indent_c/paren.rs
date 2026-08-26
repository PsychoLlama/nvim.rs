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
//! Every answer here is a `pos_T` by value: the searches call `findmatch`
//! more than once and each one used to overwrite the last answer's storage.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};
use core::ptr;

/// The column `trypos.col` sits at once comments and strings before it are
/// stepped over -- so a `{` inside a comment answers a column past its own,
/// which is how the searches below reject it.
///
/// # Safety
/// Reads the buffer; may unlock the current line.
pub(crate) unsafe fn cin_skip2pos(trypos: pos_T) -> c_int {
    // SAFETY: on the main thread with a current buffer; `ml_get` hands back
    // a NUL-terminated line, which is all the two skippers ask for, and the
    // walk stops at its NUL.  Every step of it is a pointer operation, so one
    // region around the whole walk is as tight as this gets.
    unsafe {
        let line = ml_get(trypos.lnum);
        let mut p = line.cast_const();
        while *p != 0 && (p.offset_from(line) as colnr_T) < trypos.col {
            if cin_iscomment(p) {
                p = cin_skipcomment(p);
            } else {
                let new_p = skip_string(p);
                p = if new_p == p { p.add(1) } else { new_p };
            }
        }
        p.offset_from(line) as c_int
    }
}

/// The `{` opening the block the cursor is in, or null.
///
/// A `{` inside a `//` or `/* */` comment is ignored -- which is what makes
/// the three lines of `foo()\n{\n}` indent -- and the search resumes from
/// the start of whatever comment or raw string swallowed it.
///
/// # Safety
/// Reads and restores the cursor; may unlock the current line.
pub(crate) unsafe fn find_start_brace() -> Option<pos_T> {
    let cursor_save = cur_win().w_cursor;
    let mut trypos;
    loop {
        // SAFETY: on the main thread, with a current window and buffer.
        trypos = unsafe { findmatchlimit(ptr::null_mut(), c_int::from(b'{'), FM_BLOCKSTOP, 0) };
        let Some(brace) = trypos else {
            break;
        };
        cur_win().w_cursor = brace;

        // SAFETY: `brace` is a position `findmatchlimit` found in the current
        // buffer, so `cin_skip2pos` may read its line; the comment search runs
        // only when the `{` really sits at `brace.col`, and the `&&` chain is
        // left whole so that it keeps doing so.
        let mut pos = None;
        let uncommented = unsafe {
            cin_skip2pos(brace) == brace.col && {
                pos = ind_find_start_comment_or_raw_string(None);
                pos.is_none()
            }
        };
        if uncommented {
            break;
        }
        if let Some(pos) = pos {
            cur_win().w_cursor = pos;
        }
    }
    cur_win().w_cursor = cursor_save;
    trypos
}

/// The unclosed `(` above the cursor, or null.
///
/// # Safety
/// Reads and restores the cursor; may unlock the current line.
pub(crate) unsafe fn find_match_paren(ind_maxparen: c_int) -> Option<pos_T> {
    unsafe { find_match_char(b'(', ind_maxparen) }
}

/// The unclosed `c` above the cursor, or null, ignoring one inside a comment
/// or a raw string.
///
/// When the match turns out to be inside one, the search restarts from the
/// *start* of that comment with the remaining budget -- `ind_maxparen` less
/// the lines already walked -- so the total distance searched stays bounded
/// however many comments are in the way.
///
/// # Safety
/// Reads and restores the cursor; may unlock the current line.
pub(crate) unsafe fn find_match_char(c: u8, ind_maxparen: c_int) -> Option<pos_T> {
    let cursor_save = cur_win().w_cursor;
    let mut ind_maxp_wk = ind_maxparen;

    let found = loop {
        let limit = int64_t::from(ind_maxp_wk);
        // SAFETY: on the main thread, with a current window and buffer.
        let found = unsafe { findmatchlimit(ptr::null_mut(), c_int::from(c), 0, limit) };
        let Some(trypos) = found else {
            break None;
        };

        // Is the match inside a `//` comment?
        // SAFETY: `trypos` is a position `findmatchlimit` found in the current
        // buffer, so `cin_skip2pos` may read its line.
        if unsafe { cin_skip2pos(trypos) } > trypos.col {
            ind_maxp_wk = ind_maxparen - (cursor_save.lnum - trypos.lnum);
            if ind_maxp_wk <= 0 {
                break None;
            }
            cur_win().w_cursor = trypos.with_col(0);
            continue;
        }

        cur_win().w_cursor = trypos;

        // SAFETY: on the main thread, with a current window and buffer.
        let enclosing = unsafe { ind_find_start_comment_or_raw_string(None) };
        let Some(trypos_wk) = enclosing else {
            break Some(trypos);
        };
        ind_maxp_wk = ind_maxparen - (cursor_save.lnum - trypos_wk.lnum);
        if ind_maxp_wk <= 0 {
            break None;
        }
        cur_win().w_cursor = trypos_wk;
    };

    cur_win().w_cursor = cursor_save;
    found
}

/// [`find_match_paren`], but null when an unmatched `{` is closer.
///
/// # Safety
/// Reads and restores the cursor; may unlock the current line.
pub(crate) unsafe fn find_match_paren_after_brace(ind_maxparen: c_int) -> Option<pos_T> {
    // SAFETY: searches the current buffer from the cursor, and restores it.
    let trypos = unsafe { find_match_paren(ind_maxparen) }?;
    // SAFETY: the same.
    let brace_is_further_down = unsafe { find_start_brace() }.is_some_and(|brace| {
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
pub(crate) fn corr_ind_maxparen(startpos: &pos_T) -> c_int {
    let maxparen = cur_buf().b_ind_maxparen;
    let n = startpos.lnum - cur_win().w_cursor.lnum;
    if n > 0 && n < maxparen / 2 {
        maxparen - n
    } else {
        maxparen
    }
}

/// Put `w_cursor.col` on the last unmatched `end` in `l`, answering whether
/// there was one.  `l` must be the start of the line.
///
/// Brackets inside comments and strings do not count, which is what the two
/// skips at the top of the loop are for.
///
/// # Safety
/// `l` must point at the start of a NUL-terminated line; writes the cursor
/// column.
pub(crate) unsafe fn find_last_paren(l: *const c_char, start: u8, end: u8) -> bool {
    let mut retval = false;
    let mut open_count = 0;
    cur_win().w_cursor.col = 0; // default is start of line

    let mut i: isize = 0;
    loop {
        // SAFETY: the caller's promise -- `l` starts a NUL-terminated line,
        // and neither skipper walks past its NUL, so every `i` the loop takes
        // indexes inside that line.  The whole of the walk is unsafe, so one
        // region around it is as tight as this gets.
        let c = unsafe {
            if *l.offset(i) == 0 {
                break;
            }
            i = cin_skipcomment(l.offset(i)).offset_from(l); // brackets in comments
            i = skip_string(l.offset(i)).offset_from(l); // ... and in quotes
            *l.offset(i) as u8
        };
        if c == start {
            open_count += 1;
        } else if c == end {
            if open_count > 0 {
                open_count -= 1;
            } else {
                cur_win().w_cursor.col = i as colnr_T;
                retval = true;
            }
        }
        i += 1;
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
///
/// # Safety
/// Moves the cursor; may unlock the current line.
pub(crate) unsafe fn find_match(lookfor: c_int, ourscope: linenr_T) -> bool {
    let (mut elselevel, mut whilelevel) = if lookfor == LOOKFOR_IF {
        (1, 0)
    } else {
        (0, 1)
    };

    cur_win().w_cursor.col = 0;

    while cur_win().w_cursor.lnum > ourscope + 1 {
        cur_win().w_cursor.lnum -= 1;
        cur_win().w_cursor.col = 0;

        // SAFETY: the cursor is on a line of the current buffer, and
        // `get_cursor_line_ptr` hands back a NUL-terminated one -- which is
        // all the recognisers below ask for.
        let mut look = unsafe { cin_skipcomment(get_cursor_line_ptr()) };
        // SAFETY: `look` points inside that line.  Upstream tests the four in
        // this order and stops at the first that answers; the chain is left
        // whole so that it keeps doing so.
        let interesting = unsafe {
            cin_iselse(look)
                || cin_isif(look)
                || cin_isdo(look)
                || cin_iswhileofdo(look, cur_win().w_cursor.lnum)
        };
        if !interesting {
            continue;
        }

        // Outside the braces entirely, or enclosed by a brace further
        // back than ours: out of scope either way.
        // SAFETY: searches the current buffer from the cursor, and restores it.
        let Some(theirscope) = (unsafe { find_start_brace() }) else {
            return false;
        };
        if theirscope.lnum < ourscope {
            return false;
        }
        // Enclosed by a *deeper* brace: a different scope, ignore it.
        if theirscope.lnum > ourscope {
            continue;
        }

        // SAFETY: as above -- `find_start_brace` may have unlocked the line,
        // so it is fetched again.
        look = unsafe { cin_skipcomment(get_cursor_line_ptr()) };
        // Looking for an `if`, ignore the `if`s and `else`s of a deeper
        // do-while loop.
        if !(lookfor == LOOKFOR_IF && whilelevel != 0) {
            // SAFETY: `look` points inside a NUL-terminated line.
            if unsafe { cin_iselse(look) } {
                // An `else` that is not an `else if` needs one more `if`.
                // SAFETY: `cin_iselse` matched `else` at or after `look`, so
                // `look.add(4)` is no further than that word's last byte.
                if !unsafe { cin_isif(cin_skipcomment(look.add(4))) } {
                    elselevel += 1;
                }
                continue;
            }
            // SAFETY: `look` points inside a NUL-terminated line.
            if unsafe { cin_isif(look) } {
                elselevel -= 1;
                // Once the `if` is found, `while`s stop getting in the way.
                if elselevel == 0 && lookfor == LOOKFOR_IF {
                    whilelevel = 0;
                }
            }
        }

        // SAFETY: `look` points inside a NUL-terminated line, and the cursor
        // is on a line of the current buffer.
        if unsafe { cin_iswhileofdo(look, cur_win().w_cursor.lnum) } {
            whilelevel += 1;
            continue;
        }
        // SAFETY: `look` points inside a NUL-terminated line.
        if unsafe { cin_isdo(look) } {
            whilelevel -= 1;
        }

        // All the `else`s used up: this is the one.
        if elselevel <= 0 && whilelevel <= 0 {
            return true;
        }
    }
    false
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
