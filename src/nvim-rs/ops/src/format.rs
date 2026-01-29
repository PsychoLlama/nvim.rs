//! Text formatting operators (gq, gw, =)
//!
//! This module implements calculation logic for text formatting operators
//! including paragraph formatting, indentation, and line wrapping.

use std::ffi::c_int;

use crate::types::MotionType;

// =============================================================================
// Format Type
// =============================================================================

/// Type of formatting operation.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormatType {
    /// Format text (gq)
    #[default]
    Format = 0,
    /// Format text, keep cursor (gw)
    FormatKeepCursor = 1,
    /// Indent/equalprg (=)
    Indent = 2,
    /// Internal reformat
    Internal = 3,
}

impl FormatType {
    /// Create from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::FormatKeepCursor,
            2 => Self::Indent,
            3 => Self::Internal,
            _ => Self::Format,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if cursor should be preserved.
    #[must_use]
    pub const fn keep_cursor(self) -> bool {
        matches!(self, Self::FormatKeepCursor)
    }

    /// Check if this is an indent operation.
    #[must_use]
    pub const fn is_indent(self) -> bool {
        matches!(self, Self::Indent)
    }
}

// =============================================================================
// Format Options
// =============================================================================

/// Options for formatting operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FormatOptions {
    /// Text width (textwidth option)
    pub textwidth: c_int,
    /// Whether to auto-format
    pub auto_format: bool,
    /// Whether to wrap text
    pub wrap: bool,
    /// Join paragraphs
    pub join_para: bool,
    /// Preserve trailing spaces
    pub preserve_trailing: bool,
}

impl FormatOptions {
    /// Create with default values.
    #[must_use]
    pub const fn new(textwidth: c_int) -> Self {
        Self {
            textwidth,
            auto_format: false,
            wrap: true,
            join_para: true,
            preserve_trailing: false,
        }
    }

    /// Get effective text width.
    ///
    /// Returns textwidth if set, otherwise uses window width estimate.
    #[must_use]
    pub const fn effective_width(&self, window_width: c_int) -> c_int {
        if self.textwidth > 0 {
            self.textwidth
        } else if window_width > 0 {
            window_width - 1
        } else {
            79 // Default fallback
        }
    }
}

// =============================================================================
// Format Calculations
// =============================================================================

/// Check if formatting should continue to next line.
///
/// # Arguments
/// * `cur_lnum` - Current line number
/// * `end_lnum` - End line of format range
/// * `cur_col` - Current column
/// * `textwidth` - Text width limit
///
/// # Returns
/// true if should continue formatting
#[must_use]
#[inline]
pub const fn should_continue_format(
    cur_lnum: c_int,
    end_lnum: c_int,
    _cur_col: c_int,
    _textwidth: c_int,
) -> bool {
    cur_lnum < end_lnum
}

/// Calculate where a line should be broken for formatting.
///
/// Finds the last space before textwidth that can be used as a break point.
///
/// # Arguments
/// * `line_len` - Length of the line
/// * `textwidth` - Maximum line width
/// * `first_space` - Column of first space in breaking region
///
/// # Returns
/// Column where line should be broken (0 if no break needed)
#[must_use]
#[inline]
pub const fn calc_break_col(line_len: c_int, textwidth: c_int, first_space: c_int) -> c_int {
    if line_len <= textwidth {
        0 // No break needed
    } else if first_space > 0 && first_space <= textwidth {
        first_space
    } else {
        textwidth // Hard break at width
    }
}

/// Check if a character is a format break point.
///
/// # Arguments
/// * `c` - Character to check
///
/// # Returns
/// true if character can be used as break point
#[must_use]
#[inline]
pub const fn is_format_break_char(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Calculate the indent for a formatted line.
///
/// # Arguments
/// * `first_line_indent` - Indent of first line
/// * `is_first_line` - Whether this is the first line
/// * `format_type` - Type of formatting
///
/// # Returns
/// Indent amount for the line
#[must_use]
#[inline]
pub const fn calc_format_indent(
    first_line_indent: c_int,
    _is_first_line: bool,
    _format_type: FormatType,
) -> c_int {
    // Subsequent lines typically follow first line's indent
    first_line_indent
}

/// Check if lines should be joined during formatting.
///
/// Lines are joined if the second line doesn't start a new paragraph.
///
/// # Arguments
/// * `line1_ends_sentence` - Whether first line ends with sentence-ending punctuation
/// * `line2_is_blank` - Whether second line is blank
/// * `line2_is_list_item` - Whether second line starts a list
///
/// # Returns
/// true if lines should be joined
#[must_use]
#[inline]
pub const fn should_join_lines(
    _line1_ends_sentence: bool,
    line2_is_blank: bool,
    line2_is_list_item: bool,
) -> bool {
    !line2_is_blank && !line2_is_list_item
}

/// Calculate the number of spaces between sentences.
///
/// After sentence-ending punctuation (. ! ?), typically 2 spaces in some styles.
///
/// # Arguments
/// * `ends_sentence` - Whether previous text ends a sentence
/// * `use_double_space` - Whether to use double spacing after sentences
///
/// # Returns
/// Number of spaces to insert
#[must_use]
#[inline]
pub const fn calc_sentence_spacing(ends_sentence: bool, use_double_space: bool) -> c_int {
    if ends_sentence && use_double_space {
        2
    } else {
        1
    }
}

// =============================================================================
// Indent Calculations
// =============================================================================

/// Calculate new indent for '=' operator.
///
/// # Arguments
/// * `cur_indent` - Current indentation
/// * `shiftwidth` - Shiftwidth option value
/// * `use_spaces` - Whether to use spaces (expandtab)
///
/// # Returns
/// New indent amount
#[must_use]
#[inline]
pub const fn calc_equalprg_indent(
    cur_indent: c_int,
    shiftwidth: c_int,
    _use_spaces: bool,
) -> c_int {
    // For '=' with no equalprg, use C-indenting or keep indent
    // This is a simplified version - actual implementation uses 'indentexpr' or 'cindent'
    if shiftwidth > 0 {
        // Round to nearest shiftwidth
        let blocks = cur_indent / shiftwidth;
        blocks * shiftwidth
    } else {
        cur_indent
    }
}

/// Check if line needs indentation adjustment.
///
/// # Arguments
/// * `cur_indent` - Current indentation
/// * `desired_indent` - Desired indentation
///
/// # Returns
/// true if indent needs changing
#[must_use]
#[inline]
pub const fn needs_indent_change(cur_indent: c_int, desired_indent: c_int) -> bool {
    cur_indent != desired_indent
}

// =============================================================================
// Format Line Range
// =============================================================================

/// Calculate the effective line range for formatting.
///
/// For linewise operations, uses the full line range.
/// For charwise, may exclude partial lines at start/end.
///
/// # Arguments
/// * `motion_type` - Type of motion
/// * `start_lnum` - Start line
/// * `end_lnum` - End line
/// * `start_col` - Start column (for charwise)
/// * `end_col` - End column (for charwise)
///
/// # Returns
/// `(effective_start, effective_end)` line range
#[must_use]
#[inline]
pub const fn calc_format_range(
    motion_type: MotionType,
    start_lnum: c_int,
    end_lnum: c_int,
    _start_col: c_int,
    _end_col: c_int,
) -> (c_int, c_int) {
    match motion_type {
        MotionType::LineWise => (start_lnum, end_lnum),
        _ => {
            // For charwise, include full lines
            (start_lnum, end_lnum)
        }
    }
}

/// Check if format operation affects multiple lines.
#[must_use]
#[inline]
pub const fn is_multiline_format(start_lnum: c_int, end_lnum: c_int) -> bool {
    end_lnum > start_lnum
}

// =============================================================================
// Phase O3 Format Helpers
// =============================================================================

/// Calculate number of lines affected by format.
#[must_use]
#[inline]
pub const fn calc_format_line_count(start_lnum: c_int, end_lnum: c_int) -> c_int {
    if end_lnum >= start_lnum {
        end_lnum - start_lnum + 1
    } else {
        0
    }
}

/// Check if line is too long and needs wrapping.
#[must_use]
#[inline]
pub const fn line_needs_wrap(line_len: c_int, textwidth: c_int) -> bool {
    textwidth > 0 && line_len > textwidth
}

/// Check if character ends a sentence.
#[must_use]
#[inline]
pub const fn is_sentence_end_char(c: u8) -> bool {
    c == b'.' || c == b'!' || c == b'?'
}

/// Check if character is paragraph separator.
#[must_use]
#[inline]
pub const fn is_paragraph_sep_char(c: u8) -> bool {
    c == b'\n' || c == 0
}

/// Check if line is blank (only whitespace).
#[must_use]
#[inline]
pub const fn is_blank_line(first_char: u8) -> bool {
    first_char == b'\n'
        || first_char == b'\r'
        || first_char == 0
        || first_char == b' '
        || first_char == b'\t'
}

/// Calculate wrap point considering word boundaries.
#[must_use]
#[inline]
pub const fn calc_wrap_col(line_len: c_int, textwidth: c_int, last_space: c_int) -> c_int {
    if line_len <= textwidth {
        0 // No wrap needed
    } else if last_space > 0 && last_space <= textwidth {
        last_space
    } else {
        textwidth
    }
}

/// Check if should add space when joining lines.
#[must_use]
#[inline]
pub const fn needs_join_space(prev_ends_sentence: bool, use_double: bool) -> bool {
    prev_ends_sentence && use_double
}

/// Get number of spaces to use between joined lines.
#[must_use]
#[inline]
pub const fn get_join_spaces(prev_ends_sentence: bool, use_double: bool) -> c_int {
    if prev_ends_sentence && use_double {
        2
    } else {
        1
    }
}

/// Check if format operation should use 'formatexpr'.
#[must_use]
#[inline]
pub const fn should_use_formatexpr(has_formatexpr: bool, format_type: FormatType) -> bool {
    has_formatexpr && !matches!(format_type, FormatType::Indent | FormatType::Internal)
}

/// Check if cursor position needs saving.
#[must_use]
#[inline]
pub const fn needs_cursor_save(format_type: FormatType) -> bool {
    matches!(format_type, FormatType::FormatKeepCursor)
}

/// Calculate cursor column after format.
#[must_use]
#[inline]
pub const fn calc_cursor_col_after_format(
    saved_col: c_int,
    line_changed: bool,
    new_line_len: c_int,
) -> c_int {
    if line_changed && saved_col > new_line_len {
        if new_line_len > 0 {
            new_line_len - 1
        } else {
            0
        }
    } else {
        saved_col
    }
}

/// Check if format message should be shown.
#[must_use]
#[inline]
pub const fn should_show_format_message(line_count: c_int, report_threshold: c_int) -> bool {
    line_count > 0 && report_threshold >= 0 && line_count > report_threshold
}

/// Check if format operation is gq (format text).
#[must_use]
#[inline]
pub const fn is_gq_format(format_type: FormatType) -> bool {
    matches!(format_type, FormatType::Format)
}

/// Check if format operation is gw (format, keep cursor).
#[must_use]
#[inline]
pub const fn is_gw_format(format_type: FormatType) -> bool {
    matches!(format_type, FormatType::FormatKeepCursor)
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get format type from raw value.
#[no_mangle]
pub extern "C" fn rs_format_type_from_raw(value: c_int) -> c_int {
    FormatType::from_raw(value).to_raw()
}

/// FFI: Check if format keeps cursor.
#[no_mangle]
pub extern "C" fn rs_format_keep_cursor(fmt_type: c_int) -> c_int {
    c_int::from(FormatType::from_raw(fmt_type).keep_cursor())
}

/// FFI: Check if format is indent operation.
#[no_mangle]
pub extern "C" fn rs_format_is_indent(fmt_type: c_int) -> c_int {
    c_int::from(FormatType::from_raw(fmt_type).is_indent())
}

/// FFI: Calculate break column.
#[no_mangle]
pub extern "C" fn rs_calc_break_col(
    line_len: c_int,
    textwidth: c_int,
    first_space: c_int,
) -> c_int {
    calc_break_col(line_len, textwidth, first_space)
}

/// FFI: Check if character is format break point.
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub extern "C" fn rs_is_format_break_char(c: c_int) -> c_int {
    c_int::from(is_format_break_char(c as u8))
}

/// FFI: Calculate format indent.
#[no_mangle]
pub extern "C" fn rs_calc_format_indent(
    first_line_indent: c_int,
    is_first_line: c_int,
    format_type: c_int,
) -> c_int {
    calc_format_indent(
        first_line_indent,
        is_first_line != 0,
        FormatType::from_raw(format_type),
    )
}

/// FFI: Check if lines should be joined.
#[no_mangle]
pub extern "C" fn rs_should_join_lines(
    line1_ends_sentence: c_int,
    line2_is_blank: c_int,
    line2_is_list_item: c_int,
) -> c_int {
    c_int::from(should_join_lines(
        line1_ends_sentence != 0,
        line2_is_blank != 0,
        line2_is_list_item != 0,
    ))
}

/// FFI: Calculate sentence spacing.
#[no_mangle]
pub extern "C" fn rs_calc_sentence_spacing(ends_sentence: c_int, use_double_space: c_int) -> c_int {
    calc_sentence_spacing(ends_sentence != 0, use_double_space != 0)
}

/// FFI: Calculate equalprg indent.
#[no_mangle]
pub extern "C" fn rs_calc_equalprg_indent(
    cur_indent: c_int,
    shiftwidth: c_int,
    use_spaces: c_int,
) -> c_int {
    calc_equalprg_indent(cur_indent, shiftwidth, use_spaces != 0)
}

/// FFI: Check if indent change needed.
#[no_mangle]
pub extern "C" fn rs_needs_indent_change(cur_indent: c_int, desired_indent: c_int) -> c_int {
    c_int::from(needs_indent_change(cur_indent, desired_indent))
}

/// FFI: Check if multiline format.
#[no_mangle]
pub extern "C" fn rs_is_multiline_format(start_lnum: c_int, end_lnum: c_int) -> c_int {
    c_int::from(is_multiline_format(start_lnum, end_lnum))
}

/// FFI: Get effective textwidth.
#[no_mangle]
pub extern "C" fn rs_format_effective_width(textwidth: c_int, window_width: c_int) -> c_int {
    let opts = FormatOptions::new(textwidth);
    opts.effective_width(window_width)
}

/// FFI: Calculate format line count.
#[no_mangle]
pub extern "C" fn rs_calc_format_line_count(start_lnum: c_int, end_lnum: c_int) -> c_int {
    calc_format_line_count(start_lnum, end_lnum)
}

/// FFI: Check if line needs wrapping.
#[no_mangle]
pub extern "C" fn rs_line_needs_wrap(line_len: c_int, textwidth: c_int) -> c_int {
    c_int::from(line_needs_wrap(line_len, textwidth))
}

/// FFI: Check if character ends a sentence.
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub extern "C" fn rs_is_sentence_end_char(c: c_int) -> c_int {
    c_int::from(is_sentence_end_char(c as u8))
}

/// FFI: Check if character is paragraph separator.
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub extern "C" fn rs_is_paragraph_sep_char(c: c_int) -> c_int {
    c_int::from(is_paragraph_sep_char(c as u8))
}

/// FFI: Check if line is blank.
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub extern "C" fn rs_is_blank_line(first_char: c_int) -> c_int {
    c_int::from(is_blank_line(first_char as u8))
}

/// FFI: Calculate wrap column.
#[no_mangle]
pub extern "C" fn rs_calc_wrap_col(line_len: c_int, textwidth: c_int, last_space: c_int) -> c_int {
    calc_wrap_col(line_len, textwidth, last_space)
}

/// FFI: Check if join needs extra space.
#[no_mangle]
pub extern "C" fn rs_needs_join_space(prev_ends_sentence: c_int, use_double: c_int) -> c_int {
    c_int::from(needs_join_space(prev_ends_sentence != 0, use_double != 0))
}

/// FFI: Get number of join spaces.
#[no_mangle]
pub extern "C" fn rs_get_join_spaces(prev_ends_sentence: c_int, use_double: c_int) -> c_int {
    get_join_spaces(prev_ends_sentence != 0, use_double != 0)
}

/// FFI: Check if should use formatexpr.
#[no_mangle]
pub extern "C" fn rs_should_use_formatexpr(has_formatexpr: c_int, format_type: c_int) -> c_int {
    c_int::from(should_use_formatexpr(
        has_formatexpr != 0,
        FormatType::from_raw(format_type),
    ))
}

/// FFI: Check if cursor needs saving.
#[no_mangle]
pub extern "C" fn rs_needs_cursor_save(format_type: c_int) -> c_int {
    c_int::from(needs_cursor_save(FormatType::from_raw(format_type)))
}

/// FFI: Calculate cursor column after format.
#[no_mangle]
pub extern "C" fn rs_calc_cursor_col_after_format(
    saved_col: c_int,
    line_changed: c_int,
    new_line_len: c_int,
) -> c_int {
    calc_cursor_col_after_format(saved_col, line_changed != 0, new_line_len)
}

/// FFI: Check if format message should be shown.
#[no_mangle]
pub extern "C" fn rs_should_show_format_message(
    line_count: c_int,
    report_threshold: c_int,
) -> c_int {
    c_int::from(should_show_format_message(line_count, report_threshold))
}

/// FFI: Check if gq format.
#[no_mangle]
pub extern "C" fn rs_is_gq_format(format_type: c_int) -> c_int {
    c_int::from(is_gq_format(FormatType::from_raw(format_type)))
}

/// FFI: Check if gw format.
#[no_mangle]
pub extern "C" fn rs_is_gw_format(format_type: c_int) -> c_int {
    c_int::from(is_gw_format(FormatType::from_raw(format_type)))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_type() {
        assert!(!FormatType::Format.keep_cursor());
        assert!(FormatType::FormatKeepCursor.keep_cursor());
        assert!(!FormatType::Indent.keep_cursor());

        assert!(!FormatType::Format.is_indent());
        assert!(FormatType::Indent.is_indent());

        assert_eq!(FormatType::from_raw(0), FormatType::Format);
        assert_eq!(FormatType::from_raw(1), FormatType::FormatKeepCursor);
        assert_eq!(FormatType::from_raw(2), FormatType::Indent);
    }

    #[test]
    fn test_format_options() {
        let opts = FormatOptions::new(80);
        assert_eq!(opts.textwidth, 80);
        assert!(opts.wrap);
        assert!(opts.join_para);

        // Effective width with textwidth set
        assert_eq!(opts.effective_width(100), 80);

        // Effective width with textwidth=0
        let opts = FormatOptions::new(0);
        assert_eq!(opts.effective_width(80), 79);
        assert_eq!(opts.effective_width(0), 79);
    }

    #[test]
    fn test_calc_break_col() {
        // Line within textwidth - no break
        assert_eq!(calc_break_col(50, 80, 10), 0);

        // Line over textwidth with space
        assert_eq!(calc_break_col(100, 80, 70), 70);

        // Line over textwidth with space past textwidth
        assert_eq!(calc_break_col(100, 80, 85), 80);

        // Line over textwidth with no space
        assert_eq!(calc_break_col(100, 80, 0), 80);
    }

    #[test]
    fn test_is_format_break_char() {
        assert!(is_format_break_char(b' '));
        assert!(is_format_break_char(b'\t'));
        assert!(!is_format_break_char(b'a'));
        assert!(!is_format_break_char(b'\n'));
    }

    #[test]
    fn test_should_join_lines() {
        // Normal lines - join
        assert!(should_join_lines(false, false, false));

        // Blank second line - don't join
        assert!(!should_join_lines(false, true, false));

        // List item - don't join
        assert!(!should_join_lines(false, false, true));
    }

    #[test]
    fn test_calc_sentence_spacing() {
        assert_eq!(calc_sentence_spacing(true, true), 2);
        assert_eq!(calc_sentence_spacing(true, false), 1);
        assert_eq!(calc_sentence_spacing(false, true), 1);
        assert_eq!(calc_sentence_spacing(false, false), 1);
    }

    #[test]
    fn test_calc_equalprg_indent() {
        assert_eq!(calc_equalprg_indent(8, 4, true), 8);
        assert_eq!(calc_equalprg_indent(10, 4, true), 8);
        assert_eq!(calc_equalprg_indent(6, 4, true), 4);
        assert_eq!(calc_equalprg_indent(4, 0, true), 4);
    }

    #[test]
    fn test_needs_indent_change() {
        assert!(needs_indent_change(4, 8));
        assert!(!needs_indent_change(8, 8));
    }

    #[test]
    fn test_calc_format_range() {
        // Linewise - full range
        let (s, e) = calc_format_range(MotionType::LineWise, 5, 10, 0, 50);
        assert_eq!(s, 5);
        assert_eq!(e, 10);

        // Charwise - also full lines
        let (s, e) = calc_format_range(MotionType::CharWise, 5, 10, 5, 50);
        assert_eq!(s, 5);
        assert_eq!(e, 10);
    }

    #[test]
    fn test_is_multiline_format() {
        assert!(is_multiline_format(5, 10));
        assert!(!is_multiline_format(5, 5));
    }

    #[test]
    fn test_ffi_wrappers() {
        assert_eq!(rs_format_keep_cursor(1), 1);
        assert_eq!(rs_format_keep_cursor(0), 0);
        assert_eq!(rs_format_is_indent(2), 1);
        assert_eq!(rs_calc_break_col(100, 80, 70), 70);
        assert_eq!(rs_is_format_break_char(c_int::from(b' ')), 1);
        assert_eq!(rs_calc_sentence_spacing(1, 1), 2);
    }

    // Phase O3 tests
    #[test]
    fn test_calc_format_line_count() {
        assert_eq!(calc_format_line_count(1, 10), 10);
        assert_eq!(calc_format_line_count(5, 5), 1);
        assert_eq!(calc_format_line_count(10, 5), 0);
    }

    #[test]
    fn test_line_needs_wrap() {
        assert!(line_needs_wrap(100, 80));
        assert!(!line_needs_wrap(50, 80));
        assert!(!line_needs_wrap(50, 0)); // textwidth=0 means no wrapping
    }

    #[test]
    fn test_is_sentence_end_char() {
        assert!(is_sentence_end_char(b'.'));
        assert!(is_sentence_end_char(b'!'));
        assert!(is_sentence_end_char(b'?'));
        assert!(!is_sentence_end_char(b','));
        assert!(!is_sentence_end_char(b' '));
    }

    #[test]
    fn test_is_paragraph_sep_char() {
        assert!(is_paragraph_sep_char(b'\n'));
        assert!(is_paragraph_sep_char(0));
        assert!(!is_paragraph_sep_char(b' '));
        assert!(!is_paragraph_sep_char(b'a'));
    }

    #[test]
    fn test_is_blank_line() {
        assert!(is_blank_line(b'\n'));
        assert!(is_blank_line(b' '));
        assert!(is_blank_line(b'\t'));
        assert!(is_blank_line(0));
        assert!(!is_blank_line(b'a'));
    }

    #[test]
    fn test_calc_wrap_col() {
        assert_eq!(calc_wrap_col(50, 80, 30), 0); // No wrap needed
        assert_eq!(calc_wrap_col(100, 80, 70), 70); // Wrap at space
        assert_eq!(calc_wrap_col(100, 80, 0), 80); // No space, hard break
    }

    #[test]
    fn test_needs_join_space() {
        assert!(needs_join_space(true, true));
        assert!(!needs_join_space(true, false));
        assert!(!needs_join_space(false, true));
    }

    #[test]
    fn test_get_join_spaces() {
        assert_eq!(get_join_spaces(true, true), 2);
        assert_eq!(get_join_spaces(true, false), 1);
        assert_eq!(get_join_spaces(false, true), 1);
    }

    #[test]
    fn test_should_use_formatexpr() {
        assert!(should_use_formatexpr(true, FormatType::Format));
        assert!(should_use_formatexpr(true, FormatType::FormatKeepCursor));
        assert!(!should_use_formatexpr(true, FormatType::Indent));
        assert!(!should_use_formatexpr(true, FormatType::Internal));
        assert!(!should_use_formatexpr(false, FormatType::Format));
    }

    #[test]
    fn test_needs_cursor_save() {
        assert!(needs_cursor_save(FormatType::FormatKeepCursor));
        assert!(!needs_cursor_save(FormatType::Format));
        assert!(!needs_cursor_save(FormatType::Indent));
    }

    #[test]
    fn test_calc_cursor_col_after_format() {
        // Line not changed
        assert_eq!(calc_cursor_col_after_format(50, false, 80), 50);
        // Line changed, cursor within bounds
        assert_eq!(calc_cursor_col_after_format(30, true, 80), 30);
        // Line changed, cursor past end
        assert_eq!(calc_cursor_col_after_format(90, true, 80), 79);
        // Line changed, empty line
        assert_eq!(calc_cursor_col_after_format(10, true, 0), 0);
    }

    #[test]
    fn test_should_show_format_message() {
        assert!(should_show_format_message(10, 5));
        assert!(!should_show_format_message(3, 5));
        assert!(!should_show_format_message(5, -1)); // negative threshold = silent
    }

    #[test]
    fn test_is_gq_gw_format() {
        assert!(is_gq_format(FormatType::Format));
        assert!(!is_gq_format(FormatType::FormatKeepCursor));
        assert!(is_gw_format(FormatType::FormatKeepCursor));
        assert!(!is_gw_format(FormatType::Format));
    }
}
