//! Full `op_replace` migration (Phase 3)
//!
//! Migrated from `op_replace()` in ops.c — the `r` command for
//! replacing characters in visual/operator mode.

use std::ffi::{c_int, c_void};

const OK: c_int = 1;
const FAIL: c_int = 0;
const CAR: c_int = b'\r' as c_int;
const NL: c_int = b'\n' as c_int;
const REPLACE_CR_NCHAR: c_int = -1;
const REPLACE_NL_NCHAR: c_int = -2;
const K_MT_BLOCK_WISE: c_int = 2;

extern "C" {
    fn nvim_opr_is_empty(oap: *mut c_void) -> c_int;
    fn nvim_opr_get_motion_type(oap: *mut c_void) -> c_int;
    fn nvim_opr_mb_adjust_opend(oap: *mut c_void);
    fn nvim_opr_u_save(oap: *mut c_void) -> c_int;

    // Block mode: full iteration + replacement delegated to C
    fn nvim_opr_block_loop(oap: *mut c_void, c: c_int, had_ctrl_v_cr: c_int);

    // Charwise/linewise mode: setup + full loop delegated to C
    fn nvim_opr_charwise_loop(oap: *mut c_void, c: c_int);

    // Cleanup: restore cursor, changed_lines, set marks
    fn nvim_opr_finish(oap: *mut c_void);
}

/// Full migration of `op_replace()`.
///
/// # Safety
/// - `oap` must be a valid `oparg_T *`
/// - Accesses global state via C accessors
#[unsafe(export_name = "op_replace")]
pub unsafe extern "C" fn rs_op_replace(oap: *mut c_void, c: c_int) -> c_int {
    if nvim_opr_is_empty(oap) != 0 {
        return OK;
    }

    // Normalize special replacement characters
    let (c, had_ctrl_v_cr) = match c {
        REPLACE_CR_NCHAR => (CAR, true),
        REPLACE_NL_NCHAR => (NL, true),
        _ => (c, false),
    };

    nvim_opr_mb_adjust_opend(oap);

    if nvim_opr_u_save(oap) == FAIL {
        return FAIL;
    }

    if nvim_opr_get_motion_type(oap) == K_MT_BLOCK_WISE {
        nvim_opr_block_loop(oap, c, c_int::from(had_ctrl_v_cr));
    } else {
        nvim_opr_charwise_loop(oap, c);
    }

    nvim_opr_finish(oap);
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
