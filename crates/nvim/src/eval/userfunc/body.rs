//! Reading a function body, one line at a time.
//!
//! `get_function_body` is `:function`'s line loop: it tracks `:if`/`:while`/
//! `:for`/`:try` nesting so that the matching `:endfunction` is the right
//! one, keeps continuation lines and comments verbatim, honours a
//! here-document inside the body, and refuses to nest more than
//! MAX_FUNC_NESTING definitions deep.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use crate::swmsg_c;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::*;
use crate::eval::Walk;
use crate::types::{FAIL, NUL, OK};

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
    // SAFETY: the caller's promise -- `eap` is the Ex command being run.
    let mut ea = unsafe { Ea::new(eap) };
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
        // SAFETY: `p` is inside a NUL-terminated line, and each `b(n)` is
        // read only once the bytes before it are known not to be the NUL.
        let b = |i: usize| unsafe { *p.add(i) } as u8;
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
        unsafe {
            strncmp(p, word.as_ptr(), n) == 0
                && (*p.add(n) == NUL as c_char || ascii_iswhite(*p.add(n) as c_int))
        }
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
                p = unsafe { vim_strchr(theline, b'\n' as c_int) };
                if p.is_null() {
                    line_arg = unsafe { line_arg.add(strlen(line_arg)) };
                } else {
                    unsafe { *p = NUL as c_char };
                    line_arg = unsafe { p.add(1) };
                }
            } else {
                unsafe { xfree(*line_to_free as *mut c_void) };
                theline = match ea.ea_getline {
                    None => unsafe { getcmdline(b':' as c_int, 0, indent, do_concat) },
                    Some(getline) => unsafe {
                        getline(b':' as c_int, ea.cookie, indent, do_concat)
                    },
                };
                unsafe { *line_to_free = theline };
            }
            if KeyTyped.get() {
                lines_left.set(Rows.get() - 1);
            }
            if theline.is_null() {
                if !skip_until.is_null() {
                    unsafe { semsg_c!(gettext(E_MISSING_HEREDOC_END_MARKER_STR), skip_until,) };
                } else {
                    emsg(gettext(c"E126: Missing :endfunction"));
                }
                break 'theend;
            }
            if show_block {
                debug_assert!(indent >= 0);
                unsafe { ui_ext_cmdline_block_append(indent as size_t, theline) };
            }

            // Detect line continuation: SOURCING_LNUM increased by more
            // than one.
            let mut sourcing_lnum_off = unsafe { get_sourced_lnum(ea.ea_getline, ea.cookie) };
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
                    || (is_heredoc && unsafe { skipwhite(theline) } == theline)
                    || unsafe { strncmp(theline, heredoc_trimmed, heredoc_trimmedlen) } == 0
                {
                    p = if heredoc_trimmed.is_null()
                        || (is_heredoc && unsafe { skipwhite(theline) } == theline)
                    {
                        theline
                    } else {
                        unsafe { theline.add(heredoc_trimmedlen) }
                    };
                    if unsafe { strcmp(p, skip_until) } == 0 {
                        unsafe { xfree(skip_until as *mut c_void) };
                        skip_until = ptr::null_mut();
                        unsafe { xfree(heredoc_trimmed as *mut c_void) };
                        heredoc_trimmed = ptr::null_mut();
                        heredoc_trimmedlen = 0;
                        do_concat = true;
                        is_heredoc = false;
                    }
                }
            } else {
                // Skip ':' and blanks.
                p = theline;
                // SAFETY: `theline` is NUL-terminated, so the walk stops.
                let mut w = unsafe { Walk::new(p) };
                while ascii_iswhite(c_int::from(w.byte())) || w.byte() == b':' {
                    w.step(1);
                }
                p = w.raw();

                // Check for "endfunction".  The count is decremented on
                // every one seen; only the outermost ends the body.
                if unsafe { checkforcmd(&raw mut p, c"endfunction".as_ptr(), 4) } && {
                    let outermost = nesting == 0;
                    nesting -= 1;
                    outermost
                } {
                    // SAFETY: `p` is inside the NUL-terminated line.
                    let mut w = unsafe { Walk::new(p) };
                    if w.byte() == b'!' {
                        w.step(1);
                    }
                    p = w.raw();
                    let mut nextcmd: *mut c_char = ptr::null_mut();
                    if w.byte() == b'|' {
                        nextcmd = unsafe { p.add(1) };
                    } else if !line_arg.is_null()
                        && unsafe { *skipwhite(line_arg) } != NUL as c_char
                    {
                        nextcmd = line_arg;
                    } else if w.byte() != NUL as u8 && w.byte() != b'"' && p_verbose.get() > 0 {
                        unsafe {
                            swmsg_c!(
                                true,
                                gettext(c"W22: Text found after :endfunction: %s").as_ptr(),
                                p,
                            )
                        };
                    }
                    if !nextcmd.is_null() {
                        // Another command follows.  If the line came from
                        // "eap" we can point into it, otherwise
                        // "eap->cmdlinep" has to take the line over.
                        ea.nextcmd = nextcmd;
                        if !unsafe { *line_to_free }.is_null() {
                            unsafe { xfree(*ea.cmdlinep as *mut c_void) };
                            unsafe { *ea.cmdlinep = *line_to_free };
                            unsafe { *line_to_free = ptr::null_mut() };
                        }
                    }
                    break;
                }

                // Increase the indent inside "if", "while", "for" and
                // "try", decrease it at "end".
                if indent > 2 && unsafe { strncmp(p, c"end".as_ptr(), 3) } == 0 {
                    indent -= 2;
                } else if unsafe { strncmp(p, c"if".as_ptr(), 2) } == 0
                    || unsafe { strncmp(p, c"wh".as_ptr(), 2) } == 0
                    || unsafe { strncmp(p, c"for".as_ptr(), 3) } == 0
                    || unsafe { strncmp(p, c"try".as_ptr(), 3) } == 0
                {
                    indent += 2;
                }

                // Check for defining a function inside this function.
                if unsafe { checkforcmd(&raw mut p, c"function".as_ptr(), 2) } {
                    if unsafe { *p } == b'!' as c_char {
                        p = unsafe { skipwhite(p.add(1)) };
                    }
                    p = unsafe { p.offset(eval_fname_script(p) as isize) };
                    let (pp, no_dict) = (&raw mut p, ptr::null_mut());
                    let no_partial = ptr::null_mut();
                    let nested = unsafe { trans_function_name(pp, true, 0, no_dict, no_partial) };
                    unsafe { xfree(nested as *mut c_void) };
                    if unsafe { *skipwhite(p) } == b'(' as c_char {
                        if nesting == MAX_FUNC_NESTING - 1 {
                            emsg(gettext(E_FUNCTION_NESTING_TOO_DEEP));
                        } else {
                            nesting += 1;
                            indent += 2;
                        }
                    }
                }

                // Check for ":append", ":change", ":insert", which run
                // until a line holding only a dot.
                p = unsafe { skip_range(p, ptr::null_mut()) };
                let tp = p;
                let ranged = unsafe {
                    checkforcmd(&raw mut p, c"append".as_ptr(), 1)
                        || checkforcmd(&raw mut p, c"change".as_ptr(), 1)
                        || checkforcmd(&raw mut p, c"insert".as_ptr(), 1)
                };
                // SAFETY: `p` is inside the NUL-terminated line.
                let after = unsafe { Walk::new(p) };
                if ranged
                    && (after.byte() == b'!'
                        || after.byte() == b'|'
                        || ascii_iswhite_nl_or_nul(c_int::from(after.byte())))
                {
                    skip_until =
                        unsafe { xmemdupz(c".".as_ptr() as *const c_void, 1) } as *mut c_char;
                } else {
                    p = tp;
                }

                // Heredoc: check for ":python <<EOF", ":lua <<EOF", etc.
                arg = unsafe { skipwhite(skiptowhite(p)) };
                if unsafe { *arg } == b'<' as c_char
                    && unsafe { *arg.add(1) } == b'<' as c_char
                    && heredoc_command(p)
                {
                    // ":python <<" continues until a dot, like ":append".
                    p = unsafe { skipwhite(arg.add(2)) };
                    if is_word(p, c"trim") {
                        // Ignore leading white space.
                        p = unsafe { skipwhite(p.add(4)) };
                        heredoc_trimmedlen =
                            unsafe { skipwhite(theline).offset_from(theline) } as size_t;
                        heredoc_trimmed =
                            unsafe { xmemdupz(theline as *const c_void, heredoc_trimmedlen) }
                                as *mut c_char;
                    }
                    skip_until = if unsafe { *p } == NUL as c_char {
                        unsafe { xmemdupz(c".".as_ptr() as *const c_void, 1) as *mut c_char }
                    } else {
                        unsafe {
                            xmemdupz(p as *const c_void, skiptowhite(p).offset_from(p) as size_t)
                                as *mut c_char
                        }
                    };
                    do_concat = false;
                    is_heredoc = true;
                }

                if !is_heredoc {
                    // Check for ":cmd v =<< [trim] EOF" and
                    // ":cmd [a, b] =<< [trim] EOF", where "cmd" is "let"
                    // or "const".
                    arg = p;
                    if unsafe { checkforcmd(&raw mut arg, c"let".as_ptr(), 2) }
                        || unsafe { checkforcmd(&raw mut p, c"const".as_ptr(), 5) }
                    {
                        let mut var_count = 0;
                        let mut semicolon = 0;
                        arg = unsafe {
                            skip_var_list(arg, &raw mut var_count, &raw mut semicolon, true)
                        } as *mut c_char;
                        if !arg.is_null() {
                            arg = unsafe { skipwhite(arg) };
                        }
                        if !arg.is_null() && unsafe { strncmp(arg, c"=<<".as_ptr(), 3) } == 0 {
                            p = unsafe { skipwhite(arg.add(3)) };
                            let mut has_trim = false;
                            loop {
                                // Both modifiers may appear, in either
                                // order and more than once.
                                if is_word(p, c"trim") {
                                    p = unsafe { skipwhite(p.add(4)) };
                                    has_trim = true;
                                } else if is_word(p, c"eval") {
                                    p = unsafe { skipwhite(p.add(4)) };
                                } else {
                                    break;
                                }
                            }
                            if has_trim {
                                heredoc_trimmedlen =
                                    unsafe { skipwhite(theline).offset_from(theline) } as size_t;
                                heredoc_trimmed = unsafe {
                                    xmemdupz(theline as *const c_void, heredoc_trimmedlen)
                                } as *mut c_char;
                            }
                            unsafe { xfree(skip_until as *mut c_void) };
                            let word = unsafe { skiptowhite(p).offset_from(p) } as size_t;
                            let marker = unsafe { xmemdupz(p as *const c_void, word) };
                            skip_until = marker as *mut c_char;
                            do_concat = false;
                            is_heredoc = true;
                        }
                    }
                }
            }

            // Add the line to the function.
            unsafe { ga_grow(newlines, 1 + sourcing_lnum_off as c_int) };

            // Copy the line to newly allocated memory.
            // `get_one_sourceline` allocates 250 bytes per line, so this
            // saves 80% on average at the cost of an alloc/free.
            unsafe { ga_push_string(newlines, xstrdup(theline)) };

            // Add NULL lines for the continuation lines, so that the line
            // count equals the index in the growarray.
            for _ in 0..sourcing_lnum_off {
                unsafe { ga_push_string(newlines, ptr::null_mut()) };
            }

            // Check for the end of eap->arg.
            if !line_arg.is_null() && unsafe { *line_arg } == NUL as c_char {
                line_arg = ptr::null_mut();
            }
        }

        // Return OK when no error was detected.
        if did_emsg.get() == 0 {
            ret = OK;
        }
    }

    unsafe { xfree(skip_until as *mut c_void) };
    unsafe { xfree(heredoc_trimmed as *mut c_void) };
    need_wait_return.set(need_wait_return.get() || saved_wait_return);
    ret
}
