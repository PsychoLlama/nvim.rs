//! `[]`, `[:]` and `.` applied to a value the evaluator already has.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn eval_index(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    mut verbose: bool,
) -> c_int {
    let evaluate: bool = !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as c_int != 0;
    let mut empty1: bool = false_0 != 0;
    let mut empty2: bool = false_0 != 0;
    let mut range: bool = false_0 != 0;
    let mut key: *const c_char = ::core::ptr::null::<c_char>();
    let mut keylen: ptrdiff_t = -1 as ptrdiff_t;
    if check_can_index(rettv, evaluate, verbose) == FAIL {
        return FAIL;
    }
    let mut var1: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut var2: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if **arg as c_int == '.' as c_int {
        key = (*arg).offset(1 as c_int as isize);
        keylen = 0 as ptrdiff_t;
        while eval_isdictc(*key.offset(keylen as isize) as c_int) {
            keylen += 1;
        }
        if keylen == 0 as ptrdiff_t {
            return FAIL;
        }
        *arg = skipwhite(key.offset(keylen as isize));
    } else {
        *arg = skipwhite((*arg).offset(1 as c_int as isize));
        if **arg as c_int == ':' as c_int {
            empty1 = true_0 != 0;
        } else if eval1(arg, &raw mut var1, evalarg) == FAIL {
            return FAIL;
        } else if evaluate as c_int != 0 && !tv_check_str(&raw mut var1) {
            tv_clear(&raw mut var1);
            return FAIL;
        }
        if **arg as c_int == ':' as c_int {
            range = true_0 != 0;
            *arg = skipwhite((*arg).offset(1 as c_int as isize));
            if **arg as c_int == ']' as c_int {
                empty2 = true_0 != 0;
            } else if eval1(arg, &raw mut var2, evalarg) == FAIL {
                if !empty1 {
                    tv_clear(&raw mut var1);
                }
                return FAIL;
            } else if evaluate as c_int != 0 && !tv_check_str(&raw mut var2) {
                if !empty1 {
                    tv_clear(&raw mut var1);
                }
                tv_clear(&raw mut var2);
                return FAIL;
            }
        }
        if **arg as c_int != ']' as c_int {
            if verbose {
                emsg(gettext(e_missbrac.get()));
            }
            tv_clear(&raw mut var1);
            if range {
                tv_clear(&raw mut var2);
            }
            return FAIL;
        }
        *arg = skipwhite((*arg).offset(1 as c_int as isize));
    }
    if evaluate {
        let mut res: c_int = eval_index_inner(
            rettv,
            range,
            if empty1 as c_int != 0 {
                ::core::ptr::null_mut::<typval_T>()
            } else {
                &raw mut var1
            },
            if empty2 as c_int != 0 {
                ::core::ptr::null_mut::<typval_T>()
            } else {
                &raw mut var2
            },
            false_0 != 0,
            key,
            keylen,
            verbose,
        );
        if !empty1 {
            tv_clear(&raw mut var1);
        }
        if range {
            tv_clear(&raw mut var2);
        }
        return res;
    }
    return OK;
}

pub(crate) unsafe extern "C" fn check_can_index(
    mut rettv: *mut typval_T,
    mut evaluate: bool,
    mut verbose: bool,
) -> c_int {
    match (*rettv).v_type as c_uint {
        3 | 9 => {
            if verbose {
                emsg(gettext(
                    (e_cannot_index_a_funcref.ptr() as *const _) as *const c_char,
                ));
            }
            return FAIL;
        }
        6 => {
            if verbose {
                emsg(gettext(&raw const e_using_float_as_string as *const c_char));
            }
            return FAIL;
        }
        7 | 8 => {
            if verbose {
                emsg(gettext(
                    (e_cannot_index_special_variable.ptr() as *const _) as *const c_char,
                ));
            }
            return FAIL;
        }
        0 => {
            if evaluate {
                emsg(gettext(
                    (e_cannot_index_special_variable.ptr() as *const _) as *const c_char,
                ));
                return FAIL;
            }
        }
        2 | 1 | 4 | 5 | 10 | _ => {}
    }
    return OK;
}

pub unsafe extern "C" fn f_slice(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if check_can_index(
        argvars.offset(0 as c_int as isize),
        true_0 != 0,
        false_0 != 0,
    ) != OK
    {
        return;
    }
    tv_copy(argvars, rettv);
    eval_index_inner(
        rettv,
        true_0 != 0,
        argvars.offset(1 as c_int as isize),
        if (*argvars.offset(2 as c_int as isize)).v_type as c_uint == VAR_UNKNOWN as c_int as c_uint
        {
            ::core::ptr::null_mut::<typval_T>()
        } else {
            argvars.offset(2 as c_int as isize)
        },
        true_0 != 0,
        ::core::ptr::null::<c_char>(),
        0 as ptrdiff_t,
        false_0 != 0,
    );
}

pub(crate) unsafe extern "C" fn eval_index_inner(
    mut rettv: *mut typval_T,
    mut is_range: bool,
    mut var1: *mut typval_T,
    mut var2: *mut typval_T,
    mut exclusive: bool,
    mut key: *const c_char,
    mut keylen: ptrdiff_t,
    mut verbose: bool,
) -> c_int {
    let mut n1: varnumber_T = 0 as varnumber_T;
    let mut n2: varnumber_T = 0 as varnumber_T;
    if !var1.is_null() && (*rettv).v_type as c_uint != VAR_DICT as c_int as c_uint {
        n1 = tv_get_number(var1);
    }
    if is_range {
        if (*rettv).v_type as c_uint == VAR_DICT as c_int as c_uint {
            if verbose {
                emsg(gettext(
                    (e_cannot_slice_dictionary.ptr() as *const _) as *const c_char,
                ));
            }
            return FAIL;
        }
        if !var2.is_null() {
            n2 = tv_get_number(var2);
        } else {
            n2 = VARNUMBER_MAX as varnumber_T;
        }
    }
    match (*rettv).v_type as c_uint {
        1 | 2 => {
            let s: *const c_char = tv_get_string(rettv);
            let mut v: *mut c_char = ::core::ptr::null_mut::<c_char>();
            let mut len: c_int = strlen(s) as c_int;
            if exclusive {
                if is_range {
                    v = string_slice(s, n1, n2, exclusive);
                } else {
                    v = char_from_string(s, n1);
                }
            } else if is_range {
                if n1 < 0 as varnumber_T {
                    n1 = len as varnumber_T + n1;
                    if n1 < 0 as varnumber_T {
                        n1 = 0 as varnumber_T;
                    }
                }
                if n2 < 0 as varnumber_T {
                    n2 = len as varnumber_T + n2;
                } else if n2 >= len as varnumber_T {
                    n2 = len as varnumber_T;
                }
                if n1 >= len as varnumber_T || n2 < 0 as varnumber_T || n1 > n2 {
                    v = ::core::ptr::null_mut::<c_char>();
                } else {
                    v = xmemdupz(
                        s.offset(n1 as isize) as *const c_void,
                        (n2 as size_t)
                            .wrapping_sub(n1 as size_t)
                            .wrapping_add(1 as size_t),
                    ) as *mut c_char;
                }
            } else if n1 >= len as varnumber_T || n1 < 0 as varnumber_T {
                v = ::core::ptr::null_mut::<c_char>();
            } else {
                v = xmemdupz(s.offset(n1 as isize) as *const c_void, 1 as size_t) as *mut c_char;
            }
            tv_clear(rettv);
            (*rettv).v_type = VAR_STRING;
            (*rettv).vval.v_string = v;
        }
        10 => {
            tv_blob_slice_or_index((*rettv).vval.v_blob, is_range, n1, n2, exclusive, rettv);
        }
        4 => {
            if var1.is_null() {
                n1 = 0 as varnumber_T;
            }
            if var2.is_null() {
                n2 = VARNUMBER_MAX as varnumber_T;
            }
            if tv_list_slice_or_index(
                (*rettv).vval.v_list,
                is_range,
                n1,
                n2,
                exclusive,
                rettv,
                verbose,
            ) == FAIL
            {
                return FAIL;
            }
        }
        5 => {
            if key.is_null() {
                key = tv_get_string_chk(var1);
                if key.is_null() {
                    return FAIL;
                }
            }
            let item: *mut dictitem_T = tv_dict_find((*rettv).vval.v_dict, key, keylen);
            if item.is_null() && verbose as c_int != 0 {
                if keylen > 0 as ptrdiff_t {
                    semsg(
                        gettext(&raw const e_dictkey_len as *const c_char),
                        keylen,
                        key,
                    );
                } else {
                    semsg(gettext(&raw const e_dictkey as *const c_char), key);
                }
            }
            if item.is_null() || tv_is_luafunc(&raw mut (*item).di_tv) as c_int != 0 {
                return FAIL;
            }
            let mut tmp: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            tv_copy(&raw mut (*item).di_tv, &raw mut tmp);
            tv_clear(rettv);
            *rettv = tmp;
        }
        7 | 8 | 3 | 6 | 9 | 0 | _ => {}
    }
    return OK;
}

pub unsafe extern "C" fn char_from_string(
    mut str: *const c_char,
    mut index: varnumber_T,
) -> *mut c_char {
    let mut nchar: varnumber_T = index;
    if str.is_null() {
        return ::core::ptr::null_mut::<c_char>();
    }
    let mut slen: size_t = strlen(str);
    if index < 0 as varnumber_T {
        let mut clen: c_int = 0 as c_int;
        let mut nbyte: size_t = 0 as size_t;
        while nbyte < slen {
            nbyte = nbyte.wrapping_add(utfc_ptr2len(str.offset(nbyte as isize)) as size_t);
            clen += 1;
        }
        nchar = clen as varnumber_T + index;
        if nchar < 0 as varnumber_T {
            return ::core::ptr::null_mut::<c_char>();
        }
    }
    let mut nbyte_0: size_t = 0 as size_t;
    while nchar > 0 as varnumber_T && nbyte_0 < slen {
        nbyte_0 = nbyte_0.wrapping_add(utfc_ptr2len(str.offset(nbyte_0 as isize)) as size_t);
        nchar -= 1;
    }
    if nbyte_0 >= slen {
        return ::core::ptr::null_mut::<c_char>();
    }
    return xmemdupz(
        str.offset(nbyte_0 as isize) as *const c_void,
        utfc_ptr2len(str.offset(nbyte_0 as isize)) as size_t,
    ) as *mut c_char;
}

pub(crate) unsafe extern "C" fn char_idx2byte(
    mut str: *const c_char,
    mut str_len: size_t,
    mut idx: varnumber_T,
) -> ssize_t {
    let mut nchar: varnumber_T = idx;
    let mut nbyte: size_t = 0 as size_t;
    if nchar >= 0 as varnumber_T {
        while nchar > 0 as varnumber_T && nbyte < str_len {
            nbyte = nbyte.wrapping_add(utfc_ptr2len(str.offset(nbyte as isize)) as size_t);
            nchar -= 1;
        }
    } else {
        nbyte = str_len;
        while nchar < 0 as varnumber_T && nbyte > 0 as size_t {
            nbyte = nbyte.wrapping_sub(1);
            nbyte = nbyte.wrapping_sub(utf_head_off(str, str.offset(nbyte as isize)) as size_t);
            nchar += 1;
        }
        if nchar < 0 as varnumber_T {
            return -1 as ssize_t;
        }
    }
    return nbyte as ssize_t;
}

pub unsafe extern "C" fn string_slice(
    mut str: *const c_char,
    mut first: varnumber_T,
    mut last: varnumber_T,
    mut exclusive: bool,
) -> *mut c_char {
    if str.is_null() {
        return ::core::ptr::null_mut::<c_char>();
    }
    let mut slen: size_t = strlen(str);
    let mut start_byte: ssize_t = char_idx2byte(str, slen, first);
    if start_byte < 0 as ssize_t {
        start_byte = 0 as ssize_t;
    }
    let mut end_byte: ssize_t = 0;
    if last == -1 as varnumber_T && !exclusive || last == VARNUMBER_MAX as varnumber_T {
        end_byte = slen as ssize_t;
    } else {
        end_byte = char_idx2byte(str, slen, last);
        if !exclusive && end_byte >= 0 as ssize_t && end_byte < slen as ssize_t {
            end_byte += utfc_ptr2len(str.offset(end_byte as isize)) as ssize_t;
        }
    }
    if start_byte >= slen as ssize_t || end_byte <= start_byte {
        return ::core::ptr::null_mut::<c_char>();
    }
    return xmemdupz(
        str.offset(start_byte as isize) as *const c_void,
        (end_byte - start_byte) as size_t,
    ) as *mut c_char;
}

pub unsafe extern "C" fn handle_subscript(
    arg: *mut *const c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    mut verbose: bool,
) -> c_int {
    let evaluate: bool = !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as c_int != 0;
    let mut ret: c_int = OK;
    let mut selfdict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    let mut lua_funcname: *const c_char = ::core::ptr::null::<c_char>();
    if tv_is_luafunc(rettv) {
        if !evaluate {
            tv_clear(rettv);
        }
        if **arg as c_int != '.' as c_int {
            tv_clear(rettv);
            ret = FAIL;
        } else {
            *arg = (*arg).offset(1);
            lua_funcname = *arg;
            let len: c_int = check_luafunc_name(*arg, true_0 != 0);
            if len == 0 as c_int {
                tv_clear(rettv);
                ret = FAIL;
            }
            *arg = (*arg).offset(len as isize);
        }
    }
    while ret == OK
        && ((**arg as c_int == '[' as c_int
            || **arg as c_int == '.' as c_int
                && (*rettv).v_type as c_uint == VAR_DICT as c_int as c_uint
            || **arg as c_int == '(' as c_int && (!evaluate || tv_is_func(*rettv) as c_int != 0))
            && !ascii_iswhite(*(*arg).offset(-(1 as c_int as isize)) as c_int)
            || **arg as c_int == '-' as c_int
                && *(*arg).offset(1 as c_int as isize) as c_int == '>' as c_int)
    {
        if **arg as c_int == '(' as c_int {
            ret = call_func_rettv(
                arg as *mut *mut c_char,
                evalarg,
                rettv,
                evaluate,
                selfdict,
                ::core::ptr::null_mut::<typval_T>(),
                lua_funcname,
            );
            if aborting() {
                if ret == OK {
                    tv_clear(rettv);
                }
                ret = FAIL;
            }
            tv_dict_unref(selfdict);
            selfdict = ::core::ptr::null_mut::<dict_T>();
        } else if **arg as c_int == '-' as c_int {
            if *(*arg).offset(2 as c_int as isize) as c_int == '{' as c_int {
                ret = eval_lambda(arg as *mut *mut c_char, rettv, evalarg, verbose);
            } else {
                ret = eval_method(arg as *mut *mut c_char, rettv, evalarg, verbose);
            }
        } else {
            tv_dict_unref(selfdict);
            if (*rettv).v_type as c_uint == VAR_DICT as c_int as c_uint {
                selfdict = (*rettv).vval.v_dict;
                if !selfdict.is_null() {
                    (*selfdict).dv_refcount += 1;
                }
            } else {
                selfdict = ::core::ptr::null_mut::<dict_T>();
            }
            if eval_index(arg as *mut *mut c_char, rettv, evalarg, verbose) == FAIL {
                tv_clear(rettv);
                ret = FAIL;
            }
        }
    }
    if !selfdict.is_null() && tv_is_func(*rettv) as c_int != 0 {
        set_selfdict(rettv, selfdict);
    }
    tv_dict_unref(selfdict);
    return ret;
}

pub unsafe extern "C" fn set_selfdict(rettv: *mut typval_T, selfdict: *mut dict_T) {
    if (*rettv).v_type as c_uint == VAR_PARTIAL as c_int as c_uint
        && !(*(*rettv).vval.v_partial).pt_auto
        && !(*(*rettv).vval.v_partial).pt_dict.is_null()
    {
        return;
    }
    make_partial(selfdict, rettv);
}
