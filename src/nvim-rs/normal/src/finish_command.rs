//! Normal mode finish command handling.
//!
//! This module provides the Rust implementation of `normal_finish_command()`
//! from `src/nvim/normal.c`. Handles post-command cleanup: operator
//! resolution, mode messages, cursor shape, scrollbind, cursorbind,
//! and potential restart of insert/visual-select mode.

use std::ffi::c_int;

use crate::dispatch::types::NormalStateHandle;
use crate::{CapHandle, OapHandle};

// =============================================================================
// Constants (verified with _Static_assert in normal.c)
// =============================================================================

const K_IGNORE: c_int = -13821;
const K_MOUSEMOVE: c_int = -25853;
const K_EVENT: c_int = -26365;
const OP_NOP: c_int = 0;
const OP_COLON: c_int = 10;
const CA_COMMAND_BUSY: c_int = 1;
const NV_KEEPREG: c_int = 0x100;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // NormalState accessors
    fn nvim_ns_get_command_finished(s: NormalStateHandle) -> bool;
    fn nvim_ns_get_idx(s: NormalStateHandle) -> c_int;
    fn nvim_ns_get_old_mapped_len(s: NormalStateHandle) -> c_int;
    fn nvim_ns_set_old_mapped_len(s: NormalStateHandle, val: c_int);
    fn nvim_ns_get_old_col(s: NormalStateHandle) -> c_int;
    fn nvim_ns_get_toplevel(s: NormalStateHandle) -> bool;
    fn nvim_ns_get_ca_ptr(s: NormalStateHandle) -> CapHandle;
    fn nvim_ns_get_oa_ptr(s: NormalStateHandle) -> OapHandle;

    // cmdarg_T accessors
    fn nvim_cap_get_cmdchar(cap: CapHandle) -> c_int;
    fn nvim_cap_get_nchar(cap: CapHandle) -> c_int;
    fn nvim_cap_get_retval(cap: CapHandle) -> c_int;
    fn nvim_cap_get_opcount(cap: CapHandle) -> c_int;

    // oparg_T accessors
    fn nvim_oap_get_op_type_ptr(oap: OapHandle) -> c_int;
    fn nvim_oap_get_regname_ptr(oap: OapHandle) -> c_int;

    // Global accessors
    fn nvim_get_finish_op() -> c_int;
    fn nvim_set_finish_op(val: bool);
    static mut VIsual_active: bool;
    fn nvim_set_msg_nowait(val: c_int);
    static mut restart_edit: c_int;
    fn nvim_get_restart_VIsual_select() -> c_int;
    fn nvim_set_restart_VIsual_select(val: c_int);
    fn nvim_set_VIsual_select(val: bool);
    fn nvim_set_VIsual_select_reg(val: c_int);
    fn nvim_set_opcount(val: c_int);
    fn nvim_stuff_empty() -> bool;

    // Phase 3 wrappers
    fn rs_clearop(oap: OapHandle);
    fn nvim_set_reg_var_default();
    fn nvim_typebuf_maplen_wrapper() -> c_int;
    fn nvim_do_pending_operator_call(ca: CapHandle, old_col: c_int, gui_yank: bool);
    fn rs_normal_need_redraw_mode_message(s: NormalStateHandle) -> bool;
    fn rs_normal_redraw_mode_message(s: NormalStateHandle);
    fn nvim_may_trigger_modechanged();
    fn nvim_ui_cursor_shape_wrapper();
    fn rs_clear_showcmd();
    fn nvim_checkpcmark_wrapper();
    fn nvim_xfree_cap_searchbuf(ca: CapHandle);
    fn nvim_mb_check_adjust_col_wrapper();
    fn nvim_curwin_get_p_scb() -> bool;
    fn nvim_curwin_get_p_crb() -> bool;
    fn nvim_validate_cursor_curwin_wrapper();
    fn nvim_do_check_scrollbind_wrapper(flag: bool);
    fn nvim_do_check_cursorbind_wrapper();
    fn nvim_edit_wrapper(cmd: c_int, startln: bool, count: c_int);
    fn nvim_showmode();
}

/// Finish a normal-mode command: operator resolution, mode messages,
/// cursor shape, scrollbind/cursorbind, and potential edit() restart.
///
/// This is the Rust implementation of `normal_finish_command()` from normal.c.
/// The `goto normal_end` pattern is replaced with a labeled block.
///
/// # Safety
/// `s` must be a valid NormalState pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_normal_finish_command(s: NormalStateHandle) {
    let ca = nvim_ns_get_ca_ptr(s);
    let oa = nvim_ns_get_oa_ptr(s);

    let mut did_visual_op = false;

    // The 'finish: block replaces `goto normal_end` — breaking out of
    // this block skips directly to the normal_end cleanup code.
    'finish: {
        if nvim_ns_get_command_finished(s) {
            break 'finish;
        }

        // If we didn't start or finish an operator, reset oap->regname,
        // unless we need it later.
        let idx = nvim_ns_get_idx(s);
        if nvim_get_finish_op() == 0
            && nvim_oap_get_op_type_ptr(oa) == 0
            && (idx < 0 || (crate::dispatch::table::rs_table_get_cmd_flags(idx) & NV_KEEPREG == 0))
        {
            rs_clearop(oa);
            nvim_set_reg_var_default();
        }

        // Get the length of mapped chars again after typing a count,
        // second character or "z333<cr>".
        if nvim_ns_get_old_mapped_len(s) > 0 {
            nvim_ns_set_old_mapped_len(s, nvim_typebuf_maplen_wrapper());
        }

        // If an operation is pending, handle it. But not for K_IGNORE or K_MOUSEMOVE.
        let cmdchar = nvim_cap_get_cmdchar(ca);
        if cmdchar != K_IGNORE && cmdchar != K_MOUSEMOVE {
            let op_type = nvim_oap_get_op_type_ptr(oa);
            did_visual_op = VIsual_active && op_type != OP_NOP && op_type != OP_COLON;
            nvim_do_pending_operator_call(ca, nvim_ns_get_old_col(s), false);
        }

        // Wait for a moment when a message is displayed that will be
        // overwritten by the mode message.
        if rs_normal_need_redraw_mode_message(s) {
            rs_normal_redraw_mode_message(s);
        }
    }
    // normal_end:

    nvim_set_msg_nowait(0);

    if nvim_get_finish_op() != 0 || did_visual_op {
        nvim_set_reg_var_default();
    }

    let prev_finish_op = nvim_get_finish_op() != 0;
    if nvim_oap_get_op_type_ptr(oa) == OP_NOP {
        // Reset finish_op, in case it was set.
        nvim_set_finish_op(false);
        nvim_may_trigger_modechanged();
    }

    // Redraw the cursor with another shape, if we were in Operator-pending
    // mode or did a replace command.
    let cmdchar = nvim_cap_get_cmdchar(ca);
    if prev_finish_op
        || cmdchar == c_int::from(b'r')
        || (cmdchar == c_int::from(b'g') && nvim_cap_get_nchar(ca) == c_int::from(b'r'))
    {
        nvim_ui_cursor_shape_wrapper();
    }

    if nvim_oap_get_op_type_ptr(oa) == OP_NOP
        && nvim_oap_get_regname_ptr(oa) == 0
        && cmdchar != K_EVENT
    {
        rs_clear_showcmd();
    }

    nvim_checkpcmark_wrapper();
    nvim_xfree_cap_searchbuf(ca);

    nvim_mb_check_adjust_col_wrapper();

    if nvim_curwin_get_p_scb() && nvim_ns_get_toplevel(s) {
        nvim_validate_cursor_curwin_wrapper();
        nvim_do_check_scrollbind_wrapper(true);
    }

    if nvim_curwin_get_p_crb() && nvim_ns_get_toplevel(s) {
        nvim_validate_cursor_curwin_wrapper();
        nvim_do_check_cursorbind_wrapper();
    }

    // May restart edit(), if we got here with CTRL-O in Insert mode
    // (but not if still inside a mapping that started in Visual mode).
    // May switch from Visual to Select mode after CTRL-O command.
    if nvim_oap_get_op_type_ptr(oa) == OP_NOP
        && ((restart_edit != 0 && !VIsual_active && nvim_ns_get_old_mapped_len(s) == 0)
            || nvim_get_restart_VIsual_select() == 1)
        && (nvim_cap_get_retval(ca) & CA_COMMAND_BUSY == 0)
        && nvim_stuff_empty()
        && nvim_oap_get_regname_ptr(oa) == 0
    {
        if nvim_get_restart_VIsual_select() == 1 {
            nvim_set_VIsual_select(true);
            nvim_set_VIsual_select_reg(0);
            nvim_may_trigger_modechanged();
            nvim_showmode();
            nvim_set_restart_VIsual_select(0);
        }
        if restart_edit != 0 && !VIsual_active && nvim_ns_get_old_mapped_len(s) == 0 {
            nvim_edit_wrapper(restart_edit, false, 1);
        }
    }

    if nvim_get_restart_VIsual_select() == 2 {
        nvim_set_restart_VIsual_select(1);
    }

    // Save count before an operator for next time.
    nvim_set_opcount(nvim_cap_get_opcount(ca));
}
