//! Miscellaneous variable utility functions for VimL.
//!
//! Phase 9: Migrated from `src/nvim/eval/vars.c`.
//! Phase 11: Migrated eval_charconvert, eval_diff, eval_patch.
//!
//! Functions:
//! - `rs_garbage_collect_scriptvars`: GC mark pass for script variables
//! - `rs_set_internal_string_var`: Set an internal string variable by name
//! - `rs_eval_charconvert`: Evaluate charconvert expression
//! - `rs_eval_diff`: Evaluate diffexpr
//! - `rs_eval_patch`: Evaluate patchexpr

#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::manual_c_str_literals)]
#![allow(clippy::if_not_else)]

use std::ffi::{c_char, c_int, c_void};

// VAR_STRING typval type constant (matches C VarType::VAR_STRING = 1)
const VAR_STRING: c_int = 1;

// Typval size in bytes (must match sizeof(typval_T) = 24)
const TYPVAL_SIZE: usize = 24;

// OK / FAIL return codes
const OK: c_int = 1;
const FAIL: c_int = 0;

// VimVarIndex constants for charconvert/diff/patch/spell vimvars
const VV_CC_FROM: c_int = 16;
const VV_CC_TO: c_int = 17;
const VV_FNAME_IN: c_int = 18;
const VV_FNAME_OUT: c_int = 19;
const VV_FNAME_NEW: c_int = 20;
const VV_FNAME_DIFF: c_int = 21;

// =============================================================================
// C extern declarations
// =============================================================================

extern "C" {
    // --- script vars iteration ---
    fn nvim_script_items_len() -> c_int;
    fn nvim_get_script_vars_dict(sid: c_int) -> *mut c_void;
    fn nvim_dict_get_hashtab(dict: *mut c_void) -> *mut c_void;

    // --- set_var ---
    fn set_var(name: *const c_char, name_len: usize, tv: *const c_void, copy: bool);

    // --- string ops ---
    fn strlen(s: *const c_char) -> usize;

    // --- GC ---
    fn rs_set_ref_in_ht(ht: *mut c_void, copy_id: c_int, list_stack: *mut *mut c_void) -> bool;

    // (rs_set_vim_var_string is called directly as crate::vimvar_accessors::rs_set_vim_var_string)

    // --- sctx save/restore/apply (Phase 11) ---
    // nvim_save_current_sctx: defined in vars.c, returns heap-allocated sctx_T*
    fn nvim_save_current_sctx() -> *mut c_void;
    // nvim_restore_current_sctx: defined in eval_shim.c, takes sctx_T*, frees it
    fn nvim_restore_current_sctx(s: *mut c_void);
    fn nvim_apply_option_sctx(opt: c_int);

    // --- option constants (Phase 11) ---
    fn nvim_kopt_charconvert() -> c_int;
    fn nvim_kopt_diffexpr() -> c_int;
    fn nvim_kopt_patchexpr() -> c_int;

    // --- eval expressions (Phase 11) ---
    fn nvim_eval_charconvert_expr() -> bool;
    fn nvim_eval_diffexpr();
    fn nvim_eval_patchexpr();

    // --- eval_spell_expr (Phase 12) ---
    fn nvim_prepare_vimvar(idx: c_int, save_tv: *mut c_void);
    fn nvim_restore_vimvar(idx: c_int, save_tv: *mut c_void);
    fn nvim_get_p_verbose() -> c_int;
    fn nvim_may_call_simple_func(p: *const c_char, rettv: *mut c_void) -> c_int;
    fn nvim_eval1_evaluate(arg: *mut *mut c_char, rettv: *mut c_void) -> c_int;
    fn nvim_apply_spellsuggest_sctx();
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn tv_clear(tv: *mut c_void);
    fn nvim_tv_get_type(tv: *mut c_void) -> c_int;
    fn nvim_tv_get_list(tv: *mut c_void) -> *mut c_void;
    fn nvim_emsg_off_inc();
    fn nvim_emsg_off_dec();
}

/// Mark all script variable hashtabs as referenced for garbage collection.
///
/// Matches C `garbage_collect_scriptvars`. Returns true if GC should abort.
///
/// # Safety
/// Called only during the GC mark phase with a valid copy_id.
#[no_mangle]
pub unsafe extern "C" fn rs_garbage_collect_scriptvars(copy_id: c_int) -> bool {
    let mut abort = false;
    let len = nvim_script_items_len();
    for i in 1..=len {
        let dict = nvim_get_script_vars_dict(i);
        if !dict.is_null() {
            let ht = nvim_dict_get_hashtab(dict);
            if !ht.is_null() {
                abort = abort || rs_set_ref_in_ht(ht, copy_id, std::ptr::null_mut());
            }
        }
    }
    abort
}

/// Set an internal variable to a string value.
///
/// Matches C `set_internal_string_var`. Creates the variable if it does not
/// already exist.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string.
/// `value` must be a valid NUL-terminated C string or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_set_internal_string_var(name: *const c_char, value: *mut c_char) {
    // Build a VAR_STRING typval on the heap.
    let mut tv_buf = [0u8; TYPVAL_SIZE];
    let tv = tv_buf.as_mut_ptr() as *mut c_void;

    // v_type at offset 0
    let vtype_ptr = tv as *mut c_int;
    *vtype_ptr = VAR_STRING;

    // vval.v_string at offset 8
    let vstring_ptr = tv.add(8) as *mut *mut c_char;
    *vstring_ptr = value;

    let name_len = strlen(name);
    set_var(name, name_len, tv, true);
}

/// Evaluate the 'charconvert' expression.
///
/// Matches C `eval_charconvert`. Returns OK (1) on success, FAIL (0) on error.
///
/// # Safety
/// All string arguments must be valid NUL-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_charconvert(
    enc_from: *const c_char,
    enc_to: *const c_char,
    fname_from: *const c_char,
    fname_to: *const c_char,
) -> c_int {
    let saved_sctx = nvim_save_current_sctx();

    crate::vimvar_accessors::rs_set_vim_var_string(VV_CC_FROM, enc_from, -1);
    crate::vimvar_accessors::rs_set_vim_var_string(VV_CC_TO, enc_to, -1);
    crate::vimvar_accessors::rs_set_vim_var_string(VV_FNAME_IN, fname_from, -1);
    crate::vimvar_accessors::rs_set_vim_var_string(VV_FNAME_OUT, fname_to, -1);

    nvim_apply_option_sctx(nvim_kopt_charconvert());

    let err = nvim_eval_charconvert_expr();

    crate::vimvar_accessors::rs_set_vim_var_string(VV_CC_FROM, std::ptr::null(), -1);
    crate::vimvar_accessors::rs_set_vim_var_string(VV_CC_TO, std::ptr::null(), -1);
    crate::vimvar_accessors::rs_set_vim_var_string(VV_FNAME_IN, std::ptr::null(), -1);
    crate::vimvar_accessors::rs_set_vim_var_string(VV_FNAME_OUT, std::ptr::null(), -1);

    nvim_restore_current_sctx(saved_sctx);

    if err {
        FAIL
    } else {
        OK
    }
}

/// Evaluate the 'diffexpr' expression.
///
/// Matches C `eval_diff`. Errors are ignored.
///
/// # Safety
/// All string arguments must be valid NUL-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_diff(
    origfile: *const c_char,
    newfile: *const c_char,
    outfile: *const c_char,
) {
    let saved_sctx = nvim_save_current_sctx();

    crate::vimvar_accessors::rs_set_vim_var_string(VV_FNAME_IN, origfile, -1);
    crate::vimvar_accessors::rs_set_vim_var_string(VV_FNAME_NEW, newfile, -1);
    crate::vimvar_accessors::rs_set_vim_var_string(VV_FNAME_OUT, outfile, -1);

    nvim_apply_option_sctx(nvim_kopt_diffexpr());
    nvim_eval_diffexpr();

    crate::vimvar_accessors::rs_set_vim_var_string(VV_FNAME_IN, std::ptr::null(), -1);
    crate::vimvar_accessors::rs_set_vim_var_string(VV_FNAME_NEW, std::ptr::null(), -1);
    crate::vimvar_accessors::rs_set_vim_var_string(VV_FNAME_OUT, std::ptr::null(), -1);

    nvim_restore_current_sctx(saved_sctx);
}

/// Evaluate the 'patchexpr' expression.
///
/// Matches C `eval_patch`. Errors are ignored.
///
/// # Safety
/// All string arguments must be valid NUL-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_patch(
    origfile: *const c_char,
    difffile: *const c_char,
    outfile: *const c_char,
) {
    let saved_sctx = nvim_save_current_sctx();

    crate::vimvar_accessors::rs_set_vim_var_string(VV_FNAME_IN, origfile, -1);
    crate::vimvar_accessors::rs_set_vim_var_string(VV_FNAME_DIFF, difffile, -1);
    crate::vimvar_accessors::rs_set_vim_var_string(VV_FNAME_OUT, outfile, -1);

    nvim_apply_option_sctx(nvim_kopt_patchexpr());
    nvim_eval_patchexpr();

    crate::vimvar_accessors::rs_set_vim_var_string(VV_FNAME_IN, std::ptr::null(), -1);
    crate::vimvar_accessors::rs_set_vim_var_string(VV_FNAME_DIFF, std::ptr::null(), -1);
    crate::vimvar_accessors::rs_set_vim_var_string(VV_FNAME_OUT, std::ptr::null(), -1);

    nvim_restore_current_sctx(saved_sctx);
}

// =============================================================================
// Phase 12: eval_spell_expr
// =============================================================================

// VV_VAL index (35 in the VimVarIndex enum)
const VV_VAL: c_int = 35;
// VAR_LIST type constant (matches VarType::VAR_LIST = 4)
const VAR_LIST_TYPE: c_int = 4;
// NOTDONE constant
const NOTDONE: c_int = 2;

/// Evaluate an expression to a list with suggestions.
///
/// For the "expr:" part of 'spellsuggest'. Returns NULL on error.
///
/// Matches C `eval_spell_expr`.
///
/// # Safety
/// `badword` and `expr` must be valid NUL-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_spell_expr(
    badword: *const c_char,
    expr: *mut c_char,
) -> *mut c_void {
    let p = skipwhite(expr);
    let saved_sctx = nvim_save_current_sctx();

    let mut save_val_buf = [0u8; TYPVAL_SIZE];
    let save_val = save_val_buf.as_mut_ptr() as *mut c_void;

    nvim_prepare_vimvar(VV_VAL, save_val);
    crate::vimvar_accessors::rs_set_vim_var_string(VV_VAL, badword, -1);

    if nvim_get_p_verbose() == 0 {
        nvim_emsg_off_inc();
    }

    nvim_apply_spellsuggest_sctx();

    let mut rettv_buf = [0u8; TYPVAL_SIZE];
    let rettv = rettv_buf.as_mut_ptr() as *mut c_void;
    // Initialize rettv.v_type = VAR_UNKNOWN (0)
    *(rettv as *mut c_int) = 0;

    let r = nvim_may_call_simple_func(p, rettv);
    let r = if r == NOTDONE {
        let mut p_mut = p;
        nvim_eval1_evaluate(std::ptr::addr_of_mut!(p_mut), rettv)
    } else {
        r
    };

    let list_ptr: *mut c_void = if r == OK {
        if nvim_tv_get_type(rettv) != VAR_LIST_TYPE {
            tv_clear(rettv);
            std::ptr::null_mut()
        } else {
            nvim_tv_get_list(rettv)
        }
    } else {
        std::ptr::null_mut()
    };

    if nvim_get_p_verbose() == 0 {
        nvim_emsg_off_dec();
    }

    // Clear v:val and restore
    let vv_val_tv = crate::vimvar_accessors::rs_get_vim_var_tv(VV_VAL);
    tv_clear(vv_val_tv.cast());
    nvim_restore_vimvar(VV_VAL, save_val);
    nvim_restore_current_sctx(saved_sctx);

    list_ptr
}
