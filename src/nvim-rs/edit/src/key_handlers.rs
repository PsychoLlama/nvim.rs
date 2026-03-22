//! Insert-mode key handler functions migrated from edit.c
//!
//! These handle navigation keys (arrows, Home, End, Page Up/Down),
//! shift-arrow word movement, and control keys (Insert, Ctrl-O,
//! Ctrl-^, Ctrl-_, Ctrl-G, Ctrl-D/T shift, Delete).

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]
#![allow(non_snake_case)]

use std::ffi::{c_int, c_uint, c_void};

/// Column number type (matches `colnr_T` in Neovim).
type ColnrT = i32;

/// Line number type (matches `linenr_T` in Neovim).
type LinenrT = i32;

// ============================================================================
// C accessor functions
// ============================================================================

extern "C" {
    // -- Globals (get/set) --
    fn nvim_get_dont_sync_undo() -> c_int;
    fn nvim_set_dont_sync_undo(val: c_int);
    fn nvim_get_revins_scol() -> c_int;
    fn nvim_get_revins_legal() -> c_int;
    fn nvim_set_revins_legal(val: c_int);
    fn nvim_get_revins_chars() -> c_int;
    fn nvim_set_revins_chars(val: c_int);
    fn nvim_set_can_cindent(val: c_int);
    fn nvim_get_p_ri() -> c_int;

    // Cursor
    fn nvim_curwin_get_cursor_col() -> ColnrT;
    fn nvim_curwin_set_cursor_col(col: ColnrT);
    fn nvim_curwin_get_cursor_lnum() -> LinenrT;

    // Cursor shape / mode
    fn nvim_get_curwin() -> *mut c_void;

    // Phase 3 accessors (movement)
    fn nvim_set_curwin_cursor_coladd(val: ColnrT);
    fn nvim_curwin_set_w_set_curswant(val: bool);
    fn nvim_coladvance(col: ColnrT);
    fn virtual_active(wp: *mut c_void) -> bool;

    // -- Navigation helpers (all new Phase 4 accessors in edit.c) --
    fn nvim_edit_fdo_hor_and_key_typed() -> c_int;
    fn nvim_edit_save_cursor(slot: c_int);
    fn nvim_edit_start_arrow_from_slot(slot: c_int);
    fn nvim_edit_start_arrow_with_change_from_slot(slot: c_int, end_change: c_int);
    fn nvim_edit_start_arrow_curpos();
    fn AppendCharToRedobuff(c: c_int);
    fn vim_beep(val: c_uint);
    fn nvim_edit_ww_allows(ch: c_int) -> c_int;
    fn nvim_edit_set_cursor_lnum_rel(delta: LinenrT);

    // Movement functions (canonical names, exported from Rust)
    fn oneleft() -> c_int;
    fn oneright() -> c_int;
    fn undisplay_dollar();
    fn cursor_up(n: LinenrT, upd_topline: bool) -> c_int;
    fn cursor_down(n: c_int, upd_topline: bool) -> c_int;

    // Word movement
    fn nvim_bck_word(count: c_int, bigword: bool, stop: bool) -> c_int;
    fn nvim_fwd_word(count: c_int, bigword: bool, eol: c_int) -> c_int;

    // Character operations
    fn nvim_gchar_cursor() -> c_int;

    // Fold
    fn rs_foldOpenCursor();

    // Scrolling / tab pages
    fn nvim_edit_pagescroll_backward() -> c_int;
    fn nvim_edit_pagescroll_forward() -> c_int;
    fn nvim_edit_has_next_tabpage() -> c_int;
    fn nvim_goto_tabpage(n: c_int);

    // Up/Down with Insstart column
    fn nvim_edit_coladvance_insstart();
    fn nvim_edit_topline_changed() -> c_int;
    fn nvim_edit_redraw_later_valid();

    // -- Control key helpers (Phase 4b) --
    fn nvim_edit_ins_insert(replace_state: c_int);
    fn nvim_edit_ins_ctrl_o();
    fn nvim_edit_ins_ctrl_hat();
    fn nvim_edit_ins_ctrl_(revins_on: c_int);
    fn nvim_edit_ins_start_select(c: c_int) -> c_int;
    fn nvim_edit_ins_ctrl_g_get_key() -> c_int;
    fn nvim_edit_ins_shift(c: c_int, lastc: c_int);
    fn nvim_edit_ins_del();

    // State
    fn nvim_get_State() -> c_int;

    // p_ww (whichwrap) checks
    fn nvim_qf_curbuf_line_count() -> LinenrT;

    // utfc_ptr2len and cursor pos (u8 matches other modules; c_char in helpers.rs is ABI-compatible)
    #[allow(clashing_extern_declarations)]
    fn nvim_get_cursor_pos_ptr() -> *const u8;
    fn utfc_ptr2len(p: *const u8) -> c_int;
}

// ============================================================================
// Constants (verified against C headers with `_Static_assert` in `edit.c`)
// ============================================================================

/// `OK` from `vim_defs.h`
const OK: c_int = 1;

/// `kFalse` from `types_defs.h` — `TriState`
const K_FALSE: c_int = 0;

/// `kNone` from `types_defs.h` — `TriState`
const K_NONE: c_int = -1;

/// `MAXCOL` from `pos_defs.h`
const MAXCOL: ColnrT = 0x7fff_ffff;

// Key constants passed to AppendCharToRedobuff — values verified via _Static_assert
const K_LEFT: c_int = -27755;
const K_RIGHT: c_int = -29291;
const K_S_LEFT: c_int = -13347;
const K_S_RIGHT: c_int = -26917;

// Ctrl-G sub-commands
const CTRL_G_UP: c_int = 1;
const CTRL_G_DOWN: c_int = 2;
const CTRL_G_U_SYNC: c_int = 3;
const CTRL_G_NO_SYNC: c_int = 4;
const CTRL_G_ESC: c_int = 5;
const CTRL_G_UNKNOWN: c_int = 0;

// ============================================================================
// ins_left
// ============================================================================

/// Handle Left arrow in Insert mode.
unsafe fn ins_left_impl() {
    let end_change = nvim_get_dont_sync_undo() == K_FALSE;

    if nvim_edit_fdo_hor_and_key_typed() != 0 {
        rs_foldOpenCursor();
    }
    undisplay_dollar();
    nvim_edit_save_cursor(0); // save to slot 0
    if oneleft() == OK {
        nvim_edit_start_arrow_with_change_from_slot(0, c_int::from(end_change));
        if !end_change {
            AppendCharToRedobuff(K_LEFT);
        }
        // If exit reversed string, position is fixed
        if nvim_get_revins_scol() != -1
            && nvim_curwin_get_cursor_col() as c_int >= nvim_get_revins_scol()
        {
            nvim_set_revins_legal(nvim_get_revins_legal() + 1);
        }
        nvim_set_revins_chars(nvim_get_revins_chars() + 1);
    } else if nvim_edit_ww_allows(i32::from(b'[')) != 0 && nvim_curwin_get_cursor_lnum() > 1 {
        // if 'whichwrap' set for cursor in insert mode may go to previous line
        nvim_edit_start_arrow_from_slot(0);
        nvim_edit_set_cursor_lnum_rel(-1);
        nvim_coladvance(MAXCOL);
        nvim_curwin_set_w_set_curswant(true);
    } else {
        vim_beep(K_BO_FLAG_CURSOR as c_uint);
    }
    nvim_set_dont_sync_undo(K_FALSE);
}

#[unsafe(export_name = "ins_left")]
pub unsafe extern "C" fn rs_ins_left() {
    ins_left_impl();
}

// ============================================================================
// ins_right
// ============================================================================

/// Handle Right arrow in Insert mode.
unsafe fn ins_right_impl() {
    let end_change = nvim_get_dont_sync_undo() == K_FALSE;

    if nvim_edit_fdo_hor_and_key_typed() != 0 {
        rs_foldOpenCursor();
    }
    undisplay_dollar();
    if nvim_gchar_cursor() != 0 || virtual_active(nvim_get_curwin()) {
        nvim_edit_start_arrow_with_change_curpos(end_change);
        if !end_change {
            AppendCharToRedobuff(K_RIGHT);
        }
        nvim_curwin_set_w_set_curswant(true);
        if virtual_active(nvim_get_curwin()) {
            oneright();
        } else {
            let ptr = nvim_get_cursor_pos_ptr();
            let l = utfc_ptr2len(ptr);
            nvim_curwin_set_cursor_col(nvim_curwin_get_cursor_col() + l as ColnrT);
        }

        nvim_set_revins_legal(nvim_get_revins_legal() + 1);
        if nvim_get_revins_chars() != 0 {
            nvim_set_revins_chars(nvim_get_revins_chars() - 1);
        }
    } else if nvim_edit_ww_allows(i32::from(b']')) != 0
        && nvim_curwin_get_cursor_lnum() < nvim_qf_curbuf_line_count()
    {
        // if 'whichwrap' set for cursor in insert mode, may move to next line
        nvim_edit_start_arrow_curpos();
        nvim_curwin_set_w_set_curswant(true);
        nvim_edit_set_cursor_lnum_rel(1);
        nvim_curwin_set_cursor_col(0);
    } else {
        vim_beep(K_BO_FLAG_CURSOR as c_uint);
    }
    nvim_set_dont_sync_undo(K_FALSE);
}

#[unsafe(export_name = "ins_right")]
pub unsafe extern "C" fn rs_ins_right() {
    ins_right_impl();
}

// ============================================================================
// ins_s_left
// ============================================================================

/// Handle Shift-Left arrow in Insert mode (word left).
unsafe fn ins_s_left_impl() {
    let end_change = nvim_get_dont_sync_undo() == K_FALSE;

    if nvim_edit_fdo_hor_and_key_typed() != 0 {
        rs_foldOpenCursor();
    }
    undisplay_dollar();
    if nvim_curwin_get_cursor_lnum() > 1 || nvim_curwin_get_cursor_col() > 0 {
        nvim_edit_start_arrow_with_change_curpos(end_change);
        if !end_change {
            AppendCharToRedobuff(K_S_LEFT);
        }
        nvim_bck_word(1, false, false);
        nvim_curwin_set_w_set_curswant(true);
    } else {
        vim_beep(K_BO_FLAG_CURSOR as c_uint);
    }
    nvim_set_dont_sync_undo(K_FALSE);
}

#[unsafe(export_name = "ins_s_left")]
pub unsafe extern "C" fn rs_ins_s_left() {
    ins_s_left_impl();
}

// ============================================================================
// ins_s_right
// ============================================================================

/// Handle Shift-Right arrow in Insert mode (word right).
unsafe fn ins_s_right_impl() {
    let end_change = nvim_get_dont_sync_undo() == K_FALSE;

    if nvim_edit_fdo_hor_and_key_typed() != 0 {
        rs_foldOpenCursor();
    }
    undisplay_dollar();
    if nvim_curwin_get_cursor_lnum() < nvim_qf_curbuf_line_count() || nvim_gchar_cursor() != 0 {
        nvim_edit_start_arrow_with_change_curpos(end_change);
        if !end_change {
            AppendCharToRedobuff(K_S_RIGHT);
        }
        nvim_fwd_word(1, false, 0);
        nvim_curwin_set_w_set_curswant(true);
    } else {
        vim_beep(K_BO_FLAG_CURSOR as c_uint);
    }
    nvim_set_dont_sync_undo(K_FALSE);
}

#[unsafe(export_name = "ins_s_right")]
pub unsafe extern "C" fn rs_ins_s_right() {
    ins_s_right_impl();
}

// ============================================================================
// ins_home
// ============================================================================

/// Handle Home key in Insert mode.
unsafe fn ins_home_impl(c: c_int) {
    if nvim_edit_fdo_hor_and_key_typed() != 0 {
        rs_foldOpenCursor();
    }
    undisplay_dollar();
    nvim_edit_save_cursor(0);
    if c == K_C_HOME {
        nvim_edit_set_cursor_lnum_abs(1);
    }
    nvim_curwin_set_cursor_col(0);
    nvim_set_curwin_cursor_coladd(0);
    nvim_edit_set_w_curswant(0);
    nvim_edit_start_arrow_from_slot(0);
}

#[unsafe(export_name = "ins_home")]
pub unsafe extern "C" fn rs_ins_home(c: c_int) {
    ins_home_impl(c);
}

// ============================================================================
// ins_end
// ============================================================================

/// Handle End key in Insert mode.
unsafe fn ins_end_impl(c: c_int) {
    if nvim_edit_fdo_hor_and_key_typed() != 0 {
        rs_foldOpenCursor();
    }
    undisplay_dollar();
    nvim_edit_save_cursor(0);
    if c == K_C_END {
        nvim_edit_set_cursor_lnum_abs(nvim_qf_curbuf_line_count());
    }
    nvim_coladvance(MAXCOL);
    nvim_edit_set_w_curswant(MAXCOL);
    nvim_edit_start_arrow_from_slot(0);
}

#[unsafe(export_name = "ins_end")]
pub unsafe extern "C" fn rs_ins_end(c: c_int) {
    ins_end_impl(c);
}

// ============================================================================
// ins_up
// ============================================================================

/// Handle Up arrow in Insert mode.
unsafe fn ins_up_impl(startcol: bool) {
    nvim_edit_save_topline();
    undisplay_dollar();
    nvim_edit_save_cursor(0);
    if cursor_up(1, true) == OK {
        if startcol {
            nvim_edit_coladvance_insstart();
        }
        if nvim_edit_topline_changed() != 0 {
            nvim_edit_redraw_later_valid();
        }
        nvim_edit_start_arrow_from_slot(0);
        nvim_set_can_cindent(1);
    } else {
        vim_beep(K_BO_FLAG_CURSOR as c_uint);
    }
}

#[unsafe(export_name = "ins_up")]
pub unsafe extern "C" fn rs_ins_up(startcol: c_int) {
    ins_up_impl(startcol != 0);
}

// ============================================================================
// ins_down
// ============================================================================

/// Handle Down arrow in Insert mode.
unsafe fn ins_down_impl(startcol: bool) {
    nvim_edit_save_topline();
    undisplay_dollar();
    nvim_edit_save_cursor(0);
    if cursor_down(1, true) == OK {
        if startcol {
            nvim_edit_coladvance_insstart();
        }
        if nvim_edit_topline_changed() != 0 {
            nvim_edit_redraw_later_valid();
        }
        nvim_edit_start_arrow_from_slot(0);
        nvim_set_can_cindent(1);
    } else {
        vim_beep(K_BO_FLAG_CURSOR as c_uint);
    }
}

#[unsafe(export_name = "ins_down")]
pub unsafe extern "C" fn rs_ins_down(startcol: c_int) {
    ins_down_impl(startcol != 0);
}

// ============================================================================
// ins_pageup
// ============================================================================

/// Handle `PageUp` in Insert mode.
unsafe fn ins_pageup_impl() {
    undisplay_dollar();

    if nvim_edit_mod_mask_ctrl() != 0 {
        // <C-PageUp>: tab page back
        if nvim_edit_has_next_tabpage() != 0 {
            nvim_edit_start_arrow_curpos();
            nvim_goto_tabpage(-1);
        }
        return;
    }

    nvim_edit_save_cursor(0);
    if nvim_edit_pagescroll_backward() == OK {
        nvim_edit_start_arrow_from_slot(0);
        nvim_set_can_cindent(1);
    } else {
        vim_beep(K_BO_FLAG_CURSOR as c_uint);
    }
}

#[unsafe(export_name = "ins_pageup")]
pub unsafe extern "C" fn rs_ins_pageup() {
    ins_pageup_impl();
}

// ============================================================================
// ins_pagedown
// ============================================================================

/// Handle `PageDown` in Insert mode.
unsafe fn ins_pagedown_impl() {
    undisplay_dollar();

    if nvim_edit_mod_mask_ctrl() != 0 {
        // <C-PageDown>: tab page forward
        if nvim_edit_has_next_tabpage() != 0 {
            nvim_edit_start_arrow_curpos();
            nvim_goto_tabpage(0);
        }
        return;
    }

    nvim_edit_save_cursor(0);
    if nvim_edit_pagescroll_forward() == OK {
        nvim_edit_start_arrow_from_slot(0);
        nvim_set_can_cindent(1);
    } else {
        vim_beep(K_BO_FLAG_CURSOR as c_uint);
    }
}

#[unsafe(export_name = "ins_pagedown")]
pub unsafe extern "C" fn rs_ins_pagedown() {
    ins_pagedown_impl();
}

// ============================================================================
// ins_insert (Phase 4b - delegated to C helper)
// ============================================================================

#[unsafe(export_name = "ins_insert")]
pub unsafe extern "C" fn rs_ins_insert(replace_state: c_int) {
    nvim_edit_ins_insert(replace_state);
}

// ============================================================================
// ins_ctrl_o (Phase 4b - delegated to C helper)
// ============================================================================

#[unsafe(export_name = "ins_ctrl_o")]
pub unsafe extern "C" fn rs_ins_ctrl_o() {
    nvim_edit_ins_ctrl_o();
}

// ============================================================================
// ins_ctrl_hat (Phase 4b - delegated to C helper)
// ============================================================================

#[unsafe(export_name = "ins_ctrl_hat")]
pub unsafe extern "C" fn rs_ins_ctrl_hat() {
    nvim_edit_ins_ctrl_hat();
}

// ============================================================================
// ins_ctrl_ (Phase 4b)
// ============================================================================

/// Handle Ctrl-_ in Insert mode (toggle reverse insert).
unsafe fn ins_ctrl__impl() {
    let revins_on_val = nvim_get_p_ri() != 0 && nvim_get_State() == MODE_INSERT;
    nvim_edit_ins_ctrl_(c_int::from(revins_on_val));
}

#[unsafe(export_name = "ins_ctrl_")]
pub unsafe extern "C" fn rs_ins_ctrl_() {
    ins_ctrl__impl();
}

// ============================================================================
// ins_start_select (Phase 4b - delegated to C helper)
// ============================================================================

#[must_use]
#[unsafe(export_name = "ins_start_select")]
pub unsafe extern "C" fn rs_ins_start_select(c: c_int) -> c_int {
    nvim_edit_ins_start_select(c)
}

// ============================================================================
// ins_ctrl_g (Phase 4b)
// ============================================================================

/// Handle Ctrl-G in Insert mode.
unsafe fn ins_ctrl_g_impl() {
    let key = nvim_edit_ins_ctrl_g_get_key();
    match key {
        CTRL_G_UP => ins_up_impl(true),
        CTRL_G_DOWN => ins_down_impl(true),
        CTRL_G_U_SYNC => {
            nvim_edit_ctrl_g_u_sync();
        }
        CTRL_G_NO_SYNC => {
            nvim_set_dont_sync_undo(K_NONE);
        }
        CTRL_G_ESC | CTRL_G_UNKNOWN => {
            if key == CTRL_G_UNKNOWN {
                vim_beep(K_BO_FLAG_CTRLG as c_uint);
            }
        }
        _ => {}
    }
}

#[unsafe(export_name = "ins_ctrl_g")]
pub unsafe extern "C" fn rs_ins_ctrl_g() {
    ins_ctrl_g_impl();
}

// ============================================================================
// ins_shift (Phase 4b - delegated to C helper)
// ============================================================================

#[unsafe(export_name = "ins_shift")]
pub unsafe extern "C" fn rs_ins_shift(c: c_int, lastc: c_int) {
    nvim_edit_ins_shift(c, lastc);
}

// ============================================================================
// ins_del (Phase 4b - delegated to C helper)
// ============================================================================

#[unsafe(export_name = "ins_del")]
pub unsafe extern "C" fn rs_ins_del() {
    nvim_edit_ins_del();
}

// ============================================================================
// Additional extern declarations for helpers used above
// ============================================================================

extern "C" {
    fn nvim_edit_start_arrow_with_change_curpos(end_change: bool);
    fn nvim_edit_save_topline();
    fn nvim_edit_mod_mask_ctrl() -> c_int;
    fn nvim_edit_set_w_curswant(val: ColnrT);
    fn nvim_edit_set_cursor_lnum_abs(lnum: LinenrT);
    fn nvim_edit_ctrl_g_u_sync();
}

// ============================================================================
// Belloff flag constants (verified via `_Static_assert` in `edit.c`)
// ============================================================================

const K_BO_FLAG_CURSOR: c_int = 0x04;
const K_BO_FLAG_CTRLG: c_int = 0x20;

// Key constants for Home/End (verified via `_Static_assert` in `edit.c`)
const K_C_HOME: c_int = -22525;
const K_C_END: c_int = -22781;

// Mode constant
const MODE_INSERT: c_int = 0x10;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(OK, 1);
        assert_eq!(K_FALSE, 0);
        assert_eq!(K_NONE, -1);
        assert_eq!(MAXCOL, 0x7fff_ffff);
        assert_eq!(K_BO_FLAG_CURSOR, 0x04);
        assert_eq!(K_BO_FLAG_CTRLG, 0x20);
        assert_eq!(MODE_INSERT, 0x10);
    }

    #[test]
    fn test_key_constants() {
        // TERMCAP2KEY produces negative values — verify exact values
        assert_eq!(K_LEFT, -27755);
        assert_eq!(K_RIGHT, -29291);
        assert_eq!(K_S_LEFT, -13347);
        assert_eq!(K_S_RIGHT, -26917);
        assert_eq!(K_C_HOME, -22525);
        assert_eq!(K_C_END, -22781);
    }

    #[test]
    fn test_ctrl_g_sub_commands() {
        // Sub-command enum values are distinct
        let vals = [
            CTRL_G_UNKNOWN,
            CTRL_G_UP,
            CTRL_G_DOWN,
            CTRL_G_U_SYNC,
            CTRL_G_NO_SYNC,
            CTRL_G_ESC,
        ];
        for i in 0..vals.len() {
            for j in (i + 1)..vals.len() {
                assert_ne!(vals[i], vals[j]);
            }
        }
    }
}
