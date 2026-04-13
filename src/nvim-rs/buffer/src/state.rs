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

use std::ffi::{c_char, c_int};
use std::sync::atomic::{AtomicI32, Ordering};

use crate::{
    buf_struct::{buf_mut, buf_ref},
    BufHandle,
};

// =============================================================================
// Global counters (migrated from C static variables in buffer.c)
// =============================================================================

/// Number of times `free_buffer()` was called.
/// Corresponds to the C static `buf_free_count` in `buffer.c`.
static BUF_FREE_COUNT: AtomicI32 = AtomicI32::new(0);

/// Highest file number assigned to a buffer.
/// Corresponds to the C static `top_file_num` in `buffer.c`.
static TOP_FILE_NUM: AtomicI32 = AtomicI32::new(1);

/// Get the `buf_free_count` global.
///
/// Called from Rust code that needs to validate buffer references.
#[inline]
#[must_use]
pub fn get_buf_free_count() -> c_int {
    BUF_FREE_COUNT.load(Ordering::Relaxed)
}

/// Increment `buf_free_count`.
///
/// Called when a buffer is freed.
#[inline]
pub fn inc_buf_free_count() {
    BUF_FREE_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Get the `top_file_num` global.
#[inline]
#[must_use]
pub fn get_top_file_num() -> c_int {
    TOP_FILE_NUM.load(Ordering::Relaxed)
}

/// Increment `top_file_num` and return the old value (the number to assign).
#[inline]
#[must_use]
pub fn inc_top_file_num() -> c_int {
    TOP_FILE_NUM.fetch_add(1, Ordering::Relaxed)
}

/// Set `top_file_num` to the given value.
#[inline]
pub fn set_top_file_num(val: c_int) {
    TOP_FILE_NUM.store(val, Ordering::Relaxed);
}

/// Reset `top_file_num` to 1.
#[inline]
pub fn reset_top_file_num() {
    TOP_FILE_NUM.store(1, Ordering::Relaxed);
}

// ---------------------------------------------------------------------------
// C-ABI exports for the above globals (used by C code in buffer_shim.c etc.)
// ---------------------------------------------------------------------------

/// C accessor: `nvim_get_buf_free_count`.
#[unsafe(no_mangle)]
pub extern "C" fn nvim_get_buf_free_count() -> c_int {
    get_buf_free_count()
}

/// C mutator: `nvim_inc_buf_free_count`.
#[unsafe(no_mangle)]
pub extern "C" fn nvim_inc_buf_free_count() {
    inc_buf_free_count();
}

/// C accessor: `nvim_get_top_file_num`.
#[unsafe(no_mangle)]
pub extern "C" fn nvim_get_top_file_num() -> c_int {
    get_top_file_num()
}

/// C mutator: `nvim_inc_top_file_num` — increments and returns the old value.
#[unsafe(no_mangle)]
pub extern "C" fn nvim_inc_top_file_num() -> c_int {
    inc_top_file_num()
}

/// C mutator: `nvim_set_top_file_num`.
#[unsafe(no_mangle)]
pub extern "C" fn nvim_set_top_file_num(val: c_int) {
    set_top_file_num(val);
}

/// C mutator: `nvim_reset_top_file_num`.
#[unsafe(no_mangle)]
pub extern "C" fn nvim_reset_top_file_num() {
    reset_top_file_num();
}

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_buf_get_changedtick(buf: BufHandle) -> c_int;
    fn ml_get_buf(buf: BufHandle, lnum: c_int) -> *const c_char;

    // Phase 1: unchanged, changedtick, autocmd, close_buffer
    fn unchanged(buf: BufHandle, ff: bool, always_inc_changedtick: bool);
    fn nvim_buf_get_changedtick_direct(buf: BufHandle) -> i64;
    fn block_autocmds();
    fn unblock_autocmds();
    fn close_buffer(
        win: crate::WinHandle,
        buf: BufHandle,
        action: c_int,
        abort_if_last: bool,
        ignore_abort: bool,
    ) -> bool;

    // Phase 2: changedtick_di watcher machinery
    fn nvim_buf_changedtick_di_tv_copy(buf: BufHandle, out: *mut u8);
    fn nvim_buf_changedtick_di_set_number(buf: BufHandle, val: i64);
    fn nvim_tv_dict_is_watched(dict: *const std::ffi::c_void) -> bool;
    fn nvim_tv_dict_watcher_notify(
        dict: *mut std::ffi::c_void,
        key: *const c_char,
        newtv: *mut u8,
        oldtv: *mut u8,
    );
    fn nvim_buf_changedtick_di_key(buf: BufHandle) -> *const c_char;
    fn nvim_buf_changedtick_di_tv_ptr(buf: BufHandle) -> *mut u8;
    fn nvim_buf_b_locked_inc(buf: BufHandle);
    fn nvim_buf_b_locked_dec(buf: BufHandle);
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
        fnum: buf_ref(buf).handle,
        changedtick: nvim_buf_get_changedtick(buf),
        line_count: buf_ref(buf).ml_line_count,
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
            fnum: buf_ref(buf).handle,
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

        let buf_fnum = buf_ref(buf).handle;
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
            self.fnum = buf_ref(buf).handle;
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
    buf_ref(buf).ml_line_count
}

/// Check if buffer is empty (line count == 1 and first line is empty).
///
/// This matches the C `buf_is_empty()` semantics exactly.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn buf_is_empty(buf: BufHandle) -> bool {
    if buf.is_null() {
        return true;
    }
    buf_ref(buf).ml_line_count == 1 && *ml_get_buf(buf, 1) == 0
}

/// FFI wrapper for `buf_is_empty`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_is_empty(buf: BufHandle) -> bool {
    buf_is_empty(buf)
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
// Buffer State Management (Phase 1: Wave 2)
// =============================================================================

/// ML_EMPTY flag from memline_defs.h — empty buffer.
const ML_EMPTY: c_int = 0x01;

/// DOBUF_WIPE action for close_buffer — wipe buffer completely.
const DOBUF_WIPE: c_int = 4;

/// Reset buffer file state (make buffer not contain a file).
///
/// Sets line count to 1, marks as unchanged, clears eol/eof/bomb flags,
/// nulls the memfile pointer, and sets ml_flags to ML_EMPTY.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_clear_file(buf: BufHandle) {
    if buf.is_null() {
        return;
    }
    buf_mut(buf).ml_line_count = 1;
    unchanged(buf, true, true);
    buf_mut(buf).b_p_eof = 0;
    buf_mut(buf).b_start_eof = 0;
    buf_mut(buf).b_p_eol = 1;
    buf_mut(buf).b_start_eol = 1;
    buf_mut(buf).b_p_bomb = 0;
    buf_mut(buf).b_start_bomb = 0;
    buf_mut(buf).ml_mfp = std::ptr::null_mut();
    buf_mut(buf).ml_flags = ML_EMPTY;
}

/// Size of typval_T in bytes (asserted in testing.c).
const TYPVAL_SIZE: usize = 16;

/// Set `b:changedtick`, triggering dict watcher notification if watched.
///
/// Mirrors C `buf_set_changedtick` / `nvim_buf_set_changedtick_compound`.
/// Omits the debug-only asserts (`#ifndef NDEBUG` block) from the C version.
///
/// # Safety
/// Must be called on the main thread with valid Neovim state.
#[unsafe(export_name = "buf_set_changedtick")]
pub unsafe extern "C" fn rs_buf_set_changedtick(buf: BufHandle, changedtick: i64) {
    if buf.is_null() {
        return;
    }
    // Copy old typval before overwriting.
    let mut old_val = [0u8; TYPVAL_SIZE];
    nvim_buf_changedtick_di_tv_copy(buf, old_val.as_mut_ptr());

    // Set new value.
    nvim_buf_changedtick_di_set_number(buf, changedtick);

    // Notify dict watchers if any.
    let b_vars = buf_ref(buf).b_vars;
    if nvim_tv_dict_is_watched(b_vars.cast_const()) {
        // Increment b_locked around the notify to match C semantics.
        nvim_buf_b_locked_inc(buf);
        let newtv = nvim_buf_changedtick_di_tv_ptr(buf);
        nvim_tv_dict_watcher_notify(
            b_vars,
            nvim_buf_changedtick_di_key(buf),
            newtv,
            old_val.as_mut_ptr(),
        );
        nvim_buf_b_locked_dec(buf);
    }
}

/// Increment `b:changedtick` value.
///
/// Delegates to `buf_set_changedtick(buf, buf_get_changedtick(buf) + 1)`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_inc_changedtick(buf: BufHandle) {
    if buf.is_null() {
        return;
    }
    let tick = nvim_buf_get_changedtick_direct(buf);
    rs_buf_set_changedtick(buf, tick + 1);
}

/// Wipe out a buffer, optionally blocking autocommands.
///
/// Calls `close_buffer(NULL, buf, DOBUF_WIPE, false, true)`.
/// If `aucmd` is false, blocks autocommands around the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_wipe_buffer(buf: BufHandle, aucmd: bool) {
    if buf.is_null() {
        return;
    }
    if !aucmd {
        block_autocmds();
    }
    close_buffer(
        crate::WinHandle(std::ptr::null_mut()),
        buf,
        DOBUF_WIPE,
        false,
        true,
    );
    if !aucmd {
        unblock_autocmds();
    }
}

// =============================================================================
// C-named symbol exports (for Rust crates that call by canonical C name)
// =============================================================================

/// C export: `buf_clear_file`.
#[unsafe(export_name = "buf_clear_file")]
pub unsafe extern "C" fn buf_clear_file_export(buf: BufHandle) {
    rs_buf_clear_file(buf);
}

/// C export: `wipe_buffer`.
#[unsafe(export_name = "wipe_buffer")]
pub unsafe extern "C" fn wipe_buffer_export(buf: BufHandle, aucmd: bool) {
    rs_wipe_buffer(buf, aucmd);
}

/// C export: `buf_is_empty`.
#[must_use]
#[unsafe(export_name = "buf_is_empty")]
pub unsafe extern "C" fn buf_is_empty_export(buf: BufHandle) -> bool {
    buf_is_empty(buf)
}

/// C export: `buf_inc_changedtick`.
#[unsafe(export_name = "buf_inc_changedtick")]
pub unsafe extern "C" fn buf_inc_changedtick_export(buf: BufHandle) {
    rs_buf_inc_changedtick(buf);
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
