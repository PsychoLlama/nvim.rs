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

use std::ffi::{c_char, c_int};
use std::ptr;

use crate::types::{BufHandle, ColNr, LineNr, PosHandle, ML_ALLOCATED, ML_LINE_DIRTY};

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

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
}
