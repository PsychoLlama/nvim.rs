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
    fn nvim_set_ctrl_x_mode(val: c_int);
    fn nvim_set_compl_cont_mode(val: c_int);
    fn nvim_set_compl_cont_status(val: c_int);
    fn nvim_get_compl_cont_status() -> c_int;
    fn nvim_get_compl_cont_mode() -> c_int;
    fn nvim_set_edit_submode_scroll(is_replace: c_int);
    fn nvim_set_edit_submode_null();
    fn nvim_set_redraw_mode_true();
    fn nvim_get_state_replace_flag() -> c_int;
    fn nvim_spell_back_safe();
    fn nvim_vpeekc() -> c_int;
    fn nvim_get_cpt_sources_index() -> c_int;

    // Phase 2 accessors
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_started() -> c_int;
    fn nvim_set_compl_used_match(val: c_int);
    fn nvim_set_compl_get_longest(val: c_int);
    fn nvim_get_cot_flags_global() -> u32;
    fn nvim_curbuf_get_b_cot_flags() -> u32;
    fn nvim_clear_edit_submode_extra();

    // Phase 2: C functions called (not pure accessors)
    fn vim_is_ctrl_x_key(c: c_int) -> bool;
    fn ins_compl_stop(c: c_int, prev_mode: c_int, retval: bool) -> bool;
    fn do_autocmd_completedone(c: c_int, mode: c_int, word: *mut c_char);
    fn may_trigger_modechanged();
    fn rs_ins_compl_pum_key(c: c_int) -> c_int;
}

// =============================================================================
// Phase 1: rs_set_ctrl_x_mode
// =============================================================================

/// Set the CTRL-X completion mode based on the key typed after CTRL-X.
///
/// Translates a key character into the appropriate completion mode and updates
/// global state (ctrl_x_mode, edit_submode, compl_cont_mode, compl_cont_status).
///
/// Returns 1 when the character is not to be inserted (i.e., CTRL-Z was typed),
/// 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_set_ctrl_x_mode(c: c_int) -> c_int {
    let mut retval = false;

    match c {
        CTRL_E | CTRL_Y => {
            // scroll the window one line up or down
            nvim_set_ctrl_x_mode(CTRL_X_SCROLL);
            nvim_set_edit_submode_scroll(nvim_get_state_replace_flag());
            // edit_submode_pre = NULL and redraw_mode = true are set by the accessor
        }
        CTRL_L => {
            nvim_set_ctrl_x_mode(CTRL_X_WHOLE_LINE);
        }
        CTRL_F => {
            nvim_set_ctrl_x_mode(CTRL_X_FILES);
        }
        CTRL_K => {
            nvim_set_ctrl_x_mode(CTRL_X_DICTIONARY);
        }
        CTRL_R => {
            // When CTRL-R is followed by '=', don't trigger register completion.
            if nvim_vpeekc() != i32::from(b'=') {
                nvim_set_ctrl_x_mode(CTRL_X_REGISTER);
            }
            // else: do nothing, retval stays false
        }
        CTRL_T => {
            nvim_set_ctrl_x_mode(CTRL_X_THESAURUS);
        }
        CTRL_U => {
            nvim_set_ctrl_x_mode(CTRL_X_FUNCTION);
        }
        CTRL_O => {
            nvim_set_ctrl_x_mode(CTRL_X_OMNI);
        }
        x if x == i32::from(b's') || x == CTRL_S => {
            nvim_set_ctrl_x_mode(CTRL_X_SPELL);
            nvim_spell_back_safe();
        }
        CTRL_RSB => {
            nvim_set_ctrl_x_mode(CTRL_X_TAGS);
        }
        CTRL_I | K_S_TAB => {
            nvim_set_ctrl_x_mode(CTRL_X_PATH_PATTERNS);
        }
        CTRL_D => {
            nvim_set_ctrl_x_mode(CTRL_X_PATH_DEFINES);
        }
        CTRL_V | CTRL_Q => {
            nvim_set_ctrl_x_mode(CTRL_X_CMDLINE);
        }
        CTRL_Z => {
            nvim_set_ctrl_x_mode(CTRL_X_NORMAL);
            nvim_set_edit_submode_null();
            nvim_set_redraw_mode_true();
            retval = true;
        }
        CTRL_P | CTRL_N => {
            // ^X^P means LOCAL expansion if nothing interrupted.
            // Do normal expansion when interrupting a different mode.
            // Nothing changes if interrupting mode 0.
            let cont_status = nvim_get_compl_cont_status();
            if (cont_status & CONT_INTRPT) == 0 {
                nvim_set_compl_cont_status(cont_status | CONT_LOCAL);
            } else if nvim_get_compl_cont_mode() != 0 {
                nvim_set_compl_cont_status(cont_status & !CONT_LOCAL);
            }
            // FALLTHROUGH to default logic
            set_ctrl_x_mode_default(c);
        }
        _ => {
            set_ctrl_x_mode_default(c);
        }
    }

    c_int::from(retval)
}

/// Shared default logic for CTRL-X mode (handles CTRL-X and all other keys).
///
/// Called from the Ctrl_P/Ctrl_N arm (after local-mode flag adjustment)
/// and from the default arm.
unsafe fn set_ctrl_x_mode_default(c: c_int) {
    if c == CTRL_X {
        if nvim_get_compl_cont_mode() != 0 {
            nvim_set_compl_cont_status(0);
        } else {
            nvim_set_compl_cont_mode(CTRL_X_NOT_DEFINED_YET);
        }
    }
    nvim_set_ctrl_x_mode(CTRL_X_NORMAL);
    nvim_set_edit_submode_null();
    nvim_set_redraw_mode_true();
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
    if nvim_get_cpt_sources_index() == -1 {
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
        nvim_get_cot_flags_global()
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
    let prev_mode = nvim_get_ctrl_x_mode();

    // Forget any previous 'special' messages if this is actually a ^X mode key
    // - bar ^R, in which case we wait to see what it gives us.
    if c != CTRL_R && vim_is_ctrl_x_key(c) {
        nvim_clear_edit_submode_extra();
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

    if nvim_get_ctrl_x_mode() == CTRL_X_CMDLINE_CTRL_X && c != CTRL_X {
        // In all cases, drop back to CMDLINE mode first.
        nvim_set_ctrl_x_mode(CTRL_X_CMDLINE);
        if c == CTRL_V
            || c == CTRL_Q
            || c == CTRL_Z
            || rs_ins_compl_pum_key(c) != 0
            || !vim_is_ctrl_x_key(c)
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
            nvim_set_ctrl_x_mode(CTRL_X_NOT_DEFINED_YET);
        }
    }

    // Set "compl_get_longest" when finding the first matches.
    if nvim_get_ctrl_x_mode() == CTRL_X_NOT_DEFINED_YET
        || (nvim_get_ctrl_x_mode() == CTRL_X_NORMAL && nvim_get_compl_started() == 0)
    {
        let longest = (get_cot_flags() & K_OPT_COT_FLAG_LONGEST) != 0;
        nvim_set_compl_get_longest(c_int::from(longest));
        nvim_set_compl_used_match(1);
    }

    if nvim_get_ctrl_x_mode() == CTRL_X_NOT_DEFINED_YET {
        // We have just typed CTRL-X and aren't sure which mode yet. Now decide.
        retval = rs_set_ctrl_x_mode(c) != 0;
    } else if nvim_get_ctrl_x_mode() != CTRL_X_NORMAL {
        // We're already in CTRL-X mode, do we stay in it?
        if !vim_is_ctrl_x_key(c) {
            let new_mode = if nvim_get_ctrl_x_mode() == CTRL_X_SCROLL {
                CTRL_X_NORMAL
            } else {
                CTRL_X_FINISHED
            };
            nvim_set_ctrl_x_mode(new_mode);
            nvim_set_edit_submode_null();
        }
        nvim_set_redraw_mode_true();
    }

    if nvim_get_compl_started() != 0 || nvim_get_ctrl_x_mode() == CTRL_X_FINISHED {
        // Show error message from attempted keyword completion until another key
        // is hit, then go back to showing what mode we are in.
        nvim_set_redraw_mode_true();
        if (nvim_get_ctrl_x_mode() == CTRL_X_NORMAL
            && c != CTRL_N
            && c != CTRL_P
            && c != CTRL_R
            && rs_ins_compl_pum_key(c) == 0)
            || nvim_get_ctrl_x_mode() == CTRL_X_FINISHED
        {
            retval = ins_compl_stop(c, prev_mode, retval);
        }
    } else if nvim_get_ctrl_x_mode() == CTRL_X_LOCAL_MSG {
        // Trigger the CompleteDone event to give scripts a chance to act upon
        // the (possibly failed) completion.
        do_autocmd_completedone(c, nvim_get_ctrl_x_mode(), std::ptr::null_mut());
    }

    may_trigger_modechanged();

    // reset continue_* if we left expansion mode; if we stay they'll be
    // (re)set properly in ins_complete()
    if !vim_is_ctrl_x_key(c) {
        nvim_set_compl_cont_status(0);
        nvim_set_compl_cont_mode(0);
    }

    c_int::from(retval)
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
