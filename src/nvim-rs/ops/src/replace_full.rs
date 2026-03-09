//! Full `op_replace` migration (Phase 3)
//!
//! Migrated from `op_replace()` in ops.c — the `r` command for
//! replacing characters in visual/operator mode.
//! Phase 4 absorption: nvim_opr_finish ported inline.

use std::ffi::{c_int, c_void};

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
    fn nvim_oap_get_empty(oap: *const c_void) -> c_int;
    fn nvim_oap_get_motion_type(oap: *const c_void) -> c_int;
    fn nvim_opd_mb_adjust_opend(oap: *mut c_void);
    fn nvim_oap_get_start_lnum(oap: *const c_void) -> c_int;
    fn nvim_oap_get_start_col(oap: *const c_void) -> c_int;
    fn nvim_oap_get_end_lnum(oap: *const c_void) -> c_int;
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

/// Inline port of `nvim_opr_finish`.
///
/// # Safety
/// Reads oap fields and sets cursor/marks via C shims.
unsafe fn opr_finish(oap: *mut c_void) {
    let oap_const: *const c_void = oap;
    nvim_curwin_set_cursor_from_oap_start(oap);
    nvim_check_cursor();
    let start_lnum = nvim_oap_get_start_lnum(oap_const);
    let start_col = nvim_oap_get_start_col(oap_const);
    let end_lnum = nvim_oap_get_end_lnum(oap_const);
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
    if nvim_curbuf_ml_empty() || nvim_oap_get_empty(oap) != 0 {
        return OK;
    }

    // Normalize special replacement characters
    let (c, had_ctrl_v_cr) = match c {
        REPLACE_CR_NCHAR => (CAR, true),
        REPLACE_NL_NCHAR => (NL, true),
        _ => (c, false),
    };

    nvim_opd_mb_adjust_opend(oap);

    let start_lnum = nvim_oap_get_start_lnum(oap);
    let end_lnum = nvim_oap_get_end_lnum(oap);
    if nvim_u_save(start_lnum - 1, end_lnum + 1) == FAIL {
        return FAIL;
    }

    if nvim_oap_get_motion_type(oap) == K_MT_BLOCK_WISE {
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
