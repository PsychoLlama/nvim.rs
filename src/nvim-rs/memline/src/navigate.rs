//! Cursor navigation and byte offset functions for the memline system.
//!
//! This module provides Rust implementations for position navigation,
//! including cursor increment/decrement operations and byte offset calculations.
//!
//! # Position Navigation
//!
//! The `inc`/`dec` family of functions move a position forward or backward
//! through the buffer, properly handling line boundaries and multi-byte characters.
//!
//! # Byte Offsets
//!
//! The memline tracks byte offsets using "chunks" of ~800 lines to optimize
//! `line2byte()` and `byte2line()` operations.

use std::ffi::c_int;

use crate::types::{BufHandle, ColNr, LineNr, PosHandle};

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
    // Buffer Accessors
    // -------------------------------------------------------------------------

    /// Get buffer's line count (`buf->b_ml.ml_line_count`)
    fn nvim_buf_get_ml_line_count(buf: *mut BufHandle) -> LineNr;

    // -------------------------------------------------------------------------
    // Position Accessors
    // -------------------------------------------------------------------------

    /// Get line number from position (`pos->lnum`)
    fn nvim_pos_get_lnum(pos: *const PosHandle) -> LineNr;

    /// Get column from position (`pos->col`)
    fn nvim_pos_get_col(pos: *const PosHandle) -> ColNr;

    // Note: nvim_pos_set_lnum, nvim_pos_set_col, nvim_pos_set_coladd are available
    // in C but currently the inc/dec functions handle position updates internally.

    /// MAXCOL constant
    fn nvim_get_maxcol() -> ColNr;

    // -------------------------------------------------------------------------
    // C Implementation Functions
    // -------------------------------------------------------------------------

    /// Increment position (C implementation)
    fn inc(lp: *mut PosHandle) -> c_int;

    /// Increment position, skipping NUL at end of non-empty lines (C implementation)
    fn incl(lp: *mut PosHandle) -> c_int;

    /// Decrement position (C implementation)
    fn dec(lp: *mut PosHandle) -> c_int;

    /// Decrement position, skipping NUL at end of non-empty lines (C implementation)
    fn decl(lp: *mut PosHandle) -> c_int;

    /// Find byte offset for line, or line at byte offset (C implementation)
    fn ml_find_line_or_offset(
        buf: *mut BufHandle,
        lnum: LineNr,
        offp: *mut c_int,
        no_ff: c_int,
    ) -> c_int;

    /// Go to a byte position in the buffer (C implementation)
    fn goto_byte(cnt: c_int);

    /// Flush deleted bytes counter (C implementation)
    fn ml_flush_deleted_bytes(
        buf: *mut BufHandle,
        codepoints: *mut usize,
        codeunits: *mut usize,
    ) -> usize;
}

// =============================================================================
// Position Increment/Decrement Functions
// =============================================================================

/// Increment a position, crossing line boundaries as necessary.
///
/// Returns:
/// - `1` when moving to the next line
/// - `2` when moving forward onto a NUL at end of line
/// - `-1` when at end of file
/// - `0` otherwise
///
/// # Safety
/// - `lp` must be a valid position pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_inc(lp: *mut PosHandle) -> c_int {
    if lp.is_null() {
        return -1;
    }
    inc(lp)
}

/// Like `inc()`, but skip NUL at the end of non-empty lines.
///
/// # Safety
/// - `lp` must be a valid position pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_incl(lp: *mut PosHandle) -> c_int {
    if lp.is_null() {
        return -1;
    }
    incl(lp)
}

/// Decrement a position, crossing line boundaries as necessary.
///
/// Returns:
/// - `1` when moving to the previous line
/// - `-1` when at start of file
/// - `0` otherwise
///
/// # Safety
/// - `lp` must be a valid position pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_dec(lp: *mut PosHandle) -> c_int {
    if lp.is_null() {
        return -1;
    }
    dec(lp)
}

/// Like `dec()`, but skip NUL at the end of non-empty lines.
///
/// # Safety
/// - `lp` must be a valid position pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_decl(lp: *mut PosHandle) -> c_int {
    if lp.is_null() {
        return -1;
    }
    decl(lp)
}

// =============================================================================
// Byte Offset Functions
// =============================================================================

/// Get the byte offset of a line in a buffer.
///
/// This returns the 0-based byte offset of the start of line `lnum`.
/// Returns -1 if the information is not available.
///
/// # Arguments
/// * `buf` - Buffer to query
/// * `lnum` - Line number (1-based)
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_line2byte(buf: *mut BufHandle, lnum: LineNr) -> c_int {
    if buf.is_null() {
        return -1;
    }
    ml_find_line_or_offset(buf, lnum, std::ptr::null_mut(), 1)
}

/// Get the byte offset of a line, considering file format.
///
/// This includes the effect of DOS line endings (CR+LF) if applicable.
///
/// # Arguments
/// * `buf` - Buffer to query
/// * `lnum` - Line number (1-based)
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_line2byte_ff(buf: *mut BufHandle, lnum: LineNr) -> c_int {
    if buf.is_null() {
        return -1;
    }
    ml_find_line_or_offset(buf, lnum, std::ptr::null_mut(), 0)
}

/// Get the line number containing a byte offset.
///
/// # Arguments
/// * `buf` - Buffer to query
/// * `byte` - Byte offset (0-based)
/// * `col_out` - Output: column within the line (0-based)
///
/// # Returns
/// Line number (1-based) or 0 if not found
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
/// - `col_out` must be a valid pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_byte2line(
    buf: *mut BufHandle,
    byte: c_int,
    col_out: *mut c_int,
) -> LineNr {
    if buf.is_null() {
        return 0;
    }

    let mut offset = byte;
    let result = ml_find_line_or_offset(buf, 0, std::ptr::addr_of_mut!(offset), 1);

    if result > 0 && !col_out.is_null() {
        *col_out = offset;
    }

    if result < 0 {
        0
    } else {
        result.into()
    }
}

/// Go to a byte position in the current buffer.
///
/// Moves the cursor to the position at byte offset `cnt`.
///
/// # Safety
/// Modifies cursor position and buffer state.
#[no_mangle]
pub unsafe extern "C" fn rs_goto_byte(cnt: c_int) {
    goto_byte(cnt);
}

// =============================================================================
// Deleted Bytes Tracking
// =============================================================================

/// Flush the deleted bytes counter for a buffer.
///
/// Returns the accumulated deleted byte count and resets the counter.
/// Also returns codepoint and codeunit counts via output parameters.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
/// - Output pointers must be valid or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_flush_deleted_bytes(
    buf: *mut BufHandle,
    codepoints: *mut usize,
    codeunits: *mut usize,
) -> usize {
    if buf.is_null() {
        if !codepoints.is_null() {
            *codepoints = 0;
        }
        if !codeunits.is_null() {
            *codeunits = 0;
        }
        return 0;
    }

    ml_flush_deleted_bytes(buf, codepoints, codeunits)
}

// =============================================================================
// Position Validation
// =============================================================================

/// Check if a position is valid in the current buffer.
///
/// A position is valid if:
/// - Line number is between 1 and line_count
/// - Column is non-negative
///
/// # Safety
/// - `pos` must be a valid position pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_pos_valid(pos: *const PosHandle) -> c_int {
    if pos.is_null() {
        return 0;
    }

    let curbuf = nvim_get_curbuf();
    if curbuf.is_null() {
        return 0;
    }

    let lnum = nvim_pos_get_lnum(pos);
    let col = nvim_pos_get_col(pos);
    let line_count = nvim_buf_get_ml_line_count(curbuf);

    c_int::from(lnum >= 1 && lnum <= line_count && col >= 0)
}

/// Check if a position is at the start of the buffer.
///
/// # Safety
/// - `pos` must be a valid position pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_pos_at_start(pos: *const PosHandle) -> c_int {
    if pos.is_null() {
        return 0;
    }

    let lnum = nvim_pos_get_lnum(pos);
    let col = nvim_pos_get_col(pos);

    c_int::from(lnum == 1 && col == 0)
}

/// Check if a position is at the end of the buffer.
///
/// A position is at the end if it's on the last line and past all characters.
///
/// # Safety
/// - `pos` must be a valid position pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_pos_at_end(pos: *const PosHandle) -> c_int {
    if pos.is_null() {
        return 0;
    }

    let curbuf = nvim_get_curbuf();
    if curbuf.is_null() {
        return 0;
    }

    let lnum = nvim_pos_get_lnum(pos);
    let col = nvim_pos_get_col(pos);
    let line_count = nvim_buf_get_ml_line_count(curbuf);
    let maxcol = nvim_get_maxcol();

    c_int::from(lnum == line_count && (col == maxcol || col < 0))
}

// =============================================================================
// Chunk Management Helpers
// =============================================================================

/// Get chunk-related statistics for a buffer.
///
/// Returns the number of chunks used, or -1 if chunking is disabled.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_get_used_chunks(buf: *mut BufHandle) -> c_int {
    if buf.is_null() {
        return -1;
    }

    nvim_buf_get_ml_usedchunks(buf)
}

extern "C" {
    fn nvim_buf_get_ml_usedchunks(buf: *mut BufHandle) -> c_int;
}

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
}
