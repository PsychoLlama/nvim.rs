//! Spell completion support.
//!
//! This module provides helper functions for spell completion (CTRL-X s).
//! The core spell checking operations remain in C.

#![allow(dead_code, unused_imports)]
use std::os::raw::c_int;

// C accessor functions

// CTRL-X mode constant
const CTRL_X_SPELL: c_int = 14;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constant() {
        assert_eq!(CTRL_X_SPELL, 14);
    }
}
