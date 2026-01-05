//! Terminal emulator utilities for Neovim
//!
//! This crate provides Rust implementations for terminal-related functions,
//! primarily working with the libvterm-based terminal emulator.

#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::must_use_candidate)]

use std::ffi::c_int;
use std::os::raw::c_void;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to Terminal struct (terminal.c struct terminal)
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalHandle(*mut c_void);

impl TerminalHandle {
    /// Create a handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `Terminal*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// C Accessor Functions
// =============================================================================

extern "C" {
    /// Get the `closed` field from a Terminal.
    fn nvim_terminal_get_closed(term: TerminalHandle) -> c_int;

    /// Get the `buf_handle` field from a Terminal.
    fn nvim_terminal_get_buf_handle(term: TerminalHandle) -> c_int;

    /// Get the `theme_updates` field from a Terminal.
    fn nvim_terminal_get_theme_updates(term: TerminalHandle) -> c_int;

    /// Get the `forward_mouse` field from a Terminal.
    fn nvim_terminal_get_forward_mouse(term: TerminalHandle) -> c_int;

    /// Get the cursor row from a Terminal.
    fn nvim_terminal_get_cursor_row(term: TerminalHandle) -> c_int;

    /// Get the cursor col from a Terminal.
    fn nvim_terminal_get_cursor_col(term: TerminalHandle) -> c_int;

    /// Get the cursor visible flag from a Terminal.
    fn nvim_terminal_get_cursor_visible(term: TerminalHandle) -> c_int;

    /// Get the cursor shape from a Terminal.
    fn nvim_terminal_get_cursor_shape(term: TerminalHandle) -> c_int;

    /// Get the cursor blink flag from a Terminal.
    fn nvim_terminal_get_cursor_blink(term: TerminalHandle) -> c_int;
}

// =============================================================================
// Terminal Status Functions
// =============================================================================

/// Check if a terminal is running (not closed).
///
/// This is the Rust equivalent of `terminal_running()` in terminal.c.
#[inline]
fn terminal_running_impl(term: TerminalHandle) -> bool {
    if term.is_null() {
        return false;
    }
    unsafe { nvim_terminal_get_closed(term) == 0 }
}

/// FFI wrapper for `terminal_running`.
///
/// Returns 1 if the terminal is running, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_terminal_running(term: TerminalHandle) -> c_int {
    c_int::from(terminal_running_impl(term))
}

// =============================================================================
// Terminal Buffer Functions
// =============================================================================

/// Get the buffer handle associated with a terminal.
///
/// This is the Rust equivalent of `terminal_buf()` in terminal.c.
#[inline]
fn terminal_buf_impl(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    unsafe { nvim_terminal_get_buf_handle(term) }
}

/// FFI wrapper for `terminal_buf`.
#[no_mangle]
pub extern "C" fn rs_terminal_buf(term: TerminalHandle) -> c_int {
    terminal_buf_impl(term)
}

// =============================================================================
// Terminal Cursor Functions
// =============================================================================

/// Get the cursor row for a terminal.
#[inline]
fn terminal_cursor_row_impl(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    unsafe { nvim_terminal_get_cursor_row(term) }
}

/// FFI wrapper for getting terminal cursor row.
#[no_mangle]
pub extern "C" fn rs_terminal_cursor_row(term: TerminalHandle) -> c_int {
    terminal_cursor_row_impl(term)
}

/// Get the cursor column for a terminal.
#[inline]
fn terminal_cursor_col_impl(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    unsafe { nvim_terminal_get_cursor_col(term) }
}

/// FFI wrapper for getting terminal cursor column.
#[no_mangle]
pub extern "C" fn rs_terminal_cursor_col(term: TerminalHandle) -> c_int {
    terminal_cursor_col_impl(term)
}

/// Check if the terminal cursor is visible.
#[inline]
fn terminal_cursor_visible_impl(term: TerminalHandle) -> bool {
    if term.is_null() {
        return false;
    }
    unsafe { nvim_terminal_get_cursor_visible(term) != 0 }
}

/// FFI wrapper for checking if terminal cursor is visible.
#[no_mangle]
pub extern "C" fn rs_terminal_cursor_visible(term: TerminalHandle) -> c_int {
    c_int::from(terminal_cursor_visible_impl(term))
}

/// Get the terminal cursor shape.
#[inline]
fn terminal_cursor_shape_impl(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    unsafe { nvim_terminal_get_cursor_shape(term) }
}

/// FFI wrapper for getting terminal cursor shape.
#[no_mangle]
pub extern "C" fn rs_terminal_cursor_shape(term: TerminalHandle) -> c_int {
    terminal_cursor_shape_impl(term)
}

/// Check if the terminal cursor should blink.
#[inline]
fn terminal_cursor_blink_impl(term: TerminalHandle) -> bool {
    if term.is_null() {
        return false;
    }
    unsafe { nvim_terminal_get_cursor_blink(term) != 0 }
}

/// FFI wrapper for checking if terminal cursor should blink.
#[no_mangle]
pub extern "C" fn rs_terminal_cursor_blink(term: TerminalHandle) -> c_int {
    c_int::from(terminal_cursor_blink_impl(term))
}

// =============================================================================
// Terminal Property Functions
// =============================================================================

/// Check if the terminal forwards mouse events.
#[inline]
fn terminal_forward_mouse_impl(term: TerminalHandle) -> bool {
    if term.is_null() {
        return false;
    }
    unsafe { nvim_terminal_get_forward_mouse(term) != 0 }
}

/// FFI wrapper for checking if terminal forwards mouse.
#[no_mangle]
pub extern "C" fn rs_terminal_forward_mouse(term: TerminalHandle) -> c_int {
    c_int::from(terminal_forward_mouse_impl(term))
}

/// Check if the terminal wants theme update notifications.
#[inline]
fn terminal_theme_updates_impl(term: TerminalHandle) -> bool {
    if term.is_null() {
        return false;
    }
    unsafe { nvim_terminal_get_theme_updates(term) != 0 }
}

/// FFI wrapper for checking if terminal wants theme updates.
#[no_mangle]
pub extern "C" fn rs_terminal_theme_updates(term: TerminalHandle) -> c_int {
    c_int::from(terminal_theme_updates_impl(term))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_handle_null() {
        let handle = unsafe { TerminalHandle::from_ptr(std::ptr::null_mut()) };
        assert!(handle.is_null());
        assert!(!terminal_running_impl(handle));
        assert!(!terminal_forward_mouse_impl(handle));
        assert!(!terminal_theme_updates_impl(handle));
    }

    #[test]
    fn test_terminal_handle_non_null() {
        let fake_ptr = 0x1000 as *mut c_void;
        let handle = unsafe { TerminalHandle::from_ptr(fake_ptr) };
        assert!(!handle.is_null());
        assert_eq!(handle.as_ptr(), fake_ptr);
    }
}
