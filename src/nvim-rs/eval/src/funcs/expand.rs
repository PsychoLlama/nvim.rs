//! Rust implementation of the VimL `expand()` built-in function.
//!
//! Ported from `f_expand` in `src/nvim/eval/funcs_shim.c` (Phase 30).
//! The C body has been deleted; this file is the authoritative implementation.

#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_ptr_alignment)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::ptr_cast_constness)]
#![allow(clashing_extern_declarations)]

use std::ffi::{c_char, c_int, c_void};

use crate::typval::TypvalT;

// ─── Constants ────────────────────────────────────────────────────────────────

const VAR_UNKNOWN: c_int = 0;
const VAR_STRING: c_int = 2;
const VAR_LIST: c_int = 4;

/// `WILD_SILENT | WILD_USE_NL | WILD_LIST_NOTFOUND`
const WILD_SILENT: c_int = 0x40;
const WILD_USE_NL: c_int = 0x04;
const WILD_LIST_NOTFOUND: c_int = 0x01;
const WILD_KEEP_ALL: c_int = 0x20;
const WILD_ICASE: c_int = 0x100;

/// `WILD_ALL = 6`, `WILD_ALL_KEEP = 8`
const WILD_ALL: c_int = 6;
const WILD_ALL_KEEP: c_int = 8;

/// `EXPAND_FILES = 2`
const EXPAND_FILES: c_int = 2;

/// `expand_T` is 392 bytes; pinned by `_Static_assert(sizeof(expand_T) == 392)` in C.
const EXPAND_T_SIZE: usize = 392;

/// `EvalFuncData` — opaque union
type EvalFuncData = *mut c_void;

// ─── FFI imports ─────────────────────────────────────────────────────────────

extern "C" {
    fn tv_get_string(tv: *const c_void) -> *const c_char;
    fn tv_get_number_chk(tv: *const c_void, error: *mut bool) -> i64;

    // tv_list_set_ret: set rettv to VAR_LIST with the given list_T (may be NULL)
    fn nvim_eval_tv_list_set_ret(tv: *mut c_void, l: *mut c_void);

    fn tv_list_alloc_ret(rettv: *mut c_void, len: isize) -> *mut c_void;
    fn tv_list_append_string(l: *mut c_void, s: *const c_char, len: isize);

    // eval_vars: expand % # < etc
    fn eval_vars(
        src: *mut c_char,
        srcstart: *const c_char,
        usedlen: *mut usize,
        lnump: *mut c_void,
        errormsg: *mut *const c_char,
        escaped: *mut c_void,
        empty_is_spec: bool,
    ) -> *mut c_char;

    // emsg_off inc/dec
    fn nvim_eval_emsg_off_inc();
    fn nvim_eval_emsg_off_dec();
    fn nvim_eval_get_p_verbose() -> c_int;
    fn emsg(s: *const c_char) -> bool;

    // p_wic (wildignorecase) accessor
    fn nvim_eval_get_p_wic() -> c_int;

    // expand_T lifecycle
    fn ExpandInit(xp: *mut c_void);
    fn ExpandOne(
        xp: *mut c_void,
        s: *const c_char,
        orig: *mut c_char,
        options: c_int,
        mode: c_int,
    ) -> *mut c_char;
    fn ExpandCleanup(xp: *mut c_void);

    // xp_context field accessor (for setting EXPAND_FILES)
    fn nvim_eval_xpc_set_context(xpc: *mut c_void, ctx: c_int);

    // xp_numfiles / xp_files field accessors
    fn nvim_eval_expand_numfiles(xpc: *mut c_void) -> c_int;
    fn nvim_eval_expand_file_at(xpc: *mut c_void, i: c_int) -> *const c_char;

    // p_csl save/restore (#ifdef-guarded, no-ops on Linux)
    fn nvim_eval_expand_save_csl();
    fn nvim_eval_expand_restore_csl();

    fn xfree(p: *mut c_void);
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Return a pointer to `argvars[i]` (each typval_T is 16 bytes).
#[inline]
unsafe fn argvar(argvars: *const c_void, i: usize) -> *const c_void {
    argvars.cast::<u8>().add(i * 16).cast::<c_void>()
}

// ─── f_expand ────────────────────────────────────────────────────────────────

/// `expand()` VimL function — filename/variable expansion.
///
/// # Safety
///
/// `argvars` and `rettv` must be valid `typval_T *`.
#[export_name = "f_expand"]
pub unsafe extern "C" fn rs_f_expand(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: EvalFuncData,
) {
    let mut options = WILD_SILENT | WILD_USE_NL | WILD_LIST_NOTFOUND;
    let mut error = false;

    // Save/restore p_csl (Windows-only; no-op on Linux)
    nvim_eval_expand_save_csl();

    // Start with VAR_STRING return type
    (*rettv.cast::<TypvalT>()).v_type = VAR_STRING;

    // If arg2 is present and truthy → switch to list return type
    let arg1 = argvar(argvars, 1);
    let arg2 = argvar(argvars, 2);
    if (*arg1.cast::<TypvalT>()).v_type != VAR_UNKNOWN
        && (*arg2.cast::<TypvalT>()).v_type != VAR_UNKNOWN
        && tv_get_number_chk(arg2, &raw mut error) != 0
        && !error
    {
        nvim_eval_tv_list_set_ret(rettv, std::ptr::null_mut());
    }

    let s: *const c_char = tv_get_string(argvar(argvars, 0));
    let first = *s.cast::<u8>();

    if first == b'%' || first == b'#' || first == b'<' {
        // Special expansion: % # <...>
        if nvim_eval_get_p_verbose() == 0 {
            nvim_eval_emsg_off_inc();
        }
        let mut usedlen: usize = 0;
        let mut errormsg: *const c_char = std::ptr::null();
        let result = eval_vars(
            s.cast_mut(),
            s,
            &raw mut usedlen,
            std::ptr::null_mut(),
            &raw mut errormsg,
            std::ptr::null_mut(),
            false,
        );
        if nvim_eval_get_p_verbose() == 0 {
            nvim_eval_emsg_off_dec();
        } else if !errormsg.is_null() {
            emsg(errormsg);
        }

        if (*rettv.cast::<TypvalT>()).v_type == VAR_LIST {
            let retlist = tv_list_alloc_ret(rettv, isize::from(!result.is_null()));
            if !result.is_null() {
                tv_list_append_string(retlist, result, -1);
            }
            xfree(result.cast::<c_void>());
        } else {
            (*rettv.cast::<TypvalT>()).vval.v_string = result;
        }
    } else {
        // Wildcard expansion
        if (*arg1.cast::<TypvalT>()).v_type != VAR_UNKNOWN
            && tv_get_number_chk(arg1, &raw mut error) != 0
        {
            options |= WILD_KEEP_ALL;
        }

        if error {
            (*rettv.cast::<TypvalT>()).vval.v_string = std::ptr::null_mut();
        } else {
            // Stack-allocate expand_T as opaque byte buffer
            let mut xpc_buf = [0u8; EXPAND_T_SIZE];
            let xpc = xpc_buf.as_mut_ptr().cast::<c_void>();
            ExpandInit(xpc);
            nvim_eval_xpc_set_context(xpc, EXPAND_FILES);
            if nvim_eval_get_p_wic() != 0 {
                options += WILD_ICASE;
            }

            if (*rettv.cast::<TypvalT>()).v_type == VAR_STRING {
                (*rettv.cast::<TypvalT>()).vval.v_string =
                    ExpandOne(xpc, s, std::ptr::null_mut(), options, WILD_ALL);
            } else {
                ExpandOne(xpc, s, std::ptr::null_mut(), options, WILD_ALL_KEEP);
                let numfiles = nvim_eval_expand_numfiles(xpc);
                let retlist = tv_list_alloc_ret(rettv, numfiles as isize);
                for i in 0..numfiles {
                    let fname = nvim_eval_expand_file_at(xpc, i);
                    tv_list_append_string(retlist, fname, -1);
                }
                ExpandCleanup(xpc);
            }
        }
    }

    nvim_eval_expand_restore_csl();
}
