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

use std::ffi::{c_char, c_int};

use crate::types::{BufHandle, ColNr, LineNr, ML_ALLOCATED, ML_APPEND_MARK, ML_APPEND_NEW,
                   ML_DEL_MESSAGE, ML_EMPTY, ML_LINE_DIRTY};

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

    /// Get buffer's cached line number (`buf->b_ml.ml_line_lnum`)
    fn nvim_buf_get_ml_line_lnum(buf: *mut BufHandle) -> LineNr;

    /// Get buffer's cached line pointer (`buf->b_ml.ml_line_ptr`)
    fn nvim_buf_get_ml_line_ptr(buf: *mut BufHandle) -> *mut c_char;

    /// Check if buffer has a valid memfile (`buf->b_ml.ml_mfp != NULL`)
    fn nvim_buf_has_ml_mfp(buf: *mut BufHandle) -> c_int;

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

    /// Set marked flag on a line (C implementation)
    fn ml_setmarked(lnum: LineNr);

    /// Find first marked line (C implementation)
    fn ml_firstmarked() -> LineNr;

    /// Clear all marked lines (C implementation)
    fn ml_clearmarked();

    /// xfree wrapper
    fn xfree(ptr: *mut std::ffi::c_void);
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

/// Set the DB_MARKED flag for a line.
///
/// Used by the :global command to mark lines for later processing.
///
/// # Arguments
/// * `lnum` - Line number to mark
///
/// # Safety
/// Modifies buffer state.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_setmarked(lnum: LineNr) {
    ml_setmarked(lnum);
}

/// Find the first marked line.
///
/// Returns the line number of the first marked line, or 0 if none.
///
/// # Safety
/// Reads buffer state.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_firstmarked() -> LineNr {
    ml_firstmarked()
}

/// Clear all marked lines.
///
/// Removes the DB_MARKED flag from all lines.
///
/// # Safety
/// Modifies buffer state.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_clearmarked() {
    ml_clearmarked();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_constants() {
        assert_eq!(rs_ml_append_new_flag(), ML_APPEND_NEW);
        assert_eq!(rs_ml_append_mark_flag(), ML_APPEND_MARK);
        assert_eq!(rs_ml_del_message_flag(), ML_DEL_MESSAGE);
    }
}
