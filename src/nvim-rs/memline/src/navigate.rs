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
//!
//! # Tree Navigation
//!
//! The memline B-tree uses a stack to track the path from root to the current
//! data block. This module provides helpers for stack management and tree traversal.

use std::ffi::c_int;

use crate::types::{
    BlockNr, BufHandle, ColNr, InfoPtrHandle, LineNr, PointerBlockHeader, PointerEntry, PosHandle,
    ML_DELETE, ML_FIND, ML_FLUSH, ML_INSERT, STACK_INCR,
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

// =============================================================================
// Stack Management Helpers
// =============================================================================

/// Get the stack entry at a given index.
///
/// # Safety
/// - `stack` must be a valid pointer to an array of `InfoPtrHandle`
/// - `index` must be within bounds
#[no_mangle]
#[allow(clippy::zst_offset)] // InfoPtrHandle is an opaque handle, not actually ZST in C
pub unsafe extern "C" fn rs_ml_stack_get_entry(
    stack: *const InfoPtrHandle,
    index: c_int,
) -> *const InfoPtrHandle {
    if stack.is_null() || index < 0 {
        return std::ptr::null();
    }
    #[allow(clippy::cast_sign_loss)]
    stack.add(index as usize)
}

/// Calculate the new stack size when growing.
///
/// The stack grows by `STACK_INCR` entries at a time.
#[no_mangle]
pub extern "C" fn rs_ml_stack_grow_size(current_size: c_int) -> c_int {
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let incr = STACK_INCR as c_int;
    current_size + incr
}

/// Check if the stack needs to grow to accommodate another entry.
#[no_mangle]
pub extern "C" fn rs_ml_stack_needs_grow(top: c_int, size: c_int) -> c_int {
    c_int::from(top >= size)
}

// =============================================================================
// Tree Traversal Helpers
// =============================================================================

/// Check if a line number is within a range.
///
/// Used during tree traversal to check if the target line is in the current branch.
#[no_mangle]
pub extern "C" fn rs_ml_line_in_range(lnum: LineNr, low: LineNr, high: LineNr) -> c_int {
    c_int::from(lnum >= low && lnum <= high)
}

/// Calculate the new high value after an action.
///
/// - For INSERT: high + 1
/// - For DELETE: high - 1
/// - Otherwise: high (unchanged)
#[no_mangle]
pub extern "C" fn rs_ml_adjust_high(high: LineNr, action: c_int) -> LineNr {
    if action == ML_INSERT {
        high + 1
    } else if action == ML_DELETE {
        high - 1
    } else {
        high
    }
}

/// Calculate the new line add value after an action.
///
/// - For INSERT: lineadd + 1
/// - For DELETE: lineadd - 1
/// - Otherwise: lineadd (unchanged)
#[no_mangle]
pub extern "C" fn rs_ml_adjust_lineadd(lineadd: c_int, action: c_int) -> c_int {
    if action == ML_INSERT {
        lineadd + 1
    } else if action == ML_DELETE {
        lineadd - 1
    } else {
        lineadd
    }
}

/// Check if an action is a simple tree operation.
///
/// Simple actions are: ML_DELETE, ML_INSERT, ML_FIND
/// Non-simple actions (like ML_FLUSH) bypass the locked block check.
#[no_mangle]
pub extern "C" fn rs_ml_action_is_simple(action: c_int) -> c_int {
    c_int::from(action == ML_DELETE || action == ML_INSERT || action == ML_FIND)
}

/// Check if an action is ML_FLUSH.
#[no_mangle]
pub extern "C" fn rs_ml_action_is_flush(action: c_int) -> c_int {
    c_int::from(action == ML_FLUSH)
}

// =============================================================================
// Pointer Block Traversal
// =============================================================================

/// Find the index of the pointer entry containing a line number.
///
/// Iterates through the pointer block entries, accumulating line counts
/// until we find the entry containing the target line.
///
/// # Arguments
/// * `entries` - Pointer to the first PointerEntry
/// * `count` - Number of entries in the block
/// * `lnum` - Target line number
/// * `low` - Current low line number (will be updated)
/// * `high_out` - Output: high line number for the found entry
/// * `bnum_out` - Output: block number for the found entry
/// * `page_count_out` - Output: page count for the found entry
///
/// # Returns
/// Index of the entry containing the line, or -1 if not found
///
/// # Safety
/// - `entries` must be a valid pointer to an array of PointerEntry
/// - All output pointers must be valid or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_find_ptr_entry(
    entries: *const PointerEntry,
    count: u16,
    lnum: LineNr,
    low: LineNr,
    high_out: *mut LineNr,
    bnum_out: *mut BlockNr,
    page_count_out: *mut c_int,
) -> c_int {
    if entries.is_null() {
        return -1;
    }

    let mut current_low = low;

    for idx in 0..count {
        #[allow(clippy::cast_sign_loss)]
        let entry = &*entries.add(idx as usize);
        let line_count = entry.pe_line_count;

        // Check if line count is zero (corrupted block)
        if line_count == 0 {
            return -1;
        }

        current_low += line_count;
        if current_low > lnum {
            // Found the entry
            if !high_out.is_null() {
                *high_out = current_low - 1;
            }
            if !bnum_out.is_null() {
                *bnum_out = entry.pe_bnum;
            }
            if !page_count_out.is_null() {
                *page_count_out = entry.pe_page_count;
            }
            return c_int::from(idx);
        }
    }

    -1 // Not found
}

/// Update the line count in a pointer entry after insert/delete.
///
/// # Safety
/// - `entry` must be a valid pointer to a PointerEntry
#[no_mangle]
pub unsafe extern "C" fn rs_ml_update_ptr_line_count(entry: *mut PointerEntry, action: c_int) {
    if entry.is_null() {
        return;
    }

    if action == ML_INSERT {
        (*entry).pe_line_count += 1;
    } else if action == ML_DELETE {
        (*entry).pe_line_count -= 1;
    }
}

/// Check if a pointer block header is valid (has correct ID).
///
/// # Safety
/// - `header` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_ml_check_ptr_block(header: *const PointerBlockHeader) -> c_int {
    if header.is_null() {
        return 0;
    }
    c_int::from((*header).is_valid())
}

// =============================================================================
// Line Adjustment for Error Recovery
// =============================================================================

/// Calculate the correction needed after a failed insert/delete.
///
/// When an insert/delete fails after we've already updated the tree,
/// we need to reverse the line count changes:
/// - Failed INSERT: need to subtract 1 (return -1)
/// - Failed DELETE: need to add 1 (return 1)
/// - Other: no correction needed (return 0)
#[no_mangle]
pub extern "C" fn rs_ml_error_correction(action: c_int) -> c_int {
    match action {
        x if x == ML_INSERT => -1,
        x if x == ML_DELETE => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_in_range() {
        assert_eq!(rs_ml_line_in_range(5, 1, 10), 1);
        assert_eq!(rs_ml_line_in_range(1, 1, 10), 1);
        assert_eq!(rs_ml_line_in_range(10, 1, 10), 1);
        assert_eq!(rs_ml_line_in_range(0, 1, 10), 0);
        assert_eq!(rs_ml_line_in_range(11, 1, 10), 0);
    }

    #[test]
    fn test_adjust_high() {
        assert_eq!(rs_ml_adjust_high(100, ML_INSERT), 101);
        assert_eq!(rs_ml_adjust_high(100, ML_DELETE), 99);
        assert_eq!(rs_ml_adjust_high(100, ML_FIND), 100);
        assert_eq!(rs_ml_adjust_high(100, ML_FLUSH), 100);
    }

    #[test]
    fn test_adjust_lineadd() {
        assert_eq!(rs_ml_adjust_lineadd(5, ML_INSERT), 6);
        assert_eq!(rs_ml_adjust_lineadd(5, ML_DELETE), 4);
        assert_eq!(rs_ml_adjust_lineadd(5, ML_FIND), 5);
    }

    #[test]
    fn test_action_is_simple() {
        assert_eq!(rs_ml_action_is_simple(ML_INSERT), 1);
        assert_eq!(rs_ml_action_is_simple(ML_DELETE), 1);
        assert_eq!(rs_ml_action_is_simple(ML_FIND), 1);
        assert_eq!(rs_ml_action_is_simple(ML_FLUSH), 0);
    }

    #[test]
    fn test_error_correction() {
        assert_eq!(rs_ml_error_correction(ML_INSERT), -1);
        assert_eq!(rs_ml_error_correction(ML_DELETE), 1);
        assert_eq!(rs_ml_error_correction(ML_FIND), 0);
    }

    #[test]
    fn test_stack_grow() {
        assert_eq!(rs_ml_stack_grow_size(0), c_int::try_from(STACK_INCR).unwrap());
        assert_eq!(rs_ml_stack_grow_size(5), 5 + c_int::try_from(STACK_INCR).unwrap());
    }

    #[test]
    fn test_stack_needs_grow() {
        assert_eq!(rs_ml_stack_needs_grow(5, 10), 0); // 5 < 10, no grow
        assert_eq!(rs_ml_stack_needs_grow(10, 10), 1); // 10 >= 10, need grow
        assert_eq!(rs_ml_stack_needs_grow(11, 10), 1); // 11 >= 10, need grow
    }

    #[test]
    fn test_find_ptr_entry() {
        let entries = [
            PointerEntry::with_values(10, 50, 1, 1),  // lines 1-50
            PointerEntry::with_values(20, 30, 51, 1), // lines 51-80
            PointerEntry::with_values(30, 20, 81, 1), // lines 81-100
        ];

        unsafe {
            let mut high = 0;
            let mut bnum = 0;
            let mut page_count = 0;

            // Find line 25 (in first entry)
            let idx = rs_ml_find_ptr_entry(
                entries.as_ptr(),
                3,
                25,
                1,
                &raw mut high,
                &raw mut bnum,
                &raw mut page_count,
            );
            assert_eq!(idx, 0);
            assert_eq!(high, 50);
            assert_eq!(bnum, 10);

            // Find line 60 (in second entry)
            let idx = rs_ml_find_ptr_entry(
                entries.as_ptr(),
                3,
                60,
                1,
                &raw mut high,
                &raw mut bnum,
                &raw mut page_count,
            );
            assert_eq!(idx, 1);
            assert_eq!(high, 80);
            assert_eq!(bnum, 20);

            // Find line 90 (in third entry)
            let idx = rs_ml_find_ptr_entry(
                entries.as_ptr(),
                3,
                90,
                1,
                &raw mut high,
                &raw mut bnum,
                &raw mut page_count,
            );
            assert_eq!(idx, 2);
            assert_eq!(high, 100);
            assert_eq!(bnum, 30);

            // Find line 101 (not found)
            let idx = rs_ml_find_ptr_entry(
                entries.as_ptr(),
                3,
                101,
                1,
                &raw mut high,
                &raw mut bnum,
                &raw mut page_count,
            );
            assert_eq!(idx, -1);
        }
    }

    #[test]
    fn test_update_ptr_line_count() {
        let mut entry = PointerEntry::with_values(10, 50, 1, 1);

        unsafe {
            rs_ml_update_ptr_line_count(&raw mut entry, ML_INSERT);
            assert_eq!(entry.pe_line_count, 51);

            rs_ml_update_ptr_line_count(&raw mut entry, ML_DELETE);
            assert_eq!(entry.pe_line_count, 50);

            rs_ml_update_ptr_line_count(&raw mut entry, ML_FIND);
            assert_eq!(entry.pe_line_count, 50); // unchanged
        }
    }
}
