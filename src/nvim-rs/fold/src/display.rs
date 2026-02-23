//! Fold display logic
//!
//! This module provides Rust implementations for fold display,
//! including fold text generation and visual representation.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int};
use std::ptr;

use nvim_mbyte::utfc_ptr2len;
use nvim_window::WinHandle;

use crate::markers::parse_marker_impl;

/// Line number type
type LinenrT = i32;

// =============================================================================
// C accessor declarations for foldtext_cleanup
// =============================================================================

extern "C" {
    /// Get curbuf's commentstring option (b_p_cms).
    fn nvim_get_curbuf_b_p_cms() -> *const c_char;
    /// Get the current window handle.
    fn nvim_get_curwin() -> WinHandle;
    /// Skip whitespace at the beginning of a string.
    fn nvim_skipwhite(s: *const c_char) -> *const c_char;
}

// =============================================================================
// Fold Display Constants
// =============================================================================

/// Default fold fill character
pub const FOLD_FILL_CHAR: u8 = b'-';

/// Fold closed indicator
pub const FOLD_CLOSED_CHAR: u8 = b'+';

/// Fold open indicator
pub const FOLD_OPEN_CHAR: u8 = b'-';

/// Maximum foldtext length
pub const FOLDTEXT_MAX_LEN: usize = 256;

// =============================================================================
// Fold Column Display
// =============================================================================

/// Fold column character types
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldColumnChar {
    /// No fold at this position
    None = 0,
    /// Fold starts here (closed)
    ClosedStart = 1,
    /// Fold starts here (open)
    OpenStart = 2,
    /// Inside a fold (vertical bar)
    Inside = 3,
    /// Fold ends here
    End = 4,
    /// Nested fold indicator
    Nested = 5,
}

impl FoldColumnChar {
    /// Get display character for this type
    pub const fn as_char(self) -> u8 {
        match self {
            Self::None => b' ',
            Self::ClosedStart => b'+',
            Self::OpenStart => b'-',
            Self::Inside => b'|',
            Self::End => b'|',
            Self::Nested => b'>',
        }
    }

    /// Check if this represents a fold start
    pub const fn is_start(self) -> bool {
        matches!(self, Self::ClosedStart | Self::OpenStart)
    }

    /// Check if this represents a clickable fold
    pub const fn is_clickable(self) -> bool {
        matches!(self, Self::ClosedStart | Self::OpenStart)
    }
}

// =============================================================================
// Fold Display Info
// =============================================================================

/// Information about a fold for display purposes
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldDisplayInfo {
    /// First line of fold (1-based)
    pub first_line: LinenrT,
    /// Last line of fold (1-based)
    pub last_line: LinenrT,
    /// Number of lines in fold
    pub line_count: LinenrT,
    /// Fold level (1-based)
    pub level: c_int,
    /// Whether fold is closed
    pub closed: bool,
    /// Whether fold has nested folds
    pub has_nested: bool,
}

impl Default for FoldDisplayInfo {
    fn default() -> Self {
        Self {
            first_line: 0,
            last_line: 0,
            line_count: 0,
            level: 0,
            closed: false,
            has_nested: false,
        }
    }
}

impl FoldDisplayInfo {
    /// Create display info for a closed fold
    pub const fn closed(first: LinenrT, last: LinenrT, level: c_int) -> Self {
        Self {
            first_line: first,
            last_line: last,
            line_count: last - first + 1,
            level,
            closed: true,
            has_nested: false,
        }
    }

    /// Create display info for an open fold
    pub const fn open(first: LinenrT, last: LinenrT, level: c_int) -> Self {
        Self {
            first_line: first,
            last_line: last,
            line_count: last - first + 1,
            level,
            closed: false,
            has_nested: false,
        }
    }

    /// Check if this is a valid fold
    pub const fn is_valid(&self) -> bool {
        self.first_line > 0 && self.last_line >= self.first_line
    }
}

// =============================================================================
// Fold Text Generation
// =============================================================================

/// Format string for default fold text
pub const DEFAULT_FOLDTEXT_FORMAT: &str = "+-- %d lines: %s";

/// Components for fold text
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldTextComponents {
    /// Number of lines in fold
    pub line_count: LinenrT,
    /// Level of the fold
    pub level: c_int,
    /// Number of dashes to show (based on level)
    pub dash_count: c_int,
    /// Whether to show percentage
    pub show_percent: bool,
}

impl FoldTextComponents {
    /// Create components for a fold
    pub const fn new(line_count: LinenrT, level: c_int) -> Self {
        let dashes = level.saturating_sub(1);
        Self {
            line_count,
            level,
            dash_count: if dashes > 0 { dashes } else { 0 },
            show_percent: false,
        }
    }

    /// Calculate percentage of buffer that fold represents
    pub fn percent_of_buffer(&self, total_lines: LinenrT) -> u8 {
        if total_lines <= 0 {
            return 0;
        }
        let percent = (self.line_count as i64 * 100) / total_lines as i64;
        percent.min(100) as u8
    }
}

// =============================================================================
// Fold Column Configuration
// =============================================================================

/// Fold column configuration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldColumnConfig {
    /// Width of fold column (0 = hidden)
    pub width: c_int,
    /// Maximum level to show
    pub max_level: c_int,
    /// Whether to show level numbers
    pub show_numbers: bool,
    /// Fill character
    pub fill_char: u8,
}

impl Default for FoldColumnConfig {
    fn default() -> Self {
        Self {
            width: 0,
            max_level: 20,
            show_numbers: false,
            fill_char: b' ',
        }
    }
}

impl FoldColumnConfig {
    /// Create with width
    pub const fn with_width(width: c_int) -> Self {
        Self {
            width,
            max_level: 20,
            show_numbers: false,
            fill_char: b' ',
        }
    }

    /// Check if fold column is visible
    pub const fn is_visible(&self) -> bool {
        self.width > 0
    }

    /// Clamp level to displayable range
    pub const fn clamp_level(&self, level: c_int) -> c_int {
        if level < 1 {
            1
        } else if level > self.max_level {
            self.max_level
        } else {
            level
        }
    }
}

// =============================================================================
// Visual Range
// =============================================================================

/// A visual range in the display (for highlighting)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldVisualRange {
    /// Start column (0-based)
    pub start_col: c_int,
    /// End column (0-based, exclusive)
    pub end_col: c_int,
    /// Highlight group ID
    pub hl_id: c_int,
}

impl FoldVisualRange {
    /// Create a new visual range
    pub const fn new(start: c_int, end: c_int, hl_id: c_int) -> Self {
        Self {
            start_col: start,
            end_col: end,
            hl_id,
        }
    }

    /// Check if range is valid
    pub const fn is_valid(&self) -> bool {
        self.end_col > self.start_col
    }

    /// Get width of range
    pub const fn width(&self) -> c_int {
        self.end_col - self.start_col
    }
}

// =============================================================================
// Fold Highlight Info
// =============================================================================

/// Highlight information for fold display
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FoldHighlight {
    /// Highlight ID for fold text
    pub text_hl: c_int,
    /// Highlight ID for fold column
    pub column_hl: c_int,
    /// Highlight ID for fold sign
    pub sign_hl: c_int,
}

// =============================================================================
// foldtext_cleanup implementation
// =============================================================================

/// Check if a character byte is ASCII whitespace (space or tab).
#[inline]
const fn is_ascii_white(c: c_char) -> bool {
    c == b' ' as c_char || c == b'\t' as c_char
}

/// Check if a character byte is an ASCII digit.
#[inline]
const fn is_ascii_digit(c: c_char) -> bool {
    c >= b'0' as c_char && c <= b'9' as c_char
}

/// Compare `n` bytes starting at `s1` and `s2`.
/// Returns true if they are equal.
#[inline]
unsafe fn strncmp_bytes(s1: *const c_char, s2: *const c_char, n: usize) -> bool {
    for i in 0..n {
        if *s1.add(i) != *s2.add(i) {
            return false;
        }
        if *s1.add(i) == 0 {
            return true;
        }
    }
    true
}

/// Count bytes in a NUL-terminated string.
unsafe fn cstr_len(s: *const c_char) -> usize {
    let mut p = s;
    while *p != 0 {
        p = p.add(1);
    }
    #[allow(clippy::cast_sign_loss)]
    let len = p.offset_from(s) as usize;
    len
}

/// Find the offset of `%s` within the first `len` bytes of `s`.
/// Returns `Some(offset)` if found, `None` otherwise.
unsafe fn find_percent_s(s: *const c_char, len: usize) -> Option<usize> {
    if len < 2 {
        return None;
    }
    (0..=(len - 2)).find(|&i| *s.add(i) == b'%' as c_char && *s.add(i + 1) == b's' as c_char)
}

/// Remove 'foldmarker' and 'commentstring' from `str` (in-place).
///
/// This is the Rust reimplementation of the C `foldtext_cleanup` function.
/// The string `str` is modified in-place: fold markers and the commentstring
/// wrapping them are removed.
///
/// # Safety
/// `str` must be a valid, NUL-terminated, mutable C string pointer.
/// The string may be modified in-place.
#[allow(clippy::too_many_lines)]
unsafe fn foldtext_cleanup_impl(s: *mut c_char) {
    if s.is_null() {
        return;
    }

    // Get curbuf's commentstring option and skip leading whitespace.
    let cms_raw = nvim_get_curbuf_b_p_cms();
    if cms_raw.is_null() {
        return;
    }
    let cms_start = nvim_skipwhite(cms_raw);

    // Compute strlen(cms_start), then trim trailing whitespace.
    let mut cms_slen = cstr_len(cms_start);
    while cms_slen > 0 && is_ascii_white(*(cms_start.add(cms_slen - 1))) {
        cms_slen -= 1;
    }

    // Locate "%s" in commentstring; split into start and end parts.
    let cms_end_ptr: *const c_char;
    let mut cms_end_len: usize = 0;
    if let Some(offset) = find_percent_s(cms_start, cms_slen) {
        let raw_cms_end = cms_start.add(offset);
        cms_end_len = cms_slen - offset;
        // exclude white space before "%s"
        let mut new_slen = offset;
        while new_slen > 0 && is_ascii_white(*(cms_start.add(new_slen - 1))) {
            new_slen -= 1;
        }
        cms_slen = new_slen;
        // skip "%s" and white space after it
        let after_pct_s = raw_cms_end.add(2);
        let cms_end_skip = nvim_skipwhite(after_pct_s);
        #[allow(clippy::cast_sign_loss)]
        let skip_count = cms_end_skip.offset_from(raw_cms_end) as usize;
        cms_end_len -= skip_count;
        cms_end_ptr = cms_end_skip;
    } else {
        cms_end_ptr = ptr::null();
    }

    // Parse fold markers for curwin
    let wp = nvim_get_curwin();
    let marker_info = parse_marker_impl(wp);

    let mut did1 = false;
    let mut did2 = false;

    let mut cur = s;
    while *cur != 0 {
        // Determine if current position is a fold marker (start or end).
        let marker_match_len: usize = if !marker_info.start_marker.is_null()
            && marker_info.start_marker_len > 0
            && strncmp_bytes(cur, marker_info.start_marker, marker_info.start_marker_len)
        {
            marker_info.start_marker_len
        } else if !marker_info.end_marker.is_null()
            && marker_info.end_marker_len > 0
            && strncmp_bytes(cur, marker_info.end_marker, marker_info.end_marker_len)
        {
            marker_info.end_marker_len
        } else {
            0
        };
        let mut len = marker_match_len;

        if len > 0 {
            // Found a fold marker; if followed by a digit, include it
            if is_ascii_digit(*(cur.add(len))) {
                len += 1;
            }

            // May remove 'commentstring' start before the marker.
            // Walk backwards past whitespace to find potential cms_start.
            if cms_slen > 0 {
                let mut p = cur;
                while p > s && is_ascii_white(*(p.sub(1))) {
                    p = p.sub(1);
                }
                #[allow(clippy::cast_sign_loss)]
                let back = p.offset_from(s) as usize;
                if back >= cms_slen && strncmp_bytes(p.sub(cms_slen), cms_start, cms_slen) {
                    // Include the whitespace and cms_start in the removal
                    #[allow(clippy::cast_sign_loss)]
                    let extra = cur.offset_from(p.sub(cms_slen)) as usize;
                    len += extra;
                    cur = p.sub(cms_slen);
                }
            }
        } else if !cms_end_ptr.is_null() {
            // No marker found; check for commentstring parts
            if !did1 && cms_slen > 0 && strncmp_bytes(cur, cms_start, cms_slen) {
                len = cms_slen;
                did1 = true;
            } else if !did2 && cms_end_len > 0 && strncmp_bytes(cur, cms_end_ptr, cms_end_len) {
                len = cms_end_len;
                did2 = true;
            }
        }

        if len != 0 {
            // Skip trailing whitespace after the removed region
            while is_ascii_white(*(cur.add(len))) {
                len += 1;
            }
            // STRMOVE(cur, cur + len)
            let src = cur.add(len);
            let src_len = cstr_len(src);
            ptr::copy(src, cur, src_len + 1);
        } else {
            // Advance past current character (MB_PTR_ADV)
            let remaining = cstr_len(cur);
            if remaining == 0 {
                break;
            }
            let slice = std::slice::from_raw_parts(cur as *const u8, remaining + 1);
            let advance = utfc_ptr2len(slice).max(1);
            cur = cur.add(advance);
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Remove 'foldmarker' and 'commentstring' from `str` (in-place).
///
/// # Safety
/// `str` must be a valid, NUL-terminated, mutable C string pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_foldtext_cleanup(str: *mut c_char) {
    foldtext_cleanup_impl(str);
}

/// FFI export: Get fold column character
#[no_mangle]
pub extern "C" fn rs_fold_column_char(char_type: FoldColumnChar) -> u8 {
    char_type.as_char()
}

/// FFI export: Check if fold column char is clickable
#[no_mangle]
pub extern "C" fn rs_fold_column_is_clickable(char_type: c_int) -> c_int {
    let char_type = match char_type {
        1 => FoldColumnChar::ClosedStart,
        2 => FoldColumnChar::OpenStart,
        _ => FoldColumnChar::None,
    };
    c_int::from(char_type.is_clickable())
}

/// FFI export: Create fold text components
#[no_mangle]
pub extern "C" fn rs_fold_text_components(line_count: LinenrT, level: c_int) -> FoldTextComponents {
    FoldTextComponents::new(line_count, level)
}

/// FFI export: Calculate fold percent of buffer
#[no_mangle]
pub extern "C" fn rs_fold_percent_of_buffer(line_count: LinenrT, total_lines: LinenrT) -> c_int {
    let components = FoldTextComponents::new(line_count, 1);
    c_int::from(components.percent_of_buffer(total_lines))
}

/// FFI export: Check if fold column is visible
#[no_mangle]
pub extern "C" fn rs_fold_column_is_visible(width: c_int) -> c_int {
    c_int::from(width > 0)
}

/// FFI export: Clamp fold level
#[no_mangle]
pub extern "C" fn rs_fold_clamp_level(level: c_int, max_level: c_int) -> c_int {
    let config = FoldColumnConfig {
        max_level,
        ..Default::default()
    };
    config.clamp_level(level)
}

/// FFI export: Create fold display info
#[no_mangle]
pub extern "C" fn rs_fold_display_info(
    first: LinenrT,
    last: LinenrT,
    level: c_int,
    closed: c_int,
) -> FoldDisplayInfo {
    if closed != 0 {
        FoldDisplayInfo::closed(first, last, level)
    } else {
        FoldDisplayInfo::open(first, last, level)
    }
}

/// FFI export: Check if display info is valid
#[no_mangle]
pub extern "C" fn rs_fold_display_is_valid(info: *const FoldDisplayInfo) -> c_int {
    if info.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*info).is_valid() })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fold_column_char() {
        assert_eq!(FoldColumnChar::None.as_char(), b' ');
        assert_eq!(FoldColumnChar::ClosedStart.as_char(), b'+');
        assert_eq!(FoldColumnChar::OpenStart.as_char(), b'-');
        assert_eq!(FoldColumnChar::Inside.as_char(), b'|');

        assert!(FoldColumnChar::ClosedStart.is_start());
        assert!(FoldColumnChar::OpenStart.is_start());
        assert!(!FoldColumnChar::Inside.is_start());

        assert!(FoldColumnChar::ClosedStart.is_clickable());
        assert!(!FoldColumnChar::Inside.is_clickable());
    }

    #[test]
    fn test_fold_display_info() {
        let closed = FoldDisplayInfo::closed(10, 20, 1);
        assert!(closed.is_valid());
        assert_eq!(closed.line_count, 11);
        assert!(closed.closed);

        let open = FoldDisplayInfo::open(5, 15, 2);
        assert!(open.is_valid());
        assert!(!open.closed);

        let invalid = FoldDisplayInfo::default();
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_fold_text_components() {
        let components = FoldTextComponents::new(100, 2);
        assert_eq!(components.line_count, 100);
        assert_eq!(components.level, 2);
        assert_eq!(components.dash_count, 1);

        assert_eq!(components.percent_of_buffer(1000), 10);
        assert_eq!(components.percent_of_buffer(100), 100);
        assert_eq!(components.percent_of_buffer(0), 0);
    }

    #[test]
    fn test_fold_column_config() {
        let config = FoldColumnConfig::default();
        assert!(!config.is_visible());

        let config = FoldColumnConfig::with_width(4);
        assert!(config.is_visible());
        assert_eq!(config.clamp_level(0), 1);
        assert_eq!(config.clamp_level(5), 5);
        assert_eq!(config.clamp_level(100), 20);
    }

    #[test]
    fn test_fold_visual_range() {
        let range = FoldVisualRange::new(0, 10, 1);
        assert!(range.is_valid());
        assert_eq!(range.width(), 10);

        let empty = FoldVisualRange::new(5, 5, 1);
        assert!(!empty.is_valid());
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_fold_column_char(FoldColumnChar::ClosedStart), b'+');
        assert_eq!(rs_fold_column_is_clickable(1), 1);
        assert_eq!(rs_fold_column_is_clickable(3), 0);

        assert_eq!(rs_fold_percent_of_buffer(50, 100), 50);
        assert_eq!(rs_fold_clamp_level(25, 20), 20);
    }
}
