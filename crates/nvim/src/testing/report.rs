//! Building the line a failed `assert_*()` appends to `v:errors`.
//!
//! [`prepare_assert_error`] opens the buffer with the sourcing position,
//! [`fill_assert_error`] says what was expected and what arrived, and the
//! caller publishes it with [`report_assert_error`]. Unprintable bytes are
//! escaped on the way in, and a long run of one character is collapsed, so
//! that a failure over binary data is still readable.
//!
//! Every string in here is matched on by tests. None of it may drift.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use crate::eval::encode::{encode_tv2echo, encode_tv2string};
use crate::eval::typval::{
    tv_clear, tv_dict_add_tv, tv_dict_alloc, tv_dict_find, tv_dict_hi2di, tv_dict_iter, tv_equal,
};
use crate::eval::vars::assert_error;
use crate::garray::{ga_append, ga_clear, ga_concat, ga_concat_len, ga_init};
use crate::mbyte::{mb_cptr2char_adv, utf_ptr2char};
use crate::memory::xfree;
use crate::os::libc::strlen;
use crate::runtime::{estack_sfile, exestack};
use crate::strings::vim_snprintf_safelen;
use crate::types::{
    VAR_DICT, VAR_STRING, VAR_UNKNOWN, estack_T, garray_T, int64_t, linenr_T, size_t, typval_T,
};

use super::{AssertType, ESTACK_NONE, NUMBUFLEN};

/// An unopened growable byte buffer.
pub(super) fn empty_garray() -> garray_T {
    garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ptr::null_mut(),
    }
}

/// `GA_CONCAT_LITERAL`: append a C literal, whose length is known here.
///
/// # Safety
/// `gap` is an open byte garray.
pub(super) unsafe fn ga_concat_lit(gap: *mut garray_T, text: &'static CStr) {
    // SAFETY: the caller's garray; a `CStr` knows its own length.
    unsafe { ga_concat_len(gap, text.as_ptr(), text.count_bytes()) };
}

/// Line number being sourced or executed: the top of the exestack.
///
/// # Safety
/// The exestack is non-empty, which it is for the whole of a session.
pub(super) unsafe fn sourcing_lnum() -> linenr_T {
    let es = exestack.get();
    // SAFETY: the caller's contract; the stack holds `ga_len` entries.
    unsafe { (*(es.ga_data as *mut estack_T).offset(es.ga_len as isize - 1)).es_lnum }
}

/// A fresh error buffer, opened with the sourcing position: `script line N: `.
///
/// # Safety
/// Called while a script or function is executing.
pub(super) unsafe fn prepare_assert_error() -> garray_T {
    let mut ga = empty_garray();
    let gap = &raw mut ga;
    // SAFETY: the local garray, and the `estack_sfile` allocation freed here.
    unsafe {
        let sname = estack_sfile(ESTACK_NONE);
        ga_init(gap, 1, 100);
        let lnum = sourcing_lnum();
        if !sname.is_null() {
            ga_concat(gap, sname);
            if lnum > 0 {
                ga_concat(gap, c" ".as_ptr());
            }
        }
        if lnum > 0 {
            let mut buf = [0 as c_char; NUMBUFLEN];
            let buflen = vim_snprintf_safelen(
                buf.as_mut_ptr(),
                NUMBUFLEN,
                c"line %ld".as_ptr(),
                lnum as int64_t,
            );
            ga_concat_len(gap, buf.as_ptr(), buflen);
        }
        if !sname.is_null() || lnum > 0 {
            ga_concat_lit(gap, c": ");
        }
        xfree(sname.cast());
    }
    ga
}

/// Publish `gap` as one `v:errors` entry and release it.
///
/// # Safety
/// `gap` is an open byte garray this call takes over.
pub(super) unsafe fn report_assert_error(gap: *mut garray_T) {
    // SAFETY: the caller's garray.
    unsafe {
        assert_error(gap);
        ga_clear(gap);
    }
}

// ---------------------------------------------------------------------------
// Escaping
// ---------------------------------------------------------------------------

/// Append `p[..clen]` to `gap`, escaping the unprintable single bytes.
///
/// NL becomes `\n`, CR `\r`, and anything else below space (or DEL) becomes
/// `\xNN`. A multibyte character goes through unchanged.
///
/// # Safety
/// `gap` is open and `p` has at least `clen` readable bytes.
unsafe fn ga_concat_esc(gap: *mut garray_T, p: *const c_char, clen: c_int) {
    // SAFETY: the caller's garray and buffer.
    unsafe {
        if clen > 1 {
            ga_concat_len(gap, p, clen as size_t);
            return;
        }
        let byte = *p;
        let escaped = match byte as u8 {
            b'\x08' => Some(c"\\b"),
            b'\x1b' => Some(c"\\e"),
            b'\x0c' => Some(c"\\f"),
            b'\n' => Some(c"\\n"),
            b'\t' => Some(c"\\t"),
            b'\r' => Some(c"\\r"),
            b'\\' => Some(c"\\\\"),
            _ => None,
        };
        if let Some(text) = escaped {
            ga_concat_lit(gap, text);
        } else if (byte as u8) < b' ' || byte as u8 == 0x7f {
            let mut buf = [0 as c_char; NUMBUFLEN];
            let buflen = vim_snprintf_safelen(
                buf.as_mut_ptr(),
                NUMBUFLEN,
                c"\\x%02x".as_ptr(),
                byte as c_int,
            );
            ga_concat_len(gap, buf.as_ptr(), buflen);
        } else {
            ga_append(gap, byte as u8);
        }
    }
}

/// Append `str` to `gap` escaped, collapsing a run of more than 20 identical
/// characters into `\[c occurs N times]` so a long message stays readable.
///
/// # Safety
/// `gap` is open; `str` is null or a C string.
unsafe fn ga_concat_shorten_esc(gap: *mut garray_T, str: *const c_char) {
    // SAFETY: the caller's garray and string; the walk stops at the NUL.
    unsafe {
        if str.is_null() {
            ga_concat_lit(gap, c"NULL");
            return;
        }
        let mut p = str;
        while *p != 0 {
            let mut s = p;
            let c = mb_cptr2char_adv(&raw mut s);
            let clen = s.offset_from(p) as c_int;
            let mut same_len = 1;
            while *s != 0 && c == utf_ptr2char(s) {
                same_len += 1;
                s = s.offset(clen as isize);
            }
            if same_len > 20 {
                ga_concat_lit(gap, c"\\[");
                ga_concat_esc(gap, p, clen);
                ga_concat_lit(gap, c" occurs ");
                let mut buf = [0 as c_char; NUMBUFLEN];
                let buflen =
                    vim_snprintf_safelen(buf.as_mut_ptr(), NUMBUFLEN, c"%d".as_ptr(), same_len);
                ga_concat_len(gap, buf.as_ptr(), buflen);
                ga_concat_lit(gap, c" times]");
                p = s;
            } else {
                ga_concat_esc(gap, p, clen);
                p = p.offset(clen as isize);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The failure message
// ---------------------------------------------------------------------------

/// Prefix `gap` with the caller's own message, when they gave one.
///
/// An empty string counts as no message, which is what lets every
/// `assert_*()`'s optional `msg` argument be passed through unconditionally.
///
/// # Safety
/// `gap` is open and `opt_msg_tv` is a live typval.
unsafe fn append_opt_msg(gap: *mut garray_T, opt_msg_tv: *mut typval_T) {
    // SAFETY: the caller's garray and typval; `encode_tv2echo` allocates.
    unsafe {
        let msg = &*opt_msg_tv;
        let blank =
            msg.v_type == VAR_STRING && (msg.vval.v_string.is_null() || *msg.vval.v_string == 0);
        if msg.v_type == VAR_UNKNOWN || blank {
            return;
        }
        let tofree = encode_tv2echo(opt_msg_tv, ptr::null_mut());
        ga_concat(gap, tofree);
        xfree(tofree.cast());
        ga_concat_lit(gap, c": ");
    }
}

/// Whether `tv` holds a non-null dictionary.
///
/// # Safety
/// `tv` is a live typval.
unsafe fn is_dict(tv: *mut typval_T) -> bool {
    // SAFETY: the caller's typval.
    unsafe { (*tv).v_type == VAR_DICT && !(*tv).vval.v_dict.is_null() }
}

/// Replace both dictionaries with copies holding only the entries that differ,
/// and answer how many equal ones were dropped.
///
/// Comparing two large dictionaries is unreadable unless the equal items go
/// away. The caller owns the two new dictionaries and clears them.
///
/// # Safety
/// Both typvals hold non-null dictionaries.
unsafe fn prune_equal_dict_items(exp_tv: *mut typval_T, got_tv: *mut typval_T) -> c_int {
    // SAFETY: the caller's dictionaries. The two walks only ever add to the
    // *new* dictionaries, so neither hashtab is rehashed under its own walk.
    unsafe {
        let exp_d = (*exp_tv).vval.v_dict;
        let got_d = (*got_tv).vval.v_dict;
        (*exp_tv).vval.v_dict = tv_dict_alloc();
        (*got_tv).vval.v_dict = tv_dict_alloc();

        let mut omitted = 0;
        for hi in tv_dict_iter(&*exp_d) {
            let key = (*hi).hi_key;
            let expected = &raw mut (*tv_dict_hi2di(hi)).di_tv;
            let item2 = tv_dict_find(got_d, key, -1);
            if !item2.is_null() && tv_equal(expected, &raw mut (*item2).di_tv, false) {
                omitted += 1;
                continue;
            }
            // Absent from the actual value, or present with a different one.
            let key_len = strlen(key);
            tv_dict_add_tv((*exp_tv).vval.v_dict, key, key_len, expected);
            if !item2.is_null() {
                tv_dict_add_tv((*got_tv).vval.v_dict, key, key_len, &raw mut (*item2).di_tv);
            }
        }

        // Entries only the actual value has.
        for hi in tv_dict_iter(&*got_d) {
            let key = (*hi).hi_key;
            if tv_dict_find(exp_d, key, -1).is_null() {
                let got = &raw mut (*tv_dict_hi2di(hi)).di_tv;
                tv_dict_add_tv((*got_tv).vval.v_dict, key, strlen(key), got);
            }
        }
        omitted
    }
}

/// Fill `gap` with what was expected and what arrived.
///
/// The expectation is either `exp_str` (already formatted, e.g. `"True"` or a
/// range) or `exp_tv` (encoded here). `ASSERT_NOTEQUAL` prints no "but got"
/// half — for it the actual value *is* the expected one.
///
/// # Safety
/// `gap` is open; the typvals are live, and `exp_tv`/`got_tv` may be null only
/// when `exp_str` is not.
pub(super) unsafe fn fill_assert_error(
    gap: *mut garray_T,
    opt_msg_tv: *mut typval_T,
    exp_str: *const c_char,
    exp_tv: *mut typval_T,
    got_tv: *mut typval_T,
    atype: AssertType,
) {
    let mut did_copy = false;
    let mut omitted = 0;

    // SAFETY: the caller's garray and typvals; each `encode_tv2*` allocation
    // is freed where it is made.
    unsafe {
        append_opt_msg(gap, opt_msg_tv);
        ga_concat_lit(
            gap,
            match atype {
                AssertType::Match | AssertType::NotMatch => c"Pattern ",
                AssertType::NotEqual => c"Expected not equal to ",
                _ => c"Expected ",
            },
        );

        if exp_str.is_null() {
            if atype != AssertType::NotEqual && is_dict(exp_tv) && is_dict(got_tv) {
                did_copy = true;
                omitted = prune_equal_dict_items(exp_tv, got_tv);
            }
            let tofree = encode_tv2string(exp_tv, ptr::null_mut());
            ga_concat_shorten_esc(gap, tofree);
            xfree(tofree.cast());
        } else {
            let quoted = atype == AssertType::Fails;
            if quoted {
                ga_concat_lit(gap, c"'");
            }
            ga_concat_shorten_esc(gap, exp_str);
            if quoted {
                ga_concat_lit(gap, c"'");
            }
        }

        if atype != AssertType::NotEqual {
            ga_concat_lit(
                gap,
                match atype {
                    AssertType::Match => c" does not match ",
                    AssertType::NotMatch => c" does match ",
                    _ => c" but got ",
                },
            );
            let tofree = encode_tv2string(got_tv, ptr::null_mut());
            ga_concat_shorten_esc(gap, tofree);
            xfree(tofree.cast());

            if omitted != 0 {
                let mut buf = [0 as c_char; 100];
                let buflen = vim_snprintf_safelen(
                    buf.as_mut_ptr(),
                    buf.len(),
                    c" - %d equal item%s omitted".as_ptr(),
                    omitted,
                    if omitted == 1 {
                        c"".as_ptr()
                    } else {
                        c"s".as_ptr()
                    },
                );
                ga_concat_len(gap, buf.as_ptr(), buflen);
            }
        }

        if did_copy {
            tv_clear(exp_tv);
            tv_clear(got_tv);
        }
    }
}
