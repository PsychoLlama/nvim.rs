//! Filename escaping utilities for command-line mode
//!
//! This module provides utilities for escaping filenames used in command-line
//! arguments, handling special characters that have meaning in Vim commands.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::{c_char, c_int};

// =============================================================================
// Escape Mode Types
// =============================================================================

/// Filename escaping mode.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FnameEscapeMode {
    /// Escape for normal path usage
    #[default]
    Path = 0,
    /// Escape for shell command
    Shell = 1,
    /// Escape for :buffer command
    Buffer = 2,
}

impl FnameEscapeMode {
    /// Parse from integer.
    #[must_use]
    pub const fn from_int(val: i32) -> Self {
        match val {
            1 => Self::Shell,
            2 => Self::Buffer,
            _ => Self::Path,
        }
    }
}

// =============================================================================
// Character Classification
// =============================================================================

/// Characters that need escaping in paths (Unix).
pub const PATH_ESC_CHARS: &[u8] = b" \t\n*?[{`$\\%#'\"|!<";

/// Characters that need escaping for shell commands (Unix).
pub const SHELL_ESC_CHARS: &[u8] = b" \t\n*?[{`$\\%#'\"|!<>();&";

/// Characters that need escaping for :buffer command (Unix).
pub const BUFFER_ESC_CHARS: &[u8] = b" \t\n*?[`$\\%#'\"|!<";

/// Get the escape characters for a given mode.
#[must_use]
pub const fn escape_chars_for_mode(mode: FnameEscapeMode) -> &'static [u8] {
    match mode {
        FnameEscapeMode::Path => PATH_ESC_CHARS,
        FnameEscapeMode::Shell => SHELL_ESC_CHARS,
        FnameEscapeMode::Buffer => BUFFER_ESC_CHARS,
    }
}

/// Check if a character needs escaping.
#[must_use]
pub fn needs_escape(c: u8, mode: FnameEscapeMode) -> bool {
    let chars = escape_chars_for_mode(mode);
    chars.contains(&c)
}

/// Check if a character is special at the start of a filename.
///
/// '>', '+' are special at the start of some commands like :edit and :write.
/// '-' followed by nothing is special (e.g., "cd -").
#[must_use]
pub const fn is_special_start_char(c: u8) -> bool {
    c == b'>' || c == b'+'
}

/// Check if a filename needs a leading backslash escape.
///
/// Returns true for filenames starting with '>', '+', or "-" (when followed by NUL).
#[must_use]
pub fn needs_leading_escape(fname: &[u8]) -> bool {
    if fname.is_empty() {
        return false;
    }
    let first = fname[0];
    if is_special_start_char(first) {
        return true;
    }
    // "-" alone needs escaping
    first == b'-' && fname.len() == 1
}

// =============================================================================
// Escaping Functions
// =============================================================================

/// Calculate the length needed for an escaped string.
#[must_use]
pub fn escaped_len(s: &[u8], mode: FnameEscapeMode) -> usize {
    let esc_chars = escape_chars_for_mode(mode);
    let mut len = s.len();
    for &c in s {
        if esc_chars.contains(&c) {
            len += 1; // Extra byte for backslash
        }
    }
    // Add 1 for leading backslash if needed
    if needs_leading_escape(s) {
        len += 1;
    }
    len
}

/// Escape a filename for command-line use.
///
/// Returns a new Vec with the escaped filename.
#[must_use]
pub fn escape_fname(fname: &[u8], mode: FnameEscapeMode) -> Vec<u8> {
    let esc_chars = escape_chars_for_mode(mode);
    let mut result = Vec::with_capacity(escaped_len(fname, mode));

    // Add leading backslash if needed
    if needs_leading_escape(fname) {
        result.push(b'\\');
    }

    // Escape special characters
    for &c in fname {
        if esc_chars.contains(&c) {
            result.push(b'\\');
        }
        result.push(c);
    }

    result
}

/// Escape a string in place by adding backslashes.
///
/// This is a simplified version that just prepends a backslash.
#[must_use]
pub fn prepend_backslash(s: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(s.len() + 1);
    result.push(b'\\');
    result.extend_from_slice(s);
    result
}

// =============================================================================
// Shell Detection
// =============================================================================

/// Check if a character is a valid filename character.
///
/// This is a simplified check; the real implementation uses 'isfname' option.
#[must_use]
pub const fn is_fname_char(c: u8) -> bool {
    c.is_ascii_alphanumeric()
        || c == b'_'
        || c == b'-'
        || c == b'.'
        || c == b'/'
        || c == b'~'
        || c >= 128 // Multibyte chars
}

// =============================================================================
// Tilde Expansion
// =============================================================================

/// Check if a path starts with "~/".
#[must_use]
pub fn starts_with_tilde_slash(path: &[u8]) -> bool {
    path.len() >= 2 && path[0] == b'~' && is_path_separator(path[1])
}

/// Check if a character is a path separator.
#[must_use]
pub const fn is_path_separator(c: u8) -> bool {
    c == b'/'
}

// =============================================================================
// VSE Constants (for C compatibility)
// =============================================================================

/// VSE_NONE - no special escaping
pub const VSE_NONE: c_int = 0;
/// VSE_SHELL - escape for shell command
pub const VSE_SHELL: c_int = 1;
/// VSE_BUFFER - escape for :buffer command
pub const VSE_BUFFER: c_int = 2;

// =============================================================================
// FFI Exports
// =============================================================================

/// Get the escape mode constant for path escaping.
#[no_mangle]
pub extern "C" fn rs_vse_none() -> c_int {
    VSE_NONE
}

/// Get the escape mode constant for shell escaping.
#[no_mangle]
pub extern "C" fn rs_vse_shell() -> c_int {
    VSE_SHELL
}

/// Get the escape mode constant for buffer escaping.
#[no_mangle]
pub extern "C" fn rs_vse_buffer() -> c_int {
    VSE_BUFFER
}

/// Check if a character needs escaping for the given mode.
///
/// # Safety
///
/// `mode` should be a valid VSE_* constant.
#[no_mangle]
pub extern "C" fn rs_fname_needs_escape(c: c_int, mode: c_int) -> c_int {
    if !(0..=255).contains(&c) {
        return 0;
    }
    c_int::from(needs_escape(c as u8, FnameEscapeMode::from_int(mode)))
}

/// Check if a filename needs a leading backslash escape.
///
/// # Safety
///
/// `fname` must be a valid pointer, `len` must be the string length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_fname_needs_leading_escape(fname: *const c_char, len: usize) -> c_int {
    if fname.is_null() || len == 0 {
        return 0;
    }
    let bytes = std::slice::from_raw_parts(fname.cast::<u8>(), len);
    c_int::from(needs_leading_escape(bytes))
}

/// Check if a character is a valid filename character.
#[no_mangle]
pub extern "C" fn rs_is_fname_char(c: c_int) -> c_int {
    if !(0..=255).contains(&c) {
        return 0;
    }
    c_int::from(is_fname_char(c as u8))
}

/// Check if character is special at start of filename.
#[no_mangle]
pub extern "C" fn rs_is_special_start_char(c: c_int) -> c_int {
    if !(0..=255).contains(&c) {
        return 0;
    }
    c_int::from(is_special_start_char(c as u8))
}

/// Check if a path starts with "~/".
///
/// # Safety
///
/// `path` must be a valid pointer, `len` must be the string length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_starts_with_tilde_slash(path: *const c_char, len: usize) -> c_int {
    if path.is_null() {
        return 0;
    }
    let bytes = std::slice::from_raw_parts(path.cast::<u8>(), len);
    c_int::from(starts_with_tilde_slash(bytes))
}

/// Check if a character is a path separator.
#[no_mangle]
pub extern "C" fn rs_is_path_separator(c: c_int) -> c_int {
    if !(0..=255).contains(&c) {
        return 0;
    }
    c_int::from(is_path_separator(c as u8))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_mode() {
        assert_eq!(FnameEscapeMode::from_int(0), FnameEscapeMode::Path);
        assert_eq!(FnameEscapeMode::from_int(1), FnameEscapeMode::Shell);
        assert_eq!(FnameEscapeMode::from_int(2), FnameEscapeMode::Buffer);
        assert_eq!(FnameEscapeMode::from_int(99), FnameEscapeMode::Path);
    }

    #[test]
    fn test_needs_escape() {
        // Space needs escaping in all modes
        assert!(needs_escape(b' ', FnameEscapeMode::Path));
        assert!(needs_escape(b' ', FnameEscapeMode::Shell));
        assert!(needs_escape(b' ', FnameEscapeMode::Buffer));

        // Regular letters don't need escaping
        assert!(!needs_escape(b'a', FnameEscapeMode::Path));
        assert!(!needs_escape(b'a', FnameEscapeMode::Shell));

        // Semicolon only needs escaping in shell mode
        assert!(!needs_escape(b';', FnameEscapeMode::Path));
        assert!(needs_escape(b';', FnameEscapeMode::Shell));
    }

    #[test]
    fn test_special_start_char() {
        assert!(is_special_start_char(b'>'));
        assert!(is_special_start_char(b'+'));
        assert!(!is_special_start_char(b'-')); // Only special if alone
        assert!(!is_special_start_char(b'a'));
    }

    #[test]
    fn test_needs_leading_escape() {
        assert!(needs_leading_escape(b">file"));
        assert!(needs_leading_escape(b"+file"));
        assert!(needs_leading_escape(b"-")); // "-" alone
        assert!(!needs_leading_escape(b"-file")); // Not alone
        assert!(!needs_leading_escape(b"file"));
        assert!(!needs_leading_escape(b""));
    }

    #[test]
    fn test_escape_fname() {
        // Simple filename - no escaping
        assert_eq!(
            escape_fname(b"file.txt", FnameEscapeMode::Path),
            b"file.txt"
        );

        // Space needs escaping
        assert_eq!(
            escape_fname(b"my file.txt", FnameEscapeMode::Path),
            b"my\\ file.txt"
        );

        // Leading > needs backslash
        assert_eq!(escape_fname(b">file", FnameEscapeMode::Path), b"\\>file");

        // Leading + needs backslash
        assert_eq!(escape_fname(b"+file", FnameEscapeMode::Path), b"\\+file");
    }

    #[test]
    fn test_prepend_backslash() {
        assert_eq!(prepend_backslash(b"file"), b"\\file");
        assert_eq!(prepend_backslash(b""), b"\\");
    }

    #[test]
    fn test_is_fname_char() {
        assert!(is_fname_char(b'a'));
        assert!(is_fname_char(b'Z'));
        assert!(is_fname_char(b'0'));
        assert!(is_fname_char(b'_'));
        assert!(is_fname_char(b'-'));
        assert!(is_fname_char(b'.'));
        assert!(is_fname_char(b'/'));
        assert!(!is_fname_char(b' '));
        assert!(!is_fname_char(b'*'));
    }

    #[test]
    fn test_starts_with_tilde_slash() {
        assert!(starts_with_tilde_slash(b"~/foo"));
        assert!(!starts_with_tilde_slash(b"~foo"));
        assert!(!starts_with_tilde_slash(b"/foo"));
        assert!(!starts_with_tilde_slash(b"~"));
        assert!(!starts_with_tilde_slash(b""));
    }

    #[test]
    fn test_is_path_separator() {
        assert!(is_path_separator(b'/'));
        assert!(!is_path_separator(b'\\'));
        assert!(!is_path_separator(b'a'));
    }
}
