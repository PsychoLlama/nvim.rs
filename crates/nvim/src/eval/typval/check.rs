//! Type checks: `tv_check_*` and the per-argument `tv_check_for_*_arg` set.
//!
//! The `_arg` family is what a builtin calls before touching `argvars[idx]`
//! — each answers `OK`/`FAIL` and emits the exact `E1xxx` upstream does,
//! naming the argument's one-based position.  The `opt_` variants accept
//! `VAR_UNKNOWN` (the argument was not given) as well.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::semsg_c;
use crate::types::{FAIL, NUL, OK};

/// The tail every `tv_check_for_*_arg` shares: answer `OK`, or raise `errmsg`
/// naming the argument's one-based position and answer `FAIL`.
///
/// Private to this module, and the obligation its format string carries is
/// discharged by construction: every caller passes one of the module's own
/// `e_*_required_for_argument_nr` statics, each a NUL-terminated literal
/// with exactly one `%d`. Being on the editor's main thread is the ambient
/// precondition of the whole `eval/` tree, not this helper's own.
#[inline]
fn arg_check(
    ok: bool,
    errmsg: *const ::core::ffi::c_char,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if ok {
        return OK;
    }
    unsafe { semsg_c!(gettext(errmsg), idx + 1) };
    FAIL
}

/// Whether `tv` is a Number or a String, raising the type-specific error if
/// not.
///
/// # Safety
/// `tv` must point at an initialised value.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_str_or_nr(tv: *const typval_T) -> bool {
    let message = match unsafe { (*tv).v_type } {
        VAR_NUMBER | VAR_STRING => return true,
        VAR_FLOAT => c"E805: Expected a Number or a String, Float found",
        VAR_PARTIAL | VAR_FUNC => c"E703: Expected a Number or a String, Funcref found",
        VAR_LIST => c"E745: Expected a Number or a String, List found",
        VAR_DICT => c"E728: Expected a Number or a String, Dictionary found",
        VAR_BLOB => c"E974: Expected a Number or a String, Blob found",
        VAR_BOOL => c"E5299: Expected a Number or a String, Boolean found",
        VAR_SPECIAL => c"E5300: Expected a Number or a String",
        VAR_UNKNOWN => {
            unsafe { semsg_c!(tr(e_intern2), c"tv_check_str_or_nr(UNKNOWN)".as_ptr(),) };
            return false;
        }
        _ => unsafe { abort() },
    };
    unsafe { emsg(gettext(message.as_ptr())) };
    false
}

/// Whether `tv` has a Number value, raising the type-specific error if not.
///
/// # Safety
/// `tv` must point at an initialised value.
/// The message comes out of the global `num_errors` table, so the caller must
/// be on the editor's main thread.
pub unsafe fn tv_check_num(tv: *const typval_T) -> bool {
    match unsafe { (*tv).v_type } {
        VAR_NUMBER | VAR_BOOL | VAR_SPECIAL | VAR_STRING => true,
        VAR_FUNC | VAR_PARTIAL | VAR_LIST | VAR_DICT | VAR_FLOAT | VAR_BLOB | VAR_UNKNOWN => {
            unsafe { emsg(gettext(num_errors[(*tv).v_type as usize])) };
            false
        }
        _ => unsafe { abort() },
    }
}

/// Whether `tv` has a String value, raising the type-specific error if not.
///
/// # Safety
/// `tv` must point at an initialised value.
/// The message comes out of the global `str_errors` table, so the caller must
/// be on the editor's main thread.
pub unsafe fn tv_check_str(tv: *const typval_T) -> bool {
    match unsafe { (*tv).v_type } {
        VAR_NUMBER | VAR_BOOL | VAR_SPECIAL | VAR_STRING | VAR_FLOAT => true,
        VAR_PARTIAL | VAR_FUNC | VAR_LIST | VAR_DICT | VAR_BLOB | VAR_UNKNOWN => {
            unsafe { emsg(gettext(str_errors[(*tv).v_type as usize])) };
            false
        }
        _ => unsafe { abort() },
    }
}

/// `E1174`: argument `idx` must be a String.
///
/// # Safety
/// `args` must point at at least `idx + 1` initialised values and `idx`
/// must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_string_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let arg = unsafe { &*args.offset(idx as isize) };
    arg_check(
        arg.v_type == VAR_STRING,
        e_string_required_for_argument_nr.as_ptr(),
        idx,
    )
}

/// `E1175`: argument `idx` must be a String that is not empty.
///
/// # Safety
/// `args` must point at at least `idx + 1` initialised values and `idx`
/// must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_nonempty_string_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if unsafe { tv_check_for_string_arg(args, idx) } == FAIL {
        return FAIL;
    }
    let s = unsafe { (*args.offset(idx as isize)).vval.v_string };
    let nonempty = !s.is_null() && unsafe { *s } as ::core::ffi::c_int != NUL;
    arg_check(
        nonempty,
        e_non_empty_string_required_for_argument_nr.as_ptr(),
        idx,
    )
}

/// [`tv_check_for_string_arg`], accepting a missing argument.
///
/// # Safety
/// `args` must point at at least `idx + 1` values, the last of which may
/// be the `VAR_UNKNOWN` terminator, and `idx` must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_opt_string_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if unsafe { (*args.offset(idx as isize)).v_type } == VAR_UNKNOWN {
        return OK;
    }
    unsafe { tv_check_for_string_arg(args, idx) }
}

/// `E1210`: argument `idx` must be a Number.
///
/// # Safety
/// `args` must point at at least `idx + 1` initialised values and `idx`
/// must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_number_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let arg = unsafe { &*args.offset(idx as isize) };
    arg_check(
        arg.v_type == VAR_NUMBER,
        e_number_required_for_argument_nr.as_ptr(),
        idx,
    )
}

/// [`tv_check_for_number_arg`], accepting a missing argument.
///
/// # Safety
/// `args` must point at at least `idx + 1` values, the last of which may
/// be the `VAR_UNKNOWN` terminator, and `idx` must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_opt_number_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if unsafe { (*args.offset(idx as isize)).v_type } == VAR_UNKNOWN {
        return OK;
    }
    unsafe { tv_check_for_number_arg(args, idx) }
}

/// `E1219`: argument `idx` must be a Float or a Number.
///
/// # Safety
/// `args` must point at at least `idx + 1` initialised values and `idx`
/// must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_float_or_nr_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let arg = unsafe { &*args.offset(idx as isize) };
    arg_check(
        arg.v_type == VAR_FLOAT || arg.v_type == VAR_NUMBER,
        e_float_or_number_required_for_argument_nr.as_ptr(),
        idx,
    )
}

/// `E1212`: argument `idx` must be a Bool, or the Number 0 or 1.
///
/// # Safety
/// `args` must point at at least `idx + 1` initialised values and `idx`
/// must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_bool_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let arg = unsafe { &*args.offset(idx as isize) };
    let numeric_bool = arg.v_type == VAR_NUMBER
        && (unsafe { arg.vval.v_number } == 0 || unsafe { arg.vval.v_number } == 1);
    arg_check(
        arg.v_type == VAR_BOOL || numeric_bool,
        e_bool_required_for_argument_nr.as_ptr(),
        idx,
    )
}

/// [`tv_check_for_bool_arg`], accepting a missing argument.
///
/// # Safety
/// `args` must point at at least `idx + 1` values, the last of which may
/// be the `VAR_UNKNOWN` terminator, and `idx` must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_opt_bool_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if unsafe { (*args.offset(idx as isize)).v_type } == VAR_UNKNOWN {
        return OK;
    }
    unsafe { tv_check_for_bool_arg(args, idx) }
}

/// `E1238`: argument `idx` must be a Blob.
///
/// # Safety
/// `args` must point at at least `idx + 1` initialised values and `idx`
/// must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_blob_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let arg = unsafe { &*args.offset(idx as isize) };
    arg_check(
        arg.v_type == VAR_BLOB,
        e_blob_required_for_argument_nr.as_ptr(),
        idx,
    )
}

/// `E1211`: argument `idx` must be a List.
///
/// # Safety
/// `args` must point at at least `idx + 1` initialised values and `idx`
/// must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_list_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let arg = unsafe { &*args.offset(idx as isize) };
    arg_check(
        arg.v_type == VAR_LIST,
        e_list_required_for_argument_nr.as_ptr(),
        idx,
    )
}

/// `E1206`: argument `idx` must be a Dictionary.
///
/// # Safety
/// `args` must point at at least `idx + 1` initialised values and `idx`
/// must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_dict_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let arg = unsafe { &*args.offset(idx as isize) };
    arg_check(
        arg.v_type == VAR_DICT,
        e_dict_required_for_argument_nr.as_ptr(),
        idx,
    )
}

/// `E1297`: argument `idx` must be a Dictionary that is not the NULL one.
///
/// # Safety
/// `args` must point at at least `idx + 1` initialised values and `idx`
/// must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_nonnull_dict_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if unsafe { tv_check_for_dict_arg(args, idx) } == FAIL {
        return FAIL;
    }
    let dict = unsafe { (*args.offset(idx as isize)).vval.v_dict };
    arg_check(
        !dict.is_null(),
        e_non_null_dict_required_for_argument_nr.as_ptr(),
        idx,
    )
}

/// [`tv_check_for_dict_arg`], accepting a missing argument.
///
/// # Safety
/// `args` must point at at least `idx + 1` values, the last of which may
/// be the `VAR_UNKNOWN` terminator, and `idx` must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_opt_dict_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if unsafe { (*args.offset(idx as isize)).v_type } == VAR_UNKNOWN {
        return OK;
    }
    unsafe { tv_check_for_dict_arg(args, idx) }
}

/// `E1220`: argument `idx` must be a String or a Number.
///
/// # Safety
/// `args` must point at at least `idx + 1` initialised values and `idx`
/// must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_string_or_number_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let arg = unsafe { &*args.offset(idx as isize) };
    arg_check(
        arg.v_type == VAR_STRING || arg.v_type == VAR_NUMBER,
        e_string_or_number_required_for_argument_nr.as_ptr(),
        idx,
    )
}

/// Argument `idx` must name a buffer: a String or a Number.
///
/// # Safety
/// `args` must point at at least `idx + 1` initialised values and `idx`
/// must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_buffer_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe { tv_check_for_string_or_number_arg(args, idx) }
}

/// Argument `idx` must name a line: a String or a Number.
///
/// # Safety
/// `args` must point at at least `idx + 1` initialised values and `idx`
/// must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_lnum_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe { tv_check_for_string_or_number_arg(args, idx) }
}

/// `E1222`: argument `idx` must be a String or a List.
///
/// # Safety
/// `args` must point at at least `idx + 1` initialised values and `idx`
/// must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_string_or_list_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let arg = unsafe { &*args.offset(idx as isize) };
    arg_check(
        arg.v_type == VAR_STRING || arg.v_type == VAR_LIST,
        e_string_or_list_required_for_argument_nr.as_ptr(),
        idx,
    )
}

/// `E1252`: argument `idx` must be a String, a List or a Blob.
///
/// # Safety
/// `args` must point at at least `idx + 1` initialised values and `idx`
/// must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_string_or_list_or_blob_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let arg = unsafe { &*args.offset(idx as isize) };
    arg_check(
        arg.v_type == VAR_STRING || arg.v_type == VAR_LIST || arg.v_type == VAR_BLOB,
        e_string_list_or_blob_required_for_argument_nr.as_ptr(),
        idx,
    )
}

/// [`tv_check_for_string_or_list_arg`], accepting a missing argument.
///
/// # Safety
/// `args` must point at at least `idx + 1` values, the last of which may
/// be the `VAR_UNKNOWN` terminator, and `idx` must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_opt_string_or_list_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if unsafe { (*args.offset(idx as isize)).v_type } == VAR_UNKNOWN {
        return OK;
    }
    unsafe { tv_check_for_string_or_list_arg(args, idx) }
}

/// `E1256`: argument `idx` must be a String, a Funcref or a partial.
///
/// # Safety
/// `args` must point at at least `idx + 1` initialised values and `idx`
/// must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_string_or_func_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let arg = unsafe { &*args.offset(idx as isize) };
    arg_check(
        arg.v_type == VAR_PARTIAL || arg.v_type == VAR_FUNC || arg.v_type == VAR_STRING,
        e_string_or_function_required_for_argument_nr.as_ptr(),
        idx,
    )
}

/// `E1226`: argument `idx` must be a List or a Blob.
///
/// # Safety
/// `args` must point at at least `idx + 1` initialised values and `idx`
/// must be non-negative.
/// Raising the error goes through the editor's message state, so the
/// caller must be on the main thread.
pub unsafe fn tv_check_for_list_or_blob_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let arg = unsafe { &*args.offset(idx as isize) };
    arg_check(
        arg.v_type == VAR_LIST || arg.v_type == VAR_BLOB,
        e_list_or_blob_required_for_argument_nr.as_ptr(),
        idx,
    )
}
