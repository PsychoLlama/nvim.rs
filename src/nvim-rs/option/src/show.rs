//! Option display functionality
//!
//! This module provides Rust implementations for displaying option
//! values via :set, :setlocal, :setglobal commands.

use std::ffi::{c_char, c_int};

use crate::{OptInt, OptScope, OptValType};

// =============================================================================
// Show Mode
// =============================================================================

/// Mode for showing options.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShowMode {
    /// Show current value
    #[default]
    Value = 0,
    /// Show all options
    All = 1,
    /// Show options that differ from default
    Changed = 2,
    /// Show terminal options
    Terminal = 3,
    /// Show one option with question mark
    Query = 4,
}

impl ShowMode {
    /// Create from C integer.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::All,
            2 => Self::Changed,
            3 => Self::Terminal,
            4 => Self::Query,
            _ => Self::Value,
        }
    }

    /// Convert to C integer.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Show Context
// =============================================================================

/// Context for displaying options.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ShowContext {
    /// Display mode
    pub mode: c_int,
    /// Scope to display
    pub scope: c_int,
    /// Include global value
    pub show_global: bool,
    /// Include local value
    pub show_local: bool,
    /// Show defaults
    pub show_default: bool,
    /// Number of columns for display
    pub columns: c_int,
    /// Current row for multi-option display
    pub row: c_int,
}

impl ShowContext {
    /// Create a new show context.
    #[must_use]
    pub const fn new(mode: ShowMode) -> Self {
        Self {
            mode: mode.to_c_int(),
            scope: OptScope::Global as c_int,
            show_global: true,
            show_local: false,
            show_default: false,
            columns: 80,
            row: 0,
        }
    }

    /// Get the display mode.
    #[must_use]
    pub const fn get_mode(&self) -> ShowMode {
        ShowMode::from_c_int(self.mode)
    }

    /// Check if showing all options.
    #[must_use]
    pub const fn is_all(&self) -> bool {
        self.mode == ShowMode::All as c_int
    }

    /// Check if showing changed options.
    #[must_use]
    pub const fn is_changed(&self) -> bool {
        self.mode == ShowMode::Changed as c_int
    }
}

// =============================================================================
// Boolean Display
// =============================================================================

/// Format a boolean option for display.
///
/// Returns:
/// - "  option" for true
/// - "nooption" for false
#[repr(C)]
pub struct BoolDisplay {
    /// Whether to show "no" prefix
    pub show_no: bool,
}

impl BoolDisplay {
    /// Create display info for boolean option.
    #[must_use]
    pub const fn new(value: bool) -> Self {
        Self { show_no: !value }
    }
}

/// FFI: Get boolean display info.
#[no_mangle]
pub extern "C" fn rs_bool_display_show_no(value: c_int) -> c_int {
    c_int::from(!value != 0)
}

// =============================================================================
// Number Display
// =============================================================================

/// Maximum digits for number display.
pub const MAX_NUM_DIGITS: usize = 24;

/// Format flags for number display.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct NumDisplayFlags {
    /// Show in hex format
    pub hex: bool,
    /// Show sign
    pub signed: bool,
    /// Pad with zeros
    pub zero_pad: bool,
    /// Minimum width
    pub min_width: c_int,
}

/// Count digits needed to display a number.
#[must_use]
pub const fn count_digits(mut value: OptInt) -> c_int {
    if value == 0 {
        return 1;
    }

    let mut count = 0;
    if value < 0 {
        count += 1; // For negative sign
        value = -value;
    }

    while value > 0 {
        count += 1;
        value /= 10;
    }

    count
}

/// FFI: Count digits for number.
#[no_mangle]
pub extern "C" fn rs_count_num_digits(value: OptInt) -> c_int {
    count_digits(value)
}

/// Count hex digits needed.
#[must_use]
pub const fn count_hex_digits(mut value: OptInt) -> c_int {
    if value == 0 {
        return 1;
    }

    let mut count = 0;
    while value > 0 {
        count += 1;
        value /= 16;
    }

    count
}

/// FFI: Count hex digits for number.
#[no_mangle]
pub extern "C" fn rs_count_hex_digits(value: OptInt) -> c_int {
    count_hex_digits(value)
}

// =============================================================================
// String Display
// =============================================================================

/// Flags for string option display.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct StrDisplayFlags {
    /// Escape special characters
    pub escape: bool,
    /// Quote the string
    pub quote: bool,
    /// Maximum display length (0 for unlimited)
    pub max_len: c_int,
    /// Truncate with "..." if too long
    pub truncate: bool,
}

/// Calculate display width for a string option.
///
/// # Arguments
/// * `value` - String value
/// * `escape` - Whether to count escaped characters
///
/// # Safety
/// `value` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_str_display_width(value: *const c_char, escape: c_int) -> c_int {
    if value.is_null() {
        return 0;
    }

    let mut width: c_int = 0;
    let mut p = value;

    while *p != 0 {
        let c = *p as u8;

        if escape != 0 && needs_escape(c) {
            width += 2; // Backslash + char
        } else {
            width += 1;
        }

        p = p.add(1);
    }

    width
}

/// Check if character needs escaping in string display.
#[must_use]
const fn needs_escape(c: u8) -> bool {
    matches!(c, b'\\' | b'"' | b' ' | b'|' | b'\t' | b'\n' | b'\r')
}

/// FFI: Check if character needs escape.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub extern "C" fn rs_char_needs_escape(c: c_int) -> c_int {
    c_int::from(needs_escape(c as u8))
}

// =============================================================================
// Column Layout
// =============================================================================

/// Information for multi-column option display.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ColumnLayout {
    /// Number of columns
    pub num_cols: c_int,
    /// Width of each column
    pub col_width: c_int,
    /// Total items
    pub total_items: c_int,
    /// Items per column
    pub items_per_col: c_int,
}

impl ColumnLayout {
    /// Calculate layout for given items and screen width.
    #[must_use]
    pub const fn calculate(total_items: c_int, max_width: c_int, screen_cols: c_int) -> Self {
        if total_items <= 0 || max_width <= 0 || screen_cols <= 0 {
            return Self {
                num_cols: 1,
                col_width: max_width,
                total_items,
                items_per_col: total_items,
            };
        }

        // Calculate number of columns that fit
        let col_width = max_width + 2; // Add spacing
        let num_cols = if screen_cols / col_width > 0 {
            screen_cols / col_width
        } else {
            1
        };

        // Calculate items per column
        let items_per_col = (total_items + num_cols - 1) / num_cols;

        Self {
            num_cols,
            col_width,
            total_items,
            items_per_col,
        }
    }

    /// Get column and row for item index.
    #[must_use]
    pub const fn position(&self, index: c_int) -> (c_int, c_int) {
        if self.items_per_col <= 0 {
            return (0, index);
        }
        let col = index / self.items_per_col;
        let row = index % self.items_per_col;
        (col, row)
    }
}

/// FFI: Calculate column layout.
#[no_mangle]
pub extern "C" fn rs_calc_column_layout(
    total_items: c_int,
    max_width: c_int,
    screen_cols: c_int,
) -> ColumnLayout {
    ColumnLayout::calculate(total_items, max_width, screen_cols)
}

/// FFI: Get column for item index.
///
/// # Safety
/// `layout` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_layout_get_col(layout: *const ColumnLayout, index: c_int) -> c_int {
    if layout.is_null() {
        return 0;
    }
    (*layout).position(index).0
}

/// FFI: Get row for item index.
///
/// # Safety
/// `layout` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_layout_get_row(layout: *const ColumnLayout, index: c_int) -> c_int {
    if layout.is_null() {
        return 0;
    }
    (*layout).position(index).1
}

// =============================================================================
// Display Formatting
// =============================================================================

/// Option display format.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayFormat {
    /// Short format: option=value
    #[default]
    Short = 0,
    /// Long format: option         value
    Long = 1,
    /// Verbose format with explanation
    Verbose = 2,
}

impl DisplayFormat {
    /// Create from C integer.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Long,
            2 => Self::Verbose,
            _ => Self::Short,
        }
    }

    /// Convert to C integer.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

/// Calculate display width for an option line.
///
/// # Arguments
/// * `name_len` - Length of option name
/// * `value_width` - Display width of value
/// * `opt_type` - Option type
/// * `format` - Display format
#[must_use]
pub const fn calc_display_width(
    name_len: c_int,
    value_width: c_int,
    opt_type: OptValType,
    format: DisplayFormat,
) -> c_int {
    match opt_type {
        OptValType::Boolean => {
            // "  name" or "noname"
            name_len + 2
        }
        OptValType::Number | OptValType::String => {
            match format {
                DisplayFormat::Short => {
                    // "name=value"
                    name_len + 1 + value_width
                }
                DisplayFormat::Long => {
                    // "name           value" (padded to column)
                    20 + value_width
                }
                DisplayFormat::Verbose => {
                    // Full line
                    name_len + 3 + value_width
                }
            }
        }
        OptValType::Nil => name_len,
    }
}

/// FFI: Calculate display width.
#[no_mangle]
pub extern "C" fn rs_calc_display_width(
    name_len: c_int,
    value_width: c_int,
    opt_type: c_int,
    format: c_int,
) -> c_int {
    let ot = match opt_type {
        0 => OptValType::Boolean,
        1 => OptValType::Number,
        2 => OptValType::String,
        _ => OptValType::Nil,
    };
    calc_display_width(name_len, value_width, ot, DisplayFormat::from_c_int(format))
}

// =============================================================================
// Show Context FFI
// =============================================================================

/// FFI: Create show context.
#[no_mangle]
pub extern "C" fn rs_show_context_new(mode: c_int) -> ShowContext {
    ShowContext::new(ShowMode::from_c_int(mode))
}

/// FFI: Check if showing all.
///
/// # Safety
/// `ctx` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_show_context_is_all(ctx: *const ShowContext) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    c_int::from((*ctx).is_all())
}

/// FFI: Check if showing changed.
///
/// # Safety
/// `ctx` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_show_context_is_changed(ctx: *const ShowContext) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    c_int::from((*ctx).is_changed())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_mode() {
        assert_eq!(ShowMode::from_c_int(0), ShowMode::Value);
        assert_eq!(ShowMode::from_c_int(1), ShowMode::All);
        assert_eq!(ShowMode::from_c_int(99), ShowMode::Value);
    }

    #[test]
    fn test_show_context() {
        let ctx = ShowContext::new(ShowMode::All);
        assert!(ctx.is_all());
        assert!(!ctx.is_changed());
    }

    #[test]
    fn test_bool_display() {
        let d = BoolDisplay::new(true);
        assert!(!d.show_no);

        let d = BoolDisplay::new(false);
        assert!(d.show_no);
    }

    #[test]
    fn test_count_digits() {
        assert_eq!(count_digits(0), 1);
        assert_eq!(count_digits(1), 1);
        assert_eq!(count_digits(10), 2);
        assert_eq!(count_digits(100), 3);
        assert_eq!(count_digits(-10), 3); // "-10"
    }

    #[test]
    fn test_count_hex_digits() {
        assert_eq!(count_hex_digits(0), 1);
        assert_eq!(count_hex_digits(15), 1);
        assert_eq!(count_hex_digits(16), 2);
        assert_eq!(count_hex_digits(255), 2);
        assert_eq!(count_hex_digits(256), 3);
    }

    #[test]
    fn test_needs_escape() {
        assert!(needs_escape(b'\\'));
        assert!(needs_escape(b'"'));
        assert!(needs_escape(b' '));
        assert!(!needs_escape(b'a'));
        assert!(!needs_escape(b'0'));
    }

    #[test]
    fn test_column_layout() {
        let layout = ColumnLayout::calculate(10, 15, 80);
        assert!(layout.num_cols > 1);
        assert!(layout.col_width > 0);

        let (col, row) = layout.position(0);
        assert_eq!(col, 0);
        assert_eq!(row, 0);
    }

    #[test]
    fn test_display_format() {
        assert_eq!(DisplayFormat::from_c_int(0), DisplayFormat::Short);
        assert_eq!(DisplayFormat::from_c_int(1), DisplayFormat::Long);
    }

    #[test]
    fn test_calc_display_width() {
        // Boolean: "  name" = 2 + name_len
        assert_eq!(calc_display_width(6, 0, OptValType::Boolean, DisplayFormat::Short), 8);

        // Number short: "name=123"
        assert_eq!(calc_display_width(4, 3, OptValType::Number, DisplayFormat::Short), 8);
    }
}
