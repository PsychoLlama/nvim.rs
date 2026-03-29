//! Replace loops absorbed from C (Phase 1)
//!
//! Ports of `nvim_opr_block_loop`, `nvim_opr_charwise_loop`, `pbyte`, and
//! `replace_character` from ops.c.  These were previously exported as C
//! functions called by `replace_full.rs`; after this migration they live
//! entirely in Rust and are called directly.

#![allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::too_many_lines,
    clippy::useless_let_if_seq,
    clippy::similar_names
)]

use nvim_normal::types::{OpargT, PosT};
use std::ffi::{c_char, c_int, c_void};

// -----------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------

const NUL: c_int = 0;
const NL: c_int = b'\n' as c_int;
const CR: c_int = b'\r' as c_int;
const TAB: c_int = b'\t' as c_int;
const MAXCOL: c_int = i32::MAX;
const K_EXTMARK_UNDO: c_int = 1; // kExtmarkUndo
const MODE_REPLACE: c_int = 0x14; // state_defs.h MODE_REPLACE
const K_MT_LINE_WISE: c_int = 1;

// -----------------------------------------------------------------------
// FFI declarations
// -----------------------------------------------------------------------

#[allow(clashing_extern_declarations)]
extern "C" {
    // Block prep
    fn block_prep(oap: *mut c_void, bdp: *mut c_void, lnum: c_int, is_del: bool);

    // Cursor accessors (from normal_shim.c)
    fn nvim_get_cursor_lnum() -> c_int;
    fn nvim_set_cursor_lnum(lnum: c_int);
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_set_cursor_col(col: c_int);
    fn nvim_get_curswant() -> c_int;

    // Cursor position helpers
    fn get_cursor_line_ptr() -> *mut c_char;
    fn get_cursor_line_len() -> c_int;
    fn get_cursor_pos_ptr() -> *const c_char;
    fn gchar_cursor() -> c_int;
    fn inc_cursor() -> c_int;
    fn dec_cursor() -> c_int;

    // Position iteration helpers
    fn dec(lp: *mut c_void) -> c_int;

    // Virtual column helpers
    fn getvpos(wp: *mut c_void, pos: *mut c_void, wcol: c_int) -> c_int;
    fn getviscol() -> c_int;
    fn getviscol2(col: c_int, coladd: c_int) -> c_int;
    fn coladvance_force(wcol: c_int);

    // Multibyte
    fn utf_char2cells(c: c_int) -> c_int;
    fn utf_char2len(c: c_int) -> c_int;
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    fn utfc_ptr2len(p: *const c_char) -> c_int;

    // Memory
    fn xmalloc(size: usize) -> *mut c_void;
    fn xmallocz(size: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_void);

    // Memline
    fn ml_replace(lnum: c_int, line: *mut c_char, copy: bool) -> c_int;
    fn ml_append(lnum: c_int, line: *mut c_char, len: c_int, newfile: bool) -> c_int;
    fn ml_get_len(lnum: c_int) -> c_int;

    // Buffer mutation (direct byte write)
    fn ml_get_buf_mut(buf: *mut c_void, lnum: c_int) -> *mut c_char;

    // Extmarks
    fn extmark_splice(
        buf: *mut c_void,
        lnum: c_int,
        col: c_int,
        old_lines: c_int,
        old_col: c_int,
        old_byte: c_int,
        new_lines: c_int,
        new_col: c_int,
        new_byte: c_int,
        op: c_int,
    );
    fn extmark_splice_cols(
        buf: *mut c_void,
        lnum: c_int,
        col: c_int,
        old_col: c_int,
        new_col: c_int,
        op: c_int,
    );
    fn appended_lines_mark(lnum: c_int, count: c_int);

    // Insert character (used by replace_character)
    fn ins_char(c: c_int);

    // Globals
    static mut State: c_int;
    static mut curbuf: *mut c_void;
    static mut curwin: *mut c_void;
    static mut curbuf_splice_pending: c_int;
    fn nvim_get_virtual_op() -> c_int;
}

// -----------------------------------------------------------------------
// BlockDefC -- mirror of C `struct block_def`
// -----------------------------------------------------------------------

/// Mirror of C `struct block_def` from register_defs.h.
/// All boolean fields in C are `int` in this struct.
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
        // SAFETY: all-zero is valid for this POD C struct
        unsafe { std::mem::zeroed() }
    }
}

// -----------------------------------------------------------------------
// `pbyte` -- put byte at position (mirrors C pbyte in ops.c)
// -----------------------------------------------------------------------

/// Write byte `c` at buffer position `lp`.
///
/// # Safety
/// - `lp` must reference a valid position in the current buffer.
/// - `c` must be <= UCHAR_MAX (i.e., a valid u8 value).
#[inline]
pub(crate) unsafe fn pbyte(lp: PosT, c: c_int) {
    let line = ml_get_buf_mut(curbuf, lp.lnum);
    *line.add(lp.col as usize) = c as u8 as c_char;
    if curbuf_splice_pending == 0 {
        extmark_splice_cols(curbuf, lp.lnum - 1, lp.col, 1, 1, K_EXTMARK_UNDO);
    }
}

// -----------------------------------------------------------------------
// `replace_character` -- replace char under cursor with `c`
// -----------------------------------------------------------------------

/// Replace the character under the cursor with `c` using insert mode.
///
/// # Safety
/// Modifies global `State` and calls `ins_char`/`dec_cursor`.
#[inline]
pub(crate) unsafe fn replace_character(c: c_int) {
    let saved_state = State;
    State = MODE_REPLACE;
    ins_char(c);
    State = saved_state;
    // Backup to the replaced character.
    dec_cursor();
}

// -----------------------------------------------------------------------
// `rs_opr_block_loop` -- block mode replacement loop
// -----------------------------------------------------------------------

/// Port of `nvim_opr_block_loop` from ops.c.
///
/// Iterates over all lines in the block and replaces characters with `c`.
/// If `had_ctrl_v_cr` is true, a newline/CR replacement splits lines.
///
/// # Safety
/// - `oap` must be a valid `oparg_T *`.
/// - Accesses global state via C functions.
pub(crate) unsafe fn rs_opr_block_loop(oap: *mut c_void, c: c_int, had_ctrl_v_cr: bool) {
    let oap_t: *mut OpargT = oap.cast();
    let mut bd = BlockDefC::zeroed();

    bd.is_max = c_int::from(nvim_get_curswant() == MAXCOL);

    // for (; curwin->w_cursor.lnum <= oap->end.lnum; curwin->w_cursor.lnum++)
    loop {
        let cursor_lnum = nvim_get_cursor_lnum();
        // Re-read end.lnum each iteration since it may change (line splits).
        if cursor_lnum > (*oap_t).end.lnum {
            break;
        }

        nvim_set_cursor_col(0);
        block_prep(oap, (&raw mut bd).cast(), cursor_lnum, true);

        let virtual_op = nvim_get_virtual_op() != 0;
        if bd.textlen == 0 && (!virtual_op || bd.is_max != 0) {
            nvim_set_cursor_lnum(cursor_lnum + 1);
            continue;
        }

        let n: c_int;
        if virtual_op && bd.is_short != 0 && *bd.textstart == 0 {
            let mut vpos: PosT = std::mem::zeroed();
            vpos.lnum = cursor_lnum;
            getvpos(curwin, (&raw mut vpos).cast(), (*oap_t).start_vcol);
            bd.startspaces += vpos.coladd;
            n = bd.startspaces;
        } else {
            n = if bd.startspaces != 0 {
                bd.start_char_vcols - 1
            } else {
                0
            };
        }

        // n is only used for computation in C; we don't need the result
        let _ = n + if bd.endspaces != 0 && bd.is_onechar == 0 && bd.end_char_vcols > 0 {
            bd.end_char_vcols - 1
        } else {
            0
        };

        let mut numc = (*oap_t).end_vcol - (*oap_t).start_vcol + 1;
        if bd.is_short != 0 && (!virtual_op || bd.is_max != 0) {
            numc -= ((*oap_t).end_vcol - bd.end_vcol) + 1;
        }

        if utf_char2cells(c) > 1 {
            if (numc & 1) != 0 && bd.is_short == 0 {
                bd.endspaces += 1;
            }
            numc /= 2;
        }

        let num_chars = numc;
        numc *= utf_char2len(c);

        let oldp = get_cursor_line_ptr();
        let oldlen = get_cursor_line_len();

        let mut newp_size = (bd.textcol as usize) + (bd.startspaces as usize);
        if had_ctrl_v_cr || (c != NL && c != CR) {
            newp_size += numc as usize;
            if bd.is_short == 0 {
                newp_size += (bd.endspaces + oldlen - bd.textcol - bd.textlen) as usize;
            }
        }
        let newp: *mut c_char = xmallocz(newp_size);

        std::ptr::copy_nonoverlapping(oldp, newp, bd.textcol as usize);
        let oldp_after = oldp.add((bd.textcol + bd.textlen) as usize);
        std::ptr::write_bytes(newp.add(bd.textcol as usize), b' ', bd.startspaces as usize);

        let mut after_p: *mut c_char = std::ptr::null_mut();
        let mut after_p_len: usize = 0;
        let col = oldlen - bd.textcol - bd.textlen + 1;
        let mut newrows: c_int = 0;
        let mut newcols: c_int = 0;

        if had_ctrl_v_cr || (c != NL && c != CR) {
            let mut newp_len = bd.textcol + bd.startspaces;
            let mut nc = num_chars;
            while nc > 0 {
                newp_len += utf_char2bytes(c, newp.add(newp_len as usize));
                nc -= 1;
            }
            if bd.is_short == 0 {
                std::ptr::write_bytes(newp.add(newp_len as usize), b' ', bd.endspaces as usize);
                newp_len += bd.endspaces;
                std::ptr::copy_nonoverlapping(
                    oldp_after,
                    newp.add(newp_len as usize),
                    col as usize,
                );
            }
            newcols = newp_len - bd.textcol;
        } else {
            after_p_len = col as usize;
            after_p = xmalloc(after_p_len).cast();
            std::ptr::copy_nonoverlapping(oldp_after, after_p, after_p_len);
            newrows = 1;
        }

        ml_replace(cursor_lnum, newp, false);
        curbuf_splice_pending += 1;
        let baselnum = cursor_lnum;
        if !after_p.is_null() {
            // ml_append(curwin->w_cursor.lnum++, ...) -- post-increment:
            // use cursor_lnum as arg, then set cursor_lnum to cursor_lnum+1
            ml_append(cursor_lnum, after_p, after_p_len as c_int, false);
            let new_lnum = cursor_lnum + 1;
            nvim_set_cursor_lnum(new_lnum);
            appended_lines_mark(new_lnum, 1);
            (*oap_t).end.lnum += 1;
            xfree(after_p.cast());
        }
        curbuf_splice_pending -= 1;
        extmark_splice(
            curbuf,
            baselnum - 1,
            bd.textcol,
            0,
            bd.textlen,
            bd.textlen,
            newrows,
            newcols,
            newrows + newcols,
            K_EXTMARK_UNDO,
        );

        // for-loop post-increment: curwin->w_cursor.lnum++
        nvim_set_cursor_lnum(nvim_get_cursor_lnum() + 1);
    }
}

// -----------------------------------------------------------------------
// `rs_opr_charwise_loop` -- charwise/linewise replacement loop
// -----------------------------------------------------------------------

/// Port of `nvim_opr_charwise_loop` from ops.c.
///
/// Iterates over the charwise or linewise range and replaces each character
/// with `c`.
///
/// # Safety
/// - `oap` must be a valid `oparg_T *`.
/// - Accesses global state via C functions.
pub(crate) unsafe fn rs_opr_charwise_loop(oap: *mut c_void, c: c_int) {
    let oap_t: *mut OpargT = oap.cast();

    if (*oap_t).motion_type == K_MT_LINE_WISE {
        (*oap_t).start.col = 0;
        nvim_set_cursor_col(0);
        let end_len = ml_get_len((*oap_t).end.lnum);
        (*oap_t).end.col = if end_len > 0 { end_len - 1 } else { 0 };
    } else if !(*oap_t).inclusive {
        // dec(&oap->end)
        dec((&raw mut (*oap_t).end).cast());
    }

    // while (ltoreq(curwin->w_cursor, oap->end))
    loop {
        let cursor_lnum = nvim_get_cursor_lnum();
        let cursor_col = nvim_get_cursor_col();
        let end_lnum = (*oap_t).end.lnum;
        let end_col = (*oap_t).end.col;

        // ltoreq: curwin->w_cursor <= oap->end
        let ltoreq = cursor_lnum < end_lnum || (cursor_lnum == end_lnum && cursor_col <= end_col);
        if !ltoreq {
            break;
        }

        let mut done = false;
        let n = gchar_cursor();

        if n != NUL {
            let new_byte_len = utf_char2len(c);
            let old_byte_len = utfc_ptr2len(get_cursor_pos_ptr());

            if new_byte_len > 1 || old_byte_len > 1 {
                if nvim_get_cursor_lnum() == (*oap_t).end.lnum {
                    (*oap_t).end.col += new_byte_len - old_byte_len;
                }
                replace_character(c);
                done = true;
            } else {
                if n == TAB {
                    let mut end_vcol: c_int = 0;
                    if nvim_get_cursor_lnum() == (*oap_t).end.lnum {
                        end_vcol = getviscol2((*oap_t).end.col, (*oap_t).end.coladd);
                    }
                    coladvance_force(getviscol());
                    if nvim_get_cursor_lnum() == (*oap_t).end.lnum {
                        getvpos(curwin, (&raw mut (*oap_t).end).cast(), end_vcol);
                    }
                }
                if gchar_cursor() != NUL {
                    let lp = PosT {
                        lnum: nvim_get_cursor_lnum(),
                        col: nvim_get_cursor_col(),
                        coladd: 0,
                    };
                    pbyte(lp, c);
                    done = true;
                }
            }
        }

        let virtual_op = nvim_get_virtual_op() != 0;
        if !done && virtual_op && nvim_get_cursor_lnum() == (*oap_t).end.lnum {
            let mut virtcols = (*oap_t).end.coladd;
            if nvim_get_cursor_lnum() == (*oap_t).start.lnum
                && (*oap_t).start.col == (*oap_t).end.col
                && (*oap_t).start.coladd != 0
            {
                virtcols -= (*oap_t).start.coladd;
            }
            coladvance_force(getviscol2((*oap_t).end.col, (*oap_t).end.coladd) + 1);
            nvim_set_cursor_col(nvim_get_cursor_col() - (virtcols + 1));
            while virtcols >= 0 {
                if utf_char2len(c) > 1 {
                    replace_character(c);
                } else {
                    let lp2 = PosT {
                        lnum: nvim_get_cursor_lnum(),
                        col: nvim_get_cursor_col(),
                        coladd: 0,
                    };
                    pbyte(lp2, c);
                }
                if inc_cursor() == -1 {
                    break;
                }
                virtcols -= 1;
            }
        }

        if inc_cursor() == -1 {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(K_MT_LINE_WISE, 1);
        assert_eq!(K_EXTMARK_UNDO, 1);
        assert_eq!(MODE_REPLACE, 0x14);
        assert_eq!(NUL, 0);
        assert_eq!(TAB, 9);
    }
}
