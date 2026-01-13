//! Visual mode operations.
//!
//! This module provides helper functions for visual operations:
//! - nv_g_cmd (visual g commands)
//! - visual_block_*
//! - charwise/linewise operations

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

use super::state::{VIS_BLOCK, VIS_CHAR, VIS_LINE};

// =============================================================================
// Visual Operation Constants
// =============================================================================

/// Visual selection - extend to end of line.
pub const VSEL_EOL: c_int = 0;
/// Visual selection - extend to start of line.
pub const VSEL_SOL: c_int = 1;
/// Visual selection - extend by word.
pub const VSEL_WORD: c_int = 2;
/// Visual selection - extend by line.
pub const VSEL_LINE: c_int = 3;

// =============================================================================
// Visual G Command Constants
// =============================================================================

/// Visual g command: goto line (gg).
pub const VG_GOTO: c_int = 0;
/// Visual g command: join (gJ).
pub const VG_JOIN: c_int = 1;
/// Visual g command: format (gq).
pub const VG_FORMAT: c_int = 2;
/// Visual g command: substitute (g&).
pub const VG_SUBST: c_int = 3;

// =============================================================================
// Visual Block Constants
// =============================================================================

/// Block insert mode.
pub const VBLOCK_INSERT: c_int = 0;
/// Block append mode.
pub const VBLOCK_APPEND: c_int = 1;
/// Block change mode.
pub const VBLOCK_CHANGE: c_int = 2;

// =============================================================================
// Visual Operation Helpers (Pure Rust)
// =============================================================================

/// Check if operation extends to end of line.
#[allow(dead_code)]
fn is_vsel_eol(vsel: c_int) -> bool {
    vsel == VSEL_EOL
}

/// Check if operation extends to start of line.
#[allow(dead_code)]
fn is_vsel_sol(vsel: c_int) -> bool {
    vsel == VSEL_SOL
}

/// Check if operation extends by word.
#[allow(dead_code)]
fn is_vsel_word(vsel: c_int) -> bool {
    vsel == VSEL_WORD
}

/// Check if operation extends by line.
#[allow(dead_code)]
fn is_vsel_line(vsel: c_int) -> bool {
    vsel == VSEL_LINE
}

/// Get g command character.
fn g_cmd_char() -> c_int {
    c_int::from(b'g')
}

/// Check if visual mode supports block operations.
fn supports_block_ops(mode: c_int) -> bool {
    mode == VIS_BLOCK
}

/// Check if visual mode is character or line based (not block).
fn is_stream_visual(mode: c_int) -> bool {
    mode == VIS_CHAR || mode == VIS_LINE
}

/// Determine if selection should include end character.
fn visual_includes_end(mode: c_int, exclusive: bool) -> bool {
    if exclusive {
        false
    } else {
        mode == VIS_CHAR
    }
}

/// Get block operation type from command character.
fn get_block_op_type(cmdchar: c_int) -> c_int {
    if cmdchar == c_int::from(b'I') {
        VBLOCK_INSERT
    } else if cmdchar == c_int::from(b'A') {
        VBLOCK_APPEND
    } else if cmdchar == c_int::from(b'c') || cmdchar == c_int::from(b'C') {
        VBLOCK_CHANGE
    } else {
        VBLOCK_INSERT // default
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get VSEL_EOL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_vsel_eol() -> c_int {
    VSEL_EOL
}

/// FFI: Get VSEL_SOL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_vsel_sol() -> c_int {
    VSEL_SOL
}

/// FFI: Get VSEL_WORD constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_vsel_word() -> c_int {
    VSEL_WORD
}

/// FFI: Get VSEL_LINE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_vsel_line() -> c_int {
    VSEL_LINE
}

/// FFI: Get VG_GOTO constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_vg_goto() -> c_int {
    VG_GOTO
}

/// FFI: Get VG_JOIN constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_vg_join() -> c_int {
    VG_JOIN
}

/// FFI: Get VG_FORMAT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_vg_format() -> c_int {
    VG_FORMAT
}

/// FFI: Get VBLOCK_INSERT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_vblock_insert() -> c_int {
    VBLOCK_INSERT
}

/// FFI: Get VBLOCK_APPEND constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_vblock_append() -> c_int {
    VBLOCK_APPEND
}

/// FFI: Get VBLOCK_CHANGE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_vblock_change() -> c_int {
    VBLOCK_CHANGE
}

/// FFI: Get g command character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_g_cmd_char() -> c_int {
    g_cmd_char()
}

/// FFI: Check if visual mode supports block operations.
#[unsafe(no_mangle)]
pub extern "C" fn rs_supports_block_ops(mode: c_int) -> c_int {
    c_int::from(supports_block_ops(mode))
}

/// FFI: Check if visual mode is stream-based (char or line).
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_stream_visual(mode: c_int) -> c_int {
    c_int::from(is_stream_visual(mode))
}

/// FFI: Check if selection includes end character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_visual_includes_end(mode: c_int, exclusive: c_int) -> c_int {
    c_int::from(visual_includes_end(mode, exclusive != 0))
}

/// FFI: Get block operation type from command character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_get_block_op_type(cmdchar: c_int) -> c_int {
    get_block_op_type(cmdchar)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vsel_constants() {
        assert_eq!(VSEL_EOL, 0);
        assert_eq!(VSEL_SOL, 1);
        assert_eq!(VSEL_WORD, 2);
        assert_eq!(VSEL_LINE, 3);
    }

    #[test]
    fn test_vg_constants() {
        assert_eq!(VG_GOTO, 0);
        assert_eq!(VG_JOIN, 1);
        assert_eq!(VG_FORMAT, 2);
        assert_eq!(VG_SUBST, 3);
    }

    #[test]
    fn test_vblock_constants() {
        assert_eq!(VBLOCK_INSERT, 0);
        assert_eq!(VBLOCK_APPEND, 1);
        assert_eq!(VBLOCK_CHANGE, 2);
    }

    #[test]
    fn test_vsel_checks() {
        assert!(is_vsel_eol(VSEL_EOL));
        assert!(!is_vsel_eol(VSEL_SOL));
        assert!(is_vsel_sol(VSEL_SOL));
        assert!(is_vsel_word(VSEL_WORD));
        assert!(is_vsel_line(VSEL_LINE));
    }

    #[test]
    fn test_g_cmd_char() {
        assert_eq!(g_cmd_char(), c_int::from(b'g'));
    }

    #[test]
    fn test_supports_block_ops() {
        assert!(supports_block_ops(VIS_BLOCK));
        assert!(!supports_block_ops(VIS_CHAR));
        assert!(!supports_block_ops(VIS_LINE));
    }

    #[test]
    fn test_is_stream_visual() {
        assert!(is_stream_visual(VIS_CHAR));
        assert!(is_stream_visual(VIS_LINE));
        assert!(!is_stream_visual(VIS_BLOCK));
    }

    #[test]
    fn test_visual_includes_end() {
        // Non-exclusive, charwise includes end
        assert!(visual_includes_end(VIS_CHAR, false));
        // Exclusive never includes end
        assert!(!visual_includes_end(VIS_CHAR, true));
        // Non-exclusive, linewise doesn't include end (whole lines)
        assert!(!visual_includes_end(VIS_LINE, false));
        // Non-exclusive, blockwise doesn't include end
        assert!(!visual_includes_end(VIS_BLOCK, false));
    }

    #[test]
    fn test_get_block_op_type() {
        assert_eq!(get_block_op_type(c_int::from(b'I')), VBLOCK_INSERT);
        assert_eq!(get_block_op_type(c_int::from(b'A')), VBLOCK_APPEND);
        assert_eq!(get_block_op_type(c_int::from(b'c')), VBLOCK_CHANGE);
        assert_eq!(get_block_op_type(c_int::from(b'C')), VBLOCK_CHANGE);
        assert_eq!(get_block_op_type(c_int::from(b'x')), VBLOCK_INSERT); // default
    }
}
