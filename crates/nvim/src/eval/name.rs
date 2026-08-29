//! Scanning a variable, function or option name out of an expression.
//!
//! Three character classes decide where a name ends and they are not the
//! same: `eval_isnamec1` is what may *start* one, `eval_isnamec` what may
//! continue it (which includes `:` and `#`), and `eval_isdictc` what a
//! `.key` may contain (which includes neither).

#![deny(unsafe_op_in_unsafe_fn)]

use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::{c_char, c_int, c_void};

use crate::ascii::ascii_isdigit;
use crate::charset::{skipwhite, vim_is_ident_char};
use crate::eval::userfunc::eval_fname_script;
use crate::eval::vars::get_vim_var_partial;
use crate::eval::{
    AUTOLOAD_CHAR, FNE_CHECK_START, FNE_INCL_BR, KS_EXTRA, eval_to_string, namespace_char,
};
use crate::keycodes::{K_SPECIAL, KE_SNR};
use crate::mbyte::utfc_ptr2len;
use crate::memory::{xfree, xmalloc};
use crate::option::find_option_end;
use crate::strings::{vim_snprintf, vim_strchr};
use crate::types::{
    NUL, OptIndex, OptionSetFlags, VAR_PARTIAL, Vv, partial_T, size_t, typval_T, uint8_t,
};
use ::libc::strlen;

/// The length of the environment-variable name at the cursor, which is
/// left after it. Zero when there is none.
///
/// # Safety
/// `arg` must point at a cursor into a NUL-terminated string.
pub unsafe fn get_env_len(arg: *mut *const c_char) -> c_int {
    // SAFETY: the caller's promise -- `arg` holds a cursor into a
    // NUL-terminated string. The walk stops at the terminator, which is not
    // an identifier character, so it never leaves the string.
    let start = unsafe { *arg };
    let mut p = start;
    while unsafe { vim_is_ident_char(*p as uint8_t as c_int) } {
        p = unsafe { p.add(1) };
    }
    if p == start {
        return 0;
    }
    // SAFETY: both cursors are into the one string.
    let len = unsafe { p.offset_from(start) } as c_int;
    // SAFETY: the caller's promise about `arg`.
    unsafe { *arg = p };
    len
}

/// The length of the plain identifier at the cursor, which is left on the
/// first non-blank after it. Zero when there is none.
///
/// A `:` is part of the name only as a leading namespace letter; anywhere
/// else it ends it.
///
/// # Safety
/// As `get_env_len`.
pub unsafe fn get_id_len(arg: *mut *const c_char) -> c_int {
    // SAFETY: the caller's promise -- a cursor into a NUL-terminated
    // string. The terminator is not a name character, so the walk stops at
    // it.
    let start = unsafe { *arg };
    let mut p = start;
    loop {
        // SAFETY: `p` is inside the string.
        let c = unsafe { *p };
        if !eval_isnamec(c as c_int) {
            break;
        }
        if c == b':' as c_char {
            // SAFETY: both cursors are into the one string.
            let len = unsafe { p.offset_from(start) } as c_int;
            // SAFETY: the string's first byte is readable.
            let scope = unsafe { *start } as uint8_t as c_int;
            // SAFETY: `namespace_char` is a NUL-terminated literal.
            let scoped = unsafe { vim_strchr(namespace_char.as_ptr(), scope) }.is_null();
            if len > 1 || (len == 1 && scoped) {
                break;
            }
        }
        // SAFETY: `c` is not the terminator, so the next byte is inside.
        p = unsafe { p.add(1) };
    }
    if p == start {
        return 0;
    }
    // SAFETY: both cursors are into the one string.
    let len = unsafe { p.offset_from(start) } as c_int;
    // SAFETY: `p` is inside the string, and so is what follows its blanks.
    unsafe { *arg = skipwhite(p) };
    len
}

/// The length of the name at the cursor, expanding a `{...}` in it.
///
/// When the name held curly braces and `evaluate` is set, `alias` comes
/// back owning the expanded spelling and the answer is *its* length rather
/// than the source text's. -1 means the expansion failed.
///
/// # Safety
/// As `get_env_len`; `alias` must be valid.
pub unsafe fn get_name_len(
    arg: *mut *const c_char,
    alias: *mut *mut c_char,
    evaluate: bool,
    verbose: bool,
) -> c_int {
    // SAFETY: the caller's promise about `alias`.
    unsafe { *alias = core::ptr::null_mut() };

    // A `<SNR>` prefix arrives as the three-byte key encoding.
    // SAFETY: the caller's promise -- a cursor into a NUL-terminated
    // string. The three bytes are compared in order, so the second and the
    // third are only read once the ones before them matched, which is what
    // keeps the reads inside the string.
    let snr = unsafe { *(*arg).add(0) } == K_SPECIAL as c_char
        && unsafe { *(*arg).add(1) } == KS_EXTRA as c_char
        && unsafe { *(*arg).add(2) } == KE_SNR as c_char;
    if snr {
        // SAFETY: the prefix is three bytes of the same string.
        unsafe { *arg = (*arg).add(3) };
        // SAFETY: `arg` is still a cursor into it.
        return unsafe { get_id_len(arg) } + 3;
    }

    // `s:` and `<SID>` are a prefix on top of the name proper.
    // SAFETY: as above.
    let mut len = unsafe { eval_fname_script(*arg) };
    if len > 0 {
        // SAFETY: the answer is the length of a prefix of the string.
        unsafe { *arg = (*arg).offset(len as isize) };
    }

    let mut expr_start: *mut c_char = core::ptr::null_mut();
    let mut expr_end: *mut c_char = core::ptr::null_mut();
    let (starts, ends) = (
        (&raw mut expr_start).cast::<*const c_char>(),
        (&raw mut expr_end).cast::<*const c_char>(),
    );
    let start_flags = if len > 0 { 0 } else { FNE_CHECK_START };
    // SAFETY: the cursor names a NUL-terminated string, and the two
    // out-parameters are this frame's.
    let p = unsafe { find_name_end(*arg, starts, ends, start_flags) };

    if !expr_start.is_null() {
        if !evaluate {
            // SAFETY: `p` and the cursor are into the one string.
            len += unsafe { p.offset_from(*arg) } as c_int;
            // SAFETY: as above.
            unsafe { *arg = skipwhite(p) };
            return len;
        }
        // The prefix is part of the name being expanded, so the start
        // is stepped back over it.
        // SAFETY: the prefix was stepped over above, so stepping back over
        // it lands inside the same string.
        let from = unsafe { (*arg).offset(-(len as isize)) };
        let at = p as *mut c_char;
        // SAFETY: all four cursors are into one writable, NUL-terminated
        // string, with the braces and the end where `find_name_end` left
        // them.
        let temp_string = unsafe { make_expanded_name(from, expr_start, expr_end, at) };
        if temp_string.is_null() {
            return -1;
        }
        // SAFETY: the caller's promise about `alias`, which takes the
        // expansion over.
        unsafe { *alias = temp_string };
        // SAFETY: `p` is into the source string.
        unsafe { *arg = skipwhite(p) };
        // SAFETY: the expansion is NUL-terminated.
        return unsafe { strlen(temp_string) } as c_int;
    }

    // SAFETY: `arg` is still a cursor into the string.
    len += unsafe { get_id_len(arg) };
    // SAFETY: as above.
    if len == 0 && verbose && unsafe { **arg } as c_int != NUL {
        // SAFETY: the format takes one string, which the cursor names.
        let arg0 = unsafe { c_str(*arg) };
        semsg!("E15: Invalid expression: \"{arg0}\"");
    }
    len
}

/// The cursor stepped over one whole character.
///
/// # Safety
/// `p` must be on a character of a NUL-terminated string, and not on the
/// terminator.
#[inline]
unsafe fn step_char(p: *const c_char) -> *const c_char {
    // SAFETY: the caller's promise; `utfc_ptr2len` answers at least one and
    // never counts past the terminator.
    unsafe { p.offset(utfc_ptr2len(p as *mut c_char) as isize) }
}

/// The end of the name starting at `arg`, stepping over `{...}` and, with
/// `FNE_INCL_BR`, over `[...]` and `.key` subscripts too. `expr_start` and
/// `expr_end` come back on the outermost pair of curly braces, when there
/// was one.
///
/// # Safety
/// `arg` must be NUL-terminated; the two out-parameters both null or both
/// valid.
pub unsafe fn find_name_end(
    arg: *const c_char,
    expr_start: *mut *const c_char,
    expr_end: *mut *const c_char,
    flags: c_int,
) -> *const c_char {
    if !expr_start.is_null() {
        // SAFETY: the caller's promise -- both out-parameters are valid
        // when the first one is.
        unsafe { *expr_start = core::ptr::null() };
        // SAFETY: as above.
        unsafe { *expr_end = core::ptr::null() };
    }
    // SAFETY: the caller's promise -- `arg` is NUL-terminated, so its first
    // byte is readable.
    let first = unsafe { *arg };
    if flags & FNE_CHECK_START != 0 && !eval_isnamec1(first as c_int) && first != b'{' as c_char {
        return arg;
    }

    let incl_br = flags & FNE_INCL_BR != 0;
    let mut mb_nest = 0;
    let mut br_nest = 0;
    let mut p = arg;
    loop {
        // The byte under the cursor, read once per turn. The two
        // string-skipping arms below leave `p` on the closing quote, which
        // is the byte this already holds, so the bracket and brace tests
        // further down may use it rather than reading again.
        // SAFETY: `p` walks the NUL-terminated string and every step below
        // stops at the terminator, so this byte is inside it.
        let c = unsafe { *p };
        if c as c_int == NUL {
            break;
        }
        // SAFETY: `c` is not the terminator, so the byte after it is still
        // inside the string. The closure keeps the read where the original
        // condition had it: only a `.` under `FNE_INCL_BR` looks ahead.
        let dict_key = || c == b'.' as c_char && eval_isdictc(unsafe { *p.add(1) } as c_int);
        let in_name = eval_isnamec(c as c_int)
            || c == b'{' as c_char
            || (incl_br && (c == b'[' as c_char || dict_key()))
            || mb_nest != 0
            || br_nest != 0;
        if !in_name {
            break;
        }

        if c == b'\'' as c_char {
            // A literal string inside `[...]`.
            // SAFETY: `c` is not the terminator.
            p = unsafe { p.add(1) };
            loop {
                // SAFETY: `p` is inside the NUL-terminated string.
                let q = unsafe { *p };
                if q as c_int == NUL || q == b'\'' as c_char {
                    break;
                }
                // SAFETY: `q` is not the terminator, so `p` is on a
                // character.
                p = unsafe { step_char(p) };
            }
            // SAFETY: `p` is inside the string.
            if unsafe { *p } as c_int == NUL {
                break;
            }
        } else if c == b'"' as c_char {
            // A double-quoted string, whose escapes must be stepped over.
            // SAFETY: `c` is not the terminator.
            p = unsafe { p.add(1) };
            loop {
                // SAFETY: `p` is inside the NUL-terminated string.
                let q = unsafe { *p };
                if q as c_int == NUL || q == b'"' as c_char {
                    break;
                }
                // SAFETY: `q` is not the terminator, so the byte after it
                // is inside the string.
                if q == b'\\' as c_char && unsafe { *p.add(1) } as c_int != NUL {
                    p = unsafe { p.add(1) };
                }
                // SAFETY: `p` is on a character of the string.
                p = unsafe { step_char(p) };
            }
            // SAFETY: `p` is inside the string.
            if unsafe { *p } as c_int == NUL {
                break;
            }
        } else if br_nest == 0 && mb_nest == 0 && c == b':' as c_char {
            // A `:` ends the name unless it is the namespace one — or
            // unless a `}` came just before it, which is a curly-braces
            // name that produced the scope letter itself.
            // SAFETY: `p` and `arg` are cursors into the one string.
            let len = unsafe { p.offset_from(arg) } as c_int;
            // SAFETY: `len > 1`, so the byte before `p` is inside it.
            let after_brace = len > 1 && unsafe { *p.offset(-1) } != b'}' as c_char;
            // SAFETY: the string's first byte is readable.
            let scope = unsafe { *arg } as uint8_t as c_int;
            // SAFETY: `namespace_char` is a NUL-terminated literal.
            let scoped = unsafe { vim_strchr(namespace_char.as_ptr(), scope) }.is_null();
            if after_brace || (len == 1 && scoped) {
                break;
            }
        }

        if mb_nest == 0 {
            if c == b'[' as c_char {
                br_nest += 1;
            } else if c == b']' as c_char {
                br_nest -= 1;
            }
        }
        if br_nest == 0 {
            if c == b'{' as c_char {
                mb_nest += 1;
                // SAFETY: the caller's promise about the out-parameters.
                if !expr_start.is_null() && unsafe { *expr_start }.is_null() {
                    // SAFETY: as above.
                    unsafe { *expr_start = p };
                }
            } else if c == b'}' as c_char {
                mb_nest -= 1;
                // SAFETY: the caller's promise about the out-parameters.
                if !expr_start.is_null() && mb_nest == 0 && unsafe { *expr_end }.is_null() {
                    // SAFETY: as above.
                    unsafe { *expr_end = p };
                }
            }
        }
        // SAFETY: `c` is not the terminator, so `p` is on a character.
        p = unsafe { step_char(p) };
    }
    p
}

/// Expand the `{expr}` between `expr_start` and `expr_end` and answer the
/// whole name with it substituted in, or null when the expression failed.
/// The result is re-scanned, so nested curly braces expand too.
///
/// # Safety
/// All four pointers must be into one writable, NUL-terminated string;
/// `expr_start`/`expr_end` on the braces and `in_end` at the name's end.
pub(crate) unsafe fn make_expanded_name(
    in_start: *const c_char,
    expr_start: *mut c_char,
    expr_end: *mut c_char,
    in_end: *mut c_char,
) -> *mut c_char {
    if expr_end.is_null() || in_end.is_null() {
        return core::ptr::null_mut();
    }

    // The three pieces are cut apart in place — the braces and the end
    // become terminators — and put back before returning.
    // SAFETY: the caller's promise -- all four point into one writable
    // NUL-terminated string.
    let c1 = unsafe { *in_end };
    // SAFETY: as above -- the three cuts, put back before returning.
    unsafe { *expr_start = NUL as c_char };
    // SAFETY: as above.
    unsafe { *expr_end = NUL as c_char };
    // SAFETY: as above.
    unsafe { *in_end = NUL as c_char };

    let mut retval: *mut c_char = core::ptr::null_mut();
    // SAFETY: the text after the opening brace is its own NUL-terminated
    // string now that the closing one has been overwritten.
    let temp_result = unsafe { eval_to_string(expr_start.add(1), false, false) };
    if !temp_result.is_null() {
        // SAFETY: all four cursors are into the one string, and
        // `temp_result` is NUL-terminated.
        let before = unsafe { expr_start.offset_from(in_start) } as size_t;
        // SAFETY: as above.
        let after = unsafe { in_end.offset_from(expr_end) } as size_t;
        // SAFETY: `temp_result` is NUL-terminated.
        let retvalsize = before + unsafe { strlen(temp_result) } + after + 1;
        // SAFETY: `xmalloc` never answers NULL.
        retval = unsafe { xmalloc(retvalsize) as *mut c_char };
        // SAFETY: the tail begins after the closing brace.
        let tail = unsafe { expr_end.add(1) };
        let fmt = c"%s%s%s".as_ptr();
        // SAFETY: three NUL-terminated pieces into a buffer sized for them.
        unsafe { vim_snprintf(retval, retvalsize, fmt, in_start, temp_result, tail) };
    }
    // SAFETY: the expression's result is an owned string, and null is fine.
    unsafe { xfree(temp_result as *mut c_void) };

    // SAFETY: the three bytes cut out above are put back where they were.
    unsafe { *in_end = c1 };
    // SAFETY: as above.
    unsafe { *expr_start = b'{' as c_char };
    // SAFETY: as above.
    unsafe { *expr_end = b'}' as c_char };

    if !retval.is_null() {
        // The expansion may itself hold curly braces.
        let mut inner_start: *mut c_char = core::ptr::null_mut();
        let mut inner_end: *mut c_char = core::ptr::null_mut();
        let (starts, ends) = (
            (&raw mut inner_start).cast::<*const c_char>(),
            (&raw mut inner_end).cast::<*const c_char>(),
        );
        // SAFETY: `retval` is the NUL-terminated expansion and the two
        // out-parameters are this frame's.
        let name_end = unsafe { find_name_end(retval, starts, ends, 0) } as *mut c_char;
        if !inner_start.is_null() {
            // SAFETY: all four cursors are into `retval`, which is
            // writable and NUL-terminated.
            let expanded = unsafe { make_expanded_name(retval, inner_start, inner_end, name_end) };
            // SAFETY: the expansion copied what it needed.
            unsafe { xfree(retval as *mut c_void) };
            retval = expanded;
        }
    }
    retval
}

/// An ASCII letter, tested on the code point rather than on a byte: the
/// callers pass a `c_char` widened to `c_int`, so a multibyte lead byte
/// arrives negative and must not match.
#[inline(always)]
fn is_alpha(c: c_int) -> bool {
    (c >= b'A' as c_int && c <= b'Z' as c_int) || (c >= b'a' as c_int && c <= b'z' as c_int)
}

/// May this character be part of a variable name?
pub fn eval_isnamec(c: c_int) -> bool {
    is_alpha(c)
        || ascii_isdigit(c)
        || c == b'_' as c_int
        || c == b':' as c_int
        || c == AUTOLOAD_CHAR
}

/// May this character *start* a variable name?
pub fn eval_isnamec1(c: c_int) -> bool {
    is_alpha(c) || c == b'_' as c_int
}

/// May this character be part of a `.key` subscript? Unlike a variable
/// name, no `:` and no `#`.
pub fn eval_isdictc(c: c_int) -> bool {
    is_alpha(c) || ascii_isdigit(c) || c == b'_' as c_int
}

/// Is this partial the one `v:lua` stands for?
///
/// # Safety
/// `partial` must be null or valid.
pub unsafe fn is_luafunc(partial: *mut partial_T) -> bool {
    unsafe { partial == get_vim_var_partial(Vv::Lua) }
}

/// Is this typval `v:lua`?
///
/// # Safety
/// `tv` must be valid.
pub(crate) unsafe fn tv_is_luafunc(tv: *mut typval_T) -> bool {
    unsafe { (*tv).v_type == VAR_PARTIAL && is_luafunc((*tv).vval.v_partial) }
}

/// The end of a `v:lua.` function name, which may hold `.`, `-` and `'`
/// as well as the usual name characters.
///
/// # Safety
/// `p` must be NUL-terminated.
pub unsafe fn skip_luafunc_name(p: *const c_char) -> *const c_char {
    let mut p = p;
    loop {
        // SAFETY: the caller's promise -- `p` walks a NUL-terminated
        // string, and the terminator is none of the accepted bytes, so the
        // walk stops on it.
        let b = unsafe { *p } as u8;
        if !(b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'\'')) {
            return p;
        }
        // SAFETY: `b` is not the terminator.
        p = unsafe { p.add(1) };
    }
}

/// The length of the `v:lua.` function name at `str`, or zero when what
/// follows it is not the expected terminator.
///
/// # Safety
/// `str` must be NUL-terminated.
pub unsafe fn check_luafunc_name(str: *const c_char, paren: bool) -> c_int {
    // SAFETY: the caller's promise -- `str` is NUL-terminated.
    let p = unsafe { skip_luafunc_name(str) };
    let want = if paren { b'(' as c_char } else { NUL as c_char };
    // SAFETY: `p` is inside the same string.
    if unsafe { *p } != want {
        return 0;
    }
    // SAFETY: both cursors are into the one string.
    unsafe { p.offset_from(str) as c_int }
}

/// The end of the option name at the cursor, with `opt_idxp` and
/// `opt_flags` describing which option and which scope. The cursor is left
/// after any `g:`/`l:` prefix, but only when a name was found.
///
/// # Safety
/// `arg` must point at a cursor on the `&` or `+`; the out-parameters
/// valid.
pub unsafe fn find_option_var_end(
    arg: *mut *const c_char,
    opt_idxp: *mut OptIndex,
    opt_flags: *mut OptionSetFlags,
) -> *const c_char {
    // SAFETY: the caller's promise -- the cursor is on the `&` or `+` of a
    // NUL-terminated string, so the byte after it is inside it.
    let start = unsafe { *arg };
    let mut p = unsafe { start.add(1) };
    // SAFETY: `p` is inside the string.
    let scope = unsafe { *p };
    let scoped = scope == b'g' as c_char || scope == b'l' as c_char;
    // SAFETY: a scope letter is not the terminator, so the byte after it is
    // inside the string too. The second read happens exactly where the
    // original `&&` chain had it.
    if scoped && unsafe { *p.add(1) } == b':' as c_char {
        let flags = if scope == b'g' as c_char {
            OptionSetFlags::GLOBAL
        } else {
            OptionSetFlags::LOCAL
        };
        // SAFETY: the caller's promise about `opt_flags`.
        unsafe { *opt_flags = flags };
        // SAFETY: the two bytes just matched are inside the string.
        p = unsafe { p.add(2) };
    } else {
        // SAFETY: the caller's promise about `opt_flags`.
        unsafe { *opt_flags = OptionSetFlags::NONE };
    }
    // SAFETY: `p` is a cursor into the same string, and `opt_idxp` is the
    // caller's.
    let end = unsafe { find_option_end(p, opt_idxp) };
    // SAFETY: the caller's promise about `arg`.
    unsafe { *arg = if end.is_null() { start } else { p } };
    end
}
