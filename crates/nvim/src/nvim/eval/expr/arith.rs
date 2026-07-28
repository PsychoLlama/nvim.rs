//! What the arithmetic levels do once both operands are in hand.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn num_divide(mut n1: varnumber_T, mut n2: varnumber_T) -> varnumber_T {
    let mut result: varnumber_T = 0;
    if n2 == 0 as varnumber_T {
        if n1 == 0 as varnumber_T {
            result = VARNUMBER_MIN as varnumber_T;
        } else if n1 < 0 as varnumber_T {
            result = -VARNUMBER_MAX as varnumber_T;
        } else {
            result = VARNUMBER_MAX as varnumber_T;
        }
    } else if n1 == VARNUMBER_MIN as varnumber_T && n2 == -1 as varnumber_T {
        result = VARNUMBER_MAX as varnumber_T;
    } else {
        result = n1 / n2;
    }
    return result;
}

pub unsafe extern "C" fn num_modulus(mut n1: varnumber_T, mut n2: varnumber_T) -> varnumber_T {
    return if n2 == 0 as varnumber_T {
        0 as varnumber_T
    } else {
        n1 % n2
    };
}

pub(crate) unsafe extern "C" fn eval_addblob(mut tv1: *mut typval_T, mut tv2: *mut typval_T) {
    let b1: *const blob_T = (*tv1).vval.v_blob;
    let b2: *const blob_T = (*tv2).vval.v_blob;
    let b: *mut blob_T = tv_blob_alloc();
    let mut len1: int64_t = tv_blob_len(b1) as int64_t;
    let mut len2: int64_t = tv_blob_len(b2) as int64_t;
    let mut totallen: int64_t = len1 + len2;
    if totallen >= 0 as int64_t && totallen <= INT_MAX as int64_t {
        ga_grow(&raw mut (*b).bv_ga, totallen as c_int);
        if len1 > 0 as int64_t {
            memmove(
                (*b).bv_ga.ga_data as *mut c_char as *mut c_void,
                (*b1).bv_ga.ga_data,
                len1 as size_t,
            );
        }
        if len2 > 0 as int64_t {
            memmove(
                ((*b).bv_ga.ga_data as *mut c_char).offset(len1 as isize) as *mut c_void,
                (*b2).bv_ga.ga_data,
                len2 as size_t,
            );
        }
        (*b).bv_ga.ga_len = totallen as c_int;
    }
    tv_clear(tv1);
    tv_blob_set_ret(tv1, b);
}

pub(crate) unsafe extern "C" fn eval_addlist(
    mut tv1: *mut typval_T,
    mut tv2: *mut typval_T,
) -> c_int {
    let mut var3: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if tv_list_concat((*tv1).vval.v_list, (*tv2).vval.v_list, &raw mut var3) == FAIL {
        tv_clear(tv1);
        tv_clear(tv2);
        return FAIL;
    }
    tv_clear(tv1);
    *tv1 = var3;
    return OK;
}

pub unsafe extern "C" fn grow_string_tv(mut tv1: *mut typval_T, mut s2: *const c_char) -> c_int {
    if (*tv1).v_type as c_uint != VAR_STRING as c_int as c_uint || (*tv1).vval.v_string.is_null() {
        return FAIL;
    }
    let mut len1: size_t = strlen((*tv1).vval.v_string);
    let mut len2: size_t = strlen(s2);
    let mut p: *mut c_char = xrealloc(
        (*tv1).vval.v_string as *mut c_void,
        len1.wrapping_add(len2).wrapping_add(1 as size_t),
    ) as *mut c_char;
    memmove(
        p.offset(len1 as isize) as *mut c_void,
        s2 as *const c_void,
        len2.wrapping_add(1 as size_t),
    );
    (*tv1).vval.v_string = p;
    return OK;
}

pub(crate) unsafe extern "C" fn eval_concat_str(
    mut tv1: *mut typval_T,
    mut tv2: *mut typval_T,
) -> c_int {
    let mut buf1: [c_char; 65] = [0; 65];
    let mut buf2: [c_char; 65] = [0; 65];
    let s1: *const c_char = tv_get_string_buf(tv1, &raw mut buf1 as *mut c_char);
    let s2: *const c_char = tv_get_string_buf_chk(tv2, &raw mut buf2 as *mut c_char);
    if s2.is_null() {
        tv_clear(tv1);
        tv_clear(tv2);
        return FAIL;
    }
    if grow_string_tv(tv1, s2) == OK {
        return OK;
    }
    let mut p: *mut c_char = concat_str(s1, s2);
    tv_clear(tv1);
    (*tv1).v_type = VAR_STRING;
    (*tv1).vval.v_string = p;
    return OK;
}

pub(crate) unsafe extern "C" fn eval_addsub_number(
    mut tv1: *mut typval_T,
    mut tv2: *mut typval_T,
    mut op: c_int,
) -> c_int {
    let mut error: bool = false_0 != 0;
    let mut n1: varnumber_T = 0;
    let mut n2: varnumber_T = 0;
    let mut f1: float_T = 0 as c_int as float_T;
    let mut f2: float_T = 0 as c_int as float_T;
    if (*tv1).v_type as c_uint == VAR_FLOAT as c_int as c_uint {
        f1 = (*tv1).vval.v_float;
        n1 = 0 as varnumber_T;
    } else {
        n1 = tv_get_number_chk(tv1, &raw mut error);
        if error {
            tv_clear(tv1);
            tv_clear(tv2);
            return FAIL;
        }
        if (*tv2).v_type as c_uint == VAR_FLOAT as c_int as c_uint {
            f1 = n1 as float_T;
        }
    }
    if (*tv2).v_type as c_uint == VAR_FLOAT as c_int as c_uint {
        f2 = (*tv2).vval.v_float;
        n2 = 0 as varnumber_T;
    } else {
        n2 = tv_get_number_chk(tv2, &raw mut error);
        if error {
            tv_clear(tv1);
            tv_clear(tv2);
            return FAIL;
        }
        if (*tv1).v_type as c_uint == VAR_FLOAT as c_int as c_uint {
            f2 = n2 as float_T;
        }
    }
    tv_clear(tv1);
    if (*tv1).v_type as c_uint == VAR_FLOAT as c_int as c_uint
        || (*tv2).v_type as c_uint == VAR_FLOAT as c_int as c_uint
    {
        if op == '+' as c_int {
            f1 = f1 + f2;
        } else {
            f1 = f1 - f2;
        }
        (*tv1).v_type = VAR_FLOAT;
        (*tv1).vval.v_float = f1;
    } else {
        if op == '+' as c_int {
            n1 = n1 + n2;
        } else {
            n1 = n1 - n2;
        }
        (*tv1).v_type = VAR_NUMBER;
        (*tv1).vval.v_number = n1;
    }
    return OK;
}

pub(crate) unsafe extern "C" fn eval_multdiv_number(
    mut tv1: *mut typval_T,
    mut tv2: *mut typval_T,
    mut op: c_int,
) -> c_int {
    let mut n1: varnumber_T = 0;
    let mut n2: varnumber_T = 0;
    let mut use_float: bool = false_0 != 0;
    let mut f1: float_T = 0 as c_int as float_T;
    let mut f2: float_T = 0 as c_int as float_T;
    let mut error: bool = false_0 != 0;
    if (*tv1).v_type as c_uint == VAR_FLOAT as c_int as c_uint {
        f1 = (*tv1).vval.v_float;
        use_float = true_0 != 0;
        n1 = 0 as varnumber_T;
    } else {
        n1 = tv_get_number_chk(tv1, &raw mut error);
    }
    tv_clear(tv1);
    if error {
        tv_clear(tv2);
        return FAIL;
    }
    if (*tv2).v_type as c_uint == VAR_FLOAT as c_int as c_uint {
        if !use_float {
            f1 = n1 as float_T;
            use_float = true_0 != 0;
        }
        f2 = (*tv2).vval.v_float;
        n2 = 0 as varnumber_T;
    } else {
        n2 = tv_get_number_chk(tv2, &raw mut error);
        tv_clear(tv2);
        if error {
            return FAIL;
        }
        if use_float {
            f2 = n2 as float_T;
        }
    }
    if use_float {
        if op == '*' as c_int {
            f1 = f1 * f2;
        } else if op == '/' as c_int {
            f1 = if f2 == 0 as c_int as float_T {
                if f1 == 0 as c_int as float_T {
                    ::core::f32::NAN as float_T
                } else if f1 > 0 as c_int as float_T {
                    ::core::f32::INFINITY as float_T
                } else {
                    -::core::f32::INFINITY as float_T
                }
            } else {
                f1 / f2
            };
        } else {
            emsg(gettext(
                b"E804: Cannot use '%' with Float\0".as_ptr() as *const c_char
            ));
            return FAIL;
        }
        (*tv1).v_type = VAR_FLOAT;
        (*tv1).vval.v_float = f1;
    } else {
        if op == '*' as c_int {
            // Vimscript arithmetic wraps on overflow (C two's-complement).
            n1 = n1.wrapping_mul(n2);
        } else if op == '/' as c_int {
            n1 = num_divide(n1, n2);
        } else {
            n1 = num_modulus(n1, n2);
        }
        (*tv1).v_type = VAR_NUMBER;
        (*tv1).vval.v_number = n1;
    }
    return OK;
}
