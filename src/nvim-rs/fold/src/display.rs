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

use std::ffi::c_int;

/// Line number type
type LinenrT = i32;

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
// FFI Exports
// =============================================================================

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
pub extern "C" fn rs_fold_percent_of_buffer(
    line_count: LinenrT,
    total_lines: LinenrT,
) -> c_int {
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
