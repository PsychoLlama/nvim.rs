//! Reading a function body, one line at a time.
//!
//! `get_function_body` is `:function`'s line loop: it tracks `:if`/`:while`/
//! `:for`/`:try` nesting so that the matching `:endfunction` is the right
//! one, keeps continuation lines and comments verbatim, honours a
//! here-document inside the body, and refuses to nest more than
//! MAX_FUNC_NESTING definitions deep.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::cstr::byte_at;
use crate::ex_docmd::check_for_word_in;
use crate::memory::XString;
use crate::semsg;
use crate::swmsg;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::*;
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
/// `excmd` is a live `:function` command, `newlines` an initialised `char *`
/// garray, and `line_to_free` owns whatever the last read handed back.
pub(crate) unsafe fn get_function_body(
    excmd: &mut ExArg,
    newlines: *mut GArray,
    line_arg_in: *mut c_char,
    line_to_free: *mut *mut c_char,
    show_block: bool,
) -> c_int {
    // SAFETY: the caller's promise -- `excmd` is the Ex command being run.
    let mut saved_wait_return = need_wait_return.get();
    let mut line_arg = line_arg_in;
    let mut indent = 2;
    let mut nesting = 0;
    // The line that ends a heredoc or an `:append`, while one is open.
    let mut skip_until: Option<Vec<u8>> = None;
    let mut ret = FAIL;
    let mut is_heredoc = false;
    // The indent a `trim` heredoc's lines lose.
    let mut heredoc_trimmed: Vec<u8> = Vec::new();
    let mut do_concat = true;

    // Whether the command `cmd` starts with is one of the interpreter
    // commands that takes a `<<` heredoc, matched by its shortest
    // abbreviation plus whatever may follow: `py`/`py3`/`pyx`/`pyt`hon,
    // `pe`rl, `tc`l, `lua`, `rub`y, `mz`scheme.
    let heredoc_command = |cmd: &[u8]| {
        let b = |i: usize| cmd.get(i).copied().unwrap_or(0);
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
    let is_word = |text: &[u8], word: &[u8]| {
        text.starts_with(word) && matches!(text.get(word.len()), None | Some(b' ' | b'\t'))
    };

    'theend: {
        loop {
            if KeyTyped.get() {
                msg_scroll.set(1);
                saved_wait_return = false;
            }
            need_wait_return.set(false);

            let theline;
            if !line_arg.is_null() {
                // Use eap->arg, split up in parts by line breaks.
                theline = line_arg;
                // SAFETY: `line_arg` walks the command's own NUL-terminated
                // line, which this loop splits in place.
                let rest = unsafe { cstr::bytes_at(line_arg) };
                match rest.iter().position(|&b| b == b'\n') {
                    None => line_arg = line_arg.wrapping_add(rest.len()),
                    Some(at) => {
                        // SAFETY: as above -- the newline is inside the line.
                        unsafe { *line_arg.add(at) = NUL as c_char };
                        line_arg = line_arg.wrapping_add(at + 1);
                    }
                }
            } else {
                unsafe { xfree(*line_to_free as *mut c_void) };
                theline = match excmd.ea_getline {
                    None => getcmdline(b':' as c_int, 0, indent, do_concat),
                    Some(getline) => unsafe {
                        getline(b':' as c_int, excmd.cookie, indent, do_concat)
                    },
                };
                unsafe { *line_to_free = theline };
            }
            if KeyTyped.get() {
                lines_left.set(Rows.get() - 1);
            }
            // SAFETY: the line just read is null or NUL-terminated, and
            // nothing writes it while this walk reads it.
            let Some(line) = (unsafe { cstr::at_opt(theline) }).map(CStr::to_bytes) else {
                if let Some(marker) = &skip_until {
                    let marker = msg_bytes(marker);
                    semsg!("E1145: Missing heredoc end marker: {marker}");
                } else {
                    emsg(gettext(c"E126: Missing :endfunction"));
                }
                break 'theend;
            };
            if show_block {
                debug_assert!(indent >= 0);
                ui_ext_cmdline_block_append(indent as size_t, line);
            }

            // Detect line continuation: SOURCING_LNUM increased by more
            // than one.
            let mut sourcing_lnum_off = unsafe { get_sourced_lnum(excmd.ea_getline, excmd.cookie) };
            if sourcing_lnum() < sourcing_lnum_off {
                sourcing_lnum_off -= sourcing_lnum();
            } else {
                sourcing_lnum_off = 0;
            }

            let indented = skip::white(line);
            if let Some(marker) = &skip_until {
                // Don't check for ":endfunc" between
                // * ":append" and "."
                // * ":python <<EOF" and "EOF"
                // * ":let {var-name} =<< [trim] {marker}" and "{marker}"
                let unindented = heredoc_trimmed.is_empty() || (is_heredoc && indented == 0);
                if unindented || line.starts_with(&heredoc_trimmed) {
                    let at = if unindented { 0 } else { heredoc_trimmed.len() };
                    if line[at..] == marker[..] {
                        skip_until = None;
                        heredoc_trimmed.clear();
                        do_concat = true;
                        is_heredoc = false;
                    }
                }
            } else {
                // Skip ':' and blanks.
                let mut p = line
                    .iter()
                    .take_while(|&&b| ascii_iswhite(c_int::from(b)) || b == b':')
                    .count();

                // Check for "endfunction".  The count is decremented on
                // every one seen; only the outermost ends the body.
                if let Some(after) = check_for_word_in(line, p, b"endfunction", 4)
                    && {
                        let outermost = nesting == 0;
                        nesting -= 1;
                        outermost
                    }
                {
                    let mut after = after;
                    if byte_at(line, after) == b'!' {
                        after += 1;
                    }
                    let mut nextcmd: *mut c_char = ptr::null_mut();
                    // SAFETY: `line_arg` walks the NUL-terminated command
                    // line, where it is not null.
                    let more_args = !line_arg.is_null()
                        && skip::white(unsafe { cstr::bytes_at(line_arg) })
                            < unsafe { cstr::bytes_at(line_arg) }.len();
                    if byte_at(line, after) == b'|' {
                        nextcmd = theline.wrapping_add(after + 1);
                    } else if more_args {
                        nextcmd = line_arg;
                    } else if !matches!(byte_at(line, after), 0 | b'"') && p_verbose() > 0 {
                        let rest = msg_bytes(&line[after..]);
                        swmsg!(true, "W22: Text found after :endfunction: {rest}");
                    }
                    if !nextcmd.is_null() {
                        // Another command follows. When it is in the
                        // command's own line the offset is all that is
                        // needed; when it is in the last line the getter
                        // handed back, the command takes that line over.
                        if excmd.line.contains(nextcmd) {
                            excmd.line.next = Some(excmd.line.offset_of(nextcmd));
                        } else {
                            // SAFETY: `nextcmd` is inside the line the last
                            // read handed back, an `xmalloc` block.
                            let base = unsafe { *line_to_free };
                            let at = nextcmd.addr() - base.addr();
                            // SAFETY: as above.
                            let taken = unsafe { XString::from_raw(base) };
                            excmd.line.take_over(taken.into_vec());
                            excmd.line.next = Some(at);
                            // SAFETY: the caller's slot, now empty.
                            unsafe { *line_to_free = ptr::null_mut() };
                        }
                    }
                    break;
                }

                // Increase the indent inside "if", "while", "for" and
                // "try", decrease it at "end".
                let word = &line[p..];
                if indent > 2 && word.starts_with(b"end") {
                    indent -= 2;
                } else if word.starts_with(b"if")
                    || word.starts_with(b"wh")
                    || word.starts_with(b"for")
                    || word.starts_with(b"try")
                {
                    indent += 2;
                }

                // Check for defining a function inside this function.
                if let Some(after) = check_for_word_in(line, p, b"function", 2) {
                    p = after;
                    if byte_at(line, p) == b'!' {
                        p += 1 + skip::white(&line[p + 1..]);
                    }
                    p += fname_script_len(&line[p..]);
                    p += trans_function_name(&line[p..], true, 0, false).end;
                    if byte_at(line, p + skip::white(&line[p..])) == b'(' {
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
                // A null context is "not completing".
                p += skip_range(&line[p..], None);
                let ranged = check_for_word_in(line, p, b"append", 1)
                    .or_else(|| check_for_word_in(line, p, b"change", 1))
                    .or_else(|| check_for_word_in(line, p, b"insert", 1));
                if let Some(after) = ranged
                    && (byte_at(line, after) == b'!'
                        || byte_at(line, after) == b'|'
                        || ascii_iswhite_nl_or_nul(c_int::from(byte_at(line, after))))
                {
                    skip_until = Some(b".".to_vec());
                    p = after;
                }

                // Heredoc: check for ":python <<EOF", ":lua <<EOF", etc.
                let mut arg = p + skip::to_white(&line[p..]);
                arg += skip::white(&line[arg..]);
                if line[arg..].starts_with(b"<<") && heredoc_command(&line[p..]) {
                    // ":python <<" continues until a dot, like ":append".
                    p = arg + 2 + skip::white(&line[arg + 2..]);
                    if is_word(&line[p..], b"trim") {
                        // Ignore leading white space.
                        p += 4 + skip::white(&line[p + 4..]);
                        heredoc_trimmed = line[..indented].to_vec();
                    }
                    skip_until = Some(if p == line.len() {
                        b".".to_vec()
                    } else {
                        line[p..p + skip::to_white(&line[p..])].to_vec()
                    });
                    do_concat = false;
                    is_heredoc = true;
                }

                if !is_heredoc {
                    // Check for ":cmd v =<< [trim] EOF" and
                    // ":cmd [a, b] =<< [trim] EOF", where "cmd" is "let"
                    // or "const".
                    //
                    // Upstream steps past "let" with the copy it parses the
                    // targets from, but past "const" with the line's own
                    // cursor, so for "const" the targets are read from the
                    // word "const" itself and never reach the "=<<".
                    let targets = match check_for_word_in(line, p, b"let", 2) {
                        Some(after) => Some(after),
                        None => check_for_word_in(line, p, b"const", 5).map(|after| {
                            let at = p;
                            p = after;
                            at
                        }),
                    };
                    let operator = targets
                        .and_then(|at| skip_var_list(&line[at..], true).map(|list| at + list.end))
                        .map(|at| at + skip::white(&line[at..]));
                    if let Some(operator) = operator
                        && line[operator..].starts_with(b"=<<")
                    {
                        p = operator + 3 + skip::white(&line[operator + 3..]);
                        let mut has_trim = false;
                        loop {
                            // Both modifiers may appear, in either order
                            // and more than once.
                            if is_word(&line[p..], b"trim") {
                                p += 4 + skip::white(&line[p + 4..]);
                                has_trim = true;
                            } else if is_word(&line[p..], b"eval") {
                                p += 4 + skip::white(&line[p + 4..]);
                            } else {
                                break;
                            }
                        }
                        if has_trim {
                            heredoc_trimmed = line[..indented].to_vec();
                        }
                        skip_until = Some(line[p..p + skip::to_white(&line[p..])].to_vec());
                        do_concat = false;
                        is_heredoc = true;
                    }
                }
            }

            // Add the line to the function.
            unsafe { ga_grow(newlines, 1 + sourcing_lnum_off as c_int) };

            // Copy the line to newly allocated memory.
            // `get_one_sourceline` allocates 250 bytes per line, so this
            // saves 80% on average at the cost of an alloc/free.
            unsafe { ga_push_string(newlines, XString::from_bytes(line).into_raw()) };

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

    need_wait_return.set(need_wait_return.get() || saved_wait_return);
    ret
}
