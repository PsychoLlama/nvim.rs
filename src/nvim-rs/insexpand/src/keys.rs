//! Completion key handling support.
//!
//! This module provides helper functions for handling keys during completion.
//! The actual key processing remains in C, but Rust provides utilities.

#![allow(dead_code, unused_imports)]
use std::os::raw::c_int;
use std::sync::atomic::{AtomicI32, Ordering};

// C accessor functions

// Key constants (from keycode definitions)
const CTRL_N: c_int = 14; // ^N
const CTRL_P: c_int = 16; // ^P
const CTRL_Y: c_int = 25; // ^Y
const CTRL_E: c_int = 5; // ^E
const BS: c_int = 8; // Backspace
const TAB: c_int = 9; // Tab
const CR: c_int = 13; // Carriage return / Enter
const ESC: c_int = 27; // Escape

// =============================================================================
// Phase 7: Extended Key Handling Functions
// =============================================================================

// Additional C accessor functions
extern "C" {
    fn nvim_get_cursor_col() -> c_int;
}

// CTRL-X mode constants
const CTRL_X_OMNI: c_int = 13;
const CTRL_X_EVAL: c_int = 16;

// Special key codes (from keycode definitions)
// TERMCAP2KEY(a, b) = -((a) + ((b) << 8))
const K_UP: c_int = -30059; // TERMCAP2KEY('k', 'u') = -(107 + 117*256)
const K_DOWN: c_int = -25707; // TERMCAP2KEY('k', 'd') = -(107 + 100*256)
const K_PAGEUP: c_int = -48;
const K_PAGEDOWN: c_int = -49;
const K_S_UP: c_int = -50;
const K_S_DOWN: c_int = -51;
// K_IGNORE = TERMCAP2KEY(KS_EXTRA=253, KE_IGNORE=53) = -(253 + 53*256) = -13821
const K_IGNORE: c_int = -13821;

// =============================================================================
// Phase 4: rs_ins_compl_check_keys
// =============================================================================

// Static counter equivalent to the C static local variable
static CHECK_KEYS_COUNT: AtomicI32 = AtomicI32::new(0);

// Additional C accessor functions for Phase 4
extern "C" {
    fn vpeekc_any() -> c_int;
    #[link_name = "test_disable_char_avail"]
    static mut nvim_test_disable_char_avail: bool;
    fn safe_vgetc() -> c_int;
    fn vungetc(c: c_int);
    #[link_name = "got_int"]
    static mut nvim_got_int: bool;
    #[link_name = "KeyTyped"]
    static mut nvim_key_typed: bool;
    #[link_name = "using_script"]
    fn nvim_using_script() -> c_int;
    #[link_name = "ex_normal_busy"]
    static mut nvim_ex_normal_busy: c_int;
    // (compl_pending moved to Rust static in state.rs)
    fn nvim_cot_flags_has_noinsert_fuzzy() -> c_int;
    // (nvim_cpt_sources_array_exists: inlined in vars.rs Phase 23)
    // nvim_p_cto: inlined in vars.rs (Phase 29)
    fn rs_ctrl_x_mode_normal() -> c_int;
    fn rs_ctrl_x_mode_line_or_eval() -> c_int;
    fn rs_check_elapsed_time();
    fn rs_ins_compl_has_preinsert() -> c_int;
}

// Continuation status flags
const CONT_LOCAL: c_int = 32;

// Control key constants
const CTRL_X: c_int = 24;
const CTRL_R: c_int = 18;
const NUL: c_int = 0;

/// Check for user keystrokes during completion search.
///
/// Called at regular intervals (every `frequency` calls) during completion
/// to allow the user to interrupt or navigate completions.
///
/// `in_compl_func` is non-zero when called from `complete_check()`.
///
/// # Safety
/// Requires valid completion state and input handling state.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_check_keys(frequency: c_int, in_compl_func: c_int) {
    // Don't check when reading keys from a script, :normal or feedkeys().
    // That would break the test scripts. But do check for keys when called
    // from complete_check().
    if in_compl_func == 0 && (nvim_using_script() != 0 || nvim_ex_normal_busy != 0) {
        return;
    }

    // Only do this at regular intervals
    let count = CHECK_KEYS_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
    if count < frequency {
        return;
    }
    CHECK_KEYS_COUNT.store(0, Ordering::Relaxed);

    // Check for a typed key. Do use mappings, otherwise vim_is_ctrl_x_key()
    // can't do its work correctly.
    let peeked = vpeekc_any();
    if peeked != NUL && !nvim_test_disable_char_avail {
        // Eat or inspect the character
        let c = safe_vgetc();
        if crate::rs_vim_is_ctrl_x_key(peeked) != 0 && peeked != CTRL_X && peeked != CTRL_R {
            crate::vars::nvim_set_compl_shows_dir(crate::rs_ins_compl_key2dir(c));
            let todo = crate::rs_ins_compl_key2count(c);
            let advance = c_int::from(c != K_UP && c != K_DOWN);
            crate::next::rs_ins_compl_next(0, todo, advance);
        } else {
            // Need to have KeyTyped set.  We'll put it back with vungetc() below.
            // But skip K_IGNORE.
            if c != K_IGNORE {
                // Don't interrupt completion when the character wasn't typed,
                // e.g., when doing @q to replay keys.
                if c != CTRL_R && nvim_key_typed {
                    crate::vars::nvim_set_compl_interrupted(1);
                }
                vungetc(c);
            }
        }
    } else {
        let normal_mode_strict = rs_ctrl_x_mode_normal() != 0
            && rs_ctrl_x_mode_line_or_eval() == 0
            && (crate::vars::nvim_get_compl_cont_status() & CONT_LOCAL) == 0
            && crate::vars::nvim_cpt_sources_array_exists() != 0
            && crate::vars::nvim_get_cpt_sources_index() >= 0;
        if normal_mode_strict
            && (crate::vars::nvim_get_compl_autocomplete() != 0 || crate::vars::nvim_p_cto() > 0)
        {
            rs_check_elapsed_time();
        }
    }

    let pending = crate::state::COMPL_PENDING;
    if pending != 0
        && !nvim_got_int
        && nvim_cot_flags_has_noinsert_fuzzy() == 0
        && (crate::vars::nvim_get_compl_autocomplete() == 0 || rs_ins_compl_has_preinsert() != 0)
    {
        // Insert the first match immediately and advance compl_shown_match,
        // before finding other matches.
        let todo = pending.abs();
        crate::state::COMPL_PENDING = 0;
        crate::next::rs_ins_compl_next(0, todo, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_constants() {
        assert_eq!(CTRL_N, 14);
        assert_eq!(CTRL_P, 16);
        assert_eq!(CTRL_Y, 25);
        assert_eq!(CTRL_E, 5);
        assert_eq!(BS, 8);
        assert_eq!(TAB, 9);
        assert_eq!(CR, 13);
        assert_eq!(ESC, 27);
    }
}
