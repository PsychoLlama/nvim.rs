//! Joining two containers, and inserting into one -- `extend()`,
//! `extendnew()` and `insert()`.
//!
//! `extend` is the shared body of `extend()`/`extendnew()`: for Lists it splices
//! the second list in at an index, for Dicts it merges keys under a
//! `"keep"`/`"force"`/`"error"` policy.  `f_insert` is the single-item form, and
//! it takes the same care as the List walk in [`super::filtermap`] does about an
//! index counted from the end.
//!
//! Original: `src/nvim/eval/list.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{TV_TRANSLATE, false_0, true_0};
use crate::semsg_c;
use crate::src::nvim::eval::get_copyID;
use crate::src::nvim::eval::typval::{tv_blob_len, tv_list_len, tv_list_locked};
use crate::src::nvim::eval::typval::{
    tv_copy, tv_dict_copy, tv_dict_extend, tv_dict_unref, tv_get_number_chk, tv_get_string,
    tv_get_string_chk, tv_list_copy, tv_list_extend, tv_list_find, tv_list_insert_tv,
    tv_list_unref, value_check_lock,
};
use crate::src::nvim::garray::ga_grow;
use crate::src::nvim::main::{
    e_invarg2, e_list_index_out_of_range_nr, e_listblobarg, e_listdictarg,
};
use crate::src::nvim::os::libc::{memmove, strcmp};
use crate::src::nvim::types::{
    EvalFuncData, VAR_BLOB, VAR_DICT, VAR_FIXED, VAR_LIST, VAR_UNKNOWN, VAR_UNLOCKED, blob_T,
    dict_T, int64_t, list_T, listitem_T, size_t, typval_T, typval_vval_union, uint8_t, vimconv_T,
};

unsafe extern "C" fn extend_dict(
    mut argvars: *mut typval_T,
    mut arg_errmsg: *const ::core::ffi::c_char,
    mut is_new: bool,
    mut rettv: *mut typval_T,
) {
    unsafe {
        let mut d1: *mut dict_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        if d1.is_null() {
            let locked: bool = value_check_lock(VAR_FIXED, arg_errmsg, TV_TRANSLATE as size_t);
            debug_assert!(
                locked as ::core::ffi::c_int == 1 as ::core::ffi::c_int,
                "locked == true"
            );
            return;
        }
        let d2: *mut dict_T = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        if d2.is_null() {
            tv_copy(argvars.offset(0 as ::core::ffi::c_int as isize), rettv);
            return;
        }
        if !is_new
            && value_check_lock((*d1).dv_lock, arg_errmsg, TV_TRANSLATE as size_t)
                as ::core::ffi::c_int
                != 0
        {
            return;
        }
        if is_new {
            d1 = tv_dict_copy(
                ::core::ptr::null::<vimconv_T>(),
                d1,
                false_0 != 0,
                get_copyID(),
            );
            if d1.is_null() {
                return;
            }
        }
        let mut action: *const ::core::ffi::c_char = c"force".as_ptr();
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let av: [*const ::core::ffi::c_char; 3] =
                [c"keep".as_ptr(), c"force".as_ptr(), c"error".as_ptr()];
            action = tv_get_string_chk(argvars.offset(2 as ::core::ffi::c_int as isize));
            if action.is_null() {
                if is_new {
                    tv_dict_unref(d1);
                }
                return;
            }
            let mut i: size_t = 0;
            i = 0 as size_t;
            while i < ::core::mem::size_of::<[*const ::core::ffi::c_char; 3]>()
                .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[*const ::core::ffi::c_char; 3]>()
                        .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as usize,
                )
            {
                if strcmp(action, av[i as usize]) == 0 as ::core::ffi::c_int {
                    break;
                }
                i = i.wrapping_add(1);
            }
            if i == 3 as size_t {
                if is_new {
                    tv_dict_unref(d1);
                }
                semsg_c!(
                    &raw const e_invarg2 as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    action,
                );
                return;
            }
        }
        tv_dict_extend(d1, d2, action);
        if is_new {
            *rettv = typval_T {
                v_type: VAR_DICT,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_dict: d1 },
            };
        } else {
            tv_copy(argvars.offset(0 as ::core::ffi::c_int as isize), rettv);
        };
    }
}

unsafe extern "C" fn extend_list(
    mut argvars: *mut typval_T,
    mut arg_errmsg: *const ::core::ffi::c_char,
    mut is_new: bool,
    mut rettv: *mut typval_T,
) {
    unsafe {
        let mut error: bool = false_0 != 0;
        let mut l1: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
        let l2: *mut list_T = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
        if !is_new
            && value_check_lock(tv_list_locked(l1), arg_errmsg, TV_TRANSLATE as size_t)
                as ::core::ffi::c_int
                != 0
        {
            return;
        }
        if is_new {
            l1 = tv_list_copy(
                ::core::ptr::null::<vimconv_T>(),
                l1,
                false_0 != 0,
                get_copyID(),
            );
            if l1.is_null() {
                return;
            }
        }
        let mut item: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
        's_92: {
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut before: ::core::ffi::c_int = tv_get_number_chk(
                    argvars.offset(2 as ::core::ffi::c_int as isize),
                    &raw mut error,
                ) as ::core::ffi::c_int;
                if !error {
                    if before == tv_list_len(l1) {
                        item = ::core::ptr::null_mut::<listitem_T>();
                        break 's_92;
                    } else {
                        item = tv_list_find(l1, before);
                        if item.is_null() {
                            semsg_c!(
                                &raw const e_list_index_out_of_range_nr
                                    as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                before as int64_t,
                            );
                        } else {
                            break 's_92;
                        }
                    }
                }
                if is_new {
                    tv_list_unref(l1);
                }
                return;
            } else {
                item = ::core::ptr::null_mut::<listitem_T>();
            }
        }
        tv_list_extend(l1, l2, item);
        if is_new {
            *rettv = typval_T {
                v_type: VAR_LIST,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_list: l1 },
            };
        } else {
            tv_copy(argvars.offset(0 as ::core::ffi::c_int as isize), rettv);
        };
    }
}

unsafe extern "C" fn extend(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut arg_errmsg: *mut ::core::ffi::c_char,
    mut is_new: bool,
) {
    unsafe {
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            extend_list(argvars, arg_errmsg, is_new, rettv);
        } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            extend_dict(argvars, arg_errmsg, is_new, rettv);
        } else {
            semsg_c!(
                &raw const e_listdictarg as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                if is_new as ::core::ffi::c_int != 0 {
                    c"extendnew()".as_ptr()
                } else {
                    c"extend()".as_ptr()
                },
            );
        };
    }
}

pub unsafe extern "C" fn f_extend(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut errmsg: *mut ::core::ffi::c_char =
            c"extend() argument".as_ptr() as *mut ::core::ffi::c_char;
        extend(argvars, rettv, errmsg, false_0 != 0);
    }
}

pub unsafe extern "C" fn f_extendnew(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut errmsg: *mut ::core::ffi::c_char =
            c"extendnew() argument".as_ptr() as *mut ::core::ffi::c_char;
        extend(argvars, rettv, errmsg, true_0 != 0);
    }
}

pub unsafe extern "C" fn f_insert(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut error: bool = false_0 != 0;
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let b: *mut blob_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_blob;
            if b.is_null()
                || value_check_lock(
                    (*b).bv_lock,
                    c"insert() argument".as_ptr(),
                    TV_TRANSLATE as size_t,
                ) as ::core::ffi::c_int
                    != 0
            {
                return;
            }
            let mut before: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let len: ::core::ffi::c_int = tv_blob_len(b);
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                before = tv_get_number_chk(
                    argvars.offset(2 as ::core::ffi::c_int as isize),
                    &raw mut error,
                ) as ::core::ffi::c_int;
                if error {
                    return;
                }
                if before < 0 as ::core::ffi::c_int || before > len {
                    semsg_c!(
                        &raw const e_invarg2 as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        tv_get_string(argvars.offset(2 as ::core::ffi::c_int as isize)),
                    );
                    return;
                }
            }
            let val: ::core::ffi::c_int = tv_get_number_chk(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as ::core::ffi::c_int;
            if error {
                return;
            }
            if val < 0 as ::core::ffi::c_int || val > 255 as ::core::ffi::c_int {
                semsg_c!(
                    &raw const e_invarg2 as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
                );
                return;
            }
            ga_grow(&raw mut (*b).bv_ga, 1 as ::core::ffi::c_int);
            let p: *mut uint8_t = (*b).bv_ga.ga_data as *mut uint8_t;
            memmove(
                p.offset(before as isize)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_void,
                p.offset(before as isize) as *const ::core::ffi::c_void,
                (len - before) as size_t,
            );
            *p.offset(before as isize) = val as uint8_t;
            (*b).bv_ga.ga_len += 1;
            tv_copy(argvars.offset(0 as ::core::ffi::c_int as isize), rettv);
        } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            semsg_c!(
                &raw const e_listblobarg as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                c"insert()".as_ptr(),
            );
        } else {
            let mut l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_list;
            if value_check_lock(
                tv_list_locked(l),
                c"insert() argument".as_ptr(),
                TV_TRANSLATE as size_t,
            ) {
                return;
            }
            let mut before_0: int64_t = 0 as int64_t;
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                before_0 = tv_get_number_chk(
                    argvars.offset(2 as ::core::ffi::c_int as isize),
                    &raw mut error,
                ) as int64_t;
            }
            if error {
                return;
            }
            let mut item: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
            if before_0 != tv_list_len(l) as int64_t {
                item = tv_list_find(l, before_0 as ::core::ffi::c_int);
                if item.is_null() {
                    semsg_c!(
                        &raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        before_0,
                    );
                    l = ::core::ptr::null_mut::<list_T>();
                }
            }
            if !l.is_null() {
                tv_list_insert_tv(l, argvars.offset(1 as ::core::ffi::c_int as isize), item);
                tv_copy(argvars.offset(0 as ::core::ffi::c_int as isize), rettv);
            }
        };
    }
}
