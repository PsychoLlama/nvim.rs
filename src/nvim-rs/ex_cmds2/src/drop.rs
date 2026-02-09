//! ex_drop implementation
//!
//! Port of `ex_drop`.

use std::ffi::{c_char, c_int, c_void};

use crate::autowrite_impl::BufHandle;
use crate::check_changed_any::{TabHandle, WinHandle};

const ML_EMPTY: c_int = 0x01;
const DOCMD_VERBOSE: c_int = 0x01;

// ExArgHandle from script_host
type ExArgHandle = crate::script_host::ExArgHandle;

extern "C" {
    // --- already declared ---
    fn nvim_ex2_eap_get_arg(eap: *mut ExArgHandle) -> *mut c_char;
    fn nvim_ex2_get_curbuf() -> *mut BufHandle;
    fn nvim_ex2_bufIsChanged(buf: *mut BufHandle) -> bool;
    fn nvim_ex2_buf_hide(buf: *mut BufHandle) -> bool;
    fn nvim_ex2_buflist_findnr(nr: c_int) -> *mut BufHandle;
    fn nvim_ex2_get_first_tabpage() -> *mut TabHandle;
    fn nvim_ex2_tp_next(tp: *mut TabHandle) -> *mut TabHandle;
    fn nvim_ex2_tp_firstwin(tp: *mut TabHandle) -> *mut WinHandle;
    fn nvim_ex2_win_next(win: *mut WinHandle) -> *mut WinHandle;
    fn nvim_ex2_win_get_buffer(win: *mut WinHandle) -> *mut BufHandle;
    fn nvim_ex2_goto_tabpage_win(tp: *mut TabHandle, wp: *mut WinHandle);

    fn rs_check_changed(buf: *mut BufHandle, flags: c_int) -> bool;

    // --- drop-specific accessors ---
    fn nvim_ex2_set_arglist(arg: *mut c_char);
    fn nvim_ex2_get_argcount() -> c_int;
    fn nvim_ex2_get_arglist_fnum(idx: c_int) -> c_int;
    fn nvim_ex2_ex_all(eap: *mut ExArgHandle);
    fn nvim_ex2_ex_rewind(eap: *mut ExArgHandle);
    fn nvim_ex2_curwin_set_arg_idx(val: c_int);
    fn nvim_ex2_curbuf_get_b_p_ar() -> c_int;
    fn nvim_ex2_curbuf_set_b_p_ar(val: c_int);
    fn nvim_ex2_curbuf_get_ml_flags() -> c_int;
    fn nvim_ex2_eap_get_do_ecmd_cmd(eap: *mut ExArgHandle) -> *mut c_char;
    fn nvim_ex2_set_swapcommand(cmd: *const c_char, zero: c_int) -> bool;
    fn nvim_ex2_do_cmdline(
        cmd: *mut c_char,
        getline: *mut c_void,
        cookie: *mut c_void,
        flags: c_int,
    );
    fn nvim_ex2_clear_swapcommand();
    fn nvim_ex2_buf_check_timestamp_curbuf();
    fn nvim_ex2_get_emsg_off() -> c_int;
    fn nvim_ex2_set_emsg_off(val: c_int);
    fn nvim_ex2_get_cmod_tab() -> c_int;
    fn nvim_ex2_set_cmod_tab(val: c_int);
    fn nvim_ex2_eap_set_cmdidx(eap: *mut ExArgHandle, val: c_int);
    fn nvim_ex2_eap_set_cmd0(eap: *mut ExArgHandle, ch: c_int);
    fn nvim_ex2_get_cmd_sfirst() -> c_int;
    fn nvim_ex2_get_cmd_first() -> c_int;
}

// CCGD_AW=1, CCGD_EXCMD=16
const CCGD_AW: c_int = 1;
const CCGD_EXCMD: c_int = 16;

/// Port of `ex_drop`
#[no_mangle]
pub unsafe extern "C" fn rs_ex_drop(eap: *mut ExArgHandle) {
    let mut split = false;

    // Check if the first argument is already being edited in a window.
    let arg = unsafe { nvim_ex2_eap_get_arg(eap) };
    unsafe { nvim_ex2_set_arglist(arg) };

    // Expanding wildcards may result in an empty argument list.
    if unsafe { nvim_ex2_get_argcount() } == 0 {
        return;
    }

    if unsafe { nvim_ex2_get_cmod_tab() } != 0 {
        // ":tab drop file ...": open a tab for each argument that isn't
        // edited in a window yet.
        unsafe { nvim_ex2_ex_all(eap) };
        unsafe { nvim_ex2_set_cmod_tab(0) };
        unsafe { nvim_ex2_ex_rewind(eap) };
        return;
    }

    // ":drop file ...": Edit the first argument. Jump to an existing
    // window if possible.
    let fnum = unsafe { nvim_ex2_get_arglist_fnum(0) };
    let buf = unsafe { nvim_ex2_buflist_findnr(fnum) };

    // Search all tab/window combinations for a window with this buffer
    let mut tp = unsafe { nvim_ex2_get_first_tabpage() };
    while !tp.is_null() {
        let mut wp = unsafe { nvim_ex2_tp_firstwin(tp) };
        while !wp.is_null() {
            if unsafe { nvim_ex2_win_get_buffer(wp) } == buf {
                unsafe { nvim_ex2_goto_tabpage_win(tp, wp) };
                unsafe { nvim_ex2_curwin_set_arg_idx(0) };
                if !unsafe { nvim_ex2_bufIsChanged(nvim_ex2_get_curbuf()) } {
                    let save_ar = unsafe { nvim_ex2_curbuf_get_b_p_ar() };
                    // reload the file if it is newer
                    unsafe { nvim_ex2_curbuf_set_b_p_ar(1) };
                    unsafe { nvim_ex2_buf_check_timestamp_curbuf() };
                    unsafe { nvim_ex2_curbuf_set_b_p_ar(save_ar) };
                }
                if (unsafe { nvim_ex2_curbuf_get_ml_flags() } & ML_EMPTY) != 0 {
                    unsafe { nvim_ex2_ex_rewind(eap) };
                }

                // execute [+cmd]
                let do_ecmd_cmd = unsafe { nvim_ex2_eap_get_do_ecmd_cmd(eap) };
                if !do_ecmd_cmd.is_null() {
                    let did_set = unsafe { nvim_ex2_set_swapcommand(do_ecmd_cmd, 0) };
                    unsafe {
                        nvim_ex2_do_cmdline(
                            do_ecmd_cmd,
                            std::ptr::null_mut(),
                            std::ptr::null_mut(),
                            DOCMD_VERBOSE,
                        );
                    }
                    if did_set {
                        unsafe { nvim_ex2_clear_swapcommand() };
                    }
                }

                return;
            }
            wp = unsafe { nvim_ex2_win_next(wp) };
        }
        tp = unsafe { nvim_ex2_tp_next(tp) };
    }

    // Check whether the current buffer is changed.
    if !unsafe { nvim_ex2_buf_hide(nvim_ex2_get_curbuf()) } {
        let emsg_off = unsafe { nvim_ex2_get_emsg_off() };
        unsafe { nvim_ex2_set_emsg_off(emsg_off + 1) };
        split = unsafe { rs_check_changed(nvim_ex2_get_curbuf(), CCGD_AW | CCGD_EXCMD) };
        unsafe { nvim_ex2_set_emsg_off(emsg_off) };
    }

    // Fake a ":sfirst" or ":first" command
    if split {
        unsafe { nvim_ex2_eap_set_cmdidx(eap, nvim_ex2_get_cmd_sfirst()) };
        unsafe { nvim_ex2_eap_set_cmd0(eap, b's' as c_int) };
    } else {
        unsafe { nvim_ex2_eap_set_cmdidx(eap, nvim_ex2_get_cmd_first()) };
    }
    unsafe { nvim_ex2_ex_rewind(eap) };
}
