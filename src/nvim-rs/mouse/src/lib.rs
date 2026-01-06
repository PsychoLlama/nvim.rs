//! Mouse event handling for Neovim
//!
//! This crate provides Rust implementations of mouse-related functions
//! from `src/nvim/mouse.c`. It handles:
//! - Mouse button state tracking
//! - Click position computation
//! - Drag state machine
//! - Mouse mode flags
//!
//! The crate uses the opaque handle pattern for window access.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const

use std::ffi::{c_char, c_int};

// =============================================================================
// Mouse button constants (from mouse.h)
// =============================================================================

/// Left mouse button
pub const MOUSE_LEFT: c_int = 0x00;
/// Middle mouse button
pub const MOUSE_MIDDLE: c_int = 0x01;
/// Right mouse button
pub const MOUSE_RIGHT: c_int = 0x02;
/// Mouse button release
pub const MOUSE_RELEASE: c_int = 0x03;
/// Mouse button X1 (6th button)
pub const MOUSE_X1: c_int = 0x300;
/// Mouse button X2
pub const MOUSE_X2: c_int = 0x400;

// =============================================================================
// jump_to_mouse() return values (from mouse.h)
// =============================================================================

/// Unknown position
pub const IN_UNKNOWN: c_int = 0;
/// In buffer text
pub const IN_BUFFER: c_int = 1;
/// On status or command line
pub const IN_STATUS_LINE: c_int = 2;
/// On vertical separator line
pub const IN_SEP_LINE: c_int = 4;
/// In other window but can't go there
pub const IN_OTHER_WIN: c_int = 8;
/// Cursor has moved
pub const CURSOR_MOVED: c_int = 0x100;
/// Clicked on '-' in fold column
pub const MOUSE_FOLD_CLOSE: c_int = 0x200;
/// Clicked on '+' in fold column
pub const MOUSE_FOLD_OPEN: c_int = 0x400;
/// In window toolbar
pub const MOUSE_WINBAR: c_int = 0x800;
/// In 'statuscolumn'
pub const MOUSE_STATUSCOL: c_int = 0x1000;

// =============================================================================
// Flags for jump_to_mouse() (from mouse.h)
// =============================================================================

/// Need to stay in this window
pub const MOUSE_FOCUS: c_int = 0x01;
/// May start Visual mode
pub const MOUSE_MAY_VIS: c_int = 0x02;
/// Only act when mouse has moved
pub const MOUSE_DID_MOVE: c_int = 0x04;
/// Only set current mouse position
pub const MOUSE_SETPOS: c_int = 0x08;
/// May stop Visual mode
pub const MOUSE_MAY_STOP_VIS: c_int = 0x10;
/// Button was released
pub const MOUSE_RELEASED: c_int = 0x20;

// =============================================================================
// Scroll direction constants (from mouse.h)
// =============================================================================

/// Scroll down (must be false/0)
pub const MSCR_DOWN: c_int = 0;
/// Scroll up
pub const MSCR_UP: c_int = 1;
/// Scroll left
pub const MSCR_LEFT: c_int = -1;
/// Scroll right
pub const MSCR_RIGHT: c_int = -2;

// =============================================================================
// Character class for word selection
// =============================================================================

/// Character class for mouse selection:
/// - 0: blank (space, tab)
/// - 1: punctuation groups (-+*/%<>&|^!=)
/// - 2: normal word character
/// - >2: multi-byte word character class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct CharClass(pub c_int);

impl CharClass {
    /// Blank character (space or tab)
    pub const BLANK: Self = Self(0);
    /// Punctuation group
    pub const PUNCTUATION: Self = Self(1);
    /// Normal word character
    pub const WORD: Self = Self(2);
}

// =============================================================================
// ASCII constants
// =============================================================================

const NUL: u8 = 0;
const SPACE: u8 = b' ';
const TAB: u8 = b'\t';

// =============================================================================
// Imports from other crates
// =============================================================================

// Re-use existing Rust implementations from mbyte and charset crates
use nvim_charset::rs_vim_iswordc;
use nvim_mbyte::{rs_mb_get_class, rs_utf_ptr2len};

// =============================================================================
// Type aliases for C types
// =============================================================================

/// Line number type (from `pos_defs.h`)
#[allow(non_camel_case_types)]
pub type linenr_T = i64;

/// Opaque handle for window pointer
pub type WinHandle = *mut std::ffi::c_void;

// =============================================================================
// C accessors for mouse state
// =============================================================================

#[allow(dead_code)]
extern "C" {
    /// Get the original topline for double-click detection.
    fn nvim_get_orig_topline() -> linenr_T;

    /// Set the original topline for double-click detection.
    fn nvim_set_orig_topline(val: linenr_T);

    /// Get the original topfill for double-click detection.
    fn nvim_get_orig_topfill() -> c_int;

    /// Set the original topfill for double-click detection.
    fn nvim_set_orig_topfill(val: c_int);

    /// Get whether a click was received.
    fn nvim_get_got_click() -> bool;

    /// Set whether a click was received.
    fn nvim_set_got_click(val: bool);

    /// Get the window being dragged.
    fn nvim_get_dragwin() -> WinHandle;

    /// Set the window being dragged.
    fn nvim_set_dragwin(wp: WinHandle);

    /// Check if a window is being dragged.
    fn nvim_is_dragging() -> bool;
}

// =============================================================================
// Character Classification Functions
// =============================================================================

/// Get class of a character for mouse word selection.
///
/// Returns:
/// - 0: blank (space, tab)
/// - 1: punctuation groups
/// - 2: normal word character
/// - >2: multi-byte word character class
///
/// # Safety
/// `p` must be a valid pointer to a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_mouse_class(p: *const c_char) -> c_int {
    if p.is_null() {
        return 0;
    }

    // Check for multi-byte character
    let first_byte = *p.cast::<u8>();
    if rs_utf_ptr2len(p) > 1 {
        return rs_mb_get_class(p);
    }

    // Single-byte character checks
    if first_byte == SPACE || first_byte == TAB {
        return CharClass::BLANK.0;
    }

    if rs_vim_iswordc(c_int::from(first_byte)) != 0 {
        return CharClass::WORD.0;
    }

    // Check for punctuation groups (-+*/%<>&|^!=)
    if first_byte != NUL {
        let punct_chars = b"-+*/%<>&|^!=";
        if punct_chars.contains(&first_byte) {
            return CharClass::PUNCTUATION.0;
        }
    }

    // Each character is its own class
    c_int::from(first_byte)
}

/// Check if 'mousemodel' is set to "popup" or "`popup_setpos`".
///
/// Returns true when the first character of 'mousem' is 'p'.
///
/// # Safety
/// `p_mousem` must be a valid pointer to a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_mouse_model_popup(p_mousem: *const c_char) -> bool {
    if p_mousem.is_null() {
        return false;
    }
    *p_mousem.cast::<u8>() == b'p'
}

// =============================================================================
// External UTF-8 helpers from mbyte crate
// =============================================================================

extern "C" {
    /// Get length of UTF-8 character at pointer
    fn rs_utfc_ptr2len(p: *const c_char) -> c_int;

    /// Get offset to start of UTF-8 character
    fn rs_utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
}

// =============================================================================
// Word Boundary Detection Functions
// =============================================================================

/// Find the start of the word at the given column.
///
/// Given a line and starting column, returns the column position of the
/// start of the word that contains the starting column position.
///
/// # Safety
/// - `line` must be a valid pointer to a NUL-terminated string.
/// - `col` must be a valid byte offset within the line.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_find_start_of_word(line: *const c_char, col: c_int) -> c_int {
    if line.is_null() || col <= 0 {
        return 0;
    }

    let mut pos_col = col;
    let cclass = rs_get_mouse_class(line.add(pos_col as usize));

    while pos_col > 0 {
        // Move back one character
        let mut new_col = pos_col - 1;
        // Adjust for multi-byte character start
        new_col -= rs_utf_head_off(line, line.add(new_col as usize));

        // Check if character class changed
        if rs_get_mouse_class(line.add(new_col as usize)) != cclass {
            break;
        }
        pos_col = new_col;
    }

    pos_col
}

/// Find the end of the word at the given column.
///
/// Given a line and starting column, returns the column position of the
/// end of the word that contains the starting column position.
/// If `sel_exclusive` is true, the position is just after the word (for exclusive selection).
///
/// # Safety
/// - `line` must be a valid pointer to a NUL-terminated string.
/// - `col` must be a valid byte offset within the line.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_find_end_of_word(
    line: *const c_char,
    col: c_int,
    sel_exclusive: bool,
) -> c_int {
    if line.is_null() {
        return col;
    }

    let mut pos_col = col;

    // For exclusive selection, adjust start position if col > 0
    if sel_exclusive && pos_col > 0 {
        pos_col -= 1;
        pos_col -= rs_utf_head_off(line, line.add(pos_col as usize));
    }

    let cclass = rs_get_mouse_class(line.add(pos_col as usize));

    // Scan forward while same character class
    while *line.add(pos_col as usize) != 0 {
        let next_col = pos_col + rs_utfc_ptr2len(line.add(pos_col as usize));
        if rs_get_mouse_class(line.add(next_col as usize)) != cclass {
            // For exclusive selection, move past the last character
            if sel_exclusive {
                return next_col;
            }
            break;
        }
        pos_col = next_col;
    }

    pos_col
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mouse_button_constants() {
        assert_eq!(MOUSE_LEFT, 0x00);
        assert_eq!(MOUSE_MIDDLE, 0x01);
        assert_eq!(MOUSE_RIGHT, 0x02);
        assert_eq!(MOUSE_RELEASE, 0x03);
        assert_eq!(MOUSE_X1, 0x300);
        assert_eq!(MOUSE_X2, 0x400);
    }

    #[test]
    fn test_jump_to_mouse_constants() {
        assert_eq!(IN_UNKNOWN, 0);
        assert_eq!(IN_BUFFER, 1);
        assert_eq!(IN_STATUS_LINE, 2);
        assert_eq!(IN_SEP_LINE, 4);
        assert_eq!(IN_OTHER_WIN, 8);
        assert_eq!(CURSOR_MOVED, 0x100);
        assert_eq!(MOUSE_FOLD_CLOSE, 0x200);
        assert_eq!(MOUSE_FOLD_OPEN, 0x400);
        assert_eq!(MOUSE_WINBAR, 0x800);
        assert_eq!(MOUSE_STATUSCOL, 0x1000);
    }

    #[test]
    fn test_mouse_flags_constants() {
        assert_eq!(MOUSE_FOCUS, 0x01);
        assert_eq!(MOUSE_MAY_VIS, 0x02);
        assert_eq!(MOUSE_DID_MOVE, 0x04);
        assert_eq!(MOUSE_SETPOS, 0x08);
        assert_eq!(MOUSE_MAY_STOP_VIS, 0x10);
        assert_eq!(MOUSE_RELEASED, 0x20);
    }

    #[test]
    fn test_scroll_constants() {
        assert_eq!(MSCR_DOWN, 0);
        assert_eq!(MSCR_UP, 1);
        assert_eq!(MSCR_LEFT, -1);
        assert_eq!(MSCR_RIGHT, -2);
    }

    #[test]
    fn test_char_class() {
        assert_eq!(CharClass::BLANK.0, 0);
        assert_eq!(CharClass::PUNCTUATION.0, 1);
        assert_eq!(CharClass::WORD.0, 2);
    }

    #[test]
    fn test_mouse_model_popup_null() {
        unsafe {
            assert!(!rs_mouse_model_popup(std::ptr::null()));
        }
    }

    #[test]
    fn test_mouse_model_popup_popup() {
        let popup = b"popup\0";
        unsafe {
            assert!(rs_mouse_model_popup(popup.as_ptr().cast()));
        }
    }

    #[test]
    fn test_mouse_model_popup_other() {
        let extend = b"extend\0";
        unsafe {
            assert!(!rs_mouse_model_popup(extend.as_ptr().cast()));
        }
    }
}
