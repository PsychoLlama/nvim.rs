//! Indenting a line inside unclosed parentheses.
//!
//! The 'cinoptions' letters that live here are `(` (`b_ind_unclosed`, how far
//! from the line holding the `(`), `u` (`unclosed2`, per additional unclosed
//! `(`), `U` (`unclosed_noignore`), `w` (`unclosed_whiteok`), `W`
//! (`unclosed_wrapped`, for a `(` last on its line), `m` (`matching_paren`, a
//! `)` under its opener's line start), `M` (`paren_prev`, a `)` under the
//! previous line) and `k` (`if_for_while`, a different amount when the paren
//! belongs to an `if`/`for`/`while`).
//!
//! Two answers compete throughout: `amount`, the indent of the *line* the
//! paren is on, and `cur_amount`, the column of the paren (or of what follows
//! it).  Which one wins is the whole of the second half.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

/// The indent for a line inside the unclosed paren at `our_paren_pos`.
///
/// # Safety
/// Moves the cursor; may unlock the current line.
pub(crate) unsafe fn indent_in_parens(line: &Line, our_paren_pos: pos_T) -> c_int {
    let mut our_paren_pos = our_paren_pos;
    let mut cur_amount = MAXCOL;

    // SAFETY: `line.theline` is a NUL-terminated copy of the cursor's line,
    // alive for the whole call.
    let mut amount = if unsafe { line.starts_with(b')') } && cur_buf().b_ind_paren_prev != 0 {
        // Line up with the start of the matching paren's line.
        // SAFETY: on the main thread with a current buffer; a bad line number
        // is `ml_get`'s own to report, as upstream leaves it.
        unsafe { get_indent_lnum(cur_win().w_cursor.lnum - 1) }
    } else {
        // If the matching paren is more than one line away, use the
        // indent of a previous non-empty line that matches the *same*
        // paren -- a line under a different one is a different question.
        // SAFETY: this function's own contract -- the cursor is ours to move
        // and the current line may be unlocked.
        unsafe { previous_line_under_same_paren(line, our_paren_pos, &mut cur_amount) }
    };

    if amount == -1 {
        // SAFETY: the same.
        amount = unsafe { align_with_unclosed_paren(line, &mut our_paren_pos, &mut cur_amount) };
    }

    // Extra indent for a comment.  `get_c_indent` adds `b_ind_comment`
    // again for the whole "inside something" branch, so a comment inside
    // unclosed parens gets it **twice**; upstream's two `if
    // (cin_iscomment(theline))` blocks are both on this path
    // (`v0.12.4:src/nvim/indent_c.c:2430` and `:3419`).  Reproduced.
    // SAFETY: `line.theline` is NUL-terminated.
    if unsafe { cin_iscomment(line.theline) } {
        amount += cur_buf().b_ind_comment;
    }
    amount
}

/// The indent of the nearest non-empty line above that is under the *same*
/// unclosed paren, or -1 when there is none.
///
/// A line starting with `)` does not take that indent -- it only lowers
/// `cur_amount`, so that the close paren cannot end up further right than the
/// lines it closes.
///
/// # Safety
/// Moves the cursor; may unlock the current line.
unsafe fn previous_line_under_same_paren(
    line: &Line,
    our_paren_pos: pos_T,
    cur_amount: &mut c_int,
) -> c_int {
    let mut amount = -1;
    let mut lnum = line.cur_curpos.lnum - 1;
    while lnum > our_paren_pos.lnum {
        // SAFETY: `lnum` sits between the paren's line and the cursor, so it
        // is a line of the current buffer, and `ml_get` hands back a
        // NUL-terminated one that `skipwhite` stops inside.
        let mut l = unsafe { skipwhite(ml_get(lnum)) }.cast_const();
        // A comment line, or a #define / #if continuation: ignore it.  The
        // `||` keeps upstream's order -- `cin_ispreproc_cont` runs only when
        // the line holds code.
        //
        // SAFETY: `l` is that NUL-terminated line.  The borrows handed to
        // `cin_ispreproc_cont` are this scan's own locals, and it reads only
        // the line it is given and the current *buffer*, never `curwin`.
        let skip = unsafe { cin_nocode(l) || cin_ispreproc_cont(&mut l, &mut lnum, &mut amount) };
        if !skip {
            cur_win().w_cursor.lnum = lnum;

            // Skip a comment or raw string.
            // SAFETY: the cursor is on `lnum`, a line of the current buffer.
            let trypos = unsafe { ind_find_start_comment_or_raw_string(None) };
            if let Some(trypos) = trypos {
                lnum = trypos.lnum + 1;
            } else {
                let maxparen = corr_ind_maxparen(&line.cur_curpos);
                // SAFETY: the same -- the search runs back from the cursor.
                let trypos = unsafe { find_match_paren(maxparen) };
                if let Some(trypos) = trypos
                    && trypos.lnum == our_paren_pos.lnum
                    && trypos.col == our_paren_pos.col
                {
                    // SAFETY: `lnum` is still a line of the current buffer.
                    amount = unsafe { get_indent_lnum(lnum) };
                    // SAFETY: `line.theline` is still valid.
                    if unsafe { line.starts_with(b')') } {
                        if our_paren_pos.lnum != lnum && *cur_amount > amount {
                            *cur_amount = amount;
                        }
                        amount = -1;
                    }
                    break;
                }
            }
        }
        lnum -= 1;
    }
    amount
}

/// Line up with the unclosed paren itself: with the line it is on, with the
/// character after it, or a fixed amount in from either.
///
/// # Safety
/// Moves the cursor; may unlock the current line.
unsafe fn align_with_unclosed_paren(
    line: &Line,
    our_paren_pos: &mut pos_T,
    cur_amount: &mut c_int,
) -> c_int {
    let mut ignore_paren_col = 0;
    let mut is_if_for_while = false;

    if cur_buf().b_ind_if_for_while != 0 {
        // Find the outermost opening paren on that line and ask whether
        // it belongs to an "if", "for" or "while".
        let cursor_save = cur_win().w_cursor;
        let mut outermost = *our_paren_pos;
        loop {
            cur_win().w_cursor.lnum = outermost.lnum;
            cur_win().w_cursor.col = outermost.col;
            // SAFETY: the cursor was just put on `outermost`, a paren
            // position in the current buffer, which is where the search for
            // the next one out starts.
            match unsafe { find_match_paren(cur_buf().b_ind_maxparen) } {
                Some(pos) if pos.lnum == outermost.lnum => outermost = pos,
                _ => break,
            }
        }
        cur_win().w_cursor = cursor_save;
        // SAFETY: `outermost` is a paren position in the current buffer, so
        // its line number is one of that buffer's and its column indexes
        // inside the NUL-terminated line `ml_get` hands back.
        let text = unsafe { ml_get(outermost.lnum) };
        // SAFETY: the same, and `outermost.col` is this function's own copy.
        is_if_for_while = unsafe { cin_is_if_for_while_before_offset(text, &mut outermost.col) };
    }

    let mut look = ::core::ptr::null::<::core::ffi::c_char>();
    // SAFETY: `our_paren_pos.lnum` is a line of the current buffer, and
    // `skip_label` writes a pointer into it through `look`.
    let mut amount = unsafe { skip_label(our_paren_pos.lnum, &mut look) };
    // SAFETY: `look` now points into that NUL-terminated line.
    look = unsafe { skipwhite(look) };
    // SAFETY: the same -- `skipwhite` stops at the NUL at the latest.
    if unsafe { *look as u8 == b'(' } {
        // Ignore a '(' in front of the line that has a match *before* our
        // matching '(' -- a `(void)` cast, say.
        let save_lnum = cur_win().w_cursor.lnum;
        cur_win().w_cursor.lnum = our_paren_pos.lnum;
        // SAFETY: the cursor was just moved onto `our_paren_pos.lnum`, so
        // `get_cursor_line_ptr` hands back the very line `look` points into
        // -- the two pointers are into the same allocation.
        let look_col = unsafe { look.offset_from(get_cursor_line_ptr()) } as colnr_T;
        cur_win().w_cursor.col = look_col + 1;
        let no_oparg = ::core::ptr::null_mut::<oparg_T>();
        let maxparen = int64_t::from(cur_buf().b_ind_maxparen);
        // SAFETY: the cursor is just past that `(`, which is where the match
        // search starts; `findmatchlimit` takes a null `oparg` for "no
        // operator pending".
        let trypos = unsafe { findmatchlimit(no_oparg, c_int::from(b')'), 0, maxparen) };
        if let Some(trypos) = trypos
            && trypos.lnum == our_paren_pos.lnum
            && trypos.col < our_paren_pos.col
        {
            ignore_paren_col = trypos.col + 1;
        }
        cur_win().w_cursor.lnum = save_lnum;
        // SAFETY: the search above may have unlocked the line, so `look` is
        // refetched at the same column of the same line -- `look_col` came
        // from that line and is therefore inside it.
        look = unsafe { ml_get(our_paren_pos.lnum).offset(look_col as isize) };
    }

    // "line up with the paren itself" applies to a zero `(` with no `k`
    // in play, and to a line whose own leading `(` is being ignored.
    // Upstream reads `*look` at both of the two places this is tested,
    // once before and once after a `getvcol`, so it stays a closure.
    let line_up_with_paren = || {
        // SAFETY: `look` points into a NUL-terminated line -- the `skipwhite`d
        // label tail, or the same column of that line refetched above -- and
        // nothing between here and either call site moves it.
        cur_buf().b_ind_unclosed == 0 && !is_if_for_while
            || cur_buf().b_ind_unclosed_noignore == 0
                && unsafe { *look as u8 == b'(' }
                && ignore_paren_col == 0
    };

    // SAFETY: `line.theline` is a NUL-terminated copy of the cursor's line.
    if unsafe { line.starts_with(b')') } || line_up_with_paren() {
        // SAFETY: the same.
        if !unsafe { line.starts_with(b')') } {
            *cur_amount = MAXCOL;
            // SAFETY: `our_paren_pos` is a paren position in this buffer, so
            // `ml_get` hands back the NUL-terminated line holding it.
            let l = unsafe { ml_get(our_paren_pos.lnum) };
            // SAFETY: `l` is that line; the `&&` keeps the scan behind the
            // option test, as upstream does.
            if cur_buf().b_ind_unclosed_wrapped != 0 && unsafe { cin_ends_in(l, b"(") } {
                // The paren is the last non-white character of its line:
                // indent one `W` level per nesting level instead.
                let mut n = 1;
                for col in 0..our_paren_pos.col {
                    // SAFETY: `col` is below `our_paren_pos.col`, the column
                    // of a `(` found on this line, so it indexes inside it.
                    match unsafe { *l.offset(col as isize) } as u8 {
                        b'(' | b'{' => n += 1,
                        b')' | b'}' if n > 1 => {
                            n -= 1;
                        }
                        _ => {}
                    }
                }
                our_paren_pos.col = 0;
                amount += n * cur_buf().b_ind_unclosed_wrapped;
            } else if cur_buf().b_ind_unclosed_whiteok != 0 {
                our_paren_pos.col += 1;
            } else {
                let mut col = our_paren_pos.col + 1;
                // SAFETY: `col` starts just past the `(`, so at most at the
                // line's NUL, and `ascii_iswhite` is false for a NUL -- the
                // walk cannot run off the end of `l`.
                while ascii_iswhite(c_int::from(unsafe { *l.offset(col as isize) } as u8)) {
                    col += 1;
                }
                // In case of trailing space, stay on the paren.
                // SAFETY: `col` is at most the line's NUL, per the above.
                our_paren_pos.col = if unsafe { *l.offset(col as isize) } == 0 {
                    our_paren_pos.col + 1
                } else {
                    col
                };
            }
        }
        // How indented the paren is, or the character after it if the
        // block above moved onto one.
        if our_paren_pos.col > 0 {
            // SAFETY: `our_paren_pos` is still a position in this buffer --
            // the block above only moved its column within its own line.
            let vcol = unsafe { line_vcol(our_paren_pos.lnum, our_paren_pos.col) };
            *cur_amount = (*cur_amount).min(vcol);
        }
    }

    // SAFETY: `line.theline` is a NUL-terminated copy of the cursor's line.
    if unsafe { line.starts_with(b')') } && cur_buf().b_ind_matching_paren != 0 {
        // 'cinoptions' `m`: line up with the start of the matching
        // paren's line, which `amount` already holds.
    } else if line_up_with_paren() {
        if *cur_amount != MAXCOL {
            amount = *cur_amount;
        }
    } else {
        // Add `u` for each '(' before our matching one, ignoring a
        // `(void)` before the line (`ignore_paren_col`).
        let mut col = our_paren_pos.col;
        while our_paren_pos.col > ignore_paren_col {
            our_paren_pos.col -= 1;
            // SAFETY: `our_paren_pos` is a position in the current buffer,
            // its column only walked back towards column 0, so `ml_get_pos`
            // hands back a byte of its own line.
            match unsafe { *ml_get_pos(&raw mut *our_paren_pos) } as u8 {
                b'(' => {
                    amount += cur_buf().b_ind_unclosed2;
                    col = our_paren_pos.col;
                }
                b')' => {
                    amount -= cur_buf().b_ind_unclosed2;
                    col = MAXCOL;
                }
                _ => {}
            }
        }

        // Use `(` once, when the first '(' is not inside braces.
        if col == MAXCOL {
            amount += cur_buf().b_ind_unclosed;
        } else {
            cur_win().w_cursor.lnum = our_paren_pos.lnum;
            cur_win().w_cursor.col = col;
            // SAFETY: the cursor was just put on the paren at `col` of
            // `our_paren_pos.lnum`, a position in the current buffer.
            if unsafe { find_match_paren_after_brace(cur_buf().b_ind_maxparen) }.is_some() {
                amount += cur_buf().b_ind_unclosed2;
            } else if is_if_for_while {
                amount += cur_buf().b_ind_if_for_while;
            } else {
                amount += cur_buf().b_ind_unclosed;
            }
        }

        // For a line starting with ')' take the smaller of the two, so it
        // does not get more indent than the lines above:
        //     func_long_name(               if (x
        //       arg                                 && yy
        //       )         ^ not here           )    ^ not here
        amount = amount.min(*cur_amount);
    }
    amount
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
