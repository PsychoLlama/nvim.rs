//! Matching and substituting with a regexp built from an expression.
//!
//! Both entry points compile their pattern with 'cpoptions' emptied, so
//! that a user's `cpo` flags cannot change what an expression's pattern
//! means. Restoring it is not a plain assignment — see `do_string_sub`.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr::{copy_nonoverlapping, null_mut};

use crate::api::private::helpers::cstr_as_string;
use crate::eval::{NUL, REGSUB_COPY, REGSUB_MAGIC, kOptValTypeString};
use crate::garray::{ga_clear, ga_grow, ga_init};
use crate::main::{empty_string_option, p_cpo, p_ic};
use crate::mbyte::utfc_ptr2len;
use crate::option::set_option_value_give_err;
use crate::options::kOptCpoptions;
use crate::optionstr::free_string_option;
use crate::regexp::{RE_MAGIC, RE_STRING, vim_regcomp, vim_regexec_nl, vim_regfree, vim_regsub};
use crate::strings::xstrnsave;
use crate::types::{
    OptVal, OptValData, colnr_T, garray_T, regmatch_T, regprog_T, size_t, typval_T,
};
use ::libc::strcpy;

/// A `regmatch_T` with nothing in it.
const EMPTY_REGMATCH: regmatch_T = regmatch_T {
    regprog: null_mut::<regprog_T>(),
    startp: [null_mut::<c_char>(); 10],
    endp: [null_mut::<c_char>(); 10],
    rm_matchcol: 0,
    rm_ic: false,
};

/// An empty garray, sized in bytes.
const EMPTY_GARRAY: garray_T = garray_T {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: 0,
    ga_growsize: 0,
    ga_data: null_mut::<c_void>(),
};

/// 'cpoptions' emptied for the duration of a pattern, restored on drop.
///
/// A bare assignment would be wrong: the expression a `\=` replacement runs
/// may itself have set the option, and then the *current* value has to be
/// put back through the option machinery so its notifications fire.
struct QuietCpo {
    saved: *mut c_char,
}

impl QuietCpo {
    /// # Safety
    /// Must be dropped before anything else writes 'cpoptions'.
    unsafe fn enter() -> Self {
        let saved = p_cpo.get();
        p_cpo.set(empty_string_option.ptr() as *mut c_char);
        Self { saved }
    }
}

impl Drop for QuietCpo {
    fn drop(&mut self) {
        // SAFETY: `saved` is the pointer 'cpoptions' held on entry.
        unsafe {
            if p_cpo.get() == empty_string_option.ptr() as *mut c_char {
                // Nothing touched it: put the old pointer straight back.
                p_cpo.set(self.saved);
                return;
            }
            // Something replaced it. If what it left is *another* empty
            // string, the old value has to go back through the option
            // machinery rather than by assignment.
            if *p_cpo.get() == NUL as c_char {
                set_option_value_give_err(
                    kOptCpoptions,
                    OptVal {
                        type_0: kOptValTypeString,
                        data: OptValData {
                            string: cstr_as_string(self.saved),
                        },
                    },
                    0,
                );
            }
            free_string_option(self.saved);
        }
    }
}

/// Does `pat` match anywhere in `text`?
///
/// # Safety
/// Both arguments must be NUL-terminated strings.
pub unsafe fn pattern_match(pat: *const c_char, text: *const c_char, ic: bool) -> bool {
    unsafe {
        let _cpo = QuietCpo::enter();
        let mut regmatch = EMPTY_REGMATCH;
        regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
        if regmatch.regprog.is_null() {
            return false;
        }
        regmatch.rm_ic = ic;
        let matched = vim_regexec_nl(&raw mut regmatch, text, 0 as colnr_T);
        vim_regfree(regmatch.regprog);
        matched
    }
}

/// `substitute()`: replace `pat` in `str` with `sub`, or with the result of
/// `expr` for a `\=` replacement.
///
/// Answers a fresh allocation the caller owns, and its length through
/// `ret_len` when that is not null.
///
/// # Safety
/// `str` must have `len` readable bytes and be NUL-terminated; `pat`, `sub`
/// and `flags` must be NUL-terminated; `expr` may be null.
pub unsafe fn do_string_sub(
    str: *mut c_char,
    len: size_t,
    pat: *mut c_char,
    sub: *mut c_char,
    expr: *mut typval_T,
    flags: *const c_char,
    ret_len: *mut size_t,
) -> *mut c_char {
    unsafe {
        let _cpo = QuietCpo::enter();
        let mut ga = EMPTY_GARRAY;
        ga_init(&raw mut ga, 1, 200);
        let mut regmatch = EMPTY_REGMATCH;
        regmatch.rm_ic = p_ic.get() != 0;
        regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);

        if !regmatch.regprog.is_null() {
            let mut tail = str;
            let end = str.add(len);
            let do_all = *flags == b'g' as c_char;
            // The start of the last zero-width match, so that the next one
            // at the same place is stepped over rather than repeated.
            let mut zero_width: *mut c_char = null_mut();

            while vim_regexec_nl(&raw mut regmatch, str, tail.offset_from(str) as colnr_T) {
                if regmatch.startp[0] == regmatch.endp[0] {
                    if zero_width == regmatch.startp[0] {
                        // Copy one whole character across and try again.
                        let i = utfc_ptr2len(tail);
                        copy_nonoverlapping(
                            tail,
                            (ga.ga_data as *mut c_char).offset(ga.ga_len as isize),
                            i as usize,
                        );
                        ga.ga_len += i;
                        tail = tail.offset(i as isize);
                        continue;
                    }
                    zero_width = regmatch.startp[0];
                }

                // First pass measures the replacement, second writes it.
                let sublen =
                    vim_regsub(&raw mut regmatch, sub, expr, tail, 0, REGSUB_MAGIC as c_int);
                if sublen <= 0 {
                    ga_clear(&raw mut ga);
                    break;
                }
                let matched = regmatch.endp[0].offset_from(regmatch.startp[0]);
                ga_grow(
                    &raw mut ga,
                    (end.offset_from(tail) + sublen as isize - matched) as c_int,
                );
                let before = regmatch.startp[0].offset_from(tail) as c_int;
                copy_nonoverlapping(
                    tail,
                    (ga.ga_data as *mut c_char).offset(ga.ga_len as isize),
                    before as usize,
                );
                vim_regsub(
                    &raw mut regmatch,
                    sub,
                    expr,
                    (ga.ga_data as *mut c_char)
                        .offset(ga.ga_len as isize)
                        .offset(before as isize),
                    sublen,
                    REGSUB_COPY as c_int | REGSUB_MAGIC as c_int,
                );
                // `sublen` counts the terminator the second pass wrote.
                ga.ga_len += before + sublen - 1;
                tail = regmatch.endp[0];
                if *tail == NUL as c_char || !do_all {
                    break;
                }
            }

            if !ga.ga_data.is_null() {
                strcpy((ga.ga_data as *mut c_char).offset(ga.ga_len as isize), tail);
                ga.ga_len += end.offset_from(tail) as c_int;
            }
            vim_regfree(regmatch.regprog);
        }

        // With no match at all the garray is never allocated and the input
        // is copied through unchanged.
        let (source, length) = if ga.ga_data.is_null() {
            (str, len)
        } else {
            (ga.ga_data as *mut c_char, ga.ga_len as size_t)
        };
        let ret = xstrnsave(source, length);
        ga_clear(&raw mut ga);
        if !ret_len.is_null() {
            *ret_len = length;
        }
        ret
    }
}
