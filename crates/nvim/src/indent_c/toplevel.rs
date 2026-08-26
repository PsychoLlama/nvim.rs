//! Indenting a line that is not inside anything at all.
//!
//! At the top level everything should basically match the line above, except
//! for the lines just after a function declaration, which are K&R-style
//! parameters and do get indented.  The 'cinoptions' letters here are `f`
//! (`first_open`, the column of a function's opening brace), `t`
//! (`func_type`, a line that is a function's return type), `p` (`param`, K&R
//! parameters), `+` (`continuation`) and `i` (`cpp_baseclass`).

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

/// The indent for a line at the top level.
///
/// # Safety
/// Moves the cursor; may unlock the current line.
pub(crate) unsafe fn indent_at_top_level(line: &Line) -> c_int {
    // A line starting with an open brace forgets any prevailing indent
    // and looks like the start of a function.
    // SAFETY: `line`'s copy of the text is alive for the whole call.
    if unsafe { line.starts_with(b'{') } {
        return cur_buf().b_ind_first_open;
    }

    // If the NEXT line is a function declaration, this one is its type
    // specification.  Not for a comment, a terminated line, or one
    // holding a brace: `void f() {\n if (1)`.
    //
    // SAFETY: `line.theline` is a NUL-terminated copy of the cursor's line,
    // and `lnum + 1` is a line of the buffer because the `ml_line_count`
    // test guards it -- the chain is left whole so that it keeps doing so.
    let is_func_type = unsafe {
        line.cur_curpos.lnum < cur_buf().b_ml.ml_line_count
            && !cin_nocode(line.theline)
            && vim_strchr(line.theline, c_int::from(b'{')).is_null()
            && vim_strchr(line.theline, c_int::from(b'}')).is_null()
            && !cin_ends_in(line.theline, b":")
            && !cin_ends_in(line.theline, b",")
            && cin_isfuncdecl(None, line.cur_curpos.lnum + 1, line.cur_curpos.lnum + 1)
            && cin_isterminated(line.theline, false, true) == 0
    };
    if is_func_type {
        return cur_buf().b_ind_func_type;
    }

    // SAFETY: the cursor is ours to move and `line` outlives the call.
    let mut amount = unsafe { search_backwards(line) };

    // Extra indent for a comment.
    // SAFETY: `line.theline` is NUL-terminated.
    if unsafe { cin_iscomment(line.theline) } {
        amount += cur_buf().b_ind_comment;
    }

    // Extra indent when the previous line ended in a backslash:
    //          "asdfasdf\
    //              here";
    //        char *foo = "asdf\
    //                     here";
    if line.cur_curpos.lnum > 1 {
        // SAFETY: `lnum - 1` is at least 1, so it is a line of the buffer,
        // and `ml_get` hands back a NUL-terminated one.
        let continued = unsafe { cin_ends_in_backslash(ml_get(line.cur_curpos.lnum - 1)) };
        if continued {
            // SAFETY: the same line number, still in range.
            match unsafe { cin_get_equal_amount(line.cur_curpos.lnum - 1) } {
                n if n > 0 => amount = n,
                0 => amount += cur_buf().b_ind_continuation,
                _ => {}
            }
        }
    }
    amount
}

/// Search backwards until something recognisable turns up.
///
/// # Safety
/// Moves the cursor; may unlock the current line.
unsafe fn search_backwards(line: &Line) -> c_int {
    let mut amount = 0;
    let mut cache = cpp_baseclass_cache_T {
        found: 0,
        lpos: lpos_T {
            lnum: MAXLNUM as linenr_T,
            col: 0,
        },
    };

    cur_win().w_cursor = line.cur_curpos;
    while cur_win().w_cursor.lnum > 1 {
        cur_win().w_cursor.lnum -= 1;
        cur_win().w_cursor.col = 0;

        // In a comment or raw string now: skip to the start of it.
        // SAFETY: on the main thread, with a cursor on a line of the buffer.
        if let Some(trypos) = unsafe { ind_find_start_comment_or_raw_string(None) } {
            cur_win().w_cursor.lnum = trypos.lnum + 1;
            cur_win().w_cursor.col = 0;
            continue;
        }

        // The start of a C++ base-class declaration or constructor
        // initialisation?
        // SAFETY: the same, and `cache` is this scan's own.
        if cur_buf().b_ind_cpp_baseclass != 0 && unsafe { cin_is_cpp_baseclass(&mut cache) } {
            // SAFETY: the same; the column came out of `cache`.
            return unsafe { get_baseclass_amount(cache.lpos.col) };
        }
        // SAFETY: the cursor is on a line of the current buffer.
        let mut l = get_cursor_line_ptr().cast_const();

        // Skip preprocessor directives and blank lines.
        //
        // SAFETY: `l` is the cursor's line, NUL-terminated.  Handing over a
        // borrow of `w_cursor.lnum` reborrows the whole window, which is
        // sound only because `cin_ispreproc_cont` reads the line it is given
        // and the current *buffer*, never `curwin`.
        let skipped = unsafe {
            cin_ispreproc_cont(&mut l, &mut cur_win().w_cursor.lnum, &mut amount) || cin_nocode(l)
        };
        if skipped {
            continue;
        }

        // A previous line ending in ',' means one level of indentation:
        //     int foo,
        //         bar;
        // Do this before checking for '}', for the sake of
        //     enum foobar
        //     {
        //       ...
        //     } foo,
        //       bar;
        // SAFETY: `l` is a NUL-terminated line.
        let ends_in_backslash = unsafe { cin_ends_in_backslash(l) };
        // SAFETY: the same.
        if unsafe { cin_ends_in(l, b",") } || ends_in_backslash {
            // Take us back to the opening paren.
            // SAFETY: `l` is the cursor's line; both move the cursor inside
            // the current buffer, and `find_match_paren` runs only when
            // `find_last_paren` found one, as upstream has it.
            let opening = unsafe {
                find_last_paren(l, b'(', b')')
                    .then(|| find_match_paren(cur_buf().b_ind_maxparen))
                    .flatten()
            };
            if let Some(trypos) = opening {
                cur_win().w_cursor = trypos;
            }

            // A line ending in ',' that is a continuation line: go back
            // to the first line with a backslash --
            //     char *foo = "bla\
            //               bla",
            //          here;
            while !ends_in_backslash && cur_win().w_cursor.lnum > 1 {
                // SAFETY: on the main thread, with a current buffer.
                let above = ml_get(cur_win().w_cursor.lnum - 1);
                // SAFETY: `ml_get` hands back a NUL-terminated line.
                if !unsafe { cin_ends_in_backslash(above) } {
                    break;
                }
                cur_win().w_cursor.lnum -= 1;
                cur_win().w_cursor.col = 0;
            }

            // SAFETY: reads the cursor's line of the current buffer.
            amount = get_indent();
            if amount == 0 {
                // SAFETY: the same.
                amount = unsafe { cin_first_id_amount() };
            }
            if amount == 0 {
                amount = cur_buf().b_ind_continuation;
            }
            return amount;
        }

        // A function declaration, and not in a comment: the left margin.
        // SAFETY: on the main thread, with a current buffer.
        if unsafe { cin_isfuncdecl(None, line.cur_curpos.lnum, 0) } {
            return amount;
        }
        // SAFETY: the cursor is on a line of the current buffer.
        l = get_cursor_line_ptr();

        // The closing '}' of a previous function, for 'cinoptions' `fs`;
        // or a line ending in '};' (maybe followed by comments) --
        //     char *string_array[] = { "foo",
        //         /* x */ "b};ar" }; /* foobar */
        // Both put the current line at column 0.
        // SAFETY: `l` is a NUL-terminated line, so `skipwhite` stops at its
        // NUL at the latest.
        if unsafe { *skipwhite(l) as u8 == b'}' || cin_ends_in(l, b"};") } {
            return amount;
        }

        // A previous line ending in '[' is probably an array constant:
        //     something = [
        //         234,  <- extra indent
        // SAFETY: `l` is a NUL-terminated line.
        if unsafe { cin_ends_in(l, b"[") } {
            // SAFETY: reads the cursor's line of the current buffer.
            return get_indent() + cur_buf().b_ind_continuation;
        }

        // A line holding only a semicolon that belongs to a previous line
        // ending in '}', e.g. before an #endif: do not increase indent.
        // SAFETY: `l` is a NUL-terminated line; `skipwhite` stops at its NUL
        // at the latest, so `look.add(1)` is at worst one past the end,
        // which `cin_nocode` only compares.
        let mut look = unsafe { skipwhite(l) }.cast_const();
        if unsafe { *look as u8 == b';' && cin_nocode(look.add(1)) } {
            let curpos_save = cur_win().w_cursor;
            while cur_win().w_cursor.lnum > 1 {
                cur_win().w_cursor.lnum -= 1;
                // SAFETY: on the main thread with a current buffer; the
                // window borrow is sound for the reason given above.
                let keep_going = unsafe {
                    look = ml_get(cur_win().w_cursor.lnum);
                    cin_nocode(look)
                        || cin_ispreproc_cont(&mut look, &mut cur_win().w_cursor.lnum, &mut amount)
                };
                if !keep_going {
                    break;
                }
            }
            // SAFETY: `look` is a NUL-terminated line.
            if cur_win().w_cursor.lnum > 0 && unsafe { cin_ends_in(look, b"}") } {
                return amount;
            }
            cur_win().w_cursor = curpos_save;
        }

        // If the PREVIOUS line is a function declaration, this line (and
        // the ones after it) are parameters.
        // SAFETY: `l` is a NUL-terminated line and the cursor is on a line
        // of the current buffer.
        if unsafe { cin_isfuncdecl(Some(&mut l), cur_win().w_cursor.lnum, 0) } {
            return cur_buf().b_ind_param;
        }

        // A previous line ending in ';' whose own predecessor ends in ','
        // or '\': indent to column zero --
        //     int foo,
        //         bar;
        //     indent_to_0 here;
        // SAFETY: `l` is a NUL-terminated line.
        if unsafe { cin_ends_in(l, b";") } {
            // SAFETY: on the main thread with a current buffer; `ml_get`
            // reports a line number of its own that is out of range.
            let above = ml_get(cur_win().w_cursor.lnum - 1);
            // SAFETY: `above` is NUL-terminated.
            if unsafe { cin_ends_in(above, b",") || cin_ends_in_backslash(above) } {
                return amount;
            }
            // SAFETY: the cursor is on a line of the current buffer.
            l = get_cursor_line_ptr();
        }

        // Nothing interesting: use this line's indent.  Position on the
        // rightmost paren first, so that matching it takes us to the
        // start of the line.
        // SAFETY: `l` is the cursor's line; both move the cursor inside the
        // current buffer.
        let opening = unsafe {
            find_last_paren(l, b'(', b')');
            find_match_paren(cur_buf().b_ind_maxparen)
        };
        if let Some(trypos) = opening {
            cur_win().w_cursor = trypos;
        }
        // SAFETY: reads the cursor's line of the current buffer.
        return get_indent();
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
