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
    static mut msg_silent: c_int;
    static mut State: c_int;
    // State
    fn nvim_get_did_restart_edit() -> c_int;
    fn nvim_set_did_restart_edit(val: c_int);
    static mut restart_edit: c_int;
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
    fn nvim_save_cursor_pos(lnum_out: *mut LinenrT, col_out: *mut ColnrT, coladd_out: *mut ColnrT);
    fn nvim_restore_cursor_pos(lnum: LinenrT, col: ColnrT, coladd: ColnrT);
    fn nvim_cursor_equals_saved(lnum: LinenrT, col: ColnrT, coladd: ColnrT) -> c_int;
    fn nvim_cursor_on_tab_or_inline() -> c_int;
    fn nvim_get_curwin() -> nvim_window::WinHandle;
    fn nvim_check_cursor_col_insert_mode();

    // Buffer
    fn nvim_get_curbuf_ml_line_count() -> LinenrT;
    fn nvim_get_curbuf_b_p_iminsert() -> c_int;

    // Insstart
    fn nvim_init_Insstart(startln: c_int);
    fn nvim_edit_init_Insstart_textlen();

    // ai_col
    fn nvim_get_ai_col() -> ColnrT;
    fn nvim_set_ai_col(val: ColnrT);
    fn nvim_get_did_ai() -> bool;

    // orig_line_count / vr_lines_changed
    fn nvim_get_orig_line_count() -> LinenrT;
    fn nvim_set_orig_line_count(val: LinenrT);
    fn nvim_set_vr_lines_changed(val: c_int);

    // Flags
    fn nvim_get_stop_insert_mode() -> c_int;
    fn nvim_set_stop_insert_mode(val: c_int);
    fn nvim_clear_where_paste_started();
    fn nvim_get_arrow_used() -> c_int;
    fn nvim_set_arrow_used(val: c_int);
    fn nvim_set_ins_at_eol(val: bool);
    fn nvim_get_o_lnum() -> LinenrT;
    fn nvim_set_o_lnum(val: LinenrT);
    fn nvim_set_need_start_insertmode(val: c_int);
    fn nvim_set_ins_need_undo(val: c_int);
    fn nvim_get_can_cindent() -> c_int;
    fn nvim_set_can_cindent(val: c_int);
    // nvim_get_p_smd: inlined (Phase 39, use p_smd directly)
    #[link_name = "p_smd"]
    static p_smd: c_int;
    fn nvim_set_old_indent(val: c_int);
    fn nvim_set_new_insert_skip(val: c_int);
    fn nvim_get_p_ri() -> c_int;
    fn nvim_get_need_highlight_changed() -> c_int;
    fn nvim_set_did_cursorhold(val: bool);

    // AutocmdS
    fn nvim_set_vv_insertmode(cmdchar: c_int);
    fn nvim_textfmt_clear_vv_char();
    fn nvim_ins_apply_insertenter();
    fn nvim_ins_apply_insertleave();
    fn nvim_vv_char_is_empty() -> c_int;
    fn highlight_changed();

    // Redo
    fn ResetRedobuff();
    fn AppendNumberToRedobuff(n: c_int);
    fn AppendCharToRedobuff(c: c_int);

    // Mode state machinery
    fn may_trigger_modechanged();
    fn setmouse();
    fn gchar_cursor() -> c_int;

    // Utilities
    fn msg_check_for_delay(check_msg_scroll: std::ffi::c_int);
    fn showmode() -> c_int;
    fn nvim_change_warning_col(col: c_int);
    fn pum_check_clear();
    fn nvim_state_enter(state: *mut c_void);
    fn ins_esc(count: *mut c_int, cmdchar: c_int, nomove: c_int) -> c_int;
    fn nvim_get_inserted_size() -> c_int;
    fn nvim_update_o_lnum_if_at_eol();
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
    nvim_set_did_restart_edit(restart_edit);

    // Sleep before redrawing, needed for "CTRL-O :" that results in an error message
    msg_check_for_delay(0);

    // Set Insstart_orig to Insstart
    nvim_set_update_Insstart_orig(1);

    rs_ins_compl_clear(); // clear stuff for CTRL-X mode

    // Trigger InsertEnter autocommands. Do not do this for "r<CR>" or "grx".
    let cmdchar = (*s).cmdchar;
    if cmdchar != c_int::from(b'r') && cmdchar != c_int::from(b'v') {
        let mut save_lnum: LinenrT = 0;
        let mut save_col: ColnrT = 0;
        let mut save_coladd: ColnrT = 0;
        nvim_save_cursor_pos(&raw mut save_lnum, &raw mut save_col, &raw mut save_coladd);

        nvim_set_vv_insertmode(cmdchar);
        nvim_textfmt_clear_vv_char();
        nvim_ins_apply_insertenter();

        // Check for changed highlighting, e.g. for ModeMsg.
        if nvim_get_need_highlight_changed() != 0 {
            highlight_changed();
        }

        // Make sure the cursor didn't move. Do call check_cursor_col() in
        // case the text was modified.
        if nvim_cursor_equals_saved(save_lnum, save_col, save_coladd) == 0
            && nvim_vv_char_is_empty() != 0
            && save_lnum <= nvim_get_curbuf_ml_line_count()
        {
            nvim_restore_cursor_pos(save_lnum, save_col, save_coladd);
            nvim_check_cursor_col_insert_mode();
        }
    }

    // When doing a paste with the middle mouse button, Insstart is set to
    // where the paste started.
    nvim_init_Insstart((*s).startln);

    nvim_edit_init_Insstart_textlen();

    if !nvim_get_did_ai() {
        nvim_set_ai_col(0);
    }

    // Set up redo buffer
    if cmdchar != 0 && restart_edit == 0 {
        ResetRedobuff();
        AppendNumberToRedobuff((*s).count);
        if cmdchar == c_int::from(b'V') || cmdchar == c_int::from(b'v') {
            // "gR" or "gr" command
            AppendCharToRedobuff(c_int::from(b'g'));
            AppendCharToRedobuff(if cmdchar == c_int::from(b'v') {
                c_int::from(b'r')
            } else {
                c_int::from(b'R')
            });
        } else {
            AppendCharToRedobuff(cmdchar);
            if cmdchar == c_int::from(b'g') {
                // "gI" command
                AppendCharToRedobuff(c_int::from(b'I'));
            } else if cmdchar == c_int::from(b'r') {
                // "r<CR>" command — insert only one <CR>
                (*s).count = 1;
            }
        }
    }

    // Set State
    if cmdchar == c_int::from(b'R') {
        State = MODE_REPLACE;
    } else if cmdchar == c_int::from(b'V') || cmdchar == c_int::from(b'v') {
        State = MODE_VREPLACE;
        (*s).replace_state = MODE_VREPLACE;
        nvim_set_orig_line_count(nvim_get_curbuf_ml_line_count());
        nvim_set_vr_lines_changed(1);
    } else {
        State = MODE_INSERT;
    }

    may_trigger_modechanged();
    nvim_set_stop_insert_mode(0);

    // Need to position cursor again when on a TAB and
    // when on a char with inline virtual text
    if nvim_cursor_on_tab_or_inline() != 0 {
        {
            // VALID_WROW = 0x01, VALID_WCOL = 0x02, VALID_VIRTCOL = 0x04
            nvim_window::win_struct::win_mut(nvim_get_curwin()).w_valid &= !(0x01 | 0x02 | 0x04);
        }
    }

    // Enable langmap or IME, indicated by 'iminsert'.
    if nvim_get_curbuf_b_p_iminsert() == B_IMODE_LMAP {
        State |= MODE_LANGMAP;
    }

    setmouse();
    rs_clear_showcmd();

    // There is no reverse replace mode
    let in_insert_mode = State == MODE_INSERT;
    let revins_on_val = in_insert_mode && nvim_get_p_ri() != 0;
    nvim_edit_set_revins_on(c_int::from(revins_on_val));
    if revins_on_val {
        undisplay_dollar();
    }
    nvim_set_revins_chars(0);
    nvim_set_revins_legal(0);
    nvim_set_revins_scol(-1);

    // Handle restarting Insert mode.
    handle_restart_edit_cursor_impl();

    // We are in insert mode now, don't need to start it anymore
    nvim_set_need_start_insertmode(0);

    // Need to save the line for undo before inserting the first char.
    nvim_set_ins_need_undo(1);

    nvim_clear_where_paste_started();
    nvim_set_can_cindent(1);

    // The cursor line is not in a closed fold, unless restarting.
    if nvim_get_did_restart_edit() == 0 {
        rs_foldOpenCursor();
    }

    // If 'showmode' is set, show the current (insert/replace/..) mode.
    // A warning message for changing a readonly file is given here, before
    // actually changing anything.
    let show_i = if p_smd != 0 && msg_silent == 0 {
        showmode()
    } else {
        0
    };
    (*s).i = show_i;

    if nvim_get_did_restart_edit() == 0 {
        nvim_change_warning_col(if show_i == 0 { 0 } else { show_i + 1 });
    }

    // nvim calls ui_cursor_shape() and do_digraph(-1) via C helpers not yet wrapped;
    // call them via the wrappers available in the dispatch module.
    nvim_ui_cursor_shape_and_clear_digraph();

    // Get the current length of the redo buffer.
    let insert_skip = nvim_get_inserted_size();
    nvim_set_new_insert_skip(insert_skip);

    nvim_set_old_indent(0);

    // Main insert event loop.
    // If count != 0, ins_esc will return false to repeat.
    loop {
        nvim_state_enter(std::ptr::addr_of_mut!((*s).state).cast::<c_void>());
        if ins_esc(&raw mut (*s).count, (*s).cmdchar, c_int::from((*s).nomove)) != 0 {
            break;
        }
    }

    // Always update o_lnum, so that a "CTRL-O ." that adds a line
    // still puts the cursor back after the inserted text.
    nvim_update_o_lnum_if_at_eol();

    pum_check_clear();

    rs_foldUpdateAfterInsert();

    // When CTRL-C was typed got_int will be set, with the result
    // that the autocommands won't be executed. When mapped got_int
    // is not set, but let's keep the behavior the same.
    if cmdchar != c_int::from(b'r') && cmdchar != c_int::from(b'v') && (*s).c != CTRL_C {
        nvim_ins_apply_insertleave();
    }
    nvim_set_did_cursorhold(false);

    // ins_redraw() triggers TextChangedI only when no characters are in the
    // typeahead buffer, so reset curbuf->b_last_changedtick if TextChangedI
    // was not blocked by char_avail() and has been triggered.
    nvim_curbuf_sync_changedtick_after_insert();
}

// ============================================================================
// Helper: ui_cursor_shape and do_digraph(-1) combined wrapper
// ============================================================================

extern "C" {
    fn nvim_ui_cursor_shape_and_clear_digraph();
}

// ============================================================================
// nvim_edit_handle_restart_edit_cursor -- Phase 2
// ============================================================================

extern "C" {
    // These are needed for handle_restart_edit_cursor
    fn nvim_stuff_empty() -> c_int;
    fn nvim_get_where_paste_started_lnum() -> LinenrT;
    fn nvim_validate_virtcol_curwin();
    fn nvim_update_curswant();
    fn nvim_get_ins_at_eol() -> bool;
    fn nvim_curwin_get_curswant() -> ColnrT;
    fn nvim_curwin_get_w_virtcol() -> ColnrT;
    fn nvim_get_cursor_line_ptr() -> *const std::ffi::c_char;
    fn nvim_utfc_ptr2len(p: *const std::ffi::c_char) -> c_int;
    fn nvim_curwin_set_cursor_col(col: ColnrT);
}

/// Handle cursor positioning when `restart_edit` is set at insert mode entry.
///
/// Returns 0 if `arrow_used` was set to false (no `restart_edit`),
///         1 if `restart_edit` was handled (`arrow_used` set from paste position).
///
/// # Safety
/// Accesses multiple global variables via C accessor functions.
unsafe fn handle_restart_edit_cursor_impl() -> c_int {
    if restart_edit != 0 && nvim_stuff_empty() != 0 {
        let paste_lnum = nvim_get_where_paste_started_lnum();
        nvim_set_arrow_used(c_int::from(paste_lnum == 0));
        restart_edit = 0;

        nvim_validate_virtcol_curwin();
        nvim_update_curswant();

        let cursor_lnum = nvim_curwin_get_cursor_lnum();
        let cursor_col = nvim_curwin_get_cursor_col();
        let ins_at_eol = nvim_get_ins_at_eol();
        let o_lnum = nvim_get_o_lnum();
        let curswant = nvim_curwin_get_curswant();
        let virtcol = nvim_curwin_get_w_virtcol();

        if (ins_at_eol && cursor_lnum == o_lnum) || curswant > virtcol {
            // Check character at current position
            let line_ptr = nvim_get_cursor_line_ptr();
            if !line_ptr.is_null() {
                let ptr = line_ptr.add(cursor_col as usize);
                if *ptr != 0 {
                    // Not at NUL
                    if *ptr.add(1) == 0 {
                        nvim_curwin_set_cursor_col(cursor_col + 1);
                    } else {
                        let char_len = nvim_utfc_ptr2len(ptr);
                        if *ptr.add(char_len as usize) == 0 {
                            nvim_curwin_set_cursor_col(cursor_col + char_len);
                        }
                    }
                }
            }
        }

        nvim_set_ins_at_eol(false);
        return 1;
    }

    nvim_set_arrow_used(0);
    0
}

/// Exported as `nvim_edit_handle_restart_edit_cursor` (replaces C function).
///
/// # Safety
/// Accesses global state via C accessor functions.
#[unsafe(export_name = "nvim_edit_handle_restart_edit_cursor")]
#[must_use]
pub unsafe extern "C" fn rs_handle_restart_edit_cursor() -> c_int {
    handle_restart_edit_cursor_impl()
}

// ============================================================================
// nvim_edit_init_prompt_impl -- Phase 3
// ============================================================================

extern "C" {
    // Phase 3: init_prompt_impl dependencies (not already declared above)
    #[link_name = "ml_replace"]
    fn nvim_ml_replace(lnum: LinenrT, line: *const std::ffi::c_char, copy: bool) -> c_int;
    #[link_name = "ml_append"]
    fn nvim_ml_append(
        lnum: LinenrT,
        line: *const std::ffi::c_char,
        len: c_int,
        newfile: bool,
    ) -> c_int;
    fn nvim_curbuf_get_b_prompt_start_lnum() -> LinenrT;
    fn nvim_curbuf_inc_prompt_start_lnum();
    fn nvim_curwin_set_cursor_lnum(lnum: LinenrT);
    fn nvim_curwin_set_cursor_lnum_to_line_count();
    fn nvim_coladvance(col: ColnrT);
    fn nvim_inserted_bytes_prompt(lnum: LinenrT, new_col: ColnrT);
    fn nvim_get_Insstart_orig_lnum() -> LinenrT;
    fn nvim_get_Insstart_orig_col() -> ColnrT;
    fn nvim_set_Insstart(lnum: LinenrT, col: ColnrT);
    fn nvim_set_Insstart_orig(lnum: LinenrT, col: ColnrT);
    fn nvim_set_Insstart_textlen(val: ColnrT);
    fn nvim_set_Insstart_blank_vcol(val: ColnrT);
    fn nvim_check_cursor_curwin();
}

/// Maximum column value (from `pos_defs.h`).
const MAXCOL: ColnrT = 0x7fff_ffff;

/// Compute byte length of a NUL-terminated C string.
const unsafe fn c_strlen_bytes(s: *const std::ffi::c_char) -> usize {
    let mut len = 0usize;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

/// Prepare the prompt buffer for insert mode.
///
/// Ensures the last line has prompt text and positions the cursor.
/// Called from the insert mode entry path when editing a prompt buffer.
///
/// # Safety
/// Accesses multiple global variables (curwin, curbuf, Insstart*) via C accessors.
unsafe fn init_prompt_impl(cmdchar_todo: c_int) {
    // prompt_text() is implemented in Rust (lib.rs).
    let prompt = crate::rs_prompt_text();
    if prompt.is_null() {
        return;
    }
    let prompt_len = c_strlen_bytes(prompt) as ColnrT;

    let prompt_start_lnum = nvim_curbuf_get_b_prompt_start_lnum();
    let cursor_lnum = nvim_curwin_get_cursor_lnum();

    // Ensure cursor is at or after prompt start line.
    if cursor_lnum < prompt_start_lnum {
        nvim_curwin_set_cursor_lnum(prompt_start_lnum);
    }

    let text = nvim_get_cursor_line_ptr();
    let cursor_lnum = nvim_curwin_get_cursor_lnum(); // re-read after potential update

    let need_insert = if prompt_start_lnum == cursor_lnum {
        // Prompt start is on cursor line: check if prompt text matches
        let text_slice = std::slice::from_raw_parts(text.cast::<u8>(), prompt_len as usize);
        let prompt_slice = std::slice::from_raw_parts(prompt.cast::<u8>(), prompt_len as usize);
        text_slice != prompt_slice
    } else {
        prompt_start_lnum > cursor_lnum
    };

    if need_insert {
        let line_count = nvim_get_curbuf_ml_line_count();
        if *text == 0 {
            // Empty line: replace it with the prompt
            nvim_ml_replace(line_count, prompt, true);
        } else {
            // Non-empty line: append a new line with the prompt
            nvim_ml_append(line_count, prompt, 0, false);
            nvim_curbuf_inc_prompt_start_lnum();
        }
        nvim_curwin_set_cursor_lnum_to_line_count();
        nvim_coladvance(MAXCOL);
        let new_line_count = nvim_get_curbuf_ml_line_count();
        nvim_inserted_bytes_prompt(new_line_count, prompt_len);
    }

    // Update Insstart / Insstart_orig if needed.
    let prompt_start_lnum = nvim_curbuf_get_b_prompt_start_lnum();
    if nvim_get_Insstart_orig_lnum() != prompt_start_lnum
        || nvim_get_Insstart_orig_col() != prompt_len
    {
        nvim_set_Insstart(prompt_start_lnum, prompt_len);
        nvim_set_Insstart_orig(prompt_start_lnum, prompt_len);
        nvim_set_Insstart_textlen(prompt_len);
        nvim_set_Insstart_blank_vcol(MAXCOL);
        nvim_set_arrow_used(0);
    }

    if cmdchar_todo == c_int::from(b'A') {
        nvim_coladvance(MAXCOL);
    }

    let cursor_lnum = nvim_curwin_get_cursor_lnum();
    let prompt_start_lnum = nvim_curbuf_get_b_prompt_start_lnum();
    if prompt_start_lnum == cursor_lnum {
        let cursor_col = nvim_curwin_get_cursor_col();
        let new_col = cursor_col.max(prompt_len);
        nvim_curwin_set_cursor_col(new_col);
    }

    nvim_check_cursor_curwin();
}

/// Exported as `nvim_edit_init_prompt_impl` (replaces C function).
///
/// # Safety
/// Accesses global state via C accessor functions.
#[unsafe(export_name = "nvim_edit_init_prompt_impl")]
pub unsafe extern "C" fn rs_init_prompt_impl(cmdchar_todo: c_int) {
    init_prompt_impl(cmdchar_todo);
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
