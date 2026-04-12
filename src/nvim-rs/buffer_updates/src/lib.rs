//! Buffer update tracking utilities for Neovim
//!
//! This module provides Rust implementations for buffer update state checking
//! and notification.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::c_int;

/// Opaque handle to a `buf_T` structure.
#[repr(C)]
pub struct BufHandle {
    _private: [u8; 0],
}

/// `BufUpdateCallbacks` struct matching C definition in `buffer_defs.h`.
/// Contains `LuaRef` fields (`c_int` each) and two bool fields.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BufUpdateCallbacks {
    pub on_lines: LuaRef,
    pub on_bytes: LuaRef,
    pub on_changedtick: LuaRef,
    pub on_detach: LuaRef,
    pub on_reload: LuaRef,
    pub utf_sizes: bool,
    pub preview: bool,
}

/// `LuaRef` type matching C definition (`int`).
pub type LuaRef = c_int;

// C accessors for buf_T fields
extern "C" {
    fn nvim_buf_get_update_channels_size(buf: *const BufHandle) -> usize;
    fn nvim_buf_get_update_callbacks_size(buf: *const BufHandle) -> usize;

    /// Free the `LuaRef` fields of a `BufUpdateCallbacks`.
    fn nvim_buf_callbacks_free_refs(cb: BufUpdateCallbacks);

    /// Send `nvim_buf_changedtick_event` to a single channel.
    fn nvim_buf_send_changedtick_event(buf: *mut BufHandle, channel_id: u64);

    /// Send `nvim_buf_detach_event` to a single channel.
    fn nvim_buf_send_detach_event(buf: *mut BufHandle, channel_id: u64);
}

/// Check if a buffer has any active update listeners.
///
/// Returns true if the buffer has any update channels or callbacks registered.
///
/// # Safety
/// `buf` must be a valid pointer to a `buf_T` structure.
#[export_name = "buf_updates_active"]
#[allow(clippy::must_use_candidate)]
pub unsafe extern "C" fn rs_buf_updates_active(buf: *const BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }

    let has_channels = nvim_buf_get_update_channels_size(buf) > 0;
    let has_callbacks = nvim_buf_get_update_callbacks_size(buf) > 0;

    has_channels || has_callbacks
}

/// Free the `LuaRef` fields of a `BufUpdateCallbacks` struct.
///
/// # Safety
/// The `LuaRef` fields in `cb` must be valid (or `LUA_NOREF`).
#[export_name = "buffer_update_callbacks_free"]
pub unsafe extern "C" fn rs_buffer_update_callbacks_free(cb: BufUpdateCallbacks) {
    nvim_buf_callbacks_free_refs(cb);
}

/// Send the `nvim_buf_changedtick_event` RPC event to a single channel.
///
/// # Safety
/// `buf` must be a valid pointer to a `buf_T` structure.
#[export_name = "buf_updates_changedtick_single"]
pub unsafe extern "C" fn rs_buf_updates_changedtick_single(buf: *mut BufHandle, channel_id: u64) {
    nvim_buf_send_changedtick_event(buf, channel_id);
}

/// Send the `nvim_buf_detach_event` RPC event to a single channel.
///
/// # Safety
/// `buf` must be a valid pointer to a `buf_T` structure.
#[export_name = "buf_updates_send_end"]
pub unsafe extern "C" fn rs_buf_updates_send_end(buf: *mut BufHandle, channel_id: u64) {
    nvim_buf_send_detach_event(buf, channel_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buf_update_callbacks_layout() {
        // `BufUpdateCallbacks` has 5 `LuaRef` (c_int = i32 = 4 bytes each) + 2 bool (1 byte each)
        // With typical alignment: 5*4 + 2*1 + padding = 24 bytes
        assert!(std::mem::size_of::<BufUpdateCallbacks>() >= 22);
    }
}
