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
use crate::api::private::helpers::{ERROR_INIT, Reported, api_try};
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::types::{FAIL, OK};
use core::ffi::{CStr, c_char, c_int};
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

pub unsafe fn nvim_eval(expr: String_0, arena: *mut Arena) -> Result<Object, Error> {
    static recursive: GlobalCell<c_int> = GlobalCell::new(0);
    let mut evalarg = EVALARG_EVALUATE;
    let mut error = ERROR_INIT;
    let mut rv = Object::NIL;
    let _nesting = enter_recursive(&recursive);
    let mut rettv: typval_T = TV_INITIAL_VALUE;
    let ok = api_try(&mut error, |_| {
        let no_eap = ptr::null_mut::<exarg_T>();
        let (ret, ea) = (&raw mut rettv, &raw mut evalarg);
        // SAFETY: `expr` names its own bytes, and `rettv`/`evalarg` are
        // this frame's.
        let ok = unsafe { eval0(expr.data(), ret, no_eap, ea) };
        // SAFETY: `evalarg` is this frame's.
        unsafe { clear_evalarg(ea, no_eap) };
        ok
    });
    if error.type_0 == kErrorTypeNone {
        if ok == FAIL {
            // The expression is quoted back at the user, capped so a huge
            // one does not become the whole message.
            let fmt = c"Failed to evaluate expression: '%.*s'".as_ptr();
            let (cap, text) = (256 as c_int, expr.data());
            // SAFETY: `error` is this frame's slot; the format takes the
            // length and the string it is given.
            unsafe { api_set_error(&raw mut error, kErrorTypeException, fmt, cap, text) };
        } else {
            // SAFETY: `rettv` is this frame's and `arena` the caller's.
            rv = unsafe { vim_to_object(&raw mut rettv, arena, false) };
        }
    }
    // SAFETY: `rettv` is this frame's.
    unsafe { tv_clear(&raw mut rettv) };
    rv.reported(error)
}

/// Call `fn_0` with `args`, optionally as a method of `self_0`.
///
/// # Safety
/// `fn_0`/`args` must name their own storage, `self_0` must be null or a
/// live dictionary, and `err` must be the caller's error slot.
unsafe fn call_function_with(
    fn_0: String_0,
    args: Array,
    self_0: *mut dict_T,
    arena: *mut Arena,
    err: *mut Error,
) -> Object {
    static recursive: GlobalCell<c_int> = GlobalCell::new(0);
    if args.size > MAX_FUNC_ARGS as size_t {
        let msg = c"Function called with too many arguments".as_ptr();
        // SAFETY: the caller's promise about `err`.
        unsafe { api_set_error(err, kErrorTypeValidation, msg) };
        return Object::NIL;
    }
    // MAX_FUNC_ARGS + 1: `call_func` reads one past the last argument.
    let mut vim_args: [typval_T; 21] = [TV_INITIAL_VALUE; 21];
    for (i, slot) in vim_args[..args.size].iter_mut().enumerate() {
        // SAFETY: `i` is below `size`, so the object is inside `items`; the
        // slot is this frame's and `err` the caller's.
        unsafe { object_to_vim(*args.items.add(i), slot, err) };
    }

    let mut rv = Object::NIL;
    {
        let _nesting = enter_recursive(&recursive);
        let mut rettv: typval_T = TV_INITIAL_VALUE;
        let mut funcexe: funcexe_T = FUNCEXE_INIT;
        // SAFETY: there is always a current window.
        let lnum = unsafe { (*curwin.get()).w_cursor.lnum };
        funcexe.fe_firstline = lnum;
        funcexe.fe_lastline = lnum;
        funcexe.fe_evaluate = true;
        funcexe.fe_selfdict = self_0;
        let mut tstate: TryState = TRY_STATE_INIT;
        // SAFETY: `tstate` is this frame's, live until the `try_leave`
        // below.
        unsafe { try_enter(&raw mut tstate) };
        let (name, name_len) = (fn_0.data(), fn_0.len() as c_int);
        let (argc, argv) = (args.size as c_int, vim_args.as_mut_ptr());
        let (ret, fe) = (&raw mut rettv, &raw mut funcexe);
        // SAFETY: `name` names `name_len` bytes, `argv` holds `argc`
        // converted arguments, and `rettv`/`funcexe` are this frame's.
        unsafe { call_func(name, name_len, ret, argc, argv, fe) };
        // SAFETY: `tstate` is what the `try_enter` above filled in, and
        // `err` is the caller's slot.
        unsafe { try_leave(&raw mut tstate, err) };
        // SAFETY: the caller's promise about `err`.
        if unsafe { (*err).type_0 } == kErrorTypeNone {
            // SAFETY: `rettv` is this frame's and `arena` the caller's.
            rv = unsafe { vim_to_object(ret, arena, false) };
        }
        // SAFETY: `rettv` is this frame's.
        unsafe { tv_clear(ret) };
    }
    // Converted arguments are cleared in reverse, as the C did.
    for i in (0..args.size).rev() {
        // SAFETY: the slot is this frame's array.
        unsafe { tv_clear(&raw mut vim_args[i]) };
    }
    rv
}

pub unsafe fn nvim_call_function(
    fn_0: String_0,
    args: Array,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    // SAFETY: `fn_0`/`args`/`arena` are the caller's, and `err` this
    // frame's slot; a null self dictionary means a plain function call.
    let rv = unsafe { call_function_with(fn_0, args, ptr::null_mut::<dict_T>(), arena, err) };
    rv.reported(error)
}

pub unsafe fn nvim_call_dict_function(
    dict: Object,
    mut fn_0: String_0,
    args: Array,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut evalarg = EVALARG_EVALUATE;
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut rettv: typval_T = TV_INITIAL_VALUE;
    // Only the evaluated form owns what it produced.
    let mut mustfree = false;
    if let Some(expr) = dict.as_string() {
        let mut tstate: TryState = TRY_STATE_INIT;
        // SAFETY: `tstate` is this frame's, live until the `try_leave`
        // below.
        unsafe { try_enter(&raw mut tstate) };
        let no_eap = ptr::null_mut::<exarg_T>();
        let (ret, ea) = (&raw mut rettv, &raw mut evalarg);
        // SAFETY: `expr` names its own bytes, and `rettv`/`evalarg` are
        // this frame's.
        let eval_ret = unsafe { eval0(expr.data(), ret, no_eap, ea) };
        // SAFETY: `evalarg` is this frame's.
        unsafe { clear_evalarg(ea, no_eap) };
        // SAFETY: `tstate` is what the `try_enter` above filled in.
        unsafe { try_leave(&raw mut tstate, err) };
        if error.type_0 != kErrorTypeNone {
            return Object::NIL.reported(error);
        }
        if eval_ret != OK {
            // `eval0` answers `FAIL` only by throwing, which `try_leave`
            // would have turned into an error.
            // SAFETY: `abort` takes nothing.
            unsafe { abort() };
        }
        mustfree = true;
    } else if dict.type_0 == kObjectTypeDict {
        // SAFETY: `dict` is the caller's and `rettv`/`err` are this frame's.
        unsafe { object_to_vim(dict, &raw mut rettv, err) };
    } else {
        let want = c"String or Dict".as_ptr();
        // SAFETY: `err` is this frame's slot and both strings are static.
        unsafe { api_err_exp(err, c"dict argument".as_ptr(), want, ptr::null::<c_char>()) };
        return Object::NIL.reported(error);
    }
    // SAFETY: `rettv` is this frame's; a non-dictionary leaves the union's
    // pointer arm holding whatever the value was, which `call_in_dict`
    // refuses after checking `v_type`.
    let self_dict: *mut dict_T = unsafe { rettv.vval.v_dict };
    // SAFETY: as above, plus `fn_0`/`args`/`arena` are the caller's.
    let rv = unsafe { call_in_dict(&mut fn_0, dict, args, self_dict, &rettv, arena, err) };
    if mustfree {
        // SAFETY: the evaluated value is this frame's.
        unsafe { tv_clear(&raw mut rettv) };
    }
    rv.reported(error)
}

/// The tail of [`nvim_call_dict_function`]: resolve `fn_0` inside `self_dict`
/// when it was named rather than given, then call it.
///
/// # Safety
/// `self_dict` must be null or the dictionary `rettv` holds, and `err` must
/// be the caller's error slot.
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
    // Every refusal below is a validation error, with or without the name
    // it is about.
    // SAFETY: the caller's promise about `err`; the message takes nothing.
    let refuse = |msg: &CStr| unsafe { api_set_error(err, kErrorTypeValidation, msg.as_ptr()) };
    // SAFETY: as above; the format takes the one C string it is given.
    let refuse_named = |fmt: &CStr, name: *const c_char| unsafe {
        api_set_error(err, kErrorTypeValidation, fmt.as_ptr(), name);
    };

    if rettv.v_type != VAR_DICT || self_dict.is_null() {
        refuse(c"dict not found");
        return Object::NIL;
    }
    // A Dict argument was converted whole, so its function member is
    // already `fn_0`; a String argument named a dictionary to look in.
    if !fn_0.data().is_null() && !fn_0.is_empty() && dict.type_0 != kObjectTypeDict {
        // SAFETY: `self_dict` is live and `fn_0` names its own bytes.
        let di: *mut dictitem_T =
            unsafe { tv_dict_find(self_dict, fn_0.data(), fn_0.len() as ptrdiff_t) };
        if di.is_null() {
            refuse_named(c"Not found: %s", fn_0.data());
            return Object::NIL;
        }
        // SAFETY: the lookup answered a live item of `self_dict`.
        let v_type = unsafe { (*di).di_tv.v_type };
        if v_type == VAR_PARTIAL {
            refuse(c"partial function not supported");
            return Object::NIL;
        }
        if v_type != VAR_FUNC {
            refuse_named(c"Not a function: %s", fn_0.data());
            return Object::NIL;
        }
        // SAFETY: a `VAR_FUNC` carries a NUL-terminated function name.
        let name = unsafe { (*di).di_tv.vval.v_string };
        // SAFETY: as above.
        *fn_0 = String_0::from_raw_parts(name, unsafe { strlen(name) });
    }
    if fn_0.data().is_null() || fn_0.is_empty() {
        refuse_named(c"Invalid function name: %s", c"(empty)".as_ptr());
        return Object::NIL;
    }
    // SAFETY: `fn_0` names its own bytes and `self_dict` is the live
    // dictionary the call is a method of.
    unsafe { call_function_with(*fn_0, args, self_dict, arena, err) }
}
