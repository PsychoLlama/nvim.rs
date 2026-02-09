//! Autowrite and buffer-change checking functions
//!
//! Ports of `check_fname`, `buf_write_all`, `autowrite`, `autowrite_all`,
//! `can_abandon`, and `check_changed`.

use std::ffi::{c_char, c_int, c_uint};
use std::ptr;

use crate::CcgdFlags;

const OK: c_int = 1;
const FAIL: c_int = 0;
const CMOD_CONFIRM: c_uint = 0x0080;
const HLF_W: c_int = 26;

// Opaque handle types
#[repr(C)]
pub struct BufHandle {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct BufrefHandle {
    _opaque: [u8; 0],
}

extern "C" {
    // --- buf_T field accessors ---
    fn nvim_ex2_buf_get_ffname(buf: *mut BufHandle) -> *const c_char;
    fn nvim_ex2_buf_get_fname(buf: *mut BufHandle) -> *const c_char;
    fn nvim_ex2_buf_get_b_p_ro(buf: *mut BufHandle) -> c_int;
    fn nvim_ex2_buf_get_nwindows(buf: *mut BufHandle) -> c_int;
    fn nvim_ex2_buf_get_ml_line_count(buf: *mut BufHandle) -> i32;

    // --- buffer iteration ---
    fn nvim_ex2_get_firstbuf() -> *mut BufHandle;
    fn nvim_ex2_buf_next(buf: *mut BufHandle) -> *mut BufHandle;

    // --- globals ---
    fn nvim_ex2_get_p_aw() -> bool;
    fn nvim_ex2_get_p_awa() -> bool;
    fn nvim_ex2_get_p_write() -> bool;
    fn nvim_ex2_get_p_confirm() -> bool;
    fn nvim_ex2_get_curbuf() -> *mut BufHandle;
    fn nvim_ex2_get_cmod_flags() -> c_uint;

    // --- bufref operations (heap-allocated) ---
    fn nvim_ex2_bufref_create(buf: *mut BufHandle) -> *mut BufrefHandle;
    fn nvim_ex2_bufref_valid(br: *mut BufrefHandle) -> bool;
    fn nvim_ex2_bufref_free(br: *mut BufrefHandle);

    // --- function wrappers ---
    fn nvim_ex2_bufIsChanged(buf: *mut BufHandle) -> bool;
    fn nvim_ex2_bt_dontwrite(buf: *mut BufHandle) -> bool;
    fn nvim_ex2_buf_hide(buf: *mut BufHandle) -> bool;
    fn nvim_ex2_buf_write(
        buf: *mut BufHandle,
        ffname: *const c_char,
        fname: *const c_char,
        start: i32,
        end: i32,
        eap: *mut std::ffi::c_void,
        append: bool,
        forceit: bool,
        reset_changed: bool,
        filtering: bool,
    ) -> c_int;
    fn nvim_ex2_msg_source(hl: c_int);
    fn nvim_ex2_msg(s: *const c_char, attr: c_int);
    fn nvim_ex2_no_write_message();
    fn nvim_ex2_no_write_message_nobang(buf: *mut BufHandle);
    fn nvim_ex2_dialog_changed(buf: *mut BufHandle, checkall: bool);
    fn nvim_ex2_emsg(s: *const c_char) -> bool;
    fn nvim_ex2_gettext(s: *const c_char) -> *const c_char;
}

// ---------------------------------------------------------------------------
// Internal Rust-callable implementations
// ---------------------------------------------------------------------------

/// Internal: write all lines in a buffer. Port of `buf_write_all`.
unsafe fn buf_write_all_impl(buf: *mut BufHandle, forceit: bool) -> c_int {
    let old_curbuf = unsafe { nvim_ex2_get_curbuf() };

    let ffname = unsafe { nvim_ex2_buf_get_ffname(buf) };
    let fname = unsafe { nvim_ex2_buf_get_fname(buf) };
    let line_count = unsafe { nvim_ex2_buf_get_ml_line_count(buf) };

    let retval = unsafe {
        nvim_ex2_buf_write(
            buf,
            ffname,
            fname,
            1,
            line_count,
            ptr::null_mut(),
            false,
            forceit,
            true,
            false,
        )
    };

    if unsafe { nvim_ex2_get_curbuf() } != old_curbuf {
        unsafe { nvim_ex2_msg_source(HLF_W) };
        unsafe {
            nvim_ex2_msg(
                nvim_ex2_gettext(
                    b"Warning: Entered other buffer unexpectedly (check autocommands)\0"
                        .as_ptr()
                        .cast(),
                ),
                0,
            );
        }
    }
    retval
}

/// Internal: autowrite a buffer. Port of `autowrite`.
unsafe fn autowrite_impl(buf: *mut BufHandle, forceit: bool) -> c_int {
    if !(unsafe { nvim_ex2_get_p_aw() } || unsafe { nvim_ex2_get_p_awa() })
        || !unsafe { nvim_ex2_get_p_write() }
        || unsafe { nvim_ex2_bt_dontwrite(buf) }
        || (!forceit && unsafe { nvim_ex2_buf_get_b_p_ro(buf) } != 0)
        || unsafe { nvim_ex2_buf_get_ffname(buf) }.is_null()
    {
        return FAIL;
    }

    let bufref = unsafe { nvim_ex2_bufref_create(buf) };
    let write_result = unsafe { buf_write_all_impl(buf, forceit) };

    // Writing may succeed but the buffer still changed, e.g., when there is a
    // conversion error. We do want to return FAIL then.
    let r = if unsafe { nvim_ex2_bufref_valid(bufref) } && unsafe { nvim_ex2_bufIsChanged(buf) } {
        FAIL
    } else {
        write_result
    };
    unsafe { nvim_ex2_bufref_free(bufref) };
    r
}

/// Internal: check if buffer was changed and cannot be abandoned.
unsafe fn check_changed_impl(buf: *mut BufHandle, flags: c_int) -> bool {
    let ccgd = CcgdFlags::from_bits_truncate(flags as u32);
    let forceit = ccgd.contains(CcgdFlags::FORCEIT);

    let bufref = unsafe { nvim_ex2_bufref_create(buf) };

    if !forceit
        && unsafe { nvim_ex2_bufIsChanged(buf) }
        && (ccgd.contains(CcgdFlags::MULTWIN) || unsafe { nvim_ex2_buf_get_nwindows(buf) } <= 1)
        && (!ccgd.contains(CcgdFlags::AW) || unsafe { autowrite_impl(buf, forceit) } == FAIL)
    {
        if (unsafe { nvim_ex2_get_p_confirm() }
            || (unsafe { nvim_ex2_get_cmod_flags() } & CMOD_CONFIRM) != 0)
            && unsafe { nvim_ex2_get_p_write() }
        {
            let mut count: c_int = 0;

            if ccgd.contains(CcgdFlags::ALLBUF) {
                let mut buf2 = unsafe { nvim_ex2_get_firstbuf() };
                while !buf2.is_null() {
                    if unsafe { nvim_ex2_bufIsChanged(buf2) }
                        && !unsafe { nvim_ex2_buf_get_ffname(buf2) }.is_null()
                    {
                        count += 1;
                    }
                    buf2 = unsafe { nvim_ex2_buf_next(buf2) };
                }
            }
            if !unsafe { nvim_ex2_bufref_valid(bufref) } {
                unsafe { nvim_ex2_bufref_free(bufref) };
                return false;
            }
            unsafe { nvim_ex2_dialog_changed(buf, count > 1) };
            if !unsafe { nvim_ex2_bufref_valid(bufref) } {
                unsafe { nvim_ex2_bufref_free(bufref) };
                return false;
            }
            let result = unsafe { nvim_ex2_bufIsChanged(buf) };
            unsafe { nvim_ex2_bufref_free(bufref) };
            return result;
        }
        unsafe { nvim_ex2_bufref_free(bufref) };
        if ccgd.contains(CcgdFlags::EXCMD) {
            unsafe { nvim_ex2_no_write_message() };
        } else {
            unsafe { nvim_ex2_no_write_message_nobang(nvim_ex2_get_curbuf()) };
        }
        return true;
    }
    unsafe { nvim_ex2_bufref_free(bufref) };
    false
}

// ---------------------------------------------------------------------------
// Exported `rs_*` functions
// ---------------------------------------------------------------------------

/// Port of `check_fname`
#[no_mangle]
pub unsafe extern "C" fn rs_check_fname() -> c_int {
    let curbuf = unsafe { nvim_ex2_get_curbuf() };
    if unsafe { nvim_ex2_buf_get_ffname(curbuf) }.is_null() {
        unsafe {
            nvim_ex2_emsg(nvim_ex2_gettext(b"E32: No file name\0".as_ptr().cast()));
        }
        return FAIL;
    }
    OK
}

/// Port of `buf_write_all`
#[no_mangle]
pub unsafe extern "C" fn rs_buf_write_all(buf: *mut BufHandle, forceit: bool) -> c_int {
    unsafe { buf_write_all_impl(buf, forceit) }
}

/// Port of `autowrite`
#[no_mangle]
pub unsafe extern "C" fn rs_autowrite(buf: *mut BufHandle, forceit: bool) -> c_int {
    unsafe { autowrite_impl(buf, forceit) }
}

/// Port of `autowrite_all`
#[no_mangle]
pub unsafe extern "C" fn rs_autowrite_all() {
    if !(unsafe { nvim_ex2_get_p_aw() } || unsafe { nvim_ex2_get_p_awa() })
        || !unsafe { nvim_ex2_get_p_write() }
    {
        return;
    }

    let mut buf = unsafe { nvim_ex2_get_firstbuf() };
    while !buf.is_null() {
        if unsafe { nvim_ex2_bufIsChanged(buf) }
            && unsafe { nvim_ex2_buf_get_b_p_ro(buf) } == 0
            && !unsafe { nvim_ex2_bt_dontwrite(buf) }
        {
            let bufref = unsafe { nvim_ex2_bufref_create(buf) };
            unsafe { buf_write_all_impl(buf, false) };
            // an autocommand may have deleted the buffer
            if !unsafe { nvim_ex2_bufref_valid(bufref) } {
                buf = unsafe { nvim_ex2_get_firstbuf() };
            }
            unsafe { nvim_ex2_bufref_free(bufref) };
        }
        buf = unsafe { nvim_ex2_buf_next(buf) };
    }
}

/// Port of `can_abandon`
#[no_mangle]
pub unsafe extern "C" fn rs_can_abandon(buf: *mut BufHandle, forceit: bool) -> bool {
    unsafe {
        nvim_ex2_buf_hide(buf)
            || !nvim_ex2_bufIsChanged(buf)
            || nvim_ex2_buf_get_nwindows(buf) > 1
            || autowrite_impl(buf, forceit) == OK
            || forceit
    }
}

/// Port of `check_changed`
#[no_mangle]
pub unsafe extern "C" fn rs_check_changed(buf: *mut BufHandle, flags: c_int) -> bool {
    unsafe { check_changed_impl(buf, flags) }
}
