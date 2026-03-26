//! Full `op_delete` migration (Phase 4)
//!
//! Migrated from `op_delete()` in ops.c — delete operations (d, x, D, etc).
//! Phase 3 absorption: simple accessor functions ported inline.

use nvim_normal::types::OpargT;
use std::ffi::{c_char, c_int, c_void};

const OK: c_int = 1;
const FAIL: c_int = 0;
const K_MT_BLOCK_WISE: c_int = 2;
const K_MT_LINE_WISE: c_int = 1;

const CPO_EMPTYREGION: c_int = b'E' as c_int;
const OP_DELETE: c_int = 1;
const OP_CHANGE: c_int = 3;
const K_MT_CHAR_WISE: c_int = 0;
const NUL: c_int = 0;
const BL_WHITE: c_int = 1; // cursor on first non-white
const BL_FIX: c_int = 4; // don't leave cursor on NUL

extern "C" {
    // Generic buffer/undo accessors (reuse existing C shims)
    fn nvim_curbuf_ml_empty() -> bool;
    fn nvim_u_save_cursor() -> c_int;
    fn nvim_curbuf_is_modifiable() -> bool;
    fn nvim_emsg_modifiable();
    fn nvim_curbuf_get_ml_line_count() -> c_int;

    // VIsual state
    fn nvim_get_VIsual_select() -> bool;
    fn nvim_get_VIsual_select_reg() -> c_int;

    // Virtual op flag
    fn nvim_get_virtual_op() -> c_int;

    // cpo option
    static mut p_cpo: *mut c_char;
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;

    // Beep
    fn nvim_beep_flush();

    // Line content
    fn nvim_ml_get(lnum: c_int) -> *const c_char;

    // skipwhite and inindent
    fn skipwhite(s: *const c_char) -> *mut c_char;
    fn nvim_inindent_zero() -> bool;

    // marks: set curbuf->b_op_start/end from oap
    fn nvim_cmdmod_has_lockmarks() -> c_int;
    fn nvim_curbuf_set_op_start_from_oap_start(oap: *mut c_void);
    fn nvim_curbuf_set_op_end_from_oap_start(oap: *mut c_void);
    fn nvim_curbuf_set_op_end_blockwise(oap: *mut c_void);

    // msgmore
    fn nvim_textfmt_msgmore(n: c_int);

    // mb_adjust_opend helpers
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    // linewise_delete helpers
    fn nvim_del_lines(count: c_int, undo: bool);
    fn nvim_truncate_line(del_newline: bool);
    fn nvim_curbuf_get_b_p_ai() -> c_int;
    fn nvim_set_did_ai(val: bool);
    fn nvim_set_ai_col(val: c_int);
    fn nvim_beginline(flags: c_int);
    fn nvim_u_clearline_curbuf();
    fn nvim_get_cursor_lnum() -> c_int;
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_set_cursor_lnum(lnum: c_int);

    // Complex ops: still delegated to C
    fn nvim_opd_do_yank_and_registers(oap: *mut c_void) -> c_int;
    fn nvim_opd_block_delete(oap: *mut c_void) -> c_int;
    fn nvim_opd_charwise_delete(oap: *mut c_void) -> c_int;
}

/// Port of `mb_adjust_opend` -- adjust opend for multi-byte character boundary.
///
/// If the operation is inclusive, advance oap->end.col to the last byte of the
/// multi-byte character that starts at or before oap->end.col.
///
/// # Safety
/// `oap` must be a valid `oparg_T *`.
#[inline]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
unsafe fn mb_adjust_opend(oap: *mut c_void) {
    let oap_c: *const c_void = oap;
    if !(*oap_c.cast::<OpargT>()).inclusive {
        return;
    }
    let end_lnum = (*oap_c.cast::<OpargT>()).end.lnum;
    let end_col = (*oap_c.cast::<OpargT>()).end.col;
    let line = nvim_ml_get(end_lnum);
    if line.is_null() {
        return;
    }
    let ptr = line.add(end_col as usize);
    if *ptr != 0 {
        let ptr = ptr.sub(utf_head_off(line, ptr) as usize);
        let ptr = ptr.add((utfc_ptr2len(ptr) - 1) as usize);
        let new_col = ptr.offset_from(line) as c_int;
        (*oap.cast::<OpargT>()).end.col = new_col;
    }
}

/// Inline port of `nvim_opd_setup_visual_reg`.
/// If VIsual_select and oap->is_VIsual, set oap->regname = VIsual_select_reg.
///
/// # Safety
/// Reads global VIsual_select state via C shims.
#[inline]
unsafe fn opd_setup_visual_reg(oap: *mut c_void) {
    let oap_const: *const c_void = oap;
    if nvim_get_VIsual_select() && (*oap_const.cast::<OpargT>()).is_visual {
        (*oap.cast::<OpargT>()).regname = nvim_get_VIsual_select_reg();
    }
}

/// Inline port of `nvim_opd_check_empty_line`.
/// Returns: 0 = proceed, 1 = return OK, 2 = goto setmarks.
///
/// # Safety
/// Reads oap fields and buffer state via C shims.
unsafe fn opd_check_empty_line(oap: *const c_void) -> c_int {
    let motion_type = (*oap.cast::<OpargT>()).motion_type;
    let line_count = (*oap.cast::<OpargT>()).line_count;
    let op_type = (*oap.cast::<OpargT>()).op_type;
    if motion_type != K_MT_LINE_WISE && line_count == 1 && op_type == OP_DELETE {
        let start_lnum = (*oap.cast::<OpargT>()).start.lnum;
        let line = nvim_ml_get(start_lnum);
        if !line.is_null() && unsafe { *line } == 0 {
            if nvim_get_virtual_op() != 0 {
                return 2; // goto setmarks
            }
            if !vim_strchr(p_cpo, CPO_EMPTYREGION).is_null() {
                nvim_beep_flush();
            }
            return 1; // return OK
        }
    }
    0 // proceed
}

/// Inline port of `nvim_opd_maybe_promote_to_linewise`.
/// If the delete motion ends at whitespace-only suffix, promote to linewise.
///
/// # Safety
/// Reads oap fields and buffer state via C shims.
#[allow(clippy::cast_sign_loss)]
unsafe fn opd_maybe_promote_to_linewise(oap: *mut c_void) {
    let oap_const: *const c_void = oap;
    let motion_type = (*oap_const.cast::<OpargT>()).motion_type;
    let is_visual = (*oap_const.cast::<OpargT>()).is_visual;
    let line_count = (*oap_const.cast::<OpargT>()).line_count;
    let motion_force = (*oap_const.cast::<OpargT>()).motion_force;
    let op_type = (*oap_const.cast::<OpargT>()).op_type;
    if motion_type == K_MT_CHAR_WISE
        && !is_visual
        && line_count > 1
        && motion_force == NUL
        && op_type == OP_DELETE
    {
        let end_lnum = (*oap_const.cast::<OpargT>()).end.lnum;
        let end_col = (*oap_const.cast::<OpargT>()).end.col;
        let line = nvim_ml_get(end_lnum);
        if line.is_null() {
            return;
        }
        let mut ptr = line.add(end_col as usize);
        if unsafe { *ptr } != 0 {
            let inclusive = (*oap_const.cast::<OpargT>()).inclusive;
            ptr = ptr.add(usize::from(inclusive));
        }
        let ptr = skipwhite(ptr);
        if unsafe { *ptr } == 0 && nvim_inindent_zero() {
            (*oap.cast::<OpargT>()).motion_type = K_MT_LINE_WISE;
        }
    }
}

/// Port of `nvim_opd_linewise_delete` -- handle linewise deletion.
///
/// # Safety
/// `oap` must be a valid `oparg_T *`.
unsafe fn opd_linewise_delete(oap: *mut c_void) -> c_int {
    let oap_c: *const c_void = oap;
    let op_type = (*oap_c.cast::<OpargT>()).op_type;
    let line_count = (*oap_c.cast::<OpargT>()).line_count;
    if op_type == OP_CHANGE {
        if line_count > 1 {
            let lnum = nvim_get_cursor_lnum();
            nvim_set_cursor_lnum(lnum + 1);
            nvim_del_lines(line_count - 1, true);
            nvim_set_cursor_lnum(lnum);
        }
        if nvim_u_save_cursor() == FAIL {
            return FAIL;
        }
        if nvim_curbuf_get_b_p_ai() != 0 {
            nvim_beginline(BL_WHITE);
            nvim_set_did_ai(true);
            nvim_set_ai_col(nvim_get_cursor_col());
        } else {
            nvim_beginline(0);
        }
        nvim_truncate_line(false);
        if line_count > 1 {
            nvim_u_clearline_curbuf();
        }
    } else {
        nvim_del_lines(line_count, true);
        nvim_beginline(BL_WHITE | BL_FIX);
        nvim_u_clearline_curbuf();
    }
    OK
}

/// Inline port of `nvim_opd_setmarks`.
/// Sets curbuf->b_op_start and b_op_end from oap, respecting LOCKMARKS.
///
/// # Safety
/// Reads oap fields and sets curbuf marks via C shims.
#[inline]
unsafe fn opd_setmarks(oap: *mut c_void) {
    if nvim_cmdmod_has_lockmarks() != 0 {
        return;
    }
    let oap_const: *const c_void = oap;
    if (*oap_const.cast::<OpargT>()).motion_type == K_MT_BLOCK_WISE {
        nvim_curbuf_set_op_end_blockwise(oap);
    } else {
        nvim_curbuf_set_op_end_from_oap_start(oap);
    }
    nvim_curbuf_set_op_start_from_oap_start(oap);
}

/// Inline port of `nvim_opd_finish`.
/// Calls msgmore, then sets marks.
///
/// # Safety
/// Reads oap fields and curbuf state via C shims.
#[inline]
unsafe fn opd_finish(oap: *mut c_void, old_lcount: c_int) {
    nvim_textfmt_msgmore(nvim_curbuf_get_ml_line_count() - old_lcount);
    opd_setmarks(oap);
}

/// Full migration of `op_delete()`.
///
/// # Safety
/// - `oap` must be a valid `oparg_T *`
/// - Accesses global state via C accessors
#[unsafe(export_name = "op_delete")]
pub unsafe extern "C" fn rs_op_delete(oap: *mut c_void) -> c_int {
    if nvim_curbuf_ml_empty() {
        return OK;
    }

    if (*oap.cast::<OpargT>()).empty {
        return nvim_u_save_cursor();
    }

    if !nvim_curbuf_is_modifiable() {
        nvim_emsg_modifiable();
        return FAIL;
    }

    opd_setup_visual_reg(oap);
    mb_adjust_opend(oap);
    opd_maybe_promote_to_linewise(oap);

    // Check for empty line special cases
    let oap_const: *const c_void = oap;
    let empty_check = opd_check_empty_line(oap_const);
    if empty_check == 1 {
        return OK;
    }
    if empty_check == 2 {
        // goto setmarks
        opd_setmarks(oap);
        return OK;
    }

    // Do yank/register handling (returns FAIL if read-only register)
    let yank_result = nvim_opd_do_yank_and_registers(oap);
    if yank_result == OK {
        // yank returned OK = read-only register, return OK
        // (the C code returns OK after beep_flush)
        return OK;
    }
    // yank_result == 2 means proceed normally

    let old_lcount = nvim_curbuf_get_ml_line_count();
    let motion_type = (*oap_const.cast::<OpargT>()).motion_type;

    let result = if motion_type == K_MT_BLOCK_WISE {
        nvim_opd_block_delete(oap)
    } else if motion_type == K_MT_LINE_WISE {
        opd_linewise_delete(oap)
    } else {
        nvim_opd_charwise_delete(oap)
    };

    if result == FAIL {
        return FAIL;
    }

    opd_finish(oap, old_lcount);
    OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
        assert_eq!(K_MT_BLOCK_WISE, 2);
        assert_eq!(K_MT_LINE_WISE, 1);
    }
}
