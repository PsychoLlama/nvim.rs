//! Display command implementations.
//!
//! This module provides implementations for Ex commands that display buffer content:
//! - `:print` (`:p`) - Print lines
//! - `:number` (`:nu`, `:#`) - Print lines with line numbers
//! - `:list` (`:l`) - Print lines with special characters visible
//! - `:=` - Print line number
//!
//! ## Implementation Notes
//!
//! These commands output text to the message area. This module provides
//! type definitions and formatting helpers. The actual output is performed
//! by Neovim's message system.

use std::ffi::c_int;

use crate::range::{LineNr, LineRange};

/// Display mode for `:print`, `:number`, and `:list` commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum DisplayMode {
    /// Normal print (`:p`)
    #[default]
    Print = 0,
    /// Print with line numbers (`:nu`, `:#`)
    Number = 1,
    /// Print with special characters visible (`:l`)
    List = 2,
}

impl DisplayMode {
    /// Check if line numbers should be shown.
    #[inline]
    #[must_use]
    pub const fn shows_numbers(&self) -> bool {
        matches!(self, DisplayMode::Number)
    }

    /// Check if special characters should be visible.
    #[inline]
    #[must_use]
    pub const fn shows_specials(&self) -> bool {
        matches!(self, DisplayMode::List)
    }

    /// Convert from C integer.
    #[inline]
    #[must_use]
    pub fn from_c(value: c_int) -> Self {
        match value {
            0 => DisplayMode::Print,
            1 => DisplayMode::Number,
            2 => DisplayMode::List,
            _ => DisplayMode::Print,
        }
    }

    /// Convert to C integer.
    #[inline]
    #[must_use]
    pub fn to_c(self) -> c_int {
        self as c_int
    }
}

/// Flags for display commands (from exarg.flags).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DisplayFlags {
    /// Show as list (l flag).
    pub list: bool,
    /// Show line numbers (# flag).
    pub number: bool,
    /// Print mode (p flag).
    pub print: bool,
}

impl DisplayFlags {
    /// Create default display flags.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            list: false,
            number: false,
            print: false,
        }
    }

    /// Parse from exarg flags value.
    #[must_use]
    pub const fn from_exflag(flags: c_int) -> Self {
        Self {
            list: (flags & 0x01) != 0,   // EXFLAG_LIST
            number: (flags & 0x02) != 0, // EXFLAG_NR
            print: (flags & 0x04) != 0,  // EXFLAG_PRINT
        }
    }

    /// Convert to exarg flags value.
    #[must_use]
    pub const fn to_exflag(&self) -> c_int {
        let mut flags = 0;
        if self.list {
            flags |= 0x01;
        }
        if self.number {
            flags |= 0x02;
        }
        if self.print {
            flags |= 0x04;
        }
        flags
    }

    /// Get the effective display mode from these flags.
    #[must_use]
    pub const fn display_mode(&self) -> DisplayMode {
        if self.list {
            DisplayMode::List
        } else if self.number {
            DisplayMode::Number
        } else {
            DisplayMode::Print
        }
    }
}

/// Options for display commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DisplayOptions {
    /// Range of lines to display.
    pub range: LineRange,
    /// Display mode.
    pub mode: DisplayMode,
    /// Additional flags.
    pub flags: DisplayFlags,
}

impl DisplayOptions {
    /// Create options for printing a range.
    #[must_use]
    pub const fn print(range: LineRange) -> Self {
        Self {
            range,
            mode: DisplayMode::Print,
            flags: DisplayFlags::new(),
        }
    }

    /// Create options for printing with line numbers.
    #[must_use]
    pub const fn number(range: LineRange) -> Self {
        Self {
            range,
            mode: DisplayMode::Number,
            flags: DisplayFlags {
                list: false,
                number: true,
                print: false,
            },
        }
    }

    /// Create options for list mode.
    #[must_use]
    pub const fn list(range: LineRange) -> Self {
        Self {
            range,
            mode: DisplayMode::List,
            flags: DisplayFlags {
                list: true,
                number: false,
                print: false,
            },
        }
    }
}

/// Special characters to display in list mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListChars {
    /// Character to show at end of line.
    pub eol: char,
    /// Character to show for tab.
    pub tab: char,
    /// Character to show for trailing spaces.
    pub trail: char,
    /// Character to show for non-breakable space.
    pub nbsp: char,
}

impl Default for ListChars {
    fn default() -> Self {
        Self {
            eol: '$',
            tab: '>',
            trail: '-',
            nbsp: '+',
        }
    }
}

impl ListChars {
    /// Create default list characters.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            eol: '$',
            tab: '>',
            trail: '-',
            nbsp: '+',
        }
    }
}

/// Line number formatting options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineNumberFormat {
    /// Minimum width for line numbers.
    pub min_width: usize,
    /// Right-align line numbers.
    pub right_align: bool,
}

impl Default for LineNumberFormat {
    fn default() -> Self {
        Self {
            min_width: 7,
            right_align: true,
        }
    }
}

impl LineNumberFormat {
    /// Create a new format with specified width.
    #[must_use]
    pub const fn with_width(min_width: usize) -> Self {
        Self {
            min_width,
            right_align: true,
        }
    }

    /// Calculate the width needed for a maximum line number.
    #[must_use]
    pub fn width_for_max(max_lnum: LineNr) -> usize {
        if max_lnum <= 0 {
            return 1;
        }
        // Number of digits in max_lnum
        let mut n = max_lnum;
        let mut digits = 0;
        while n > 0 {
            digits += 1;
            n /= 10;
        }
        digits
    }
}

/// Result of the `:=` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineNumberResult {
    /// The line number.
    pub lnum: LineNr,
}

impl LineNumberResult {
    /// Create a new result.
    #[must_use]
    pub const fn new(lnum: LineNr) -> Self {
        Self { lnum }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Convert display mode from exarg flags.
///
/// Returns 0 for print, 1 for number, 2 for list.
pub extern "C" fn rs_display_mode_from_flags(flags: c_int) -> c_int {
    DisplayFlags::from_exflag(flags).display_mode().to_c()
}

/// Calculate width needed for line numbers.
pub extern "C" fn rs_line_number_width(max_lnum: c_int) -> c_int {
    LineNumberFormat::width_for_max(max_lnum) as c_int
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_mode() {
        assert!(!DisplayMode::Print.shows_numbers());
        assert!(DisplayMode::Number.shows_numbers());
        assert!(!DisplayMode::List.shows_numbers());

        assert!(!DisplayMode::Print.shows_specials());
        assert!(!DisplayMode::Number.shows_specials());
        assert!(DisplayMode::List.shows_specials());
    }

    #[test]
    fn test_display_mode_from_c() {
        assert_eq!(DisplayMode::from_c(0), DisplayMode::Print);
        assert_eq!(DisplayMode::from_c(1), DisplayMode::Number);
        assert_eq!(DisplayMode::from_c(2), DisplayMode::List);
        assert_eq!(DisplayMode::from_c(99), DisplayMode::Print);
    }

    #[test]
    fn test_display_flags() {
        let flags = DisplayFlags::new();
        assert!(!flags.list);
        assert!(!flags.number);
        assert!(!flags.print);
    }

    #[test]
    fn test_display_flags_from_exflag() {
        let flags = DisplayFlags::from_exflag(0x01); // EXFLAG_LIST
        assert!(flags.list);
        assert!(!flags.number);

        let flags = DisplayFlags::from_exflag(0x02); // EXFLAG_NR
        assert!(!flags.list);
        assert!(flags.number);

        let flags = DisplayFlags::from_exflag(0x03); // EXFLAG_LIST | EXFLAG_NR
        assert!(flags.list);
        assert!(flags.number);
    }

    #[test]
    fn test_display_flags_display_mode() {
        let flags = DisplayFlags::new();
        assert_eq!(flags.display_mode(), DisplayMode::Print);

        let flags = DisplayFlags {
            list: true,
            ..DisplayFlags::new()
        };
        assert_eq!(flags.display_mode(), DisplayMode::List);

        let flags = DisplayFlags {
            number: true,
            ..DisplayFlags::new()
        };
        assert_eq!(flags.display_mode(), DisplayMode::Number);

        // List takes precedence over number
        let flags = DisplayFlags {
            list: true,
            number: true,
            print: false,
        };
        assert_eq!(flags.display_mode(), DisplayMode::List);
    }

    #[test]
    fn test_display_options() {
        let range = LineRange::new(1, 10);

        let opts = DisplayOptions::print(range);
        assert_eq!(opts.mode, DisplayMode::Print);

        let opts = DisplayOptions::number(range);
        assert_eq!(opts.mode, DisplayMode::Number);
        assert!(opts.flags.number);

        let opts = DisplayOptions::list(range);
        assert_eq!(opts.mode, DisplayMode::List);
        assert!(opts.flags.list);
    }

    #[test]
    fn test_list_chars() {
        let chars = ListChars::new();
        assert_eq!(chars.eol, '$');
        assert_eq!(chars.tab, '>');
        assert_eq!(chars.trail, '-');
        assert_eq!(chars.nbsp, '+');
    }

    #[test]
    fn test_line_number_format() {
        let fmt = LineNumberFormat::default();
        assert_eq!(fmt.min_width, 7);
        assert!(fmt.right_align);

        let fmt = LineNumberFormat::with_width(4);
        assert_eq!(fmt.min_width, 4);
    }

    #[test]
    fn test_line_number_width() {
        assert_eq!(LineNumberFormat::width_for_max(0), 1);
        assert_eq!(LineNumberFormat::width_for_max(9), 1);
        assert_eq!(LineNumberFormat::width_for_max(10), 2);
        assert_eq!(LineNumberFormat::width_for_max(99), 2);
        assert_eq!(LineNumberFormat::width_for_max(100), 3);
        assert_eq!(LineNumberFormat::width_for_max(999), 3);
        assert_eq!(LineNumberFormat::width_for_max(1000), 4);
    }

    #[test]
    fn test_line_number_result() {
        let result = LineNumberResult::new(42);
        assert_eq!(result.lnum, 42);
    }

    #[test]
    fn test_rs_display_mode_from_flags() {
        assert_eq!(rs_display_mode_from_flags(0), 0); // Print
        assert_eq!(rs_display_mode_from_flags(0x01), 2); // List
        assert_eq!(rs_display_mode_from_flags(0x02), 1); // Number
    }

    #[test]
    fn test_rs_line_number_width() {
        assert_eq!(rs_line_number_width(9), 1);
        assert_eq!(rs_line_number_width(100), 3);
        assert_eq!(rs_line_number_width(10000), 5);
    }
}
