//! List and dict literals, including the `#{}` form.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn eval_list(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    let evaluate: bool = if evalarg.is_null() {
        false_0
    } else {
        (*evalarg).eval_flags & EVAL_EVALUATE as c_int
    } != 0;
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    if evaluate {
        l = tv_list_alloc(kListLenShouldKnow as c_int as ptrdiff_t);
    }
    *arg = skipwhite((*arg).offset(1 as c_int as isize));
    '_failret: {
        while **arg as c_int != ']' as c_int && **arg as c_int != NUL {
            let mut tv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            if eval1(arg, &raw mut tv, evalarg) == FAIL {
                break '_failret;
            }
            if evaluate {
                tv.v_lock = VAR_UNLOCKED;
                tv_list_append_owned_tv(l, tv);
            }
            let mut had_comma: bool = **arg as c_int == ',' as c_int;
            if had_comma {
                *arg = skipwhite((*arg).offset(1 as c_int as isize));
            }
            if **arg as c_int == ']' as c_int {
                break;
            }
            if had_comma {
                continue;
            }
            semsg(
                gettext(b"E696: Missing comma in List: %s\0".as_ptr() as *const c_char),
                *arg,
            );
            break '_failret;
        }
        if **arg as c_int != ']' as c_int {
            semsg(gettext(e_list_end.get()), *arg);
        } else {
            *arg = skipwhite((*arg).offset(1 as c_int as isize));
            if evaluate {
                tv_list_set_ret(rettv, l);
            }
            return OK;
        }
    }
    if evaluate {
        tv_list_free(l);
    }
    return FAIL;
}

pub(crate) unsafe extern "C" fn get_literal_key(
    mut arg: *mut *mut c_char,
    mut tv: *mut typval_T,
) -> c_int {
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if !(**arg as c_uint >= 'A' as c_uint && **arg as c_uint <= 'Z' as c_uint
        || **arg as c_uint >= 'a' as c_uint && **arg as c_uint <= 'z' as c_uint
        || ascii_isdigit(**arg as c_int) as c_int != 0)
        && **arg as c_int != '_' as c_int
        && **arg as c_int != '-' as c_int
    {
        return FAIL;
    }
    p = *arg;
    while *p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint
        || *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint
        || ascii_isdigit(*p as c_int) as c_int != 0
        || *p as c_int == '_' as c_int
        || *p as c_int == '-' as c_int
    {
        p = p.offset(1);
    }
    (*tv).v_type = VAR_STRING;
    (*tv).vval.v_string =
        xmemdupz(*arg as *const c_void, p.offset_from(*arg) as size_t) as *mut c_char;
    *arg = skipwhite(p);
    return OK;
}

pub(crate) unsafe extern "C" fn eval_dict(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    mut literal: bool,
) -> c_int {
    let evaluate: bool = if evalarg.is_null() {
        false_0
    } else {
        (*evalarg).eval_flags & EVAL_EVALUATE as c_int
    } != 0;
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut key: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut curly_expr: *mut c_char = skipwhite((*arg).offset(1 as c_int as isize));
    let mut buf: [c_char; 65] = [0; 65];
    if *curly_expr as c_int != '}' as c_int
        && !literal
        && eval1(
            &raw mut curly_expr,
            &raw mut tv,
            ::core::ptr::null_mut::<evalarg_T>(),
        ) == OK
        && *skipwhite(curly_expr) as c_int == '}' as c_int
    {
        return NOTDONE;
    }
    let mut d: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    if evaluate {
        d = tv_dict_alloc();
    }
    let mut tvkey: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    tvkey.v_type = VAR_UNKNOWN;
    tv.v_type = VAR_UNKNOWN;
    *arg = skipwhite((*arg).offset(1 as c_int as isize));
    '_failret: {
        while **arg as c_int != '}' as c_int && **arg as c_int != NUL {
            if (if literal as c_int != 0 {
                get_literal_key(arg, &raw mut tvkey)
            } else {
                eval1(arg, &raw mut tvkey, evalarg)
            }) == FAIL
            {
                break '_failret;
            }
            if **arg as c_int != ':' as c_int {
                semsg(
                    gettext(b"E720: Missing colon in Dictionary: %s\0".as_ptr() as *const c_char),
                    *arg,
                );
                tv_clear(&raw mut tvkey);
                break '_failret;
            } else {
                if evaluate {
                    key = tv_get_string_buf_chk(&raw mut tvkey, &raw mut buf as *mut c_char)
                        as *mut c_char;
                    if key.is_null() {
                        tv_clear(&raw mut tvkey);
                        break '_failret;
                    }
                }
                *arg = skipwhite((*arg).offset(1 as c_int as isize));
                if eval1(arg, &raw mut tv, evalarg) == FAIL {
                    tv_clear(&raw mut tvkey);
                    break '_failret;
                } else {
                    if evaluate {
                        let mut item: *mut dictitem_T = tv_dict_find(d, key, -1 as ptrdiff_t);
                        if !item.is_null() {
                            semsg(
                                gettext(b"E721: Duplicate key in Dictionary: \"%s\"\0".as_ptr()
                                    as *const c_char),
                                key,
                            );
                            tv_clear(&raw mut tvkey);
                            tv_clear(&raw mut tv);
                            break '_failret;
                        } else {
                            item = tv_dict_item_alloc(key);
                            (*item).di_tv = tv;
                            (*item).di_tv.v_lock = VAR_UNLOCKED;
                            if tv_dict_add(d, item) == FAIL {
                                tv_dict_item_free(item);
                            }
                        }
                    }
                    tv_clear(&raw mut tvkey);
                    let mut had_comma: bool = **arg as c_int == ',' as c_int;
                    if had_comma {
                        *arg = skipwhite((*arg).offset(1 as c_int as isize));
                    }
                    if **arg as c_int == '}' as c_int {
                        break;
                    }
                    if had_comma {
                        continue;
                    }
                    semsg(
                        gettext(
                            b"E722: Missing comma in Dictionary: %s\0".as_ptr() as *const c_char
                        ),
                        *arg,
                    );
                    break '_failret;
                }
            }
        }
        if **arg as c_int != '}' as c_int {
            semsg(
                gettext(b"E723: Missing end of Dictionary '}': %s\0".as_ptr() as *const c_char),
                *arg,
            );
        } else {
            *arg = skipwhite((*arg).offset(1 as c_int as isize));
            if evaluate {
                tv_dict_set_ret(rettv, d);
            }
            return OK;
        }
    }
    if !d.is_null() {
        tv_dict_free(d);
    }
    return FAIL;
}

pub(crate) unsafe extern "C" fn eval_lit_dict(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    let mut ret: c_int = OK;
    if *(*arg).offset(1 as c_int as isize) as c_int == '{' as c_int {
        *arg = (*arg).offset(1);
        ret = eval_dict(arg, rettv, evalarg, true_0 != 0);
    } else {
        ret = NOTDONE;
    }
    return ret;
}
