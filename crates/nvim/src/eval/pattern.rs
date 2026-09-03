//! Matching and substituting with a regexp built from an expression.
//!
//! Both entry points compile their pattern with 'cpoptions' emptied, so
//! that a user's `cpo` flags cannot change what an expression's pattern
//! means. Restoring it is not a plain assignment — see `do_string_sub`.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr::{copy_nonoverlapping, null_mut};

use crate::api::private::helpers::cstr_as_string;
use crate::eval::{REGSUB_COPY, REGSUB_MAGIC};
use crate::main::{p_cpo, p_ic};
use crate::mbyte::utfc_ptr2len;
use crate::option::set_option_value_give_err;
use crate::options::kOptCpoptions;
use crate::optionstr::{empty_option, free_string_option, is_empty_option};
use crate::regexp::{RE_MAGIC, RE_STRING, vim_regcomp, vim_regexec_nl, vim_regfree, vim_regsub};
use crate::strings::xstrnsave;
use crate::types::{NUL, OptVal, OptionSetFlags, colnr_T, regmatch_T, regprog_T, size_t, typval_T};
use core::slice;

/// A `regmatch_T` with nothing in it.
const EMPTY_REGMATCH: regmatch_T = regmatch_T {
    regprog: null_mut::<regprog_T>(),
    startp: [null_mut::<c_char>(); 10],
    endp: [null_mut::<c_char>(); 10],
    rm_matchcol: 0,
    rm_ic: false,
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
                OptVal::String(unsafe { cstr_as_string(self.saved) }),
                OptionSetFlags::NONE,
            );
        }
        unsafe { free_string_option(self.saved) };
    }
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
    let mut out = Vec::<u8>::new();
    // Whether anything was substituted. The garray answered this by having
    // been allocated at all; a `Vec` cannot, and an empty result is a real
    // answer (`substitute("x", "x", "", "")`).
    let mut substituted = false;
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
                    // SAFETY: `tail` is inside the subject string, so its
                    // first character's bytes are readable.
                    let i = unsafe { utfc_ptr2len(tail) };
                    let run = unsafe { slice::from_raw_parts(tail.cast(), i as usize) };
                    out.extend_from_slice(run);
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
                out.clear();
                substituted = false;
                break;
            }
            // SAFETY: both ends of the match are inside the subject
            // string, as are `tail` and `end`.
            let matched = unsafe { regmatch.endp[0].offset_from(regmatch.startp[0]) };
            // SAFETY: as above.
            let grow = unsafe { end.offset_from(tail) + sublen as isize - matched } as c_int;
            out.reserve(grow as usize);
            // SAFETY: the match starts at or after `tail`.
            let before = unsafe { regmatch.startp[0].offset_from(tail) } as usize;
            let copy = REGSUB_COPY as c_int | REGSUB_MAGIC as c_int;
            let at = out.len();
            // SAFETY: `grow` is at least `before + sublen` (the match ends
            // no later than `end`), so both writes land in the capacity
            // reserved just above, and `set_len` covers only what was
            // written -- `sublen` counts the terminator the second pass
            // wrote, which the length excludes again.
            unsafe {
                let dest = out.as_mut_ptr().add(at).cast::<c_char>();
                copy_nonoverlapping(tail, dest, before);
                vim_regsub(&raw mut regmatch, sub, expr, dest.add(before), sublen, copy);
                out.set_len(at + before + sublen as usize - 1);
            }
            substituted = true;
            tail = regmatch.endp[0];
            // SAFETY: `tail` is inside the NUL-terminated subject.
            if unsafe { *tail } == NUL as c_char || !do_all {
                break;
            }
        }

        if substituted {
            // SAFETY: `tail` and `end` are both inside the subject string.
            let rest = unsafe { end.offset_from(tail) } as usize;
            out.extend_from_slice(unsafe { slice::from_raw_parts(tail.cast::<u8>(), rest) });
        }
        // SAFETY: nothing else owns the compiled program.
        unsafe { vim_regfree(regmatch.regprog) };
    }

    // With no match at all the input is copied through unchanged.
    let (source, length) = if substituted {
        (out.as_mut_ptr().cast::<c_char>(), out.len())
    } else {
        (str, len)
    };
    // SAFETY: `source` has `length` readable bytes -- either the caller's
    // subject or the buffer built above.
    let ret = unsafe { xstrnsave(source, length) };
    if !ret_len.is_null() {
        // SAFETY: the caller's promise -- a non-null `ret_len` is valid.
        unsafe { *ret_len = length };
    }
    ret
}
