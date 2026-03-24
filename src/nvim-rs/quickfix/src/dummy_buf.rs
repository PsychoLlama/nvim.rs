//! Dummy buffer management for vimgrep.
//!
//! Implements `nvim_load_dummy_buf`, `nvim_wipe_dummy_buffer`, and
//! `nvim_unload_dummy_buffer`, migrated from the C static functions
//! `vgr_load_dummy_buf`, `load_dummy_buffer`, `wipe_dummy_buffer`,
//! `unload_dummy_buffer`, and `restore_start_dir` in `quickfix_shim.c`.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::{c_char, c_int, c_void};

type BufHandle = *mut c_void;
type WinHandle = *mut c_void;

// Buffer flag constants
const BF_DUMMY: c_int = 0x80;
const BF_NEW: c_int = 0x10;

// Buffer list new flags
const BLN_DUMMY: c_int = 4;

// readfile result value
const OK: c_int = 1;

const MAXPATHL: usize = 4096;

extern "C" {
    // Globals
    static mut got_int: bool;
    static mut curbuf: BufHandle;

    // Buffer lifecycle
    fn nvim_buflist_new(
        ffname: *mut c_char,
        sfname: *mut c_char,
        lnum: i32,
        flags: c_int,
    ) -> BufHandle;
    fn nvim_buf_copy_options_enter(buf: BufHandle);
    fn nvim_aucmd_prepbuf_alloc(buf: BufHandle) -> *mut c_void;
    fn nvim_aucmd_restbuf_free(aco: *mut c_void);
    fn nvim_setfname_curbuf(fname: *mut c_char);
    fn nvim_check_need_swap_newfile();
    fn nvim_readfile_for_dummy(fname: *mut c_char) -> c_int;
    fn nvim_buf_inc_locked(buf: BufHandle);
    fn nvim_buf_dec_locked(buf: BufHandle);
    fn nvim_close_buffer_unload(buf: BufHandle) -> c_int;

    // Buffer flag accessors
    fn nvim_buf_get_b_flags(buf: BufHandle) -> c_int;
    fn nvim_buf_set_b_flags(buf: BufHandle, val: c_int);

    // Window iteration
    fn nvim_get_firstwin() -> WinHandle;
    fn nvim_win_get_next_in_tab(wp: WinHandle) -> WinHandle;
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_win_close_no_free(wp: WinHandle) -> c_int;
    fn nvim_buf_get_nwindows(buf: BufHandle) -> c_int;

    // Cleanup (exception state save/restore)
    fn nvim_cleanup_enter_alloc() -> *mut c_void;
    fn nvim_cleanup_leave_free(cs: *mut c_void);

    // bufref helpers (heap-allocated C bufref_T)
    fn nvim_qf_bufref_alloc(buf: BufHandle) -> *mut c_void;
    fn nvim_qf_bufref_is_valid(br: *mut c_void) -> bool;
    fn nvim_qf_bufref_get_buf(br: *mut c_void) -> BufHandle;
    fn nvim_qf_bufref_set_buf_null(br: *mut c_void);
    fn nvim_qf_bufref_free(br: *mut c_void);

    // wipe_buffer (already Rust-exported as rs_wipe_buffer, called via C name)
    fn wipe_buffer(buf: BufHandle, aucmd: bool);

    // autocmd blocking
    fn block_autocmds();
    fn unblock_autocmds();

    // au_event_disable / au_event_restore (already Rust-exported under these C names)
    fn au_event_disable(what: *const c_char) -> *mut c_char;
    fn au_event_restore(old_ei: *mut c_char);

    // Directory
    fn os_dirname(buf: *mut c_char, size: usize);
    fn nvim_ex_cd_arg(arg: *mut c_char, is_lcd: bool);
    fn nvim_docmd_curwin_has_localdir() -> bool;

    // OptInt p_mls (modeline setting)
    static mut p_mls: i64;
}

/// Restore the current working directory to `dirname_start` if they differ.
///
/// Mirrors `restore_start_dir` in `quickfix_shim.c`.
unsafe fn restore_start_dir(dirname_start: *mut c_char) {
    let mut dirname_now = vec![0u8; MAXPATHL];
    os_dirname(dirname_now.as_mut_ptr().cast::<c_char>(), MAXPATHL);

    // Compare dirname_start to dirname_now
    let start = std::ffi::CStr::from_ptr(dirname_start).to_bytes();
    let now_cstr = std::ffi::CStr::from_ptr(dirname_now.as_ptr().cast::<c_char>());
    let now = now_cstr.to_bytes();

    if start != now {
        nvim_ex_cd_arg(dirname_start, nvim_docmd_curwin_has_localdir());
    }
}

/// Wipe a dummy buffer, closing any windows that display it first.
///
/// Mirrors `wipe_dummy_buffer` in `quickfix_shim.c`.
///
/// # Safety
///
/// `buf` must be a valid pointer to a `buf_T`. `dirname_start` may be NULL.
pub unsafe fn wipe_dummy_buffer_internal(buf: BufHandle, dirname_start: *mut c_char) {
    // Close any windows displaying this buffer.
    loop {
        if nvim_buf_get_nwindows(buf) == 0 {
            break;
        }
        let mut did_one = false;
        let firstwin = nvim_get_firstwin();
        if !nvim_win_get_next_in_tab(firstwin).is_null() {
            let mut wp = firstwin;
            while !wp.is_null() {
                if nvim_win_get_buffer(wp) == buf {
                    if nvim_win_close_no_free(wp) == 1 {
                        // OK = 1
                        did_one = true;
                    }
                    break;
                }
                wp = nvim_win_get_next_in_tab(wp);
            }
        }
        if !did_one {
            // Cannot close all windows; fall through to fail path
            // (clear BF_DUMMY and return without wiping)
            let flags = nvim_buf_get_b_flags(buf);
            nvim_buf_set_b_flags(buf, flags & !BF_DUMMY);
            return;
        }
    }

    // Safety check: curbuf must not be buf
    if curbuf != buf && nvim_buf_get_nwindows(buf) == 0 {
        let cs = nvim_cleanup_enter_alloc();
        wipe_buffer(buf, true);
        nvim_cleanup_leave_free(cs);

        if !dirname_start.is_null() {
            restore_start_dir(dirname_start);
        }
    } else {
        // Keeping the buffer, remove dummy flag
        let flags = nvim_buf_get_b_flags(buf);
        nvim_buf_set_b_flags(buf, flags & !BF_DUMMY);
    }
}

/// Unload a dummy buffer.
///
/// Mirrors `unload_dummy_buffer` in `quickfix_shim.c`.
///
/// # Safety
///
/// `buf` must be a valid pointer to a `buf_T`.
unsafe fn unload_dummy_buffer_internal(buf: BufHandle, dirname_start: *mut c_char) {
    if curbuf == buf {
        // Safety check
        return;
    }
    nvim_close_buffer_unload(buf);
    restore_start_dir(dirname_start);
}

/// Load a dummy buffer for vimgrep with Filetype autocmds and modelines disabled.
///
/// Mirrors `vgr_load_dummy_buf` -> `load_dummy_buffer` chain in `quickfix_shim.c`.
///
/// # Safety
///
/// All pointer arguments must be valid.
unsafe fn load_dummy_buf_internal(
    fname: *mut c_char,
    dirname_start: *mut c_char,
    dirname_now: *mut c_char,
) -> BufHandle {
    // Suppress Filetype autocommands and modelines for speed.
    let save_ei = au_event_disable(c",Filetype".as_ptr());
    let save_mls = p_mls;
    p_mls = 0;

    let buf = load_dummy_buffer(fname, dirname_start, dirname_now);

    p_mls = save_mls;
    au_event_restore(save_ei);

    buf
}

/// Core dummy buffer loading logic.
///
/// Mirrors `load_dummy_buffer` in `quickfix_shim.c`.
unsafe fn load_dummy_buffer(
    fname: *mut c_char,
    dirname_start: *mut c_char,
    resulting_dir: *mut c_char,
) -> BufHandle {
    let newbuf = nvim_buflist_new(std::ptr::null_mut(), std::ptr::null_mut(), 1, BLN_DUMMY);
    if newbuf.is_null() {
        return std::ptr::null_mut();
    }

    let newbufref = nvim_qf_bufref_alloc(newbuf);
    nvim_buf_copy_options_enter(newbuf);

    // OKval: ml_open returns OK (1) on success
    // We declare ml_open separately since its signature is buf-specific
    let ml_open_ok = {
        extern "C" {
            fn ml_open(buf: BufHandle) -> c_int;
        }
        ml_open(newbuf) == OK
    };

    let mut failed = true;
    let mut result_buf = newbuf;

    if ml_open_ok {
        nvim_buf_inc_locked(newbuf);
        let aco = nvim_aucmd_prepbuf_alloc(newbuf);

        nvim_setfname_curbuf(fname);
        nvim_check_need_swap_newfile();

        // Clear BF_DUMMY on curbuf so autocommands fire properly
        let flags = nvim_buf_get_b_flags(curbuf);
        nvim_buf_set_b_flags(curbuf, flags & !BF_DUMMY);

        let newbuf_to_wipe_ref = nvim_qf_bufref_alloc(std::ptr::null_mut());
        nvim_qf_bufref_set_buf_null(newbuf_to_wipe_ref);

        let readfile_result = nvim_readfile_for_dummy(fname);
        nvim_buf_dec_locked(newbuf);

        if readfile_result == OK && !got_int && (nvim_buf_get_b_flags(curbuf) & BF_NEW) == 0 {
            failed = false;
            if curbuf != newbuf {
                // Autocommands changed the buffer (e.g. netrw).
                // Use curbuf instead, wipe the dummy newbuf afterwards.
                let old_newbuf = newbuf;
                nvim_qf_bufref_free(newbuf_to_wipe_ref);
                let newbuf_to_wipe_ref2 = nvim_qf_bufref_alloc(old_newbuf);
                nvim_aucmd_restbuf_free(aco);

                if !nvim_qf_bufref_get_buf(newbuf_to_wipe_ref2).is_null()
                    && nvim_qf_bufref_is_valid(newbuf_to_wipe_ref2)
                {
                    block_autocmds();
                    wipe_dummy_buffer_internal(
                        nvim_qf_bufref_get_buf(newbuf_to_wipe_ref2),
                        std::ptr::null_mut(),
                    );
                    unblock_autocmds();
                }
                nvim_qf_bufref_free(newbuf_to_wipe_ref2);

                result_buf = curbuf;
                // Set BF_DUMMY on result_buf
                let f2 = nvim_buf_get_b_flags(result_buf);
                nvim_buf_set_b_flags(result_buf, f2 | BF_DUMMY);

                os_dirname(resulting_dir, MAXPATHL);
                restore_start_dir(dirname_start);

                if !nvim_qf_bufref_is_valid(newbufref) {
                    nvim_qf_bufref_free(newbufref);
                    return std::ptr::null_mut();
                }
                nvim_qf_bufref_free(newbufref);
                return result_buf;
            }
        }

        nvim_aucmd_restbuf_free(aco);

        let wipe_buf = nvim_qf_bufref_get_buf(newbuf_to_wipe_ref);
        if !wipe_buf.is_null() && nvim_qf_bufref_is_valid(newbuf_to_wipe_ref) {
            block_autocmds();
            wipe_dummy_buffer_internal(wipe_buf, std::ptr::null_mut());
            unblock_autocmds();
        }
        nvim_qf_bufref_free(newbuf_to_wipe_ref);

        // Restore BF_DUMMY flag
        let f = nvim_buf_get_b_flags(result_buf);
        nvim_buf_set_b_flags(result_buf, f | BF_DUMMY);
    }

    os_dirname(resulting_dir, MAXPATHL);
    restore_start_dir(dirname_start);

    if !nvim_qf_bufref_is_valid(newbufref) {
        nvim_qf_bufref_free(newbufref);
        return std::ptr::null_mut();
    }
    nvim_qf_bufref_free(newbufref);

    if failed {
        wipe_dummy_buffer_internal(result_buf, dirname_start);
        return std::ptr::null_mut();
    }

    result_buf
}

// =============================================================================
// Exported functions (replace C nvim_* wrappers)
// =============================================================================

/// Load a dummy buffer to search for a pattern using vimgrep.
///
/// Replaces C `nvim_load_dummy_buf` / `vgr_load_dummy_buf` / `load_dummy_buffer`.
///
/// # Safety
///
/// All pointer arguments must be valid C strings.
#[no_mangle]
pub unsafe extern "C" fn nvim_load_dummy_buf(
    fname: *const c_char,
    dirname_start: *mut c_char,
    dirname_now: *mut c_char,
) -> BufHandle {
    load_dummy_buf_internal(fname.cast_mut(), dirname_start, dirname_now)
}

/// Wipe a dummy buffer, closing any windows first.
///
/// Replaces C `nvim_wipe_dummy_buffer` / `wipe_dummy_buffer`.
///
/// # Safety
///
/// `buf` must be a valid pointer to a `buf_T`. `dirname_start` may be NULL.
#[no_mangle]
pub unsafe extern "C" fn nvim_wipe_dummy_buffer(buf: BufHandle, dirname_start: *mut c_char) {
    wipe_dummy_buffer_internal(buf, dirname_start);
}

/// Unload a dummy buffer.
///
/// Replaces C `nvim_unload_dummy_buffer` / `unload_dummy_buffer`.
///
/// # Safety
///
/// `buf` must be a valid pointer to a `buf_T`.
#[no_mangle]
pub unsafe extern "C" fn nvim_unload_dummy_buffer(buf: BufHandle, dirname_start: *mut c_char) {
    unload_dummy_buffer_internal(buf, dirname_start);
}
