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

/// `LUA_NOREF`: Lua reference that is not set.
#[allow(dead_code)]
const LUA_NOREF: LuaRef = -2;

/// `LUA_INTERNAL_CALL`: channel ID used for Lua internal calls.
/// Matches C: `(VIML_INTERNAL_CALL + 1) = (INTERNAL_CALL_MASK + 1)`.
const LUA_INTERNAL_CALL: u64 = (1u64 << 63) + 1;

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

    // Phase 2: channel management accessors
    fn nvim_buf_update_channels_get(buf: *mut BufHandle, i: usize) -> u64;
    fn nvim_buf_update_channels_set(buf: *mut BufHandle, i: usize, channel_id: u64);
    fn nvim_buf_update_channels_shrink(buf: *mut BufHandle, count: usize);
    fn nvim_buf_update_channels_push(buf: *mut BufHandle, channel_id: u64);
    fn nvim_buf_update_channels_destroy(buf: *mut BufHandle);
    fn nvim_buf_update_callbacks_push(buf: *mut BufHandle, cb: BufUpdateCallbacks);
    fn nvim_buf_set_update_need_codepoints(buf: *mut BufHandle, val: bool);
    fn nvim_buf_send_initial_lines(buf: *mut BufHandle, channel_id: u64);
    fn nvim_buf_get_ml_mfp_is_null(buf: *mut BufHandle) -> bool;
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

/// Register a channel or callback for buffer updates.
///
/// Returns `true` if the channel/callback was added or was already registered.
/// Returns `false` if the buffer is not loaded.
///
/// # Safety
/// `buf` must be a valid pointer to a loaded `buf_T` structure.
#[export_name = "buf_updates_register"]
pub unsafe extern "C" fn rs_buf_updates_register(
    buf: *mut BufHandle,
    channel_id: u64,
    cb: BufUpdateCallbacks,
    send_buffer: bool,
) -> bool {
    // must fail if the buffer isn't loaded
    if nvim_buf_get_ml_mfp_is_null(buf) {
        return false;
    }

    if channel_id == LUA_INTERNAL_CALL {
        nvim_buf_update_callbacks_push(buf, cb);
        if cb.utf_sizes {
            nvim_buf_set_update_need_codepoints(buf, true);
        }
        return true;
    }

    // check if channel is already registered
    let size = nvim_buf_get_update_channels_size(buf);
    for i in 0..size {
        if nvim_buf_update_channels_get(buf, i) == channel_id {
            return true;
        }
    }

    // append the channel id to the list
    nvim_buf_update_channels_push(buf, channel_id);

    if send_buffer {
        nvim_buf_send_initial_lines(buf, channel_id);
    } else {
        nvim_buf_send_changedtick_event(buf, channel_id);
    }

    true
}

/// Unregister a channel from buffer updates.
///
/// Removes the channel from `update_channels` and sends a detach event.
///
/// # Safety
/// `buf` must be a valid pointer to a `buf_T` structure.
#[export_name = "buf_updates_unregister"]
pub unsafe extern "C" fn rs_buf_updates_unregister(buf: *mut BufHandle, channel_id: u64) {
    let size = nvim_buf_get_update_channels_size(buf);
    if size == 0 {
        return;
    }

    // compact: keep channels that don't match channel_id
    let mut j = 0usize;
    let mut found = 0usize;
    for i in 0..size {
        if nvim_buf_update_channels_get(buf, i) == channel_id {
            found += 1;
        } else {
            if i != j {
                let val = nvim_buf_update_channels_get(buf, i);
                nvim_buf_update_channels_set(buf, j, val);
            }
            j += 1;
        }
    }

    if found > 0 {
        // remove found items from the end
        nvim_buf_update_channels_shrink(buf, found);

        // send detach event
        nvim_buf_send_detach_event(buf, channel_id);

        if found == size {
            // all channels were removed: destroy and reinit
            nvim_buf_update_channels_destroy(buf);
        }
    }
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

    #[test]
    fn test_lua_internal_call_value() {
        // LUA_INTERNAL_CALL = (1u64 << 63) + 1
        assert_eq!(LUA_INTERNAL_CALL, 9_223_372_036_854_775_809u64);
    }

    #[test]
    fn test_lua_noref_value() {
        assert_eq!(LUA_NOREF, -2);
    }
}
