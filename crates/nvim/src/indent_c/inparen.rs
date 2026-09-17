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
#![allow(unsafe_code)]

use super::*;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

/// The indent for a line inside the unclosed paren at `our_paren_pos`.
pub(crate) fn indent_in_parens(line: &Line, our_paren_pos: Pos) -> c_int {
    let mut our_paren_pos = our_paren_pos;
    let mut cur_amount = MAXCOL;

    let mut amount = if line.starts_with(b')') && Buf::current().b_ind_paren_prev != 0 {
        // Line up with the start of the matching paren's line.
        // SAFETY: on the main thread with a current buffer; a bad line number
        // is `ml_get`'s own to report, as upstream leaves it.
        get_indent_lnum(Win::current().w_cursor.lnum - 1)
    } else {
        // If the matching paren is more than one line away, use the
        // indent of a previous non-empty line that matches the *same*
        // paren -- a line under a different one is a different question.
        previous_line_under_same_paren(line, our_paren_pos, &mut cur_amount)
    };

    if amount == -1 {
        amount = align_with_unclosed_paren(line, &mut our_paren_pos, &mut cur_amount);
    }

    // Extra indent for a comment.  `get_c_indent` adds `b_ind_comment`
    // again for the whole "inside something" branch, so a comment inside
    // unclosed parens gets it **twice**; upstream's two `if
    // `starts_comment(theline)`) blocks are both on this path
    // (`v0.12.4:src/nvim/indent_c.c:2430` and `:3419`).  Reproduced.
    if starts_comment(line.theline(), 0) {
        amount += Buf::current().b_ind_comment;
    }
    amount
}

/// The indent of the nearest non-empty line above that is under the *same*
/// unclosed paren, or -1 when there is none.
///
/// A line starting with `)` does not take that indent -- it only lowers
/// `cur_amount`, so that the close paren cannot end up further right than the
/// lines it closes.
fn previous_line_under_same_paren(
    line: &Line,
    our_paren_pos: Pos,
    cur_amount: &mut c_int,
) -> c_int {
    let mut amount = -1;
    let mut lnum = line.cur_curpos.lnum - 1;
    while lnum > our_paren_pos.lnum {
        // A comment line, or a #define / #if continuation: ignore it.  The
        // `||` keeps upstream's order -- `preproc_start` runs only when the
        // line holds code, and its own line number is this scan's local.
        let no_code = {
            let mut lines = Lines::current();
            let text = lines.line(lnum);
            only_comment_left(text, skip::white(text))
        };
        let skip = no_code || preproc_start(&mut lnum, &mut amount);
        if !skip {
            Win::current().w_cursor.lnum = lnum;

            // Skip a comment or raw string.
            let trypos = ind_find_start_comment_or_raw_string(None);
            if let Some(trypos) = trypos {
                lnum = trypos.lnum + 1;
            } else {
                let maxparen = corr_ind_maxparen(&line.cur_curpos);
                let trypos = find_match_paren(maxparen);
                if let Some(trypos) = trypos
                    && trypos.lnum == our_paren_pos.lnum
                    && trypos.col == our_paren_pos.col
                {
                    // SAFETY: `lnum` is still a line of the current buffer.
                    amount = get_indent_lnum(lnum);
                    if line.starts_with(b')') {
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
fn align_with_unclosed_paren(
    line: &Line,
    our_paren_pos: &mut Pos,
    cur_amount: &mut c_int,
) -> c_int {
    let mut ignore_paren_col = 0;
    let mut is_if_for_while = false;

    if Buf::current().b_ind_if_for_while != 0 {
        // Find the outermost opening paren on that line and ask whether
        // it belongs to an "if", "for" or "while".
        let cursor_save = Win::current().w_cursor;
        let mut outermost = *our_paren_pos;
        loop {
            Win::current().w_cursor.lnum = outermost.lnum;
            Win::current().w_cursor.col = outermost.col;
            match find_match_paren(Buf::current().b_ind_maxparen) {
                Some(pos) if pos.lnum == outermost.lnum => outermost = pos,
                _ => break,
            }
        }
        Win::current().w_cursor = cursor_save;
        // `outermost` is a paren position in the current buffer, so the
        // cache answers with the line it sits on; `outermost.col` is this
        // function's own copy.
        let mut lines = Lines::current();
        is_if_for_while = control_clause_before(lines.line(outermost.lnum), &mut outermost.col);
    }

    let (mut amount, at) = skip_label(our_paren_pos.lnum);
    let look_col = {
        let mut lines = Lines::current();
        let text = lines.line(our_paren_pos.lnum);
        at + skip::white(&text[at.min(text.len())..])
    };
    if byte_at(Lines::current().line(our_paren_pos.lnum), look_col) == b'(' {
        // Ignore a '(' in front of the line that has a match *before* our
        // matching '(' -- a `(void)` cast, say.
        let save_lnum = Win::current().w_cursor.lnum;
        Win::current().w_cursor.lnum = our_paren_pos.lnum;
        Win::current().w_cursor.col = look_col as ColNr + 1;
        let maxparen = int64_t::from(Buf::current().b_ind_maxparen);
        // SAFETY: the cursor is just past that `(`, which is where the match
        // search starts; `findmatchlimit` takes a null `oparg` for "no
        // operator pending".
        let trypos = findmatchlimit(None, c_int::from(b')'), 0, maxparen);
        if let Some(trypos) = trypos
            && trypos.lnum == our_paren_pos.lnum
            && trypos.col < our_paren_pos.col
        {
            ignore_paren_col = trypos.col + 1;
        }
        Win::current().w_cursor.lnum = save_lnum;
    }

    // "line up with the paren itself" applies to a zero `(` with no `k`
    // in play, and to a line whose own leading `(` is being ignored.
    // Upstream reads `*look` at both of the two places this is tested,
    // once before and once after a `getvcol`, so it stays a closure.
    let line_up_with_paren = || {
        // The column is read from the cache each time: the searches in
        // between unlock lines, and it is the same line and column either
        // way -- the `skipwhite`d label tail of `our_paren_pos.lnum`.
        Buf::current().b_ind_unclosed == 0 && !is_if_for_while
            || Buf::current().b_ind_unclosed_noignore == 0
                && byte_at(Lines::current().line(our_paren_pos.lnum), look_col) == b'('
                && ignore_paren_col == 0
    };

    if line.starts_with(b')') || line_up_with_paren() {
        if !line.starts_with(b')') {
            *cur_amount = MAXCOL;
            // `our_paren_pos` is a paren position in this buffer, so the
            // cache answers with the line holding it; the `&&` keeps the
            // scan behind the option test, as upstream does.
            let mut lines = Lines::current();
            let l = lines.line(our_paren_pos.lnum);
            if Buf::current().b_ind_unclosed_wrapped != 0 && ends_in(l, 0, b"(") {
                // The paren is the last non-white character of its line:
                // indent one `W` level per nesting level instead.
                let mut n = 1;
                for col in 0..our_paren_pos.col {
                    // `col` is below `our_paren_pos.col`, the column of a
                    // `(` found on this line, so it indexes inside it.
                    match byte_at(l, col as usize) {
                        b'(' | b'{' => n += 1,
                        b')' | b'}' if n > 1 => {
                            n -= 1;
                        }
                        _ => {}
                    }
                }
                our_paren_pos.col = 0;
                amount += n * Buf::current().b_ind_unclosed_wrapped;
            } else if Buf::current().b_ind_unclosed_whiteok != 0 {
                our_paren_pos.col += 1;
            } else {
                let mut col = our_paren_pos.col + 1;
                // `col` starts just past the `(`, and the terminator is not
                // white space, so the walk cannot run off the end of `l`.
                while ascii_iswhite(c_int::from(byte_at(l, col as usize))) {
                    col += 1;
                }
                // In case of trailing space, stay on the paren.
                our_paren_pos.col = if byte_at(l, col as usize) == 0 {
                    our_paren_pos.col + 1
                } else {
                    col
                };
            }
        }
        // How indented the paren is, or the character after it if the
        // block above moved onto one.
        if our_paren_pos.col > 0 {
            let vcol = line_vcol(our_paren_pos.lnum, our_paren_pos.col);
            *cur_amount = (*cur_amount).min(vcol);
        }
    }

    if line.starts_with(b')') && Buf::current().b_ind_matching_paren != 0 {
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
            // `our_paren_pos` is a position in the current buffer, its
            // column only walked back towards column 0, so the cache answers
            // with a byte of its own line.
            let byte = byte_at(
                Lines::current().line(our_paren_pos.lnum),
                our_paren_pos.col as usize,
            );
            match byte {
                b'(' => {
                    amount += Buf::current().b_ind_unclosed2;
                    col = our_paren_pos.col;
                }
                b')' => {
                    amount -= Buf::current().b_ind_unclosed2;
                    col = MAXCOL;
                }
                _ => {}
            }
        }

        // Use `(` once, when the first '(' is not inside braces.
        if col == MAXCOL {
            amount += Buf::current().b_ind_unclosed;
        } else {
            Win::current().w_cursor.lnum = our_paren_pos.lnum;
            Win::current().w_cursor.col = col;
            if find_match_paren_after_brace(Buf::current().b_ind_maxparen).is_some() {
                amount += Buf::current().b_ind_unclosed2;
            } else if is_if_for_while {
                amount += Buf::current().b_ind_if_for_while;
            } else {
                amount += Buf::current().b_ind_unclosed;
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
