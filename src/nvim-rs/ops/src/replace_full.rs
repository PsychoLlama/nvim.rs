//! Full `op_replace` migration (Phase 3)
//!
//! Migrated from `op_replace()` in ops.c — the `r` command for
//! replacing characters in visual/operator mode.
//! Phase 4 absorption: nvim_opr_finish ported inline.

use nvim_normal::types::OpargT;
use std::ffi::{c_char, c_int, c_void};

const OK: c_int = 1;
const FAIL: c_int = 0;
const CAR: c_int = b'\r' as c_int;
const NL: c_int = b'\n' as c_int;
const REPLACE_CR_NCHAR: c_int = -1;
const REPLACE_NL_NCHAR: c_int = -2;
const K_MT_BLOCK_WISE: c_int = 2;

extern "C" {
    // Generic accessors (reuse existing C shims)
    fn nvim_curbuf_ml_empty() -> bool;
    fn nvim_ml_get(lnum: c_int) -> *const c_char;
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn nvim_u_save(top: c_int, bot: c_int) -> c_int;

    // Block mode: full iteration + replacement delegated to C
    fn nvim_opr_block_loop(oap: *mut c_void, c: c_int, had_ctrl_v_cr: c_int);

    // Charwise/linewise mode: setup + full loop delegated to C
    fn nvim_opr_charwise_loop(oap: *mut c_void, c: c_int);

    // Finish: restore cursor, changed_lines, set marks (absorbed below)
    fn nvim_curwin_set_cursor_from_oap_start(oap: *mut c_void);
    fn nvim_check_cursor();
    fn nvim_changed_lines_call(lnum: c_int, col: c_int, lnum_end: c_int, do_concealed: bool);
    fn nvim_cmdmod_has_lockmarks() -> c_int;
    fn nvim_curbuf_set_op_start_from_oap_start(oap: *mut c_void);
    fn nvim_curbuf_set_op_end_from_oap_end(oap: *mut c_void);
}

/// Port of `mb_adjust_opend` -- adjust oap->end.col to multi-byte char boundary.
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

/// Inline port of `nvim_opr_finish`.
///
/// # Safety
/// Reads oap fields and sets cursor/marks via C shims.
unsafe fn opr_finish(oap: *mut c_void) {
    let oap_const: *const c_void = oap;
    nvim_curwin_set_cursor_from_oap_start(oap);
    nvim_check_cursor();
    let start_lnum = (*oap_const.cast::<OpargT>()).start.lnum;
    let start_col = (*oap_const.cast::<OpargT>()).start.col;
    let end_lnum = (*oap_const.cast::<OpargT>()).end.lnum;
    nvim_changed_lines_call(start_lnum, start_col, end_lnum + 1, true);
    if nvim_cmdmod_has_lockmarks() == 0 {
        nvim_curbuf_set_op_start_from_oap_start(oap);
        nvim_curbuf_set_op_end_from_oap_end(oap);
    }
}

/// Full migration of `op_replace()`.
///
/// # Safety
/// - `oap` must be a valid `oparg_T *`
/// - Accesses global state via C accessors
#[unsafe(export_name = "op_replace")]
pub unsafe extern "C" fn rs_op_replace(oap: *mut c_void, c: c_int) -> c_int {
    if nvim_curbuf_ml_empty() || (*oap.cast::<OpargT>()).empty {
        return OK;
    }

    // Normalize special replacement characters
    let (c, had_ctrl_v_cr) = match c {
        REPLACE_CR_NCHAR => (CAR, true),
        REPLACE_NL_NCHAR => (NL, true),
        _ => (c, false),
    };

    mb_adjust_opend(oap);

    let start_lnum = (*oap.cast::<OpargT>()).start.lnum;
    let end_lnum = (*oap.cast::<OpargT>()).end.lnum;
    if nvim_u_save(start_lnum - 1, end_lnum + 1) == FAIL {
        return FAIL;
    }

    if (*oap.cast::<OpargT>()).motion_type == K_MT_BLOCK_WISE {
        nvim_opr_block_loop(oap, c, c_int::from(had_ctrl_v_cr));
    } else {
        nvim_opr_charwise_loop(oap, c);
    }

    opr_finish(oap);
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
        assert_eq!(REPLACE_CR_NCHAR, -1);
        assert_eq!(REPLACE_NL_NCHAR, -2);
    }

    #[test]
    fn test_normalize_replace_char() {
        // CR special
        let (c, flag) = match REPLACE_CR_NCHAR {
            REPLACE_CR_NCHAR => (CAR, true),
            REPLACE_NL_NCHAR => (NL, true),
            _ => (REPLACE_CR_NCHAR, false),
        };
        assert_eq!(c, CAR);
        assert!(flag);

        // NL special
        let (c, flag) = match REPLACE_NL_NCHAR {
            REPLACE_CR_NCHAR => (CAR, true),
            REPLACE_NL_NCHAR => (NL, true),
            _ => (REPLACE_NL_NCHAR, false),
        };
        assert_eq!(c, NL);
        assert!(flag);

        // Regular char unchanged
        let ch = c_int::from(b'x');
        let (c, flag) = match ch {
            REPLACE_CR_NCHAR => (CAR, true),
            REPLACE_NL_NCHAR => (NL, true),
            _ => (ch, false),
        };
        assert_eq!(c, c_int::from(b'x'));
        assert!(!flag);
    }
}
