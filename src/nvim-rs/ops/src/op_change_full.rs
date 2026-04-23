//! Full `op_change` migration (Phase 2)
//!
//! Migrated from `op_change()` in ops.c.
//! Handles the c/C change operator (visual block change).

#![allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::too_many_lines,
    clippy::similar_names
)]

use nvim_normal::types::OpargT;
use std::ffi::{c_char, c_int, c_void};

// -----------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------

const FAIL: c_int = 0;
const NUL: c_int = 0;
const K_MT_BLOCK_WISE: c_int = 2; // kMTBlockWise
const K_MT_LINE_WISE: c_int = 1; // kMTLineWise
const K_EXTMARK_UNDO: c_int = 1; // kExtmarkUndo

// -----------------------------------------------------------------------
// FFI declarations
// -----------------------------------------------------------------------

#[allow(clashing_extern_declarations)]
extern "C" {
    // Undo
    fn u_save_cursor() -> c_int;

    // Block preparation
    fn block_prep(oap: *mut c_void, bdp: *mut c_void, lnum: c_int, is_del: bool);

    // Line content
    fn ml_get(lnum: c_int) -> *mut c_char;
    fn ml_get_len(lnum: c_int) -> c_int;
    fn ml_replace(lnum: c_int, line: *mut c_char, copy: bool) -> c_int;

    // Memory (C allocator)
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn xmemcpyz(dst: *mut c_void, src: *const c_void, len: usize);

    // Extmarks
    fn extmark_splice_cols(
        buf: *mut c_void,
        lnum: c_int,
        col: c_int,
        old_col: c_int,
        new_col: c_int,
        op: c_int,
    );

    // Cursor movement
    fn check_cursor(wp: *mut c_void);
    fn inc_cursor() -> c_int;
    fn coladvance_force(wcol: c_int);
    fn getviscol() -> c_int;
    fn gchar_cursor() -> c_int;

    // Edit (enters insert mode)
    fn edit(cmdchar: c_int, startln: bool, count: c_int) -> c_int;

    // Indent helpers
    fn fix_indent();
    fn getwhitecols(line: *const c_char) -> usize;

    // Virtual position
    fn getvpos(wp: *mut c_void, pos: *mut c_void, wcol: c_int) -> c_int;

    // Auto format
    fn auto_format(trailblank: bool, prev_line: bool);

    // Change notification
    fn changed_lines(
        buf: *mut c_void,
        lnum: c_int,
        col: c_int,
        lnum_end: c_int,
        added: c_int,
        do_buf_event: bool,
    );

    // Globals
    static mut curbuf: *mut c_void;
    static mut curwin: *mut c_void;
    static mut got_int: bool;

    // Cursor accessors
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_get_cursor_coladd() -> c_int;

    // finish_op (direct static)
    static mut finish_op: bool;

    // virtual_op accessor
    static mut virtual_op: c_int;

    // can_si accessor
    fn nvim_set_can_si(val: bool);

    // may_do_si
    fn may_do_si() -> bool;

    // Buffer empty check
    fn nvim_curbuf_ml_empty() -> bool;

    // curwin pointer for check_cursor
    fn nvim_dpo_get_curwin() -> *mut c_void;
}

// -----------------------------------------------------------------------
// BlockDef mirror (same as in op_insert_full.rs / tilde_full.rs)
// -----------------------------------------------------------------------

#[repr(C)]
struct BlockDefC {
    startspaces: c_int,
    endspaces: c_int,
    textlen: c_int,
    textstart: *mut c_char,
    textcol: c_int,
    start_vcol: c_int,
    end_vcol: c_int,
    is_short: c_int,
    is_max: c_int,
    is_onechar: c_int,
    pre_whitesp: c_int,
    pre_whitesp_c: c_int,
    end_char_vcols: c_int,
    start_char_vcols: c_int,
}

impl BlockDefC {
    fn zeroed() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

// pos_T mirror for getvpos
#[repr(C)]
struct PosC {
    lnum: c_int,
    col: c_int,
    coladd: c_int,
}

impl PosC {
    fn zeroed() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

// -----------------------------------------------------------------------
// op_change -- exported drop-in replacement
// -----------------------------------------------------------------------

/// Handle a change operation (c/C operator).
///
/// Returns: true if edit() returns because of a CTRL-O command.
///
/// # Safety
/// - `oap` must be a valid `oparg_T *`
/// - Accesses global state via C functions
#[unsafe(export_name = "op_change")]
pub unsafe extern "C" fn rs_op_change(oap: *mut c_void) -> c_int {
    let oap_t: *mut OpargT = oap.cast();

    let mut pre_textlen: c_int = 0;
    let mut pre_indent: c_int = 0;
    let mut bd = BlockDefC::zeroed();

    let l = (*oap_t).start.col;
    let l = if (*oap_t).motion_type == K_MT_LINE_WISE {
        nvim_set_can_si(may_do_si()); // Like opening a new line, do smart indent
        0
    } else {
        l
    };

    // First delete the text in the region. In an empty buffer only need to save for undo.
    if nvim_curbuf_ml_empty() {
        if u_save_cursor() == FAIL {
            return 0; // false
        }
    } else if crate::delete_full::rs_op_delete(oap) == FAIL {
        return 0; // false
    }

    // if l > cursor.col && !LINEEMPTY(lnum) && !virtual_op
    let cursor_col = nvim_get_cursor_col();
    let cursor_lnum = {
        // read via oap->start.lnum context -- just use the current lnum from ops
        // Actually we need the current cursor lnum after op_delete moved it
        // Use the existing nvim accessor
        extern "C" {
            fn nvim_get_cursor_lnum() -> c_int;
        }
        nvim_get_cursor_lnum()
    };
    if l > cursor_col && virtual_op == 0 {
        // LINEEMPTY = *ml_get(lnum) == NUL
        let line = ml_get(cursor_lnum);
        let lineempty = *line == 0 as c_char;
        if !lineempty {
            inc_cursor();
        }
    }

    // check for still on same line (skip blank lines too)
    if (*oap_t).motion_type == K_MT_BLOCK_WISE {
        // Add spaces before getting the current line length.
        if virtual_op != 0 && (nvim_get_cursor_coladd() > 0 || gchar_cursor() == NUL) {
            coladvance_force(getviscol());
        }
        let firstline = ml_get((*oap_t).start.lnum);
        pre_textlen = ml_get_len((*oap_t).start.lnum);
        pre_indent = getwhitecols(firstline) as c_int;
        bd.textcol = nvim_get_cursor_col();
    }

    if (*oap_t).motion_type == K_MT_LINE_WISE {
        fix_indent();
    }

    // Reset finish_op now, don't want it set inside edit().
    let save_finish_op = finish_op;
    finish_op = false;

    let retval = edit(NUL, false, 1);

    finish_op = save_finish_op;

    // In Visual block mode, handle copying the new text to all lines of the block.
    // Don't repeat the insert when Insert mode ended with CTRL-C.
    if (*oap_t).motion_type == K_MT_BLOCK_WISE
        && (*oap_t).start.lnum != (*oap_t).end.lnum
        && !got_int
    {
        // Auto-indenting may have changed the indent. If the cursor was past
        // the indent, exclude that indent change from the inserted text.
        let firstline = ml_get((*oap_t).start.lnum);
        if bd.textcol > pre_indent {
            let new_indent = getwhitecols(firstline) as c_int;
            pre_textlen += new_indent - pre_indent;
            bd.textcol += new_indent - pre_indent;
        }

        let ins_len = ml_get_len((*oap_t).start.lnum) - pre_textlen;
        if ins_len > 0 {
            // Subsequent calls to ml_get() flush the firstline data - take a
            // copy of the inserted text.
            let firstline = ml_get((*oap_t).start.lnum);
            let ins_text: *mut c_char = xmalloc(ins_len as usize + 1).cast();
            xmemcpyz(
                ins_text.cast::<c_void>(),
                firstline.add(bd.textcol as usize).cast::<c_void>(),
                ins_len as usize,
            );

            let mut linenr = (*oap_t).start.lnum + 1;
            while linenr <= (*oap_t).end.lnum {
                block_prep(oap, (&raw mut bd).cast::<c_void>(), linenr, true);
                if bd.is_short == 0 || virtual_op != 0 {
                    let mut vpos = PosC::zeroed();

                    // If the block starts in virtual space, count the
                    // initial coladd offset as part of "startspaces"
                    if bd.is_short != 0 {
                        vpos.lnum = linenr;
                        getvpos(
                            curwin,
                            (&raw mut vpos).cast::<c_void>(),
                            (*oap_t).start_vcol,
                        );
                    }
                    // (else vpos.coladd = 0, already zeroed)

                    let oldp = ml_get(linenr);
                    let oldp_len = ml_get_len(linenr) as usize;
                    let newp: *mut c_char =
                        xmalloc(oldp_len + vpos.coladd as usize + ins_len as usize + 1).cast();
                    // copy up to block start
                    std::ptr::copy_nonoverlapping(oldp, newp, bd.textcol as usize);
                    let mut newlen = bd.textcol as usize;
                    // add virtual spaces
                    std::ptr::write_bytes(newp.add(newlen), b' ', vpos.coladd as usize);
                    newlen += vpos.coladd as usize;
                    // copy inserted text
                    std::ptr::copy_nonoverlapping(ins_text, newp.add(newlen), ins_len as usize);
                    newlen += ins_len as usize;
                    // copy rest of old line
                    let src = oldp.add(bd.textcol as usize);
                    let src_len = std::ffi::CStr::from_ptr(src).to_bytes().len();
                    std::ptr::copy_nonoverlapping(src, newp.add(newlen), src_len + 1);
                    ml_replace(linenr, newp, false);
                    extmark_splice_cols(
                        curbuf,
                        linenr - 1,
                        bd.textcol,
                        0,
                        vpos.coladd + ins_len,
                        K_EXTMARK_UNDO,
                    );
                }
                linenr += 1;
            }
            check_cursor(nvim_dpo_get_curwin());
            changed_lines(
                curbuf,
                (*oap_t).start.lnum + 1,
                0,
                (*oap_t).end.lnum + 1,
                0,
                true,
            );
            xfree(ins_text.cast::<c_void>());
        }
    }
    auto_format(false, true);

    retval
}
