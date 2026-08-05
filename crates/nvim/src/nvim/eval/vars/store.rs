//! Assigning to a name, and the four checks that can refuse.
//!
//! `set_var_const` is the single entry point every assignment reaches; the
//! `var_check_*` trio reads `di_flags` and produces E46 / E1122 / E795, and
//! `var_wrong_func_name` and `valid_varname` reject the name itself.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn set_var(
    mut name: *const ::core::ffi::c_char,
    name_len: size_t,
    tv: *mut typval_T,
    copy: bool,
) {
    unsafe {
        set_var_const(name, name_len, tv, copy, false_0 != 0);
    }
}

pub unsafe extern "C" fn set_var_const(
    mut name: *const ::core::ffi::c_char,
    name_len: size_t,
    tv: *mut typval_T,
    copy: bool,
    is_const: bool,
) {
    unsafe {
        let mut varname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
        let mut ht: *mut hashtab_T =
            find_var_ht_dict(name, name_len, &raw mut varname, &raw mut dict);
        let watched: bool = tv_dict_is_watched(dict);
        if ht.is_null() || *varname as ::core::ffi::c_int == NUL {
            semsg(
                gettext(&raw const e_illvar as *const ::core::ffi::c_char),
                name,
            );
            return;
        }
        let varname_len: size_t = name_len.wrapping_sub(varname.offset_from(name) as size_t);
        let mut di: *mut dictitem_T =
            find_var_in_ht(ht, 0 as ::core::ffi::c_int, varname, varname_len, true_0);
        if di.is_null() {
            di = find_var_in_scoped_ht(name, name_len, true_0);
        }
        if tv_is_func(*tv) as ::core::ffi::c_int != 0
            && var_wrong_func_name(name, di.is_null()) as ::core::ffi::c_int != 0
        {
            return;
        }
        let mut oldtv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if !di.is_null() {
            if is_const {
                emsg(gettext(
                    &raw const e_cannot_mod as *const ::core::ffi::c_char,
                ));
                return;
            }
            if var_check_ro((*di).di_flags as ::core::ffi::c_int, name, name_len)
                as ::core::ffi::c_int
                != 0
                || value_check_lock((*di).di_tv.v_lock, name, name_len) as ::core::ffi::c_int != 0
                || var_check_lock((*di).di_flags as ::core::ffi::c_int, name, name_len)
                    as ::core::ffi::c_int
                    != 0
            {
                return;
            }
            let mut type_error: bool = false_0 != 0;
            if ht == &raw mut (*vimvardict.ptr()).dv_hashtab
                && !before_set_vvar(varname, di, tv, copy, watched, &raw mut type_error)
            {
                if type_error {
                    semsg(
                        gettext(
                            (e_setting_v_str_to_value_with_wrong_type.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ),
                        varname,
                    );
                }
                return;
            }
            if watched {
                tv_copy(&raw mut (*di).di_tv, &raw mut oldtv);
            }
            tv_clear(&raw mut (*di).di_tv);
        } else {
            if ht == &raw mut (*vimvardict.ptr()).dv_hashtab || ht == get_funccal_args_ht() {
                semsg(
                    gettext(&raw const e_illvar as *const ::core::ffi::c_char),
                    name,
                );
                return;
            }
            if !valid_varname(varname) {
                return;
            }
            '_c2rust_label: {
                if !dict.is_null() {
                } else {
                    __assert_fail(
                    b"dict != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/vars.rs\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    2883 as ::core::ffi::c_uint,
                    b"void set_var_const(const char *, const size_t, typval_T *const, const _Bool, const _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
                }
            };
            di = xmalloc(
                (17 as size_t)
                    .wrapping_add(varname_len)
                    .wrapping_add(1 as size_t),
            ) as *mut dictitem_T;
            memcpy(
                &raw mut (*di).di_key as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                varname as *const ::core::ffi::c_void,
                varname_len.wrapping_add(1 as size_t),
            );
            if hash_add(ht, &raw mut (*di).di_key as *mut ::core::ffi::c_char) == FAIL {
                xfree(di as *mut ::core::ffi::c_void);
                return;
            }
            (*di).di_flags = DI_FLAGS_ALLOC as ::core::ffi::c_int as uint8_t;
            if is_const {
                (*di).di_flags = ((*di).di_flags as ::core::ffi::c_int
                    | DI_FLAGS_LOCK as ::core::ffi::c_int)
                    as uint8_t;
            }
        }
        if copy as ::core::ffi::c_int != 0
            || (*tv).v_type as ::core::ffi::c_uint
                == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*tv).v_type as ::core::ffi::c_uint
                == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_copy(tv, &raw mut (*di).di_tv);
        } else {
            (*di).di_tv = *tv;
            (*di).di_tv.v_lock = VAR_UNLOCKED;
            tv_init(tv);
        }
        if watched {
            tv_dict_watcher_notify(
                dict,
                &raw mut (*di).di_key as *mut ::core::ffi::c_char,
                &raw mut (*di).di_tv,
                &raw mut oldtv,
            );
            tv_clear(&raw mut oldtv);
        }
        if is_const {
            tv_item_lock(&raw mut (*di).di_tv, DICT_MAXNEST, true_0 != 0, true_0 != 0);
        }
    }
}

pub unsafe extern "C" fn var_check_ro(
    flags: ::core::ffi::c_int,
    mut name: *const ::core::ffi::c_char,
    mut name_len: size_t,
) -> bool {
    unsafe {
        let mut error_message: *const ::core::ffi::c_char =
            ::core::ptr::null::<::core::ffi::c_char>();
        if flags & DI_FLAGS_RO as ::core::ffi::c_int != 0 {
            error_message =
                &raw const e_cannot_change_readonly_variable_str as *const ::core::ffi::c_char;
        } else if flags & DI_FLAGS_RO_SBX as ::core::ffi::c_int != 0 && sandbox.get() != 0 {
            error_message =
                &raw const e_cannot_set_variable_in_sandbox_str as *const ::core::ffi::c_char;
        }
        if error_message.is_null() {
            return false_0 != 0;
        }
        if name_len == TV_TRANSLATE as size_t {
            name = gettext(name);
            name_len = strlen(name);
        } else if name_len == TV_CSTRING as size_t {
            name_len = strlen(name);
        }
        semsg(gettext(error_message), name_len as ::core::ffi::c_int, name);
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn var_check_lock(
    flags: ::core::ffi::c_int,
    mut name: *const ::core::ffi::c_char,
    mut name_len: size_t,
) -> bool {
    unsafe {
        if flags & DI_FLAGS_LOCK as ::core::ffi::c_int == 0 {
            return false_0 != 0;
        }
        if name_len == TV_TRANSLATE as size_t {
            name = gettext(name);
            name_len = strlen(name);
        } else if name_len == TV_CSTRING as size_t {
            name_len = strlen(name);
        }
        semsg(
            gettext(b"E1122: Variable is locked: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            name_len as ::core::ffi::c_int,
            name,
        );
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn var_check_fixed(
    flags: ::core::ffi::c_int,
    mut name: *const ::core::ffi::c_char,
    mut name_len: size_t,
) -> bool {
    unsafe {
        if flags & DI_FLAGS_FIX as ::core::ffi::c_int != 0 {
            if name_len == TV_TRANSLATE as size_t {
                name = gettext(name);
                name_len = strlen(name);
            } else if name_len == TV_CSTRING as size_t {
                name_len = strlen(name);
            }
            semsg(
                gettext(&raw const e_cannot_delete_variable_str as *const ::core::ffi::c_char),
                name_len as ::core::ffi::c_int,
                name,
            );
            return true_0 != 0;
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn var_wrong_func_name(
    name: *const ::core::ffi::c_char,
    new_var: bool,
) -> bool {
    unsafe {
        if !(!vim_strchr(
            b"wbst\0".as_ptr() as *const ::core::ffi::c_char,
            *name.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
            && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ':' as ::core::ffi::c_int)
            && !((if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '\0' as ::core::ffi::c_int
                && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ':' as ::core::ffi::c_int
            {
                *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            }) as ::core::ffi::c_uint
                >= 'A' as ::core::ffi::c_uint
                && (if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '\0' as ::core::ffi::c_int
                    && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ':' as ::core::ffi::c_int
                {
                    *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                } else {
                    *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                }) as ::core::ffi::c_uint
                    <= 'Z' as ::core::ffi::c_uint)
            && vim_strchr(name, '#' as ::core::ffi::c_int).is_null()
        {
            semsg(
                gettext(
                    b"E704: Funcref variable name must start with a capital: %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                name,
            );
            return true_0 != 0;
        }
        if new_var as ::core::ffi::c_int != 0
            && function_exists(name, false_0 != 0) as ::core::ffi::c_int != 0
        {
            semsg(
                gettext(
                    b"E705: Variable name conflicts with existing function: %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                name,
            );
            return true_0 != 0;
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn valid_varname(mut varname: *const ::core::ffi::c_char) -> bool {
    unsafe {
        let mut p: *const ::core::ffi::c_char = varname;
        while *p as ::core::ffi::c_int != NUL {
            if !eval_isnamec1(*p as uint8_t as ::core::ffi::c_int)
                && (p == varname || !ascii_isdigit(*p as ::core::ffi::c_int))
                && *p as ::core::ffi::c_int != AUTOLOAD_CHAR
            {
                semsg(
                    gettext(&raw const e_illvar as *const ::core::ffi::c_char),
                    varname,
                );
                return false_0 != 0;
            }
            p = p.offset(1);
        }
        return true_0 != 0;
    }
}
