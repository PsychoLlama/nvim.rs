//! Line access functions for the memline system.
//!
//! This module provides Rust implementations for accessing lines in buffers.
//! The core implementation remains in C (`ml_get_buf_impl`) due to its deep
//! integration with memfile operations, but helper functions and accessors
//! are implemented here.
//!
//! # Line Caching
//!
//! The memline caches the most recently accessed line in `ml_line_ptr` with
//! its line number in `ml_line_lnum`. This cache is invalidated on buffer
//! changes and when switching to a different line.
//!
//! # Data Block Line Access
//!
//! Lines in a data block are stored in reverse order. The first line's text
//! is at the end of the block. Each line has an index entry (`db_index[i]`)
//! that stores the start offset of that line's text. The text ends at the
//! previous line's start offset (or at `db_txt_end` for the first line).

use std::ffi::{c_char, c_int};
use std::ptr;

use crate::types::{
    BufHandle, ColNr, DataBlockHeader, LineNr, PosHandle, DB_INDEX_MASK, ML_ALLOCATED,
    ML_LINE_DIRTY,
};

// =============================================================================
// C Accessor Declarations
// =============================================================================

extern "C" {
    // -------------------------------------------------------------------------
    // Global State
    // -------------------------------------------------------------------------

    /// Get the current buffer (`curbuf`)
    fn nvim_get_curbuf() -> *mut BufHandle;

    // -------------------------------------------------------------------------
    // Buffer Memline Accessors
    // -------------------------------------------------------------------------

    /// Get buffer's line count (`buf->b_ml.ml_line_count`)
    fn nvim_buf_get_ml_line_count(buf: *mut BufHandle) -> LineNr;

    /// Get buffer's ml_flags (`buf->b_ml.ml_flags`)
    fn nvim_buf_get_ml_flags(buf: *mut BufHandle) -> c_int;

    /// Get buffer's cached line number (`buf->b_ml.ml_line_lnum`)
    fn nvim_buf_get_ml_line_lnum(buf: *mut BufHandle) -> LineNr;

    /// Get buffer's cached line length (`buf->b_ml.ml_line_len`)
    fn nvim_buf_get_ml_line_len(buf: *mut BufHandle) -> ColNr;

    /// Get buffer's cached line pointer (`buf->b_ml.ml_line_ptr`)
    fn nvim_buf_get_ml_line_ptr(buf: *mut BufHandle) -> *mut c_char;

    // Note: nvim_buf_set_ml_line_len is available in C but not currently used

    // -------------------------------------------------------------------------
    // Position Accessors
    // -------------------------------------------------------------------------

    /// Get line number from position (`pos->lnum`)
    fn nvim_pos_get_lnum(pos: *const PosHandle) -> LineNr;

    /// Get column from position (`pos->col`)
    fn nvim_pos_get_col(pos: *const PosHandle) -> ColNr;

    // -------------------------------------------------------------------------
    // C Implementation Functions
    // -------------------------------------------------------------------------

    /// Get a line from curbuf (C implementation)
    fn ml_get(lnum: LineNr) -> *mut c_char;

    /// Get a line from a specific buffer (C implementation)
    fn ml_get_buf(buf: *mut BufHandle, lnum: LineNr) -> *mut c_char;

    // Note: ml_get_buf_mut is available but not currently used from Rust

    /// Get UTF-8 character from string (in mbyte.c)
    fn utf_ptr2char(p: *const c_char) -> c_int;

    /// MAXCOL constant
    fn nvim_get_maxcol() -> ColNr;
}

// =============================================================================
// Line Length Functions
// =============================================================================

/// Get the length (excluding NUL) of a line in the current buffer.
///
/// This calls `ml_get_buf` to ensure the line is cached, then returns
/// the cached length minus one (to exclude the NUL terminator).
///
/// # Safety
/// Calls C functions that access buffer state.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_get_len(lnum: LineNr) -> ColNr {
    rs_ml_get_buf_len(nvim_get_curbuf(), lnum)
}

/// Get the length (excluding NUL) of a line in a specific buffer.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
/// - Calls C functions that access buffer state
#[no_mangle]
pub unsafe extern "C" fn rs_ml_get_buf_len(buf: *mut BufHandle, lnum: LineNr) -> ColNr {
    if buf.is_null() {
        return 0;
    }

    // Call ml_get_buf to ensure the line is cached
    let line = ml_get_buf(buf, lnum);
    if line.is_null() {
        return 0;
    }

    // Check if the line is empty
    if *line == 0 {
        return 0;
    }

    // Return cached length minus NUL
    nvim_buf_get_ml_line_len(buf) - 1
}

/// Get the length (excluding NUL) of text after a position.
///
/// Returns the length of the line minus the column offset.
///
/// # Safety
/// - `pos` must be a valid position pointer or NULL
/// - Calls C functions that access buffer state
#[no_mangle]
pub unsafe extern "C" fn rs_ml_get_pos_len(pos: *mut PosHandle) -> ColNr {
    if pos.is_null() {
        return 0;
    }

    let curbuf = nvim_get_curbuf();
    let lnum = nvim_pos_get_lnum(pos);
    let col = nvim_pos_get_col(pos);

    rs_ml_get_buf_len(curbuf, lnum) - col
}

// =============================================================================
// Line Access Functions
// =============================================================================

/// Get a pointer to a position in the current buffer.
///
/// Returns a pointer to the character at `pos->col` in line `pos->lnum`.
///
/// # Safety
/// - `pos` must be a valid position pointer or NULL
/// - Calls C functions that access buffer state
/// - The returned pointer is valid only until the next buffer modification
#[no_mangle]
pub unsafe extern "C" fn rs_ml_get_pos(pos: *const PosHandle) -> *mut c_char {
    if pos.is_null() {
        return ptr::null_mut();
    }

    let curbuf = nvim_get_curbuf();
    let lnum = nvim_pos_get_lnum(pos);
    let col = nvim_pos_get_col(pos);

    let line = ml_get_buf(curbuf, lnum);
    if line.is_null() {
        return ptr::null_mut();
    }

    #[allow(clippy::cast_sign_loss)]
    line.add(col as usize)
}

/// Get the codepoint at a position.
///
/// Returns NUL (0) if the position is at or past the end of the line.
///
/// # Safety
/// - `pos` must be a valid position pointer or NULL
/// - Calls C functions that access buffer state
#[no_mangle]
pub unsafe extern "C" fn rs_gchar_pos(pos: *mut PosHandle) -> c_int {
    if pos.is_null() {
        return 0; // NUL
    }

    let col = nvim_pos_get_col(pos);
    let maxcol = nvim_get_maxcol();

    // Check for end-of-line conditions
    if col == maxcol {
        return 0; // NUL
    }

    let lnum = nvim_pos_get_lnum(pos);
    let line_len = rs_ml_get_len(lnum);

    if col > line_len {
        return 0; // NUL
    }

    let line_ptr = rs_ml_get_pos(pos);
    if line_ptr.is_null() {
        return 0; // NUL
    }

    utf_ptr2char(line_ptr)
}

// =============================================================================
// Cache Status Functions
// =============================================================================

/// Check if a specific line is currently cached in a buffer.
///
/// Returns true if `ml_line_lnum` matches the given line number.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_line_cached(buf: *mut BufHandle, lnum: LineNr) -> c_int {
    if buf.is_null() {
        return 0;
    }

    c_int::from(nvim_buf_get_ml_line_lnum(buf) == lnum)
}

/// Get the currently cached line number for a buffer.
///
/// Returns 0 if no line is cached.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_get_cached_lnum(buf: *mut BufHandle) -> LineNr {
    if buf.is_null() {
        return 0;
    }

    nvim_buf_get_ml_line_lnum(buf)
}

/// Get the length of the currently cached line for a buffer.
///
/// Returns 0 if no line is cached.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_get_cached_len(buf: *mut BufHandle) -> ColNr {
    if buf.is_null() {
        return 0;
    }

    nvim_buf_get_ml_line_len(buf)
}

/// Get the pointer to the currently cached line for a buffer.
///
/// Returns NULL if no line is cached.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
/// - The returned pointer is valid only until the next buffer modification
#[no_mangle]
pub unsafe extern "C" fn rs_ml_get_cached_ptr(buf: *mut BufHandle) -> *mut c_char {
    if buf.is_null() {
        return ptr::null_mut();
    }

    nvim_buf_get_ml_line_ptr(buf)
}

// =============================================================================
// Line Count Functions
// =============================================================================

/// Get the number of lines in a buffer.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_get_line_count(buf: *mut BufHandle) -> LineNr {
    if buf.is_null() {
        return 0;
    }

    nvim_buf_get_ml_line_count(buf)
}

/// Check if a line number is valid for a buffer.
///
/// A line number is valid if it's between 1 and the buffer's line count (inclusive).
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_valid_lnum(buf: *mut BufHandle, lnum: LineNr) -> c_int {
    if buf.is_null() {
        return 0;
    }

    let line_count = nvim_buf_get_ml_line_count(buf);
    c_int::from(lnum >= 1 && lnum <= line_count)
}

// =============================================================================
// Line Empty Check
// =============================================================================

/// Check if a line is empty in the current buffer.
///
/// A line is empty if its first character is NUL.
///
/// # Safety
/// Calls C functions that access buffer state.
#[no_mangle]
pub unsafe extern "C" fn rs_line_empty(lnum: LineNr) -> c_int {
    let line = ml_get(lnum);
    if line.is_null() {
        return 1; // Treat as empty
    }

    c_int::from(*line == 0)
}

/// Check if a line is empty in a specific buffer.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
/// - Calls C functions that access buffer state
#[no_mangle]
pub unsafe extern "C" fn rs_line_empty_buf(buf: *mut BufHandle, lnum: LineNr) -> c_int {
    if buf.is_null() {
        return 1; // Treat as empty
    }

    let line = ml_get_buf(buf, lnum);
    if line.is_null() {
        return 1; // Treat as empty
    }

    c_int::from(*line == 0)
}

// =============================================================================
// Allocation Status
// =============================================================================

/// Check if the cached line for a buffer is in allocated memory.
///
/// Returns true if `ML_LINE_DIRTY` or `ML_ALLOCATED` is set.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_line_alloced_buf(buf: *mut BufHandle) -> c_int {
    if buf.is_null() {
        return 0;
    }

    let flags = nvim_buf_get_ml_flags(buf);
    c_int::from((flags & (ML_LINE_DIRTY | ML_ALLOCATED)) != 0)
}

// =============================================================================
// Data Block Line Access Helpers
// =============================================================================

/// Get the text start offset for a line in a data block.
///
/// The index array stores the start offset of each line's text, with the
/// high bit potentially used for marking (DB_MARKED).
///
/// # Arguments
/// * `db_index` - Pointer to the db_index array
/// * `idx` - Index of the line (0-based within the block)
///
/// # Returns
/// Start offset of the line's text within the block
///
/// # Safety
/// - `db_index` must be a valid pointer to an array
/// - `idx` must be within bounds
#[no_mangle]
pub unsafe extern "C" fn rs_ml_db_get_line_start(db_index: *const u32, idx: c_int) -> u32 {
    if db_index.is_null() || idx < 0 {
        return 0;
    }
    #[allow(clippy::cast_sign_loss)]
    let entry = *db_index.add(idx as usize);
    entry & DB_INDEX_MASK
}

/// Get the text end offset for a line in a data block.
///
/// The text of line `idx` ends where the text of line `idx-1` starts,
/// or at `db_txt_end` for line 0.
///
/// # Arguments
/// * `db_index` - Pointer to the db_index array
/// * `idx` - Index of the line (0-based within the block)
/// * `db_txt_end` - The db_txt_end value from the block header
///
/// # Returns
/// End offset of the line's text within the block
///
/// # Safety
/// - `db_index` must be a valid pointer to an array
/// - `idx` must be within bounds
#[no_mangle]
pub unsafe extern "C" fn rs_ml_db_get_line_end(
    db_index: *const u32,
    idx: c_int,
    db_txt_end: u32,
) -> u32 {
    if db_index.is_null() || idx < 0 {
        return 0;
    }
    if idx == 0 {
        db_txt_end
    } else {
        #[allow(clippy::cast_sign_loss)]
        let prev_entry = *db_index.add((idx - 1) as usize);
        prev_entry & DB_INDEX_MASK
    }
}

/// Calculate the length of a line in a data block.
///
/// The length is the difference between the end and start offsets.
///
/// # Arguments
/// * `db_index` - Pointer to the db_index array
/// * `idx` - Index of the line (0-based within the block)
/// * `db_txt_end` - The db_txt_end value from the block header
///
/// # Returns
/// Length of the line including NUL terminator
///
/// # Safety
/// - `db_index` must be a valid pointer to an array
/// - `idx` must be within bounds
#[no_mangle]
pub unsafe extern "C" fn rs_ml_db_get_line_len(
    db_index: *const u32,
    idx: c_int,
    db_txt_end: u32,
) -> ColNr {
    let start = rs_ml_db_get_line_start(db_index, idx);
    let end = rs_ml_db_get_line_end(db_index, idx, db_txt_end);

    if end >= start {
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let len = (end - start) as ColNr;
        len
    } else {
        0
    }
}

/// Check if a line is marked in the data block index.
///
/// The high bit of db_index is used for marking lines (e.g., by the :global command).
///
/// # Safety
/// - `db_index` must be a valid pointer to an array
/// - `idx` must be within bounds
#[no_mangle]
pub unsafe extern "C" fn rs_ml_db_line_is_marked(db_index: *const u32, idx: c_int) -> c_int {
    if db_index.is_null() || idx < 0 {
        return 0;
    }
    #[allow(clippy::cast_sign_loss)]
    let entry = *db_index.add(idx as usize);
    c_int::from((entry & !DB_INDEX_MASK) != 0)
}

/// Set or clear the mark on a line in the data block index.
///
/// # Arguments
/// * `db_index` - Pointer to the db_index array
/// * `idx` - Index of the line
/// * `mark` - Non-zero to set mark, zero to clear
///
/// # Safety
/// - `db_index` must be a valid pointer to a mutable array
/// - `idx` must be within bounds
#[no_mangle]
pub unsafe extern "C" fn rs_ml_db_set_line_mark(db_index: *mut u32, idx: c_int, mark: c_int) {
    if db_index.is_null() || idx < 0 {
        return;
    }
    #[allow(clippy::cast_sign_loss)]
    let entry = db_index.add(idx as usize);
    if mark != 0 {
        *entry |= !DB_INDEX_MASK;
    } else {
        *entry &= DB_INDEX_MASK;
    }
}

/// Calculate the index within a data block for a given line number.
///
/// # Arguments
/// * `lnum` - The line number (1-based)
/// * `locked_low` - The first line number in the locked block
///
/// # Returns
/// Index into the data block (0-based)
#[no_mangle]
pub extern "C" fn rs_ml_lnum_to_db_idx(lnum: LineNr, locked_low: LineNr) -> c_int {
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let idx = (lnum - locked_low) as c_int;
    idx
}

/// Calculate the line number from a data block index.
///
/// # Arguments
/// * `idx` - Index into the data block (0-based)
/// * `locked_low` - The first line number in the locked block
///
/// # Returns
/// Line number (1-based)
#[no_mangle]
pub extern "C" fn rs_ml_db_idx_to_lnum(idx: c_int, locked_low: LineNr) -> LineNr {
    locked_low + LineNr::from(idx)
}

// =============================================================================
// Data Block Header Accessors
// =============================================================================

/// Get a pointer to the db_index array from a data block.
///
/// The index array starts immediately after the DataBlockHeader.
///
/// # Safety
/// - `header` must be a valid pointer to a DataBlockHeader
#[no_mangle]
pub unsafe extern "C" fn rs_ml_db_get_index_ptr(header: *const DataBlockHeader) -> *const u32 {
    if header.is_null() {
        return ptr::null();
    }
    // The index array follows immediately after the header
    header.add(1).cast()
}

/// Get a mutable pointer to the db_index array from a data block.
///
/// # Safety
/// - `header` must be a valid pointer to a DataBlockHeader
#[no_mangle]
pub unsafe extern "C" fn rs_ml_db_get_index_ptr_mut(header: *mut DataBlockHeader) -> *mut u32 {
    if header.is_null() {
        return ptr::null_mut();
    }
    header.add(1).cast()
}

/// Get a pointer to the text area of a data block.
///
/// # Arguments
/// * `block_ptr` - Pointer to the start of the data block
/// * `offset` - Offset from the start of the block
///
/// # Safety
/// - `block_ptr` must be a valid pointer
/// - `offset` must be within the block
#[no_mangle]
pub unsafe extern "C" fn rs_ml_db_get_text_ptr(
    block_ptr: *const c_char,
    offset: u32,
) -> *const c_char {
    if block_ptr.is_null() {
        return ptr::null();
    }
    #[allow(clippy::cast_sign_loss)]
    block_ptr.add(offset as usize)
}

/// Check if a data block has room for a new line.
///
/// # Arguments
/// * `header` - Pointer to the data block header
/// * `text_len` - Length of the text to add (including NUL)
///
/// # Returns
/// Non-zero if there's room, zero otherwise
///
/// # Safety
/// - `header` must be a valid pointer to a DataBlockHeader
#[no_mangle]
pub unsafe extern "C" fn rs_ml_db_has_room(header: *const DataBlockHeader, text_len: u32) -> c_int {
    if header.is_null() {
        return 0;
    }
    // Space needed: text length + one index entry (u32 = 4 bytes)
    let space_needed = text_len + 4;
    c_int::from((*header).db_free >= space_needed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lnum_to_db_idx() {
        // Line 100 with locked_low=95 should give index 5
        assert_eq!(rs_ml_lnum_to_db_idx(100, 95), 5);
        // Line 1 with locked_low=1 should give index 0
        assert_eq!(rs_ml_lnum_to_db_idx(1, 1), 0);
    }

    #[test]
    fn test_db_idx_to_lnum() {
        assert_eq!(rs_ml_db_idx_to_lnum(5, 95), 100);
        assert_eq!(rs_ml_db_idx_to_lnum(0, 1), 1);
    }

    #[test]
    fn test_db_line_offsets() {
        // Simulate a data block with 3 lines
        // Line 0: offset 900 (ends at 1000 = db_txt_end)
        // Line 1: offset 850 (ends at 900)
        // Line 2: offset 800 (ends at 850)
        let db_index: [u32; 3] = [900, 850, 800];
        let db_txt_end: u32 = 1000;

        unsafe {
            // Line 0: start=900, end=1000, len=100
            assert_eq!(rs_ml_db_get_line_start(db_index.as_ptr(), 0), 900);
            assert_eq!(rs_ml_db_get_line_end(db_index.as_ptr(), 0, db_txt_end), 1000);
            assert_eq!(rs_ml_db_get_line_len(db_index.as_ptr(), 0, db_txt_end), 100);

            // Line 1: start=850, end=900, len=50
            assert_eq!(rs_ml_db_get_line_start(db_index.as_ptr(), 1), 850);
            assert_eq!(rs_ml_db_get_line_end(db_index.as_ptr(), 1, db_txt_end), 900);
            assert_eq!(rs_ml_db_get_line_len(db_index.as_ptr(), 1, db_txt_end), 50);

            // Line 2: start=800, end=850, len=50
            assert_eq!(rs_ml_db_get_line_start(db_index.as_ptr(), 2), 800);
            assert_eq!(rs_ml_db_get_line_end(db_index.as_ptr(), 2, db_txt_end), 850);
            assert_eq!(rs_ml_db_get_line_len(db_index.as_ptr(), 2, db_txt_end), 50);
        }
    }

    #[test]
    fn test_db_line_mark() {
        let mut db_index: [u32; 2] = [500, 400];

        unsafe {
            // Initially not marked
            assert_eq!(rs_ml_db_line_is_marked(db_index.as_ptr(), 0), 0);

            // Set mark
            rs_ml_db_set_line_mark(db_index.as_mut_ptr(), 0, 1);
            assert_eq!(rs_ml_db_line_is_marked(db_index.as_ptr(), 0), 1);

            // Verify offset is still correct
            assert_eq!(rs_ml_db_get_line_start(db_index.as_ptr(), 0), 500);

            // Clear mark
            rs_ml_db_set_line_mark(db_index.as_mut_ptr(), 0, 0);
            assert_eq!(rs_ml_db_line_is_marked(db_index.as_ptr(), 0), 0);
        }
    }

    #[test]
    fn test_db_has_room() {
        let header = DataBlockHeader {
            db_id: 0x6461,
            db_free: 100,
            db_txt_start: 900,
            db_txt_end: 1000,
            db_line_count: 5,
        };

        unsafe {
            // Room for 50 bytes + 4 byte index = 54 bytes needed
            assert_eq!(rs_ml_db_has_room(&header, 50), 1);

            // Not room for 100 bytes + 4 byte index = 104 bytes needed
            assert_eq!(rs_ml_db_has_room(&header, 100), 0);
        }
    }
}
