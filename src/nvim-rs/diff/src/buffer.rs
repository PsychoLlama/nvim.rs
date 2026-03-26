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
    fn nvim_tabpage_is_diff_invalid(tp: TabpageHandle) -> bool;
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
    fn nvim_get_diff_flags() -> c_int;
    #[link_name = "diff_redraw"]
    fn rs_diff_redraw(dofold: bool);
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
/// DIFF_FILLER flag value (must match C).
const DIFF_FILLER: c_int = 0x001;
/// DIFF_IWHITE flag value (must match C).
const DIFF_IWHITE: c_int = 0x008;
/// DIFF_IWHITEALL flag value (must match C).
const DIFF_IWHITEALL: c_int = 0x010;
/// MAXLNUM constant (matches C).
const MAXLNUM: LinenrT = 0x7fff_ffff;
/// Direction constants (must match C).
const FORWARD: c_int = 1;
const BACKWARD: c_int = -1;

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
            rs_diff_redraw(true);
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
                rs_diff_redraw(true);
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
            rs_diff_redraw(true);
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
                rs_diff_redraw(true);
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

    let diff_flags = nvim_get_diff_flags();

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

    let diff_flags = nvim_get_diff_flags();
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
    nvim_get_diff_flags() & DIFF_INTERNAL != 0
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
    let old_flags = nvim_get_diff_flags();
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

    rs_diff_redraw(true);
    nvim_diff_check_scrollbind();
    OK
}

/// Handle 'diffanchors' option changes.
///
/// Equivalent to C `diffanchors_changed`.
#[no_mangle]
pub unsafe extern "C" fn rs_diffanchors_changed(buflocal: bool) -> c_int {
    let curbuf = nvim_get_curbuf();
    let result = rs_parse_diffanchors(true, curbuf, std::ptr::null_mut(), std::ptr::null_mut());
    if result == OK && (nvim_get_diff_flags() & DIFF_ANCHOR != 0) {
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

// =============================================================================
// Phase 3: Diff Computation Pipeline
// =============================================================================

/// Opaque handle to a diffio_T allocated on the C heap.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiffioHandle(*mut c_void);

impl DiffioHandle {
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

/// Opaque exarg_T handle.
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ExargHandle(*const c_void);

impl ExargHandle {
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// Phase 3 C accessors
#[allow(dead_code)]
extern "C" {
    fn nvim_diffio_new(use_internal: bool) -> DiffioHandle;
    fn nvim_diffio_free(dio: DiffioHandle);
    fn nvim_diffio_is_internal(dio: DiffioHandle) -> bool;
    fn nvim_diffio_init_ga(dio: DiffioHandle);
    fn nvim_diffio_alloc_tempfiles(dio: DiffioHandle) -> bool;
    fn nvim_diffio_free_tempfiles(dio: DiffioHandle);
    fn nvim_diffio_write_orig(
        dio: DiffioHandle,
        buf: BufHandle,
        start: LinenrT,
        end: LinenrT,
    ) -> c_int;
    fn nvim_diffio_write_new(
        dio: DiffioHandle,
        buf: BufHandle,
        start: LinenrT,
        end: LinenrT,
    ) -> c_int;
    fn nvim_diffio_run_diff(dio: DiffioHandle) -> c_int;
    fn nvim_diffio_check_external(dio: DiffioHandle) -> c_int;
    fn nvim_diffio_clear_new(dio: DiffioHandle);
    fn nvim_diffio_clear_output(dio: DiffioHandle);
    fn nvim_diffio_clear_orig(dio: DiffioHandle);
    fn nvim_diffio_get_hunk_count(dio: DiffioHandle) -> c_int;
    fn nvim_diffio_get_hunk(
        dio: DiffioHandle,
        idx: c_int,
        lnum_orig: *mut LinenrT,
        count_orig: *mut c_int,
        lnum_new: *mut LinenrT,
        count_new: *mut c_int,
    ) -> bool;
    fn nvim_diffio_open_output(dio: DiffioHandle) -> *mut c_void;
    fn nvim_diff_fgets(fd: *mut c_void, buf: *mut c_char, buflen: c_int) -> bool;
    fn nvim_diff_fclose(fd: *mut c_void);
    fn nvim_diff_buf_valid(buf: BufHandle) -> bool;
    fn nvim_diff_buf_check_timestamp(buf: BufHandle);
    fn nvim_diff_buf_is_loaded(buf: BufHandle) -> bool;
    fn nvim_diff_curtab_set_first_diff(dp: DiffBlockHandle);
    fn nvim_eap_forceit(eap: ExargHandle) -> bool;
    fn nvim_diff_invalidate_cursor();
    fn nvim_diff_fire_diffupdated();
    fn nvim_is_diffexpr_empty() -> bool;
    fn nvim_diff_get_need_update() -> bool;
    fn nvim_diff_set_need_update(val: bool);
    fn nvim_diff_set_busy(val: bool);
    fn nvim_diff_max_anchors() -> c_int;
    fn nvim_diff_emsg_e98();
    fn nvim_diff_emsg_anchors();
    fn nvim_diff_parse_buf_anchors(
        buf: BufHandle,
        anchors: *mut LinenrT,
        max_anchors: c_int,
    ) -> c_int;
    fn nvim_diff_sort_lnums(arr: *mut LinenrT, count: c_int);
    fn nvim_diff_parse_ed(
        line: *const c_char,
        lnum_orig: *mut LinenrT,
        count_orig: *mut c_int,
        lnum_new: *mut LinenrT,
        count_new: *mut c_int,
    ) -> c_int;
    fn nvim_diff_parse_unified(
        line: *const c_char,
        lnum_orig: *mut LinenrT,
        count_orig: *mut c_int,
        lnum_new: *mut LinenrT,
        count_new: *mut c_int,
    ) -> c_int;
}

// Phase 4 C accessors: window fields (from window.c), diff_context, external calls
#[allow(dead_code)]
extern "C" {
    // Window field accessors (defined in window.c)
    fn nvim_win_get_topline(wp: WinHandle) -> LinenrT;
    fn nvim_win_set_topline(wp: WinHandle, lnum: LinenrT);
    fn nvim_win_get_topfill(wp: WinHandle) -> c_int;
    fn nvim_win_set_topfill(wp: WinHandle, fill: c_int);
    fn nvim_win_get_botline(wp: WinHandle) -> LinenrT;
    fn nvim_win_set_botfill(wp: WinHandle, val: c_int);
    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> LinenrT;
    fn nvim_win_set_cursor_lnum(wp: WinHandle, lnum: LinenrT);
    fn nvim_win_set_cursor_col(wp: WinHandle, col: c_int);
    // Diff-specific accessors (defined in diff.c)
    fn nvim_diff_get_context() -> c_int;
    fn nvim_diff_hasFolding(wp: WinHandle, lnum: LinenrT) -> bool;
    fn nvim_diff_hasFolding_topline(wp: WinHandle, lnum: LinenrT, topline: *mut LinenrT) -> bool;
    fn nvim_diff_decor_conceal_line(wp: WinHandle, lnum: LinenrT) -> bool;
    fn nvim_diff_invalidate_botline_win(wp: WinHandle);
    fn nvim_diff_changed_line_abv_curs_win(wp: WinHandle);
    fn nvim_diff_check_topfill(wp: WinHandle, down: bool);
    fn nvim_diff_setpcmark();
    #[link_name = "rs_run_linematch"]
    fn nvim_diff_run_linematch(dp: DiffBlockHandle);
}

// Phase 5 C accessors for inline change detection
type DiffLineChangeHandle = *mut c_void;
type DifflineHandle = *mut c_void;

extern "C" {
    // diff block inline change accessors
    fn nvim_diffblock_get_has_changes(dp: DiffBlockHandle) -> bool;
    fn nvim_diffblock_set_has_changes(dp: DiffBlockHandle, val: bool);
    fn nvim_diffblock_reset_changes_len(dp: DiffBlockHandle);
    fn nvim_diffblock_get_changes_len(dp: DiffBlockHandle) -> c_int;
    fn nvim_diffblock_get_change(dp: DiffBlockHandle, change_idx: c_int) -> DiffLineChangeHandle;
    // simple_diffline_change sentinel
    fn nvim_diff_get_simple_change() -> DiffLineChangeHandle;
    fn nvim_diff_is_simple_change(change: DiffLineChangeHandle) -> bool;
    // inline diff computation is now done by rs_compute_inline_diff (inline_compute.rs)
    // diffline_change_T field accessors
    fn nvim_diffchange_get_start_lnum_off(change: DiffLineChangeHandle, idx: c_int) -> c_int;
    fn nvim_diffchange_get_end_lnum_off(change: DiffLineChangeHandle, idx: c_int) -> c_int;
    fn nvim_diffchange_get_start(change: DiffLineChangeHandle, idx: c_int) -> LinenrT;
    fn nvim_diffchange_get_end(change: DiffLineChangeHandle, idx: c_int) -> LinenrT;
    // string helpers
    fn nvim_diff_skipwhite(p: *const c_char) -> *const c_char;
    // UTF-8
    fn utf_head_off(base: *const c_char, ptr: *const c_char) -> c_int;
}

/// Diff output style.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DiffStyle {
    Ed,
    Unified,
    None,
}

/// A diff hunk (line numbers and counts for orig and new).
#[derive(Clone, Copy, Debug, Default)]
struct DiffHunk {
    lnum_orig: LinenrT,
    count_orig: c_int,
    lnum_new: LinenrT,
    count_new: c_int,
}

const LBUFLEN: usize = 50;

/// Extract a hunk from internal diff output.
/// Returns true on EOF.
unsafe fn extract_hunk_internal(
    dio: DiffioHandle,
    hunk: &mut DiffHunk,
    line_idx: &mut c_int,
) -> bool {
    let count = nvim_diffio_get_hunk_count(dio);
    if *line_idx >= count {
        return true;
    }
    let mut lnum_orig: LinenrT = 0;
    let mut count_orig: c_int = 0;
    let mut lnum_new: LinenrT = 0;
    let mut count_new: c_int = 0;
    if nvim_diffio_get_hunk(
        dio,
        *line_idx,
        &raw mut lnum_orig,
        &raw mut count_orig,
        &raw mut lnum_new,
        &raw mut count_new,
    ) {
        hunk.lnum_orig = lnum_orig;
        hunk.count_orig = count_orig;
        hunk.lnum_new = lnum_new;
        hunk.count_new = count_new;
        *line_idx += 1;
    }
    false
}

/// Extract a hunk from external diff file output.
/// Returns true on EOF.
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
unsafe fn extract_hunk_external(
    fd: *mut c_void,
    hunk: &mut DiffHunk,
    diffstyle: &mut DiffStyle,
) -> bool {
    let mut line = [0u8; LBUFLEN];

    loop {
        if nvim_diff_fgets(fd, line.as_mut_ptr().cast::<c_char>(), LBUFLEN as c_int) {
            return true; // EOF
        }

        if *diffstyle == DiffStyle::None {
            // Determine diff style
            if (line[0] as char).is_ascii_digit() {
                *diffstyle = DiffStyle::Ed;
            } else if line.starts_with(b"@@ ") {
                *diffstyle = DiffStyle::Unified;
            } else if line.starts_with(b"--- ") {
                // Check for unified diff header: ---, +++, @@
                let mut line2 = [0u8; LBUFLEN];
                if nvim_diff_fgets(fd, line2.as_mut_ptr().cast::<c_char>(), LBUFLEN as c_int) {
                    continue;
                }
                if !line2.starts_with(b"+++ ") {
                    continue;
                }
                if nvim_diff_fgets(fd, line2.as_mut_ptr().cast::<c_char>(), LBUFLEN as c_int) {
                    continue;
                }
                if line2.starts_with(b"@@ ") {
                    *diffstyle = DiffStyle::Unified;
                    // Use this line for parsing below
                    line.copy_from_slice(&line2);
                } else {
                    continue;
                }
            } else {
                continue;
            }
        }

        let mut lnum_orig: LinenrT = 0;
        let mut count_orig: c_int = 0;
        let mut lnum_new: LinenrT = 0;
        let mut count_new: c_int = 0;

        if *diffstyle == DiffStyle::Ed {
            if !(line[0] as char).is_ascii_digit() {
                continue;
            }
            if nvim_diff_parse_ed(
                line.as_ptr().cast::<c_char>(),
                &raw mut lnum_orig,
                &raw mut count_orig,
                &raw mut lnum_new,
                &raw mut count_new,
            ) == FAIL
            {
                continue;
            }
        } else {
            // Unified
            if !line.starts_with(b"@@ ") {
                continue;
            }
            if nvim_diff_parse_unified(
                line.as_ptr().cast::<c_char>(),
                &raw mut lnum_orig,
                &raw mut count_orig,
                &raw mut lnum_new,
                &raw mut count_new,
            ) == FAIL
            {
                continue;
            }
        }

        hunk.lnum_orig = lnum_orig;
        hunk.count_orig = count_orig;
        hunk.lnum_new = lnum_new;
        hunk.count_new = count_new;
        return false;
    }
}

/// Process a single diff hunk into the diff block list.
///
/// Equivalent to C `process_hunk`.
#[no_mangle]
#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::similar_names
)]
pub unsafe extern "C" fn rs_process_hunk(
    dpp: *mut DiffBlockHandle,
    dprevp: *mut DiffBlockHandle,
    idx_orig: c_int,
    idx_new: c_int,
    lnum_orig: LinenrT,
    count_orig: c_int,
    lnum_new: LinenrT,
    count_new: c_int,
    notsetp: *mut bool,
) {
    if dpp.is_null() || dprevp.is_null() || notsetp.is_null() {
        return;
    }

    let mut dp = *dpp;
    let mut dprev = *dprevp;
    let notset = &mut *notsetp;

    // Go over blocks before the change, for which orig and new are equal.
    // Copy blocks from orig to new.
    while !dp.is_null()
        && lnum_orig
            > nvim_diffblock_get_lnum(dp, idx_orig) + nvim_diffblock_get_count(dp, idx_orig)
    {
        if *notset {
            crate::rs_diff_copy_entry(dprev.as_ptr(), dp.as_ptr(), idx_orig, idx_new);
        }
        dprev = dp;
        dp = nvim_diffblock_get_next(dp);
        *notset = true;
    }

    if !dp.is_null()
        && lnum_orig
            <= nvim_diffblock_get_lnum(dp, idx_orig) + nvim_diffblock_get_count(dp, idx_orig)
        && lnum_orig + count_orig as LinenrT >= nvim_diffblock_get_lnum(dp, idx_orig)
    {
        // New block overlaps with existing block(s).
        // Find last block that overlaps.
        let mut dpl = dp;
        loop {
            let dpl_next = nvim_diffblock_get_next(dpl);
            if dpl_next.is_null() {
                break;
            }
            if lnum_orig + (count_orig as LinenrT) < nvim_diffblock_get_lnum(dpl_next, idx_orig) {
                break;
            }
            dpl = dpl_next;
        }

        let off = nvim_diffblock_get_lnum(dp, idx_orig) - lnum_orig;

        if off > 0 {
            for i in idx_orig..idx_new {
                if !nvim_get_curtab_diffbuf(i).is_null() {
                    let lnum = nvim_diffblock_get_lnum(dp, i);
                    let count = nvim_diffblock_get_count(dp, i);
                    nvim_diffblock_set_lnum(dp, i, lnum - off);
                    nvim_diffblock_set_count(dp, i, count + off);
                }
            }
            nvim_diffblock_set_lnum(dp, idx_new, lnum_new);
            nvim_diffblock_set_count(dp, idx_new, count_new as LinenrT);
        } else if *notset {
            // new block inside existing one, adjust new block
            nvim_diffblock_set_lnum(dp, idx_new, lnum_new + off);
            nvim_diffblock_set_count(dp, idx_new, count_new as LinenrT - off);
        } else {
            // second overlap of new block with existing block
            let dp_lnum = nvim_diffblock_get_lnum(dp, idx_orig);
            let dp_count = nvim_diffblock_get_count(dp, idx_orig);
            let orig_size_in_dp = (count_orig as LinenrT).min(dp_lnum + dp_count - lnum_orig);
            let size_diff = count_new as LinenrT - orig_size_in_dp;
            let dp_count_new = nvim_diffblock_get_count(dp, idx_new);
            nvim_diffblock_set_count(dp, idx_new, dp_count_new + size_diff);

            // grow existing block to include the overlap completely
            let dp_lnum_new = nvim_diffblock_get_lnum(dp, idx_new);
            let dp_count_new = nvim_diffblock_get_count(dp, idx_new);
            let grow = lnum_new + count_new as LinenrT - (dp_lnum_new + dp_count_new);
            if grow > 0 {
                nvim_diffblock_set_count(dp, idx_new, dp_count_new + grow);
            }
        }

        // Adjust the size of the block to include all the lines to the end
        let dpl_lnum = nvim_diffblock_get_lnum(dpl, idx_orig);
        let dpl_count = nvim_diffblock_get_count(dpl, idx_orig);
        let mut off = (lnum_orig + count_orig as LinenrT) - (dpl_lnum + dpl_count);

        if off < 0 {
            if *notset || dp != dpl {
                let c = nvim_diffblock_get_count(dp, idx_new);
                nvim_diffblock_set_count(dp, idx_new, c + (-off));
            }
            off = 0;
        }

        for i in idx_orig..idx_new {
            if !nvim_get_curtab_diffbuf(i).is_null() {
                let dpl_l = nvim_diffblock_get_lnum(dpl, i);
                let dpl_c = nvim_diffblock_get_count(dpl, i);
                let dp_l = nvim_diffblock_get_lnum(dp, i);
                nvim_diffblock_set_count(dp, i, dpl_l + dpl_c - dp_l + off);
            }
        }

        // Delete merged diff blocks
        let mut dn = nvim_diffblock_get_next(dp);
        let dp_next_after_dpl = nvim_diffblock_get_next(dpl);
        nvim_diff_set_next(dp, dp_next_after_dpl);

        while dn != dp_next_after_dpl {
            let dpl_next = nvim_diffblock_get_next(dn);
            nvim_diffblock_clear_and_free(dn);
            dn = dpl_next;
        }
    } else {
        // Allocate a new diffblock.
        dp = rs_diff_alloc_new(nvim_get_curtab(), dprev, dp);

        nvim_diffblock_set_lnum(dp, idx_orig, lnum_orig);
        nvim_diffblock_set_count(dp, idx_orig, count_orig as LinenrT);
        nvim_diffblock_set_lnum(dp, idx_new, lnum_new);
        nvim_diffblock_set_count(dp, idx_new, count_new as LinenrT);

        // Set values for other buffers
        for i in (idx_orig + 1)..idx_new {
            if !nvim_get_curtab_diffbuf(i).is_null() {
                crate::rs_diff_copy_entry(dprev.as_ptr(), dp.as_ptr(), idx_orig, i);
            }
        }
    }

    *notset = false;
    *dpp = dp;
    *dprevp = dprev;
}

/// Read the diff output and add each entry to the diff list.
///
/// Equivalent to C `diff_read`.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_read(idx_orig: c_int, idx_new: c_int, dio: DiffioHandle) {
    let is_internal = nvim_diffio_is_internal(dio);
    let mut line_hunk_idx: c_int = 0;
    let mut dprev = DiffBlockHandle::null();
    let mut dp = nvim_get_diff_first_block();
    let mut notset = true;
    let mut diffstyle = DiffStyle::None;

    let fd = if is_internal {
        std::ptr::null_mut()
    } else {
        let f = nvim_diffio_open_output(dio);
        if f.is_null() {
            nvim_diff_emsg_e98();
            return;
        }
        f
    };

    loop {
        let mut hunk = DiffHunk::default();
        let eof = if is_internal {
            extract_hunk_internal(dio, &mut hunk, &mut line_hunk_idx)
        } else {
            extract_hunk_external(fd, &mut hunk, &mut diffstyle)
        };

        if eof {
            break;
        }

        rs_process_hunk(
            &raw mut dp,
            &raw mut dprev,
            idx_orig,
            idx_new,
            hunk.lnum_orig,
            hunk.count_orig,
            hunk.lnum_new,
            hunk.count_new,
            &raw mut notset,
        );
    }

    // For remaining diff blocks, orig and new are equal
    while !dp.is_null() {
        if notset {
            crate::rs_diff_copy_entry(dprev.as_ptr(), dp.as_ptr(), idx_orig, idx_new);
        }
        dprev = dp;
        dp = nvim_diffblock_get_next(dp);
        notset = true;
    }

    if !fd.is_null() {
        nvim_diff_fclose(fd);
    }
}

const MAX_DIFF_ANCHORS_RUST: usize = 20;

/// Update diffs for all buffers involved.
///
/// Equivalent to C `diff_try_update`.
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::too_many_lines)]
pub unsafe extern "C" fn rs_diff_try_update(dio: DiffioHandle, idx_orig: c_int, eap: ExargHandle) {
    if dio.is_null() {
        return;
    }

    let is_internal = nvim_diffio_is_internal(dio);

    if is_internal {
        nvim_diffio_init_ga(dio);
    } else {
        // Allocate temp filenames
        if !nvim_diffio_alloc_tempfiles(dio) {
            // goto theend equivalent: free and return
            nvim_diffio_free_tempfiles(dio);
            return;
        }
        // Check external diff actually works
        if nvim_diffio_check_external(dio) == FAIL {
            nvim_diffio_free_tempfiles(dio);
            return;
        }
    }

    // :diffupdate! — check timestamps
    if nvim_eap_forceit(eap) {
        for idx_new in idx_orig..DB_COUNT {
            let buf = nvim_get_curtab_diffbuf(idx_new);
            if nvim_diff_buf_valid(buf) {
                nvim_diff_buf_check_timestamp(buf);
            }
        }
    }

    // Parse and sort diff anchors if enabled
    let diff_flags = nvim_get_diff_flags();
    let mut num_anchors: c_int = i32::MAX;
    let mut anchors = [[0 as LinenrT; MAX_DIFF_ANCHORS_RUST]; DB_COUNT as usize];

    if (diff_flags & DIFF_ANCHOR) != 0 {
        for idx in 0..DB_COUNT {
            let buf = nvim_get_curtab_diffbuf(idx);
            if buf.is_null() {
                continue;
            }
            let mut buf_num: c_int = 0;
            let result = rs_parse_diffanchors(
                false,
                buf,
                anchors[idx as usize].as_mut_ptr(),
                std::ptr::addr_of_mut!(buf_num),
            );
            if result != OK {
                nvim_diff_emsg_anchors();
                num_anchors = 0;
                anchors = [[0; MAX_DIFF_ANCHORS_RUST]; DB_COUNT as usize];
                break;
            }
            if buf_num < num_anchors {
                num_anchors = buf_num;
            }
            if buf_num > 0 {
                nvim_diff_sort_lnums(anchors[idx as usize].as_mut_ptr(), buf_num);
            }
        }
    }
    if num_anchors == i32::MAX {
        num_anchors = 0;
    }

    // Process sections split by anchors
    for anchor_i in 0..=num_anchors {
        let mut orig_diff = DiffBlockHandle::null();
        if anchor_i != 0 {
            orig_diff = nvim_get_diff_first_block();
            nvim_diff_curtab_set_first_diff(DiffBlockHandle::null());
        }

        let lnum_start = if anchor_i == 0 {
            1
        } else {
            anchors[idx_orig as usize][(anchor_i - 1) as usize]
        };
        let lnum_end = if anchor_i == num_anchors {
            -1
        } else {
            anchors[idx_orig as usize][anchor_i as usize] - 1
        };

        // Write the first buffer
        let buf = nvim_get_curtab_diffbuf(idx_orig);
        if nvim_diffio_write_orig(dio, buf, lnum_start, lnum_end) == FAIL {
            if !orig_diff.is_null() {
                nvim_diff_curtab_set_first_diff(orig_diff);
                rs_diff_clear(nvim_get_curtab());
            }
            nvim_diffio_free_tempfiles(dio);
            return;
        }

        // Compare with every other buffer
        for idx_new in (idx_orig + 1)..DB_COUNT {
            let buf = nvim_get_curtab_diffbuf(idx_new);
            if buf.is_null() || !nvim_diff_buf_is_loaded(buf) {
                continue;
            }

            let new_start = if anchor_i == 0 {
                1
            } else {
                anchors[idx_new as usize][(anchor_i - 1) as usize]
            };
            let new_end = if anchor_i == num_anchors {
                -1
            } else {
                anchors[idx_new as usize][anchor_i as usize] - 1
            };

            if nvim_diffio_write_new(dio, buf, new_start, new_end) == FAIL {
                continue;
            }
            if nvim_diffio_run_diff(dio) == FAIL {
                continue;
            }

            rs_diff_read(idx_orig, idx_new, dio);

            nvim_diffio_clear_new(dio);
            nvim_diffio_clear_output(dio);
        }
        nvim_diffio_clear_orig(dio);

        if anchor_i != 0 {
            // Combine new diff blocks with existing ones
            let mut dp = nvim_get_diff_first_block();
            while !dp.is_null() {
                for idx in 0..DB_COUNT {
                    if anchors[idx as usize][(anchor_i - 1) as usize] > 0 {
                        let lnum = nvim_diffblock_get_lnum(dp, idx);
                        nvim_diffblock_set_lnum(
                            dp,
                            idx,
                            lnum + anchors[idx as usize][(anchor_i - 1) as usize] - 1,
                        );
                    }
                }
                dp = nvim_diffblock_get_next(dp);
            }

            if !orig_diff.is_null() {
                // Find last block in orig_diff chain
                let mut last_diff = orig_diff;
                loop {
                    let next = nvim_diffblock_get_next(last_diff);
                    if next.is_null() {
                        break;
                    }
                    last_diff = next;
                }
                let cur_first = nvim_get_diff_first_block();
                nvim_diff_set_next(last_diff, cur_first);
                nvim_diff_curtab_set_first_diff(orig_diff);
            }
        }
    }

    nvim_diffio_free_tempfiles(dio);
}

// =============================================================================
// Phase 4: Diff Status Checking & Navigation
// =============================================================================

/// Helper: find the top diff block containing `topline` for `fromidx`, and the
/// next non-adjacent block after it.
unsafe fn find_top_diff_block(
    fromidx: c_int,
    topline: LinenrT,
) -> (DiffBlockHandle, DiffBlockHandle) {
    let mut thistopdiff = DiffBlockHandle::null();
    let mut next_adjacent = DiffBlockHandle::null();
    let mut localtopdiff = DiffBlockHandle::null();
    let mut topdiffchange = true;

    let mut dp = nvim_get_diff_first_block();
    while !dp.is_null() {
        if localtopdiff.is_null() || topdiffchange {
            localtopdiff = dp;
            topdiffchange = false;
        }

        let dp_lnum = nvim_diffblock_get_lnum(dp, fromidx);
        let dp_count = nvim_diffblock_get_count(dp, fromidx);

        if topline >= dp_lnum && topline <= dp_lnum + dp_count && thistopdiff.is_null() {
            thistopdiff = localtopdiff;
        }

        let next = nvim_diffblock_get_next(dp);
        let is_adjacent =
            !next.is_null() && nvim_diffblock_get_lnum(next, fromidx) == dp_lnum + dp_count;

        if !is_adjacent {
            topdiffchange = true;
            if !thistopdiff.is_null() {
                next_adjacent = next;
                break;
            }
        }

        dp = next;
    }

    (thistopdiff, next_adjacent)
}

/// Helper: calculate topfill and topline for a target diff window.
#[allow(clippy::too_many_lines)]
unsafe fn calculate_topfill_and_topline(
    fromidx: c_int,
    toidx: c_int,
    from_topline: LinenrT,
    from_topfill: c_int,
) -> (c_int, LinenrT) {
    let (thistopdiff, next_adjacent) = find_top_diff_block(fromidx, from_topline);

    let mut virtual_lines_passed: i32 = 0;
    let mut curdif = thistopdiff;
    while !curdif.is_null() && curdif != next_adjacent {
        let cur_lnum = nvim_diffblock_get_lnum(curdif, fromidx);
        let cur_count = nvim_diffblock_get_count(curdif, fromidx);
        if cur_lnum + cur_count > from_topline {
            break;
        }
        virtual_lines_passed += rs_get_max_diff_length_c(curdif);
        curdif = nvim_diffblock_get_next(curdif);
    }

    if curdif != next_adjacent && !curdif.is_null() {
        virtual_lines_passed += from_topline - nvim_diffblock_get_lnum(curdif, fromidx);
    }
    virtual_lines_passed -= from_topfill;
    if virtual_lines_passed < 0 {
        virtual_lines_passed = 0;
    }

    let mut curlinenum_to: LinenrT = if thistopdiff.is_null() {
        1
    } else {
        nvim_diffblock_get_lnum(thistopdiff, toidx)
    };

    let mut virt_lines_left = virtual_lines_passed;
    curdif = thistopdiff;
    while virt_lines_left > 0 && !curdif.is_null() && curdif != next_adjacent {
        let count_to = nvim_diffblock_get_count(curdif, toidx);
        let max_len = rs_get_max_diff_length_c(curdif);
        curlinenum_to += virt_lines_left.min(count_to);
        virt_lines_left -= virt_lines_left.min(max_len);
        curdif = nvim_diffblock_get_next(curdif);
    }

    let mut max_virt_lines: i32 = 0;
    let mut dp = thistopdiff;
    while !dp.is_null() {
        let dp_lnum_to = nvim_diffblock_get_lnum(dp, toidx);
        let dp_count_to = nvim_diffblock_get_count(dp, toidx);
        if dp_lnum_to + dp_count_to <= curlinenum_to {
            max_virt_lines += rs_get_max_diff_length_c(dp);
        } else {
            if dp_lnum_to <= curlinenum_to {
                max_virt_lines += curlinenum_to - dp_lnum_to;
            }
            break;
        }
        dp = nvim_diffblock_get_next(dp);
    }

    let diff_flags = nvim_get_diff_flags();
    let topfill = if diff_flags & DIFF_FILLER != 0 {
        max_virt_lines - virtual_lines_passed
    } else {
        0
    };

    (topfill, curlinenum_to)
}

/// Check diff status for line "lnum" in window "wp".
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_diff_check_with_linestatus(
    wp: WinHandle,
    lnum: LinenrT,
    linestatus: *mut c_int,
) -> c_int {
    if !linestatus.is_null() {
        *linestatus = 0;
    }

    let tp = nvim_get_curtab();
    if nvim_tabpage_is_diff_invalid(tp) {
        rs_diff_ex_diffupdate(ExargHandle(std::ptr::null()));
    }

    let first_diff = nvim_tabpage_get_first_diff(tp);
    if first_diff.is_null() || nvim_win_get_p_diff(wp) == 0 {
        return 0;
    }

    let buf = nvim_win_get_w_buffer(wp);
    let line_count = nvim_buf_get_ml_line_count(buf);
    if lnum < 1 || lnum > line_count + 1 {
        return 0;
    }

    let idx = rs_diff_buf_idx_tp(buf, tp);
    if idx == DB_COUNT {
        return 0;
    }

    if nvim_diff_hasFolding(wp, lnum) || nvim_diff_decor_conceal_line(wp, lnum) {
        return 0;
    }

    // Find the diff block that includes lnum
    let mut dp = nvim_tabpage_get_first_diff(tp);
    while !dp.is_null() {
        if lnum <= nvim_diffblock_get_lnum(dp, idx) + nvim_diffblock_get_count(dp, idx) {
            break;
        }
        dp = nvim_diffblock_get_next(dp);
    }

    if dp.is_null() || lnum < nvim_diffblock_get_lnum(dp, idx) {
        return 0;
    }

    // Run linematch if on-screen and not yet matched
    let topline = nvim_win_get_topline(wp);
    let botline = nvim_win_get_botline(wp);
    if lnum >= topline
        && lnum < botline
        && !nvim_diffblock_is_linematched(dp)
        && rs_diff_linematch(dp)
        && rs_diff_check_sanity(tp, dp) != 0
    {
        nvim_diff_run_linematch(dp);
    }

    // Count filler lines from adjacent blocks
    let diff_flags = nvim_get_diff_flags();
    let mut num_fill: c_int = 0;
    while lnum == nvim_diffblock_get_lnum(dp, idx) + nvim_diffblock_get_count(dp, idx) {
        if diff_flags & DIFF_FILLER != 0 {
            let maxcount = rs_get_max_diff_length_c(dp);
            num_fill += maxcount - nvim_diffblock_get_count(dp, idx);
        }

        let next = nvim_diffblock_get_next(dp);
        if !next.is_null()
            && lnum >= nvim_diffblock_get_lnum(next, idx)
            && lnum <= nvim_diffblock_get_lnum(next, idx) + nvim_diffblock_get_count(next, idx)
        {
            dp = next;
        } else {
            break;
        }
    }

    if lnum < nvim_diffblock_get_lnum(dp, idx) + nvim_diffblock_get_count(dp, idx) {
        let mut zero = false;
        let mut cmp = false;

        for i in 0..DB_COUNT {
            if i != idx && !nvim_get_curtab_diffbuf(i).is_null() {
                if nvim_diffblock_get_count(dp, i) == 0 {
                    zero = true;
                } else if nvim_diffblock_get_count(dp, i) != nvim_diffblock_get_count(dp, idx) {
                    if !linestatus.is_null() {
                        *linestatus = -1;
                    }
                    return num_fill;
                } else {
                    cmp = true;
                }
            }
        }

        if cmp {
            for i in 0..DB_COUNT {
                if i != idx
                    && !nvim_get_curtab_diffbuf(i).is_null()
                    && nvim_diffblock_get_count(dp, i) != 0
                    && !rs_diff_equal_entry_full(dp, idx, i)
                {
                    if !linestatus.is_null() {
                        *linestatus = -1;
                    }
                    return num_fill;
                }
            }
        }

        if !zero {
            return num_fill;
        }
        if !linestatus.is_null() {
            *linestatus = -2;
        }
        return num_fill;
    }

    num_fill
}

/// Check filler lines for "lnum" in window "wp".
#[no_mangle]
pub unsafe extern "C" fn rs_diff_check_fill(wp: WinHandle, lnum: LinenrT) -> c_int {
    let diff_flags = nvim_get_diff_flags();
    if diff_flags & DIFF_FILLER == 0 {
        return 0;
    }
    let n = rs_diff_check_with_linestatus(wp, lnum, std::ptr::null_mut());
    if n > 0 {
        n
    } else {
        0
    }
}

/// Set the topline of "towin" to match "fromwin" for diff scroll sync.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_diff_set_topline(fromwin: WinHandle, towin: WinHandle) {
    let tp = nvim_get_curtab();
    let frombuf = nvim_win_get_w_buffer(fromwin);
    let fromidx = rs_diff_buf_idx_tp(frombuf, tp);
    if fromidx == DB_COUNT {
        return;
    }

    if nvim_tabpage_is_diff_invalid(tp) {
        rs_diff_ex_diffupdate(ExargHandle(std::ptr::null()));
    }

    let lnum = nvim_win_get_topline(fromwin);
    nvim_win_set_topfill(towin, 0);

    // Find diff block that includes lnum
    let mut dp = nvim_tabpage_get_first_diff(tp);
    while !dp.is_null() {
        if lnum <= nvim_diffblock_get_lnum(dp, fromidx) + nvim_diffblock_get_count(dp, fromidx) {
            break;
        }
        dp = nvim_diffblock_get_next(dp);
    }

    if dp.is_null() {
        // After last change, compute relative to end of file
        let to_buf = nvim_win_get_w_buffer(towin);
        let to_line_count = nvim_buf_get_ml_line_count(to_buf);
        let from_line_count = nvim_buf_get_ml_line_count(frombuf);
        nvim_win_set_topline(towin, to_line_count - (from_line_count - lnum));
    } else {
        let tobuf = nvim_win_get_w_buffer(towin);
        let toidx = rs_diff_buf_idx_tp(tobuf, tp);
        if toidx == DB_COUNT {
            return;
        }
        let new_topline =
            lnum + (nvim_diffblock_get_lnum(dp, toidx) - nvim_diffblock_get_lnum(dp, fromidx));
        nvim_win_set_topline(towin, new_topline);

        if lnum >= nvim_diffblock_get_lnum(dp, fromidx) {
            let from_topfill = nvim_win_get_topfill(fromwin);
            let (topfill, topline) =
                calculate_topfill_and_topline(fromidx, toidx, lnum, from_topfill);
            nvim_win_set_topfill(towin, topfill);
            nvim_win_set_topline(towin, topline);
        }
    }

    // Safety checks
    nvim_win_set_botfill(towin, 0);
    let to_buf = nvim_win_get_w_buffer(towin);
    let to_line_count = nvim_buf_get_ml_line_count(to_buf);

    if nvim_win_get_topline(towin) > to_line_count {
        nvim_win_set_topline(towin, to_line_count);
        nvim_win_set_botfill(towin, 1);
    }
    if nvim_win_get_topline(towin) < 1 {
        nvim_win_set_topline(towin, 1);
        nvim_win_set_topfill(towin, 0);
    }

    nvim_diff_invalidate_botline_win(towin);
    nvim_diff_changed_line_abv_curs_win(towin);
    nvim_diff_check_topfill(towin, false);

    let mut new_topline = nvim_win_get_topline(towin);
    nvim_diff_hasFolding_topline(towin, new_topline, &raw mut new_topline);
    nvim_win_set_topline(towin, new_topline);
}

/// Check if line should be folded in diff mode.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_infold(wp: WinHandle, lnum: LinenrT) -> bool {
    if nvim_win_get_p_diff(wp) == 0 {
        return false;
    }

    let tp = nvim_get_curtab();
    let buf = nvim_win_get_w_buffer(wp);
    let mut idx: c_int = -1;
    let mut other = false;
    for i in 0..DB_COUNT {
        if nvim_tabpage_get_diffbuf(tp, i) == buf {
            idx = i;
        } else if !nvim_tabpage_get_diffbuf(tp, i).is_null() {
            other = true;
        }
    }

    if idx == -1 || !other {
        return false;
    }

    if nvim_tabpage_is_diff_invalid(tp) {
        rs_diff_ex_diffupdate(ExargHandle(std::ptr::null()));
    }

    let first_diff = nvim_tabpage_get_first_diff(tp);
    if first_diff.is_null() {
        return true;
    }

    let diff_ctx = nvim_diff_get_context();
    let mut dp = first_diff;
    while !dp.is_null() {
        let dp_lnum = nvim_diffblock_get_lnum(dp, idx);
        let dp_count = nvim_diffblock_get_count(dp, idx);
        if dp_lnum - diff_ctx > lnum {
            break;
        }
        if dp_lnum + dp_count + diff_ctx > lnum {
            return false;
        }
        dp = nvim_diffblock_get_next(dp);
    }
    true
}

/// Move cursor to next/prev diff block.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_move_to(dir: c_int, count: c_int) -> c_int {
    let tp = nvim_get_curtab();
    let curwin = nvim_get_curwin();
    let curbuf = nvim_get_curbuf();
    let mut lnum = nvim_win_get_cursor_lnum(curwin);
    let idx = rs_diff_buf_idx_tp(curbuf, tp);

    if idx == DB_COUNT || nvim_tabpage_get_first_diff(tp).is_null() {
        return FAIL;
    }

    if nvim_tabpage_is_diff_invalid(tp) {
        rs_diff_ex_diffupdate(ExargHandle(std::ptr::null()));
    }

    if nvim_tabpage_get_first_diff(tp).is_null() {
        return FAIL;
    }

    let mut remaining = count;
    while remaining > 0 {
        remaining -= 1;

        let first_diff = nvim_tabpage_get_first_diff(tp);
        if dir == BACKWARD && lnum <= nvim_diffblock_get_lnum(first_diff, idx) {
            break;
        }

        let mut dp = first_diff;
        loop {
            if dp.is_null() {
                break;
            }

            let next = nvim_diffblock_get_next(dp);
            if (dir == FORWARD && lnum < nvim_diffblock_get_lnum(dp, idx))
                || (dir == BACKWARD
                    && (next.is_null() || lnum <= nvim_diffblock_get_lnum(next, idx)))
            {
                lnum = nvim_diffblock_get_lnum(dp, idx);
                break;
            }
            dp = next;
        }
    }

    // Clamp to buffer end
    let line_count = nvim_buf_get_ml_line_count(curbuf);
    if lnum > line_count {
        lnum = line_count;
    }

    if lnum == nvim_win_get_cursor_lnum(curwin) {
        return FAIL;
    }

    nvim_diff_setpcmark();
    nvim_win_set_cursor_lnum(curwin, lnum);
    nvim_win_set_cursor_col(curwin, 0);

    OK
}

/// Find the corresponding line in curbuf for lnum1 in buf1.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_get_corresponding_line_int(
    buf1: BufHandle,
    lnum1: LinenrT,
) -> LinenrT {
    let tp = nvim_get_curtab();
    let curbuf = nvim_get_curbuf();
    let curwin = nvim_get_curwin();
    let idx1 = rs_diff_buf_idx_tp(buf1, tp);
    let idx2 = rs_diff_buf_idx_tp(curbuf, tp);

    if idx1 == DB_COUNT || idx2 == DB_COUNT || nvim_tabpage_get_first_diff(tp).is_null() {
        return lnum1;
    }

    if nvim_tabpage_is_diff_invalid(tp) {
        rs_diff_ex_diffupdate(ExargHandle(std::ptr::null()));
    }

    if nvim_tabpage_get_first_diff(tp).is_null() {
        return lnum1;
    }

    let mut baseline: LinenrT = 0;
    let mut dp = nvim_tabpage_get_first_diff(tp);
    while !dp.is_null() {
        let dp_lnum1 = nvim_diffblock_get_lnum(dp, idx1);
        let dp_count1 = nvim_diffblock_get_count(dp, idx1);
        let dp_lnum2 = nvim_diffblock_get_lnum(dp, idx2);
        let dp_count2 = nvim_diffblock_get_count(dp, idx2);

        if dp_lnum1 > lnum1 {
            return lnum1 - baseline;
        }
        if dp_lnum1 + dp_count1 > lnum1 {
            // Inside the diffblock
            baseline = lnum1 - dp_lnum1;
            if baseline > dp_count2 {
                baseline = dp_count2;
            }
            return dp_lnum2 + baseline;
        }
        if dp_lnum1 == lnum1
            && dp_count1 == 0
            && dp_lnum2 <= nvim_win_get_cursor_lnum(curwin)
            && dp_lnum2 + dp_count2 > nvim_win_get_cursor_lnum(curwin)
        {
            return nvim_win_get_cursor_lnum(curwin);
        }
        baseline = (dp_lnum1 + dp_count1) - (dp_lnum2 + dp_count2);

        dp = nvim_diffblock_get_next(dp);
    }

    lnum1 - baseline
}

/// Find the corresponding line, clamped to buffer end.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_get_corresponding_line(
    buf1: BufHandle,
    lnum1: LinenrT,
) -> LinenrT {
    let lnum = rs_diff_get_corresponding_line_int(buf1, lnum1);
    let curbuf = nvim_get_curbuf();
    let line_count = nvim_buf_get_ml_line_count(curbuf);
    if lnum < line_count {
        lnum
    } else {
        line_count
    }
}

/// For line "lnum" in the current window find the equivalent lnum in window "wp".
#[no_mangle]
pub unsafe extern "C" fn rs_diff_lnum_win(lnum: LinenrT, wp: WinHandle) -> LinenrT {
    let tp = nvim_get_curtab();
    let curbuf = nvim_get_curbuf();
    let idx = rs_diff_buf_idx_tp(curbuf, tp);

    if idx == DB_COUNT {
        return 0;
    }

    if nvim_tabpage_is_diff_invalid(tp) {
        rs_diff_ex_diffupdate(ExargHandle(std::ptr::null()));
    }

    // Find the diff block that includes lnum
    let mut dp = nvim_tabpage_get_first_diff(tp);
    while !dp.is_null() {
        if lnum <= nvim_diffblock_get_lnum(dp, idx) + nvim_diffblock_get_count(dp, idx) {
            break;
        }
        dp = nvim_diffblock_get_next(dp);
    }

    if dp.is_null() {
        // After last change, compute relative to end of file
        let wp_buf = nvim_win_get_w_buffer(wp);
        let wp_line_count = nvim_buf_get_ml_line_count(wp_buf);
        let cur_line_count = nvim_buf_get_ml_line_count(curbuf);
        return wp_line_count - (cur_line_count - lnum);
    }

    let wp_buf = nvim_win_get_w_buffer(wp);
    let i = rs_diff_buf_idx_tp(wp_buf, tp);
    if i == DB_COUNT {
        return 0;
    }

    let n = lnum + (nvim_diffblock_get_lnum(dp, i) - nvim_diffblock_get_lnum(dp, idx));
    let max_n = nvim_diffblock_get_lnum(dp, i) + nvim_diffblock_get_count(dp, i);
    if n < max_n {
        n
    } else {
        max_n
    }
}

// =============================================================================
// Phase 5: Inline Change Detection
// =============================================================================

const ALL_INLINE_DIFF: c_int = 0x8000 | 0x10000; // DIFF_INLINE_CHAR | DIFF_INLINE_WORD
const DIFF_INLINE_NONE: c_int = 0x2000;
const MAXCOL: LinenrT = i32::MAX;

/// Rust-side representation of C diffline_T struct.
/// Must match layout: { diffline_change_T *changes; int num_changes; int bufidx; int lineoff; }
#[repr(C)]
struct DifflineRepr {
    changes: *mut c_void,
    num_changes: c_int,
    bufidx: c_int,
    lineoff: c_int,
}

/// Rust-side representation of C diffline_change_T struct.
/// Layout: { colnr_T dc_start[8]; colnr_T dc_end[8]; int dc_start_lnum_off[8]; int dc_end_lnum_off[8]; }
#[repr(C)]
#[allow(clippy::struct_field_names)]
struct DifflineChangeRepr {
    dc_start: [c_int; DB_COUNT as usize],
    dc_end: [c_int; DB_COUNT as usize],
    dc_start_lnum_off: [c_int; DB_COUNT as usize],
    dc_end_lnum_off: [c_int; DB_COUNT as usize],
}

/// Invalidate the inline diff cache for the diff block containing `lnum`.
///
/// Called when a line is changed in Insert mode to clear cached inline diff results.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_diff_update_line(lnum: LinenrT) {
    let diff_flags = nvim_get_diff_flags();
    if (diff_flags & ALL_INLINE_DIFF) == 0 {
        // Only care if doing inline-diff where we cache results
        return;
    }

    let curbuf = nvim_get_curbuf();
    let tp = nvim_get_curtab();
    let idx = rs_diff_buf_idx_tp(curbuf, tp);
    if idx == DB_COUNT {
        return;
    }

    let mut dp = nvim_tabpage_get_first_diff(tp);
    while !dp.is_null() {
        let dp_lnum = nvim_diffblock_get_lnum(dp, idx);
        let dp_count = nvim_diffblock_get_count(dp, idx);
        if lnum <= dp_lnum + dp_count {
            break;
        }
        dp = nvim_diffblock_get_next(dp);
    }

    // Clear the inline change cache
    if !dp.is_null() {
        nvim_diffblock_set_has_changes(dp, false);
        nvim_diffblock_reset_changes_len(dp);
    }
}

/// Parse a diffline struct and return [start,end] byte offsets.
///
/// Returns true if this change was added (no other buffer has it).
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_diff_change_parse(
    diffline: DifflineHandle,
    change: DiffLineChangeHandle,
    change_start: *mut c_int,
    change_end: *mut c_int,
) -> bool {
    if diffline.is_null() || change.is_null() || change_start.is_null() || change_end.is_null() {
        return false;
    }

    let dl = &*(diffline.cast::<DifflineRepr>());
    let bufidx = dl.bufidx;
    let lineoff = dl.lineoff;

    let start_lnum_off = nvim_diffchange_get_start_lnum_off(change, bufidx);
    let end_lnum_off = nvim_diffchange_get_end_lnum_off(change, bufidx);

    if start_lnum_off < lineoff {
        *change_start = 0;
    } else {
        *change_start = nvim_diffchange_get_start(change, bufidx);
    }

    if end_lnum_off > lineoff {
        *change_end = i32::MAX;
    } else {
        *change_end = nvim_diffchange_get_end(change, bufidx);
    }

    // Check if this is the simple_diffline_change sentinel
    if nvim_diff_is_simple_change(change) {
        return false;
    }

    // Check if this is an addition: all other buffers have empty ranges
    for i in 0..DB_COUNT {
        if i == bufidx {
            continue;
        }
        let dc_start = nvim_diffchange_get_start(change, i);
        let dc_end = nvim_diffchange_get_end(change, i);
        let dc_start_off = nvim_diffchange_get_start_lnum_off(change, i);
        let dc_end_off = nvim_diffchange_get_end_lnum_off(change, i);
        if dc_start != dc_end || dc_end_off != dc_start_off {
            return false;
        }
    }
    true
}

/// Compare two characters for diff equality — delegates to the existing rs_diff_equal_char.
#[allow(clippy::ref_as_ptr)]
unsafe fn diff_equal_char_p5(s1: *const c_char, s2: *const c_char, len: &mut c_int) -> bool {
    rs_diff_equal_char(s1, s2, std::ptr::from_mut(len))
}

/// Find the difference within a changed line — simple algorithm.
///
/// Returns true if the line was added (not present in any other buffer).
#[no_mangle]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::ref_as_ptr
)]
pub unsafe extern "C" fn rs_diff_find_change_simple(
    wp: WinHandle,
    lnum: LinenrT,
    dp: DiffBlockHandle,
    idx: c_int,
    startp: *mut c_int,
    endp: *mut c_int,
) -> bool {
    if wp.is_null() || dp.is_null() || startp.is_null() || endp.is_null() {
        return false;
    }

    let diff_flags = nvim_get_diff_flags();
    let buf = nvim_win_get_w_buffer(wp);

    // Get the original line
    let line_org: *mut c_char = if (diff_flags & DIFF_INLINE_NONE) != 0 {
        std::ptr::null_mut()
    } else {
        // Make a copy, the next ml_get will invalidate it
        nvim_diff_xstrdup(nvim_diff_ml_get_buf(buf, lnum))
    };

    let off = lnum - nvim_diffblock_get_lnum(dp, idx);
    let mut added = true;

    for i in 0..DB_COUNT {
        let other_buf = nvim_get_curtab_diffbuf(i);
        if other_buf.is_null() || i == idx {
            continue;
        }

        // Skip lines not in the other change (filler lines)
        let other_count = nvim_diffblock_get_count(dp, i);
        if off >= other_count {
            continue;
        }

        added = false;

        if (diff_flags & DIFF_INLINE_NONE) != 0 {
            break;
        }

        let other_lnum = nvim_diffblock_get_lnum(dp, i) + off;
        let line_new = nvim_diff_ml_get_buf(other_buf, other_lnum);

        if !line_org.is_null() && !line_new.is_null() {
            // Find start of difference
            let mut si_org: c_int = 0;
            let mut si_new: c_int = 0;

            while *line_org.offset(si_org as isize) != 0 {
                if ((diff_flags & DIFF_IWHITE) != 0
                    && is_white(*line_org.offset(si_org as isize) as u8)
                    && is_white(*line_new.offset(si_new as isize) as u8))
                    || ((diff_flags & DIFF_IWHITEALL) != 0
                        && (is_white(*line_org.offset(si_org as isize) as u8)
                            || is_white(*line_new.offset(si_new as isize) as u8)))
                {
                    // Skip whitespace
                    let new_org = nvim_diff_skipwhite(line_org.offset(si_org as isize));
                    si_org = new_org.offset_from(line_org) as c_int;
                    let new_new = nvim_diff_skipwhite(line_new.offset(si_new as isize));
                    si_new = new_new.offset_from(line_new) as c_int;
                } else {
                    let mut l: c_int = 0;
                    if !diff_equal_char_p5(
                        line_org.offset(si_org as isize),
                        line_new.offset(si_new as isize),
                        &mut l,
                    ) {
                        break;
                    }
                    si_org += l;
                    si_new += l;
                }
            }

            // Move back to first byte of character
            si_org -= utf_head_off(line_org, line_org.offset(si_org as isize));
            si_new -= utf_head_off(line_new, line_new.offset(si_new as isize));

            *startp = (*startp).min(si_org);

            // Search for end of difference
            if *line_org.offset(si_org as isize) != 0 || *line_new.offset(si_new as isize) != 0 {
                let mut ei_org: c_int = libc::strlen(line_org.cast()) as c_int;
                let mut ei_new: c_int = libc::strlen(line_new.cast()) as c_int;

                while ei_org >= *startp && ei_new >= si_new && ei_org >= 0 && ei_new >= 0 {
                    if ((diff_flags & DIFF_IWHITE) != 0
                        && is_white(*line_org.offset(ei_org as isize) as u8)
                        && is_white(*line_new.offset(ei_new as isize) as u8))
                        || ((diff_flags & DIFF_IWHITEALL) != 0
                            && (is_white(*line_org.offset(ei_org as isize) as u8)
                                || is_white(*line_new.offset(ei_new as isize) as u8)))
                    {
                        while ei_org >= *startp && is_white(*line_org.offset(ei_org as isize) as u8)
                        {
                            ei_org -= 1;
                        }
                        while ei_new >= si_new && is_white(*line_new.offset(ei_new as isize) as u8)
                        {
                            ei_new -= 1;
                        }
                    } else {
                        let p1 = line_org.offset(ei_org as isize);
                        let p2 = line_new.offset(ei_new as isize);
                        let p1_adj = p1.offset(-(utf_head_off(line_org, p1) as isize));
                        let p2_adj = p2.offset(-(utf_head_off(line_new, p2) as isize));

                        let mut l: c_int = 0;
                        if !diff_equal_char_p5(p1_adj, p2_adj, &mut l) {
                            break;
                        }
                        ei_org -= l;
                        ei_new -= l;
                    }
                }

                *endp = (*endp).max(ei_org);
            }
        }
    }

    if !line_org.is_null() {
        nvim_diff_xfree(line_org.cast());
    }

    added
}

/// Helper: check if byte is whitespace (space or tab).
#[inline]
const fn is_white(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Find the difference within a changed line — main dispatcher.
///
/// Returns true if the line was added (no other buffer has it).
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::ref_as_ptr)]
pub unsafe extern "C" fn rs_diff_find_change(
    wp: WinHandle,
    lnum: LinenrT,
    diffline: DifflineHandle,
) -> bool {
    if wp.is_null() || diffline.is_null() {
        return false;
    }

    let tp = nvim_get_curtab();
    let buf = nvim_win_get_w_buffer(wp);
    let idx = rs_diff_buf_idx_tp(buf, tp);

    if idx == DB_COUNT {
        return false;
    }

    // Search for the diff block containing lnum
    let mut dp = nvim_tabpage_get_first_diff(tp);
    while !dp.is_null() {
        if lnum < nvim_diffblock_get_lnum(dp, idx) + nvim_diffblock_get_count(dp, idx) {
            break;
        }
        dp = nvim_diffblock_get_next(dp);
    }

    if dp.is_null() || rs_diff_check_sanity(tp, dp) == FAIL {
        return false;
    }

    let off = lnum - nvim_diffblock_get_lnum(dp, idx);
    let diff_flags = nvim_get_diff_flags();

    if (diff_flags & ALL_INLINE_DIFF) == 0 {
        // Simple algorithm
        let mut change_start: c_int = MAXCOL;
        let mut change_end: c_int = -1;

        let ret = rs_diff_find_change_simple(
            wp,
            lnum,
            dp,
            idx,
            &raw mut change_start,
            &raw mut change_end,
        );

        // Convert from inclusive end to exclusive end
        change_end += 1;

        // Get the simple_diffline_change sentinel and fill it
        let simple_change = nvim_diff_get_simple_change();

        // Write diffline struct
        let dl = &mut *(diffline.cast::<DifflineRepr>());
        dl.changes = simple_change;
        dl.num_changes = 1;
        dl.bufidx = idx;
        dl.lineoff = off;

        // Zero and fill the simple change struct
        let sc = &mut *(simple_change.cast::<DifflineChangeRepr>());
        *sc = std::mem::zeroed();
        sc.dc_start[idx as usize] = change_start;
        sc.dc_end[idx as usize] = change_end;
        sc.dc_start_lnum_off[idx as usize] = off;
        sc.dc_end_lnum_off[idx as usize] = off;

        return ret;
    }

    // Inline diff algorithm
    if !nvim_diffblock_get_has_changes(dp) {
        crate::inline_compute::rs_compute_inline_diff(dp);
    }

    let changes_len = nvim_diffblock_get_changes_len(dp);

    // Linear search for changes on this line
    let mut num_changes: c_int = 0;
    let mut first_change: DiffLineChangeHandle = std::ptr::null_mut();
    let mut last_change_idx: c_int = 0;

    for change_idx in 0..changes_len {
        let change = nvim_diffblock_get_change(dp, change_idx);
        if change.is_null() {
            continue;
        }

        let end_lnum_off = nvim_diffchange_get_end_lnum_off(change, idx);
        let start_lnum_off = nvim_diffchange_get_start_lnum_off(change, idx);

        if end_lnum_off < off {
            continue;
        }
        if start_lnum_off > off {
            last_change_idx = change_idx;
            break;
        }
        if first_change.is_null() {
            first_change = change;
        }
        num_changes += 1;
        last_change_idx = change_idx + 1;
    }

    // Write diffline struct
    let dl = &mut *(diffline.cast::<DifflineRepr>());
    dl.changes = first_change;
    dl.num_changes = num_changes;
    dl.bufidx = idx;
    dl.lineoff = off;

    // Detect added lines
    let mut added = false;
    if num_changes == 1 && last_change_idx == changes_len {
        added = true;
        let last_change = nvim_diffblock_get_change(dp, changes_len - 1);
        for i in 0..DB_COUNT {
            if i == idx {
                continue;
            }
            if nvim_get_curtab_diffbuf(i).is_null() {
                continue;
            }
            if nvim_diffchange_get_start_lnum_off(last_change, i) != i32::MAX {
                added = false;
                break;
            }
        }
    }

    added
}

// =============================================================================
// Phase 6: Ex command migrations
// =============================================================================

/// Rust implementation of `ex_diffupdate`.
///
/// Completely updates the diffs for the buffers involved in the current tab.
#[export_name = "ex_diffupdate"]
pub unsafe extern "C" fn rs_diff_ex_diffupdate(eap: ExargHandle) {
    // If busy, defer the update.
    if nvim_diff_get_busy() {
        nvim_diff_set_need_update(true);
        return;
    }

    let curtab = nvim_get_curtab();
    let had_diffs = !nvim_tabpage_get_first_diff(curtab).is_null();

    // Delete all diffblocks.
    rs_diff_clear(curtab);
    nvim_tabpage_set_diff_invalid(curtab, 0);

    // Use the first buffer as the original text.
    let mut idx_orig: c_int = 0;
    while idx_orig < DB_COUNT {
        if !nvim_get_curtab_diffbuf(idx_orig).is_null() {
            break;
        }
        idx_orig += 1;
    }

    if idx_orig < DB_COUNT {
        // Only need to do something when there is another buffer.
        let mut idx_new = idx_orig + 1;
        while idx_new < DB_COUNT {
            if !nvim_get_curtab_diffbuf(idx_new).is_null() {
                break;
            }
            idx_new += 1;
        }

        if idx_new < DB_COUNT {
            // Only use the internal method if it did not fail for one of the buffers.
            let use_internal =
                (nvim_get_diff_flags() & DIFF_INTERNAL) != 0 && nvim_is_diffexpr_empty();
            let dio = nvim_diffio_new(use_internal);

            rs_diff_try_update(dio, idx_orig, eap);
            nvim_diffio_free(dio);

            // Force updating cursor position on screen.
            nvim_diff_invalidate_cursor();
        }
    }

    // A redraw is needed if there were diffs and they were cleared, or there
    // are diffs now, which means they got updated.
    let curtab = nvim_get_curtab();
    if had_diffs || !nvim_tabpage_get_first_diff(curtab).is_null() {
        rs_diff_redraw(true);
        nvim_diff_fire_diffupdated();
    }
}

// =============================================================================
// Phase 6: parse_diffanchors migration
// =============================================================================

// C accessor declarations for parse_diffanchors.
#[allow(dead_code)]
extern "C" {
    fn nvim_diff_get_buf_dia(buf: BufHandle) -> *const c_char;
    fn nvim_diff_get_p_dia() -> *const c_char;
    fn nvim_diff_set_curwin_curbuf(wp: WinHandle);
    fn nvim_diff_restore_curwin_curbuf(old_curwin: WinHandle);
    fn nvim_diff_buf_get_ml_line_count(buf: BufHandle) -> LinenrT;
    fn nvim_diff_emsg_hidden_diff_anchors();
    fn nvim_diff_emsg_invrange();
    fn nvim_diff_semsg_too_many_anchors(max: c_int);
    fn nvim_diff_get_firstwin() -> WinHandle;
    fn nvim_win_get_w_p_diff(wp: WinHandle) -> bool;
    fn nvim_diff_emsg(msg: *const c_char);
}

extern "C" {
    #[link_name = "get_address"]
    fn rs_get_address(
        eap: *mut c_void,
        ptr: *mut *mut c_char,
        addr_type: c_int,
        skip: bool,
        silent: bool,
        to_other_file: c_int,
        address_count: c_int,
        errormsg: *mut *const c_char,
    ) -> LinenrT;
}

// ADDR_LINES constant (must match C).
const ADDR_LINES: c_int = 0;
// Comma character as c_char (i8 on Linux; b',' = 44, fits in i8).
const COMMA_CHAR: c_char = 44;
// MAXLNUM as i32.
const MAXLNUM_I32: LinenrT = i32::MAX;

/// Parse the diff anchors option for buffer `buf`.
///
/// If `check_only` is true, only validates syntax.
/// If `check_only` is false, resolves addresses and writes to `anchors`/`num_anchors`.
///
/// Returns `OK` on success, `FAIL` on error.
///
/// Replaces C `parse_diffanchors`.
///
/// # Safety
/// Calls C functions that access global Neovim state.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_diffanchors(
    check_only: bool,
    buf: BufHandle,
    anchors: *mut LinenrT,
    num_anchors: *mut c_int,
) -> c_int {
    // Get the option string: buf-local first, fall back to global.
    let buf_dia = nvim_diff_get_buf_dia(buf);
    let global_dia = nvim_diff_get_p_dia();
    let dia_ptr: *const c_char = if *buf_dia == 0 { global_dia } else { buf_dia };

    let orig_curwin = nvim_get_curwin();

    // Find the window to use for address resolution.
    let bufwin: WinHandle = if check_only {
        orig_curwin
    } else {
        // Find the first diff window for this buffer.
        let mut w = nvim_diff_get_firstwin();
        let mut found = WinHandle::null();
        while !w.is_null() {
            if nvim_win_get_w_buffer(w) == buf && nvim_win_get_w_p_diff(w) {
                found = w;
                break;
            }
            w = nvim_win_next(w);
        }
        if found.is_null() && *dia_ptr != 0 {
            // Buffer is hidden and anchors are specified: unsupported.
            nvim_diff_emsg_hidden_diff_anchors();
            return FAIL;
        }
        found
    };

    let max_anchors = nvim_diff_max_anchors();
    let mut i: c_int = 0;

    // Make a mutable copy of the pointer into the option string.
    // Safety: get_address advances the pointer but does not write through it for
    // read-only option strings accessed in check_only mode; and in full mode the
    // string comes from a mutable Neovim option buffer.
    let mut dia: *mut c_char = dia_ptr.cast_mut();

    while i < max_anchors && *dia != 0 {
        // Disallow empty values (leading comma).
        if *dia == COMMA_CHAR {
            return FAIL;
        }

        // Temporarily set curwin/curbuf for address resolution.
        if bufwin.is_null() {
            // No window (hidden buf with empty dia): loop body unreachable.
        } else {
            nvim_diff_set_curwin_curbuf(bufwin);
        }

        let mut errormsg: *const c_char = std::ptr::null();
        let lnum = rs_get_address(
            std::ptr::null_mut(),
            std::ptr::addr_of_mut!(dia),
            ADDR_LINES,
            check_only,
            true,
            0, // to_other_file = false
            1,
            std::ptr::addr_of_mut!(errormsg),
        );

        // Restore curwin/curbuf.
        nvim_diff_restore_curwin_curbuf(orig_curwin);

        if !errormsg.is_null() {
            nvim_diff_emsg(errormsg);
        }
        if dia.is_null() {
            // Error detected by get_address.
            return FAIL;
        }
        if *dia != COMMA_CHAR && *dia != 0 {
            return FAIL;
        }

        if !check_only
            && (lnum == MAXLNUM_I32 || lnum <= 0 || lnum > nvim_diff_buf_get_ml_line_count(buf) + 1)
        {
            nvim_diff_emsg_invrange();
            return FAIL;
        }

        if !anchors.is_null() {
            // Safety: i < max_anchors <= MAX_DIFF_ANCHORS which is 20, so i fits in usize.
            *anchors.add(i.unsigned_abs() as usize) = lnum;
        }

        if *dia == COMMA_CHAR {
            dia = dia.add(1);
        }

        i += 1;
    }

    if i == max_anchors && *dia != 0 {
        nvim_diff_semsg_too_many_anchors(max_anchors);
        return FAIL;
    }

    if !num_anchors.is_null() {
        *num_anchors = i;
    }

    OK
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
