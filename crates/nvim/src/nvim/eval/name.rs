//! Scanning a variable, function or option name out of an expression.
//!
//! Three character classes decide where a name ends and they are not the
//! same: `eval_isnamec1` is what may *start* one, `eval_isnamec` what may
//! continue it (which includes `:` and `#`), and `eval_isdictc` what a
//! `.key` may contain (which includes neither).

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};

use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::charset::{skipwhite, vim_isIDc};
use crate::src::nvim::eval::userfunc::eval_fname_script;
use crate::src::nvim::eval::vars::get_vim_var_partial;
use crate::src::nvim::eval::{
    AUTOLOAD_CHAR, FNE_CHECK_START, FNE_INCL_BR, K_SPECIAL, KE_SNR, KS_EXTRA, NUL, OPT_GLOBAL,
    OPT_LOCAL, VAR_PARTIAL, VV_LUA, eval_to_string, namespace_char,
};
use crate::src::nvim::main::e_invexpr2;
use crate::src::nvim::mbyte::utfc_ptr2len;
use crate::src::nvim::memory::{xfree, xmalloc};
use crate::src::nvim::message::semsg;
use crate::src::nvim::option::find_option_end;
use crate::src::nvim::os::libc::{gettext, strlen};
use crate::src::nvim::strings::{vim_snprintf, vim_strchr};
use crate::src::nvim::types::OptIndex;
use crate::src::nvim::types::{partial_T, size_t, typval_T, uint8_t};

/// The length of the environment-variable name at the cursor, which is
/// left after it. Zero when there is none.
///
/// # Safety
/// `arg` must point at a cursor into a NUL-terminated string.
pub unsafe fn get_env_len(arg: *mut *const c_char) -> c_int {
    unsafe {
        let mut p = *arg;
        while vim_isIDc(*p as uint8_t as c_int) {
            p = p.add(1);
        }
        if p == *arg {
            return 0;
        }
        let len = p.offset_from(*arg) as c_int;
        *arg = p;
        len
    }
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
    unsafe {
        let mut p = *arg;
        while eval_isnamec(*p as c_int) {
            if *p == b':' as c_char {
                let len = p.offset_from(*arg) as c_int;
                if len > 1
                    || (len == 1
                        && vim_strchr(namespace_char.as_ptr(), **arg as uint8_t as c_int).is_null())
                {
                    break;
                }
            }
            p = p.add(1);
        }
        if p == *arg {
            return 0;
        }
        let len = p.offset_from(*arg) as c_int;
        *arg = skipwhite(p);
        len
    }
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
    unsafe {
        *alias = core::ptr::null_mut();

        // A `<SNR>` prefix arrives as the three-byte key encoding.
        if *(*arg).add(0) == K_SPECIAL as c_char
            && *(*arg).add(1) == KS_EXTRA as c_char
            && *(*arg).add(2) == KE_SNR as c_char
        {
            *arg = (*arg).add(3);
            return get_id_len(arg) + 3;
        }

        // `s:` and `<SID>` are a prefix on top of the name proper.
        let mut len = eval_fname_script(*arg);
        if len > 0 {
            *arg = (*arg).offset(len as isize);
        }

        let mut expr_start: *mut c_char = core::ptr::null_mut();
        let mut expr_end: *mut c_char = core::ptr::null_mut();
        let p = find_name_end(
            *arg,
            (&raw mut expr_start).cast::<*const c_char>(),
            (&raw mut expr_end).cast::<*const c_char>(),
            if len > 0 { 0 } else { FNE_CHECK_START },
        );

        if !expr_start.is_null() {
            if !evaluate {
                len += p.offset_from(*arg) as c_int;
                *arg = skipwhite(p);
                return len;
            }
            // The prefix is part of the name being expanded, so the start
            // is stepped back over it.
            let temp_string = make_expanded_name(
                (*arg).offset(-(len as isize)),
                expr_start,
                expr_end,
                p as *mut c_char,
            );
            if temp_string.is_null() {
                return -1;
            }
            *alias = temp_string;
            *arg = skipwhite(p);
            return strlen(temp_string) as c_int;
        }

        len += get_id_len(arg);
        if len == 0 && verbose && **arg as c_int != NUL {
            semsg(gettext(e_invexpr2.ptr().cast()), *arg);
        }
        len
    }
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
    unsafe {
        if !expr_start.is_null() {
            *expr_start = core::ptr::null();
            *expr_end = core::ptr::null();
        }
        if flags & FNE_CHECK_START != 0 && !eval_isnamec1(*arg as c_int) && *arg != b'{' as c_char {
            return arg;
        }

        let mut mb_nest = 0;
        let mut br_nest = 0;
        let mut p = arg;
        while *p as c_int != NUL
            && (eval_isnamec(*p as c_int)
                || *p == b'{' as c_char
                || (flags & FNE_INCL_BR != 0
                    && (*p == b'[' as c_char
                        || (*p == b'.' as c_char && eval_isdictc(*p.add(1) as c_int))))
                || mb_nest != 0
                || br_nest != 0)
        {
            if *p == b'\'' as c_char {
                // A literal string inside `[...]`.
                p = p.add(1);
                while *p as c_int != NUL && *p != b'\'' as c_char {
                    p = p.offset(utfc_ptr2len(p as *mut c_char) as isize);
                }
                if *p as c_int == NUL {
                    break;
                }
            } else if *p == b'"' as c_char {
                // A double-quoted string, whose escapes must be stepped over.
                p = p.add(1);
                while *p as c_int != NUL && *p != b'"' as c_char {
                    if *p == b'\\' as c_char && *p.add(1) as c_int != NUL {
                        p = p.add(1);
                    }
                    p = p.offset(utfc_ptr2len(p as *mut c_char) as isize);
                }
                if *p as c_int == NUL {
                    break;
                }
            } else if br_nest == 0 && mb_nest == 0 && *p == b':' as c_char {
                // A `:` ends the name unless it is the namespace one — or
                // unless a `}` came just before it, which is a curly-braces
                // name that produced the scope letter itself.
                let len = p.offset_from(arg) as c_int;
                if (len > 1 && *p.offset(-1) != b'}' as c_char)
                    || (len == 1
                        && vim_strchr(namespace_char.as_ptr(), *arg as uint8_t as c_int).is_null())
                {
                    break;
                }
            }

            if mb_nest == 0 {
                if *p == b'[' as c_char {
                    br_nest += 1;
                } else if *p == b']' as c_char {
                    br_nest -= 1;
                }
            }
            if br_nest == 0 {
                if *p == b'{' as c_char {
                    mb_nest += 1;
                    if !expr_start.is_null() && (*expr_start).is_null() {
                        *expr_start = p;
                    }
                } else if *p == b'}' as c_char {
                    mb_nest -= 1;
                    if !expr_start.is_null() && mb_nest == 0 && (*expr_end).is_null() {
                        *expr_end = p;
                    }
                }
            }
            p = p.offset(utfc_ptr2len(p as *mut c_char) as isize);
        }
        p
    }
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
    unsafe {
        if expr_end.is_null() || in_end.is_null() {
            return core::ptr::null_mut();
        }

        // The three pieces are cut apart in place — the braces and the end
        // become terminators — and put back before returning.
        *expr_start = NUL as c_char;
        *expr_end = NUL as c_char;
        let c1 = *in_end;
        *in_end = NUL as c_char;

        let mut retval: *mut c_char = core::ptr::null_mut();
        let temp_result = eval_to_string(expr_start.add(1), false, false);
        if !temp_result.is_null() {
            let retvalsize = expr_start.offset_from(in_start) as size_t
                + strlen(temp_result)
                + in_end.offset_from(expr_end) as size_t
                + 1;
            retval = xmalloc(retvalsize) as *mut c_char;
            vim_snprintf(
                retval,
                retvalsize,
                c"%s%s%s".as_ptr(),
                in_start,
                temp_result,
                expr_end.add(1),
            );
        }
        xfree(temp_result as *mut c_void);

        *in_end = c1;
        *expr_start = b'{' as c_char;
        *expr_end = b'}' as c_char;

        if !retval.is_null() {
            // The expansion may itself hold curly braces.
            let mut inner_start: *mut c_char = core::ptr::null_mut();
            let mut inner_end: *mut c_char = core::ptr::null_mut();
            let name_end = find_name_end(
                retval,
                (&raw mut inner_start).cast::<*const c_char>(),
                (&raw mut inner_end).cast::<*const c_char>(),
                0,
            ) as *mut c_char;
            if !inner_start.is_null() {
                let expanded = make_expanded_name(retval, inner_start, inner_end, name_end);
                xfree(retval as *mut c_void);
                retval = expanded;
            }
        }
        retval
    }
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
    unsafe { partial == get_vim_var_partial(VV_LUA) }
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
    unsafe {
        let mut p = p;
        loop {
            let b = *p as u8;
            if !(b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'\'')) {
                return p;
            }
            p = p.add(1);
        }
    }
}

/// The length of the `v:lua.` function name at `str`, or zero when what
/// follows it is not the expected terminator.
///
/// # Safety
/// `str` must be NUL-terminated.
pub unsafe fn check_luafunc_name(str: *const c_char, paren: bool) -> c_int {
    unsafe {
        let p = skip_luafunc_name(str);
        let want = if paren { b'(' as c_char } else { NUL as c_char };
        if *p != want {
            return 0;
        }
        p.offset_from(str) as c_int
    }
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
    opt_flags: *mut c_int,
) -> *const c_char {
    unsafe {
        let mut p = (*arg).add(1);
        if *p == b'g' as c_char && *p.add(1) == b':' as c_char {
            *opt_flags = OPT_GLOBAL as c_int;
            p = p.add(2);
        } else if *p == b'l' as c_char && *p.add(1) == b':' as c_char {
            *opt_flags = OPT_LOCAL as c_int;
            p = p.add(2);
        } else {
            *opt_flags = 0;
        }
        let end = find_option_end(p, opt_idxp);
        *arg = if end.is_null() { *arg } else { p };
        end
    }
}
