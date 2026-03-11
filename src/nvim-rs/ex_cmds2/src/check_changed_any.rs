//! check_changed_any and ex_checktime implementations
//!
//! Ports of `check_changed_any`, `add_bufnum`, and `ex_checktime`.

use std::ffi::{c_char, c_int};

use crate::autowrite_impl::{BufHandle, BufrefHandle};
use crate::script_host::ExArgHandle;

const DOBUF_UNLOAD: c_int = 2;
const DOBUF_GOTO: c_int = 0;
const CMOD_CONFIRM: u32 = 0x0080;

// Opaque handle types for windows and tabpages
#[repr(C)]
pub struct WinHandle {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct TabHandle {
    _opaque: [u8; 0],
}

extern "C" {
    // --- already declared in other modules ---
    fn nvim_ex2_buf_get_fname(buf: *mut BufHandle) -> *const c_char;
    fn nvim_ex2_buf_get_nwindows(buf: *mut BufHandle) -> c_int;
    fn nvim_ex2_get_firstbuf() -> *mut BufHandle;
    fn nvim_ex2_buf_next(buf: *mut BufHandle) -> *mut BufHandle;
    fn nvim_ex2_get_curbuf() -> *mut BufHandle;
    fn nvim_ex2_get_cmod_flags() -> u32;
    fn nvim_ex2_bufIsChanged(buf: *mut BufHandle) -> bool;
    fn nvim_ex2_bufref_create(buf: *mut BufHandle) -> *mut BufrefHandle;
    fn nvim_ex2_bufref_valid(br: *mut BufrefHandle) -> bool;
    fn nvim_ex2_bufref_free(br: *mut BufrefHandle);
    fn nvim_ex2_gettext(s: *const c_char) -> *const c_char;
    fn nvim_ex2_buf_get_fnum(buf: *mut BufHandle) -> c_int;
    fn nvim_ex2_get_p_awa() -> bool;
    fn nvim_ex2_get_p_confirm() -> bool;

    // --- check_changed (already in Rust, exported as "check_changed") ---
    #[link_name = "check_changed"]
    fn rs_check_changed(buf: *mut BufHandle, flags: c_int) -> bool;

    // --- new accessors for check_changed_any ---
    fn nvim_ex2_win_next(win: *mut WinHandle) -> *mut WinHandle;
    fn nvim_ex2_win_get_buffer(win: *mut WinHandle) -> *mut BufHandle;
    fn nvim_ex2_get_curtab() -> *mut TabHandle;
    fn nvim_ex2_get_first_tabpage() -> *mut TabHandle;
    fn nvim_ex2_tp_next(tp: *mut TabHandle) -> *mut TabHandle;
    fn nvim_ex2_tp_firstwin(tp: *mut TabHandle) -> *mut WinHandle;
    fn nvim_ex2_get_vgetc_busy() -> c_int;
    fn nvim_ex2_set_msg_row(val: c_int);
    fn nvim_ex2_set_msg_col(val: c_int);
    fn nvim_ex2_set_msg_didout(val: bool);
    fn nvim_ex2_get_msg_didany() -> bool;
    fn nvim_ex2_get_cmdline_row() -> c_int;
    fn nvim_ex2_get_no_wait_return() -> c_int;
    fn nvim_ex2_set_no_wait_return(val: c_int);
    fn nvim_ex2_set_exiting(val: bool);
    fn nvim_ex2_buflist_findnr(nr: c_int) -> *mut BufHandle;
    fn nvim_ex2_set_curbuf(buf: *mut BufHandle, action: c_int, prevbuf: bool);
    fn nvim_ex2_buf_spname(buf: *mut BufHandle) -> *const c_char;
    fn nvim_ex2_goto_tabpage_win(tp: *mut TabHandle, wp: *mut WinHandle);
    fn nvim_ex2_wait_return(redraw: bool);
    fn nvim_ex2_buf_has_running_job(buf: *mut BufHandle) -> bool;
    fn nvim_ex2_semsg(fmt: *const c_char, arg: *const c_char) -> bool;

    // --- ex_checktime ---
    fn nvim_ex2_eap_get_addr_count(eap: *mut ExArgHandle) -> c_int;
    fn nvim_ex2_eap_get_line2(eap: *mut ExArgHandle) -> i32;
    fn nvim_ex2_get_no_check_timestamps() -> c_int;
    fn nvim_ex2_set_no_check_timestamps(val: c_int);
    fn nvim_ex2_check_timestamps(focus: bool);
    fn nvim_ex2_buf_check_timestamp(buf: *mut BufHandle);
}

/// Add a buffer number to the vector if not already present (replaces `add_bufnum`)
fn bufnum_add(bufnrs: &mut Vec<c_int>, nr: c_int) {
    if !bufnrs.contains(&nr) {
        bufnrs.push(nr);
    }
}

/// Port of `check_changed_any`
#[allow(clippy::too_many_lines)]
#[export_name = "check_changed_any"]
pub unsafe extern "C" fn rs_check_changed_any(hidden: bool, unload: bool) -> bool {
    let mut ret = false;

    // Count buffers
    let mut bufcount: usize = 0;
    let mut buf = unsafe { nvim_ex2_get_firstbuf() };
    while !buf.is_null() {
        bufcount += 1;
        buf = unsafe { nvim_ex2_buf_next(buf) };
    }

    if bufcount == 0 {
        return false;
    }

    // Build priority list of buffer numbers using Vec instead of xmalloc
    let mut bufnrs: Vec<c_int> = Vec::with_capacity(bufcount);

    // curbuf first
    let curbuf = unsafe { nvim_ex2_get_curbuf() };
    bufnrs.push(unsafe { nvim_ex2_buf_get_fnum(curbuf) });

    // buffers in current tab
    let curtab = unsafe { nvim_ex2_get_curtab() };
    let mut wp = unsafe { nvim_ex2_tp_firstwin(curtab) };
    while !wp.is_null() {
        let wbuf = unsafe { nvim_ex2_win_get_buffer(wp) };
        if wbuf != curbuf {
            bufnum_add(&mut bufnrs, unsafe { nvim_ex2_buf_get_fnum(wbuf) });
        }
        wp = unsafe { nvim_ex2_win_next(wp) };
    }

    // buffers in other tabs
    let mut tp = unsafe { nvim_ex2_get_first_tabpage() };
    while !tp.is_null() {
        if tp != curtab {
            let mut wp2 = unsafe { nvim_ex2_tp_firstwin(tp) };
            while !wp2.is_null() {
                let wbuf = unsafe { nvim_ex2_win_get_buffer(wp2) };
                bufnum_add(&mut bufnrs, unsafe { nvim_ex2_buf_get_fnum(wbuf) });
                wp2 = unsafe { nvim_ex2_win_next(wp2) };
            }
        }
        tp = unsafe { nvim_ex2_tp_next(tp) };
    }

    // any other buffer
    buf = unsafe { nvim_ex2_get_firstbuf() };
    while !buf.is_null() {
        bufnum_add(&mut bufnrs, unsafe { nvim_ex2_buf_get_fnum(buf) });
        buf = unsafe { nvim_ex2_buf_next(buf) };
    }

    // Find first changed buffer that can't be abandoned
    let mut found_buf: *mut BufHandle = std::ptr::null_mut();

    for &nr in &bufnrs {
        let b = unsafe { nvim_ex2_buflist_findnr(nr) };
        if b.is_null() {
            continue;
        }
        if (!hidden || unsafe { nvim_ex2_buf_get_nwindows(b) } == 0)
            && unsafe { nvim_ex2_bufIsChanged(b) }
        {
            let bufref = unsafe { nvim_ex2_bufref_create(b) };

            // Try auto-writing the buffer.
            let ccgd_flags = i32::from(unsafe { nvim_ex2_get_p_awa() }) | 2 | 8; // AW | MULTWIN | ALLBUF
            if unsafe { rs_check_changed(b, ccgd_flags) }
                && unsafe { nvim_ex2_bufref_valid(bufref) }
            {
                found_buf = b;
                unsafe { nvim_ex2_bufref_free(bufref) };
                break;
            }
            unsafe { nvim_ex2_bufref_free(bufref) };
        }
    }

    if found_buf.is_null() {
        // No changed buffer found
        return ret;
    }

    // "buf" cannot be abandoned.
    ret = true;
    unsafe { nvim_ex2_set_exiting(false) };

    // When ":confirm" used, don't give an error message.
    if !(unsafe { nvim_ex2_get_p_confirm() }
        || (unsafe { nvim_ex2_get_cmod_flags() } & CMOD_CONFIRM) != 0)
    {
        if unsafe { nvim_ex2_get_vgetc_busy() } > 0 {
            unsafe { nvim_ex2_set_msg_row(nvim_ex2_get_cmdline_row()) };
            unsafe { nvim_ex2_set_msg_col(0) };
            unsafe { nvim_ex2_set_msg_didout(false) };
        }

        let has_job = unsafe { nvim_ex2_buf_has_running_job(found_buf) };
        let fname = unsafe { nvim_ex2_buf_get_fname(found_buf) };

        let msg_ok = if has_job {
            unsafe {
                nvim_ex2_semsg(
                    nvim_ex2_gettext(
                        b"E947: Job still running in buffer \"%s\"\0"
                            .as_ptr()
                            .cast(),
                    ),
                    fname,
                )
            }
        } else {
            let spname = unsafe { nvim_ex2_buf_spname(found_buf) };
            let display = if spname.is_null() { fname } else { spname };
            unsafe {
                nvim_ex2_semsg(
                    nvim_ex2_gettext(
                        b"E162: No write since last change for buffer \"%s\"\0"
                            .as_ptr()
                            .cast(),
                    ),
                    display,
                )
            }
        };

        if msg_ok && unsafe { nvim_ex2_get_msg_didany() } {
            let save = unsafe { nvim_ex2_get_no_wait_return() };
            unsafe { nvim_ex2_set_no_wait_return(0) };
            unsafe { nvim_ex2_wait_return(false) };
            unsafe { nvim_ex2_set_no_wait_return(save) };
        }
    }

    // Try to find a window that contains the buffer.
    let curbuf_now = unsafe { nvim_ex2_get_curbuf() };
    if found_buf != curbuf_now {
        let mut found_win = false;
        let mut tp2 = unsafe { nvim_ex2_get_first_tabpage() };
        'tab_search: while !tp2.is_null() {
            let mut wp3 = unsafe { nvim_ex2_tp_firstwin(tp2) };
            while !wp3.is_null() {
                if unsafe { nvim_ex2_win_get_buffer(wp3) } == found_buf {
                    let bufref = unsafe { nvim_ex2_bufref_create(found_buf) };
                    unsafe { nvim_ex2_goto_tabpage_win(tp2, wp3) };
                    if !unsafe { nvim_ex2_bufref_valid(bufref) } {
                        unsafe { nvim_ex2_bufref_free(bufref) };
                        return ret;
                    }
                    unsafe { nvim_ex2_bufref_free(bufref) };
                    found_win = true;
                    break 'tab_search;
                }
                wp3 = unsafe { nvim_ex2_win_next(wp3) };
            }
            tp2 = unsafe { nvim_ex2_tp_next(tp2) };
        }
        let _ = found_win; // silence unused warning
    }

    // Open the changed buffer in the current window.
    let curbuf_now = unsafe { nvim_ex2_get_curbuf() };
    if found_buf != curbuf_now {
        unsafe {
            nvim_ex2_set_curbuf(
                found_buf,
                if unload { DOBUF_UNLOAD } else { DOBUF_GOTO },
                true,
            );
        }
    }

    ret
}

/// Port of `ex_checktime`
#[export_name = "ex_checktime"]
pub unsafe extern "C" fn rs_ex_checktime(eap: *mut ExArgHandle) {
    let save = unsafe { nvim_ex2_get_no_check_timestamps() };

    unsafe { nvim_ex2_set_no_check_timestamps(0) };
    if unsafe { nvim_ex2_eap_get_addr_count(eap) } == 0 {
        // default is all buffers
        unsafe { nvim_ex2_check_timestamps(false) };
    } else {
        let line2 = unsafe { nvim_ex2_eap_get_line2(eap) };
        let buf = unsafe { nvim_ex2_buflist_findnr(line2) };
        if !buf.is_null() {
            unsafe { nvim_ex2_buf_check_timestamp(buf) };
        }
    }
    unsafe { nvim_ex2_set_no_check_timestamps(save) };
}
