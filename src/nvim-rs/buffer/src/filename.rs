//! Buffer file name management helpers.
//!
//! Implements `setfname`, `buf_set_name`, and `buf_name_changed` which manage
//! the file name association for buffers.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int};

use crate::BufHandle;

// FileID size: sizeof(FileID) <= 16, asserted in buffer_shim.c
const FILE_ID_SIZE: usize = 16;

// BF_DUMMY flag value (from buffer_defs.h)
const BF_DUMMY: c_int = 0x80;

// Return codes from C
const FAIL: c_int = 0;
const OK: c_int = 1;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    // buf_set_name helpers
    fn rs_buflist_findnr(fnum: c_int) -> BufHandle;
    fn nvim_buf_set_name_body(buf: BufHandle, name: *mut c_char);

    // buf_name_changed helpers
    fn nvim_buf_get_ml_mfp_null(buf: BufHandle) -> c_int;
    fn nvim_ml_setname(buf: BufHandle);
    fn nvim_check_arg_idx_if_curbuf(buf: BufHandle);
    fn nvim_maketitle();
    fn nvim_status_redraw_all();
    fn nvim_fmarks_check_names(buf: BufHandle);
    fn nvim_ml_timestamp(buf: BufHandle);

    // setfname helpers
    fn nvim_fname_expand(
        buf: BufHandle,
        ffname_ptr: *mut *mut c_char,
        sfname_ptr: *mut *mut c_char,
    );
    fn nvim_os_fileid(path: *const c_char, file_id_out: *mut u8) -> bool;
    fn nvim_buf_get_flags(buf: BufHandle) -> c_int;
    fn nvim_buf_get_ml_mfp(buf: BufHandle) -> *mut std::ffi::c_void;
    fn buflist_findname_file_id(
        ffname: *const c_char,
        file_id: *const u8,
        file_id_valid: bool,
    ) -> BufHandle;
    fn nvim_buf_is_in_any_window(buf: BufHandle) -> bool;
    fn nvim_emsg_e95_buffer_exists();
    fn nvim_xfree_char(ptr: *mut c_char);
    // nvim_close_buffer_wipe defined in quickfix_shim.c with void* parameter
    fn nvim_close_buffer_wipe(obuf: *mut std::ffi::c_void);
    fn nvim_buf_remove_fnames(buf: BufHandle);
    fn nvim_buf_set_fnames(buf: BufHandle, ffname: *mut c_char, sfname: *mut c_char);
    fn nvim_buf_set_file_id_data(buf: BufHandle, file_id: *const u8, valid: bool);
    fn buf_name_changed(buf: BufHandle);
}

// =============================================================================
// setfname implementation
// =============================================================================

/// Set the file name for `buf` to `ffname_arg` / `sfname_arg`.
///
/// The full path is also remembered for when `:cd` is used.
/// `message`: give a message when a buffer with the name already exists.
///
/// Returns `OK` or `FAIL` (name already in use by a loaded buffer).
///
/// Mirrors C `setfname`.
///
/// # Safety
///
/// Must be called on the main thread with valid Neovim state.
#[unsafe(export_name = "setfname")]
pub unsafe extern "C" fn rs_setfname(
    buf: BufHandle,
    ffname_arg: *mut c_char,
    sfname_arg: *mut c_char,
    message: bool,
) -> c_int {
    if ffname_arg.is_null() || *ffname_arg == 0 {
        // Removing the name.
        nvim_buf_remove_fnames(buf);
        nvim_buf_set_file_id_data(buf, std::ptr::null(), false);
        buf_name_changed(buf);
        return OK;
    }

    let mut ffname: *mut c_char = ffname_arg;
    let mut sfname: *mut c_char = sfname_arg;
    nvim_fname_expand(buf, &raw mut ffname, &raw mut sfname);

    if ffname.is_null() {
        // out of memory
        return FAIL;
    }

    // Check if the file is already in another buffer.
    let mut file_id = [0u8; FILE_ID_SIZE];
    let file_id_valid = nvim_os_fileid(ffname, file_id.as_mut_ptr());

    let obuf = if (nvim_buf_get_flags(buf) & BF_DUMMY) == 0 {
        buflist_findname_file_id(ffname, file_id.as_ptr(), file_id_valid)
    } else {
        BufHandle(std::ptr::null_mut())
    };

    if !obuf.is_null() && obuf != buf {
        // Is obuf loaded or used in a window?
        let in_use = !nvim_buf_get_ml_mfp(obuf).is_null() || nvim_buf_is_in_any_window(obuf);
        if in_use {
            if message {
                nvim_emsg_e95_buffer_exists();
            }
            nvim_xfree_char(ffname);
            return FAIL;
        }
        // Not loaded and not in any window: wipe it from the list.
        nvim_close_buffer_wipe(obuf.0);
    }

    // Set the new names (frees old names, dups sfname, applies path_fix_case).
    nvim_buf_set_fnames(buf, ffname, sfname);
    nvim_buf_set_file_id_data(buf, file_id.as_ptr(), file_id_valid);

    buf_name_changed(buf);
    OK
}

// =============================================================================
// buf_set_name implementation
// =============================================================================

/// Set the file name for buffer `fnum` to `name`, expanding to full path.
///
/// # Safety
///
/// Must be called on the main thread with valid Neovim state.
#[unsafe(export_name = "buf_set_name")]
pub unsafe extern "C" fn rs_buf_set_name(fnum: c_int, name: *mut c_char) {
    let buf = rs_buflist_findnr(fnum);
    if buf.is_null() {
        return;
    }
    nvim_buf_set_name_body(buf, name);
}

// =============================================================================
// buf_name_changed implementation
// =============================================================================

/// Take care of what needs to be done when the name of buffer `buf` has changed.
///
/// - Updates the swap file name if a memfile is open.
/// - Checks the arg list index if buf is current.
/// - Redraws the title and status lines.
/// - Checks named file marks and resets the memline timestamp.
///
/// # Safety
///
/// Must be called on the main thread with valid Neovim state.
#[unsafe(export_name = "buf_name_changed")]
pub unsafe extern "C" fn rs_buf_name_changed(buf: BufHandle) {
    // If the file name changed, also change the name of the swapfile
    if nvim_buf_get_ml_mfp_null(buf) == 0 {
        nvim_ml_setname(buf);
    }

    nvim_check_arg_idx_if_curbuf(buf); // check file name for arg list
    nvim_maketitle(); // set window title
    nvim_status_redraw_all(); // status lines need to be redrawn
    nvim_fmarks_check_names(buf); // check named file marks
    nvim_ml_timestamp(buf); // reset timestamp
}
