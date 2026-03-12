//! Option apply pipeline: did_set_option, set_option, apply_optionset_autocmd
//!
//! This module provides Rust implementations of the core option-setting pipeline.
//! All complex C struct construction and function pointer calls are delegated to
//! C trampolines to avoid FFI layout issues.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]

use std::ffi::{c_char, c_int, c_uint};

use crate::storage::OptVal;
use crate::{
    K_OPT_FLAG_CURSWANT, K_OPT_FLAG_HL_ONLY, K_OPT_FLAG_INSECURE, K_OPT_FLAG_REDR_ALL,
    K_OPT_FLAG_SECURE, K_OPT_FLAG_UI_OPTION, OPT_MODELINE, SID_NONE, UPD_NOT_VALID,
};

// =============================================================================
// C External Declarations
// =============================================================================

extern "C" {
    // Phase 9 trampoline: invoke the option's did_set_cb (constructs optset_T in C)
    fn nvim_invoke_did_set_cb(
        opt_idx: c_int,
        varp: *mut std::ffi::c_void,
        old_data: OptVal,
        new_data: OptVal,
        opt_flags: c_int,
        errbuf: *mut c_char,
        errbuflen: usize,
        value_changed_out: *mut c_int,
        value_checked_out: *mut c_int,
        restore_chartab_out: *mut c_int,
    ) -> *const c_char;

    // Check if option has a did_set_cb
    fn nvim_option_has_did_set_cb(opt_idx: c_int) -> c_int;

    // Error message accessors
    fn nvim_get_e_unsupportedoption() -> *const c_char;
    fn nvim_get_e_secure() -> *const c_char;
    fn nvim_get_e_invarg() -> *const c_char;

    // Option metadata accessors
    fn nvim_get_option_immutable(opt_idx: c_int) -> c_int;
    fn nvim_option_get_flags_val(opt_idx: c_int) -> c_uint;
    fn nvim_option_set_was_set_flag(opt_idx: c_int);

    // Security/sandbox state
    fn nvim_get_secure() -> c_int;
    fn nvim_get_sandbox() -> c_int;

    // Illegal path name check
    fn nvim_check_illegal_path_names(varp: *mut std::ffi::c_void, flags: c_uint) -> c_int;

    // OptVal value operations (already in Rust but called from C side via FFI)
    fn rs_optval_equal(o1: OptVal, o2: OptVal) -> c_int;
    fn rs_optval_free(o: OptVal);
    fn rs_optval_copy(o: OptVal) -> OptVal;

    // varp dispatch
    fn nvim_get_varp_scope_opt(opt_idx: c_int, opt_flags: c_int) -> *mut std::ffi::c_void;
    fn nvim_get_varp_opt(opt_idx: c_int) -> *mut std::ffi::c_void;
    fn nvim_option_get_var_ptr(opt_idx: c_int) -> *mut std::ffi::c_void;

    // Read/write option variable pointer values
    fn rs_optval_from_varp(opt_idx: c_int, varp: *mut std::ffi::c_void) -> OptVal;
    fn rs_set_option_varp(
        opt_idx: c_int,
        varp: *mut std::ffi::c_void,
        value: OptVal,
        free_oldval: c_int,
    );

    // option global-local check
    fn rs_option_is_global_local(opt_idx: c_int) -> c_int;
    fn rs_get_option_unset_value(opt_idx: c_int) -> OptVal;

    // Script context handling: construct sctx from set_sid and call set_option_sctx
    // Uses C to avoid sctx_T layout issues (sc_chan field missing in Rust's ScriptContext)
    fn nvim_set_option_sctx_from_sid(opt_idx: c_int, opt_flags: c_int, set_sid: c_int);

    // buf_init_chartab(curbuf, true)
    fn nvim_call_buf_init_chartab();

    // Pointer address comparisons for special-cased options
    fn nvim_curbuf_b_p_syn_addr() -> *mut std::ffi::c_void;
    fn nvim_curbuf_b_p_ft_addr() -> *mut std::ffi::c_void;
    fn nvim_curwin_b_p_spl_addr() -> *mut std::ffi::c_void;
    fn nvim_get_p_mouse_addr() -> *mut std::ffi::c_void;
    fn nvim_get_p_flp_addr() -> *mut std::ffi::c_void;
    fn nvim_curbuf_b_p_flp_addr() -> *mut std::ffi::c_void;
    fn nvim_get_p_wbr_addr() -> *mut std::ffi::c_void;
    fn nvim_curwin_p_wbr_addr() -> *mut std::ffi::c_void;

    // Special autocmds for file type / syntax / spell
    fn rs_do_syntax_autocmd(buf: *mut std::ffi::c_void, value_changed: c_int);
    fn nvim_do_filetype_autocmd(value_changed: c_int);
    fn rs_do_spelllang_source(win: *mut std::ffi::c_void);

    // curbuf / curwin accessors
    fn nvim_opt_get_curbuf() -> *mut std::ffi::c_void;
    fn nvim_opt_get_curwin() -> *mut std::ffi::c_void;
    fn nvim_curwin_get_w_curswant() -> c_int;
    fn nvim_curwin_set_w_set_curswant(val: c_int);
    fn nvim_curwin_get_w_briopt_list() -> c_int;

    // comp_col(), setmouse(), redraw_all_later(), set_winbar()
    fn nvim_call_comp_col();
    fn nvim_call_setmouse();
    fn nvim_call_redraw_all_later(kind: c_int);
    fn nvim_call_set_winbar();
    fn rs_check_redraw_for(buf: *mut std::ffi::c_void, win: *mut std::ffi::c_void, flags: c_uint);

    // insecure_flag pointer
    #[link_name = "insecure_flag"]
    fn rs_insecure_flag(wp: *mut std::ffi::c_void, opt_idx: c_int, opt_flags: c_int)
        -> *mut c_uint;

    // opt_flags constants accessors
    fn nvim_get_maxcol() -> c_int;

    // is_option_local_value_unset (gated behind #[cfg(not(test))] in value.rs, call via FFI)
    fn rs_is_option_local_value_unset(opt_idx: c_int) -> c_int;

    // validate option value (gated behind #[cfg(not(test))] in validate.rs, call via FFI)
    fn rs_validate_option_value(
        opt_idx: c_int,
        newval: *mut OptVal,
        opt_flags: c_int,
        errbuf: *mut c_char,
        errbuflen: usize,
    ) -> *const c_char;

    // set_option support
    fn nvim_get_starting() -> c_int;
    fn nvim_set_secure(val: c_int);
    fn nvim_apply_optionset_autocmd(
        opt_idx: c_int,
        opt_flags: c_int,
        oldval: OptVal,
        oldval_g: OptVal,
        oldval_l: OptVal,
        newval: OptVal,
        errmsg: *const c_char,
    );
    fn nvim_ui_call_option_set(opt_idx: c_int, saved_new_value: OptVal);
}

// OPT_LOCAL / OPT_GLOBAL flags
const OPT_LOCAL: c_int = 0x02;
const OPT_GLOBAL: c_int = 0x01;

// =============================================================================
// rs_did_set_option
// =============================================================================

/// Rust implementation of did_set_option.
///
/// Processes side effects after an option value is changed: validates,
/// invokes callbacks, sets flags, triggers autocmds, and redraws.
///
/// # Safety
/// All pointer arguments must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_option(
    opt_idx: c_int,
    varp: *mut std::ffi::c_void,
    old_value: OptVal,
    new_value: OptVal,
    opt_flags: c_int,
    set_sid: c_int,
    direct: c_int,
    value_replaced: c_int,
    errbuf: *mut c_char,
    errbuflen: usize,
) -> *const c_char {
    let mut errmsg: *const c_char = std::ptr::null();
    let mut restore_chartab = 0i32;
    let mut value_changed = 0i32;
    let mut value_checked = 0i32;

    let opt_flags_val = nvim_option_get_flags_val(opt_idx);

    if direct != 0 {
        // Don't do any extra processing if setting directly.
    } else if nvim_get_option_immutable(opt_idx) != 0 && rs_optval_equal(old_value, new_value) == 0
    {
        // Disallow changing immutable options.
        errmsg = nvim_get_e_unsupportedoption();
    } else if (nvim_get_secure() != 0 || nvim_get_sandbox() != 0)
        && (opt_flags_val & K_OPT_FLAG_SECURE) != 0
    {
        // Disallow changing some options from secure mode.
        errmsg = nvim_get_e_secure();
    } else {
        // Check for illegal path names in string options.
        use crate::OptValType;
        if new_value.type_ == OptValType::String
            && nvim_check_illegal_path_names(varp, opt_flags_val) != 0
        {
            errmsg = nvim_get_e_invarg();
        } else if nvim_option_has_did_set_cb(opt_idx) != 0 {
            // Invoke the option-specific callback function.
            errmsg = nvim_invoke_did_set_cb(
                opt_idx,
                varp,
                old_value,
                new_value,
                opt_flags,
                errbuf,
                errbuflen,
                &raw mut value_changed,
                &raw mut value_checked,
                &raw mut restore_chartab,
            );
        }
    }

    // If an error was detected, restore the old value.
    if !errmsg.is_null() {
        rs_set_option_varp(opt_idx, varp, old_value, 1);
        if restore_chartab != 0 {
            nvim_call_buf_init_chartab();
        }
        return errmsg;
    }

    // Re-read the value (callback may have modified it).
    let new_value = rs_optval_from_varp(opt_idx, varp);

    // Set the script context.
    if set_sid != SID_NONE {
        nvim_set_option_sctx_from_sid(opt_idx, opt_flags, set_sid);
    }

    rs_optval_free(old_value);

    let scope_both = (opt_flags & (OPT_LOCAL | OPT_GLOBAL)) == 0;

    if scope_both {
        if rs_option_is_global_local(opt_idx) != 0 {
            // Global option with local value: free local and reset to unset.
            let varp_local = nvim_get_varp_scope_opt(opt_idx, OPT_LOCAL);
            let unset_value = rs_get_option_unset_value(opt_idx);
            rs_set_option_varp(opt_idx, varp_local, rs_optval_copy(unset_value), 1);
        } else {
            // Local option: also set global value.
            let varp_global = nvim_get_varp_scope_opt(opt_idx, OPT_GLOBAL);
            rs_set_option_varp(opt_idx, varp_global, rs_optval_copy(new_value), 1);
        }
    }

    // Don't do anything else if setting directly.
    if direct != 0 {
        return errmsg;
    }

    let curbuf = nvim_opt_get_curbuf();
    let curwin = nvim_opt_get_curwin();

    // Trigger autocmds for special options.
    let syn_addr = nvim_curbuf_b_p_syn_addr();
    let ft_addr = nvim_curbuf_b_p_ft_addr();
    let spl_addr = nvim_curwin_b_p_spl_addr();

    if varp == syn_addr {
        rs_do_syntax_autocmd(curbuf, value_changed);
    } else if varp == ft_addr {
        if (opt_flags & OPT_MODELINE) == 0 || value_changed != 0 {
            nvim_do_filetype_autocmd(value_changed);
        }
    } else if varp == spl_addr {
        rs_do_spelllang_source(curwin);
    }

    // Redraw comp_col in case ruler/showcmd/columns/ls changed.
    nvim_call_comp_col();

    let mouse_addr = nvim_get_p_mouse_addr();
    let flp_addr = nvim_get_p_flp_addr();
    let buf_flp_addr = nvim_curbuf_b_p_flp_addr();
    let wbr_addr = nvim_get_p_wbr_addr();
    let win_wbr_addr = nvim_curwin_p_wbr_addr();

    if varp == mouse_addr {
        nvim_call_setmouse();
    } else if (varp == flp_addr || varp == buf_flp_addr) && nvim_curwin_get_w_briopt_list() != 0 {
        nvim_call_redraw_all_later(UPD_NOT_VALID);
    } else if varp == wbr_addr || varp == win_wbr_addr {
        nvim_call_set_winbar();
    }

    let maxcol = nvim_get_maxcol();

    if nvim_curwin_get_w_curswant() != maxcol
        && (opt_flags_val & (K_OPT_FLAG_CURSWANT | K_OPT_FLAG_REDR_ALL)) != 0
        && (opt_flags_val & K_OPT_FLAG_HL_ONLY) == 0
    {
        nvim_curwin_set_w_set_curswant(1);
    }

    rs_check_redraw_for(curbuf, curwin, opt_flags_val);

    if errmsg.is_null() {
        nvim_option_set_was_set_flag(opt_idx);

        let flagsp = rs_insecure_flag(curwin, opt_idx, opt_flags);
        let flagsp_local = if scope_both {
            rs_insecure_flag(curwin, opt_idx, OPT_LOCAL)
        } else {
            std::ptr::null_mut()
        };

        if value_checked == 0
            && (nvim_get_secure() != 0
                || nvim_get_sandbox() != 0
                || (opt_flags & OPT_MODELINE) != 0)
        {
            *flagsp |= K_OPT_FLAG_INSECURE;
            if !flagsp_local.is_null() {
                *flagsp_local |= K_OPT_FLAG_INSECURE;
            }
        } else if value_replaced != 0 {
            *flagsp &= !K_OPT_FLAG_INSECURE;
            if !flagsp_local.is_null() {
                *flagsp_local &= !K_OPT_FLAG_INSECURE;
            }
        }
    }

    errmsg
}

// =============================================================================
// rs_set_option_impl
// =============================================================================

/// Rust implementation of set_option.
///
/// Validates the new value, sets it, triggers side effects via did_set_option,
/// fires the OptionSet autocmd, and notifies the UI.
///
/// # Safety
/// All pointer arguments must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_set_option_impl(
    opt_idx: c_int,
    value: OptVal,
    opt_flags: c_int,
    set_sid: c_int,
    direct: c_int,
    value_replaced: c_int,
    errbuf: *mut c_char,
    errbuflen: usize,
) -> *const c_char {
    let mut value = value;

    if direct == 0 {
        let validate_errmsg =
            rs_validate_option_value(opt_idx, &raw mut value, opt_flags, errbuf, errbuflen);
        if !validate_errmsg.is_null() {
            rs_optval_free(value);
            return validate_errmsg;
        }
    }

    let scope_local = (opt_flags & OPT_LOCAL) != 0;
    let scope_global = (opt_flags & OPT_GLOBAL) != 0;
    let scope_both = !scope_local && !scope_global;
    let is_opt_local_unset = rs_is_option_local_value_unset(opt_idx) != 0;

    // When using ":set opt=val" for a global option with a local value, use opt->var.
    let varp = if scope_both && rs_option_is_global_local(opt_idx) != 0 {
        nvim_option_get_var_ptr(opt_idx)
    } else {
        nvim_get_varp_scope_opt(opt_idx, opt_flags)
    };
    let varp_local = nvim_get_varp_scope_opt(opt_idx, OPT_LOCAL);
    let varp_global = nvim_get_varp_scope_opt(opt_idx, OPT_GLOBAL);

    let old_value = rs_optval_from_varp(opt_idx, varp);
    let old_global_value = rs_optval_from_varp(opt_idx, varp_global);
    // If local value of global-local option is unset, use global value as local value.
    let old_local_value = if is_opt_local_unset {
        old_global_value
    } else {
        rs_optval_from_varp(opt_idx, varp_local)
    };
    // Value actually being used (for OptionSet autocmd).
    let used_old_value = if scope_local && is_opt_local_unset {
        rs_optval_from_varp(opt_idx, nvim_get_varp_opt(opt_idx))
    } else {
        old_value
    };

    // Save copies in case they get changed by autocommands.
    let saved_used_value = rs_optval_copy(used_old_value);
    let saved_old_global_value = rs_optval_copy(old_global_value);
    let saved_old_local_value = rs_optval_copy(old_local_value);
    let saved_new_value = rs_optval_copy(value);

    let curwin = nvim_opt_get_curwin();
    let p = rs_insecure_flag(curwin, opt_idx, opt_flags);
    let secure_saved = nvim_get_secure();

    // Enable secure mode if needed.
    if (opt_flags & OPT_MODELINE) != 0
        || nvim_get_sandbox() != 0
        || (value_replaced == 0 && (*p & K_OPT_FLAG_INSECURE) != 0)
    {
        nvim_set_secure(1);
    }

    // Set option through its variable pointer.
    rs_set_option_varp(opt_idx, varp, value, 0);
    // Process side effects.
    let errmsg = rs_did_set_option(
        opt_idx,
        varp,
        old_value,
        value,
        opt_flags,
        set_sid,
        direct,
        value_replaced,
        errbuf,
        errbuflen,
    );

    nvim_set_secure(secure_saved);

    if errmsg.is_null() && direct == 0 {
        if nvim_get_starting() == 0 {
            nvim_apply_optionset_autocmd(
                opt_idx,
                opt_flags,
                saved_used_value,
                saved_old_global_value,
                saved_old_local_value,
                saved_new_value,
                errmsg,
            );
        }
        if (nvim_option_get_flags_val(opt_idx) & K_OPT_FLAG_UI_OPTION) != 0 {
            nvim_ui_call_option_set(opt_idx, saved_new_value);
        }
    }

    // Free saved values.
    rs_optval_free(saved_used_value);
    rs_optval_free(saved_old_local_value);
    rs_optval_free(saved_old_global_value);
    rs_optval_free(saved_new_value);

    errmsg
}
