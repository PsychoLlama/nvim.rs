//! CTRL-X mode state machine and completion lifecycle functions.
//!
//! This module implements:
//! - `rs_set_ctrl_x_mode`: validates a key after CTRL-X and sets the completion mode
//! - `rs_may_advance_cpt_index`: checks if the 'complete' option index can advance
//! - `rs_ins_compl_prep`: prepares for or stops insert-mode completion
//! - `rs_ins_compl_stop`: finalizes completion, handles cleanup and redo
//! - `rs_ins_compl_cancel`: thin wrapper around rs_ins_compl_stop

#![allow(clippy::doc_markdown)]
#![allow(dead_code)]

use std::os::raw::{c_char, c_int};

// =============================================================================
// CTRL-X mode constants (from insexpand_shim.c)
// =============================================================================

const CTRL_X_NORMAL: c_int = 0;
const CTRL_X_NOT_DEFINED_YET: c_int = 1;
const CTRL_X_SCROLL: c_int = 2;
const CTRL_X_WHOLE_LINE: c_int = 3;
const CTRL_X_FILES: c_int = 4;
const CTRL_X_WANT_IDENT: c_int = 0x100;
const CTRL_X_TAGS: c_int = 5 + CTRL_X_WANT_IDENT;
const CTRL_X_PATH_PATTERNS: c_int = 6 + CTRL_X_WANT_IDENT;
const CTRL_X_PATH_DEFINES: c_int = 7 + CTRL_X_WANT_IDENT;
const CTRL_X_DICTIONARY: c_int = 9 + CTRL_X_WANT_IDENT;
const CTRL_X_THESAURUS: c_int = 10 + CTRL_X_WANT_IDENT;
const CTRL_X_CMDLINE: c_int = 11;
const CTRL_X_FUNCTION: c_int = 12;
const CTRL_X_OMNI: c_int = 13;
const CTRL_X_SPELL: c_int = 14;
const CTRL_X_LOCAL_MSG: c_int = 15; // only used in ctrl_x_msgs / ins_compl_prep
const CTRL_X_FINISHED: c_int = 8;
const CTRL_X_EVAL: c_int = 16;
const CTRL_X_CMDLINE_CTRL_X: c_int = 17;
const CTRL_X_BUFNAMES: c_int = 18;
const CTRL_X_REGISTER: c_int = 19;

// =============================================================================
// Continuation status flags (from insexpand_shim.c)
// =============================================================================

const CONT_INTRPT: c_int = 2 + 4; // 6: a ^X interrupted the current expansion
const CONT_LOCAL: c_int = 32; // for ctrl_x_mode 0, ^X^P/^X^N do local expansion

// =============================================================================
// Control key constants (ASCII)
// =============================================================================

const CTRL_C: c_int = 3;
const CTRL_D: c_int = 4;
const CTRL_E: c_int = 5;
const CTRL_F: c_int = 6;
const CTRL_I: c_int = 9;
const CTRL_K: c_int = 11;
const CTRL_L: c_int = 12;
const CTRL_N: c_int = 14;
const CTRL_O: c_int = 15;
const CTRL_P: c_int = 16;
const CTRL_Q: c_int = 17;
const CTRL_R: c_int = 18;
const CTRL_S: c_int = 19;
const CTRL_T: c_int = 20;
const CTRL_U: c_int = 21;
const CTRL_V: c_int = 22;
const CTRL_X: c_int = 24;
const CTRL_Y: c_int = 25;
const CTRL_Z: c_int = 26;
const CTRL_RSB: c_int = 29;

// ASCII special characters
const CAR: c_int = 0x0D; // carriage return
const NL: c_int = 0x0A; // newline

// =============================================================================
// Key code constants
// TERMCAP2KEY(a, b) = -((a) + ((b) << 8))
// =============================================================================

const fn termcap2key(a: c_int, b: c_int) -> c_int {
    -((a) + (b << 8))
}

const KS_EXTRA: c_int = 253;
const KS_SELECT: c_int = 245;
const KE_FILLER: c_int = b'X' as c_int; // 88

const KE_MOUSEDOWN: c_int = 75;
const KE_MOUSEUP: c_int = 76;
const KE_MOUSELEFT: c_int = 77;
const KE_MOUSERIGHT: c_int = 78;
const KE_MOUSEMOVE: c_int = 100;

const K_SELECT: c_int = termcap2key(KS_SELECT, KE_FILLER);
const K_MOUSEDOWN: c_int = termcap2key(KS_EXTRA, KE_MOUSEDOWN);
const K_MOUSEUP: c_int = termcap2key(KS_EXTRA, KE_MOUSEUP);
const K_MOUSELEFT: c_int = termcap2key(KS_EXTRA, KE_MOUSELEFT);
const K_MOUSERIGHT: c_int = termcap2key(KS_EXTRA, KE_MOUSERIGHT);
const K_MOUSEMOVE: c_int = termcap2key(KS_EXTRA, KE_MOUSEMOVE);

const KE_EVENT: c_int = 102;
const KE_COMMAND: c_int = 104;
const KE_LUA: c_int = 107;
const K_EVENT: c_int = termcap2key(KS_EXTRA, KE_EVENT);
const K_COMMAND: c_int = termcap2key(KS_EXTRA, KE_COMMAND);
const K_LUA: c_int = termcap2key(KS_EXTRA, KE_LUA);

// K_S_TAB = TERMCAP2KEY('k', 'B')
const K_S_TAB: c_int = termcap2key(b'k' as c_int, b'B' as c_int);

// K_KENTER = TERMCAP2KEY('K', 'A')
const K_KENTER: c_int = termcap2key(b'K' as c_int, b'A' as c_int);

// Completeopt flags
const K_OPT_COT_FLAG_LONGEST: u32 = 0x04;

// =============================================================================
// C accessor FFI declarations
// =============================================================================

extern "C" {
    // Phase 1 accessors
    fn nvim_set_edit_submode_scroll(is_replace: c_int);
    #[link_name = "edit_submode"]
    static mut g_edit_submode: *mut c_char;
    #[link_name = "edit_submode_extra"]
    static mut g_edit_submode_extra: *mut c_char;
    #[link_name = "edit_submode_pre"]
    static mut g_edit_submode_pre: *mut c_char;
    #[link_name = "redraw_mode"]
    static mut g_redraw_mode: bool;
    fn nvim_get_state_replace_flag() -> c_int;
    fn nvim_spell_back_safe();
    fn vpeekc() -> c_int;

    // nvim_get_cot_flags_global: inlined in vars.rs (Phase 30)
    fn nvim_curbuf_get_b_cot_flags() -> u32;

    // Phase 2: C functions called (not pure accessors)
    fn rs_do_autocmd_completedone(c: c_int, mode: c_int, word: *mut c_char);
    fn may_trigger_modechanged();
    fn rs_ins_compl_pum_key(c: c_int) -> c_int;
    // rs_* functions from lib.rs accessible via no_mangle extern "C"
    fn rs_ctrl_x_mode_cmdline() -> c_int;

    // Phase 3 state accessors
    fn pum_visible() -> c_int;
    // (nvim_get_compl_curr_match_str_data: inlined in match_list.rs)
    fn nvim_get_compl_shown_match_str_dup() -> *mut c_char;
    // nvim_clear_compl_best_matches: inlined in vars.rs (Phase 24)
    fn nvim_get_arrow_used() -> c_int;
    fn nvim_get_cmdwin_type() -> c_int;
    fn nvim_cursor_on_nul() -> c_int;
    fn nvim_get_cursor_col() -> c_int;

    // Phase 3 compound accessors
    fn nvim_ins_apply_autocmds_completedonepre();
    fn nvim_shortmess_completionmenu() -> bool;
    fn nvim_in_cinkeys_key_complete(when: c_int, line_is_empty: bool) -> bool;
    fn nvim_set_edit_submode_null_if_set();
    fn nvim_get_curwin() -> *mut u8; // opaque curwin pointer
    fn nvim_get_curwin_cursor_lnum() -> c_int;
    fn nvim_xfree(ptr: *mut u8);

    // Phase 3 C functions called (not pure accessors)
    fn rs_ins_compl_preinsert_effect() -> c_int;
    fn rs_ins_compl_win_active(wp: *mut u8) -> c_int;
    fn rs_ins_compl_fixRedoBufForLeader(ptr: *const c_char);
    fn rs_ins_compl_free();
    fn rs_get_compl_len() -> c_int;
    fn nvim_ins_compl_insert_bytes(p: *const c_char, len: c_int);
    fn nvim_restore_orig_extmarks();
    fn get_can_cindent() -> bool;
    fn cindent_on() -> bool;
    fn do_c_expr_indent();
    fn dec_cursor();
    fn inc_cursor();
    fn ins_need_undo_get() -> bool;
    fn insertchar(c: c_int, flags: c_int, second_indent: c_int);
    fn redrawWinline(wp: *mut u8, lnum: c_int);
    fn auto_format(trailblank: bool, prev_line: bool);
    fn msg_clr_cmdline();
    fn update_screen();
}

// =============================================================================
// Phase 1: rs_set_ctrl_x_mode
// =============================================================================

/// Set the CTRL-X completion mode based on the key typed after CTRL-X.
///
/// Returns 1 when the character is not to be inserted (i.e., CTRL-Z was typed),
/// 0 otherwise.
unsafe fn rs_set_ctrl_x_mode(c: c_int) -> c_int {
    let mut retval = false;

    match c {
        CTRL_E | CTRL_Y => {
            crate::vars::nvim_set_ctrl_x_mode(CTRL_X_SCROLL);
            nvim_set_edit_submode_scroll(nvim_get_state_replace_flag());
        }
        CTRL_L => crate::vars::nvim_set_ctrl_x_mode(CTRL_X_WHOLE_LINE),
        CTRL_F => crate::vars::nvim_set_ctrl_x_mode(CTRL_X_FILES),
        CTRL_K => crate::vars::nvim_set_ctrl_x_mode(CTRL_X_DICTIONARY),
        CTRL_R => {
            if vpeekc() != i32::from(b'=') {
                crate::vars::nvim_set_ctrl_x_mode(CTRL_X_REGISTER);
            }
        }
        CTRL_T => crate::vars::nvim_set_ctrl_x_mode(CTRL_X_THESAURUS),
        CTRL_U => crate::vars::nvim_set_ctrl_x_mode(CTRL_X_FUNCTION),
        CTRL_O => crate::vars::nvim_set_ctrl_x_mode(CTRL_X_OMNI),
        x if x == i32::from(b's') || x == CTRL_S => {
            crate::vars::nvim_set_ctrl_x_mode(CTRL_X_SPELL);
            nvim_spell_back_safe();
        }
        CTRL_RSB => crate::vars::nvim_set_ctrl_x_mode(CTRL_X_TAGS),
        CTRL_I | K_S_TAB => crate::vars::nvim_set_ctrl_x_mode(CTRL_X_PATH_PATTERNS),
        CTRL_D => crate::vars::nvim_set_ctrl_x_mode(CTRL_X_PATH_DEFINES),
        CTRL_V | CTRL_Q => crate::vars::nvim_set_ctrl_x_mode(CTRL_X_CMDLINE),
        CTRL_Z => {
            crate::vars::nvim_set_ctrl_x_mode(CTRL_X_NORMAL);
            g_edit_submode = core::ptr::null_mut();
            g_redraw_mode = true;
            retval = true;
        }
        CTRL_P | CTRL_N => {
            let cont_status = crate::vars::nvim_get_compl_cont_status();
            if (cont_status & CONT_INTRPT) == 0 {
                crate::vars::nvim_set_compl_cont_status(cont_status | CONT_LOCAL);
            } else if crate::vars::nvim_get_compl_cont_mode() != 0 {
                crate::vars::nvim_set_compl_cont_status(cont_status & !CONT_LOCAL);
            }
            set_ctrl_x_mode_default(c);
        }
        _ => set_ctrl_x_mode_default(c),
    }

    c_int::from(retval)
}

/// Shared default logic for CTRL-X mode (handles CTRL-X and all other keys).
///
/// Called from the Ctrl_P/Ctrl_N arm (after local-mode flag adjustment)
/// and from the default arm.
unsafe fn set_ctrl_x_mode_default(c: c_int) {
    if c == CTRL_X {
        if crate::vars::nvim_get_compl_cont_mode() != 0 {
            crate::vars::nvim_set_compl_cont_status(0);
        } else {
            crate::vars::nvim_set_compl_cont_mode(CTRL_X_NOT_DEFINED_YET);
        }
    }
    crate::vars::nvim_set_ctrl_x_mode(CTRL_X_NORMAL);
    g_edit_submode = core::ptr::null_mut();
    g_redraw_mode = true;
}

// =============================================================================
// Phase 1: rs_may_advance_cpt_index
// =============================================================================

/// Check if the 'complete' option index can advance past any delimiters.
///
/// Returns 1 if cpt_sources_index is valid and there is a non-delimiter
/// character remaining in the option string, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_may_advance_cpt_index(cpt: *const c_char) -> c_int {
    if crate::vars::nvim_get_cpt_sources_index() == -1 {
        return 0;
    }

    let mut p = cpt;
    // Skip delimiters: ',' and ' '
    #[allow(clippy::cast_possible_wrap)]
    while !p.is_null() && (*p == b',' as c_char || *p == b' ' as c_char) {
        p = p.add(1);
    }

    c_int::from(!p.is_null() && *p != 0)
}

// =============================================================================
// Phase 2 helper: get effective cot_flags
// =============================================================================

unsafe fn get_cot_flags() -> u32 {
    let b = nvim_curbuf_get_b_cot_flags();
    if b != 0 {
        b
    } else {
        crate::vars::nvim_get_cot_flags_global()
    }
}

// =============================================================================
// Phase 2: rs_ins_compl_prep
// =============================================================================

/// Prepare for Insert mode completion, or stop it.
/// Called just after typing a character in Insert mode.
///
/// Returns 1 when the character is not to be inserted, 0 otherwise.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_ins_compl_prep(c: c_int) -> c_int {
    let mut retval = false;
    let prev_mode = crate::vars::nvim_get_ctrl_x_mode();

    // Forget any previous 'special' messages if this is actually a ^X mode key
    // - bar ^R, in which case we wait to see what it gives us.
    if c != CTRL_R && crate::rs_vim_is_ctrl_x_key(c) != 0 {
        g_edit_submode_extra = core::ptr::null_mut();
    }

    // Ignore end of Select mode mapping and mouse scroll/movement.
    if c == K_SELECT
        || c == K_MOUSEDOWN
        || c == K_MOUSEUP
        || c == K_MOUSELEFT
        || c == K_MOUSERIGHT
        || c == K_MOUSEMOVE
        || c == K_EVENT
        || c == K_COMMAND
        || c == K_LUA
    {
        return 0;
    }

    if crate::vars::nvim_get_ctrl_x_mode() == CTRL_X_CMDLINE_CTRL_X && c != CTRL_X {
        // In all cases, drop back to CMDLINE mode first.
        crate::vars::nvim_set_ctrl_x_mode(CTRL_X_CMDLINE);
        if c == CTRL_V
            || c == CTRL_Q
            || c == CTRL_Z
            || rs_ins_compl_pum_key(c) != 0
            || crate::rs_vim_is_ctrl_x_key(c) == 0
        {
            // Not starting another completion mode.
            // CTRL-X CTRL-Z should stop completion without inserting anything.
            if c == CTRL_Z {
                retval = true;
            }
        } else {
            // Other CTRL-X keys first stop completion, then start another
            // completion mode.
            rs_ins_compl_prep(c_int::from(b' '));
            crate::vars::nvim_set_ctrl_x_mode(CTRL_X_NOT_DEFINED_YET);
        }
    }

    // Set "compl_get_longest" when finding the first matches.
    if crate::vars::nvim_get_ctrl_x_mode() == CTRL_X_NOT_DEFINED_YET
        || (crate::vars::nvim_get_ctrl_x_mode() == CTRL_X_NORMAL
            && crate::vars::nvim_get_compl_started() == 0)
    {
        let longest = (get_cot_flags() & K_OPT_COT_FLAG_LONGEST) != 0;
        crate::vars::nvim_set_compl_get_longest(c_int::from(longest));
        crate::vars::nvim_set_compl_used_match(1);
    }

    if crate::vars::nvim_get_ctrl_x_mode() == CTRL_X_NOT_DEFINED_YET {
        // We have just typed CTRL-X and aren't sure which mode yet. Now decide.
        retval = rs_set_ctrl_x_mode(c) != 0;
    } else if crate::vars::nvim_get_ctrl_x_mode() != CTRL_X_NORMAL {
        // We're already in CTRL-X mode, do we stay in it?
        if crate::rs_vim_is_ctrl_x_key(c) == 0 {
            let new_mode = if crate::vars::nvim_get_ctrl_x_mode() == CTRL_X_SCROLL {
                CTRL_X_NORMAL
            } else {
                CTRL_X_FINISHED
            };
            crate::vars::nvim_set_ctrl_x_mode(new_mode);
            g_edit_submode = core::ptr::null_mut();
        }
        g_redraw_mode = true;
    }

    if crate::vars::nvim_get_compl_started() != 0
        || crate::vars::nvim_get_ctrl_x_mode() == CTRL_X_FINISHED
    {
        // Show error message from attempted keyword completion until another key
        // is hit, then go back to showing what mode we are in.
        g_redraw_mode = true;
        if (crate::vars::nvim_get_ctrl_x_mode() == CTRL_X_NORMAL
            && c != CTRL_N
            && c != CTRL_P
            && c != CTRL_R
            && rs_ins_compl_pum_key(c) == 0)
            || crate::vars::nvim_get_ctrl_x_mode() == CTRL_X_FINISHED
        {
            retval = rs_ins_compl_stop(c, prev_mode, c_int::from(retval)) != 0;
        }
    } else if crate::vars::nvim_get_ctrl_x_mode() == CTRL_X_LOCAL_MSG {
        // Trigger the CompleteDone event to give scripts a chance to act upon
        // the (possibly failed) completion.
        rs_do_autocmd_completedone(c, crate::vars::nvim_get_ctrl_x_mode(), std::ptr::null_mut());
    }

    may_trigger_modechanged();

    // reset continue_* if we left expansion mode; if we stay they'll be
    // (re)set properly in ins_complete()
    if crate::rs_vim_is_ctrl_x_key(c) == 0 {
        crate::vars::nvim_set_compl_cont_status(0);
        crate::vars::nvim_set_compl_cont_mode(0);
    }

    c_int::from(retval)
}

// =============================================================================
// Phase 3: rs_ins_compl_stop
// =============================================================================

/// Stop insert completion mode.
///
/// Finalizes completion: handles redo buffer fixup, indent correction,
/// CTRL-Y/CTRL-E acceptance, fires CompleteDonePre/CompleteDone events,
/// resets all completion state.
///
/// Returns 1 when the character should not be inserted, 0 otherwise.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_ins_compl_stop(c: c_int, prev_mode: c_int, retval: c_int) -> c_int {
    let mut retval = retval != 0;

    // Remove pre-inserted text when present.
    if rs_ins_compl_preinsert_effect() != 0 && rs_ins_compl_win_active(nvim_get_curwin()) != 0 {
        crate::insert::rs_ins_compl_delete(0);
    }

    // Get here when we have finished typing a sequence of ^N and ^P or other
    // completion characters in CTRL-X mode. Free up memory that was used, and
    // make sure we can redo the insert.
    let curr_match_str = crate::match_list::curr_match_cp_str_data();
    let leader_data = crate::vars::nvim_get_compl_leader_data();
    if !curr_match_str.is_null() || !leader_data.is_null() || c == CTRL_E {
        // If any of the original typed text has been changed (e.g. when
        // ignorecase is set), we must add back-spaces to the redo buffer.
        // When using the longest match, edited the match or used CTRL-E then
        // don't use the current match.
        let ptr: *const c_char = if !curr_match_str.is_null()
            && crate::vars::nvim_get_compl_used_match() != 0
            && c != CTRL_E
        {
            curr_match_str
        } else {
            std::ptr::null()
        };
        rs_ins_compl_fixRedoBufForLeader(ptr);
    }

    let mut want_cindent = get_can_cindent() && cindent_on();

    // When completing whole lines: fix indent for 'cindent'.
    // Otherwise, break line if it's too long.
    if crate::vars::nvim_get_compl_cont_mode() == CTRL_X_WHOLE_LINE {
        // re-indent the current line
        if want_cindent {
            do_c_expr_indent();
            want_cindent = false; // don't do it again
        }
    } else {
        let prev_col = nvim_get_cursor_col();

        // put the cursor on the last char, for 'tw' formatting
        if prev_col > 0 {
            dec_cursor();
        }

        // only format when something was inserted
        if nvim_get_arrow_used() == 0 && !ins_need_undo_get() && c != CTRL_E {
            insertchar(0 /* NUL */, 0, -1);
        }

        if prev_col > 0 && nvim_cursor_on_nul() != 0 {
            inc_cursor();
        }
    }

    let mut word: *mut u8 = std::ptr::null_mut();

    // If the popup menu is displayed pressing CTRL-Y means accepting the
    // selection without inserting anything. When compl_enter_selects is set
    // the Enter key does the same.
    if (c == CTRL_Y
        || (crate::vars::nvim_get_compl_enter_selects() != 0
            && (c == CAR || c == K_KENTER || c == NL)))
        && pum_visible() != 0
    {
        word = nvim_get_compl_shown_match_str_dup().cast::<u8>();
        retval = true;
        // May need to remove ComplMatchIns highlight.
        redrawWinline(nvim_get_curwin(), nvim_get_curwin_cursor_lnum());
    }

    // CTRL-E means completion is Ended, go back to the typed text.
    // but only do this if the popup is still visible.
    if c == CTRL_E {
        crate::insert::rs_ins_compl_delete(0);
        let mut p: *const c_char = std::ptr::null();
        let mut plen: usize = 0;
        let leader = crate::vars::nvim_get_compl_leader_data();
        if !leader.is_null() {
            p = leader;
            plen = crate::vars::nvim_get_compl_leader_size();
        } else if !crate::match_list::compl_first_match.is_null() {
            p = crate::vars::nvim_get_compl_orig_text_data();
            plen = crate::vars::nvim_get_compl_orig_text_size();
        }
        if !p.is_null() {
            let compl_len = rs_get_compl_len();
            #[allow(
                clippy::cast_possible_wrap,
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss
            )]
            if (plen as c_int) > compl_len {
                nvim_ins_compl_insert_bytes(p.add(compl_len as usize), (plen as c_int) - compl_len);
            }
        }
        nvim_restore_orig_extmarks();
        retval = true;
    }

    auto_format(false, true);

    // Trigger the CompleteDonePre event to give scripts a chance to act upon
    // the completion before clearing the info, and restore ctrl_x_mode so
    // that complete_info() can be used.
    crate::vars::nvim_set_ctrl_x_mode(prev_mode);
    nvim_ins_apply_autocmds_completedonepre();

    rs_ins_compl_free();
    crate::vars::nvim_set_compl_started(0);
    crate::vars::nvim_set_compl_matches(0);
    if !nvim_shortmess_completionmenu() {
        msg_clr_cmdline(); // necessary for "noshowmode"
    }
    crate::vars::nvim_set_ctrl_x_mode(CTRL_X_NORMAL);
    crate::vars::nvim_set_compl_enter_selects(0);
    nvim_set_edit_submode_null_if_set();
    crate::vars::nvim_set_compl_autocomplete(0);
    crate::vars::nvim_set_compl_from_nonkeyword(0);
    crate::vars::nvim_clear_compl_best_matches();
    crate::vars::nvim_set_compl_ins_end_col(0);

    if c == CTRL_C && nvim_get_cmdwin_type() != 0 {
        // Avoid the popup menu remaining displayed when leaving the command
        // line window.
        update_screen();
    }

    // Indent now if a key was typed that is in 'cinkeys'.
    if want_cindent && nvim_in_cinkeys_key_complete(c_int::from(b' '), inindent(0)) {
        do_c_expr_indent();
    }

    // Trigger the CompleteDone event to give scripts a chance to act upon the
    // end of completion.
    rs_do_autocmd_completedone(c, prev_mode, word.cast::<c_char>());
    nvim_xfree(word);

    c_int::from(retval)
}

// =============================================================================
// Phase 3: rs_ins_compl_cancel
// =============================================================================

/// Cancel completion: calls rs_ins_compl_stop(' ', ctrl_x_mode, true).
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_cancel() -> c_int {
    rs_ins_compl_stop(c_int::from(b' '), crate::vars::nvim_get_ctrl_x_mode(), 1)
}

extern "C" {
    fn inindent(extra: c_int) -> bool;
}

// =============================================================================
// Phase 4: rs_ins_ctrl_x
// =============================================================================

// Additional extern "C" declarations for Phase 4
extern "C" {
    fn nvim_set_edit_submode_ctrl_x_msg(mode: c_int);
    fn nvim_may_trigger_modechanged();
}

// Additional constants for ins_ctrl_x
const CONT_N_ADDS: c_int = 4; // next ^X<> will add-new or expand-current
                              // CONT_INTRPT is already defined above as 6

/// Handle CTRL-X keypress in insert mode.
///
/// If not in cmdline CTRL-X mode: resets or sets interrupt flag in compl_cont_status,
/// sets ctrl_x_mode to NOT_DEFINED_YET, updates edit_submode and triggers modechanged.
/// If in cmdline CTRL-X mode: sets ctrl_x_mode to CTRL_X_CMDLINE_CTRL_X.
///
/// # Safety
/// Requires valid global state.
#[export_name = "ins_ctrl_x"]
pub unsafe extern "C" fn rs_ins_ctrl_x() {
    if rs_ctrl_x_mode_cmdline() == 0 {
        // if the next ^X<> won't ADD nothing, then reset compl_cont_status
        let status = crate::vars::nvim_get_compl_cont_status();
        if (status & CONT_N_ADDS) != 0 {
            crate::vars::nvim_set_compl_cont_status(status | CONT_INTRPT);
        } else {
            crate::vars::nvim_set_compl_cont_status(0);
        }
        // We're not sure which CTRL-X mode it will be yet
        crate::vars::nvim_set_ctrl_x_mode(CTRL_X_NOT_DEFINED_YET);
        nvim_set_edit_submode_ctrl_x_msg(CTRL_X_NOT_DEFINED_YET);
        g_edit_submode_pre = core::ptr::null_mut();
        g_redraw_mode = true;
    } else {
        // CTRL-X in CTRL-X CTRL-V mode behaves differently to make CTRL-X
        // CTRL-V look like CTRL-N
        crate::vars::nvim_set_ctrl_x_mode(CTRL_X_CMDLINE_CTRL_X);
    }

    nvim_may_trigger_modechanged();
}

// =============================================================================
// Phase 4: rs_check_compl_option
// =============================================================================

// Additional extern "C" declarations for check_compl_option
extern "C" {
    fn nvim_check_compl_option_dict() -> c_int;
    fn nvim_check_compl_option_tsr() -> c_int;
    fn nvim_emsg_dict_empty(is_dict: c_int);
    fn nvim_emsg_silent_is_zero() -> c_int;
    fn nvim_in_assert_fails() -> bool;
    fn nvim_vim_beep_complete();
    fn setcursor();
    fn nvim_ui_has_messages() -> c_int;
    fn nvim_ui_flush();
    fn nvim_os_delay(ms: std::os::raw::c_long, allow_input: bool);
}

/// Check that the 'dictionary' or 'thesaurus' option can be used.
///
/// Returns 1 if the option is usable, 0 if it is empty.
/// If empty: resets ctrl_x_mode, clears edit_submode, emits error, shows beep.
///
/// `dict_opt` is 1 for 'dictionary', 0 for 'thesaurus'.
///
/// # Safety
/// Requires valid global state.
#[export_name = "check_compl_option"]
#[must_use]
pub unsafe extern "C" fn rs_check_compl_option(dict_opt: c_int) -> c_int {
    let is_empty = if dict_opt != 0 {
        nvim_check_compl_option_dict()
    } else {
        nvim_check_compl_option_tsr()
    };

    if is_empty != 0 {
        crate::vars::nvim_set_ctrl_x_mode(CTRL_X_NORMAL);
        g_edit_submode = core::ptr::null_mut();
        nvim_emsg_dict_empty(dict_opt);
        if nvim_emsg_silent_is_zero() != 0 && !nvim_in_assert_fails() {
            nvim_vim_beep_complete();
            setcursor();
            if nvim_ui_has_messages() == 0 {
                nvim_ui_flush();
                nvim_os_delay(2004, false);
            }
        }
        return 0;
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constants() {
        assert_eq!(CTRL_X_NORMAL, 0);
        assert_eq!(CTRL_X_NOT_DEFINED_YET, 1);
        assert_eq!(CTRL_X_SCROLL, 2);
        assert_eq!(CTRL_X_WHOLE_LINE, 3);
        assert_eq!(CTRL_X_FILES, 4);
        assert_eq!(CTRL_X_EVAL, 16);
        assert_eq!(CTRL_X_CMDLINE_CTRL_X, 17);
        assert_eq!(CTRL_X_BUFNAMES, 18);
        assert_eq!(CTRL_X_REGISTER, 19);
    }

    #[test]
    fn test_key_code_constants() {
        // K_S_TAB = TERMCAP2KEY('k', 'B')
        assert_eq!(K_S_TAB, -(107 + (66 << 8)));
    }

    #[test]
    fn test_cont_flags() {
        assert_eq!(CONT_INTRPT, 6);
        assert_eq!(CONT_LOCAL, 32);
    }
}
