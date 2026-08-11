//! The four per-container walks behind `filter()`/`map()`/`mapnew()`/
//! `foreach()`.
//!
//! A Dict walks its hashtab with the table locked so a callback cannot rehash
//! it; a List walks its items keeping the next pointer *before* the callback
//! runs, because the callback may remove the current one; a Blob walks bytes
//! and rewrites them in place, compacting when `filter()` drops one; a String
//! walks characters and rebuilds the result in a `garray_T`.  Each is entered
//! from `filter_map` and calls back into `filter_map_one`.
//!
//! Original: `src/nvim/eval/list.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::super::{
    FAIL, FILTERMAP_FILTER, FILTERMAP_FOREACH, FILTERMAP_MAP, FILTERMAP_MAPNEW, NUL, TV_TRANSLATE,
    filtermap_T,
};
use super::filter_map_one;
use crate::src::nvim::eval::typval::{
    tv_blob_copy, tv_clear, tv_dict_add_tv, tv_dict_alloc_ret, tv_dict_item_remove,
    tv_list_alloc_ret, tv_list_append_owned_tv, tv_list_item_remove, value_check_lock,
};
use crate::src::nvim::eval::typval::{
    tv_blob_get, tv_blob_set, tv_list_first, tv_list_locked, tv_list_set_lock,
};
use crate::src::nvim::eval::vars::{
    get_vim_var_tv, set_vim_var_nr, set_vim_var_string, set_vim_var_type, var_check_fixed,
    var_check_ro,
};
use crate::src::nvim::garray::{ga_append, ga_concat, ga_init};
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::hashtab::{hash_lock, hash_unlock};
use crate::src::nvim::main::{did_emsg, e_invalblob, e_string_required};
use crate::src::nvim::mbyte::utfc_ptr2len;
use crate::src::nvim::memory::xmemdupz;
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::{memmove, strlen};
use crate::src::nvim::types::{
    VAR_BLOB, VAR_BOOL, VAR_DICT, VAR_LIST, VAR_LOCKED, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN,
    VAR_UNLOCKED, VV_KEY, VarLockStatus, blob_T, dict_T, dictitem_T, garray_T, hashitem_T,
    hashtab_T, kListLenUnknown, list_T, listitem_T, ptrdiff_t, size_t, typval_T, typval_vval_union,
    uint8_t, varnumber_T,
};

pub(crate) unsafe extern "C" fn filter_map_dict(
    mut d: *mut dict_T,
    mut filtermap: filtermap_T,
    mut _func_name: *const ::core::ffi::c_char,
    mut arg_errmsg: *const ::core::ffi::c_char,
    mut expr: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    unsafe {
        if filtermap as ::core::ffi::c_uint
            == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*rettv).v_type = VAR_DICT;
            (*rettv).vval.v_dict = ::core::ptr::null_mut::<dict_T>();
        }
        if d.is_null()
            || filtermap as ::core::ffi::c_uint
                == FILTERMAP_FILTER as ::core::ffi::c_int as ::core::ffi::c_uint
                && value_check_lock((*d).dv_lock, arg_errmsg, TV_TRANSLATE as size_t)
                    as ::core::ffi::c_int
                    != 0
        {
            return;
        }
        let mut d_ret: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
        if filtermap as ::core::ffi::c_uint
            == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_dict_alloc_ret(rettv);
            d_ret = (*rettv).vval.v_dict;
        }
        let prev_lock: VarLockStatus = (*d).dv_lock;
        if (*d).dv_lock as ::core::ffi::c_uint
            == VAR_UNLOCKED as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*d).dv_lock = VAR_LOCKED;
        }
        hash_lock(&raw mut (*d).dv_hashtab);
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
                if filtermap as ::core::ffi::c_uint
                    == FILTERMAP_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
                    && (value_check_lock(
                        (*di).di_tv.v_lock,
                        arg_errmsg,
                        18446744073709551615 as size_t,
                    ) as ::core::ffi::c_int
                        != 0
                        || var_check_ro(
                            (*di).di_flags as ::core::ffi::c_int,
                            arg_errmsg,
                            18446744073709551615 as size_t,
                        ) as ::core::ffi::c_int
                            != 0)
                {
                    break;
                }
                set_vim_var_string(
                    VV_KEY,
                    &raw mut (*di).di_key as *mut ::core::ffi::c_char,
                    -1 as ptrdiff_t,
                );
                let mut newtv: typval_T = typval_T {
                    v_type: VAR_UNKNOWN,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union { v_number: 0 },
                };
                let mut rem: bool = false;
                let mut r: ::core::ffi::c_int = filter_map_one(
                    &raw mut (*di).di_tv,
                    expr,
                    filtermap,
                    &raw mut newtv,
                    &raw mut rem,
                );
                tv_clear(get_vim_var_tv(VV_KEY));
                if r == 0 as ::core::ffi::c_int || did_emsg.get() != 0 {
                    tv_clear(&raw mut newtv);
                    break;
                } else if filtermap as ::core::ffi::c_uint
                    == FILTERMAP_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    tv_clear(&raw mut (*di).di_tv);
                    newtv.v_lock = VAR_UNLOCKED;
                    (*di).di_tv = newtv;
                } else if filtermap as ::core::ffi::c_uint
                    == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    r = tv_dict_add_tv(
                        d_ret,
                        &raw mut (*di).di_key as *mut ::core::ffi::c_char,
                        strlen(&raw mut (*di).di_key as *mut ::core::ffi::c_char),
                        &raw mut newtv,
                    );
                    tv_clear(&raw mut newtv);
                    if r == 0 as ::core::ffi::c_int {
                        break;
                    }
                } else if filtermap as ::core::ffi::c_uint
                    == FILTERMAP_FILTER as ::core::ffi::c_int as ::core::ffi::c_uint
                    && rem as ::core::ffi::c_int != 0
                {
                    if var_check_fixed(
                        (*di).di_flags as ::core::ffi::c_int,
                        arg_errmsg,
                        18446744073709551615 as size_t,
                    ) as ::core::ffi::c_int
                        != 0
                        || var_check_ro(
                            (*di).di_flags as ::core::ffi::c_int,
                            arg_errmsg,
                            18446744073709551615 as size_t,
                        ) as ::core::ffi::c_int
                            != 0
                    {
                        break;
                    }
                    tv_dict_item_remove(d, di);
                }
            }
            dihi_ = dihi_.offset(1);
        }
        hash_unlock(&raw mut (*d).dv_hashtab);
        (*d).dv_lock = prev_lock;
    }
}

pub(crate) unsafe extern "C" fn filter_map_blob(
    mut blob_arg: *mut blob_T,
    mut filtermap: filtermap_T,
    mut expr: *mut typval_T,
    mut arg_errmsg: *const ::core::ffi::c_char,
    mut rettv: *mut typval_T,
) {
    unsafe {
        if filtermap as ::core::ffi::c_uint
            == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*rettv).v_type = VAR_BLOB;
            (*rettv).vval.v_blob = ::core::ptr::null_mut::<blob_T>();
        }
        let mut b: *mut blob_T = blob_arg;
        if b.is_null()
            || filtermap as ::core::ffi::c_uint
                == FILTERMAP_FILTER as ::core::ffi::c_int as ::core::ffi::c_uint
                && value_check_lock((*b).bv_lock, arg_errmsg, TV_TRANSLATE as size_t)
                    as ::core::ffi::c_int
                    != 0
        {
            return;
        }
        let mut b_ret: *mut blob_T = b;
        if filtermap as ::core::ffi::c_uint
            == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_blob_copy(b, rettv);
            b_ret = (*rettv).vval.v_blob;
        }
        set_vim_var_type(VV_KEY, VAR_NUMBER);
        let prev_lock: VarLockStatus = (*b).bv_lock;
        if (*b).bv_lock as ::core::ffi::c_uint == 0 as ::core::ffi::c_uint {
            (*b).bv_lock = VAR_LOCKED;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*b).bv_ga.ga_len {
            let val: varnumber_T = tv_blob_get(b, i) as varnumber_T;
            let mut tv: typval_T = typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: val },
            };
            set_vim_var_nr(VV_KEY, idx as varnumber_T);
            let mut newtv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            let mut rem: bool = false;
            if filter_map_one(&raw mut tv, expr, filtermap, &raw mut newtv, &raw mut rem) == FAIL
                || did_emsg.get() != 0
            {
                break;
            }
            if filtermap as ::core::ffi::c_uint
                != FILTERMAP_FOREACH as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if newtv.v_type as ::core::ffi::c_uint
                    != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                    && newtv.v_type as ::core::ffi::c_uint
                        != VAR_BOOL as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    tv_clear(&raw mut newtv);
                    emsg(
                        &raw const e_invalblob as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    );
                    break;
                } else if filtermap as ::core::ffi::c_uint
                    != FILTERMAP_FILTER as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if newtv.vval.v_number != val {
                        tv_blob_set(b_ret, i, newtv.vval.v_number as uint8_t);
                    }
                } else if rem {
                    let p: *mut ::core::ffi::c_char =
                        (*blob_arg).bv_ga.ga_data as *mut ::core::ffi::c_char;
                    memmove(
                        p.offset(i as isize) as *mut ::core::ffi::c_void,
                        p.offset(i as isize)
                            .offset(1 as ::core::ffi::c_int as isize)
                            as *const ::core::ffi::c_void,
                        ((*b).bv_ga.ga_len - i - 1 as ::core::ffi::c_int) as size_t,
                    );
                    (*b).bv_ga.ga_len -= 1;
                    i -= 1;
                }
            }
            idx += 1;
            i += 1;
        }
        (*b).bv_lock = prev_lock;
    }
}

pub(crate) unsafe extern "C" fn filter_map_string(
    mut str: *const ::core::ffi::c_char,
    mut filtermap: filtermap_T,
    mut expr: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        set_vim_var_type(VV_KEY, VAR_NUMBER);
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
            80 as ::core::ffi::c_int,
        );
        let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut p: *const ::core::ffi::c_char = str;
        while *p as ::core::ffi::c_int != NUL {
            len = utfc_ptr2len(p);
            let mut tv: typval_T = typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union {
                    v_string: xmemdupz(p as *const ::core::ffi::c_void, len as size_t)
                        as *mut ::core::ffi::c_char,
                },
            };
            set_vim_var_nr(VV_KEY, idx as varnumber_T);
            let mut newtv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            let mut rem: bool = false;
            if filter_map_one(&raw mut tv, expr, filtermap, &raw mut newtv, &raw mut rem) == FAIL
                || did_emsg.get() != 0
            {
                tv_clear(&raw mut newtv);
                tv_clear(&raw mut tv);
                break;
            } else {
                if filtermap as ::core::ffi::c_uint
                    == FILTERMAP_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
                    || filtermap as ::core::ffi::c_uint
                        == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if newtv.v_type as ::core::ffi::c_uint
                        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        tv_clear(&raw mut newtv);
                        tv_clear(&raw mut tv);
                        emsg(
                            &raw const e_string_required as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                        );
                        break;
                    } else {
                        ga_concat(&raw mut ga, newtv.vval.v_string);
                    }
                } else if filtermap as ::core::ffi::c_uint
                    == FILTERMAP_FOREACH as ::core::ffi::c_int as ::core::ffi::c_uint
                    || !rem
                {
                    ga_concat(&raw mut ga, tv.vval.v_string);
                }
                tv_clear(&raw mut newtv);
                tv_clear(&raw mut tv);
                idx += 1;
                p = p.offset(len as isize);
            }
        }
        ga_append(&raw mut ga, NUL as uint8_t);
        (*rettv).vval.v_string = ga.ga_data as *mut ::core::ffi::c_char;
    }
}

pub(crate) unsafe extern "C" fn filter_map_list(
    mut l: *mut list_T,
    mut filtermap: filtermap_T,
    mut _func_name: *const ::core::ffi::c_char,
    mut arg_errmsg: *const ::core::ffi::c_char,
    mut expr: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    unsafe {
        if filtermap as ::core::ffi::c_uint
            == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*rettv).v_type = VAR_LIST;
            (*rettv).vval.v_list = ::core::ptr::null_mut::<list_T>();
        }
        if l.is_null()
            || filtermap as ::core::ffi::c_uint
                == FILTERMAP_FILTER as ::core::ffi::c_int as ::core::ffi::c_uint
                && value_check_lock(tv_list_locked(l), arg_errmsg, TV_TRANSLATE as size_t)
                    as ::core::ffi::c_int
                    != 0
        {
            return;
        }
        let mut l_ret: *mut list_T = ::core::ptr::null_mut::<list_T>();
        if filtermap as ::core::ffi::c_uint
            == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
            l_ret = (*rettv).vval.v_list;
        }
        set_vim_var_type(VV_KEY, VAR_NUMBER);
        let prev_lock: VarLockStatus = tv_list_locked(l);
        if tv_list_locked(l) as ::core::ffi::c_uint
            == VAR_UNLOCKED as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_list_set_lock(l, VAR_LOCKED);
        }
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut li: *mut listitem_T = tv_list_first(l);
        while !li.is_null() {
            if filtermap as ::core::ffi::c_uint
                == FILTERMAP_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
                && value_check_lock((*li).li_tv.v_lock, arg_errmsg, TV_TRANSLATE as size_t)
                    as ::core::ffi::c_int
                    != 0
            {
                break;
            }
            set_vim_var_nr(VV_KEY, idx as varnumber_T);
            let mut newtv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            let mut rem: bool = false;
            if filter_map_one(
                &raw mut (*li).li_tv,
                expr,
                filtermap,
                &raw mut newtv,
                &raw mut rem,
            ) == FAIL
            {
                break;
            }
            if did_emsg.get() != 0 {
                tv_clear(&raw mut newtv);
                break;
            } else {
                if filtermap as ::core::ffi::c_uint
                    == FILTERMAP_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    tv_clear(&raw mut (*li).li_tv);
                    newtv.v_lock = VAR_UNLOCKED;
                    (*li).li_tv = newtv;
                } else if filtermap as ::core::ffi::c_uint
                    == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    tv_list_append_owned_tv(l_ret, newtv);
                }
                if filtermap as ::core::ffi::c_uint
                    == FILTERMAP_FILTER as ::core::ffi::c_int as ::core::ffi::c_uint
                    && rem as ::core::ffi::c_int != 0
                {
                    li = tv_list_item_remove(l, li);
                } else {
                    li = (*li).li_next;
                }
                idx += 1;
            }
        }
        tv_list_set_lock(l, prev_lock);
    }
}
