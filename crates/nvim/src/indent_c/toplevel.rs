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
use core::ffi::c_int;

/// The indent for a line at the top level.
///
/// # Safety
/// Moves the cursor; may unlock the current line.
pub(crate) unsafe fn indent_at_top_level(line: &Line) -> c_int {
    unsafe {
        // A line starting with an open brace forgets any prevailing indent
        // and looks like the start of a function.
        if line.starts_with(b'{') {
            return (*curbuf.get()).b_ind_first_open;
        }

        // If the NEXT line is a function declaration, this one is its type
        // specification.  Not for a comment, a terminated line, or one
        // holding a brace: `void f() {\n if (1)`.
        if line.cur_curpos.lnum < (*curbuf.get()).b_ml.ml_line_count
            && !cin_nocode(line.theline)
            && vim_strchr(line.theline, c_int::from(b'{')).is_null()
            && vim_strchr(line.theline, c_int::from(b'}')).is_null()
            && !cin_ends_in(line.theline, b":")
            && !cin_ends_in(line.theline, b",")
            && cin_isfuncdecl(None, line.cur_curpos.lnum + 1, line.cur_curpos.lnum + 1)
            && cin_isterminated(line.theline, false, true) == 0
        {
            return (*curbuf.get()).b_ind_func_type;
        }

        let mut amount = search_backwards(line);

        // Extra indent for a comment.
        if cin_iscomment(line.theline) {
            amount += (*curbuf.get()).b_ind_comment;
        }

        // Extra indent when the previous line ended in a backslash:
        //          "asdfasdf\
        //              here";
        //        char *foo = "asdf\
        //                     here";
        if line.cur_curpos.lnum > 1 {
            let above = ml_get(line.cur_curpos.lnum - 1);
            if *above != 0 && *above.add(strlen(above) - 1) as u8 == b'\\' {
                match cin_get_equal_amount(line.cur_curpos.lnum - 1) {
                    n if n > 0 => amount = n,
                    0 => amount += (*curbuf.get()).b_ind_continuation,
                    _ => {}
                }
            }
        }
        amount
    }
}

/// Search backwards until something recognisable turns up.
///
/// # Safety
/// Moves the cursor; may unlock the current line.
unsafe fn search_backwards(line: &Line) -> c_int {
    unsafe {
        let mut amount = 0;
        let mut cache = cpp_baseclass_cache_T {
            found: 0,
            lpos: lpos_T {
                lnum: MAXLNUM as linenr_T,
                col: 0,
            },
        };

        (*curwin.get()).w_cursor = line.cur_curpos;
        while (*curwin.get()).w_cursor.lnum > 1 {
            (*curwin.get()).w_cursor.lnum -= 1;
            (*curwin.get()).w_cursor.col = 0;

            // In a comment or raw string now: skip to the start of it.
            if let Some(trypos) = ind_find_start_comment_or_raw_string(None) {
                (*curwin.get()).w_cursor.lnum = trypos.lnum + 1;
                (*curwin.get()).w_cursor.col = 0;
                continue;
            }

            // The start of a C++ base-class declaration or constructor
            // initialisation?
            if (*curbuf.get()).b_ind_cpp_baseclass != 0 && cin_is_cpp_baseclass(&mut cache) {
                return get_baseclass_amount(cache.lpos.col);
            }
            let mut l = get_cursor_line_ptr().cast_const();

            // Skip preprocessor directives and blank lines.
            if cin_ispreproc_cont(&mut l, &mut (*curwin.get()).w_cursor.lnum, &mut amount)
                || cin_nocode(l)
            {
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
            let ends_in_backslash = *l != 0 && *l.add(strlen(l) - 1) as u8 == b'\\';
            if cin_ends_in(l, b",") || ends_in_backslash {
                // Take us back to the opening paren.
                if find_last_paren(l, b'(', b')')
                    && let Some(trypos) = find_match_paren((*curbuf.get()).b_ind_maxparen)
                {
                    (*curwin.get()).w_cursor = trypos;
                }

                // A line ending in ',' that is a continuation line: go back
                // to the first line with a backslash --
                //     char *foo = "bla\
                //               bla",
                //          here;
                while !ends_in_backslash && (*curwin.get()).w_cursor.lnum > 1 {
                    let above = ml_get((*curwin.get()).w_cursor.lnum - 1);
                    if *above == 0 || *above.add(strlen(above) - 1) as u8 != b'\\' {
                        break;
                    }
                    (*curwin.get()).w_cursor.lnum -= 1;
                    (*curwin.get()).w_cursor.col = 0;
                }

                amount = get_indent();
                if amount == 0 {
                    amount = cin_first_id_amount();
                }
                if amount == 0 {
                    amount = (*curbuf.get()).b_ind_continuation;
                }
                return amount;
            }

            // A function declaration, and not in a comment: the left margin.
            if cin_isfuncdecl(None, line.cur_curpos.lnum, 0) {
                return amount;
            }
            l = get_cursor_line_ptr();

            // The closing '}' of a previous function, for 'cinoptions' `fs`;
            // or a line ending in '};' (maybe followed by comments) --
            //     char *string_array[] = { "foo",
            //         /* x */ "b};ar" }; /* foobar */
            // Both put the current line at column 0.
            if *skipwhite(l) as u8 == b'}' || cin_ends_in(l, b"};") {
                return amount;
            }

            // A previous line ending in '[' is probably an array constant:
            //     something = [
            //         234,  <- extra indent
            if cin_ends_in(l, b"[") {
                return get_indent() + (*curbuf.get()).b_ind_continuation;
            }

            // A line holding only a semicolon that belongs to a previous line
            // ending in '}', e.g. before an #endif: do not increase indent.
            let mut look = skipwhite(l).cast_const();
            if *look as u8 == b';' && cin_nocode(look.add(1)) {
                let curpos_save = (*curwin.get()).w_cursor;
                while (*curwin.get()).w_cursor.lnum > 1 {
                    (*curwin.get()).w_cursor.lnum -= 1;
                    look = ml_get((*curwin.get()).w_cursor.lnum);
                    if !(cin_nocode(look)
                        || cin_ispreproc_cont(
                            &mut look,
                            &mut (*curwin.get()).w_cursor.lnum,
                            &mut amount,
                        ))
                    {
                        break;
                    }
                }
                if (*curwin.get()).w_cursor.lnum > 0 && cin_ends_in(look, b"}") {
                    return amount;
                }
                (*curwin.get()).w_cursor = curpos_save;
            }

            // If the PREVIOUS line is a function declaration, this line (and
            // the ones after it) are parameters.
            if cin_isfuncdecl(Some(&mut l), (*curwin.get()).w_cursor.lnum, 0) {
                return (*curbuf.get()).b_ind_param;
            }

            // A previous line ending in ';' whose own predecessor ends in ','
            // or '\': indent to column zero --
            //     int foo,
            //         bar;
            //     indent_to_0 here;
            if cin_ends_in(l, b";") {
                let above = ml_get((*curwin.get()).w_cursor.lnum - 1);
                if cin_ends_in(above, b",")
                    || (*above != 0 && *above.add(strlen(above) - 1) as u8 == b'\\')
                {
                    return amount;
                }
                l = get_cursor_line_ptr();
            }

            // Nothing interesting: use this line's indent.  Position on the
            // rightmost paren first, so that matching it takes us to the
            // start of the line.
            find_last_paren(l, b'(', b')');
            if let Some(trypos) = find_match_paren((*curbuf.get()).b_ind_maxparen) {
                (*curwin.get()).w_cursor = trypos;
            }
            return get_indent();
        }
        amount
    }
}
