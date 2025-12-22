//! Buffer update tracking utilities for Neovim
//!
//! This module provides Rust implementations for buffer update state checking.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]

use std::os::raw::c_int;

/// Opaque handle to a `buf_T` structure.
#[repr(C)]
pub struct BufHandle {
    _private: [u8; 0],
}

// C accessors for buf_T fields
extern "C" {
    fn nvim_buf_get_update_channels_size(buf: *const BufHandle) -> usize;
    fn nvim_buf_get_update_callbacks_size(buf: *const BufHandle) -> usize;
}

/// Check if a buffer has any active update listeners.
///
/// Returns true if the buffer has any update channels or callbacks registered.
///
/// # Safety
/// `buf` must be a valid pointer to a `buf_T` structure.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_updates_active(buf: *const BufHandle) -> c_int {
    if buf.is_null() {
        return 0;
    }

    let has_channels = nvim_buf_get_update_channels_size(buf) > 0;
    let has_callbacks = nvim_buf_get_update_callbacks_size(buf) > 0;

    c_int::from(has_channels || has_callbacks)
}

#[cfg(test)]
mod tests {
    // Tests would require mocking the C accessor functions
}
