//! Buffer state and modification tracking helpers
//!
//! This module provides helpers for tracking buffer modification state,
//! changed ticks, and buffer flags.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::doc_markdown)]

use std::ffi::c_int;

use crate::BufHandle;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_buf_get_changedtick(buf: BufHandle) -> c_int;
    fn nvim_buf_get_ml_line_count(buf: BufHandle) -> c_int;
    fn nvim_buf_get_fnum(buf: BufHandle) -> c_int;
}

// =============================================================================
// Buffer Flags
// =============================================================================

/// Buffer state flags (from buffer_defs.h)
pub mod buf_flags {
    use std::ffi::c_int;

    /// Buffer was unloaded (detached from memory)
    pub const BF_UNLOADED: c_int = 0x01;
    /// Buffer was :wnew'd
    pub const BF_NEW: c_int = 0x02;
    /// User doesn't want to write buffer
    pub const BF_READERR: c_int = 0x04;
    /// Buffer has been preserved
    pub const BF_PRESERVED: c_int = 0x08;
    /// Auto-command for this buffer has been applied
    pub const BF_SYN_SET: c_int = 0x10;
    /// Window was wiped out (buffer still exists)
    pub const BF_WIPED: c_int = 0x20;
    /// No undo/redo for this buffer
    pub const BF_NOTEDITED: c_int = 0x40;
    /// Dummy buffer used for preview, etc.
    pub const BF_DUMMY: c_int = 0x80;
}

/// Check if a buffer has a specific flag set.
#[must_use]
#[inline]
pub fn has_buf_flag(flags: c_int, flag: c_int) -> bool {
    (flags & flag) != 0
}

/// Set a buffer flag.
#[must_use]
#[inline]
pub const fn set_buf_flag(flags: c_int, flag: c_int) -> c_int {
    flags | flag
}

/// Clear a buffer flag.
#[must_use]
#[inline]
pub const fn clear_buf_flag(flags: c_int, flag: c_int) -> c_int {
    flags & !flag
}

// =============================================================================
// Buffer State Information
// =============================================================================

/// Complete state information for a buffer.
///
/// Note: This is a simplified version that only includes fields
/// accessible via available C accessor functions.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BufState {
    /// File number (unique buffer identifier)
    pub fnum: c_int,
    /// Change tick counter
    pub changedtick: c_int,
    /// Number of lines
    pub line_count: c_int,
}

impl BufState {
    /// Check if buffer is empty (one or zero lines).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.line_count <= 1
    }

    /// Check if buffer has valid content.
    #[must_use]
    pub const fn has_content(&self) -> bool {
        self.line_count > 1
    }
}

/// Get complete state information for a buffer.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn get_buf_state(buf: BufHandle) -> BufState {
    if buf.is_null() {
        return BufState::default();
    }

    BufState {
        fnum: nvim_buf_get_fnum(buf),
        changedtick: nvim_buf_get_changedtick(buf),
        line_count: nvim_buf_get_ml_line_count(buf),
    }
}

// =============================================================================
// Change Tick Tracking
// =============================================================================

/// Saved changedtick for detecting modifications.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChangedTickRef {
    /// Saved changedtick value
    pub tick: c_int,
    /// Buffer fnum when tick was saved
    pub fnum: c_int,
}

impl ChangedTickRef {
    /// Create a new changedtick reference.
    #[must_use]
    pub const fn new(tick: c_int, fnum: c_int) -> Self {
        Self { tick, fnum }
    }

    /// Create from a buffer.
    ///
    /// # Safety
    ///
    /// Calls external C functions.
    #[must_use]
    pub unsafe fn from_buf(buf: BufHandle) -> Self {
        if buf.is_null() {
            return Self::default();
        }
        Self {
            tick: nvim_buf_get_changedtick(buf),
            fnum: nvim_buf_get_fnum(buf),
        }
    }

    /// Check if buffer has changed since this reference was created.
    ///
    /// # Safety
    ///
    /// Calls external C functions.
    #[must_use]
    pub unsafe fn has_changed(&self, buf: BufHandle) -> bool {
        if buf.is_null() {
            return true; // Buffer is gone, consider changed
        }

        let buf_fnum = nvim_buf_get_fnum(buf);
        if buf_fnum != self.fnum {
            return true; // Different buffer
        }

        let current_tick = nvim_buf_get_changedtick(buf);
        current_tick != self.tick
    }

    /// Update this reference to the current buffer state.
    ///
    /// # Safety
    ///
    /// Calls external C functions.
    pub unsafe fn update(&mut self, buf: BufHandle) {
        if buf.is_null() {
            *self = Self::default();
        } else {
            self.tick = nvim_buf_get_changedtick(buf);
            self.fnum = nvim_buf_get_fnum(buf);
        }
    }
}

// =============================================================================
// Modification State
// =============================================================================

/// Result of checking buffer modification.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifiedState {
    /// Buffer is not modified
    Unmodified = 0,
    /// Buffer is modified
    Modified = 1,
    /// Buffer is modified and unsaveable (special buftype)
    ModifiedUnsaveable = 2,
}

impl ModifiedState {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            0 => Self::Unmodified,
            1 => Self::Modified,
            _ => Self::ModifiedUnsaveable,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if modified in any way.
    #[must_use]
    pub const fn is_modified(self) -> bool {
        !matches!(self, Self::Unmodified)
    }

    /// Check if saveable (has a filename).
    #[must_use]
    pub const fn is_saveable(self) -> bool {
        !matches!(self, Self::ModifiedUnsaveable)
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if a flags value has a specific flag set.
#[unsafe(no_mangle)]
pub extern "C" fn rs_buf_flags_has(flags: c_int, flag: c_int) -> c_int {
    c_int::from(has_buf_flag(flags, flag))
}

/// Set a flag in a flags value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_buf_flags_set(flags: c_int, flag: c_int) -> c_int {
    set_buf_flag(flags, flag)
}

/// Clear a flag in a flags value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_buf_flags_clear(flags: c_int, flag: c_int) -> c_int {
    clear_buf_flag(flags, flag)
}

/// Get buffer changedtick.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_get_changedtick(buf: BufHandle) -> c_int {
    if buf.is_null() {
        return 0;
    }
    nvim_buf_get_changedtick(buf)
}

/// Get buffer line count.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_get_line_count(buf: BufHandle) -> c_int {
    if buf.is_null() {
        return 0;
    }
    nvim_buf_get_ml_line_count(buf)
}

/// Check if buffer is empty.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_is_empty(buf: BufHandle) -> c_int {
    if buf.is_null() {
        return 1;
    }
    c_int::from(nvim_buf_get_ml_line_count(buf) <= 1)
}

/// Check if changedtick has changed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_changedtick_changed(
    buf: BufHandle,
    saved_tick: c_int,
    saved_fnum: c_int,
) -> c_int {
    let ref_ = ChangedTickRef::new(saved_tick, saved_fnum);
    c_int::from(ref_.has_changed(buf))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buf_flags() {
        let flags = buf_flags::BF_NEW | buf_flags::BF_DUMMY;

        assert!(has_buf_flag(flags, buf_flags::BF_NEW));
        assert!(has_buf_flag(flags, buf_flags::BF_DUMMY));
        assert!(!has_buf_flag(flags, buf_flags::BF_UNLOADED));

        let flags = set_buf_flag(flags, buf_flags::BF_UNLOADED);
        assert!(has_buf_flag(flags, buf_flags::BF_UNLOADED));

        let flags = clear_buf_flag(flags, buf_flags::BF_NEW);
        assert!(!has_buf_flag(flags, buf_flags::BF_NEW));
        assert!(has_buf_flag(flags, buf_flags::BF_DUMMY));
    }

    #[test]
    fn test_buf_state() {
        let state = BufState {
            fnum: 1,
            changedtick: 5,
            line_count: 100,
        };

        assert!(!state.is_empty());
        assert!(state.has_content());

        let empty_state = BufState {
            fnum: 2,
            changedtick: 1,
            line_count: 1,
        };

        assert!(empty_state.is_empty());
        assert!(!empty_state.has_content());
    }

    #[test]
    fn test_changedtick_ref() {
        let ref1 = ChangedTickRef::new(5, 1);
        let ref2 = ChangedTickRef::new(5, 1);
        let ref3 = ChangedTickRef::new(6, 1);

        assert_eq!(ref1, ref2);
        assert_ne!(ref1, ref3);
    }

    #[test]
    fn test_modified_state() {
        assert!(!ModifiedState::Unmodified.is_modified());
        assert!(ModifiedState::Modified.is_modified());
        assert!(ModifiedState::ModifiedUnsaveable.is_modified());

        assert!(ModifiedState::Unmodified.is_saveable());
        assert!(ModifiedState::Modified.is_saveable());
        assert!(!ModifiedState::ModifiedUnsaveable.is_saveable());
    }
}
