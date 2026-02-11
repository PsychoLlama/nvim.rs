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

use std::ffi::{c_int, c_long, CStr};

use crate::range::{LineNr, LineRange};
use crate::ExArgHandle;

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
// ex_z types and pure logic
// =============================================================================

/// EXFLAG constants
const EXFLAG_LIST: c_int = 0x01;
const EXFLAG_NR: c_int = 0x02;

/// Kind of `:z` display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZKind {
    /// `-`: show lines above
    Minus,
    /// `=`: center on current line with separator
    Equals,
    /// `^`: show two pages above
    Caret,
    /// `.`: center on current line
    Dot,
    /// `+` or default: show lines below
    Plus,
}

impl ZKind {
    /// Parse from the kind character.
    fn from_char(c: u8) -> Self {
        match c {
            b'-' => ZKind::Minus,
            b'=' => ZKind::Equals,
            b'^' => ZKind::Caret,
            b'.' => ZKind::Dot,
            _ => ZKind::Plus,
        }
    }
}

/// Result of calculating the z-command range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZRange {
    pub start: LineNr,
    pub end: LineNr,
    pub curs: LineNr,
    pub show_separator: bool,
}

/// Calculate the display range for `:z`.
///
/// Pure function - no FFI calls. This is the core logic extracted for testing.
///
/// # Arguments
/// * `kind` - The z-command kind character
/// * `lnum` - The line number from the command
/// * `bigness` - The size of the display area
/// * `repeat_count` - For +/-, the number of consecutive kind chars (x - kind)
/// * `addr_count` - Number of addresses given in the command
/// * `line_count` - Total lines in the buffer
pub fn calculate_z_range(
    kind: ZKind,
    lnum: LineNr,
    bigness: LineNr,
    repeat_count: LineNr,
    addr_count: c_int,
    line_count: LineNr,
) -> ZRange {
    let (start, end, curs, show_separator) = match kind {
        ZKind::Minus => {
            let s = lnum - bigness * repeat_count + 1;
            let e = s + bigness - 1;
            (s, e, e, false)
        }
        ZKind::Equals => {
            let s = lnum - (bigness + 1) / 2 + 1;
            let e = lnum + (bigness + 1) / 2 - 1;
            (s, e, lnum, true)
        }
        ZKind::Caret => {
            let s = lnum - bigness * 2;
            let e = lnum - bigness;
            (s, e, lnum - bigness, false)
        }
        ZKind::Dot => {
            let s = lnum - (bigness + 1) / 2 + 1;
            let e = lnum + (bigness + 1) / 2 - 1;
            (s, e, e, false)
        }
        ZKind::Plus => {
            let mut s = lnum;
            if repeat_count > 1 {
                // Was explicitly '+', multiply by repeat count
                s += bigness * (repeat_count - 1) + 1;
            } else if addr_count == 0 {
                s += 1;
            }
            let e = s + bigness - 1;
            (s, e, e, false)
        }
    };

    // Clamp to buffer bounds
    let start = start.max(1);
    let end = end.min(line_count);
    let curs = curs.max(1).min(line_count);

    ZRange {
        start,
        end,
        curs,
        show_separator,
    }
}

extern "C" {
    fn atol(s: *const std::ffi::c_char) -> c_long;
}

// =============================================================================
// FFI Exports
// =============================================================================

/// `:z` command implementation.
///
/// # Safety
/// `eap` must be a valid pointer to an exarg_T.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_z(eap: *mut ExArgHandle) {
    use crate::{
        emsg, msg_putchar, nvim_curbuf_get_line_count, nvim_curwin_get_cursor_lnum,
        nvim_curwin_get_p_scr, nvim_curwin_get_view_height, nvim_curwin_set_cursor_col,
        nvim_curwin_set_cursor_lnum, nvim_exarg_get_addr_count, nvim_exarg_get_arg,
        nvim_exarg_get_flags, nvim_exarg_get_forceit, nvim_exarg_get_line2, nvim_get_Columns,
        nvim_get_Rows, nvim_is_one_window, nvim_set_ex_no_reprint, nvim_set_p_window, print_line,
    };

    let lnum = nvim_exarg_get_line2(eap);

    // Vi compatible: ":z!" uses display height, without a count uses 'scroll'
    let forceit = nvim_exarg_get_forceit(eap) != 0;
    let mut bigness: i64 = if forceit {
        i64::from(nvim_get_Rows()) - 1
    } else if nvim_is_one_window() != 0 {
        nvim_curwin_get_p_scr() * 2
    } else {
        i64::from(nvim_curwin_get_view_height()) - 3
    };
    if bigness < 1 {
        bigness = 1;
    }

    let arg = nvim_exarg_get_arg(eap);
    let arg_bytes = CStr::from_ptr(arg).to_bytes();

    // Parse kind character
    let mut pos = 0;
    let kind_byte =
        if !arg_bytes.is_empty() && matches!(arg_bytes[0], b'-' | b'+' | b'=' | b'^' | b'.') {
            let k = arg_bytes[0];
            pos = 1;
            k
        } else {
            0 // default ('+' behavior)
        };

    // Skip additional -/+ signs
    while pos < arg_bytes.len() && (arg_bytes[pos] == b'-' || arg_bytes[pos] == b'+') {
        pos += 1;
    }

    // Check for numeric argument
    if pos < arg_bytes.len() && arg_bytes[pos] != 0 {
        if !arg_bytes[pos].is_ascii_digit() {
            // E144: Non-numeric argument to :z
            emsg(c"E144: Non-numeric argument to :z".as_ptr());
            return;
        }
        // Use C's atol to parse the number (same as original code)
        // arg + pos points to the digit start
        bigness = atol(arg.add(pos)) as i64;

        // bigness could be < 0 if atol overflows
        let line_count_2x = 2 * i64::from(nvim_curbuf_get_line_count());
        if bigness > line_count_2x || bigness < 0 {
            bigness = line_count_2x;
        }

        nvim_set_p_window(bigness);
        if kind_byte == b'=' {
            bigness += 2;
        }
    }

    // Count repeat chars for '-' and '+' (the number of consecutive kind chars)
    let kind = ZKind::from_char(kind_byte);
    let repeat_count: LineNr = if kind_byte == b'-' || kind_byte == b'+' {
        // Count: kind char itself (pos=1 at start) + additional same chars
        // In C: for (x = kind + 1; *x == *kind; x++) {} then (x - kind)
        // We started with pos after kind. Continue counting same chars from pos=1.
        let mut count: LineNr = 1;
        let mut i = 1;
        while i < arg_bytes.len() && arg_bytes[i] == kind_byte {
            count += 1;
            i += 1;
        }
        count
    } else {
        1 // for '+' behavior (repeat_count > 1 means explicit '+' was typed)
    };

    // For the Plus default case, we need to distinguish explicit '+' from default.
    // In the C code: default case checks if (*kind == '+') for the multiplication.
    // repeat_count > 1 means we need multiplication, but for explicit '+' with count 1
    // it also applies. Let me re-check the C logic:
    //
    // In C default case:
    //   start = lnum;
    //   if (*kind == '+') { start += bigness * (x - kind - 1) + 1; }
    //   else if (addr_count == 0) { start++; }
    //
    // (x - kind) is the total offset from kind to x (including kind char itself).
    // For a single '+', x points just past it, so (x - kind) = 1, (x - kind - 1) = 0.
    // So start = lnum + 0 + 1 = lnum + 1.
    // For '++', (x - kind) = 2, (x-kind-1) = 1, start = lnum + bigness*1 + 1.
    //
    // For default (no kind char), we fall through to the Plus case with repeat_count=1.
    // The C code checks (*kind == '+') which is false for default, so addr_count check runs.

    let is_explicit_plus = kind_byte == b'+';
    let addr_count = nvim_exarg_get_addr_count(eap);

    // Recalculate for Plus case with proper semantics
    let bigness_nr = bigness as LineNr;
    let line_count = nvim_curbuf_get_line_count();

    let range = if kind == ZKind::Plus {
        // Handle Plus/default case directly since it has special repeat logic
        let mut start = lnum;
        if is_explicit_plus {
            start += bigness_nr * (repeat_count - 1) + 1;
        } else if addr_count == 0 {
            start += 1;
        }
        let end = start + bigness_nr - 1;
        let curs = end;

        ZRange {
            start: start.max(1),
            end: end.min(line_count),
            curs: curs.max(1).min(line_count),
            show_separator: false,
        }
    } else {
        calculate_z_range(kind, lnum, bigness_nr, repeat_count, addr_count, line_count)
    };

    // Display the lines
    let flags = nvim_exarg_get_flags(eap);
    let use_number = (flags & EXFLAG_NR) != 0;
    let use_list = (flags & EXFLAG_LIST) != 0;

    for i in range.start..=range.end {
        if range.show_separator && i == lnum {
            msg_putchar(b'\n' as c_int);
            let columns = nvim_get_Columns();
            for _j in 1..columns {
                msg_putchar(b'-' as c_int);
            }
        }

        print_line(
            i,
            c_int::from(use_number),
            c_int::from(use_list),
            c_int::from(i == range.start),
        );

        if range.show_separator && i == lnum {
            msg_putchar(b'\n' as c_int);
            let columns = nvim_get_Columns();
            for _j in 1..columns {
                msg_putchar(b'-' as c_int);
            }
        }
    }

    if nvim_curwin_get_cursor_lnum() != range.curs {
        nvim_curwin_set_cursor_lnum(range.curs);
        nvim_curwin_set_cursor_col(0);
    }
    nvim_set_ex_no_reprint(1);
}

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

    // =========================================================================
    // ex_z tests
    // =========================================================================

    #[test]
    fn test_z_kind_from_char() {
        assert_eq!(ZKind::from_char(b'-'), ZKind::Minus);
        assert_eq!(ZKind::from_char(b'+'), ZKind::Plus);
        assert_eq!(ZKind::from_char(b'='), ZKind::Equals);
        assert_eq!(ZKind::from_char(b'^'), ZKind::Caret);
        assert_eq!(ZKind::from_char(b'.'), ZKind::Dot);
        assert_eq!(ZKind::from_char(0), ZKind::Plus);
        assert_eq!(ZKind::from_char(b'x'), ZKind::Plus);
    }

    #[test]
    fn test_calculate_z_range_minus() {
        // :z- at line 50, bigness=20
        let r = calculate_z_range(ZKind::Minus, 50, 20, 1, 0, 100);
        assert_eq!(r.start, 31); // 50 - 20*1 + 1
        assert_eq!(r.end, 50); // 31 + 20 - 1
        assert_eq!(r.curs, 50);
        assert!(!r.show_separator);
    }

    #[test]
    fn test_calculate_z_range_minus_double() {
        // :z-- at line 50, bigness=20
        let r = calculate_z_range(ZKind::Minus, 50, 20, 2, 0, 100);
        assert_eq!(r.start, 11); // 50 - 20*2 + 1
        assert_eq!(r.end, 30); // 11 + 20 - 1
        assert_eq!(r.curs, 30);
    }

    #[test]
    fn test_calculate_z_range_equals() {
        // :z= at line 50, bigness=22
        let r = calculate_z_range(ZKind::Equals, 50, 22, 1, 0, 100);
        assert_eq!(r.start, 40); // 50 - (22+1)/2 + 1 = 50 - 11 + 1
        assert_eq!(r.end, 60); // 50 + (22+1)/2 - 1 = 50 + 11 - 1
        assert_eq!(r.curs, 50);
        assert!(r.show_separator);
    }

    #[test]
    fn test_calculate_z_range_caret() {
        // :z^ at line 50, bigness=20
        let r = calculate_z_range(ZKind::Caret, 50, 20, 1, 0, 100);
        assert_eq!(r.start, 10); // 50 - 20*2
        assert_eq!(r.end, 30); // 50 - 20
        assert_eq!(r.curs, 30); // 50 - 20
    }

    #[test]
    fn test_calculate_z_range_dot() {
        // :z. at line 50, bigness=20
        let r = calculate_z_range(ZKind::Dot, 50, 20, 1, 0, 100);
        assert_eq!(r.start, 41); // 50 - (20+1)/2 + 1 = 50 - 10 + 1
        assert_eq!(r.end, 59); // 50 + (20+1)/2 - 1 = 50 + 10 - 1
        assert_eq!(r.curs, 59);
        assert!(!r.show_separator);
    }

    #[test]
    fn test_calculate_z_range_clamping() {
        // Range extends before start of buffer
        let r = calculate_z_range(ZKind::Minus, 5, 20, 1, 0, 100);
        assert_eq!(r.start, 1); // clamped from -14
        assert_eq!(r.end, 5);

        // Range extends past end of buffer
        let r = calculate_z_range(ZKind::Dot, 95, 20, 1, 0, 100);
        assert_eq!(r.end, 100); // clamped from 105
    }

    #[test]
    fn test_exflag_constants() {
        assert_eq!(EXFLAG_LIST, 0x01);
        assert_eq!(EXFLAG_NR, 0x02);
    }
}
