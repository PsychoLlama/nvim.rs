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

use crate::cstr;
use core::ffi::{CStr, c_char, c_int};
use core::{ptr, slice};

use crate::eval::encode::{encode_tv2echo, encode_tv2string};
use crate::eval::typval::{
    tv_clear, tv_dict_add_tv, tv_dict_alloc, tv_dict_find, tv_dict_hi2di, tv_dict_iter, tv_equal,
};
use crate::eval::vars::assert_error;
use crate::mbyte::{mb_cptr2char_adv, utf_ptr2char};
use crate::memory::xfree;
use crate::runtime::estack_sfile;
use crate::types::{VAR_DICT, VAR_STRING, VAR_UNKNOWN, linenr_T, typval_T};

use super::{AssertType, ESTACK_NONE};

/// `GA_CONCAT_LITERAL`: append a C literal, whose length is known here.
pub(super) fn ga_concat_lit(gap: &mut Vec<u8>, text: &'static CStr) {
    gap.extend_from_slice(text.to_bytes());
}

/// Append a NUL-terminated string's bytes, terminator excluded. A null
/// string appends nothing, as `ga_concat` did.
///
/// # Safety
/// `s` is null or NUL-terminated.
pub(super) unsafe fn ga_concat_cstr(gap: &mut Vec<u8>, s: *const c_char) {
    if s.is_null() {
        return;
    }
    // SAFETY: the caller's promise.
    gap.extend_from_slice(unsafe { CStr::from_ptr(s) }.to_bytes());
}

/// Append `len` bytes of `p`, NULs and all.
///
/// # Safety
/// `p` is readable for `len` bytes.
pub(super) unsafe fn ga_concat_bytes(gap: &mut Vec<u8>, p: *const c_char, len: usize) {
    // SAFETY: the caller's promise.
    gap.extend_from_slice(unsafe { slice::from_raw_parts(p.cast::<u8>(), len) });
}

/// Line number being sourced or executed: the top of the exestack.
pub(super) fn sourcing_lnum() -> linenr_T {
    crate::runtime::innermost_frame().es_lnum
}

/// A fresh error buffer, opened with the sourcing position: `script line N: `.
///
/// # Safety
/// Called while a script or function is executing.
pub(super) unsafe fn prepare_assert_error() -> Vec<u8> {
    let mut ga = Vec::<u8>::new();
    let gap = &mut ga;
    // SAFETY: the `estack_sfile` allocation, freed here.
    let sname = unsafe { estack_sfile(ESTACK_NONE) };
    let lnum = sourcing_lnum();
    if !sname.is_null() {
        // SAFETY: `estack_sfile` answers a NUL-terminated name.
        unsafe { ga_concat_cstr(gap, sname) };
        if lnum > 0 {
            gap.push(b' ');
        }
    }
    if lnum > 0 {
        gap.extend_from_slice(format!("line {lnum}").as_bytes());
    }
    if !sname.is_null() || lnum > 0 {
        ga_concat_lit(gap, c": ");
    }
    // SAFETY: the allocation this function owns.
    unsafe { xfree(sname.cast()) };
    ga
}

/// Publish `gap` as one `v:errors` entry.
pub(super) fn report_assert_error(gap: &[u8]) {
    assert_error(gap);
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
unsafe fn ga_concat_esc(gap: &mut Vec<u8>, p: *const c_char, clen: c_int) {
    // SAFETY: the caller's buffer.
    if clen > 1 {
        unsafe { ga_concat_bytes(gap, p, clen as usize) };
        return;
    }
    let byte = unsafe { *p };
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
        gap.extend_from_slice(format!("\\x{:02x}", byte as u8).as_bytes());
    } else {
        gap.push(byte as u8);
    }
}

/// Append `str` to `gap` escaped, collapsing a run of more than 20 identical
/// characters into `\[c occurs N times]` so a long message stays readable.
///
/// # Safety
/// `gap` is open; `str` is null or a C string.
unsafe fn ga_concat_shorten_esc(gap: &mut Vec<u8>, str: *const c_char) {
    // SAFETY: the caller's garray and string; the walk stops at the NUL.
    if str.is_null() {
        ga_concat_lit(gap, c"NULL");
        return;
    }
    let mut p = str;
    while unsafe { *p } != 0 {
        let mut s = p;
        let c = unsafe { mb_cptr2char_adv(&raw mut s) };
        let clen = unsafe { s.offset_from(p) } as c_int;
        let mut same_len = 1;
        while unsafe { *s } != 0 && c == unsafe { utf_ptr2char(s) } {
            same_len += 1;
            s = unsafe { s.offset(clen as isize) };
        }
        if same_len > 20 {
            ga_concat_lit(gap, c"\\[");
            unsafe { ga_concat_esc(gap, p, clen) };
            ga_concat_lit(gap, c" occurs ");
            gap.extend_from_slice(format!("{same_len}").as_bytes());
            ga_concat_lit(gap, c" times]");
            p = s;
        } else {
            unsafe { ga_concat_esc(gap, p, clen) };
            p = unsafe { p.offset(clen as isize) };
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
unsafe fn append_opt_msg(gap: &mut Vec<u8>, opt_msg_tv: *mut typval_T) {
    // SAFETY: the caller's garray and typval; `encode_tv2echo` allocates.
    let msg = unsafe { &*opt_msg_tv };
    let blank = msg.v_type == VAR_STRING
        && (unsafe { msg.vval.v_string }.is_null() || unsafe { *msg.vval.v_string } == 0);
    if msg.v_type == VAR_UNKNOWN || blank {
        return;
    }
    let tofree = unsafe { encode_tv2echo(opt_msg_tv, ptr::null_mut()) };
    unsafe { ga_concat_cstr(gap, tofree) };
    unsafe { xfree(tofree.cast()) };
    ga_concat_lit(gap, c": ");
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
    let (exp_d, got_d) = unsafe { ((*exp_tv).vval.v_dict, (*got_tv).vval.v_dict) };
    // The pruned copies that replace them, which the caller then owns.
    let (exp, got) = unsafe { (tv_dict_alloc(), tv_dict_alloc()) };
    unsafe { (*exp_tv).vval.v_dict = exp };
    unsafe { (*got_tv).vval.v_dict = got };

    let mut omitted = 0;
    for hi in unsafe { tv_dict_iter(exp_d) } {
        let key = hi.hi_key;
        let expected = unsafe { &raw mut (*tv_dict_hi2di(hi)).di_tv };
        let item2 = unsafe { tv_dict_find(got_d, key, -1) };
        if !item2.is_null() && unsafe { tv_equal(expected, &raw mut (*item2).di_tv, false) } {
            omitted += 1;
            continue;
        }
        // Absent from the actual value, or present with a different one.
        let key_len = unsafe { cstr::bytes_at(key) }.len();
        let _ = unsafe { tv_dict_add_tv(exp, key, key_len, expected) };
        if !item2.is_null() {
            let _ = unsafe { tv_dict_add_tv(got, key, key_len, &raw mut (*item2).di_tv) };
        }
    }

    // Entries only the actual value has.
    for hi in unsafe { tv_dict_iter(got_d) } {
        let key = hi.hi_key;
        if unsafe { tv_dict_find(exp_d, key, -1) }.is_null() {
            let tv = unsafe { &raw mut (*tv_dict_hi2di(hi)).di_tv };
            let _ = unsafe { tv_dict_add_tv(got, key, cstr::bytes_at(key).len(), tv) };
        }
    }
    omitted
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
    gap: &mut Vec<u8>,
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
    unsafe { append_opt_msg(gap, opt_msg_tv) };
    ga_concat_lit(
        gap,
        match atype {
            AssertType::Match | AssertType::NotMatch => c"Pattern ",
            AssertType::NotEqual => c"Expected not equal to ",
            _ => c"Expected ",
        },
    );

    if exp_str.is_null() {
        if atype != AssertType::NotEqual && unsafe { is_dict(exp_tv) } && unsafe { is_dict(got_tv) }
        {
            did_copy = true;
            omitted = unsafe { prune_equal_dict_items(exp_tv, got_tv) };
        }
        let tofree = unsafe { encode_tv2string(exp_tv, ptr::null_mut()) };
        unsafe { ga_concat_shorten_esc(gap, tofree) };
        unsafe { xfree(tofree.cast()) };
    } else {
        let quoted = atype == AssertType::Fails;
        if quoted {
            ga_concat_lit(gap, c"'");
        }
        unsafe { ga_concat_shorten_esc(gap, exp_str) };
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
        let tofree = unsafe { encode_tv2string(got_tv, ptr::null_mut()) };
        unsafe { ga_concat_shorten_esc(gap, tofree) };
        unsafe { xfree(tofree.cast()) };

        if omitted != 0 {
            let plural = if omitted == 1 { "" } else { "s" };
            let text = format!(" - {omitted} equal item{plural} omitted");
            gap.extend_from_slice(text.as_bytes());
        }
    }

    if did_copy {
        unsafe { tv_clear(exp_tv) };
        unsafe { tv_clear(got_tv) };
    }
}
