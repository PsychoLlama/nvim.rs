//! Indenting a line that is not inside anything at all.
//!
//! At the top level everything should basically match the line above, except
//! for the lines just after a function declaration, which are K&R-style
//! parameters and do get indented.  The 'cinoptions' letters here are `f`
//! (`first_open`, the column of a function's opening brace), `t`
//! (`func_type`, a line that is a function's return type), `p` (`param`, K&R
//! parameters), `+` (`continuation`) and `i` (`cpp_baseclass`).

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

/// The indent for a line at the top level.
pub(crate) fn indent_at_top_level(line: &Line) -> c_int {
    // A line starting with an open brace forgets any prevailing indent
    // and looks like the start of a function.
    if line.starts_with(b'{') {
        return Buf::current().b_ind_first_open;
    }

    // If the NEXT line is a function declaration, this one is its type
    // specification.  Not for a comment, a terminated line, or one
    // holding a brace: `void f() {\n if (1)`.
    //
    // `lnum + 1` is a line of the buffer because the `ml_line_count` test
    // guards it -- the chain is left whole so that it keeps doing so, and so
    // that the search only runs once the cheap text tests have passed.
    let theline = line.theline();
    let is_func_type = line.cur_curpos.lnum < Buf::current().b_ml.ml_line_count
        && !only_comment_left(theline, 0)
        && !theline.contains(&b'{')
        && !theline.contains(&b'}')
        && !ends_in(theline, 0, b":")
        && !ends_in(theline, 0, b",")
        && is_func_decl(line.cur_curpos.lnum + 1, line.cur_curpos.lnum + 1)
        && terminator(theline, 0, false, true) == 0;
    if is_func_type {
        return Buf::current().b_ind_func_type;
    }

    let mut amount = search_backwards(line);

    // Extra indent for a comment.
    if starts_comment(line.theline(), 0) {
        amount += Buf::current().b_ind_comment;
    }

    // Extra indent when the previous line ended in a backslash:
    //          "asdfasdf\
    //              here";
    //        char *foo = "asdf\
    //                     here";
    if line.cur_curpos.lnum > 1 {
        // `lnum - 1` is at least 1, so it is a line of the buffer.
        let above = line.cur_curpos.lnum - 1;
        let continued = ends_in_backslash(Lines::current().line(above));
        if continued {
            match equal_amount(above) {
                n if n > 0 => amount = n,
                0 => amount += Buf::current().b_ind_continuation,
                _ => {}
            }
        }
    }
    amount
}

/// Search backwards until something recognisable turns up.
fn search_backwards(line: &Line) -> c_int {
    let mut amount = 0;
    let mut cache = CppBaseclassCache {
        found: 0,
        lpos: LPos {
            lnum: MAXLNUM,
            col: 0,
        },
    };

    Win::current().w_cursor = line.cur_curpos;
    while Win::current().w_cursor.lnum > 1 {
        Win::current().w_cursor.lnum -= 1;
        Win::current().w_cursor.col = 0;

        // In a comment or raw string now: skip to the start of it.
        if let Some(trypos) = ind_find_start_comment_or_raw_string(None) {
            Win::current().w_cursor.lnum = trypos.lnum + 1;
            Win::current().w_cursor.col = 0;
            continue;
        }

        // The start of a C++ base-class declaration or constructor
        // initialisation?
        if Buf::current().b_ind_cpp_baseclass != 0 && in_baseclass_list(&mut cache) {
            return get_baseclass_amount(cache.lpos.col);
        }
        // Skip preprocessor directives and blank lines.  `preproc_start`
        // may move the cursor's line number, so the text is read after it.
        let skipped = preproc_start(&mut Win::current().w_cursor.lnum, &mut amount) || {
            let cursor_lnum = Win::current().w_cursor.lnum;
            only_comment_left(Lines::current().line(cursor_lnum), 0)
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
        let cursor_lnum = Win::current().w_cursor.lnum;
        let (continued, ends_in_comma) = {
            let mut lines = Lines::current();
            let l = lines.line(cursor_lnum);
            (ends_in_backslash(l), ends_in(l, 0, b","))
        };
        if ends_in_comma || continued {
            // Take us back to the opening paren.  The borrow ends with the
            // statement: the match search reads other lines, and only runs
            // when `find_last_paren` found one, as upstream has it.
            let has_paren = find_last_paren(Lines::current().line(cursor_lnum), b'(', b')');
            let opening = has_paren
                .then(|| find_match_paren(Buf::current().b_ind_maxparen))
                .flatten();
            if let Some(trypos) = opening {
                Win::current().w_cursor = trypos;
            }

            // A line ending in ',' that is a continuation line: go back
            // to the first line with a backslash --
            //     char *foo = "bla\
            //               bla",
            //          here;
            while !continued && Win::current().w_cursor.lnum > 1 {
                let above = Win::current().w_cursor.lnum - 1;
                if !ends_in_backslash(Lines::current().line(above)) {
                    break;
                }
                Win::current().w_cursor.lnum -= 1;
                Win::current().w_cursor.col = 0;
            }

            // SAFETY: reads the cursor's line of the current buffer.
            amount = get_indent();
            if amount == 0 {
                amount = first_id_amount();
            }
            if amount == 0 {
                amount = Buf::current().b_ind_continuation;
            }
            return amount;
        }

        // A function declaration, and not in a comment: the left margin.
        if is_func_decl(line.cur_curpos.lnum, 0) {
            return amount;
        }

        // The closing '}' of a previous function, for 'cinoptions' `fs`;
        // or a line ending in '};' (maybe followed by comments) --
        //     char *string_array[] = { "foo",
        //         /* x */ "b};ar" }; /* foobar */
        // Both put the current line at column 0.  A previous line ending in
        // '[' is probably an array constant:
        //     something = [
        //         234,  <- extra indent
        let cursor_lnum = Win::current().w_cursor.lnum;
        let (closes, opens_array) = {
            let mut lines = Lines::current();
            let l = lines.line(cursor_lnum);
            (
                byte_at(l, skip::white(l)) == b'}' || ends_in(l, 0, b"};"),
                ends_in(l, 0, b"["),
            )
        };
        if closes {
            return amount;
        }
        if opens_array {
            // SAFETY: reads the cursor's line of the current buffer.
            return get_indent() + Buf::current().b_ind_continuation;
        }

        // A line holding only a semicolon that belongs to a previous line
        // ending in '}', e.g. before an #endif: do not increase indent.
        let lone_semicolon = {
            let mut lines = Lines::current();
            let l = lines.line(cursor_lnum);
            let at = skip::white(l);
            byte_at(l, at) == b';' && only_comment_left(l, at + 1)
        };
        if lone_semicolon {
            let curpos_save = Win::current().w_cursor;
            while Win::current().w_cursor.lnum > 1 {
                Win::current().w_cursor.lnum -= 1;
                let lnum = Win::current().w_cursor.lnum;
                let no_code = only_comment_left(Lines::current().line(lnum), 0);
                let keep_going =
                    no_code || preproc_start(&mut Win::current().w_cursor.lnum, &mut amount);
                if !keep_going {
                    break;
                }
            }
            // Upstream reads the line `cin_ispreproc_cont` last left it on,
            // which is the cursor's own either way.
            let lnum = Win::current().w_cursor.lnum;
            if lnum > 0 && ends_in(Lines::current().line(lnum), 0, b"}") {
                return amount;
            }
            Win::current().w_cursor = curpos_save;
        }

        // If the PREVIOUS line is a function declaration, this line (and
        // the ones after it) are parameters.
        if is_func_decl(Win::current().w_cursor.lnum, 0) {
            return Buf::current().b_ind_param;
        }

        // A previous line ending in ';' whose own predecessor ends in ','
        // or '\': indent to column zero --
        //     int foo,
        //         bar;
        //     indent_to_0 here;
        let cursor_lnum = Win::current().w_cursor.lnum;
        if ends_in(Lines::current().line(cursor_lnum), 0, b";") {
            // Line 0 is line 1, as it is for `ml_get`.
            let mut lines = Lines::current();
            let above = lines.line(cursor_lnum - 1);
            if ends_in(above, 0, b",") || ends_in_backslash(above) {
                return amount;
            }
        }

        // Nothing interesting: use this line's indent.  Position on the
        // rightmost paren first, so that matching it takes us to the
        // start of the line.  The borrow ends with the statement.
        find_last_paren(Lines::current().line(cursor_lnum), b'(', b')');
        if let Some(trypos) = find_match_paren(Buf::current().b_ind_maxparen) {
            Win::current().w_cursor = trypos;
        }
        // SAFETY: reads the cursor's line of the current buffer.
        return get_indent();
    }
    amount
}
