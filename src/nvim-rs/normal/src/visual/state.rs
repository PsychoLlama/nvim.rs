//! Visual mode state management - command helpers.
//!
//! This module provides helper functions for visual mode commands:
//! - nv_visual
//! - start_selection
//! - may_start_select
//! - v_* operations

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Visual Mode Constants
// =============================================================================

/// Characterwise visual mode ('v').
pub const VIS_CHAR: c_int = 0;
/// Linewise visual mode ('V').
pub const VIS_LINE: c_int = 1;
/// Blockwise visual mode (CTRL-V).
pub const VIS_BLOCK: c_int = 2;

// =============================================================================
// Visual Selection Constants
// =============================================================================

/// Selection is inclusive (default).
pub const SEL_INCLUSIVE: c_int = 0;
/// Selection is exclusive (like 'selection' option "exclusive").
pub const SEL_EXCLUSIVE: c_int = 1;

// =============================================================================
// Visual Mode State Helpers (Pure Rust)
// =============================================================================

/// Get visual mode character 'v'.
fn visual_char() -> c_int {
    c_int::from(b'v')
}

/// Get visual line character 'V'.
fn visual_line_char() -> c_int {
    c_int::from(b'V')
}

/// Get visual block character (CTRL-V = 0x16).
fn visual_block_char() -> c_int {
    0x16 // CTRL-V
}

/// Check if character starts visual mode.
fn is_visual_cmd(cmdchar: c_int) -> bool {
    // 0x16 is CTRL-V
    cmdchar == c_int::from(b'v') || cmdchar == c_int::from(b'V') || cmdchar == 0x16
}

/// Get visual mode type from command character.
fn get_visual_mode(cmdchar: c_int) -> c_int {
    if cmdchar == c_int::from(b'v') {
        VIS_CHAR
    } else if cmdchar == c_int::from(b'V') {
        VIS_LINE
    } else if cmdchar == 0x16 {
        // CTRL-V
        VIS_BLOCK
    } else {
        VIS_CHAR // default
    }
}

/// Check if visual mode is characterwise.
fn is_visual_char_mode(mode: c_int) -> bool {
    mode == VIS_CHAR
}

/// Check if visual mode is linewise.
fn is_visual_line_mode(mode: c_int) -> bool {
    mode == VIS_LINE
}

/// Check if visual mode is blockwise.
fn is_visual_block_mode(mode: c_int) -> bool {
    mode == VIS_BLOCK
}

/// Check if selection should be exclusive.
fn is_selection_exclusive(sel_mode: c_int) -> bool {
    sel_mode == SEL_EXCLUSIVE
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get VIS_CHAR constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_vis_char_mode() -> c_int {
    VIS_CHAR
}

/// FFI: Get VIS_LINE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_vis_line_mode() -> c_int {
    VIS_LINE
}

/// FFI: Get VIS_BLOCK constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_vis_block_mode() -> c_int {
    VIS_BLOCK
}

/// FFI: Get SEL_INCLUSIVE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_sel_inclusive() -> c_int {
    SEL_INCLUSIVE
}

/// FFI: Get SEL_EXCLUSIVE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_sel_exclusive() -> c_int {
    SEL_EXCLUSIVE
}

/// FFI: Get visual mode character 'v'.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_visual_char() -> c_int {
    visual_char()
}

/// FFI: Get visual line character 'V'.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_visual_line_char() -> c_int {
    visual_line_char()
}

/// FFI: Get visual block character (CTRL-V).
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_visual_block_char() -> c_int {
    visual_block_char()
}

/// FFI: Check if character starts visual mode.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_is_visual_cmd(cmdchar: c_int) -> c_int {
    c_int::from(is_visual_cmd(cmdchar))
}

/// FFI: Get visual mode type from command character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_visual_mode_from_char(cmdchar: c_int) -> c_int {
    get_visual_mode(cmdchar)
}

/// FFI: Check if visual mode is characterwise.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_is_visual_char(mode: c_int) -> c_int {
    c_int::from(is_visual_char_mode(mode))
}

/// FFI: Check if visual mode is linewise.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_is_visual_line(mode: c_int) -> c_int {
    c_int::from(is_visual_line_mode(mode))
}

/// FFI: Check if visual mode is blockwise.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_is_visual_block(mode: c_int) -> c_int {
    c_int::from(is_visual_block_mode(mode))
}

/// FFI: Check if selection should be exclusive.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_is_selection_exclusive(sel_mode: c_int) -> c_int {
    c_int::from(is_selection_exclusive(sel_mode))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_mode_constants() {
        assert_eq!(VIS_CHAR, 0);
        assert_eq!(VIS_LINE, 1);
        assert_eq!(VIS_BLOCK, 2);
    }

    #[test]
    fn test_selection_constants() {
        assert_eq!(SEL_INCLUSIVE, 0);
        assert_eq!(SEL_EXCLUSIVE, 1);
    }

    #[test]
    fn test_visual_chars() {
        assert_eq!(visual_char(), c_int::from(b'v'));
        assert_eq!(visual_line_char(), c_int::from(b'V'));
        assert_eq!(visual_block_char(), 0x16);
    }

    #[test]
    fn test_is_visual_cmd() {
        assert!(is_visual_cmd(c_int::from(b'v')));
        assert!(is_visual_cmd(c_int::from(b'V')));
        assert!(is_visual_cmd(0x16));
        assert!(!is_visual_cmd(c_int::from(b'd')));
        assert!(!is_visual_cmd(c_int::from(b'y')));
    }

    #[test]
    fn test_get_visual_mode() {
        assert_eq!(get_visual_mode(c_int::from(b'v')), VIS_CHAR);
        assert_eq!(get_visual_mode(c_int::from(b'V')), VIS_LINE);
        assert_eq!(get_visual_mode(0x16), VIS_BLOCK);
        assert_eq!(get_visual_mode(c_int::from(b'x')), VIS_CHAR); // default
    }

    #[test]
    fn test_visual_mode_checks() {
        assert!(is_visual_char_mode(VIS_CHAR));
        assert!(!is_visual_char_mode(VIS_LINE));
        assert!(is_visual_line_mode(VIS_LINE));
        assert!(!is_visual_line_mode(VIS_BLOCK));
        assert!(is_visual_block_mode(VIS_BLOCK));
        assert!(!is_visual_block_mode(VIS_CHAR));
    }

    #[test]
    fn test_selection_exclusive() {
        assert!(!is_selection_exclusive(SEL_INCLUSIVE));
        assert!(is_selection_exclusive(SEL_EXCLUSIVE));
    }
}
