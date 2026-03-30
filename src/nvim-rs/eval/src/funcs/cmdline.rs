//! VimL built-in cmdline functions.
//!
//! Migrated from `src/nvim/eval/funcs.c`:
//! - `f_getcmdcomplpat` — get current cmdline completion pattern
//! - `f_getcmdcompltype` — get current cmdline completion type
//! - `f_setcmdline` — set the command line contents
//! - `f_setcmdpos` — set the cursor position in the command line
//! - `f_wildtrigger` — trigger wildmenu completion

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{c_char, c_int, c_void};

use super::dispatch::{rettv_set_number, TypevalPtrMut};

// =============================================================================
// C accessor declarations
// =============================================================================

extern "C" {
    // Cmdline star (secret mode) check
    fn nvim_eval_get_cmdline_star() -> c_int;

    // CmdlineInfo accessors (returned as opaque *mut c_void pointers)
    fn nvim_get_ccline_ptr() -> *mut c_void;
    fn nvim_eval_ccline_get_xpc(p: *mut c_void) -> *mut c_void;

    // expand_T (xpc) field access
    fn nvim_eval_xpc_get_context(xpc: *mut c_void) -> c_int;
    fn nvim_eval_xpc_set_context(xpc: *mut c_void, ctx: c_int);
    fn nvim_eval_xpc_set_expand_context(xpc: *mut c_void);
    fn nvim_eval_xpc_get_pattern(xpc: *mut c_void) -> *const c_char;
    fn nvim_eval_xpc_dup_pattern(xpc: *mut c_void) -> *mut c_char;
    fn nvim_eval_cmdcomplete_type_to_str(ctx: c_int, xpc: *mut c_void) -> *mut c_char;

    // setcmdline combined helper (sets contents, pos, redraws, fires autocmd)
    fn nvim_eval_setcmdline_str(p: *mut c_void, str_: *const c_char, pos: c_int) -> c_int;

    // setcmdpos helpers
    fn nvim_set_new_cmdpos(val: c_int);

    // wildtrigger helpers
    fn nvim_eval_wildtrigger_possible() -> c_int;
    fn nvim_eval_get_cmdline_type() -> c_int;
    fn nvim_eval_ins_k_wild();

    // typval helpers
    fn nvim_tv_set_string_copy(tv: *mut c_void, s: *const u8, len: c_int);
    #[link_name = "tv_get_string"]
    fn cl_tv_get_string(tv: *mut c_void) -> *const c_char;
    #[link_name = "tv_get_number"]
    fn cl_tv_get_number(tv: *const c_void) -> i64;
    #[link_name = "tv_get_number_chk"]
    fn cl_tv_get_number_chk(tv: *mut c_void, error: *mut bool) -> i64;
    #[link_name = "tv_check_for_string_arg"]
    fn cl_tv_check_for_string_arg(argvars: *const c_void, idx: c_int) -> c_int;
    #[link_name = "tv_check_for_opt_number_arg"]
    fn cl_tv_check_for_opt_number_arg(argvars: *const c_void, idx: c_int) -> c_int;
    fn emsg(s: *const c_char) -> c_int;
}

// TYPVAL_SZ: size of typval_T for pointer arithmetic
const TYPVAL_SZ: usize = 16;
// FAIL constant (C return value for error)
const FAIL: c_int = 0;

// Expand context constants (from cmdexpand_defs.h)
const EXPAND_NOTHING: c_int = 0;
const EXPAND_UNSUCCESSFUL: c_int = -2;

// =============================================================================
// "getcmdcomplpat()" function
// =============================================================================

/// "getcmdcomplpat()" function - get the current cmdline completion pattern.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_getcmdcomplpat"]
pub unsafe extern "C" fn rs_f_getcmdcomplpat(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    // Set return type to VAR_STRING with NULL default
    nvim_tv_set_string_copy(rettv, std::ptr::null(), 0);

    if nvim_eval_get_cmdline_star() > 0 {
        return;
    }
    let p = nvim_get_ccline_ptr();
    if p.is_null() {
        return;
    }
    let xpc = nvim_eval_ccline_get_xpc(p);
    if xpc.is_null() {
        return;
    }

    let mut xp_context = nvim_eval_xpc_get_context(xpc);
    if xp_context == EXPAND_NOTHING {
        nvim_eval_xpc_set_expand_context(xpc);
        xp_context = nvim_eval_xpc_get_context(xpc);
        nvim_eval_xpc_set_context(xpc, EXPAND_NOTHING);
    }
    if xp_context == EXPAND_UNSUCCESSFUL || nvim_eval_xpc_get_pattern(xpc).is_null() {
        return;
    }
    // Return xstrdup of xp_pattern (takes ownership, freed by VimL GC)
    let s = nvim_eval_xpc_dup_pattern(xpc);
    // Set rettv->v_type = VAR_STRING; rettv->vval.v_string = s (transfer ownership)
    // We use nvim_tv_set_string_copy with length -1 to xstrdup, but we already
    // have an allocated string. Use the raw pattern by setting via a direct call.
    // Actually nvim_tv_set_string_copy xstrdup's the input, but we want to transfer
    // ownership of s directly. Use len=-1 to indicate NUL-terminated copy.
    nvim_tv_set_string_copy(rettv, s.cast::<u8>(), -1);
    // Free the xstrdup copy since nvim_tv_set_string_copy made another copy
    libc::free(s.cast::<libc::c_void>());
}

// =============================================================================
// "getcmdcompltype()" function
// =============================================================================

/// "getcmdcompltype()" function - get the current cmdline completion type.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_getcmdcompltype"]
pub unsafe extern "C" fn rs_f_getcmdcompltype(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_tv_set_string_copy(rettv, std::ptr::null(), 0);

    if nvim_eval_get_cmdline_star() > 0 {
        return;
    }
    let p = nvim_get_ccline_ptr();
    if p.is_null() {
        return;
    }
    let xpc = nvim_eval_ccline_get_xpc(p);
    if xpc.is_null() {
        return;
    }

    let mut xp_context = nvim_eval_xpc_get_context(xpc);
    if xp_context == EXPAND_NOTHING {
        nvim_eval_xpc_set_expand_context(xpc);
        xp_context = nvim_eval_xpc_get_context(xpc);
        nvim_eval_xpc_set_context(xpc, EXPAND_NOTHING);
    }
    if xp_context == EXPAND_UNSUCCESSFUL {
        return;
    }
    let s = nvim_eval_cmdcomplete_type_to_str(xp_context, xpc);
    nvim_tv_set_string_copy(rettv, s.cast::<u8>(), -1);
    libc::free(s.cast::<libc::c_void>());
}

// =============================================================================
// "setcmdline()" function
// =============================================================================

/// "setcmdline()" function - set the command line to str at position pos.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_setcmdline"]
pub unsafe extern "C" fn rs_f_setcmdline(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    // Initialize rettv to 0 (success indicator: 0=ok, 1=not in cmdline)
    // The default v_number is 0 (VAR_NUMBER default).
    // Check arg types
    if cl_tv_check_for_string_arg(argvars, 0) == FAIL
        || cl_tv_check_for_opt_number_arg(argvars, 1) == FAIL
    {
        return;
    }

    let mut pos: c_int = -1;
    // Check argvars[1]
    let av1 = argvars.cast::<u8>().add(TYPVAL_SZ).cast::<c_void>();
    let av1_type = *(av1.cast::<c_int>()); // v_type is first field (c_int = 4 bytes)
    if av1_type != 0
    // VAR_UNKNOWN = 0
    {
        let mut error = false;
        #[allow(clippy::cast_possible_truncation)]
        let n = cl_tv_get_number_chk(av1.cast_mut(), &raw mut error) as c_int - 1;
        if error {
            return;
        }
        if n < 0 {
            emsg(c"E487: Argument must be positive".as_ptr());
            return;
        }
        pos = n;
    }

    let str_ = cl_tv_get_string(argvars.cast_mut());
    let p = nvim_get_ccline_ptr();
    if p.is_null() {
        // Not in cmdline mode: return 1
        rettv_set_number(TypevalPtrMut::from_raw(rettv), 1);
    } else {
        nvim_eval_setcmdline_str(p, str_, pos);
        // rettv->vval.v_number = 0 (success) — already the default
    }
}

// =============================================================================
// "setcmdpos()" function
// =============================================================================

/// "setcmdpos()" function - set the cursor position in the command line.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_setcmdpos"]
pub unsafe extern "C" fn rs_f_setcmdpos(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    #[allow(clippy::cast_possible_truncation)]
    let pos = cl_tv_get_number(argvars) as c_int - 1;
    if pos >= 0 {
        let p = nvim_get_ccline_ptr();
        if p.is_null() {
            // Not in cmdline mode: return 1
            rettv_set_number(TypevalPtrMut::from_raw(rettv), 1);
        } else {
            nvim_set_new_cmdpos(pos.max(0));
            // rettv->vval.v_number = 0 (success) — already the default
        }
    }
}

// =============================================================================
// "wildtrigger()" function
// =============================================================================

/// "wildtrigger()" function - trigger wildmenu completion.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_wildtrigger"]
pub unsafe extern "C" fn rs_f_wildtrigger(
    _argvars: *const c_void,
    _rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    if nvim_eval_wildtrigger_possible() == 0 {
        return;
    }
    let cmd_type = nvim_eval_get_cmdline_type();
    if cmd_type == c_int::from(b':')
        || cmd_type == c_int::from(b'/')
        || cmd_type == c_int::from(b'?')
    {
        nvim_eval_ins_k_wild();
    }
}
