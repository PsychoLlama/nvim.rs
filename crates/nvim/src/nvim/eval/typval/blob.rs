//! `blob_T`: a reference-counted byte vector, and the builtins over it.
//!
//! [`tv_blob_alloc`] / [`tv_blob_unref`] are the lifetime pair.
//! [`tv_blob_slice_or_index`] is the subscript, [`tv_blob_set_range`] and
//! [`tv_blob_set_append`] the two ways an assignment writes into one, and
//! [`tv_blob_remove`] is `remove()`.  [`f_blob2list`] and [`f_list2blob`]
//! convert to and from a list of byte numbers.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn tv_blob_alloc() -> *mut blob_T {
    unsafe {
        let blob: *mut blob_T =
            xcalloc(1 as size_t, ::core::mem::size_of::<blob_T>()) as *mut blob_T;
        ga_init(
            &raw mut (*blob).bv_ga,
            1 as ::core::ffi::c_int,
            100 as ::core::ffi::c_int,
        );
        return blob;
    }
}

pub unsafe extern "C" fn tv_blob_free(b: *mut blob_T) {
    unsafe {
        ga_clear(&raw mut (*b).bv_ga);
        xfree(b as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C" fn tv_blob_unref(b: *mut blob_T) {
    unsafe {
        if !b.is_null() && {
            (*b).bv_refcount -= 1;
            (*b).bv_refcount <= 0 as ::core::ffi::c_int
        } {
            tv_blob_free(b);
        }
    }
}

pub unsafe extern "C" fn tv_blob_equal(b1: *const blob_T, b2: *const blob_T) -> bool {
    unsafe {
        let len1: ::core::ffi::c_int = tv_blob_len(b1);
        let len2: ::core::ffi::c_int = tv_blob_len(b2);
        if len1 == 0 as ::core::ffi::c_int && len2 == 0 as ::core::ffi::c_int {
            return true_0 != 0;
        }
        if b1 == b2 {
            return true_0 != 0;
        }
        if len1 != len2 {
            return false_0 != 0;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*b1).bv_ga.ga_len {
            if tv_blob_get(b1, i) as ::core::ffi::c_int != tv_blob_get(b2, i) as ::core::ffi::c_int
            {
                return false_0 != 0;
            }
            i += 1;
        }
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn tv_blob_slice(
    mut _blob: *const blob_T,
    mut len: ::core::ffi::c_int,
    mut n1: varnumber_T,
    mut n2: varnumber_T,
    mut exclusive: bool,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
        if n1 < 0 as varnumber_T {
            n1 = len as varnumber_T + n1;
            if n1 < 0 as varnumber_T {
                n1 = 0 as varnumber_T;
            }
        }
        if n2 < 0 as varnumber_T {
            n2 = len as varnumber_T + n2;
        } else if n2 >= len as varnumber_T {
            n2 = (len
                - (if exclusive as ::core::ffi::c_int != 0 {
                    0 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                })) as varnumber_T;
        }
        if exclusive {
            n2 -= 1;
        }
        if n1 >= len as varnumber_T || n2 < 0 as varnumber_T || n1 > n2 {
            tv_clear(rettv);
            (*rettv).v_type = VAR_BLOB;
            (*rettv).vval.v_blob = ::core::ptr::null_mut::<blob_T>();
        } else {
            let new_blob: *mut blob_T = tv_blob_alloc();
            ga_grow(
                &raw mut (*new_blob).bv_ga,
                (n2 - n1 + 1 as varnumber_T) as ::core::ffi::c_int,
            );
            (*new_blob).bv_ga.ga_len = (n2 - n1 + 1 as varnumber_T) as ::core::ffi::c_int;
            let mut i: ::core::ffi::c_int = n1 as ::core::ffi::c_int;
            while i <= n2 as ::core::ffi::c_int {
                tv_blob_set(
                    new_blob,
                    i - n1 as ::core::ffi::c_int,
                    tv_blob_get((*rettv).vval.v_blob, i),
                );
                i += 1;
            }
            tv_clear(rettv);
            tv_blob_set_ret(rettv, new_blob);
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn tv_blob_index(
    mut _blob: *const blob_T,
    mut len: ::core::ffi::c_int,
    mut idx: varnumber_T,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
        if idx < 0 as varnumber_T {
            idx = len as varnumber_T + idx;
        }
        if idx < len as varnumber_T && idx >= 0 as varnumber_T {
            let v: ::core::ffi::c_int =
                tv_blob_get((*rettv).vval.v_blob, idx as ::core::ffi::c_int) as ::core::ffi::c_int;
            tv_clear(rettv);
            (*rettv).v_type = VAR_NUMBER;
            (*rettv).vval.v_number = v as varnumber_T;
        } else {
            semsg(
                gettext(&raw const e_blobidx as *const ::core::ffi::c_char),
                idx,
            );
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_blob_slice_or_index(
    mut blob: *const blob_T,
    mut is_range: bool,
    mut n1: varnumber_T,
    mut n2: varnumber_T,
    mut exclusive: bool,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut len: ::core::ffi::c_int = tv_blob_len((*rettv).vval.v_blob);
        if is_range {
            return tv_blob_slice(blob, len, n1, n2, exclusive, rettv);
        } else {
            return tv_blob_index(blob, len, n1, rettv);
        };
    }
}

pub unsafe extern "C" fn tv_blob_check_index(
    mut bloblen: ::core::ffi::c_int,
    mut n1: varnumber_T,
    mut quiet: bool,
) -> ::core::ffi::c_int {
    unsafe {
        if n1 < 0 as varnumber_T || n1 > bloblen as varnumber_T {
            if !quiet {
                semsg(
                    gettext(&raw const e_blobidx as *const ::core::ffi::c_char),
                    n1,
                );
            }
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_blob_check_range(
    mut bloblen: ::core::ffi::c_int,
    mut n1: varnumber_T,
    mut n2: varnumber_T,
    mut quiet: bool,
) -> ::core::ffi::c_int {
    unsafe {
        if n2 < 0 as varnumber_T || n2 >= bloblen as varnumber_T || n2 < n1 {
            if !quiet {
                semsg(
                    gettext(&raw const e_blobidx as *const ::core::ffi::c_char),
                    n2,
                );
            }
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_blob_set_range(
    mut dest: *mut blob_T,
    mut n1: varnumber_T,
    mut n2: varnumber_T,
    mut src: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
        if n2 - n1 + 1 as varnumber_T != tv_blob_len((*src).vval.v_blob) as varnumber_T {
            emsg(gettext(
                b"E972: Blob value does not have the right number of bytes\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        let mut il: ::core::ffi::c_int = n1 as ::core::ffi::c_int;
        let mut ir: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while il <= n2 as ::core::ffi::c_int {
            let c2rust_fresh9 = ir;
            ir = ir + 1;
            tv_blob_set(dest, il, tv_blob_get((*src).vval.v_blob, c2rust_fresh9));
            il += 1;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_blob_set_append(
    mut blob: *mut blob_T,
    mut idx: ::core::ffi::c_int,
    mut byte: uint8_t,
) {
    unsafe {
        let mut gap: *mut garray_T = &raw mut (*blob).bv_ga;
        if idx <= (*gap).ga_len {
            if idx == (*gap).ga_len {
                ga_grow(gap, 1 as ::core::ffi::c_int);
                (*gap).ga_len += 1;
            }
            tv_blob_set(blob, idx, byte);
        }
    }
}

pub unsafe extern "C" fn tv_blob_remove(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut arg_errmsg: *const ::core::ffi::c_char,
) {
    unsafe {
        let b: *mut blob_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_blob;
        if !b.is_null()
            && value_check_lock((*b).bv_lock, arg_errmsg, TV_TRANSLATE as size_t)
                as ::core::ffi::c_int
                != 0
        {
            return;
        }
        let mut error: bool = false_0 != 0;
        let mut idx: int64_t = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        );
        if !error {
            let len: ::core::ffi::c_int = tv_blob_len(b);
            if idx < 0 as int64_t {
                idx = len as int64_t + idx;
            }
            if idx < 0 as int64_t || idx >= len as int64_t {
                semsg(
                    gettext(&raw const e_blobidx as *const ::core::ffi::c_char),
                    idx,
                );
                return;
            }
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let p: *mut uint8_t = (*b).bv_ga.ga_data as *mut uint8_t;
                (*rettv).vval.v_number = *p.offset(idx as isize) as varnumber_T;
                memmove(
                    p.offset(idx as isize) as *mut ::core::ffi::c_void,
                    p.offset(idx as isize)
                        .offset(1 as ::core::ffi::c_int as isize)
                        as *const ::core::ffi::c_void,
                    (len as int64_t - idx - 1 as int64_t) as size_t,
                );
                (*b).bv_ga.ga_len -= 1;
            } else {
                let mut end: int64_t = tv_get_number_chk(
                    argvars.offset(2 as ::core::ffi::c_int as isize),
                    &raw mut error,
                );
                if error {
                    return;
                }
                if end < 0 as int64_t {
                    end = len as int64_t + end;
                }
                if end >= len as int64_t || idx > end {
                    semsg(
                        gettext(&raw const e_blobidx as *const ::core::ffi::c_char),
                        end,
                    );
                    return;
                }
                let blob: *mut blob_T = tv_blob_alloc();
                (*blob).bv_ga.ga_len = (end - idx + 1 as int64_t) as ::core::ffi::c_int;
                ga_grow(
                    &raw mut (*blob).bv_ga,
                    (end - idx + 1 as int64_t) as ::core::ffi::c_int,
                );
                let p_0: *mut uint8_t = (*b).bv_ga.ga_data as *mut uint8_t;
                memmove(
                    (*blob).bv_ga.ga_data,
                    p_0.offset(idx as isize) as *const ::core::ffi::c_void,
                    (end - idx + 1 as int64_t) as size_t,
                );
                tv_blob_set_ret(rettv, blob);
                if len as int64_t - end - 1 as int64_t > 0 as int64_t {
                    memmove(
                        p_0.offset(idx as isize) as *mut ::core::ffi::c_void,
                        p_0.offset(end as isize)
                            .offset(1 as ::core::ffi::c_int as isize)
                            as *const ::core::ffi::c_void,
                        (len as int64_t - end - 1 as int64_t) as size_t,
                    );
                }
                (*b).bv_ga.ga_len -= (end - idx + 1 as int64_t) as ::core::ffi::c_int;
            }
        }
    }
}

pub unsafe extern "C" fn f_blob2list(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        if tv_check_for_blob_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
            return;
        }
        let blob: *mut blob_T = (*argvars).vval.v_blob;
        let l: *mut list_T = (*rettv).vval.v_list;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < tv_blob_len(blob) {
            tv_list_append_number(l, tv_blob_get(blob, i) as varnumber_T);
            i += 1;
        }
    }
}

pub unsafe extern "C" fn f_list2blob(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut blob: *mut blob_T = tv_blob_alloc_ret(rettv);
        if tv_check_for_list_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
            return;
        }
        let l: *mut list_T = (*argvars).vval.v_list;
        if l.is_null() {
            return;
        }
        let l_: *const list_T = l;
        if !l_.is_null() {
            let mut li: *const listitem_T = (*l_).lv_first;
            while !li.is_null() {
                let mut error: bool = false;
                let mut n: varnumber_T = tv_get_number_chk(&raw const (*li).li_tv, &raw mut error);
                if error as ::core::ffi::c_int != 0
                    || n < 0 as varnumber_T
                    || n > 255 as varnumber_T
                {
                    if !error {
                        semsg(
                            gettext(
                                &raw const e_invalid_value_for_blob_nr
                                    as *const ::core::ffi::c_char,
                            ),
                            n as ::core::ffi::c_int,
                        );
                    }
                    ga_clear(&raw mut (*blob).bv_ga);
                    return;
                }
                ga_append(&raw mut (*blob).bv_ga, n as uint8_t);
                li = (*li).li_next;
            }
        }
    }
}

pub unsafe extern "C" fn tv_blob_alloc_ret(ret_tv: *mut typval_T) -> *mut blob_T {
    unsafe {
        let b: *mut blob_T = tv_blob_alloc();
        tv_blob_set_ret(ret_tv, b);
        return b;
    }
}

pub unsafe extern "C" fn tv_blob_copy(from: *mut blob_T, to: *mut typval_T) {
    unsafe {
        (*to).v_type = VAR_BLOB;
        (*to).v_lock = VAR_UNLOCKED;
        if from.is_null() {
            (*to).vval.v_blob = ::core::ptr::null_mut::<blob_T>();
        } else {
            tv_blob_alloc_ret(to);
            let mut len: ::core::ffi::c_int = (*from).bv_ga.ga_len;
            if len > 0 as ::core::ffi::c_int {
                (*(*to).vval.v_blob).bv_ga.ga_data = xmemdup((*from).bv_ga.ga_data, len as size_t);
            }
            (*(*to).vval.v_blob).bv_ga.ga_len = len;
            (*(*to).vval.v_blob).bv_ga.ga_maxlen = len;
        };
    }
}
