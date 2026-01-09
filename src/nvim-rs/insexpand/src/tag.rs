//! Tag completion support.
//!
//! This module provides helper functions for tag completion (CTRL-X CTRL-]).
//! The core tag lookup operations remain in C.

use std::os::raw::c_int;

// C accessor functions
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_direction() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;
}

// CTRL-X mode constant
const CTRL_X_TAGS: c_int = 5 + 0x100; // 5 + CTRL_X_WANT_IDENT

/// Check if we're in tag completion mode.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_is_tags_mode() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    c_int::from(mode == CTRL_X_TAGS)
}

/// Check if completion was interrupted during tag search.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_was_interrupted() -> c_int {
    nvim_get_compl_interrupted()
}

/// Get the current completion direction for tag search.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_get_direction() -> c_int {
    nvim_get_compl_direction()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constant() {
        assert_eq!(CTRL_X_TAGS, 5 + 0x100);
    }
}
