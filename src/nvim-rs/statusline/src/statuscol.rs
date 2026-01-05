//! Status column rendering for statusline
//!
//! This module provides status column rendering utilities for line numbers,
//! fold columns, and sign columns in the statuscolumn.

use std::ffi::c_int;
use std::io::Write;

/// Line number display mode for status column.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineNumberMode {
    /// No line numbers
    None = 0,
    /// Absolute line numbers
    Absolute = 1,
    /// Relative line numbers
    Relative = 2,
    /// Hybrid (relative for non-cursor, absolute for cursor)
    Hybrid = 3,
}

impl LineNumberMode {
    /// Determine mode from number/relativenumber options.
    pub const fn from_options(number: bool, relativenumber: bool) -> Self {
        match (number, relativenumber) {
            (false, false) => Self::None,
            (true, false) => Self::Absolute,
            (false, true) => Self::Relative,
            (true, true) => Self::Hybrid,
        }
    }
}

/// Status column context for rendering.
#[derive(Debug, Clone)]
pub struct StatusColContext {
    /// Current line number (1-based)
    pub lnum: c_int,
    /// Relative line number (0 for cursor line)
    pub relnum: c_int,
    /// Total lines in buffer
    pub line_count: c_int,
    /// Width of the status column
    pub width: c_int,
    /// Fold level at this line
    pub fold_level: c_int,
    /// Whether this line is folded
    pub is_folded: bool,
    /// Whether this is a virtual/wrapped line
    pub is_virtual: bool,
    /// Sign column width
    pub sign_width: c_int,
    /// Fold column width
    pub fold_width: c_int,
}

impl Default for StatusColContext {
    fn default() -> Self {
        Self {
            lnum: 1,
            relnum: 0,
            line_count: 1,
            width: 0,
            fold_level: 0,
            is_folded: false,
            is_virtual: false,
            sign_width: 0,
            fold_width: 0,
        }
    }
}

impl StatusColContext {
    /// Create a new status column context.
    pub const fn new(lnum: c_int, relnum: c_int, line_count: c_int) -> Self {
        Self {
            lnum,
            relnum,
            line_count,
            width: 0,
            fold_level: 0,
            is_folded: false,
            is_virtual: false,
            sign_width: 0,
            fold_width: 0,
        }
    }

    /// Check if this is the cursor line.
    pub const fn is_cursor_line(&self) -> bool {
        self.relnum == 0
    }
}

/// Format a line number according to the display mode.
///
/// Returns the formatted number string and its width.
#[allow(clippy::cast_possible_wrap)]
pub fn format_line_number(ctx: &StatusColContext, mode: LineNumberMode) -> (String, c_int) {
    match mode {
        LineNumberMode::None => (String::new(), 0),
        LineNumberMode::Absolute => {
            let s = format!("{}", ctx.lnum);
            #[allow(clippy::cast_possible_truncation)]
            let width = s.len() as c_int;
            (s, width)
        }
        LineNumberMode::Relative => {
            let s = format!("{}", ctx.relnum);
            #[allow(clippy::cast_possible_truncation)]
            let width = s.len() as c_int;
            (s, width)
        }
        LineNumberMode::Hybrid => {
            // Show absolute for cursor line, relative for others
            let num = if ctx.is_cursor_line() {
                ctx.lnum
            } else {
                ctx.relnum
            };
            let s = format!("{num}");
            #[allow(clippy::cast_possible_truncation)]
            let width = s.len() as c_int;
            (s, width)
        }
    }
}

/// Format a line number with padding.
///
/// Returns the formatted string with appropriate padding to fit `width` characters.
#[allow(clippy::cast_sign_loss)]
pub fn format_line_number_padded(
    ctx: &StatusColContext,
    mode: LineNumberMode,
    width: c_int,
    right_align: bool,
) -> String {
    if mode == LineNumberMode::None {
        return " ".repeat(width.max(0) as usize);
    }

    let (num_str, _) = format_line_number(ctx, mode);

    if width <= 0 {
        return num_str;
    }

    #[allow(clippy::cast_sign_loss)]
    let width_usize = width as usize;
    if num_str.len() >= width_usize {
        // Truncate if too long (shouldn't happen normally)
        return num_str[..width_usize].to_string();
    }

    let padding = width_usize - num_str.len();
    if right_align {
        format!("{}{}", " ".repeat(padding), num_str)
    } else {
        format!("{}{}", num_str, " ".repeat(padding))
    }
}

/// Calculate the required width for line numbers.
///
/// This calculates how many digits are needed to display line numbers
/// based on the total number of lines in the buffer.
pub fn calc_number_width(line_count: c_int) -> c_int {
    if line_count <= 0 {
        return 1;
    }

    // Calculate digits needed
    let mut digits = 1;
    let mut n = line_count;
    while n >= 10 {
        n /= 10;
        digits += 1;
    }

    // Minimum width of 2 (matching Vim default)
    digits.max(2)
}

/// Fold column characters.
pub mod fold_chars {
    /// Fold closed marker
    pub const CLOSED: char = '+';
    /// Fold open marker
    pub const OPEN: char = '-';
    /// Fold continuation marker
    pub const CONT: char = '│';
    /// Fold separator marker
    pub const SEP: char = '·';
    /// Empty fold column
    pub const EMPTY: char = ' ';
}

/// Render fold column segment.
///
/// Returns the string representation of the fold column.
pub fn render_fold_column(
    fold_level: c_int,
    is_folded: bool,
    max_level: c_int,
    width: c_int,
) -> String {
    if width <= 0 {
        return String::new();
    }

    #[allow(clippy::cast_sign_loss)]
    let width_usize = width as usize;
    let mut result = String::with_capacity(width_usize * 3); // UTF-8 chars can be up to 3 bytes

    let display_level = fold_level.min(max_level).max(0);

    for i in 0..width {
        let ch = if i < display_level {
            if i == display_level - 1 {
                // Last level marker
                if is_folded {
                    fold_chars::CLOSED
                } else {
                    fold_chars::OPEN
                }
            } else {
                // Continuation marker
                fold_chars::CONT
            }
        } else {
            fold_chars::EMPTY
        };
        result.push(ch);
    }

    result
}

/// Sign placeholder for when no sign is present.
pub const SIGN_EMPTY: &str = "  "; // Two spaces

/// Render sign column segment.
///
/// If `sign_text` is None or empty, renders placeholder spaces.
pub fn render_sign_column(sign_text: Option<&str>, width: c_int) -> String {
    #[allow(clippy::cast_sign_loss)]
    let width_usize = (width * 2) as usize; // Signs are 2 chars wide

    match sign_text {
        Some(text) if !text.is_empty() => {
            // Pad or truncate to exact width
            let text_chars: Vec<char> = text.chars().collect();
            if text_chars.len() >= width_usize {
                text_chars[..width_usize].iter().collect()
            } else {
                let mut result: String = text_chars.iter().collect();
                let padding = width_usize - text_chars.len();
                for _ in 0..padding {
                    result.push(' ');
                }
                result
            }
        }
        _ => " ".repeat(width_usize),
    }
}

/// Render a complete status column line.
///
/// Combines line number, fold column, and sign column into a single string.
pub fn render_statuscol_line(
    buf: &mut [u8],
    ctx: &StatusColContext,
    mode: LineNumberMode,
    num_width: c_int,
    fold_width: c_int,
    sign_width: c_int,
    sign_text: Option<&str>,
) -> c_int {
    if buf.is_empty() {
        return 0;
    }

    let mut cursor = std::io::Cursor::new(buf);

    // Render fold column first (if present)
    if fold_width > 0 {
        let fold_str = render_fold_column(ctx.fold_level, ctx.is_folded, 9, fold_width);
        let _ = write!(cursor, "{fold_str}");
    }

    // Render sign column (if present)
    if sign_width > 0 {
        let sign_str = render_sign_column(sign_text, sign_width);
        let _ = write!(cursor, "{sign_str}");
    }

    // Render line number (if present)
    if mode != LineNumberMode::None && num_width > 0 {
        let num_str = format_line_number_padded(ctx, mode, num_width, true);
        let _ = write!(cursor, "{num_str} "); // Space after number
    }

    #[allow(clippy::cast_possible_truncation)]
    (cursor.position() as c_int)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_number_mode_from_options() {
        assert_eq!(
            LineNumberMode::from_options(false, false),
            LineNumberMode::None
        );
        assert_eq!(
            LineNumberMode::from_options(true, false),
            LineNumberMode::Absolute
        );
        assert_eq!(
            LineNumberMode::from_options(false, true),
            LineNumberMode::Relative
        );
        assert_eq!(
            LineNumberMode::from_options(true, true),
            LineNumberMode::Hybrid
        );
    }

    #[test]
    fn test_format_line_number_absolute() {
        let ctx = StatusColContext::new(42, 5, 100);
        let (s, w) = format_line_number(&ctx, LineNumberMode::Absolute);
        assert_eq!(s, "42");
        assert_eq!(w, 2);
    }

    #[test]
    fn test_format_line_number_relative() {
        let ctx = StatusColContext::new(42, 5, 100);
        let (s, w) = format_line_number(&ctx, LineNumberMode::Relative);
        assert_eq!(s, "5");
        assert_eq!(w, 1);
    }

    #[test]
    fn test_format_line_number_hybrid_cursor() {
        let ctx = StatusColContext::new(42, 0, 100);
        let (s, _) = format_line_number(&ctx, LineNumberMode::Hybrid);
        assert_eq!(s, "42"); // Absolute on cursor line
    }

    #[test]
    fn test_format_line_number_hybrid_other() {
        let ctx = StatusColContext::new(42, 5, 100);
        let (s, _) = format_line_number(&ctx, LineNumberMode::Hybrid);
        assert_eq!(s, "5"); // Relative on other lines
    }

    #[test]
    fn test_format_line_number_padded_right() {
        let ctx = StatusColContext::new(42, 0, 1000);
        let s = format_line_number_padded(&ctx, LineNumberMode::Absolute, 4, true);
        assert_eq!(s, "  42");
    }

    #[test]
    fn test_format_line_number_padded_left() {
        let ctx = StatusColContext::new(42, 0, 1000);
        let s = format_line_number_padded(&ctx, LineNumberMode::Absolute, 4, false);
        assert_eq!(s, "42  ");
    }

    #[test]
    fn test_calc_number_width() {
        assert_eq!(calc_number_width(1), 2); // Minimum 2
        assert_eq!(calc_number_width(9), 2); // Still 2 for single digit
        assert_eq!(calc_number_width(99), 2); // 2 digits
        assert_eq!(calc_number_width(100), 3); // 3 digits
        assert_eq!(calc_number_width(999), 3);
        assert_eq!(calc_number_width(1000), 4);
        assert_eq!(calc_number_width(10000), 5);
    }

    #[test]
    fn test_render_fold_column_empty() {
        let s = render_fold_column(0, false, 9, 3);
        assert_eq!(s, "   ");
    }

    #[test]
    fn test_render_fold_column_level_1_open() {
        let s = render_fold_column(1, false, 9, 3);
        assert_eq!(s, "-  ");
    }

    #[test]
    fn test_render_fold_column_level_1_closed() {
        let s = render_fold_column(1, true, 9, 3);
        assert_eq!(s, "+  ");
    }

    #[test]
    fn test_render_fold_column_level_3() {
        let s = render_fold_column(3, false, 9, 5);
        assert_eq!(s, "││-  ");
    }

    #[test]
    fn test_render_sign_column_empty() {
        let s = render_sign_column(None, 1);
        assert_eq!(s, "  ");
    }

    #[test]
    fn test_render_sign_column_with_sign() {
        let s = render_sign_column(Some(">>"), 1);
        assert_eq!(s, ">>");
    }

    #[test]
    fn test_render_sign_column_padding() {
        let s = render_sign_column(Some("X"), 1);
        assert_eq!(s, "X ");
    }

    #[test]
    fn test_status_col_context_is_cursor_line() {
        let cursor_ctx = StatusColContext::new(10, 0, 100);
        assert!(cursor_ctx.is_cursor_line());

        let other_ctx = StatusColContext::new(15, 5, 100);
        assert!(!other_ctx.is_cursor_line());
    }

    #[test]
    fn test_render_statuscol_line() {
        let ctx = StatusColContext {
            lnum: 42,
            relnum: 0,
            line_count: 100,
            fold_level: 1,
            is_folded: false,
            ..Default::default()
        };

        let mut buf = [0u8; 64];
        let len = render_statuscol_line(
            &mut buf,
            &ctx,
            LineNumberMode::Absolute,
            3,
            2,
            1,
            Some(">>"),
        );

        #[allow(clippy::cast_sign_loss)]
        let result = std::str::from_utf8(&buf[..len as usize]).unwrap();
        // Should contain: fold column (2), sign (2), line number (3) + space
        assert!(result.contains("42"));
        assert!(result.contains(">>"));
    }
}
