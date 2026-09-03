//! `=<< MARKER` -- the here-document form of an assignment.
//!
//! [`heredoc_get`] collects the lines and applies `trim`'s indent rules; the
//! two `eval_*_expr_in_str` implement `eval`'s `{expr}` interpolation, which
//! is the only thing in the file that evaluates its own input.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::memory::handoff::owned_cstr;
use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::{c_char, c_int};
use core::ptr;
use core::slice;

use super::*;
use crate::types::NUL;

/// The comment character a marker line may carry after it.
const COMMENT_CHAR: c_char = b'"' as c_char;

/// Evaluate the `{expr}` block at `p` and append its value to `gap`.
/// Answers the character past the closing brace, or NULL.
///
/// `evaluate` false parses the block without running it, which is what a
/// skipped `:let` wants.
///
/// # Safety
/// `p` points at the `{` of a NUL-terminated string, writable in place.
pub unsafe fn eval_one_expr_in_str(
    p: *mut c_char,
    gap: &mut Vec<u8>,
    evaluate: bool,
) -> *mut c_char {
    // SAFETY: the caller's obligation throughout -- `p` points at the `{`
    // of a NUL-terminated string that is writable in place, and every walk
    // below stops at that NUL.
    let block_start = unsafe { skipwhite(p.add(1)) };
    let mut block_end = block_start;
    if unsafe { *block_start } == NUL as c_char {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let p = unsafe { c_str(p) };
        semsg!("E1279: Missing '}}': {p}");
        return ptr::null_mut();
    }
    if unsafe { skip_expr(&raw mut block_end, ptr::null_mut()) }.is_err() {
        return ptr::null_mut();
    }
    block_end = unsafe { skipwhite(block_end) };
    if unsafe { *block_end } != b'}' as c_char {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let p = unsafe { c_str(p) };
        semsg!("E1279: Missing '}}': {p}");
        return ptr::null_mut();
    }
    if evaluate {
        // Terminate the expression in place for `eval_to_string`.
        unsafe { *block_end = NUL as c_char };
        let expr_val = unsafe { eval_to_string(block_start, false, false) };
        unsafe { *block_end = b'}' as c_char };
        if expr_val.is_null() {
            return ptr::null_mut();
        }
        // SAFETY: `eval_to_string` answers an owned NUL-terminated string.
        gap.extend_from_slice(unsafe { cstr::bytes_at(expr_val) });
        unsafe { xfree(expr_val.cast()) };
    }
    unsafe { block_end.add(1) }
}

/// Evaluate every `{expr}` in `str` and answer the result as an allocated
/// string, or NULL.  `{{` and `}}` are the escapes for a literal brace.
///
/// # Safety
/// `str` is a NUL-terminated string, writable in place.
unsafe fn eval_all_expr_in_str(str: *mut c_char) -> *mut c_char {
    let mut text = Vec::<u8>::new();
    let mut p = str;

    // SAFETY: the caller's obligation throughout -- `str` is NUL-terminated
    // and writable in place, and every walk below stops at that NUL.
    while unsafe { *p } != NUL as c_char {
        let mut escaped_brace = false;

        // Everything up to the next brace is literal.
        let lit_start = p;
        while !matches!(unsafe { *p } as u8, b'{' | b'}' | 0) {
            p = unsafe { p.add(1) };
        }

        let here = unsafe { *p } as u8;
        if here != 0 && here == unsafe { *p.add(1) } as u8 {
            // A doubled brace: keep one of the pair in the literal part
            // and skip the other below.
            p = unsafe { p.add(1) };
            escaped_brace = true;
        } else if here == b'}' {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg0 = unsafe { c_str(str) };
            semsg!("E1278: Stray '}}' without a matching '{{': {arg0}");
            return ptr::null_mut();
        }

        let lit_len = unsafe { p.offset_from(lit_start) } as usize;
        text.extend_from_slice(unsafe { slice::from_raw_parts(lit_start.cast::<u8>(), lit_len) });
        if here == 0 {
            break;
        }
        if escaped_brace {
            p = unsafe { p.add(1) };
            continue;
        }

        p = unsafe { eval_one_expr_in_str(p, &mut text, true) };
        if p.is_null() {
            return ptr::null_mut();
        }
    }
    owned_cstr(text)
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
    // SAFETY: the caller's obligation -- a live command, which the
    // `do_cmdline` frame that owns the `exarg_T` outlives.
    let mut ea = unsafe { Ea::new(eap) };
    let mut marker_indent_len: c_int = 0;
    let mut text_indent_len: c_int = 0;
    let mut text_indent: *mut c_char = ptr::null_mut();
    let dot = [b'.' as c_char, NUL as c_char];

    // A here-document inside a string argument is the whole body,
    // newline-separated, rather than lines read from the source.
    let mut line_arg: *mut c_char = ptr::null_mut();
    let nl_ptr = unsafe { vim_strchr(cmd, b'\n' as c_int) };
    let heredoc_in_string = !nl_ptr.is_null();
    if heredoc_in_string {
        line_arg = unsafe { nl_ptr.add(1) };
        unsafe { *nl_ptr = NUL as c_char };
    } else if ea.ea_getline.is_none() {
        emsg_static(e_cannot_use_heredoc_here);
        return ptr::null_mut();
    }

    // Whether `at` starts with the four-letter `word` as a whole word.
    // A closure, so that it stays inside this function's one `unsafe`
    // block.
    let is_word = |at: *const c_char, word: &CStr| -> bool {
        debug_assert!(word.to_bytes().len() == 4);
        // SAFETY: `at` is NUL-terminated, so the fifth byte is only read
        // once the first four have proved not to hold the terminator.
        let same = unsafe { cstr::prefix_eq(at, word.as_ptr(), 4) };
        same && ascii_iswhite_or_nul(c_int::from(unsafe { *at.add(4) }))
    };

    // The optional `trim` and `eval` words before the marker, in either
    // order and either number.
    cmd = unsafe { skipwhite(cmd) };
    let mut evalstr = false;
    loop {
        if is_word(cmd, c"trim") {
            cmd = unsafe { skipwhite(cmd.add(4)) };
            // The end marker is matched with the `:let` line's own
            // indentation stripped; the body's comes from its first
            // line, which `text_indent_len == -1` asks for below.
            // SAFETY: a live command's own command line.
            let mut p = unsafe { *ea.cmdlinep };
            while ascii_iswhite(c_int::from(unsafe { *p })) {
                p = unsafe { p.add(1) };
                marker_indent_len += 1;
            }
            text_indent_len = -1;
        } else if is_word(cmd, c"eval") {
            cmd = unsafe { skipwhite(cmd.add(4)) };
            evalstr = true;
        } else {
            break;
        }
    }

    // The marker is the next word.
    let marker;
    let lead = unsafe { *cmd };
    if lead != NUL as c_char && lead != COMMENT_CHAR {
        marker = unsafe { skipwhite(cmd) };
        let p = unsafe { skiptowhite(marker) };
        let after = unsafe { *skipwhite(p) };
        if after != NUL as c_char && after != COMMENT_CHAR {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let p = unsafe { c_str(p) };
            semsg!("E488: Trailing characters: {p}");
            return ptr::null_mut();
        }
        unsafe { *p = NUL as c_char };
        // `islower` here is the locale's, not ASCII's: `_ISlower` is the
        // bit `__ctype_b_loc()`'s table sets, which in a non-C locale
        // covers more than a-z.
        let class = unsafe { *(*__ctype_b_loc()).offset(*marker as uint8_t as isize) };
        if !script_get && c_int::from(class) & _ISlower as c_int != 0 {
            let msg = c"E221: Marker cannot start with lower case letter";
            emsg_static(msg);
            return ptr::null_mut();
        }
    } else if script_get {
        // An embedded script with no marker takes '.'.
        marker = dot.as_ptr() as *mut c_char;
    } else {
        emsg_static(c"E172: Missing marker");
        return ptr::null_mut();
    }

    let mut theline: *mut c_char = ptr::null_mut();
    let mut eval_failed = false;
    let l = unsafe { tv_list_alloc(0) };
    loop {
        if heredoc_in_string {
            if unsafe { *line_arg } == NUL as c_char {
                if !script_get {
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let marker = unsafe { c_str(marker) };
                    semsg!("E990: Missing end marker '{marker}'");
                }
                break;
            }
            theline = line_arg;
            let next_line = unsafe { vim_strchr(theline, b'\n' as c_int) };
            if next_line.is_null() {
                line_arg = unsafe { line_arg.add(cstr::bytes_at(line_arg).len()) };
            } else {
                unsafe { *next_line = NUL as c_char };
                line_arg = unsafe { next_line.add(1) };
            }
        } else {
            unsafe { xfree(theline.cast()) };
            // SAFETY: a live command, whose line getter reads its own
            // cookie.
            let getline = ea.ea_getline.expect("non-null function pointer");
            theline = unsafe { getline(NUL as c_int, ea.cookie, 0, false) };
            if theline.is_null() {
                if !script_get {
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let marker = unsafe { c_str(marker) };
                    semsg!("E990: Missing end marker '{marker}'");
                }
                break;
            }
        }

        // With `trim`, skip the indent matching the `:let` line before
        // looking for the marker.
        let mut mi = 0;
        let indent = marker_indent_len as size_t;
        if marker_indent_len > 0 && unsafe { cstr::prefix_eq(theline, *ea.cmdlinep, indent) } {
            mi = marker_indent_len;
        }
        if unsafe { cstr::eq(marker, theline.offset(mi as isize)) } {
            break;
        }

        // Once interpolation has failed, the rest of the body is only
        // read to find the end marker.
        if eval_failed {
            continue;
        }

        if text_indent_len == -1 && unsafe { *theline } != NUL as c_char {
            // The body's indent is the first non-empty line's.
            let mut p = theline;
            text_indent_len = 0;
            while ascii_iswhite(c_int::from(unsafe { *p })) {
                p = unsafe { p.add(1) };
                text_indent_len += 1;
            }
            text_indent =
                unsafe { xmemdupz(theline.cast(), text_indent_len as size_t) } as *mut c_char;
        }
        // With `trim`, skip as much of that indent as this line matches.
        let mut ti = 0;
        if !text_indent.is_null() {
            while ti < text_indent_len
                && unsafe { *theline.offset(ti as isize) == *text_indent.offset(ti as isize) }
            {
                ti += 1;
            }
        }

        let str = unsafe { theline.offset(ti as isize) };
        if evalstr && ea.skip == 0 {
            let evaluated = unsafe { eval_all_expr_in_str(str) };
            if evaluated.is_null() {
                eval_failed = true;
                continue;
            }
            unsafe { tv_list_append_allocated_string(l, evaluated) };
        } else {
            unsafe { tv_list_append_string(l, str, -1) };
        }
    }

    if heredoc_in_string {
        // The next command follows the here-document in the string.
        ea.nextcmd = line_arg;
    } else {
        unsafe { xfree(theline.cast()) };
    }
    unsafe { xfree(text_indent.cast()) };

    if eval_failed {
        unsafe { tv_list_free(l) };
        return ptr::null_mut();
    }
    l
}
