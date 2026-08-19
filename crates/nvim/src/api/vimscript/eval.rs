//! Evaluating an expression and calling a function.
//!
//! [`nvim_eval`] runs one expression through `eval0` and converts the resulting
//! typval to an api Object.  [`call_function_with`] is the shared call path for
//! [`nvim_call_function`] and [`nvim_call_dict_function`], which differ in
//! whether the function is looked up in a dictionary -- itself given either as
//! a value or as an expression to evaluate first.
//!
//! `recursive` is why the three abort/throw flags are only reset by an
//! outermost call: an API call made *from* Vimscript that was itself called
//! from an API call must not clear the state its caller is unwinding through.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::types::{FAIL, OK};
use core::ffi::{c_char, c_int};
use core::ptr;

/// Clear the abort/throw state, but only for a call that is not nested inside
/// another one. The returned guard counts the nesting back down.
fn enter_recursive(recursive: &'static GlobalCell<c_int>) -> RecursionGuard {
    if recursive.get() == 0 {
        force_abort.set(false);
        suppress_errthrow.set(false);
        did_throw.set(false);
        did_emsg.set(0);
    }
    recursive.set(recursive.get() + 1);
    RecursionGuard(recursive)
}

struct RecursionGuard(&'static GlobalCell<c_int>);

impl Drop for RecursionGuard {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1);
    }
}

pub unsafe extern "C" fn nvim_eval(expr: String_0, arena: *mut Arena, err: *mut Error) -> Object {
    unsafe {
        static recursive: GlobalCell<c_int> = GlobalCell::new(0);
        let mut rv = Object::NIL;
        let _nesting = enter_recursive(&recursive);
        let mut rettv: typval_T = TV_INITIAL_VALUE;
        let mut tstate: TryState = TRY_STATE_INIT;
        try_enter(&raw mut tstate);
        let ok = eval0(
            expr.data,
            &raw mut rettv,
            ptr::null_mut::<exarg_T>(),
            EVALARG_EVALUATE.ptr(),
        );
        clear_evalarg(EVALARG_EVALUATE.ptr(), ptr::null_mut::<exarg_T>());
        try_leave(&raw mut tstate, err);
        if (*err).type_0 == kErrorTypeNone {
            if ok == FAIL {
                // The expression is quoted back at the user, capped so a huge
                // one does not become the whole message.
                api_set_error(
                    err,
                    kErrorTypeException,
                    c"Failed to evaluate expression: '%.*s'".as_ptr(),
                    256 as c_int,
                    expr.data,
                );
            } else {
                rv = vim_to_object(&raw mut rettv, arena, false);
            }
        }
        tv_clear(&raw mut rettv);
        rv
    }
}

/// Call `fn_0` with `args`, optionally as a method of `self_0`.
unsafe fn call_function_with(
    fn_0: String_0,
    args: Array,
    self_0: *mut dict_T,
    arena: *mut Arena,
    err: *mut Error,
) -> Object {
    unsafe {
        static recursive: GlobalCell<c_int> = GlobalCell::new(0);
        if args.size > MAX_FUNC_ARGS as size_t {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"Function called with too many arguments".as_ptr(),
            );
            return Object::NIL;
        }
        // MAX_FUNC_ARGS + 1: `call_func` reads one past the last argument.
        let mut vim_args: [typval_T; 21] = [TV_INITIAL_VALUE; 21];
        for i in 0..args.size {
            object_to_vim(*args.items.add(i), &raw mut vim_args[i], err);
        }

        let mut rv = Object::NIL;
        {
            let _nesting = enter_recursive(&recursive);
            let mut rettv: typval_T = TV_INITIAL_VALUE;
            let mut funcexe: funcexe_T = FUNCEXE_INIT;
            funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
            funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
            funcexe.fe_evaluate = true;
            funcexe.fe_selfdict = self_0;
            let mut tstate: TryState = TRY_STATE_INIT;
            try_enter(&raw mut tstate);
            call_func(
                fn_0.data,
                fn_0.size as c_int,
                &raw mut rettv,
                args.size as c_int,
                vim_args.as_mut_ptr(),
                &raw mut funcexe,
            );
            try_leave(&raw mut tstate, err);
            if (*err).type_0 == kErrorTypeNone {
                rv = vim_to_object(&raw mut rettv, arena, false);
            }
            tv_clear(&raw mut rettv);
        }
        // Converted arguments are cleared in reverse, as the C did.
        for i in (0..args.size).rev() {
            tv_clear(&raw mut vim_args[i]);
        }
        rv
    }
}

pub unsafe extern "C" fn nvim_call_function(
    fn_0: String_0,
    args: Array,
    arena: *mut Arena,
    err: *mut Error,
) -> Object {
    unsafe { call_function_with(fn_0, args, ptr::null_mut::<dict_T>(), arena, err) }
}

pub unsafe extern "C" fn nvim_call_dict_function(
    dict: Object,
    mut fn_0: String_0,
    args: Array,
    arena: *mut Arena,
    err: *mut Error,
) -> Object {
    unsafe {
        let mut rettv: typval_T = TV_INITIAL_VALUE;
        // Only the evaluated form owns what it produced.
        let mut mustfree = false;
        match dict.type_0 {
            kObjectTypeString => {
                let mut tstate: TryState = TRY_STATE_INIT;
                try_enter(&raw mut tstate);
                let eval_ret = eval0(
                    dict.data.string.data,
                    &raw mut rettv,
                    ptr::null_mut::<exarg_T>(),
                    EVALARG_EVALUATE.ptr(),
                );
                clear_evalarg(EVALARG_EVALUATE.ptr(), ptr::null_mut::<exarg_T>());
                try_leave(&raw mut tstate, err);
                if (*err).type_0 != kErrorTypeNone {
                    return Object::NIL;
                }
                if eval_ret != OK {
                    abort();
                }
                mustfree = true;
            }
            kObjectTypeDict => object_to_vim(dict, &raw mut rettv, err),
            _ => {
                api_err_exp(
                    err,
                    c"dict argument".as_ptr(),
                    c"String or Dict".as_ptr(),
                    ptr::null::<c_char>(),
                );
                return Object::NIL;
            }
        }
        let self_dict: *mut dict_T = rettv.vval.v_dict;
        let rv = call_in_dict(&mut fn_0, dict, args, self_dict, &rettv, arena, err);
        if mustfree {
            tv_clear(&raw mut rettv);
        }
        rv
    }
}

/// The tail of [`nvim_call_dict_function`]: resolve `fn_0` inside `self_dict`
/// when it was named rather than given, then call it.
#[allow(clippy::too_many_arguments)]
unsafe fn call_in_dict(
    fn_0: &mut String_0,
    dict: Object,
    args: Array,
    self_dict: *mut dict_T,
    rettv: &typval_T,
    arena: *mut Arena,
    err: *mut Error,
) -> Object {
    unsafe {
        if rettv.v_type != VAR_DICT || self_dict.is_null() {
            api_set_error(err, kErrorTypeValidation, c"dict not found".as_ptr());
            return Object::NIL;
        }
        // A Dict argument was converted whole, so its function member is
        // already `fn_0`; a String argument named a dictionary to look in.
        if !fn_0.data.is_null() && fn_0.size > 0 && dict.type_0 != kObjectTypeDict {
            let di: *mut dictitem_T = tv_dict_find(self_dict, fn_0.data, fn_0.size as ptrdiff_t);
            if di.is_null() {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"Not found: %s".as_ptr(),
                    fn_0.data,
                );
                return Object::NIL;
            }
            if (*di).di_tv.v_type == VAR_PARTIAL {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"partial function not supported".as_ptr(),
                );
                return Object::NIL;
            }
            if (*di).di_tv.v_type != VAR_FUNC {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"Not a function: %s".as_ptr(),
                    fn_0.data,
                );
                return Object::NIL;
            }
            *fn_0 = String_0 {
                data: (*di).di_tv.vval.v_string,
                size: strlen((*di).di_tv.vval.v_string),
            };
        }
        if fn_0.data.is_null() || fn_0.size < 1 {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"Invalid function name: %s".as_ptr(),
                c"(empty)".as_ptr(),
            );
            return Object::NIL;
        }
        call_function_with(*fn_0, args, self_dict, arena, err)
    }
}
