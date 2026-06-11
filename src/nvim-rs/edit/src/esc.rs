//! `ins_esc` -- ESC handler for insert mode
//!
//! Ported from `edit.c` `ins_esc()`. Handles:
//! - Repeat insert (counting down with redo buffer)
//! - Calling `stop_insert`
//! - Cursor adjustment (back one char)
//! - Mode change to `MODE_NORMAL`
//! - `showmode`/`unshowmode` update

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::similar_names)]

use std::ffi::{c_int, c_uint, c_void};

// ============================================================================
// Constants
// ============================================================================

/// `MODE_NORMAL` from `state_defs.h`
const MODE_NORMAL: c_int = 0x01;

/// `REPLACE_FLAG` bit
const REPLACE_FLAG: c_int = 0x100;

/// `kOptVeFlagAll` (virtualedit=all) value from `option_vars.generated.h`.
const K_OPT_VE_FLAG_ALL: c_uint = 0x04;

// ============================================================================
// C accessors
// ============================================================================

extern "C" {
    static mut got_int: bool;
    static mut State: c_int;
    // Spell redraw
    fn check_spell_redraw();

    // `RedrawingDisabled` counter
    fn nvim_dec_RedrawingDisabled();
    fn nvim_inc_RedrawingDisabled();

    // Cursor
    fn nvim_curwin_get_cursor_col() -> c_int;
    fn nvim_curwin_get_cursor_coladd() -> c_int;
    fn nvim_set_curwin_cursor_coladd(val: c_int);
    fn nvim_curwin_set_cursor_col(col: c_int);
    fn mb_adjust_cursor();

    // Arrow used
    fn nvim_get_arrow_used() -> c_int;

    // Redo buffer / interrupt
    fn AppendToRedobuff(s: *const std::ffi::c_char);
    fn line_breakcheck();
    fn start_redo_ins() -> c_int;
    fn stuffRedoReadbuff(s: *const u8);

    // State

    // `CPO_REPLCNT` / `p_cpo`
    fn nvim_p_cpo_has_replcnt() -> bool;

    // `ve_flags`
    fn nvim_get_ve_flags() -> c_uint;

    // `curswant`
    fn nvim_curwin_set_w_set_curswant(val: bool);

    // Last insert mark
    fn nvim_set_b_last_insert_mark();

    // `cmod_flags` / `CMOD_KEEPJUMPS`
    fn nvim_cmod_keepjumps() -> bool;

    // `oneleft` (already in Rust with `c_int` return)
    fn oneleft() -> c_int;

    // `VIsual_active`
    fn nvim_VIsual_active() -> c_int;

    // `gchar_cursor`
    fn gchar_cursor() -> c_int;

    // `buf_meta_total` curbuf inline
    fn nvim_curbuf_meta_total_inline() -> c_int;

    // `revins_on`
    fn nvim_get_revins_on() -> c_int;

    // `may_trigger_modechanged`
    fn may_trigger_modechanged();

    // `setmouse` / `showmode` / `unshowmode`
    fn setmouse();
    fn showmode() -> c_int;
    fn nvim_unshowmode_false();
    // nvim_get_p_smd: inlined (Phase 39, use p_smd directly)
    #[link_name = "p_smd"]
    static p_smd: c_int;
    fn skip_showmode() -> bool;
    fn ui_cursor_shape();

    // `reg_recording`
    fn nvim_get_reg_recording() -> c_int;

    // `restart_edit`
    static mut restart_edit: c_int;

    // `p_ch == 0 && !ui_has(kUIMessages)`
    fn nvim_get_p_ch_zero_no_ui_messages() -> c_int;

    // Autocmds
    fn nvim_ins_apply_autocmds_insertleavepre();

    // curwin->w_cursor pointer (used by esc caller)
    fn nvim_curwin_get_cursor_ptr() -> *const c_void;

    // `undisplay_dollar`
    fn undisplay_dollar();

    // -- stop_insert dependencies (Phase 3 migration) --
    fn stop_redo_ins();
    fn rs_replace_stack_clear();

    // get_inserted: splits String into (data, size) to avoid FFI String layout
    fn nvim_stop_insert_get_inserted(data_out: *mut *mut u8, size_out: *mut usize);
    fn nvim_get_new_insert_skip() -> c_int;
    fn nvim_get_did_restart_edit() -> c_int;
    fn nvim_clear_last_insert();
    fn nvim_set_last_insert(data: *mut std::ffi::c_char, size: usize);
    fn nvim_set_last_insert_skip(val: c_int);
    fn xfree(ptr: *mut c_void);

    fn nvim_get_ins_need_undo() -> c_int;
    #[link_name = "has_format_option"]
    fn nvim_has_format_option(c: c_int) -> bool;
    fn dec_cursor() -> c_int;
    fn inc_cursor() -> c_int;
    fn auto_format(trailblank: bool, prev_line: bool);
    fn check_auto_format(end_insert: bool);
    fn nvim_get_did_ai() -> bool;
    fn nvim_p_cpo_has_indent() -> bool;
    fn nvim_stop_insert_pos_get_lnum(pos: *mut c_void) -> i32;
    fn nvim_stop_insert_pos_get_col(pos: *mut c_void) -> i32;
    fn nvim_curbuf_get_ml_line_count() -> i32;
    fn check_cursor_col(wp: *mut c_void);
    fn nvim_get_curwin() -> *mut c_void;
    fn nvim_curwin_get_cursor_lnum() -> i32;
    fn nvim_save_cursor_pos(lnum: *mut i32, col: *mut i32, coladd: *mut i32);
    fn nvim_restore_cursor_pos(lnum: i32, col: i32, coladd: i32);
    fn del_char(fixpos: c_int) -> c_int;
    fn nvim_gchar_pos_lnum_col_coladd(lnum: i32, col: i32, coladd: i32) -> c_int;
    fn nvim_get_VIsual_active() -> c_int;
    fn check_visual_pos();
    fn nvim_set_did_ai(val: bool);
    fn nvim_set_did_si(val: bool);
    fn nvim_set_can_si(val: bool);
    fn nvim_set_can_si_back(val: bool);
    fn nvim_stop_insert_set_b_op_start_insstart();
    fn nvim_stop_insert_set_b_op_start_orig_insstart_orig();
    fn nvim_stop_insert_set_b_op_end_from_pos(pos: *mut c_void);
}

// ============================================================================
// Static state
// ============================================================================

/// Tracks whether `RedrawingDisabled` was incremented (for the repeat path).
/// Matches `static bool disabled_redraw` in the C source.
static mut DISABLED_REDRAW: bool = false;

// ============================================================================
// Constants for strings
// ============================================================================

const ESC_STR: &[u8; 2] = b"\x1b\0";

// -- stop_insert constants --

/// `FO_AUTO` format option — automatic formatting (from `option_vars.h`)
const FO_AUTO: c_int = b'a' as c_int;

/// `FAIL` from `vim_defs.h`
const FAIL: c_int = 0;

// ============================================================================
// stop_insert — full Rust implementation (Phase 3 migration)
// ============================================================================

/// Returns true if `c` is an ASCII whitespace character.
#[inline]
fn ascii_iswhite(c: c_int) -> bool {
    c == c_int::from(b' ') || c == c_int::from(b'\t')
}

/// Stop insert mode: save inserted text, auto-format, strip trailing whitespace.
///
/// Ported from `nvim_edit_stop_insert` in `edit_shim.c`.
///
/// # Safety
/// Accesses global state via C accessor functions.
/// `end_insert_pos` must be NULL or a valid `pos_T*`.
#[allow(clippy::too_many_lines)]
#[unsafe(export_name = "nvim_edit_stop_insert")]
pub unsafe extern "C" fn rs_stop_insert(end_insert_pos: *mut c_void, esc: c_int, nomove: c_int) {
    stop_redo_ins();
    rs_replace_stack_clear();

    // Save inserted text for redo (^@ / CTRL-A).
    let mut data_ptr: *mut u8 = std::ptr::null_mut();
    let mut size: usize = 0;
    nvim_stop_insert_get_inserted(&raw mut data_ptr, &raw mut size);
    let added = if data_ptr.is_null() {
        0
    } else {
        size as c_int - nvim_get_new_insert_skip()
    };
    if nvim_get_did_restart_edit() == 0 || added > 0 {
        nvim_clear_last_insert();
        nvim_set_last_insert(data_ptr.cast::<std::ffi::c_char>(), size);
        nvim_set_last_insert_skip(if added < 0 {
            0
        } else {
            nvim_get_new_insert_skip()
        });
    } else {
        xfree(data_ptr.cast::<c_void>());
    }

    if nvim_get_arrow_used() == 0 && !end_insert_pos.is_null() {
        let pos = end_insert_pos;
        let mut cc: c_int;
        if nvim_get_ins_need_undo() == 0 && nvim_has_format_option(FO_AUTO) {
            // Save current cursor position.
            let mut tpos_lnum: i32 = 0;
            let mut tpos_col: i32 = 0;
            let mut tpos_coladd: i32 = 0;
            nvim_save_cursor_pos(&raw mut tpos_lnum, &raw mut tpos_col, &raw mut tpos_coladd);
            cc = c_int::from(b'x');
            if nvim_curwin_get_cursor_col() > 0 && gchar_cursor() == 0 {
                dec_cursor();
                cc = gchar_cursor();
                if !ascii_iswhite(cc) {
                    nvim_restore_cursor_pos(tpos_lnum, tpos_col, tpos_coladd);
                }
            }
            auto_format(true, false);
            if ascii_iswhite(cc) {
                if gchar_cursor() != 0 {
                    inc_cursor();
                }
                if gchar_cursor() == 0
                    && nvim_curwin_get_cursor_lnum() == tpos_lnum
                    && nvim_curwin_get_cursor_col() == tpos_col
                {
                    nvim_restore_cursor_pos(tpos_lnum, tpos_col, tpos_coladd);
                }
            }
        }
        check_auto_format(true);
        let pos_lnum = nvim_stop_insert_pos_get_lnum(pos);
        let pos_col = nvim_stop_insert_pos_get_col(pos);
        if nomove == 0
            && nvim_get_did_ai()
            && (esc != 0 || (!nvim_p_cpo_has_indent() && nvim_curwin_get_cursor_lnum() != pos_lnum))
            && pos_lnum <= nvim_curbuf_get_ml_line_count()
        {
            // Save current cursor.
            let mut tpos_lnum: i32 = 0;
            let mut tpos_col: i32 = 0;
            let mut tpos_coladd: i32 = 0;
            nvim_save_cursor_pos(&raw mut tpos_lnum, &raw mut tpos_col, &raw mut tpos_coladd);
            let prev_col = pos_col;
            // Move cursor to pos.
            nvim_restore_cursor_pos(pos_lnum, pos_col, 0);
            check_cursor_col(nvim_get_curwin());
            // Strip trailing whitespace.
            loop {
                if gchar_cursor() == 0 && nvim_curwin_get_cursor_col() > 0 {
                    nvim_curwin_set_cursor_col(nvim_curwin_get_cursor_col() - 1);
                }
                cc = gchar_cursor();
                if !ascii_iswhite(cc) {
                    break;
                }
                if del_char(1) == FAIL {
                    break;
                }
            }
            if nvim_curwin_get_cursor_lnum() != tpos_lnum {
                nvim_restore_cursor_pos(tpos_lnum, tpos_col, tpos_coladd);
            } else if nvim_curwin_get_cursor_col() < prev_col {
                let mut tpos2_lnum: i32 = 0;
                let mut tpos2_col: i32 = 0;
                let mut tpos2_coladd: i32 = 0;
                nvim_save_cursor_pos(
                    &raw mut tpos2_lnum,
                    &raw mut tpos2_col,
                    &raw mut tpos2_coladd,
                );
                let tpos2_col_plus1 = tpos2_col + 1;
                if cc != 0
                    && nvim_gchar_pos_lnum_col_coladd(tpos2_lnum, tpos2_col_plus1, tpos2_coladd)
                        == 0
                {
                    nvim_curwin_set_cursor_col(tpos2_col + 1);
                }
            }
            if nvim_get_VIsual_active() != 0 {
                check_visual_pos();
            }
        }
    }

    nvim_set_did_ai(false);
    nvim_set_did_si(false);
    nvim_set_can_si(false);
    nvim_set_can_si_back(false);
    if !end_insert_pos.is_null() {
        nvim_stop_insert_set_b_op_start_insstart();
        nvim_stop_insert_set_b_op_start_orig_insstart_orig();
        nvim_stop_insert_set_b_op_end_from_pos(end_insert_pos);
    }
}

// ============================================================================
// Implementation
// ============================================================================

/// Handle ESC in insert mode.
///
/// # Arguments
/// - `count` -- repeat count (decremented on each pass)
/// - `cmdchar` -- command that started the insert
/// - `nomove` -- when non-zero, don't move the cursor
///
/// # Returns
/// Non-zero when leaving insert mode, zero when repeating the insert.
///
/// # Safety
/// Accesses global state via C accessor functions.
/// `count` must be a valid non-null pointer.
#[unsafe(export_name = "ins_esc")]
pub unsafe extern "C" fn rs_ins_esc(count: *mut c_int, cmdchar: c_int, nomove: c_int) -> c_int {
    check_spell_redraw();

    let temp = nvim_curwin_get_cursor_col();
    if DISABLED_REDRAW {
        nvim_dec_RedrawingDisabled();
        DISABLED_REDRAW = false;
    }
    if nvim_get_arrow_used() == 0 {
        // Don't append ESC for "r<CR>" and "grx".
        if cmdchar != c_int::from(b'r') && cmdchar != c_int::from(b'v') {
            AppendToRedobuff(ESC_STR.as_ptr().cast());
        }

        // Repeating insert may take a long time; check for interrupt.
        if *count > 0 {
            line_breakcheck();
            if unsafe { got_int } {
                *count = 0;
            }
        }

        *count -= 1;
        if *count > 0 {
            // Vi repeats insert without replacing characters.
            if nvim_p_cpo_has_replcnt() {
                let state = State;
                State = state & !REPLACE_FLAG;
            }

            start_redo_ins();
            if cmdchar == c_int::from(b'r') || cmdchar == c_int::from(b'v') {
                stuffRedoReadbuff(ESC_STR.as_ptr());
            }
            nvim_inc_RedrawingDisabled();
            DISABLED_REDRAW = true;
            return 0; // repeat the insert
        }

        rs_stop_insert(nvim_curwin_get_cursor_ptr().cast_mut(), 1, nomove);
        undisplay_dollar();
    }

    if cmdchar != c_int::from(b'r') && cmdchar != c_int::from(b'v') {
        nvim_ins_apply_autocmds_insertleavepre();
    }

    // When an autoindent was removed, curswant stays after the indent
    if restart_edit == 0 && temp == nvim_curwin_get_cursor_col() {
        nvim_curwin_set_w_set_curswant(true);
    }

    // Remember the last Insert position in the '^ mark.
    if !nvim_cmod_keepjumps() {
        nvim_set_b_last_insert_mark();
    }

    // The cursor should end up on the last inserted character.
    if nomove == 0
        && (nvim_curwin_get_cursor_col() != 0 || nvim_curwin_get_cursor_coladd() > 0)
        && (restart_edit == 0 || (gchar_cursor() == 0 && nvim_VIsual_active() == 0))
        && nvim_get_revins_on() == 0
    {
        if nvim_curwin_get_cursor_coladd() > 0 || nvim_get_ve_flags() == K_OPT_VE_FLAG_ALL {
            oneleft();
            if restart_edit != 0 {
                nvim_set_curwin_cursor_coladd(nvim_curwin_get_cursor_coladd() + 1);
            }
        } else {
            let col = nvim_curwin_get_cursor_col();
            nvim_curwin_set_cursor_col(col - 1);
            {
                // VALID_WCOL = 0x02, VALID_VIRTCOL = 0x04 (from cursor_defs.h)
                let curwin = nvim_window::WinHandle::from_ptr(nvim_get_curwin());
                nvim_window::win_struct::win_mut(curwin).w_valid &= !(0x02 | 0x04);
            }
            mb_adjust_cursor();
        }
    }

    State = MODE_NORMAL;
    may_trigger_modechanged();

    // Need to position cursor again when on a TAB and
    // when on a char with inline virtual text.
    if gchar_cursor() == c_int::from(b'\t') || nvim_curbuf_meta_total_inline() > 0 {
        {
            // VALID_WROW = 0x01, VALID_WCOL = 0x02, VALID_VIRTCOL = 0x04
            let curwin = nvim_window::WinHandle::from_ptr(nvim_get_curwin());
            nvim_window::win_struct::win_mut(curwin).w_valid &= !(0x01 | 0x02 | 0x04);
        }
    }

    setmouse();
    ui_cursor_shape();

    // When recording or for CTRL-O, need to display the new mode.
    // Otherwise remove the mode message.
    if nvim_get_reg_recording() != 0 || restart_edit != 0 {
        showmode();
    } else if p_smd != 0
        && (unsafe { got_int } || !skip_showmode())
        && nvim_get_p_ch_zero_no_ui_messages() == 0
    {
        nvim_unshowmode_false();
    }

    1 // leaving insert mode
}
