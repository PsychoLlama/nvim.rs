//! Line modification functions for the memline system.
//!
//! This module provides Rust implementations for line modification operations
//! including append, delete, and replace. The core B-tree modification logic
//! remains in C due to its complexity, but helper functions and state management
//! are implemented here.
//!
//! # Line Modification Flow
//!
//! 1. **Append**: Insert a new line after a given line number
//! 2. **Delete**: Remove a line from the buffer
//! 3. **Replace**: Change the content of an existing line
//!
//! All modifications use the line cache (`ml_line_ptr`) for buffering and
//! eventually flush changes to the B-tree data blocks.
//!
//! # Block Splitting
//!
//! When a data block becomes full during append, it must be split. This module
//! provides helper functions for calculating split positions and managing the
//! resulting block allocations.

use std::ffi::{c_char, c_int, c_uint, c_void};

use crate::types::{
    BufHandle, ColNr, DataBlockHeader, LineNr, PointerBlockHeader, DATA_BLOCK_HEADER_SIZE,
    DB_INDEX_MASK, DB_MARKED, INDEX_SIZE, ML_ALLOCATED, ML_APPEND_MARK, ML_APPEND_NEW,
    ML_DEL_MESSAGE, ML_EMPTY, ML_LINE_DIRTY, ML_LOCKED_DIRTY, ML_LOCKED_POS,
};

// =============================================================================
// C Accessor Declarations
// =============================================================================

extern "C" {
    // -------------------------------------------------------------------------
    // Buffer Memline Accessors
    // -------------------------------------------------------------------------

    /// Get buffer's line count (`buf->b_ml.ml_line_count`)
    fn nvim_buf_get_ml_line_count(buf: *mut BufHandle) -> LineNr;

    /// Get buffer's ml_flags (`buf->b_ml.ml_flags`)
    fn nvim_buf_get_ml_flags(buf: *mut BufHandle) -> c_int;

    /// Set buffer's ml_flags (`buf->b_ml.ml_flags`)
    fn nvim_buf_set_ml_flags(buf: *mut BufHandle, flags: c_int);

    /// Get buffer's cached line number (`buf->b_ml.ml_line_lnum`)
    fn nvim_buf_get_ml_line_lnum(buf: *mut BufHandle) -> LineNr;

    /// Get buffer's cached line pointer (`buf->b_ml.ml_line_ptr`)
    fn nvim_buf_get_ml_line_ptr(buf: *mut BufHandle) -> *mut c_char;

    /// Check if buffer has a valid memfile (`buf->b_ml.ml_mfp != NULL`)
    fn nvim_buf_has_ml_mfp(buf: *mut BufHandle) -> c_int;

    /// Get buffer's ml_locked_low
    fn nvim_buf_get_ml_locked_low(buf: *mut BufHandle) -> LineNr;

    /// Get buffer's ml_locked_high
    fn nvim_buf_get_ml_locked_high(buf: *mut BufHandle) -> LineNr;

    /// Get current buffer (curbuf)
    fn nvim_get_curbuf() -> *mut BufHandle;

    /// Find data block containing a line (public wrapper around ml_find_line)
    fn nvim_ml_find_line(buf: *mut BufHandle, lnum: LineNr, action: c_int) -> *mut c_void;

    /// Get bh_data pointer from block header
    fn nvim_bhdr_get_bh_data(hp: *mut c_void) -> *mut c_void;

    // -------------------------------------------------------------------------
    // C Implementation Functions
    // -------------------------------------------------------------------------

    /// Append a line (C implementation)
    fn ml_append(lnum: LineNr, line: *mut c_char, len: ColNr, newfile: c_int) -> c_int;

    /// Append a line with flags (C implementation)
    fn ml_append_flags(lnum: LineNr, line: *mut c_char, len: ColNr, flags: c_int) -> c_int;

    /// Append a line to a specific buffer (C implementation)
    fn ml_append_buf(
        buf: *mut BufHandle,
        lnum: LineNr,
        line: *mut c_char,
        len: ColNr,
        newfile: c_int,
    ) -> c_int;

    /// Replace a line (C implementation)
    fn ml_replace(lnum: LineNr, line: *mut c_char, copy: c_int) -> c_int;

    /// Replace a line with explicit length (C implementation)
    fn ml_replace_len(lnum: LineNr, line: *mut c_char, len: usize, copy: c_int) -> c_int;

    /// Replace a line in a specific buffer (C implementation)
    fn ml_replace_buf(
        buf: *mut BufHandle,
        lnum: LineNr,
        line: *mut c_char,
        copy: c_int,
        noalloc: c_int,
    ) -> c_int;

    /// Delete a line (C implementation)
    fn ml_delete(lnum: LineNr) -> c_int;

    /// Delete a line with flags (C implementation)
    fn ml_delete_flags(lnum: LineNr, flags: c_int) -> c_int;

    /// Delete a line from a specific buffer (C implementation)
    fn ml_delete_buf(buf: *mut BufHandle, lnum: LineNr, message: c_int) -> c_int;

    /// Flush cached line to data block (C implementation)
    fn ml_flush_line(buf: *mut BufHandle, noalloc: c_int);

    /// Add deleted length tracking (C implementation)
    fn ml_add_deleted_len(ptr: *mut c_char, len: isize);

    /// Add deleted length tracking for a specific buffer (C implementation)
    fn ml_add_deleted_len_buf(buf: *mut BufHandle, ptr: *mut c_char, len: isize);

    /// xfree wrapper
    fn xfree(ptr: *mut std::ffi::c_void);
}

// =============================================================================
// Mark Tracking State (Phase 1 migration)
// =============================================================================

/// The lowest line number where a mark may exist (0 means no marks).
///
/// This mirrors the C static `lowest_marked`. It is only used for curbuf
/// (the :global command never changes buffers while marks are live).
///
/// # Safety
/// Only written from the main Nvim thread. No concurrent access.
static mut LOWEST_MARKED: LineNr = 0;

/// Get the value of `LOWEST_MARKED`.
///
/// # Safety
/// Must only be called from the main Nvim thread.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_get_lowest_marked() -> LineNr {
    LOWEST_MARKED
}

/// Set the value of `LOWEST_MARKED`.
///
/// # Safety
/// Must only be called from the main Nvim thread.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_set_lowest_marked(lnum: LineNr) {
    LOWEST_MARKED = lnum;
}

/// Adjust `LOWEST_MARKED` after inserting a line at `lnum`.
///
/// Implements: `if (lowest_marked && lowest_marked > lnum) { lowest_marked = lnum + 1; }`
///
/// # Safety
/// Must only be called from the main Nvim thread.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_adjust_lowest_marked_for_insert(lnum: LineNr) {
    if LOWEST_MARKED != 0 && LOWEST_MARKED > lnum {
        LOWEST_MARKED = lnum + 1;
    }
}

/// Adjust `LOWEST_MARKED` after deleting a line at `lnum`.
///
/// Implements: `if (lowest_marked && lowest_marked > lnum) { lowest_marked--; }`
///
/// # Safety
/// Must only be called from the main Nvim thread.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_adjust_lowest_marked_for_delete(lnum: LineNr) {
    if LOWEST_MARKED != 0 && LOWEST_MARKED > lnum {
        LOWEST_MARKED -= 1;
    }
}

/// Get a mutable pointer to the db_index array inside a data block.
///
/// The db_index array immediately follows the `DataBlockHeader` in memory,
/// matching the C flexible array member `db_index[]`.
///
/// # Safety
/// - `dp` must be a valid pointer to a `DataBlock` (DataBlockHeader + db_index[])
#[inline]
unsafe fn db_index_ptr(dp: *mut c_void) -> *mut u32 {
    #[allow(clippy::cast_ptr_alignment)]
    dp.cast::<u8>()
        .add(DATA_BLOCK_HEADER_SIZE)
        .cast::<u32>()
}

extern "C" {
    // -------------------------------------------------------------------------
    // Phase 4: Block allocation (ml_new_ptr, ml_new_data)
    // -------------------------------------------------------------------------

    /// Allocate a new block in the memfile
    fn mf_new(
        mfp: *mut std::ffi::c_void,
        negative: bool,
        page_count: c_uint,
    ) -> *mut std::ffi::c_void;

    /// Get the page size of a memfile
    fn nvim_mf_get_page_size(mfp: *mut std::ffi::c_void) -> c_uint;
}

// =============================================================================
// Append Functions
// =============================================================================

/// Append a new line after a given line number in the current buffer.
///
/// This is a wrapper around the C `ml_append` function.
///
/// # Arguments
/// * `lnum` - Line number to append after (0 to insert at beginning)
/// * `line` - The line text (must be NUL-terminated)
/// * `len` - Length including NUL, or 0 to auto-calculate
/// * `newfile` - If true, this is a new file (for recovery tracking)
///
/// # Returns
/// OK (1) on success, FAIL (0) on failure
///
/// # Safety
/// - `line` must be a valid C string or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_append(
    lnum: LineNr,
    line: *mut c_char,
    len: ColNr,
    newfile: c_int,
) -> c_int {
    ml_append(lnum, line, len, newfile)
}

/// Append a new line with flags in the current buffer.
///
/// # Arguments
/// * `lnum` - Line number to append after (0 to insert at beginning)
/// * `line` - The line text (must be NUL-terminated)
/// * `len` - Length including NUL, or 0 to auto-calculate
/// * `flags` - ML_APPEND_NEW and/or ML_APPEND_MARK
///
/// # Returns
/// OK (1) on success, FAIL (0) on failure
///
/// # Safety
/// - `line` must be a valid C string or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_append_flags(
    lnum: LineNr,
    line: *mut c_char,
    len: ColNr,
    flags: c_int,
) -> c_int {
    ml_append_flags(lnum, line, len, flags)
}

/// Append a new line to a specific buffer.
///
/// The buffer must already have a memline initialized.
///
/// # Arguments
/// * `buf` - Buffer to append to
/// * `lnum` - Line number to append after (0 to insert at beginning)
/// * `line` - The line text (must be NUL-terminated)
/// * `len` - Length including NUL, or 0 to auto-calculate
/// * `newfile` - If true, this is a new file (for recovery tracking)
///
/// # Returns
/// OK (1) on success, FAIL (0) on failure
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
/// - `line` must be a valid C string or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_append_buf(
    buf: *mut BufHandle,
    lnum: LineNr,
    line: *mut c_char,
    len: ColNr,
    newfile: c_int,
) -> c_int {
    if buf.is_null() {
        return 0; // FAIL
    }
    ml_append_buf(buf, lnum, line, len, newfile)
}

// =============================================================================
// Replace Functions
// =============================================================================

/// Replace a line in the current buffer.
///
/// # Arguments
/// * `lnum` - Line number to replace
/// * `line` - The new line text (must be NUL-terminated)
/// * `copy` - If true, make a copy of the line; if false, takes ownership
///
/// # Returns
/// OK (1) on success, FAIL (0) on failure
///
/// # Safety
/// - `line` must be a valid C string or NULL
/// - If `copy` is false, the caller must not use `line` after this call
#[no_mangle]
pub unsafe extern "C" fn rs_ml_replace(lnum: LineNr, line: *mut c_char, copy: c_int) -> c_int {
    ml_replace(lnum, line, copy)
}

/// Replace a line with explicit length in the current buffer.
///
/// # Arguments
/// * `lnum` - Line number to replace
/// * `line` - The new line text
/// * `len` - Length of text, excluding NUL
/// * `copy` - If true, make a copy of the line; if false, takes ownership
///
/// # Returns
/// OK (1) on success, FAIL (0) on failure
///
/// # Safety
/// - `line` must be a valid pointer to at least `len` bytes
/// - If `copy` is false, the caller must not use `line` after this call
#[no_mangle]
pub unsafe extern "C" fn rs_ml_replace_len(
    lnum: LineNr,
    line: *mut c_char,
    len: usize,
    copy: c_int,
) -> c_int {
    ml_replace_len(lnum, line, len, copy)
}

/// Replace a line in a specific buffer.
///
/// # Arguments
/// * `buf` - Buffer to modify
/// * `lnum` - Line number to replace
/// * `line` - The new line text (must be NUL-terminated)
/// * `copy` - If true, make a copy of the line; if false, takes ownership
/// * `noalloc` - If true, flush immediately without allocating
///
/// # Returns
/// OK (1) on success, FAIL (0) on failure
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
/// - `line` must be a valid C string or NULL
/// - If `copy` is false, the caller must not use `line` after this call
#[no_mangle]
pub unsafe extern "C" fn rs_ml_replace_buf(
    buf: *mut BufHandle,
    lnum: LineNr,
    line: *mut c_char,
    copy: c_int,
    noalloc: c_int,
) -> c_int {
    if buf.is_null() {
        return 0; // FAIL
    }
    ml_replace_buf(buf, lnum, line, copy, noalloc)
}

// =============================================================================
// Delete Functions
// =============================================================================

/// Delete a line from the current buffer.
///
/// # Arguments
/// * `lnum` - Line number to delete
///
/// # Returns
/// OK (1) on success, FAIL (0) on failure
///
/// # Safety
/// Modifies buffer state.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_delete(lnum: LineNr) -> c_int {
    ml_delete(lnum)
}

/// Delete a line from the current buffer with flags.
///
/// # Arguments
/// * `lnum` - Line number to delete
/// * `flags` - ML_DEL_MESSAGE to show "No lines in buffer" message
///
/// # Returns
/// OK (1) on success, FAIL (0) on failure
///
/// # Safety
/// Modifies buffer state.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_delete_flags(lnum: LineNr, flags: c_int) -> c_int {
    ml_delete_flags(lnum, flags)
}

/// Delete a line from a specific buffer.
///
/// # Arguments
/// * `buf` - Buffer to modify
/// * `lnum` - Line number to delete
/// * `message` - If true, may show "No lines in buffer" message
///
/// # Returns
/// OK (1) on success, FAIL (0) on failure
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
/// - Modifies buffer state
#[no_mangle]
pub unsafe extern "C" fn rs_ml_delete_buf(
    buf: *mut BufHandle,
    lnum: LineNr,
    message: c_int,
) -> c_int {
    if buf.is_null() {
        return 0; // FAIL
    }
    ml_delete_buf(buf, lnum, message)
}

// =============================================================================
// Line Cache Functions
// =============================================================================

/// Flush the cached line to the data block.
///
/// This writes any pending changes to the line cache back to the B-tree.
///
/// # Arguments
/// * `buf` - Buffer to flush
/// * `noalloc` - If true, don't allocate, write directly
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_flush_line(buf: *mut BufHandle, noalloc: c_int) {
    if !buf.is_null() {
        ml_flush_line(buf, noalloc);
    }
}

/// Check if the buffer has a dirty cached line.
///
/// Returns true if there's a cached line that needs to be flushed.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_has_dirty_line(buf: *mut BufHandle) -> c_int {
    if buf.is_null() {
        return 0;
    }

    let flags = nvim_buf_get_ml_flags(buf);
    let lnum = nvim_buf_get_ml_line_lnum(buf);

    // Has dirty line if ml_line_lnum is set and ML_LINE_DIRTY flag is set
    c_int::from(lnum != 0 && (flags & ML_LINE_DIRTY) != 0)
}

/// Clear the cached line without flushing.
///
/// This discards any pending changes. Use with caution!
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_clear_cached_line(buf: *mut BufHandle) {
    if buf.is_null() {
        return;
    }

    let flags = nvim_buf_get_ml_flags(buf);
    let lnum = nvim_buf_get_ml_line_lnum(buf);

    if lnum != 0 && (flags & (ML_LINE_DIRTY | ML_ALLOCATED)) != 0 {
        let ptr = nvim_buf_get_ml_line_ptr(buf);
        if !ptr.is_null() {
            xfree(ptr.cast());
        }
    }

    // Clear the cache state (would need setter for ml_line_lnum)
    // Note: This is incomplete - would need more C accessors
}

// =============================================================================
// Deleted Text Tracking
// =============================================================================

/// Track deleted text length for the current buffer.
///
/// This is used for undo/redo and buffer update callbacks.
///
/// # Arguments
/// * `ptr` - Pointer to the deleted text
/// * `len` - Length of deleted text, or -1 to use strlen
///
/// # Safety
/// - `ptr` must be a valid C string
#[no_mangle]
pub unsafe extern "C" fn rs_ml_add_deleted_len(ptr: *mut c_char, len: isize) {
    ml_add_deleted_len(ptr, len);
}

/// Track deleted text length for a specific buffer.
///
/// # Arguments
/// * `buf` - Buffer to track for
/// * `ptr` - Pointer to the deleted text
/// * `len` - Length of deleted text, or -1 to use strlen
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
/// - `ptr` must be a valid C string
#[no_mangle]
pub unsafe extern "C" fn rs_ml_add_deleted_len_buf(
    buf: *mut BufHandle,
    ptr: *mut c_char,
    len: isize,
) {
    if !buf.is_null() {
        ml_add_deleted_len_buf(buf, ptr, len);
    }
}

// =============================================================================
// Line Marking (for :global command)
// =============================================================================

/// Set the DB_MARKED flag for a line (Rust implementation).
///
/// Used by the :global command to mark lines for later processing.
/// Mirrors the C `ml_setmarked` function.
///
/// # Arguments
/// * `lnum` - Line number to mark
///
/// # Safety
/// Modifies buffer state. Only call from main Nvim thread.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_setmarked(lnum: LineNr) {
    let buf = nvim_get_curbuf();
    if lnum < 1
        || lnum > nvim_buf_get_ml_line_count(buf)
        || nvim_buf_has_ml_mfp(buf) == 0
    {
        return;
    }

    if LOWEST_MARKED == 0 || LOWEST_MARKED > lnum {
        LOWEST_MARKED = lnum;
    }

    // Find the data block containing the line.
    let hp = nvim_ml_find_line(buf, lnum, crate::types::ML_FIND);
    if hp.is_null() {
        return;
    }
    let dp = nvim_bhdr_get_bh_data(hp);
    let db_idx = db_index_ptr(dp);
    #[allow(clippy::cast_sign_loss)]
    let i = (lnum - nvim_buf_get_ml_locked_low(buf)) as usize;
    *db_idx.add(i) |= DB_MARKED;

    // Mark the block dirty
    let flags = nvim_buf_get_ml_flags(buf);
    nvim_buf_set_ml_flags(buf, flags | ML_LOCKED_DIRTY);
}

/// Find the first marked line (Rust implementation).
///
/// Returns the line number of the first marked line, clearing its mark.
/// Mirrors the C `ml_firstmarked` function.
///
/// # Safety
/// Modifies buffer state. Only call from main Nvim thread.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_firstmarked() -> LineNr {
    let buf = nvim_get_curbuf();
    if nvim_buf_has_ml_mfp(buf) == 0 {
        return 0;
    }

    let mut lnum = LOWEST_MARKED;
    let line_count = nvim_buf_get_ml_line_count(buf);

    while lnum <= line_count {
        let hp = nvim_ml_find_line(buf, lnum, crate::types::ML_FIND);
        if hp.is_null() {
            return 0;
        }
        let dp = nvim_bhdr_get_bh_data(hp);
        let db_idx = db_index_ptr(dp);

        let locked_low = nvim_buf_get_ml_locked_low(buf);
        let locked_high = nvim_buf_get_ml_locked_high(buf);

        #[allow(clippy::cast_sign_loss)]
        let mut i = (lnum - locked_low) as usize;
        while lnum <= locked_high {
            if (*db_idx.add(i)) & DB_MARKED != 0 {
                *db_idx.add(i) &= DB_INDEX_MASK;
                let flags = nvim_buf_get_ml_flags(buf);
                nvim_buf_set_ml_flags(buf, flags | ML_LOCKED_DIRTY);
                LOWEST_MARKED = lnum + 1;
                return lnum;
            }
            i += 1;
            lnum += 1;
        }
    }

    0
}

/// Clear all marked lines (Rust implementation).
///
/// Removes the DB_MARKED flag from all lines.
/// Mirrors the C `ml_clearmarked` function.
///
/// # Safety
/// Modifies buffer state. Only call from main Nvim thread.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_clearmarked() {
    let buf = nvim_get_curbuf();
    if nvim_buf_has_ml_mfp(buf) == 0 {
        return;
    }

    let mut lnum = LOWEST_MARKED;
    let line_count = nvim_buf_get_ml_line_count(buf);

    while lnum <= line_count {
        let hp = nvim_ml_find_line(buf, lnum, crate::types::ML_FIND);
        if hp.is_null() {
            return;
        }
        let dp = nvim_bhdr_get_bh_data(hp);
        let db_idx = db_index_ptr(dp);

        let locked_low = nvim_buf_get_ml_locked_low(buf);
        let locked_high = nvim_buf_get_ml_locked_high(buf);

        #[allow(clippy::cast_sign_loss)]
        let mut i = (lnum - locked_low) as usize;
        while lnum <= locked_high {
            if (*db_idx.add(i)) & DB_MARKED != 0 {
                *db_idx.add(i) &= DB_INDEX_MASK;
                let flags = nvim_buf_get_ml_flags(buf);
                nvim_buf_set_ml_flags(buf, flags | ML_LOCKED_DIRTY);
            }
            i += 1;
            lnum += 1;
        }
    }

    LOWEST_MARKED = 0;
}

// =============================================================================
// Validation Helpers
// =============================================================================

/// Check if a line modification is valid.
///
/// Returns true if the line number is valid for modification in the buffer.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_can_modify(buf: *mut BufHandle, lnum: LineNr) -> c_int {
    if buf.is_null() {
        return 0;
    }

    // Must have memfile
    if nvim_buf_has_ml_mfp(buf) == 0 {
        return 0;
    }

    let line_count = nvim_buf_get_ml_line_count(buf);

    // Line number must be valid
    c_int::from(lnum >= 1 && lnum <= line_count)
}

/// Check if we can append after a line.
///
/// Append is valid for lnum 0 (beginning) through line_count.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_can_append(buf: *mut BufHandle, lnum: LineNr) -> c_int {
    if buf.is_null() {
        return 0;
    }

    // Must have memfile
    if nvim_buf_has_ml_mfp(buf) == 0 {
        return 0;
    }

    let line_count = nvim_buf_get_ml_line_count(buf);

    // Can append after line 0 through line_count
    c_int::from(lnum >= 0 && lnum <= line_count)
}

/// Check if the buffer is empty (only contains one empty line).
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_is_empty(buf: *mut BufHandle) -> c_int {
    if buf.is_null() {
        return 1; // Treat null as empty
    }

    let flags = nvim_buf_get_ml_flags(buf);
    c_int::from((flags & ML_EMPTY) != 0)
}

// =============================================================================
// Flag Constants for FFI
// =============================================================================

/// Get the ML_APPEND_NEW flag value.
#[no_mangle]
pub extern "C" fn rs_ml_append_new_flag() -> c_int {
    ML_APPEND_NEW
}

/// Get the ML_APPEND_MARK flag value.
#[no_mangle]
pub extern "C" fn rs_ml_append_mark_flag() -> c_int {
    ML_APPEND_MARK
}

/// Get the ML_DEL_MESSAGE flag value.
#[no_mangle]
pub extern "C" fn rs_ml_del_message_flag() -> c_int {
    ML_DEL_MESSAGE
}

// =============================================================================
// Append Operation Helpers
// =============================================================================

/// Calculate the space needed to insert a line.
///
/// This includes the text length plus the index entry size.
///
/// # Arguments
/// * `text_len` - Length of the line text including NUL terminator
#[no_mangle]
pub extern "C" fn rs_ml_space_needed(text_len: ColNr) -> c_int {
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let space = text_len as c_int + INDEX_SIZE as c_int;
    space
}

/// Calculate the db_idx for a line within a locked block.
///
/// # Arguments
/// * `lnum` - Line number (1-based), or 0 for prepending before line 1
/// * `locked_low` - First line number in the locked block
///
/// # Returns
/// Index into the data block (-1 if lnum is 0, meaning prepend)
#[no_mangle]
pub extern "C" fn rs_ml_calc_db_idx(lnum: LineNr, locked_low: LineNr) -> c_int {
    if lnum == 0 {
        -1
    } else {
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let idx = (lnum - locked_low) as c_int;
        idx
    }
}

/// Calculate the line count in a locked block.
///
/// # Arguments
/// * `locked_high` - Last line number in the locked block
/// * `locked_low` - First line number in the locked block
#[no_mangle]
pub extern "C" fn rs_ml_calc_line_count(locked_high: LineNr, locked_low: LineNr) -> c_int {
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let count = (locked_high - locked_low) as c_int;
    count
}

/// Check if a line should be inserted in the next block instead.
///
/// Returns true if:
/// - Not enough room in current block
/// - Appending to the last line in the block
/// - Not appending to the last line in the file
///
/// # Arguments
/// * `db_free` - Free space in the data block
/// * `space_needed` - Space needed for the new line
/// * `db_idx` - Current index in the block
/// * `line_count` - Number of lines in the block
/// * `lnum` - Line number being appended after
/// * `total_lines` - Total lines in the buffer
#[no_mangle]
pub extern "C" fn rs_ml_should_use_next_block(
    db_free: u32,
    space_needed: c_int,
    db_idx: c_int,
    line_count: c_int,
    lnum: LineNr,
    total_lines: LineNr,
) -> c_int {
    // Compare as i64 to avoid wrap issues
    let not_enough_room = i64::from(db_free) < i64::from(space_needed);
    let at_last_in_block = db_idx == line_count - 1;
    let not_at_file_end = lnum < total_lines;

    c_int::from(not_enough_room && at_last_in_block && not_at_file_end)
}

/// Calculate the offset for text insertion.
///
/// When inserting after db_idx, the offset is:
/// - db_txt_end if db_idx < 0 (prepending)
/// - db_index[db_idx] otherwise (position after the line we're appending to)
///
/// # Safety
/// - `db_index` must be a valid pointer if db_idx >= 0
#[no_mangle]
pub unsafe extern "C" fn rs_ml_get_insert_offset(
    db_index: *const u32,
    db_idx: c_int,
    db_txt_end: u32,
) -> u32 {
    if db_idx < 0 {
        db_txt_end
    } else if db_index.is_null() {
        0
    } else {
        #[allow(clippy::cast_sign_loss)]
        let entry = *db_index.add(db_idx as usize);
        entry & DB_INDEX_MASK
    }
}

/// Update a data block header after inserting a line.
///
/// # Safety
/// - `header` must be a valid pointer to a DataBlockHeader
#[no_mangle]
pub unsafe extern "C" fn rs_ml_db_insert_update_header(
    header: *mut DataBlockHeader,
    text_len: ColNr,
) {
    if header.is_null() {
        return;
    }
    #[allow(clippy::cast_sign_loss)]
    let len = text_len as u32;
    (*header).db_txt_start -= len;
    #[allow(clippy::cast_possible_truncation)]
    let space = len + INDEX_SIZE as u32;
    (*header).db_free -= space;
    (*header).db_line_count += 1;
}

/// Calculate flags to set after a successful append.
///
/// # Arguments
/// * `current_flags` - Current ml_flags value
/// * `append_flags` - Flags passed to ml_append (ML_APPEND_NEW, etc.)
///
/// # Returns
/// New flags value with ML_LOCKED_DIRTY set, and ML_LOCKED_POS set
/// unless ML_APPEND_NEW was specified
#[no_mangle]
pub extern "C" fn rs_ml_calc_append_flags(current_flags: c_int, append_flags: c_int) -> c_int {
    let mut flags = current_flags | ML_LOCKED_DIRTY;
    if (append_flags & ML_APPEND_NEW) == 0 {
        flags |= ML_LOCKED_POS;
    }
    flags
}

/// Clear the ML_EMPTY flag from buffer flags.
#[no_mangle]
pub extern "C" fn rs_ml_clear_empty_flag(flags: c_int) -> c_int {
    flags & !ML_EMPTY
}

// =============================================================================
// Block Split Calculations
// =============================================================================

/// Determine the split strategy when a block is full.
///
/// When inserting at db_idx:
/// - If db_idx < 0: new line goes in left (new) block
/// - Otherwise: calculate based on data distribution
///
/// # Arguments
/// * `db_idx` - Index where insertion happens (-1 for prepend)
/// * `line_count` - Current line count in block
/// * `db_free` - Free space in current block
/// * `space_needed` - Space needed for new line + any moved lines
///
/// # Returns
/// * 0 - Put new line in left block
/// * 1 - Put new line in right block
#[no_mangle]
pub extern "C" fn rs_ml_split_strategy(
    db_idx: c_int,
    _line_count: c_int,
    _db_free: u32,
    _space_needed: c_int,
) -> c_int {
    // Simplified: if prepending (db_idx < 0), put in left block (return 0)
    // Otherwise put in right block (return 1)
    // This matches the basic case in ml_append_int
    c_int::from(db_idx >= 0)
}

/// Calculate how many lines need to be moved to the right block.
///
/// When splitting, lines after db_idx move to the right block.
///
/// # Arguments
/// * `db_idx` - Index where insertion happens
/// * `line_count` - Current line count in block
#[no_mangle]
pub extern "C" fn rs_ml_lines_to_move(db_idx: c_int, line_count: c_int) -> c_int {
    if db_idx < 0 {
        0
    } else {
        line_count - db_idx - 1
    }
}

/// Calculate the data bytes to move to the right block.
///
/// This is the total bytes from db_txt_start to the start of line db_idx.
///
/// # Safety
/// - `db_index` must be a valid pointer if db_idx >= 0
#[no_mangle]
pub unsafe extern "C" fn rs_ml_data_to_move(
    db_index: *const u32,
    db_idx: c_int,
    db_txt_start: u32,
) -> c_int {
    if db_idx < 0 || db_index.is_null() {
        return 0;
    }
    #[allow(clippy::cast_sign_loss)]
    let line_start = (*db_index.add(db_idx as usize)) & DB_INDEX_MASK;
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let moved = (line_start - db_txt_start) as c_int;
    moved
}

/// Calculate total bytes to move (data + index entries).
///
/// # Arguments
/// * `data_moved` - Bytes of text data to move
/// * `lines_moved` - Number of lines to move
#[no_mangle]
pub extern "C" fn rs_ml_total_to_move(data_moved: c_int, lines_moved: c_int) -> c_int {
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let index_size = INDEX_SIZE as c_int;
    data_moved + lines_moved * index_size
}

/// Calculate the number of pages needed for a new block.
///
/// # Arguments
/// * `space_needed` - Total bytes needed
/// * `header_size` - Size of the block header
/// * `page_size` - Size of one page
#[no_mangle]
pub extern "C" fn rs_ml_pages_needed(
    space_needed: c_int,
    header_size: c_int,
    page_size: c_int,
) -> c_int {
    if page_size <= 0 {
        return 1;
    }
    (space_needed + header_size + page_size - 1) / page_size
}

// =============================================================================
// Delete Operation Helpers
// =============================================================================

/// Calculate the line count before deletion.
///
/// In delete context, count = locked_high - locked_low + 2
/// (different from append which uses locked_high - locked_low)
///
/// # Arguments
/// * `locked_high` - Last line number in the locked block
/// * `locked_low` - First line number in the locked block
#[no_mangle]
pub extern "C" fn rs_ml_delete_line_count(locked_high: LineNr, locked_low: LineNr) -> c_int {
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let count = (locked_high - locked_low + 2) as c_int;
    count
}

/// Calculate the size of a line in a data block.
///
/// The size includes the text and NUL terminator.
///
/// # Arguments
/// * `db_index` - Pointer to the db_index array
/// * `idx` - Index of the line to get size of
/// * `db_txt_end` - The db_txt_end value for line 0
///
/// # Safety
/// - `db_index` must be a valid pointer to an array
#[no_mangle]
pub unsafe extern "C" fn rs_ml_calc_line_size(
    db_index: *const u32,
    idx: c_int,
    db_txt_end: u32,
) -> c_int {
    if db_index.is_null() || idx < 0 {
        return 0;
    }

    #[allow(clippy::cast_sign_loss)]
    let line_start = (*db_index.add(idx as usize)) & DB_INDEX_MASK;

    let line_end = if idx == 0 {
        db_txt_end
    } else {
        #[allow(clippy::cast_sign_loss)]
        let prev = (*db_index.add((idx - 1) as usize)) & DB_INDEX_MASK;
        prev
    };

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let size = (line_end - line_start) as c_int;
    size
}

/// Update a data block header after deleting a line.
///
/// # Safety
/// - `header` must be a valid pointer to a DataBlockHeader
#[no_mangle]
pub unsafe extern "C" fn rs_ml_db_delete_update_header(
    header: *mut DataBlockHeader,
    line_size: c_int,
) {
    if header.is_null() || line_size <= 0 {
        return;
    }
    #[allow(clippy::cast_sign_loss)]
    let size = line_size as u32;
    #[allow(clippy::cast_possible_truncation)]
    let space = size + INDEX_SIZE as u32;
    (*header).db_free += space;
    (*header).db_txt_start += size;
    (*header).db_line_count -= 1;
}

/// Check if a data block becomes empty after deletion.
///
/// Returns true if there's only one line in the block.
#[no_mangle]
pub extern "C" fn rs_ml_block_becomes_empty(count: c_int) -> c_int {
    c_int::from(count == 1)
}

/// Calculate the flags to set after a successful delete.
///
/// After delete, both ML_LOCKED_DIRTY and ML_LOCKED_POS are set.
#[no_mangle]
pub extern "C" fn rs_ml_calc_delete_flags(current_flags: c_int) -> c_int {
    current_flags | ML_LOCKED_DIRTY | ML_LOCKED_POS
}

/// Check if the buffer becomes empty after delete.
///
/// Returns true if line_count == 1 (only the line being deleted).
#[no_mangle]
pub extern "C" fn rs_ml_buffer_becomes_empty(line_count: LineNr) -> c_int {
    c_int::from(line_count == 1)
}

// =============================================================================
// Replace Operation Helpers
// =============================================================================

/// Check if a replace operation is valid.
///
/// Replace is valid if the line number is within range and the buffer has a memfile.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_can_replace(buf: *mut BufHandle, lnum: LineNr) -> c_int {
    if buf.is_null() {
        return 0;
    }

    // Must have memfile
    if nvim_buf_has_ml_mfp(buf) == 0 {
        return 0;
    }

    let line_count = nvim_buf_get_ml_line_count(buf);

    // Line number must be valid (1 to line_count)
    c_int::from(lnum >= 1 && lnum <= line_count)
}

/// Calculate size difference when replacing a line.
///
/// Positive means new line is larger, negative means smaller.
///
/// # Arguments
/// * `old_len` - Length of the old line
/// * `new_len` - Length of the new line
#[no_mangle]
pub extern "C" fn rs_ml_replace_size_diff(old_len: ColNr, new_len: ColNr) -> c_int {
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let diff = new_len as c_int - old_len as c_int;
    diff
}

/// Check if a replace operation fits in the current block.
///
/// Returns true if the block has enough free space for the size difference.
///
/// # Arguments
/// * `db_free` - Free space in the data block
/// * `size_diff` - Size difference (new_len - old_len)
#[no_mangle]
pub extern "C" fn rs_ml_replace_fits_in_block(db_free: u32, size_diff: c_int) -> c_int {
    if size_diff <= 0 {
        // New line is same size or smaller - always fits
        return 1;
    }
    // New line is larger - check if we have room
    #[allow(clippy::cast_sign_loss)]
    let needed = size_diff as u32;
    c_int::from(db_free >= needed)
}

// =============================================================================
// Index Array Update Helpers
// =============================================================================

/// Shift index entries after deletion.
///
/// Moves indexes from `idx+1` onwards to fill the gap at `idx`,
/// adjusting each by adding `line_size` to account for text movement.
///
/// # Arguments
/// * `db_index` - Pointer to the db_index array
/// * `idx` - Index of the deleted line
/// * `count` - Original line count in block
/// * `line_size` - Size of the deleted line
///
/// # Safety
/// - `db_index` must be a valid pointer to a mutable array
#[no_mangle]
pub unsafe extern "C" fn rs_ml_shift_indexes_delete(
    db_index: *mut u32,
    idx: c_int,
    count: c_int,
    line_size: c_int,
) {
    if db_index.is_null() || idx < 0 || count <= 0 || line_size <= 0 {
        return;
    }

    // Shift indexes and adjust for text movement
    for i in idx..(count - 1) {
        #[allow(clippy::cast_sign_loss)]
        let next_idx = (i + 1) as usize;
        #[allow(clippy::cast_sign_loss)]
        let curr_idx = i as usize;
        #[allow(clippy::cast_sign_loss)]
        let adjustment = line_size as u32;
        *db_index.add(curr_idx) = (*db_index.add(next_idx)) + adjustment;
    }
}

/// Shift index entries after insertion.
///
/// Moves indexes from `idx+1` backwards to make room for a new entry at `idx+1`,
/// adjusting each by subtracting `text_len` to account for text movement.
///
/// # Arguments
/// * `db_index` - Pointer to the db_index array
/// * `db_idx` - Index after which we're inserting
/// * `line_count` - Original line count in block
/// * `text_len` - Length of the new line
///
/// # Safety
/// - `db_index` must be a valid pointer to a mutable array
#[no_mangle]
pub unsafe extern "C" fn rs_ml_shift_indexes_insert(
    db_index: *mut u32,
    db_idx: c_int,
    line_count: c_int,
    text_len: ColNr,
) {
    if db_index.is_null() {
        return;
    }

    // Move indexes backwards (from end to db_idx+1)
    for i in ((db_idx + 1)..line_count).rev() {
        #[allow(clippy::cast_sign_loss)]
        let curr_idx = i as usize;
        #[allow(clippy::cast_sign_loss)]
        let next_idx = (i + 1) as usize;
        #[allow(clippy::cast_sign_loss)]
        let adjustment = text_len as u32;
        *db_index.add(next_idx) = (*db_index.add(curr_idx)) - adjustment;
    }
}

/// Set the index entry for a newly inserted line.
///
/// # Arguments
/// * `db_index` - Pointer to the db_index array
/// * `idx` - Index where to store
/// * `offset` - Text offset for the new line
/// * `mark` - If non-zero, set the DB_MARKED flag
///
/// # Safety
/// - `db_index` must be a valid pointer to a mutable array
#[no_mangle]
pub unsafe extern "C" fn rs_ml_set_index_entry(
    db_index: *mut u32,
    idx: c_int,
    offset: u32,
    mark: c_int,
) {
    if db_index.is_null() || idx < 0 {
        return;
    }
    #[allow(clippy::cast_sign_loss)]
    let entry = db_index.add(idx as usize);
    *entry = offset;
    if mark != 0 {
        *entry |= !DB_INDEX_MASK;
    }
}

// =============================================================================
// Block Allocation Functions
// =============================================================================

/// Allocate a new data block in the memfile and initialize its header.
///
/// Sets `db_id`, `db_txt_start`, `db_txt_end`, `db_free`, and `db_line_count`.
///
/// # Safety
/// - `mfp` must be a valid memfile pointer
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_ml_new_data(
    mfp: *mut c_void,
    negative: bool,
    page_count: c_int,
) -> *mut c_void {
    debug_assert!(page_count >= 0);
    let hp = mf_new(mfp, negative, page_count as c_uint);
    let page_size = nvim_mf_get_page_size(mfp);
    let block_size = page_count as u32 * page_size;
    let header = DataBlockHeader::new(block_size);
    let dp = nvim_bhdr_get_bh_data(hp).cast::<DataBlockHeader>();
    std::ptr::write(dp, header);
    hp
}

/// Allocate a new pointer block in the memfile and initialize its header.
///
/// Sets `pb_id`, `pb_count` (0), and `pb_count_max`.
///
/// # Safety
/// - `mfp` must be a valid memfile pointer
#[no_mangle]
pub unsafe extern "C" fn rs_ml_new_ptr(mfp: *mut c_void) -> *mut c_void {
    let hp = mf_new(mfp, false, 1);
    let page_size = nvim_mf_get_page_size(mfp);
    let count_max = crate::types::pb_count_max(page_size as usize);
    let header = PointerBlockHeader::new(count_max);
    let pp = nvim_bhdr_get_bh_data(hp).cast::<PointerBlockHeader>();
    std::ptr::write(pp, header);
    hp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_constants() {
        assert_eq!(rs_ml_append_new_flag(), ML_APPEND_NEW);
        assert_eq!(rs_ml_append_mark_flag(), ML_APPEND_MARK);
        assert_eq!(rs_ml_del_message_flag(), ML_DEL_MESSAGE);
    }

    #[test]
    fn test_space_needed() {
        // 10 bytes of text + 4 bytes for index = 14
        assert_eq!(rs_ml_space_needed(10), 14);
    }

    #[test]
    fn test_calc_db_idx() {
        // Line 0 -> -1 (prepend)
        assert_eq!(rs_ml_calc_db_idx(0, 1), -1);
        // Line 5 with locked_low=3 -> 2
        assert_eq!(rs_ml_calc_db_idx(5, 3), 2);
        // Line 10 with locked_low=10 -> 0
        assert_eq!(rs_ml_calc_db_idx(10, 10), 0);
    }

    #[test]
    fn test_calc_line_count() {
        assert_eq!(rs_ml_calc_line_count(50, 40), 10);
        assert_eq!(rs_ml_calc_line_count(100, 1), 99);
    }

    #[test]
    fn test_should_use_next_block() {
        // Enough room - don't use next block
        assert_eq!(rs_ml_should_use_next_block(100, 50, 5, 10, 50, 100), 0);

        // Not enough room, at last in block, not at file end - use next block
        assert_eq!(rs_ml_should_use_next_block(30, 50, 9, 10, 50, 100), 1);

        // Not enough room, but not at last in block - don't use next block
        assert_eq!(rs_ml_should_use_next_block(30, 50, 5, 10, 50, 100), 0);

        // Not enough room, at last in block, but at file end - don't use next block
        assert_eq!(rs_ml_should_use_next_block(30, 50, 9, 10, 100, 100), 0);
    }

    #[test]
    fn test_get_insert_offset() {
        let db_index: [u32; 3] = [900, 850, 800];

        unsafe {
            // db_idx < 0 returns db_txt_end
            assert_eq!(rs_ml_get_insert_offset(db_index.as_ptr(), -1, 1000), 1000);

            // db_idx = 0 returns first entry
            assert_eq!(rs_ml_get_insert_offset(db_index.as_ptr(), 0, 1000), 900);

            // db_idx = 1 returns second entry
            assert_eq!(rs_ml_get_insert_offset(db_index.as_ptr(), 1, 1000), 850);
        }
    }

    #[test]
    fn test_calc_append_flags() {
        let base_flags = 0;

        // Without ML_APPEND_NEW, both DIRTY and POS are set
        let result = rs_ml_calc_append_flags(base_flags, 0);
        assert_eq!(result & ML_LOCKED_DIRTY, ML_LOCKED_DIRTY);
        assert_eq!(result & ML_LOCKED_POS, ML_LOCKED_POS);

        // With ML_APPEND_NEW, only DIRTY is set
        let result = rs_ml_calc_append_flags(base_flags, ML_APPEND_NEW);
        assert_eq!(result & ML_LOCKED_DIRTY, ML_LOCKED_DIRTY);
        assert_eq!(result & ML_LOCKED_POS, 0);
    }

    #[test]
    fn test_clear_empty_flag() {
        let flags = ML_EMPTY | ML_LINE_DIRTY;
        let result = rs_ml_clear_empty_flag(flags);
        assert_eq!(result & ML_EMPTY, 0);
        assert_eq!(result & ML_LINE_DIRTY, ML_LINE_DIRTY);
    }

    #[test]
    fn test_lines_to_move() {
        // Prepend: no lines to move
        assert_eq!(rs_ml_lines_to_move(-1, 10), 0);

        // Append at index 5 in block of 10: move 4 lines (6,7,8,9)
        assert_eq!(rs_ml_lines_to_move(5, 10), 4);

        // Append at last position: move 0 lines
        assert_eq!(rs_ml_lines_to_move(9, 10), 0);
    }

    #[test]
    fn test_total_to_move() {
        // 100 bytes of data, 5 lines = 100 + 5*4 = 120
        assert_eq!(rs_ml_total_to_move(100, 5), 120);
    }

    #[test]
    fn test_pages_needed() {
        // 1000 bytes + 24 header on 512 page = ceil(1024/512) = 2
        assert_eq!(rs_ml_pages_needed(1000, 24, 512), 2);

        // 500 bytes + 24 header on 512 page = ceil(524/512) = 2
        assert_eq!(rs_ml_pages_needed(500, 24, 512), 2);

        // 400 bytes + 24 header on 512 page = ceil(424/512) = 1
        assert_eq!(rs_ml_pages_needed(400, 24, 512), 1);
    }

    #[test]
    fn test_delete_line_count() {
        // locked_high=50, locked_low=40 -> 50-40+2 = 12
        assert_eq!(rs_ml_delete_line_count(50, 40), 12);
    }

    #[test]
    fn test_calc_line_size() {
        // Line 0: starts at 900, ends at db_txt_end=1000 -> size 100
        // Line 1: starts at 850, ends at 900 (prev entry) -> size 50
        let db_index: [u32; 3] = [900, 850, 800];

        unsafe {
            assert_eq!(rs_ml_calc_line_size(db_index.as_ptr(), 0, 1000), 100);
            assert_eq!(rs_ml_calc_line_size(db_index.as_ptr(), 1, 1000), 50);
            assert_eq!(rs_ml_calc_line_size(db_index.as_ptr(), 2, 1000), 50);
        }
    }

    #[test]
    fn test_block_becomes_empty() {
        assert_eq!(rs_ml_block_becomes_empty(1), 1);
        assert_eq!(rs_ml_block_becomes_empty(2), 0);
        assert_eq!(rs_ml_block_becomes_empty(0), 0);
    }

    #[test]
    fn test_buffer_becomes_empty() {
        assert_eq!(rs_ml_buffer_becomes_empty(1), 1);
        assert_eq!(rs_ml_buffer_becomes_empty(2), 0);
        assert_eq!(rs_ml_buffer_becomes_empty(100), 0);
    }

    #[test]
    fn test_replace_size_diff() {
        // Same size
        assert_eq!(rs_ml_replace_size_diff(10, 10), 0);
        // New is larger
        assert_eq!(rs_ml_replace_size_diff(10, 15), 5);
        // New is smaller
        assert_eq!(rs_ml_replace_size_diff(10, 5), -5);
    }

    #[test]
    fn test_replace_fits_in_block() {
        // Smaller or same always fits
        assert_eq!(rs_ml_replace_fits_in_block(50, 0), 1);
        assert_eq!(rs_ml_replace_fits_in_block(50, -10), 1);

        // Larger fits if enough room
        assert_eq!(rs_ml_replace_fits_in_block(50, 30), 1);
        assert_eq!(rs_ml_replace_fits_in_block(50, 50), 1);

        // Larger doesn't fit if not enough room
        assert_eq!(rs_ml_replace_fits_in_block(50, 51), 0);
        assert_eq!(rs_ml_replace_fits_in_block(50, 100), 0);
    }

    #[test]
    fn test_shift_indexes_delete() {
        let mut db_index: [u32; 4] = [900, 850, 800, 750];

        unsafe {
            // Delete line at idx=1, count=4, line_size=50
            // After: idx[1]=idx[2]+50=850, idx[2]=idx[3]+50=800
            rs_ml_shift_indexes_delete(db_index.as_mut_ptr(), 1, 4, 50);

            assert_eq!(db_index[0], 900); // unchanged
            assert_eq!(db_index[1], 850); // was 800+50
            assert_eq!(db_index[2], 800); // was 750+50
                                          // db_index[3] is now garbage
        }
    }

    #[test]
    fn test_set_index_entry() {
        let mut db_index: [u32; 2] = [0, 0];

        unsafe {
            // Set entry without mark
            rs_ml_set_index_entry(db_index.as_mut_ptr(), 0, 500, 0);
            assert_eq!(db_index[0], 500);

            // Set entry with mark
            rs_ml_set_index_entry(db_index.as_mut_ptr(), 1, 600, 1);
            assert_eq!(db_index[1] & DB_INDEX_MASK, 600);
            assert_ne!(db_index[1] & !DB_INDEX_MASK, 0);
        }
    }
}
