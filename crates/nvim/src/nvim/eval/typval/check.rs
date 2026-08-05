//! Type checks: `tv_check_*` and the per-argument `tv_check_for_*_arg` set.
//!
//! The `_arg` family is what a builtin calls before touching `argvars[idx]`
//! — each answers `OK`/`FAIL` and emits the exact `E1xxx` upstream does,
//! naming the argument's one-based position.  The `opt_` variants accept
//! `VAR_UNKNOWN` (the argument was not given) as well.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_check_str_or_nr(tv: *const typval_T) -> bool {
    unsafe {
        match (*tv).v_type as ::core::ffi::c_uint {
            1 | 2 => return true_0 != 0,
            6 => {
                emsg(gettext(
                    b"E805: Expected a Number or a String, Float found\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                return false_0 != 0;
            }
            9 | 3 => {
                emsg(gettext(
                    b"E703: Expected a Number or a String, Funcref found\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                return false_0 != 0;
            }
            4 => {
                emsg(gettext(
                    b"E745: Expected a Number or a String, List found\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                return false_0 != 0;
            }
            5 => {
                emsg(gettext(
                    b"E728: Expected a Number or a String, Dictionary found\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                return false_0 != 0;
            }
            10 => {
                emsg(gettext(
                    b"E974: Expected a Number or a String, Blob found\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                return false_0 != 0;
            }
            7 => {
                emsg(gettext(
                    b"E5299: Expected a Number or a String, Boolean found\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                return false_0 != 0;
            }
            8 => {
                emsg(gettext(b"E5300: Expected a Number or a String\0".as_ptr()
                    as *const ::core::ffi::c_char));
                return false_0 != 0;
            }
            0 => {
                semsg(
                    gettext(&raw const e_intern2 as *const ::core::ffi::c_char),
                    b"tv_check_str_or_nr(UNKNOWN)\0".as_ptr() as *const ::core::ffi::c_char,
                );
                return false_0 != 0;
            }
            _ => {}
        }
        abort();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_check_num(tv: *const typval_T) -> bool {
    unsafe {
        match (*tv).v_type as ::core::ffi::c_uint {
            1 | 7 | 8 | 2 => return true_0 != 0,
            3 | 9 | 4 | 5 | 6 | 10 | 0 => {
                emsg(gettext((*num_errors.ptr())[(*tv).v_type as usize]));
                return false_0 != 0;
            }
            _ => {}
        }
        abort();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_check_str(tv: *const typval_T) -> bool {
    unsafe {
        match (*tv).v_type as ::core::ffi::c_uint {
            1 | 7 | 8 | 2 | 6 => return true_0 != 0,
            9 | 3 | 4 | 5 | 10 | 0 => {
                emsg(gettext((*str_errors.ptr())[(*tv).v_type as usize]));
                return false_0 != 0;
            }
            _ => {}
        }
        abort();
    }
}

pub unsafe extern "C" fn tv_check_for_string_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            semsg(
                gettext(
                    (e_string_required_for_argument_nr.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                idx + 1 as ::core::ffi::c_int,
            );
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_check_for_nonempty_string_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if tv_check_for_string_arg(args, idx) == FAIL {
            return FAIL;
        }
        if (*args.offset(idx as isize)).vval.v_string.is_null()
            || *(*args.offset(idx as isize)).vval.v_string as ::core::ffi::c_int == NUL
        {
            semsg(
                gettext(
                    (e_non_empty_string_required_for_argument_nr.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                idx + 1 as ::core::ffi::c_int,
            );
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_check_for_opt_string_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        return if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            || tv_check_for_string_arg(args, idx) != FAIL
        {
            OK
        } else {
            FAIL
        };
    }
}

pub unsafe extern "C" fn tv_check_for_number_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            semsg(
                gettext(
                    (e_number_required_for_argument_nr.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                idx + 1 as ::core::ffi::c_int,
            );
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_check_for_opt_number_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        return if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            || tv_check_for_number_arg(args, idx) != FAIL
        {
            OK
        } else {
            FAIL
        };
    }
}

pub unsafe extern "C" fn tv_check_for_float_or_nr_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            != VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
                != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            semsg(
                gettext(
                    (e_float_or_number_required_for_argument_nr.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                idx + 1 as ::core::ffi::c_int,
            );
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_check_for_bool_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            != VAR_BOOL as ::core::ffi::c_int as ::core::ffi::c_uint
            && !((*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
                == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                && ((*args.offset(idx as isize)).vval.v_number == 0 as varnumber_T
                    || (*args.offset(idx as isize)).vval.v_number == 1 as varnumber_T))
        {
            semsg(
                gettext(
                    (e_bool_required_for_argument_nr.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                idx + 1 as ::core::ffi::c_int,
            );
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_check_for_opt_bool_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return OK;
        }
        return tv_check_for_bool_arg(args, idx);
    }
}

pub unsafe extern "C" fn tv_check_for_blob_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            != VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            semsg(
                gettext(
                    (e_blob_required_for_argument_nr.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                idx + 1 as ::core::ffi::c_int,
            );
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_check_for_list_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            semsg(
                gettext(
                    (e_list_required_for_argument_nr.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                idx + 1 as ::core::ffi::c_int,
            );
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_check_for_dict_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            semsg(
                gettext(
                    (e_dict_required_for_argument_nr.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                idx + 1 as ::core::ffi::c_int,
            );
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_check_for_nonnull_dict_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if tv_check_for_dict_arg(args, idx) == FAIL {
            return FAIL;
        }
        if (*args.offset(idx as isize)).vval.v_dict.is_null() {
            semsg(
                gettext(
                    (e_non_null_dict_required_for_argument_nr.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                idx + 1 as ::core::ffi::c_int,
            );
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_check_for_opt_dict_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        return if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            || tv_check_for_dict_arg(args, idx) != FAIL
        {
            OK
        } else {
            FAIL
        };
    }
}

pub unsafe extern "C" fn tv_check_for_string_or_number_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
                != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            semsg(
                gettext(
                    (e_string_or_number_required_for_argument_nr.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                idx + 1 as ::core::ffi::c_int,
            );
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_check_for_buffer_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        return tv_check_for_string_or_number_arg(args, idx);
    }
}

pub unsafe extern "C" fn tv_check_for_lnum_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        return tv_check_for_string_or_number_arg(args, idx);
    }
}

pub unsafe extern "C" fn tv_check_for_string_or_list_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
                != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            semsg(
                gettext(
                    (e_string_or_list_required_for_argument_nr.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                idx + 1 as ::core::ffi::c_int,
            );
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_check_for_string_or_list_or_blob_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
                != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
                != VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            semsg(
                gettext(
                    (e_string_list_or_blob_required_for_argument_nr.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                idx + 1 as ::core::ffi::c_int,
            );
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_check_for_opt_string_or_list_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        return if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            || tv_check_for_string_or_list_arg(args, idx) != FAIL
        {
            OK
        } else {
            FAIL
        };
    }
}

pub unsafe extern "C" fn tv_check_for_string_or_func_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            != VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
                != VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            semsg(
                gettext(
                    (e_string_or_function_required_for_argument_nr.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                idx + 1 as ::core::ffi::c_int,
            );
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_check_for_list_or_blob_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
                != VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            semsg(
                gettext(
                    (e_list_or_blob_required_for_argument_nr.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                idx + 1 as ::core::ffi::c_int,
            );
            return FAIL;
        }
        return OK;
    }
}
