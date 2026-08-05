//! Reading and writing `v:` from C.
//!
//! Two families: the `get_vim_var_*` readers, which are how the rest of the
//! editor asks what a `v:` variable holds, and the `set_vim_var_*` writers,
//! which are how it publishes one.  `before_set_vvar` is the Vimscript side
//! of the same thing: the type enforcement `:let v:x = …` goes through.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn prepare_vimvar(mut idx: ::core::ffi::c_int, mut save_tv: *mut typval_T) {
    unsafe {
        *save_tv = (*vimvars.ptr())[idx as usize].vv_di.di_tv;
        (*vimvars.ptr())[idx as usize].vv_di.di_tv.vval.v_string =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        if (*vimvars.ptr())[idx as usize].vv_di.di_tv.v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            hash_add(
                &raw mut (*vimvardict.ptr()).dv_hashtab,
                &raw mut (*(vimvars.ptr() as *mut vimvar).offset(idx as isize))
                    .vv_di
                    .di_key as *mut ::core::ffi::c_char,
            );
        }
    }
}

pub unsafe extern "C" fn restore_vimvar(mut idx: ::core::ffi::c_int, mut save_tv: *mut typval_T) {
    unsafe {
        (*vimvars.ptr())[idx as usize].vv_di.di_tv = *save_tv;
        if (*vimvars.ptr())[idx as usize].vv_di.di_tv.v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return;
        }
        let mut hi: *mut hashitem_T = hash_find(
            &raw mut (*vimvardict.ptr()).dv_hashtab,
            &raw mut (*(vimvars.ptr() as *mut vimvar).offset(idx as isize))
                .vv_di
                .di_key as *mut ::core::ffi::c_char,
        );
        if (*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
        {
            internal_error(b"restore_vimvar()\0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            hash_remove(&raw mut (*vimvardict.ptr()).dv_hashtab, hi);
        };
    }
}

pub unsafe extern "C" fn set_vim_var_tv(idx: VimVarIndex, tv: *mut typval_T) {
    unsafe {
        let mut tv_out: *mut typval_T = get_vim_var_tv(idx);
        tv_clear(tv_out);
        tv_copy(tv, tv_out);
    }
}

pub unsafe extern "C" fn get_vim_var_name(idx: VimVarIndex) -> *mut ::core::ffi::c_char {
    unsafe {
        return (*vimvars.ptr())[idx as usize].vv_name;
    }
}

pub unsafe extern "C" fn get_vim_var_tv(idx: VimVarIndex) -> *mut typval_T {
    unsafe {
        return &raw mut (*(vimvars.ptr() as *mut vimvar).offset(idx as isize))
            .vv_di
            .di_tv;
    }
}

pub unsafe extern "C" fn get_vim_var_nr(idx: VimVarIndex) -> varnumber_T {
    unsafe {
        let mut tv: *mut typval_T = get_vim_var_tv(idx);
        return (*tv).vval.v_number;
    }
}

pub unsafe extern "C" fn get_vim_var_list(idx: VimVarIndex) -> *mut list_T {
    unsafe {
        let mut tv: *mut typval_T = get_vim_var_tv(idx);
        return (*tv).vval.v_list;
    }
}

pub unsafe extern "C" fn get_vim_var_dict(idx: VimVarIndex) -> *mut dict_T {
    unsafe {
        let mut tv: *mut typval_T = get_vim_var_tv(idx);
        return (*tv).vval.v_dict;
    }
}

pub unsafe extern "C" fn get_vim_var_str(idx: VimVarIndex) -> *mut ::core::ffi::c_char {
    unsafe {
        return tv_get_string(get_vim_var_tv(idx)) as *mut ::core::ffi::c_char;
    }
}

pub unsafe extern "C" fn get_vim_var_partial(idx: VimVarIndex) -> *mut partial_T {
    unsafe {
        let mut tv: *mut typval_T = get_vim_var_tv(idx);
        return (*tv).vval.v_partial;
    }
}

pub unsafe extern "C" fn set_vim_var_type(idx: VimVarIndex, type_0: VarType) {
    unsafe {
        let mut tv: *mut typval_T = get_vim_var_tv(idx);
        (*tv).v_type = type_0;
    }
}

pub unsafe extern "C" fn set_vim_var_nr(idx: VimVarIndex, val: varnumber_T) {
    unsafe {
        let mut tv: *mut typval_T = get_vim_var_tv(idx);
        tv_clear(tv);
        (*tv).vval.v_number = val;
    }
}

pub unsafe extern "C" fn set_vim_var_bool(idx: VimVarIndex, val: BoolVarValue) {
    unsafe {
        let mut tv: *mut typval_T = get_vim_var_tv(idx);
        tv_clear(tv);
        (*tv).v_type = VAR_BOOL;
        (*tv).vval.v_bool = val;
    }
}

pub unsafe extern "C" fn set_vim_var_special(idx: VimVarIndex, val: SpecialVarValue) {
    unsafe {
        let mut tv: *mut typval_T = get_vim_var_tv(idx);
        tv_clear(tv);
        (*tv).v_type = VAR_SPECIAL;
        (*tv).vval.v_special = val;
    }
}

pub unsafe extern "C" fn set_vim_var_char(mut c: ::core::ffi::c_int) {
    unsafe {
        let mut buf: [::core::ffi::c_char; 7] = [0; 7];
        let mut buflen: ::core::ffi::c_int =
            utf_char2bytes(c, &raw mut buf as *mut ::core::ffi::c_char);
        buf[buflen as usize] = NUL;
        set_vim_var_string(
            VV_CHAR,
            &raw mut buf as *mut ::core::ffi::c_char,
            buflen as ptrdiff_t,
        );
    }
}

pub unsafe extern "C" fn set_vim_var_string(
    idx: VimVarIndex,
    val: *const ::core::ffi::c_char,
    len: ptrdiff_t,
) {
    unsafe {
        let mut tv: *mut typval_T = get_vim_var_tv(idx);
        tv_clear(tv);
        (*tv).v_type = VAR_STRING;
        if val.is_null() {
            (*tv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else if len == -1 as ptrdiff_t {
            (*tv).vval.v_string = xstrdup(val);
        } else {
            (*tv).vval.v_string = xstrndup(val, len as size_t);
        };
    }
}

pub unsafe extern "C" fn set_vim_var_list(idx: VimVarIndex, val: *mut list_T) {
    unsafe {
        let mut tv: *mut typval_T = get_vim_var_tv(idx);
        tv_clear(tv);
        (*tv).v_type = VAR_LIST;
        (*tv).vval.v_list = val;
        if !val.is_null() {
            tv_list_ref(val);
        }
    }
}

pub unsafe extern "C" fn set_vim_var_dict(idx: VimVarIndex, val: *mut dict_T) {
    unsafe {
        let mut tv: *mut typval_T = get_vim_var_tv(idx);
        tv_clear(tv);
        (*tv).v_type = VAR_DICT;
        (*tv).vval.v_dict = val;
        if val.is_null() {
            return;
        }
        (*val).dv_refcount += 1;
        tv_dict_set_keys_readonly(val);
    }
}

pub unsafe extern "C" fn set_vim_var_partial(idx: VimVarIndex, mut val: *mut partial_T) {
    unsafe {
        let mut tv: *mut typval_T = get_vim_var_tv(idx);
        (*tv).vval.v_partial = val;
    }
}

pub unsafe extern "C" fn set_reg_var(mut c: ::core::ffi::c_int) {
    unsafe {
        let mut regname: [::core::ffi::c_char; 2] = [0; 2];
        if c == 0 as ::core::ffi::c_int || c == ' ' as ::core::ffi::c_int {
            regname[0 as ::core::ffi::c_int as usize] = '"' as ::core::ffi::c_char;
        } else {
            regname[0 as ::core::ffi::c_int as usize] = c as ::core::ffi::c_char;
        }
        regname[1 as ::core::ffi::c_int as usize] = NUL;
        let mut tv: *mut typval_T = get_vim_var_tv(VV_REG);
        if (*tv).vval.v_string.is_null()
            || *(*tv).vval.v_string.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != c
        {
            set_vim_var_string(
                VV_REG,
                &raw mut regname as *mut ::core::ffi::c_char,
                1 as ptrdiff_t,
            );
        }
    }
}

pub unsafe extern "C" fn v_exception(
    mut oldval: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut tv: *mut typval_T = get_vim_var_tv(VV_EXCEPTION);
        if oldval.is_null() {
            return (*tv).vval.v_string;
        }
        (*tv).vval.v_string = oldval;
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub unsafe extern "C" fn set_cmdarg(
    mut eap: *mut exarg_T,
    mut oldarg: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut len: size_t = 0;
        let mut newval_len: size_t = 0;
        let mut newval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut xlen: size_t = 0;
        let mut rc: ::core::ffi::c_int = 0;
        let mut tv: *mut typval_T = get_vim_var_tv(VV_CMDARG);
        let mut oldval: *mut ::core::ffi::c_char = (*tv).vval.v_string;
        '_error: {
            if !eap.is_null() {
                len = 0 as size_t;
                if (*eap).force_bin == FORCE_BIN {
                    len = len.wrapping_add(6 as size_t);
                } else if (*eap).force_bin == FORCE_NOBIN {
                    len = len.wrapping_add(8 as size_t);
                }
                if (*eap).read_edit != 0 {
                    len = len.wrapping_add(7 as size_t);
                }
                if (*eap).force_ff != 0 as ::core::ffi::c_int {
                    len = len.wrapping_add(10 as size_t);
                }
                if (*eap).force_enc != 0 as ::core::ffi::c_int {
                    len = len.wrapping_add(
                        strlen((*eap).cmd.offset((*eap).force_enc as isize))
                            .wrapping_add(7 as size_t),
                    );
                }
                if (*eap).bad_char != 0 as ::core::ffi::c_int {
                    len = len.wrapping_add(
                        (7 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as size_t,
                    );
                }
                if (*eap).mkdir_p != 0 as ::core::ffi::c_int {
                    len = len.wrapping_add(4 as size_t);
                }
                newval_len = len.wrapping_add(1 as size_t);
                newval = xmalloc(newval_len) as *mut ::core::ffi::c_char;
                xlen = 0 as size_t;
                rc = 0 as ::core::ffi::c_int;
                if (*eap).force_bin == FORCE_BIN {
                    rc = snprintf(
                        newval,
                        newval_len,
                        b" ++bin\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                } else if (*eap).force_bin == FORCE_NOBIN {
                    rc = snprintf(
                        newval,
                        newval_len,
                        b" ++nobin\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                } else {
                    *newval = NUL;
                }
                if rc >= 0 as ::core::ffi::c_int {
                    xlen = xlen.wrapping_add(rc as size_t);
                    if (*eap).read_edit != 0 {
                        rc = snprintf(
                            newval.offset(xlen as isize),
                            newval_len.wrapping_sub(xlen),
                            b" ++edit\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        if rc < 0 as ::core::ffi::c_int {
                            break '_error;
                        } else {
                            xlen = xlen.wrapping_add(rc as size_t);
                        }
                    }
                    if (*eap).force_ff != 0 as ::core::ffi::c_int {
                        rc = snprintf(
                            newval.offset(xlen as isize),
                            newval_len.wrapping_sub(xlen),
                            b" ++ff=%s\0".as_ptr() as *const ::core::ffi::c_char,
                            if (*eap).force_ff == 'u' as ::core::ffi::c_int {
                                b"unix\0".as_ptr() as *const ::core::ffi::c_char
                            } else if (*eap).force_ff == 'd' as ::core::ffi::c_int {
                                b"dos\0".as_ptr() as *const ::core::ffi::c_char
                            } else {
                                b"mac\0".as_ptr() as *const ::core::ffi::c_char
                            },
                        );
                        if rc < 0 as ::core::ffi::c_int {
                            break '_error;
                        } else {
                            xlen = xlen.wrapping_add(rc as size_t);
                        }
                    }
                    if (*eap).force_enc != 0 as ::core::ffi::c_int {
                        rc = snprintf(
                            newval.offset(xlen as isize),
                            newval_len.wrapping_sub(xlen),
                            b" ++enc=%s\0".as_ptr() as *const ::core::ffi::c_char,
                            (*eap).cmd.offset((*eap).force_enc as isize),
                        );
                        if rc < 0 as ::core::ffi::c_int {
                            break '_error;
                        } else {
                            xlen = xlen.wrapping_add(rc as size_t);
                        }
                    }
                    if (*eap).bad_char == BAD_KEEP {
                        rc = snprintf(
                            newval.offset(xlen as isize),
                            newval_len.wrapping_sub(xlen),
                            b" ++bad=keep\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        if rc < 0 as ::core::ffi::c_int {
                            break '_error;
                        } else {
                            xlen = xlen.wrapping_add(rc as size_t);
                        }
                    } else if (*eap).bad_char == BAD_DROP {
                        rc = snprintf(
                            newval.offset(xlen as isize),
                            newval_len.wrapping_sub(xlen),
                            b" ++bad=drop\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        if rc < 0 as ::core::ffi::c_int {
                            break '_error;
                        } else {
                            xlen = xlen.wrapping_add(rc as size_t);
                        }
                    } else if (*eap).bad_char != 0 as ::core::ffi::c_int {
                        rc = snprintf(
                            newval.offset(xlen as isize),
                            newval_len.wrapping_sub(xlen),
                            b" ++bad=%c\0".as_ptr() as *const ::core::ffi::c_char,
                            (*eap).bad_char,
                        );
                        if rc < 0 as ::core::ffi::c_int {
                            break '_error;
                        } else {
                            xlen = xlen.wrapping_add(rc as size_t);
                        }
                    }
                    if (*eap).mkdir_p != 0 as ::core::ffi::c_int {
                        rc = snprintf(
                            newval.offset(xlen as isize),
                            newval_len.wrapping_sub(xlen),
                            b" ++p\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        if rc < 0 as ::core::ffi::c_int {
                            break '_error;
                        } else {
                            xlen = xlen.wrapping_add(rc as size_t);
                        }
                    }
                    '_c2rust_label: {
                        if xlen <= newval_len {
                        } else {
                            __assert_fail(
                                b"xlen <= newval_len\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/eval/vars.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                2297 as ::core::ffi::c_uint,
                                b"char *set_cmdarg(exarg_T *, char *)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    (*tv).vval.v_string = newval;
                    return oldval;
                }
            }
        }
        xfree(oldval as *mut ::core::ffi::c_void);
        (*tv).vval.v_string = oldarg;
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub unsafe extern "C" fn v_throwpoint(
    mut oldval: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut tv: *mut typval_T = get_vim_var_tv(VV_THROWPOINT);
        if oldval.is_null() {
            return (*tv).vval.v_string;
        }
        (*tv).vval.v_string = oldval;
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub unsafe extern "C" fn set_vcount(
    mut count: int64_t,
    mut count1: int64_t,
    mut set_prevcount: bool,
) {
    unsafe {
        if set_prevcount {
            (*get_vim_var_tv(VV_PREVCOUNT)).vval.v_number = get_vim_var_nr(VV_COUNT);
        }
        (*get_vim_var_tv(VV_COUNT)).vval.v_number = count as varnumber_T;
        (*get_vim_var_tv(VV_COUNT1)).vval.v_number = count1 as varnumber_T;
    }
}

pub unsafe extern "C" fn before_set_vvar(
    varname: *const ::core::ffi::c_char,
    di: *mut dictitem_T,
    tv: *mut typval_T,
    copy: bool,
    watched: bool,
    type_error: *mut bool,
) -> bool {
    unsafe {
        if (*di).di_tv.v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut oldtv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            if watched {
                tv_copy(&raw mut (*di).di_tv, &raw mut oldtv);
            }
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*di).di_tv.vval.v_string as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
            if copy as ::core::ffi::c_int != 0
                || (*tv).v_type as ::core::ffi::c_uint
                    != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let val: *const ::core::ffi::c_char = tv_get_string(tv);
                if (*di).di_tv.vval.v_string.is_null() {
                    (*di).di_tv.vval.v_string = xstrdup(val);
                }
            } else {
                (*di).di_tv.vval.v_string = (*tv).vval.v_string;
                (*tv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            if watched {
                tv_dict_watcher_notify(
                    vimvardict.ptr(),
                    varname,
                    &raw mut (*di).di_tv,
                    &raw mut oldtv,
                );
                tv_clear(&raw mut oldtv);
            }
            return false_0 != 0;
        } else if (*di).di_tv.v_type as ::core::ffi::c_uint
            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut oldtv_0: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            if watched {
                tv_copy(&raw mut (*di).di_tv, &raw mut oldtv_0);
            }
            (*di).di_tv.vval.v_number = tv_get_number(tv);
            if strcmp(
                varname,
                b"searchforward\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                set_search_direction(if (*di).di_tv.vval.v_number != 0 {
                    '/' as ::core::ffi::c_int
                } else {
                    '?' as ::core::ffi::c_int
                });
            } else if strcmp(
                varname,
                b"hlsearch\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                no_hlsearch.set((*di).di_tv.vval.v_number == 0);
                redraw_all_later(UPD_SOME_VALID);
            }
            if watched {
                tv_dict_watcher_notify(
                    vimvardict.ptr(),
                    varname,
                    &raw mut (*di).di_tv,
                    &raw mut oldtv_0,
                );
                tv_clear(&raw mut oldtv_0);
            }
            return false_0 != 0;
        } else if (*di).di_tv.v_type as ::core::ffi::c_uint != (*tv).v_type as ::core::ffi::c_uint {
            *type_error = true_0 != 0;
            return false_0 != 0;
        }
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn reset_v_option_vars() {
    unsafe {
        set_vim_var_string(
            VV_OPTION_NEW,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        set_vim_var_string(
            VV_OPTION_OLD,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        set_vim_var_string(
            VV_OPTION_OLDLOCAL,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        set_vim_var_string(
            VV_OPTION_OLDGLOBAL,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        set_vim_var_string(
            VV_OPTION_COMMAND,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        set_vim_var_string(
            VV_OPTION_TYPE,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
    }
}
