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
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::api::private::helpers::{Reported, api_try};
use crate::api::private::validate::err_expected;
use crate::api_error;
use crate::cstr;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::message_fmt::{c_str, c_str_len};
use crate::narrow::len_as_int;
use crate::winlayer::Win;
use core::ffi::c_int;
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

/// # Safety
///
/// `expr` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `arena` must point at a live arena, which the memory this
/// answers with is taken from and must outlive.
pub unsafe fn nvim_eval(expr: String_0) -> Result<Object, Error> {
    static recursive: GlobalCell<c_int> = GlobalCell::new(0);
    let mut evalarg = EVALARG_EVALUATE;
    let _nesting = enter_recursive(&recursive);
    let mut rettv: TypVal = TV_INITIAL_VALUE;
    let evaluated = api_try(|| {
        let no_eap = ptr::null_mut::<ExArg>();
        let ea = &raw mut evalarg;
        // SAFETY: `expr` names its own bytes, and `evalarg` is this frame's.
        let ok = unsafe { eval0(expr.data(), &mut rettv, no_eap, ea) };
        // SAFETY: `evalarg` is this frame's.
        unsafe { clear_evalarg(ea, no_eap) };
        ok
    });
    // A thrown exception outranks the generic message, and `rettv` is cleared
    // whichever way this went -- so the answer is held rather than returned
    // from inside the match.
    let answer = match evaluated {
        Err(caught) => Err(caught),
        Ok(Err(_)) => {
            // The expression is quoted back at the user, capped so a huge
            // one does not become the whole message. Upstream's `%.*s` stops
            // at the terminator as well as the cap, which is what the `min`
            // is: `expr` need not hold 256 bytes.
            let shown = expr.len().min(256);
            // SAFETY: `expr` names its own bytes, per this call's contract.
            let text = unsafe { c_str_len(expr.data(), shown) }.null_as_empty();
            Err(api_error!(
                kErrorTypeException,
                "Failed to evaluate expression: '{text}'"
            ))
        }
        // SAFETY: `rettv` is this frame's and `arena` the caller's.
        Ok(Ok(())) => Ok(Object::from(&rettv)),
    };
    // SAFETY: `rettv` is this frame's.
    unsafe { tv_clear(&mut rettv) };
    answer
}

/// Call `fn_0` with `args`, optionally as a method of `self_0`.
///
/// # Safety
/// `fn_0`/`args` must name their own storage, `self_0` must be null or a
/// live dictionary.
unsafe fn call_function_with(
    fn_0: String_0,
    args: Array,
    self_0: *mut Dict,
) -> Result<Object, Error> {
    static recursive: GlobalCell<c_int> = GlobalCell::new(0);
    if args.len() > MAX_FUNC_ARGS as size_t {
        return Err(Error::validation(
            c"Function called with too many arguments",
        ));
    }
    let mut vim_args = [TV_INITIAL_VALUE; MAX_FUNC_ARGS as usize];
    for (i, slot) in vim_args[..args.len()].iter_mut().enumerate() {
        // SAFETY: `i` is below `size`, so the object is inside `items`; the
        // slot is this frame's and `err` the caller's.
        *slot = TypVal::from(&args[i]);
    }

    let rv;
    {
        let _nesting = enter_recursive(&recursive);
        let mut rettv: TypVal = TV_INITIAL_VALUE;
        let mut funcexe: FuncExe = FUNCEXE_INIT;
        let lnum = Win::current().w_cursor.lnum;
        funcexe.fe_firstline = lnum;
        funcexe.fe_lastline = lnum;
        funcexe.fe_evaluate = true;
        funcexe.fe_selfdict = self_0;
        let mut tstate: TryState = TRY_STATE_INIT;
        // SAFETY: `tstate` is this frame's, live until the `try_leave`
        // below.
        unsafe { try_enter(&raw mut tstate) };
        let (name, name_len) = (fn_0.data(), len_as_int(fn_0.len()));
        let argv = &vim_args[..args.len()];
        let (ret, fe) = (&raw mut rettv, &raw mut funcexe);
        // SAFETY: `name` names `name_len` bytes and `rettv`/`funcexe` are
        // this frame's.
        let _ = unsafe { call_func(name, name_len, &mut rettv, argv, fe) };
        // SAFETY: `tstate` is what the `try_enter` above filled in.
        rv = match unsafe { try_leave(&raw mut tstate) } {
            // SAFETY: `rettv` is this frame's and `arena` the caller's.
            // SAFETY: `ret` is this frame's return slot.
            Ok(()) => Ok(Object::from(unsafe { &*ret })),
            Err(e) => Err(e),
        };
        // SAFETY: `rettv` is this frame's.
        unsafe { tv_clear(&mut *ret) };
    }
    rv
}

/// # Safety
///
/// `fn_0` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `args` must be a well-formed API array, its `size`
/// elements initialized. `arena` must point at a live arena, which the memory
/// this answers with is taken from and must outlive.
pub unsafe fn nvim_call_function(fn_0: String_0, args: Array) -> Result<Object, Error> {
    // SAFETY: `fn_0`/`args`/`arena` are the caller's; a null self dictionary
    // means a plain function call.
    unsafe { call_function_with(fn_0, args, ptr::null_mut::<Dict>()) }
}

/// # Safety
///
/// `dict` must be a well-formed API object the caller owns for the call. `mut
/// fn_0` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `args` must be a well-formed API array, its `size`
/// elements initialized. `arena` must point at a live arena, which the memory
/// this answers with is taken from and must outlive.
pub unsafe fn nvim_call_dict_function(
    dict: Object,
    mut fn_0: String_0,
    args: Array,
) -> Result<Object, Error> {
    let mut evalarg = EVALARG_EVALUATE;
    let mut error = Error::none();
    let mut rettv: TypVal = TV_INITIAL_VALUE;
    // Only the evaluated form owns what it produced.
    let mut mustfree = false;
    let mut dict_given = false;
    if let Some(expr) = dict.as_string() {
        let mut tstate: TryState = TRY_STATE_INIT;
        // SAFETY: `tstate` is this frame's, live until the `try_leave`
        // below.
        unsafe { try_enter(&raw mut tstate) };
        let no_eap = ptr::null_mut::<ExArg>();
        let ea = &raw mut evalarg;
        // SAFETY: `expr` names its own bytes, and `evalarg` is this frame's.
        let eval_ret = unsafe { eval0(expr.data(), &mut rettv, no_eap, ea) };
        // SAFETY: `evalarg` is this frame's.
        unsafe { clear_evalarg(ea, no_eap) };
        // SAFETY: `tstate` is what the `try_enter` above filled in.
        error.absorb(unsafe { try_leave(&raw mut tstate) });
        if error.is_set() {
            return Object::Nil.reported(error);
        }
        if eval_ret.is_err() {
            // `eval0` answers `FAIL` only by throwing, which `try_leave`
            // would have turned into an error.
            // SAFETY: `abort` takes nothing.
            unsafe { abort() };
        }
        mustfree = true;
    } else if matches!(dict, Object::Dict(_)) {
        dict_given = true;
        rettv = TypVal::from(dict);
    } else {
        let want = c"String or Dict";
        // SAFETY: `error` is this frame's slot and both strings are static.
        error = err_expected(c"dict argument", want, None);
        return Object::Nil.reported(error);
    }
    // A non-dictionary answers NULL, which `call_in_dict` reads as "no
    // `self`".
    let self_dict: *mut Dict = rettv.dict_or_null();
    // SAFETY: `rettv` is this frame's, and `fn_0`/`args`/`arena` are the
    // caller's.
    let rv = unsafe { call_in_dict(&mut fn_0, dict_given, args, self_dict, &rettv) };
    if mustfree {
        // SAFETY: the evaluated value is this frame's.
        unsafe { tv_clear(&mut rettv) };
    }
    rv
}

/// The tail of [`nvim_call_dict_function`]: resolve `fn_0` inside `self_dict`
/// when it was named rather than given, then call it.
///
/// # Safety
/// `self_dict` must be null or the dictionary `result` holds.
unsafe fn call_in_dict(
    fn_0: &mut String_0,
    dict_given: bool,
    args: Array,
    self_dict: *mut Dict,
    result: &TypVal,
) -> Result<Object, Error> {
    if result.v_type() != VAR_DICT || self_dict.is_null() {
        return Err(Error::validation(c"dict not found"));
    }
    // A Dict argument was converted whole, so its function member is
    // already `fn_0`; a String argument named a dictionary to look in.
    if !fn_0.data().is_null() && !fn_0.is_empty() && !dict_given {
        // SAFETY: `self_dict` is live and `fn_0` names its own bytes.
        let len: ptrdiff_t = fn_0.len().cast_signed();
        let di: *mut DictItem = unsafe { tv_dict_find(self_dict, fn_0.data(), len) };
        if di.is_null() {
            // SAFETY: `fn_0` names its own NUL-terminated bytes.
            let name = unsafe { c_str(fn_0.data()) };
            return Err(api_error!(kErrorTypeValidation, "Not found: {name}"));
        }
        // SAFETY: the lookup answered a live item of `self_dict`.
        let v_type = unsafe { (*di).di_tv.v_type() };
        if v_type == VAR_PARTIAL {
            return Err(Error::validation(c"partial function not supported"));
        }
        if v_type != VAR_FUNC {
            // SAFETY: `fn_0` names its own NUL-terminated bytes.
            let name = unsafe { c_str(fn_0.data()) };
            return Err(api_error!(kErrorTypeValidation, "Not a function: {name}"));
        }
        // SAFETY: a `VAR_FUNC` carries a NUL-terminated function name.
        let name = unsafe { (*di).di_tv.func_name_or_null() };
        // SAFETY: as above.
        *fn_0 = String_0::from_bytes(unsafe { cstr::bytes_at(name) });
    }
    if fn_0.data().is_null() || fn_0.is_empty() {
        return Err(Error::validation(c"Invalid function name: (empty)"));
    }
    // SAFETY: `fn_0` names its own bytes and `self_dict` is the live
    // dictionary the call is a method of.
    unsafe { call_function_with(fn_0.clone(), args, self_dict) }
}
