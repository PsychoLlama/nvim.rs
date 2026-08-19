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
use core::ffi::c_int;

/// The indent for a line inside the unclosed paren at `our_paren_pos`.
///
/// # Safety
/// Moves the cursor; may unlock the current line.
pub(crate) unsafe fn indent_in_parens(line: &Line, our_paren_pos: pos_T) -> c_int {
    unsafe {
        let mut our_paren_pos = our_paren_pos;
        let mut cur_amount = MAXCOL;

        let mut amount = if line.starts_with(b')') && (*curbuf.get()).b_ind_paren_prev != 0 {
            // Line up with the start of the matching paren's line.
            get_indent_lnum((*curwin.get()).w_cursor.lnum - 1)
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
        // (cin_iscomment(theline))` blocks are both on this path
        // (`v0.12.4:src/nvim/indent_c.c:2430` and `:3419`).  Reproduced.
        if cin_iscomment(line.theline) {
            amount += (*curbuf.get()).b_ind_comment;
        }
        amount
    }
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
    unsafe {
        let mut amount = -1;
        let mut lnum = line.cur_curpos.lnum - 1;
        while lnum > our_paren_pos.lnum {
            let mut l = skipwhite(ml_get(lnum)).cast_const();
            if cin_nocode(l) {
                // Skip comment lines.
            } else if cin_ispreproc_cont(&mut l, &mut lnum, &mut amount) {
                // Ignore #define, #if, etc.
            } else {
                (*curwin.get()).w_cursor.lnum = lnum;

                // Skip a comment or raw string.
                let trypos = ind_find_start_CORS(None);
                if !trypos.is_null() {
                    lnum = (*trypos).lnum + 1;
                } else {
                    let trypos = find_match_paren(corr_ind_maxparen(&line.cur_curpos));
                    if !trypos.is_null()
                        && (*trypos).lnum == our_paren_pos.lnum
                        && (*trypos).col == our_paren_pos.col
                    {
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
    unsafe {
        let mut ignore_paren_col = 0;
        let mut is_if_for_while = false;

        if (*curbuf.get()).b_ind_if_for_while != 0 {
            // Find the outermost opening paren on that line and ask whether
            // it belongs to an "if", "for" or "while".
            let cursor_save = (*curwin.get()).w_cursor;
            let mut outermost = *our_paren_pos;
            loop {
                (*curwin.get()).w_cursor.lnum = outermost.lnum;
                (*curwin.get()).w_cursor.col = outermost.col;
                let trypos = find_match_paren((*curbuf.get()).b_ind_maxparen);
                if trypos.is_null() || (*trypos).lnum != outermost.lnum {
                    break;
                }
                outermost = *trypos;
            }
            (*curwin.get()).w_cursor = cursor_save;
            let text = ml_get(outermost.lnum);
            is_if_for_while = cin_is_if_for_while_before_offset(text, &mut outermost.col);
        }

        let mut look = ::core::ptr::null::<::core::ffi::c_char>();
        let mut amount = skip_label(our_paren_pos.lnum, &mut look);
        look = skipwhite(look);
        if *look as u8 == b'(' {
            // Ignore a '(' in front of the line that has a match *before* our
            // matching '(' -- a `(void)` cast, say.
            let save_lnum = (*curwin.get()).w_cursor.lnum;
            (*curwin.get()).w_cursor.lnum = our_paren_pos.lnum;
            let look_col = look.offset_from(get_cursor_line_ptr()) as colnr_T;
            (*curwin.get()).w_cursor.col = look_col + 1;
            let trypos = findmatchlimit(
                ::core::ptr::null_mut::<oparg_T>(),
                c_int::from(b')'),
                0,
                int64_t::from((*curbuf.get()).b_ind_maxparen),
            );
            if !trypos.is_null()
                && (*trypos).lnum == our_paren_pos.lnum
                && (*trypos).col < our_paren_pos.col
            {
                ignore_paren_col = (*trypos).col + 1;
            }
            (*curwin.get()).w_cursor.lnum = save_lnum;
            look = ml_get(our_paren_pos.lnum).offset(look_col as isize);
        }

        // "line up with the paren itself" applies to a zero `(` with no `k`
        // in play, and to a line whose own leading `(` is being ignored.
        // Upstream reads `*look` at both of the two places this is tested,
        // once before and once after a `getvcol`, so it stays a closure.
        let line_up_with_paren = || {
            (*curbuf.get()).b_ind_unclosed == 0 && !is_if_for_while
                || (*curbuf.get()).b_ind_unclosed_noignore == 0
                    && *look as u8 == b'('
                    && ignore_paren_col == 0
        };

        if line.starts_with(b')') || line_up_with_paren() {
            if !line.starts_with(b')') {
                *cur_amount = MAXCOL;
                let l = ml_get(our_paren_pos.lnum);
                if (*curbuf.get()).b_ind_unclosed_wrapped != 0 && cin_ends_in(l, b"(") {
                    // The paren is the last non-white character of its line:
                    // indent one `W` level per nesting level instead.
                    let mut n = 1;
                    for col in 0..our_paren_pos.col {
                        match *l.offset(col as isize) as u8 {
                            b'(' | b'{' => n += 1,
                            b')' | b'}' if n > 1 => {
                                n -= 1;
                            }
                            _ => {}
                        }
                    }
                    our_paren_pos.col = 0;
                    amount += n * (*curbuf.get()).b_ind_unclosed_wrapped;
                } else if (*curbuf.get()).b_ind_unclosed_whiteok != 0 {
                    our_paren_pos.col += 1;
                } else {
                    let mut col = our_paren_pos.col + 1;
                    while ascii_iswhite(c_int::from(*l.offset(col as isize) as u8)) {
                        col += 1;
                    }
                    // In case of trailing space, stay on the paren.
                    our_paren_pos.col = if *l.offset(col as isize) == 0 {
                        our_paren_pos.col + 1
                    } else {
                        col
                    };
                }
            }
            // How indented the paren is, or the character after it if the
            // block above moved onto one.
            if our_paren_pos.col > 0 {
                *cur_amount = (*cur_amount).min(line_vcol(our_paren_pos.lnum, our_paren_pos.col));
            }
        }

        if line.starts_with(b')') && (*curbuf.get()).b_ind_matching_paren != 0 {
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
                match *ml_get_pos(&raw mut *our_paren_pos) as u8 {
                    b'(' => {
                        amount += (*curbuf.get()).b_ind_unclosed2;
                        col = our_paren_pos.col;
                    }
                    b')' => {
                        amount -= (*curbuf.get()).b_ind_unclosed2;
                        col = MAXCOL;
                    }
                    _ => {}
                }
            }

            // Use `(` once, when the first '(' is not inside braces.
            if col == MAXCOL {
                amount += (*curbuf.get()).b_ind_unclosed;
            } else {
                (*curwin.get()).w_cursor.lnum = our_paren_pos.lnum;
                (*curwin.get()).w_cursor.col = col;
                if !find_match_paren_after_brace((*curbuf.get()).b_ind_maxparen).is_null() {
                    amount += (*curbuf.get()).b_ind_unclosed2;
                } else if is_if_for_while {
                    amount += (*curbuf.get()).b_ind_if_for_while;
                } else {
                    amount += (*curbuf.get()).b_ind_unclosed;
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
}
