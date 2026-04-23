//! Normal mode finish command handling.
//!
//! This module provides the Rust implementation of `normal_finish_command()`
//! from `src/nvim/normal.c`. Handles post-command cleanup: operator
//! resolution, mode messages, cursor shape, scrollbind, cursorbind,
//! and potential restart of insert/visual-select mode.

use std::ffi::c_int;

use crate::dispatch::types::NormalStateHandle;
use crate::types::{CmdargT, NormalState};
use crate::{CapHandle, OapHandle, WinHandle};

/// Cast `NormalStateHandle` to a typed `*mut NormalState`.
///
/// # Safety
/// The handle must be a valid non-null `NormalState*`.
#[inline]
unsafe fn ns(s: NormalStateHandle) -> *mut NormalState {
    s.as_ptr().cast::<NormalState>()
}

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
    // cmdarg_T accessors
    // oparg_T accessors

    // Global accessors (direct linkage)
    static mut finish_op: bool;
    static mut VIsual_active: bool;
    static mut VIsual_select: bool;
    static mut msg_nowait: bool;
    static mut restart_edit: c_int;
    static mut opcount: c_int;
    static mut VIsual_select_reg: c_int;
    static mut restart_VIsual_select: c_int;
    fn stuff_empty() -> bool;

    // Phase 3 wrappers
    fn rs_clearop(oap: OapHandle);
    fn set_reg_var(c: c_int);
    fn get_default_register_name() -> c_int;
    fn typebuf_maplen() -> c_int;
    fn do_pending_operator(ca: CapHandle, old_col: c_int, gui_yank: bool);
    fn rs_normal_need_redraw_mode_message(s: NormalStateHandle) -> bool;
    fn rs_normal_redraw_mode_message(s: NormalStateHandle);
    fn may_trigger_modechanged();
    fn ui_cursor_shape();
    fn rs_clear_showcmd();
    fn checkpcmark();
    fn xfree(ptr: *mut std::ffi::c_void);
    fn nvim_curwin_get_p_scb() -> bool;
    fn nvim_curwin_get_p_crb() -> bool;
    fn nvim_validate_cursor_curwin_wrapper();
    fn do_check_scrollbind(flag: bool);
    fn do_check_cursorbind();
    fn edit(cmd: c_int, startln: bool, count: c_int) -> bool;
    fn showmode() -> c_int;
    fn nvim_get_curwin() -> WinHandle;
    fn mb_check_adjust_col(win: WinHandle);
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
    let sp = ns(s);
    let ca: CapHandle = (&raw mut (*sp).ca).cast();
    let oa: OapHandle = (&raw mut (*sp).oa).cast();

    let mut did_visual_op = false;

    // The 'finish: block replaces `goto normal_end` — breaking out of
    // this block skips directly to the normal_end cleanup code.
    'finish: {
        if (*sp).command_finished {
            break 'finish;
        }

        // If we didn't start or finish an operator, reset oap->regname,
        // unless we need it later.
        let idx = (*sp).idx;
        if !finish_op
            && (*oa).op_type == 0
            && (idx < 0 || (crate::dispatch::table::rs_table_get_cmd_flags(idx) & NV_KEEPREG == 0))
        {
            rs_clearop(oa);
            set_reg_var(get_default_register_name());
        }

        // Get the length of mapped chars again after typing a count,
        // second character or "z333<cr>".
        if (*sp).old_mapped_len > 0 {
            (*sp).old_mapped_len = typebuf_maplen();
        }

        // If an operation is pending, handle it. But not for K_IGNORE or K_MOUSEMOVE.
        let cmdchar = (*ca).cmdchar;
        if cmdchar != K_IGNORE && cmdchar != K_MOUSEMOVE {
            let op_type = (*oa).op_type;
            did_visual_op = VIsual_active && op_type != OP_NOP && op_type != OP_COLON;
            do_pending_operator(ca, (*sp).old_col, false);
        }

        // Wait for a moment when a message is displayed that will be
        // overwritten by the mode message.
        if rs_normal_need_redraw_mode_message(s) {
            rs_normal_redraw_mode_message(s);
        }
    }
    // normal_end:

    msg_nowait = false;

    if finish_op || did_visual_op {
        set_reg_var(get_default_register_name());
    }

    let prev_finish_op = finish_op;
    if (*oa).op_type == OP_NOP {
        // Reset finish_op, in case it was set.
        finish_op = false;
        may_trigger_modechanged();
    }

    // Redraw the cursor with another shape, if we were in Operator-pending
    // mode or did a replace command.
    let cmdchar = (*ca).cmdchar;
    if prev_finish_op
        || cmdchar == c_int::from(b'r')
        || (cmdchar == c_int::from(b'g') && (*ca).nchar == c_int::from(b'r'))
    {
        ui_cursor_shape();
    }

    if (*oa).op_type == OP_NOP && (*oa).regname == 0 && cmdchar != K_EVENT {
        rs_clear_showcmd();
    }

    checkpcmark();
    {
        let cap_typed = ca.cast::<CmdargT>();
        xfree((*cap_typed).searchbuf.cast::<std::ffi::c_void>());
        (*cap_typed).searchbuf = std::ptr::null_mut();
    }

    mb_check_adjust_col(nvim_get_curwin());

    if nvim_curwin_get_p_scb() && (*sp).toplevel {
        nvim_validate_cursor_curwin_wrapper();
        do_check_scrollbind(true);
    }

    if nvim_curwin_get_p_crb() && (*sp).toplevel {
        nvim_validate_cursor_curwin_wrapper();
        do_check_cursorbind();
    }

    // May restart edit(), if we got here with CTRL-O in Insert mode
    // (but not if still inside a mapping that started in Visual mode).
    // May switch from Visual to Select mode after CTRL-O command.
    if (*oa).op_type == OP_NOP
        && ((restart_edit != 0 && !VIsual_active && (*sp).old_mapped_len == 0)
            || restart_VIsual_select == 1)
        && ((*ca).retval & CA_COMMAND_BUSY == 0)
        && stuff_empty()
        && (*oa).regname == 0
    {
        if restart_VIsual_select == 1 {
            VIsual_select = true;
            VIsual_select_reg = 0;
            may_trigger_modechanged();
            showmode();
            restart_VIsual_select = 0;
        }
        if restart_edit != 0 && !VIsual_active && (*sp).old_mapped_len == 0 {
            edit(restart_edit, false, 1);
        }
    }

    if restart_VIsual_select == 2 {
        restart_VIsual_select = 1;
    }

    // Save count before an operator for next time.
    opcount = (*ca.cast::<CmdargT>()).opcount;
}
