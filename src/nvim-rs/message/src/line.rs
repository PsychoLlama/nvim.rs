//! Line printing utilities
//!
//! Provides helpers for printing lines with the :print and :list commands,
//! handling special characters, tabs, and list mode display.

use std::ffi::c_int;

// C function declarations
extern "C" {
    static Columns: c_int;
    static mut got_int: bool;
    /// Check if in list mode
    fn nvim_get_list_mode() -> c_int;
    /// Get current message column
    fn nvim_get_msg_col() -> c_int;
}

// ============================================================================
// Line Display Constants
// ============================================================================

/// Default tab width
pub const DEFAULT_TAB_WIDTH: c_int = 8;

/// Character for end of line in list mode (typically $)
pub const LIST_EOL_CHAR: c_int = b'$' as c_int;

/// Character for tab in list mode (typically >)
pub const LIST_TAB_CHAR: c_int = b'>' as c_int;

/// Character for trailing space in list mode (typically ·)
pub const LIST_TRAIL_CHAR: c_int = b'.' as c_int;

// ============================================================================
// Tab Handling
// ============================================================================

/// Calculate tab padding based on current column.
///
/// Returns the number of spaces to the next tab stop.
#[no_mangle]
pub const extern "C" fn rs_tab_padding(col: c_int, tabstop: c_int) -> c_int {
    if tabstop <= 0 {
        return DEFAULT_TAB_WIDTH - (col % DEFAULT_TAB_WIDTH);
    }
    tabstop - (col % tabstop)
}

/// Calculate the column position after a tab.
#[no_mangle]
pub const extern "C" fn rs_tab_next_col(col: c_int, tabstop: c_int) -> c_int {
    col + rs_tab_padding(col, tabstop)
}

/// Check if column is at a tab stop.
#[no_mangle]
pub const extern "C" fn rs_is_tab_stop(col: c_int, tabstop: c_int) -> c_int {
    if tabstop <= 0 {
        return (col % DEFAULT_TAB_WIDTH == 0) as c_int;
    }
    (col % tabstop == 0) as c_int
}

// ============================================================================
// Line State Helpers
// ============================================================================

/// Check if line printing should continue.
///
/// Returns false if interrupted (unsafe { got_int } set).
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_line_continue() -> c_int {
    c_int::from(!unsafe { got_int })
}

/// Calculate remaining columns on current line.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_line_remaining() -> c_int {
    let columns = Columns;
    let msg_col = nvim_get_msg_col();
    if msg_col < columns {
        columns - msg_col
    } else {
        0
    }
}

/// Check if current position is at end of line.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_at_line_end() -> c_int {
    c_int::from(rs_line_remaining() == 0)
}

// ============================================================================
// Character Classification for Line Printing
// ============================================================================

// Note: rs_is_whitespace is defined in edit/insert.rs

/// Check if character is a space.
#[no_mangle]
pub const extern "C" fn rs_is_space(c: c_int) -> c_int {
    (c == b' ' as c_int) as c_int
}

/// Check if character is a tab.
#[no_mangle]
pub const extern "C" fn rs_is_tab(c: c_int) -> c_int {
    (c == b'\t' as c_int) as c_int
}

/// Check if character is printable ASCII.
#[no_mangle]
pub const extern "C" fn rs_is_printable_ascii(c: c_int) -> c_int {
    (c >= 0x20 && c < 0x7F) as c_int
}

/// Check if character needs special display in list mode.
#[no_mangle]
pub const extern "C" fn rs_needs_list_display(c: c_int) -> c_int {
    // Needs special display: NUL, TAB, trailing space, non-printable
    (c == 0 || c == b'\t' as c_int || c < 0x20 || c >= 0x7F) as c_int
}

// ============================================================================
// List Mode Display
// ============================================================================

/// Get the display character for a control character in list mode.
///
/// Returns the character to show after ^ (e.g., ^I for tab).
#[no_mangle]
pub const extern "C" fn rs_ctrl_display_char(c: c_int) -> c_int {
    if c >= 0 && c < 0x20 {
        // Control characters display as ^@ through ^_
        b'@' as c_int + c
    } else if c == 0x7F {
        // DEL displays as ^?
        b'?' as c_int
    } else {
        // Not a control character
        c
    }
}

// Note: rs_char_cells is defined in edit/replace.rs

/// Calculate display width of a character for line printing.
///
/// Control characters take 2 cells (^X), TAB variable, others 1.
/// This is a const helper for use in this module.
const fn line_char_cells(c: c_int, col: c_int, tabstop: c_int) -> c_int {
    if c == b'\t' as c_int {
        rs_tab_padding(col, tabstop)
    } else if (c >= 0 && c < 0x20) || c == 0x7F {
        // Control characters (^X format) and DEL (^?)
        2
    } else {
        1
    }
}

/// Check if in list mode.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_in_list_mode() -> c_int {
    nvim_get_list_mode()
}

// ============================================================================
// Column Tracking
// ============================================================================

/// Calculate new column after outputting a character.
#[no_mangle]
pub const extern "C" fn rs_advance_col(col: c_int, c: c_int, tabstop: c_int) -> c_int {
    col + line_char_cells(c, col, tabstop)
}

/// Calculate columns needed to display a string of given length.
///
/// Assumes all characters are single-width printable ASCII.
#[no_mangle]
pub const extern "C" fn rs_string_cells(len: c_int) -> c_int {
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_padding() {
        assert_eq!(rs_tab_padding(0, 8), 8);
        assert_eq!(rs_tab_padding(1, 8), 7);
        assert_eq!(rs_tab_padding(7, 8), 1);
        assert_eq!(rs_tab_padding(8, 8), 8);
    }

    #[test]
    fn test_tab_next_col() {
        assert_eq!(rs_tab_next_col(0, 8), 8);
        assert_eq!(rs_tab_next_col(1, 8), 8);
        assert_eq!(rs_tab_next_col(8, 8), 16);
    }

    #[test]
    fn test_is_tab_stop() {
        assert_eq!(rs_is_tab_stop(0, 8), 1);
        assert_eq!(rs_is_tab_stop(8, 8), 1);
        assert_eq!(rs_is_tab_stop(1, 8), 0);
        assert_eq!(rs_is_tab_stop(7, 8), 0);
    }

    #[test]
    fn test_is_printable_ascii() {
        assert_eq!(rs_is_printable_ascii(c_int::from(b' ')), 1);
        assert_eq!(rs_is_printable_ascii(c_int::from(b'~')), 1);
        assert_eq!(rs_is_printable_ascii(0x1F), 0);
        assert_eq!(rs_is_printable_ascii(0x7F), 0);
    }

    #[test]
    fn test_ctrl_display_char() {
        assert_eq!(rs_ctrl_display_char(0), c_int::from(b'@')); // ^@
        assert_eq!(rs_ctrl_display_char(9), c_int::from(b'I')); // ^I (tab)
        assert_eq!(rs_ctrl_display_char(0x7F), c_int::from(b'?')); // ^?
    }

    #[test]
    fn test_line_char_cells() {
        assert_eq!(line_char_cells(c_int::from(b'a'), 0, 8), 1);
        assert_eq!(line_char_cells(0, 0, 8), 2); // ^@
        assert_eq!(line_char_cells(c_int::from(b'\t'), 0, 8), 8);
        assert_eq!(line_char_cells(c_int::from(b'\t'), 4, 8), 4);
    }
}
