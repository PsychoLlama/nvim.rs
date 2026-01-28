//! Buffer functions for VimL.
//!
//! This module implements buffer-related functions from `src/nvim/eval/funcs.c`:
//! - Buffer identification helpers
//! - Buffer name/number conversion utilities
//!
//! ## Note
//!
//! These are helper functions that work with buffer identifiers.
//! The actual buffer operations require C FFI calls that access buffer state.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::manual_range_contains)]

use std::ffi::c_int;

// =============================================================================
// Buffer Identifier Types
// =============================================================================

/// Buffer identifier types in VimL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum BufIdType {
    /// Buffer number (integer)
    Number = 0,
    /// Buffer name (string pattern)
    Name = 1,
    /// Current buffer ($)
    Current = 2,
    /// Alternate buffer (#)
    Alternate = 3,
    /// Last buffer (%)
    Last = 4,
    /// Invalid buffer id
    Invalid = -1,
}

impl BufIdType {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Number,
            1 => Self::Name,
            2 => Self::Current,
            3 => Self::Alternate,
            4 => Self::Last,
            _ => Self::Invalid,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Buffer Existence Check Helpers
// =============================================================================

/// Special buffer number values.
pub mod special_bufnr {
    /// Current buffer
    pub const CURRENT: i64 = 0;
    /// Alternate buffer
    pub const ALTERNATE: i64 = -1;
    /// No buffer / invalid
    pub const NONE: i64 = -2;
}

/// Check if a buffer number is valid (positive).
pub const fn is_valid_bufnr(bufnr: i64) -> bool {
    bufnr > 0
}

/// Check if a buffer number is a special value.
pub const fn is_special_bufnr(bufnr: i64) -> bool {
    bufnr <= 0
}

/// FFI export: check if buffer number is valid.
#[no_mangle]
pub extern "C" fn rs_buf_is_valid_bufnr(bufnr: i64) -> bool {
    is_valid_bufnr(bufnr)
}

/// FFI export: check if buffer number is special.
#[no_mangle]
pub extern "C" fn rs_buf_is_special_bufnr(bufnr: i64) -> bool {
    is_special_bufnr(bufnr)
}

// =============================================================================
// Buffer Line Operations Helpers
// =============================================================================

/// Validate line number for a buffer.
///
/// VimL line numbers are 1-based.
/// Returns true if the line number is valid for a buffer with `line_count` lines.
pub const fn is_valid_lnum(lnum: i64, line_count: i64) -> bool {
    lnum >= 1 && lnum <= line_count
}

/// Normalize line number (handle special values).
///
/// VimL special line values:
/// - 0: means line before first line (invalid in most contexts)
/// - $: means last line (represented as large negative or special value)
/// - .: means current line
pub const fn normalize_lnum(lnum: i64, line_count: i64, _current_line: i64) -> Option<i64> {
    if lnum == 0 {
        // Line 0 is usually invalid
        return None;
    }
    if lnum < 0 {
        // Negative could mean from end, but VimL doesn't support this for lines
        return None;
    }
    if lnum > line_count {
        // Beyond end of buffer
        return None;
    }
    Some(lnum)
}

/// FFI export: validate line number.
#[no_mangle]
pub extern "C" fn rs_buf_is_valid_lnum(lnum: i64, line_count: i64) -> bool {
    is_valid_lnum(lnum, line_count)
}

// =============================================================================
// Buffer Option Helpers
// =============================================================================

/// Buffer-local option scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum BufOptScope {
    /// Buffer-local value
    Local = 0,
    /// Global default
    Global = 1,
}

impl BufOptScope {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Global,
            _ => Self::Local,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Buffer Info Types
// =============================================================================

/// Buffer type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum BufType {
    /// Normal buffer
    Normal = 0,
    /// Help buffer
    Help = 1,
    /// Quickfix buffer
    Quickfix = 2,
    /// Terminal buffer
    Terminal = 3,
    /// Prompt buffer
    Prompt = 4,
    /// Popup buffer
    Popup = 5,
    /// Scratch buffer (nofile, noswap)
    Scratch = 6,
}

impl BufType {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Help,
            2 => Self::Quickfix,
            3 => Self::Terminal,
            4 => Self::Prompt,
            5 => Self::Popup,
            6 => Self::Scratch,
            _ => Self::Normal,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if buffer type is special (non-file).
    pub const fn is_special(self) -> bool {
        !matches!(self, Self::Normal)
    }
}

/// FFI: Check if buffer type is special.
#[no_mangle]
pub extern "C" fn rs_buf_type_is_special(buf_type: c_int) -> c_int {
    c_int::from(BufType::from_c_int(buf_type).is_special())
}

// =============================================================================
// Buffer State Flags
// =============================================================================

/// Buffer state flags.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BufStateFlags {
    /// Buffer is modified
    pub modified: bool,
    /// Buffer is read-only
    pub readonly: bool,
    /// Buffer is listed
    pub listed: bool,
    /// Buffer is loaded
    pub loaded: bool,
    /// Buffer has a file name
    pub named: bool,
    /// Buffer has a swap file
    pub swapfile: bool,
}

impl BufStateFlags {
    /// Create new flags with all false.
    pub const fn new() -> Self {
        Self {
            modified: false,
            readonly: false,
            listed: true,
            loaded: false,
            named: false,
            swapfile: true,
        }
    }
}

/// FFI: Create default buffer state flags.
#[no_mangle]
pub extern "C" fn rs_buf_state_flags_new() -> BufStateFlags {
    BufStateFlags::new()
}

// =============================================================================
// Line Range Operations
// =============================================================================

/// Clamp a line number to valid range.
///
/// # Arguments
/// * `lnum` - Line number to clamp
/// * `line_count` - Total lines in buffer
///
/// # Returns
/// Clamped line number (1 <= result <= line_count)
pub const fn clamp_lnum(lnum: i64, line_count: i64) -> i64 {
    if lnum < 1 {
        1
    } else if lnum > line_count {
        line_count
    } else {
        lnum
    }
}

/// FFI: Clamp line number.
#[no_mangle]
pub extern "C" fn rs_buf_clamp_lnum(lnum: i64, line_count: i64) -> i64 {
    clamp_lnum(lnum, line_count)
}

/// Calculate line range for operations like deletebufline().
///
/// # Arguments
/// * `first` - First line (1-based)
/// * `last` - Last line (1-based, or 0 for same as first)
/// * `line_count` - Total lines in buffer
///
/// # Returns
/// (start, end) tuple with validated range, or None if invalid
pub const fn calc_line_range(first: i64, last: i64, line_count: i64) -> Option<(i64, i64)> {
    if first < 1 || first > line_count {
        return None;
    }

    let end = if last == 0 { first } else { last };

    if end < first || end > line_count {
        return None;
    }

    Some((first, end))
}

/// FFI: Calculate line range start.
#[no_mangle]
pub extern "C" fn rs_buf_calc_range_start(first: i64, last: i64, line_count: i64) -> i64 {
    calc_line_range(first, last, line_count).map_or(-1, |(s, _)| s)
}

/// FFI: Calculate line range end.
#[no_mangle]
pub extern "C" fn rs_buf_calc_range_end(first: i64, last: i64, line_count: i64) -> i64 {
    calc_line_range(first, last, line_count).map_or(-1, |(_, e)| e)
}

// =============================================================================
// Buffer Variable Helpers
// =============================================================================

/// Check if a variable name is valid for buffer variables.
///
/// Buffer variable names must start with a letter and contain only
/// alphanumeric characters and underscores.
pub fn is_valid_bufvar_name(name: &[u8]) -> bool {
    if name.is_empty() {
        return false;
    }

    // First character must be a letter
    if !name[0].is_ascii_alphabetic() {
        return false;
    }

    // Rest must be alphanumeric or underscore
    name[1..]
        .iter()
        .all(|&c| c.is_ascii_alphanumeric() || c == b'_')
}

// =============================================================================
// Line/Byte Conversion Helpers
// =============================================================================

/// Result of line2byte or byte2line conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineByteResult {
    /// Valid result
    Valid(i64),
    /// Line/byte is out of range
    OutOfRange,
    /// Buffer has no byte offset information
    NoInfo,
}

impl LineByteResult {
    /// Convert to C int (-1 for errors, value for success).
    pub const fn to_c_int(&self) -> i64 {
        match self {
            Self::Valid(v) => *v,
            Self::OutOfRange => -1,
            Self::NoInfo => -1,
        }
    }
}

/// Calculate the byte offset from line contents.
///
/// This is a pure calculation helper - actual byte offset lookup
/// requires memline FFI.
pub fn calculate_byte_offset(line_lengths: &[usize], target_line: i64) -> LineByteResult {
    if target_line < 1 {
        return LineByteResult::OutOfRange;
    }

    let target = target_line as usize;
    if target > line_lengths.len() + 1 {
        return LineByteResult::OutOfRange;
    }

    let mut offset: i64 = 0;
    for (i, &len) in line_lengths.iter().enumerate() {
        if i + 1 >= target {
            break;
        }
        offset += len as i64 + 1; // +1 for newline
    }

    LineByteResult::Valid(offset + 1) // VimL line2byte is 1-based
}

/// Find line number from byte offset.
///
/// Returns the line number (1-based) that contains the byte at `byte_offset`.
pub fn find_line_from_byte(line_lengths: &[usize], byte_offset: i64) -> LineByteResult {
    if byte_offset < 1 {
        return LineByteResult::OutOfRange;
    }

    let mut current_offset: i64 = 1; // 1-based
    for (i, &len) in line_lengths.iter().enumerate() {
        let line_end = current_offset + len as i64;
        if byte_offset <= line_end {
            return LineByteResult::Valid((i + 1) as i64);
        }
        current_offset = line_end + 1; // +1 for newline
    }

    LineByteResult::OutOfRange
}

// =============================================================================
// Column Helpers
// =============================================================================

/// Maximum column value (end of line).
pub const MAX_COL: i64 = 2_147_483_647;

/// Check if column number is valid.
pub const fn is_valid_col(col: i64) -> bool {
    col >= 1
}

/// Clamp column to valid range for a line.
pub const fn clamp_col(col: i64, line_len: i64) -> i64 {
    if col < 1 {
        1
    } else if col > line_len + 1 {
        line_len + 1 // Can position after last char
    } else {
        col
    }
}

/// FFI: Check if column is valid.
#[no_mangle]
pub extern "C" fn rs_buf_is_valid_col(col: i64) -> bool {
    is_valid_col(col)
}

/// FFI: Clamp column.
#[no_mangle]
pub extern "C" fn rs_buf_clamp_col(col: i64, line_len: i64) -> i64 {
    clamp_col(col, line_len)
}

// =============================================================================
// Virtcol Helpers
// =============================================================================

/// Virtual column calculation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum VirtcolMode {
    /// First position of tab/multi-cell character
    Start = 0,
    /// Last position of tab/multi-cell character
    End = 1,
    /// Cursor position (same as End for most purposes)
    Cursor = 2,
}

impl VirtcolMode {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::End,
            2 => Self::Cursor,
            _ => Self::Start,
        }
    }
}

/// Calculate display width of a character for virtcol.
///
/// Returns the number of screen cells the character occupies.
pub const fn char_display_width(c: u8, tab_width: usize) -> usize {
    match c {
        b'\t' => tab_width,
        // Control characters display as ^X (2 cells)
        0..=31 => 2,
        // DEL displays as ^? (2 cells)
        127 => 2,
        // Regular ASCII is 1 cell
        _ => 1,
    }
}

/// FFI: Get character display width.
#[no_mangle]
pub extern "C" fn rs_buf_char_display_width(c: c_int, tab_width: c_int) -> c_int {
    if c < 0 || c > 255 || tab_width < 1 {
        return 1;
    }
    char_display_width(c as u8, tab_width as usize) as c_int
}

// =============================================================================
// Indentation Helpers
// =============================================================================

/// Count leading whitespace (indent level).
///
/// Returns (indent_chars, indent_width) where:
/// - indent_chars: number of whitespace characters
/// - indent_width: visual indent width (tabs expanded)
pub fn count_indent(line: &[u8], tab_width: usize) -> (usize, usize) {
    let mut chars = 0;
    let mut width = 0;

    for &c in line {
        match c {
            b' ' => {
                chars += 1;
                width += 1;
            }
            b'\t' => {
                chars += 1;
                // Tab aligns to next tab stop
                width = (width / tab_width + 1) * tab_width;
            }
            _ => break,
        }
    }

    (chars, width)
}

/// Generate indentation string of given width.
///
/// Returns the number of bytes written to `out`.
pub fn make_indent(width: usize, use_tabs: bool, tab_width: usize, out: &mut [u8]) -> usize {
    if out.is_empty() || width == 0 {
        return 0;
    }

    let mut pos = 0;

    if use_tabs && tab_width > 0 {
        // Use tabs where possible
        let num_tabs = width / tab_width;
        let remaining_spaces = width % tab_width;

        for _ in 0..num_tabs {
            if pos >= out.len() {
                return pos;
            }
            out[pos] = b'\t';
            pos += 1;
        }

        for _ in 0..remaining_spaces {
            if pos >= out.len() {
                return pos;
            }
            out[pos] = b' ';
            pos += 1;
        }
    } else {
        // Use spaces only
        for _ in 0..width {
            if pos >= out.len() {
                return pos;
            }
            out[pos] = b' ';
            pos += 1;
        }
    }

    pos
}

/// FFI: Count indent width.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_count_indent_width(
    line: *const u8,
    len: c_int,
    tab_width: c_int,
) -> c_int {
    if line.is_null() || len < 0 || tab_width < 1 {
        return 0;
    }
    let slice = std::slice::from_raw_parts(line, len as usize);
    let (_, width) = count_indent(slice, tab_width as usize);
    width as c_int
}

/// FFI: Count indent chars.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_count_indent_chars(
    line: *const u8,
    len: c_int,
    tab_width: c_int,
) -> c_int {
    if line.is_null() || len < 0 || tab_width < 1 {
        return 0;
    }
    let slice = std::slice::from_raw_parts(line, len as usize);
    let (chars, _) = count_indent(slice, tab_width as usize);
    chars as c_int
}

// =============================================================================
// Append/Insert Position Helpers
// =============================================================================

/// Append position for append(), appendbufline().
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum AppendPos {
    /// Append after specified line
    After = 0,
    /// Insert before specified line
    Before = 1,
    /// Append at end of buffer
    End = 2,
}

impl AppendPos {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Before,
            2 => Self::End,
            _ => Self::After,
        }
    }
}

/// Validate append position.
///
/// For append operations:
/// - Line 0 means before first line
/// - Line $ (last) means after last line
/// - Other lines must be valid
pub const fn validate_append_lnum(lnum: i64, line_count: i64) -> bool {
    lnum >= 0 && lnum <= line_count
}

/// FFI: Validate append position.
#[no_mangle]
pub extern "C" fn rs_buf_validate_append_lnum(lnum: i64, line_count: i64) -> bool {
    validate_append_lnum(lnum, line_count)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buf_id_type() {
        assert_eq!(BufIdType::from_c_int(0), BufIdType::Number);
        assert_eq!(BufIdType::from_c_int(-1), BufIdType::Invalid);
    }

    #[test]
    fn test_is_valid_bufnr() {
        assert!(is_valid_bufnr(1));
        assert!(is_valid_bufnr(100));
        assert!(!is_valid_bufnr(0));
        assert!(!is_valid_bufnr(-1));
    }

    #[test]
    fn test_is_special_bufnr() {
        assert!(is_special_bufnr(0));
        assert!(is_special_bufnr(-1));
        assert!(!is_special_bufnr(1));
    }

    #[test]
    fn test_is_valid_lnum() {
        // Buffer with 10 lines
        assert!(is_valid_lnum(1, 10));
        assert!(is_valid_lnum(10, 10));
        assert!(!is_valid_lnum(0, 10));
        assert!(!is_valid_lnum(11, 10));
        assert!(!is_valid_lnum(-1, 10));
    }

    #[test]
    fn test_normalize_lnum() {
        // Buffer with 10 lines, current line 5
        assert_eq!(normalize_lnum(1, 10, 5), Some(1));
        assert_eq!(normalize_lnum(10, 10, 5), Some(10));
        assert_eq!(normalize_lnum(0, 10, 5), None);
        assert_eq!(normalize_lnum(11, 10, 5), None);
        assert_eq!(normalize_lnum(-1, 10, 5), None);
    }

    #[test]
    fn test_buf_type() {
        assert_eq!(BufType::from_c_int(0), BufType::Normal);
        assert_eq!(BufType::from_c_int(3), BufType::Terminal);
        assert!(!BufType::Normal.is_special());
        assert!(BufType::Help.is_special());
        assert!(BufType::Terminal.is_special());
    }

    #[test]
    fn test_clamp_lnum() {
        assert_eq!(clamp_lnum(5, 10), 5);
        assert_eq!(clamp_lnum(0, 10), 1);
        assert_eq!(clamp_lnum(-5, 10), 1);
        assert_eq!(clamp_lnum(15, 10), 10);
    }

    #[test]
    fn test_calc_line_range() {
        // Valid ranges
        assert_eq!(calc_line_range(1, 5, 10), Some((1, 5)));
        assert_eq!(calc_line_range(5, 0, 10), Some((5, 5))); // last=0 means same as first
        assert_eq!(calc_line_range(1, 10, 10), Some((1, 10)));

        // Invalid ranges
        assert_eq!(calc_line_range(0, 5, 10), None); // first < 1
        assert_eq!(calc_line_range(11, 15, 10), None); // first > line_count
        assert_eq!(calc_line_range(5, 3, 10), None); // end < first
        assert_eq!(calc_line_range(5, 15, 10), None); // end > line_count
    }

    #[test]
    fn test_is_valid_bufvar_name() {
        assert!(is_valid_bufvar_name(b"foo"));
        assert!(is_valid_bufvar_name(b"myVar123"));
        assert!(is_valid_bufvar_name(b"my_var"));
        assert!(!is_valid_bufvar_name(b""));
        assert!(!is_valid_bufvar_name(b"123var")); // starts with digit
        assert!(!is_valid_bufvar_name(b"_var")); // starts with underscore
    }

    #[test]
    fn test_line_byte_conversion() {
        // Lines with lengths: 5, 10, 3 (+ newlines)
        let line_lengths = [5, 10, 3];

        // Line 1 starts at byte 1
        assert_eq!(
            calculate_byte_offset(&line_lengths, 1),
            LineByteResult::Valid(1)
        );
        // Line 2 starts at byte 7 (5 + newline + 1)
        assert_eq!(
            calculate_byte_offset(&line_lengths, 2),
            LineByteResult::Valid(7)
        );
        // Line 3 starts at byte 18 (5 + 1 + 10 + 1 + 1)
        assert_eq!(
            calculate_byte_offset(&line_lengths, 3),
            LineByteResult::Valid(18)
        );

        // Out of range
        assert_eq!(
            calculate_byte_offset(&line_lengths, 0),
            LineByteResult::OutOfRange
        );
    }

    #[test]
    fn test_find_line_from_byte() {
        let line_lengths = [5, 10, 3];

        // Byte 1-5 are on line 1
        assert_eq!(
            find_line_from_byte(&line_lengths, 1),
            LineByteResult::Valid(1)
        );
        assert_eq!(
            find_line_from_byte(&line_lengths, 5),
            LineByteResult::Valid(1)
        );

        // Byte 7-16 are on line 2
        assert_eq!(
            find_line_from_byte(&line_lengths, 7),
            LineByteResult::Valid(2)
        );

        // Out of range
        assert_eq!(
            find_line_from_byte(&line_lengths, 0),
            LineByteResult::OutOfRange
        );
    }

    #[test]
    fn test_column_helpers() {
        assert!(is_valid_col(1));
        assert!(!is_valid_col(0));
        assert!(!is_valid_col(-1));

        assert_eq!(clamp_col(5, 10), 5);
        assert_eq!(clamp_col(0, 10), 1);
        assert_eq!(clamp_col(15, 10), 11); // can be after last char
    }

    #[test]
    fn test_char_display_width() {
        assert_eq!(char_display_width(b'a', 8), 1);
        assert_eq!(char_display_width(b'\t', 8), 8);
        assert_eq!(char_display_width(b'\t', 4), 4);
        assert_eq!(char_display_width(0, 8), 2); // NUL = ^@
        assert_eq!(char_display_width(127, 8), 2); // DEL = ^?
    }

    #[test]
    fn test_count_indent() {
        assert_eq!(count_indent(b"    hello", 8), (4, 4));
        assert_eq!(count_indent(b"\thello", 8), (1, 8));
        assert_eq!(count_indent(b"  \thello", 8), (3, 8)); // 2 spaces + tab to 8
        assert_eq!(count_indent(b"hello", 8), (0, 0));
        assert_eq!(count_indent(b"", 8), (0, 0));
    }

    #[test]
    fn test_make_indent() {
        let mut out = [0u8; 32];

        // Spaces only
        let len = make_indent(4, false, 8, &mut out);
        assert_eq!(&out[..len], b"    ");

        // With tabs
        let len = make_indent(10, true, 4, &mut out);
        assert_eq!(&out[..len], b"\t\t  "); // 2 tabs (8) + 2 spaces
    }

    #[test]
    fn test_validate_append_lnum() {
        assert!(validate_append_lnum(0, 10)); // before first
        assert!(validate_append_lnum(5, 10)); // after line 5
        assert!(validate_append_lnum(10, 10)); // after last
        assert!(!validate_append_lnum(-1, 10));
        assert!(!validate_append_lnum(11, 10));
    }
}
