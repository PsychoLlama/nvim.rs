//! Type checks: `tv_check_*` and the per-argument `tv_check_for_*_arg` set.
//!
//! The `_arg` family is what a builtin calls before touching `argvars[idx]`
//! — each answers `OK`/`FAIL` and emits the exact `E1xxx` upstream does,
//! naming the argument's one-based position.  The `opt_` variants accept
//! `VAR_UNKNOWN` (the argument was not given) as well.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
#[allow(unused_imports)]
use crate::semsg_c;

/// The tail every `tv_check_for_*_arg` shares: answer `OK`, or raise `errmsg`
/// naming the argument's one-based position and answer `FAIL`.
#[inline]
unsafe fn arg_check(
    ok: bool,
    errmsg: *const ::core::ffi::c_char,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if ok {
        return OK;
    }
    unsafe {
        semsg_c!(gettext(errmsg), idx + 1);
    }
    FAIL
}

/// Whether `tv` is a Number or a String, raising the type-specific error if
/// not.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_check_str_or_nr(tv: *const typval_T) -> bool {
    unsafe {
        let message = match (*tv).v_type {
            VAR_NUMBER | VAR_STRING => return true,
            VAR_FLOAT => c"E805: Expected a Number or a String, Float found",
            VAR_PARTIAL | VAR_FUNC => c"E703: Expected a Number or a String, Funcref found",
            VAR_LIST => c"E745: Expected a Number or a String, List found",
            VAR_DICT => c"E728: Expected a Number or a String, Dictionary found",
            VAR_BLOB => c"E974: Expected a Number or a String, Blob found",
            VAR_BOOL => c"E5299: Expected a Number or a String, Boolean found",
            VAR_SPECIAL => c"E5300: Expected a Number or a String",
            VAR_UNKNOWN => {
                semsg_c!(
                    gettext(&raw const e_intern2 as *const ::core::ffi::c_char),
                    c"tv_check_str_or_nr(UNKNOWN)".as_ptr(),
                );
                return false;
            }
            _ => abort(),
        };
        emsg(gettext(message.as_ptr()));
        false
    }
}

/// Whether `tv` has a Number value, raising the type-specific error if not.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_check_num(tv: *const typval_T) -> bool {
    unsafe {
        match (*tv).v_type {
            VAR_NUMBER | VAR_BOOL | VAR_SPECIAL | VAR_STRING => true,
            VAR_FUNC | VAR_PARTIAL | VAR_LIST | VAR_DICT | VAR_FLOAT | VAR_BLOB | VAR_UNKNOWN => {
                emsg(gettext((*num_errors.ptr())[(*tv).v_type as usize]));
                false
            }
            _ => abort(),
        }
    }
}

/// Whether `tv` has a String value, raising the type-specific error if not.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_check_str(tv: *const typval_T) -> bool {
    unsafe {
        match (*tv).v_type {
            VAR_NUMBER | VAR_BOOL | VAR_SPECIAL | VAR_STRING | VAR_FLOAT => true,
            VAR_PARTIAL | VAR_FUNC | VAR_LIST | VAR_DICT | VAR_BLOB | VAR_UNKNOWN => {
                emsg(gettext((*str_errors.ptr())[(*tv).v_type as usize]));
                false
            }
            _ => abort(),
        }
    }
}

/// `E1174`: argument `idx` must be a String.
pub unsafe extern "C" fn tv_check_for_string_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let arg = &*args.offset(idx as isize);
        arg_check(
            arg.v_type == VAR_STRING,
            e_string_required_for_argument_nr.ptr().cast(),
            idx,
        )
    }
}

/// `E1175`: argument `idx` must be a String that is not empty.
pub unsafe extern "C" fn tv_check_for_nonempty_string_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if tv_check_for_string_arg(args, idx) == FAIL {
            return FAIL;
        }
        let s = (*args.offset(idx as isize)).vval.v_string;
        arg_check(
            !s.is_null() && *s as ::core::ffi::c_int != NUL,
            e_non_empty_string_required_for_argument_nr.ptr().cast(),
            idx,
        )
    }
}

/// [`tv_check_for_string_arg`], accepting a missing argument.
pub unsafe extern "C" fn tv_check_for_opt_string_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*args.offset(idx as isize)).v_type == VAR_UNKNOWN {
            return OK;
        }
        tv_check_for_string_arg(args, idx)
    }
}

/// `E1210`: argument `idx` must be a Number.
pub unsafe extern "C" fn tv_check_for_number_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let arg = &*args.offset(idx as isize);
        arg_check(
            arg.v_type == VAR_NUMBER,
            e_number_required_for_argument_nr.ptr().cast(),
            idx,
        )
    }
}

/// [`tv_check_for_number_arg`], accepting a missing argument.
pub unsafe extern "C" fn tv_check_for_opt_number_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*args.offset(idx as isize)).v_type == VAR_UNKNOWN {
            return OK;
        }
        tv_check_for_number_arg(args, idx)
    }
}

/// `E1219`: argument `idx` must be a Float or a Number.
pub unsafe extern "C" fn tv_check_for_float_or_nr_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let arg = &*args.offset(idx as isize);
        arg_check(
            arg.v_type == VAR_FLOAT || arg.v_type == VAR_NUMBER,
            e_float_or_number_required_for_argument_nr.ptr().cast(),
            idx,
        )
    }
}

/// `E1212`: argument `idx` must be a Bool, or the Number 0 or 1.
pub unsafe extern "C" fn tv_check_for_bool_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let arg = &*args.offset(idx as isize);
        let numeric_bool =
            arg.v_type == VAR_NUMBER && (arg.vval.v_number == 0 || arg.vval.v_number == 1);
        arg_check(
            arg.v_type == VAR_BOOL || numeric_bool,
            e_bool_required_for_argument_nr.ptr().cast(),
            idx,
        )
    }
}

/// [`tv_check_for_bool_arg`], accepting a missing argument.
pub unsafe extern "C" fn tv_check_for_opt_bool_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*args.offset(idx as isize)).v_type == VAR_UNKNOWN {
            return OK;
        }
        tv_check_for_bool_arg(args, idx)
    }
}

/// `E1238`: argument `idx` must be a Blob.
pub unsafe extern "C" fn tv_check_for_blob_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let arg = &*args.offset(idx as isize);
        arg_check(
            arg.v_type == VAR_BLOB,
            e_blob_required_for_argument_nr.ptr().cast(),
            idx,
        )
    }
}

/// `E1211`: argument `idx` must be a List.
pub unsafe extern "C" fn tv_check_for_list_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let arg = &*args.offset(idx as isize);
        arg_check(
            arg.v_type == VAR_LIST,
            e_list_required_for_argument_nr.ptr().cast(),
            idx,
        )
    }
}

/// `E1206`: argument `idx` must be a Dictionary.
pub unsafe extern "C" fn tv_check_for_dict_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let arg = &*args.offset(idx as isize);
        arg_check(
            arg.v_type == VAR_DICT,
            e_dict_required_for_argument_nr.ptr().cast(),
            idx,
        )
    }
}

/// `E1297`: argument `idx` must be a Dictionary that is not the NULL one.
pub unsafe extern "C" fn tv_check_for_nonnull_dict_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if tv_check_for_dict_arg(args, idx) == FAIL {
            return FAIL;
        }
        arg_check(
            !(*args.offset(idx as isize)).vval.v_dict.is_null(),
            e_non_null_dict_required_for_argument_nr.ptr().cast(),
            idx,
        )
    }
}

/// [`tv_check_for_dict_arg`], accepting a missing argument.
pub unsafe extern "C" fn tv_check_for_opt_dict_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*args.offset(idx as isize)).v_type == VAR_UNKNOWN {
            return OK;
        }
        tv_check_for_dict_arg(args, idx)
    }
}

/// `E1220`: argument `idx` must be a String or a Number.
pub unsafe extern "C" fn tv_check_for_string_or_number_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let arg = &*args.offset(idx as isize);
        arg_check(
            arg.v_type == VAR_STRING || arg.v_type == VAR_NUMBER,
            e_string_or_number_required_for_argument_nr.ptr().cast(),
            idx,
        )
    }
}

/// Argument `idx` must name a buffer: a String or a Number.
pub unsafe extern "C" fn tv_check_for_buffer_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe { tv_check_for_string_or_number_arg(args, idx) }
}

/// Argument `idx` must name a line: a String or a Number.
pub unsafe extern "C" fn tv_check_for_lnum_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe { tv_check_for_string_or_number_arg(args, idx) }
}

/// `E1222`: argument `idx` must be a String or a List.
pub unsafe extern "C" fn tv_check_for_string_or_list_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let arg = &*args.offset(idx as isize);
        arg_check(
            arg.v_type == VAR_STRING || arg.v_type == VAR_LIST,
            e_string_or_list_required_for_argument_nr.ptr().cast(),
            idx,
        )
    }
}

/// `E1252`: argument `idx` must be a String, a List or a Blob.
pub unsafe extern "C" fn tv_check_for_string_or_list_or_blob_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let arg = &*args.offset(idx as isize);
        arg_check(
            arg.v_type == VAR_STRING || arg.v_type == VAR_LIST || arg.v_type == VAR_BLOB,
            e_string_list_or_blob_required_for_argument_nr.ptr().cast(),
            idx,
        )
    }
}

/// [`tv_check_for_string_or_list_arg`], accepting a missing argument.
pub unsafe extern "C" fn tv_check_for_opt_string_or_list_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*args.offset(idx as isize)).v_type == VAR_UNKNOWN {
            return OK;
        }
        tv_check_for_string_or_list_arg(args, idx)
    }
}

/// `E1256`: argument `idx` must be a String, a Funcref or a partial.
pub unsafe extern "C" fn tv_check_for_string_or_func_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let arg = &*args.offset(idx as isize);
        arg_check(
            arg.v_type == VAR_PARTIAL || arg.v_type == VAR_FUNC || arg.v_type == VAR_STRING,
            e_string_or_function_required_for_argument_nr.ptr().cast(),
            idx,
        )
    }
}

/// `E1226`: argument `idx` must be a List or a Blob.
pub unsafe extern "C" fn tv_check_for_list_or_blob_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let arg = &*args.offset(idx as isize);
        arg_check(
            arg.v_type == VAR_LIST || arg.v_type == VAR_BLOB,
            e_list_or_blob_required_for_argument_nr.ptr().cast(),
            idx,
        )
    }
}
