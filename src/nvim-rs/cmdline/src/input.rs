//! Command line input validation and buffer size calculations
//!
//! This module provides validation functions for command line input,
//! buffer sizing utilities, and input classification helpers.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::c_int;

use crate::state::{INITIAL_CMDBUFF_SIZE, MAX_CMDLINE_LEVEL, MIN_CMDBUFF_GROWTH};

// =============================================================================
// Buffer Size Calculations
// =============================================================================

/// Calculate the recommended buffer size for a given content length.
///
/// This adds extra space to avoid frequent reallocations.
/// Matches the C logic in `alloc_cmdbuff()`.
#[must_use]
pub const fn calc_buffer_size(content_len: usize) -> usize {
    if content_len < 80 {
        100
    } else {
        content_len + 20
    }
}

/// Calculate the minimum buffer growth when reallocating.
///
/// Returns the larger of the minimum growth or the required size.
#[must_use]
pub const fn calc_realloc_size(required: usize, current: usize) -> usize {
    if required <= current {
        return current;
    }

    let growth = required.saturating_sub(current);
    if growth < MIN_CMDBUFF_GROWTH {
        current + MIN_CMDBUFF_GROWTH
    } else {
        required
    }
}

/// Check if a buffer needs to be reallocated for the given length.
#[must_use]
pub const fn needs_realloc(required: usize, current: usize) -> bool {
    required > current
}

// =============================================================================
// Input Validation
// =============================================================================

/// Check if a character is a valid command line input character.
///
/// Control characters except specific ones (like Ctrl-C, Ctrl-D) are invalid.
#[must_use]
pub const fn is_valid_input_char(c: i32) -> bool {
    // NUL is never valid input
    if c == 0 {
        return false;
    }

    // Negative values are invalid
    if c < 0 {
        return false;
    }

    // All positive values are potentially valid
    // (actual validation happens during character insertion)
    true
}

/// Check if a character is a control character that should be handled specially.
#[must_use]
pub const fn is_control_char(c: i32) -> bool {
    c >= 0 && c < 32
}

/// Check if a character is printable in the command line.
///
/// This is a simplified check; actual printing depends on 'isprint' option.
#[must_use]
pub const fn is_printable(c: i32) -> bool {
    c >= 32 && c < 127 || c >= 128
}

/// Check if a key is a special key that needs handling.
///
/// Special keys have codes >= 256 (K_SPECIAL, etc.)
#[must_use]
pub const fn is_special_key(key: i32) -> bool {
    key >= 256
}

// =============================================================================
// Command Line Level Validation
// =============================================================================

/// Check if a new command line level can be created.
///
/// Returns true if the current level is below MAX_CMDLINE_LEVEL.
#[must_use]
pub const fn can_enter_cmdline(current_level: i32) -> bool {
    current_level < MAX_CMDLINE_LEVEL
}

/// Get the next command line level.
///
/// Returns None if at maximum level.
#[must_use]
pub const fn next_cmdline_level(current_level: i32) -> Option<i32> {
    if current_level < MAX_CMDLINE_LEVEL {
        Some(current_level + 1)
    } else {
        None
    }
}

// =============================================================================
// Prompt Validation
// =============================================================================

/// Valid prompt characters
pub const VALID_PROMPTS: &[u8] = b":/?=>@";

/// Check if a character is a valid command line prompt.
#[must_use]
pub const fn is_valid_prompt(c: u8) -> bool {
    matches!(c, b':' | b'/' | b'?' | b'=' | b'>' | b'@' | 0)
}

/// Get the prompt type from a character.
///
/// Returns the normalized prompt character, or 0 for invalid/no prompt.
#[must_use]
pub const fn normalize_prompt(c: u8) -> u8 {
    if c == b'@' {
        0 // '@' (input function) normalizes to NUL
    } else if is_valid_prompt(c) {
        c
    } else {
        0
    }
}

// =============================================================================
// Input Position Validation
// =============================================================================

/// Check if a cursor position is valid for the given command length.
#[must_use]
pub const fn is_valid_cursor_pos(pos: usize, cmdlen: usize) -> bool {
    pos <= cmdlen
}

/// Clamp a cursor position to valid range.
#[must_use]
pub const fn clamp_cursor_pos(pos: usize, cmdlen: usize) -> usize {
    if pos > cmdlen {
        cmdlen
    } else {
        pos
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Calculate recommended buffer size for given content length.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_cmdline_calc_buffer_size(content_len: c_int) -> c_int {
    if content_len < 0 {
        INITIAL_CMDBUFF_SIZE as c_int
    } else {
        calc_buffer_size(content_len as usize) as c_int
    }
}

/// Check if buffer needs reallocation.
#[no_mangle]
pub extern "C" fn rs_cmdline_needs_realloc(required: c_int, current: c_int) -> c_int {
    if required < 0 || current < 0 {
        return 0;
    }
    c_int::from(needs_realloc(required as usize, current as usize))
}

/// Calculate reallocation size.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_cmdline_calc_realloc_size(required: c_int, current: c_int) -> c_int {
    if required < 0 || current < 0 {
        return INITIAL_CMDBUFF_SIZE as c_int;
    }
    calc_realloc_size(required as usize, current as usize) as c_int
}

/// Check if a character is valid input.
#[no_mangle]
pub extern "C" fn rs_cmdline_is_valid_input(c: c_int) -> c_int {
    c_int::from(is_valid_input_char(c))
}

/// Check if a character is a control character.
#[no_mangle]
pub extern "C" fn rs_cmdline_is_control_char(c: c_int) -> c_int {
    c_int::from(is_control_char(c))
}

/// Check if a character is printable.
#[no_mangle]
pub extern "C" fn rs_cmdline_is_printable(c: c_int) -> c_int {
    c_int::from(is_printable(c))
}

/// Check if a key is a special key.
#[no_mangle]
pub extern "C" fn rs_cmdline_is_special_key(key: c_int) -> c_int {
    c_int::from(is_special_key(key))
}

/// Check if a new command line level can be entered.
#[no_mangle]
pub extern "C" fn rs_cmdline_can_enter(current_level: c_int) -> c_int {
    c_int::from(can_enter_cmdline(current_level))
}

/// Get the next command line level, or -1 if at max.
#[no_mangle]
pub extern "C" fn rs_cmdline_next_level(current_level: c_int) -> c_int {
    next_cmdline_level(current_level).unwrap_or(-1)
}

/// Check if a prompt character is valid.
#[no_mangle]
#[allow(clippy::manual_range_contains)]
pub extern "C" fn rs_cmdline_is_valid_prompt(c: c_int) -> c_int {
    if c < 0 || c > 255 {
        return 0;
    }
    c_int::from(is_valid_prompt(c as u8))
}

/// Normalize a prompt character.
#[no_mangle]
#[allow(clippy::manual_range_contains)]
pub extern "C" fn rs_cmdline_normalize_prompt(c: c_int) -> c_int {
    if c < 0 || c > 255 {
        return 0;
    }
    c_int::from(normalize_prompt(c as u8))
}

/// Check if cursor position is valid.
#[no_mangle]
pub extern "C" fn rs_cmdline_is_valid_pos(pos: c_int, cmdlen: c_int) -> c_int {
    if pos < 0 || cmdlen < 0 {
        return 0;
    }
    c_int::from(is_valid_cursor_pos(pos as usize, cmdlen as usize))
}

/// Clamp cursor position to valid range.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_cmdline_clamp_pos(pos: c_int, cmdlen: c_int) -> c_int {
    if pos < 0 {
        return 0;
    }
    if cmdlen < 0 {
        return pos;
    }
    clamp_cursor_pos(pos as usize, cmdlen as usize) as c_int
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_buffer_size() {
        assert_eq!(calc_buffer_size(0), 100);
        assert_eq!(calc_buffer_size(50), 100);
        assert_eq!(calc_buffer_size(79), 100);
        assert_eq!(calc_buffer_size(80), 100); // 80 + 20 = 100
        assert_eq!(calc_buffer_size(100), 120); // 100 + 20 = 120
    }

    #[test]
    fn test_calc_realloc_size() {
        // No realloc needed
        assert_eq!(calc_realloc_size(50, 100), 100);

        // Needs realloc, small growth
        assert_eq!(calc_realloc_size(110, 100), 100 + MIN_CMDBUFF_GROWTH);

        // Needs realloc, large growth
        assert_eq!(calc_realloc_size(500, 100), 500);
    }

    #[test]
    fn test_needs_realloc() {
        assert!(!needs_realloc(50, 100));
        assert!(!needs_realloc(100, 100));
        assert!(needs_realloc(101, 100));
    }

    #[test]
    fn test_is_valid_input_char() {
        assert!(!is_valid_input_char(0)); // NUL invalid
        assert!(!is_valid_input_char(-1)); // Negative invalid
        assert!(is_valid_input_char(1)); // Ctrl-A valid (handled specially)
        assert!(is_valid_input_char(65)); // 'A' valid
        assert!(is_valid_input_char(0x80)); // High byte valid
    }

    #[test]
    fn test_is_control_char() {
        assert!(is_control_char(0)); // NUL
        assert!(is_control_char(1)); // Ctrl-A
        assert!(is_control_char(31)); // Last control
        assert!(!is_control_char(32)); // Space
        assert!(!is_control_char(-1)); // Negative
    }

    #[test]
    fn test_is_printable() {
        assert!(!is_printable(0)); // NUL
        assert!(!is_printable(31)); // Control
        assert!(is_printable(32)); // Space
        assert!(is_printable(65)); // 'A'
        assert!(is_printable(126)); // '~'
        assert!(!is_printable(127)); // DEL
        assert!(is_printable(128)); // High bytes printable
    }

    #[test]
    fn test_is_special_key() {
        assert!(!is_special_key(0));
        assert!(!is_special_key(255));
        assert!(is_special_key(256));
        assert!(is_special_key(1000));
    }

    #[test]
    fn test_cmdline_level() {
        assert!(can_enter_cmdline(0));
        assert!(can_enter_cmdline(MAX_CMDLINE_LEVEL - 1));
        assert!(!can_enter_cmdline(MAX_CMDLINE_LEVEL));

        assert_eq!(next_cmdline_level(0), Some(1));
        assert_eq!(
            next_cmdline_level(MAX_CMDLINE_LEVEL - 1),
            Some(MAX_CMDLINE_LEVEL)
        );
        assert_eq!(next_cmdline_level(MAX_CMDLINE_LEVEL), None);
    }

    #[test]
    fn test_is_valid_prompt() {
        assert!(is_valid_prompt(b':'));
        assert!(is_valid_prompt(b'/'));
        assert!(is_valid_prompt(b'?'));
        assert!(is_valid_prompt(b'='));
        assert!(is_valid_prompt(b'>'));
        assert!(is_valid_prompt(b'@'));
        assert!(is_valid_prompt(0)); // NUL is valid (no prompt)
        assert!(!is_valid_prompt(b'x'));
    }

    #[test]
    fn test_normalize_prompt() {
        assert_eq!(normalize_prompt(b':'), b':');
        assert_eq!(normalize_prompt(b'/'), b'/');
        assert_eq!(normalize_prompt(b'@'), 0); // '@' normalizes to NUL
        assert_eq!(normalize_prompt(b'x'), 0); // Invalid normalizes to NUL
    }

    #[test]
    fn test_cursor_pos_validation() {
        assert!(is_valid_cursor_pos(0, 10));
        assert!(is_valid_cursor_pos(5, 10));
        assert!(is_valid_cursor_pos(10, 10)); // At end is valid
        assert!(!is_valid_cursor_pos(11, 10)); // Past end invalid

        assert_eq!(clamp_cursor_pos(5, 10), 5);
        assert_eq!(clamp_cursor_pos(15, 10), 10);
        assert_eq!(clamp_cursor_pos(0, 10), 0);
    }
}
