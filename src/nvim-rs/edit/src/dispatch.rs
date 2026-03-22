//! Insert mode key dispatch — the main switch statement.
//!
//! Migrated from `edit.c`:
//! - `insert_handle_key` — the big switch dispatching insert-mode keys
//! - `insert_do_complete` — completion trigger helper
//! - `insert_do_cindent` — cindent helper
//! - `insert_handle_key_post` — post-key-handling cleanup

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cognitive_complexity)]
#![allow(dead_code)] // Many constants are for documentation purposes
#![allow(clippy::wildcard_imports)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_int, c_void};

/// Line number type (matches `linenr_T`)
type LinenrT = i32;

// ============================================================================
// InsertState struct — must exactly match the C layout in edit.c
// ============================================================================

/// Opaque VimState: only carries function pointers, never accessed from Rust.
#[repr(C)]
pub struct VimState {
    execute: Option<unsafe extern "C" fn(*mut VimState, c_int) -> c_int>,
    check: Option<unsafe extern "C" fn(*mut VimState) -> c_int>,
}

/// Insert-mode state (must match C `InsertState` layout exactly).
/// Fields are accessed via pointer; the `state` and `ca` fields are opaque.
#[repr(C)]
pub struct InsertState {
    pub state: VimState,
    pub ca: *mut c_void, // cmdarg_T*, unused here
    pub mincol: c_int,
    pub cmdchar: c_int,
    pub cmdchar_todo: c_int,
    pub ins_just_started: bool,
    pub startln: c_int,
    pub count: c_int,
    pub c: c_int,
    pub lastc: c_int,
    pub i: c_int,
    pub did_backspace: bool,
    pub line_is_white: bool,
    pub old_topline: LinenrT,
    pub old_topfill: c_int,
    pub inserted_space: c_int,
    pub replace_state: c_int,
    pub did_restart_edit: c_int,
    pub nomove: bool,
}

// ============================================================================
// C accessor and helper function declarations
// ============================================================================

extern "C" {
    // -- Completion mode checks (already-migrated Rust symbols) --
    fn rs_ctrl_x_mode_none() -> c_int;
    fn rs_ctrl_x_mode_normal() -> c_int;
    fn rs_ctrl_x_mode_scroll() -> c_int;
    fn rs_ctrl_x_mode_whole_line() -> c_int;
    fn rs_ctrl_x_mode_files() -> c_int;
    fn rs_ctrl_x_mode_tags() -> c_int;
    fn rs_ctrl_x_mode_path_patterns() -> c_int;
    fn rs_ctrl_x_mode_path_defines() -> c_int;
    fn rs_ctrl_x_mode_thesaurus() -> c_int;
    fn rs_ctrl_x_mode_dictionary() -> c_int;
    fn rs_ctrl_x_mode_cmdline() -> c_int;
    fn rs_ctrl_x_mode_function() -> c_int;
    fn rs_ctrl_x_mode_omni() -> c_int;
    fn rs_ctrl_x_mode_spell() -> c_int;
    fn rs_ctrl_x_mode_line_or_eval() -> c_int;
    fn rs_ctrl_x_mode_register() -> c_int;

    // -- Completion state (already-migrated Rust symbols) --
    fn rs_ins_compl_active() -> c_int;
    fn rs_ins_compl_win_active(wp: *mut c_void) -> c_int;
    fn rs_ins_compl_cancel();
    fn rs_ins_compl_prep(c: c_int) -> c_int;
    fn rs_ins_compl_has_autocomplete() -> c_int;
    fn rs_ins_compl_enable_autocomplete();
    fn rs_ins_compl_init_get_longest();
    fn rs_compl_status_clear();
    fn rs_ins_compl_col() -> c_int;

    // -- Key handlers (canonical names, exported from Rust) --
    fn ins_insert(replace_state: c_int);
    fn ins_ctrl_o();
    fn ins_ctrl_hat();
    fn ins_ctrl_();
    fn ins_start_select(c: c_int) -> c_int;
    fn ins_ctrl_g();
    fn ins_shift(c: c_int, lastc: c_int);
    fn ins_del();
    fn ins_left();
    fn ins_right();
    fn ins_s_left();
    fn ins_s_right();
    fn ins_home(c: c_int);
    fn ins_end(c: c_int);
    fn ins_up(startcol: c_int);
    fn ins_down(startcol: c_int);
    fn ins_pageup();
    fn ins_pagedown();
    fn ins_ctrl_v();
    fn ins_ctrl_ey(tc: c_int) -> c_int;
    fn ins_digraph() -> c_int;
    fn stop_arrow() -> c_int;
    fn rs_foldOpenCursor();
    fn insert_special(c: c_int, allow_modmask: c_int, ctrlv: c_int);
    fn stuff_inserted(c: c_int, count: c_int, no_esc: c_int) -> c_int;
    fn do_insert_char_pre(c: c_int) -> *mut c_char;
    fn echeck_abbr(c: c_int) -> c_int;
    fn get_nolist_virtcol() -> c_int;

    // -- Tab / EOL (Rust exports) --
    fn ins_tab() -> bool;
    fn ins_eol(c: c_int) -> bool;
    fn ins_bs(c: c_int, mode: c_int, inserted_space_p: *mut c_int) -> bool;

    // -- backspace mode constants (C enums) --
    // BACKSPACE_CHAR=1, BACKSPACE_WORD=2, BACKSPACE_WORD_NOT_SPACE=3, BACKSPACE_LINE=4

    // -- Globals --
    fn nvim_get_can_cindent() -> c_int;
    fn nvim_set_can_cindent(val: c_int);
    fn nvim_get_arrow_used() -> c_int;
    fn nvim_get_revins_legal() -> c_int;
    fn nvim_set_revins_legal(val: c_int);
    fn nvim_get_revins_chars() -> c_int;
    fn nvim_set_revins_chars(val: c_int);
    fn nvim_set_ins_need_undo(val: c_int);
    fn nvim_get_Insstart_lnum() -> LinenrT;
    fn nvim_get_Insstart_blank_vcol() -> c_int;
    fn nvim_set_Insstart_blank_vcol(val: c_int);
    fn nvim_curwin_get_cursor_lnum() -> LinenrT;
    fn nvim_curwin_get_cursor_col() -> c_int;
    fn nvim_get_mod_mask() -> c_int;
    fn nvim_edit_mod_mask_ctrl() -> c_int;
    fn nvim_get_got_int() -> c_int;
    fn nvim_set_got_int(val: c_int);

    // -- Phase 4 dispatch accessors --
    fn pum_visible() -> bool;
    fn nvim_get_pum_want_active() -> c_int;
    fn nvim_edit_set_pum_want_active(val: c_int);
    fn nvim_edit_get_pum_want_finish() -> c_int;
    fn nvim_edit_clear_edit_submode_extra();
    fn nvim_get_cmdwin_type() -> c_int;
    fn nvim_edit_set_cmdwin_result(val: c_int);
    fn nvim_set_ins_at_eol(val: bool);
    fn nvim_set_did_cursorhold(val: bool);
    fn nvim_edit_inc_disable_fold_update();
    fn nvim_edit_dec_disable_fold_update();
    fn nvim_edit_set_compl_busy(val: c_int);
    fn nvim_edit_update_can_si_from_may_do_si();
    fn nvim_edit_ins_complete(c: c_int) -> c_int;
    fn nvim_edit_check_compl_option(allow_always: c_int) -> c_int;
    fn ins_ctrl_x();
    fn nvim_edit_do_cmdline_getcmdkeycmd();
    fn nvim_edit_map_execute_lua();
    fn nvim_edit_paste_repeat();
    fn state_handle_k_event();
    fn nvim_edit_curwin_is_qf_not_ll() -> c_int;
    fn nvim_edit_quickfix_cc();
    fn nvim_edit_quickfix_ll();
    fn invoke_prompt_interrupt() -> bool;
    fn prompt_invoke_callback();
    fn nvim_edit_get_curbuf_b_u_synced() -> c_int;
    fn nvim_get_p_paste() -> c_int;
    fn char_before_cursor() -> c_int;
    fn char_avail() -> bool;
    fn nvim_inindent_zero() -> bool;
    fn nvim_edit_auto_format(force_format: c_int);
    fn nvim_edit_in_cinkeys(c: c_int, r#type: c_int, line_is_white: c_int) -> c_int;
    fn do_c_expr_indent();
    fn ins_reg();
    fn ins_try_si(c: c_int);
    fn update_screen() -> c_int;
    fn ui_flush();
    fn nvim_bt_quickfix_curbuf() -> c_int;
    fn nvim_bt_prompt_curbuf() -> bool;
    fn nvim_edit_get_curwin_p_rl() -> c_int;
    fn nvim_edit_cursor_col_ge_compl_col() -> c_int;
    fn nvim_edit_get_cpt_first_char() -> c_int;
    fn vim_iswordc(c: c_int) -> bool;
    fn nvim_edit_ve_onemore() -> c_int;
    fn nvim_edit_redraw_later_valid();
    fn vim_isprintc(c: c_int) -> bool;
    fn nvim_get_dont_sync_undo() -> c_int;
    fn nvim_set_dont_sync_undo(val: c_int);

    // -- curwin for ins_compl_win_active --
    fn nvim_curwin_get_cursor_ptr() -> *const c_void;

    // -- ins_mouse / ins_mousescroll --
    fn ins_mouse(c: c_int);
    fn ins_mousescroll(dir: c_int);
}

// ============================================================================
// Constants (verified against C headers via `_Static_assert` in `edit.c`)
// ============================================================================

/// `FAIL` from `vim_defs.h`
const FAIL: c_int = 0;
/// `OK` from `vim_defs.h`
const OK: c_int = 1;

// Special ASCII / control characters
const NUL: c_int = 0;
const ESC: c_int = 0x1b;
const TAB: c_int = 0x09;
const CAR: c_int = 0x0d;
const NL: c_int = 0x0a;
const CTRL_A: c_int = 1;
const CTRL_C: c_int = 3;
const CTRL_D: c_int = 4;
const CTRL_E: c_int = 5;
const CTRL_F: c_int = 6;
const CTRL_G: c_int = 7;
const CTRL_H: c_int = 8;
const CTRL_K: c_int = 11;
const CTRL_L: c_int = 12;
const CTRL_N: c_int = 14;
const CTRL_O: c_int = 15;
const CTRL_P: c_int = 16;
const CTRL_Q: c_int = 17;
const CTRL_R: c_int = 18;
const CTRL_RSB: c_int = 29; // Ctrl-]
const CTRL_T: c_int = 20;
const CTRL_U: c_int = 21;
const CTRL_V: c_int = 22;
const CTRL_W: c_int = 23;
const CTRL_X: c_int = 24;
const CTRL_Y: c_int = 25;
const CTRL_Z: c_int = 26;
const CTRL_BSL: c_int = 28; // Ctrl-backslash
const CTRL_HAT: c_int = 30; // Ctrl-^
const CTRL__: c_int = 31; // Ctrl-_
const SPACE: c_int = 0x20;
const S_LOWERCASE_CHAR: c_int = b's' as c_int;

/// `ABBR_OFF` from keycodes
const ABBR_OFF: c_int = 0x100;

/// Modifier mask — shift
const MOD_MASK_SHIFT: c_int = 0x02;
/// Modifier mask — ctrl
const MOD_MASK_CTRL: c_int = 0x04;

/// `MAXCOL` from pos_defs.h
const MAXCOL: c_int = 0x7fff_ffff;

// Key constants — computed from TERMCAP2KEY macro: -(a + (b << 8))
// KS_EXTRA = 253, KS_SELECT = 245, KS_ZERO = 255

/// `K_BS` — Backspace key
const K_BS: c_int = -25195; // TERMCAP2KEY('k','b')
/// `K_INS` — Insert key
const K_INS: c_int = -18795; // TERMCAP2KEY('k','I')
/// `K_KINS` — Keypad Insert
const K_KINS: c_int = -20477; // TERMCAP2KEY(KS_EXTRA, 79)
/// `K_DEL` — Delete key
const K_DEL: c_int = -17515; // TERMCAP2KEY('k','D')
/// `K_KDEL` — Keypad Delete
const K_KDEL: c_int = -20733; // TERMCAP2KEY(KS_EXTRA, 80)
/// `K_HOME` — Home key
const K_HOME: c_int = -26731; // TERMCAP2KEY('k','h')
/// `K_KHOME` — Keypad Home
const K_KHOME: c_int = -12619; // TERMCAP2KEY('K','1')
/// `K_S_HOME` — Shift-Home
const K_S_HOME: c_int = -12835; // TERMCAP2KEY(KS_EXTRA, 50) = -(253 + (50<<8)) = -(253+12800)
/// `K_C_HOME` — Ctrl-Home
const K_C_HOME: c_int = -22525; // TERMCAP2KEY(KS_EXTRA, 87) = -(253 + (87<<8)) = -(253+22272)
/// `K_END` — End key
const K_END: c_int = -14144; // TERMCAP2KEY('@','7')
/// `K_KEND` — Keypad End
const K_KEND: c_int = -13387; // TERMCAP2KEY('K','4')
/// `K_S_END` — Shift-End
const K_S_END: c_int = -14122; // TERMCAP2KEY(KS_EXTRA, 55) = -(253 + (55<<8)) = -(253+14080)
/// `K_C_END` — Ctrl-End
const K_C_END: c_int = -22781; // TERMCAP2KEY(KS_EXTRA, 88) = -(253 + (88<<8)) = -(253+22528)
/// `K_LEFT` — Left arrow
const K_LEFT: c_int = -27755; // TERMCAP2KEY('k','l')
/// `K_S_LEFT` — Shift-Left
const K_S_LEFT: c_int = -13347; // TERMCAP2KEY('#','4')
/// `K_C_LEFT` — Ctrl-Left
const K_C_LEFT: c_int = -22013; // TERMCAP2KEY(KS_EXTRA, 85) = -(253 + (85<<8)) = -(253+21760)
/// `K_RIGHT` — Right arrow
const K_RIGHT: c_int = -29291; // TERMCAP2KEY('k','r')
/// `K_S_RIGHT` — Shift-Right
const K_S_RIGHT: c_int = -26917; // TERMCAP2KEY('%','i')
/// `K_C_RIGHT` — Ctrl-Right
const K_C_RIGHT: c_int = -22269; // TERMCAP2KEY(KS_EXTRA, 86) = -(253 + (86<<8)) = -(253+22016)
/// `K_UP` — Up arrow
const K_UP: c_int = -30059; // TERMCAP2KEY('k','u')
/// `K_S_UP` — Shift-Up
const K_S_UP: c_int = -1277; // TERMCAP2KEY(KS_EXTRA, 4) = -(253 + (4<<8)) = -(253+1024)
/// `K_DOWN` — Down arrow
const K_DOWN: c_int = -25707; // TERMCAP2KEY('k','d')
/// `K_S_DOWN` — Shift-Down
const K_S_DOWN: c_int = -1533; // TERMCAP2KEY(KS_EXTRA, 5) = -(253 + (5<<8)) = -(253+1280)
/// `K_PAGEUP` — Page Up
const K_PAGEUP: c_int = -20587; // TERMCAP2KEY('k','P')
/// `K_KPAGEUP` — Keypad Page Up
const K_KPAGEUP: c_int = -13131; // TERMCAP2KEY('K','3')
/// `K_PAGEDOWN` — Page Down
const K_PAGEDOWN: c_int = -20075; // TERMCAP2KEY('k','N')
/// `K_KPAGEDOWN` — Keypad Page Down
const K_KPAGEDOWN: c_int = -13643; // TERMCAP2KEY('K','5')
/// `K_S_TAB` — Shift-Tab
const K_S_TAB: c_int = -17003; // TERMCAP2KEY('k','B')
/// `K_KENTER` — Keypad Enter
const K_KENTER: c_int = -16715; // TERMCAP2KEY('K','A')
/// `K_HELP` — Help key
const K_HELP: c_int = -12581; // TERMCAP2KEY('%','1')
/// `K_F1` — F1 key
const K_F1: c_int = -12651; // TERMCAP2KEY('k','1')
/// `K_XF1` — Extra F1
const K_XF1: c_int = -14845; // TERMCAP2KEY(KS_EXTRA, 57)
/// `K_SELECT` — Select key end (KS_SELECT=245, KE_FILLER='X'=88)
const K_SELECT: c_int = -22773; // -(245 + (88<<8)) = -(245+22528)
/// `K_ZERO` — Previously inserted text (KS_ZERO=255, KE_FILLER='X'=88)
const K_ZERO: c_int = -22783; // -(255 + (88<<8)) = -(255+22528)
/// `K_IGNORE` — Mapped to nothing
const K_IGNORE: c_int = -13821; // TERMCAP2KEY(KS_EXTRA, 53)
/// `K_NOP` — No-op
const K_NOP: c_int = -25085; // TERMCAP2KEY(KS_EXTRA, 97)
/// `K_EVENT` — Some event
const K_EVENT: c_int = -26365; // TERMCAP2KEY(KS_EXTRA, 102)
/// `K_COMMAND` — <Cmd>
const K_COMMAND: c_int = -26877; // TERMCAP2KEY(KS_EXTRA, 104)
/// `K_LUA` — Lua key
const K_LUA: c_int = -26621; // TERMCAP2KEY(KS_EXTRA, 103)
/// `K_PASTE_START` — Paste start
const K_PASTE_START: c_int = -21328; // TERMCAP2KEY('P','S')

// Mouse button keys (KS_EXTRA=253)
const K_LEFTMOUSE: c_int = -11517; // TERMCAP2KEY(253, 44)
const K_LEFTMOUSE_NM: c_int = -17917; // TERMCAP2KEY(253, 69)
const K_LEFTDRAG: c_int = -11773; // TERMCAP2KEY(253, 45)
const K_LEFTRELEASE: c_int = -12029; // TERMCAP2KEY(253, 46)
const K_LEFTRELEASE_NM: c_int = -18173; // TERMCAP2KEY(253, 70)
const K_MOUSEMOVE: c_int = -25853; // TERMCAP2KEY(253, 100)
const K_MIDDLEMOUSE: c_int = -12285; // TERMCAP2KEY(253, 47)
const K_MIDDLEDRAG: c_int = -12541; // TERMCAP2KEY(253, 48)
const K_MIDDLERELEASE: c_int = -12797; // TERMCAP2KEY(253, 49)
const K_RIGHTMOUSE: c_int = -13053; // TERMCAP2KEY(253, 50)
const K_RIGHTDRAG: c_int = -13309; // TERMCAP2KEY(253, 51)
const K_RIGHTRELEASE: c_int = -13565; // TERMCAP2KEY(253, 52)
const K_X1MOUSE: c_int = -23037; // TERMCAP2KEY(253, 89)
const K_X1DRAG: c_int = -23293; // TERMCAP2KEY(253, 90)
const K_X1RELEASE: c_int = -23549; // TERMCAP2KEY(253, 91)
const K_X2MOUSE: c_int = -23805; // TERMCAP2KEY(253, 92)
const K_X2DRAG: c_int = -24061; // TERMCAP2KEY(253, 93)
const K_X2RELEASE: c_int = -24317; // TERMCAP2KEY(253, 94)
const K_MOUSEDOWN: c_int = -19453; // TERMCAP2KEY(253, 75)
const K_MOUSEUP: c_int = -19709; // TERMCAP2KEY(253, 76)
const K_MOUSELEFT: c_int = -19965; // TERMCAP2KEY(253, 77)
const K_MOUSERIGHT: c_int = -20221; // TERMCAP2KEY(253, 78)

// Mouse scroll directions (from mouse.h)
const MSCR_DOWN: c_int = 0;
const MSCR_UP: c_int = 1;
const MSCR_LEFT: c_int = -1;
const MSCR_RIGHT: c_int = -2;

// Backspace mode constants (from edit.c enum)
const BACKSPACE_CHAR: c_int = 1;
const BACKSPACE_WORD: c_int = 2;
const BACKSPACE_LINE: c_int = 4;

// TriState constants (kFalse=0, kNone=-1, kTrue=1)
const K_TRUE: c_int = 1;
const K_NONE: c_int = -1;
const K_FALSE: c_int = 0;

// ============================================================================
// Action enum for goto-translation
// ============================================================================

/// Translates C `goto normalchar` / `goto check_pum` patterns.
enum SwitchAction {
    /// Normal case: proceed to `insert_handle_key_post` and return 1.
    Continue,
    /// `goto normalchar`: insert the character as a normal char.
    NormalChar,
    /// `goto check_pum`: run the PUM check before continuing.
    CheckPum,
    /// Return the specified value immediately (exit or continue from switch).
    Exit(c_int),
}

// ============================================================================
// insert_do_complete
// ============================================================================

/// Trigger insert completion.
///
/// Translated from C `insert_do_complete`.
///
/// # Safety
/// Accesses C globals via accessor functions.
#[unsafe(export_name = "insert_do_complete")]
pub unsafe extern "C" fn rs_insert_do_complete(s: *mut InsertState) {
    nvim_edit_set_compl_busy(1);
    nvim_edit_inc_disable_fold_update();
    if nvim_edit_ins_complete((*s).c) == FAIL {
        rs_compl_status_clear();
    }
    nvim_edit_dec_disable_fold_update();
    nvim_edit_set_compl_busy(0);
    nvim_edit_update_can_si_from_may_do_si();
}

// ============================================================================
// insert_do_cindent
// ============================================================================

/// Handle cindent for the given key.
///
/// Translated from C `insert_do_cindent`.
///
/// # Safety
/// Accesses C globals via accessor functions.
#[unsafe(export_name = "insert_do_cindent")]
pub unsafe extern "C" fn rs_insert_do_cindent(s: *mut InsertState) {
    if nvim_edit_in_cinkeys((*s).c, c_int::from(b' '), c_int::from((*s).line_is_white)) != 0
        && stop_arrow() == OK
    {
        do_c_expr_indent();
    }
}

// ============================================================================
// insert_handle_key_post
// ============================================================================

/// Post-key-handling cleanup.
///
/// Translated from C `insert_handle_key_post`.
///
/// # Safety
/// Accesses C globals via accessor functions.
#[unsafe(export_name = "insert_handle_key_post")]
pub unsafe extern "C" fn rs_insert_handle_key_post(s: *mut InsertState) {
    // If typed something, may trigger CursorHoldI again.
    // But not in CTRL-X mode — a script can't restore the state.
    if (*s).c != K_EVENT && rs_ctrl_x_mode_normal() != 0 {
        nvim_set_did_cursorhold(false);
    }

    // Cancel completion if window or tab page changed.
    if rs_ins_compl_active() != 0 {
        // curwin handle: use the cursor pointer as an opaque win identifier
        #[allow(clippy::ptr_cast_constness)]
        let curwin_ptr = nvim_curwin_get_cursor_ptr().cast_mut();
        if rs_ins_compl_win_active(curwin_ptr) == 0 {
            rs_ins_compl_cancel();
        }
    }

    // If the cursor was moved, we didn't just insert a space.
    if nvim_get_arrow_used() != 0 {
        (*s).inserted_space = 0;
    }

    if nvim_get_can_cindent() != 0 && rs_cindent_on() && rs_ctrl_x_mode_normal() != 0 {
        rs_insert_do_cindent(s);
    }
}

// ============================================================================
// check_pum helper — shared by K_EVENT / K_COMMAND / K_LUA / K_PASTE_START
// ============================================================================

/// Handle the `check_pum` label logic.
unsafe fn do_check_pum(s: *mut InsertState) {
    if nvim_get_pum_want_active() != 0 {
        if pum_visible() {
            nvim_edit_clear_edit_submode_extra();
            rs_insert_do_complete(s);
            if nvim_edit_get_pum_want_finish() != 0 {
                rs_ins_compl_prep(CTRL_Y);
            }
        }
        nvim_edit_set_pum_want_active(0);
    }

    if nvim_edit_get_curbuf_b_u_synced() != 0 {
        nvim_set_ins_need_undo(1);
    }
}

// ============================================================================
// trigger_autocomplete helper
// ============================================================================

/// Trigger autocomplete (equivalent to the C `TRIGGER_AUTOCOMPLETE` macro).
/// Returns true when autocomplete was triggered (caller should break from switch).
unsafe fn trigger_autocomplete(s: *mut InsertState) {
    nvim_edit_redraw_later_valid();
    update_screen();
    ui_flush();
    rs_ins_compl_enable_autocomplete();
    rs_insert_do_complete(s);
}

/// May trigger autocomplete (equivalent to `MAY_TRIGGER_AUTOCOMPLETE` macro).
/// Updates `s->c` to the char before cursor, returns true if triggered.
unsafe fn may_trigger_autocomplete(s: *mut InsertState) -> bool {
    if rs_ins_compl_has_autocomplete() != 0 && !char_avail() && nvim_curwin_get_cursor_col() > 0 {
        (*s).c = char_before_cursor();
        if vim_isprintc((*s).c) {
            trigger_autocomplete(s);
            return true;
        }
    }
    false
}

// ============================================================================
// insert_handle_key — the big switch
// ============================================================================

/// Handle a single key in insert mode.
///
/// Returns 0 to exit insert mode, 1 to continue.
///
/// Translated from C `insert_handle_key`.
///
/// # Safety
/// Accesses C globals extensively via accessor functions.
#[unsafe(export_name = "insert_handle_key")]
pub unsafe extern "C" fn rs_insert_handle_key(s: *mut InsertState) -> c_int {
    // Handle right-to-left key swapping (p_rl option).
    if nvim_edit_get_curwin_p_rl() != 0 {
        (*s).c = match (*s).c {
            K_LEFT => K_RIGHT,
            K_S_LEFT => K_S_RIGHT,
            K_C_LEFT => K_C_RIGHT,
            K_RIGHT => K_LEFT,
            K_S_RIGHT => K_S_LEFT,
            K_C_RIGHT => K_C_LEFT,
            c => c,
        };
    }

    // If 'keymodel' contains "startsel", may start selection.
    if ins_start_select((*s).c) != 0 {
        return 1; // continue
    }

    let action = handle_key_switch(s);

    match action {
        SwitchAction::Exit(v) => {
            return v;
        }
        SwitchAction::CheckPum => {
            do_check_pum(s);
            // fall through to post
        }
        SwitchAction::NormalChar => {
            handle_normalchar(s);
        }
        SwitchAction::Continue => {
            // fall through to post
        }
    }

    rs_insert_handle_key_post(s);
    1 // continue
}

/// Run the normalchar path (insert character as normal char).
unsafe fn handle_normalchar(s: *mut InsertState) {
    if nvim_get_p_paste() == 0 {
        let str_ptr = do_insert_char_pre((*s).c);
        if !str_ptr.is_null() {
            if *str_ptr != 0 && stop_arrow() != FAIL {
                // Insert the new value of v:char literally.
                let mut p = str_ptr;
                while *p != 0 {
                    // Decode one UTF-8 character
                    let c = utf_ptr2char_and_advance(std::ptr::addr_of_mut!(p));
                    if c == CAR || c == K_KENTER || c == NL {
                        ins_eol(c);
                    } else {
                        insert_special(c, 0, 0);
                    }
                }
                append_to_redo_lit(str_ptr);
            }
            xfree_void(str_ptr.cast::<c_void>());
            (*s).c = NUL;
        }
        if (*s).c == NUL {
            rs_insert_handle_key_post(s);
            return;
        }
    }

    ins_try_si((*s).c);

    if (*s).c == SPACE {
        (*s).inserted_space = 1;
        if nvim_inindent_zero() {
            nvim_set_can_cindent(0);
        }
        if nvim_get_Insstart_blank_vcol() == MAXCOL
            && nvim_curwin_get_cursor_lnum() == nvim_get_Insstart_lnum()
        {
            nvim_set_Insstart_blank_vcol(get_nolist_virtcol());
        }
    }

    if vim_iswordc((*s).c)
        || (echeck_abbr(if (*s).c >= ABBR_OFF {
            (*s).c + ABBR_OFF
        } else {
            (*s).c
        }) == 0
            && (*s).c != CTRL_RSB)
    {
        insert_special((*s).c, 0, 0);
        nvim_set_revins_legal(nvim_get_revins_legal() + 1);
        nvim_set_revins_chars(nvim_get_revins_chars() + 1);
    }

    nvim_edit_auto_format(1); // prev_line=true

    rs_foldOpenCursor();

    // Trigger autocompletion
    if rs_ins_compl_has_autocomplete() != 0 && !char_avail() && vim_isprintc((*s).c) {
        trigger_autocomplete(s);
    }

    rs_insert_handle_key_post(s);
}

/// The main switch over `s->c`.
/// Returns the action to take after the switch.
unsafe fn handle_key_switch(s: *mut InsertState) -> SwitchAction {
    match (*s).c {
        // ESC / Ctrl-C: end input mode
        ESC => {
            if echeck_abbr(ESC + ABBR_OFF) != 0 {
                return SwitchAction::Continue;
            }
            // FALLTHROUGH to Ctrl-C
            SwitchAction::Exit(0)
        }

        CTRL_C => {
            if nvim_get_cmdwin_type() != 0 {
                nvim_edit_set_cmdwin_result(K_IGNORE);
                nvim_set_got_int(0);
                (*s).nomove = true;
                return SwitchAction::Exit(0);
            }
            if nvim_bt_prompt_curbuf() && invoke_prompt_interrupt() {
                if !nvim_bt_prompt_curbuf() {
                    return SwitchAction::Exit(0);
                }
                return SwitchAction::Continue;
            }
            SwitchAction::Exit(0)
        }

        CTRL_Z => SwitchAction::NormalChar, // insert CTRL-Z as normal char

        CTRL_O => {
            if rs_ctrl_x_mode_omni() != 0 {
                rs_insert_do_complete(s);
                return SwitchAction::Continue;
            }
            if echeck_abbr(CTRL_O + ABBR_OFF) != 0 {
                return SwitchAction::Continue;
            }
            ins_ctrl_o();
            if nvim_edit_ve_onemore() != 0 {
                nvim_set_ins_at_eol(false);
                (*s).nomove = true;
            }
            (*s).count = 0;
            SwitchAction::Exit(0)
        }

        K_INS | K_KINS => {
            ins_insert((*s).replace_state);
            SwitchAction::Continue
        }

        K_SELECT | K_IGNORE => SwitchAction::Continue, // Select mode end / mapped to nothing

        K_HELP | K_F1 | K_XF1 => {
            // Help key works like <ESC> <Help>
            // stuffcharReadbuff is called in C; we use the accessor
            nvim_stuffcharReadbuff(K_HELP);
            SwitchAction::Exit(0)
        }

        SPACE => {
            if nvim_edit_mod_mask_ctrl() != 0 {
                // FALLTHROUGH to K_ZERO / NUL / Ctrl-A
                handle_insert_previously_inserted(s)
            } else {
                SwitchAction::NormalChar
            }
        }

        K_ZERO | NUL | CTRL_A => handle_insert_previously_inserted(s),

        CTRL_R => {
            if rs_ctrl_x_mode_register() != 0 && rs_ins_compl_active() == 0 {
                rs_insert_do_complete(s);
                return SwitchAction::Continue;
            }
            ins_reg();
            nvim_edit_auto_format(1);
            (*s).inserted_space = 0;
            SwitchAction::Continue
        }

        CTRL_G => {
            ins_ctrl_g();
            SwitchAction::Continue
        }

        CTRL_HAT => {
            ins_ctrl_hat();
            SwitchAction::Continue
        }

        CTRL__ => {
            if nvim_get_p_ari() == 0 {
                return SwitchAction::NormalChar;
            }
            ins_ctrl_();
            SwitchAction::Continue
        }

        CTRL_D => {
            if rs_ctrl_x_mode_path_defines() != 0 {
                rs_insert_do_complete(s);
                return SwitchAction::Continue;
            }
            // FALLTHROUGH to CTRL_T
            ins_shift((*s).c, (*s).lastc);
            nvim_edit_auto_format(1);
            (*s).inserted_space = 0;
            SwitchAction::Continue
        }

        CTRL_T => {
            if rs_ctrl_x_mode_thesaurus() != 0 {
                if nvim_edit_check_compl_option(0) != 0 {
                    rs_insert_do_complete(s);
                }
                return SwitchAction::Continue;
            }
            ins_shift((*s).c, (*s).lastc);
            nvim_edit_auto_format(1);
            (*s).inserted_space = 0;
            SwitchAction::Continue
        }

        K_DEL | K_KDEL => {
            ins_del();
            nvim_edit_auto_format(1);
            SwitchAction::Continue
        }

        K_BS | CTRL_H => {
            (*s).did_backspace = ins_bs((*s).c, BACKSPACE_CHAR, &raw mut (*s).inserted_space);
            nvim_edit_auto_format(1);
            if (*s).did_backspace && may_trigger_autocomplete(s) {
                return SwitchAction::Continue;
            }
            SwitchAction::Continue
        }

        CTRL_W => {
            if nvim_bt_prompt_curbuf() && (nvim_get_mod_mask() & MOD_MASK_SHIFT) == 0 {
                nvim_stuffcharReadbuff(CTRL_W);
                nvim_set_restart_edit(c_int::from(b'A'));
                (*s).nomove = true;
                (*s).count = 0;
                return SwitchAction::Exit(0);
            }
            (*s).did_backspace = ins_bs((*s).c, BACKSPACE_WORD, &raw mut (*s).inserted_space);
            nvim_edit_auto_format(1);
            if (*s).did_backspace && may_trigger_autocomplete(s) {
                return SwitchAction::Continue;
            }
            SwitchAction::Continue
        }

        CTRL_U => {
            if rs_ctrl_x_mode_function() != 0 {
                rs_insert_do_complete(s);
            } else {
                (*s).did_backspace = ins_bs((*s).c, BACKSPACE_LINE, &raw mut (*s).inserted_space);
                nvim_edit_auto_format(1);
                (*s).inserted_space = 0;
                if (*s).did_backspace && may_trigger_autocomplete(s) {
                    return SwitchAction::Continue;
                }
            }
            SwitchAction::Continue
        }

        // Mouse keys
        K_LEFTMOUSE | K_LEFTMOUSE_NM | K_LEFTDRAG | K_LEFTRELEASE | K_LEFTRELEASE_NM
        | K_MOUSEMOVE | K_MIDDLEMOUSE | K_MIDDLEDRAG | K_MIDDLERELEASE | K_RIGHTMOUSE
        | K_RIGHTDRAG | K_RIGHTRELEASE | K_X1MOUSE | K_X1DRAG | K_X1RELEASE | K_X2MOUSE
        | K_X2DRAG | K_X2RELEASE => {
            ins_mouse((*s).c);
            SwitchAction::Continue
        }

        K_MOUSEDOWN => {
            ins_mousescroll(MSCR_DOWN);
            SwitchAction::Continue
        }
        K_MOUSEUP => {
            ins_mousescroll(MSCR_UP);
            SwitchAction::Continue
        }
        K_MOUSELEFT => {
            ins_mousescroll(MSCR_LEFT);
            SwitchAction::Continue
        }
        K_MOUSERIGHT => {
            ins_mousescroll(MSCR_RIGHT);
            SwitchAction::Continue
        }

        // K_IGNORE: merged with K_SELECT above
        K_PASTE_START => {
            nvim_edit_paste_repeat();
            SwitchAction::CheckPum
        }

        K_EVENT => {
            state_handle_k_event();
            // If CTRL-G U was used, apply it to the next typed key.
            if nvim_get_dont_sync_undo() == K_TRUE {
                nvim_set_dont_sync_undo(K_NONE);
            }
            SwitchAction::CheckPum
        }

        K_COMMAND => {
            nvim_edit_do_cmdline_getcmdkeycmd();
            SwitchAction::CheckPum
        }

        K_LUA => {
            nvim_edit_map_execute_lua();
            SwitchAction::CheckPum
        }

        // Navigation keys
        K_HOME | K_KHOME | K_S_HOME | K_C_HOME => {
            ins_home((*s).c);
            SwitchAction::Continue
        }

        K_END | K_KEND | K_S_END | K_C_END => {
            ins_end((*s).c);
            SwitchAction::Continue
        }

        K_LEFT => {
            if nvim_get_mod_mask() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0 {
                ins_s_left();
            } else {
                ins_left();
            }
            SwitchAction::Continue
        }

        K_S_LEFT | K_C_LEFT => {
            ins_s_left();
            SwitchAction::Continue
        }

        K_RIGHT => {
            if nvim_get_mod_mask() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0 {
                ins_s_right();
            } else {
                ins_right();
            }
            SwitchAction::Continue
        }

        K_S_RIGHT | K_C_RIGHT => {
            ins_s_right();
            SwitchAction::Continue
        }

        K_UP => {
            if pum_visible() {
                rs_insert_do_complete(s);
            } else if nvim_get_mod_mask() & MOD_MASK_SHIFT != 0 {
                ins_pageup();
            } else {
                ins_up(0);
            }
            SwitchAction::Continue
        }

        K_S_UP | K_PAGEUP | K_KPAGEUP => {
            if pum_visible() {
                rs_insert_do_complete(s);
            } else {
                ins_pageup();
            }
            SwitchAction::Continue
        }

        K_DOWN => {
            if pum_visible() {
                rs_insert_do_complete(s);
            } else if nvim_get_mod_mask() & MOD_MASK_SHIFT != 0 {
                ins_pagedown();
            } else {
                ins_down(0);
            }
            SwitchAction::Continue
        }

        K_S_DOWN | K_PAGEDOWN | K_KPAGEDOWN => {
            if pum_visible() {
                rs_insert_do_complete(s);
            } else {
                ins_pagedown();
            }
            SwitchAction::Continue
        }

        K_S_TAB => {
            (*s).c = TAB;
            // FALLTHROUGH to TAB
            handle_tab(s)
        }

        TAB => handle_tab(s),

        K_KENTER => {
            (*s).c = CAR;
            // FALLTHROUGH to CAR/NL
            handle_enter(s)
        }

        CAR | NL => handle_enter(s),

        CTRL_K => {
            if rs_ctrl_x_mode_dictionary() != 0 {
                if nvim_edit_check_compl_option(1) != 0 {
                    rs_insert_do_complete(s);
                }
                return SwitchAction::Continue;
            }
            (*s).c = ins_digraph();
            if (*s).c == NUL {
                return SwitchAction::Continue;
            }
            SwitchAction::NormalChar
        }

        CTRL_X => {
            ins_ctrl_x();
            SwitchAction::Continue
        }

        CTRL_RSB => {
            if rs_ctrl_x_mode_tags() == 0 {
                SwitchAction::NormalChar
            } else {
                rs_insert_do_complete(s);
                SwitchAction::Continue
            }
        }

        CTRL_F => {
            if rs_ctrl_x_mode_files() == 0 {
                SwitchAction::NormalChar
            } else {
                rs_insert_do_complete(s);
                SwitchAction::Continue
            }
        }

        // 's' (115) or Ctrl-S (19) — spell completion
        115 | 19 => {
            if rs_ctrl_x_mode_spell() == 0 {
                SwitchAction::NormalChar
            } else {
                rs_insert_do_complete(s);
                SwitchAction::Continue
            }
        }

        CTRL_L => {
            if rs_ctrl_x_mode_whole_line() == 0 {
                return SwitchAction::NormalChar;
            }
            // FALLTHROUGH to CTRL_P / CTRL_N
            handle_completion_pn(s)
        }

        CTRL_P | CTRL_N => handle_completion_pn(s),

        CTRL_Y | CTRL_E => {
            (*s).c = ins_ctrl_ey((*s).c);
            SwitchAction::Continue
        }

        _ => SwitchAction::NormalChar, // default: normalchar
    }
}

// ============================================================================
// Sub-handlers extracted from switch arms
// ============================================================================

unsafe fn handle_insert_previously_inserted(s: *mut InsertState) -> SwitchAction {
    if stuff_inserted(NUL, 1, c_int::from((*s).c == CTRL_A)) == FAIL && (*s).c != CTRL_A {
        return SwitchAction::Exit(0);
    }
    (*s).inserted_space = 0;
    SwitchAction::Continue
}

unsafe fn handle_tab(s: *mut InsertState) -> SwitchAction {
    if rs_ctrl_x_mode_path_patterns() != 0 {
        rs_insert_do_complete(s);
        return SwitchAction::Continue;
    }
    (*s).inserted_space = 0;
    if ins_tab() {
        return SwitchAction::NormalChar; // insert TAB as normal char
    }
    nvim_edit_auto_format(1);
    SwitchAction::Continue
}

unsafe fn handle_enter(s: *mut InsertState) -> SwitchAction {
    // In quickfix window, <CR> jumps to error under cursor.
    if nvim_bt_quickfix_curbuf() != 0 && (*s).c == CAR {
        if nvim_edit_curwin_is_qf_not_ll() != 0 {
            nvim_edit_quickfix_cc();
        } else {
            nvim_edit_quickfix_ll();
        }
        return SwitchAction::Continue;
    }
    if nvim_get_cmdwin_type() != 0 {
        nvim_edit_set_cmdwin_result(CAR);
        return SwitchAction::Exit(0);
    }
    if (nvim_get_mod_mask() & MOD_MASK_SHIFT) == 0 && nvim_bt_prompt_curbuf() {
        prompt_invoke_callback();
        if !nvim_bt_prompt_curbuf() {
            return SwitchAction::Exit(0);
        }
        return SwitchAction::Continue;
    }
    if !ins_eol((*s).c) {
        return SwitchAction::Exit(0); // out of memory
    }
    nvim_edit_auto_format(0);
    (*s).inserted_space = 0;
    SwitchAction::Continue
}

unsafe fn handle_completion_pn(s: *mut InsertState) -> SwitchAction {
    // If 'complete' is empty then plain ^P is no longer special,
    // but it is under other ^X modes.
    if nvim_edit_get_cpt_first_char() == NUL as c_int
        && (rs_ctrl_x_mode_normal() != 0 || rs_ctrl_x_mode_whole_line() != 0)
        && rs_compl_status_local() == 0
    {
        return SwitchAction::NormalChar;
    }
    rs_insert_do_complete(s);
    SwitchAction::Continue
}

// ============================================================================
// External helpers (not yet in edit.c accessors)
// ============================================================================

extern "C" {
    fn nvim_stuffcharReadbuff(c: c_int);
    fn nvim_set_restart_edit(val: c_int);
    fn nvim_get_p_ari() -> c_int;
    fn rs_compl_status_local() -> c_int;
    #[link_name = "cindent_on"]
    fn rs_cindent_on() -> bool;
    #[link_name = "xfree"]
    fn xfree_void(ptr: *mut c_void);
    // UTF helpers for normalchar path
    fn utf_ptr2char(p: *const u8) -> c_int;
    fn utfc_ptr2len(p: *const u8) -> c_int;
    fn AppendToRedobuffLit(s: *const c_char, len: c_int);
}

/// Decode one UTF-8 character and advance `p` past it.
unsafe fn utf_ptr2char_and_advance(p: *mut *mut c_char) -> c_int {
    let c = utf_ptr2char((*p).cast::<u8>());
    let len = utfc_ptr2len((*p).cast::<u8>());
    *p = (*p).add(len as usize);
    c
}

/// Call `AppendToRedobuffLit(str, -1)`.
unsafe fn append_to_redo_lit(s: *mut c_char) {
    AppendToRedobuffLit(s, -1);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_constants_range() {
        // All special keys are negative; verify with non-trivial comparison
        assert_eq!(K_BS, -25195);
        assert_eq!(K_LEFT, -27755);
        assert_eq!(K_RIGHT, -29291);
        assert_eq!(K_UP, -30059);
        assert_eq!(K_DOWN, -25707);
        assert_eq!(K_EVENT, -26365);
        assert_eq!(K_COMMAND, -26877);
        assert_eq!(K_LUA, -26621);
    }

    #[test]
    fn test_insert_state_layout() {
        // InsertState must be larger than VimState (2 function pointers)
        assert!(std::mem::size_of::<InsertState>() > std::mem::size_of::<VimState>());
    }

    #[test]
    fn test_control_char_values() {
        assert_eq!(ESC, 0x1b);
        assert_eq!(TAB, 0x09);
        assert_eq!(CTRL_V, 22);
        assert_eq!(CTRL_C, 3);
        assert_eq!(NUL, 0);
    }
}
