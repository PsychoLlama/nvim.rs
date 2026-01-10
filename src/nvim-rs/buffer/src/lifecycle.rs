//! Buffer lifecycle management helpers
//!
//! This module provides Rust implementations for buffer lifecycle operations
//! including creation validation, cleanup preparation, and state transitions.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]
#![allow(dead_code)]

use std::ffi::{c_char, c_int, c_void};

use crate::BufHandle;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_get_firstbuf() -> BufHandle;
    fn nvim_get_lastbuf() -> BufHandle;
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_buf_get_prev(buf: BufHandle) -> BufHandle;
    fn nvim_buf_get_next(buf: BufHandle) -> BufHandle;
    fn nvim_buf_get_fnum(buf: BufHandle) -> c_int;
    fn nvim_buf_get_ffname(buf: BufHandle) -> *const c_char;
    fn nvim_buf_get_nwindows(buf: BufHandle) -> c_int;
    fn nvim_buf_get_locked(buf: BufHandle) -> c_int;
    fn nvim_buf_get_locked_split(buf: BufHandle) -> c_int;
    fn nvim_buf_get_flags(buf: BufHandle) -> c_int;
    fn nvim_buf_get_terminal(buf: BufHandle) -> c_int;
    fn nvim_buf_get_bufhidden(buf: BufHandle) -> c_char;
    fn nvim_buf_get_changed(buf: BufHandle) -> c_int;
    fn nvim_buf_get_ml_mfp(buf: BufHandle) -> *mut c_void;
}

// =============================================================================
// Buffer Flags (from buffer_defs.h)
// =============================================================================

/// Buffer flags matching `buffer_defs.h`
pub mod buf_flags {
    use std::ffi::c_int;

    /// Buffer needs read-only check
    pub const BF_CHECK_RO: c_int = 0x01;
    /// Buffer was never loaded
    pub const BF_NEVERLOADED: c_int = 0x02;
    /// Buffer has read error
    pub const BF_READERR: c_int = 0x04;
    /// Preserve buffer (no reclaim)
    pub const BF_PRESERVED: c_int = 0x08;
    /// Dummy buffer for internal use
    pub const BF_DUMMY: c_int = 0x8000;
}

/// Buffer action types for `close_buffer`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BufferAction {
    /// Don't unload the buffer
    #[default]
    None = 0,
    /// Unload the buffer
    Unload = 1,
    /// Delete the buffer
    Delete = 2,
    /// Wipe the buffer completely
    Wipe = 3,
}

impl BufferAction {
    /// Create from raw integer (matching `DOBUF_*` constants)
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Unload,
            2 => Self::Delete,
            3 => Self::Wipe,
            _ => Self::None,
        }
    }

    /// Convert to raw integer
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this action unloads the buffer
    #[must_use]
    pub const fn unloads(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Check if this action deletes the buffer
    #[must_use]
    pub const fn deletes(self) -> bool {
        matches!(self, Self::Delete | Self::Wipe)
    }

    /// Check if this action wipes the buffer
    #[must_use]
    pub const fn wipes(self) -> bool {
        matches!(self, Self::Wipe)
    }
}

// =============================================================================
// Buffer Lifecycle State
// =============================================================================

/// Current lifecycle state of a buffer
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LifecycleState {
    /// Buffer has never been loaded (`BF_NEVERLOADED` set)
    #[default]
    NeverLoaded = 0,
    /// Buffer is not loaded (`ml_mfp` is NULL)
    NotLoaded = 1,
    /// Buffer is loaded but not displayed (`b_nwindows` == 0)
    Hidden = 2,
    /// Buffer is loaded and displayed in a window
    Normal = 3,
}

impl LifecycleState {
    /// Check if buffer is loaded (in memory)
    #[must_use]
    pub const fn is_loaded(self) -> bool {
        matches!(self, Self::Hidden | Self::Normal)
    }

    /// Check if buffer is visible in a window
    #[must_use]
    pub const fn is_visible(self) -> bool {
        matches!(self, Self::Normal)
    }

    /// Convert to raw integer
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Create from raw integer
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::NotLoaded,
            2 => Self::Hidden,
            3 => Self::Normal,
            _ => Self::NeverLoaded,
        }
    }
}

/// Get the lifecycle state of a buffer.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn get_lifecycle_state(buf: BufHandle) -> LifecycleState {
    if buf.is_null() {
        return LifecycleState::NeverLoaded;
    }

    let flags = nvim_buf_get_flags(buf);
    if (flags & buf_flags::BF_NEVERLOADED) != 0 {
        return LifecycleState::NeverLoaded;
    }

    let ml_mfp = nvim_buf_get_ml_mfp(buf);
    if ml_mfp.is_null() {
        return LifecycleState::NotLoaded;
    }

    let nwindows = nvim_buf_get_nwindows(buf);
    if nwindows == 0 {
        LifecycleState::Hidden
    } else {
        LifecycleState::Normal
    }
}

// =============================================================================
// Buffer Close Preparation
// =============================================================================

/// Result of checking if a buffer can be unloaded
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct UnloadCheck {
    /// Whether the buffer can be unloaded
    pub can_unload: bool,
    /// Whether the buffer is locked
    pub is_locked: bool,
    /// Whether the buffer is locked for split
    pub is_locked_split: bool,
    /// Whether the buffer is in use by a terminal
    pub has_terminal: bool,
    /// Number of windows displaying this buffer
    pub nwindows: c_int,
}

/// Check if a buffer can be unloaded.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn check_can_unload(buf: BufHandle) -> UnloadCheck {
    if buf.is_null() {
        return UnloadCheck::default();
    }

    let is_locked = nvim_buf_get_locked(buf) > 0;
    let is_locked_split = nvim_buf_get_locked_split(buf) > 0;
    let has_terminal = nvim_buf_get_terminal(buf) != 0;
    let nwindows = nvim_buf_get_nwindows(buf);

    // Buffer can be unloaded if not locked
    let can_unload = !is_locked;

    UnloadCheck {
        can_unload,
        is_locked,
        is_locked_split,
        has_terminal,
        nwindows,
    }
}

/// Determine the effective action for closing a buffer based on bufhidden.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn effective_close_action(buf: BufHandle, requested: BufferAction) -> BufferAction {
    if buf.is_null() {
        return requested;
    }

    // Terminal buffers can only be wiped
    if nvim_buf_get_terminal(buf) != 0
        && (requested.unloads() || requested.deletes() || requested.wipes())
    {
        return BufferAction::Wipe;
    }

    // Check bufhidden option
    let bh = nvim_buf_get_bufhidden(buf) as u8;
    match bh {
        b'd' => {
            // bufhidden=delete
            BufferAction::Delete
        }
        b'w' => {
            // bufhidden=wipe
            BufferAction::Wipe
        }
        b'u' => {
            // bufhidden=unload
            if requested.deletes() || requested.wipes() {
                requested
            } else {
                BufferAction::Unload
            }
        }
        _ => requested,
    }
}

// =============================================================================
// Buffer List Operations
// =============================================================================

/// Check if buffer has a file name.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn has_filename(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    !nvim_buf_get_ffname(buf).is_null()
}

/// Check if buffer is modified.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn is_modified(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    nvim_buf_get_changed(buf) != 0
}

/// Check if buffer is a dummy buffer.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn is_dummy(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    (nvim_buf_get_flags(buf) & buf_flags::BF_DUMMY) != 0
}

/// Check if buffer was never loaded.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn is_never_loaded(buf: BufHandle) -> bool {
    if buf.is_null() {
        return true;
    }
    (nvim_buf_get_flags(buf) & buf_flags::BF_NEVERLOADED) != 0
}

/// Check if buffer is last in window (nwindows == 1).
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn is_last_in_window(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    nvim_buf_get_nwindows(buf) == 1
}

/// Check if this is the current buffer.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn is_curbuf(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    buf == nvim_get_curbuf()
}

// =============================================================================
// Buffer Position in List
// =============================================================================

/// Information about a buffer's position for lifecycle operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LifecyclePosition {
    /// File number
    pub fnum: c_int,
    /// Lifecycle state
    pub state: LifecycleState,
    /// Number of windows
    pub nwindows: c_int,
    /// Whether buffer has filename
    pub has_filename: bool,
    /// Whether buffer is modified
    pub is_modified: bool,
    /// Whether buffer is current
    pub is_current: bool,
    /// Whether buffer is dummy
    pub is_dummy: bool,
}

/// Get lifecycle position info for a buffer.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn get_lifecycle_position(buf: BufHandle) -> LifecyclePosition {
    if buf.is_null() {
        return LifecyclePosition::default();
    }

    LifecyclePosition {
        fnum: nvim_buf_get_fnum(buf),
        state: get_lifecycle_state(buf),
        nwindows: nvim_buf_get_nwindows(buf),
        has_filename: has_filename(buf),
        is_modified: is_modified(buf),
        is_current: is_curbuf(buf),
        is_dummy: is_dummy(buf),
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get buffer lifecycle state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_lifecycle_state(buf: BufHandle) -> c_int {
    get_lifecycle_state(buf).to_raw()
}

/// Check if buffer can be unloaded.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_can_unload(buf: BufHandle) -> c_int {
    c_int::from(check_can_unload(buf).can_unload)
}

/// Get effective close action for buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_effective_action(buf: BufHandle, action: c_int) -> c_int {
    effective_close_action(buf, BufferAction::from_raw(action)).to_raw()
}

/// Check if buffer has filename.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_has_filename(buf: BufHandle) -> c_int {
    c_int::from(has_filename(buf))
}

/// Check if buffer is modified.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_is_modified(buf: BufHandle) -> c_int {
    c_int::from(is_modified(buf))
}

/// Check if buffer is dummy.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_is_dummy(buf: BufHandle) -> c_int {
    c_int::from(is_dummy(buf))
}

/// Check if buffer was never loaded.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_is_never_loaded(buf: BufHandle) -> c_int {
    c_int::from(is_never_loaded(buf))
}

/// Check if buffer is last in window.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_is_last_in_window(buf: BufHandle) -> c_int {
    c_int::from(is_last_in_window(buf))
}

/// Check if buffer is curbuf.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_is_curbuf(buf: BufHandle) -> c_int {
    c_int::from(is_curbuf(buf))
}

/// Get buffer nwindows count.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_nwindows(buf: BufHandle) -> c_int {
    if buf.is_null() {
        return 0;
    }
    nvim_buf_get_nwindows(buf)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_action() {
        assert!(!BufferAction::None.unloads());
        assert!(BufferAction::Unload.unloads());
        assert!(BufferAction::Delete.unloads());
        assert!(BufferAction::Wipe.unloads());

        assert!(!BufferAction::None.deletes());
        assert!(!BufferAction::Unload.deletes());
        assert!(BufferAction::Delete.deletes());
        assert!(BufferAction::Wipe.deletes());

        assert!(!BufferAction::None.wipes());
        assert!(!BufferAction::Unload.wipes());
        assert!(!BufferAction::Delete.wipes());
        assert!(BufferAction::Wipe.wipes());
    }

    #[test]
    fn test_lifecycle_state() {
        assert!(!LifecycleState::NeverLoaded.is_loaded());
        assert!(!LifecycleState::NotLoaded.is_loaded());
        assert!(LifecycleState::Hidden.is_loaded());
        assert!(LifecycleState::Normal.is_loaded());

        assert!(!LifecycleState::NeverLoaded.is_visible());
        assert!(!LifecycleState::NotLoaded.is_visible());
        assert!(!LifecycleState::Hidden.is_visible());
        assert!(LifecycleState::Normal.is_visible());
    }

    #[test]
    fn test_buffer_action_roundtrip() {
        for i in 0..4 {
            let action = BufferAction::from_raw(i);
            assert_eq!(action.to_raw(), i);
        }
    }

    #[test]
    fn test_lifecycle_state_roundtrip() {
        for i in 0..4 {
            let state = LifecycleState::from_raw(i);
            assert_eq!(state.to_raw(), i);
        }
    }

    #[test]
    fn test_unload_check_default() {
        let check = UnloadCheck::default();
        assert!(!check.can_unload);
        assert!(!check.is_locked);
        assert!(!check.is_locked_split);
        assert!(!check.has_terminal);
        assert_eq!(check.nwindows, 0);
    }
}
