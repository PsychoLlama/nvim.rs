//! Character insertion engine for insert mode.
//!
//! This module provides Rust implementations for character insertion
//! operations in insert mode, including handling of multi-byte characters,
//! special characters, and replace mode behavior.

use std::ffi::c_int;

use crate::state::{ColnrT, LinenrT};

/// Maximum bytes for a multi-byte character (from `mbyte_defs.h`).
pub const MB_MAXCHAR: usize = 6;

// C functions for character operations.
extern "C" {
    static mut State: c_int;
    // Character conversion
    fn utf_char2bytes(c: c_int, buf: *mut u8) -> c_int;
    fn utf_ptr2char(p: *const u8) -> c_int;
    fn utfc_ptr2len(p: *const u8) -> c_int;

    // Buffer operations
    fn nvim_curwin_get_cursor_col() -> ColnrT;
    fn nvim_curwin_get_cursor_lnum() -> LinenrT;

    // Mode check
}

/// Mode flags for State (from `vim_defs.h`).
mod mode_flags {
    use std::ffi::c_int;

    pub const REPLACE_FLAG: c_int = 0x40;
    pub const VREPLACE_FLAG: c_int = 0x80;
    pub const MODE_INSERT: c_int = 0x10;
}

/// Information about a character to be inserted.
#[derive(Debug, Clone, Copy)]
pub struct CharInfo {
    /// The character codepoint.
    pub codepoint: i32,
    /// Number of bytes when encoded as UTF-8.
    pub byte_len: usize,
    /// Whether this is a special character (NUL, NL, etc.).
    pub is_special: bool,
}

impl CharInfo {
    /// Create character info from a codepoint.
    #[must_use]
    pub fn from_codepoint(c: i32) -> Self {
        let mut buf = [0u8; MB_MAXCHAR + 1];
        // SAFETY: We pass a valid buffer
        // SAFETY: utf_char2bytes always returns a positive value (0-6 bytes)
        #[allow(clippy::cast_sign_loss)]
        let byte_len = unsafe { utf_char2bytes(c, buf.as_mut_ptr()) as usize };

        // Check for special characters
        let is_special = c == 0 || c == i32::from(b'\n') || c == i32::from(b'\r');

        Self {
            codepoint: c,
            byte_len,
            is_special,
        }
    }

    /// Check if this character is a NUL that should be converted to NL.
    ///
    /// When "c" is 0x100, 0x200, etc. we don't want to insert a NUL byte.
    /// Happens for CTRL-Vu9900.
    #[inline]
    #[must_use]
    pub fn needs_nul_to_nl_conversion(&self) -> bool {
        // Check if the first byte would be NUL
        let mut buf = [0u8; MB_MAXCHAR + 1];
        // SAFETY: We pass a valid buffer
        unsafe {
            utf_char2bytes(self.codepoint, buf.as_mut_ptr());
        }
        buf[0] == 0
    }

    /// Get the UTF-8 bytes for this character.
    ///
    /// Returns the bytes and the actual length. If NUL conversion is needed,
    /// the first byte is replaced with NL.
    #[must_use]
    pub fn to_bytes(&self) -> ([u8; MB_MAXCHAR + 1], usize) {
        let mut buf = [0u8; MB_MAXCHAR + 1];
        // SAFETY: We pass a valid buffer
        // SAFETY: utf_char2bytes always returns a positive value (0-6 bytes)
        #[allow(clippy::cast_sign_loss)]
        let len = unsafe { utf_char2bytes(self.codepoint, buf.as_mut_ptr()) as usize };

        // Handle NUL conversion
        if buf[0] == 0 {
            buf[0] = b'\n';
        }

        (buf, len)
    }
}

/// Check if we're in replace mode.
#[inline]
#[must_use]
pub fn in_replace_mode() -> bool {
    // SAFETY: Simple global accessor
    let state = unsafe { State };
    (state & mode_flags::REPLACE_FLAG) != 0
}

/// Check if we're in virtual replace mode.
#[inline]
#[must_use]
pub fn in_vreplace_mode() -> bool {
    // SAFETY: Simple global accessor
    let state = unsafe { State };
    (state & mode_flags::VREPLACE_FLAG) != 0
}

/// Check if we're in insert mode.
#[inline]
#[must_use]
pub fn in_insert_mode_flag() -> bool {
    // SAFETY: Simple global accessor
    let state = unsafe { State };
    (state & mode_flags::MODE_INSERT) != 0
}

/// Get the current cursor column.
#[inline]
#[must_use]
pub fn cursor_col() -> ColnrT {
    // SAFETY: Simple global accessor
    unsafe { nvim_curwin_get_cursor_col() }
}

/// Get the current cursor line number.
#[inline]
#[must_use]
pub fn cursor_lnum() -> LinenrT {
    // SAFETY: Simple global accessor
    unsafe { nvim_curwin_get_cursor_lnum() }
}

/// Calculate the length of a character at a given pointer.
///
/// # Safety
/// The pointer must point to valid UTF-8 data.
#[inline]
#[must_use]
pub unsafe fn char_len_at(p: *const u8) -> usize {
    // utfc_ptr2len always returns a positive value (1-6 bytes)
    #[allow(clippy::cast_sign_loss)]
    let len = utfc_ptr2len(p) as usize;
    len
}

/// Get the codepoint of a character at a given pointer.
///
/// # Safety
/// The pointer must point to valid UTF-8 data.
#[inline]
#[must_use]
pub unsafe fn char_at(p: *const u8) -> i32 {
    utf_ptr2char(p)
}

/// Information about a character insertion operation.
#[derive(Debug, Clone, Copy)]
pub struct InsertOp {
    /// Line number where insertion happens.
    pub lnum: LinenrT,
    /// Column where insertion happens.
    pub col: ColnrT,
    /// Number of bytes to insert.
    pub insert_len: usize,
    /// Number of bytes to delete (for replace mode).
    pub delete_len: usize,
    /// Whether this is a replace operation.
    pub is_replace: bool,
}

impl InsertOp {
    /// Create a simple insert operation.
    #[must_use]
    pub const fn insert(lnum: LinenrT, col: ColnrT, len: usize) -> Self {
        Self {
            lnum,
            col,
            insert_len: len,
            delete_len: 0,
            is_replace: false,
        }
    }

    /// Create a replace operation.
    #[must_use]
    pub const fn replace(lnum: LinenrT, col: ColnrT, insert_len: usize, delete_len: usize) -> Self {
        Self {
            lnum,
            col,
            insert_len,
            delete_len,
            is_replace: true,
        }
    }
}

/// Check if a character is a printable character.
///
/// This is a simplified check - the real implementation should use `vim_isprintc`.
#[inline]
#[must_use]
#[allow(clippy::manual_range_contains)] // Keep as const fn
pub const fn is_printable(c: i32) -> bool {
    // Basic printable check: ASCII printable or multibyte
    (c >= 0x20 && c < 0x7f) || c >= 0x100
}

/// Space character as i32.
const SPACE: i32 = b' ' as i32;
/// Tab character as i32.
const TAB: i32 = b'\t' as i32;
/// Newline character as i32.
const NL: i32 = b'\n' as i32;
/// Carriage return character as i32.
const CR: i32 = b'\r' as i32;

/// Check if a character is whitespace.
#[inline]
#[must_use]
pub const fn is_whitespace(c: i32) -> bool {
    c == SPACE || c == TAB
}

/// Check if a character is a newline.
#[inline]
#[must_use]
pub const fn is_newline(c: i32) -> bool {
    c == NL || c == CR
}

// FFI exports

/// FFI: Create character info from codepoint.
#[no_mangle]
pub extern "C" fn rs_char_info_byte_len(c: c_int) -> c_int {
    let info = CharInfo::from_codepoint(c);
    // byte_len is always 0-6, so this is safe
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let result = info.byte_len as c_int;
    result
}

/// FFI: Check if character needs NUL to NL conversion.
#[no_mangle]
pub extern "C" fn rs_char_needs_nul_conversion(c: c_int) -> c_int {
    let info = CharInfo::from_codepoint(c);
    c_int::from(info.needs_nul_to_nl_conversion())
}

/// FFI: Check if in replace mode.
#[no_mangle]
pub extern "C" fn rs_insert_in_replace_mode() -> c_int {
    c_int::from(in_replace_mode())
}

/// FFI: Check if in virtual replace mode.
#[no_mangle]
pub extern "C" fn rs_insert_in_vreplace_mode() -> c_int {
    c_int::from(in_vreplace_mode())
}

/// FFI: Check if character is printable.
#[no_mangle]
pub extern "C" fn rs_is_printable(c: c_int) -> c_int {
    c_int::from(is_printable(c))
}

/// FFI: Check if character is whitespace.
#[no_mangle]
pub extern "C" fn rs_is_whitespace(c: c_int) -> c_int {
    c_int::from(is_whitespace(c))
}

/// FFI: Check if character is newline.
#[no_mangle]
pub extern "C" fn rs_is_newline(c: c_int) -> c_int {
    c_int::from(is_newline(c))
}

/// FFI: Get cursor column.
#[no_mangle]
pub extern "C" fn rs_insert_cursor_col() -> ColnrT {
    cursor_col()
}

/// FFI: Get cursor line number.
#[no_mangle]
pub extern "C" fn rs_insert_cursor_lnum() -> LinenrT {
    cursor_lnum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_whitespace() {
        assert!(is_whitespace(i32::from(b' ')));
        assert!(is_whitespace(i32::from(b'\t')));
        assert!(!is_whitespace(i32::from(b'a')));
        assert!(!is_whitespace(i32::from(b'\n')));
    }

    #[test]
    fn test_is_newline() {
        assert!(is_newline(i32::from(b'\n')));
        assert!(is_newline(i32::from(b'\r')));
        assert!(!is_newline(i32::from(b' ')));
        assert!(!is_newline(i32::from(b'a')));
    }

    #[test]
    fn test_is_printable() {
        // ASCII printable
        assert!(is_printable(i32::from(b'a')));
        assert!(is_printable(i32::from(b'Z')));
        assert!(is_printable(i32::from(b'0')));
        assert!(is_printable(i32::from(b' ')));
        assert!(is_printable(i32::from(b'~')));

        // Non-printable ASCII
        assert!(!is_printable(0x00)); // NUL
        assert!(!is_printable(0x1f)); // Unit separator
        assert!(!is_printable(0x7f)); // DEL

        // Multi-byte (considered printable by our simplified check)
        assert!(is_printable(0x100));
        assert!(is_printable(0x1000));
    }

    #[test]
    fn test_insert_op_insert() {
        let op = InsertOp::insert(10, 5, 3);
        assert_eq!(op.lnum, 10);
        assert_eq!(op.col, 5);
        assert_eq!(op.insert_len, 3);
        assert_eq!(op.delete_len, 0);
        assert!(!op.is_replace);
    }

    #[test]
    fn test_insert_op_replace() {
        let op = InsertOp::replace(10, 5, 3, 2);
        assert_eq!(op.lnum, 10);
        assert_eq!(op.col, 5);
        assert_eq!(op.insert_len, 3);
        assert_eq!(op.delete_len, 2);
        assert!(op.is_replace);
    }

    #[test]
    fn test_mb_maxchar() {
        // MB_MAXCHAR should be 6 for UTF-8
        assert_eq!(MB_MAXCHAR, 6);
    }
}
