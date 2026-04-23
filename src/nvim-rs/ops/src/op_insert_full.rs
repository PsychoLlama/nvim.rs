//! Full `op_insert` migration (Phase 1)
//!
//! Migrated from `block_insert()` and `op_insert()` in ops.c.
//! Handles the visual-block I (insert) and A (append) operators.

#![allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::useless_let_if_seq
)]

use nvim_normal::types::{OpargT, PosT};
use nvim_window::win_struct::{win_mut, win_ref};
use std::ffi::{c_char, c_int, c_void};

// -----------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------

const OK: c_int = 1;
const FAIL: c_int = 0;
const NUL: c_int = 0;
const TAB: u8 = b'\t';
const K_MT_BLOCK_WISE: c_int = 2; // kMTBlockWise
const K_EXTMARK_UNDO: c_int = 1; // kExtmarkUndo
const UPD_INVERTED: c_int = 14; // from drawscreen.h
const OP_APPEND: c_int = 18; // OP_APPEND from ops.h
const OP_INSERT: c_int = 17; // OP_INSERT from ops.h
const MODE_INSERT: c_int = 0x10; // state_defs.h
const MAXCOL: c_int = i32::MAX; // MAXCOL
const K_OPT_VE_FLAG_ALL: u32 = 0x04; // kOptVeFlagAll

// -----------------------------------------------------------------------
// C `struct block_def` mirror (register_defs.h)
//
// On 64-bit Linux: 3xint(12) + pad(4) + ptr(8) + 10xint(40) = 64 bytes.
// -----------------------------------------------------------------------

/// Mirror of C `struct block_def`.
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
    is_max: c_int, // is_MAX in C
    is_onechar: c_int,
    pre_whitesp: c_int,
    pre_whitesp_c: c_int,
    end_char_vcols: c_int,
    start_char_vcols: c_int,
}

impl BlockDefC {
    fn zeroed() -> Self {
        // SAFETY: all-zero is valid for this POD C struct
        unsafe { std::mem::zeroed() }
    }
}

// -----------------------------------------------------------------------
// FFI declarations
// -----------------------------------------------------------------------

#[allow(clashing_extern_declarations)]
extern "C" {
    // Undo
    fn u_save_cursor() -> c_int;
    fn u_save(top: c_int, bot: c_int) -> c_int;

    // Block preparation (use *mut c_void to avoid clashing_extern_declarations)
    fn block_prep(oap: *mut c_void, bdp: *mut c_void, lnum: c_int, is_del: bool);

    // Line content
    fn ml_get(lnum: c_int) -> *mut c_char;
    fn ml_get_len(lnum: c_int) -> c_int;
    fn ml_replace(lnum: c_int, line: *mut c_char, copy: bool) -> c_int;

    // Memory (C allocator -- NOT Rust allocator)
    fn xmalloc(size: usize) -> *mut c_void;
    fn xmemdupz(src: *const c_void, len: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_void);

    // Multibyte
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;

    // Extmarks
    fn extmark_splice_cols(
        buf: *mut c_void,
        lnum: c_int,
        col: c_int,
        old_col: c_int,
        new_col: c_int,
        op: c_int,
    );

    // Screen/redraw
    fn redraw_curbuf_later(update_type: c_int);
    fn update_screen();

    // Cursor movement / adjustment
    fn check_cursor(wp: *mut c_void);
    fn inc_cursor() -> c_int;
    fn coladvance_force(wcol: c_int);
    fn getviscol() -> c_int;
    fn getviscol2(col: c_int, coladd: c_int) -> c_int;
    fn get_cursor_pos_ptr() -> *const c_char;
    fn ins_char(c: c_int);

    // Indent helpers
    fn getwhitecols_curline() -> c_int;
    fn get_indent() -> c_int;

    // Edit (enters insert mode)
    fn edit(cmdchar: c_int, startln: bool, count: c_int) -> c_int;

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
    static mut got_int: bool;

    // Cursor accessors (shims)
    fn nvim_get_cursor_lnum() -> c_int;
    fn nvim_set_cursor_lnum(lnum: c_int);
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_get_cursor_coladd() -> c_int;
    fn nvim_set_cursor_col(col: c_int);

    // curwin->w_curswant
    fn nvim_get_curswant() -> c_int;

    // curwin->w_set_curswant
    fn nvim_curwin_set_curswant(val: bool);

    // curbuf->b_op_start_orig accessors (ops_shim.c)
    fn nvim_curbuf_get_b_op_start_orig_lnum() -> c_int;
    fn nvim_curbuf_get_b_op_start_orig_col() -> c_int;
    fn nvim_curbuf_get_b_op_start_orig_coladd() -> c_int;
    fn nvim_oap_set_start_from_b_op_start_orig(oap: *mut c_void);

    // Set/get curbuf->b_op_end (ops_shim.c)
    fn nvim_curbuf_set_op_end_lnum_col(lnum: c_int, col: c_int);
    fn nvim_curwin_set_cursor_from_oap_end(oap: *mut c_void);

    // Insstart accessors (edit.c)
    fn nvim_get_Insstart_lnum() -> c_int;
    fn nvim_get_Insstart_col() -> c_int;

    // State global
    static mut State: c_int;
    fn nvim_set_State(val: c_int);

    // curwin global (direct linkage)
    static mut curwin: *mut c_void;
}

// -----------------------------------------------------------------------
// block_insert -- private helper (mirrors static block_insert() in ops.c)
// -----------------------------------------------------------------------

/// Insert string `s` (b_insert ? before : after) block.
/// Caller must prepare for undo.
///
/// # Safety
/// - `oap` must be a valid `oparg_T *`
/// - `s` must be valid for `slen` bytes
/// - `bdp` must be a valid `*mut BlockDefC` (passed in, re-populated each iteration)
unsafe fn block_insert(
    oap: *mut c_void,
    s: *const c_char,
    slen: usize,
    b_insert: bool,
    bdp: *mut BlockDefC,
) {
    let oap_t: *mut OpargT = oap.cast();
    let start_lnum = (*oap_t).start.lnum;
    let end_lnum = (*oap_t).end.lnum;

    let oldstate = State;
    nvim_set_State(MODE_INSERT); // don't want MODE_REPLACE for State

    let mut lnum = start_lnum + 1;
    while lnum <= end_lnum {
        block_prep(oap, bdp.cast::<c_void>(), lnum, true);
        if (*bdp).is_short != 0 && b_insert {
            lnum += 1;
            continue; // OP_INSERT, line ends before block start
        }

        let oldp: *mut c_char = ml_get(lnum);

        let ts_val: c_int;
        let spaces: c_int;
        let mut count: c_int;
        let mut offset: c_int;

        if b_insert {
            ts_val = (*bdp).start_char_vcols;
            spaces = (*bdp).startspaces;
            count = if spaces != 0 { ts_val - 1 } else { 0 }; // cutting a TAB
            offset = (*bdp).textcol;
        } else {
            // append
            ts_val = (*bdp).end_char_vcols;
            if (*bdp).is_short == 0 {
                // spaces = padding after block
                spaces = if (*bdp).endspaces != 0 {
                    ts_val - (*bdp).endspaces
                } else {
                    0
                };
                count = if spaces != 0 { ts_val - 1 } else { 0 }; // cutting a TAB
                offset = (*bdp).textcol + (*bdp).textlen - i32::from(spaces != 0);
            } else {
                // spaces = padding to block edge
                // if $ used, just append to EOL (ie spaces==0)
                spaces = if (*bdp).is_max == 0 {
                    ((*oap_t).end_vcol - (*bdp).end_vcol) + 1
                } else {
                    0
                };
                count = spaces;
                offset = (*bdp).textcol + (*bdp).textlen;
            }
        }

        // avoid copying part of a multi-byte character
        let spaces = if spaces > 0 {
            let base = oldp.cast_const();
            let p = oldp.cast_const().add(offset as usize);
            offset -= utf_head_off(base, p);
            spaces.max(0) // can happen when cursor was moved
        } else {
            0
        };

        assert!(count >= 0);

        // Calculate allocation size (matching C exactly):
        //   ml_get_len(lnum) + spaces + slen
        //   + (spaces > 0 && !bdp->is_short ? (ts_val - spaces) : 0)
        //   + count + 1
        let oldp_len = ml_get_len(lnum) as usize;
        let post_pad = if spaces > 0 && (*bdp).is_short == 0 {
            (ts_val - spaces) as usize
        } else {
            0
        };
        let alloc_size = oldp_len + spaces as usize + slen + post_pad + count as usize + 1;
        let newp: *mut c_char = xmalloc(alloc_size).cast();

        // copy up to shifted part
        std::ptr::copy_nonoverlapping(oldp, newp, offset as usize);
        // oldp_cur points to oldp + offset (the part we haven't copied yet)
        let mut oldp_cur = oldp.add(offset as usize);
        let startcol = offset;

        // insert pre-padding
        std::ptr::write_bytes(newp.add(offset as usize), b' ', spaces as usize);

        // copy the new text
        std::ptr::copy_nonoverlapping(s, newp.add(offset as usize + spaces as usize), slen);
        offset += slen as c_int;

        let mut skipped: c_int = 0;
        if spaces > 0 && (*bdp).is_short == 0 {
            if *oldp_cur.cast::<u8>() == TAB {
                // insert post-padding
                std::ptr::write_bytes(
                    newp.add(offset as usize + spaces as usize),
                    b' ',
                    (ts_val - spaces) as usize,
                );
                // We're splitting a TAB, don't copy it.
                oldp_cur = oldp_cur.add(1);
                // We allowed for that TAB, remember this now
                count += 1;
                skipped = 1;
            } else {
                // Not a TAB, no extra spaces
                count = spaces;
            }
        }

        if spaces > 0 {
            offset += count;
        }

        // STRCPY(newp + offset, oldp_cur)
        let src_len = std::ffi::CStr::from_ptr(oldp_cur).to_bytes().len();
        std::ptr::copy_nonoverlapping(oldp_cur, newp.add(offset as usize), src_len + 1);

        ml_replace(lnum, newp, false);
        extmark_splice_cols(
            curbuf,
            lnum - 1,
            startcol,
            skipped,
            offset - startcol,
            K_EXTMARK_UNDO,
        );

        if lnum == end_lnum {
            // Set "']" mark to the end of the block
            nvim_curbuf_set_op_end_lnum_col(end_lnum, offset);
        }

        lnum += 1;
    } // for all lnum

    nvim_set_State(oldstate);

    // Only call changed_lines if we actually modified additional lines
    if start_lnum < end_lnum {
        changed_lines(curbuf, start_lnum + 1, 0, end_lnum + 1, 0, true);
    }
}

// -----------------------------------------------------------------------
// op_insert -- exported drop-in replacement
// -----------------------------------------------------------------------

/// Insert and append operators for Visual mode.
///
/// # Safety
/// - `oap` must be a valid `oparg_T *`
/// - Accesses global state via C functions
#[unsafe(export_name = "op_insert")]
pub unsafe extern "C" fn rs_op_insert(oap: *mut c_void, count1: c_int) {
    let oap_t: *mut OpargT = oap.cast();

    let mut pre_textlen: c_int = 0;
    let mut ind_pre_col: c_int = 0;
    let mut ind_pre_vcol: c_int = 0;
    let mut bd = BlockDefC::zeroed();

    // edit() changes w_curswant -- record it for OP_APPEND
    bd.is_max = i32::from(nvim_get_curswant() == MAXCOL);

    // vis block is still marked. Get rid of it now.
    nvim_set_cursor_lnum((*oap_t).start.lnum);
    redraw_curbuf_later(UPD_INVERTED);
    update_screen();

    if (*oap_t).motion_type == K_MT_BLOCK_WISE {
        if nvim_get_cursor_coladd() > 0 {
            let win = nvim_window::WinHandle::from_ptr(curwin);
            let old_ve_flags = win_ref(win).w_ve_flags();

            if u_save_cursor() == FAIL {
                return;
            }
            win_mut(win).set_w_ve_flags(K_OPT_VE_FLAG_ALL);
            let target_col = if (*oap_t).op_type == OP_APPEND {
                (*oap_t).end_vcol + 1
            } else {
                getviscol()
            };
            coladvance_force(target_col);
            if (*oap_t).op_type == OP_APPEND {
                let col = nvim_get_cursor_col();
                nvim_set_cursor_col(col - 1);
            }
            win_mut(win).set_w_ve_flags(old_ve_flags);
        }
        // Get the info about the block before entering the text
        block_prep(
            oap,
            (&raw mut bd).cast::<c_void>(),
            (*oap_t).start.lnum,
            true,
        );
        // Get indent information
        ind_pre_col = getwhitecols_curline();
        ind_pre_vcol = get_indent();
        pre_textlen = ml_get_len((*oap_t).start.lnum) - bd.textcol;
        if (*oap_t).op_type == OP_APPEND {
            pre_textlen -= bd.textlen;
        }
    }

    if (*oap_t).op_type == OP_APPEND {
        if (*oap_t).motion_type == K_MT_BLOCK_WISE && nvim_get_cursor_coladd() == 0 {
            // Move the cursor to the character right of the block.
            nvim_curwin_set_curswant(true);
            loop {
                let p = get_cursor_pos_ptr();
                let at_eol = *p == 0 as c_char;
                let past_block = nvim_get_cursor_col() >= bd.textcol + bd.textlen;
                if at_eol || past_block {
                    break;
                }
                nvim_set_cursor_col(nvim_get_cursor_col() + 1);
            }
            if bd.is_short != 0 && bd.is_max == 0 {
                // First line was too short, make it longer and adjust "bd".
                if u_save_cursor() == FAIL {
                    return;
                }
                for _ in 0..bd.endspaces {
                    ins_char(b' ' as c_int);
                }
                bd.textlen += bd.endspaces;
            }
        } else {
            // curwin->w_cursor = oap->end; check_cursor_col(curwin);
            nvim_curwin_set_cursor_from_oap_end(oap);

            // Works just like an 'i'nsert on the next character.
            // if (!LINEEMPTY(lnum) && start_vcol != end_vcol)
            let lnum = nvim_get_cursor_lnum();
            let line = ml_get(lnum);
            let lineempty = *line == 0 as c_char;
            if !lineempty && (*oap_t).start_vcol != (*oap_t).end_vcol {
                inc_cursor();
            }
        }
    }

    // Record cursor position before entering insert mode
    let t1: PosT = (*oap_t).start;
    let start_insert = PosT {
        lnum: nvim_get_cursor_lnum(),
        col: nvim_get_cursor_col(),
        coladd: nvim_get_cursor_coladd(),
    };
    edit(NUL, false, count1);

    // When a tab was inserted, check if b_op_start_orig moved back
    let orig_lnum = nvim_curbuf_get_b_op_start_orig_lnum();
    if t1.lnum == orig_lnum {
        let orig_col = nvim_curbuf_get_b_op_start_orig_col();
        let orig_coladd = nvim_curbuf_get_b_op_start_orig_coladd();
        // lt(curbuf->b_op_start_orig, t1)
        if orig_col < t1.col || (orig_col == t1.col && orig_coladd < t1.coladd) {
            nvim_oap_set_start_from_b_op_start_orig(oap);
        }
    }

    // If user moved off this line, or insert ended with CTRL-C, do nothing.
    if nvim_get_cursor_lnum() != (*oap_t).start.lnum || got_int {
        return;
    }

    if (*oap_t).motion_type == K_MT_BLOCK_WISE {
        let mut ind_post_vcol: c_int = 0;
        let mut bd2 = BlockDefC::zeroed();
        let mut did_indent = false;

        // if indent kicked in, the firstline might have changed
        let ind_post_col = getwhitecols_curline();
        if (*oap_t).start.col > ind_pre_col && ind_post_col > ind_pre_col {
            bd.textcol += ind_post_col - ind_pre_col;
            ind_post_vcol = get_indent();
            bd.start_vcol += ind_post_vcol - ind_pre_vcol;
            did_indent = true;
        }

        // Adjust block for user cursor movement (but not indent changes)
        let orig_lnum = nvim_curbuf_get_b_op_start_orig_lnum();
        if (*oap_t).start.lnum == orig_lnum && bd.is_max == 0 && !did_indent {
            let orig_col = nvim_curbuf_get_b_op_start_orig_col();
            let orig_coladd = nvim_curbuf_get_b_op_start_orig_coladd();
            let t = getviscol2(orig_col, orig_coladd);

            if (*oap_t).op_type == OP_INSERT
                && (*oap_t).start.col + (*oap_t).start.coladd != orig_col + orig_coladd
            {
                (*oap_t).start.col = orig_col;
                pre_textlen -= t - (*oap_t).start_vcol;
                (*oap_t).start_vcol = t;
            } else if (*oap_t).op_type == OP_APPEND
                && (*oap_t).start.col + (*oap_t).start.coladd >= orig_col + orig_coladd
            {
                (*oap_t).start.col = orig_col;
                // reset pre_textlen to OP_INSERT value
                pre_textlen += bd.textlen;
                pre_textlen -= t - (*oap_t).start_vcol;
                (*oap_t).start_vcol = t;
                (*oap_t).op_type = OP_INSERT;
            }
        }

        // Correct for indent changes: adjust selection cols
        if did_indent && bd.textcol - ind_post_col > 0 {
            (*oap_t).start.col += ind_post_col - ind_pre_col;
            (*oap_t).start_vcol += ind_post_vcol - ind_pre_vcol;
            (*oap_t).end.col += ind_post_col - ind_pre_col;
            (*oap_t).end_vcol += ind_post_vcol - ind_pre_vcol;
        }
        block_prep(
            oap,
            (&raw mut bd2).cast::<c_void>(),
            (*oap_t).start.lnum,
            true,
        );
        if did_indent && bd.textcol - ind_post_col > 0 {
            // undo the temp adjustments
            (*oap_t).start.col -= ind_post_col - ind_pre_col;
            (*oap_t).start_vcol -= ind_post_vcol - ind_pre_vcol;
            (*oap_t).end.col -= ind_post_col - ind_pre_col;
            (*oap_t).end_vcol -= ind_post_vcol - ind_pre_vcol;
        }
        if bd.is_max == 0 || bd2.textlen < bd.textlen {
            if (*oap_t).op_type == OP_APPEND {
                pre_textlen += bd2.textlen - bd.textlen;
                if bd2.endspaces != 0 {
                    bd2.textlen -= 1;
                }
            }
            bd.textcol = bd2.textcol;
            bd.textlen = bd2.textlen;
        }

        // Subsequent calls to ml_get() flush the firstline data -- take a copy.
        let firstline = ml_get((*oap_t).start.lnum);
        let len = ml_get_len((*oap_t).start.lnum);
        let mut add = bd.textcol;
        let mut offset: c_int = 0; // offset when cursor was moved in insert mode
        if (*oap_t).op_type == OP_APPEND {
            add += bd.textlen;
            // account for pressing cursor in insert mode when '$' was used
            let insstart_lnum = nvim_get_Insstart_lnum();
            let insstart_col = nvim_get_Insstart_col();
            if bd.is_max != 0
                && start_insert.lnum == insstart_lnum
                && start_insert.col > insstart_col
            {
                offset = start_insert.col - insstart_col;
                add -= offset;
                if (*oap_t).end_vcol > offset {
                    (*oap_t).end_vcol -= offset + 1;
                } else {
                    // moved outside of the visual block, what to do?
                    return;
                }
            }
        }
        add = add.min(len); // short line, point to the NUL
        let firstline = firstline.add(add as usize);
        let len = len - add;
        let ins_len = len - pre_textlen - offset;
        if pre_textlen >= 0 && ins_len > 0 {
            let ins_text = xmemdupz(firstline.cast::<c_void>(), ins_len as usize);
            // block handled here
            if u_save((*oap_t).start.lnum, (*oap_t).end.lnum + 1) == OK {
                let b_insert = (*oap_t).op_type != OP_APPEND;
                block_insert(oap, ins_text, ins_len as usize, b_insert, &raw mut bd);
            }

            nvim_set_cursor_col((*oap_t).start.col);
            check_cursor(curwin);
            xfree(ins_text.cast::<c_void>());
        }
    }
}
