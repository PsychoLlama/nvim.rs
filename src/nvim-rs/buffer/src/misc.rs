//! Miscellaneous standalone buffer functions
//!
//! This module contains small, self-contained buffer utility functions
//! migrated from `src/nvim/buffer.c` in Phase 1.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void};

use crate::BufHandle;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    // curbuf accessors
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_curbuf_get_ffname() -> *const c_char;
    fn nvim_curbuf_ml_line_count() -> c_int;
    fn nvim_get_curbuf_ml_flags() -> c_int;

    // Buffer accessors
    fn nvim_buf_channel_job_running(buf: BufHandle) -> c_int;
    fn nvim_get_buf_free_count() -> c_int;
    fn nvim_buf_get_fnum(buf: BufHandle) -> c_int;

    // Option accessors
    fn nvim_get_p_acd() -> c_int;

    // C functions we call
    fn gettext(msgid: *const c_char) -> *const c_char;
    fn emsg(s: *const c_char) -> c_int;
    fn vim_chdirfile(fname: *mut c_char, cause: c_int) -> c_int;
    fn shorten_fnames(force: bool);
    fn extmark_free_all(buf: BufHandle);
    fn ml_delete(lnum: c_int) -> c_int;
    fn deleted_lines_mark(lnum: c_int, count: c_int);
    fn text_locked() -> bool;
    fn curbuf_locked() -> bool;
    fn get_text_locked_msg() -> *const c_char;
}

// =============================================================================
// External C Statics
// =============================================================================

extern "C" {
    static mut starting: c_int;
    static mut last_chdir_reason: *const c_char;
}

// =============================================================================
// Constants
// =============================================================================

/// OK return value from C functions
const OK: c_int = 1;
/// `ML_EMPTY` flag value (memline has no lines)
const ML_EMPTY: c_int = 0x01;
/// kCdCauseAuto = 2 (from `vim_defs.h`: Other=-1, Manual=0, Window=1, Auto=2)
const K_CD_CAUSE_AUTO: c_int = 2;

// =============================================================================
// bufref_T layout (must match buffer_defs.h exactly)
// =============================================================================

/// Rust mirror of `bufref_T` from `buffer_defs.h`.
///
/// # Safety
/// This struct MUST match the C layout exactly:
/// `{ buf_T *br_buf; int br_fnum; int br_buf_free_count; }`
#[repr(C)]
pub struct BufRef {
    pub br_buf: *mut c_void, // buf_T*
    pub br_fnum: c_int,
    pub br_buf_free_count: c_int,
}

// =============================================================================
// Phase 1 Implementations
// =============================================================================

/// Change to the directory of the current buffer.
///
/// Rust port of C `do_autochdir()`.
///
/// # Safety
/// Accesses global Neovim state.
#[no_mangle]
pub unsafe extern "C" fn do_autochdir() {
    if nvim_get_p_acd() != 0 {
        let ffname = nvim_curbuf_get_ffname();
        if starting == 0 && !ffname.is_null() {
            // vim_chdirfile takes a mutable pointer; const-cast is safe here
            // as the function only reads the string.
            let fname_mut = ffname.cast_mut();
            if vim_chdirfile(fname_mut, K_CD_CAUSE_AUTO) == OK {
                last_chdir_reason = c"autochdir".as_ptr();
                shorten_fnames(true);
            }
        }
    }
}

/// Emit an error for a buffer that has unsaved changes.
///
/// Rust port of C `no_write_message()`.
///
/// # Safety
/// Accesses global Neovim state.
#[no_mangle]
pub unsafe extern "C" fn no_write_message() {
    let curbuf = nvim_get_curbuf();
    if nvim_buf_channel_job_running(curbuf) != 0 {
        emsg(gettext(
            c"E948: Job still running (add ! to end the job)".as_ptr(),
        ));
    } else {
        emsg(gettext(
            c"E37: No write since last change (add ! to override)".as_ptr(),
        ));
    }
}

/// Emit an error for a buffer that has unsaved changes (no-bang variant).
///
/// Rust port of C `no_write_message_nobang()`.
///
/// # Safety
/// `buf` must be a valid buffer handle.
#[no_mangle]
pub unsafe extern "C" fn no_write_message_nobang(buf: BufHandle) {
    if nvim_buf_channel_job_running(buf) != 0 {
        emsg(gettext(c"E948: Job still running".as_ptr()));
    } else {
        emsg(gettext(c"E37: No write since last change".as_ptr()));
    }
}

/// Emit an error message when text is locked (cmdline window open, etc.).
///
/// Rust port of C `text_locked_msg()`.
///
/// # Safety
/// Accesses global Neovim state.
#[no_mangle]
pub unsafe extern "C" fn text_locked_msg() {
    emsg(gettext(get_text_locked_msg()));
}

/// Check for text, window or buffer locked.
///
/// Returns `true` and emits an error message if something is locked.
///
/// Rust port of C `text_or_buf_locked()`.
///
/// # Safety
/// Accesses global Neovim state.
#[no_mangle]
pub unsafe extern "C" fn text_or_buf_locked() -> bool {
    if text_locked() {
        text_locked_msg();
        return true;
    }
    curbuf_locked()
}

/// Clear the current buffer contents.
///
/// Deletes all lines and extmarks from `curbuf`.
///
/// Rust port of C `buf_clear()`.
///
/// # Safety
/// Accesses global `curbuf`. Must be called on the main Neovim thread.
#[no_mangle]
pub unsafe extern "C" fn buf_clear() {
    let line_count = nvim_curbuf_ml_line_count();
    let curbuf = nvim_get_curbuf();
    extmark_free_all(curbuf);
    while nvim_get_curbuf_ml_flags() & ML_EMPTY == 0 {
        ml_delete(1);
    }
    deleted_lines_mark(1, line_count);
}

/// Initialize a buffer reference (`bufref_T`).
///
/// Sets `br_buf`, `br_fnum`, and `br_buf_free_count` on the given reference.
///
/// Rust port of C `set_bufref()`.
///
/// # Safety
/// `bufref` must be a valid non-null pointer to a `bufref_T`.
/// `buf` may be null (represents no buffer).
#[no_mangle]
pub unsafe extern "C" fn set_bufref(bufref: *mut BufRef, buf: BufHandle) {
    if bufref.is_null() {
        return;
    }
    let br = &mut *bufref;
    br.br_buf = buf.as_ptr();
    br.br_fnum = if buf.is_null() {
        0
    } else {
        nvim_buf_get_fnum(buf)
    };
    br.br_buf_free_count = nvim_get_buf_free_count();
}
