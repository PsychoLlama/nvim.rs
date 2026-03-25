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
const NV_STS: c_int = 0x40;
const OP_NOP: c_int = 0;
// K_DEL = TERMCAP2KEY('k','D') = -((107) + (68 << 8)) = -17515
const K_DEL: c_int = -17515;
// K_KDEL = TERMCAP2KEY(KS_EXTRA=253, KE_KDEL=80) = -((253) + (80 << 8)) = -20733
const K_KDEL: c_int = -20733;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    static mut State: c_int;
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

    // Global accessors (existing)
    static mut restart_edit: c_int;
    static mut VIsual_active: bool;
    fn nvim_get_VIsual_select() -> bool;
    fn nvim_get_KeyTyped() -> bool;
    fn nvim_get_keystuffed() -> c_int;
    fn nvim_get_mod_mask() -> c_int;
    #[link_name = "get_real_state"]
    fn nvim_get_real_state() -> c_int;
    fn nvim_get_vgetc_char() -> c_int;
    fn nvim_get_vgetc_mod_mask() -> c_int;
    fn nvim_get_km_startsel() -> bool;
    fn nvim_get_curwin_w_p_rl() -> bool;
    fn nvim_get_curswant() -> c_int;
    static mut msg_nowait: bool;
    static mut msg_didout: bool;
    static mut msg_col: c_int;
    fn nvim_set_did_cursorhold(val: bool);

    // Function wrappers (existing)
    fn nvim_langmap_adjust(c: c_int, condition: bool) -> c_int;
    fn nvim_typebuf_maplen_wrapper() -> c_int;
    fn rs_clearop(oap: OapHandle);

    // Phase 3
    fn rs_normal_finish_command(s: NormalStateHandle);

    // Phase 4B wrappers
    fn vim_isprintc(c: c_int) -> bool;
    fn ins_char_typebuf(c: c_int, modifiers: c_int, on_key_ignore: bool) -> c_int;
    fn ungetchars(len: c_int);
    fn readbuf1_empty() -> bool;
    fn nvim_add_to_showcmd_wrapper(c: c_int) -> bool;
    fn nvim_set_vcount_call(count: i64, count1: i64, set_prevcount: bool);
    fn rs_find_command(cmdchar: c_int) -> c_int;
    fn rs_clearopbeep(oap: OapHandle);
    fn rs_check_text_or_curbuf_locked(oap: OapHandle) -> bool;
    fn rs_invert_horizontal(cmdchar: c_int) -> c_int;
    fn rs_need_additional_char(idx: c_int, cmdchar: c_int, pending_op: bool) -> bool;
    fn rs_normal_get_additional_char(s: NormalStateHandle);
    fn nvim_ui_flush_wrapper();
    fn rs_start_selection();
    fn rs_unshift_special(cmdchar: c_int, modp: *mut c_int) -> c_int;
    fn nvim_set_mod_mask(val: c_int);
    fn nvim_mod_mask_clear_shift();
    #[link_name = "rs_execute_dispatch"]
    fn nvim_execute_nv_cmd(idx: c_int, ca: CapHandle);

    // Phase 5 count/visual accessors
    fn nvim_oap_get_op_type_ptr(oap: OapHandle) -> c_int;
    fn nvim_del_from_showcmd_wrapper(len: c_int);
    fn nvim_inc_no_mapping();
    fn nvim_dec_no_mapping();
    fn nvim_inc_allow_keys();
    fn nvim_dec_allow_keys();
    fn nvim_inc_no_zero_mapping();
    fn nvim_dec_no_zero_mapping();
    fn nvim_plain_vgetc_wrapper() -> c_int;
    fn nvim_ns_set_need_flushbuf_or(s: NormalStateHandle, val: bool);
    fn nvim_ns_set_set_prevcount(s: NormalStateHandle, val: bool);
    fn nvim_get_km_stopsel() -> bool;
    fn nvim_redraw_curbuf_inverted();
    #[link_name = "end_visual_mode"]
    fn rs_end_visual_mode();
    fn rs_set_vcount_ca(cap: CapHandle, set_prevcount: *mut bool);
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
    nvim_ns_set_old_col(s, nvim_get_curswant());
    nvim_ns_set_c(s, key);

    // LANGMAP_ADJUST
    let adjusted = nvim_langmap_adjust(nvim_ns_get_c(s), nvim_get_real_state() != MODE_SELECT);
    nvim_ns_set_c(s, adjusted);

    // If a mapping was started in Visual or Select mode, remember the length.
    if restart_edit == 0 {
        nvim_ns_set_old_mapped_len(s, 0);
    } else if nvim_ns_get_old_mapped_len(s) != 0
        || (VIsual_active && nvim_ns_get_mapped_len(s) == 0 && nvim_typebuf_maplen_wrapper() > 0)
    {
        nvim_ns_set_old_mapped_len(s, nvim_typebuf_maplen_wrapper());
    }

    if nvim_ns_get_c(s) == NUL {
        nvim_ns_set_c(s, K_ZERO);
    }

    // In Select mode, typed text replaces the selection.
    let c = nvim_ns_get_c(s);
    if VIsual_active
        && nvim_get_VIsual_select()
        && (vim_isprintc(c) || c == NL || c == CAR || c == K_KENTER)
    {
        let len = ins_char_typebuf(nvim_get_vgetc_char(), nvim_get_vgetc_mod_mask(), true);

        if nvim_get_KeyTyped() {
            ungetchars(len);
        }

        if restart_edit != 0 {
            nvim_ns_set_c(s, c_int::from(b'd'));
        } else {
            nvim_ns_set_c(s, c_int::from(b'c'));
        }
        msg_nowait = true;
        nvim_ns_set_old_mapped_len(s, 0);
    }

    nvim_ns_set_need_flushbuf(s, nvim_add_to_showcmd_wrapper(nvim_ns_get_c(s)));

    while rs_normal_get_command_count(s) {}

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
            rs_clearopbeep(oa);
            nvim_ns_set_command_finished(s, true);
            break 'finish;
        }

        if (crate::dispatch::table::rs_table_get_cmd_flags(idx) & NV_NCW != 0)
            && rs_check_text_or_curbuf_locked(oa)
        {
            nvim_ns_set_command_finished(s, true);
            break 'finish;
        }

        // In Visual/Select mode, a few keys are handled in a special way.
        if VIsual_active && rs_normal_handle_special_visual_command(s) {
            nvim_ns_set_command_finished(s, true);
            break 'finish;
        }

        if nvim_get_curwin_w_p_rl()
            && nvim_get_KeyTyped()
            && nvim_get_keystuffed() == 0
            && (crate::dispatch::table::rs_table_get_cmd_flags(nvim_ns_get_idx(s)) & NV_RL != 0)
        {
            let new_cmdchar = rs_invert_horizontal(nvim_cap_get_cmdchar(ca));
            nvim_cap_set_cmdchar(ca, new_cmdchar);
            nvim_ns_set_idx(s, rs_find_command(new_cmdchar));
        }

        // Get an additional character if we need one.
        let pending_op = nvim_oap_get_op_type_ptr(nvim_ns_get_oa_ptr(s)) != OP_NOP;
        if rs_need_additional_char(nvim_ns_get_idx(s), nvim_cap_get_cmdchar(ca), pending_op) {
            rs_normal_get_additional_char(s);
        }

        // Flush showcmd characters.
        if nvim_ns_get_need_flushbuf(s) {
            nvim_ui_flush_wrapper();
        }

        if nvim_cap_get_cmdchar(ca) != K_IGNORE && nvim_cap_get_cmdchar(ca) != K_EVENT {
            nvim_set_did_cursorhold(false);
        }

        State = MODE_NORMAL;

        if nvim_cap_get_nchar(ca) == ESC || nvim_cap_get_extra_char(ca) == ESC {
            rs_clearop(oa);
            nvim_ns_set_command_finished(s, true);
            break 'finish;
        }

        if nvim_cap_get_cmdchar(ca) != K_IGNORE {
            msg_didout = false;
            msg_col = 0;
        }

        nvim_ns_set_old_pos(s); // remember where the cursor was

        // When 'keymodel' contains "startsel" some keys start Select/Visual mode.
        if !VIsual_active && nvim_get_km_startsel() {
            let cur_idx = nvim_ns_get_idx(s);
            let flags = crate::dispatch::table::rs_table_get_cmd_flags(cur_idx);
            if flags & NV_SS != 0 {
                rs_start_selection();
                let mut mm = nvim_get_mod_mask();
                let new_cmdchar = rs_unshift_special(nvim_cap_get_cmdchar(ca), &raw mut mm);
                nvim_set_mod_mask(mm);
                nvim_cap_set_cmdchar(ca, new_cmdchar);
                let new_idx = rs_find_command(new_cmdchar);
                debug_assert!(new_idx >= 0);
                nvim_ns_set_idx(s, new_idx);
            } else if (flags & NV_SSS != 0) && (nvim_get_mod_mask() & MOD_MASK_SHIFT != 0) {
                rs_start_selection();
                nvim_mod_mask_clear_shift();
            }
        }

        // Execute the command!
        nvim_execute_nv_cmd(nvim_ns_get_idx(s), ca);
    }

    rs_normal_finish_command(s);
    1
}

/// Rust implementation of normal_get_command_count.
///
/// Reads digit keys to build ca.count0, handles CTRL-W prefix.
/// Returns true if the outer loop should iterate again (CTRL-W case).
///
/// # Safety
/// `s` must be a valid NormalState pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_normal_get_command_count(s: NormalStateHandle) -> bool {
    if VIsual_active && nvim_get_VIsual_select() {
        return false;
    }

    let ca = nvim_ns_get_ca_ptr(s);
    let oa = nvim_ns_get_oa_ptr(s);

    // Handle a count before a command and compute ca.count0.
    // Note that '0' is a command and not the start of a count, but it's
    // part of a count after other digits.
    while {
        let c = nvim_ns_get_c(s);
        (c >= c_int::from(b'1') && c <= c_int::from(b'9'))
            || (nvim_cap_get_count0(ca) != 0
                && (c == K_DEL || c == K_KDEL || c == c_int::from(b'0')))
    } {
        let c = nvim_ns_get_c(s);
        if c == K_DEL || c == K_KDEL {
            let new_count0 = nvim_cap_get_count0(ca) / 10;
            nvim_cap_set_count0(ca, new_count0);
            nvim_del_from_showcmd_wrapper(4); // delete the digit and ~@%
        } else if nvim_cap_get_count0(ca) > 99_999_999 {
            nvim_cap_set_count0(ca, 999_999_999);
        } else {
            let new_count0 = nvim_cap_get_count0(ca) * 10 + (c - c_int::from(b'0'));
            nvim_cap_set_count0(ca, new_count0);
        }

        // Set v:count here, when called from main() and not a stuffed
        // command, so that v:count can be used in an expression mapping
        // right after the count. Do set it for redo.
        if nvim_ns_get_toplevel(s) && readbuf1_empty() {
            let mut set_prevcount = nvim_ns_get_set_prevcount(s);
            rs_set_vcount_ca(ca, std::ptr::addr_of_mut!(set_prevcount));
            nvim_ns_set_set_prevcount(s, set_prevcount);
        }

        if nvim_ns_get_ctrl_w(s) {
            nvim_inc_no_mapping();
            nvim_inc_allow_keys(); // no mapping for nchar, but keys
        }

        nvim_inc_no_zero_mapping(); // don't map zero here
        let new_c = nvim_plain_vgetc_wrapper();
        let adjusted = nvim_langmap_adjust(new_c, true);
        nvim_ns_set_c(s, adjusted);
        nvim_dec_no_zero_mapping();
        if nvim_ns_get_ctrl_w(s) {
            nvim_dec_no_mapping();
            nvim_dec_allow_keys();
        }
        nvim_ns_set_need_flushbuf_or(s, nvim_add_to_showcmd_wrapper(nvim_ns_get_c(s)));
    }

    // If we got CTRL-W there may be a/another count
    if nvim_ns_get_c(s) == CTRL_W
        && !nvim_ns_get_ctrl_w(s)
        && nvim_oap_get_op_type_ptr(oa) == OP_NOP
    {
        nvim_ns_set_ctrl_w(s, true);
        nvim_cap_set_opcount(ca, nvim_cap_get_count0(ca)); // remember first count
        nvim_cap_set_count0(ca, 0);
        nvim_inc_no_mapping();
        nvim_inc_allow_keys(); // no mapping for nchar, but keys
        let new_c = nvim_plain_vgetc_wrapper(); // get next character
        let adjusted = nvim_langmap_adjust(new_c, true);
        nvim_ns_set_c(s, adjusted);
        nvim_dec_no_mapping();
        nvim_dec_allow_keys();
        nvim_ns_set_need_flushbuf_or(s, nvim_add_to_showcmd_wrapper(nvim_ns_get_c(s)));
        return true;
    }

    false
}

/// Rust implementation of normal_handle_special_visual_command.
///
/// Handles keymodel stopsel/startsel in visual mode.
/// Returns true if the command was consumed (clearopbeep case).
///
/// # Safety
/// `s` must be a valid NormalState pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_normal_handle_special_visual_command(s: NormalStateHandle) -> bool {
    let ca = nvim_ns_get_ca_ptr(s);
    let oa = nvim_ns_get_oa_ptr(s);
    let idx = nvim_ns_get_idx(s);

    // When 'keymodel' contains "stopsel" may stop Select/Visual mode
    if nvim_get_km_stopsel()
        && (crate::dispatch::table::rs_table_get_cmd_flags(idx) & NV_STS != 0)
        && (nvim_get_mod_mask() & MOD_MASK_SHIFT == 0)
    {
        rs_end_visual_mode();
        nvim_redraw_curbuf_inverted();
    }

    // Keys that work differently when 'keymodel' contains "startsel"
    if nvim_get_km_startsel() {
        let flags = crate::dispatch::table::rs_table_get_cmd_flags(idx);
        if flags & NV_SS != 0 {
            let mut mm = nvim_get_mod_mask();
            let new_cmdchar = rs_unshift_special(nvim_cap_get_cmdchar(ca), &raw mut mm);
            nvim_set_mod_mask(mm);
            nvim_cap_set_cmdchar(ca, new_cmdchar);
            let new_idx = rs_find_command(new_cmdchar);
            nvim_ns_set_idx(s, new_idx);
            if new_idx < 0 {
                // Just in case
                rs_clearopbeep(oa);
                return true;
            }
        } else if (flags & NV_SSS != 0) && (nvim_get_mod_mask() & MOD_MASK_SHIFT != 0) {
            nvim_mod_mask_clear_shift();
        }
    }
    false
}
