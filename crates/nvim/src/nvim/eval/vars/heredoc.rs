//! `=<< MARKER` -- the here-document form of an assignment.
//!
//! [`heredoc_get`] collects the lines and applies `trim`'s indent rules; the
//! two `eval_*_expr_in_str` implement `eval`'s `{expr}` interpolation, which
//! is the only thing in the file that evaluates its own input.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr;

#[allow(unused_imports)]
use super::*;

/// The comment character a marker line may carry after it.
const COMMENT_CHAR: c_char = b'"' as c_char;

/// Evaluate the `{expr}` block at `p` and append its value to `gap`.
/// Answers the character past the closing brace, or NULL.
///
/// `evaluate` false parses the block without running it, which is what a
/// skipped `:let` wants.
///
/// # Safety
/// `p` points at the `{` of a NUL-terminated string, writable in place;
/// `gap` is a byte garray.
pub unsafe fn eval_one_expr_in_str(
    p: *mut c_char,
    gap: *mut garray_T,
    evaluate: bool,
) -> *mut c_char {
    unsafe {
        let block_start = skipwhite(p.add(1));
        let mut block_end = block_start;
        if *block_start == NUL {
            semsg(
                gettext(&raw const e_missing_close_curly_str as *const c_char),
                p,
            );
            return ptr::null_mut();
        }
        if skip_expr(&raw mut block_end, ptr::null_mut()) == FAIL {
            return ptr::null_mut();
        }
        block_end = skipwhite(block_end);
        if *block_end != b'}' as c_char {
            semsg(
                gettext(&raw const e_missing_close_curly_str as *const c_char),
                p,
            );
            return ptr::null_mut();
        }
        if evaluate {
            // Terminate the expression in place for `eval_to_string`.
            *block_end = NUL;
            let expr_val = eval_to_string(block_start, false, false);
            *block_end = b'}' as c_char;
            if expr_val.is_null() {
                return ptr::null_mut();
            }
            ga_concat(gap, expr_val);
            xfree(expr_val.cast());
        }
        block_end.add(1)
    }
}

/// Evaluate every `{expr}` in `str` and answer the result as an allocated
/// string, or NULL.  `{{` and `}}` are the escapes for a literal brace.
///
/// # Safety
/// `str` is a NUL-terminated string, writable in place.
unsafe fn eval_all_expr_in_str(str: *mut c_char) -> *mut c_char {
    unsafe {
        let mut ga = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ptr::null_mut(),
        };
        ga_init(&raw mut ga, 1, 80);
        let mut p = str;

        while *p != NUL {
            let mut escaped_brace = false;

            // Everything up to the next brace is literal.
            let lit_start = p;
            while *p != b'{' as c_char && *p != b'}' as c_char && *p != NUL {
                p = p.add(1);
            }

            if *p != NUL && *p == *p.add(1) {
                // A doubled brace: keep one of the pair in the literal part
                // and skip the other below.
                p = p.add(1);
                escaped_brace = true;
            } else if *p == b'}' as c_char {
                semsg(
                    gettext(&raw const e_stray_closing_curly_str as *const c_char),
                    str,
                );
                ga_clear(&raw mut ga);
                return ptr::null_mut();
            }

            ga_concat_len(&raw mut ga, lit_start, p.offset_from(lit_start) as size_t);
            if *p == NUL {
                break;
            }
            if escaped_brace {
                p = p.add(1);
                continue;
            }

            p = eval_one_expr_in_str(p, &raw mut ga, true);
            if p.is_null() {
                ga_clear(&raw mut ga);
                return ptr::null_mut();
            }
        }
        ga_append(&raw mut ga, NUL as uint8_t);
        ga.ga_data as *mut c_char
    }
}

/// Collect the lines of a here-document into a List, or answer NULL.
///
/// ```text
///     cmd << {marker}
///       {line1}
///       {line2}
///       ...
///     {marker}
/// ```
///
/// `trim` before the marker strips the leading indentation the *first* body
/// line has, and strips the `:let` line's own indentation when looking for
/// the end marker.  `eval` runs `{expr}` interpolation over every line.
/// `script_get` is an embedded script (`:lua <<`, `:python <<` and friends):
/// a missing marker is then `.`, a lower-case marker is allowed, and a
/// missing end marker is not an error.
///
/// # Safety
/// `eap` is a live command and `cmd` points into its argument, writable in
/// place.
pub unsafe fn heredoc_get(
    eap: *mut exarg_T,
    mut cmd: *mut c_char,
    script_get: bool,
) -> *mut list_T {
    unsafe {
        let mut marker_indent_len = 0;
        let mut text_indent_len = 0;
        let mut text_indent: *mut c_char = ptr::null_mut();
        let dot = [b'.' as c_char, NUL];

        // A here-document inside a string argument is the whole body,
        // newline-separated, rather than lines read from the source.
        let mut line_arg: *mut c_char = ptr::null_mut();
        let nl_ptr = vim_strchr(cmd, b'\n' as c_int);
        let heredoc_in_string = !nl_ptr.is_null();
        if heredoc_in_string {
            line_arg = nl_ptr.add(1);
            *nl_ptr = NUL;
        } else if (*eap).ea_getline.is_none() {
            emsg(gettext(e_cannot_use_heredoc_here.as_ptr()));
            return ptr::null_mut();
        }

        // Whether `at` starts with the four-letter `word` as a whole word.
        // A closure, so that it stays inside this function's one `unsafe`
        // block.
        let is_word = |at: *const c_char, word: &CStr| -> bool {
            debug_assert!(word.to_bytes().len() == 4);
            strncmp(at, word.as_ptr(), 4) == 0
                && (*at.add(4) == NUL || ascii_iswhite(*at.add(4) as c_int))
        };

        // The optional `trim` and `eval` words before the marker, in either
        // order and either number.
        cmd = skipwhite(cmd);
        let mut evalstr = false;
        loop {
            if is_word(cmd, c"trim") {
                cmd = skipwhite(cmd.add(4));
                // The end marker is matched with the `:let` line's own
                // indentation stripped; the body's comes from its first
                // line, which `text_indent_len == -1` asks for below.
                let mut p = *(*eap).cmdlinep;
                while ascii_iswhite(*p as c_int) {
                    p = p.add(1);
                    marker_indent_len += 1;
                }
                text_indent_len = -1;
            } else if is_word(cmd, c"eval") {
                cmd = skipwhite(cmd.add(4));
                evalstr = true;
            } else {
                break;
            }
        }

        // The marker is the next word.
        let marker;
        if *cmd != NUL && *cmd != COMMENT_CHAR {
            marker = skipwhite(cmd);
            let p = skiptowhite(marker);
            if *skipwhite(p) != NUL && *skipwhite(p) != COMMENT_CHAR {
                semsg(gettext(&raw const e_trailing_arg as *const c_char), p);
                return ptr::null_mut();
            }
            *p = NUL;
            // `islower` here is the locale's, not ASCII's: `_ISlower` is the
            // bit `__ctype_b_loc()`'s table sets, which in a non-C locale
            // covers more than a-z.
            let lower = *(*__ctype_b_loc()).offset(*marker as uint8_t as isize) as c_int
                & _ISlower as c_int
                != 0;
            if !script_get && lower {
                emsg(gettext(
                    c"E221: Marker cannot start with lower case letter".as_ptr(),
                ));
                return ptr::null_mut();
            }
        } else if script_get {
            // An embedded script with no marker takes '.'.
            marker = dot.as_ptr() as *mut c_char;
        } else {
            emsg(gettext(c"E172: Missing marker".as_ptr()));
            return ptr::null_mut();
        }

        let mut theline: *mut c_char = ptr::null_mut();
        let mut eval_failed = false;
        let l = tv_list_alloc(0);
        loop {
            if heredoc_in_string {
                if *line_arg == NUL {
                    if !script_get {
                        semsg(gettext(e_missing_end_marker_str.as_ptr()), marker);
                    }
                    break;
                }
                theline = line_arg;
                let next_line = vim_strchr(theline, b'\n' as c_int);
                if next_line.is_null() {
                    line_arg = line_arg.add(strlen(line_arg));
                } else {
                    *next_line = NUL;
                    line_arg = next_line.add(1);
                }
            } else {
                xfree(theline.cast());
                theline = (*eap).ea_getline.expect("non-null function pointer")(
                    NUL as c_int,
                    (*eap).cookie,
                    0,
                    false,
                );
                if theline.is_null() {
                    if !script_get {
                        semsg(gettext(e_missing_end_marker_str.as_ptr()), marker);
                    }
                    break;
                }
            }

            // With `trim`, skip the indent matching the `:let` line before
            // looking for the marker.
            let mut mi = 0;
            if marker_indent_len > 0
                && strncmp(theline, *(*eap).cmdlinep, marker_indent_len as size_t) == 0
            {
                mi = marker_indent_len;
            }
            if strcmp(marker, theline.offset(mi as isize)) == 0 {
                break;
            }

            // Once interpolation has failed, the rest of the body is only
            // read to find the end marker.
            if eval_failed {
                continue;
            }

            if text_indent_len == -1 && *theline != NUL {
                // The body's indent is the first non-empty line's.
                let mut p = theline;
                text_indent_len = 0;
                while ascii_iswhite(*p as c_int) {
                    p = p.add(1);
                    text_indent_len += 1;
                }
                text_indent = xmemdupz(theline.cast(), text_indent_len as size_t) as *mut c_char;
            }
            // With `trim`, skip as much of that indent as this line matches.
            let mut ti = 0;
            if !text_indent.is_null() {
                while ti < text_indent_len
                    && *theline.offset(ti as isize) == *text_indent.offset(ti as isize)
                {
                    ti += 1;
                }
            }

            let str = theline.offset(ti as isize);
            if evalstr && (*eap).skip == 0 {
                let evaluated = eval_all_expr_in_str(str);
                if evaluated.is_null() {
                    eval_failed = true;
                    continue;
                }
                tv_list_append_allocated_string(l, evaluated);
            } else {
                tv_list_append_string(l, str, -1);
            }
        }

        if heredoc_in_string {
            // The next command follows the here-document in the string.
            (*eap).nextcmd = line_arg;
        } else {
            xfree(theline.cast());
        }
        xfree(text_indent.cast());

        if eval_failed {
            tv_list_free(l);
            return ptr::null_mut();
        }
        l
    }
}
