//! Variables themselves: `islocked()`, `id()` and the dictionary
//! watchers.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub unsafe extern "C" fn f_dictwatcheradd(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"dict\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_dict
        .is_null()
    {
        let arg_errmsg: *const ::core::ffi::c_char =
            gettext(b"dictwatcheradd() argument\0".as_ptr() as *const ::core::ffi::c_char);
        let arg_errmsg_len: size_t = strlen(arg_errmsg);
        semsg(
            gettext(&raw const e_cannot_change_readonly_variable_str as *const ::core::ffi::c_char),
            arg_errmsg_len as ::core::ffi::c_int,
            arg_errmsg,
        );
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"key\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let key_pattern: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
    if key_pattern.is_null() {
        return;
    }
    let key_pattern_len: size_t = strlen(key_pattern);
    let mut callback: Callback = Callback {
        data: C2Rust_Unnamed_22 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    };
    if !callback_from_typval(
        &raw mut callback,
        argvars.offset(2 as ::core::ffi::c_int as isize),
    ) {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"funcref\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    tv_dict_watcher_add(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_dict,
        key_pattern,
        key_pattern_len,
        callback,
    );
}
pub unsafe extern "C" fn f_dictwatcherdel(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"dict\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"funcref\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let key_pattern: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
    if key_pattern.is_null() {
        return;
    }
    let mut callback: Callback = Callback {
        data: C2Rust_Unnamed_22 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    };
    if !callback_from_typval(
        &raw mut callback,
        argvars.offset(2 as ::core::ffi::c_int as isize),
    ) {
        return;
    }
    if !tv_dict_watcher_remove(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_dict,
        key_pattern,
        strlen(key_pattern),
        callback,
    ) {
        emsg(
            b"Couldn't find a watcher matching key and callback\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
    }
    callback_free(&raw mut callback);
}
pub unsafe extern "C" fn f_islocked(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
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
    (*rettv).vval.v_number = -1 as varnumber_T;
    let end: *const ::core::ffi::c_char = get_lval(
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_char,
        ::core::ptr::null_mut::<typval_T>(),
        &raw mut lv,
        false_0 != 0,
        false_0 != 0,
        GLV_NO_AUTOLOAD as ::core::ffi::c_int | GLV_READ_ONLY as ::core::ffi::c_int,
        FNE_CHECK_START,
    );
    if !end.is_null() && !lv.ll_name.is_null() {
        if *end as ::core::ffi::c_int != NUL {
            semsg(
                gettext(if lv.ll_name_len == 0 as size_t {
                    &raw const e_invarg2 as *const ::core::ffi::c_char
                } else {
                    &raw const e_trailing_arg as *const ::core::ffi::c_char
                }),
                end,
            );
        } else if lv.ll_tv.is_null() {
            let mut di: *mut dictitem_T = find_var(
                lv.ll_name,
                lv.ll_name_len,
                ::core::ptr::null_mut::<*mut hashtab_T>(),
                true_0,
            );
            if !di.is_null() {
                (*rettv).vval.v_number = ((*di).di_flags as ::core::ffi::c_int
                    & DI_FLAGS_LOCK as ::core::ffi::c_int
                    != 0
                    || tv_islocked(&raw mut (*di).di_tv) as ::core::ffi::c_int != 0)
                    as ::core::ffi::c_int as varnumber_T;
            }
        } else if lv.ll_range {
            emsg(gettext(
                b"E786: Range not allowed\0".as_ptr() as *const ::core::ffi::c_char
            ));
        } else if !lv.ll_newkey.is_null() {
            semsg(
                gettext(&raw const e_dictkey as *const ::core::ffi::c_char),
                lv.ll_newkey,
            );
        } else if !lv.ll_list.is_null() {
            (*rettv).vval.v_number = tv_islocked(&raw mut (*lv.ll_li).li_tv) as varnumber_T;
        } else {
            (*rettv).vval.v_number = tv_islocked(&raw mut (*lv.ll_di).di_tv) as varnumber_T;
        }
    }
    clear_lval(&raw mut lv);
}
pub unsafe extern "C" fn f_id(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let len: ::core::ffi::c_int = vim_vsnprintf_typval(
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        0 as size_t,
        b"%p\0".as_ptr() as *const ::core::ffi::c_char,
        (*dummy_ap.ptr()).clone(),
        argvars,
    );
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string =
        xmalloc((len as size_t).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    vim_vsnprintf_typval(
        (*rettv).vval.v_string,
        (len as size_t).wrapping_add(1 as size_t),
        b"%p\0".as_ptr() as *const ::core::ffi::c_char,
        (*dummy_ap.ptr()).clone(),
        argvars,
    );
}
