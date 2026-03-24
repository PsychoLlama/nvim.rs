//! Buffer file name management helpers.
//!
//! Implements `buf_set_name` and `buf_name_changed` which manage the
//! file name association for buffers.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int};

use crate::BufHandle;

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
