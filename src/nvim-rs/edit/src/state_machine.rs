//! `insert_check` and `insert_execute` — `VimState` callbacks for insert mode.
//!
//! Migrated from `edit.c`. These are the two callbacks assigned to
//! `InsertState.state.check` and `InsertState.state.execute`.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cognitive_complexity)]
#![allow(dead_code)]

use std::ffi::{c_int, c_uint, c_void};

use crate::dispatch::{InsertState, VimState};

// ============================================================================
// Type aliases
// ============================================================================

type LinenrT = i32;

// ============================================================================
// Constants
// ============================================================================

const OK: c_int = 1;
const NUL: c_int = 0;
const ESC: c_int = 0x1b;

// key constants
const K_IGNORE: c_int = -13821;
const K_NOP: c_int = -25085;
const K_EVENT: c_int = -26365;

// kOptFdoFlagInsert (verified via _Static_assert in edit.c)
const K_OPT_FDO_FLAG_INSERT: c_uint = 0x100;

// TriState values (matches C kFalse=0, kTrue=1, kNone=-1)
const K_FALSE: c_int = 0;
const K_TRUE: c_int = 1;
const K_NONE: c_int = -1; // kNone

// ============================================================================
// C accessor declarations
// ============================================================================

extern "C" {
    // --- revins state ---
    fn nvim_get_revins_legal() -> c_int;
    fn nvim_set_revins_legal(val: c_int);
    fn nvim_get_revins_scol() -> c_int;
    fn nvim_set_revins_scol(val: c_int);
    fn nvim_get_revins_on() -> c_int;

    // --- arrow_used ---
    fn nvim_get_arrow_used() -> c_int;

    // --- Insstart / Insstart_orig ---
    fn nvim_get_update_Insstart_orig() -> c_int;
    fn nvim_set_Insstart_orig_from_Insstart();

    // --- stop_insert_mode / terminal ---
    fn nvim_get_stop_insert_mode() -> c_int;
    fn nvim_set_stop_insert_mode(val: c_int);
    fn nvim_curbuf_is_terminal() -> c_int;
    fn stuffcharReadbuff(c: c_int);
    static mut restart_edit: c_int;

    // --- completion ---
    fn rs_ins_compl_active() -> c_int;
    fn rs_ins_compl_has_autocomplete() -> c_int;
    fn rs_ins_compl_enable_autocomplete();
    fn rs_ins_compl_init_get_longest();

    // --- curswant ---
    fn nvim_curwin_set_w_set_curswant(val: bool);

    // --- timestamps ---
    fn stuff_empty() -> bool;
    fn nvim_set_did_check_timestamps(val: bool);
    fn nvim_get_need_check_timestamps() -> bool;
    fn check_timestamps(focus: bool);

    // --- msg_scroll ---
    static mut msg_scroll: c_int;

    // --- fold ---
    fn nvim_get_fdo_flags() -> c_uint;
    fn rs_foldOpenCursor();
    fn rs_foldCheckClose();
    fn nvim_char_avail() -> c_int;

    // --- prompt buffer ---
    fn nvim_bt_prompt_curbuf() -> bool;
    fn nvim_edit_init_prompt_impl(cmdchar_todo: c_int);

    // --- scroll detection (composite accessor) ---
    /// Handles the scroll-up check block from `insert_check`.
    /// Returns the new mincol if scroll detected (modifies curwin), or -1 if no action.
    fn nvim_insert_check_scroll(
        mincol: c_int,
        old_topline: LinenrT,
        old_topfill: c_int,
        did_backspace: c_int,
        count: c_int,
    ) -> c_int;

    // --- topline update ---
    fn update_topline(wp: *mut c_void);
    fn nvim_get_curwin() -> *mut c_void;

    // --- validate cursor ---
    fn nvim_validate_cursor_curwin_wrapper();

    // --- ins_redraw (delegate to C composite to avoid same-crate FFI loop) ---
    fn nvim_edit_ins_redraw_impl(ready: c_int);

    // --- scroll bind ---
    fn nvim_curwin_get_p_scb() -> bool;
    fn do_check_scrollbind(flag: bool);
    fn nvim_curwin_get_w_p_crb() -> c_int;
    fn do_check_cursorbind();

    // --- curswant update ---
    fn update_curswant();

    // --- topline / topfill for saving ---
    fn nvim_curwin_get_topline() -> LinenrT;
    fn nvim_curwin_get_topfill() -> c_int;

    // --- dont_sync_undo ---
    fn nvim_get_dont_sync_undo() -> c_int;
    fn nvim_set_dont_sync_undo(val: c_int);

    // --- autocomplete trigger ---
    fn char_before_cursor() -> c_int;
    fn char_avail() -> bool;
    fn nvim_curwin_get_cursor_col() -> c_int;
    fn vim_isprintc(c: c_int) -> bool;

    // --- insert_handle_key / post ---
    fn insert_do_complete(s: *mut InsertState);
    fn insert_handle_key_post(s: *mut InsertState);

    // --- insert_execute helpers ---
    fn rs_ins_compl_bs() -> c_int;
    fn rs_ins_compl_used_match() -> c_int;
    fn rs_ins_compl_long_shown_match() -> c_int;
    fn rs_ins_compl_addfrommatch();
    fn rs_ins_compl_accept_char(c: c_int) -> c_int;
    fn do_insert_char_pre(c: c_int) -> *mut std::ffi::c_char;
    fn rs_ins_compl_addleader(c: c_int);
    fn rs_ins_compl_enter_selects() -> c_int;
    fn rs_ins_compl_preinsert_longest() -> c_int;
    fn rs_ins_compl_is_match_selected() -> c_int;
    fn rs_ins_compl_delete(do_bs: c_int);
    fn rs_ins_compl_insert(preinsert: c_int, new_leader: c_int);
    fn rs_ins_compl_preinsert_effect() -> c_int;
    fn rs_ascii_iswhite_nl_or_nul(c: c_int) -> c_int;
    fn rs_ins_compl_prep(c: c_int) -> c_int;
    fn rs_ins_compl_has_shown_match() -> c_int;
    fn rs_pum_wanted() -> c_int;
    fn stop_arrow() -> c_int;
    fn vungetc(c: c_int);
    fn ins_ctrl_o();
    fn do_digraph(c: c_int) -> c_int;
    fn rs_ctrl_x_mode_none() -> c_int;
    fn rs_ctrl_x_mode_cmdline() -> c_int;
    fn insert_handle_key(s: *mut InsertState) -> c_int;
    fn ins_ctrl_v();
    fn cindent_on() -> bool;
    fn in_cinkeys(c: c_int, ty: std::ffi::c_char, line_is_white: bool) -> c_int;
    fn do_c_expr_indent();
    fn inindent(n: c_int) -> c_int;
    fn ins_start_select(c: c_int) -> c_int;
    fn nvim_search_get_curwin_w_p_rl() -> c_int;

    fn xfree(p: *mut c_void);
}

// ============================================================================
// insert_check — VimState check callback
// ============================================================================

/// `insert_check` ported to Rust.
///
/// Called before each key is processed in insert mode.
/// Handles: revins, arrow, timestamps, fold, prompt, scroll, redraw, autocomplete.
///
/// # Safety
/// Called from C state machine only, with valid `InsertState` pointer.
#[unsafe(export_name = "insert_check_rs")]
pub unsafe extern "C" fn rs_insert_check(state: *mut VimState) -> c_int {
    let s = state.cast::<InsertState>();

    // Revins: reset on illegal motions
    if unsafe { nvim_get_revins_legal() } == 0 {
        unsafe { nvim_set_revins_scol(-1) };
    } else {
        unsafe { nvim_set_revins_legal(0) };
    }

    // Don't repeat insert when arrow key used
    if unsafe { nvim_get_arrow_used() } != 0 {
        unsafe { (*s).count = 0 };
    }

    // Update Insstart_orig if flag is set
    if unsafe { nvim_get_update_Insstart_orig() } != 0 {
        unsafe { nvim_set_Insstart_orig_from_Insstart() };
    }

    // Terminal buffer: exit insert mode and go to terminal mode
    if unsafe { nvim_curbuf_is_terminal() } != 0 && unsafe { nvim_get_stop_insert_mode() } == 0 {
        unsafe { nvim_set_stop_insert_mode(1) };
        unsafe { restart_edit = c_int::from(b'I') };
        unsafe { stuffcharReadbuff(K_NOP) };
    }

    // ":stopinsert" used
    if unsafe { nvim_get_stop_insert_mode() } != 0 && unsafe { rs_ins_compl_active() } == 0 {
        unsafe { (*s).count = 0 };
        return 0; // exit insert mode
    }

    // Set curwin->w_curswant for next K_DOWN or K_UP
    if unsafe { nvim_get_arrow_used() } == 0 {
        unsafe { nvim_curwin_set_w_set_curswant(true) };
    }

    // Check for timestamps when no typeahead
    if unsafe { stuff_empty() } {
        unsafe { nvim_set_did_check_timestamps(false) };
        if unsafe { nvim_get_need_check_timestamps() } {
            unsafe { check_timestamps(false) };
        }
    }

    // Clear msg_scroll (set by emsg())
    unsafe { msg_scroll = 0 };

    // Open fold at cursor line per 'foldopen'
    if unsafe { nvim_get_fdo_flags() } & K_OPT_FDO_FLAG_INSERT != 0 {
        unsafe { rs_foldOpenCursor() };
    }

    // Close folds where cursor isn't, per 'foldclose'
    if unsafe { nvim_char_avail() } == 0 {
        unsafe { rs_foldCheckClose() };
    }

    // Prompt buffer: ensure prompt text exists
    if unsafe { nvim_bt_prompt_curbuf() } {
        let cmdchar_todo = unsafe { (*s).cmdchar_todo };
        unsafe { nvim_edit_init_prompt_impl(cmdchar_todo) };
        unsafe { (*s).cmdchar_todo = NUL };
    }

    // Scroll check: may scroll window up one line
    {
        let mincol = unsafe { (*s).mincol };
        let old_topline = unsafe { (*s).old_topline };
        let old_topfill = unsafe { (*s).old_topfill };
        let did_backspace = c_int::from(unsafe { (*s).did_backspace });
        let count = unsafe { (*s).count };
        let new_mincol = unsafe {
            nvim_insert_check_scroll(mincol, old_topline, old_topfill, did_backspace, count)
        };
        if new_mincol >= 0 {
            unsafe { (*s).mincol = new_mincol };
        }
    }

    // Update topline
    if unsafe { (*s).count } <= 1 {
        unsafe { update_topline(nvim_get_curwin()) };
    }

    unsafe { (*s).did_backspace = false };

    if unsafe { (*s).count } <= 1 {
        unsafe { nvim_validate_cursor_curwin_wrapper() };
    }

    // Redraw when no chars waiting
    unsafe { nvim_edit_ins_redraw_impl(1) };

    if unsafe { nvim_curwin_get_p_scb() } {
        unsafe { do_check_scrollbind(true) };
    }
    if unsafe { nvim_curwin_get_w_p_crb() } != 0 {
        unsafe { do_check_cursorbind() };
    }

    if unsafe { (*s).count } <= 1 {
        unsafe { update_curswant() };
    }

    unsafe { (*s).old_topline = nvim_curwin_get_topline() };
    unsafe { (*s).old_topfill = nvim_curwin_get_topfill() };

    if unsafe { (*s).c } != K_EVENT {
        unsafe { (*s).lastc = (*s).c }; // remember previous char for CTRL-D
    }

    // After CTRL-G U: next cursor key won't break undo
    let dont_sync = unsafe { nvim_get_dont_sync_undo() };
    if dont_sync == K_NONE {
        // kNone -> kTrue
        unsafe { nvim_set_dont_sync_undo(K_TRUE) };
    } else {
        unsafe { nvim_set_dont_sync_undo(K_FALSE) };
    }

    // Trigger autocomplete on entering insert mode (before first char typed)
    if unsafe { (*s).ins_just_started } {
        unsafe { (*s).ins_just_started = false };
        if unsafe { rs_ins_compl_has_autocomplete() } != 0
            && !unsafe { char_avail() }
            && unsafe { nvim_curwin_get_cursor_col() } > 0
        {
            let c = unsafe { char_before_cursor() };
            if unsafe { vim_isprintc(c) } {
                unsafe { (*s).c = c };
                unsafe { rs_ins_compl_enable_autocomplete() };
                unsafe { rs_ins_compl_init_get_longest() };
                unsafe { insert_do_complete(s) };
                unsafe { insert_handle_key_post(s) };
                return 1;
            }
        }
    }

    1
}

// ============================================================================
// insert_execute — VimState execute callback
// ============================================================================

/// `insert_execute` ported to Rust.
///
/// Called for each key event in insert mode.
///
/// # Safety
/// Called from C state machine only.
#[unsafe(export_name = "insert_execute_rs")]
pub unsafe extern "C" fn rs_insert_execute(state: *mut VimState, key: c_int) -> c_int {
    let s = state.cast::<InsertState>();

    // Insert mode ended from callback
    if unsafe { nvim_get_stop_insert_mode() } != 0 {
        if key != K_IGNORE && key != K_NOP {
            unsafe { vungetc(key) };
        }
        unsafe { (*s).count = 0 };
        unsafe { (*s).nomove = true };
        unsafe { rs_ins_compl_prep(ESC) };
        return 0;
    }

    if key == K_IGNORE || key == K_NOP {
        return -1; // get another key
    }

    unsafe { (*s).c = key };

    // Don't want K_EVENT to clear cursorhold for second key (e.g. after CTRL-V)
    if key != K_EVENT {
        // did_cursorhold = true (set via accessor)
        nvim_edit_set_did_cursorhold_true();
    }

    // Special handling when popup menu is visible/wanted and cursor is in completed word
    if unsafe { rs_ins_compl_active() } != 0
        && unsafe { nvim_cursor_col_ge_compl_col() } != 0
        && unsafe { rs_ins_compl_has_shown_match() } != 0
        && unsafe { rs_pum_wanted() } != 0
    {
        let c = unsafe { (*s).c };

        // BS: delete one char from compl_leader
        if (c == K_BS || c == CTRL_H)
            && unsafe { nvim_curwin_get_cursor_col() } > unsafe { rs_ins_compl_col() }
        {
            let new_c = unsafe { rs_ins_compl_bs() };
            if new_c == NUL {
                unsafe { (*s).c = new_c };
                return 1; // continue
            }
            unsafe { (*s).c = new_c };
        }

        // When no match was selected or it was edited
        if unsafe { rs_ins_compl_used_match() } == 0 {
            let c = unsafe { (*s).c };
            // CTRL-L: add one char from current match to compl_leader
            if c == CTRL_L
                && (unsafe { rs_ctrl_x_mode_line_or_eval() } == 0
                    || unsafe { rs_ins_compl_long_shown_match() } != 0)
            {
                unsafe { rs_ins_compl_addfrommatch() };
                return 1;
            }

            // Non-white char that fits with current completion: add to compl_leader
            if unsafe { rs_ins_compl_accept_char(c) } != 0 {
                // Trigger InsertCharPre
                let str_ptr = unsafe { do_insert_char_pre(c) };
                if str_ptr.is_null() {
                    unsafe { rs_ins_compl_addleader(c) };
                } else {
                    // iterate through the string
                    let mut p = str_ptr;
                    while unsafe { *p != 0 } {
                        let ch = unsafe { nvim_utf_ptr2char(p) };
                        unsafe { rs_ins_compl_addleader(ch) };
                        // advance pointer (MB_PTR_ADV)
                        let len = unsafe { nvim_utf_ptr2len(p) } as usize;
                        if len == 0 {
                            break;
                        }
                        unsafe { p = p.add(len) };
                    }
                    unsafe { xfree(str_ptr.cast()) };
                }
                return 1;
            }

            // CTRL-Y or Enter when compl_enter_selects: select current match
            if (c == CTRL_Y
                || (unsafe { rs_ins_compl_enter_selects() } != 0
                    && (c == CAR || c == K_KENTER || c == NL)))
                && unsafe { stop_arrow() } == OK
            {
                unsafe { rs_ins_compl_delete(0) };
                if unsafe { rs_ins_compl_preinsert_longest() } != 0
                    && unsafe { rs_ins_compl_is_match_selected() } == 0
                {
                    unsafe { rs_ins_compl_insert(0, 1) };
                    unsafe { rs_ins_compl_init_get_longest() };
                    return 1;
                }
                unsafe { rs_ins_compl_insert(0, 0) };
            } else if unsafe { rs_ascii_iswhite_nl_or_nul(c) } != 0
                && unsafe { rs_ins_compl_preinsert_effect() } != 0
            {
                // Delete preinserted text when typing special chars
                unsafe { rs_ins_compl_delete(0) };
            }
        }
    }

    // Prepare for or stop CTRL-X mode
    unsafe { rs_ins_compl_init_get_longest() };
    if unsafe { rs_ins_compl_prep((*s).c) } != 0 {
        return 1;
    }

    // CTRL-\ CTRL-N/O/G: normal mode or CTRL-O without cursor move
    if unsafe { (*s).c } == CTRL_BSL {
        unsafe { nvim_edit_ins_redraw_impl(0) };
        let c2 = unsafe {
            no_mapping += 1;
            allow_keys += 1;
            let c = plain_vgetc();
            no_mapping -= 1;
            allow_keys -= 1;
            c
        };
        if c2 != CTRL_N && c2 != CTRL_G && c2 != CTRL_O {
            unsafe { vungetc(c2) };
            unsafe { (*s).c = CTRL_BSL };
        } else {
            if c2 == CTRL_O {
                unsafe { ins_ctrl_o() };
                unsafe { nvim_set_ins_at_eol(false) }; // cursor keeps column
                unsafe { (*s).nomove = true };
            }
            unsafe { (*s).count = 0 };
            return 0;
        }
    }

    if unsafe { (*s).c } != K_EVENT {
        unsafe { (*s).c = do_digraph((*s).c) };
    }

    // CTRL-V/Q in cmdline completion mode
    if (unsafe { (*s).c } == CTRL_V || unsafe { (*s).c } == CTRL_Q)
        && unsafe { rs_ctrl_x_mode_cmdline() } != 0
    {
        unsafe { insert_do_complete(s) };
        unsafe { insert_handle_key_post(s) };
        return 1;
    }

    if unsafe { (*s).c } == CTRL_V || unsafe { (*s).c } == CTRL_Q {
        unsafe { ins_ctrl_v() };
        unsafe { (*s).c = CTRL_V }; // pretend CTRL-V is last typed character
        return 1;
    }

    // Cindent key handling
    if unsafe { cindent_on() } && unsafe { rs_ctrl_x_mode_none() } != 0 {
        unsafe { (*s).line_is_white = inindent(0) != 0 };
        let line_is_white = unsafe { (*s).line_is_white };
        let c = unsafe { (*s).c };
        // '!' prefix: not to be inserted
        if unsafe { in_cinkeys(c, b'!' as std::ffi::c_char, line_is_white) } != 0
            && unsafe { stop_arrow() } == OK
        {
            unsafe { do_c_expr_indent() };
            return 1;
        }
        // '*' prefix: indent before inserting
        if unsafe { nvim_get_can_cindent() } != 0
            && unsafe { in_cinkeys(c, b'*' as std::ffi::c_char, line_is_white) } != 0
            && unsafe { stop_arrow() } == OK
        {
            unsafe { do_c_expr_indent() };
        }
    }

    // RTL key swap
    if unsafe { nvim_search_get_curwin_w_p_rl() } != 0 {
        let c = unsafe { (*s).c };
        unsafe {
            (*s).c = match c {
                K_LEFT => K_RIGHT,
                K_S_LEFT => K_S_RIGHT,
                K_C_LEFT => K_C_RIGHT,
                K_RIGHT => K_LEFT,
                K_S_RIGHT => K_S_LEFT,
                K_C_RIGHT => K_C_LEFT,
                _ => c,
            }
        };
    }

    // keymodel startsel handling
    if unsafe { ins_start_select((*s).c) } != 0 {
        return 1;
    }

    unsafe { insert_handle_key(s) }
}

// ============================================================================
// Helper C function wrappers (declared here to avoid polluting dispatch.rs)
// ============================================================================

extern "C" {
    fn nvim_get_can_cindent() -> c_int;
    fn rs_ins_compl_col() -> c_int;
    fn nvim_cursor_col_ge_compl_col() -> c_int;
    fn nvim_set_did_cursorhold(val: bool);
    fn nvim_set_ins_at_eol(val: bool);
    fn nvim_utf_ptr2char(p: *const std::ffi::c_char) -> c_int;
    fn nvim_utf_ptr2len(p: *const std::ffi::c_char) -> c_int;
    fn rs_ctrl_x_mode_line_or_eval() -> c_int;
    fn plain_vgetc() -> c_int;
    static mut no_mapping: c_int;
    static mut allow_keys: c_int;
}

// Key constants used in insert_execute
const K_BS: c_int = -25195;
const CTRL_H: c_int = 8;
const CTRL_L: c_int = 12;
const CTRL_N: c_int = 14;
const CTRL_O: c_int = 15;
const CTRL_Q: c_int = 17;
const CTRL_G: c_int = 7;
const CTRL_V: c_int = 22;
const CTRL_Y: c_int = 25;
const CTRL_BSL: c_int = 28;
const CAR: c_int = 0x0d;
const NL: c_int = 0x0a;
const K_KENTER: c_int = -16715;

// Arrow key constants for RTL swap
const K_LEFT: c_int = -27755;
const K_S_LEFT: c_int = -13347;
const K_C_LEFT: c_int = -22013;
const K_RIGHT: c_int = -29291;
const K_S_RIGHT: c_int = -26917;
const K_C_RIGHT: c_int = -22269;

/// Wrapper to avoid duplicate declaration of `nvim_edit_set_did_cursorhold`.
#[inline]
unsafe fn nvim_edit_set_did_cursorhold_true() {
    unsafe { nvim_set_did_cursorhold(true) };
}
