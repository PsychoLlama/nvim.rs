//! Command line screen position calculations
//!
//! This module provides utilities for calculating cursor positions and
//! screen coordinates in command-line mode.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int};

// =============================================================================
// C Function Declarations
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Global screen state
    fn nvim_get_columns() -> c_int;
    fn nvim_get_rows() -> c_int;
    fn nvim_get_key_typed() -> c_int;
    fn nvim_get_cmdline_star() -> c_int;
    fn nvim_get_cmdline_row() -> c_int;
    fn nvim_cmdline_win_is_active() -> c_int;
    fn nvim_cmdline_win_width() -> c_int;
    fn nvim_cmdline_win_height() -> c_int;

    // Command line state
    fn nvim_get_ccline_cmdpos() -> c_int;
    fn nvim_get_ccline_cmdlen() -> c_int;
    fn nvim_get_ccline_cmdspos() -> c_int;
    fn nvim_get_ccline_cmdindent() -> c_int;
    fn nvim_get_ccline_cmdfirstc() -> c_int;
    fn nvim_get_ccline_cmdbuff() -> *mut c_char;

    // Character utilities
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_ptr2cells(p: *const c_char) -> c_int;
    fn ptr2cells(p: *const c_char) -> c_int;
}

// =============================================================================
// Constants
// =============================================================================

/// Maximum column value (used for overflow protection)
pub const MAXCOL: c_int = 0x7FFF_FFFF;

/// NUL character
const NUL: c_int = 0;

// =============================================================================
// Screen Dimensions
// =============================================================================

/// Get the screen width in columns.
///
/// # Safety
///
/// Calls C function to access global.
#[must_use]
pub unsafe fn get_columns() -> c_int {
    nvim_get_columns()
}

/// Get the screen height in rows.
///
/// # Safety
///
/// Calls C function to access global.
#[must_use]
pub unsafe fn get_rows() -> c_int {
    nvim_get_rows()
}

/// Check if a key was typed (vs from a mapping).
///
/// # Safety
///
/// Calls C function to access global.
#[must_use]
pub unsafe fn key_was_typed() -> bool {
    nvim_get_key_typed() != 0
}

/// Get the cmdline_star flag (password mode).
///
/// # Safety
///
/// Calls C function to access global.
#[must_use]
pub unsafe fn get_cmdline_star() -> c_int {
    nvim_get_cmdline_star()
}

/// Check if password mode is active.
///
/// # Safety
///
/// Calls C function to access global.
#[must_use]
pub unsafe fn is_password_mode() -> bool {
    nvim_get_cmdline_star() > 0
}

/// Get the row where the command line starts.
///
/// # Safety
///
/// Calls C function to access global.
#[must_use]
pub unsafe fn get_cmdline_row() -> c_int {
    nvim_get_cmdline_row()
}

// =============================================================================
// Command Line Window
// =============================================================================

/// Check if a command-line window is active.
///
/// # Safety
///
/// Calls C function to check state.
#[must_use]
pub unsafe fn cmdline_win_active() -> bool {
    nvim_cmdline_win_is_active() != 0
}

/// Get the command-line window width (if active).
///
/// # Safety
///
/// Calls C function to access window state.
#[must_use]
pub unsafe fn cmdline_win_width() -> c_int {
    nvim_cmdline_win_width()
}

/// Get the command-line window height (if active).
///
/// # Safety
///
/// Calls C function to access window state.
#[must_use]
pub unsafe fn cmdline_win_height() -> c_int {
    nvim_cmdline_win_height()
}

// =============================================================================
// Position Calculations
// =============================================================================

/// Compute the starting column for the cursor (includes indent and prompt).
///
/// # Safety
///
/// Calls C functions to access ccline state.
#[must_use]
pub unsafe fn cmd_startcol() -> c_int {
    let cmdindent = nvim_get_ccline_cmdindent();
    let cmdfirstc = nvim_get_ccline_cmdfirstc();
    cmdindent + c_int::from(cmdfirstc != NUL)
}

/// Get the character width at a byte position in the command buffer.
///
/// If in password mode, always returns 1 (showing '*').
///
/// # Safety
///
/// `idx` must be a valid byte offset in the command buffer.
/// Calls C functions to access buffer and calculate width.
#[must_use]
pub unsafe fn cmdline_charsize(idx: c_int) -> c_int {
    if nvim_get_cmdline_star() > 0 {
        return 1;
    }
    let cmdbuff = nvim_get_ccline_cmdbuff();
    if cmdbuff.is_null() {
        return 1;
    }
    ptr2cells(cmdbuff.add(idx as usize))
}

/// Correct screen column for multi-byte characters that don't fit.
///
/// When a double-wide character doesn't fit at the end of a line,
/// we need to account for the '>' placeholder.
///
/// # Safety
///
/// `idx` must be a valid byte offset in the command buffer.
/// Calls C functions to access buffer and check character width.
pub unsafe fn correct_screencol(idx: c_int, cells: c_int, col: &mut c_int) {
    let cmdbuff = nvim_get_ccline_cmdbuff();
    if cmdbuff.is_null() {
        return;
    }
    let p = cmdbuff.add(idx as usize);
    let columns = nvim_get_columns();

    // If multi-byte char (>1 byte) is double-wide (2 cells)
    // and doesn't fit at end of line, increment column
    if utfc_ptr2len(p) > 1 && utf_ptr2cells(p) > 1 && (*col % columns) + cells > columns {
        *col += 1;
    }
}

/// Compute the column position for a byte position on the command line.
///
/// This is the main function for converting byte positions to screen positions.
///
/// # Safety
///
/// `bytepos` must be a valid byte offset in the command buffer.
/// Calls C functions to access buffer and screen state.
#[must_use]
pub unsafe fn cmd_screencol(bytepos: c_int) -> c_int {
    let columns = nvim_get_columns();
    let rows = nvim_get_rows();
    let mut col = cmd_startcol();

    // Calculate maximum displayable column
    let m = if key_was_typed() {
        if cmdline_win_active() {
            let wwidth = nvim_cmdline_win_width();
            let wheight = nvim_cmdline_win_height();
            let product = wwidth.saturating_mul(wheight);
            if product < 0 { MAXCOL } else { product }
        } else {
            let product = columns.saturating_mul(rows);
            if product < 0 { MAXCOL } else { product }
        }
    } else {
        MAXCOL
    };

    let cmdbuff = nvim_get_ccline_cmdbuff();
    if cmdbuff.is_null() {
        return col;
    }

    let cmdlen = nvim_get_ccline_cmdlen();
    let mut i = 0;

    while i < cmdlen && i < bytepos {
        let c = cmdline_charsize(i);

        // Count ">" for double-wide multi-byte char that doesn't fit
        correct_screencol(i, c, &mut col);

        // If the cmdline doesn't fit, show cursor on last visible char
        if col + c >= m {
            break;
        }
        col += c;

        // Move to next character
        i += utfc_ptr2len(cmdbuff.add(i as usize));
    }

    col
}

/// Get the screen position of the cursor.
///
/// # Safety
///
/// Calls C functions to access ccline state.
#[must_use]
pub unsafe fn cursor_screencol() -> c_int {
    cmd_screencol(nvim_get_ccline_cmdpos())
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Compute starting column for cursor on command line.
///
/// # Safety
///
/// Calls C functions to access ccline state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmd_startcol() -> c_int {
    cmd_startcol()
}

/// Get character width at position in command buffer.
///
/// # Safety
///
/// `idx` must be valid byte offset.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_charsize(idx: c_int) -> c_int {
    cmdline_charsize(idx)
}

/// Compute screen column for byte position.
///
/// # Safety
///
/// `bytepos` must be valid byte offset.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmd_screencol(bytepos: c_int) -> c_int {
    cmd_screencol(bytepos)
}

/// Get cursor screen column.
///
/// # Safety
///
/// Calls C functions to access state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cursor_screencol() -> c_int {
    cursor_screencol()
}

/// Check if password mode is active.
///
/// # Safety
///
/// Calls C function to access global.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_is_password_mode() -> c_int {
    c_int::from(is_password_mode())
}

/// Get screen columns.
///
/// # Safety
///
/// Calls C function to access global.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_get_columns() -> c_int {
    nvim_get_columns()
}

/// Get screen rows.
///
/// # Safety
///
/// Calls C function to access global.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_get_rows() -> c_int {
    nvim_get_rows()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maxcol() {
        assert_eq!(MAXCOL, 0x7FFF_FFFF);
    }

    #[test]
    fn test_nul_constant() {
        assert_eq!(NUL, 0);
    }
}
