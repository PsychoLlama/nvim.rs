//! Insert mode entry commands.
//!
//! This module provides helper functions for entering insert mode:
//! - nv_edit (i/a/I/A/o/O)
//! - nv_open
//! - nv_append
//! - invoke_edit

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Insert Mode Entry Constants
// =============================================================================

/// Insert before cursor (i command).
pub const INSERT_BEFORE: c_int = 0;
/// Insert after cursor (a command).
pub const INSERT_AFTER: c_int = 1;
/// Insert at start of line (I command).
pub const INSERT_LINE_START: c_int = 2;
/// Insert at end of line (A command).
pub const INSERT_LINE_END: c_int = 3;
/// Open line below (o command).
pub const INSERT_OPEN_BELOW: c_int = 4;
/// Open line above (O command).
pub const INSERT_OPEN_ABOVE: c_int = 5;

// =============================================================================
// Insert State Constants
// =============================================================================

/// Normal insert mode.
pub const INSERT_NORMAL: c_int = 0;
/// Replace mode (R command).
pub const INSERT_REPLACE: c_int = 1;
/// Virtual replace mode (gR command).
pub const INSERT_VREPLACE: c_int = 2;

// =============================================================================
// Insert Mode Entry Helpers (Pure Rust)
// =============================================================================

/// Get insert command character 'i'.
fn insert_char() -> c_int {
    c_int::from(b'i')
}

/// Get append command character 'a'.
fn append_char() -> c_int {
    c_int::from(b'a')
}

/// Get insert-at-start command character 'I'.
#[allow(dead_code)]
fn insert_line_start_char() -> c_int {
    c_int::from(b'I')
}

/// Get append-at-end command character 'A'.
#[allow(dead_code)]
fn append_line_end_char() -> c_int {
    c_int::from(b'A')
}

/// Get open-below command character 'o'.
fn open_below_char() -> c_int {
    c_int::from(b'o')
}

/// Get open-above command character 'O'.
fn open_above_char() -> c_int {
    c_int::from(b'O')
}

/// Get replace command character 'R'.
#[allow(dead_code)]
fn replace_mode_char() -> c_int {
    c_int::from(b'R')
}

/// Check if command enters insert mode.
fn is_insert_cmd(cmdchar: c_int) -> bool {
    cmdchar == c_int::from(b'i')
        || cmdchar == c_int::from(b'a')
        || cmdchar == c_int::from(b'I')
        || cmdchar == c_int::from(b'A')
        || cmdchar == c_int::from(b'o')
        || cmdchar == c_int::from(b'O')
}

/// Check if command is an open command (o/O).
fn is_open_cmd(cmdchar: c_int) -> bool {
    cmdchar == c_int::from(b'o') || cmdchar == c_int::from(b'O')
}

/// Check if command is an append command (a/A).
fn is_append_cmd(cmdchar: c_int) -> bool {
    cmdchar == c_int::from(b'a') || cmdchar == c_int::from(b'A')
}

/// Get insert entry type from command character.
fn get_insert_type(cmdchar: c_int) -> c_int {
    if cmdchar == c_int::from(b'i') {
        INSERT_BEFORE
    } else if cmdchar == c_int::from(b'a') {
        INSERT_AFTER
    } else if cmdchar == c_int::from(b'I') {
        INSERT_LINE_START
    } else if cmdchar == c_int::from(b'A') {
        INSERT_LINE_END
    } else if cmdchar == c_int::from(b'o') {
        INSERT_OPEN_BELOW
    } else if cmdchar == c_int::from(b'O') {
        INSERT_OPEN_ABOVE
    } else {
        INSERT_BEFORE // default
    }
}

/// Check if insert type opens a new line.
fn insert_opens_line(insert_type: c_int) -> bool {
    insert_type == INSERT_OPEN_BELOW || insert_type == INSERT_OPEN_ABOVE
}

/// Check if insert type moves cursor after current position.
fn insert_moves_cursor(insert_type: c_int) -> bool {
    insert_type == INSERT_AFTER || insert_type == INSERT_LINE_END
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get INSERT_BEFORE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_insert_before() -> c_int {
    INSERT_BEFORE
}

/// FFI: Get INSERT_AFTER constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_insert_after() -> c_int {
    INSERT_AFTER
}

/// FFI: Get INSERT_LINE_START constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_insert_line_start() -> c_int {
    INSERT_LINE_START
}

/// FFI: Get INSERT_LINE_END constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_insert_line_end() -> c_int {
    INSERT_LINE_END
}

/// FFI: Get INSERT_OPEN_BELOW constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_insert_open_below() -> c_int {
    INSERT_OPEN_BELOW
}

/// FFI: Get INSERT_OPEN_ABOVE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_insert_open_above() -> c_int {
    INSERT_OPEN_ABOVE
}

/// FFI: Get INSERT_NORMAL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_insert_normal_mode() -> c_int {
    INSERT_NORMAL
}

/// FFI: Get INSERT_REPLACE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_insert_replace_mode() -> c_int {
    INSERT_REPLACE
}

/// FFI: Get INSERT_VREPLACE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_insert_vreplace_mode() -> c_int {
    INSERT_VREPLACE
}

/// FFI: Get insert command character 'i'.
#[unsafe(no_mangle)]
pub extern "C" fn rs_insert_char() -> c_int {
    insert_char()
}

/// FFI: Get append command character 'a'.
#[unsafe(no_mangle)]
pub extern "C" fn rs_append_char() -> c_int {
    append_char()
}

/// FFI: Get open-below command character 'o'.
#[unsafe(no_mangle)]
pub extern "C" fn rs_open_below_char() -> c_int {
    open_below_char()
}

/// FFI: Get open-above command character 'O'.
#[unsafe(no_mangle)]
pub extern "C" fn rs_open_above_char() -> c_int {
    open_above_char()
}

/// FFI: Check if command enters insert mode.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_insert_cmd(cmdchar: c_int) -> c_int {
    c_int::from(is_insert_cmd(cmdchar))
}

/// FFI: Check if command is open (o/O).
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_open_cmd(cmdchar: c_int) -> c_int {
    c_int::from(is_open_cmd(cmdchar))
}

/// FFI: Check if command is append (a/A).
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_append_cmd(cmdchar: c_int) -> c_int {
    c_int::from(is_append_cmd(cmdchar))
}

/// FFI: Get insert type from command character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_get_insert_type(cmdchar: c_int) -> c_int {
    get_insert_type(cmdchar)
}

/// FFI: Check if insert type opens a new line.
#[unsafe(no_mangle)]
pub extern "C" fn rs_insert_opens_line(insert_type: c_int) -> c_int {
    c_int::from(insert_opens_line(insert_type))
}

/// FFI: Check if insert type moves cursor.
#[unsafe(no_mangle)]
pub extern "C" fn rs_insert_moves_cursor(insert_type: c_int) -> c_int {
    c_int::from(insert_moves_cursor(insert_type))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_entry_constants() {
        assert_eq!(INSERT_BEFORE, 0);
        assert_eq!(INSERT_AFTER, 1);
        assert_eq!(INSERT_LINE_START, 2);
        assert_eq!(INSERT_LINE_END, 3);
        assert_eq!(INSERT_OPEN_BELOW, 4);
        assert_eq!(INSERT_OPEN_ABOVE, 5);
    }

    #[test]
    fn test_insert_state_constants() {
        assert_eq!(INSERT_NORMAL, 0);
        assert_eq!(INSERT_REPLACE, 1);
        assert_eq!(INSERT_VREPLACE, 2);
    }

    #[test]
    fn test_command_chars() {
        assert_eq!(insert_char(), c_int::from(b'i'));
        assert_eq!(append_char(), c_int::from(b'a'));
        assert_eq!(insert_line_start_char(), c_int::from(b'I'));
        assert_eq!(append_line_end_char(), c_int::from(b'A'));
        assert_eq!(open_below_char(), c_int::from(b'o'));
        assert_eq!(open_above_char(), c_int::from(b'O'));
        assert_eq!(replace_mode_char(), c_int::from(b'R'));
    }

    #[test]
    fn test_is_insert_cmd() {
        assert!(is_insert_cmd(c_int::from(b'i')));
        assert!(is_insert_cmd(c_int::from(b'a')));
        assert!(is_insert_cmd(c_int::from(b'I')));
        assert!(is_insert_cmd(c_int::from(b'A')));
        assert!(is_insert_cmd(c_int::from(b'o')));
        assert!(is_insert_cmd(c_int::from(b'O')));
        assert!(!is_insert_cmd(c_int::from(b'd')));
        assert!(!is_insert_cmd(c_int::from(b'R'))); // R is replace, not insert
    }

    #[test]
    fn test_is_open_cmd() {
        assert!(is_open_cmd(c_int::from(b'o')));
        assert!(is_open_cmd(c_int::from(b'O')));
        assert!(!is_open_cmd(c_int::from(b'i')));
        assert!(!is_open_cmd(c_int::from(b'a')));
    }

    #[test]
    fn test_is_append_cmd() {
        assert!(is_append_cmd(c_int::from(b'a')));
        assert!(is_append_cmd(c_int::from(b'A')));
        assert!(!is_append_cmd(c_int::from(b'i')));
        assert!(!is_append_cmd(c_int::from(b'o')));
    }

    #[test]
    fn test_get_insert_type() {
        assert_eq!(get_insert_type(c_int::from(b'i')), INSERT_BEFORE);
        assert_eq!(get_insert_type(c_int::from(b'a')), INSERT_AFTER);
        assert_eq!(get_insert_type(c_int::from(b'I')), INSERT_LINE_START);
        assert_eq!(get_insert_type(c_int::from(b'A')), INSERT_LINE_END);
        assert_eq!(get_insert_type(c_int::from(b'o')), INSERT_OPEN_BELOW);
        assert_eq!(get_insert_type(c_int::from(b'O')), INSERT_OPEN_ABOVE);
        assert_eq!(get_insert_type(c_int::from(b'x')), INSERT_BEFORE); // default
    }

    #[test]
    fn test_insert_opens_line() {
        assert!(insert_opens_line(INSERT_OPEN_BELOW));
        assert!(insert_opens_line(INSERT_OPEN_ABOVE));
        assert!(!insert_opens_line(INSERT_BEFORE));
        assert!(!insert_opens_line(INSERT_AFTER));
        assert!(!insert_opens_line(INSERT_LINE_START));
        assert!(!insert_opens_line(INSERT_LINE_END));
    }

    #[test]
    fn test_insert_moves_cursor() {
        assert!(insert_moves_cursor(INSERT_AFTER));
        assert!(insert_moves_cursor(INSERT_LINE_END));
        assert!(!insert_moves_cursor(INSERT_BEFORE));
        assert!(!insert_moves_cursor(INSERT_LINE_START));
        assert!(!insert_moves_cursor(INSERT_OPEN_BELOW));
        assert!(!insert_moves_cursor(INSERT_OPEN_ABOVE));
    }
}
