//! `insert_enter` — insert mode entry and main event loop
//!
//! Ported from `edit.c` `insert_enter()`. Handles:
//! - `InsertEnter` autocommands
//! - Mode state initialization (INSERT, REPLACE, VREPLACE)
//! - Redo buffer setup
//! - The `do { state_enter } while (!ins_esc)` event loop
//! - `InsertLeave` autocommands and cleanup

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cognitive_complexity)]
#![allow(dead_code)]

use std::ffi::{c_int, c_void};

use crate::dispatch::InsertState;

// ============================================================================
// Type aliases
// ============================================================================

type LinenrT = i32;
type ColnrT = i32;

// ============================================================================
// Mode constants (from state_defs.h)
// ============================================================================

const MODE_INSERT: c_int = 0x10;
const MODE_LANGMAP: c_int = 0x20;
const MODE_REPLACE: c_int = 0x110;
const MODE_VREPLACE: c_int = 0x310;
const B_IMODE_LMAP: c_int = 1;
const CTRL_C: c_int = 3;

// ============================================================================
// C accessor / helper functions
// ============================================================================

extern "C" {
    // State
    fn nvim_get_State() -> c_int;
    fn nvim_set_State(val: c_int);
    fn nvim_edit_set_State(val: c_int);
    fn nvim_get_did_restart_edit() -> c_int;
    fn nvim_set_did_restart_edit(val: c_int);
    fn nvim_edit_get_restart_edit() -> c_int;
    fn nvim_edit_set_restart_edit(val: c_int);
    fn nvim_get_update_Insstart_orig() -> c_int;
    fn nvim_set_update_Insstart_orig(val: c_int);

    // Revins
    fn nvim_get_revins_on() -> c_int;
    fn nvim_edit_set_revins_on(val: c_int);
    fn nvim_set_revins_chars(val: c_int);
    fn nvim_set_revins_legal(val: c_int);
    fn nvim_set_revins_scol(val: c_int);

    // Cursor
    fn nvim_curwin_get_cursor_lnum() -> LinenrT;
    fn nvim_curwin_get_cursor_col() -> ColnrT;
    fn nvim_edit_save_cursor_pos(
        lnum_out: *mut LinenrT,
        col_out: *mut ColnrT,
        coladd_out: *mut ColnrT,
    );
    fn nvim_edit_restore_cursor_pos(lnum: LinenrT, col: ColnrT, coladd: ColnrT);
    fn nvim_edit_cursor_equals_saved(lnum: LinenrT, col: ColnrT, coladd: ColnrT) -> c_int;
    fn nvim_edit_cursor_on_tab_or_inline() -> c_int;
    fn nvim_edit_invalidate_wrow_wcol_virtcol();
    fn nvim_edit_check_cursor_col_in_insert_mode();

    // Buffer
    fn nvim_edit_get_curbuf_ml_line_count() -> c_int;
    fn nvim_edit_get_curbuf_b_p_iminsert() -> c_int;

    // Insstart
    fn nvim_edit_init_Insstart(startln: c_int);
    fn nvim_edit_init_Insstart_textlen();

    // ai_col
    fn nvim_edit_get_ai_col() -> ColnrT;
    fn nvim_edit_set_ai_col(val: ColnrT);
    fn nvim_edit_get_did_ai() -> c_int;

    // orig_line_count / vr_lines_changed
    fn nvim_edit_get_orig_line_count() -> LinenrT;
    fn nvim_edit_set_orig_line_count(val: LinenrT);
    fn nvim_edit_set_vr_lines_changed(val: c_int);

    // Flags
    fn nvim_edit_get_stop_insert_mode() -> c_int;
    fn nvim_edit_set_stop_insert_mode(val: c_int);
    fn nvim_edit_clear_where_paste_started();
    fn nvim_get_arrow_used() -> c_int;
    fn nvim_set_arrow_used(val: c_int);
    fn nvim_edit_set_ins_at_eol(val: c_int);
    fn nvim_get_o_lnum() -> LinenrT;
    fn nvim_set_o_lnum(val: LinenrT);
    fn nvim_edit_set_need_start_insertmode(val: c_int);
    fn nvim_set_ins_need_undo(val: c_int);
    fn nvim_get_can_cindent() -> c_int;
    fn nvim_set_can_cindent(val: c_int);
    fn nvim_edit_get_p_smd() -> c_int;
    fn nvim_edit_get_msg_silent() -> c_int;
    fn nvim_edit_set_old_indent(val: c_int);
    fn nvim_set_new_insert_skip(val: c_int);
    fn nvim_get_p_ri() -> c_int;
    fn nvim_edit_get_need_highlight_changed() -> c_int;
    fn nvim_edit_set_did_cursorhold(val: c_int);

    // AutocmdS
    fn nvim_edit_set_vv_insertmode(cmdchar: c_int);
    fn nvim_edit_clear_vv_char();
    fn nvim_edit_ins_apply_insertenter();
    fn nvim_edit_ins_apply_insertleave();
    fn nvim_edit_vv_char_is_empty() -> c_int;
    fn nvim_edit_highlight_changed();

    // Redo
    fn nvim_edit_ResetRedobuff();
    fn nvim_edit_AppendNumberToRedobuff(n: c_int);
    fn nvim_edit_append_char_to_redobuff(c: c_int);

    // Mode state machinery
    fn nvim_may_trigger_modechanged();
    fn nvim_setmouse();
    fn nvim_edit_gchar_cursor() -> c_int;

    // Utilities
    fn nvim_edit_msg_check_for_delay();
    fn nvim_edit_showmode() -> c_int;
    fn nvim_edit_change_warning(col: c_int);
    fn nvim_edit_pum_check_clear();
    fn nvim_edit_state_enter(state: *mut c_void);
    fn nvim_edit_ins_esc(count: *mut c_int, cmdchar: c_int, nomove: c_int) -> c_int;
    fn nvim_edit_get_inserted_size() -> c_int;
    fn nvim_edit_handle_restart_edit_cursor() -> c_int;
    fn nvim_edit_update_o_lnum_if_at_eol();
    fn nvim_curbuf_sync_changedtick_after_insert();

    // Rust fold FFI (already defined in Rust)
    fn rs_ins_compl_clear();
    fn rs_foldOpenCursor();
    fn rs_foldUpdateAfterInsert();
    fn rs_clear_showcmd();
    fn undisplay_dollar();
}

// ============================================================================
// Implementation
// ============================================================================

/// Insert mode entry function.
///
/// Initializes mode state, fires `InsertEnter` autocmds, runs the main insert
/// event loop via `state_enter`/`ins_esc`, and fires `InsertLeave` on exit.
///
/// # Safety
/// Accesses many global variables via C accessor functions.
#[unsafe(export_name = "insert_enter")]
pub unsafe extern "C" fn rs_insert_enter(s: *mut InsertState) {
    (*s).did_backspace = true;
    (*s).old_topfill = -1;
    (*s).replace_state = MODE_REPLACE;
    (*s).cmdchar_todo = (*s).cmdchar;
    (*s).ins_just_started = true;

    // Remember whether editing was restarted after CTRL-O
    nvim_set_did_restart_edit(nvim_edit_get_restart_edit());

    // Sleep before redrawing, needed for "CTRL-O :" that results in an error message
    nvim_edit_msg_check_for_delay();

    // Set Insstart_orig to Insstart
    nvim_set_update_Insstart_orig(1);

    rs_ins_compl_clear(); // clear stuff for CTRL-X mode

    // Trigger InsertEnter autocommands. Do not do this for "r<CR>" or "grx".
    let cmdchar = (*s).cmdchar;
    if cmdchar != c_int::from(b'r') && cmdchar != c_int::from(b'v') {
        let mut save_lnum: LinenrT = 0;
        let mut save_col: ColnrT = 0;
        let mut save_coladd: ColnrT = 0;
        nvim_edit_save_cursor_pos(&raw mut save_lnum, &raw mut save_col, &raw mut save_coladd);

        nvim_edit_set_vv_insertmode(cmdchar);
        nvim_edit_clear_vv_char();
        nvim_edit_ins_apply_insertenter();

        // Check for changed highlighting, e.g. for ModeMsg.
        if nvim_edit_get_need_highlight_changed() != 0 {
            nvim_edit_highlight_changed();
        }

        // Make sure the cursor didn't move. Do call check_cursor_col() in
        // case the text was modified.
        if nvim_edit_cursor_equals_saved(save_lnum, save_col, save_coladd) == 0
            && nvim_edit_vv_char_is_empty() != 0
            && save_lnum <= nvim_edit_get_curbuf_ml_line_count()
        {
            nvim_edit_restore_cursor_pos(save_lnum, save_col, save_coladd);
            nvim_edit_check_cursor_col_in_insert_mode();
        }
    }

    // When doing a paste with the middle mouse button, Insstart is set to
    // where the paste started.
    nvim_edit_init_Insstart((*s).startln);

    nvim_edit_init_Insstart_textlen();

    if nvim_edit_get_did_ai() == 0 {
        nvim_edit_set_ai_col(0);
    }

    // Set up redo buffer
    if cmdchar != 0 && nvim_edit_get_restart_edit() == 0 {
        nvim_edit_ResetRedobuff();
        nvim_edit_AppendNumberToRedobuff((*s).count);
        if cmdchar == c_int::from(b'V') || cmdchar == c_int::from(b'v') {
            // "gR" or "gr" command
            nvim_edit_append_char_to_redobuff(c_int::from(b'g'));
            nvim_edit_append_char_to_redobuff(if cmdchar == c_int::from(b'v') {
                c_int::from(b'r')
            } else {
                c_int::from(b'R')
            });
        } else {
            nvim_edit_append_char_to_redobuff(cmdchar);
            if cmdchar == c_int::from(b'g') {
                // "gI" command
                nvim_edit_append_char_to_redobuff(c_int::from(b'I'));
            } else if cmdchar == c_int::from(b'r') {
                // "r<CR>" command — insert only one <CR>
                (*s).count = 1;
            }
        }
    }

    // Set State
    if cmdchar == c_int::from(b'R') {
        nvim_set_State(MODE_REPLACE);
    } else if cmdchar == c_int::from(b'V') || cmdchar == c_int::from(b'v') {
        nvim_set_State(MODE_VREPLACE);
        (*s).replace_state = MODE_VREPLACE;
        nvim_edit_set_orig_line_count(nvim_edit_get_curbuf_ml_line_count());
        nvim_edit_set_vr_lines_changed(1);
    } else {
        nvim_set_State(MODE_INSERT);
    }

    nvim_may_trigger_modechanged();
    nvim_edit_set_stop_insert_mode(0);

    // Need to position cursor again when on a TAB and
    // when on a char with inline virtual text
    if nvim_edit_cursor_on_tab_or_inline() != 0 {
        nvim_edit_invalidate_wrow_wcol_virtcol();
    }

    // Enable langmap or IME, indicated by 'iminsert'.
    if nvim_edit_get_curbuf_b_p_iminsert() == B_IMODE_LMAP {
        nvim_set_State(nvim_get_State() | MODE_LANGMAP);
    }

    nvim_setmouse();
    rs_clear_showcmd();

    // There is no reverse replace mode
    let in_insert_mode = nvim_get_State() == MODE_INSERT;
    let revins_on_val = in_insert_mode && nvim_get_p_ri() != 0;
    nvim_edit_set_revins_on(c_int::from(revins_on_val));
    if revins_on_val {
        undisplay_dollar();
    }
    nvim_set_revins_chars(0);
    nvim_set_revins_legal(0);
    nvim_set_revins_scol(-1);

    // Handle restarting Insert mode.
    nvim_edit_handle_restart_edit_cursor();

    // We are in insert mode now, don't need to start it anymore
    nvim_edit_set_need_start_insertmode(0);

    // Need to save the line for undo before inserting the first char.
    nvim_set_ins_need_undo(1);

    nvim_edit_clear_where_paste_started();
    nvim_set_can_cindent(1);

    // The cursor line is not in a closed fold, unless restarting.
    if nvim_get_did_restart_edit() == 0 {
        rs_foldOpenCursor();
    }

    // If 'showmode' is set, show the current (insert/replace/..) mode.
    // A warning message for changing a readonly file is given here, before
    // actually changing anything.
    let show_i = if nvim_edit_get_p_smd() != 0 && nvim_edit_get_msg_silent() == 0 {
        nvim_edit_showmode()
    } else {
        0
    };
    (*s).i = show_i;

    if nvim_get_did_restart_edit() == 0 {
        nvim_edit_change_warning(if show_i == 0 { 0 } else { show_i + 1 });
    }

    // nvim calls ui_cursor_shape() and do_digraph(-1) via C helpers not yet wrapped;
    // call them via the wrappers available in the dispatch module.
    nvim_edit_ui_cursor_shape_and_clear_digraph();

    // Get the current length of the redo buffer.
    let insert_skip = nvim_edit_get_inserted_size();
    nvim_set_new_insert_skip(insert_skip);

    nvim_edit_set_old_indent(0);

    // Main insert event loop.
    // If count != 0, ins_esc will return false to repeat.
    loop {
        nvim_edit_state_enter(std::ptr::addr_of_mut!((*s).state).cast::<c_void>());
        if nvim_edit_ins_esc(&raw mut (*s).count, (*s).cmdchar, c_int::from((*s).nomove)) != 0 {
            break;
        }
    }

    // Always update o_lnum, so that a "CTRL-O ." that adds a line
    // still puts the cursor back after the inserted text.
    nvim_edit_update_o_lnum_if_at_eol();

    nvim_edit_pum_check_clear();

    rs_foldUpdateAfterInsert();

    // When CTRL-C was typed got_int will be set, with the result
    // that the autocommands won't be executed. When mapped got_int
    // is not set, but let's keep the behavior the same.
    if cmdchar != c_int::from(b'r') && cmdchar != c_int::from(b'v') && (*s).c != CTRL_C {
        nvim_edit_ins_apply_insertleave();
    }
    nvim_edit_set_did_cursorhold(0);

    // ins_redraw() triggers TextChangedI only when no characters are in the
    // typeahead buffer, so reset curbuf->b_last_changedtick if TextChangedI
    // was not blocked by char_avail() and has been triggered.
    nvim_curbuf_sync_changedtick_after_insert();
}

// ============================================================================
// Helper: ui_cursor_shape and do_digraph(-1) combined wrapper
// ============================================================================

extern "C" {
    fn nvim_edit_ui_cursor_shape_and_clear_digraph();
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_constants() {
        assert_eq!(MODE_INSERT, 0x10);
        assert_eq!(MODE_LANGMAP, 0x20);
        assert_eq!(MODE_REPLACE, 0x110);
        assert_eq!(MODE_VREPLACE, 0x310);
        assert_eq!(B_IMODE_LMAP, 1);
        assert_eq!(CTRL_C, 3);
    }
}
