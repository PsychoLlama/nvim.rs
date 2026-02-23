//! Tag completion support.
//!
//! This module provides helper functions for tag completion (CTRL-X CTRL-]).
//! The core tag lookup operations remain in C.

#![allow(dead_code, unused_imports)]
use std::os::raw::c_int;

// C accessor functions
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_direction() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;
}

// CTRL-X mode constant
const CTRL_X_TAGS: c_int = 5 + 0x100; // 5 + CTRL_X_WANT_IDENT

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constant() {
        assert_eq!(CTRL_X_TAGS, 5 + 0x100);
    }
}
