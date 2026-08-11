//! Counting, and the one-item append -- `count()` and `add()`.
//!
//! `f_count` dispatches to `count_string`, `count_list` or `count_dict`; the
//! String form is the interesting one, since it counts *overlapping-free*
//! occurrences of a substring and honours `ic` with `mb_strnicmp`, so it has to
//! step by whole characters rather than bytes.  `f_add` is here because it is
//! the other builtin whose whole job is the container's length.
//!
//! Original: `src/nvim/eval/list.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{NUL, TV_TRANSLATE, e_argument_of_str_must_be_list_string_or_dictionary, false_0};
use crate::semsg_c;
use crate::src::nvim::eval::typval::{
    tv_copy, tv_equal, tv_get_number_chk, tv_get_string_chk, tv_list_append_tv, tv_list_find,
    value_check_lock,
};
use crate::src::nvim::eval::typval::{tv_list_len, tv_list_locked};
use crate::src::nvim::garray::ga_append;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::main::{e_invarg, e_list_index_out_of_range_nr, e_listblobreq};
use crate::src::nvim::mbyte::{mb_strnicmp, utfc_ptr2len};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::{strlen, strstr};
use crate::src::nvim::types::{
    EvalFuncData, VAR_BLOB, VAR_DICT, VAR_LIST, VAR_STRING, VAR_UNKNOWN, blob_T, dict_T,
    dictitem_T, hashitem_T, hashtab_T, int64_t, list_T, listitem_T, size_t, typval_T, uint8_t,
    varnumber_T,
};

pub unsafe extern "C" fn f_add(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = 1 as varnumber_T;
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_list;
            if !value_check_lock(
                tv_list_locked(l),
                c"add() argument".as_ptr(),
                TV_TRANSLATE as size_t,
            ) {
                tv_list_append_tv(l, argvars.offset(1 as ::core::ffi::c_int as isize));
                tv_copy(argvars.offset(0 as ::core::ffi::c_int as isize), rettv);
            }
        } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let b: *mut blob_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_blob;
            if !b.is_null()
                && !value_check_lock(
                    (*b).bv_lock,
                    c"add() argument".as_ptr(),
                    TV_TRANSLATE as size_t,
                )
            {
                let mut error: bool = false_0 != 0;
                let n: varnumber_T = tv_get_number_chk(
                    argvars.offset(1 as ::core::ffi::c_int as isize),
                    &raw mut error,
                );
                if !error {
                    ga_append(&raw mut (*b).bv_ga, n as uint8_t);
                    tv_copy(argvars.offset(0 as ::core::ffi::c_int as isize), rettv);
                }
            }
        } else {
            emsg(
                &raw const e_listblobreq as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
        };
    }
}

unsafe extern "C" fn count_string(
    mut haystack: *const ::core::ffi::c_char,
    mut needle: *const ::core::ffi::c_char,
    mut ic: bool,
) -> varnumber_T {
    unsafe {
        let mut n: varnumber_T = 0 as varnumber_T;
        let mut p: *const ::core::ffi::c_char = haystack;
        if p.is_null() || needle.is_null() || *needle as ::core::ffi::c_int == NUL {
            return 0 as varnumber_T;
        }
        let mut needlelen: size_t = strlen(needle);
        if ic {
            while *p as ::core::ffi::c_int != NUL {
                if mb_strnicmp(p, needle, needlelen) == 0 as ::core::ffi::c_int {
                    n += 1;
                    p = p.add(needlelen);
                } else {
                    p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
                }
            }
        } else {
            let mut next: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
            loop {
                next = strstr(p, needle);
                if next.is_null() {
                    break;
                }
                n += 1;
                p = next.add(needlelen);
            }
        }
        return n;
    }
}

unsafe extern "C" fn count_list(
    mut l: *mut list_T,
    mut needle: *mut typval_T,
    mut idx: int64_t,
    mut ic: bool,
) -> varnumber_T {
    unsafe {
        if tv_list_len(l) == 0 as ::core::ffi::c_int {
            return 0 as varnumber_T;
        }
        let mut li: *mut listitem_T = tv_list_find(l, idx as ::core::ffi::c_int);
        if li.is_null() {
            semsg_c!(
                &raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                idx,
            );
            return 0 as varnumber_T;
        }
        let mut n: varnumber_T = 0 as varnumber_T;
        while !li.is_null() {
            if tv_equal(&raw mut (*li).li_tv, needle, ic) {
                n += 1;
            }
            li = (*li).li_next;
        }
        return n;
    }
}

unsafe extern "C" fn count_dict(
    mut d: *mut dict_T,
    mut needle: *mut typval_T,
    mut ic: bool,
) -> varnumber_T {
    unsafe {
        if d.is_null() {
            return 0 as varnumber_T;
        }
        let mut n: varnumber_T = 0 as varnumber_T;
        let dihi_ht_: *mut hashtab_T = &raw mut (*d).dv_hashtab;
        let mut dihi_todo_: size_t = (*dihi_ht_).ht_used;
        let mut dihi_: *mut hashitem_T = (*dihi_ht_).ht_array;
        while dihi_todo_ != 0 {
            if !((*dihi_).hi_key.is_null()
                || (*dihi_).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                dihi_todo_ = dihi_todo_.wrapping_sub(1);
                let di: *mut dictitem_T = (*dihi_)
                    .hi_key
                    .offset(-(17 as ::core::ffi::c_ulong as isize))
                    as *mut dictitem_T;
                if tv_equal(&raw mut (*di).di_tv, needle, ic) {
                    n += 1;
                }
            }
            dihi_ = dihi_.offset(1);
        }
        return n;
    }
}

pub unsafe extern "C" fn f_count(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut n: varnumber_T = 0 as varnumber_T;
        let mut ic: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut error: bool = false_0 != 0;
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ic = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as ::core::ffi::c_int;
        }
        if !error
            && (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            n = count_string(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_string,
                tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize)),
                ic != 0,
            );
        } else if !error
            && (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut idx: int64_t = 0 as int64_t;
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                idx = tv_get_number_chk(
                    argvars.offset(3 as ::core::ffi::c_int as isize),
                    &raw mut error,
                );
            }
            if !error {
                n = count_list(
                    (*argvars.offset(0 as ::core::ffi::c_int as isize))
                        .vval
                        .v_list,
                    argvars.offset(1 as ::core::ffi::c_int as isize),
                    idx,
                    ic != 0,
                );
            }
        } else if !error
            && (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut d: *mut dict_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_dict;
            if !d.is_null() {
                if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                    && (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type
                        as ::core::ffi::c_uint
                        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    emsg(
                        &raw const e_invarg as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    );
                } else {
                    n = count_dict(
                        (*argvars.offset(0 as ::core::ffi::c_int as isize))
                            .vval
                            .v_dict,
                        argvars.offset(1 as ::core::ffi::c_int as isize),
                        ic != 0,
                    );
                }
            }
        } else if !error {
            semsg_c!(
                e_argument_of_str_must_be_list_string_or_dictionary.as_ptr()
                    as *mut ::core::ffi::c_char,
                c"count()".as_ptr(),
            );
        }
        (*rettv).vval.v_number = n;
    }
}
