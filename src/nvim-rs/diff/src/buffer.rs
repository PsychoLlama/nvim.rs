//! Diff buffer state and synchronization
//!
//! This module provides Rust implementations for diff buffer management,
//! including buffer registration, diff state tracking, and buffer synchronization.

#![allow(clippy::must_use_candidate)]

use std::ffi::c_void;
use std::os::raw::c_int;

/// Line number type matching linenr_T (i32).
type LinenrT = i32;

/// Result constants matching Neovim's OK/FAIL.
#[allow(dead_code)]
const OK: c_int = 1;
#[allow(dead_code)]
const FAIL: c_int = 0;

/// Maximum number of diff buffers.
pub const DB_COUNT: c_int = 8;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to a diff block (diff_T).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiffBlockHandle(*mut c_void);

impl DiffBlockHandle {
    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle.
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Get the raw pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Opaque handle to a buffer (buf_T).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufHandle(*mut c_void);

impl BufHandle {
    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle.
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Get the raw pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Opaque handle to a tabpage (tabpage_T).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabpageHandle(*mut c_void);

impl TabpageHandle {
    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle.
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

/// Opaque handle to a window (win_T).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WinHandle(*mut c_void);

impl WinHandle {
    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle.
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

// =============================================================================
// External C Functions
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Tabpage accessors
    fn nvim_get_curtab() -> TabpageHandle;
    fn nvim_get_curtab_diffbuf(idx: c_int) -> BufHandle;
    fn nvim_get_curtab_diff_invalid() -> c_int;
    fn nvim_tabpage_get_diffbuf(tp: TabpageHandle, idx: c_int) -> BufHandle;
    fn nvim_tabpage_get_diff_invalid(tp: TabpageHandle) -> c_int;
    fn nvim_tabpage_set_diff_invalid(tp: TabpageHandle, val: c_int);

    // Diff block accessors
    fn nvim_get_diff_first_block() -> DiffBlockHandle;
    fn nvim_tabpage_get_first_diff(tp: TabpageHandle) -> DiffBlockHandle;
    fn nvim_diffblock_get_next(dp: DiffBlockHandle) -> DiffBlockHandle;
    fn nvim_diffblock_get_lnum(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_get_count(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_set_lnum(dp: DiffBlockHandle, idx: c_int, lnum: LinenrT);
    fn nvim_diffblock_set_count(dp: DiffBlockHandle, idx: c_int, count: LinenrT);

    // Buffer accessors
    fn nvim_buf_get_ml_line_count(buf: BufHandle) -> LinenrT;

    // Tab iteration
    fn nvim_get_first_tabpage() -> TabpageHandle;
    fn nvim_tabpage_get_next(tp: TabpageHandle) -> TabpageHandle;
}

// =============================================================================
// Diff Buffer State
// =============================================================================

/// Diff buffer state for tracking buffer membership in diff mode.
#[derive(Debug, Clone, Copy)]
pub struct DiffBufferState {
    /// Index of the buffer in the diff list (-1 if not found).
    pub idx: c_int,
    /// Whether the diff is invalid and needs update.
    pub invalid: bool,
    /// Number of active diff buffers.
    pub buffer_count: c_int,
}

impl DiffBufferState {
    /// Create a new state indicating buffer not in diff.
    #[must_use]
    pub const fn not_in_diff() -> Self {
        Self {
            idx: -1,
            invalid: false,
            buffer_count: 0,
        }
    }
}

/// Find the index of a buffer in the current tab's diff list.
///
/// Returns the buffer index (0 to DB_COUNT-1) or -1 if not found.
pub fn diff_buf_idx(buf: BufHandle) -> c_int {
    if buf.is_null() {
        return -1;
    }

    unsafe {
        for i in 0..DB_COUNT {
            let diffbuf = nvim_get_curtab_diffbuf(i);
            if !diffbuf.is_null() && diffbuf.0 == buf.0 {
                return i;
            }
        }
        -1
    }
}

/// Find the index of a buffer in a specific tab's diff list.
///
/// Returns the buffer index (0 to DB_COUNT-1) or DB_COUNT if not found.
///
/// Note: This function requires nvim_tabpage_get_diffbuf C accessor which doesn't exist yet.
#[allow(dead_code)]
#[cfg(feature = "tabpage_diff")]
pub fn diff_buf_idx_tp(buf: BufHandle, tp: TabpageHandle) -> c_int {
    if buf.is_null() || tp.is_null() {
        return DB_COUNT;
    }

    unsafe {
        for i in 0..DB_COUNT {
            let diffbuf = nvim_tabpage_get_diffbuf(tp, i);
            if !diffbuf.is_null() && diffbuf.0 == buf.0 {
                return i;
            }
        }
        DB_COUNT
    }
}

/// Check if the diff list is invalid (needs update).
pub fn diff_check_invalid() -> bool {
    unsafe { nvim_get_curtab_diff_invalid() != 0 }
}

/// Count the number of active diff buffers in the current tab.
pub fn diff_count_buffers() -> c_int {
    unsafe {
        let mut count = 0;
        for i in 0..DB_COUNT {
            if !nvim_get_curtab_diffbuf(i).is_null() {
                count += 1;
            }
        }
        count
    }
}

/// Check if a buffer is in diff mode in the current tab.
pub fn diff_buf_is_diffed(buf: BufHandle) -> bool {
    diff_buf_idx(buf) >= 0
}

/// Check if a buffer is in diff mode in any tab.
///
/// Note: This function requires nvim_tabpage_get_diffbuf C accessor which doesn't exist yet.
#[allow(dead_code)]
#[cfg(feature = "tabpage_diff")]
pub fn diff_mode_buf(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }

    unsafe {
        let mut tp = nvim_get_first_tabpage();
        while !tp.is_null() {
            if diff_buf_idx_tp(buf, tp) != DB_COUNT {
                return true;
            }
            tp = nvim_tabpage_get_next(tp);
        }
        false
    }
}

/// Get the full diff buffer state for a buffer.
pub fn get_diff_buffer_state(buf: BufHandle) -> DiffBufferState {
    let idx = diff_buf_idx(buf);
    DiffBufferState {
        idx,
        invalid: diff_check_invalid(),
        buffer_count: diff_count_buffers(),
    }
}

// =============================================================================
// Diff Block Queries
// =============================================================================

/// Find the diff block that contains a given line number.
///
/// Returns the diff block handle or null if not found.
pub fn diff_find_block_for_line(buf_idx: c_int, lnum: LinenrT) -> DiffBlockHandle {
    if !(0..DB_COUNT).contains(&buf_idx) {
        return DiffBlockHandle::null();
    }

    unsafe {
        let mut dp = nvim_get_diff_first_block();
        while !dp.is_null() {
            let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
            let block_count = nvim_diffblock_get_count(dp, buf_idx);

            // Check if lnum is within this block
            if lnum >= block_lnum && lnum < block_lnum + block_count.max(1) {
                return dp;
            }

            // If we've passed the line, stop searching
            if block_lnum > lnum {
                break;
            }

            dp = nvim_diffblock_get_next(dp);
        }
        DiffBlockHandle::null()
    }
}

/// Calculate the number of filler lines at a given line.
///
/// Filler lines are displayed to align diff blocks between buffers.
pub fn diff_get_filler_lines(buf_idx: c_int, lnum: LinenrT) -> c_int {
    if !(0..DB_COUNT).contains(&buf_idx) {
        return 0;
    }

    unsafe {
        let mut dp = nvim_get_diff_first_block();
        while !dp.is_null() {
            let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
            let block_count = nvim_diffblock_get_count(dp, buf_idx);

            // Filler lines appear above the diff block
            if lnum == block_lnum && block_count == 0 {
                // This is a pure insertion in other buffer(s)
                // Count max lines in other buffers
                let mut max_count = 0;
                for i in 0..DB_COUNT {
                    if i != buf_idx && !nvim_get_curtab_diffbuf(i).is_null() {
                        let count = nvim_diffblock_get_count(dp, i);
                        max_count = max_count.max(count);
                    }
                }
                return max_count;
            }

            // If we've passed the line, stop searching
            if block_lnum > lnum {
                break;
            }

            dp = nvim_diffblock_get_next(dp);
        }
        0
    }
}

/// Get the maximum diff block length across all buffers.
pub fn get_max_diff_length(dp: DiffBlockHandle) -> c_int {
    if dp.is_null() {
        return 0;
    }

    unsafe {
        let mut max_length = 0;
        for i in 0..DB_COUNT {
            if !nvim_get_curtab_diffbuf(i).is_null() {
                let count = nvim_diffblock_get_count(dp, i);
                if count > max_length {
                    max_length = count;
                }
            }
        }
        max_length
    }
}

/// Check if two entries in a diff block are equal.
///
/// This compares the line counts to determine if the entries match.
pub fn diff_equal_entry(dp: DiffBlockHandle, idx1: c_int, idx2: c_int) -> bool {
    if dp.is_null() {
        return false;
    }

    unsafe { nvim_diffblock_get_count(dp, idx1) == nvim_diffblock_get_count(dp, idx2) }
}

// =============================================================================
// Diff Block Copying
// =============================================================================

/// Copy diff block entry from one buffer index to another.
///
/// This computes the line number for `idx_new` based on the offset between
/// the two buffers from the previous diff block.
pub fn diff_copy_entry(
    dprev: DiffBlockHandle,
    dp: DiffBlockHandle,
    idx_orig: c_int,
    idx_new: c_int,
) {
    if dp.is_null() {
        return;
    }

    unsafe {
        let off = if dprev.is_null() {
            0
        } else {
            // Calculate offset: (prev_lnum_orig + prev_count_orig) - (prev_lnum_new + prev_count_new)
            (nvim_diffblock_get_lnum(dprev, idx_orig) + nvim_diffblock_get_count(dprev, idx_orig))
                - (nvim_diffblock_get_lnum(dprev, idx_new)
                    + nvim_diffblock_get_count(dprev, idx_new))
        };

        // dp->df_lnum[idx_new] = dp->df_lnum[idx_orig] - off
        nvim_diffblock_set_lnum(dp, idx_new, nvim_diffblock_get_lnum(dp, idx_orig) - off);
        // dp->df_count[idx_new] = dp->df_count[idx_orig]
        nvim_diffblock_set_count(dp, idx_new, nvim_diffblock_get_count(dp, idx_orig));
    }
}

// =============================================================================
// Diff Sanity Checks
// =============================================================================

/// Check if a diff block contains valid line numbers.
///
/// Returns OK if valid, FAIL if invalid.
///
/// Note: This function requires nvim_tabpage_get_diffbuf C accessor which doesn't exist yet.
#[allow(dead_code)]
#[cfg(feature = "tabpage_diff")]
pub fn diff_check_sanity(tp: TabpageHandle, dp: DiffBlockHandle) -> c_int {
    if tp.is_null() || dp.is_null() {
        return FAIL;
    }

    unsafe {
        for i in 0..DB_COUNT {
            let buf = nvim_tabpage_get_diffbuf(tp, i);
            if !buf.is_null() {
                let lnum = nvim_diffblock_get_lnum(dp, i);
                let count = nvim_diffblock_get_count(dp, i);
                let line_count = nvim_buf_get_ml_line_count(buf);

                if lnum + count - 1 > line_count {
                    return FAIL;
                }
            }
        }
        OK
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

// FFI exports removed - the main ones are in lib.rs to maintain existing API compatibility.
// Additional exports below don't call functions requiring nvim_tabpage_get_diffbuf which doesn't
// exist yet in the C codebase.

/// FFI export: Get maximum diff block length.
#[no_mangle]
pub extern "C" fn rs_get_max_diff_length(dp: DiffBlockHandle) -> c_int {
    get_max_diff_length(dp)
}

/// FFI export: Check if diff block entries are equal.
#[no_mangle]
pub extern "C" fn rs_diff_equal_entry(dp: DiffBlockHandle, idx1: c_int, idx2: c_int) -> c_int {
    c_int::from(diff_equal_entry(dp, idx1, idx2))
}

/// FFI export: Copy diff block entry (new version).
#[no_mangle]
pub extern "C" fn rs_diff_copy_entry_new(
    dprev: DiffBlockHandle,
    dp: DiffBlockHandle,
    idx_orig: c_int,
    idx_new: c_int,
) {
    diff_copy_entry(dprev, dp, idx_orig, idx_new);
}

// Note: The following FFI exports are disabled because they require C accessor functions
// (nvim_tabpage_get_diffbuf, nvim_tabpage_get_diff_invalid, nvim_tabpage_set_diff_invalid)
// that don't exist yet in the C codebase:
//   rs_diff_buf_idx_tp, rs_diff_mode_buf, rs_diff_check_sanity

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_count() {
        assert_eq!(DB_COUNT, 8);
    }

    #[test]
    fn test_diff_block_handle_null() {
        let handle = DiffBlockHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_buf_handle_null() {
        let handle = BufHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_tabpage_handle_null() {
        let handle = TabpageHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_win_handle_null() {
        let handle = WinHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_diff_buffer_state_not_in_diff() {
        let state = DiffBufferState::not_in_diff();
        assert_eq!(state.idx, -1);
        assert!(!state.invalid);
        assert_eq!(state.buffer_count, 0);
    }

    #[test]
    fn test_handle_sizes() {
        use std::mem::size_of;
        assert_eq!(size_of::<DiffBlockHandle>(), size_of::<*mut c_void>());
        assert_eq!(size_of::<BufHandle>(), size_of::<*mut c_void>());
        assert_eq!(size_of::<TabpageHandle>(), size_of::<*mut c_void>());
        assert_eq!(size_of::<WinHandle>(), size_of::<*mut c_void>());
    }
}
