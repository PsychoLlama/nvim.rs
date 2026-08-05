//! `:redir => var` -- capturing messages into a variable.
//!
//! `var_redir_start` resolves the target once and seeds it, `var_redir_str`
//! appends every message to a growable buffer, and `var_redir_stop` stores
//! the result.  `assert_error` is the same trick for `v:errors`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn assert_error(mut gap: *mut garray_T) {
    unsafe {
        let mut tv: *mut typval_T = get_vim_var_tv(VV_ERRORS);
        if (*tv).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*tv).vval.v_list.is_null()
        {
            set_vim_var_list(VV_ERRORS, tv_list_alloc(1 as ptrdiff_t));
        }
        tv_list_append_string(
            get_vim_var_list(VV_ERRORS),
            (*gap).ga_data as *const ::core::ffi::c_char,
            (*gap).ga_len as ssize_t,
        );
    }
}

static redir_lval: GlobalCell<*mut lval_T> = GlobalCell::new(::core::ptr::null_mut::<lval_T>());

static redir_ga: GlobalCell<garray_T> = GlobalCell::new(garray_T {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: 0,
    ga_growsize: 0,
    ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
});

static redir_endp: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());

static redir_varname: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());

pub unsafe extern "C" fn var_redir_start(
    mut name: *mut ::core::ffi::c_char,
    mut append: bool,
) -> ::core::ffi::c_int {
    unsafe {
        if !eval_isnamec1(*name as ::core::ffi::c_int) {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return FAIL;
        }
        redir_varname.set(xstrdup(name));
        redir_lval.set(xcalloc(1 as size_t, ::core::mem::size_of::<lval_T>()) as *mut lval_T);
        ga_init(
            redir_ga.ptr(),
            ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
            500 as ::core::ffi::c_int,
        );
        redir_endp.set(get_lval(
            redir_varname.get(),
            ::core::ptr::null_mut::<typval_T>(),
            redir_lval.get(),
            false_0 != 0,
            false_0 != 0,
            0 as ::core::ffi::c_int,
            FNE_CHECK_START,
        ));
        if (*redir_endp.ptr()).is_null()
            || (*redir_lval.get()).ll_name.is_null()
            || *redir_endp.get() as ::core::ffi::c_int != NUL
        {
            clear_lval(redir_lval.get());
            if !(*redir_endp.ptr()).is_null() && *redir_endp.get() as ::core::ffi::c_int != NUL {
                semsg(
                    gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                    redir_endp.get(),
                );
            } else {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    name,
                );
            }
            redir_endp.set(::core::ptr::null_mut::<::core::ffi::c_char>());
            var_redir_stop();
            return FAIL;
        }
        let called_emsg_before: ::core::ffi::c_int = called_emsg.get();
        did_emsg.set(false_0);
        let mut tv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        tv.v_type = VAR_STRING;
        tv.vval.v_string = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        if append {
            set_var_lval(
                redir_lval.get(),
                redir_endp.get(),
                &raw mut tv,
                true_0 != 0,
                false_0 != 0,
                b".\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else {
            set_var_lval(
                redir_lval.get(),
                redir_endp.get(),
                &raw mut tv,
                true_0 != 0,
                false_0 != 0,
                b"=\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        clear_lval(redir_lval.get());
        if called_emsg.get() > called_emsg_before {
            redir_endp.set(::core::ptr::null_mut::<::core::ffi::c_char>());
            var_redir_stop();
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn var_redir_str(
    mut value: *const ::core::ffi::c_char,
    mut value_len: ::core::ffi::c_int,
) {
    unsafe {
        if (*redir_lval.ptr()).is_null() {
            return;
        }
        let mut len: ::core::ffi::c_int = 0;
        if value_len == -1 as ::core::ffi::c_int {
            len = strlen(value) as ::core::ffi::c_int;
        } else {
            len = value_len;
        }
        ga_grow(redir_ga.ptr(), len);
        memmove(
            ((*redir_ga.ptr()).ga_data as *mut ::core::ffi::c_char)
                .offset((*redir_ga.ptr()).ga_len as isize) as *mut ::core::ffi::c_void,
            value as *const ::core::ffi::c_void,
            len as size_t,
        );
        (*redir_ga.ptr()).ga_len += len;
    }
}

pub unsafe extern "C" fn var_redir_stop() {
    unsafe {
        if !(*redir_lval.ptr()).is_null() {
            if !(*redir_endp.ptr()).is_null() {
                ga_append(redir_ga.ptr(), NUL as uint8_t);
                let mut tv: typval_T = typval_T {
                    v_type: VAR_UNKNOWN,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union { v_number: 0 },
                };
                tv.v_type = VAR_STRING;
                tv.vval.v_string = (*redir_ga.ptr()).ga_data as *mut ::core::ffi::c_char;
                redir_endp.set(get_lval(
                    redir_varname.get(),
                    ::core::ptr::null_mut::<typval_T>(),
                    redir_lval.get(),
                    false_0 != 0,
                    false_0 != 0,
                    0 as ::core::ffi::c_int,
                    FNE_CHECK_START,
                ));
                if !(*redir_endp.ptr()).is_null() && !(*redir_lval.get()).ll_name.is_null() {
                    set_var_lval(
                        redir_lval.get(),
                        redir_endp.get(),
                        &raw mut tv,
                        false_0 != 0,
                        false_0 != 0,
                        b".\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
                clear_lval(redir_lval.get());
            }
            let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut (*redir_ga.ptr()).ga_data;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                redir_lval.ptr() as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL;
            let _ = *ptr__0;
        }
        let mut ptr__1: *mut *mut ::core::ffi::c_void =
            redir_varname.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__1);
        *ptr__1 = NULL;
        let _ = *ptr__1;
    }
}
