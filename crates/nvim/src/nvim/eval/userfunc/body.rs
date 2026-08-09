//! Reading a function body, one line at a time.
//!
//! `get_function_body` is `:function`'s line loop: it tracks `:if`/`:while`/
//! `:for`/`:try` nesting so that the matching `:endfunction` is the right
//! one, keeps continuation lines and comments verbatim, honours a
//! here-document inside the body, and refuses to nest more than
//! MAX_FUNC_NESTING definitions deep.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::{semsg_c, swmsg_c};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

#[allow(unused_imports)]
use super::*;

/// How many `:function` definitions may nest inside one another.
pub const MAX_FUNC_NESTING: c_int = 50;

/// Read the body of a `:function`, up to its `:endfunction`.
///
/// Lines come either from `line_arg_in` (an `:execute`d definition, split on
/// newlines) or from the command line / the script being sourced.  Each one
/// is appended to `newlines`, with a NULL line per continuation line so that
/// the index in the array stays the line number.
///
/// # Safety
/// `eap` is a live `:function` command, `newlines` an initialised `char *`
/// garray, and `line_to_free` owns whatever the last read handed back.
pub(crate) unsafe fn get_function_body(
    eap: *mut exarg_T,
    newlines: *mut garray_T,
    line_arg_in: *mut c_char,
    line_to_free: *mut *mut c_char,
    show_block: bool,
) -> c_int {
    unsafe {
        let mut saved_wait_return = need_wait_return.get();
        let mut line_arg = line_arg_in;
        let mut indent = 2;
        let mut nesting = 0;
        let mut skip_until: *mut c_char = ptr::null_mut();
        let mut ret = FAIL;
        let mut is_heredoc = false;
        let mut heredoc_trimmed: *mut c_char = ptr::null_mut();
        let mut heredoc_trimmedlen: size_t = 0;
        let mut do_concat = true;

        // Whether the command at `p` is one of the interpreter commands that
        // takes a `<<` heredoc, matched by its shortest abbreviation plus
        // whatever may follow: `py`/`py3`/`pyx`/`pyt`hon, `pe`rl, `tc`l,
        // `lua`, `rub`y, `mz`scheme.  Each `p[n]` is read only once the
        // bytes before it are known not to be the terminator.
        let heredoc_command = |p: *const c_char| {
            let b = |i: usize| *p.add(i) as u8;
            (b(0) == b'p'
                && b(1) == b'y'
                && (!b(2).is_ascii_alphanumeric()
                    || b(2) == b't'
                    || ((b(2) == b'3' || b(2) == b'x') && !b(3).is_ascii_alphabetic())))
                || (b(0) == b'p' && b(1) == b'e' && (!b(2).is_ascii_alphabetic() || b(2) == b'r'))
                || (b(0) == b't' && b(1) == b'c' && (!b(2).is_ascii_alphabetic() || b(2) == b'l'))
                || (b(0) == b'l' && b(1) == b'u' && b(2) == b'a' && !b(3).is_ascii_alphabetic())
                || (b(0) == b'r'
                    && b(1) == b'u'
                    && b(2) == b'b'
                    && (!b(3).is_ascii_alphabetic() || b(3) == b'y'))
                || (b(0) == b'm' && b(1) == b'z' && (!b(2).is_ascii_alphabetic() || b(2) == b's'))
        };

        // `[trim]` in a heredoc introducer: the body's lines then have the
        // introducer's own indent stripped.
        let is_word = |p: *const c_char, word: &CStr| {
            let n = word.count_bytes();
            strncmp(p, word.as_ptr(), n) == 0
                && (*p.add(n) == NUL as c_char || ascii_iswhite(*p.add(n) as c_int))
        };

        'theend: {
            loop {
                if KeyTyped.get() {
                    msg_scroll.set(1);
                    saved_wait_return = false;
                }
                need_wait_return.set(false);

                let theline;
                let mut p;
                let mut arg: *mut c_char;
                if !line_arg.is_null() {
                    // Use eap->arg, split up in parts by line breaks.
                    theline = line_arg;
                    p = vim_strchr(theline, b'\n' as c_int);
                    if p.is_null() {
                        line_arg = line_arg.add(strlen(line_arg));
                    } else {
                        *p = NUL as c_char;
                        line_arg = p.add(1);
                    }
                } else {
                    xfree(*line_to_free as *mut c_void);
                    theline = match (*eap).ea_getline {
                        None => getcmdline(b':' as c_int, 0, indent, do_concat),
                        Some(getline) => getline(b':' as c_int, (*eap).cookie, indent, do_concat),
                    };
                    *line_to_free = theline;
                }
                if KeyTyped.get() {
                    lines_left.set(Rows.get() - 1);
                }
                if theline.is_null() {
                    if !skip_until.is_null() {
                        semsg_c!(
                            gettext(E_MISSING_HEREDOC_END_MARKER_STR.as_ptr()),
                            skip_until,
                        );
                    } else {
                        emsg(gettext(c"E126: Missing :endfunction".as_ptr()));
                    }
                    break 'theend;
                }
                if show_block {
                    debug_assert!(indent >= 0);
                    ui_ext_cmdline_block_append(indent as size_t, theline);
                }

                // Detect line continuation: SOURCING_LNUM increased by more
                // than one.
                let mut sourcing_lnum_off = get_sourced_lnum((*eap).ea_getline, (*eap).cookie);
                if sourcing_lnum() < sourcing_lnum_off {
                    sourcing_lnum_off -= sourcing_lnum();
                } else {
                    sourcing_lnum_off = 0;
                }

                if !skip_until.is_null() {
                    // Don't check for ":endfunc" between
                    // * ":append" and "."
                    // * ":python <<EOF" and "EOF"
                    // * ":let {var-name} =<< [trim] {marker}" and "{marker}"
                    if heredoc_trimmed.is_null()
                        || (is_heredoc && skipwhite(theline) == theline)
                        || strncmp(theline, heredoc_trimmed, heredoc_trimmedlen) == 0
                    {
                        p = if heredoc_trimmed.is_null() {
                            theline
                        } else if is_heredoc && skipwhite(theline) == theline {
                            theline
                        } else {
                            theline.add(heredoc_trimmedlen)
                        };
                        if strcmp(p, skip_until) == 0 {
                            xfree(skip_until as *mut c_void);
                            skip_until = ptr::null_mut();
                            xfree(heredoc_trimmed as *mut c_void);
                            heredoc_trimmed = ptr::null_mut();
                            heredoc_trimmedlen = 0;
                            do_concat = true;
                            is_heredoc = false;
                        }
                    }
                } else {
                    // Skip ':' and blanks.
                    p = theline;
                    while ascii_iswhite(*p as c_int) || *p == b':' as c_char {
                        p = p.add(1);
                    }

                    // Check for "endfunction".  The count is decremented on
                    // every one seen; only the outermost ends the body.
                    if checkforcmd(&raw mut p, c"endfunction".as_ptr(), 4) && {
                        let outermost = nesting == 0;
                        nesting -= 1;
                        outermost
                    } {
                        if *p == b'!' as c_char {
                            p = p.add(1);
                        }
                        let mut nextcmd: *mut c_char = ptr::null_mut();
                        if *p == b'|' as c_char {
                            nextcmd = p.add(1);
                        } else if !line_arg.is_null() && *skipwhite(line_arg) != NUL as c_char {
                            nextcmd = line_arg;
                        } else if *p != NUL as c_char && *p != b'"' as c_char && p_verbose.get() > 0
                        {
                            swmsg_c!(
                                true,
                                gettext(c"W22: Text found after :endfunction: %s".as_ptr()),
                                p,
                            );
                        }
                        if !nextcmd.is_null() {
                            // Another command follows.  If the line came from
                            // "eap" we can point into it, otherwise
                            // "eap->cmdlinep" has to take the line over.
                            (*eap).nextcmd = nextcmd;
                            if !(*line_to_free).is_null() {
                                xfree(*(*eap).cmdlinep as *mut c_void);
                                *(*eap).cmdlinep = *line_to_free;
                                *line_to_free = ptr::null_mut();
                            }
                        }
                        break;
                    }

                    // Increase the indent inside "if", "while", "for" and
                    // "try", decrease it at "end".
                    if indent > 2 && strncmp(p, c"end".as_ptr(), 3) == 0 {
                        indent -= 2;
                    } else if strncmp(p, c"if".as_ptr(), 2) == 0
                        || strncmp(p, c"wh".as_ptr(), 2) == 0
                        || strncmp(p, c"for".as_ptr(), 3) == 0
                        || strncmp(p, c"try".as_ptr(), 3) == 0
                    {
                        indent += 2;
                    }

                    // Check for defining a function inside this function.
                    if checkforcmd(&raw mut p, c"function".as_ptr(), 2) {
                        if *p == b'!' as c_char {
                            p = skipwhite(p.add(1));
                        }
                        p = p.offset(eval_fname_script(p) as isize);
                        xfree(trans_function_name(
                            &raw mut p,
                            true,
                            0,
                            ptr::null_mut(),
                            ptr::null_mut(),
                        ) as *mut c_void);
                        if *skipwhite(p) == b'(' as c_char {
                            if nesting == MAX_FUNC_NESTING - 1 {
                                emsg(gettext(E_FUNCTION_NESTING_TOO_DEEP.as_ptr()));
                            } else {
                                nesting += 1;
                                indent += 2;
                            }
                        }
                    }

                    // Check for ":append", ":change", ":insert", which run
                    // until a line holding only a dot.
                    p = skip_range(p, ptr::null_mut());
                    let tp = p;
                    if (checkforcmd(&raw mut p, c"append".as_ptr(), 1)
                        || checkforcmd(&raw mut p, c"change".as_ptr(), 1)
                        || checkforcmd(&raw mut p, c"insert".as_ptr(), 1))
                        && (*p == b'!' as c_char
                            || *p == b'|' as c_char
                            || ascii_iswhite_nl_or_nul(*p as c_int))
                    {
                        skip_until = xmemdupz(c".".as_ptr() as *const c_void, 1) as *mut c_char;
                    } else {
                        p = tp;
                    }

                    // Heredoc: check for ":python <<EOF", ":lua <<EOF", etc.
                    arg = skipwhite(skiptowhite(p));
                    if *arg == b'<' as c_char && *arg.add(1) == b'<' as c_char && heredoc_command(p)
                    {
                        // ":python <<" continues until a dot, like ":append".
                        p = skipwhite(arg.add(2));
                        if is_word(p, c"trim") {
                            // Ignore leading white space.
                            p = skipwhite(p.add(4));
                            heredoc_trimmedlen = skipwhite(theline).offset_from(theline) as size_t;
                            heredoc_trimmed = xmemdupz(theline as *const c_void, heredoc_trimmedlen)
                                as *mut c_char;
                        }
                        skip_until = if *p == NUL as c_char {
                            xmemdupz(c".".as_ptr() as *const c_void, 1) as *mut c_char
                        } else {
                            xmemdupz(p as *const c_void, skiptowhite(p).offset_from(p) as size_t)
                                as *mut c_char
                        };
                        do_concat = false;
                        is_heredoc = true;
                    }

                    if !is_heredoc {
                        // Check for ":cmd v =<< [trim] EOF" and
                        // ":cmd [a, b] =<< [trim] EOF", where "cmd" is "let"
                        // or "const".
                        arg = p;
                        if checkforcmd(&raw mut arg, c"let".as_ptr(), 2)
                            || checkforcmd(&raw mut p, c"const".as_ptr(), 5)
                        {
                            let mut var_count = 0;
                            let mut semicolon = 0;
                            arg = skip_var_list(arg, &raw mut var_count, &raw mut semicolon, true)
                                as *mut c_char;
                            if !arg.is_null() {
                                arg = skipwhite(arg);
                            }
                            if !arg.is_null() && strncmp(arg, c"=<<".as_ptr(), 3) == 0 {
                                p = skipwhite(arg.add(3));
                                let mut has_trim = false;
                                loop {
                                    // Both modifiers may appear, in either
                                    // order and more than once.
                                    if is_word(p, c"trim") {
                                        p = skipwhite(p.add(4));
                                        has_trim = true;
                                    } else if is_word(p, c"eval") {
                                        p = skipwhite(p.add(4));
                                    } else {
                                        break;
                                    }
                                }
                                if has_trim {
                                    heredoc_trimmedlen =
                                        skipwhite(theline).offset_from(theline) as size_t;
                                    heredoc_trimmed =
                                        xmemdupz(theline as *const c_void, heredoc_trimmedlen)
                                            as *mut c_char;
                                }
                                xfree(skip_until as *mut c_void);
                                skip_until = xmemdupz(
                                    p as *const c_void,
                                    skiptowhite(p).offset_from(p) as size_t,
                                ) as *mut c_char;
                                do_concat = false;
                                is_heredoc = true;
                            }
                        }
                    }
                }

                // Add the line to the function.
                ga_grow(newlines, 1 + sourcing_lnum_off as c_int);

                // Copy the line to newly allocated memory.
                // `get_one_sourceline` allocates 250 bytes per line, so this
                // saves 80% on average at the cost of an alloc/free.
                ga_push_string(newlines, xstrdup(theline));

                // Add NULL lines for the continuation lines, so that the line
                // count equals the index in the growarray.
                for _ in 0..sourcing_lnum_off {
                    ga_push_string(newlines, ptr::null_mut());
                }

                // Check for the end of eap->arg.
                if !line_arg.is_null() && *line_arg == NUL as c_char {
                    line_arg = ptr::null_mut();
                }
            }

            // Return OK when no error was detected.
            if did_emsg.get() == 0 {
                ret = OK;
            }
        }

        xfree(skip_until as *mut c_void);
        xfree(heredoc_trimmed as *mut c_void);
        need_wait_return.set(need_wait_return.get() || saved_wait_return);
        ret
    }
}
