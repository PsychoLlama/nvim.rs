//! Byte offset chunk cache maintenance for the memline system.
//!
//! This module provides the Rust implementation of `ml_updatechunk`, which
//! maintains the `ml_chunksize` array -- a cache of (numlines, totalsize) pairs
//! used to speed up `line2byte()` and `byte2line()` conversions.
//!
//! # Chunk Structure
//!
//! The buffer's text is divided into chunks of at most `MLCS_MAXL` (800) lines.
//! Each chunk records:
//! - `mlcs_numlines` -- number of lines in this chunk
//! - `mlcs_totalsize` -- total byte size of all lines in this chunk
//!
//! # Update Types
//!
//! - `ML_CHNK_ADDLINE` -- a line was appended; may split a chunk that becomes too large
//! - `ML_CHNK_DELLINE` -- a line was deleted; may collapse two adjacent small chunks
//! - `ML_CHNK_UPDLINE` -- a line was replaced in place; only the size changes
//!
//! # Cache Variables
//!
//! The four `static mut` variables mirror the C file-scope statics:
//! - `ML_UPD_LASTBUF` -- buffer pointer from last call (NULL if invalid)
//! - `ML_UPD_LASTLINE` -- line number of last addline call
//! - `ML_UPD_LASTCURLINE` -- start-of-chunk line at last call
//! - `ML_UPD_LASTCURIX` -- chunk index at last call
//!
//! These are only valid when `ML_UPD_LASTBUF == buf` and the update type is
//! `ML_CHNK_ADDLINE` with consecutive line numbers. Neovim is single-threaded
//! so `static mut` is safe here.

use std::ffi::c_int;

use nvim_buffer::buf_struct::BufStruct;

use crate::types::{
    BufHandle, ChunkSizeEntry, DataBlockHeader, LineNr, DATA_BLOCK_HEADER_SIZE, DB_INDEX_MASK,
    ML_CHNK_ADDLINE, ML_CHNK_DELLINE, ML_CHNK_UPDLINE, ML_FIND,
};

// =============================================================================
// Chunk Cache Constants
// =============================================================================

/// Maximum number of lines per chunk before it is split.
const MLCS_MAXL: c_int = 800;

/// Minimum lines per chunk (target for each half after a split).
const MLCS_MINL: c_int = 400;

// =============================================================================
// C Accessor Declarations
// =============================================================================

extern "C" {
    /// Allocate memory (like malloc, but panics on OOM)
    fn xmalloc(size: usize) -> *mut std::ffi::c_void;

    /// Reallocate memory
    fn xrealloc(ptr: *mut std::ffi::c_void, size: usize) -> *mut std::ffi::c_void;

    /// Get bh_data pointer from block header
    fn nvim_bhdr_get_bh_data(hp: *mut std::ffi::c_void) -> *mut std::ffi::c_void;

    /// Find a data block (the B-tree traversal function)
    fn rs_ml_find_line(buf: *mut BufHandle, lnum: LineNr, action: c_int) -> *mut std::ffi::c_void;
}

// =============================================================================
// Direct-access helpers
// =============================================================================

/// Get a typed pointer to the chunksize array for a buffer.
///
/// # Safety
/// `buf` must be a valid buffer pointer; `ml_chunksize` must be initialized (non-null).
#[inline]
unsafe fn chunksize_ptr(buf: *mut BufHandle) -> *mut ChunkSizeEntry {
    (*buf.cast::<BufStruct>())
        .ml_chunksize
        .cast::<ChunkSizeEntry>()
}

// =============================================================================
// Cache State (mirrors C file-scope statics in ml_updatechunk)
// =============================================================================

/// Last buffer pointer this cache is valid for (NULL means invalid).
///
/// # Safety
/// Only accessed from the main Neovim thread (single-threaded).
static mut ML_UPD_LASTBUF: *mut BufHandle = std::ptr::null_mut();

/// Line number of the last `ML_CHNK_ADDLINE` call.
static mut ML_UPD_LASTLINE: LineNr = 0;

/// Start-of-chunk line number cached from the last call.
static mut ML_UPD_LASTCURLINE: LineNr = 0;

/// Chunk index cached from the last call.
static mut ML_UPD_LASTCURIX: c_int = 0;

// =============================================================================
// Chunk array lifecycle (migrated from C shim)
// =============================================================================

/// Allocate initial ml_chunksize array (100 entries), set first entry to (1, 1).
///
/// # Safety
/// `buf` must be a valid buffer pointer.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
unsafe fn ml_chunksize_init(buf: *mut BufHandle) {
    let ptr = xmalloc(std::mem::size_of::<ChunkSizeEntry>() * 100);
    (*buf.cast::<BufStruct>()).ml_chunksize = ptr;
    (*buf.cast::<BufStruct>()).ml_numchunks = 100;
    (*buf.cast::<BufStruct>()).ml_usedchunks = 1;
    (*chunksize_ptr(buf).add(0)).mlcs_numlines = 1;
    (*chunksize_ptr(buf).add(0)).mlcs_totalsize = 1;
}

/// Ensure the ml_chunksize array has capacity for usedchunks+1.
///
/// Grows the array by factor 3/2 if needed.
/// # Safety
/// `buf` must be a valid buffer pointer with an initialized chunksize array.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
unsafe fn ml_chunksize_ensure_capacity(buf: *mut BufHandle) {
    let bs = buf.cast::<BufStruct>();
    let usedchunks = (*bs).ml_usedchunks;
    let numchunks = (*bs).ml_numchunks;
    if usedchunks + 1 >= numchunks {
        let new_numchunks = numchunks * 3 / 2;
        (*bs).ml_numchunks = new_numchunks;
        let old_ptr = (*bs).ml_chunksize;
        let new_ptr = xrealloc(
            old_ptr,
            std::mem::size_of::<ChunkSizeEntry>() * new_numchunks as usize,
        );
        (*bs).ml_chunksize = new_ptr;
    }
}

// =============================================================================
// Implementation
// =============================================================================

/// Update the byte offset chunk cache after a line was added, deleted, or changed.
///
/// This is the Rust port of the C `ml_updatechunk` function.
///
/// # Arguments
/// * `buf`     - Buffer whose chunk cache to update
/// * `line`    - Line number that was modified (1-based)
/// * `len`     - Byte size of the line (positive for add/upd, raw value for del)
/// * `updtype` - One of `ML_CHNK_ADDLINE`, `ML_CHNK_DELLINE`, `ML_CHNK_UPDLINE`
///
/// # Safety
/// - `buf` must be a valid, non-null buffer pointer.
/// - Must only be called from the main Neovim thread.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]
pub unsafe extern "C" fn rs_ml_updatechunk(
    buf: *mut BufHandle,
    line: LineNr,
    len: c_int,
    updtype: c_int,
) {
    let bs = buf.cast::<BufStruct>();

    // Quick exits
    if (*bs).ml_usedchunks == -1 || len == 0 {
        return;
    }

    // Initialize chunk array if it doesn't exist yet
    if (*bs).ml_chunksize.is_null() {
        ml_chunksize_init(buf);
    }

    // Special case: single-line buffer updated via ml_flush_line -- reset cache
    if updtype == ML_CHNK_UPDLINE && (*bs).ml_line_count == 1 {
        (*bs).ml_usedchunks = 1;
        (*chunksize_ptr(buf).add(0)).mlcs_numlines = 1;
        (*chunksize_ptr(buf).add(0)).mlcs_totalsize = (*bs).ml_line_len;
        return;
    }

    // Find the chunk index (curix) that contains `line`.
    // `curline` is the line number at the start of chunk `curix`.
    let mut curline: LineNr;
    let mut curix: c_int;

    let usedchunks = (*bs).ml_usedchunks;

    if ML_UPD_LASTBUF != buf || line != ML_UPD_LASTLINE + 1 || updtype != ML_CHNK_ADDLINE {
        // Full scan: find the chunk that contains `line`
        curline = 1;
        curix = 0;
        while curix < usedchunks - 1 {
            let chunk_lines = LineNr::from((*chunksize_ptr(buf).add(curix as usize)).mlcs_numlines);
            if line < curline + chunk_lines {
                break;
            }
            curline += chunk_lines;
            curix += 1;
        }
    } else {
        // Use cached position from last ADDLINE call, but advance if needed
        curline = ML_UPD_LASTCURLINE;
        curix = ML_UPD_LASTCURIX;
        if curix < usedchunks - 1 {
            let chunk_lines = LineNr::from((*chunksize_ptr(buf).add(curix as usize)).mlcs_numlines);
            if line >= curline + chunk_lines {
                curline += chunk_lines;
                curix += 1;
            }
        }
    }

    // Apply the size change to curchnk->mlcs_totalsize
    let size_delta = if updtype == ML_CHNK_DELLINE {
        -len
    } else {
        len
    };
    (*chunksize_ptr(buf).add(curix as usize)).mlcs_totalsize += size_delta;

    if updtype == ML_CHNK_ADDLINE {
        (*chunksize_ptr(buf).add(curix as usize)).mlcs_numlines += 1;

        // Grow the array if needed before potentially splitting
        ml_chunksize_ensure_capacity(buf);

        let curchnk_numlines = (*chunksize_ptr(buf).add(curix as usize)).mlcs_numlines;
        if curchnk_numlines >= MLCS_MAXL {
            // Chunk is too large -- split it.
            // Insert a copy of curix at curix+1 by shifting entries right.
            let usedchunks2 = (*bs).ml_usedchunks;
            std::ptr::copy(
                chunksize_ptr(buf).add(curix as usize),
                chunksize_ptr(buf).add((curix + 1) as usize),
                (usedchunks2 - curix) as usize,
            );

            // Count MLCS_MINL lines from curline to determine split point.
            let mut size: c_int = 0;
            let mut linecnt: c_int = 0;
            let mut scan_curline = curline;
            while scan_curline < LineNr::from((*bs).ml_line_count) && linecnt < MLCS_MINL {
                let hp = rs_ml_find_line(buf, scan_curline, ML_FIND);
                if hp.is_null() {
                    (*bs).ml_usedchunks = -1;
                    return;
                }
                let dp_raw = nvim_bhdr_get_bh_data(hp);
                let dp = dp_raw.cast::<u8>();
                let dp_header = dp_raw.cast::<DataBlockHeader>();

                let count = (LineNr::from((*bs).ml_locked_high) - LineNr::from((*bs).ml_locked_low)
                    + 1) as c_int;
                let idx = (scan_curline - LineNr::from((*bs).ml_locked_low)) as c_int;
                scan_curline = LineNr::from((*bs).ml_locked_high) + 1;

                let rest = count - idx;
                let end_idx: c_int;
                if linecnt + rest > MLCS_MINL {
                    end_idx = idx + MLCS_MINL - linecnt - 1;
                    linecnt = MLCS_MINL;
                } else {
                    end_idx = count - 1;
                    linecnt += rest;
                }

                let db_index: *const u32 = dp.add(DATA_BLOCK_HEADER_SIZE).cast();
                let text_end: c_int = if idx == 0 {
                    (*dp_header).db_txt_end as c_int
                } else {
                    (*db_index.add((idx - 1) as usize) & DB_INDEX_MASK) as c_int
                };
                size += text_end - (*db_index.add(end_idx as usize) & DB_INDEX_MASK) as c_int;
            }

            // Write the split: left half gets linecnt lines, right half gets the rest
            let curix_e = chunksize_ptr(buf).add(curix as usize);
            let next_e = chunksize_ptr(buf).add((curix + 1) as usize);
            (*next_e).mlcs_numlines -= linecnt;
            (*curix_e).mlcs_numlines = linecnt;
            (*next_e).mlcs_totalsize -= size;
            (*curix_e).mlcs_totalsize = size;
            (*bs).ml_usedchunks += 1;

            ML_UPD_LASTBUF = std::ptr::null_mut(); // force recalc next time
            return;
        } else if curchnk_numlines >= MLCS_MINL
            && curix == (*bs).ml_usedchunks - 1
            && LineNr::from((*bs).ml_line_count) - line <= 1
        {
            // We are in the last chunk and it is cheap to create a new one
            // after this. Do it now to avoid the full-scan loop later.
            let new_curix = curix + 1;
            (*bs).ml_usedchunks += 1;

            if line == LineNr::from((*bs).ml_line_count) {
                (*chunksize_ptr(buf).add(new_curix as usize)).mlcs_numlines = 0;
                (*chunksize_ptr(buf).add(new_curix as usize)).mlcs_totalsize = 0;
            } else {
                // Line is just prior to the last line -- move the last line's
                // count to the new chunk. This is the common case when loading.
                let hp = rs_ml_find_line(buf, LineNr::from((*bs).ml_line_count), ML_FIND);
                if hp.is_null() {
                    (*bs).ml_usedchunks = -1;
                    return;
                }
                let dp_raw = nvim_bhdr_get_bh_data(hp);
                let dp = dp_raw.cast::<u8>();
                let dp_header = dp_raw.cast::<DataBlockHeader>();
                let db_index: *const u32 = dp.add(DATA_BLOCK_HEADER_SIZE).cast();

                let rest: c_int = if (*dp_header).db_line_count == 1 {
                    ((*dp_header).db_txt_end - (*dp_header).db_txt_start) as c_int
                } else {
                    let idx = (*dp_header).db_line_count as usize - 2;
                    (*db_index.add(idx) & DB_INDEX_MASK) as c_int
                        - (*dp_header).db_txt_start as c_int
                };

                let new_e = chunksize_ptr(buf).add(new_curix as usize);
                let cur_e = chunksize_ptr(buf).add(curix as usize);
                (*new_e).mlcs_totalsize = rest;
                (*new_e).mlcs_numlines = 1;
                (*cur_e).mlcs_totalsize -= rest;
                (*cur_e).mlcs_numlines -= 1;
            }
        }
        // (fall through to update cache variables below)
    } else if updtype == ML_CHNK_DELLINE {
        (*chunksize_ptr(buf).add(curix as usize)).mlcs_numlines -= 1;
        ML_UPD_LASTBUF = std::ptr::null_mut(); // force recalc next time

        let usedchunks3 = (*bs).ml_usedchunks;
        let curchnk_numlines2 = (*chunksize_ptr(buf).add(curix as usize)).mlcs_numlines;

        if curix < usedchunks3 - 1 {
            let next_numlines = (*chunksize_ptr(buf).add((curix + 1) as usize)).mlcs_numlines;
            if curchnk_numlines2 + next_numlines <= MLCS_MINL {
                // Merge with next chunk
                let new_curix = curix + 1;
                collapse_chunks(buf, new_curix, usedchunks3);
                return;
            }
        } else if curix == 0 && curchnk_numlines2 <= 0 {
            // First chunk is now empty -- remove it
            let new_used = usedchunks3 - 1;
            (*bs).ml_usedchunks = new_used;
            std::ptr::copy(
                chunksize_ptr(buf).add(1),
                chunksize_ptr(buf).add(0),
                new_used as usize,
            );
            return;
        } else if curix == 0
            || (curchnk_numlines2 > 10
                && (curchnk_numlines2
                    + (*chunksize_ptr(buf).add((curix - 1) as usize)).mlcs_numlines)
                    > MLCS_MINL)
        {
            // Current chunk is still large enough -- nothing more to do
            return;
        }

        // Collapse curix into curix-1
        collapse_chunks(buf, curix, usedchunks3);
        return;
    }

    // Update cache variables (for ADDLINE and UPDLINE)
    ML_UPD_LASTBUF = buf;
    ML_UPD_LASTLINE = line;
    ML_UPD_LASTCURLINE = curline;
    ML_UPD_LASTCURIX = curix;
}

/// Merge chunk `curix` into chunk `curix - 1` and remove `curix`.
///
/// # Safety
/// `buf` must be valid; `curix >= 1`; `usedchunks` is the current count.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
unsafe fn collapse_chunks(buf: *mut BufHandle, curix: c_int, usedchunks: c_int) {
    let bs = buf.cast::<BufStruct>();
    let prev = (curix - 1) as usize;
    let cur_numlines = (*chunksize_ptr(buf).add(curix as usize)).mlcs_numlines;
    let cur_totalsize = (*chunksize_ptr(buf).add(curix as usize)).mlcs_totalsize;
    (*chunksize_ptr(buf).add(prev)).mlcs_numlines += cur_numlines;
    (*chunksize_ptr(buf).add(prev)).mlcs_totalsize += cur_totalsize;
    let new_used = usedchunks - 1;
    (*bs).ml_usedchunks = new_used;
    if curix < new_used {
        std::ptr::copy(
            chunksize_ptr(buf).add((curix + 1) as usize),
            chunksize_ptr(buf).add(curix as usize),
            (new_used - curix) as usize,
        );
    }
}
