//! Delete helpers absorbed from C (Phase 2)
//!
//! Ports of `nvim_opd_do_yank_and_registers`, `nvim_opd_block_delete`, and
//! `nvim_opd_charwise_delete` from ops.c.  These were previously exported as
//! C functions called by `delete_full.rs`; after this migration they live
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

use crate::yank::is_append_register;
use nvim_normal::types::{OpargT, PosT};
use std::ffi::{c_char, c_int, c_void};

// -----------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------

const OK: c_int = 1;
const FAIL: c_int = 0;
const NUL: c_int = 0;
const TAB: c_int = b'\t' as c_int;
const K_MT_LINE_WISE: c_int = 1;
const K_EXTMARK_UNDO: c_int = 1; // kExtmarkUndo
const OP_CHANGE: c_int = 3;
const OP_DELETE: c_int = 1;
const CPO_DOLLAR: c_int = b'$' as c_int;
const YREG_YANK: c_int = 1; // register_defs.h YREG_YANK=1 (YREG_PASTE=0)

// -----------------------------------------------------------------------
// FFI declarations
// -----------------------------------------------------------------------

#[allow(clashing_extern_declarations)]
extern "C" {
    // Yank register functions
    fn valid_yank_reg(regname: c_int, writing: bool) -> bool;
    #[link_name = "beep_flush"]
    fn nvim_beep_flush_opd();
    fn get_yank_register(regname: c_int, mode: c_int) -> *mut c_void;
    fn op_yank_reg(oap: *mut c_void, message: bool, reg: *mut c_void, append: bool);
    fn shift_delete_registers(y_append: bool);
    fn get_y_register(reg: c_int) -> *mut c_void;
    fn set_clipboard(name: c_int, reg: *mut c_void);
    fn do_autocmd_textyankpost(oap: *mut c_void, reg: *mut c_void);

    // Undo
    fn u_save(top: c_int, bot: c_int) -> c_int;
    fn u_save_cursor() -> c_int;

    // Block prep
    fn block_prep(oap: *mut c_void, bdp: *mut c_void, lnum: c_int, is_del: bool);

    // Cursor accessors
    fn nvim_get_cursor_lnum() -> c_int;
    fn nvim_set_cursor_lnum(lnum: c_int);
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_set_cursor_col(col: c_int);
    fn nvim_get_cursor_coladd() -> c_int;
    fn nvim_set_cursor_coladd(val: c_int);
    fn nvim_curwin_set_cursor_from_pos(pos: *const c_void);

    // Cursor position helpers
    fn gchar_pos(pos: *const c_void) -> c_int;
    fn gchar_cursor() -> c_int;
    fn get_cursor_line_len() -> c_int;

    // Virtual column helpers
    fn getviscol2(col: c_int, coladd: c_int) -> c_int;
    fn coladvance_force(wcol: c_int);
    fn coladvance(wp: *mut c_void, col: c_int) -> c_int;

    // Multibyte
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn utfc_ptr2len(p: *const c_char) -> c_int;

    // Buffer / line
    fn ml_get(lnum: c_int) -> *mut c_char;
    fn ml_get_len(lnum: c_int) -> c_int;
    fn ml_replace(lnum: c_int, line: *mut c_char, copy: bool) -> c_int;

    // Memory
    fn xmalloc(size: usize) -> *mut c_void;

    // Delete operations
    fn del_bytes(count: c_int, fixpos: c_int, use_delcombine: c_int) -> c_int;
    fn del_lines(nlines: c_int, undo: bool);
    fn truncate_line(fixpos: c_int);
    fn do_join(
        count: usize,
        join_spaces: bool,
        insert_space: bool,
        save_undo: bool,
        use_formatoptions: bool,
    ) -> c_int;
    fn auto_format(trailblank: bool, prev_line: bool);

    // Check cursor
    fn check_cursor_col(wp: *mut c_void);

    // Display
    fn display_dollar(col: c_int);

    // cpo
    static mut p_cpo: *mut c_char;
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;

    // Extmarks / change notifications
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
    static mut curbuf_splice_pending: c_int;
    static mut virtual_op: c_int;
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

// -----------------------------------------------------------------------
// `opd_do_yank_and_registers`
// -----------------------------------------------------------------------

/// Port of `nvim_opd_do_yank_and_registers` from ops.c.
///
/// Returns `OK` (1) if a read-only register was used (caller should return),
/// or `2` to proceed normally.
///
/// # Safety
/// - `oap` must be a valid `oparg_T *`.
pub(crate) unsafe fn opd_do_yank_and_registers(oap: *mut c_void) -> c_int {
    let oap_t: *const OpargT = oap.cast();
    let regname = (*oap_t).regname;

    if regname != c_int::from(b'_') {
        let mut reg: *mut c_void = std::ptr::null_mut();
        let mut did_yank = false;

        if regname != 0 {
            if !valid_yank_reg(regname, true) {
                nvim_beep_flush_opd();
                return OK;
            }
            reg = get_yank_register(regname, YREG_YANK);
            op_yank_reg(oap, false, reg, is_append_register(regname));
            did_yank = true;
        }

        let motion_type = (*oap_t).motion_type;
        let line_count = (*oap_t).line_count;
        let use_reg_one = (*oap_t).use_reg_one;
        if motion_type == K_MT_LINE_WISE || line_count > 1 || use_reg_one {
            shift_delete_registers(is_append_register(regname));
            reg = get_y_register(1);
            op_yank_reg(oap, false, reg, false);
            did_yank = true;
        }

        if regname == 0 && motion_type != K_MT_LINE_WISE && line_count == 1 {
            reg = get_yank_register(c_int::from(b'-'), YREG_YANK);
            op_yank_reg(oap, false, reg, false);
            did_yank = true;
        }

        if did_yank || regname == 0 {
            debug_assert!(!reg.is_null());
            set_clipboard(regname, reg);
            do_autocmd_textyankpost(oap, reg);
        }
    }
    2 // proceed normally
}

// -----------------------------------------------------------------------
// `opd_block_delete`
// -----------------------------------------------------------------------

/// Port of `nvim_opd_block_delete` from ops.c.
///
/// # Safety
/// - `oap` must be a valid `oparg_T *`.
pub(crate) unsafe fn opd_block_delete(oap: *mut c_void) -> c_int {
    let oap_t: *const OpargT = oap.cast();
    // Zero-initialize bd (C uses `= { 0 }`)
    let mut bd: BlockDefC = std::mem::zeroed();

    let start_lnum = (*oap_t).start.lnum;
    let end_lnum = (*oap_t).end.lnum;

    if u_save(start_lnum - 1, end_lnum + 1) == FAIL {
        return FAIL;
    }

    let cursor_lnum = nvim_get_cursor_lnum();
    let mut lnum = cursor_lnum;
    while lnum <= end_lnum {
        block_prep(oap, (&raw mut bd).cast(), lnum, true);
        if bd.textlen == 0 {
            lnum += 1;
            continue;
        }

        if lnum == cursor_lnum {
            nvim_set_cursor_col(bd.textcol + bd.startspaces);
            nvim_set_cursor_coladd(0);
        }

        let n = bd.textlen - bd.startspaces - bd.endspaces;
        let oldp = ml_get(lnum);
        let old_line_len = ml_get_len(lnum);
        let newp: *mut c_char = xmalloc((old_line_len as usize) - (n as usize) + 1).cast();

        std::ptr::copy_nonoverlapping(oldp, newp, bd.textcol as usize);
        std::ptr::write_bytes(
            newp.add(bd.textcol as usize),
            b' ',
            (bd.startspaces + bd.endspaces) as usize,
        );
        // STRCPY(newp + bd.textcol + bd.startspaces + bd.endspaces,
        //        oldp + bd.textcol + bd.textlen)
        let dst = newp.add((bd.textcol + bd.startspaces + bd.endspaces) as usize);
        let src = oldp.add((bd.textcol + bd.textlen) as usize);
        // Length of source string including NUL: old_line_len - (bd.textcol + bd.textlen) + 1
        let src_len = (old_line_len - bd.textcol - bd.textlen + 1) as usize;
        std::ptr::copy_nonoverlapping(src, dst, src_len);

        ml_replace(lnum, newp, false);
        extmark_splice_cols(
            curbuf,
            lnum - 1,
            bd.textcol,
            bd.textlen,
            bd.startspaces + bd.endspaces,
            K_EXTMARK_UNDO,
        );

        lnum += 1;
    }

    check_cursor_col(curwin);
    let cursor_col = nvim_get_cursor_col();
    changed_lines(curbuf, cursor_lnum, cursor_col, end_lnum + 1, 0, true);
    (*oap.cast::<OpargT>()).line_count = 0;
    OK
}

// -----------------------------------------------------------------------
// `opd_charwise_delete`
// -----------------------------------------------------------------------

/// Port of `nvim_opd_charwise_delete` from ops.c.
///
/// # Safety
/// - `oap` must be a valid `oparg_T *`.
pub(crate) unsafe fn opd_charwise_delete(oap: *mut c_void) -> c_int {
    let oap_t: *mut OpargT = oap.cast();
    let is_virtual_op = virtual_op != 0;

    if is_virtual_op {
        if gchar_pos((&raw const (*oap_t).start).cast()) == TAB {
            let mut endcol: c_int = 0;
            if u_save_cursor() == FAIL {
                return FAIL;
            }
            if (*oap_t).line_count == 1 {
                endcol = getviscol2((*oap_t).end.col, (*oap_t).end.coladd);
            }
            coladvance_force(getviscol2((*oap_t).start.col, (*oap_t).start.coladd));
            // oap->start = curwin->w_cursor
            (*oap_t).start = PosT {
                lnum: nvim_get_cursor_lnum(),
                col: nvim_get_cursor_col(),
                coladd: nvim_get_cursor_coladd(),
            };
            if (*oap_t).line_count == 1 {
                coladvance(curwin, endcol);
                (*oap_t).end.col = nvim_get_cursor_col();
                (*oap_t).end.coladd = nvim_get_cursor_coladd();
                // curwin->w_cursor = oap->start
                nvim_curwin_set_cursor_from_pos((&raw const (*oap_t).start).cast());
            }
        }

        if gchar_pos((&raw const (*oap_t).end).cast()) == TAB
            && (*oap_t).end.coladd == 0
            && (*oap_t).inclusive
        {
            if u_save((*oap_t).end.lnum - 1, (*oap_t).end.lnum + 1) == FAIL {
                return FAIL;
            }
            // curwin->w_cursor = oap->end
            nvim_curwin_set_cursor_from_pos((&raw const (*oap_t).end).cast());
            coladvance_force(getviscol2((*oap_t).end.col, (*oap_t).end.coladd));
            // oap->end = curwin->w_cursor
            (*oap_t).end = PosT {
                lnum: nvim_get_cursor_lnum(),
                col: nvim_get_cursor_col(),
                coladd: nvim_get_cursor_coladd(),
            };
            // curwin->w_cursor = oap->start
            nvim_curwin_set_cursor_from_pos((&raw const (*oap_t).start).cast());
        }

        // mb_adjust_opend inlined: adjust end.col for multi-byte boundary
        if (*oap_t).inclusive {
            let line = ml_get((*oap_t).end.lnum);
            let ptr = line.add((*oap_t).end.col as usize);
            if *ptr != 0 {
                let ptr = ptr.sub(utf_head_off(line, ptr) as usize);
                let ptr = ptr.add((utfc_ptr2len(ptr) - 1) as usize);
                (*oap_t).end.col = ptr.offset_from(line) as c_int;
            }
        }
    }

    if (*oap_t).line_count == 1 {
        if u_save_cursor() == FAIL {
            return FAIL;
        }

        if !vim_strchr(p_cpo, CPO_DOLLAR).is_null()
            && (*oap_t).op_type == OP_CHANGE
            && (*oap_t).end.lnum == nvim_get_cursor_lnum()
            && !(*oap_t).is_visual
        {
            display_dollar((*oap_t).end.col - c_int::from(!(*oap_t).inclusive));
        }

        let mut n = (*oap_t).end.col - (*oap_t).start.col + 1 - c_int::from(!(*oap_t).inclusive);

        if is_virtual_op {
            let len = get_cursor_line_len();
            if (*oap_t).end.coladd != 0 && (*oap_t).end.col >= len - 1 && (*oap_t).start.coladd == 0
            {
                n += 1;
            }
            if n == 0 && (*oap_t).start.coladd != (*oap_t).end.coladd {
                n = 1;
            }
            if gchar_cursor() != NUL {
                nvim_set_cursor_coladd(0);
            }
        }

        del_bytes(
            n,
            c_int::from(!is_virtual_op),
            c_int::from((*oap_t).op_type == OP_DELETE && !(*oap_t).is_visual),
        );
    } else {
        let cursor_lnum = nvim_get_cursor_lnum();
        if u_save(cursor_lnum - 1, cursor_lnum + (*oap_t).line_count) == FAIL {
            return FAIL;
        }

        curbuf_splice_pending += 1;
        let startpos = PosT {
            lnum: cursor_lnum,
            col: nvim_get_cursor_col(),
            coladd: nvim_get_cursor_coladd(),
        };
        let deleted_bytes = (crate::get_region_bytecount(
            curbuf,
            startpos.lnum,
            (*oap_t).end.lnum,
            startpos.col,
            (*oap_t).end.col,
        ) + isize::from((*oap_t).inclusive)) as c_int;
        truncate_line(1); // true → fixpos=1

        let curpos = PosT {
            lnum: nvim_get_cursor_lnum(),
            col: nvim_get_cursor_col(),
            coladd: nvim_get_cursor_coladd(),
        };
        nvim_set_cursor_lnum(nvim_get_cursor_lnum() + 1);
        del_lines((*oap_t).line_count - 2, false);

        let n = (*oap_t).end.col + 1 - c_int::from(!(*oap_t).inclusive);
        nvim_set_cursor_col(0);
        del_bytes(
            n,
            c_int::from(!is_virtual_op),
            c_int::from((*oap_t).op_type == OP_DELETE && !(*oap_t).is_visual),
        );
        // curwin->w_cursor = curpos
        nvim_curwin_set_cursor_from_pos((&raw const curpos).cast());
        do_join(2, false, false, false, false);
        curbuf_splice_pending -= 1;
        extmark_splice(
            curbuf,
            startpos.lnum - 1,
            startpos.col,
            (*oap_t).line_count - 1,
            n,
            deleted_bytes,
            0,
            0,
            0,
            K_EXTMARK_UNDO,
        );
    }

    if (*oap_t).op_type == OP_DELETE {
        auto_format(false, true);
    }
    OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
        assert_eq!(K_MT_LINE_WISE, 1);
        assert_eq!(K_EXTMARK_UNDO, 1);
        assert_eq!(YREG_YANK, 1);
        assert_eq!(CPO_DOLLAR, b'$' as c_int);
    }
}
