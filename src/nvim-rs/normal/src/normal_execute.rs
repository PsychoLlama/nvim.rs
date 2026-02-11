//! Normal mode command execution.
//!
//! This module provides the Rust implementation of `normal_execute()`
//! from `src/nvim/normal.c`. This is the per-key callback for normal mode:
//! it processes a key press, sets up the command, handles counts, dispatches
//! the command handler, and calls `normal_finish_command`.

use std::ffi::c_int;

use crate::dispatch::types::NormalStateHandle;
use crate::{CapHandle, OapHandle};

// =============================================================================
// Constants (verified with _Static_assert in normal.c)
// =============================================================================

const K_IGNORE: c_int = -13821;
const K_EVENT: c_int = -26365;
const K_KENTER: c_int = -16715;
const K_ZERO: c_int = -22783;
const NUL: c_int = 0;
const NL: c_int = 10;
const CAR: c_int = 13;
const ESC: c_int = 27;
const CTRL_W: c_int = 23;
const MOD_MASK_SHIFT: c_int = 0x02;
const MODE_NORMAL: c_int = 0x01;
const MODE_SELECT: c_int = 0x40;
const NV_NCW: c_int = 0x200;
const NV_RL: c_int = 0x80;
const NV_SS: c_int = 0x10;
const NV_SSS: c_int = 0x20;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // NormalState accessors
    fn nvim_ns_set_command_finished(s: NormalStateHandle, val: bool);
    fn nvim_ns_set_ctrl_w(s: NormalStateHandle, val: bool);
    fn nvim_ns_get_ctrl_w(s: NormalStateHandle) -> bool;
    fn nvim_ns_set_old_col(s: NormalStateHandle, val: c_int);
    fn nvim_ns_get_c(s: NormalStateHandle) -> c_int;
    fn nvim_ns_set_c(s: NormalStateHandle, val: c_int);
    fn nvim_ns_get_old_mapped_len(s: NormalStateHandle) -> c_int;
    fn nvim_ns_set_old_mapped_len(s: NormalStateHandle, val: c_int);
    fn nvim_ns_get_mapped_len(s: NormalStateHandle) -> c_int;
    fn nvim_ns_set_need_flushbuf(s: NormalStateHandle, val: bool);
    fn nvim_ns_get_need_flushbuf(s: NormalStateHandle) -> bool;
    fn nvim_ns_get_set_prevcount(s: NormalStateHandle) -> bool;
    fn nvim_ns_get_toplevel(s: NormalStateHandle) -> bool;
    fn nvim_ns_set_idx(s: NormalStateHandle, val: c_int);
    fn nvim_ns_get_idx(s: NormalStateHandle) -> c_int;
    fn nvim_ns_set_old_pos(s: NormalStateHandle);
    fn nvim_ns_get_ca_ptr(s: NormalStateHandle) -> CapHandle;
    fn nvim_ns_get_oa_ptr(s: NormalStateHandle) -> OapHandle;

    // cmdarg_T accessors
    fn nvim_cap_get_cmdchar(cap: CapHandle) -> c_int;
    fn nvim_cap_set_cmdchar(cap: CapHandle, val: c_int);
    fn nvim_cap_get_nchar(cap: CapHandle) -> c_int;
    fn nvim_cap_set_nchar(cap: CapHandle, val: c_int);
    fn nvim_cap_get_extra_char(cap: CapHandle) -> c_int;
    fn nvim_cap_get_opcount(cap: CapHandle) -> c_int;
    fn nvim_cap_set_opcount(cap: CapHandle, val: c_int);
    fn nvim_cap_get_count0(cap: CapHandle) -> c_int;
    fn nvim_cap_set_count0(cap: CapHandle, val: c_int);
    fn nvim_cap_set_count1(cap: CapHandle, val: c_int);

    // oparg_T accessors
    fn nvim_oap_set_prev_opcount(oap: OapHandle, val: c_int);
    fn nvim_oap_set_prev_count0(oap: OapHandle, val: c_int);

    // Command table accessors
    fn nvim_get_nv_cmd_flags(idx: c_int) -> c_int;

    // Global accessors (existing)
    fn nvim_get_restart_edit() -> c_int;
    fn nvim_get_VIsual_active() -> c_int;
    fn nvim_get_VIsual_select() -> bool;
    fn nvim_get_KeyTyped() -> bool;
    fn nvim_get_keystuffed() -> c_int;
    fn nvim_get_mod_mask() -> c_int;
    fn nvim_get_real_state() -> c_int;
    fn nvim_get_vgetc_char() -> c_int;
    fn nvim_get_vgetc_mod_mask() -> c_int;
    fn nvim_get_km_startsel() -> bool;
    fn nvim_get_curwin_w_p_rl() -> bool;
    fn nvim_get_curwin_w_curswant() -> c_int;
    fn nvim_set_msg_nowait(val: c_int);
    fn nvim_set_msg_didout(val: c_int);
    fn nvim_set_msg_col(val: c_int);
    fn nvim_set_did_cursorhold(val: bool);
    fn nvim_set_State(val: c_int);

    // Function wrappers (existing)
    fn nvim_langmap_adjust(c: c_int, condition: bool) -> c_int;
    fn nvim_typebuf_maplen_wrapper() -> c_int;
    fn nvim_clearop_wrapper(oap: OapHandle);

    // Phase 3
    fn rs_normal_finish_command(s: NormalStateHandle);

    // Phase 4B wrappers
    fn vim_isprintc(c: c_int) -> bool;
    fn ins_char_typebuf(c: c_int, modifiers: c_int, on_key_ignore: bool) -> c_int;
    fn ungetchars(len: c_int);
    fn readbuf1_empty() -> bool;
    fn nvim_add_to_showcmd_wrapper(c: c_int) -> bool;
    fn nvim_normal_get_command_count_loop(s: NormalStateHandle);
    fn nvim_set_vcount_call(count: i64, count1: i64, set_prevcount: bool);
    fn rs_find_command(cmdchar: c_int) -> c_int;
    fn nvim_clearopbeep_wrapper(oap: OapHandle);
    fn nvim_check_text_or_curbuf_locked_wrapper(oap: OapHandle) -> bool;
    fn nvim_normal_handle_special_visual_command_wrapper(s: NormalStateHandle) -> bool;
    fn nvim_normal_invert_horizontal_wrapper(s: NormalStateHandle);
    fn nvim_normal_need_additional_char_wrapper(s: NormalStateHandle) -> bool;
    fn rs_normal_get_additional_char(s: NormalStateHandle);
    fn nvim_ui_flush_wrapper();
    fn nvim_start_selection_wrapper();
    fn nvim_unshift_special_wrapper(ca: CapHandle);
    fn nvim_mod_mask_clear_shift();
    fn nvim_execute_nv_cmd(idx: c_int, ca: CapHandle);
}

/// Per-key callback for normal mode.
///
/// Processes a key press: sets up command state, handles select-mode
/// replacement, processes counts, dispatches the command, and finishes.
///
/// # Safety
/// `s` must be a valid NormalState pointer.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_normal_execute(s: NormalStateHandle, key: c_int) -> c_int {
    let ca = nvim_ns_get_ca_ptr(s);
    let oa = nvim_ns_get_oa_ptr(s);

    nvim_ns_set_command_finished(s, false);
    nvim_ns_set_ctrl_w(s, false);
    nvim_ns_set_old_col(s, nvim_get_curwin_w_curswant());
    nvim_ns_set_c(s, key);

    // LANGMAP_ADJUST
    let adjusted = nvim_langmap_adjust(nvim_ns_get_c(s), nvim_get_real_state() != MODE_SELECT);
    nvim_ns_set_c(s, adjusted);

    // If a mapping was started in Visual or Select mode, remember the length.
    if nvim_get_restart_edit() == 0 {
        nvim_ns_set_old_mapped_len(s, 0);
    } else if nvim_ns_get_old_mapped_len(s) != 0
        || (nvim_get_VIsual_active() != 0
            && nvim_ns_get_mapped_len(s) == 0
            && nvim_typebuf_maplen_wrapper() > 0)
    {
        nvim_ns_set_old_mapped_len(s, nvim_typebuf_maplen_wrapper());
    }

    if nvim_ns_get_c(s) == NUL {
        nvim_ns_set_c(s, K_ZERO);
    }

    // In Select mode, typed text replaces the selection.
    let c = nvim_ns_get_c(s);
    if nvim_get_VIsual_active() != 0
        && nvim_get_VIsual_select()
        && (vim_isprintc(c) || c == NL || c == CAR || c == K_KENTER)
    {
        let len = ins_char_typebuf(nvim_get_vgetc_char(), nvim_get_vgetc_mod_mask(), true);

        if nvim_get_KeyTyped() {
            ungetchars(len);
        }

        if nvim_get_restart_edit() != 0 {
            nvim_ns_set_c(s, c_int::from(b'd'));
        } else {
            nvim_ns_set_c(s, c_int::from(b'c'));
        }
        nvim_set_msg_nowait(1);
        nvim_ns_set_old_mapped_len(s, 0);
    }

    nvim_ns_set_need_flushbuf(s, nvim_add_to_showcmd_wrapper(nvim_ns_get_c(s)));

    nvim_normal_get_command_count_loop(s);

    if nvim_ns_get_c(s) == K_EVENT {
        // Save count values for K_EVENT re-entry.
        nvim_oap_set_prev_opcount(oa, nvim_cap_get_opcount(ca));
        nvim_oap_set_prev_count0(oa, nvim_cap_get_count0(ca));
    } else if nvim_cap_get_opcount(ca) != 0 {
        // Multiply counts: "3dw" → "d3w".
        let opcount = nvim_cap_get_opcount(ca);
        let count0 = nvim_cap_get_count0(ca);
        if count0 != 0 {
            if opcount >= 999_999_999 / count0 {
                nvim_cap_set_count0(ca, 999_999_999);
            } else {
                nvim_cap_set_count0(ca, count0 * opcount);
            }
        } else {
            nvim_cap_set_count0(ca, opcount);
        }
    }

    // Always remember the count.
    let count0 = nvim_cap_get_count0(ca);
    nvim_cap_set_opcount(ca, count0);
    nvim_cap_set_count1(ca, if count0 == 0 { 1 } else { count0 });

    // Only set v:count when called from main() and not a stuffed command.
    if nvim_ns_get_toplevel(s) && readbuf1_empty() {
        nvim_set_vcount_call(
            i64::from(nvim_cap_get_count0(ca)),
            i64::from(nvim_cap_get_count0(ca).max(1)),
            nvim_ns_get_set_prevcount(s),
        );
    }

    // Find the command character in the table of commands.
    if nvim_ns_get_ctrl_w(s) {
        nvim_cap_set_nchar(ca, nvim_ns_get_c(s));
        nvim_cap_set_cmdchar(ca, CTRL_W);
    } else {
        nvim_cap_set_cmdchar(ca, nvim_ns_get_c(s));
    }

    let idx = rs_find_command(nvim_cap_get_cmdchar(ca));
    nvim_ns_set_idx(s, idx);

    'finish: {
        if idx < 0 {
            nvim_clearopbeep_wrapper(oa);
            nvim_ns_set_command_finished(s, true);
            break 'finish;
        }

        if (nvim_get_nv_cmd_flags(idx) & NV_NCW != 0)
            && nvim_check_text_or_curbuf_locked_wrapper(oa)
        {
            nvim_ns_set_command_finished(s, true);
            break 'finish;
        }

        // In Visual/Select mode, a few keys are handled in a special way.
        if nvim_get_VIsual_active() != 0 && nvim_normal_handle_special_visual_command_wrapper(s) {
            nvim_ns_set_command_finished(s, true);
            break 'finish;
        }

        if nvim_get_curwin_w_p_rl()
            && nvim_get_KeyTyped()
            && nvim_get_keystuffed() == 0
            && (nvim_get_nv_cmd_flags(nvim_ns_get_idx(s)) & NV_RL != 0)
        {
            nvim_normal_invert_horizontal_wrapper(s);
        }

        // Get an additional character if we need one.
        if nvim_normal_need_additional_char_wrapper(s) {
            rs_normal_get_additional_char(s);
        }

        // Flush showcmd characters.
        if nvim_ns_get_need_flushbuf(s) {
            nvim_ui_flush_wrapper();
        }

        if nvim_cap_get_cmdchar(ca) != K_IGNORE && nvim_cap_get_cmdchar(ca) != K_EVENT {
            nvim_set_did_cursorhold(false);
        }

        nvim_set_State(MODE_NORMAL);

        if nvim_cap_get_nchar(ca) == ESC || nvim_cap_get_extra_char(ca) == ESC {
            nvim_clearop_wrapper(oa);
            nvim_ns_set_command_finished(s, true);
            break 'finish;
        }

        if nvim_cap_get_cmdchar(ca) != K_IGNORE {
            nvim_set_msg_didout(0);
            nvim_set_msg_col(0);
        }

        nvim_ns_set_old_pos(s); // remember where the cursor was

        // When 'keymodel' contains "startsel" some keys start Select/Visual mode.
        if nvim_get_VIsual_active() == 0 && nvim_get_km_startsel() {
            let cur_idx = nvim_ns_get_idx(s);
            let flags = nvim_get_nv_cmd_flags(cur_idx);
            if flags & NV_SS != 0 {
                nvim_start_selection_wrapper();
                nvim_unshift_special_wrapper(ca);
                let new_idx = rs_find_command(nvim_cap_get_cmdchar(ca));
                debug_assert!(new_idx >= 0);
                nvim_ns_set_idx(s, new_idx);
            } else if (flags & NV_SSS != 0) && (nvim_get_mod_mask() & MOD_MASK_SHIFT != 0) {
                nvim_start_selection_wrapper();
                nvim_mod_mask_clear_shift();
            }
        }

        // Execute the command!
        nvim_execute_nv_cmd(nvim_ns_get_idx(s), ca);
    }

    rs_normal_finish_command(s);
    1
}
