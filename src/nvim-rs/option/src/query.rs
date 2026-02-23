//! Option query utility functions (Phase 4, pass 1)
//!
//! Rust implementations of small utility/query functions from option_shim.c.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int, c_uint};

use crate::{BufHandle, WinHandle};

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    fn vim_strchr(s: *const c_char, c: c_int) -> *const c_char;

    // can_bs
    fn nvim_option_get_p_bs() -> *const c_char;
    fn nvim_curbuf_is_prompt() -> c_int;

    // get_equalprg
    fn nvim_curbuf_get_b_p_ep() -> *const c_char;
    fn nvim_option_get_p_ep() -> *const c_char;

    // get_findfunc
    fn nvim_curbuf_get_b_p_ffu() -> *const c_char;
    fn nvim_option_get_p_ffu() -> *const c_char;

    // get_bkc_flags
    fn nvim_get_bkc_flags() -> c_uint;
    fn nvim_buf_get_bkc_flags(buf: BufHandle) -> c_uint;

    // get_flp_value
    fn nvim_option_get_p_flp() -> *const c_char;
    fn nvim_buf_get_p_flp(buf: BufHandle) -> *const c_char;

    // get_ve_flags
    fn nvim_get_ve_flags_global() -> c_uint;
    fn nvim_win_get_ve_flags(wp: WinHandle) -> c_uint;

    // redraw_titles
    fn nvim_callback_set_need_maketitle(value: c_int);
    fn nvim_set_redraw_tabline(val: c_int);

    // vimrc_found
    fn nvim_option_vim_getenv(envname: *const c_char) -> *mut c_char;
    fn nvim_option_FullName_save(fname: *const c_char, force: bool) -> *mut c_char;
    fn nvim_option_os_setenv(name: *const c_char, value: *const c_char, overwrite: c_int) -> c_int;
    fn xfree(ptr: *mut c_char);

    // set_iminsert_global / set_imsearch_global
    fn nvim_set_p_iminsert(v: i64);
    fn nvim_set_p_imsearch(v: i64);
    fn nvim_buf_get_b_p_iminsert(buf: BufHandle) -> i64;
    fn nvim_buf_get_b_p_imsearch(buf: BufHandle) -> i64;

    // reset_modifiable
    fn nvim_option_set_p_ma(v: c_int);
    fn nvim_curbuf_set_b_p_ma(v: c_int);
    fn nvim_change_option_default_bool(opt_idx: c_int, value: c_int);
    fn nvim_get_opt_idx_modifiable() -> c_int;
}

// BS constants matching option_vars.h (BS_INDENT='i', BS_EOL='l' are valid but not compared directly)
const BS_START: c_int = b's' as c_int;
const BS_NOSTOP: c_int = b'p' as c_int;

/// Check if backspacing over something is allowed.
///
/// `what` is one of BS_INDENT, BS_EOL, BS_START, or BS_NOSTOP.
#[no_mangle]
pub unsafe extern "C" fn rs_can_bs(what: c_int) -> c_int {
    // BS_START is disallowed in prompt buffers
    if what == BS_START && nvim_curbuf_is_prompt() != 0 {
        return 0;
    }
    let p_bs = nvim_option_get_p_bs();
    if p_bs.is_null() {
        return 0;
    }
    // Legacy: '2' means allow backspace over everything except nostop
    if *p_bs as u8 == b'2' {
        return c_int::from(what != BS_NOSTOP);
    }
    c_int::from(!vim_strchr(p_bs, what).is_null())
}

/// Get the value of 'equalprg', either the buffer-local one or the global one.
#[no_mangle]
pub unsafe extern "C" fn rs_get_equalprg() -> *const c_char {
    let b_p_ep = nvim_curbuf_get_b_p_ep();
    if !b_p_ep.is_null() && *b_p_ep != 0 {
        b_p_ep
    } else {
        nvim_option_get_p_ep()
    }
}

/// Get the value of 'findfunc', either the buffer-local one or the global one.
#[no_mangle]
pub unsafe extern "C" fn rs_get_findfunc() -> *const c_char {
    let b_p_ffu = nvim_curbuf_get_b_p_ffu();
    if !b_p_ffu.is_null() && *b_p_ffu != 0 {
        b_p_ffu
    } else {
        nvim_option_get_p_ffu()
    }
}

/// Get the local or global value of 'backupcopy' flags.
#[no_mangle]
pub unsafe extern "C" fn rs_get_bkc_flags(buf: BufHandle) -> c_uint {
    let local = nvim_buf_get_bkc_flags(buf);
    if local != 0 {
        local
    } else {
        nvim_get_bkc_flags()
    }
}

/// Get the local or global value of 'formatlistpat'.
#[no_mangle]
pub unsafe extern "C" fn rs_get_flp_value(buf: BufHandle) -> *const c_char {
    let b_p_flp = nvim_buf_get_p_flp(buf);
    if !b_p_flp.is_null() && *b_p_flp != 0 {
        b_p_flp
    } else {
        nvim_option_get_p_flp()
    }
}

// kOptVeFlag constants (from auto/option_vars.generated.h)
const K_OPT_VE_FLAG_NONE: c_uint = 0x10;
const K_OPT_VE_FLAG_NONE_U: c_uint = 0x20;

/// Get the local or global value of 'virtualedit' flags.
#[no_mangle]
pub unsafe extern "C" fn rs_get_ve_flags(wp: WinHandle) -> c_uint {
    let w_ve_flags = nvim_win_get_ve_flags(wp);
    let flags = if w_ve_flags != 0 {
        w_ve_flags
    } else {
        nvim_get_ve_flags_global()
    };
    flags & !(K_OPT_VE_FLAG_NONE | K_OPT_VE_FLAG_NONE_U)
}

/// Redraw the window title and/or tab page text later.
#[no_mangle]
pub unsafe extern "C" fn rs_redraw_titles() {
    nvim_callback_set_need_maketitle(1);
    nvim_set_redraw_tabline(1);
}

/// Handle vimrc file discovery: set $envname if not already set.
///
/// Called when a vimrc or "VIMINIT" has been found.
#[no_mangle]
pub unsafe extern "C" fn rs_vimrc_found(fname: *const c_char, envname: *const c_char) {
    if fname.is_null() || envname.is_null() {
        return;
    }
    let p = nvim_option_vim_getenv(envname);
    if p.is_null() {
        // Set $envname to the full path of the first vimrc file found.
        let full = nvim_option_FullName_save(fname, false);
        if !full.is_null() {
            nvim_option_os_setenv(envname, full, 1);
            xfree(full);
        }
    } else {
        xfree(p);
    }
}

/// Set the global value for 'iminsert' to the local value.
#[no_mangle]
pub unsafe extern "C" fn rs_set_iminsert_global(buf: BufHandle) {
    let val = nvim_buf_get_b_p_iminsert(buf);
    nvim_set_p_iminsert(val);
}

/// Set the global value for 'imsearch' to the local value.
#[no_mangle]
pub unsafe extern "C" fn rs_set_imsearch_global(buf: BufHandle) {
    let val = nvim_buf_get_b_p_imsearch(buf);
    nvim_set_p_imsearch(val);
}

/// Reset the 'modifiable' option and its default value.
#[no_mangle]
pub unsafe extern "C" fn rs_reset_modifiable() {
    nvim_curbuf_set_b_p_ma(0);
    nvim_option_set_p_ma(0);
    let opt_idx = nvim_get_opt_idx_modifiable();
    nvim_change_option_default_bool(opt_idx, 0);
}
