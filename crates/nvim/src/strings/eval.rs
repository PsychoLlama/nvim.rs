//! The remaining Vimscript string builtins.
//!
//! Measurement (`strlen`, `strchars`, `strcharlen`, `strwidth`,
//! `strdisplaywidth`), search (`stridx`, `strridx`), conversion (`str2nr`,
//! `str2list`, `string`, `strtrans`) and transformation (`tolower`, `toupper`,
//! `tr`, `trim`).  `strchar_common` is the character count `strchars()` and
//! `strcharlen()` share, differing only in whether composing characters count
//! separately.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::semsg;
use core::ffi::{c_char, c_int};
use core::ptr;

use super::{given, strcase_save, strict_bool_arg, xstrnsave};
use crate::charset::{Str2NrBases, skipwhite, transstr, vim_str2nr};
use crate::eval::encode::encode_tv2string;
use crate::eval::typval::{
    NumBuf, tv_check_for_opt_string_arg, tv_get_bool, tv_get_number, tv_get_number_chk,
    tv_get_string_buf_chk, tv_list_alloc_ret, tv_list_append_number,
};
use crate::mbyte::{
    char_at, char_count, char_len, cluster_len, clusters, mb_cptr2char_adv, mb_ptr2char_adv,
    mb_string2cells, utf_head_off,
};
use crate::memory::handoff::owned_cstr;
use crate::message::e_invarg;
use crate::message::emsg;
use crate::message_fmt::c_str;
use crate::os::cshim::{gettext, strstr};
use crate::plines::linetabsize_col;
use crate::types::{EvalFuncData, TypVal, VAR_STRING, VarNumber, kListLenUnknown, ptrdiff_t};
use core::ffi::CStr;

/// The scratch buffer `tv_get_string_buf_chk` renders a Number into.
/// `NUMBUFLEN` in the C.
const NUMBUFLEN: usize = 65;

/// "str2list()" function: the string as a list of code points.
///
/// # Safety
///
/// `args` must point at an initialized typval, unaliased for the call.
/// `result` must point at the caller's return slot: an initialized typval it
/// owns and will clear. `_fptr` must be an initialized `EvalFuncData` whose
/// pointer fields point at live data for the call.
pub unsafe fn f_str2list(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe { tv_list_alloc_ret(result, kListLenUnknown as ptrdiff_t) };
    // SAFETY: the argument was converted to a NUL-terminated string.
    let bytes = unsafe { cstr::bytes_at(numbuf.string(args)) };
    let mut at = 0;
    while at < bytes.len() {
        let rest = &bytes[at..];
        unsafe { tv_list_append_number((*result).list_or_null(), VarNumber::from(char_at(rest))) };
        at += char_len(rest);
    }
}

/// "str2nr()" function.
///
/// The sign is handled here rather than by `vim_str2nr`, so that a base
/// prefix may follow it and so that whitespace between the two is allowed.
/// Text after the number is silently ignored.
///
/// # Safety
///
/// `args` must point at an initialized typval, unaliased for the call.
/// `result` must point at the caller's return slot: an initialized typval it
/// owns and will clear. `_fptr` must be an initialized `EvalFuncData` whose
/// pointer fields point at live data for the call.
pub unsafe fn f_str2nr(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut base = 10;
    let mut what = Str2NrBases::NONE;
    if given(unsafe { &*args.add(1) }) {
        base = unsafe { tv_get_number(args.add(1)) as c_int };
        if !matches!(base, 2 | 8 | 10 | 16) {
            emsg(gettext(e_invarg));
            return;
        }
        if given(unsafe { &*args.add(2) }) && unsafe { tv_get_bool(args.add(2)) } != 0 {
            what |= Str2NrBases::QUOTE;
        }
    }

    let mut p = unsafe { skipwhite(numbuf.string(args)) };
    let isneg = unsafe { *p } == b'-' as c_char;
    if unsafe { *p } == b'+' as c_char || unsafe { *p } == b'-' as c_char {
        p = unsafe { skipwhite(p.add(1)) };
    }

    // An explicit base forces that radix; base 10 accepts none.
    what |= match base {
        2 => Str2NrBases::BIN | Str2NrBases::FORCE,
        8 => Str2NrBases::OCT_ANY | Str2NrBases::FORCE,
        16 => Str2NrBases::HEX | Str2NrBases::FORCE,
        _ => Str2NrBases::NONE,
    };

    let mut n: VarNumber = 0;
    // Only the number and the base matter here: every other output --
    // the prefix length, the digit count, the unsigned value and the
    // overflow flag -- is one `vim_str2nr` may skip.
    let out = &raw mut n;
    let pre = ptr::null_mut();
    let len = ptr::null_mut();
    let uns = ptr::null_mut();
    let ovf = ptr::null_mut();
    unsafe { vim_str2nr(p, pre, len, what, out, uns, 0, false, ovf) };
    unsafe { (*result).write_number(if isneg { -n } else { n }) };
}

/// "stridx()" function: the byte index of the first occurrence.
///
/// # Safety
///
/// `args` must point at an initialized typval, unaliased for the call.
/// `result` must point at the caller's return slot: an initialized typval it
/// owns and will clear. `_fptr` must be an initialized `EvalFuncData` whose
/// pointer fields point at live data for the call.
pub unsafe fn f_stridx(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe { (*result).write_number(-1) };

    let mut buf = [0 as c_char; NUMBUFLEN];
    let needle = unsafe { numbuf.string_chk(args.add(1)) };
    let haystack_start = unsafe { tv_get_string_buf_chk(args, buf.as_mut_ptr()) };
    let mut haystack = haystack_start;
    if needle.is_null() || haystack.is_null() {
        return;
    }

    if given(unsafe { &*args.add(2) }) {
        let mut error = false;
        let start_idx = unsafe { tv_get_number_chk(args.add(2), &raw mut error) as ptrdiff_t };
        if error || start_idx >= unsafe { cstr::bytes_at(haystack).len() as ptrdiff_t } {
            return;
        }
        // A negative start is ignored, not counted from the end.
        if start_idx >= 0 {
            haystack = unsafe { haystack.offset(start_idx as isize) };
        }
    }

    let pos = unsafe { strstr(haystack, needle) };
    if !pos.is_null() {
        // Reported against the whole string, not against the start.
        unsafe { (*result).write_number(pos.offset_from(haystack_start) as VarNumber) };
    }
}

/// "strridx()" function: the byte index of the last occurrence at or
/// before `end_idx`.
///
/// # Safety
///
/// `args` must point at an initialized typval, unaliased for the call.
/// `result` must point at the caller's return slot: an initialized typval it
/// owns and will clear. `_fptr` must be an initialized `EvalFuncData` whose
/// pointer fields point at live data for the call.
pub unsafe fn f_strridx(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe { (*result).write_number(-1) };

    let mut buf = [0 as c_char; NUMBUFLEN];
    let needle = unsafe { numbuf.string_chk(args.add(1)) };
    let haystack = unsafe { tv_get_string_buf_chk(args, buf.as_mut_ptr()) };
    if needle.is_null() || haystack.is_null() {
        return;
    }

    let end_idx = if given(unsafe { &*args.add(2) }) {
        let idx = unsafe { tv_get_number_chk(args.add(2), ptr::null_mut()) as ptrdiff_t };
        if idx < 0 {
            return;
        }
        idx
    } else {
        unsafe { cstr::bytes_at(haystack).len() as ptrdiff_t }
    };
    let last_allowed = unsafe { haystack.offset(end_idx as isize) };

    let lastmatch = if unsafe { *needle } == 0 {
        // The empty needle matches at the end of the range.
        last_allowed
    } else {
        let mut found = ptr::null();
        let mut rest = haystack;
        while unsafe { *rest } != 0 {
            rest = unsafe { strstr(rest, needle) };
            if rest.is_null() || rest > last_allowed {
                break;
            }
            found = rest;
            rest = unsafe { rest.add(1) };
        }
        found
    };

    if !lastmatch.is_null() {
        unsafe { (*result).write_number(lastmatch.offset_from(haystack) as VarNumber) };
    }
}

/// "string()" function.
///
/// # Safety
///
/// `args` must point at an initialized typval, unaliased for the call.
/// `result` must point at the caller's return slot: an initialized typval it
/// owns and will clear. `_fptr` must be an initialized `EvalFuncData` whose
/// pointer fields point at live data for the call.
pub unsafe fn f_string(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    unsafe { (*result).write_string(encode_tv2string(args, ptr::null_mut())) };
}

/// "strlen()" function: the length in bytes.
///
/// # Safety
///
/// `args` must point at an initialized typval, unaliased for the call.
/// `result` must point at the caller's return slot: an initialized typval it
/// owns and will clear. `_fptr` must be an initialized `EvalFuncData` whose
/// pointer fields point at live data for the call.
pub unsafe fn f_strlen(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe { (*result).write_number(cstr::bytes_at(numbuf.string(args)).len() as VarNumber) };
}

/// The character count `strchars()` and `strcharlen()` share.
///
/// `skipcc` folds a composing character into the base character it
/// follows; without it each one counts on its own.
///
/// # Safety
///
/// `args` must point at an initialized typval, unaliased for the call.
/// `result` must point at the caller's return slot: an initialized typval it
/// owns and will clear.
unsafe fn strchar_common(args: *mut TypVal, result: *mut TypVal, skipcc: bool) {
    let mut numbuf = NumBuf::new();
    let next_char: unsafe fn(*mut *const c_char) -> c_int = if skipcc {
        mb_ptr2char_adv
    } else {
        mb_cptr2char_adv
    };
    let mut s = unsafe { numbuf.string(args) };
    let mut len: VarNumber = 0;
    while unsafe { *s } != 0 {
        unsafe { next_char(&raw mut s) };
        len += 1;
    }
    unsafe { (*result).write_number(len) };
}

/// "strcharlen()" function: characters, composing characters folded in.
///
/// # Safety
///
/// `args` must point at an initialized typval, unaliased for the call.
/// `result` must point at the caller's return slot: an initialized typval it
/// owns and will clear. `_fptr` must be an initialized `EvalFuncData` whose
/// pointer fields point at live data for the call.
pub unsafe fn f_strcharlen(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    unsafe { strchar_common(args, result, true) }
}

/// "strchars()" function: characters, composing ones counted unless the
/// optional `skipcc` argument says otherwise.
///
/// # Safety
///
/// `args` must point at an initialized typval, unaliased for the call.
/// `result` must point at the caller's return slot: an initialized typval it
/// owns and will clear. `_fptr` must be an initialized `EvalFuncData` whose
/// pointer fields point at live data for the call.
pub unsafe fn f_strchars(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let skipcc = if given(unsafe { &*args.add(1) }) {
        match unsafe { strict_bool_arg(args.add(1)) } {
            Some(flag) => flag,
            None => return,
        }
    } else {
        false
    };
    unsafe { strchar_common(args, result, skipcc) };
}

/// "strdisplaywidth()" function: screen cells, tabs expanded against the
/// optional starting column.
///
/// # Safety
///
/// `args` must point at an initialized typval, unaliased for the call.
/// `result` must point at the caller's return slot: an initialized typval it
/// owns and will clear. `_fptr` must be an initialized `EvalFuncData` whose
/// pointer fields point at live data for the call.
pub unsafe fn f_strdisplaywidth(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let s = unsafe { numbuf.string(args) };
    let col = if given(unsafe { &*args.add(1) }) {
        unsafe { tv_get_number(args.add(1)) as c_int }
    } else {
        0
    };
    unsafe { (*result).write_number((linetabsize_col(col, s as *mut c_char) - col) as VarNumber) };
}

/// "strwidth()" function: screen cells, with a tab counting as one.
///
/// # Safety
///
/// `args` must point at an initialized typval, unaliased for the call.
/// `result` must point at the caller's return slot: an initialized typval it
/// owns and will clear. `_fptr` must be an initialized `EvalFuncData` whose
/// pointer fields point at live data for the call.
pub unsafe fn f_strwidth(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe { (*result).write_number(mb_string2cells(numbuf.string(args)) as VarNumber) };
}

/// "strtrans()" function: unprintable characters as `^X`/`<xx>`.
///
/// # Safety
///
/// `args` must point at an initialized typval, unaliased for the call.
/// `result` must point at the caller's return slot: an initialized typval it
/// owns and will clear. `_fptr` must be an initialized `EvalFuncData` whose
/// pointer fields point at live data for the call.
pub unsafe fn f_strtrans(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe { (*result).write_string(transstr(numbuf.string(args), true)) };
}

/// "tolower()" function.
///
/// # Safety
///
/// `args` must point at an initialized typval, unaliased for the call.
/// `result` must point at the caller's return slot: an initialized typval it
/// owns and will clear. `_fptr` must be an initialized `EvalFuncData` whose
/// pointer fields point at live data for the call.
pub unsafe fn f_tolower(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe { (*result).write_string(strcase_save(numbuf.string(args), false)) };
}

/// "toupper()" function.
///
/// # Safety
///
/// `args` must point at an initialized typval, unaliased for the call.
/// `result` must point at the caller's return slot: an initialized typval it
/// owns and will clear. `_fptr` must be an initialized `EvalFuncData` whose
/// pointer fields point at live data for the call.
pub unsafe fn f_toupper(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe { (*result).write_string(strcase_save(numbuf.string(args), true)) };
}

/// "tr()" function: character-wise translation.
///
/// `fromstr` and `tostr` must hold the same number of characters, which is
/// checked lazily -- either when a mapped character's counterpart runs off
/// the end of `tostr`, or once, the first time an input character is *not*
/// in `fromstr` and the counts can be compared directly. So
/// `tr('a', 'ab', 'x')` is an error but `tr('a', 'a', 'x')` is not.
///
/// # Safety
///
/// `args` must point at an initialized typval, unaliased for the call.
/// `result` must point at the caller's return slot: an initialized typval it
/// owns and will clear. `_fptr` must be an initialized `EvalFuncData` whose
/// pointer fields point at live data for the call.
pub unsafe fn f_tr(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut buf = [0 as c_char; NUMBUFLEN];
    let mut buf2 = [0 as c_char; NUMBUFLEN];
    let in_str = unsafe { numbuf.string(args) };
    let fromstr = unsafe { tv_get_string_buf_chk(args.add(1), buf.as_mut_ptr()) };
    let tostr = unsafe { tv_get_string_buf_chk(args.add(2), buf2.as_mut_ptr()) };

    unsafe { (*result).write_string(ptr::null_mut()) };
    if fromstr.is_null() || tostr.is_null() {
        return; // Type error; the message is already out.
    }

    // SAFETY: all three were converted to NUL-terminated strings above.
    let (in_bytes, from_bytes, to_bytes) = unsafe {
        (
            cstr::bytes_at(in_str),
            cstr::bytes_at(fromstr),
            cstr::bytes_at(tostr),
        )
    };

    /// The `n`-th character of a set, as its bytes.
    fn nth_char(set: &[u8], n: usize) -> Option<&[u8]> {
        let mut at = 0;
        for _ in 0..n {
            at += cluster_len(set.get(at..).filter(|rest| !rest.is_empty())?);
        }
        let rest = set.get(at..).filter(|rest| !rest.is_empty())?;
        Some(&rest[..cluster_len(rest)])
    }

    let mut out = Vec::<u8>::new();

    let mut lengths_checked = false;
    let mut at = 0;
    'error: {
        while at < in_bytes.len() {
            let cur = &in_bytes[at..];
            let ch = &cur[..cluster_len(cur)];

            // Which character of `fromstr` is this, if any?
            let mut idx = 0;
            let mut found = false;
            let mut from_at = 0;
            while from_at < from_bytes.len() {
                let from_rest = &from_bytes[from_at..];
                let from_ch = &from_rest[..cluster_len(from_rest)];
                if from_ch == ch {
                    found = true;
                    break;
                }
                idx += 1;
                from_at += from_ch.len();
            }

            let replacement = if found {
                match nth_char(to_bytes, idx) {
                    Some(hit) => hit,
                    None => break 'error, // tostr is shorter than fromstr
                }
            } else {
                if !lengths_checked {
                    lengths_checked = true;
                    // `idx` is now `fromstr`'s character count.
                    if char_count(to_bytes) != idx {
                        break 'error;
                    }
                }
                ch
            };

            out.extend_from_slice(replacement);
            at += ch.len();
        }
        unsafe { (*result).write_string(owned_cstr(out)) };
        return;
    }
    // SAFETY: a message argument the caller holds as a NUL-terminated string.
    let fromstr = unsafe { c_str(fromstr) };
    semsg!("E475: Invalid argument: {fromstr}");
}

/// "trim()" function.
///
/// `dir` is 0 (both ends, the default), 1 (leading) or 2 (trailing). With
/// no mask the set trimmed is whitespace plus U+00A0; with one it is
/// exactly the mask's characters, and an empty mask reverts to the
/// default.
///
/// # Safety
///
/// `args` must point at an initialized typval, unaliased for the call.
/// `result` must point at the caller's return slot: an initialized typval it
/// owns and will clear. `_fptr` must be an initialized `EvalFuncData` whose
/// pointer fields point at live data for the call.
pub unsafe fn f_trim(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut buf1 = [0 as c_char; NUMBUFLEN];
    let mut buf2 = [0 as c_char; NUMBUFLEN];
    let head = unsafe { tv_get_string_buf_chk(args, buf1.as_mut_ptr()) };
    let mut mask = ptr::null::<c_char>();
    let mut dir = 0;

    unsafe { (*result).write_string(ptr::null_mut()) };
    if head.is_null() || unsafe { tv_check_for_opt_string_arg(args, 1) }.is_err() {
        return;
    }

    if unsafe { (*args.add(1)).v_type } == VAR_STRING {
        mask = unsafe { tv_get_string_buf_chk(args.add(1), buf2.as_mut_ptr()) };
        if unsafe { *mask } == 0 {
            mask = ptr::null();
        }
        if given(unsafe { &*args.add(2) }) {
            let mut error = false;
            dir = unsafe { tv_get_number_chk(args.add(2), &raw mut error) as c_int };
            if error {
                return;
            }
            if !(0..=2).contains(&dir) {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let arg0 = unsafe { c_str(numbuf.string(args.add(2))) };
                semsg!("E475: Invalid argument: {arg0}");
                return;
            }
        }
    }

    // SAFETY: both were converted to NUL-terminated strings above, and the
    // mask may be absent.
    let bytes = unsafe { cstr::bytes_at(head) };
    let mask = unsafe { cstr::at_opt(mask) }.map(CStr::to_bytes);

    // Whitespace and NBSP by default, else exactly the mask's set.
    let trimmable = |c: c_int| -> bool {
        let Some(mask) = mask else {
            return c <= c_int::from(b' ') || c == 0xa0;
        };
        clusters(mask).any(|(_, mask_char)| mask_char == c)
    };

    let mut start = 0;
    if dir == 0 || dir == 1 {
        while start < bytes.len() && trimmable(char_at(&bytes[start..])) {
            start += cluster_len(&bytes[start..]);
        }
    }

    let mut end = bytes.len();
    if dir == 0 || dir == 2 {
        while end > start {
            // Step back over one whole character.
            // SAFETY: `head` is the NUL-terminated string `bytes` borrows,
            // and `end - 1` is a byte of it.
            let back = unsafe { utf_head_off(head.cast_mut(), head.add(end - 1)) };
            let prev = end - 1 - usize::try_from(back).expect("never negative");
            if !trimmable(char_at(&bytes[prev..])) {
                break;
            }
            end = prev;
        }
    }

    // SAFETY: `start..end` is a span of the string at `head`.
    unsafe { (*result).write_string(xstrnsave(head.add(start), end - start)) };
}
