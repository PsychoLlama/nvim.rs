//! Matching and substituting with a regexp built from an expression.
//!
//! Both entry points compile their pattern with 'cpoptions' emptied, so
//! that a user's `cpo` flags cannot change what an expression's pattern
//! means. Restoring it is not a plain assignment — see `do_string_sub`.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr::{copy_nonoverlapping, null_mut};

use crate::api::private::helpers::cstr_as_string;
use crate::eval::{REGSUB_COPY, REGSUB_MAGIC, kOptValTypeString};
use crate::garray::{ga_clear, ga_grow, ga_init};
use crate::main::{p_cpo, p_ic};
use crate::mbyte::utfc_ptr2len;
use crate::option::set_option_value_give_err;
use crate::options::kOptCpoptions;
use crate::optionstr::{empty_option, free_string_option, is_empty_option};
use crate::regexp::{RE_MAGIC, RE_STRING, vim_regcomp, vim_regexec_nl, vim_regfree, vim_regsub};
use crate::strings::xstrnsave;
use crate::types::{
    NUL, OptVal, OptValData, OptionSetFlags, colnr_T, garray_T, regmatch_T, regprog_T, size_t,
    typval_T,
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
        p_cpo.set(empty_option());
        Self { saved }
    }
}

impl Drop for QuietCpo {
    fn drop(&mut self) {
        // SAFETY: `saved` is the pointer 'cpoptions' held on entry.
        if is_empty_option(p_cpo.get()) {
            // Nothing touched it: put the old pointer straight back.
            p_cpo.set(self.saved);
            return;
        }
        // Something replaced it. If what it left is *another* empty
        // string, the old value has to go back through the option
        // machinery rather than by assignment.
        if unsafe { *p_cpo.get() } == NUL as c_char {
            set_option_value_give_err(
                kOptCpoptions,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: unsafe { cstr_as_string(self.saved) },
                    },
                },
                OptionSetFlags::NONE,
            );
        }
        unsafe { free_string_option(self.saved) };
    }
}

/// The address `at` bytes into a garray's buffer.
///
/// Computing it reads nothing, so it is ordinary code; what makes the byte
/// there *writable* is the `ga_grow` each caller does first.
fn ga_at(ga: &garray_T, at: isize) -> *mut c_char {
    (ga.ga_data as *mut c_char).wrapping_offset(at)
}

/// Does `pat` match anywhere in `text`?
///
/// # Safety
/// Both arguments must be NUL-terminated strings.
pub unsafe fn pattern_match(pat: *const c_char, text: *const c_char, ic: bool) -> bool {
    // SAFETY: the guard is dropped at the end of this body, before
    // anything else writes 'cpoptions'.
    let _cpo = unsafe { QuietCpo::enter() };
    let mut regmatch = EMPTY_REGMATCH;
    // SAFETY: the caller's promise -- `pat` is NUL-terminated.
    regmatch.regprog = unsafe { vim_regcomp(pat, RE_MAGIC + RE_STRING) };
    if regmatch.regprog.is_null() {
        return false;
    }
    regmatch.rm_ic = ic;
    // SAFETY: `regmatch` is this frame's and holds a compiled program;
    // `text` is the caller's NUL-terminated subject.
    let matched = unsafe { vim_regexec_nl(&raw mut regmatch, text, 0 as colnr_T) };
    // SAFETY: nothing else owns the program.
    unsafe { vim_regfree(regmatch.regprog) };
    matched
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
    // SAFETY: the guard is dropped at the end of this body, before
    // anything else writes 'cpoptions'.
    let _cpo = unsafe { QuietCpo::enter() };
    let mut ga = EMPTY_GARRAY;
    // SAFETY: `ga` is this frame's.
    unsafe { ga_init(&raw mut ga, 1, 200) };
    let mut regmatch = EMPTY_REGMATCH;
    regmatch.rm_ic = p_ic.get() != 0;
    // SAFETY: the caller's promise -- `pat` is NUL-terminated.
    regmatch.regprog = unsafe { vim_regcomp(pat, RE_MAGIC + RE_STRING) };

    if !regmatch.regprog.is_null() {
        let mut tail = str;
        // SAFETY: the caller's promise -- `str` has `len` readable bytes,
        // so one past the last is a valid end pointer.
        let end = unsafe { str.add(len) };
        // SAFETY: the caller's promise -- `flags` is NUL-terminated.
        let do_all = unsafe { *flags } == b'g' as c_char;
        // The start of the last zero-width match, so that the next one
        // at the same place is stepped over rather than repeated.
        let mut zero_width: *mut c_char = null_mut();

        // SAFETY: `regmatch` holds a compiled program, `str` is the
        // caller's subject and `tail` is inside it.
        while unsafe { vim_regexec_nl(&raw mut regmatch, str, tail.offset_from(str) as colnr_T) } {
            if regmatch.startp[0] == regmatch.endp[0] {
                if zero_width == regmatch.startp[0] {
                    // Copy one whole character across and try again.
                    // SAFETY: `tail` is inside the subject string, and
                    // the growth below the loop left room for one more
                    // character past `ga_len`.
                    let i = unsafe { utfc_ptr2len(tail) };
                    let dest = ga_at(&ga, ga.ga_len as isize);
                    // SAFETY: as above -- `i` bytes fit at `dest`.
                    unsafe { copy_nonoverlapping(tail, dest, i as usize) };
                    ga.ga_len += i;
                    // SAFETY: the character just copied is inside the
                    // subject, so its end is too.
                    tail = unsafe { tail.offset(i as isize) };
                    continue;
                }
                zero_width = regmatch.startp[0];
            }

            // First pass measures the replacement, second writes it.
            let magic = REGSUB_MAGIC as c_int;
            // SAFETY: `regmatch` holds this iteration's match, `sub` and
            // `expr` are the caller's, and a length of 0 means "measure".
            let sublen = unsafe { vim_regsub(&raw mut regmatch, sub, expr, tail, 0, magic) };
            if sublen <= 0 {
                // SAFETY: `ga` is this frame's.
                unsafe { ga_clear(&raw mut ga) };
                break;
            }
            // SAFETY: both ends of the match are inside the subject
            // string, as are `tail` and `end`.
            let matched = unsafe { regmatch.endp[0].offset_from(regmatch.startp[0]) };
            // SAFETY: as above.
            let grow = unsafe { end.offset_from(tail) + sublen as isize - matched } as c_int;
            // SAFETY: `ga` is this frame's.
            unsafe { ga_grow(&raw mut ga, grow) };
            // SAFETY: the match starts at or after `tail`.
            let before = unsafe { regmatch.startp[0].offset_from(tail) } as c_int;
            let dest = ga_at(&ga, ga.ga_len as isize);
            // SAFETY: the growth above covers the text before the match.
            unsafe { copy_nonoverlapping(tail, dest, before as usize) };
            let dest = ga_at(&ga, ga.ga_len as isize + before as isize);
            let copy = REGSUB_COPY as c_int | REGSUB_MAGIC as c_int;
            // SAFETY: the growth above covers the `sublen` bytes the second
            // pass writes at `dest`.
            unsafe { vim_regsub(&raw mut regmatch, sub, expr, dest, sublen, copy) };
            // `sublen` counts the terminator the second pass wrote.
            ga.ga_len += before + sublen - 1;
            tail = regmatch.endp[0];
            // SAFETY: `tail` is inside the NUL-terminated subject.
            if unsafe { *tail } == NUL as c_char || !do_all {
                break;
            }
        }

        if !ga.ga_data.is_null() {
            let dest = ga_at(&ga, ga.ga_len as isize);
            // SAFETY: the last growth covered the rest of the subject.
            unsafe { strcpy(dest, tail) };
            // SAFETY: `tail` and `end` are both inside the subject string.
            ga.ga_len += unsafe { end.offset_from(tail) } as c_int;
        }
        // SAFETY: nothing else owns the compiled program.
        unsafe { vim_regfree(regmatch.regprog) };
    }

    // With no match at all the garray is never allocated and the input
    // is copied through unchanged.
    let (source, length) = if ga.ga_data.is_null() {
        (str, len)
    } else {
        (ga.ga_data as *mut c_char, ga.ga_len as size_t)
    };
    // SAFETY: `source` has `length` readable bytes -- either the caller's
    // subject or the array built above.
    let ret = unsafe { xstrnsave(source, length) };
    // SAFETY: `ga` is this frame's.
    unsafe { ga_clear(&raw mut ga) };
    if !ret_len.is_null() {
        // SAFETY: the caller's promise -- a non-null `ret_len` is valid.
        unsafe { *ret_len = length };
    }
    ret
}
