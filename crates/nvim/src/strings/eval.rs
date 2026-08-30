//! The remaining Vimscript string builtins.
//!
//! Measurement (`strlen`, `strchars`, `strcharlen`, `strwidth`,
//! `strdisplaywidth`), search (`stridx`, `strridx`), conversion (`str2nr`,
//! `str2list`, `string`, `strtrans`) and transformation (`tolower`, `toupper`,
//! `tr`, `trim`).  `strchar_common` is the character count `strchars()` and
//! `strcharlen()` share, differing only in whether composing characters count
//! separately.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::{given, strcase_save, strict_bool_arg, xstrnsave};
use crate::charset::{
    STR2NR_BIN, STR2NR_FORCE, STR2NR_HEX, STR2NR_OCT, STR2NR_OOCT, STR2NR_QUOTE, skipwhite,
    transstr, vim_str2nr,
};
use crate::eval::encode::encode_tv2string;
use crate::eval::typval::{
    NumBuf, tv_check_for_opt_string_arg, tv_get_bool, tv_get_number, tv_get_number_chk,
    tv_get_string_buf_chk, tv_list_alloc_ret, tv_list_append_number,
};
use crate::garray::{ga_append, ga_clear, ga_grow, ga_init};
use crate::main::e_invarg;
use crate::mbyte::{
    mb_cptr2char_adv, mb_ptr2char_adv, mb_string2cells, utf_head_off, utf_ptr2char, utf_ptr2len,
    utfc_ptr2len,
};
use crate::message::emsg;
use crate::message_fmt::c_str;
use crate::os::cshim::{gettext, strncmp, strstr};
use crate::plines::linetabsize_col;
use crate::types::{
    EvalFuncData, VAR_STRING, garray_T, kListLenUnknown, ptrdiff_t, size_t, typval_T, uint8_t,
    varnumber_T,
};
use ::libc::strlen;

/// The scratch buffer `tv_get_string_buf_chk` renders a Number into.
/// `NUMBUFLEN` in the C.
const NUMBUFLEN: usize = 65;

/// "str2list()" function: the string as a list of code points.
pub unsafe fn f_str2list(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe { tv_list_alloc_ret(rettv, kListLenUnknown as ptrdiff_t) };
    let mut p = unsafe { numbuf.string(argvars) };
    while unsafe { *p } != 0 {
        unsafe { tv_list_append_number((*rettv).vval.v_list, utf_ptr2char(p) as varnumber_T) };
        p = unsafe { p.offset(utf_ptr2len(p) as isize) };
    }
}

/// "str2nr()" function.
///
/// The sign is handled here rather than by `vim_str2nr`, so that a base
/// prefix may follow it and so that whitespace between the two is allowed.
/// Text after the number is silently ignored.
pub unsafe fn f_str2nr(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut base = 10;
    let mut what = 0;
    if given(unsafe { &*argvars.add(1) }) {
        base = unsafe { tv_get_number(argvars.add(1)) as c_int };
        if !matches!(base, 2 | 8 | 10 | 16) {
            emsg(gettext(e_invarg));
            return;
        }
        if given(unsafe { &*argvars.add(2) }) && unsafe { tv_get_bool(argvars.add(2)) } != 0 {
            what |= STR2NR_QUOTE;
        }
    }

    let mut p = unsafe { skipwhite(numbuf.string(argvars)) };
    let isneg = unsafe { *p } == b'-' as c_char;
    if unsafe { *p } == b'+' as c_char || unsafe { *p } == b'-' as c_char {
        p = unsafe { skipwhite(p.add(1)) };
    }

    // An explicit base forces that radix; base 10 accepts none.
    what |= match base {
        2 => STR2NR_BIN | STR2NR_FORCE,
        8 => STR2NR_OCT | STR2NR_OOCT | STR2NR_FORCE,
        16 => STR2NR_HEX | STR2NR_FORCE,
        _ => 0,
    };

    let mut n: varnumber_T = 0;
    // Only the number and the base matter here: every other output --
    // the prefix length, the digit count, the unsigned value and the
    // overflow flag -- is one `vim_str2nr` may skip.
    let out = &raw mut n;
    let pre = ptr::null_mut();
    let len = ptr::null_mut();
    let uns = ptr::null_mut();
    let ovf = ptr::null_mut();
    unsafe { vim_str2nr(p, pre, len, what, out, uns, 0, false, ovf) };
    unsafe { (*rettv).vval.v_number = if isneg { -n } else { n } };
}

/// "stridx()" function: the byte index of the first occurrence.
pub unsafe fn f_stridx(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe { (*rettv).vval.v_number = -1 };

    let mut buf = [0 as c_char; NUMBUFLEN];
    let needle = unsafe { numbuf.string_chk(argvars.add(1)) };
    let haystack_start = unsafe { tv_get_string_buf_chk(argvars, buf.as_mut_ptr()) };
    let mut haystack = haystack_start;
    if needle.is_null() || haystack.is_null() {
        return;
    }

    if given(unsafe { &*argvars.add(2) }) {
        let mut error = false;
        let start_idx = unsafe { tv_get_number_chk(argvars.add(2), &raw mut error) as ptrdiff_t };
        if error || start_idx >= unsafe { strlen(haystack) as ptrdiff_t } {
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
        unsafe { (*rettv).vval.v_number = pos.offset_from(haystack_start) as varnumber_T };
    }
}

/// "strridx()" function: the byte index of the last occurrence at or
/// before `end_idx`.
pub unsafe fn f_strridx(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe { (*rettv).vval.v_number = -1 };

    let mut buf = [0 as c_char; NUMBUFLEN];
    let needle = unsafe { numbuf.string_chk(argvars.add(1)) };
    let haystack = unsafe { tv_get_string_buf_chk(argvars, buf.as_mut_ptr()) };
    if needle.is_null() || haystack.is_null() {
        return;
    }

    let end_idx = if given(unsafe { &*argvars.add(2) }) {
        let idx = unsafe { tv_get_number_chk(argvars.add(2), ptr::null_mut()) as ptrdiff_t };
        if idx < 0 {
            return;
        }
        idx
    } else {
        unsafe { strlen(haystack) as ptrdiff_t }
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
        unsafe { (*rettv).vval.v_number = lastmatch.offset_from(haystack) as varnumber_T };
    }
}

/// "string()" function.
pub unsafe fn f_string(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe { (*rettv).v_type = VAR_STRING };
    unsafe { (*rettv).vval.v_string = encode_tv2string(argvars, ptr::null_mut()) };
}

/// "strlen()" function: the length in bytes.
pub unsafe fn f_strlen(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe { (*rettv).vval.v_number = strlen(numbuf.string(argvars)) as varnumber_T };
}

/// The character count `strchars()` and `strcharlen()` share.
///
/// `skipcc` folds a composing character into the base character it
/// follows; without it each one counts on its own.
unsafe fn strchar_common(argvars: *mut typval_T, rettv: *mut typval_T, skipcc: bool) {
    let mut numbuf = NumBuf::new();
    let next_char: unsafe fn(*mut *const c_char) -> c_int = if skipcc {
        mb_ptr2char_adv
    } else {
        mb_cptr2char_adv
    };
    let mut s = unsafe { numbuf.string(argvars) };
    let mut len: varnumber_T = 0;
    while unsafe { *s } != 0 {
        unsafe { next_char(&raw mut s) };
        len += 1;
    }
    unsafe { (*rettv).vval.v_number = len };
}

/// "strcharlen()" function: characters, composing characters folded in.
pub unsafe fn f_strcharlen(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe { strchar_common(argvars, rettv, true) }
}

/// "strchars()" function: characters, composing ones counted unless the
/// optional `skipcc` argument says otherwise.
pub unsafe fn f_strchars(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let skipcc = if given(unsafe { &*argvars.add(1) }) {
        match unsafe { strict_bool_arg(argvars.add(1)) } {
            Some(flag) => flag,
            None => return,
        }
    } else {
        false
    };
    unsafe { strchar_common(argvars, rettv, skipcc) };
}

/// "strdisplaywidth()" function: screen cells, tabs expanded against the
/// optional starting column.
pub unsafe fn f_strdisplaywidth(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let s = unsafe { numbuf.string(argvars) };
    let col = if given(unsafe { &*argvars.add(1) }) {
        unsafe { tv_get_number(argvars.add(1)) as c_int }
    } else {
        0
    };
    unsafe {
        (*rettv).vval.v_number = (linetabsize_col(col, s as *mut c_char) - col) as varnumber_T
    };
}

/// "strwidth()" function: screen cells, with a tab counting as one.
pub unsafe fn f_strwidth(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe { (*rettv).vval.v_number = mb_string2cells(numbuf.string(argvars)) as varnumber_T };
}

/// "strtrans()" function: unprintable characters as `^X`/`<xx>`.
pub unsafe fn f_strtrans(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe { (*rettv).v_type = VAR_STRING };
    unsafe { (*rettv).vval.v_string = transstr(numbuf.string(argvars), true) };
}

/// "tolower()" function.
pub unsafe fn f_tolower(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe { (*rettv).v_type = VAR_STRING };
    unsafe { (*rettv).vval.v_string = strcase_save(numbuf.string(argvars), false) };
}

/// "toupper()" function.
pub unsafe fn f_toupper(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe { (*rettv).v_type = VAR_STRING };
    unsafe { (*rettv).vval.v_string = strcase_save(numbuf.string(argvars), true) };
}

/// "tr()" function: character-wise translation.
///
/// `fromstr` and `tostr` must hold the same number of characters, which is
/// checked lazily -- either when a mapped character's counterpart runs off
/// the end of `tostr`, or once, the first time an input character is *not*
/// in `fromstr` and the counts can be compared directly. So
/// `tr('a', 'ab', 'x')` is an error but `tr('a', 'a', 'x')` is not.
pub unsafe fn f_tr(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut buf = [0 as c_char; NUMBUFLEN];
    let mut buf2 = [0 as c_char; NUMBUFLEN];
    let mut in_str = unsafe { numbuf.string(argvars) };
    let fromstr = unsafe { tv_get_string_buf_chk(argvars.add(1), buf.as_mut_ptr()) };
    let tostr = unsafe { tv_get_string_buf_chk(argvars.add(2), buf2.as_mut_ptr()) };

    unsafe { (*rettv).v_type = VAR_STRING };
    unsafe { (*rettv).vval.v_string = ptr::null_mut() };
    if fromstr.is_null() || tostr.is_null() {
        return; // Type error; the message is already out.
    }

    // The `n`-th character of a set, as (start, byte length).
    let nth_char = |set: *const c_char, n: c_int| -> Option<(*const c_char, c_int)> {
        let mut p = set;
        let mut left = n;
        while unsafe { *p } != 0 {
            let len = unsafe { utfc_ptr2len(p) };
            if left == 0 {
                return Some((p, len));
            }
            left -= 1;
            p = unsafe { p.offset(len as isize) };
        }
        None
    };
    let count_chars = |set: *const c_char| -> c_int {
        let mut p = set;
        let mut n = 0;
        while unsafe { *p } != 0 {
            p = unsafe { p.offset(utfc_ptr2len(p) as isize) };
            n += 1;
        }
        n
    };

    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ptr::null_mut::<c_void>(),
    };
    unsafe { ga_init(&raw mut ga, ::core::mem::size_of::<c_char>() as c_int, 80) };

    let mut lengths_checked = false;
    'error: {
        while unsafe { *in_str } != 0 {
            let inlen = unsafe { utfc_ptr2len(in_str) };

            // Which character of `fromstr` is this, if any?
            let mut idx = 0;
            let mut found = false;
            let mut p = fromstr;
            while unsafe { *p } != 0 {
                let fromlen = unsafe { utfc_ptr2len(p) };
                if fromlen == inlen && unsafe { strncmp(in_str, p, inlen as size_t) } == 0 {
                    found = true;
                    break;
                }
                idx += 1;
                p = unsafe { p.offset(fromlen as isize) };
            }

            let (cpstr, cplen) = if found {
                match nth_char(tostr, idx) {
                    Some(hit) => hit,
                    None => break 'error, // tostr is shorter than fromstr
                }
            } else {
                if !lengths_checked {
                    lengths_checked = true;
                    // `idx` is now `fromstr`'s character count.
                    if count_chars(tostr) != idx {
                        break 'error;
                    }
                }
                (in_str, inlen)
            };

            unsafe { ga_grow(&raw mut ga, cplen) };
            let end = unsafe { (ga.ga_data as *mut c_char).offset(ga.ga_len as isize) };
            unsafe { ptr::copy_nonoverlapping(cpstr, end, cplen as usize) };
            ga.ga_len += cplen;
            in_str = unsafe { in_str.offset(inlen as isize) };
        }
        unsafe { ga_append(&raw mut ga, 0 as uint8_t) };
        unsafe { (*rettv).vval.v_string = ga.ga_data as *mut c_char };
        return;
    }
    // SAFETY: a message argument the caller holds as a NUL-terminated string.
    let fromstr = unsafe { c_str(fromstr) };
    semsg!("E475: Invalid argument: {fromstr}");
    unsafe { ga_clear(&raw mut ga) };
}

/// "trim()" function.
///
/// `dir` is 0 (both ends, the default), 1 (leading) or 2 (trailing). With
/// no mask the set trimmed is whitespace plus U+00A0; with one it is
/// exactly the mask's characters, and an empty mask reverts to the
/// default.
pub unsafe fn f_trim(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut buf1 = [0 as c_char; NUMBUFLEN];
    let mut buf2 = [0 as c_char; NUMBUFLEN];
    let mut head = unsafe { tv_get_string_buf_chk(argvars, buf1.as_mut_ptr()) };
    let mut mask = ptr::null::<c_char>();
    let mut dir = 0;

    unsafe { (*rettv).v_type = VAR_STRING };
    unsafe { (*rettv).vval.v_string = ptr::null_mut() };
    if head.is_null() || unsafe { tv_check_for_opt_string_arg(argvars, 1) }.is_err() {
        return;
    }

    if unsafe { (*argvars.add(1)).v_type } == VAR_STRING {
        mask = unsafe { tv_get_string_buf_chk(argvars.add(1), buf2.as_mut_ptr()) };
        if unsafe { *mask } == 0 {
            mask = ptr::null();
        }
        if given(unsafe { &*argvars.add(2) }) {
            let mut error = false;
            dir = unsafe { tv_get_number_chk(argvars.add(2), &raw mut error) as c_int };
            if error {
                return;
            }
            if !(0..=2).contains(&dir) {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let arg0 = unsafe { c_str(numbuf.string(argvars.add(2))) };
                semsg!("E475: Invalid argument: {arg0}");
                return;
            }
        }
    }

    // Whitespace and NBSP by default, else exactly the mask's set.
    let trimmable = |c: c_int| -> bool {
        if mask.is_null() {
            return c <= b' ' as c_int || c == 0xa0;
        }
        let mut p = mask;
        while unsafe { *p } != 0 {
            if c == unsafe { utf_ptr2char(p) } {
                return true;
            }
            p = unsafe { p.offset(utfc_ptr2len(p) as isize) };
        }
        false
    };

    if dir == 0 || dir == 1 {
        while unsafe { *head } != 0 && trimmable(unsafe { utf_ptr2char(head) }) {
            head = unsafe { head.offset(utfc_ptr2len(head) as isize) };
        }
    }

    let mut tail = unsafe { head.add(strlen(head)) };
    if dir == 0 || dir == 2 {
        while tail > head {
            // Step back over one whole character.
            let prev = unsafe {
                tail.offset(-(utf_head_off(head as *mut c_char, tail.sub(1)) as isize) - 1)
            };
            if !trimmable(unsafe { utf_ptr2char(prev) }) {
                break;
            }
            tail = prev;
        }
    }

    unsafe { (*rettv).vval.v_string = xstrnsave(head, tail.offset_from(head) as size_t) };
}
