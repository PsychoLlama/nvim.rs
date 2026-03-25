//! Normal mode command execution.
//!
//! This module provides the Rust implementation of `normal_execute()`
//! from `src/nvim/normal.c`. This is the per-key callback for normal mode:
//! it processes a key press, sets up the command, handles counts, dispatches
//! the command handler, and calls `normal_finish_command`.

use std::ffi::c_int;

use crate::dispatch::types::NormalStateHandle;
use crate::types::{NormalState, OpargT};
use crate::{CapHandle, OapHandle};

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
    fn nvim_get_cursor_lnum() -> i32;
    fn nvim_get_cursor_col() -> c_int;
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
    let sp = ns(s);
    let ca: CapHandle = (&raw mut (*sp).ca).cast();
    let oa: OapHandle = (&raw mut (*sp).oa).cast();

    (*sp).command_finished = false;
    (*sp).ctrl_w = false;
    (*sp).old_col = nvim_get_curswant();
    (*sp).c = key;

    // LANGMAP_ADJUST
    let adjusted = nvim_langmap_adjust((*sp).c, nvim_get_real_state() != MODE_SELECT);
    (*sp).c = adjusted;

    // If a mapping was started in Visual or Select mode, remember the length.
    if restart_edit == 0 {
        (*sp).old_mapped_len = 0;
    } else if (*sp).old_mapped_len != 0
        || (VIsual_active && (*sp).mapped_len == 0 && nvim_typebuf_maplen_wrapper() > 0)
    {
        (*sp).old_mapped_len = nvim_typebuf_maplen_wrapper();
    }

    if (*sp).c == NUL {
        (*sp).c = K_ZERO;
    }

    // In Select mode, typed text replaces the selection.
    let c = (*sp).c;
    if VIsual_active
        && nvim_get_VIsual_select()
        && (vim_isprintc(c) || c == NL || c == CAR || c == K_KENTER)
    {
        let len = ins_char_typebuf(nvim_get_vgetc_char(), nvim_get_vgetc_mod_mask(), true);

        if nvim_get_KeyTyped() {
            ungetchars(len);
        }

        if restart_edit != 0 {
            (*sp).c = c_int::from(b'd');
        } else {
            (*sp).c = c_int::from(b'c');
        }
        msg_nowait = true;
        (*sp).old_mapped_len = 0;
    }

    (*sp).need_flushbuf = nvim_add_to_showcmd_wrapper((*sp).c);

    while rs_normal_get_command_count(s) {}

    if (*sp).c == K_EVENT {
        // Save count values for K_EVENT re-entry.
        (*oa.cast::<OpargT>()).prev_opcount = nvim_cap_get_opcount(ca);
        (*oa.cast::<OpargT>()).prev_count0 = nvim_cap_get_count0(ca);
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
    if (*sp).toplevel && readbuf1_empty() {
        nvim_set_vcount_call(
            i64::from(nvim_cap_get_count0(ca)),
            i64::from(nvim_cap_get_count0(ca).max(1)),
            (*sp).set_prevcount,
        );
    }

    // Find the command character in the table of commands.
    if (*sp).ctrl_w {
        nvim_cap_set_nchar(ca, (*sp).c);
        nvim_cap_set_cmdchar(ca, CTRL_W);
    } else {
        nvim_cap_set_cmdchar(ca, (*sp).c);
    }

    let idx = rs_find_command(nvim_cap_get_cmdchar(ca));
    (*sp).idx = idx;

    'finish: {
        if idx < 0 {
            rs_clearopbeep(oa);
            (*sp).command_finished = true;
            break 'finish;
        }

        if (crate::dispatch::table::rs_table_get_cmd_flags(idx) & NV_NCW != 0)
            && rs_check_text_or_curbuf_locked(oa)
        {
            (*sp).command_finished = true;
            break 'finish;
        }

        // In Visual/Select mode, a few keys are handled in a special way.
        if VIsual_active && rs_normal_handle_special_visual_command(s) {
            (*sp).command_finished = true;
            break 'finish;
        }

        if nvim_get_curwin_w_p_rl()
            && nvim_get_KeyTyped()
            && nvim_get_keystuffed() == 0
            && (crate::dispatch::table::rs_table_get_cmd_flags((*sp).idx) & NV_RL != 0)
        {
            let new_cmdchar = rs_invert_horizontal(nvim_cap_get_cmdchar(ca));
            nvim_cap_set_cmdchar(ca, new_cmdchar);
            (*sp).idx = rs_find_command(new_cmdchar);
        }

        // Get an additional character if we need one.
        let pending_op = nvim_oap_get_op_type_ptr(oa) != OP_NOP;
        if rs_need_additional_char((*sp).idx, nvim_cap_get_cmdchar(ca), pending_op) {
            rs_normal_get_additional_char(s);
        }

        // Flush showcmd characters.
        if (*sp).need_flushbuf {
            nvim_ui_flush_wrapper();
        }

        if nvim_cap_get_cmdchar(ca) != K_IGNORE && nvim_cap_get_cmdchar(ca) != K_EVENT {
            nvim_set_did_cursorhold(false);
        }

        State = MODE_NORMAL;

        if nvim_cap_get_nchar(ca) == ESC || nvim_cap_get_extra_char(ca) == ESC {
            rs_clearop(oa);
            (*sp).command_finished = true;
            break 'finish;
        }

        if nvim_cap_get_cmdchar(ca) != K_IGNORE {
            msg_didout = false;
            msg_col = 0;
        }

        // remember where the cursor was
        (*sp).old_pos.lnum = nvim_get_cursor_lnum();
        (*sp).old_pos.col = nvim_get_cursor_col();

        // When 'keymodel' contains "startsel" some keys start Select/Visual mode.
        if !VIsual_active && nvim_get_km_startsel() {
            let cur_idx = (*sp).idx;
            let flags = crate::dispatch::table::rs_table_get_cmd_flags(cur_idx);
            if flags & NV_SS != 0 {
                rs_start_selection();
                let mut mm = nvim_get_mod_mask();
                let new_cmdchar = rs_unshift_special(nvim_cap_get_cmdchar(ca), &raw mut mm);
                nvim_set_mod_mask(mm);
                nvim_cap_set_cmdchar(ca, new_cmdchar);
                let new_idx = rs_find_command(new_cmdchar);
                debug_assert!(new_idx >= 0);
                (*sp).idx = new_idx;
            } else if (flags & NV_SSS != 0) && (nvim_get_mod_mask() & MOD_MASK_SHIFT != 0) {
                rs_start_selection();
                nvim_mod_mask_clear_shift();
            }
        }

        // Execute the command!
        nvim_execute_nv_cmd((*sp).idx, ca);
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

    let sp = ns(s);
    let ca: CapHandle = (&raw mut (*sp).ca).cast();
    let oa: OapHandle = (&raw mut (*sp).oa).cast();

    // Handle a count before a command and compute ca.count0.
    // Note that '0' is a command and not the start of a count, but it's
    // part of a count after other digits.
    while {
        let c = (*sp).c;
        (c >= c_int::from(b'1') && c <= c_int::from(b'9'))
            || (nvim_cap_get_count0(ca) != 0
                && (c == K_DEL || c == K_KDEL || c == c_int::from(b'0')))
    } {
        let c = (*sp).c;
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
        if (*sp).toplevel && readbuf1_empty() {
            let mut set_prevcount = (*sp).set_prevcount;
            rs_set_vcount_ca(ca, std::ptr::addr_of_mut!(set_prevcount));
            (*sp).set_prevcount = set_prevcount;
        }

        if (*sp).ctrl_w {
            nvim_inc_no_mapping();
            nvim_inc_allow_keys(); // no mapping for nchar, but keys
        }

        nvim_inc_no_zero_mapping(); // don't map zero here
        let new_c = nvim_plain_vgetc_wrapper();
        let adjusted = nvim_langmap_adjust(new_c, true);
        (*sp).c = adjusted;
        nvim_dec_no_zero_mapping();
        if (*sp).ctrl_w {
            nvim_dec_no_mapping();
            nvim_dec_allow_keys();
        }
        (*sp).need_flushbuf |= nvim_add_to_showcmd_wrapper((*sp).c);
    }

    // If we got CTRL-W there may be a/another count
    if (*sp).c == CTRL_W && !(*sp).ctrl_w && nvim_oap_get_op_type_ptr(oa) == OP_NOP {
        (*sp).ctrl_w = true;
        nvim_cap_set_opcount(ca, nvim_cap_get_count0(ca)); // remember first count
        nvim_cap_set_count0(ca, 0);
        nvim_inc_no_mapping();
        nvim_inc_allow_keys(); // no mapping for nchar, but keys
        let new_c = nvim_plain_vgetc_wrapper(); // get next character
        let adjusted = nvim_langmap_adjust(new_c, true);
        (*sp).c = adjusted;
        nvim_dec_no_mapping();
        nvim_dec_allow_keys();
        (*sp).need_flushbuf |= nvim_add_to_showcmd_wrapper((*sp).c);
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
    let sp = ns(s);
    let ca: CapHandle = (&raw mut (*sp).ca).cast();
    let oa: OapHandle = (&raw mut (*sp).oa).cast();
    let idx = (*sp).idx;

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
            (*sp).idx = new_idx;
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
