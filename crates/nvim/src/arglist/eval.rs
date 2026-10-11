//! The `argc()`, `argidx()`, `arglistid()` and `argv()` builtins.
//!
//! Each takes an optional window (and tab page) to ask about; `-1` in the
//! window slot means the global argument list rather than any window's.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::memory::ThinCString;
use crate::narrow::number_as_int;
use crate::types::VAR_NUMBER;
use crate::winlayer::Win;

/// The argument list a `{winid}`-style argument selects: the current
/// window's when the argument is missing, the global one for `-1`, and
/// otherwise the named window's — `None` when there is no such window.
fn selected_arglist(arg: Option<&TypVal>) -> Option<ArgListId> {
    let Some(arg) = arg else {
        return Some(cur_alist());
    };
    if arg.v_type() == VAR_NUMBER && tv_get_number(arg) == -1 as VarNumber {
        return Some(ArgListId::GLOBAL);
    }
    find_win_by_nr_or_id(arg).map(|win| win.w_alist)
}

/// "argc()" function
pub fn f_argc(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // A window that does not exist answers -1, as it always has.
    let count = selected_arglist(args.first()).map_or(-1, alist_count);
    result.write_number(VarNumber::from(count));
}

/// "argidx()" function
pub fn f_argidx(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(VarNumber::from(Win::current().w_arg_idx));
}

/// "arglistid()" function
pub fn f_arglistid(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let found = find_tabwin(args.first(), args.get(1));
    let id = found.map_or(-1, |wp| VarNumber::from(wp.w_alist.0));
    result.write_number(id);
}

/// The names `names` as a List in `result`. No names still allocates the
/// (empty) List, which is what `argv(-1)` on a window that does not exist
/// answers.
fn arglist_as_rettv(names: &[XString], result: &mut TypVal) {
    let len = ptrdiff_t::try_from(names.len()).unwrap_or(ptrdiff_t::MAX);
    let list = tv_list_alloc_ret(result, len);
    for name in names {
        list.push_str(Some(name.as_cstr()));
    }
}

/// "argv()" function
pub fn f_argv(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    if args.is_empty() {
        // No index: the whole current argument list.
        arglist_as_rettv(&arg_names(cur_alist()), result);
        return;
    }
    // A window that does not exist leaves no list, so every index is out of
    // range.
    let alist = selected_arglist(args.get(1));
    result.write_string(None);
    let idx = number_as_int(tv_get_number_chk(&args[0]).unwrap_or(-1));
    if let Some(name) = alist.and_then(|alist| arg_name(alist, idx)) {
        result.write_string(Some(ThinCString::from_cstr(name.as_cstr())));
    } else if idx == -1 {
        let names = alist.map(arg_names).unwrap_or_default();
        arglist_as_rettv(&names, result);
    }
}
