//! Type checks: `tv_check_*` and the per-argument `tv_check_for_*_arg` set.
//!
//! The `_arg` family is what a builtin calls before touching `argvars[idx]`
//! — each answers `OK`/`FAIL` and emits the exact `E1xxx` upstream does,
//! naming the argument's one-based position.  The `opt_` variants accept
//! `VAR_UNKNOWN` (the argument was not given) as well.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::message_fmt::emsg_text;
use crate::os::cshim::gettext_ptr;
use crate::semsg;
use crate::tr_plural;
use crate::types::Failed;
use crate::types::NUL;

/// The tail every `tv_check_for_*_arg` shares: answer `Ok`, or raise `errmsg`
/// naming the argument's one-based position and answer `Err`.
///
/// Private to this module, and the obligation its format string carries is
/// discharged by construction: every caller passes one of the module's own
/// `e_*_required_for_argument_nr` statics, each a NUL-terminated literal
/// with exactly one `%d`. Being on the editor's main thread is the ambient
/// precondition of the whole `eval/` tree, not this helper's own.
#[inline]
fn arg_is(
    args: &[TypVal],
    idx: usize,
    errmsg: *const ::core::ffi::c_char,
    ok: impl Fn(&TypVal) -> bool,
) -> Result<(), Failed> {
    arg_check(args.get(idx).is_some_and(ok), errmsg, idx)
}

#[inline]
fn arg_check(ok: bool, errmsg: *const ::core::ffi::c_char, idx: usize) -> Result<(), Failed> {
    if ok {
        return Ok(());
    }
    // SAFETY: `errmsg` is one of the module's NUL-terminated statics.
    let errmsg = unsafe { gettext_ptr(errmsg) };
    let position = ::core::ffi::c_int::try_from(idx + 1).expect("an argument position");
    emsg_text(tr_plural!(errmsg, position));
    Err(Failed)
}

/// Whether `tv` is a Number or a String, raising the type-specific error if
/// not.
pub fn tv_check_str_or_nr(tv: &TypVal) -> bool {
    let message = match (*tv).v_type() {
        VAR_NUMBER | VAR_STRING => return true,
        VAR_FLOAT => c"E805: Expected a Number or a String, Float found",
        VAR_PARTIAL | VAR_FUNC => c"E703: Expected a Number or a String, Funcref found",
        VAR_LIST => c"E745: Expected a Number or a String, List found",
        VAR_DICT => c"E728: Expected a Number or a String, Dictionary found",
        VAR_BLOB => c"E974: Expected a Number or a String, Blob found",
        VAR_BOOL => c"E5299: Expected a Number or a String, Boolean found",
        VAR_SPECIAL => c"E5300: Expected a Number or a String",
        VAR_UNKNOWN => {
            let arg0 = "tv_check_str_or_nr(UNKNOWN)";
            semsg!("E685: Internal error: {arg0}");
            return false;
        }
        _ => unsafe { abort() },
    };
    emsg(gettext(message));
    false
}

/// Whether `tv` has a Number value, raising the type-specific error if not.
pub fn tv_check_num(tv: &TypVal) -> bool {
    match (*tv).v_type() {
        VAR_NUMBER | VAR_BOOL | VAR_SPECIAL | VAR_STRING => true,
        VAR_FUNC | VAR_PARTIAL | VAR_LIST | VAR_DICT | VAR_FLOAT | VAR_BLOB | VAR_UNKNOWN => {
            unsafe { emsg(gettext_ptr(num_errors[(*tv).v_type() as usize])) };
            false
        }
        _ => unsafe { abort() },
    }
}

/// Whether `tv` has a String value, raising the type-specific error if not.
pub fn tv_check_str(tv: &TypVal) -> bool {
    match (*tv).v_type() {
        VAR_NUMBER | VAR_BOOL | VAR_SPECIAL | VAR_STRING | VAR_FLOAT => true,
        VAR_PARTIAL | VAR_FUNC | VAR_LIST | VAR_DICT | VAR_BLOB | VAR_UNKNOWN => {
            unsafe { emsg(gettext_ptr(str_errors[(*tv).v_type() as usize])) };
            false
        }
        _ => unsafe { abort() },
    }
}

/// `E1174`: argument `idx` must be a String.
pub fn tv_check_for_string_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    arg_is(
        args,
        idx,
        e_string_required_for_argument_nr.as_ptr(),
        |arg| arg.v_type() == VAR_STRING,
    )
}

/// `E1175`: argument `idx` must be a String that is not empty.
pub fn tv_check_for_nonempty_string_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    tv_check_for_string_arg(args, idx)?;
    let s = args[idx].string_or_null();
    let nonempty = !s.is_null() && ::core::ffi::c_int::from(unsafe { *s }) != NUL;
    arg_check(
        nonempty,
        e_non_empty_string_required_for_argument_nr.as_ptr(),
        idx,
    )
}

/// [`tv_check_for_string_arg`], accepting a missing argument.
pub fn tv_check_for_opt_string_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    if args.len() <= idx {
        return Ok(());
    }
    tv_check_for_string_arg(args, idx)
}

/// `E1210`: argument `idx` must be a Number.
pub fn tv_check_for_number_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    arg_is(
        args,
        idx,
        e_number_required_for_argument_nr.as_ptr(),
        |arg| arg.v_type() == VAR_NUMBER,
    )
}

/// [`tv_check_for_number_arg`], accepting a missing argument.
pub fn tv_check_for_opt_number_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    if args.len() <= idx {
        return Ok(());
    }
    tv_check_for_number_arg(args, idx)
}

/// `E1219`: argument `idx` must be a Float or a Number.
pub fn tv_check_for_float_or_nr_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    arg_is(
        args,
        idx,
        e_float_or_number_required_for_argument_nr.as_ptr(),
        |arg| arg.v_type() == VAR_FLOAT || arg.v_type() == VAR_NUMBER,
    )
}

/// `E1212`: argument `idx` must be a Bool, or the Number 0 or 1.
pub fn tv_check_for_bool_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    arg_is(args, idx, e_bool_required_for_argument_nr.as_ptr(), |arg| {
        let numeric_bool =
            arg.v_type() == VAR_NUMBER && (arg.number_or_zero() == 0 || arg.number_or_zero() == 1);
        arg.v_type() == VAR_BOOL || numeric_bool
    })
}

/// [`tv_check_for_bool_arg`], accepting a missing argument.
pub fn tv_check_for_opt_bool_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    if args.len() <= idx {
        return Ok(());
    }
    tv_check_for_bool_arg(args, idx)
}

/// `E1238`: argument `idx` must be a Blob.
pub fn tv_check_for_blob_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    arg_is(args, idx, e_blob_required_for_argument_nr.as_ptr(), |arg| {
        arg.v_type() == VAR_BLOB
    })
}

/// `E1211`: argument `idx` must be a List.
pub fn tv_check_for_list_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    arg_is(args, idx, e_list_required_for_argument_nr.as_ptr(), |arg| {
        arg.v_type() == VAR_LIST
    })
}

/// `E1206`: argument `idx` must be a Dictionary.
pub fn tv_check_for_dict_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    arg_is(args, idx, e_dict_required_for_argument_nr.as_ptr(), |arg| {
        arg.v_type() == VAR_DICT
    })
}

/// `E1297`: argument `idx` must be a Dictionary that is not the NULL one.
pub fn tv_check_for_nonnull_dict_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    tv_check_for_dict_arg(args, idx)?;
    let dict = args[idx].dict_or_null();
    arg_check(
        !dict.is_null(),
        e_non_null_dict_required_for_argument_nr.as_ptr(),
        idx,
    )
}

/// [`tv_check_for_dict_arg`], accepting a missing argument.
pub fn tv_check_for_opt_dict_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    if args.len() <= idx {
        return Ok(());
    }
    tv_check_for_dict_arg(args, idx)
}

/// `E1220`: argument `idx` must be a String or a Number.
pub fn tv_check_for_string_or_number_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    arg_is(
        args,
        idx,
        e_string_or_number_required_for_argument_nr.as_ptr(),
        |arg| arg.v_type() == VAR_STRING || arg.v_type() == VAR_NUMBER,
    )
}

/// Argument `idx` must name a buffer: a String or a Number.
pub fn tv_check_for_buffer_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    tv_check_for_string_or_number_arg(args, idx)
}

/// Argument `idx` must name a line: a String or a Number.
pub fn tv_check_for_lnum_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    tv_check_for_string_or_number_arg(args, idx)
}

/// `E1222`: argument `idx` must be a String or a List.
pub fn tv_check_for_string_or_list_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    arg_is(
        args,
        idx,
        e_string_or_list_required_for_argument_nr.as_ptr(),
        |arg| arg.v_type() == VAR_STRING || arg.v_type() == VAR_LIST,
    )
}

/// `E1252`: argument `idx` must be a String, a List or a Blob.
pub fn tv_check_for_string_or_list_or_blob_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    arg_is(
        args,
        idx,
        e_string_list_or_blob_required_for_argument_nr.as_ptr(),
        |arg| arg.v_type() == VAR_STRING || arg.v_type() == VAR_LIST || arg.v_type() == VAR_BLOB,
    )
}

/// [`tv_check_for_string_or_list_arg`], accepting a missing argument.
pub fn tv_check_for_opt_string_or_list_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    if args.len() <= idx {
        return Ok(());
    }
    tv_check_for_string_or_list_arg(args, idx)
}

/// `E1256`: argument `idx` must be a String, a Funcref or a partial.
pub fn tv_check_for_string_or_func_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    arg_is(
        args,
        idx,
        e_string_or_function_required_for_argument_nr.as_ptr(),
        |arg| arg.v_type() == VAR_PARTIAL || arg.v_type() == VAR_FUNC || arg.v_type() == VAR_STRING,
    )
}

/// `E1226`: argument `idx` must be a List or a Blob.
pub fn tv_check_for_list_or_blob_arg(args: &[TypVal], idx: usize) -> Result<(), Failed> {
    arg_is(
        args,
        idx,
        e_list_or_blob_required_for_argument_nr.as_ptr(),
        |arg| arg.v_type() == VAR_LIST || arg.v_type() == VAR_BLOB,
    )
}
