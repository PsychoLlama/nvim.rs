//! Formatting and alignment commands.
//!
//! This module provides types and utilities for text formatting Ex commands:
//! - `:retab` - Convert between tabs and spaces
//! - `:left` - Left-align text
//! - `:center` - Center text
//! - `:right` - Right-align text
//!
//! ## Implementation Notes
//!
//! These commands modify whitespace and indentation in the buffer.
//! The actual text modification is performed by Neovim's buffer functions.

use std::ffi::{c_char, c_int};

use crate::range::{LineNr, LineRange};
use crate::ExArgHandle;

// =============================================================================
// Alignment Type
// =============================================================================

/// Text alignment type for `:left`, `:center`, and `:right` commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum Alignment {
    /// Left alignment (`:left`)
    #[default]
    Left = 0,
    /// Center alignment (`:center`)
    Center = 1,
    /// Right alignment (`:right`)
    Right = 2,
}

impl Alignment {
    /// Convert from C integer.
    #[inline]
    #[must_use]
    pub fn from_c(value: c_int) -> Self {
        match value {
            0 => Alignment::Left,
            1 => Alignment::Center,
            2 => Alignment::Right,
            _ => Alignment::Left,
        }
    }

    /// Convert to C integer.
    #[inline]
    #[must_use]
    pub fn to_c(self) -> c_int {
        self as c_int
    }

    /// Swap left/right for right-to-left mode.
    #[must_use]
    pub fn swap_for_rtl(self) -> Self {
        match self {
            Alignment::Left => Alignment::Right,
            Alignment::Right => Alignment::Left,
            Alignment::Center => Alignment::Center,
        }
    }
}

// =============================================================================
// Alignment Options
// =============================================================================

/// Options for alignment commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlignOptions {
    /// Line range to align.
    pub range: LineRange,
    /// Alignment type.
    pub alignment: Alignment,
    /// Width for centering/right-aligning (0 to use textwidth).
    pub width: i32,
    /// Whether right-to-left mode is active.
    pub rtl: bool,
}

impl AlignOptions {
    /// Create left alignment options.
    #[must_use]
    pub const fn left(range: LineRange, indent: i32) -> Self {
        Self {
            range,
            alignment: Alignment::Left,
            width: indent,
            rtl: false,
        }
    }

    /// Create center alignment options.
    #[must_use]
    pub const fn center(range: LineRange, width: i32) -> Self {
        Self {
            range,
            alignment: Alignment::Center,
            width,
            rtl: false,
        }
    }

    /// Create right alignment options.
    #[must_use]
    pub const fn right(range: LineRange, width: i32) -> Self {
        Self {
            range,
            alignment: Alignment::Right,
            width,
            rtl: false,
        }
    }

    /// Get the effective alignment, accounting for RTL mode.
    #[must_use]
    pub fn effective_alignment(&self) -> Alignment {
        if self.rtl {
            self.alignment.swap_for_rtl()
        } else {
            self.alignment
        }
    }
}

/// Calculate the new indent for a line based on alignment.
///
/// # Arguments
/// * `alignment` - The alignment type
/// * `width` - The total width for center/right alignment
/// * `line_len` - The length of the line content (excluding leading whitespace)
///
/// # Returns
/// The new indent value (number of spaces)
#[must_use]
pub fn calculate_alignment_indent(alignment: Alignment, width: i32, line_len: i32) -> i32 {
    if line_len <= 0 {
        return 0;
    }

    match alignment {
        Alignment::Left => 0, // For :left, indent is set separately
        Alignment::Center => {
            let indent = (width - line_len) / 2;
            indent.max(0)
        }
        Alignment::Right => {
            let indent = width - line_len;
            indent.max(0)
        }
    }
}

// =============================================================================
// Retab Options
// =============================================================================

/// Options for the `:retab` command.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RetabOptions {
    /// Line range to retab.
    pub range: LineRange,
    /// New tabstop value (None to use current 'tabstop').
    pub tabstop: Option<i32>,
    /// Variable tabstops (empty for uniform tabs).
    pub variable_tabstops: Vec<i32>,
    /// Force retabbing (convert multiple spaces even without tabs).
    pub force: bool,
    /// Only retab leading whitespace (indent).
    pub indent_only: bool,
    /// Convert tabs to spaces ('expandtab' mode).
    pub expand_tabs: bool,
}

impl RetabOptions {
    /// Create new retab options with default settings.
    #[must_use]
    pub fn new(range: LineRange) -> Self {
        Self {
            range,
            ..Default::default()
        }
    }

    /// Set the tabstop value.
    #[must_use]
    pub const fn with_tabstop(mut self, tabstop: i32) -> Self {
        self.tabstop = Some(tabstop);
        self
    }

    /// Set force mode.
    #[must_use]
    pub const fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// Set indent-only mode.
    #[must_use]
    pub const fn with_indent_only(mut self, indent_only: bool) -> Self {
        self.indent_only = indent_only;
        self
    }

    /// Set expand tabs mode.
    #[must_use]
    pub const fn with_expand_tabs(mut self, expand: bool) -> Self {
        self.expand_tabs = expand;
        self
    }
}

/// Parse the `-indentonly` flag from retab arguments.
///
/// Returns (has_flag, remaining_args).
#[must_use]
pub fn parse_retab_indent_only(args: &str) -> (bool, &str) {
    let trimmed = args.trim_start();
    if let Some(rest) = trimmed.strip_prefix("-indentonly") {
        if rest.is_empty() || rest.starts_with(char::is_whitespace) {
            return (true, rest.trim_start());
        }
    }
    (false, args)
}

// =============================================================================
// Retab Statistics
// =============================================================================

/// Statistics from a retab operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RetabStats {
    /// First line that was changed.
    pub first_changed: LineNr,
    /// Last line that was changed.
    pub last_changed: LineNr,
    /// Number of lines changed.
    pub lines_changed: i32,
}

impl RetabStats {
    /// Create empty stats (no changes).
    #[must_use]
    pub const fn none() -> Self {
        Self {
            first_changed: 0,
            last_changed: 0,
            lines_changed: 0,
        }
    }

    /// Check if any lines were changed.
    #[must_use]
    pub const fn has_changes(&self) -> bool {
        self.lines_changed > 0
    }
}

// =============================================================================
// Whitespace Analysis
// =============================================================================

/// Result of analyzing whitespace in a string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WhitespaceInfo {
    /// Number of leading spaces.
    pub leading_spaces: i32,
    /// Number of leading tabs.
    pub leading_tabs: i32,
    /// Total leading whitespace characters.
    pub leading_count: i32,
    /// Virtual column after leading whitespace.
    pub indent_vcol: i32,
}

impl WhitespaceInfo {
    /// Analyze leading whitespace in a string.
    ///
    /// # Arguments
    /// * `line` - The line to analyze
    /// * `tabstop` - The tabstop value for virtual column calculation
    #[must_use]
    pub fn analyze(line: &str, tabstop: i32) -> Self {
        let tabstop = tabstop.max(1);
        let mut spaces = 0;
        let mut tabs = 0;
        let mut vcol: i32 = 0;

        for c in line.chars() {
            match c {
                ' ' => {
                    spaces += 1;
                    vcol += 1;
                }
                '\t' => {
                    tabs += 1;
                    // Tab advances to next tabstop
                    vcol = ((vcol / tabstop) + 1) * tabstop;
                }
                _ => break,
            }
        }

        Self {
            leading_spaces: spaces,
            leading_tabs: tabs,
            leading_count: spaces + tabs,
            indent_vcol: vcol,
        }
    }

    /// Check if the line has mixed tabs and spaces.
    #[must_use]
    pub const fn has_mixed_whitespace(&self) -> bool {
        self.leading_spaces > 0 && self.leading_tabs > 0
    }
}

// =============================================================================
// Tab/Space Conversion
// =============================================================================

/// Calculate number of tabs and spaces to represent a column range.
///
/// Given a virtual column range [from_vcol, to_vcol), calculate the optimal
/// number of tabs and spaces to represent that range.
///
/// A tab character advances the column to the next tabstop boundary.
/// This function returns the optimal mix of tabs and trailing spaces.
///
/// # Arguments
/// * `from_vcol` - Starting virtual column
/// * `to_vcol` - Ending virtual column
/// * `tabstop` - Tabstop value
///
/// # Returns
/// (num_tabs, num_spaces)
#[must_use]
pub fn tabs_and_spaces_for_range(from_vcol: i32, to_vcol: i32, tabstop: i32) -> (i32, i32) {
    if from_vcol >= to_vcol || tabstop <= 0 {
        return (0, 0);
    }

    let tabstop = tabstop.max(1);
    let width = to_vcol - from_vcol;

    // Calculate the first tabstop boundary at or after from_vcol
    let next_tabstop = ((from_vcol / tabstop) + 1) * tabstop;

    if next_tabstop > to_vcol {
        // No room for any tabs, use all spaces
        return (0, width);
    }

    // A tab advances to next_tabstop
    // Then we can use more tabs from there
    let after_first_tab = next_tabstop;
    let remaining_width = to_vcol - after_first_tab;
    let additional_tabs = remaining_width / tabstop;
    let trailing_spaces = remaining_width % tabstop;

    (1 + additional_tabs, trailing_spaces)
}

/// Simpler: Calculate tabs and spaces using Vim's tabstop logic.
#[must_use]
pub fn tabstop_padding(col: i32, tabstop: i32) -> i32 {
    let tabstop = tabstop.max(1);
    tabstop - (col % tabstop)
}

// =============================================================================
// Command Index Constants (verified with _Static_assert in C)
// =============================================================================

const CMD_LEFT: c_int = 229;
const CMD_CENTER: c_int = 63;
const CMD_RIGHT: c_int = 372;

/// BL_WHITE | BL_FIX for beginline()
const BL_WHITE_FIX: c_int = 1 | 4; // BL_WHITE=1, BL_FIX=4

/// TAB character (ASCII 9)
const TAB: c_int = 9;

// =============================================================================
// Line Length Helper
// =============================================================================

/// Get the length of the current line, excluding trailing white space.
///
/// If `check_tab` is true, also checks for embedded TAB characters
/// in the non-whitespace portion and returns the result.
///
/// Returns `(line_length, has_tab)`.
///
/// # Safety
///
/// Calls C functions that operate on the current buffer line.
unsafe fn linelen(check_tab: bool) -> (c_int, bool) {
    let line = crate::get_cursor_line_ptr();
    if line.is_null() || *line == 0 {
        return (0, false);
    }

    // Find the first non-blank character
    let first = crate::skipwhite(line);

    // Find the character after the last non-blank character
    let first_len = libc::strlen(first as *const _);
    let mut last = first.add(first_len) as *mut c_char;
    while last > first as *mut c_char
        && (*last.offset(-1) == b' ' as c_char || *last.offset(-1) == b'\t' as c_char)
    {
        last = last.offset(-1);
    }

    // Temporarily NUL-terminate at the last non-blank character
    let save = *last;
    *last = 0;
    let len = crate::nvim_linetabsize_str(line);

    // Check for embedded TAB
    let has_tab = if check_tab {
        !crate::vim_strchr(first, TAB).is_null()
    } else {
        false
    };

    *last = save;

    (len, has_tab)
}

// =============================================================================
// ex_align Implementation
// =============================================================================

/// `:left`, `:center`, `:right` — align text.
///
/// # Safety
///
/// `eap` must be a valid pointer to an `exarg_T`.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_align(eap: *mut ExArgHandle) {
    let mut cmdidx = crate::nvim_exarg_get_cmdidx(eap);

    // Switch left and right aligning for right-to-left mode
    if crate::nvim_curwin_get_w_p_rl() != 0 {
        if cmdidx == CMD_RIGHT {
            cmdidx = CMD_LEFT;
        } else if cmdidx == CMD_LEFT {
            cmdidx = CMD_RIGHT;
        }
    }

    let arg = crate::nvim_exarg_get_arg(eap);
    let width_arg = libc::atoi(arg);
    let line1 = crate::nvim_exarg_get_line1(eap);
    let line2 = crate::nvim_exarg_get_line2(eap);

    // Save cursor position
    let save_lnum = crate::nvim_curwin_get_cursor_lnum();

    let mut indent = 0;
    let mut width = width_arg;

    if cmdidx == CMD_LEFT {
        // width is used for new indent
        if width >= 0 {
            indent = width;
        }
    } else {
        // if 'textwidth' set, use it
        // else if 'wrapmargin' set, use it
        // if invalid value, use 80
        if width <= 0 {
            width = crate::nvim_curbuf_get_b_p_tw();
        }
        if width == 0 && crate::nvim_curbuf_get_b_p_wm() > 0 {
            width = crate::nvim_curwin_get_view_width() - crate::nvim_curbuf_get_b_p_wm();
        }
        if width <= 0 {
            width = 80;
        }
    }

    // Save undo
    if crate::u_save(line1 - 1, line2 + 1) == 0 {
        // FAIL == 0
        return;
    }

    let mut lnum = line1;
    while lnum <= line2 {
        crate::nvim_curwin_set_cursor_lnum(lnum);

        let mut new_indent;
        if cmdidx == CMD_LEFT {
            new_indent = indent;
        } else {
            let check_tab = cmdidx == CMD_RIGHT;
            let (line_total_len, has_tab) = linelen(check_tab);
            let len = line_total_len - crate::get_indent();

            if len <= 0 {
                // skip blank lines
                lnum += 1;
                continue;
            }

            if cmdidx == CMD_CENTER {
                new_indent = (width - len) / 2;
            } else {
                // right align
                new_indent = width - len;

                // Make sure that embedded TABs don't make the text go too far
                // to the right.
                if has_tab {
                    while new_indent > 0 {
                        crate::set_indent(new_indent, 0);
                        if linelen(false).0 <= width {
                            // Now try to move the line as much as possible to
                            // the right. Stop when it moves too far.
                            loop {
                                new_indent += 1;
                                crate::set_indent(new_indent, 0);
                                if linelen(false).0 > width {
                                    break;
                                }
                            }
                            new_indent -= 1;
                            break;
                        }
                        new_indent -= 1;
                    }
                }
            }
        }

        let new_indent = if new_indent < 0 { 0 } else { new_indent };
        crate::set_indent(new_indent, 0);

        lnum += 1;
    }

    let buf = crate::nvim_get_curbuf();
    crate::changed_lines(buf, line1, 0, line2 + 1, 0, 1);
    crate::nvim_curwin_set_cursor_lnum(save_lnum);
    crate::beginline(BL_WHITE_FIX);
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Convert alignment type from C integer.
pub extern "C" fn rs_alignment_from_c(value: c_int) -> c_int {
    Alignment::from_c(value).to_c()
}

/// Swap alignment for RTL mode.
pub extern "C" fn rs_alignment_swap_rtl(alignment: c_int) -> c_int {
    Alignment::from_c(alignment).swap_for_rtl().to_c()
}

/// Calculate alignment indent.
pub extern "C" fn rs_calculate_alignment_indent(
    alignment: c_int,
    width: c_int,
    line_len: c_int,
) -> c_int {
    calculate_alignment_indent(Alignment::from_c(alignment), width, line_len)
}

/// Calculate tab padding (spaces to next tabstop).
pub extern "C" fn rs_tabstop_padding(col: c_int, tabstop: c_int) -> c_int {
    tabstop_padding(col, tabstop)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alignment_roundtrip() {
        for i in 0..=2 {
            let align = Alignment::from_c(i);
            assert_eq!(align.to_c(), i);
        }
    }

    #[test]
    fn test_alignment_swap_rtl() {
        assert_eq!(Alignment::Left.swap_for_rtl(), Alignment::Right);
        assert_eq!(Alignment::Right.swap_for_rtl(), Alignment::Left);
        assert_eq!(Alignment::Center.swap_for_rtl(), Alignment::Center);
    }

    #[test]
    fn test_align_options_effective() {
        let range = LineRange::new(1, 10);
        let opts = AlignOptions {
            range,
            alignment: Alignment::Left,
            width: 80,
            rtl: true,
        };
        assert_eq!(opts.effective_alignment(), Alignment::Right);
    }

    #[test]
    fn test_calculate_alignment_indent() {
        // Center: (80 - 20) / 2 = 30
        assert_eq!(calculate_alignment_indent(Alignment::Center, 80, 20), 30);

        // Right: 80 - 20 = 60
        assert_eq!(calculate_alignment_indent(Alignment::Right, 80, 20), 60);

        // Empty line
        assert_eq!(calculate_alignment_indent(Alignment::Center, 80, 0), 0);

        // Line longer than width
        assert_eq!(calculate_alignment_indent(Alignment::Right, 80, 100), 0);
    }

    #[test]
    fn test_retab_options() {
        let range = LineRange::new(1, 100);
        let opts = RetabOptions::new(range)
            .with_tabstop(4)
            .with_force(true)
            .with_indent_only(true)
            .with_expand_tabs(true);

        assert_eq!(opts.tabstop, Some(4));
        assert!(opts.force);
        assert!(opts.indent_only);
        assert!(opts.expand_tabs);
    }

    #[test]
    fn test_parse_retab_indent_only() {
        let (flag, rest) = parse_retab_indent_only("-indentonly 4");
        assert!(flag);
        assert_eq!(rest, "4");

        let (flag, rest) = parse_retab_indent_only("-indentonly");
        assert!(flag);
        assert_eq!(rest, "");

        let (flag, rest) = parse_retab_indent_only("4");
        assert!(!flag);
        assert_eq!(rest, "4");

        // -indentonlyfoo is NOT the flag
        let (flag, rest) = parse_retab_indent_only("-indentonlyfoo");
        assert!(!flag);
        assert_eq!(rest, "-indentonlyfoo");
    }

    #[test]
    fn test_retab_stats() {
        let stats = RetabStats::none();
        assert!(!stats.has_changes());

        let stats = RetabStats {
            first_changed: 5,
            last_changed: 10,
            lines_changed: 3,
        };
        assert!(stats.has_changes());
    }

    #[test]
    fn test_whitespace_info() {
        let info = WhitespaceInfo::analyze("    hello", 8);
        assert_eq!(info.leading_spaces, 4);
        assert_eq!(info.leading_tabs, 0);
        assert_eq!(info.indent_vcol, 4);

        let info = WhitespaceInfo::analyze("\t\thello", 8);
        assert_eq!(info.leading_spaces, 0);
        assert_eq!(info.leading_tabs, 2);
        assert_eq!(info.indent_vcol, 16);

        let info = WhitespaceInfo::analyze("  \t hello", 8);
        assert!(info.has_mixed_whitespace());
        assert_eq!(info.leading_spaces, 3); // 2 before tab, 1 after
        assert_eq!(info.leading_tabs, 1);
    }

    #[test]
    fn test_tabstop_padding() {
        // At column 0, padding to next tabstop is tabstop itself
        assert_eq!(tabstop_padding(0, 8), 8);

        // At column 3, padding to next tabstop (8) is 5
        assert_eq!(tabstop_padding(3, 8), 5);

        // At column 8, padding is 8 (full tab)
        assert_eq!(tabstop_padding(8, 8), 8);

        // At column 10, padding to 16 is 6
        assert_eq!(tabstop_padding(10, 8), 6);

        // Tabstop of 4
        assert_eq!(tabstop_padding(0, 4), 4);
        assert_eq!(tabstop_padding(1, 4), 3);
        assert_eq!(tabstop_padding(4, 4), 4);
    }

    #[test]
    fn test_cmd_constants() {
        // These must match the values in ex_cmds_enum.generated.h
        // (verified with _Static_assert in C)
        assert_eq!(CMD_LEFT, 229);
        assert_eq!(CMD_CENTER, 63);
        assert_eq!(CMD_RIGHT, 372);
        assert_eq!(BL_WHITE_FIX, 5); // BL_WHITE(1) | BL_FIX(4)
        assert_eq!(TAB, 9);
    }

    #[test]
    fn test_rs_alignment_swap_rtl() {
        assert_eq!(rs_alignment_swap_rtl(0), 2); // Left -> Right
        assert_eq!(rs_alignment_swap_rtl(2), 0); // Right -> Left
        assert_eq!(rs_alignment_swap_rtl(1), 1); // Center -> Center
    }

    #[test]
    fn test_rs_calculate_alignment_indent() {
        assert_eq!(rs_calculate_alignment_indent(1, 80, 20), 30); // Center
        assert_eq!(rs_calculate_alignment_indent(2, 80, 20), 60); // Right
    }

    #[test]
    fn test_rs_tabstop_padding() {
        assert_eq!(rs_tabstop_padding(0, 8), 8);
        assert_eq!(rs_tabstop_padding(3, 8), 5);
    }
}
