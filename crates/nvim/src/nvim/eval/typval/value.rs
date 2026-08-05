//! Whole-`typval_T` operations: clear, copy, compare, lock.
//!
//! [`tv_clear`] releases whatever a value holds and leaves `VAR_UNKNOWN`
//! behind; it hands a self-referencing container to the deep-free walk in
//! [`super::nothing`] rather than recursing.  [`tv_copy`] is the shallow
//! copy, [`tv_equal`] the recursion-limited structural comparison, and
//! [`tv_item_lock`] is `:lockvar`, which walks into containers to the depth
//! it is given.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_clear(tv: *mut typval_T) {
    unsafe {
        if tv.is_null()
            || (*tv).v_type as ::core::ffi::c_uint
                == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return;
        }
        let evn_ret: ::core::ffi::c_int = encode_vim_to_nothing(
            ::core::ptr::null::<::core::ffi::c_void>(),
            tv,
            b"tv_clear() argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
        '_c2rust_label: {
            if evn_ret == 1 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"evn_ret == OK\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3663 as ::core::ffi::c_uint,
                    b"void tv_clear(typval_T *const)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
    }
}

pub unsafe extern "C" fn tv_free(mut tv: *mut typval_T) {
    unsafe {
        if tv.is_null() {
            return;
        }
        's_68: {
            match (*tv).v_type as ::core::ffi::c_uint {
                9 => {
                    partial_unref((*tv).vval.v_partial);
                    break 's_68;
                }
                3 => {
                    func_unref((*tv).vval.v_string);
                }
                2 => {}
                10 => {
                    tv_blob_unref((*tv).vval.v_blob);
                    break 's_68;
                }
                4 => {
                    tv_list_unref((*tv).vval.v_list);
                    break 's_68;
                }
                5 => {
                    tv_dict_unref((*tv).vval.v_dict);
                    break 's_68;
                }
                7 | 8 | 1 | 6 | 0 | _ => {
                    break 's_68;
                }
            }
            xfree((*tv).vval.v_string as *mut ::core::ffi::c_void);
        }
        xfree(tv as *mut ::core::ffi::c_void);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_copy(from: *const typval_T, to: *mut typval_T) {
    unsafe {
        (*to).v_type = (*from).v_type;
        (*to).v_lock = VAR_UNLOCKED;
        memmove(
            &raw mut (*to).vval as *mut ::core::ffi::c_void,
            &raw const (*from).vval as *const ::core::ffi::c_void,
            ::core::mem::size_of::<typval_vval_union>(),
        );
        match (*from).v_type as ::core::ffi::c_uint {
            2 | 3 => {
                if !(*from).vval.v_string.is_null() {
                    (*to).vval.v_string = xstrdup((*from).vval.v_string);
                    if (*from).v_type as ::core::ffi::c_uint
                        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        func_ref((*to).vval.v_string);
                    }
                }
            }
            9 => {
                if !(*to).vval.v_partial.is_null() {
                    (*(*to).vval.v_partial).pt_refcount += 1;
                }
            }
            10 => {
                if !(*from).vval.v_blob.is_null() {
                    (*(*to).vval.v_blob).bv_refcount += 1;
                }
            }
            4 => {
                tv_list_ref((*to).vval.v_list);
            }
            5 => {
                if !(*from).vval.v_dict.is_null() {
                    (*(*to).vval.v_dict).dv_refcount += 1;
                }
            }
            0 => {
                semsg(
                    gettext(&raw const e_intern2 as *const ::core::ffi::c_char),
                    b"tv_copy(UNKNOWN)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            1 | 6 | 7 | 8 | _ => {}
        };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_item_lock(
    tv: *mut typval_T,
    deep: ::core::ffi::c_int,
    lock: bool,
    check_refcount: bool,
) {
    unsafe {
        static recurse: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        if recurse.get() >= DICT_MAXNEST {
            emsg(gettext(
                (e_variable_nested_too_deep_for_unlock.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            ));
            return;
        }
        if deep == 0 as ::core::ffi::c_int {
            return;
        }
        (*recurse.ptr()) += 1;
        (*tv).v_lock = [
            (if lock as ::core::ffi::c_int != 0 {
                VAR_LOCKED as ::core::ffi::c_int
            } else {
                VAR_UNLOCKED as ::core::ffi::c_int
            }) as VarLockStatus,
            (if lock as ::core::ffi::c_int != 0 {
                VAR_LOCKED as ::core::ffi::c_int
            } else {
                VAR_UNLOCKED as ::core::ffi::c_int
            }) as VarLockStatus,
            VAR_FIXED,
        ][(*tv).v_lock as usize];
        match (*tv).v_type as ::core::ffi::c_uint {
            10 => {
                let b: *mut blob_T = (*tv).vval.v_blob;
                if !b.is_null()
                    && !(check_refcount as ::core::ffi::c_int != 0
                        && (*b).bv_refcount > 1 as ::core::ffi::c_int)
                {
                    (*b).bv_lock = [
                        (if lock as ::core::ffi::c_int != 0 {
                            VAR_LOCKED as ::core::ffi::c_int
                        } else {
                            VAR_UNLOCKED as ::core::ffi::c_int
                        }) as VarLockStatus,
                        (if lock as ::core::ffi::c_int != 0 {
                            VAR_LOCKED as ::core::ffi::c_int
                        } else {
                            VAR_UNLOCKED as ::core::ffi::c_int
                        }) as VarLockStatus,
                        VAR_FIXED,
                    ][(*b).bv_lock as usize];
                }
            }
            4 => {
                let l: *mut list_T = (*tv).vval.v_list;
                if !l.is_null()
                    && !(check_refcount as ::core::ffi::c_int != 0
                        && (*l).lv_refcount > 1 as ::core::ffi::c_int)
                {
                    (*l).lv_lock = [
                        (if lock as ::core::ffi::c_int != 0 {
                            VAR_LOCKED as ::core::ffi::c_int
                        } else {
                            VAR_UNLOCKED as ::core::ffi::c_int
                        }) as VarLockStatus,
                        (if lock as ::core::ffi::c_int != 0 {
                            VAR_LOCKED as ::core::ffi::c_int
                        } else {
                            VAR_UNLOCKED as ::core::ffi::c_int
                        }) as VarLockStatus,
                        VAR_FIXED,
                    ][(*l).lv_lock as usize];
                    if deep < 0 as ::core::ffi::c_int || deep > 1 as ::core::ffi::c_int {
                        let l_: *mut list_T = l;
                        if !l_.is_null() {
                            let mut li: *mut listitem_T = (*l_).lv_first;
                            while !li.is_null() {
                                tv_item_lock(
                                    &raw mut (*li).li_tv,
                                    deep - 1 as ::core::ffi::c_int,
                                    lock,
                                    check_refcount,
                                );
                                li = (*li).li_next;
                            }
                        }
                    }
                }
            }
            5 => {
                let d: *mut dict_T = (*tv).vval.v_dict;
                if !d.is_null()
                    && !(check_refcount as ::core::ffi::c_int != 0
                        && (*d).dv_refcount > 1 as ::core::ffi::c_int)
                {
                    (*d).dv_lock = [
                        (if lock as ::core::ffi::c_int != 0 {
                            VAR_LOCKED as ::core::ffi::c_int
                        } else {
                            VAR_UNLOCKED as ::core::ffi::c_int
                        }) as VarLockStatus,
                        (if lock as ::core::ffi::c_int != 0 {
                            VAR_LOCKED as ::core::ffi::c_int
                        } else {
                            VAR_UNLOCKED as ::core::ffi::c_int
                        }) as VarLockStatus,
                        VAR_FIXED,
                    ][(*d).dv_lock as usize];
                    if deep < 0 as ::core::ffi::c_int || deep > 1 as ::core::ffi::c_int {
                        let dihi_ht_: *mut hashtab_T = &raw mut (*d).dv_hashtab;
                        let mut dihi_todo_: size_t = (*dihi_ht_).ht_used;
                        let mut dihi_: *mut hashitem_T = (*dihi_ht_).ht_array;
                        while dihi_todo_ != 0 {
                            if !((*dihi_).hi_key.is_null()
                                || (*dihi_).hi_key
                                    == &raw const hash_removed as *mut ::core::ffi::c_char)
                            {
                                dihi_todo_ = dihi_todo_.wrapping_sub(1);
                                let di: *mut dictitem_T = (*dihi_)
                                    .hi_key
                                    .offset(-(17 as ::core::ffi::c_ulong as isize))
                                    as *mut dictitem_T;
                                tv_item_lock(
                                    &raw mut (*di).di_tv,
                                    deep - 1 as ::core::ffi::c_int,
                                    lock,
                                    check_refcount,
                                );
                            }
                            dihi_ = dihi_.offset(1);
                        }
                    }
                }
            }
            0 => {
                abort();
            }
            1 | 6 | 2 | 3 | 9 | 7 | 8 | _ => {}
        }
        (*recurse.ptr()) -= 1;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_islocked(tv: *const typval_T) -> bool {
    unsafe {
        return (*tv).v_lock as ::core::ffi::c_uint
            == VAR_LOCKED as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*tv).v_type as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                && tv_list_locked((*tv).vval.v_list) as ::core::ffi::c_uint
                    == VAR_LOCKED as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*tv).v_type as ::core::ffi::c_uint
                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                && !(*tv).vval.v_dict.is_null()
                && (*(*tv).vval.v_dict).dv_lock as ::core::ffi::c_uint
                    == VAR_LOCKED as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_check_lock(
    mut tv: *const typval_T,
    mut name: *const ::core::ffi::c_char,
    mut name_len: size_t,
) -> bool {
    unsafe {
        let mut lock: VarLockStatus = VAR_UNLOCKED;
        match (*tv).v_type as ::core::ffi::c_uint {
            10 => {
                if !(*tv).vval.v_blob.is_null() {
                    lock = (*(*tv).vval.v_blob).bv_lock;
                }
            }
            4 => {
                if !(*tv).vval.v_list.is_null() {
                    lock = (*(*tv).vval.v_list).lv_lock;
                }
            }
            5 => {
                if !(*tv).vval.v_dict.is_null() {
                    lock = (*(*tv).vval.v_dict).dv_lock;
                }
            }
            _ => {}
        }
        return value_check_lock((*tv).v_lock, name, name_len) as ::core::ffi::c_int != 0
            || lock as ::core::ffi::c_uint
                != VAR_UNLOCKED as ::core::ffi::c_int as ::core::ffi::c_uint
                && value_check_lock(lock, name, name_len) as ::core::ffi::c_int != 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn value_check_lock(
    mut lock: VarLockStatus,
    mut name: *const ::core::ffi::c_char,
    mut name_len: size_t,
) -> bool {
    unsafe {
        let mut error_message: *const ::core::ffi::c_char =
            ::core::ptr::null::<::core::ffi::c_char>();
        match lock as ::core::ffi::c_uint {
            0 => return false_0 != 0,
            1 => {
                error_message = if name.is_null() {
                    &raw const e_value_is_locked as *const ::core::ffi::c_char
                } else {
                    &raw const e_value_is_locked_str as *const ::core::ffi::c_char
                };
            }
            2 => {
                error_message = if name.is_null() {
                    &raw const e_cannot_change_value as *const ::core::ffi::c_char
                } else {
                    &raw const e_cannot_change_value_of_str as *const ::core::ffi::c_char
                };
            }
            _ => {}
        }
        '_c2rust_label: {
            if !error_message.is_null() {
            } else {
                __assert_fail(
                    b"error_message != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3926 as ::core::ffi::c_uint,
                    b"_Bool value_check_lock(VarLockStatus, const char *, size_t)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if name.is_null() {
            emsg(gettext(error_message));
        } else {
            if name_len == TV_TRANSLATE as size_t {
                name = gettext(name);
                name_len = strlen(name);
            } else if name_len == TV_CSTRING as size_t {
                name_len = strlen(name);
            }
            semsg(gettext(error_message), name_len as ::core::ffi::c_int, name);
        }
        return true_0 != 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_equal(tv1: *mut typval_T, tv2: *mut typval_T, ic: bool) -> bool {
    unsafe {
        static recursive_cnt: GlobalCell<::core::ffi::c_int> =
            GlobalCell::new(0 as ::core::ffi::c_int);
        if !(tv_is_func(*tv1) as ::core::ffi::c_int != 0
            && tv_is_func(*tv2) as ::core::ffi::c_int != 0)
            && (*tv1).v_type as ::core::ffi::c_uint != (*tv2).v_type as ::core::ffi::c_uint
        {
            return false_0 != 0;
        }
        if recursive_cnt.get() == 0 as ::core::ffi::c_int {
            tv_equal_recurse_limit.set(1000 as ::core::ffi::c_int);
        }
        if recursive_cnt.get() >= tv_equal_recurse_limit.get() {
            (*tv_equal_recurse_limit.ptr()) -= 1;
            return true_0 != 0;
        }
        match (*tv1).v_type as ::core::ffi::c_uint {
            4 => {
                (*recursive_cnt.ptr()) += 1;
                let r: bool = tv_list_equal((*tv1).vval.v_list, (*tv2).vval.v_list, ic);
                (*recursive_cnt.ptr()) -= 1;
                return r;
            }
            5 => {
                (*recursive_cnt.ptr()) += 1;
                let r_0: bool = tv_dict_equal((*tv1).vval.v_dict, (*tv2).vval.v_dict, ic);
                (*recursive_cnt.ptr()) -= 1;
                return r_0;
            }
            9 | 3 => {
                if (*tv1).v_type as ::core::ffi::c_uint
                    == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
                    && (*tv1).vval.v_partial.is_null()
                    || (*tv2).v_type as ::core::ffi::c_uint
                        == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
                        && (*tv2).vval.v_partial.is_null()
                {
                    return false_0 != 0;
                }
                (*recursive_cnt.ptr()) += 1;
                let r_1: bool = func_equal(tv1, tv2, ic);
                (*recursive_cnt.ptr()) -= 1;
                return r_1;
            }
            10 => return tv_blob_equal((*tv1).vval.v_blob, (*tv2).vval.v_blob),
            1 => return (*tv1).vval.v_number == (*tv2).vval.v_number,
            6 => return (*tv1).vval.v_float == (*tv2).vval.v_float,
            2 => {
                let mut buf1: [::core::ffi::c_char; 65] = [0; 65];
                let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
                let mut s1: *const ::core::ffi::c_char =
                    tv_get_string_buf(tv1, &raw mut buf1 as *mut ::core::ffi::c_char);
                let mut s2: *const ::core::ffi::c_char =
                    tv_get_string_buf(tv2, &raw mut buf2 as *mut ::core::ffi::c_char);
                return mb_strcmp_ic(ic, s1, s2) == 0 as ::core::ffi::c_int;
            }
            7 => {
                return (*tv1).vval.v_bool as ::core::ffi::c_uint
                    == (*tv2).vval.v_bool as ::core::ffi::c_uint;
            }
            8 => {
                return (*tv1).vval.v_special as ::core::ffi::c_uint
                    == (*tv2).vval.v_special as ::core::ffi::c_uint;
            }
            0 => return false_0 != 0,
            _ => {}
        }
        abort();
    }
}
