//! Full `op_delete` migration (Phase 4)
//!
//! Migrated from `op_delete()` in ops.c — delete operations (d, x, D, etc).

use std::ffi::{c_int, c_void};

const OK: c_int = 1;
const FAIL: c_int = 0;
const K_MT_BLOCK_WISE: c_int = 2;
const K_MT_LINE_WISE: c_int = 1;

extern "C" {
    // Generic buffer/undo accessors (reuse existing C shims)
    fn nvim_curbuf_ml_empty() -> bool;
    fn nvim_oap_get_empty(oap: *const c_void) -> c_int;
    fn nvim_u_save_cursor() -> c_int;
    fn nvim_curbuf_is_modifiable() -> bool;
    fn nvim_emsg_modifiable();
    fn nvim_oap_get_motion_type(oap: *const c_void) -> c_int;

    // Complex ops: still delegated to C
    fn nvim_opd_mb_adjust_opend(oap: *mut c_void);
    fn nvim_opd_setup_visual_reg(oap: *mut c_void);
    fn nvim_opd_maybe_promote_to_linewise(oap: *mut c_void);
    fn nvim_opd_check_empty_line(oap: *mut c_void) -> c_int;
    fn nvim_opd_do_yank_and_registers(oap: *mut c_void) -> c_int;
    fn nvim_opd_block_delete(oap: *mut c_void) -> c_int;
    fn nvim_opd_linewise_delete(oap: *mut c_void) -> c_int;
    fn nvim_opd_charwise_delete(oap: *mut c_void) -> c_int;
    fn nvim_opd_finish(oap: *mut c_void, old_lcount: c_int);
    fn nvim_opd_setmarks(oap: *mut c_void);
    fn nvim_curbuf_get_ml_line_count() -> c_int;
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

    if nvim_oap_get_empty(oap) != 0 {
        return nvim_u_save_cursor();
    }

    if !nvim_curbuf_is_modifiable() {
        nvim_emsg_modifiable();
        return FAIL;
    }

    nvim_opd_setup_visual_reg(oap);
    nvim_opd_mb_adjust_opend(oap);
    nvim_opd_maybe_promote_to_linewise(oap);

    // Check for empty line special cases
    let empty_check = nvim_opd_check_empty_line(oap);
    if empty_check == 1 {
        return OK;
    }
    if empty_check == 2 {
        // goto setmarks
        nvim_opd_setmarks(oap);
        return OK;
    }

    // Do yank/register handling (returns FAIL if read-only register)
    let yank_result = nvim_opd_do_yank_and_registers(oap);
    if yank_result == OK {
        // yank returned OK = read-only register, return OK
        // (the C code returns OK after beep_flush)
        return OK;
    }
    // yank_result == 1 means proceed normally

    let old_lcount = nvim_curbuf_get_ml_line_count();
    let motion_type = nvim_oap_get_motion_type(oap);

    let result = if motion_type == K_MT_BLOCK_WISE {
        nvim_opd_block_delete(oap)
    } else if motion_type == K_MT_LINE_WISE {
        nvim_opd_linewise_delete(oap)
    } else {
        nvim_opd_charwise_delete(oap)
    };

    if result == FAIL {
        return FAIL;
    }

    nvim_opd_finish(oap, old_lcount);
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
