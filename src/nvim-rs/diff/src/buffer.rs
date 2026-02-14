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

    // Phase 1: State mutation accessors
    fn nvim_curtab_set_diffbuf(idx: c_int, buf: BufHandle);
    fn nvim_tabpage_set_diffbuf(tp: TabpageHandle, idx: c_int, buf: BufHandle);
    fn nvim_tabpage_set_first_diff(tp: TabpageHandle, dp: DiffBlockHandle);
    fn nvim_diff_set_next(dp: DiffBlockHandle, next: DiffBlockHandle);
    fn nvim_diffblock_clear_and_free(dp: DiffBlockHandle);
    fn nvim_diffblock_init_new(dp: DiffBlockHandle);
    fn nvim_diff_alloc_new(
        tp: TabpageHandle,
        prev: DiffBlockHandle,
        next: DiffBlockHandle,
    ) -> DiffBlockHandle;
    fn nvim_set_need_diff_redraw(val: bool);
    fn nvim_diff_get_linematch_lines() -> c_int;
    fn nvim_diff_get_diff_flags() -> c_int;
    fn nvim_diff_redraw(dofold: bool);
    fn nvim_diff_semsg_e96();
    fn nvim_redraw_later_win(wp: WinHandle, typ: c_int);
    fn nvim_upd_valid() -> c_int;

    // Window/tab iteration accessors
    fn nvim_tabpage_first_win(tp: TabpageHandle) -> WinHandle;
    fn nvim_win_next(wp: WinHandle) -> WinHandle;
    fn nvim_win_get_p_diff(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_get_curwin() -> WinHandle;

    // Diff block management accessors
    fn nvim_diff_foldUpdate(wp: WinHandle, top: LinenrT, bot: LinenrT);
    fn nvim_diff_set_diff_option(wp: WinHandle, value: bool);

    // String/memory for diff_equal_entry
    fn nvim_diff_ml_get_buf(buf: BufHandle, lnum: LinenrT) -> *const c_char;
    fn nvim_diff_xstrdup(s: *const c_char) -> *mut c_char;
    fn nvim_diff_xfree(p: *mut c_void);

    // UTF-8 helpers for diff_equal_char
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_fold(c: c_int) -> c_int;
    fn utf_ptr2char(p: *const c_char) -> c_int;

    // Phase 2: Additional accessors
    fn nvim_diff_get_busy() -> bool;
    fn nvim_diff_set_need_scrollbind(val: bool);
    fn nvim_tabpage_set_diff_update(tp: TabpageHandle, val: c_int);
    fn nvim_diffblock_is_linematched(dp: DiffBlockHandle) -> bool;
    fn nvim_diff_maxlnum() -> LinenrT;
    fn nvim_diff_get_algorithm() -> c_int;
    fn nvim_diff_set_options(
        flags: c_int,
        context: c_int,
        linematch: c_int,
        foldcol: c_int,
        algorithm: c_int,
    );
    fn nvim_diff_check_scrollbind();
    fn nvim_diff_parse_diffanchors() -> c_int;
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_diff_get_p_dip() -> *const c_char;
}

use std::ffi::c_char;

/// DIFF_LINEMATCH flag value (must match C).
const DIFF_LINEMATCH: c_int = 0x1000;
/// DIFF_ICASE flag value (must match C).
const DIFF_ICASE: c_int = 0x004;
/// DIFF_ANCHOR flag value (must match C).
const DIFF_ANCHOR: c_int = 0x20000;
/// DIFF_INTERNAL flag value (must match C).
const DIFF_INTERNAL: c_int = 0x200;
/// MAXLNUM constant (matches C).
const MAXLNUM: LinenrT = 0x7fff_ffff;

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

// =============================================================================
// Phase 1: Migrated Utility Functions
// =============================================================================

/// Free a diff block: clear df_changes garray and free memory.
///
/// Equivalent to C `clear_diffblock`.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_diffblock(dp: DiffBlockHandle) {
    if dp.is_null() {
        return;
    }
    nvim_diffblock_clear_and_free(dp);
}

/// Allocate a new diff block and link it between `dprev` and `dp`.
///
/// Equivalent to C `diff_alloc_new`.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_alloc_new(
    tp: TabpageHandle,
    dprev: DiffBlockHandle,
    dp: DiffBlockHandle,
) -> DiffBlockHandle {
    let dnew = nvim_diff_alloc_new(tp, dprev, dp);
    if !dnew.is_null() {
        nvim_diffblock_init_new(dnew);
    }
    dnew
}

/// Unlink and free a diff block, returning the next block.
///
/// Equivalent to C `diff_free`.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_free(
    tp: TabpageHandle,
    dprev: DiffBlockHandle,
    dp: DiffBlockHandle,
) -> DiffBlockHandle {
    if dp.is_null() {
        return DiffBlockHandle::null();
    }

    let ret = nvim_diffblock_get_next(dp);

    // Unlink from list
    if dprev.is_null() {
        nvim_tabpage_set_first_diff(tp, ret);
    } else {
        nvim_diff_set_next(dprev, ret);
    }

    // Free the block
    nvim_diffblock_clear_and_free(dp);

    ret
}

/// Free all diff blocks in a tabpage.
///
/// Equivalent to C `diff_clear`.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_clear(tp: TabpageHandle) {
    if tp.is_null() {
        return;
    }

    let mut p = nvim_tabpage_get_first_diff(tp);
    while !p.is_null() {
        let next_p = nvim_diffblock_get_next(p);
        nvim_diffblock_clear_and_free(p);
        p = next_p;
    }
    nvim_tabpage_set_first_diff(tp, DiffBlockHandle::null());
}

/// Clear all diff buffers in the current tab.
///
/// Equivalent to C `diff_buf_clear`.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_buf_clear() {
    let tp = nvim_get_curtab();
    for i in 0..DB_COUNT {
        if !nvim_get_curtab_diffbuf(i).is_null() {
            nvim_curtab_set_diffbuf(i, BufHandle::null());
            nvim_tabpage_set_diff_invalid(tp, 1);
            nvim_diff_redraw(true);
        }
    }
}

/// Mark the diff info involving buffer `buf` as invalid.
///
/// Equivalent to C `diff_invalidate`.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_invalidate(buf: BufHandle) {
    if buf.is_null() {
        return;
    }
    let curtab = nvim_get_curtab();
    let mut tp = nvim_get_first_tabpage();
    while !tp.is_null() {
        let i = rs_diff_buf_idx_tp(buf, tp);
        if i != DB_COUNT {
            nvim_tabpage_set_diff_invalid(tp, 1);
            if tp == curtab {
                nvim_diff_redraw(true);
            }
        }
        tp = nvim_tabpage_get_next(tp);
    }
}

/// Remove buffer from diff tracking in all tabs.
///
/// Equivalent to C `diff_buf_delete`.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_buf_delete(buf: BufHandle) {
    if buf.is_null() {
        return;
    }
    let curtab = nvim_get_curtab();
    let upd_valid = nvim_upd_valid();
    let mut tp = nvim_get_first_tabpage();
    while !tp.is_null() {
        let i = rs_diff_buf_idx_tp(buf, tp);
        if i != DB_COUNT {
            nvim_tabpage_set_diffbuf(tp, i, BufHandle::null());
            nvim_tabpage_set_diff_invalid(tp, 1);
            if tp == curtab {
                nvim_set_need_diff_redraw(true);
                nvim_redraw_later_win(nvim_get_curwin(), upd_valid);
            }
        }
        tp = nvim_tabpage_get_next(tp);
    }
}

/// Add a buffer to the diff list for the current tab.
///
/// Equivalent to C `diff_buf_add`.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_buf_add(buf: BufHandle) {
    if buf.is_null() {
        return;
    }
    let tp = nvim_get_curtab();

    // Already there?
    if rs_diff_buf_idx_tp(buf, tp) != DB_COUNT {
        return;
    }

    // Find a free slot
    for i in 0..DB_COUNT {
        if nvim_get_curtab_diffbuf(i).is_null() {
            nvim_curtab_set_diffbuf(i, buf);
            nvim_tabpage_set_diff_invalid(tp, 1);
            nvim_diff_redraw(true);
            return;
        }
    }

    nvim_diff_semsg_e96();
}

/// Check if buffer should be added/removed from diff list when window changes.
///
/// Equivalent to C `diff_buf_adjust`.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_buf_adjust(win: WinHandle) {
    if win.is_null() {
        return;
    }

    if nvim_win_get_p_diff(win) == 0 {
        // Window no longer in diff mode - check if any other window still shows
        // this buffer in diff mode
        let buf = nvim_win_get_w_buffer(win);
        let curtab = nvim_get_curtab();
        let mut found_win = false;

        let mut wp = nvim_tabpage_first_win(curtab);
        while !wp.is_null() {
            if nvim_win_get_w_buffer(wp) == buf && nvim_win_get_p_diff(wp) != 0 {
                found_win = true;
                break;
            }
            wp = nvim_win_next(wp);
        }

        if !found_win {
            let tp = nvim_get_curtab();
            let i = rs_diff_buf_idx_tp(buf, tp);
            if i != DB_COUNT {
                nvim_curtab_set_diffbuf(i, BufHandle::null());
                nvim_tabpage_set_diff_invalid(tp, 1);
                nvim_diff_redraw(true);
            }
        }
    } else {
        rs_diff_buf_add(nvim_win_get_w_buffer(win));
    }
}

/// Compare two characters at p1 and p2, possibly ignoring case.
/// Sets *len to the byte length of the character.
///
/// Equivalent to C `diff_equal_char`.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_equal_char(
    p1: *const c_char,
    p2: *const c_char,
    len: *mut c_int,
) -> bool {
    if p1.is_null() || p2.is_null() || len.is_null() {
        return false;
    }

    let l = utfc_ptr2len(p1);
    if l != utfc_ptr2len(p2) {
        return false;
    }

    let diff_flags = nvim_diff_get_diff_flags();

    #[allow(clippy::cast_sign_loss)]
    if l > 1 {
        if !bytes_equal(p1, p2, l as usize)
            && (diff_flags & DIFF_ICASE == 0
                || utf_fold(utf_ptr2char(p1)) != utf_fold(utf_ptr2char(p2)))
        {
            return false;
        }
        *len = l;
    } else {
        #[allow(clippy::cast_sign_loss)]
        let c1 = (*p1) as u8;
        #[allow(clippy::cast_sign_loss)]
        let c2 = (*p2) as u8;
        if c1 != c2 && (diff_flags & DIFF_ICASE == 0 || tolower_loc(c1) != tolower_loc(c2)) {
            return false;
        }
        *len = 1;
    }
    true
}

/// TOLOWER_LOC equivalent: lowercase ASCII byte.
#[inline]
const fn tolower_loc(c: u8) -> u8 {
    if c.is_ascii_uppercase() {
        c.to_ascii_lowercase()
    } else {
        c
    }
}

/// Compare bytes in two pointers up to `len` bytes.
///
/// # Safety
/// Both pointers must be valid for `len` bytes.
#[inline]
unsafe fn bytes_equal(p1: *const c_char, p2: *const c_char, len: usize) -> bool {
    std::slice::from_raw_parts(p1.cast::<u8>(), len)
        == std::slice::from_raw_parts(p2.cast::<u8>(), len)
}

/// Check if two entries in a diff block are equal (full line comparison).
///
/// Equivalent to C `diff_equal_entry`.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_equal_entry_full(
    dp: DiffBlockHandle,
    idx1: c_int,
    idx2: c_int,
) -> bool {
    if dp.is_null() {
        return false;
    }

    let count1 = nvim_diffblock_get_count(dp, idx1);
    let count2 = nvim_diffblock_get_count(dp, idx2);
    if count1 != count2 {
        return false;
    }

    let tp = nvim_get_curtab();
    if diff_check_sanity_internal(tp, dp) == FAIL {
        return false;
    }

    let buf1 = nvim_get_curtab_diffbuf(idx1);
    let buf2 = nvim_get_curtab_diffbuf(idx2);
    let lnum1 = nvim_diffblock_get_lnum(dp, idx1);
    let lnum2 = nvim_diffblock_get_lnum(dp, idx2);

    for i in 0..count1 {
        let line = nvim_diff_xstrdup(nvim_diff_ml_get_buf(buf1, lnum1 + i));
        let cmp = crate::rs_diff_cmp(line, nvim_diff_ml_get_buf(buf2, lnum2 + i));
        nvim_diff_xfree(line.cast::<c_void>());
        if cmp != 0 {
            return false;
        }
    }
    true
}

/// Check if diff block qualifies for linematch.
///
/// Equivalent to C `diff_linematch`.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_linematch(dp: DiffBlockHandle) -> bool {
    if dp.is_null() {
        return false;
    }

    let diff_flags = nvim_diff_get_diff_flags();
    if diff_flags & DIFF_LINEMATCH == 0 {
        return false;
    }

    let linematch_lines = nvim_diff_get_linematch_lines();
    let mut tsize: c_int = 0;
    for i in 0..DB_COUNT {
        if !nvim_get_curtab_diffbuf(i).is_null() {
            let count = nvim_diffblock_get_count(dp, i);
            if count < 0 {
                return false;
            }
            tsize += count;
        }
    }
    tsize <= linematch_lines
}

/// Get max line count in a diff block across all buffers.
///
/// Equivalent to C `get_max_diff_length`.
#[no_mangle]
pub unsafe extern "C" fn rs_get_max_diff_length_c(dp: DiffBlockHandle) -> c_int {
    if dp.is_null() {
        return 0;
    }
    let mut max_length: c_int = 0;
    for k in 0..DB_COUNT {
        if !nvim_get_curtab_diffbuf(k).is_null() {
            let count = nvim_diffblock_get_count(dp, k);
            if count > max_length {
                max_length = count;
            }
        }
    }
    max_length
}

/// qsort comparator for line numbers.
///
/// Equivalent to C `lnum_compare`.
#[no_mangle]
pub unsafe extern "C" fn rs_lnum_compare(s1: *const c_void, s2: *const c_void) -> c_int {
    if s1.is_null() || s2.is_null() {
        return 0;
    }
    let lnum1 = *s1.cast::<LinenrT>();
    let lnum2 = *s2.cast::<LinenrT>();
    match lnum1.cmp(&lnum2) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

/// Check if a diff block is still in the current tab's list.
///
/// Equivalent to C `valid_diff`.
#[no_mangle]
pub unsafe extern "C" fn rs_valid_diff(diff: DiffBlockHandle) -> bool {
    if diff.is_null() {
        return false;
    }
    let mut dp = nvim_get_diff_first_block();
    while !dp.is_null() {
        if dp == diff {
            return true;
        }
        dp = nvim_diffblock_get_next(dp);
    }
    false
}

/// Set the 'diff' option on a window.
///
/// Equivalent to C `set_diff_option`.
#[no_mangle]
pub unsafe extern "C" fn rs_set_diff_option(wp: WinHandle, value: bool) {
    if wp.is_null() {
        return;
    }
    nvim_diff_set_diff_option(wp, value);
}

/// Update folds for diff, skipping one buffer index.
///
/// Equivalent to C `diff_fold_update`.
#[no_mangle]
#[allow(clippy::suspicious_operation_groupings)]
pub unsafe extern "C" fn rs_diff_fold_update(dp: DiffBlockHandle, skip_idx: c_int) {
    if dp.is_null() {
        return;
    }
    let curtab = nvim_get_curtab();
    let mut wp = nvim_tabpage_first_win(curtab);
    while !wp.is_null() {
        for i in 0..DB_COUNT {
            if nvim_get_curtab_diffbuf(i) == nvim_win_get_w_buffer(wp) && i != skip_idx {
                let lnum = nvim_diffblock_get_lnum(dp, i);
                let count = nvim_diffblock_get_count(dp, i);
                nvim_diff_foldUpdate(wp, lnum, lnum + count);
            }
        }
        wp = nvim_win_next(wp);
    }
}

/// Check if buffer is in diff mode in any tab.
///
/// Equivalent to C `diff_mode_buf`.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_mode_buf(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    let mut tp = nvim_get_first_tabpage();
    while !tp.is_null() {
        if rs_diff_buf_idx_tp(buf, tp) != DB_COUNT {
            return true;
        }
        tp = nvim_tabpage_get_next(tp);
    }
    false
}

// =============================================================================
// Phase 2: Diff Block Management
// =============================================================================

/// Wrapper for diff_internal() check.
#[inline]
unsafe fn diff_internal() -> bool {
    nvim_diff_get_diff_flags() & DIFF_INTERNAL != 0
}

/// Called by mark_adjust(): update line numbers in all tabs for "buf".
///
/// Equivalent to C `diff_mark_adjust`.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_mark_adjust(
    buf: BufHandle,
    line1: LinenrT,
    line2: LinenrT,
    amount: LinenrT,
    amount_after: LinenrT,
) {
    if buf.is_null() {
        return;
    }
    let mut tp = nvim_get_first_tabpage();
    while !tp.is_null() {
        let idx = rs_diff_buf_idx_tp(buf, tp);
        if idx != DB_COUNT {
            rs_diff_mark_adjust_tp(tp, idx, line1, line2, amount, amount_after);
        }
        tp = nvim_tabpage_get_next(tp);
    }
}

/// Update diff blocks for tab `tp`, buffer index `idx`, after line changes.
///
/// Equivalent to C `diff_mark_adjust_tp`.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_diff_mark_adjust_tp(
    tp: TabpageHandle,
    idx: c_int,
    line1: LinenrT,
    line2: LinenrT,
    amount: LinenrT,
    amount_after: LinenrT,
) {
    if tp.is_null() {
        return;
    }

    if diff_internal() {
        nvim_tabpage_set_diff_invalid(tp, 1);
        nvim_tabpage_set_diff_update(tp, 1);
    }

    let (inserted, deleted): (LinenrT, LinenrT) = if line2 == MAXLNUM {
        (amount, 0)
    } else if amount_after > 0 {
        (amount_after, 0)
    } else {
        (0, -amount_after)
    };

    let mut dprev = DiffBlockHandle::null();
    let mut dp = nvim_tabpage_get_first_diff(tp);
    let mut lnum_deleted = line1;
    let mut deleted = deleted;
    let diff_busy = nvim_diff_get_busy();

    loop {
        // Create new diff block if change is between existing blocks
        let should_create = {
            let dp_cond = if dp.is_null() {
                true
            } else {
                let dp_lnum = nvim_diffblock_get_lnum(dp, idx);
                dp_lnum - 1 > line2 || (line2 == MAXLNUM && dp_lnum > line1)
            };
            let dprev_cond = if dprev.is_null() {
                true
            } else {
                let prev_lnum = nvim_diffblock_get_lnum(dprev, idx);
                let prev_count = nvim_diffblock_get_count(dprev, idx);
                prev_lnum + prev_count < line1
            };
            dp_cond && dprev_cond && !diff_busy
        };

        if should_create {
            let dnext = rs_diff_alloc_new(tp, dprev, dp);
            nvim_diffblock_set_lnum(dnext, idx, line1);
            nvim_diffblock_set_count(dnext, idx, inserted);

            for i in 0..DB_COUNT {
                if !nvim_tabpage_get_diffbuf(tp, i).is_null() && i != idx {
                    if dprev.is_null() {
                        nvim_diffblock_set_lnum(dnext, i, line1);
                    } else {
                        let prev_lnum_i = nvim_diffblock_get_lnum(dprev, i);
                        let prev_count_i = nvim_diffblock_get_count(dprev, i);
                        let prev_lnum_idx = nvim_diffblock_get_lnum(dprev, idx);
                        let prev_count_idx = nvim_diffblock_get_count(dprev, idx);
                        nvim_diffblock_set_lnum(
                            dnext,
                            i,
                            line1 + (prev_lnum_i + prev_count_i) - (prev_lnum_idx + prev_count_idx),
                        );
                    }
                    nvim_diffblock_set_count(dnext, i, deleted);
                }
            }
        }

        if dp.is_null() {
            break;
        }

        let dp_lnum = nvim_diffblock_get_lnum(dp, idx);
        let dp_count = nvim_diffblock_get_count(dp, idx);
        let last = dp_lnum + dp_count - 1;

        // 1. change completely above line1: nothing to do
        if last >= line1 - 1 {
            if diff_busy {
                if dp_lnum > line2 {
                    nvim_diffblock_set_lnum(dp, idx, dp_lnum + amount_after);
                }
                dprev = dp;
                dp = nvim_diffblock_get_next(dp);
                continue;
            }

            // 6. change below line2
            if dp_lnum - LinenrT::from(deleted + inserted != 0) > line2 {
                if amount_after == 0 {
                    break;
                }
                nvim_diffblock_set_lnum(dp, idx, dp_lnum + amount_after);
            } else {
                let mut check_unchanged = false;

                if deleted > 0 {
                    let n: LinenrT;
                    let off: LinenrT;
                    let dp_lnum = nvim_diffblock_get_lnum(dp, idx);
                    let dp_count = nvim_diffblock_get_count(dp, idx);

                    if dp_lnum >= line1 {
                        if last <= line2 {
                            // 4. delete all lines of diff
                            let dp_next = nvim_diffblock_get_next(dp);
                            if !dp_next.is_null()
                                && nvim_diffblock_get_lnum(dp_next, idx) - 1 <= line2
                            {
                                let next_lnum = nvim_diffblock_get_lnum(dp_next, idx);
                                n = next_lnum - lnum_deleted;
                                deleted -= n;
                                let n = n - dp_count;
                                lnum_deleted = next_lnum;
                                adjust_other_bufs(tp, dp, idx, 0, n);
                            } else {
                                let n = deleted - dp_count;
                                adjust_other_bufs(tp, dp, idx, 0, n);
                            }
                            nvim_diffblock_set_count(dp, idx, 0);
                        } else {
                            // 5. delete lines at or just before top of diff
                            off = dp_lnum - lnum_deleted;
                            n = off;
                            nvim_diffblock_set_count(dp, idx, dp_count - (line2 - dp_lnum + 1));
                            check_unchanged = true;
                            adjust_other_bufs(tp, dp, idx, off, n);
                        }
                        nvim_diffblock_set_lnum(dp, idx, line1);
                    } else if last < line2 {
                        // 2. delete at end of diff
                        nvim_diffblock_set_count(dp, idx, dp_count - (last - lnum_deleted + 1));

                        let dp_next = nvim_diffblock_get_next(dp);
                        if !dp_next.is_null() && nvim_diffblock_get_lnum(dp_next, idx) - 1 <= line2
                        {
                            let next_lnum = nvim_diffblock_get_lnum(dp_next, idx);
                            n = next_lnum - 1 - last;
                            deleted -= next_lnum - lnum_deleted;
                            lnum_deleted = next_lnum;
                        } else {
                            n = line2 - last;
                        }
                        check_unchanged = true;
                        adjust_other_bufs(tp, dp, idx, 0, n);
                    } else {
                        // 3. delete lines inside the diff
                        nvim_diffblock_set_count(dp, idx, dp_count - deleted);
                        adjust_other_bufs(tp, dp, idx, 0, 0);
                    }
                } else {
                    let dp_lnum = nvim_diffblock_get_lnum(dp, idx);
                    if dp_lnum <= line1 {
                        let dp_count = nvim_diffblock_get_count(dp, idx);
                        nvim_diffblock_set_count(dp, idx, dp_count + inserted);
                        check_unchanged = true;
                    } else {
                        nvim_diffblock_set_lnum(dp, idx, dp_lnum + inserted);
                    }
                }

                if check_unchanged {
                    rs_diff_check_unchanged(tp, dp);
                }
            }
        }

        // Check if this block touches the previous one, may merge them.
        let dp_lnum = nvim_diffblock_get_lnum(dp, idx);
        if !dprev.is_null()
            && !nvim_diffblock_is_linematched(dp)
            && !diff_busy
            && (nvim_diffblock_get_lnum(dprev, idx) + nvim_diffblock_get_count(dprev, idx)
                == dp_lnum)
        {
            for i in 0..DB_COUNT {
                if !nvim_tabpage_get_diffbuf(tp, i).is_null() {
                    let prev_count = nvim_diffblock_get_count(dprev, i);
                    let dp_count = nvim_diffblock_get_count(dp, i);
                    nvim_diffblock_set_count(dprev, i, prev_count + dp_count);
                }
            }
            dp = rs_diff_free(tp, dprev, dp);
        } else {
            dprev = dp;
            dp = nvim_diffblock_get_next(dp);
        }
    }

    // Second pass: remove entries where all counts are zero
    dprev = DiffBlockHandle::null();
    dp = nvim_tabpage_get_first_diff(tp);

    while !dp.is_null() {
        let mut all_zero = true;
        for i in 0..DB_COUNT {
            if !nvim_tabpage_get_diffbuf(tp, i).is_null() && nvim_diffblock_get_count(dp, i) != 0 {
                all_zero = false;
                break;
            }
        }

        if all_zero {
            dp = rs_diff_free(tp, dprev, dp);
        } else {
            dprev = dp;
            dp = nvim_diffblock_get_next(dp);
        }
    }

    let curtab = nvim_get_curtab();
    if tp == curtab {
        nvim_set_need_diff_redraw(true);
        nvim_diff_set_need_scrollbind(true);
    }
}

/// Helper: adjust lnum and count for other buffers in a diff block during deletion.
unsafe fn adjust_other_bufs(
    tp: TabpageHandle,
    dp: DiffBlockHandle,
    idx: c_int,
    off: LinenrT,
    n: LinenrT,
) {
    for i in 0..DB_COUNT {
        if !nvim_tabpage_get_diffbuf(tp, i).is_null() && i != idx {
            let lnum = nvim_diffblock_get_lnum(dp, i);
            if lnum > off {
                nvim_diffblock_set_lnum(dp, i, lnum - off);
            } else {
                nvim_diffblock_set_lnum(dp, i, 1);
            }
            let count = nvim_diffblock_get_count(dp, i);
            nvim_diffblock_set_count(dp, i, count + n);
        }
    }
}

/// Check if a diff block can be made smaller by removing equal lines.
///
/// Equivalent to C `diff_check_unchanged`.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_check_unchanged(tp: TabpageHandle, dp: DiffBlockHandle) {
    if tp.is_null() || dp.is_null() {
        return;
    }

    // Find the first buffer to use as the original
    let mut i_org: c_int = -1;
    for i in 0..DB_COUNT {
        if !nvim_tabpage_get_diffbuf(tp, i).is_null() {
            i_org = i;
            break;
        }
    }
    if i_org < 0 {
        return;
    }

    if diff_check_sanity_internal(tp, dp) == FAIL {
        return;
    }

    // FORWARD = 1, BACKWARD = -1
    let mut off_org: LinenrT = 0;
    let mut off_new: LinenrT;
    let mut dir: c_int = 1; // FORWARD

    loop {
        // Repeat until a line is found which is different or count becomes zero
        while nvim_diffblock_get_count(dp, i_org) > 0 {
            if dir == -1 {
                // BACKWARD
                off_org = nvim_diffblock_get_count(dp, i_org) - 1;
            }

            let org_buf = nvim_tabpage_get_diffbuf(tp, i_org);
            let org_lnum = nvim_diffblock_get_lnum(dp, i_org);
            let line_org = nvim_diff_xstrdup(nvim_diff_ml_get_buf(org_buf, org_lnum + off_org));

            let mut i_new = i_org + 1;
            let mut found_mismatch = false;
            while i_new < DB_COUNT {
                if nvim_tabpage_get_diffbuf(tp, i_new).is_null() {
                    i_new += 1;
                    continue;
                }

                off_new = if dir == -1 {
                    nvim_diffblock_get_count(dp, i_new) - 1
                } else {
                    0
                };

                if off_new < 0 || off_new >= nvim_diffblock_get_count(dp, i_new) {
                    found_mismatch = true;
                    break;
                }

                let new_buf = nvim_tabpage_get_diffbuf(tp, i_new);
                let new_lnum = nvim_diffblock_get_lnum(dp, i_new);
                if crate::rs_diff_cmp(line_org, nvim_diff_ml_get_buf(new_buf, new_lnum + off_new))
                    != 0
                {
                    found_mismatch = true;
                    break;
                }
                i_new += 1;
            }

            nvim_diff_xfree(line_org.cast::<c_void>());

            // Stop when a line isn't equal in all diff buffers
            if found_mismatch || i_new != DB_COUNT {
                break;
            }

            // Line matched in all buffers, remove it from the diff
            for j in i_org..DB_COUNT {
                if !nvim_tabpage_get_diffbuf(tp, j).is_null() {
                    if dir == 1 {
                        // FORWARD
                        let lnum = nvim_diffblock_get_lnum(dp, j);
                        nvim_diffblock_set_lnum(dp, j, lnum + 1);
                    }
                    let count = nvim_diffblock_get_count(dp, j);
                    nvim_diffblock_set_count(dp, j, count - 1);
                }
            }
        }

        if dir == -1 {
            break;
        }
        dir = -1; // switch to BACKWARD
    }
}

/// DiffoptResult matching the C struct.
#[repr(C)]
struct DiffoptResult {
    diff_flags: c_int,
    diff_algorithm: c_int,
    diff_context: c_int,
    diff_foldcolumn: c_int,
    linematch_lines: c_int,
    result: c_int,
}

extern "C" {
    fn rs_diffopt_parse(p_dip: *const c_char) -> DiffoptResult;
}

/// Handle 'diffopt' option changes.
///
/// Equivalent to C `diffopt_changed`.
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_changed() -> c_int {
    let p_dip = nvim_diff_get_p_dip();
    let parsed = rs_diffopt_parse(p_dip);
    if parsed.result == FAIL {
        return FAIL;
    }

    // If flags or algorithm changed, invalidate all tabs
    let old_flags = nvim_diff_get_diff_flags();
    let old_algorithm = nvim_diff_get_algorithm();
    if old_flags != parsed.diff_flags || old_algorithm != parsed.diff_algorithm {
        let mut tp = nvim_get_first_tabpage();
        while !tp.is_null() {
            nvim_tabpage_set_diff_invalid(tp, 1);
            tp = nvim_tabpage_get_next(tp);
        }
    }

    nvim_diff_set_options(
        parsed.diff_flags,
        parsed.diff_context,
        parsed.linematch_lines,
        parsed.diff_foldcolumn,
        parsed.diff_algorithm,
    );

    nvim_diff_redraw(true);
    nvim_diff_check_scrollbind();
    OK
}

/// Handle 'diffanchors' option changes.
///
/// Equivalent to C `diffanchors_changed`.
#[no_mangle]
pub unsafe extern "C" fn rs_diffanchors_changed(buflocal: bool) -> c_int {
    let result = nvim_diff_parse_diffanchors();
    if result == OK && (nvim_diff_get_diff_flags() & DIFF_ANCHOR != 0) {
        let curbuf = nvim_get_curbuf();
        let mut tp = nvim_get_first_tabpage();
        while !tp.is_null() {
            if buflocal {
                for idx in 0..DB_COUNT {
                    if nvim_tabpage_get_diffbuf(tp, idx) == curbuf {
                        nvim_tabpage_set_diff_invalid(tp, 1);
                        break;
                    }
                }
            } else {
                nvim_tabpage_set_diff_invalid(tp, 1);
            }
            tp = nvim_tabpage_get_next(tp);
        }
    }
    result
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
