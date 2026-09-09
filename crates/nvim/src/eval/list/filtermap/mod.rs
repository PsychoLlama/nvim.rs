//! Walking a container and applying an expression to every item --
//! `filter()`, `map()`, `mapnew()` and `foreach()`.
//!
//! All four are one [`filter_map`] with a [`FilterMap`] saying what to do
//! with each result: drop the item, replace it, collect it into a fresh
//! container, or throw it away.  [`filter_map_one`] is the per-item half --
//! it sets `v:val`, evaluates the expression or calls the Funcref, and
//! reports whether the walk should keep going -- and [`containers`] is the
//! four per-container walks it is driven from.
//!
//! # Re-entrancy
//!
//! [`filter_map_one`] re-enters the evaluator on every item, so a callback
//! may lock, unlock, extend, shorten or free the very container being
//! walked.  Each walk therefore locks its container for the duration, which
//! turns most of those into an `E741` raised by the callback rather than a
//! dangling pointer here; what survives the lock is *removal by `filter()`
//! itself*, which is why the List walk asks the list for the next item only
//! after the callback has returned.  See the safe layer's doc in [`super`].
//!
//! Original: `src/nvim/eval/list.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::CStr;

use super::{
    Container, TvRef, clear_tv, clear_vim_var, copy_tv, err_not_container, eval_expr, number_of,
    restore_vim_var, run_cmd, save_vim_var, set_vim_var_tv, string_bytes, vim_var_value,
};
use crate::eval::typval::UNSET_ARG;
use crate::message::state::did_emsg;
use crate::types::{EvalFuncData, TypVal, VAR_UNKNOWN, Vv};

// The carve of the transpiled module; see each child's docs.
mod containers;

use self::containers::{filter_map_blob, filter_map_dict, filter_map_list, filter_map_string};

/// Which of the four builtins is running: what to do with the value the
/// callback answered.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum FilterMap {
    /// `filter()`: drop the item when the answer is falsy.
    Filter,
    /// `map()`: replace the item with the answer.
    Map,
    /// `mapnew()`: collect the answers into a fresh container.
    MapNew,
    /// `foreach()`: throw the answer away.
    Foreach,
}

impl FilterMap {
    /// The name the type errors report.
    fn func_name(self) -> &'static CStr {
        match self {
            Self::Filter => c"filter()",
            Self::Map => c"map()",
            Self::MapNew => c"mapnew()",
            Self::Foreach => c"foreach()",
        }
    }

    /// The name the lock errors report.
    fn arg_errmsg(self) -> &'static CStr {
        match self {
            Self::Filter => c"filter() argument",
            Self::Map => c"map() argument",
            Self::MapNew => c"mapnew() argument",
            Self::Foreach => c"foreach() argument",
        }
    }
}

/// Handle one item: set `v:val` to `tv`, evaluate `expr`, and answer whether
/// the walk should go on.  The caller sets `v:key`.
///
/// `newtv` receives the value for `map()`/`mapnew()`; `rem` says whether
/// `filter()` should drop the item.  The two forms that do not want a value
/// clear it here.
pub(crate) fn filter_map_one(
    tv: TvRef,
    expr: &TypVal,
    filtermap: FilterMap,
    newtv: &mut TypVal,
    rem: &mut bool,
) -> bool {
    set_vim_var_tv(Vv::Val, tv);
    newtv.write_empty(VAR_UNKNOWN);

    let mut retval = false;
    'theend: {
        // foreach() is not limited to an expression.
        if let (FilterMap::Foreach, Container::Str(cmd)) = (filtermap, Container::of(expr)) {
            run_cmd(cmd);
            retval = did_emsg.get() == 0;
            break 'theend;
        }
        let mut argv = [UNSET_ARG; 3];
        argv[0] = vim_var_value(Vv::Key);
        argv[1] = vim_var_value(Vv::Val);
        if !eval_expr(expr, &mut argv, newtv) {
            break 'theend;
        }
        match filtermap {
            FilterMap::Filter => {
                // filter(): when expr is zero remove the item.
                let mut error = false;
                *rem = number_of(newtv, &mut error) == 0;
                clear_tv(newtv);
                // On a type error nothing has been removed; stop the loop
                // without an answer.  tv_get_number_chk gave the message.
                if error {
                    break 'theend;
                }
            }
            FilterMap::Foreach => clear_tv(newtv),
            FilterMap::Map | FilterMap::MapNew => {}
        }
        retval = true;
    }
    clear_vim_var(Vv::Val);
    retval
}

/// The shared body of the four builtins: check the argument, save `v:key`
/// and `v:val`, dispatch on the container, and put everything back.
fn filter_map(args: &[TypVal], result: &mut TypVal, filtermap: FilterMap) {
    let arg = &args[0];
    let container = Container::of(arg);

    // map(), filter(), foreach() return the first argument, also on failure.
    if filtermap != FilterMap::MapNew && !matches!(container, Container::Str(_)) {
        copy_tv(arg, result);
    }
    if matches!(container, Container::Other) {
        err_not_container(filtermap.func_name());
        return;
    }

    // On type errors the preceding call has already displayed an error
    // message.  Avoid a misleading one for an empty string that was not
    // passed as an argument.
    let expr = &args[1];
    if expr.v_type() == VAR_UNKNOWN {
        return;
    }

    let mut save_val = save_vim_var(Vv::Val);
    let mut save_key = save_vim_var(Vv::Key);

    // Reset did_emsg to be able to detect whether an error occurred during
    // evaluation of the expression.
    let save_did_emsg = did_emsg.get();
    did_emsg.set(0);

    let arg_errmsg = filtermap.arg_errmsg();
    match container {
        Container::Dict(d) => filter_map_dict(d, filtermap, arg_errmsg, expr, result),
        Container::Blob(b) => filter_map_blob(b, filtermap, arg_errmsg, expr, result),
        Container::Str(_) => filter_map_string(string_bytes(&args[0]), filtermap, expr, result),
        Container::List(l) => filter_map_list(l, filtermap, arg_errmsg, expr, result),
        Container::Other => unreachable!("reported above"),
    }

    restore_vim_var(Vv::Key, &mut save_key);
    restore_vim_var(Vv::Val, &mut save_val);

    did_emsg.set(did_emsg.get() | save_did_emsg);
}

/// `filter(container, expr)`: drop every item the expression calls false.
pub fn f_filter(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the caller's contract.
    filter_map(args, &mut *result, FilterMap::Filter);
}

/// `map(container, expr)`: replace every item with the expression's value.
pub fn f_map(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the caller's contract.
    filter_map(args, &mut *result, FilterMap::Map);
}

/// `mapnew(container, expr)`: `map()` into a fresh container, leaving the
/// argument alone.
pub fn f_mapnew(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the caller's contract.
    filter_map(args, &mut *result, FilterMap::MapNew);
}

/// `foreach(container, expr)`: evaluate the expression -- or run the Ex
/// command line -- once per item, for its side effects.
pub fn f_foreach(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the caller's contract.
    filter_map(args, &mut *result, FilterMap::Foreach);
}
