//! Core message output functions
//!
//! Provides Rust implementations for the fundamental message output operations:
//! - `msg()` - Display a message with optional highlight
//! - `msg_puts()` - Output a string to the message area
//! - `msg_putchar()` - Output a single character to the message area
//!
//! These functions form the foundation of the message display system.

use std::ffi::{c_char, c_int};

// Use the mbyte crate for UTF-8 encoding
use nvim_mbyte::rs_utf_char2bytes;

// ============================================================================
// C Function Declarations
// ============================================================================

extern "C" {
    // Core message output functions (call into C until fully migrated)
    fn msg_keep(s: *const c_char, hl_id: c_int, keep: c_int, multiline: c_int) -> c_int;
    fn msg_puts_hl(s: *const c_char, hl_id: c_int, hist: c_int);
    fn msg_start();
    fn msg_end() -> c_int;
    fn msg_clr_eos();

    // State accessors
    fn nvim_get_msg_silent() -> c_int;
}

/// Maximum bytes for a single UTF-8 character (including composing chars)
const MB_MAXCHAR: usize = 6;

/// Special key indicator
const K_SPECIAL: u8 = 0x80;

/// Check if a character is a special key code.
#[inline]
const fn is_special(c: c_int) -> bool {
    c < 0
}

/// Get the second byte of a special key.
#[allow(clippy::cast_sign_loss)]
#[inline]
const fn k_second(c: c_int) -> u8 {
    (((-c - 1) >> 8) & 0xff) as u8
}

/// Get the third byte of a special key.
#[allow(clippy::cast_sign_loss)]
#[inline]
const fn k_third(c: c_int) -> u8 {
    ((-c - 1) & 0xff) as u8
}

// ============================================================================
// Core Message Output Functions
// ============================================================================

/// Display a message to the user.
///
/// This is the primary function for displaying a message string.
/// The message is displayed at the current message position.
///
/// # Arguments
/// * `s` - The message string to display (NUL-terminated)
/// * `hl_id` - Highlight group ID (0 for default)
///
/// # Returns
/// * `true` if wait_return() was not called
/// * `false` if wait_return() was called
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_msg(s: *const c_char, hl_id: c_int) -> c_int {
    msg_keep(s, hl_id, 0, 0)
}

/// Display a message and optionally keep it displayed.
///
/// # Arguments
/// * `s` - The message string to display (NUL-terminated)
/// * `hl_id` - Highlight group ID (0 for default)
/// * `keep` - If true, keep the message displayed (set keep_msg)
///
/// # Returns
/// * `true` if wait_return() was not called
/// * `false` if wait_return() was called
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_msg_keep(s: *const c_char, hl_id: c_int, keep: c_int) -> c_int {
    msg_keep(s, hl_id, keep, 0)
}

/// Display a multiline message.
///
/// # Arguments
/// * `s` - The message string to display (NUL-terminated)
/// * `hl_id` - Highlight group ID (0 for default)
/// * `multiline` - If true, handle embedded newlines specially
///
/// # Returns
/// * `true` if wait_return() was not called
/// * `false` if wait_return() was called
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_msg_multiline_simple(
    s: *const c_char,
    hl_id: c_int,
    multiline: c_int,
) -> c_int {
    msg_keep(s, hl_id, 0, multiline)
}

/// Output a string to the message area.
///
/// Outputs the string at the current msg_row, msg_col position.
/// Does not add the string to message history.
///
/// # Arguments
/// * `s` - The string to output (NUL-terminated)
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_msg_puts(s: *const c_char) {
    msg_puts_hl(s, 0, 0);
}

/// Output a string with highlight and history option.
///
/// # Arguments
/// * `s` - The string to output (NUL-terminated)
/// * `hl_id` - Highlight group ID (0 for default)
/// * `hist` - If true, add to message history
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_msg_puts_hl(s: *const c_char, hl_id: c_int, hist: c_int) {
    msg_puts_hl(s, hl_id, hist);
}

/// Output a single character to the message area.
///
/// Outputs the character at the current msg_row, msg_col position.
/// Handles multi-byte UTF-8 characters and special key codes.
///
/// # Arguments
/// * `c` - The character to output (Unicode code point or special key)
///
/// # Safety
/// This function is safe to call with any integer value.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_putchar(c: c_int) {
    rs_msg_putchar_hl(c, 0);
}

/// Output a single character with highlight.
///
/// Outputs the character at the current msg_row, msg_col position.
/// Handles multi-byte UTF-8 characters and special key codes.
///
/// # Arguments
/// * `c` - The character to output (Unicode code point or special key)
/// * `hl_id` - Highlight group ID (0 for default)
///
/// # Safety
/// This function is safe to call with any integer value for `c`.
#[no_mangle]
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_msg_putchar_hl(c: c_int, hl_id: c_int) {
    let mut buf: [c_char; MB_MAXCHAR + 1] = [0; MB_MAXCHAR + 1];

    if is_special(c) {
        // Special key code: encode as K_SPECIAL + two bytes
        buf[0] = K_SPECIAL as c_char;
        buf[1] = k_second(c) as c_char;
        buf[2] = k_third(c) as c_char;
        buf[3] = 0; // NUL terminator
    } else {
        // Regular character: encode as UTF-8
        let len = rs_utf_char2bytes(c, buf.as_mut_ptr());
        buf[len as usize] = 0; // NUL terminator
    }

    msg_puts_hl(buf.as_ptr(), hl_id, 0);
}

/// Output a number to the message area.
///
/// Converts the number to a string and outputs it.
///
/// # Arguments
/// * `n` - The number to output
///
/// # Safety
/// This function is safe to call with any integer value.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_msg_outnum(n: c_int) {
    // Format number as string (max 20 chars for i32)
    let mut buf: [c_char; 20] = [0; 20];

    // Use snprintf-like formatting
    let s = format!("{n}");
    let bytes = s.as_bytes();
    let len = bytes.len().min(19);

    for (i, &b) in bytes[..len].iter().enumerate() {
        buf[i] = b as c_char;
    }
    buf[len] = 0;

    msg_puts_hl(buf.as_ptr(), 0, 0);
}

// ============================================================================
// Message Control Functions
// ============================================================================

/// Start a new message.
///
/// Prepares the message area for output. This should be called before
/// writing message content.
///
/// # Safety
/// Calls C function that modifies global state.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_start() {
    msg_start();
}

/// End a message.
///
/// Finalizes message output and handles wait_return if needed.
///
/// # Returns
/// * `true` (1) if wait_return() was not called
/// * `false` (0) if wait_return() was called
///
/// # Safety
/// Calls C function that modifies global state.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_end() -> c_int {
    msg_end()
}

/// Clear from current message position to end of screen.
///
/// # Safety
/// Calls C function that modifies display state.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_clr_eos() {
    msg_clr_eos();
}

/// Check if messages are silent.
///
/// # Returns
/// * Non-zero if msg_silent > 0 (messages are being suppressed)
/// * Zero if messages are not silent
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_is_silent() -> c_int {
    nvim_get_msg_silent()
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Output a title string (highlighted as HLF_T).
///
/// # Arguments
/// * `s` - The string to output (NUL-terminated)
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_msg_puts_title(s: *const c_char) {
    // HLF_T is the title highlight - use the constant from C
    // For now, use hl_id 0 as placeholder; actual HLF_T value from C will be used
    msg_puts_hl(s, HLF_T, 0);
}

/// Highlight face for title
const HLF_T: c_int = 25; // From highlight.h: HLF_T = 25

/// Highlight face for "8" (truncation indicator)
#[allow(dead_code)]
const HLF_8: c_int = 38; // From highlight.h: HLF_8 = 38

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_special() {
        // Special keys are negative
        assert!(is_special(-1));
        assert!(is_special(-100));
        assert!(!is_special(0));
        assert!(!is_special(65)); // 'A'
    }

    #[test]
    fn test_k_second_third() {
        // Test special key byte extraction
        // For a special key c = -(1 + (second << 8) + third)
        let second: u8 = 0x12;
        let third: u8 = 0x34;
        let c = -1 - (c_int::from(second) << 8) - c_int::from(third);

        assert_eq!(k_second(c), second);
        assert_eq!(k_third(c), third);
    }
}
