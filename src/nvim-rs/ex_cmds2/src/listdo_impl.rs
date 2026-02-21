//! ex_listdo implementation
//!
//! Port of `ex_listdo` — the `:argdo`, `:windo`, `:bufdo`, `:tabdo`,
//! `:cdo`, `:ldo`, `:cfdo`, `:lfdo` handler.

use std::ffi::{c_char, c_int, c_void};

use crate::autowrite_impl::BufHandle;
use crate::check_changed_any::{TabHandle, WinHandle};

type ExArgHandle = crate::script_host::ExArgHandle;

extern "C" {
    // --- already declared ---
    fn nvim_ex2_eap_get_arg(eap: *mut ExArgHandle) -> *mut c_char;
    fn nvim_ex2_eap_get_line1(eap: *mut ExArgHandle) -> i32;
    fn nvim_ex2_eap_get_line2(eap: *mut ExArgHandle) -> i32;
    fn nvim_ex2_eap_get_forceit(eap: *mut ExArgHandle) -> c_int;
    fn nvim_ex2_get_firstbuf() -> *mut BufHandle;
    fn nvim_ex2_buf_next(buf: *mut BufHandle) -> *mut BufHandle;
    fn nvim_ex2_get_curbuf() -> *mut BufHandle;
    fn nvim_ex2_buf_get_fnum(buf: *mut BufHandle) -> c_int;
    fn nvim_ex2_buf_hide(buf: *mut BufHandle) -> bool;
    fn nvim_ex2_get_firstwin() -> *mut WinHandle;
    fn nvim_ex2_win_next(win: *mut WinHandle) -> *mut WinHandle;
    fn nvim_ex2_get_first_tabpage() -> *mut TabHandle;
    fn nvim_ex2_tp_next(tp: *mut TabHandle) -> *mut TabHandle;
    fn nvim_ex2_get_curwin() -> *mut WinHandle;

    fn rs_check_changed(buf: *mut BufHandle, flags: c_int) -> bool;

    // --- listdo-specific accessors ---
    fn nvim_ex2_eap_get_cmdidx(eap: *mut ExArgHandle) -> c_int;
    fn nvim_ex2_eap_get_ea_getline(eap: *mut ExArgHandle) -> *mut c_void;
    fn nvim_ex2_eap_get_cookie(eap: *mut ExArgHandle) -> *mut c_void;
    fn nvim_ex2_eap_get_addr_count(eap: *mut ExArgHandle) -> c_int;
    fn nvim_ex2_eap_set_line2(eap: *mut ExArgHandle, val: i32);

    // CMD_* enum value accessors
    fn nvim_ex2_get_cmd_windo() -> c_int;
    fn nvim_ex2_get_cmd_tabdo() -> c_int;
    fn nvim_ex2_get_cmd_bufdo() -> c_int;
    fn nvim_ex2_get_cmd_argdo() -> c_int;
    fn nvim_ex2_get_cmd_cdo() -> c_int;
    fn nvim_ex2_get_cmd_ldo() -> c_int;
    fn nvim_ex2_get_cmd_cfdo() -> c_int;
    fn nvim_ex2_get_cmd_lfdo() -> c_int;

    // window/buffer globals and operations
    fn nvim_ex2_get_curwin_w_p_wfb() -> bool;
    fn nvim_ex2_get_prevwin() -> *mut WinHandle;
    fn nvim_ex2_win_valid(win: *mut WinHandle) -> bool;
    fn nvim_ex2_win_get_w_p_wfb(win: *mut WinHandle) -> bool;
    fn nvim_ex2_win_goto(win: *mut WinHandle);
    fn nvim_ex2_win_split() -> c_int;
    fn nvim_ex2_emsg_winfixbuf();

    // msg_listdo_overwrite
    fn nvim_ex2_inc_msg_listdo_overwrite();
    fn nvim_ex2_dec_msg_listdo_overwrite();

    // autocmd operations
    fn nvim_ex2_au_event_disable_syntax() -> *mut c_char;

    // buffer flags
    fn nvim_ex2_buf_clear_bf_syn_set(buf: *mut BufHandle);
    fn nvim_ex2_buf_get_b_p_bl(buf: *mut BufHandle) -> bool;

    // quickfix
    fn nvim_ex2_qf_get_valid_size(eap: *mut ExArgHandle) -> usize;
    fn nvim_ex2_qf_get_cur_idx(eap: *mut ExArgHandle) -> usize;
    fn rs_ex_cc(eap: *mut ExArgHandle);
    fn rs_ex_cnext(eap: *mut ExArgHandle);

    // listdo operations
    fn nvim_ex2_get_got_int() -> bool;
    fn nvim_ex2_set_listcmd_busy(val: bool);
    fn nvim_ex2_setpcmark();
    fn nvim_ex2_get_argcount() -> c_int;
    fn nvim_ex2_curwin_get_arg_idx() -> c_int;
    fn nvim_ex2_editing_arg_idx() -> bool;
    fn nvim_ex2_do_argfile(eap: *mut ExArgHandle, idx: c_int);
    fn nvim_ex2_goto_buffer(eap: *mut ExArgHandle, start: c_int, dir: c_int, fnum: c_int);
    fn nvim_ex2_win_get_w_floating(win: *mut WinHandle) -> bool;
    fn nvim_ex2_win_get_w_config_hide(win: *mut WinHandle) -> bool;
    fn nvim_ex2_win_get_w_config_focusable(win: *mut WinHandle) -> bool;
    fn nvim_ex2_valid_tabpage(tp: *mut TabHandle) -> bool;
    fn nvim_ex2_goto_tabpage_tp(tp: *mut TabHandle, trigger_enter: bool, trigger_leave: bool);
    fn nvim_ex2_do_cmdline(
        cmd: *mut c_char,
        getline: *mut c_void,
        cookie: *mut c_void,
        flags: c_int,
    );
    fn nvim_ex2_validate_cursor();
    fn nvim_ex2_curwin_get_w_p_scb() -> bool;
    fn nvim_ex2_do_check_scrollbind(check: bool);

    // syntax restore
    fn nvim_ex2_listdo_restore_syntax(save_ei: *mut c_char);
}

const CCGD_AW: c_int = 1;
const CCGD_FORCEIT: c_int = 4;
const CCGD_EXCMD: c_int = 16;
const FORWARD: c_int = 1;
const DOBUF_FIRST: c_int = 1;
const DOCMD_VERBOSE: c_int = 0x01;
const DOCMD_NOWAIT: c_int = 0x02;

/// Port of `ex_listdo`
#[allow(clippy::too_many_lines)]
#[allow(clippy::missing_panics_doc)]
#[no_mangle]
pub unsafe extern "C" fn rs_ex_listdo(eap: *mut ExArgHandle) {
    let cmdidx = unsafe { nvim_ex2_eap_get_cmdidx(eap) };
    let cmd_windo = unsafe { nvim_ex2_get_cmd_windo() };
    let cmd_tabdo = unsafe { nvim_ex2_get_cmd_tabdo() };
    let cmd_bufdo = unsafe { nvim_ex2_get_cmd_bufdo() };
    let cmd_argdo = unsafe { nvim_ex2_get_cmd_argdo() };
    let cmd_cdo = unsafe { nvim_ex2_get_cmd_cdo() };
    let cmd_ldo = unsafe { nvim_ex2_get_cmd_ldo() };
    let cmd_cfdo = unsafe { nvim_ex2_get_cmd_cfdo() };
    let cmd_lfdo = unsafe { nvim_ex2_get_cmd_lfdo() };

    // Handle winfixbuf
    if unsafe { nvim_ex2_get_curwin_w_p_wfb() } {
        if (cmdidx == cmd_ldo || cmdidx == cmd_lfdo)
            && unsafe { nvim_ex2_eap_get_forceit(eap) } == 0
        {
            unsafe { nvim_ex2_emsg_winfixbuf() };
            return;
        }

        let prevwin = unsafe { nvim_ex2_get_prevwin() };
        if unsafe { nvim_ex2_win_valid(prevwin) } && !unsafe { nvim_ex2_win_get_w_p_wfb(prevwin) } {
            unsafe { nvim_ex2_win_goto(prevwin) };
        }
        if unsafe { nvim_ex2_get_curwin_w_p_wfb() } {
            unsafe { nvim_ex2_win_split() };

            if unsafe { nvim_ex2_get_curwin_w_p_wfb() } {
                unsafe { nvim_ex2_emsg_winfixbuf() };
                return;
            }
        }
    }

    let mut save_ei: *mut c_char = std::ptr::null_mut();

    // Temporarily override SHM_OVER and SHM_OVERALL
    unsafe { nvim_ex2_inc_msg_listdo_overwrite() };

    if cmdidx != cmd_windo && cmdidx != cmd_tabdo {
        // Don't do syntax HL autocommands
        save_ei = unsafe { nvim_ex2_au_event_disable_syntax() };

        let mut buf = unsafe { nvim_ex2_get_firstbuf() };
        while !buf.is_null() {
            unsafe { nvim_ex2_buf_clear_bf_syn_set(buf) };
            buf = unsafe { nvim_ex2_buf_next(buf) };
        }
    }

    let forceit = unsafe { nvim_ex2_eap_get_forceit(eap) } != 0;
    if cmdidx == cmd_windo
        || cmdidx == cmd_tabdo
        || unsafe { nvim_ex2_buf_hide(nvim_ex2_get_curbuf()) }
        || !unsafe {
            rs_check_changed(
                nvim_ex2_get_curbuf(),
                CCGD_AW | (if forceit { CCGD_FORCEIT } else { 0 }) | CCGD_EXCMD,
            )
        }
    {
        let mut next_fnum: c_int = 0;
        let mut i: c_int = 0;
        let line1 = unsafe { nvim_ex2_eap_get_line1(eap) };
        let mut line2 = unsafe { nvim_ex2_eap_get_line2(eap) };
        let mut wp = unsafe { nvim_ex2_get_firstwin() };
        let mut tp = unsafe { nvim_ex2_get_first_tabpage() };

        // Position to start
        if cmdidx == cmd_windo {
            while !wp.is_null() && i + 1 < line1 {
                wp = unsafe { nvim_ex2_win_next(wp) };
                i += 1;
            }
        } else if cmdidx == cmd_tabdo {
            while !tp.is_null() && i + 1 < line1 {
                tp = unsafe { nvim_ex2_tp_next(tp) };
                i += 1;
            }
        } else if cmdidx == cmd_argdo {
            i = line1 - 1;
        }

        let mut buf = unsafe { nvim_ex2_get_curbuf() };
        let mut qf_size: usize = 0;

        // set pcmark now
        if cmdidx == cmd_bufdo {
            // Advance to the first listed buffer after "eap->line1".
            buf = unsafe { nvim_ex2_get_firstbuf() };
            while !buf.is_null()
                && (unsafe { nvim_ex2_buf_get_fnum(buf) } < line1
                    || !unsafe { nvim_ex2_buf_get_b_p_bl(buf) })
            {
                if unsafe { nvim_ex2_buf_get_fnum(buf) } > line2 {
                    buf = std::ptr::null_mut();
                    break;
                }
                buf = unsafe { nvim_ex2_buf_next(buf) };
            }
            if !buf.is_null() {
                unsafe {
                    nvim_ex2_goto_buffer(eap, DOBUF_FIRST, FORWARD, nvim_ex2_buf_get_fnum(buf));
                }
            }
        } else if cmdidx == cmd_cdo || cmdidx == cmd_ldo || cmdidx == cmd_cfdo || cmdidx == cmd_lfdo
        {
            qf_size = unsafe { nvim_ex2_qf_get_valid_size(eap) };
            if qf_size == 0 || (line1 as usize) > qf_size {
                buf = std::ptr::null_mut();
            } else {
                unsafe { rs_ex_cc(eap) };
                buf = unsafe { nvim_ex2_get_curbuf() };
                i = line1 - 1;
                if unsafe { nvim_ex2_eap_get_addr_count(eap) } <= 0 {
                    assert!(qf_size < i32::MAX as usize);
                    line2 = qf_size as i32;
                    unsafe { nvim_ex2_eap_set_line2(eap, line2) };
                }
            }
        } else {
            unsafe { nvim_ex2_setpcmark() };
        }
        unsafe { nvim_ex2_set_listcmd_busy(true) };

        while !unsafe { nvim_ex2_get_got_int() } && !buf.is_null() {
            let mut execute = true;

            if cmdidx == cmd_argdo {
                if i == unsafe { nvim_ex2_get_argcount() } {
                    break;
                }
                if unsafe { nvim_ex2_curwin_get_arg_idx() } != i
                    || !unsafe { nvim_ex2_editing_arg_idx() }
                {
                    unsafe { nvim_ex2_do_argfile(eap, i) };
                }
                if unsafe { nvim_ex2_curwin_get_arg_idx() } != i {
                    break;
                }
            } else if cmdidx == cmd_windo {
                if !unsafe { nvim_ex2_win_valid(wp) } {
                    break;
                }
                execute = !unsafe { nvim_ex2_win_get_w_floating(wp) }
                    || (!unsafe { nvim_ex2_win_get_w_config_hide(wp) }
                        && unsafe { nvim_ex2_win_get_w_config_focusable(wp) });
                if execute {
                    unsafe { nvim_ex2_win_goto(wp) };
                    if unsafe { nvim_ex2_get_curwin() } != wp {
                        break;
                    }
                }
                wp = unsafe { nvim_ex2_win_next(wp) };
            } else if cmdidx == cmd_tabdo {
                if !unsafe { nvim_ex2_valid_tabpage(tp) } {
                    break;
                }
                unsafe { nvim_ex2_goto_tabpage_tp(tp, true, true) };
                tp = unsafe { nvim_ex2_tp_next(tp) };
            } else if cmdidx == cmd_bufdo {
                // Remember the number of the next listed buffer
                next_fnum = -1;
                let mut bp = unsafe { nvim_ex2_buf_next(nvim_ex2_get_curbuf()) };
                while !bp.is_null() {
                    if unsafe { nvim_ex2_buf_get_b_p_bl(bp) } {
                        next_fnum = unsafe { nvim_ex2_buf_get_fnum(bp) };
                        break;
                    }
                    bp = unsafe { nvim_ex2_buf_next(bp) };
                }
            }

            i += 1;
            // execute the command
            if execute {
                let arg = unsafe { nvim_ex2_eap_get_arg(eap) };
                let getline = unsafe { nvim_ex2_eap_get_ea_getline(eap) };
                let cookie = unsafe { nvim_ex2_eap_get_cookie(eap) };
                unsafe {
                    nvim_ex2_do_cmdline(arg, getline, cookie, DOCMD_VERBOSE + DOCMD_NOWAIT);
                }
            }

            if cmdidx == cmd_bufdo {
                // Done?
                let line2_now = unsafe { nvim_ex2_eap_get_line2(eap) };
                if next_fnum < 0 || next_fnum > line2_now {
                    break;
                }

                // Check if the buffer still exists.
                let mut buf_still_exists = false;
                let mut bp = unsafe { nvim_ex2_get_firstbuf() };
                while !bp.is_null() {
                    if unsafe { nvim_ex2_buf_get_fnum(bp) } == next_fnum {
                        buf_still_exists = true;
                        break;
                    }
                    bp = unsafe { nvim_ex2_buf_next(bp) };
                }
                if !buf_still_exists {
                    break;
                }

                // Go to the next buffer.
                unsafe { nvim_ex2_goto_buffer(eap, DOBUF_FIRST, FORWARD, next_fnum) };

                // If autocommands took us elsewhere, quit here.
                if unsafe { nvim_ex2_buf_get_fnum(nvim_ex2_get_curbuf()) } != next_fnum {
                    break;
                }
            }

            if cmdidx == cmd_cdo || cmdidx == cmd_ldo || cmdidx == cmd_cfdo || cmdidx == cmd_lfdo {
                let line2_now = unsafe { nvim_ex2_eap_get_line2(eap) };
                if (i as usize) >= qf_size || i >= line2_now {
                    break;
                }

                let qf_idx = unsafe { nvim_ex2_qf_get_cur_idx(eap) };
                unsafe { rs_ex_cnext(eap) };

                if unsafe { nvim_ex2_qf_get_cur_idx(eap) } == qf_idx {
                    break;
                }
            }

            if cmdidx == cmd_windo && execute {
                unsafe { nvim_ex2_validate_cursor() };
                if unsafe { nvim_ex2_curwin_get_w_p_scb() } {
                    unsafe { nvim_ex2_do_check_scrollbind(true) };
                }
            }
            if cmdidx == cmd_windo || cmdidx == cmd_tabdo {
                let line2_now = unsafe { nvim_ex2_eap_get_line2(eap) };
                if i + 1 > line2_now {
                    break;
                }
            }
            if cmdidx == cmd_argdo {
                let line2_now = unsafe { nvim_ex2_eap_get_line2(eap) };
                if i >= line2_now {
                    break;
                }
            }
        }
        unsafe { nvim_ex2_set_listcmd_busy(false) };
    }

    unsafe { nvim_ex2_dec_msg_listdo_overwrite() };
    if !save_ei.is_null() {
        unsafe { nvim_ex2_listdo_restore_syntax(save_ei) };
    }
}
