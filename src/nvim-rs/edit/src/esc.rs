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

use std::ffi::c_int;

// ============================================================================
// Constants
// ============================================================================

/// `MODE_NORMAL` from `state_defs.h`
const MODE_NORMAL: c_int = 0x01;

/// `REPLACE_FLAG` bit
const REPLACE_FLAG: c_int = 0x100;

/// `kOptVeFlagAll` (virtualedit=all) value
const K_OPT_VE_FLAG_ALL: c_int = 0x01;

// ============================================================================
// C accessors
// ============================================================================

extern "C" {
    // Spell redraw
    fn check_spell_redraw();

    // `RedrawingDisabled` counter
    fn nvim_edit_dec_redrawing_disabled();
    fn nvim_edit_inc_RedrawingDisabled();

    // Cursor
    fn nvim_curwin_get_cursor_col() -> c_int;
    fn nvim_curwin_get_cursor_coladd() -> c_int;
    fn nvim_edit_set_cursor_coladd(val: c_int);
    fn nvim_curwin_set_cursor_col(col: c_int);
    fn nvim_edit_curwin_clear_wcol_virtcol();
    fn nvim_edit_curwin_clear_wrow_wcol_virtcol();
    fn mb_adjust_cursor();

    // Arrow used
    fn nvim_get_arrow_used() -> c_int;

    // Redo buffer / interrupt
    fn nvim_edit_AppendToRedobuff(s: *const std::ffi::c_char);
    fn line_breakcheck();
    fn start_redo_ins() -> c_int;
    fn stuffRedoReadbuff(s: *const u8);

    // State
    fn nvim_get_State() -> c_int;
    fn nvim_set_State(val: c_int);
    fn nvim_edit_set_State(val: c_int);

    // `CPO_REPLCNT` / `p_cpo`
    fn nvim_edit_p_cpo_has_replcnt() -> c_int;

    // `ve_flags`
    fn nvim_edit_get_ve_flags_curwin() -> c_int;

    // `curswant`
    fn nvim_edit_set_w_set_curswant(val: c_int);

    // Last insert mark
    fn nvim_edit_set_b_last_insert_mark();

    // `cmod_flags` / `CMOD_KEEPJUMPS`
    fn nvim_edit_cmod_keepjumps() -> c_int;

    // `oneleft` (already in Rust with `c_int` return)
    fn oneleft() -> c_int;

    // `VIsual_active`
    fn nvim_VIsual_active() -> c_int;

    // `gchar_cursor`
    fn nvim_edit_gchar_cursor() -> c_int;

    // `buf_meta_total` curbuf inline
    fn nvim_edit_curbuf_meta_total_inline() -> c_int;

    // `revins_on`
    fn nvim_get_revins_on() -> c_int;

    // `may_trigger_modechanged`
    fn nvim_may_trigger_modechanged();

    // `setmouse` / `showmode` / `unshowmode`
    fn nvim_setmouse();
    fn nvim_showmode();
    fn nvim_edit_unshowmode_false();
    fn nvim_edit_get_p_smd() -> c_int;
    fn nvim_edit_skip_showmode() -> c_int;
    fn nvim_get_got_int() -> c_int;
    fn ui_cursor_shape();

    // `reg_recording`
    fn nvim_get_reg_recording() -> c_int;

    // `restart_edit`
    fn nvim_edit_get_restart_edit() -> c_int;

    // `p_ch == 0 && !ui_has(kUIMessages)`
    fn nvim_edit_get_p_ch_zero_no_ui_messages() -> c_int;

    // Autocmds
    fn nvim_edit_ins_apply_autocmds_insertleavepre();

    // `stop_insert(&curwin->w_cursor, true, nomove)`
    fn nvim_edit_stop_insert_curpos(nomove: c_int);

    // `undisplay_dollar`
    fn undisplay_dollar();
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
        nvim_edit_dec_redrawing_disabled();
        DISABLED_REDRAW = false;
    }
    if nvim_get_arrow_used() == 0 {
        // Don't append ESC for "r<CR>" and "grx".
        if cmdchar != c_int::from(b'r') && cmdchar != c_int::from(b'v') {
            nvim_edit_AppendToRedobuff(ESC_STR.as_ptr().cast());
        }

        // Repeating insert may take a long time; check for interrupt.
        if *count > 0 {
            line_breakcheck();
            if nvim_get_got_int() != 0 {
                *count = 0;
            }
        }

        *count -= 1;
        if *count > 0 {
            // Vi repeats insert without replacing characters.
            if nvim_edit_p_cpo_has_replcnt() != 0 {
                let state = nvim_get_State();
                nvim_set_State(state & !REPLACE_FLAG);
            }

            start_redo_ins();
            if cmdchar == c_int::from(b'r') || cmdchar == c_int::from(b'v') {
                stuffRedoReadbuff(ESC_STR.as_ptr());
            }
            nvim_edit_inc_RedrawingDisabled();
            DISABLED_REDRAW = true;
            return 0; // repeat the insert
        }

        nvim_edit_stop_insert_curpos(nomove);
        undisplay_dollar();
    }

    if cmdchar != c_int::from(b'r') && cmdchar != c_int::from(b'v') {
        nvim_edit_ins_apply_autocmds_insertleavepre();
    }

    // When an autoindent was removed, curswant stays after the indent
    if nvim_edit_get_restart_edit() == 0 && temp == nvim_curwin_get_cursor_col() {
        nvim_edit_set_w_set_curswant(1);
    }

    // Remember the last Insert position in the '^ mark.
    if nvim_edit_cmod_keepjumps() == 0 {
        nvim_edit_set_b_last_insert_mark();
    }

    // The cursor should end up on the last inserted character.
    if nomove == 0
        && (nvim_curwin_get_cursor_col() != 0 || nvim_curwin_get_cursor_coladd() > 0)
        && (nvim_edit_get_restart_edit() == 0
            || (nvim_edit_gchar_cursor() == 0 && nvim_VIsual_active() == 0))
        && nvim_get_revins_on() == 0
    {
        if nvim_curwin_get_cursor_coladd() > 0
            || nvim_edit_get_ve_flags_curwin() == K_OPT_VE_FLAG_ALL
        {
            oneleft();
            if nvim_edit_get_restart_edit() != 0 {
                nvim_edit_set_cursor_coladd(nvim_curwin_get_cursor_coladd() + 1);
            }
        } else {
            let col = nvim_curwin_get_cursor_col();
            nvim_curwin_set_cursor_col(col - 1);
            nvim_edit_curwin_clear_wcol_virtcol();
            mb_adjust_cursor();
        }
    }

    nvim_edit_set_State(MODE_NORMAL);
    nvim_may_trigger_modechanged();

    // Need to position cursor again when on a TAB and
    // when on a char with inline virtual text.
    if nvim_edit_gchar_cursor() == c_int::from(b'\t') || nvim_edit_curbuf_meta_total_inline() > 0 {
        nvim_edit_curwin_clear_wrow_wcol_virtcol();
    }

    nvim_setmouse();
    ui_cursor_shape();

    // When recording or for CTRL-O, need to display the new mode.
    // Otherwise remove the mode message.
    if nvim_get_reg_recording() != 0 || nvim_edit_get_restart_edit() != 0 {
        nvim_showmode();
    } else if nvim_edit_get_p_smd() != 0
        && (nvim_get_got_int() != 0 || nvim_edit_skip_showmode() == 0)
        && nvim_edit_get_p_ch_zero_no_ui_messages() == 0
    {
        nvim_edit_unshowmode_false();
    }

    1 // leaving insert mode
}
