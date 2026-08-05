//! `:unlet`, `:lockvar` and `:unlockvar`.
//!
//! All three share `ex_unletlock`'s argument walk and differ only in the
//! callback it is given, so deleting and locking are written here together
//! -- as they are upstream.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe fn ex_unlet(mut eap: *mut exarg_T) {
    unsafe {
        ex_unletlock(
            eap,
            (*eap).arg,
            0 as ::core::ffi::c_int,
            if (*eap).forceit != 0 {
                GLV_QUIET as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
            Some(
                do_unlet_var
                    as unsafe extern "C" fn(
                        *mut lval_T,
                        *mut ::core::ffi::c_char,
                        *mut exarg_T,
                        ::core::ffi::c_int,
                    ) -> ::core::ffi::c_int,
            ),
        );
    }
}

pub unsafe fn ex_lockvar(mut eap: *mut exarg_T) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut deep: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
        if (*eap).forceit != 0 {
            deep = -1 as ::core::ffi::c_int;
        } else if ascii_isdigit(*arg as ::core::ffi::c_int) {
            deep = getdigits_int(&raw mut arg, false_0 != 0, -1 as ::core::ffi::c_int);
            arg = skipwhite(arg);
        }
        ex_unletlock(
            eap,
            arg,
            deep,
            0 as ::core::ffi::c_int,
            Some(
                do_lock_var
                    as unsafe extern "C" fn(
                        *mut lval_T,
                        *mut ::core::ffi::c_char,
                        *mut exarg_T,
                        ::core::ffi::c_int,
                    ) -> ::core::ffi::c_int,
            ),
        );
    }
}

unsafe extern "C" fn ex_unletlock(
    mut eap: *mut exarg_T,
    mut argstart: *mut ::core::ffi::c_char,
    mut deep: ::core::ffi::c_int,
    mut glv_flags: ::core::ffi::c_int,
    mut callback: ex_unletlock_callback,
) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = argstart;
        let mut name_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut error: bool = false_0 != 0;
        let mut lv: lval_T = lval_T {
            ll_name: ::core::ptr::null::<::core::ffi::c_char>(),
            ll_name_len: 0,
            ll_exp_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ll_tv: ::core::ptr::null_mut::<typval_T>(),
            ll_li: ::core::ptr::null_mut::<listitem_T>(),
            ll_list: ::core::ptr::null_mut::<list_T>(),
            ll_range: false,
            ll_empty2: false,
            ll_n1: 0,
            ll_n2: 0,
            ll_dict: ::core::ptr::null_mut::<dict_T>(),
            ll_di: ::core::ptr::null_mut::<dictitem_T>(),
            ll_newkey: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ll_blob: ::core::ptr::null_mut::<blob_T>(),
        };
        loop {
            if *arg as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
                lv.ll_name = arg;
                lv.ll_tv = ::core::ptr::null_mut::<typval_T>();
                arg = arg.offset(1);
                if get_env_len(&raw mut arg as *mut *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        arg.offset(-(1 as ::core::ffi::c_int as isize)),
                    );
                    return;
                }
                '_c2rust_label: {
                    if *lv.ll_name as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                        b"*lv.ll_name == '$'\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/vars.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1570 as ::core::ffi::c_uint,
                        b"void ex_unletlock(exarg_T *, char *, int, int, ex_unletlock_callback)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                    }
                };
                if !error
                    && (*eap).skip == 0
                    && callback.expect("non-null function pointer")(&raw mut lv, arg, eap, deep)
                        == FAIL
                {
                    error = true_0 != 0;
                }
                name_end = arg;
            } else {
                name_end = get_lval(
                    arg,
                    ::core::ptr::null_mut::<typval_T>(),
                    &raw mut lv,
                    true_0 != 0,
                    (*eap).skip != 0 || error as ::core::ffi::c_int != 0,
                    glv_flags,
                    FNE_CHECK_START,
                );
                if lv.ll_name.is_null() {
                    error = true_0 != 0;
                }
                if name_end.is_null()
                    || !ascii_iswhite(*name_end as ::core::ffi::c_int)
                        && ends_excmd(*name_end as ::core::ffi::c_int) == 0
                {
                    if !name_end.is_null() {
                        emsg_severe.set(true_0 != 0);
                        semsg(
                            gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                            name_end,
                        );
                    }
                    if !((*eap).skip != 0 || error as ::core::ffi::c_int != 0) {
                        clear_lval(&raw mut lv);
                    }
                    break;
                } else {
                    if !error
                        && (*eap).skip == 0
                        && callback.expect("non-null function pointer")(
                            &raw mut lv,
                            name_end,
                            eap,
                            deep,
                        ) == FAIL
                    {
                        error = true_0 != 0;
                    }
                    if (*eap).skip == 0 {
                        clear_lval(&raw mut lv);
                    }
                }
            }
            arg = skipwhite(name_end);
            if ends_excmd(*arg as ::core::ffi::c_int) != 0 {
                break;
            }
        }
        (*eap).nextcmd = check_nextcmd(arg);
    }
}

unsafe extern "C" fn do_unlet_var(
    mut lp: *mut lval_T,
    mut name_end: *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
    mut _deep: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut forceit: ::core::ffi::c_int = (*eap).forceit;
        let mut ret: ::core::ffi::c_int = OK;
        if (*lp).ll_tv.is_null() {
            let mut cc: ::core::ffi::c_int = *name_end as uint8_t as ::core::ffi::c_int;
            *name_end = NUL as ::core::ffi::c_char;
            if *(*lp).ll_name as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
                vim_unsetenv_ext((*lp).ll_name.offset(1 as ::core::ffi::c_int as isize));
            } else if do_unlet((*lp).ll_name, (*lp).ll_name_len, forceit != 0) == FAIL {
                ret = FAIL;
            }
            *name_end = cc as ::core::ffi::c_char;
        } else if !(*lp).ll_list.is_null()
            && value_check_lock(
                tv_list_locked((*lp).ll_list),
                (*lp).ll_name,
                (*lp).ll_name_len,
            ) as ::core::ffi::c_int
                != 0
            || !(*lp).ll_dict.is_null()
                && value_check_lock((*(*lp).ll_dict).dv_lock, (*lp).ll_name, (*lp).ll_name_len)
                    as ::core::ffi::c_int
                    != 0
        {
            return FAIL;
        } else if (*lp).ll_range {
            tv_list_unlet_range(
                (*lp).ll_list,
                (*lp).ll_li,
                (*lp).ll_n1,
                !(*lp).ll_empty2,
                (*lp).ll_n2,
            );
        } else if !(*lp).ll_list.is_null() {
            tv_list_item_remove((*lp).ll_list, (*lp).ll_li);
        } else {
            let mut d: *mut dict_T = (*lp).ll_dict;
            '_c2rust_label: {
                if !d.is_null() {
                } else {
                    __assert_fail(
                        b"d != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/vars.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1652 as ::core::ffi::c_uint,
                        b"int do_unlet_var(lval_T *, char *, exarg_T *, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            let mut di: *mut dictitem_T = (*lp).ll_di;
            let mut watched: bool = tv_dict_is_watched(d);
            let mut key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut oldtv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            if watched {
                tv_copy(&raw mut (*di).di_tv, &raw mut oldtv);
                key = xstrdup(&raw mut (*di).di_key as *mut ::core::ffi::c_char);
            }
            tv_dict_item_remove(d, di);
            if watched {
                tv_dict_watcher_notify(d, key, ::core::ptr::null_mut::<typval_T>(), &raw mut oldtv);
                tv_clear(&raw mut oldtv);
                xfree(key as *mut ::core::ffi::c_void);
            }
        }
        return ret;
    }
}

unsafe extern "C" fn tv_list_unlet_range(
    l: *mut list_T,
    li_first: *mut listitem_T,
    n1_arg: ::core::ffi::c_int,
    has_n2: bool,
    n2: ::core::ffi::c_int,
) {
    unsafe {
        '_c2rust_label: {
            if !l.is_null() {
            } else {
                __assert_fail(
                b"l != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/vars.rs\0".as_ptr()
                    as *const ::core::ffi::c_char,
                1681 as ::core::ffi::c_uint,
                b"void tv_list_unlet_range(list_T *const, listitem_T *const, const int, const _Bool, const int)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
            }
        };
        let mut li_last: *mut listitem_T = li_first;
        let mut n1: ::core::ffi::c_int = n1_arg;
        loop {
            let li: *mut listitem_T = (*li_last).li_next;
            n1 += 1;
            if li.is_null() || has_n2 as ::core::ffi::c_int != 0 && n2 < n1 {
                break;
            }
            li_last = li;
        }
        tv_list_remove_items(l, li_first, li_last);
    }
}

pub unsafe extern "C" fn do_unlet(
    name: *const ::core::ffi::c_char,
    name_len: size_t,
    forceit: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut varname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
        let mut ht: *mut hashtab_T =
            find_var_ht_dict(name, name_len, &raw mut varname, &raw mut dict);
        if !ht.is_null() && *varname as ::core::ffi::c_int != NUL {
            let mut d: *mut dict_T = get_current_funccal_dict(ht);
            if d.is_null() {
                if ht == &raw mut (*globvardict.ptr()).dv_hashtab {
                    d = globvardict.ptr();
                } else if ht == compat_hashtab.ptr() {
                    d = vimvardict.ptr();
                } else {
                    let di: *mut dictitem_T = find_var_in_ht(
                        ht,
                        *name as ::core::ffi::c_int,
                        b"\0".as_ptr() as *const ::core::ffi::c_char,
                        0 as size_t,
                        false_0,
                    );
                    d = (*di).di_tv.vval.v_dict;
                }
                if d.is_null() {
                    internal_error(b"do_unlet()\0".as_ptr() as *const ::core::ffi::c_char);
                    return FAIL;
                }
            }
            let mut hi: *mut hashitem_T = hash_find(ht, varname);
            if (*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
            {
                hi = find_hi_in_scoped_ht(name, &raw mut ht);
            }
            if !hi.is_null()
                && !((*hi).hi_key.is_null()
                    || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                let di_0: *mut dictitem_T =
                    (*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize)) as *mut dictitem_T;
                if var_check_fixed(
                    (*di_0).di_flags as ::core::ffi::c_int,
                    name,
                    TV_CSTRING as size_t,
                ) as ::core::ffi::c_int
                    != 0
                    || var_check_ro(
                        (*di_0).di_flags as ::core::ffi::c_int,
                        name,
                        TV_CSTRING as size_t,
                    ) as ::core::ffi::c_int
                        != 0
                    || value_check_lock((*d).dv_lock, name, TV_CSTRING as size_t)
                        as ::core::ffi::c_int
                        != 0
                {
                    return FAIL;
                }
                if value_check_lock((*d).dv_lock, name, TV_CSTRING as size_t) {
                    return FAIL;
                }
                let mut oldtv: typval_T = typval_T {
                    v_type: VAR_UNKNOWN,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union { v_number: 0 },
                };
                let mut watched: bool = tv_dict_is_watched(dict);
                if watched {
                    tv_copy(&raw mut (*di_0).di_tv, &raw mut oldtv);
                }
                delete_var(ht, hi);
                if watched {
                    tv_dict_watcher_notify(
                        dict,
                        varname,
                        ::core::ptr::null_mut::<typval_T>(),
                        &raw mut oldtv,
                    );
                    tv_clear(&raw mut oldtv);
                }
                return OK;
            }
        }
        if forceit {
            return OK;
        }
        semsg(
            gettext(b"E108: No such variable: \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
            name,
        );
        return FAIL;
    }
}

unsafe extern "C" fn do_lock_var(
    mut lp: *mut lval_T,
    mut _name_end: *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
    mut deep: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut lock: bool =
            (*eap).cmdidx as ::core::ffi::c_int == CMD_lockvar as ::core::ffi::c_int;
        let mut ret: ::core::ffi::c_int = OK;
        if (*lp).ll_tv.is_null() {
            if *(*lp).ll_name as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
                semsg(gettext(e_lock_unlock.get()), (*lp).ll_name);
                ret = FAIL;
            } else {
                let di: *mut dictitem_T = find_var(
                    (*lp).ll_name,
                    (*lp).ll_name_len,
                    ::core::ptr::null_mut::<*mut hashtab_T>(),
                    true_0,
                );
                if di.is_null() {
                    ret = FAIL;
                } else if (*di).di_flags as ::core::ffi::c_int & DI_FLAGS_FIX as ::core::ffi::c_int
                    != 0
                    && (*di).di_tv.v_type as ::core::ffi::c_uint
                        != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                    && (*di).di_tv.v_type as ::core::ffi::c_uint
                        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    semsg(gettext(e_lock_unlock.get()), (*lp).ll_name);
                    ret = FAIL;
                } else {
                    if lock {
                        (*di).di_flags = ((*di).di_flags as ::core::ffi::c_int
                            | DI_FLAGS_LOCK as ::core::ffi::c_int)
                            as uint8_t;
                    } else {
                        (*di).di_flags = ((*di).di_flags as ::core::ffi::c_int
                            & !(DI_FLAGS_LOCK as ::core::ffi::c_int) as uint8_t
                                as ::core::ffi::c_int)
                            as uint8_t;
                    }
                    if deep != 0 as ::core::ffi::c_int {
                        tv_item_lock(&raw mut (*di).di_tv, deep, lock, false_0 != 0);
                    }
                }
            }
        } else if deep != 0 as ::core::ffi::c_int {
            if (*lp).ll_range {
                let mut li: *mut listitem_T = (*lp).ll_li;
                while !li.is_null()
                    && ((*lp).ll_empty2 as ::core::ffi::c_int != 0 || (*lp).ll_n2 >= (*lp).ll_n1)
                {
                    tv_item_lock(&raw mut (*li).li_tv, deep, lock, false_0 != 0);
                    li = (*li).li_next;
                    (*lp).ll_n1 += 1;
                }
            } else if !(*lp).ll_list.is_null() {
                tv_item_lock(&raw mut (*(*lp).ll_li).li_tv, deep, lock, false_0 != 0);
            } else {
                tv_item_lock(&raw mut (*(*lp).ll_di).di_tv, deep, lock, false_0 != 0);
            }
        }
        return ret;
    }
}
