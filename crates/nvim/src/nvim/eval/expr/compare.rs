//! Comparing two values, for every operator and every pair of types.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn func_equal(
    mut tv1: *mut typval_T,
    mut tv2: *mut typval_T,
    mut ic: bool,
) -> bool {
    let mut s1: *mut c_char = if (*tv1).v_type as c_uint == VAR_FUNC as c_int as c_uint {
        (*tv1).vval.v_string
    } else {
        partial_name((*tv1).vval.v_partial)
    };
    if !s1.is_null() && *s1 as c_int == NUL {
        s1 = ::core::ptr::null_mut::<c_char>();
    }
    let mut s2: *mut c_char = if (*tv2).v_type as c_uint == VAR_FUNC as c_int as c_uint {
        (*tv2).vval.v_string
    } else {
        partial_name((*tv2).vval.v_partial)
    };
    if !s2.is_null() && *s2 as c_int == NUL {
        s2 = ::core::ptr::null_mut::<c_char>();
    }
    if s1.is_null() || s2.is_null() {
        if s1 != s2 {
            return false_0 != 0;
        }
    } else if strcmp(s1, s2) != 0 as c_int {
        return false_0 != 0;
    }
    let mut d1: *mut dict_T = if (*tv1).v_type as c_uint == VAR_FUNC as c_int as c_uint {
        ::core::ptr::null_mut::<dict_T>()
    } else {
        (*(*tv1).vval.v_partial).pt_dict
    };
    let mut d2: *mut dict_T = if (*tv2).v_type as c_uint == VAR_FUNC as c_int as c_uint {
        ::core::ptr::null_mut::<dict_T>()
    } else {
        (*(*tv2).vval.v_partial).pt_dict
    };
    if d1.is_null() || d2.is_null() {
        if d1 != d2 {
            return false_0 != 0;
        }
    } else if !tv_dict_equal(d1, d2, ic) {
        return false_0 != 0;
    }
    let mut a1: c_int = if (*tv1).v_type as c_uint == VAR_FUNC as c_int as c_uint {
        0 as c_int
    } else {
        (*(*tv1).vval.v_partial).pt_argc
    };
    let mut a2: c_int = if (*tv2).v_type as c_uint == VAR_FUNC as c_int as c_uint {
        0 as c_int
    } else {
        (*(*tv2).vval.v_partial).pt_argc
    };
    if a1 != a2 {
        return false_0 != 0;
    }
    let mut i: c_int = 0 as c_int;
    while i < a1 {
        if !tv_equal(
            (*(*tv1).vval.v_partial).pt_argv.offset(i as isize),
            (*(*tv2).vval.v_partial).pt_argv.offset(i as isize),
            ic,
        ) {
            return false_0 != 0;
        }
        i += 1;
    }
    return true_0 != 0;
}

pub unsafe extern "C" fn typval_compare(
    mut typ1: *mut typval_T,
    mut typ2: *mut typval_T,
    mut type_0: exprtype_T,
    mut ic: bool,
) -> c_int {
    let mut n1: varnumber_T = 0;
    let mut n2: varnumber_T = 0;
    let type_is: bool = type_0 as c_uint == EXPR_IS as c_int as c_uint
        || type_0 as c_uint == EXPR_ISNOT as c_int as c_uint;
    if type_is as c_int != 0 && (*typ1).v_type as c_uint != (*typ2).v_type as c_uint {
        n1 = (type_0 as c_uint == EXPR_ISNOT as c_int as c_uint) as c_int as varnumber_T;
    } else if (*typ1).v_type as c_uint == VAR_BLOB as c_int as c_uint
        || (*typ2).v_type as c_uint == VAR_BLOB as c_int as c_uint
    {
        if type_is {
            n1 = ((*typ1).v_type as c_uint == (*typ2).v_type as c_uint
                && (*typ1).vval.v_blob == (*typ2).vval.v_blob) as c_int
                as varnumber_T;
            if type_0 as c_uint == EXPR_ISNOT as c_int as c_uint {
                n1 = (n1 == 0) as c_int as varnumber_T;
            }
        } else if (*typ1).v_type as c_uint != (*typ2).v_type as c_uint
            || type_0 as c_uint != EXPR_EQUAL as c_int as c_uint
                && type_0 as c_uint != EXPR_NEQUAL as c_int as c_uint
        {
            if (*typ1).v_type as c_uint != (*typ2).v_type as c_uint {
                emsg(gettext(
                    b"E977: Can only compare Blob with Blob\0".as_ptr() as *const c_char
                ));
            } else {
                emsg(gettext(&raw const e_invalblob as *const c_char));
            }
            tv_clear(typ1);
            return FAIL;
        } else {
            n1 = tv_blob_equal((*typ1).vval.v_blob, (*typ2).vval.v_blob) as varnumber_T;
            if type_0 as c_uint == EXPR_NEQUAL as c_int as c_uint {
                n1 = (n1 == 0) as c_int as varnumber_T;
            }
        }
    } else if (*typ1).v_type as c_uint == VAR_LIST as c_int as c_uint
        || (*typ2).v_type as c_uint == VAR_LIST as c_int as c_uint
    {
        if type_is {
            n1 = ((*typ1).v_type as c_uint == (*typ2).v_type as c_uint
                && (*typ1).vval.v_list == (*typ2).vval.v_list) as c_int
                as varnumber_T;
            if type_0 as c_uint == EXPR_ISNOT as c_int as c_uint {
                n1 = (n1 == 0) as c_int as varnumber_T;
            }
        } else if (*typ1).v_type as c_uint != (*typ2).v_type as c_uint
            || type_0 as c_uint != EXPR_EQUAL as c_int as c_uint
                && type_0 as c_uint != EXPR_NEQUAL as c_int as c_uint
        {
            if (*typ1).v_type as c_uint != (*typ2).v_type as c_uint {
                emsg(gettext(
                    b"E691: Can only compare List with List\0".as_ptr() as *const c_char
                ));
            } else {
                emsg(gettext(
                    b"E692: Invalid operation for List\0".as_ptr() as *const c_char
                ));
            }
            tv_clear(typ1);
            return FAIL;
        } else {
            n1 = tv_list_equal((*typ1).vval.v_list, (*typ2).vval.v_list, ic) as varnumber_T;
            if type_0 as c_uint == EXPR_NEQUAL as c_int as c_uint {
                n1 = (n1 == 0) as c_int as varnumber_T;
            }
        }
    } else if (*typ1).v_type as c_uint == VAR_DICT as c_int as c_uint
        || (*typ2).v_type as c_uint == VAR_DICT as c_int as c_uint
    {
        if type_is {
            n1 = ((*typ1).v_type as c_uint == (*typ2).v_type as c_uint
                && (*typ1).vval.v_dict == (*typ2).vval.v_dict) as c_int
                as varnumber_T;
            if type_0 as c_uint == EXPR_ISNOT as c_int as c_uint {
                n1 = (n1 == 0) as c_int as varnumber_T;
            }
        } else if (*typ1).v_type as c_uint != (*typ2).v_type as c_uint
            || type_0 as c_uint != EXPR_EQUAL as c_int as c_uint
                && type_0 as c_uint != EXPR_NEQUAL as c_int as c_uint
        {
            if (*typ1).v_type as c_uint != (*typ2).v_type as c_uint {
                emsg(gettext(
                    b"E735: Can only compare Dictionary with Dictionary\0".as_ptr()
                        as *const c_char,
                ));
            } else {
                emsg(gettext(
                    b"E736: Invalid operation for Dictionary\0".as_ptr() as *const c_char,
                ));
            }
            tv_clear(typ1);
            return FAIL;
        } else {
            n1 = tv_dict_equal((*typ1).vval.v_dict, (*typ2).vval.v_dict, ic) as varnumber_T;
            if type_0 as c_uint == EXPR_NEQUAL as c_int as c_uint {
                n1 = (n1 == 0) as c_int as varnumber_T;
            }
        }
    } else if tv_is_func(*typ1) as c_int != 0 || tv_is_func(*typ2) as c_int != 0 {
        if type_0 as c_uint != EXPR_EQUAL as c_int as c_uint
            && type_0 as c_uint != EXPR_NEQUAL as c_int as c_uint
            && type_0 as c_uint != EXPR_IS as c_int as c_uint
            && type_0 as c_uint != EXPR_ISNOT as c_int as c_uint
        {
            emsg(gettext(
                b"E694: Invalid operation for Funcrefs\0".as_ptr() as *const c_char
            ));
            tv_clear(typ1);
            return FAIL;
        }
        if (*typ1).v_type as c_uint == VAR_PARTIAL as c_int as c_uint
            && (*typ1).vval.v_partial.is_null()
            || (*typ2).v_type as c_uint == VAR_PARTIAL as c_int as c_uint
                && (*typ2).vval.v_partial.is_null()
        {
            n1 = ((*typ1).vval.v_partial == (*typ2).vval.v_partial) as c_int as varnumber_T;
        } else if type_is {
            if (*typ1).v_type as c_uint == VAR_FUNC as c_int as c_uint
                && (*typ2).v_type as c_uint == VAR_FUNC as c_int as c_uint
            {
                n1 = tv_equal(typ1, typ2, ic) as varnumber_T;
            } else if (*typ1).v_type as c_uint == VAR_PARTIAL as c_int as c_uint
                && (*typ2).v_type as c_uint == VAR_PARTIAL as c_int as c_uint
            {
                n1 = ((*typ1).vval.v_partial == (*typ2).vval.v_partial) as c_int as varnumber_T;
            } else {
                n1 = false_0 as varnumber_T;
            }
        } else {
            n1 = tv_equal(typ1, typ2, ic) as varnumber_T;
        }
        if type_0 as c_uint == EXPR_NEQUAL as c_int as c_uint
            || type_0 as c_uint == EXPR_ISNOT as c_int as c_uint
        {
            n1 = (n1 == 0) as c_int as varnumber_T;
        }
    } else if ((*typ1).v_type as c_uint == VAR_FLOAT as c_int as c_uint
        || (*typ2).v_type as c_uint == VAR_FLOAT as c_int as c_uint)
        && type_0 as c_uint != EXPR_MATCH as c_int as c_uint
        && type_0 as c_uint != EXPR_NOMATCH as c_int as c_uint
    {
        let f1: float_T = tv_get_float(typ1);
        let f2: float_T = tv_get_float(typ2);
        n1 = false_0 as varnumber_T;
        match type_0 as c_uint {
            9 | 1 => {
                n1 = (f1 == f2) as c_int as varnumber_T;
            }
            10 | 2 => {
                n1 = (f1 != f2) as c_int as varnumber_T;
            }
            3 => {
                n1 = (f1 > f2) as c_int as varnumber_T;
            }
            4 => {
                n1 = (f1 >= f2) as c_int as varnumber_T;
            }
            5 => {
                n1 = (f1 < f2) as c_int as varnumber_T;
            }
            6 => {
                n1 = (f1 <= f2) as c_int as varnumber_T;
            }
            0 | 7 | 8 | _ => {}
        }
    } else if ((*typ1).v_type as c_uint == VAR_NUMBER as c_int as c_uint
        || (*typ2).v_type as c_uint == VAR_NUMBER as c_int as c_uint)
        && type_0 as c_uint != EXPR_MATCH as c_int as c_uint
        && type_0 as c_uint != EXPR_NOMATCH as c_int as c_uint
    {
        n1 = tv_get_number(typ1);
        n2 = tv_get_number(typ2);
        match type_0 as c_uint {
            9 | 1 => {
                n1 = (n1 == n2) as c_int as varnumber_T;
            }
            10 | 2 => {
                n1 = (n1 != n2) as c_int as varnumber_T;
            }
            3 => {
                n1 = (n1 > n2) as c_int as varnumber_T;
            }
            4 => {
                n1 = (n1 >= n2) as c_int as varnumber_T;
            }
            5 => {
                n1 = (n1 < n2) as c_int as varnumber_T;
            }
            6 => {
                n1 = (n1 <= n2) as c_int as varnumber_T;
            }
            0 | 7 | 8 | _ => {}
        }
    } else {
        let mut buf1: [c_char; 65] = [0; 65];
        let mut buf2: [c_char; 65] = [0; 65];
        let s1: *const c_char = tv_get_string_buf(typ1, &raw mut buf1 as *mut c_char);
        let s2: *const c_char = tv_get_string_buf(typ2, &raw mut buf2 as *mut c_char);
        let mut i: c_int = 0;
        if type_0 as c_uint != EXPR_MATCH as c_int as c_uint
            && type_0 as c_uint != EXPR_NOMATCH as c_int as c_uint
        {
            i = mb_strcmp_ic(ic, s1, s2);
        } else {
            i = 0 as c_int;
        }
        n1 = false_0 as varnumber_T;
        match type_0 as c_uint {
            9 | 1 => {
                n1 = (i == 0 as c_int) as c_int as varnumber_T;
            }
            10 | 2 => {
                n1 = (i != 0 as c_int) as c_int as varnumber_T;
            }
            3 => {
                n1 = (i > 0 as c_int) as c_int as varnumber_T;
            }
            4 => {
                n1 = (i >= 0 as c_int) as c_int as varnumber_T;
            }
            5 => {
                n1 = (i < 0 as c_int) as c_int as varnumber_T;
            }
            6 => {
                n1 = (i <= 0 as c_int) as c_int as varnumber_T;
            }
            7 | 8 => {
                n1 = pattern_match(s2, s1, ic) as varnumber_T;
                if type_0 as c_uint == EXPR_NOMATCH as c_int as c_uint {
                    n1 = (n1 == 0) as c_int as varnumber_T;
                }
            }
            0 | _ => {}
        }
    }
    tv_clear(typ1);
    (*typ1).v_type = VAR_NUMBER;
    (*typ1).vval.v_number = n1;
    return OK;
}
