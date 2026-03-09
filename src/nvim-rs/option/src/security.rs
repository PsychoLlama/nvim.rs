//! Option security: insecure flag and redraw trigger functions (Phase 4, pass 3)
//!
//! Rust implementations of check_redraw_for, insecure_flag, was_set_insecurely,
//! and set_option_sctx from option_shim.c.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::branches_sharing_code)]
#![allow(clippy::borrow_as_ptr)]

use std::ffi::{c_int, c_uint};

use crate::opt_index::{
    K_OPT_FOLDEXPR, K_OPT_FOLDTEXT, K_OPT_FORMATEXPR, K_OPT_INCLUDEEXPR, K_OPT_INDENTEXPR,
    K_OPT_STATUSLINE, K_OPT_WINBAR, K_OPT_WRAP,
};
use crate::{BufHandle, OptFlags, WinHandle};

// =============================================================================
// C Function Declarations
// =============================================================================

/// Script context structure (matches C's sctx_T)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ScriptContext {
    pub sc_sid: c_int,
    pub sc_seq: c_int,
    pub sc_lnum: i64,
}

// OptIndex type alias (matches C's OptIndex = int)
type OptIndex = c_int;

extern "C" {
    static mut redraw_tabline: bool;
}

extern "C" {
    // check_redraw_for
    fn nvim_call_status_redraw_all();
    fn nvim_call_changed_window_setting(win: WinHandle);
    fn nvim_call_redraw_later(win: WinHandle, redraw_type: c_int);
    fn nvim_call_redraw_buf_later(buf: BufHandle, redraw_type: c_int);
    fn nvim_call_redraw_all_later(redraw_type: c_int);

    // insecure_flag pointer accessors
    fn nvim_win_get_p_wrap_flags_ptr(wp: WinHandle) -> *mut c_uint;
    fn nvim_win_get_p_stl_flags_ptr(wp: WinHandle) -> *mut c_uint;
    fn nvim_win_get_p_wbr_flags_ptr(wp: WinHandle) -> *mut c_uint;
    fn nvim_win_get_p_fde_flags_ptr(wp: WinHandle) -> *mut c_uint;
    fn nvim_win_get_p_fdt_flags_ptr(wp: WinHandle) -> *mut c_uint;
    fn nvim_win_get_buf_p_inde_flags_ptr(wp: WinHandle) -> *mut c_uint;
    fn nvim_win_get_buf_p_fex_flags_ptr(wp: WinHandle) -> *mut c_uint;
    fn nvim_win_get_buf_p_inex_flags_ptr(wp: WinHandle) -> *mut c_uint;
    fn nvim_win_allbuf_p_wrap_flags_ptr(wp: WinHandle) -> *mut c_uint;
    fn nvim_win_allbuf_p_fde_flags_ptr(wp: WinHandle) -> *mut c_uint;
    fn nvim_win_allbuf_p_fdt_flags_ptr(wp: WinHandle) -> *mut c_uint;
    fn nvim_option_get_flags_ptr(opt_idx: OptIndex) -> *mut c_uint;

    // set_option_sctx
    fn nvim_option_get_sourcing_lnum() -> i64;
    fn nvim_call_nlua_set_sctx(sctx: *mut ScriptContext);
    fn nvim_curbuf_set_p_script_ctx(idx: c_int, sctx: ScriptContext);
    fn nvim_curwin_set_p_script_ctx(idx: c_int, sctx: ScriptContext);
    fn nvim_curwin_set_allbuf_opt_script_ctx(idx: c_int, sctx: ScriptContext);
    fn nvim_option_set_script_ctx(opt_idx: OptIndex, sctx: ScriptContext);
    fn option_has_scope(opt_idx: OptIndex, scope: c_int) -> c_int;
    fn option_scope_idx(opt_idx: OptIndex, scope: c_int) -> c_int;
    fn nvim_option_is_global_only(opt_idx: OptIndex) -> c_int;
}

// OPT_* flags matching option.h
const OPT_GLOBAL: c_int = 0x01;
const OPT_LOCAL: c_int = 0x02;
const OPT_MODELINE: c_int = 0x04;

// OptScope values matching option_defs.h
const K_OPT_SCOPE_BUF: c_int = 2;
const K_OPT_SCOPE_WIN: c_int = 1;

// UPD_NOT_VALID constant
const UPD_NOT_VALID: c_int = 40;

// =============================================================================
// check_redraw_for
// =============================================================================

/// Check option flags and trigger appropriate redraws.
///
/// Corresponds to `check_redraw_for()` in option_shim.c.
#[no_mangle]
pub unsafe extern "C" fn rs_check_redraw_for(buf: BufHandle, win: WinHandle, flags: c_uint) {
    // kOptFlagRedrAll = kOptFlagRedrBuf | kOptFlagRedrWin
    let redr_all = OptFlags::REDR_ALL.0;
    let all = (flags & redr_all) == redr_all;

    if (flags & OptFlags::REDR_STAT.0) != 0 || all {
        nvim_call_status_redraw_all();
    }

    if (flags & OptFlags::REDR_TABL.0) != 0 || all {
        redraw_tabline = true;
    }

    if (flags & OptFlags::REDR_BUF.0) != 0 || (flags & OptFlags::REDR_WIN.0) != 0 || all {
        if (flags & OptFlags::HL_ONLY.0) != 0 {
            nvim_call_redraw_later(win, UPD_NOT_VALID);
        } else {
            nvim_call_changed_window_setting(win);
        }
    }
    if (flags & OptFlags::REDR_BUF.0) != 0 {
        nvim_call_redraw_buf_later(buf, UPD_NOT_VALID);
    }
    if all {
        nvim_call_redraw_all_later(UPD_NOT_VALID);
    }
}

// =============================================================================
// insecure_flag
// =============================================================================

/// Get a pointer to the flags used for the kOptFlagInsecure flag of option `opt_idx`.
///
/// For some local options a local flags field is used.
/// NOTE: Caller must make sure that `wp` is set to the window from which the option is used.
///
/// Corresponds to `insecure_flag()` in option_shim.c.
#[no_mangle]
pub unsafe extern "C" fn rs_insecure_flag(
    wp: WinHandle,
    opt_idx: OptIndex,
    opt_flags: c_int,
) -> *mut c_uint {
    if (opt_flags & OPT_LOCAL) != 0 {
        // wp must not be null
        if opt_idx == K_OPT_WRAP {
            return nvim_win_get_p_wrap_flags_ptr(wp);
        } else if opt_idx == K_OPT_STATUSLINE {
            return nvim_win_get_p_stl_flags_ptr(wp);
        } else if opt_idx == K_OPT_WINBAR {
            return nvim_win_get_p_wbr_flags_ptr(wp);
        } else if opt_idx == K_OPT_FOLDEXPR {
            return nvim_win_get_p_fde_flags_ptr(wp);
        } else if opt_idx == K_OPT_FOLDTEXT {
            return nvim_win_get_p_fdt_flags_ptr(wp);
        } else if opt_idx == K_OPT_INDENTEXPR {
            return nvim_win_get_buf_p_inde_flags_ptr(wp);
        } else if opt_idx == K_OPT_FORMATEXPR {
            return nvim_win_get_buf_p_fex_flags_ptr(wp);
        } else if opt_idx == K_OPT_INCLUDEEXPR {
            return nvim_win_get_buf_p_inex_flags_ptr(wp);
        }
    } else {
        // For global value of window-local options, use flags in w_allbuf_opt.
        if opt_idx == K_OPT_WRAP {
            return nvim_win_allbuf_p_wrap_flags_ptr(wp);
        } else if opt_idx == K_OPT_FOLDEXPR {
            return nvim_win_allbuf_p_fde_flags_ptr(wp);
        } else if opt_idx == K_OPT_FOLDTEXT {
            return nvim_win_allbuf_p_fdt_flags_ptr(wp);
        }
    }
    // Nothing special, return global flags field.
    nvim_option_get_flags_ptr(opt_idx)
}

// =============================================================================
// was_set_insecurely
// =============================================================================

/// Check if option `opt_idx` was set insecurely (from modeline).
#[no_mangle]
pub unsafe extern "C" fn rs_was_set_insecurely(
    wp: WinHandle,
    opt_idx: OptIndex,
    opt_flags: c_int,
) -> c_int {
    let flagp = rs_insecure_flag(wp, opt_idx, opt_flags);
    c_int::from((*flagp & OptFlags::INSECURE.0) != 0)
}

// =============================================================================
// set_option_sctx
// =============================================================================

/// Set the script_ctx for an option, taking care of setting the buffer- or
/// window-local value.
///
/// Corresponds to `set_option_sctx()` in option_shim.c.
#[no_mangle]
pub unsafe extern "C" fn rs_set_option_sctx(
    opt_idx: OptIndex,
    opt_flags: c_int,
    mut script_ctx: ScriptContext,
) {
    let both = (opt_flags & (OPT_LOCAL | OPT_GLOBAL)) == 0;

    // Modeline already has the line number set.
    if (opt_flags & OPT_MODELINE) == 0 {
        script_ctx.sc_lnum += nvim_option_get_sourcing_lnum();
    }
    nvim_call_nlua_set_sctx(&mut script_ctx);

    // Remember where the option was set. For local options need to do that
    // in the buffer or window structure.
    if both || (opt_flags & OPT_GLOBAL) != 0 || nvim_option_is_global_only(opt_idx) != 0 {
        nvim_option_set_script_ctx(opt_idx, script_ctx);
    }
    if both || (opt_flags & OPT_LOCAL) != 0 {
        if option_has_scope(opt_idx, K_OPT_SCOPE_BUF) != 0 {
            let idx = option_scope_idx(opt_idx, K_OPT_SCOPE_BUF);
            nvim_curbuf_set_p_script_ctx(idx, script_ctx);
        } else if option_has_scope(opt_idx, K_OPT_SCOPE_WIN) != 0 {
            let idx = option_scope_idx(opt_idx, K_OPT_SCOPE_WIN);
            nvim_curwin_set_p_script_ctx(idx, script_ctx);
            if both {
                // also setting the "all buffers" value
                nvim_curwin_set_allbuf_opt_script_ctx(idx, script_ctx);
            }
        }
    }
}
