//! CTRL-X mode state machine and completion lifecycle functions.
//!
//! This module implements:
//! - `rs_set_ctrl_x_mode`: validates a key after CTRL-X and sets the completion mode
//! - `rs_may_advance_cpt_index`: checks if the 'complete' option index can advance

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

// K_S_TAB = TERMCAP2KEY('k', 'B')
const K_S_TAB: c_int = termcap2key(b'k' as c_int, b'B' as c_int);

// =============================================================================
// C accessor FFI declarations (Phase 1)
// =============================================================================

extern "C" {
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
