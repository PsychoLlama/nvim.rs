//! The `'spellsuggest'` option: what its items mean, and the outside
//! sources two of them name.
//!
//! [`spell_check_sps`] parses the option into [`sps_flags`](super::sps_flags)
//! and `sps_limit`, which the internal search then reads. The `expr:` and
//! `file:` items instead name a source of suggestions outside the editor's
//! own word trees, and [`spell_suggest_expr`] and [`spell_suggest_file`]
//! are those two.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{
    FAIL, MAXPATHL, MAXWLEN, NUL, OK, SCORE_FILE, SPS_BEST, SPS_DOUBLE, SPS_FAST, Sug, sps_flags,
    sps_limit,
};
use crate::charset::getdigits_int;
use crate::eval::typval::{NumBuf, tv_list_unref};
use crate::eval::vars::{eval_spell_expr, get_spellword};
use crate::fileio::vim_fgets;
use crate::main::{got_int, p_sps};
use crate::message_fmt::c_str;
use crate::option::copy_option_part;
use crate::os::fs::os_fopen;
use crate::os::input::line_breakcheck;
use crate::semsg;
use crate::spell::{captype, make_case_word};
use crate::spellsuggest::collect::{add_suggestion, check_suggestions, cleanup_suggestions};
use crate::strings::vim_strchr;
use crate::types::VAR_LIST;
use ::libc::{FILE, fclose, strcasecmp};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// Does a `timeout:` item name a number? The value may be negative, which
/// switches the timeout off.
fn is_timeout_value(value: &[u8]) -> bool {
    let digits = value.strip_prefix(b"-").unwrap_or(value);
    digits.first().is_some_and(u8::is_ascii_digit)
}

/// Check `'spellsuggest'` and set [`sps_flags`] and [`sps_limit`] from it.
///
/// Returns `FAIL` for a value the option should not take, having put both
/// back to their defaults.
///
/// # Safety
///
/// `'spellsuggest'` must hold a NUL-terminated string.
pub(crate) unsafe fn spell_check_sps() -> c_int {
    // SAFETY: the caller guarantees the option; `buf` is `MAXPATHL`, which
    // is what `copy_option_part` is told it may fill.
    let mut buf = [0 as c_char; MAXPATHL as usize];
    let bufp = buf.as_mut_ptr();

    sps_flags.set(0);
    sps_limit.set(9999);

    let mut p = p_sps.get();
    while unsafe { *p } as c_int != NUL {
        // SAFETY: `p` walks the option's NUL-terminated value and `buf` is
        // `MAXPATHL`, which is what `copy_option_part` is told it may fill.
        let sep = c",".as_ptr().cast_mut();
        unsafe { copy_option_part(&raw mut p, bufp, MAXPATHL as usize, sep) };
        let part = unsafe { CStr::from_ptr(bufp) }.to_bytes();

        // Zero means "this item said nothing about the method", -1
        // means "this item is not a valid one".
        let mut f = 0;
        if part.first().is_some_and(u8::is_ascii_digit) {
            let mut s = bufp;
            sps_limit.set(unsafe { getdigits_int(&raw mut s, true, 0) });
            if unsafe { *s } as c_int != NUL && !(unsafe { *s } as u8).is_ascii_digit() {
                f = -1;
            }
        // Keep the three names in sync with `opt_sps_values`.
        } else if part == b"best" {
            f = SPS_BEST;
        } else if part == b"fast" {
            f = SPS_FAST;
        } else if part == b"double" {
            f = SPS_DOUBLE;
        } else if !part.starts_with(b"expr:")
            && !part.starts_with(b"file:")
            && !part
                .strip_prefix(b"timeout:".as_slice())
                .is_some_and(is_timeout_value)
        {
            f = -1;
        }

        // Only one method may be named.
        if f == -1 || (sps_flags.get() != 0 && f != 0) {
            sps_flags.set(SPS_BEST);
            sps_limit.set(9999);
            return FAIL;
        }
        if f != 0 {
            sps_flags.set(f);
        }
    }

    if sps_flags.get() == 0 {
        sps_flags.set(SPS_BEST);
    }
    OK
}

/// Find suggestions by evaluating `expr`, the `expr:` item of
/// `'spellsuggest'`.
///
/// # Safety
///
/// `su` must be valid and `expr` NUL-terminated.
pub(super) unsafe fn spell_suggest_expr(mut su: Sug, expr: *mut c_char) {
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller guarantees the pointers; the list the expression
    // returns is owned here until it is unreferenced.
    // The work is split up so that `suginfo_T` need not be exported to
    // the evaluator.
    let list = unsafe { eval_spell_expr(su.su_badword() as *mut c_char, expr) };
    if !list.is_null() {
        let mut li = unsafe { (*list).lv_first };
        while !li.is_null() {
            if unsafe { (*li).li_tv.v_type } == VAR_LIST {
                // Each item is a [word, score] pair.
                let mut word: *const c_char = ptr::null();
                let score =
                    unsafe { get_spellword((*li).li_tv.vval.v_list, &raw mut word, &mut numbuf) };
                if score >= 0 && score <= su.su_maxscore {
                    let sug = su.raw();
                    let ga = su.su_ga();
                    let badlen = su.su_badlen;
                    let lang = su.su_sallang;
                    // SAFETY: `su` is live by the contract above, so `ga` is
                    // its own list of `suggest_T`; `word` is the
                    // NUL-terminated string the list item yielded.
                    unsafe { add_suggestion(sug, ga, word, badlen, score, 0, true, lang, false) };
                }
            }
            li = unsafe { (*li).li_next };
        }
        unsafe { tv_list_unref(list) };
    }

    unsafe { check_suggestions(su.raw(), su.su_ga()) };
    unsafe { cleanup_suggestions(su.su_ga(), su.su_maxscore, su.su_maxcount) };
}

/// Find suggestions in `fname`, the `file:` item of `'spellsuggest'`.
///
/// Every line of the file is `badword/goodword`.
///
/// # Safety
///
/// `su` must be valid and `fname` NUL-terminated.
pub(super) unsafe fn spell_suggest_file(mut su: Sug, fname: *mut c_char) {
    // SAFETY: the caller guarantees the pointers; `line` is what
    // `vim_fgets` is told its size is, and the good word is terminated
    // inside it before it is used.
    let fd: *mut FILE = unsafe { os_fopen(fname, c"r".as_ptr()) };
    if fd.is_null() {
        // SAFETY: the message macros expand to a `vim_snprintf` over the // format literal above and the editor's message buffers.
        let fname = unsafe { c_str(fname) };
        semsg!("E484: Can't open file {fname}");
        return;
    }

    let mut line = [0 as c_char; MAXWLEN * 2];
    let mut cword = [0 as c_char; MAXWLEN];
    let linep = line.as_mut_ptr();
    let cwordp = cword.as_mut_ptr();
    while !unsafe { vim_fgets(linep, (MAXWLEN * 2) as c_int, fd) } && !got_int.get() {
        line_breakcheck();

        let mut p = unsafe { vim_strchr(linep, '/' as c_int) };
        if p.is_null() {
            continue; // no separator, so not an entry
        }
        unsafe { *p = NUL as c_char };
        p = unsafe { p.add(1) };
        if unsafe { strcasecmp(su.su_badword() as *const c_char, linep) } != 0 {
            continue;
        }

        // A match: the good word runs to the CR or NL.
        let mut len = 0isize;
        while unsafe { *p.offset(len) } as u8 >= b' ' {
            len += 1;
        }
        unsafe { *p.offset(len) = NUL as c_char };

        // A suggestion with no case of its own takes the bad word's.
        if unsafe { captype(p, ptr::null()) } == 0 {
            unsafe { make_case_word(p, cwordp, su.su_badflags) };
            p = cwordp;
        }

        let sug = su.raw();
        let ga = su.su_ga();
        let badlen = su.su_badlen;
        let lang = su.su_sallang;
        // SAFETY: `su` is live by the contract above, so `ga` is its own
        // list of `suggest_T`; `p` is a NUL-terminated word in `line` or in
        // `cword`, both of which outlive the call.
        unsafe { add_suggestion(sug, ga, p, badlen, SCORE_FILE, 0, true, lang, false) };
    }
    unsafe { fclose(fd) };

    unsafe { check_suggestions(su.raw(), su.su_ga()) };
    unsafe { cleanup_suggestions(su.su_ga(), su.su_maxscore, su.su_maxcount) };
}
