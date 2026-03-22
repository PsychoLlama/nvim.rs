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

use std::ffi::{c_char, c_int};

use crate::types::{
    BlockNr, BufHandle, ColNr, DataBlockHeader, InfoPtrHandle, LineNr, PointerBlockHeader,
    PointerEntry, PosHandle, DATA_ID, DB_INDEX_MASK, ML_DELETE, ML_FIND, ML_FLUSH, ML_INSERT,
    ML_LOCKED_DIRTY, ML_LOCKED_POS, PTR_ID, STACK_INCR,
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

    /// Get buffer's cached line number (`buf->b_ml.ml_line_lnum`)
    fn nvim_buf_get_ml_line_lnum(buf: *mut BufHandle) -> LineNr;

    /// Get byte offset cache value (`buf->b_ml.ml_line_offset`)
    fn nvim_buf_get_ml_line_offset(buf: *mut BufHandle) -> usize;

    /// Set byte offset cache value (`buf->b_ml.ml_line_offset = offset`)
    fn nvim_buf_set_ml_line_offset(buf: *mut BufHandle, offset: usize);

    /// Check if memfile pointer is non-null
    fn nvim_buf_has_ml_mfp(buf: *mut BufHandle) -> c_int;

    /// Get usedchunks count
    fn nvim_buf_get_ml_usedchunks(buf: *mut BufHandle) -> c_int;

    /// Get ml_locked_high (last line number in locked block)
    fn nvim_buf_get_ml_locked_high(buf: *mut BufHandle) -> LineNr;

    /// Get ml_locked_low (first line number in locked block)
    fn nvim_buf_get_ml_locked_low(buf: *mut BufHandle) -> LineNr;

    /// Get ml_chunksize[idx].mlcs_numlines
    fn nvim_buf_get_ml_chunksize_numlines(buf: *mut BufHandle, idx: c_int) -> c_int;

    /// Get ml_chunksize[idx].mlcs_totalsize
    fn nvim_buf_get_ml_chunksize_totalsize(buf: *mut BufHandle, idx: c_int) -> c_int;

    /// Check if ml_chunksize is NULL
    fn nvim_buf_get_ml_chunksize_is_null(buf: *mut BufHandle) -> c_int;

    /// Get bh_data pointer from block header
    fn nvim_bhdr_get_bh_data(hp: *mut std::ffi::c_void) -> *mut std::ffi::c_void;

    /// Get b_p_fixeol
    fn nvim_buf_get_b_p_fixeol(buf: *mut BufHandle) -> c_int;

    /// Get b_p_bin
    fn nvim_buf_get_b_p_bin(buf: *mut BufHandle) -> c_int;

    /// Get b_p_eol
    fn nvim_buf_get_b_p_eol(buf: *mut BufHandle) -> c_int;

    // -------------------------------------------------------------------------
    // Position Accessors
    // -------------------------------------------------------------------------

    /// Get line number from position (`pos->lnum`)
    fn nvim_pos_get_lnum(pos: *const PosHandle) -> LineNr;

    /// Get column from position (`pos->col`)
    fn nvim_pos_get_col(pos: *const PosHandle) -> ColNr;

    /// Set line number in position (`pos->lnum = lnum`)
    fn nvim_pos_set_lnum(pos: *mut PosHandle, lnum: LineNr);

    /// Set column in position (`pos->col = col`)
    fn nvim_pos_set_col(pos: *mut PosHandle, col: ColNr);

    /// Set coladd in position (`pos->coladd = coladd`)
    fn nvim_pos_set_coladd(pos: *mut PosHandle, coladd: ColNr);

    /// MAXCOL constant
    fn nvim_get_maxcol() -> ColNr;

    /// Get character at position (uses ml_get_pos internally)
    fn ml_get_pos(pos: *const PosHandle) -> *const c_char;

    /// Get contents of line lnum (pointer valid until next ml_get call)
    fn ml_get(lnum: LineNr) -> *mut c_char;

    /// Get length of line lnum
    fn ml_get_len(lnum: LineNr) -> ColNr;

    /// Get length of UTF-8 character sequence at p
    fn utfc_ptr2len(p: *const c_char) -> c_int;

    /// Get how many bytes the char at `q` is from a char start (0 if at start)
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;

    // -------------------------------------------------------------------------
    // B-tree Traversal / Cache Management
    // -------------------------------------------------------------------------

    /// Flush the current cached line to the data block (Rust implementation)
    #[link_name = "ml_flush_line"]
    fn rs_ml_flush_line(buf: *mut BufHandle, noalloc: c_int);

    // -------------------------------------------------------------------------
    // File format
    // -------------------------------------------------------------------------

    /// Get file format (EOL_UNIX=0, EOL_DOS=1, EOL_MAC=2) from Rust buffer crate
    fn rs_get_fileformat(buf: *mut BufHandle) -> c_int;

    // -------------------------------------------------------------------------
    // Cursor / Window Accessors (for rs_goto_byte)
    // -------------------------------------------------------------------------

    /// Set curwin->w_cursor.lnum
    fn nvim_curwin_set_cursor_lnum(lnum: LineNr);

    /// Set curwin->w_cursor.col
    fn nvim_curwin_set_cursor_col(col: ColNr);

    /// Set curwin->w_cursor.coladd
    fn nvim_set_curwin_cursor_coladd(val: ColNr);

    /// Set curwin->w_set_curswant
    fn nvim_curwin_set_w_set_curswant(val: bool);

    /// Set curwin->w_curswant
    fn nvim_edit_set_w_curswant(val: ColNr);

    /// coladvance(curwin, col) wrapper
    fn nvim_coladvance(col: ColNr);

    /// setpcmark()
    fn setpcmark();

    /// check_cursor(curwin)
    fn nvim_check_cursor();

    /// mb_adjust_cursor()
    fn nvim_mb_adjust_cursor();

    /// Get buf->deleted_bytes
    fn nvim_buf_get_deleted_bytes(buf: *mut BufHandle) -> usize;
    /// Set buf->deleted_bytes
    fn nvim_buf_set_deleted_bytes(buf: *mut BufHandle, val: usize);
    /// Get buf->deleted_codepoints
    fn nvim_buf_get_deleted_codepoints(buf: *mut BufHandle) -> usize;
    /// Set buf->deleted_codepoints
    fn nvim_buf_set_deleted_codepoints(buf: *mut BufHandle, val: usize);
    /// Get buf->deleted_codeunits
    fn nvim_buf_get_deleted_codeunits(buf: *mut BufHandle) -> usize;
    /// Set buf->deleted_codeunits
    fn nvim_buf_set_deleted_codeunits(buf: *mut BufHandle, val: usize);
}

extern "C" {
    // -------------------------------------------------------------------------
    // Phase 4: ml_lineadd accessors
    // -------------------------------------------------------------------------

    /// Get top of B-tree stack (buf->b_ml.ml_stack_top)
    fn nvim_buf_get_ml_stack_top(buf: *mut BufHandle) -> c_int;

    /// Get pointer to stack entry at idx (buf->b_ml.ml_stack[idx])
    fn nvim_buf_get_ml_stack_ip(buf: *mut BufHandle, idx: c_int) -> *mut InfoPtrHandle;

    /// Get ip_bnum from stack entry
    fn nvim_ip_get_bnum(ip: *const InfoPtrHandle) -> BlockNr;

    /// Get ip_index from stack entry
    fn nvim_ip_get_index(ip: *const InfoPtrHandle) -> c_int;

    /// Add count to ip_high in stack entry
    fn nvim_ip_add_high(ip: *mut InfoPtrHandle, count: c_int);

    /// Get id field from pointer block (first u16 field)
    fn nvim_pp_get_id(pp: *const std::ffi::c_void) -> u16;

    /// Add count to pb_pointer[idx].pe_line_count
    fn nvim_pp_pe_linecount_add(pp: *mut std::ffi::c_void, idx: c_int, count: c_int);

    /// Get memfile from buffer
    fn nvim_buf_get_ml_mfp(buf: *mut BufHandle) -> *mut std::ffi::c_void;

    /// Get block from memfile
    fn mf_get(mfp: *mut std::ffi::c_void, bnum: BlockNr, count: c_int) -> *mut std::ffi::c_void;

    /// Release block back to memfile
    fn mf_put(mfp: *mut std::ffi::c_void, hp: *mut std::ffi::c_void, dirty: bool, release: bool);

    /// Print "E317: Pointer block id wrong 2" error
    fn nvim_iemsg_pointer_block_id_wrong_two();
}

extern "C" {
    // -------------------------------------------------------------------------
    // Pass 5 Phase 1: ml_find_line accessors
    // -------------------------------------------------------------------------

    /// Get ml_locked block pointer (buf->b_ml.ml_locked as void*)
    fn nvim_buf_get_ml_locked(buf: *mut BufHandle) -> *mut std::ffi::c_void;
    /// Set ml_locked block pointer
    fn nvim_buf_set_ml_locked(buf: *mut BufHandle, hp: *mut std::ffi::c_void);

    /// Get ml_locked_lineadd (buf->b_ml.ml_locked_lineadd)
    fn nvim_buf_get_ml_locked_lineadd(buf: *mut BufHandle) -> c_int;
    /// Set ml_locked_lineadd
    fn nvim_buf_set_ml_locked_lineadd(buf: *mut BufHandle, val: c_int);

    /// Set ml_locked_low
    fn nvim_buf_set_ml_locked_low(buf: *mut BufHandle, val: LineNr);
    /// Set ml_locked_high
    fn nvim_buf_set_ml_locked_high(buf: *mut BufHandle, val: LineNr);

    /// Get ip_low from stack entry
    fn nvim_ip_get_low(ip: *const InfoPtrHandle) -> LineNr;
    /// Get ip_high from stack entry
    fn nvim_ip_get_high(ip: *const InfoPtrHandle) -> LineNr;
    /// Set ip_bnum in stack entry
    fn nvim_ip_set_bnum(ip: *mut InfoPtrHandle, bnum: BlockNr);
    /// Set ip_low in stack entry
    fn nvim_ip_set_low(ip: *mut InfoPtrHandle, lnum: LineNr);
    /// Set ip_high in stack entry
    fn nvim_ip_set_high(ip: *mut InfoPtrHandle, lnum: LineNr);
    /// Set ip_index in stack entry
    fn nvim_ip_set_index(ip: *mut InfoPtrHandle, idx: c_int);

    /// Set buf->b_ml.ml_stack_top
    fn nvim_buf_set_ml_stack_top(buf: *mut BufHandle, n: c_int);

    /// Translate a negative block number to a positive one
    fn mf_trans_del(mfp: *mut std::ffi::c_void, bnum: BlockNr) -> BlockNr;

    /// Get pb_count from pointer block
    fn nvim_pp_get_count(pp: *const std::ffi::c_void) -> u16;
    /// Get pe_bnum from pointer block entry at idx
    fn nvim_pp_pe_get_bnum(pp: *const std::ffi::c_void, idx: c_int) -> BlockNr;
    /// Get pe_line_count from pointer block entry at idx
    fn nvim_pp_pe_get_line_count(pp: *const std::ffi::c_void, idx: c_int) -> LineNr;
    /// Get pe_page_count from pointer block entry at idx
    fn nvim_pp_pe_get_page_count(pp: *const std::ffi::c_void, idx: c_int) -> c_int;
    /// Set pe_bnum in pointer block entry at idx
    fn nvim_pp_pe_set_bnum(pp: *mut std::ffi::c_void, idx: c_int, bnum: BlockNr);
    /// Decrement pe_line_count in pointer block entry at idx
    fn nvim_pp_pe_dec_line_count(pp: *mut std::ffi::c_void, idx: c_int);
    /// Increment pe_line_count in pointer block entry at idx
    fn nvim_pp_pe_inc_line_count(pp: *mut std::ffi::c_void, idx: c_int);

    /// Get db_id from data block (to check if it's a data or pointer block)
    fn nvim_dp_get_id(dp: *const std::ffi::c_void) -> u16;

    /// Get buffer's ml_flags
    fn nvim_buf_get_ml_flags(buf: *mut BufHandle) -> c_int;
    /// Set buffer's ml_flags
    fn nvim_buf_set_ml_flags(buf: *mut BufHandle, flags: c_int);

    /// Print "E317: Pointer block id wrong" error (base message)
    fn nvim_iemsg_pointer_block_id_wrong();
    /// Print "E322: Line number out of range" error
    fn nvim_siemsg_line_number_out_of_range(lnum_past: i64);
    /// Print "E323: Line count wrong in block" error
    fn nvim_siemsg_line_count_wrong_in_block(bnum: i64);

    /// Add an entry to the info pointer stack, returns index of new entry (in modify.rs)
    fn rs_ml_add_stack(buf: *mut BufHandle) -> c_int;
}

// EOL_DOS constant (matches buffer crate definition)
const EOL_DOS: c_int = 1;

const MAXCOL: ColNr = 0x7fff_ffff;

// =============================================================================
// B-tree Traversal: ml_find_line
// =============================================================================

/// Look up line `lnum` in the memline B-tree.
///
/// This is the core B-tree traversal function. It locates the data block
/// containing `lnum`, locks it in `ml_locked`, and updates the traversal stack.
///
/// # Arguments
/// * `buf` - Buffer to query
/// * `lnum` - Line number to find (1-based). Pass 0 with `ML_FLUSH` to flush only.
/// * `action` - One of `ML_FIND`, `ML_INSERT`, `ML_DELETE`, or `ML_FLUSH`.
///
/// For `ML_INSERT` and `ML_DELETE`, the pointer block line counts are updated
/// as part of the traversal.
///
/// # Returns
/// Pointer to the locked block header (`bhdr_T*` as `void*`), or NULL on error.
///
/// # Safety
/// - `buf` must be a valid buffer pointer (non-null).
/// - `lnum` must be a valid line number for the action, or 0 for ML_FLUSH.
/// - Only call from the main Neovim thread.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]
pub unsafe extern "C" fn rs_ml_find_line(
    buf: *mut BufHandle,
    lnum: LineNr,
    action: c_int,
) -> *mut std::ffi::c_void {
    let mfp = nvim_buf_get_ml_mfp(buf);

    // If there is a locked block check if the wanted line is in it.
    // If not, flush and release it.
    // Don't do this for ML_FLUSH (we want to flush the locked block).
    let locked = nvim_buf_get_ml_locked(buf);
    if !locked.is_null() {
        let locked_low = nvim_buf_get_ml_locked_low(buf);
        let locked_high = nvim_buf_get_ml_locked_high(buf);
        // ML_SIMPLE(action) is: (action & 0x10) != 0
        let is_simple = (action & 0x10) != 0;

        if is_simple && locked_low <= lnum && locked_high >= lnum {
            // Line is in the cached block - just update lineadd/high if needed
            if action == ML_INSERT {
                let lineadd = nvim_buf_get_ml_locked_lineadd(buf);
                nvim_buf_set_ml_locked_lineadd(buf, lineadd + 1);
                nvim_buf_set_ml_locked_high(buf, locked_high + 1);
            } else if action == ML_DELETE {
                let lineadd = nvim_buf_get_ml_locked_lineadd(buf);
                nvim_buf_set_ml_locked_lineadd(buf, lineadd - 1);
                nvim_buf_set_ml_locked_high(buf, locked_high - 1);
            }
            return locked;
        }

        // Release the locked block
        let flags = nvim_buf_get_ml_flags(buf);
        mf_put(
            mfp,
            locked,
            (flags & ML_LOCKED_DIRTY) != 0,
            (flags & ML_LOCKED_POS) != 0,
        );
        nvim_buf_set_ml_locked(buf, std::ptr::null_mut());

        // If lines were added/deleted in the locked block, update pointer blocks
        let lineadd = nvim_buf_get_ml_locked_lineadd(buf);
        if lineadd != 0 {
            rs_ml_lineadd(buf, lineadd);
        }
    }

    if action == ML_FLUSH {
        // Nothing else to do for flush action
        return std::ptr::null_mut();
    }

    // Start traversal from root (block 1)
    let mut bnum: BlockNr = 1;
    let mut page_count: c_int = 1;
    let mut low: LineNr = 1;
    let mut high: LineNr = nvim_buf_get_ml_line_count(buf);

    if action == ML_FIND {
        // Try to find a shortcut via the stack entries
        let stack_top = nvim_buf_get_ml_stack_top(buf);
        let mut found_top = -1_i32;
        let mut top = stack_top - 1;
        while top >= 0 {
            let ip = nvim_buf_get_ml_stack_ip(buf, top);
            let ip_low = nvim_ip_get_low(ip);
            let ip_high = nvim_ip_get_high(ip);
            if ip_low <= lnum && ip_high >= lnum {
                bnum = nvim_ip_get_bnum(ip);
                low = ip_low;
                high = ip_high;
                found_top = top;
                break;
            }
            top -= 1;
        }
        if found_top >= 0 {
            // Truncate stack at the matching entry
            nvim_buf_set_ml_stack_top(buf, found_top);
        } else {
            // Not found in stack: start from root
            nvim_buf_set_ml_stack_top(buf, 0);
        }
    } else {
        // ML_DELETE or ML_INSERT: always start from root
        nvim_buf_set_ml_stack_top(buf, 0);
    }

    // Search downwards in the tree until we find a data block
    loop {
        let hp = mf_get(mfp, bnum, page_count);
        if hp.is_null() {
            // error_noblock: no mf_put needed (block not obtained)
            // Reverse line count changes from prior pointer block updates
            if action == ML_DELETE {
                rs_ml_lineadd(buf, 1);
            } else if action == ML_INSERT {
                rs_ml_lineadd(buf, -1);
            }
            nvim_buf_set_ml_stack_top(buf, 0);
            return std::ptr::null_mut();
        }

        // Update high for insert/delete (we're one level deeper in tree)
        if action == ML_INSERT {
            high += 1;
        } else if action == ML_DELETE {
            high -= 1;
        }

        let dp = nvim_bhdr_get_bh_data(hp);

        if nvim_dp_get_id(dp) == DATA_ID {
            // Found the data block - lock it
            nvim_buf_set_ml_locked(buf, hp);
            nvim_buf_set_ml_locked_low(buf, low);
            nvim_buf_set_ml_locked_high(buf, high);
            nvim_buf_set_ml_locked_lineadd(buf, 0);
            let flags = nvim_buf_get_ml_flags(buf);
            nvim_buf_set_ml_flags(buf, flags & !(ML_LOCKED_DIRTY | ML_LOCKED_POS));
            return hp;
        }

        // Must be a pointer block
        if nvim_pp_get_id(dp) != PTR_ID {
            nvim_iemsg_pointer_block_id_wrong();
            // error_block: release block without marking dirty
            mf_put(mfp, hp, false, false);
            // error_noblock continuation
            if action == ML_DELETE {
                rs_ml_lineadd(buf, 1);
            } else if action == ML_INSERT {
                rs_ml_lineadd(buf, -1);
            }
            nvim_buf_set_ml_stack_top(buf, 0);
            return std::ptr::null_mut();
        }

        // Add new entry to the traversal stack
        let top = rs_ml_add_stack(buf);
        let ip = nvim_buf_get_ml_stack_ip(buf, top);
        nvim_ip_set_bnum(ip, bnum);
        nvim_ip_set_low(ip, low);
        nvim_ip_set_high(ip, high);
        nvim_ip_set_index(ip, -1); // index not known yet

        let mut dirty = false;
        let count = c_int::from(nvim_pp_get_count(dp));
        let mut idx = 0_i32;
        let mut found = false;

        while idx < count {
            let t = nvim_pp_pe_get_line_count(dp, idx);
            // CHECK(t == 0, "pe_line_count is zero") -- debug only, omitted
            low += t;
            if low > lnum {
                nvim_ip_set_index(ip, idx);
                bnum = nvim_pp_pe_get_bnum(dp, idx);
                page_count = nvim_pp_pe_get_page_count(dp, idx);
                high = low - 1;
                low -= t;

                // A negative block number may have been changed by recovery
                if bnum < 0 {
                    let bnum2 = mf_trans_del(mfp, bnum);
                    if bnum != bnum2 {
                        bnum = bnum2;
                        nvim_pp_pe_set_bnum(dp, idx, bnum);
                        dirty = true;
                    }
                }

                found = true;
                break;
            }
            idx += 1;
        }

        if !found {
            // Past the end: something is wrong with the tree
            let line_count = nvim_buf_get_ml_line_count(buf);
            if lnum > line_count {
                nvim_siemsg_line_number_out_of_range(lnum - line_count);
            } else {
                nvim_siemsg_line_count_wrong_in_block(bnum);
            }
            // error_block
            mf_put(mfp, hp, false, false);
            // error_noblock continuation
            if action == ML_DELETE {
                rs_ml_lineadd(buf, 1);
            } else if action == ML_INSERT {
                rs_ml_lineadd(buf, -1);
            }
            nvim_buf_set_ml_stack_top(buf, 0);
            return std::ptr::null_mut();
        }

        // Update pointer entry line count for insert/delete
        if action == ML_DELETE {
            nvim_pp_pe_dec_line_count(dp, idx);
            dirty = true;
        } else if action == ML_INSERT {
            nvim_pp_pe_inc_line_count(dp, idx);
            dirty = true;
        }

        mf_put(mfp, hp, dirty, false);
        // Continue loop to descend to the next level
    }
}

// =============================================================================
// Position Increment/Decrement Functions
// =============================================================================

/// Native Rust implementation of `inc`: increment position across line boundaries.
///
/// Returns:
/// - `1` when moving to the next line
/// - `2` when moving forward onto a NUL at end of line
/// - `-1` when at end of file
/// - `0` otherwise
#[allow(clippy::cast_sign_loss)]
unsafe fn inc_native(lp: *mut PosHandle) -> c_int {
    let maxcol = nvim_get_maxcol();
    let col = nvim_pos_get_col(lp);

    // When searching, position may be set to end of a line (MAXCOL)
    if col != maxcol {
        let p = ml_get_pos(lp);
        if *p != 0 {
            // Still within line: move to next char (may be NUL)
            let l = utfc_ptr2len(p);
            nvim_pos_set_col(lp, col + l);
            return if *p.add(l as usize) != 0 { 0 } else { 2 };
        }
    }

    let lnum = nvim_pos_get_lnum(lp);
    let line_count = nvim_buf_get_ml_line_count(nvim_get_curbuf());
    if lnum != line_count {
        // There is a next line
        nvim_pos_set_col(lp, 0);
        nvim_pos_set_lnum(lp, lnum + 1);
        nvim_pos_set_coladd(lp, 0);
        return 1;
    }
    -1
}

/// Native Rust implementation of `dec`: decrement position across line boundaries.
///
/// Returns:
/// - `1` when moving to the previous line
/// - `-1` when at start of file
/// - `0` otherwise
#[allow(clippy::cast_sign_loss)]
unsafe fn dec_native(lp: *mut PosHandle) -> c_int {
    nvim_pos_set_coladd(lp, 0);

    let maxcol = nvim_get_maxcol();
    let lnum = nvim_pos_get_lnum(lp);
    let col = nvim_pos_get_col(lp);

    if col == maxcol {
        // Past end of line: move to actual end
        let p = ml_get(lnum);
        let len = ml_get_len(lnum);
        let head = utf_head_off(p, p.add(len as usize));
        nvim_pos_set_col(lp, len - head);
        return 0;
    }

    if col > 0 {
        // Still within line
        let p = ml_get(lnum);
        let head = utf_head_off(p, p.add(col as usize - 1));
        nvim_pos_set_col(lp, col - 1 - head);
        return 0;
    }

    if lnum > 1 {
        // There is a prior line
        nvim_pos_set_lnum(lp, lnum - 1);
        let p = ml_get(lnum - 1);
        let len = ml_get_len(lnum - 1);
        let head = utf_head_off(p, p.add(len as usize));
        nvim_pos_set_col(lp, len - head);
        return 1;
    }

    // At start of file
    -1
}

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
#[export_name = "inc"]
pub unsafe extern "C" fn rs_inc(lp: *mut PosHandle) -> c_int {
    if lp.is_null() {
        return -1;
    }
    inc_native(lp)
}

/// Like `rs_inc()`, but skip NUL at the end of non-empty lines.
///
/// # Safety
/// - `lp` must be a valid position pointer or NULL
#[export_name = "incl"]
pub unsafe extern "C" fn rs_incl(lp: *mut PosHandle) -> c_int {
    if lp.is_null() {
        return -1;
    }
    let r = inc_native(lp);
    if r >= 1 && nvim_pos_get_col(lp) != 0 {
        inc_native(lp)
    } else {
        r
    }
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
#[export_name = "dec"]
pub unsafe extern "C" fn rs_dec(lp: *mut PosHandle) -> c_int {
    if lp.is_null() {
        return -1;
    }
    dec_native(lp)
}

/// Like `rs_dec()`, but skip NUL at the end of non-empty lines.
///
/// # Safety
/// - `lp` must be a valid position pointer or NULL
#[export_name = "decl"]
pub unsafe extern "C" fn rs_decl(lp: *mut PosHandle) -> c_int {
    if lp.is_null() {
        return -1;
    }
    let r = dec_native(lp);
    if r == 1 && nvim_pos_get_col(lp) != 0 {
        dec_native(lp)
    } else {
        r
    }
}

// =============================================================================
// Byte Offset Functions
// =============================================================================

/// Find the byte offset of a line, or find the line at a given byte offset.
///
/// This is the core byte-tracking function. It uses the chunk cache for O(log n)
/// performance by skipping large chunks before doing per-block byte counting.
///
/// # Arguments
/// * `buf` - Buffer to query
/// * `lnum` - If > 0: find byte offset of this line (1-based). If == 0: find line
///   containing byte offset `*offp`.
/// * `offp` - If `lnum == 0`: input byte offset, output: column within found line.
///   If `lnum > 0`: should be NULL.
/// * `no_ff` - If non-zero: ignore 'fileformat', always count 1 byte per newline.
///
/// # Returns
/// * When `lnum > 0`: byte offset of start of `lnum` (or -1 if unavailable)
/// * When `lnum == 0`: line number containing offset (or -1 if past end)
///
/// # Safety
/// - `buf` must be a valid buffer pointer (not NULL)
/// - `offp` must be a valid pointer or NULL
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_ptr_alignment,
    clippy::too_many_lines
)]
pub unsafe extern "C" fn rs_ml_find_line_or_offset(
    buf: *mut BufHandle,
    lnum: LineNr,
    offp: *mut c_int,
    no_ff: c_int,
) -> c_int {
    let ffdos = (no_ff == 0) && (rs_get_fileformat(buf) == EOL_DOS);
    let ffdos_int = c_int::from(ffdos);

    // Take care of cached line first. Only needed if the cached line is before
    // the requested line. Additionally cache the value for the cached line.
    // This is used by the extmark code which needs the byte offset of the edited
    // line. So when doing multiple small edits on the same line the value is
    // only calculated once.
    //
    // NB: caching doesn't work with 'fileformat'. This is not a problem for
    // bytetracking, as bytetracking ignores 'fileformat' option. But calling
    // line2byte() will invalidate the cache for the time being.
    let can_cache = lnum != 0 && !ffdos && nvim_buf_get_ml_line_lnum(buf) == lnum;

    let curbuf = nvim_get_curbuf();
    if lnum == 0 || nvim_buf_get_ml_line_lnum(buf) < lnum || no_ff == 0 {
        rs_ml_flush_line(curbuf, 0);
    } else if can_cache && nvim_buf_get_ml_line_offset(buf) > 0 {
        return nvim_buf_get_ml_line_offset(buf) as c_int;
    }

    if nvim_buf_get_ml_usedchunks(buf) == -1
        || nvim_buf_get_ml_chunksize_is_null(buf) != 0
        || lnum < 0
    {
        // memline is currently empty. Although if it is loaded,
        // it behaves like there is one empty line.
        if no_ff != 0 && nvim_buf_has_ml_mfp(buf) != 0 && (lnum == 1 || lnum == 2) {
            return (lnum - 1) as c_int;
        }
        return -1;
    }

    let offset: c_int = if offp.is_null() { 0 } else { *offp };

    if lnum == 0 && offset <= 0 {
        return 1; // Not a "find offset" and offset 0 _must_ be in line 1
    }

    // Find the last chunk before the one containing our line. Last chunk is
    // special because it will never qualify.
    let mut curline: LineNr = 1;
    let mut curix: c_int = 0;
    let mut size: c_int = 0;
    let used_chunks = nvim_buf_get_ml_usedchunks(buf);

    while curix < used_chunks - 1 {
        let chunk_numlines = nvim_buf_get_ml_chunksize_numlines(buf, curix);
        let chunk_totalsize = nvim_buf_get_ml_chunksize_totalsize(buf, curix);

        let lnum_skip = lnum != 0 && lnum >= curline + LineNr::from(chunk_numlines);
        let offset_skip =
            offset != 0 && offset > size + chunk_totalsize + ffdos_int * chunk_numlines;

        if !lnum_skip && !offset_skip {
            break;
        }

        curline += LineNr::from(chunk_numlines);
        size += chunk_totalsize;
        if offset != 0 && ffdos {
            size += chunk_numlines;
        }
        curix += 1;
    }

    // Walk through data blocks within the identified chunk
    let line_count = nvim_buf_get_ml_line_count(buf);

    loop {
        if lnum != 0 && curline >= lnum {
            break;
        }
        if offset != 0 && size >= offset {
            break;
        }
        if lnum == 0 && offset == 0 {
            break;
        }

        if curline > line_count {
            return -1;
        }

        let hp = rs_ml_find_line(buf, curline, ML_FIND);
        if hp.is_null() {
            return -1;
        }

        let dp_raw = nvim_bhdr_get_bh_data(hp);
        let dp = dp_raw.cast::<DataBlockHeader>();

        let locked_high = nvim_buf_get_ml_locked_high(buf);
        let locked_low = nvim_buf_get_ml_locked_low(buf);
        let count = (locked_high - locked_low + 1) as c_int; // entries in block
        let start_idx = (curline - locked_low) as c_int;
        let mut idx = start_idx;

        // db_index array starts immediately after the DataBlockHeader
        let db_index: *const u32 = dp.add(1).cast();

        let text_end: c_int = if idx == 0 {
            // first line in block, text is at the end
            (*dp).db_txt_end as c_int
        } else {
            (*db_index.add((idx - 1) as usize) & DB_INDEX_MASK) as c_int
        };

        // Compute index of last line to use in this block
        if lnum != 0 {
            if curline + LineNr::from(count - idx) >= lnum {
                idx += (lnum - curline - 1) as c_int;
            } else {
                idx = count - 1;
            }
        } else {
            // byte-search mode: walk forward through lines in block
            let mut extra: c_int = 0;
            loop {
                let line_start = ((*db_index.add(idx as usize)) & DB_INDEX_MASK) as c_int;
                if offset < size + text_end - line_start + ffdos_int {
                    break;
                }
                if ffdos {
                    size += 1;
                }
                if idx == count - 1 {
                    extra = 1;
                    break;
                }
                idx += 1;
            }

            let line_start_idx = ((*db_index.add(idx as usize)) & DB_INDEX_MASK) as c_int;
            let len = text_end - line_start_idx;
            size += len;
            if size >= offset {
                if !offp.is_null() {
                    if size + ffdos_int == offset {
                        *offp = 0;
                    } else if idx == start_idx {
                        *offp = offset - size + len;
                    } else {
                        let prev_start =
                            ((*db_index.add((idx - 1) as usize)) & DB_INDEX_MASK) as c_int;
                        *offp = offset - size + len - (text_end - prev_start);
                    }
                }
                let result_lnum = curline + LineNr::from(idx - start_idx + extra);
                if result_lnum > line_count {
                    return -1; // exactly one byte beyond the end
                }
                return result_lnum as c_int;
            }
            curline = locked_high + 1;
            continue;
        }

        // line-search mode: accumulate size for lines in block
        let line_start = ((*db_index.add(idx as usize)) & DB_INDEX_MASK) as c_int;
        let len = text_end - line_start;
        size += len;
        curline = locked_high + 1;
    }

    if lnum != 0 {
        // Count extra CR characters for DOS format.
        if ffdos {
            size += (lnum - 1) as c_int;
        }

        // Don't count the last line break if 'noeol' and ('bin' or 'nofixeol').
        if (nvim_buf_get_b_p_fixeol(buf) == 0 || nvim_buf_get_b_p_bin(buf) != 0)
            && nvim_buf_get_b_p_eol(buf) == 0
            && lnum > line_count
        {
            size -= ffdos_int + 1;
        }
    }

    if can_cache && size > 0 {
        nvim_buf_set_ml_line_offset(buf, size as usize);
    }

    size
}

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
    rs_ml_find_line_or_offset(buf, lnum, std::ptr::null_mut(), 1)
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
    rs_ml_find_line_or_offset(buf, lnum, std::ptr::null_mut(), 0)
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
    let result = rs_ml_find_line_or_offset(buf, 0, std::ptr::addr_of_mut!(offset), 1);

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
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_goto_byte(cnt: c_int) {
    let mut boff = cnt;

    let curbuf = nvim_get_curbuf();
    rs_ml_flush_line(curbuf, 0); // cached line may be dirty
    setpcmark();
    if boff != 0 {
        boff -= 1;
    }
    let lnum = LineNr::from(rs_ml_find_line_or_offset(curbuf, 0, &raw mut boff, 0));
    let maxcol = MAXCOL;
    if lnum < 1 {
        // past the end
        let line_count = nvim_buf_get_ml_line_count(curbuf);
        nvim_curwin_set_cursor_lnum(line_count);
        nvim_edit_set_w_curswant(maxcol);
        nvim_coladvance(maxcol);
    } else {
        nvim_curwin_set_cursor_lnum(lnum);
        nvim_curwin_set_cursor_col(boff as ColNr);
        nvim_set_curwin_cursor_coladd(0);
        nvim_curwin_set_w_set_curswant(true);
    }
    nvim_check_cursor();
    // Make sure the cursor is on the first byte of a multi-byte char.
    nvim_mb_adjust_cursor();
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
#[allow(clippy::must_use_candidate)]
#[export_name = "ml_flush_deleted_bytes"]
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

    let ret = nvim_buf_get_deleted_bytes(buf);
    if !codepoints.is_null() {
        *codepoints = nvim_buf_get_deleted_codepoints(buf);
    }
    if !codeunits.is_null() {
        *codeunits = nvim_buf_get_deleted_codeunits(buf);
    }
    nvim_buf_set_deleted_bytes(buf, 0);
    nvim_buf_set_deleted_codepoints(buf, 0);
    nvim_buf_set_deleted_codeunits(buf, 0);
    ret
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
        let line_count = LineNr::from(entry.pe_line_count);

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

// =============================================================================
// Line Count Update in Parent Pointer Blocks
// =============================================================================

/// Update the line count in all parent pointer blocks.
///
/// Walks the B-tree stack from top to bottom and adds `count` to
/// `pb_pointer[idx].pe_line_count` and `ip_high` for each level.
///
/// # Safety
/// - `buf` must be a valid buffer pointer
#[no_mangle]
pub unsafe extern "C" fn rs_ml_lineadd(buf: *mut BufHandle, count: c_int) {
    let mfp = nvim_buf_get_ml_mfp(buf);
    let stack_top = nvim_buf_get_ml_stack_top(buf);

    for idx in (0..stack_top).rev() {
        let ip = nvim_buf_get_ml_stack_ip(buf, idx);
        let bnum = nvim_ip_get_bnum(ip);
        let hp = mf_get(mfp, bnum, 1);
        if hp.is_null() {
            break;
        }
        let pp = nvim_bhdr_get_bh_data(hp);
        if nvim_pp_get_id(pp) != crate::types::PTR_ID {
            mf_put(mfp, hp, false, false);
            nvim_iemsg_pointer_block_id_wrong_two();
            break;
        }
        let ip_index = nvim_ip_get_index(ip);
        nvim_pp_pe_linecount_add(pp, ip_index, count);
        nvim_ip_add_high(ip, count);
        mf_put(mfp, hp, true, false);
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
        assert_eq!(
            rs_ml_stack_grow_size(0),
            c_int::try_from(STACK_INCR).unwrap()
        );
        assert_eq!(
            rs_ml_stack_grow_size(5),
            5 + c_int::try_from(STACK_INCR).unwrap()
        );
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
