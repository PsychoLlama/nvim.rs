//! `:for`: the list of things to iterate, and the step from one to the next.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn eval_for_line(
    mut arg: *const c_char,
    mut errp: *mut bool,
    mut eap: *mut exarg_T,
    evalarg: *mut evalarg_T,
) -> *mut c_void {
    let mut fi: *mut forinfo_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<forinfo_T>()) as *mut forinfo_T;
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    let skip: bool = (*evalarg).eval_flags & EVAL_EVALUATE as c_int == 0;
    *errp = true_0 != 0;
    let mut expr: *const c_char = skip_var_list(
        arg,
        &raw mut (*fi).fi_varcount,
        &raw mut (*fi).fi_semicolon,
        false_0 != 0,
    );
    if expr.is_null() {
        return fi as *mut c_void;
    }
    expr = skipwhite(expr);
    if *expr.offset(0 as c_int as isize) as c_int != 'i' as c_int
        || *expr.offset(1 as c_int as isize) as c_int != 'n' as c_int
        || !(*expr.offset(2 as c_int as isize) as c_int == NUL
            || ascii_iswhite(*expr.offset(2 as c_int as isize) as c_int) as c_int != 0)
    {
        emsg(gettext(
            b"E690: Missing \"in\" after :for\0".as_ptr() as *const c_char
        ));
        return fi as *mut c_void;
    }
    if skip {
        (*emsg_skip.ptr()) += 1;
    }
    expr = skipwhite(expr.offset(2 as c_int as isize));
    if eval0(expr as *mut c_char, &raw mut tv, eap, evalarg) == OK {
        *errp = false_0 != 0;
        if !skip {
            if tv.v_type as c_uint == VAR_LIST as c_int as c_uint {
                l = tv.vval.v_list;
                if l.is_null() {
                    tv_clear(&raw mut tv);
                } else {
                    (*fi).fi_list = l;
                    tv_list_watch_add(l, &raw mut (*fi).fi_lw);
                    (*fi).fi_lw.lw_item = tv_list_first(l);
                }
            } else if tv.v_type as c_uint == VAR_BLOB as c_int as c_uint {
                (*fi).fi_bi = 0 as c_int;
                if !tv.vval.v_blob.is_null() {
                    let mut btv: typval_T = typval_T {
                        v_type: VAR_UNKNOWN,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union { v_number: 0 },
                    };
                    tv_blob_copy(tv.vval.v_blob, &raw mut btv);
                    (*fi).fi_blob = btv.vval.v_blob;
                }
                tv_clear(&raw mut tv);
            } else if tv.v_type as c_uint == VAR_STRING as c_int as c_uint {
                (*fi).fi_byte_idx = 0 as c_int;
                (*fi).fi_string = tv.vval.v_string;
                tv.vval.v_string = ::core::ptr::null_mut::<c_char>();
                if (*fi).fi_string.is_null() {
                    (*fi).fi_string = xstrdup(b"\0".as_ptr() as *const c_char);
                }
            } else {
                emsg(gettext(
                    (e_string_list_or_blob_required.ptr() as *const _) as *const c_char,
                ));
                tv_clear(&raw mut tv);
            }
        }
    }
    if skip {
        (*emsg_skip.ptr()) -= 1;
    }
    return fi as *mut c_void;
}

pub unsafe extern "C" fn next_for_item(mut fi_void: *mut c_void, mut arg: *mut c_char) -> bool {
    let mut fi: *mut forinfo_T = fi_void as *mut forinfo_T;
    if !(*fi).fi_blob.is_null() {
        if (*fi).fi_bi >= tv_blob_len((*fi).fi_blob) {
            return false_0 != 0;
        }
        let mut tv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        tv.v_type = VAR_NUMBER;
        tv.v_lock = VAR_FIXED;
        tv.vval.v_number = tv_blob_get((*fi).fi_blob, (*fi).fi_bi) as varnumber_T;
        (*fi).fi_bi += 1;
        return ex_let_vars(
            arg,
            &raw mut tv,
            true_0,
            (*fi).fi_semicolon,
            (*fi).fi_varcount,
            false_0,
            ::core::ptr::null_mut::<c_char>(),
        ) == OK;
    }
    if !(*fi).fi_string.is_null() {
        let len: c_int = utfc_ptr2len((*fi).fi_string.offset((*fi).fi_byte_idx as isize));
        if len == 0 as c_int {
            return false_0 != 0;
        }
        let mut tv_0: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        tv_0.v_type = VAR_STRING;
        tv_0.v_lock = VAR_FIXED;
        tv_0.vval.v_string = xmemdupz(
            (*fi).fi_string.offset((*fi).fi_byte_idx as isize) as *const c_void,
            len as size_t,
        ) as *mut c_char;
        (*fi).fi_byte_idx += len;
        let result: c_int = (ex_let_vars(
            arg,
            &raw mut tv_0,
            true_0,
            (*fi).fi_semicolon,
            (*fi).fi_varcount,
            false_0,
            ::core::ptr::null_mut::<c_char>(),
        ) == OK) as c_int;
        xfree(tv_0.vval.v_string as *mut c_void);
        return result != 0;
    }
    let mut item: *mut listitem_T = (*fi).fi_lw.lw_item;
    if item.is_null() {
        return false_0 != 0;
    }
    (*fi).fi_lw.lw_item = (*item).li_next;
    return ex_let_vars(
        arg,
        &raw mut (*item).li_tv,
        true_0,
        (*fi).fi_semicolon,
        (*fi).fi_varcount,
        false_0,
        ::core::ptr::null_mut::<c_char>(),
    ) == OK;
}

pub unsafe extern "C" fn free_for_info(mut fi_void: *mut c_void) {
    let mut fi: *mut forinfo_T = fi_void as *mut forinfo_T;
    if fi.is_null() {
        return;
    }
    if !(*fi).fi_list.is_null() {
        tv_list_watch_remove((*fi).fi_list, &raw mut (*fi).fi_lw);
        tv_list_unref((*fi).fi_list);
    } else if !(*fi).fi_blob.is_null() {
        tv_blob_unref((*fi).fi_blob);
    } else {
        xfree((*fi).fi_string as *mut c_void);
    }
    xfree(fi as *mut c_void);
}
