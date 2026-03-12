//! VimL function implementations for the argument list
//!
//! Phase 9: f_argc, f_argidx, f_arglistid, get_arglist_as_rettv (static), f_argv

use std::ffi::c_int;

use crate::ffi::{self, EvalFuncData, TypvalPtr};
use crate::{VAR_NUMBER, VAR_STRING, VAR_UNKNOWN};

// =============================================================================
// get_arglist_as_rettv (static in C)
// =============================================================================

/// Get the argument list for a given window and set it as a list in rettv.
unsafe fn get_arglist_as_rettv(arglist: ffi::AentryPtr, argcount: c_int, rettv: TypvalPtr) {
    ffi::nvim_al_tv_list_alloc_ret(rettv, argcount);
    if !arglist.is_null() {
        for idx in 0..argcount {
            let ae = ffi::nvim_al_ae_idx(arglist, idx);
            let name = crate::query::alist_name(ae);
            ffi::nvim_al_tv_list_append_string(rettv, name, -1);
        }
    }
}

// =============================================================================
// f_argc
// =============================================================================

/// "argc([window id])" function
#[export_name = "f_argc"]
pub extern "C" fn rs_f_argc(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let tv0 = ffi::nvim_al_tv_idx(argvars, 0);
        let v_type = ffi::nvim_al_tv_get_type(tv0);
        if v_type == VAR_UNKNOWN {
            // use the current window
            ffi::nvim_al_rettv_set_number(rettv, i64::from(ffi::nvim_al_ARGCOUNT()));
        } else if v_type == VAR_NUMBER && ffi::nvim_al_tv_get_number(tv0) == -1 {
            // use the global argument list
            ffi::nvim_al_rettv_set_number(rettv, i64::from(ffi::nvim_al_GARGCOUNT()));
        } else {
            // use the argument list of the specified window
            let wp = ffi::nvim_al_find_win_by_nr_or_id(tv0);
            if wp.is_null() {
                ffi::nvim_al_rettv_set_number(rettv, -1);
            } else {
                ffi::nvim_al_rettv_set_number(rettv, i64::from(ffi::nvim_al_WARGCOUNT(wp)));
            }
        }
    }
}

// =============================================================================
// f_argidx
// =============================================================================

/// "argidx()" function
#[export_name = "f_argidx"]
pub extern "C" fn rs_f_argidx(_argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let curwin = ffi::nvim_al_get_curwin();
        let arg_idx = ffi::nvim_al_win_get_arg_idx(curwin);
        ffi::nvim_al_rettv_set_number(rettv, i64::from(arg_idx));
    }
}

// =============================================================================
// f_arglistid
// =============================================================================

/// "arglistid()" function
#[export_name = "f_arglistid"]
pub extern "C" fn rs_f_arglistid(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        ffi::nvim_al_rettv_set_number(rettv, -1);
        let tv0 = ffi::nvim_al_tv_idx(argvars, 0);
        let tv1 = ffi::nvim_al_tv_idx(argvars, 1);
        let wp = ffi::nvim_al_find_tabwin(tv0, tv1);
        if !wp.is_null() {
            let id = ffi::nvim_al_win_get_alist_id(wp);
            ffi::nvim_al_rettv_set_number(rettv, i64::from(id));
        }
    }
}

// =============================================================================
// f_argv
// =============================================================================

/// "argv(nr)" function
#[export_name = "f_argv"]
pub extern "C" fn rs_f_argv(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let tv0 = ffi::nvim_al_tv_idx(argvars, 0);
        let v_type0 = ffi::nvim_al_tv_get_type(tv0);

        if v_type0 == VAR_UNKNOWN {
            get_arglist_as_rettv(ffi::nvim_al_ARGLIST(), ffi::nvim_al_ARGCOUNT(), rettv);
            return;
        }

        let mut arglist: ffi::AentryPtr = std::ptr::null_mut();
        let mut argcount: c_int = -1;

        let tv1 = ffi::nvim_al_tv_idx(argvars, 1);
        let v_type1 = ffi::nvim_al_tv_get_type(tv1);

        if v_type1 == VAR_UNKNOWN {
            arglist = ffi::nvim_al_ARGLIST();
            argcount = ffi::nvim_al_ARGCOUNT();
        } else if v_type1 == VAR_NUMBER && ffi::nvim_al_tv_get_number(tv1) == -1 {
            arglist = ffi::nvim_al_GARGLIST();
            argcount = ffi::nvim_al_GARGCOUNT();
        } else {
            let wp = ffi::nvim_al_find_win_by_nr_or_id(tv1);
            if !wp.is_null() {
                arglist = ffi::nvim_al_WARGLIST(wp);
                argcount = ffi::nvim_al_WARGCOUNT(wp);
            }
        }

        ffi::nvim_al_rettv_set_type(rettv, VAR_STRING);
        ffi::nvim_al_rettv_set_string(rettv, std::ptr::null_mut());

        #[allow(clippy::cast_possible_truncation)]
        let idx = ffi::nvim_al_tv_get_number_chk(tv0, std::ptr::null_mut()) as c_int;
        if !arglist.is_null() && idx >= 0 && idx < argcount {
            let ae = ffi::nvim_al_ae_idx(arglist, idx);
            let name = crate::query::alist_name(ae);
            ffi::nvim_al_rettv_set_string(rettv, ffi::nvim_al_xstrdup(name));
        } else if idx == -1 {
            get_arglist_as_rettv(arglist, argcount, rettv);
        }
    }
}
