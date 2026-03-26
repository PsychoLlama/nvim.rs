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
    BlockNr, BufHandle, ColNr, DataBlockHeader, InfoPtrHandle, LineNr, PointerBlockHeader,
    PointerEntry, DATA_BLOCK_HEADER_SIZE, DB_INDEX_MASK, DB_MARKED, INDEX_SIZE, ML_ALLOCATED,
    ML_APPEND_MARK, ML_APPEND_NEW, ML_CHNK_ADDLINE, ML_CHNK_DELLINE, ML_CHNK_UPDLINE,
    ML_DEL_MESSAGE, ML_EMPTY, ML_FIND, ML_FLUSH, ML_INSERT, ML_LINE_DIRTY, ML_LOCKED_DIRTY,
    ML_LOCKED_POS, PTR_ID, STACK_INCR,
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

    /// Find data block containing a line (Rust implementation)
    fn rs_ml_find_line(buf: *mut BufHandle, lnum: LineNr, action: c_int) -> *mut c_void;

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

    /// xfree wrapper
    fn xfree(ptr: *mut std::ffi::c_void);

    // -------------------------------------------------------------------------
    // Phase 4: Deleted-length tracking and stack accessors
    // -------------------------------------------------------------------------

    /// Get inhibit_delete_count global
    fn nvim_get_inhibit_delete_count() -> c_int;

    /// Get string length (libc)
    fn strlen(s: *const c_char) -> usize;

    /// Add n to buf->deleted_bytes
    fn nvim_buf_add_deleted_bytes(buf: *mut BufHandle, n: usize);

    /// Add n to buf->deleted_bytes2
    fn nvim_buf_add_deleted_bytes2(buf: *mut BufHandle, n: usize);

    /// Get buf->update_need_codepoints
    fn nvim_buf_get_update_need_codepoints(buf: *mut BufHandle) -> bool;

    /// Add n to buf->deleted_codepoints
    fn nvim_buf_add_deleted_codepoints(buf: *mut BufHandle, n: usize);

    /// Add n to buf->deleted_codeunits
    fn nvim_buf_add_deleted_codeunits(buf: *mut BufHandle, n: usize);

    /// Get UTF codepoint/codeunit length of string
    fn mb_utflen(s: *const c_char, len: usize, codepoints: *mut usize, codeunits: *mut usize);

    /// Get buf->b_ml.ml_stack_top
    fn nvim_buf_get_ml_stack_top(buf: *mut BufHandle) -> c_int;

    /// Get buf->b_ml.ml_stack_size
    fn nvim_buf_get_ml_stack_size(buf: *mut BufHandle) -> c_int;

    /// Set buf->b_ml.ml_stack_size
    fn nvim_buf_set_ml_stack_size(buf: *mut BufHandle, n: c_int);

    /// Increment buf->b_ml.ml_stack_top and return old value
    fn nvim_buf_inc_ml_stack_top(buf: *mut BufHandle) -> c_int;

    /// Get buf->b_ml.ml_stack as void* (same as nvim_buf_get_ml_stack_void)
    fn nvim_buf_get_ml_stack(buf: *mut BufHandle) -> *mut std::ffi::c_void;

    /// Set buf->b_ml.ml_stack
    fn nvim_buf_set_ml_stack(buf: *mut BufHandle, ptr: *mut std::ffi::c_void);

    /// Get sizeof(infoptr_T)
    fn nvim_get_infoptr_size() -> usize;

    /// Reallocate memory
    fn xrealloc(ptr: *mut std::ffi::c_void, size: usize) -> *mut std::ffi::c_void;

    // -------------------------------------------------------------------------
    // Pass 3 Phase 3: ml_replace_buf_len accessors
    // -------------------------------------------------------------------------

    /// Set buffer's cached line pointer (`buf->b_ml.ml_line_ptr = ptr`)
    fn nvim_buf_set_ml_line_ptr(buf: *mut BufHandle, ptr: *mut c_char);

    /// Set buffer's cached line number (`buf->b_ml.ml_line_lnum = lnum`)
    fn nvim_buf_set_ml_line_lnum(buf: *mut BufHandle, lnum: LineNr);

    /// Get number of update callbacks for a buffer
    fn nvim_buf_get_update_callbacks_size(buf: *mut BufHandle) -> usize;

    /// Open the buffer's memfile if needed; returns FAIL if open_buffer fails
    fn nvim_buf_open_buffer_if_needed(buf: *mut BufHandle) -> c_int;

    /// Duplicate a memory region, NUL-terminated (xmalloc + memcpy + NUL)
    fn xmemdupz(data: *const c_void, len: usize) -> *mut c_void;

    /// Set buffer's cached line length (`buf->b_ml.ml_line_len = len`)
    fn nvim_buf_set_ml_line_len(buf: *mut BufHandle, len: ColNr);
}

// =============================================================================
// Phase 6: ml_delete_int / ml_updatechunk C accessor declarations
// =============================================================================

extern "C" {
    /// Get buffer's memfile pointer (`buf->b_ml.ml_mfp`)
    fn nvim_buf_get_ml_mfp(buf: *mut BufHandle) -> *mut std::ffi::c_void;

    /// Decrement buf->b_ml.ml_line_count and return new value
    fn nvim_buf_dec_ml_line_count(buf: *mut BufHandle) -> LineNr;

    /// Get buf->b_prev_line_count
    fn nvim_buf_get_b_prev_line_count(buf: *mut BufHandle) -> LineNr;

    /// Set buf->b_prev_line_count
    fn nvim_buf_set_b_prev_line_count(buf: *mut BufHandle, val: LineNr);

    /// set_keep_msg(_(no_lines_msg), 0) -- "No lines in buffer"
    fn nvim_set_keep_msg_no_lines();

    /// iemsg for "E317: Pointer block id wrong 4"
    fn nvim_iemsg_pointer_block_id_wrong_four();

    /// Free a block in the memfile (mf_free wrapper)
    fn nvim_mf_free(mfp: *mut std::ffi::c_void, hp: *mut std::ffi::c_void);

    /// Decrement pp->pb_count and return new value
    fn nvim_pp_dec_count(pp: *mut std::ffi::c_void) -> c_int;

    /// memmove pointer entries within a pointer block
    fn nvim_pp_pe_memmove(pp: *mut std::ffi::c_void, dst_idx: c_int, src_idx: c_int, count: c_int);

    /// Get buf->b_ml.ml_locked_lineadd
    fn nvim_buf_get_ml_locked_lineadd(buf: *mut BufHandle) -> c_int;

    /// Get buf->b_ml.ml_stack[idx]
    fn nvim_buf_get_ml_stack_ip(buf: *mut BufHandle, idx: c_int) -> *mut InfoPtrHandle;

    /// Get ip->ip_index
    fn nvim_ip_get_index(ip: *const InfoPtrHandle) -> c_int;

    /// Get ip->ip_bnum as BlockNr
    fn nvim_ip_get_bnum(ip: *const InfoPtrHandle) -> BlockNr;

    /// Get memfile block (mf_get)
    fn mf_get(mfp: *mut std::ffi::c_void, bnum: BlockNr, count: c_int) -> *mut std::ffi::c_void;

    /// Release block to memfile (mf_put)
    fn mf_put(mfp: *mut std::ffi::c_void, hp: *mut std::ffi::c_void, dirty: bool, release: bool);

    /// Get pp->pb_id
    fn nvim_pp_get_id(pp: *const std::ffi::c_void) -> u16;

    /// Add count to ip->ip_high
    fn nvim_ip_add_high(ip: *mut InfoPtrHandle, count: c_int);

    /// Set ip->ip_index
    fn nvim_ip_set_index(ip: *mut InfoPtrHandle, idx: c_int);

    /// Set buf->b_ml.ml_locked
    fn nvim_buf_set_ml_locked(buf: *mut BufHandle, hp: *mut std::ffi::c_void);

    /// Set buf->b_ml.ml_stack_top
    fn nvim_buf_set_ml_stack_top(buf: *mut BufHandle, n: c_int);

    /// Adjust the B-tree pointer block line counts after insert/delete (in navigate.rs)
    fn rs_ml_lineadd(buf: *mut BufHandle, count: c_int);
}

// =============================================================================
// Pass 7 Phase 1: ml_append_int C accessor declarations
// =============================================================================

extern "C" {
    /// Get hp->bh_bnum as int64_t
    fn nvim_bhdr_get_bh_bnum(hp: *mut c_void) -> i64;

    /// Get hp->bh_page_count as int
    fn nvim_bhdr_get_bh_page_count(hp: *mut c_void) -> c_int;

    /// iemsg for "E317: Pointer block id wrong 3"
    fn nvim_iemsg_pointer_block_id_wrong_three();

    /// iemsg for "E318: Updated too many blocks?"
    fn nvim_iemsg_e318_updated_too_many();

    /// Set buf->b_ml.ml_locked_lineadd
    fn nvim_buf_set_ml_locked_lineadd(buf: *mut BufHandle, val: c_int);

    /// Increment buf->b_ml.ml_line_count and return new value
    fn nvim_buf_inc_ml_line_count(buf: *mut BufHandle) -> LineNr;

    /// Set buf->b_ml.ml_locked_high
    fn nvim_buf_set_ml_locked_high(buf: *mut BufHandle, val: LineNr);

    /// Increment pp->pb_count
    fn nvim_pp_inc_count(pp: *mut c_void);
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
    dp.cast::<u8>().add(DATA_BLOCK_HEADER_SIZE).cast::<u32>()
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

    // -------------------------------------------------------------------------
    // Pass 4 Phase 2: Public wrappers for static C functions
    // -------------------------------------------------------------------------

    /// Public wrapper around static ml_append_flush
    fn nvim_ml_append_flush(
        buf: *mut BufHandle,
        lnum: LineNr,
        line: *mut c_char,
        len: ColNr,
        flags: c_int,
    ) -> c_int;

    // -------------------------------------------------------------------------
    // Pass 5 Phase 2: rs_ml_flush_line accessors
    // -------------------------------------------------------------------------

    /// Get ml_line_len (cached line length including NUL)
    fn nvim_buf_get_ml_line_len(buf: *mut BufHandle) -> ColNr;

    /// Increment buf->flush_count
    fn nvim_buf_inc_flush_count(buf: *mut BufHandle);

    /// Print "E320: Cannot find line N" error
    fn nvim_siemsg_e320_cannot_find_line(lnum: i64);

    /// Set buffer's ml_line_offset to 0
    fn nvim_buf_set_ml_line_offset(buf: *mut BufHandle, offset: usize);
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

// OK and FAIL constants (matching C vim_defs.h)
const OK: c_int = 1;
const FAIL: c_int = 0;

/// Replace line `lnum` in buffer `buf` with explicit text length.
///
/// This is the Rust port of the C `ml_replace_buf_len` function.
///
/// # Arguments
/// * `buf` - Buffer to modify
/// * `lnum` - Line number to replace (1-based)
/// * `line_arg` - New line text
/// * `len_arg` - Length of `line_arg`, excluding NUL
/// * `copy` - If true, make a copy of `line_arg` via `xmemdupz`
/// * `noalloc` - If true, flush the line immediately via `rs_ml_flush_line(buf, true)`
///
/// # Returns
/// OK (1) on success, FAIL (0) on failure
///
/// # Safety
/// - `buf` must be a valid buffer pointer
/// - `line_arg` must point to at least `len_arg` bytes
/// - If `copy` is false, the caller must not use `line_arg` after this call
#[export_name = "ml_replace_buf_len"]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub unsafe extern "C" fn rs_ml_replace_buf_len(
    buf: *mut BufHandle,
    lnum: LineNr,
    line_arg: *mut c_char,
    len_arg: usize,
    copy: c_int,
    noalloc: c_int,
) -> c_int {
    if line_arg.is_null() {
        return FAIL;
    }

    // When starting up, might still need to create the memfile
    if nvim_buf_open_buffer_if_needed(buf) == FAIL {
        return FAIL;
    }

    let mut line = line_arg;
    let len = len_arg as ColNr;

    if copy != 0 {
        // assert(!noalloc) is guaranteed by the caller
        line = xmemdupz(line_arg.cast(), len_arg).cast::<c_char>();
    }

    if nvim_buf_get_ml_line_lnum(buf) != lnum {
        // Another line is buffered - flush it
        rs_ml_flush_line(buf, 0);
    }

    if nvim_buf_get_update_callbacks_size(buf) > 0 {
        // Track deleted bytes for update callbacks
        let current_line = crate::access::rs_ml_get_buf_impl(buf, lnum, 0);
        rs_ml_add_deleted_len_buf(buf, current_line, -1);
    }

    let flags = nvim_buf_get_ml_flags(buf);
    if (flags & (ML_LINE_DIRTY | ML_ALLOCATED)) != 0 {
        // Free previously allocated line
        xfree(nvim_buf_get_ml_line_ptr(buf).cast());
    }

    nvim_buf_set_ml_line_ptr(buf, line);
    nvim_buf_set_ml_line_len(buf, len + 1);
    nvim_buf_set_ml_line_lnum(buf, lnum);
    let new_flags = (nvim_buf_get_ml_flags(buf) | ML_LINE_DIRTY) & !ML_EMPTY;
    nvim_buf_set_ml_flags(buf, new_flags);

    if noalloc != 0 {
        rs_ml_flush_line(buf, 1);
    }

    OK
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
// Pass 4 Phase 2: Modification Dispatch _impl Functions
//
// These implement the guard/dispatch logic that was in the multi-line C
// functions. The C public functions now become thin wrappers calling these.
// The existing rs_ml_append / rs_ml_delete / rs_ml_replace wrappers continue
// to call the C public functions unchanged — there are no circular calls.
// =============================================================================

/// Implement `ml_append_flags`: open buffer if needed, then call
/// `nvim_ml_append_flush` (public wrapper for the static `ml_append_flush`).
///
/// # Safety
/// Modifies buffer state. Only call from main Nvim thread.
#[export_name = "ml_append_flags"]
pub unsafe extern "C" fn rs_ml_append_flags_impl(
    lnum: LineNr,
    line: *mut c_char,
    len: ColNr,
    flags: c_int,
) -> c_int {
    // When starting up, we might still need to create the memfile.
    let buf = nvim_get_curbuf();
    if nvim_buf_open_buffer_if_needed(buf) == FAIL {
        return FAIL;
    }
    nvim_ml_append_flush(buf, lnum, line, len, flags)
}

/// Implement `ml_append_buf`: guard on ml_mfp, then call
/// `nvim_ml_append_flush` for the specified buffer.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
///
/// Modifies buffer state. Only call from main Nvim thread.
#[export_name = "ml_append_buf"]
pub unsafe extern "C" fn rs_ml_append_buf_impl(
    buf: *mut BufHandle,
    lnum: LineNr,
    line: *mut c_char,
    len: ColNr,
    newfile: c_int,
) -> c_int {
    if buf.is_null() {
        return FAIL;
    }
    if nvim_buf_has_ml_mfp(buf) == 0 {
        return FAIL;
    }
    let flags = if newfile != 0 { ML_APPEND_NEW } else { 0 };
    nvim_ml_append_flush(buf, lnum, line, len, flags)
}

/// Implement `ml_delete_flags`: flush cached line, range-check, then delete.
///
/// # Safety
///
/// Modifies buffer state. Only call from main Nvim thread.
#[allow(clippy::must_use_candidate)]
#[export_name = "ml_delete_flags"]
pub unsafe extern "C" fn rs_ml_delete_flags_impl(lnum: LineNr, flags: c_int) -> c_int {
    let buf = nvim_get_curbuf();
    rs_ml_flush_line(buf, 0);
    let line_count = nvim_buf_get_ml_line_count(buf);
    if lnum < 1 || lnum > line_count {
        return FAIL;
    }
    rs_ml_delete_int(buf, lnum, flags)
}

/// Implement `ml_delete_buf`: flush cached line, then delete from given buffer.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
///
/// Modifies buffer state. Only call from main Nvim thread.
#[export_name = "ml_delete_buf"]
pub unsafe extern "C" fn rs_ml_delete_buf_impl(
    buf: *mut BufHandle,
    lnum: LineNr,
    message: c_int,
) -> c_int {
    if buf.is_null() {
        return FAIL;
    }
    rs_ml_flush_line(buf, 0);
    let flags = if message != 0 { ML_DEL_MESSAGE } else { 0 };
    rs_ml_delete_int(buf, lnum, flags)
}

/// Implement `ml_replace_buf`: compute strlen, then call rs_ml_replace_buf_len.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
/// - `line` must be a valid C string or NULL
///
/// Modifies buffer state. Only call from main Nvim thread.
#[export_name = "ml_replace_buf"]
pub unsafe extern "C" fn rs_ml_replace_buf_impl(
    buf: *mut BufHandle,
    lnum: LineNr,
    line: *mut c_char,
    copy: c_int,
    noalloc: c_int,
) -> c_int {
    if buf.is_null() {
        return FAIL;
    }
    let len = if line.is_null() {
        usize::MAX
    } else {
        strlen(line)
    };
    rs_ml_replace_buf_len(buf, lnum, line, len, copy, noalloc)
}

/// Get a mutable pointer to the PointerEntry array inside a pointer block.
///
/// The pb_pointer array immediately follows the `PointerBlockHeader` in memory,
/// matching the C flexible array member `pb_pointer[]`.
///
/// # Safety
/// - `pp` must be a valid pointer to a `PointerBlock` (PointerBlockHeader + pb_pointer[])
#[inline]
unsafe fn pb_pointer_ptr(pp: *mut c_void) -> *mut PointerEntry {
    #[allow(clippy::cast_ptr_alignment)]
    pp.cast::<u8>()
        .add(std::mem::size_of::<PointerBlockHeader>())
        .cast::<PointerEntry>()
}

// =============================================================================
// Pass 7 Phase 2: rs_ml_append_int -- core B-tree line insertion
// =============================================================================

/// Insert a line after `lnum` into buffer `buf`.
///
/// This is the Rust port of the C `ml_append_int` function. It handles:
/// - Simple case: Line fits in the existing data block. Shift text and indexes,
///   copy the line in.
/// - Block split: Data block is full. Allocate a new data block, decide left/right
///   placement, move lines between blocks.
/// - Pointer block update: Walk up the stack inserting the new pointer entry.
///   If a pointer block is full, split it. If block 1 is full, add a new tree
///   level.
///
/// # Arguments
/// * `buf` - Buffer to modify (must be non-null)
/// * `lnum` - Insert after this line number (0 = insert before line 1)
/// * `line_arg` - Text of the new line (NUL-terminated)
/// * `len_arg` - Length including NUL, or 0 to auto-calculate via strlen
/// * `flags` - `ML_APPEND_NEW` and/or `ML_APPEND_MARK`
///
/// # Returns
/// OK (1) on success, FAIL (0) on failure.
///
/// # Safety
/// - `buf` must be a valid, non-null buffer pointer with an initialized memline.
/// - `line_arg` must point to a valid NUL-terminated string (or have length len_arg).
/// - Must only be called from the main Neovim thread.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]
pub unsafe extern "C" fn rs_ml_append_int(
    buf: *mut BufHandle,
    lnum: LineNr,
    line_arg: *mut c_char,
    len_arg: ColNr,
    flags: c_int,
) -> c_int {
    let line = line_arg;
    let len: ColNr = if len_arg == 0 {
        strlen(line) as ColNr + 1
    } else {
        len_arg
    };

    if lnum > nvim_buf_get_ml_line_count(buf) || nvim_buf_has_ml_mfp(buf) == 0 {
        return FAIL; // lnum out of range
    }

    rs_ml_adjust_lowest_marked_for_insert(lnum);

    let space_needed = len + INDEX_SIZE as ColNr; // space for text + index entry

    let mfp = nvim_buf_get_ml_mfp(buf);
    let page_size = nvim_mf_get_page_size(mfp) as c_int;

    // Find the data block containing the previous line.
    // This fills the stack from root to the data block and releases any locked block.
    let mut hp = rs_ml_find_line(buf, if lnum == 0 { 1 } else { lnum }, ML_INSERT);
    if hp.is_null() {
        return FAIL;
    }

    // Clear ML_EMPTY flag now that we have at least one real line.
    let cur_flags = nvim_buf_get_ml_flags(buf);
    nvim_buf_set_ml_flags(buf, cur_flags & !ML_EMPTY);

    let mut db_idx: c_int = if lnum == 0 {
        -1 // prepending before line 1
    } else {
        (lnum - nvim_buf_get_ml_locked_low(buf)) as c_int
    };
    // line count (number of indexes) in current block before insertion
    let mut line_count =
        (nvim_buf_get_ml_locked_high(buf) - nvim_buf_get_ml_locked_low(buf)) as c_int;

    let mut dp = nvim_bhdr_get_bh_data(hp);
    let mut dp_header = dp.cast::<DataBlockHeader>();

    // If not enough room AND appending to last line in block AND not end of file,
    // insert in front of the next block instead.
    if ((*dp_header).db_free as c_int) < space_needed
        && db_idx == line_count - 1
        && lnum < nvim_buf_get_ml_line_count(buf)
    {
        // The line counts in pointer blocks need adjustment via ml_locked_lineadd.
        let lineadd_val = nvim_buf_get_ml_locked_lineadd(buf) - 1;
        nvim_buf_set_ml_locked_lineadd(buf, lineadd_val);
        nvim_buf_set_ml_locked_high(buf, nvim_buf_get_ml_locked_high(buf) - 1);
        hp = rs_ml_find_line(buf, lnum + 1, ML_INSERT);
        if hp.is_null() {
            return FAIL;
        }
        db_idx = -1; // prepending in this new block
        line_count = (nvim_buf_get_ml_locked_high(buf) - nvim_buf_get_ml_locked_low(buf)) as c_int;
        debug_assert!(nvim_buf_get_ml_locked_low(buf) == lnum + 1);
        dp = nvim_bhdr_get_bh_data(hp);
        dp_header = dp.cast::<DataBlockHeader>();
    }

    if nvim_buf_get_b_prev_line_count(buf) == 0 {
        nvim_buf_set_b_prev_line_count(buf, nvim_buf_get_ml_line_count(buf));
    }
    nvim_buf_inc_ml_line_count(buf);

    let db_idx_arr: *mut u32 = db_index_ptr(dp);

    if (*dp_header).db_free as c_int >= space_needed {
        // -----------------------------------------------------------------------
        // Simple case: enough room in the data block.
        // Insert the new line text and update the index array.
        // -----------------------------------------------------------------------
        (*dp_header).db_txt_start -= len as u32;
        (*dp_header).db_free -= space_needed as u32;
        (*dp_header).db_line_count += 1;

        if line_count > db_idx + 1 {
            // There are lines following the insertion point -- move their text forward.
            // offset = start position of the line at db_idx (the line we insert after).
            let offset: u32 = if db_idx < 0 {
                (*dp_header).db_txt_end
            } else {
                *db_idx_arr.add(db_idx as usize) & DB_INDEX_MASK
            };
            // Move text of lines that follow toward the start of the block.
            let txt_start = (*dp_header).db_txt_start as usize;
            let block_base = dp.cast::<u8>();
            let src = block_base.add(txt_start + len as usize);
            let dst = block_base.add(txt_start);
            let copy_len = offset as usize - (txt_start + len as usize);
            std::ptr::copy(src, dst, copy_len);
            // Adjust indexes of the lines that follow: each shifts back by len bytes.
            for i in (db_idx + 1..line_count).rev() {
                *db_idx_arr.add((i + 1) as usize) = (*db_idx_arr.add(i as usize)) - len as u32;
            }
            *db_idx_arr.add((db_idx + 1) as usize) = offset - len as u32;
        } else {
            // Adding the line at the end (start of the text area).
            *db_idx_arr.add((db_idx + 1) as usize) = (*dp_header).db_txt_start;
        }

        // Copy the new line text into the block.
        let text_dst = dp
            .cast::<u8>()
            .add((*db_idx_arr.add((db_idx + 1) as usize) & DB_INDEX_MASK) as usize);
        std::ptr::copy_nonoverlapping(line.cast::<u8>(), text_dst, len as usize);
        if (flags & ML_APPEND_MARK) != 0 {
            *db_idx_arr.add((db_idx + 1) as usize) |= DB_MARKED;
        }

        // Mark the block dirty.
        let cur_flags = nvim_buf_get_ml_flags(buf);
        let new_flags = cur_flags
            | ML_LOCKED_DIRTY
            | if (flags & ML_APPEND_NEW) == 0 {
                ML_LOCKED_POS
            } else {
                0
            };
        nvim_buf_set_ml_flags(buf, new_flags);
    } else {
        // -----------------------------------------------------------------------
        // Not enough space: create a new data block and copy some lines into it.
        // Then insert an entry in the pointer block. If that is also full, go up
        // the stack until we can insert (splitting pointer blocks as needed).
        // -----------------------------------------------------------------------
        let mut line_count_left: c_int;
        let mut line_count_right: c_int;
        let mut hp_new: *mut c_void;
        let lines_moved: c_int;
        let mut data_moved: c_int = 0;
        let mut total_moved: c_int = 0;
        let in_left: bool;

        // Decide whether the new line and/or moved lines go in the left or right block.
        if db_idx < 0 {
            // Left block is new, right block is the existing block.
            lines_moved = 0;
            in_left = true;
            // space_needed does not change
        } else {
            lines_moved = line_count - db_idx - 1;
            if lines_moved == 0 {
                in_left = false; // put new line in right (new) block
            } else {
                data_moved = ((*db_idx_arr.add(db_idx as usize) & DB_INDEX_MASK) as c_int)
                    - (*dp_header).db_txt_start as c_int;
                total_moved = data_moved + lines_moved * INDEX_SIZE as c_int;
                if (*dp_header).db_free as c_int + total_moved >= space_needed {
                    in_left = true; // put new line in left block
                                    // space_needed now only needs to hold total_moved
                } else {
                    in_left = false; // put new line in right block
                }
            }
        }
        // Recompute space_needed for the new block
        let space_for_new: c_int = if in_left && db_idx >= 0 && lines_moved > 0 {
            total_moved
        } else if !in_left && lines_moved > 0 {
            space_needed + total_moved
        } else {
            space_needed
        };
        let page_count =
            (space_for_new + DATA_BLOCK_HEADER_SIZE as c_int + page_size - 1) / page_size;

        hp_new = rs_ml_new_data(mfp, (flags & ML_APPEND_NEW) != 0, page_count);
        let hp_left: *mut c_void;
        let hp_right: *mut c_void;
        if db_idx < 0 {
            // Left block is new, right block is existing.
            hp_left = hp_new;
            hp_right = hp;
            line_count_left = 0;
            line_count_right = line_count;
        } else {
            // Right block is new, left block is existing.
            hp_left = hp;
            hp_right = hp_new;
            line_count_left = line_count;
            line_count_right = 0;
        }
        let dp_right_raw = nvim_bhdr_get_bh_data(hp_right);
        let dp_left_raw = nvim_bhdr_get_bh_data(hp_left);
        let dp_right = dp_right_raw.cast::<DataBlockHeader>();
        let dp_left = dp_left_raw.cast::<DataBlockHeader>();
        let dp_right_arr: *mut u32 = db_index_ptr(dp_right_raw);
        let dp_left_arr: *mut u32 = db_index_ptr(dp_left_raw);
        let bnum_left: BlockNr = nvim_bhdr_get_bh_bnum(hp_left);
        let bnum_right: BlockNr = nvim_bhdr_get_bh_bnum(hp_right);
        let page_count_left: c_int = nvim_bhdr_get_bh_page_count(hp_left);
        let page_count_right: c_int = nvim_bhdr_get_bh_page_count(hp_right);

        // May move the new line into the right (new) block.
        if !in_left {
            (*dp_right).db_txt_start -= len as u32;
            (*dp_right).db_free -= len as u32 + INDEX_SIZE as u32;
            *dp_right_arr.add(0) = (*dp_right).db_txt_start;
            if (flags & ML_APPEND_MARK) != 0 {
                *dp_right_arr.add(0) |= DB_MARKED;
            }
            std::ptr::copy_nonoverlapping(
                line.cast::<u8>(),
                dp_right_raw
                    .cast::<u8>()
                    .add((*dp_right).db_txt_start as usize),
                len as usize,
            );
            line_count_right += 1;
        }

        // May move lines from the left (old) block to the right (new) block.
        if lines_moved > 0 {
            (*dp_right).db_txt_start -= data_moved as u32;
            (*dp_right).db_free -= total_moved as u32;
            // Copy the text data
            std::ptr::copy_nonoverlapping(
                dp_left_raw
                    .cast::<u8>()
                    .add((*dp_left).db_txt_start as usize),
                dp_right_raw
                    .cast::<u8>()
                    .add((*dp_right).db_txt_start as usize),
                data_moved as usize,
            );
            let offset = (*dp_right).db_txt_start as c_int - (*dp_left).db_txt_start as c_int;
            (*dp_left).db_txt_start += data_moved as u32;
            (*dp_left).db_free += total_moved as u32;

            // Update indexes in the new right block.
            let mut to = line_count_right;
            for from in (db_idx + 1)..line_count_left {
                *dp_right_arr.add(to as usize) =
                    (*dp_left_arr.add(from as usize)).wrapping_add(offset as u32);
                to += 1;
            }
            line_count_right += lines_moved;
            line_count_left -= lines_moved;
        }

        // May move the new line into the left block.
        if in_left {
            (*dp_left).db_txt_start -= len as u32;
            (*dp_left).db_free -= len as u32 + INDEX_SIZE as u32;
            *dp_left_arr.add(line_count_left as usize) = (*dp_left).db_txt_start;
            if (flags & ML_APPEND_MARK) != 0 {
                *dp_left_arr.add(line_count_left as usize) |= DB_MARKED;
            }
            std::ptr::copy_nonoverlapping(
                line.cast::<u8>(),
                dp_left_raw
                    .cast::<u8>()
                    .add((*dp_left).db_txt_start as usize),
                len as usize,
            );
            line_count_left += 1;
        }

        // Compute lnum_left / lnum_right for recovery tracking.
        let (lnum_left, lnum_right): (LineNr, LineNr) = if db_idx < 0 {
            (lnum + 1, 0)
        } else {
            (0, if in_left { lnum + 2 } else { lnum + 1 })
        };
        (*dp_left).db_line_count = i64::from(line_count_left);
        (*dp_right).db_line_count = i64::from(line_count_right);

        // Release the two data blocks.
        if lines_moved > 0 || in_left {
            let cur_flags = nvim_buf_get_ml_flags(buf);
            nvim_buf_set_ml_flags(buf, cur_flags | ML_LOCKED_DIRTY);
        }
        if (flags & ML_APPEND_NEW) == 0 && db_idx >= 0 && in_left {
            let cur_flags = nvim_buf_get_ml_flags(buf);
            nvim_buf_set_ml_flags(buf, cur_flags | ML_LOCKED_POS);
        }
        mf_put(mfp, hp_new, true, false);

        // Flush the old data block.
        // Set ml_locked_lineadd to 0 because pointer block updates are done below.
        let lineadd = nvim_buf_get_ml_locked_lineadd(buf);
        nvim_buf_set_ml_locked_lineadd(buf, 0);
        rs_ml_find_line(buf, 0, ML_FLUSH); // flush data block

        // Update pointer blocks for the new data block.
        let stack_top = nvim_buf_get_ml_stack_top(buf);
        let mut bnum_left_cur: BlockNr = bnum_left;
        let mut bnum_right_cur: BlockNr = bnum_right;
        let mut page_count_left_cur = page_count_left;
        let mut page_count_right_cur = page_count_right;
        let mut line_count_left_cur = line_count_left;
        let mut line_count_right_cur = line_count_right;
        let mut lnum_left_cur = lnum_left;
        let mut lnum_right_cur = lnum_right;

        let mut stack_idx = stack_top - 1;
        while stack_idx >= 0 {
            let ip = nvim_buf_get_ml_stack_ip(buf, stack_idx);
            let pb_idx = nvim_ip_get_index(ip);
            let bnum = nvim_ip_get_bnum(ip);
            let block_hp = mf_get(mfp, bnum, 1);
            if block_hp.is_null() {
                return FAIL;
            }
            let pp = nvim_bhdr_get_bh_data(block_hp);
            let pp_header = pp.cast::<PointerBlockHeader>();
            if nvim_pp_get_id(pp) != PTR_ID {
                nvim_iemsg_pointer_block_id_wrong_three();
                mf_put(mfp, block_hp, false, false);
                return FAIL;
            }

            if (*pp_header).pb_count < (*pp_header).pb_count_max {
                // Block not full: insert one entry after pb_idx.
                let ptr_arr = pb_pointer_ptr(pp);
                let count = (*pp_header).pb_count as usize;
                if (pb_idx + 1) < c_int::from((*pp_header).pb_count) {
                    // Shift existing entries to make room.
                    std::ptr::copy(
                        ptr_arr.add((pb_idx + 1) as usize),
                        ptr_arr.add((pb_idx + 2) as usize),
                        count - (pb_idx + 1) as usize,
                    );
                }
                nvim_pp_inc_count(pp);
                (*ptr_arr.add(pb_idx as usize)).pe_line_count = line_count_left_cur as i32;
                (*ptr_arr.add(pb_idx as usize)).pe_bnum = bnum_left_cur;
                (*ptr_arr.add(pb_idx as usize)).pe_page_count = page_count_left_cur;
                (*ptr_arr.add((pb_idx + 1) as usize)).pe_line_count = line_count_right_cur as i32;
                (*ptr_arr.add((pb_idx + 1) as usize)).pe_bnum = bnum_right_cur;
                (*ptr_arr.add((pb_idx + 1) as usize)).pe_page_count = page_count_right_cur;
                if lnum_left_cur != 0 {
                    (*ptr_arr.add(pb_idx as usize)).pe_old_lnum = lnum_left_cur as i32;
                }
                if lnum_right_cur != 0 {
                    (*ptr_arr.add((pb_idx + 1) as usize)).pe_old_lnum = lnum_right_cur as i32;
                }

                mf_put(mfp, block_hp, true, false);
                nvim_buf_set_ml_stack_top(buf, stack_idx + 1); // truncate stack

                if lineadd != 0 {
                    let new_top = nvim_buf_get_ml_stack_top(buf) - 1;
                    nvim_buf_set_ml_stack_top(buf, new_top);
                    rs_ml_lineadd(buf, lineadd);
                    // fix stack itself
                    let top_ip = nvim_buf_get_ml_stack_ip(buf, nvim_buf_get_ml_stack_top(buf));
                    nvim_ip_add_high(top_ip, lineadd);
                    nvim_buf_set_ml_stack_top(buf, nvim_buf_get_ml_stack_top(buf) + 1);
                }

                // Done -- break the loop.
                crate::chunk::rs_ml_updatechunk(buf, lnum + 1, len as c_int, ML_CHNK_ADDLINE);
                return OK;
            }

            // Pointer block is full: split it.
            // Allocate a new pointer block. If this is block 1 (the root), grow
            // the tree by an extra level: copy block 1 to a new block, reset
            // block 1 to point to just the copy, then loop again to split the copy.
            // This mirrors the C `while (true)` that runs twice for block-1 splits.
            let mut cur_block_hp = block_hp;
            let mut cur_pp = pp;
            let mut cur_pb_idx = pb_idx;

            let hp_split_new = 'alloc_split: loop {
                hp_new = rs_ml_new_ptr(mfp);
                if hp_new.is_null() {
                    return FAIL;
                }
                let pp_new = nvim_bhdr_get_bh_data(hp_new);

                if nvim_bhdr_get_bh_bnum(cur_block_hp) != 1 {
                    // Normal case: not block 1, proceed with the split.
                    break 'alloc_split hp_new;
                }

                // Block 1 is full: grow the tree by one level.
                // Copy all of block 1 into hp_new, then reset block 1 to point
                // to just hp_new. Then loop again to allocate the actual split block.
                let cur_page_size = nvim_mf_get_page_size(mfp) as usize;
                std::ptr::copy_nonoverlapping(
                    cur_pp.cast::<u8>(),
                    pp_new.cast::<u8>(),
                    cur_page_size,
                );
                let pp_root = cur_pp.cast::<PointerBlockHeader>();
                (*pp_root).pb_count = 1;
                let root_arr = pb_pointer_ptr(cur_pp);
                (*root_arr).pe_bnum = nvim_bhdr_get_bh_bnum(hp_new);
                (*root_arr).pe_line_count = nvim_buf_get_ml_line_count(buf) as i32;
                (*root_arr).pe_old_lnum = 1;
                (*root_arr).pe_page_count = 1;
                mf_put(mfp, cur_block_hp, true, false); // release block 1

                // Now split hp_new (the copy of block 1).
                // The C code does: ip->ip_index = 0; stack_idx++;
                debug_assert!(
                    stack_idx == 0,
                    "stack_idx should be 0 when splitting block 1"
                );
                nvim_ip_set_index(ip, 0);
                stack_idx += 1;
                cur_block_hp = hp_new;
                cur_pp = pp_new;
                cur_pb_idx = 0;
                // Loop again to allocate the actual split block for cur_block_hp.
            };

            // Move pointers after cur_pb_idx into hp_split_new.
            let pp_split = nvim_bhdr_get_bh_data(hp_split_new);
            let ptr_arr_old = pb_pointer_ptr(cur_pp);
            let ptr_arr_new = pb_pointer_ptr(pp_split);
            let pp_header_cur = cur_pp.cast::<PointerBlockHeader>();
            let pp_split_header = pp_split.cast::<PointerBlockHeader>();

            total_moved = c_int::from((*pp_header_cur).pb_count) - cur_pb_idx - 1;
            if total_moved > 0 {
                std::ptr::copy_nonoverlapping(
                    ptr_arr_old.add((cur_pb_idx + 1) as usize),
                    ptr_arr_new,
                    total_moved as usize,
                );
                (*pp_split_header).pb_count = total_moved as u16;
                (*pp_header_cur).pb_count =
                    ((*pp_header_cur).pb_count - (total_moved as u16 - 1)) as u16;
                (*ptr_arr_old.add((cur_pb_idx + 1) as usize)).pe_bnum = bnum_right_cur;
                (*ptr_arr_old.add((cur_pb_idx + 1) as usize)).pe_line_count =
                    line_count_right_cur as i32;
                (*ptr_arr_old.add((cur_pb_idx + 1) as usize)).pe_page_count = page_count_right_cur;
                if lnum_right_cur != 0 {
                    (*ptr_arr_old.add((cur_pb_idx + 1) as usize)).pe_old_lnum =
                        lnum_right_cur as i32;
                }
            } else {
                (*pp_split_header).pb_count = 1;
                (*ptr_arr_new.add(0)).pe_bnum = bnum_right_cur;
                (*ptr_arr_new.add(0)).pe_line_count = line_count_right_cur as i32;
                (*ptr_arr_new.add(0)).pe_page_count = page_count_right_cur;
                (*ptr_arr_new.add(0)).pe_old_lnum = lnum_right_cur as i32;
            }
            (*ptr_arr_old.add(cur_pb_idx as usize)).pe_bnum = bnum_left_cur;
            (*ptr_arr_old.add(cur_pb_idx as usize)).pe_line_count = line_count_left_cur as i32;
            (*ptr_arr_old.add(cur_pb_idx as usize)).pe_page_count = page_count_left_cur;
            if lnum_left_cur != 0 {
                (*ptr_arr_old.add(cur_pb_idx as usize)).pe_old_lnum = lnum_left_cur as i32;
            }

            lnum_left_cur = 0;
            lnum_right_cur = 0;

            // Recompute line counts for the two pointer blocks.
            line_count_right_cur = 0;
            for i in 0..(*pp_split_header).pb_count as usize {
                line_count_right_cur += (*ptr_arr_new.add(i)).pe_line_count as c_int;
            }
            line_count_left_cur = 0;
            for i in 0..(*pp_header_cur).pb_count as usize {
                line_count_left_cur += (*ptr_arr_old.add(i)).pe_line_count as c_int;
            }

            bnum_left_cur = nvim_bhdr_get_bh_bnum(cur_block_hp);
            bnum_right_cur = nvim_bhdr_get_bh_bnum(hp_split_new);
            page_count_left_cur = 1;
            page_count_right_cur = 1;
            mf_put(mfp, cur_block_hp, true, false);
            mf_put(mfp, hp_split_new, true, false);

            stack_idx -= 1;
        }

        // Safety check: fell out of the for loop without inserting?
        if stack_idx < 0 {
            nvim_iemsg_e318_updated_too_many();
            nvim_buf_set_ml_stack_top(buf, 0); // invalidate stack
        }
    }

    // The line was inserted below lnum.
    crate::chunk::rs_ml_updatechunk(buf, lnum + 1, len as c_int, ML_CHNK_ADDLINE);
    OK
}

/// Implement `ml_append_flush`: flush pending cached line if needed, then
/// call `rs_ml_append_int` (Rust implementation).
///
/// This is a Rust port of the static C `ml_append_flush` function.
///
/// # Safety
/// - `buf` must be a valid buffer pointer (non-null)
///
/// Modifies buffer state. Only call from main Nvim thread.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_append_flush(
    buf: *mut BufHandle,
    lnum: LineNr,
    line: *mut c_char,
    len: ColNr,
    flags: c_int,
) -> c_int {
    if lnum > nvim_buf_get_ml_line_count(buf) {
        return FAIL; // lnum out of range
    }
    if nvim_buf_get_ml_line_lnum(buf) != 0 {
        // This may also invoke rs_ml_append_int().
        rs_ml_flush_line(buf, 0);
    }
    rs_ml_append_int(buf, lnum, line, len, flags)
}

// =============================================================================
// Line Cache Functions
// =============================================================================

/// Flush the cached line to the data block.
///
/// If the line has been changed (`ML_LINE_DIRTY`), finds the data block,
/// and either updates in-place (if the new line fits) or does a
/// delete-and-append cycle.
///
/// Uses a static reentrancy guard: if called recursively, does nothing.
///
/// # Arguments
/// * `buf` - Buffer to flush
/// * `noalloc` - If non-zero, the caller manages line memory (don't xfree)
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
/// - Only call from the main Neovim thread (static ENTERED guard is not thread-safe)
#[export_name = "ml_flush_line"]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]
pub unsafe extern "C" fn rs_ml_flush_line(buf: *mut BufHandle, noalloc: c_int) {
    // Reentrancy guard: rs_ml_flush_line must not be called recursively.
    // Neovim is single-threaded so this static is safe.
    static mut ENTERED: bool = false;

    if buf.is_null() {
        return;
    }

    if nvim_buf_get_ml_line_lnum(buf) == 0 || nvim_buf_has_ml_mfp(buf) == 0 {
        return; // nothing to do
    }

    let flags = nvim_buf_get_ml_flags(buf);

    if (flags & ML_LINE_DIRTY) != 0 {
        // This code doesn't work recursively.
        if ENTERED {
            return;
        }
        ENTERED = true;

        nvim_buf_inc_flush_count(buf);

        let lnum = nvim_buf_get_ml_line_lnum(buf);
        let new_line: *mut c_char = nvim_buf_get_ml_line_ptr(buf);

        let hp = rs_ml_find_line(buf, lnum, ML_FIND);
        if hp.is_null() {
            nvim_siemsg_e320_cannot_find_line(lnum);
        } else {
            let dp_raw = nvim_bhdr_get_bh_data(hp);
            let dp = dp_raw.cast::<u8>();
            let dp_header = dp_raw.cast::<DataBlockHeader>();

            let idx = (lnum - nvim_buf_get_ml_locked_low(buf)) as usize;

            // db_index array follows immediately after the DataBlockHeader
            let db_index: *mut u32 = dp.add(DATA_BLOCK_HEADER_SIZE).cast();

            let start = (*db_index.add(idx) & DB_INDEX_MASK) as usize;
            let old_len: usize = if idx == 0 {
                // line is last in block - text is at the end
                (*dp_header).db_txt_end as usize - start
            } else {
                // text of previous (higher) line follows
                (*db_index.add(idx - 1) & DB_INDEX_MASK) as usize - start
            };

            let new_len = nvim_buf_get_ml_line_len(buf) as usize;
            // extra is positive if line grows, negative if it shrinks
            let extra = new_len as i64 - old_len as i64;

            // If new line fits in data block, replace directly
            if i64::from((*dp_header).db_free) >= extra {
                // If length changes and there are following lines, shift text
                let count = (nvim_buf_get_ml_locked_high(buf) - nvim_buf_get_ml_locked_low(buf) + 1)
                    as usize;

                if extra != 0 && idx < count - 1 {
                    // Move text of following lines to make room (or fill gap)
                    let txt_start = (*dp_header).db_txt_start as usize;
                    let src = dp.add(txt_start);
                    let dst_offset = (txt_start as i64 - extra) as usize;
                    let dst = dp.add(dst_offset);
                    let copy_len = start - txt_start;
                    std::ptr::copy(src, dst, copy_len);

                    // Adjust db_index entries for lines above idx
                    for i in (idx + 1)..count {
                        let cur = *db_index.add(i) & DB_INDEX_MASK;
                        let new_val = (i64::from(cur) - extra) as u32;
                        *db_index.add(i) = new_val | (*db_index.add(i) & DB_MARKED);
                    }
                }

                // Update this line's index entry (subtract extra to get new start)
                let old_start_val = *db_index.add(idx);
                let new_start = (start as i64 - extra) as u32;
                *db_index.add(idx) = new_start | (old_start_val & DB_MARKED);

                // Adjust free space and text start
                let old_free = i64::from((*dp_header).db_free);
                (*dp_header).db_free = (old_free - extra) as u32;
                let old_txt_start = i64::from((*dp_header).db_txt_start);
                (*dp_header).db_txt_start = (old_txt_start - extra) as u32;

                // Copy new line into the data block (at the adjusted position)
                let old_line_ptr = dp.add(start);
                let dest = old_line_ptr.offset(-extra as isize);
                std::ptr::copy_nonoverlapping(new_line.cast::<u8>(), dest, new_len);

                // Mark the locked block dirty and needing a positive block number
                let cur_flags = nvim_buf_get_ml_flags(buf);
                nvim_buf_set_ml_flags(buf, cur_flags | ML_LOCKED_DIRTY | ML_LOCKED_POS);

                // Update chunk tracking if size changed
                if extra != 0 {
                    crate::chunk::rs_ml_updatechunk(buf, lnum, extra as c_int, ML_CHNK_UPDLINE);
                }
            } else {
                // Cannot fit in one data block: Delete and append.
                // Append first, because ml_delete_int() cannot delete the
                // last line in a buffer (would leave it empty).
                // Preserve the mark flag when appending.
                let marked_flag = if (*db_index.add(idx) & DB_MARKED) != 0 {
                    ML_APPEND_MARK
                } else {
                    0
                };
                rs_ml_append_int(buf, lnum, new_line, new_len as ColNr, marked_flag);
                rs_ml_delete_int(buf, lnum, 0);
            }
        }

        if noalloc == 0 {
            xfree(new_line.cast::<c_void>());
        }

        ENTERED = false;
    } else if (flags & ML_ALLOCATED) != 0 {
        // Line was allocated (e.g. for address sanitizer) but not dirtied.
        // assert(!noalloc) -- caller must set ML_LINE_DIRTY with noalloc
        xfree(nvim_buf_get_ml_line_ptr(buf).cast::<c_void>());
    }

    // Clear the dirty/allocated flags and invalidate the line cache
    let cur_flags = nvim_buf_get_ml_flags(buf);
    nvim_buf_set_ml_flags(buf, cur_flags & !(ML_LINE_DIRTY | ML_ALLOCATED));
    nvim_buf_set_ml_line_lnum(buf, 0);
    nvim_buf_set_ml_line_offset(buf, 0);
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
#[export_name = "ml_add_deleted_len"]
pub unsafe extern "C" fn rs_ml_add_deleted_len(ptr: *mut c_char, len: isize) {
    let buf = nvim_get_curbuf();
    rs_ml_add_deleted_len_buf(buf, ptr, len);
}

/// Track deleted text length for a specific buffer.
///
/// Checks inhibit_delete_count; uses strlen if len == -1 or len > maxlen.
/// Updates deleted_bytes, deleted_bytes2, and optionally codepoints/codeunits.
///
/// # Arguments
/// * `buf` - Buffer to track for
/// * `ptr` - Pointer to the deleted text
/// * `len` - Length of deleted text, or -1 to use strlen
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
/// - `ptr` must be a valid C string
#[export_name = "ml_add_deleted_len_buf"]
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
pub unsafe extern "C" fn rs_ml_add_deleted_len_buf(
    buf: *mut BufHandle,
    ptr: *mut c_char,
    len: isize,
) {
    if buf.is_null() {
        return;
    }
    if nvim_get_inhibit_delete_count() != 0 {
        return;
    }
    let maxlen = strlen(ptr) as isize;
    let actual_len = if len == -1 || len > maxlen {
        maxlen
    } else {
        len
    };
    let nbytes = actual_len as usize + 1; // +1 for NL
    nvim_buf_add_deleted_bytes(buf, nbytes);
    nvim_buf_add_deleted_bytes2(buf, nbytes);
    if nvim_buf_get_update_need_codepoints(buf) {
        let mut cp: usize = 0;
        let mut cu: usize = 0;
        mb_utflen(ptr, actual_len as usize, &raw mut cp, &raw mut cu);
        nvim_buf_add_deleted_codepoints(buf, cp + 1); // +1 for NL char
        nvim_buf_add_deleted_codeunits(buf, cu + 1);
    }
}

/// Add a new entry to the B-tree info pointer stack for a buffer.
///
/// Grows the stack array if needed (by STACK_INCR entries at a time).
/// Returns the index of the new top entry (the old stack_top value).
///
/// # Safety
/// - `buf` must be a valid buffer pointer
#[no_mangle]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
pub unsafe extern "C" fn rs_ml_add_stack(buf: *mut BufHandle) -> c_int {
    let top = nvim_buf_get_ml_stack_top(buf);
    let stack_size = nvim_buf_get_ml_stack_size(buf);
    if top == stack_size {
        let new_size = (stack_size as usize + STACK_INCR) * nvim_get_infoptr_size();
        let old_ptr = nvim_buf_get_ml_stack(buf);
        let new_ptr = xrealloc(old_ptr, new_size);
        nvim_buf_set_ml_stack(buf, new_ptr);
        nvim_buf_set_ml_stack_size(buf, stack_size + STACK_INCR as c_int);
    }
    nvim_buf_inc_ml_stack_top(buf)
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
#[export_name = "ml_setmarked"]
pub unsafe extern "C" fn rs_ml_setmarked(lnum: LineNr) {
    let buf = nvim_get_curbuf();
    if lnum < 1 || lnum > nvim_buf_get_ml_line_count(buf) || nvim_buf_has_ml_mfp(buf) == 0 {
        return;
    }

    if LOWEST_MARKED == 0 || LOWEST_MARKED > lnum {
        LOWEST_MARKED = lnum;
    }

    // Find the data block containing the line.
    let hp = rs_ml_find_line(buf, lnum, crate::types::ML_FIND);
    if hp.is_null() {
        return;
    }
    let dp = nvim_bhdr_get_bh_data(hp);
    let db_idx = db_index_ptr(dp);
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
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
#[export_name = "ml_firstmarked"]
pub unsafe extern "C" fn rs_ml_firstmarked() -> LineNr {
    let buf = nvim_get_curbuf();
    if nvim_buf_has_ml_mfp(buf) == 0 {
        return 0;
    }

    let mut lnum = LOWEST_MARKED;
    let line_count = nvim_buf_get_ml_line_count(buf);

    while lnum <= line_count {
        let hp = rs_ml_find_line(buf, lnum, crate::types::ML_FIND);
        if hp.is_null() {
            return 0;
        }
        let dp = nvim_bhdr_get_bh_data(hp);
        let db_idx = db_index_ptr(dp);

        let locked_low = nvim_buf_get_ml_locked_low(buf);
        let locked_high = nvim_buf_get_ml_locked_high(buf);

        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
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
#[export_name = "ml_clearmarked"]
pub unsafe extern "C" fn rs_ml_clearmarked() {
    let buf = nvim_get_curbuf();
    if nvim_buf_has_ml_mfp(buf) == 0 {
        return;
    }

    let mut lnum = LOWEST_MARKED;
    let line_count = nvim_buf_get_ml_line_count(buf);

    while lnum <= line_count {
        let hp = rs_ml_find_line(buf, lnum, crate::types::ML_FIND);
        if hp.is_null() {
            return;
        }
        let dp = nvim_bhdr_get_bh_data(hp);
        let db_idx = db_index_ptr(dp);

        let locked_low = nvim_buf_get_ml_locked_low(buf);
        let locked_high = nvim_buf_get_ml_locked_high(buf);

        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
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
// Phase 6: rs_ml_delete_int -- core B-tree line deletion
// =============================================================================

/// Delete line `lnum` from buffer `buf`.
///
/// This is the Rust port of the C `ml_delete_int` function. It handles:
/// - Empty-buffer case: replaces last line with empty string
/// - Multi-line block: shifts text and index entries, updates free space
/// - Single-line block (block becomes empty): frees data block, walks up
///   the pointer block stack to remove the entry, collapsing empty pointer
///   blocks as needed.
///
/// # Returns
/// OK (1) on success, FAIL (0) on failure.
///
/// # Safety
/// - `buf` must be a valid, non-null buffer pointer with an initialized memline.
/// - Must only be called from the main Neovim thread.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]
pub unsafe extern "C" fn rs_ml_delete_int(
    buf: *mut BufHandle,
    lnum: LineNr,
    flags: c_int,
) -> c_int {
    rs_ml_adjust_lowest_marked_for_delete(lnum);

    // If the file becomes empty, replace the last line with an empty line.
    if nvim_buf_get_ml_line_count(buf) == 1 {
        if (flags & ML_DEL_MESSAGE) != 0 {
            nvim_set_keep_msg_no_lines();
        }
        let empty: &[u8] = b"\0";
        let i = rs_ml_replace_buf_impl(
            buf,
            1,
            empty.as_ptr().cast::<c_char>().cast_mut(),
            1, // copy=true
            0, // noalloc=false
        );
        let cur_flags = nvim_buf_get_ml_flags(buf);
        nvim_buf_set_ml_flags(buf, cur_flags | ML_EMPTY);
        return i;
    }

    // Find the data block containing the line.
    // This also fills the stack with the blocks from root to data block,
    // and releases any previously locked block.
    let mfp = nvim_buf_get_ml_mfp(buf);
    if mfp.is_null() {
        return FAIL;
    }

    let hp = rs_ml_find_line(buf, lnum, crate::types::ML_DELETE);
    if hp.is_null() {
        return FAIL;
    }

    let dp_raw = nvim_bhdr_get_bh_data(hp);
    let dp = dp_raw.cast::<u8>();
    let dp_header = dp_raw.cast::<DataBlockHeader>();

    // count = number of lines in block before the delete
    let count = (nvim_buf_get_ml_locked_high(buf) - nvim_buf_get_ml_locked_low(buf) + 2) as usize;
    let idx = (lnum - nvim_buf_get_ml_locked_low(buf)) as usize;

    // Update b_prev_line_count if not already set
    if nvim_buf_get_b_prev_line_count(buf) == 0 {
        nvim_buf_set_b_prev_line_count(buf, nvim_buf_get_ml_line_count(buf));
    }
    nvim_buf_dec_ml_line_count(buf);

    // db_index array follows immediately after the DataBlockHeader
    let db_index: *mut u32 = dp.add(DATA_BLOCK_HEADER_SIZE).cast();

    let line_start = (*db_index.add(idx) & DB_INDEX_MASK) as usize;
    let line_size: usize = if idx == 0 {
        // first line in block, text is at the end
        (*dp_header).db_txt_end as usize - line_start
    } else {
        (*db_index.add(idx - 1) & DB_INDEX_MASK) as usize - line_start
    };

    // Line must always have at least 1 byte (the NUL/NL terminator)
    debug_assert!(line_size >= 1);
    rs_ml_add_deleted_len_buf(
        buf,
        dp.add(line_start).cast::<c_char>(),
        (line_size - 1) as isize,
    );

    let mut ret = FAIL;

    if count == 1 {
        // Special case: only one line in the data block -- it becomes empty.
        // Free the data block and walk up the pointer block stack to remove
        // the pointer entry. If a pointer block becomes empty, keep going up.
        nvim_mf_free(mfp, hp);
        nvim_buf_set_ml_locked(buf, std::ptr::null_mut());

        let stack_top = nvim_buf_get_ml_stack_top(buf);
        let mut stack_idx = stack_top - 1;
        while stack_idx >= 0 {
            nvim_buf_set_ml_stack_top(buf, 0); // stack is invalid when failing
            let ip = nvim_buf_get_ml_stack_ip(buf, stack_idx);
            let cur_idx = nvim_ip_get_index(ip) as usize;
            let bnum = nvim_ip_get_bnum(ip);
            let block_hp = mf_get(mfp, bnum, 1);
            if block_hp.is_null() {
                // goto theend (ret is FAIL)
                return ret;
            }
            let pp = nvim_bhdr_get_bh_data(block_hp);
            if nvim_pp_get_id(pp) != PTR_ID {
                nvim_iemsg_pointer_block_id_wrong_four();
                mf_put(mfp, block_hp, false, false);
                return ret;
            }
            let new_count = nvim_pp_dec_count(pp) as usize;
            if new_count == 0 {
                // pointer block becomes empty too -- free it and keep going up
                nvim_mf_free(mfp, block_hp);
            } else {
                if new_count != cur_idx {
                    // move entries after the deleted one to fill the gap
                    nvim_pp_pe_memmove(
                        pp,
                        cur_idx as c_int,
                        cur_idx as c_int + 1,
                        (new_count - cur_idx) as c_int,
                    );
                }
                mf_put(mfp, block_hp, true, false);

                nvim_buf_set_ml_stack_top(buf, stack_idx); // truncate stack

                // fix line count for remaining blocks in the stack
                let lineadd = nvim_buf_get_ml_locked_lineadd(buf);
                if lineadd != 0 {
                    rs_ml_lineadd(buf, lineadd);
                    let top_ip = nvim_buf_get_ml_stack_ip(buf, nvim_buf_get_ml_stack_top(buf));
                    nvim_ip_add_high(top_ip, lineadd);
                }
                nvim_buf_set_ml_stack_top(buf, nvim_buf_get_ml_stack_top(buf) + 1);

                ret = OK;
                break;
            }
            stack_idx -= 1;
        }
        // CHECK(stack_idx < 0, "deleted block 1?") -- we just skip it if stack emptied
    } else {
        // Normal case: multiple lines in the block -- shift text and indexes.

        // Delete the text by moving subsequent lines' text forward
        let text_start = (*dp_header).db_txt_start as usize;
        let src = dp.add(text_start);
        let dst = dp.add(text_start + line_size);
        std::ptr::copy(src, dst, line_start - text_start);

        // Delete the index entry by shifting subsequent entries,
        // adjusting each offset to account for the text movement.
        for i in idx..(count - 1) {
            *db_index.add(i) = (*db_index.add(i + 1)).wrapping_add(line_size as u32);
        }

        (*dp_header).db_free += line_size as u32 + INDEX_SIZE as u32;
        (*dp_header).db_txt_start += line_size as u32;
        (*dp_header).db_line_count -= 1;

        // Mark the block dirty and needing a positive block number (for recovery)
        let cur_flags = nvim_buf_get_ml_flags(buf);
        nvim_buf_set_ml_flags(buf, cur_flags | ML_LOCKED_DIRTY | ML_LOCKED_POS);

        ret = OK;
    }

    crate::chunk::rs_ml_updatechunk(buf, lnum, line_size as c_int, ML_CHNK_DELLINE);

    ret
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
