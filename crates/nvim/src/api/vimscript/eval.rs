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

#![forbid(unsafe_code)]
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
use crate::eval::typval::DictRef;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::eval::userfunc::{CallWith, call_func_with};
use crate::message_fmt::{msg_bytes, msg_cstr};
use crate::winlayer::Win;
use core::ffi::CStr;
use core::ffi::c_int;

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

pub fn nvim_eval(expr: String_0) -> Result<Object, Error> {
    static recursive: GlobalCell<c_int> = GlobalCell::new(0);
    let _nesting = enter_recursive(&recursive);
    let mut rettv: TypVal = TV_INITIAL_VALUE;
    // The expression ends at its first NUL, as it did for the C.
    let text = expr.as_cstr().to_bytes();
    let evaluated = api_try(|| eval0(text, &mut rettv, true).0);
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
            let text = msg_bytes(&expr.as_bytes()[..shown]);
            Err(api_error!(
                kErrorTypeException,
                "Failed to evaluate expression: '{text}'"
            ))
        }
        Ok(Ok(())) => Ok(Object::from(&rettv)),
    };
    tv_clear(&mut rettv);
    answer
}

/// Call `fn_0` with `args`, optionally as a method of `self_dict`.
fn call_function_with(
    fn_0: &String_0,
    args: Array,
    self_dict: Option<&DictRef>,
) -> Result<Object, Error> {
    static recursive: GlobalCell<c_int> = GlobalCell::new(0);
    if args.len() > MAX_FUNC_ARGS as size_t {
        return Err(Error::validation(
            c"Function called with too many arguments",
        ));
    }
    let mut vim_args = [TV_INITIAL_VALUE; MAX_FUNC_ARGS as usize];
    for (i, slot) in vim_args[..args.len()].iter_mut().enumerate() {
        *slot = TypVal::from(&args[i]);
    }

    let _nesting = enter_recursive(&recursive);
    let mut rettv: TypVal = TV_INITIAL_VALUE;
    let lnum = Win::current().w_cursor.lnum;
    let with = CallWith {
        firstline: lnum,
        lastline: lnum,
        selfdict: self_dict,
        ..CallWith::new(true)
    };
    // The name is the API string's, up to its first NUL, as the C's copy of
    // it read.
    let argv = &vim_args[..args.len()];
    let called = api_try(|| call_func_with(fn_0.as_cstr(), None, &mut rettv, argv, with));
    let rv = called.map(|_| Object::from(&rettv));
    tv_clear(&mut rettv);
    rv
}

pub fn nvim_call_function(fn_0: String_0, args: Array) -> Result<Object, Error> {
    call_function_with(&fn_0, args, None)
}

pub fn nvim_call_dict_function(
    dict: Object,
    mut fn_0: String_0,
    args: Array,
) -> Result<Object, Error> {
    let mut rettv: TypVal = TV_INITIAL_VALUE;
    // Only the evaluated form owns what it produced.
    let mut mustfree = false;
    let mut dict_given = false;
    if let Some(expr) = dict.as_string() {
        // The expression ends at its first NUL, as it did for the C.
        let eval_ret = api_try(|| eval0(expr.as_cstr().to_bytes(), &mut rettv, true).0)?;
        if eval_ret.is_err() {
            // `eval0` answers `FAIL` only by throwing, which `api_try`
            // would have turned into an error.
            ::std::process::abort();
        }
        mustfree = true;
    } else if matches!(dict, Object::Dict(_)) {
        dict_given = true;
        rettv = TypVal::from(dict);
    } else {
        let want = c"String or Dict";
        return Object::Nil.reported(err_expected(c"dict argument", want, None));
    }
    let rv = call_in_dict(&mut fn_0, dict_given, args, &rettv);
    if mustfree {
        tv_clear(&mut rettv);
    }
    rv
}

/// The tail of [`nvim_call_dict_function`]: resolve `fn_0` inside the
/// dictionary `result` holds when it was named rather than given, then call
/// it.
fn call_in_dict(
    fn_0: &mut String_0,
    dict_given: bool,
    args: Array,
    result: &TypVal,
) -> Result<Object, Error> {
    let self_dict = match result {
        TypVal::Dict(dict) => dict.as_ref(),
        _ => None,
    };
    let Some(self_dict) = self_dict else {
        return Err(Error::validation(c"dict not found"));
    };
    // A Dict argument was converted whole, so its function member is
    // already `fn_0`; a String argument named a dictionary to look in.
    if !fn_0.data().is_null() && !fn_0.is_empty() && !dict_given {
        let Some(di) = self_dict.find(fn_0.as_bytes()) else {
            let name = msg_cstr(fn_0.as_cstr());
            return Err(api_error!(kErrorTypeValidation, "Not found: {name}"));
        };
        let v_type = di.di_tv.v_type();
        if v_type == VAR_PARTIAL {
            return Err(Error::validation(c"partial function not supported"));
        }
        if v_type != VAR_FUNC {
            let name = msg_cstr(fn_0.as_cstr());
            return Err(api_error!(kErrorTypeValidation, "Not a function: {name}"));
        }
        let name = di.di_tv.callable_name().map_or(&[][..], CStr::to_bytes);
        *fn_0 = String_0::from_bytes(name);
    }
    if fn_0.data().is_null() || fn_0.is_empty() {
        return Err(Error::validation(c"Invalid function name: (empty)"));
    }
    call_function_with(fn_0, args, Some(self_dict))
}
