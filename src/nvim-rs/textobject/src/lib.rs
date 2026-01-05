//! Text object selection and navigation for Neovim
//!
//! This crate provides Rust implementations of text object functions
//! from `src/nvim/textobject.c`. It handles text object selection (aw, iw, as, is,
//! ap, ip, a", i", a{, i{, etc.) and word motions (w, W, b, B, e, E).

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Constants
// =============================================================================

/// Direction: move forward
pub const FORWARD: c_int = 1;

/// Direction: move backward
pub const BACKWARD: c_int = -1;

/// Function succeeded
pub const OK: c_int = 1;

/// Function failed
pub const FAIL: c_int = 0;

/// NUL character
pub const NUL: c_int = 0;

/// Whitespace character class
const CLASS_WHITESPACE: c_int = 0;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to a window (win_T*).
pub type WinHandle = *mut std::ffi::c_void;

/// Opaque handle to a buffer (buf_T*).
pub type BufHandle = *mut std::ffi::c_void;

/// Opaque handle to operator arguments (oparg_T*).
pub type OapHandle = *mut std::ffi::c_void;

/// Opaque handle to a position (pos_T*).
pub type PosHandle = *mut std::ffi::c_void;

// =============================================================================
// C Accessor Functions
// =============================================================================

extern "C" {
    /// Get character at cursor position via gchar_cursor().
    fn nvim_textobj_gchar_cursor() -> c_int;

    /// Increment cursor position. Returns -1 at EOF, 1 at EOL, 0 otherwise.
    fn nvim_textobj_inc_cursor() -> c_int;

    /// Decrement cursor position. Returns -1 at start, 1 at line start, 0 otherwise.
    fn nvim_textobj_dec_cursor() -> c_int;

    /// Get UTF character class (0=whitespace, 1=punct, 2+=word).
    fn nvim_textobj_utf_class(c: c_int) -> c_int;

    /// Get current cursor column.
    fn nvim_textobj_get_cursor_col() -> c_int;
}

// =============================================================================
// Character Classification
// =============================================================================

/// Get the class of character at cursor position.
///
/// Character classes:
/// - 0: whitespace (space, tab, NUL)
/// - 1: punctuation (or all non-blank when bigword is true)
/// - 2+: keyword characters (letters, digits, underscore)
///
/// If `bigword` is true (W, B, E motions), all non-blank characters
/// are reported as class 1 since only whitespace boundaries matter.
#[inline]
fn cls_impl(bigword: bool) -> c_int {
    // SAFETY: Accessor function is provided by C side
    let c = unsafe { nvim_textobj_gchar_cursor() };

    // Whitespace check: space, tab, or NUL
    if c == i32::from(b' ') || c == i32::from(b'\t') || c == NUL {
        return CLASS_WHITESPACE;
    }

    // SAFETY: Accessor function is provided by C side
    let class = unsafe { nvim_textobj_utf_class(c) };

    // If bigword is true, report all non-blanks as class 1
    if class != 0 && bigword {
        return 1;
    }

    class
}

/// FFI wrapper for cls() - get character class at cursor.
///
/// # Safety
/// Calls into C accessor functions which must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_cls(bigword: bool) -> c_int {
    cls_impl(bigword)
}

/// Skip characters of the same class in the given direction.
///
/// Returns true when end-of-file/start-of-file is reached, false otherwise.
#[inline]
fn skip_chars_impl(cclass: c_int, dir: c_int, bigword: bool) -> bool {
    // SAFETY: Accessor functions are provided by C side
    unsafe {
        while cls_impl(bigword) == cclass {
            let result = if dir == FORWARD {
                nvim_textobj_inc_cursor()
            } else {
                nvim_textobj_dec_cursor()
            };
            if result == -1 {
                return true;
            }
        }
    }
    false
}

/// FFI wrapper for skip_chars() - skip characters of same class.
///
/// # Safety
/// Calls into C accessor functions which must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_chars(cclass: c_int, dir: c_int, bigword: bool) -> bool {
    skip_chars_impl(cclass, dir, bigword)
}

/// Go back to the start of the word or the start of whitespace.
///
/// Moves cursor backward until it reaches the start of the line
/// or a different character class boundary.
#[inline]
fn back_in_line_impl(bigword: bool) {
    let sclass = cls_impl(bigword); // starting class

    // SAFETY: Accessor functions are provided by C side
    unsafe {
        loop {
            // Stop at start of line
            if nvim_textobj_get_cursor_col() == 0 {
                break;
            }

            nvim_textobj_dec_cursor();

            // Stop at start of word (different class)
            if cls_impl(bigword) != sclass {
                nvim_textobj_inc_cursor();
                break;
            }
        }
    }
}

/// FFI wrapper for back_in_line() - go back to word start.
///
/// # Safety
/// Calls into C accessor functions which must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_back_in_line(bigword: bool) {
    back_in_line_impl(bigword);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(FORWARD, 1);
        assert_eq!(BACKWARD, -1);
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
        assert_eq!(NUL, 0);
    }
}
