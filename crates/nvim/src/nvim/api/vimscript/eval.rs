//! Evaluating an expression and calling a function.
//!
//! `nvim_eval` runs one expression through `eval0` and converts the resulting
//! typval to an api Object.  `_call_function` is the shared call path for
//! `nvim_call_function` and `nvim_call_dict_function`, which differ in whether
//! the function is looked up in a dictionary -- itself given either as a
//! value or as an expression to evaluate first.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn nvim_eval(
    mut expr: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        static recursive: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        let mut rv: Object = object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
        if recursive.get() == 0 {
            force_abort.set(false);
            suppress_errthrow.set(false);
            did_throw.set(false);
            did_emsg.set(false_0);
        }
        (*recursive.ptr()) += 1;
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut ok: ::core::ffi::c_int = 0;
        let mut tstate: TryState = TryState {
            current_exception: ::core::ptr::null_mut::<except_T>(),
            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
            msg_list: ::core::ptr::null::<*const msglist_T>(),
            got_int: 0,
            did_throw: false,
            need_rethrow: 0,
            did_emsg: 0,
        };
        try_enter(&raw mut tstate);
        ok = eval0(
            expr.data,
            &raw mut rettv,
            ::core::ptr::null_mut::<exarg_T>(),
            EVALARG_EVALUATE.ptr(),
        );
        clear_evalarg(EVALARG_EVALUATE.ptr(), ::core::ptr::null_mut::<exarg_T>());
        try_leave(&raw mut tstate, err);
        if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
            if ok == FAIL {
                api_set_error(
                    err,
                    kErrorTypeException,
                    c"Failed to evaluate expression: '%.*s'".as_ptr(),
                    256 as ::core::ffi::c_int,
                    expr.data,
                );
            } else {
                rv = vim_to_object(&raw mut rettv, arena, false);
            }
        }
        tv_clear(&raw mut rettv);
        (*recursive.ptr()) -= 1;
        return rv;
    }
}

unsafe extern "C" fn _call_function(
    mut fn_0: String_0,
    mut args: Array,
    mut self_0: *mut dict_T,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        static recursive: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        let mut rv: Object = object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
        if args.size > MAX_FUNC_ARGS as ::core::ffi::c_int as size_t {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"Function called with too many arguments".as_ptr(),
            );
            return rv;
        }
        let mut vim_args: [typval_T; 21] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 21];
        let mut i: size_t = 0 as size_t;
        while i < args.size {
            object_to_vim(
                *args.items.offset(i as isize),
                (&raw mut vim_args as *mut typval_T).offset(i as isize),
                err,
            );
            i = i.wrapping_add(1);
        }
        if recursive.get() == 0 {
            force_abort.set(false);
            suppress_errthrow.set(false);
            did_throw.set(false);
            did_emsg.set(false_0);
        }
        (*recursive.ptr()) += 1;
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut funcexe: funcexe_T = FUNCEXE_INIT;
        funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_evaluate = true;
        funcexe.fe_selfdict = self_0;
        let mut tstate: TryState = TryState {
            current_exception: ::core::ptr::null_mut::<except_T>(),
            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
            msg_list: ::core::ptr::null::<*const msglist_T>(),
            got_int: 0,
            did_throw: false,
            need_rethrow: 0,
            did_emsg: 0,
        };
        try_enter(&raw mut tstate);
        call_func(
            fn_0.data,
            fn_0.size as ::core::ffi::c_int,
            &raw mut rettv,
            args.size as ::core::ffi::c_int,
            &raw mut vim_args as *mut typval_T,
            &raw mut funcexe,
        );
        try_leave(&raw mut tstate, err);
        if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
            rv = vim_to_object(&raw mut rettv, arena, false);
        }
        tv_clear(&raw mut rettv);
        (*recursive.ptr()) -= 1;
        while i > 0 as size_t {
            i = i.wrapping_sub(1);
            tv_clear((&raw mut vim_args as *mut typval_T).offset(i as isize));
        }
        return rv;
    }
}

pub unsafe extern "C" fn nvim_call_function(
    mut fn_0: String_0,
    mut args: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        return _call_function(fn_0, args, ::core::ptr::null_mut::<dict_T>(), arena, err);
    }
}

pub unsafe extern "C" fn nvim_call_dict_function(
    mut dict: Object,
    mut fn_0: String_0,
    mut args: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        let mut rv: Object = object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut mustfree: bool = false;
        match dict.type_0 as ::core::ffi::c_uint {
            4 => {
                let mut eval_ret: ::core::ffi::c_int = 0;
                let mut tstate: TryState = TryState {
                    current_exception: ::core::ptr::null_mut::<except_T>(),
                    private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
                    msg_list: ::core::ptr::null::<*const msglist_T>(),
                    got_int: 0,
                    did_throw: false,
                    need_rethrow: 0,
                    did_emsg: 0,
                };
                try_enter(&raw mut tstate);
                eval_ret = eval0(
                    dict.data.string.data,
                    &raw mut rettv,
                    ::core::ptr::null_mut::<exarg_T>(),
                    EVALARG_EVALUATE.ptr(),
                );
                clear_evalarg(EVALARG_EVALUATE.ptr(), ::core::ptr::null_mut::<exarg_T>());
                try_leave(&raw mut tstate, err);
                if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    return rv;
                }
                if eval_ret != OK {
                    abort();
                }
                mustfree = true;
            }
            6 => {
                object_to_vim(dict, &raw mut rettv, err);
            }
            _ => {
                if true {
                    api_err_exp(
                        err,
                        c"dict argument".as_ptr(),
                        c"String or Dict".as_ptr(),
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    return rv;
                }
            }
        }
        let mut self_dict: *mut dict_T = rettv.vval.v_dict;
        '_end: {
            if rettv.v_type as ::core::ffi::c_uint
                != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                || self_dict.is_null()
            {
                api_set_error(err, kErrorTypeValidation, c"dict not found".as_ptr());
            } else {
                if !fn_0.data.is_null()
                    && fn_0.size > 0 as size_t
                    && dict.type_0 as ::core::ffi::c_uint
                        != kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let di: *mut dictitem_T =
                        tv_dict_find(self_dict, fn_0.data, fn_0.size as ptrdiff_t);
                    if di.is_null() {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            c"Not found: %s".as_ptr(),
                            fn_0.data,
                        );
                        break '_end;
                    } else if (*di).di_tv.v_type as ::core::ffi::c_uint
                        == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            c"partial function not supported".as_ptr(),
                        );
                        break '_end;
                    } else if !((*di).di_tv.v_type as ::core::ffi::c_uint
                        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint)
                    {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            c"Not a function: %s".as_ptr(),
                            fn_0.data,
                        );
                        break '_end;
                    }
                    fn_0 = String_0 {
                        data: (*di).di_tv.vval.v_string,
                        size: strlen((*di).di_tv.vval.v_string),
                    };
                }
                if !(!fn_0.data.is_null() && fn_0.size >= 1 as size_t) {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        c"Invalid function name: %s".as_ptr(),
                        c"(empty)".as_ptr(),
                    );
                } else {
                    rv = _call_function(fn_0, args, self_dict, arena, err);
                }
            }
        }
        if mustfree {
            tv_clear(&raw mut rettv);
        }
        return rv;
    }
}
