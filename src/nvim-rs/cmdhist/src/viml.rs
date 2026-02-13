//! VimL function implementations for command history
//!
//! f_histadd, f_histdel, f_histget, f_histnr

use std::ffi::c_int;
use std::ptr;

use crate::ffi::{self, EvalFuncData, TypvalPtr};
use crate::helpers::{calc_hist_idx, get_history_idx};
use crate::{HIST_INVALID, NUMBUFLEN, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN};

// =============================================================================
// f_histadd
// =============================================================================

/// "histadd()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers. Accesses C history arrays via FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_f_histadd(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    ffi::nvim_cmdhist_rettv_set_number(rettv, 0);
    if ffi::nvim_cmdhist_check_secure() != 0 {
        return;
    }
    let tv0 = ffi::nvim_cmdhist_tv_idx(argvars, 0);
    let str = ffi::nvim_cmdhist_tv_get_string_chk(tv0);
    let histype = if !str.is_null() {
        crate::helpers::rs_get_histtype(str, ffi::nvim_cmdhist_strlen(str), 0)
    } else {
        HIST_INVALID
    };
    if histype == HIST_INVALID {
        return;
    }

    let tv1 = ffi::nvim_cmdhist_tv_idx(argvars, 1);
    let mut buf = [0i8; NUMBUFLEN];
    let entry = ffi::nvim_cmdhist_tv_get_string_buf(tv1, buf.as_mut_ptr());
    if *entry == 0 {
        return;
    }

    crate::modify::rs_init_history();
    crate::modify::rs_add_to_history(histype, entry, ffi::nvim_cmdhist_strlen(entry), 0, 0);
    ffi::nvim_cmdhist_rettv_set_number(rettv, 1);
}

// =============================================================================
// f_histdel
// =============================================================================

/// "histdel()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers. Accesses C history arrays via FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_f_histdel(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    let tv0 = ffi::nvim_cmdhist_tv_idx(argvars, 0);
    let str = ffi::nvim_cmdhist_tv_get_string_chk(tv0);
    let n;
    if str.is_null() {
        n = 0;
    } else {
        let tv1 = ffi::nvim_cmdhist_tv_idx(argvars, 1);
        let v_type1 = ffi::nvim_cmdhist_tv_get_type(tv1);
        let histype = crate::helpers::rs_get_histtype(str, ffi::nvim_cmdhist_strlen(str), 0);
        if v_type1 == VAR_UNKNOWN {
            // only one argument: clear entire history
            n = crate::modify::rs_clr_history(histype);
        } else if v_type1 == VAR_NUMBER {
            // index given: remove that entry
            #[allow(clippy::cast_possible_truncation)]
            let idx = ffi::nvim_cmdhist_tv_get_number(tv1) as c_int;
            n = crate::delete::rs_del_history_idx(histype, idx);
        } else {
            // string given: remove all matching entries
            let mut buf = [0i8; NUMBUFLEN];
            let pattern = ffi::nvim_cmdhist_tv_get_string_buf(tv1, buf.as_mut_ptr());
            n = crate::delete::rs_del_history_entry(histype, pattern);
        }
    }
    ffi::nvim_cmdhist_rettv_set_number(rettv, i64::from(n));
}

// =============================================================================
// f_histget
// =============================================================================

/// "histget()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers. Accesses C history arrays via FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_f_histget(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    let tv0 = ffi::nvim_cmdhist_tv_idx(argvars, 0);
    let str = ffi::nvim_cmdhist_tv_get_string_chk(tv0);
    if str.is_null() {
        ffi::nvim_cmdhist_rettv_set_string(rettv, ptr::null_mut());
    } else {
        let hist_type = crate::helpers::rs_get_histtype(str, ffi::nvim_cmdhist_strlen(str), 0);
        let tv1 = ffi::nvim_cmdhist_tv_idx(argvars, 1);
        let v_type1 = ffi::nvim_cmdhist_tv_get_type(tv1);
        let mut idx;
        if v_type1 == VAR_UNKNOWN {
            idx = get_history_idx(hist_type);
        } else {
            #[allow(clippy::cast_possible_truncation)]
            let num = ffi::nvim_cmdhist_tv_get_number_chk(tv1, ptr::null_mut()) as c_int;
            idx = num;
        }
        idx = calc_hist_idx(hist_type, idx);
        if idx < 0 {
            ffi::nvim_cmdhist_rettv_set_string(
                rettv,
                ffi::nvim_cmdhist_xstrnsave(c"".as_ptr(), 0),
            );
        } else {
            let hist = ffi::get_histentry(hist_type);
            let entry = ffi::nvim_cmdhist_he_at(hist, idx);
            let hisstr = ffi::nvim_cmdhist_he_get_hisstr(entry);
            let hisstrlen = ffi::nvim_cmdhist_he_get_hisstrlen(entry);
            ffi::nvim_cmdhist_rettv_set_string(
                rettv,
                ffi::nvim_cmdhist_xstrnsave(hisstr, hisstrlen),
            );
        }
    }
    ffi::nvim_cmdhist_rettv_set_type(rettv, VAR_STRING);
}

// =============================================================================
// f_histnr
// =============================================================================

/// "histnr()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers. Accesses C history arrays via FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_f_histnr(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    let tv0 = ffi::nvim_cmdhist_tv_idx(argvars, 0);
    let histname = ffi::nvim_cmdhist_tv_get_string_chk(tv0);
    let i = if histname.is_null() {
        HIST_INVALID
    } else {
        crate::helpers::rs_get_histtype(histname, ffi::nvim_cmdhist_strlen(histname), 0)
    };
    if i != HIST_INVALID {
        ffi::nvim_cmdhist_rettv_set_number(rettv, i64::from(get_history_idx(i)));
    } else {
        ffi::nvim_cmdhist_rettv_set_number(rettv, i64::from(HIST_INVALID));
    }
}
