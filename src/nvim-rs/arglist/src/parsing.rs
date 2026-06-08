//! String parsing functions for the argument list
//!
//! Phase 3: do_one_arg, get_arglist, get_arglist_exp

use std::ffi::{c_char, c_int};

use crate::ffi::{self, GarrayPtr};
use crate::{EW_FILE, EW_NOTFOUND, EW_NOTWILD};

/// NUL as c_char, avoiding clippy::cast_possible_wrap
const NUL_CHAR: c_char = 0;
/// Backtick as c_char
#[allow(clippy::cast_possible_wrap)]
const BACKTICK: c_char = b'`' as c_char;

// =============================================================================
// do_one_arg (static in C — only called from get_arglist)
// =============================================================================

/// Isolate one argument, taking backticks.
/// Changes the argument in-place, puts a NUL after it.
/// Returns a pointer to the start of the next argument.
unsafe fn do_one_arg(str_ptr: *mut c_char) -> *mut c_char {
    let mut str = str_ptr;
    let mut p = str;
    let mut inbacktick = false;

    while *str != NUL_CHAR {
        if ffi::nvim_al_rem_backslash(str) {
            *p = *str;
            p = p.add(1);
            str = str.add(1);
        } else {
            if !inbacktick && ffi::nvim_al_ascii_isspace(c_int::from(*str)) != 0 {
                break;
            }
            if *str == BACKTICK {
                inbacktick = !inbacktick;
            }
        }
        *p = *str;
        p = p.add(1);
        str = str.add(1);
    }
    str = ffi::nvim_al_skipwhite(str);
    *p = NUL_CHAR;

    str
}

// =============================================================================
// get_arglist (static in C)
// =============================================================================

/// Separate the arguments in `str` and return a list of pointers in the
/// growarray `gap`.
pub(crate) unsafe fn get_arglist(gap: GarrayPtr, mut str: *mut c_char, escaped: bool) {
    ffi::nvim_al_ga_init_charptr(gap);
    while *str != NUL_CHAR {
        ffi::nvim_al_ga_append_charptr(gap, str);

        if !escaped {
            return;
        }

        str = do_one_arg(str);
    }
}

// =============================================================================
// get_arglist_exp
// =============================================================================

#[export_name = "get_arglist_exp"]
pub extern "C" fn rs_get_arglist_exp(
    str: *mut c_char,
    fcountp: *mut c_int,
    fnamesp: *mut *mut *mut c_char,
    wig: c_int,
) -> c_int {
    unsafe {
        let ga = ffi::nvim_al_alloc_garray();

        get_arglist(ga, str, true);

        let ga_len = ffi::nvim_al_ga_get_len(ga);
        let ga_data = ffi::nvim_al_ga_get_data(ga).cast::<*mut c_char>();

        let i = if wig != 0 {
            ffi::nvim_al_expand_wildcards(
                ga_len,
                ga_data,
                fcountp,
                fnamesp,
                EW_FILE | EW_NOTFOUND | EW_NOTWILD,
            )
        } else {
            ffi::nvim_al_gen_expand_wildcards(
                ga_len,
                ga_data,
                fcountp,
                fnamesp,
                EW_FILE | EW_NOTFOUND | EW_NOTWILD,
            )
        };

        ffi::nvim_al_ga_clear(ga);
        ffi::nvim_al_free_garray(ga);
        i
    }
}
