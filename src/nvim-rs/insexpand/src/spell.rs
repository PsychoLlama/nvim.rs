//! Spell completion support.
//!
//! This module provides helper functions for spell completion (CTRL-X s).
//! The core spell checking operations remain in C.

use std::os::raw::c_int;

// C accessor functions
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_direction() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;
}

// CTRL-X mode constant
const CTRL_X_SPELL: c_int = 14;

/// Check if we're in spell completion mode.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_is_spell_mode() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    c_int::from(mode == CTRL_X_SPELL)
}

/// Check if completion was interrupted during spell search.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_was_interrupted() -> c_int {
    nvim_get_compl_interrupted()
}

/// Get the current completion direction for spell search.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_get_direction() -> c_int {
    nvim_get_compl_direction()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constant() {
        assert_eq!(CTRL_X_SPELL, 14);
    }
}
