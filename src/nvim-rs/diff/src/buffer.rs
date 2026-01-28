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
pub fn diff_buf_idx_tp(buf: BufHandle, tp: TabpageHandle) -> c_int {
    // SAFETY: Handles are checked for null
    unsafe { rs_diff_buf_idx_tp(buf, tp) }
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
#[allow(dead_code)]
pub fn diff_check_sanity_internal(tp: TabpageHandle, dp: DiffBlockHandle) -> c_int {
    // SAFETY: Handles are checked for null inside rs_diff_check_sanity
    unsafe { rs_diff_check_sanity(tp, dp) }
}

// =============================================================================
// FFI Exports (Block Queries)
// =============================================================================

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

// =============================================================================
// Buffer Registration
// =============================================================================

/// Result of attempting to add a buffer to diff mode.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffBufAddResult {
    /// Buffer was successfully added.
    Added = 0,
    /// Buffer was already in diff mode.
    AlreadyAdded = 1,
    /// No free slots available (DB_COUNT limit reached).
    NoFreeSlot = 2,
    /// Invalid buffer provided.
    InvalidBuffer = 3,
}

/// Result of buffer registration query.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DiffBufRegState {
    /// Index of buffer in diff list (-1 if not found).
    pub idx: c_int,
    /// First free slot index (-1 if none available).
    pub free_slot: c_int,
    /// Total number of registered buffers.
    pub count: c_int,
    /// Whether all slots are full.
    pub is_full: bool,
}

impl Default for DiffBufRegState {
    fn default() -> Self {
        Self {
            idx: -1,
            free_slot: -1,
            count: 0,
            is_full: false,
        }
    }
}

/// Query the buffer registration state for a tabpage.
///
/// This provides information needed to decide whether a buffer can be added
/// to diff mode and where.
///
/// # Safety
/// `buf` and `tp` must be valid handles or null.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_buf_reg_state(
    buf: BufHandle,
    tp: TabpageHandle,
) -> DiffBufRegState {
    if tp.is_null() {
        return DiffBufRegState::default();
    }

    let mut state = DiffBufRegState::default();

    for i in 0..DB_COUNT {
        let diffbuf = nvim_tabpage_get_diffbuf(tp, i);
        if diffbuf.is_null() {
            if state.free_slot < 0 {
                state.free_slot = i;
            }
        } else {
            state.count += 1;
            if !buf.is_null() && diffbuf.0 == buf.0 {
                state.idx = i;
            }
        }
    }

    state.is_full = state.count >= DB_COUNT;
    state
}

/// Check if a buffer can be added to diff mode.
///
/// Returns the result code and the slot index to use (-1 if cannot add).
///
/// # Safety
/// `buf` and `tp` must be valid handles.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_buf_can_add(
    buf: BufHandle,
    tp: TabpageHandle,
) -> DiffBufAddResult {
    if buf.is_null() || tp.is_null() {
        return DiffBufAddResult::InvalidBuffer;
    }

    let state = rs_diff_buf_reg_state(buf, tp);

    if state.idx >= 0 {
        return DiffBufAddResult::AlreadyAdded;
    }

    if state.is_full {
        return DiffBufAddResult::NoFreeSlot;
    }

    DiffBufAddResult::Added
}

/// Find the index of a buffer in a tabpage's diff list.
///
/// Returns the index (0 to DB_COUNT-1) or DB_COUNT if not found.
///
/// # Safety
/// `buf` and `tp` must be valid handles.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_buf_idx_tp(buf: BufHandle, tp: TabpageHandle) -> c_int {
    if buf.is_null() || tp.is_null() {
        return DB_COUNT;
    }

    for i in 0..DB_COUNT {
        let diffbuf = nvim_tabpage_get_diffbuf(tp, i);
        if !diffbuf.is_null() && diffbuf.0 == buf.0 {
            return i;
        }
    }
    DB_COUNT
}

/// Check if a buffer is in diff mode in a specific tabpage.
///
/// # Safety
/// `buf` and `tp` must be valid handles.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_buf_is_diffed_tp(buf: BufHandle, tp: TabpageHandle) -> bool {
    rs_diff_buf_idx_tp(buf, tp) != DB_COUNT
}

/// Find the first free slot in a tabpage's diff buffer list.
///
/// Returns the slot index (0 to DB_COUNT-1) or -1 if no free slot.
///
/// # Safety
/// `tp` must be a valid handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_find_free_slot(tp: TabpageHandle) -> c_int {
    if tp.is_null() {
        return -1;
    }

    for i in 0..DB_COUNT {
        if nvim_tabpage_get_diffbuf(tp, i).is_null() {
            return i;
        }
    }
    -1
}

/// Count the number of active diff buffers in a tabpage.
///
/// # Safety
/// `tp` must be a valid handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_count_buffers_tp(tp: TabpageHandle) -> c_int {
    if tp.is_null() {
        return 0;
    }

    let mut count = 0;
    for i in 0..DB_COUNT {
        if !nvim_tabpage_get_diffbuf(tp, i).is_null() {
            count += 1;
        }
    }
    count
}

/// Check if diff list sanity is valid (line numbers within buffer bounds).
///
/// Returns 1 (OK) if valid, 0 (FAIL) if invalid.
///
/// # Safety
/// `tp` and `dp` must be valid handles.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_check_sanity(tp: TabpageHandle, dp: DiffBlockHandle) -> c_int {
    if tp.is_null() || dp.is_null() {
        return FAIL;
    }

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

// =============================================================================
// Buffer Iteration
// =============================================================================

/// Iterator state for walking through diff buffers in a tabpage.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DiffBufIter {
    /// Tabpage being iterated.
    pub tp: TabpageHandle,
    /// Current index (0 to DB_COUNT).
    pub idx: c_int,
    /// Current buffer (null if done).
    pub current: BufHandle,
    /// Whether iteration is complete.
    pub done: bool,
}

impl Default for DiffBufIter {
    fn default() -> Self {
        Self {
            tp: TabpageHandle::null(),
            idx: 0,
            current: BufHandle::null(),
            done: true,
        }
    }
}

/// Create a new iterator for diff buffers in a tabpage.
///
/// # Safety
/// `tp` must be a valid tabpage handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_buf_iter_new(tp: TabpageHandle) -> DiffBufIter {
    if tp.is_null() {
        return DiffBufIter::default();
    }

    let mut iter = DiffBufIter {
        tp,
        idx: -1, // Will be advanced to first valid
        current: BufHandle::null(),
        done: false,
    };

    // Advance to first valid buffer
    rs_diff_buf_iter_next(&raw mut iter);
    iter
}

/// Advance the diff buffer iterator to the next buffer.
///
/// Returns true if there are more buffers, false if done.
///
/// # Safety
/// `iter` must be a valid pointer to a `DiffBufIter`.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_buf_iter_next(iter: *mut DiffBufIter) -> bool {
    if iter.is_null() {
        return false;
    }

    let it = &mut *iter;
    if it.done || it.tp.is_null() {
        it.done = true;
        it.current = BufHandle::null();
        return false;
    }

    // Find next non-null buffer
    loop {
        it.idx += 1;
        if it.idx >= DB_COUNT {
            it.done = true;
            it.current = BufHandle::null();
            return false;
        }

        let buf = nvim_tabpage_get_diffbuf(it.tp, it.idx);
        if !buf.is_null() {
            it.current = buf;
            return true;
        }
    }
}

/// Check if the diff buffer iterator is done.
#[no_mangle]
pub const extern "C" fn rs_diff_buf_iter_done(iter: &DiffBufIter) -> bool {
    iter.done
}

/// Get the current buffer from the iterator.
#[no_mangle]
pub const extern "C" fn rs_diff_buf_iter_current(iter: &DiffBufIter) -> BufHandle {
    iter.current
}

/// Get the current buffer index from the iterator.
#[no_mangle]
pub const extern "C" fn rs_diff_buf_iter_idx(iter: &DiffBufIter) -> c_int {
    iter.idx
}

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

    #[test]
    fn test_diff_buf_add_result_values() {
        assert_eq!(DiffBufAddResult::Added as c_int, 0);
        assert_eq!(DiffBufAddResult::AlreadyAdded as c_int, 1);
        assert_eq!(DiffBufAddResult::NoFreeSlot as c_int, 2);
        assert_eq!(DiffBufAddResult::InvalidBuffer as c_int, 3);
    }

    #[test]
    fn test_diff_buf_reg_state_default() {
        let state = DiffBufRegState::default();
        assert_eq!(state.idx, -1);
        assert_eq!(state.free_slot, -1);
        assert_eq!(state.count, 0);
        assert!(!state.is_full);
    }

    #[test]
    fn test_diff_buf_iter_default() {
        let iter = DiffBufIter::default();
        assert!(iter.tp.is_null());
        assert_eq!(iter.idx, 0);
        assert!(iter.current.is_null());
        assert!(iter.done);
    }

    #[test]
    fn test_diff_buf_iter_done() {
        let iter = DiffBufIter::default();
        assert!(rs_diff_buf_iter_done(&iter));
    }

    #[test]
    fn test_diff_buf_iter_current() {
        let iter = DiffBufIter::default();
        assert!(rs_diff_buf_iter_current(&iter).is_null());
    }

    #[test]
    fn test_diff_buf_iter_idx() {
        let iter = DiffBufIter::default();
        assert_eq!(rs_diff_buf_iter_idx(&iter), 0);
    }
}
