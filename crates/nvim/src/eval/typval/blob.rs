//! `blob_T`: a reference-counted byte vector, and the builtins over it.
//!
//! [`tv_blob_alloc`] / [`tv_blob_unref`] are the lifetime pair.
//! [`tv_blob_slice_or_index`] is the subscript, [`tv_blob_set_range`] and
//! [`tv_blob_set_append`] the two ways an assignment writes into one, and
//! [`tv_blob_remove`] is `remove()`.  [`f_blob2list`] and [`f_list2blob`]
//! convert to and from a list of byte numbers.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::semsg_c;
use crate::types::{FAIL, OK};

/// Allocate an empty blob.  The caller owns the reference count.
pub unsafe fn tv_blob_alloc() -> *mut blob_T {
    unsafe {
        let blob = xcalloc(1, ::core::mem::size_of::<blob_T>()) as *mut blob_T;
        ga_init(&raw mut (*blob).bv_ga, 1, 100);
        blob
    }
}

/// Free `b` and its bytes.
pub unsafe fn tv_blob_free(b: *mut blob_T) {
    unsafe {
        ga_clear(&raw mut (*b).bv_ga);
        xfree(b.cast());
    }
}

/// Drop a reference to `b`, freeing it when the last one goes.
pub unsafe fn tv_blob_unref(b: *mut blob_T) {
    unsafe {
        if let Some(blob) = b.as_mut() {
            blob.bv_refcount -= 1;
            if blob.bv_refcount <= 0 {
                tv_blob_free(b);
            }
        }
    }
}

/// Whether `b1` and `b2` hold the same bytes.  An empty blob and a NULL one
/// are equal.
pub unsafe fn tv_blob_equal(b1: *const blob_T, b2: *const blob_T) -> bool {
    unsafe {
        let len1 = tv_blob_len(b1);
        let len2 = tv_blob_len(b2);
        if len1 == 0 && len2 == 0 {
            return true;
        }
        if b1 == b2 {
            return true;
        }
        if len1 != len2 {
            return false;
        }
        let mut i = 0;
        while i < (*b1).bv_ga.ga_len {
            if tv_blob_get(b1, i) != tv_blob_get(b2, i) {
                return false;
            }
            i += 1;
        }
        true
    }
}

/// `blob[n1 : n2]`: store the sub-blob in `rettv`.
///
/// `rettv` holds the blob being subscripted on the way in.  Indexes out of
/// range give an empty result rather than an error.
pub(crate) unsafe fn tv_blob_slice(
    _blob: *const blob_T,
    len: ::core::ffi::c_int,
    mut n1: varnumber_T,
    mut n2: varnumber_T,
    exclusive: bool,
    rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
        // The resulting variable is a sub-blob.  If the indexes
        // are out of range the result is empty.
        if n1 < 0 {
            n1 = varnumber_T::from(len) + n1;
            if n1 < 0 {
                n1 = 0;
            }
        }
        if n2 < 0 {
            n2 = varnumber_T::from(len) + n2;
        } else if n2 >= varnumber_T::from(len) {
            n2 = varnumber_T::from(len - if exclusive { 0 } else { 1 });
        }
        if exclusive {
            n2 -= 1;
        }

        if n1 >= varnumber_T::from(len) || n2 < 0 || n1 > n2 {
            tv_clear(rettv);
            (*rettv).v_type = VAR_BLOB;
            (*rettv).vval.v_blob = ::core::ptr::null_mut();
        } else {
            let new_blob = tv_blob_alloc();
            let sublen = (n2 - n1 + 1) as ::core::ffi::c_int;
            ga_grow(&raw mut (*new_blob).bv_ga, sublen);
            (*new_blob).bv_ga.ga_len = sublen;
            let n1 = n1 as ::core::ffi::c_int;
            let mut i = n1;
            while i <= n2 as ::core::ffi::c_int {
                tv_blob_set(new_blob, i - n1, tv_blob_get((*rettv).vval.v_blob, i));
                i += 1;
            }
            tv_clear(rettv);
            tv_blob_set_ret(rettv, new_blob);
        }

        OK
    }
}

/// `blob[idx]`: store the byte in `rettv`.
///
/// `rettv` holds the blob being subscripted on the way in.  An index out of
/// range raises `E979`.
pub(crate) unsafe fn tv_blob_index(
    _blob: *const blob_T,
    len: ::core::ffi::c_int,
    mut idx: varnumber_T,
    rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
        // The resulting variable is a byte value.
        // If the index is too big or negative that is an error.
        if idx < 0 {
            idx = varnumber_T::from(len) + idx;
        }
        if idx >= varnumber_T::from(len) || idx < 0 {
            semsg_c!(
                gettext(&raw const e_blobidx as *const ::core::ffi::c_char),
                idx,
            );
            return FAIL;
        }

        let v = tv_blob_get((*rettv).vval.v_blob, idx as ::core::ffi::c_int);
        tv_clear(rettv);
        (*rettv).v_type = VAR_NUMBER;
        (*rettv).vval.v_number = varnumber_T::from(v);
        OK
    }
}

/// `blob[n1]` or `blob[n1 : n2]`, whichever `is_range` says.
pub unsafe fn tv_blob_slice_or_index(
    blob: *const blob_T,
    is_range: bool,
    n1: varnumber_T,
    n2: varnumber_T,
    exclusive: bool,
    rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
        let len = tv_blob_len((*rettv).vval.v_blob);
        if is_range {
            tv_blob_slice(blob, len, n1, n2, exclusive, rettv)
        } else {
            tv_blob_index(blob, len, n1, rettv)
        }
    }
}

/// Whether `n1` names a byte of a `bloblen`-byte blob, or the slot just past
/// the end (which an assignment may append to).
pub unsafe fn tv_blob_check_index(
    bloblen: ::core::ffi::c_int,
    n1: varnumber_T,
    quiet: bool,
) -> ::core::ffi::c_int {
    unsafe {
        if n1 < 0 || n1 > varnumber_T::from(bloblen) {
            if !quiet {
                semsg_c!(
                    gettext(&raw const e_blobidx as *const ::core::ffi::c_char),
                    n1,
                );
            }
            return FAIL;
        }
        OK
    }
}

/// Whether `n1..=n2` is a range of a `bloblen`-byte blob.
pub unsafe fn tv_blob_check_range(
    bloblen: ::core::ffi::c_int,
    n1: varnumber_T,
    n2: varnumber_T,
    quiet: bool,
) -> ::core::ffi::c_int {
    unsafe {
        if n2 < 0 || n2 >= varnumber_T::from(bloblen) || n2 < n1 {
            if !quiet {
                semsg_c!(
                    gettext(&raw const e_blobidx as *const ::core::ffi::c_char),
                    n2,
                );
            }
            return FAIL;
        }
        OK
    }
}

/// `dest[n1 : n2] = src`: copy `src`'s blob over that range of `dest`.
pub unsafe fn tv_blob_set_range(
    dest: *mut blob_T,
    n1: varnumber_T,
    n2: varnumber_T,
    src: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
        if n2 - n1 + 1 != varnumber_T::from(tv_blob_len((*src).vval.v_blob)) {
            emsg(gettext(
                c"E972: Blob value does not have the right number of bytes".as_ptr(),
            ));
            return FAIL;
        }
        let mut il = n1 as ::core::ffi::c_int;
        let mut ir = 0;
        while il <= n2 as ::core::ffi::c_int {
            tv_blob_set(dest, il, tv_blob_get((*src).vval.v_blob, ir));
            il += 1;
            ir += 1;
        }
        OK
    }
}

/// `blob[idx] = byte`, growing the blob by one when `idx` is the slot just
/// past the end.  Anything further out is silently ignored.
pub unsafe fn tv_blob_set_append(blob: *mut blob_T, idx: ::core::ffi::c_int, byte: uint8_t) {
    unsafe {
        let gap = &raw mut (*blob).bv_ga;

        // Allow for appending a byte.  Setting a byte beyond
        // the end is an error otherwise.
        if idx <= (*gap).ga_len {
            if idx == (*gap).ga_len {
                ga_grow(gap, 1);
                (*gap).ga_len += 1;
            }
            tv_blob_set(blob, idx, byte);
        }
    }
}

/// `remove()` over a blob: take out one byte, or the range `[idx, end]`, and
/// store what was removed in `rettv`.
pub unsafe fn tv_blob_remove(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    arg_errmsg: *const ::core::ffi::c_char,
) {
    unsafe {
        let b = (*argvars).vval.v_blob;
        if !b.is_null() && value_check_lock((*b).bv_lock, arg_errmsg, TV_TRANSLATE as size_t) {
            return;
        }

        let mut error = false;
        let mut idx = tv_get_number_chk(argvars.add(1), &raw mut error);
        if error {
            return;
        }

        let len = int64_t::from(tv_blob_len(b));
        if idx < 0 {
            // count from the end
            idx += len;
        }
        if idx < 0 || idx >= len {
            semsg_c!(
                gettext(&raw const e_blobidx as *const ::core::ffi::c_char),
                idx,
            );
            return;
        }

        if (*argvars.add(2)).v_type == VAR_UNKNOWN {
            // Remove one item, return its value.
            let p = (*b).bv_ga.ga_data.cast::<uint8_t>();
            (*rettv).vval.v_number = varnumber_T::from(*p.offset(idx as isize));
            memmove(
                p.offset(idx as isize).cast(),
                p.offset(idx as isize).add(1).cast(),
                (len - idx - 1) as size_t,
            );
            (*b).bv_ga.ga_len -= 1;
            return;
        }

        // Remove range of items, return blob with values.
        let mut end = tv_get_number_chk(argvars.add(2), &raw mut error);
        if error {
            return;
        }
        if end < 0 {
            // count from the end
            end += len;
        }
        if end >= len || idx > end {
            semsg_c!(
                gettext(&raw const e_blobidx as *const ::core::ffi::c_char),
                end,
            );
            return;
        }

        let taken = (end - idx + 1) as ::core::ffi::c_int;
        let blob = tv_blob_alloc();
        (*blob).bv_ga.ga_len = taken;
        ga_grow(&raw mut (*blob).bv_ga, taken);

        // Read `ga_data` after the allocation above, as upstream does.
        let p = (*b).bv_ga.ga_data.cast::<uint8_t>();
        memmove(
            (*blob).bv_ga.ga_data,
            p.offset(idx as isize).cast(),
            taken as size_t,
        );
        tv_blob_set_ret(rettv, blob);

        if len - end - 1 > 0 {
            memmove(
                p.offset(idx as isize).cast(),
                p.offset(end as isize).add(1).cast(),
                (len - end - 1) as size_t,
            );
        }
        (*b).bv_ga.ga_len -= taken;
    }
}

/// `blob2list()`: the blob's bytes as a list of numbers.
pub unsafe fn f_blob2list(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        tv_list_alloc_ret(rettv, kListLenMayKnow as ptrdiff_t);
        if tv_check_for_blob_arg(argvars, 0) == FAIL {
            return;
        }
        let blob = (*argvars).vval.v_blob;
        let l = (*rettv).vval.v_list;
        for i in 0..tv_blob_len(blob) {
            tv_list_append_number(l, varnumber_T::from(tv_blob_get(blob, i)));
        }
    }
}

/// `list2blob()`: a list of byte numbers as a blob.
///
/// A value outside `0..=255` raises `E1239` and answers the empty blob.
pub unsafe fn f_list2blob(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        let blob = tv_blob_alloc_ret(rettv);
        if tv_check_for_list_arg(argvars, 0) == FAIL {
            return;
        }
        let l = (*argvars).vval.v_list;
        if l.is_null() {
            return;
        }
        for li in tv_list_iter(l.as_ref()) {
            let mut error = false;
            let n = tv_get_number_chk(&raw const (*li).li_tv, &raw mut error);
            if error || !(0..=255).contains(&n) {
                if !error {
                    semsg_c!(
                        gettext(
                            &raw const e_invalid_value_for_blob_nr as *const ::core::ffi::c_char,
                        ),
                        n as ::core::ffi::c_int,
                    );
                }
                ga_clear(&raw mut (*blob).bv_ga);
                return;
            }
            ga_append(&raw mut (*blob).bv_ga, n as uint8_t);
        }
    }
}

/// Allocate an empty blob and store it in `ret_tv` as the return value.
pub unsafe fn tv_blob_alloc_ret(ret_tv: *mut typval_T) -> *mut blob_T {
    unsafe {
        let b = tv_blob_alloc();
        tv_blob_set_ret(ret_tv, b);
        b
    }
}

/// Store a copy of `from` in `to`.  A NULL blob copies as a NULL blob.
pub unsafe fn tv_blob_copy(from: *mut blob_T, to: *mut typval_T) {
    unsafe {
        (*to).v_type = VAR_BLOB;
        (*to).v_lock = VAR_UNLOCKED;
        if from.is_null() {
            (*to).vval.v_blob = ::core::ptr::null_mut();
            return;
        }

        tv_blob_alloc_ret(to);
        let len = (*from).bv_ga.ga_len;
        let ga = &raw mut (*(*to).vval.v_blob).bv_ga;
        if len > 0 {
            (*ga).ga_data = xmemdup((*from).bv_ga.ga_data, len as size_t);
        }
        (*ga).ga_len = len;
        (*ga).ga_maxlen = len;
    }
}
