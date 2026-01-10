//! Put/paste operations (p, P)
//!
//! This module implements calculation logic for the `p` and `P` operators
//! (put/paste text from register).

use std::ffi::c_int;

use crate::types::MotionType;

// =============================================================================
// Put Direction
// =============================================================================

/// Direction for put operation.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PutDirection {
    /// Put before cursor (P command)
    #[default]
    Before = 0,
    /// Put after cursor (p command)
    After = 1,
}

impl PutDirection {
    /// Create from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::After,
            _ => Self::Before,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if putting after cursor.
    #[must_use]
    pub const fn is_after(self) -> bool {
        matches!(self, Self::After)
    }
}

// =============================================================================
// Put Flags
// =============================================================================

/// Flags for put operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PutFlags {
    /// Put from register (vs from clipboard)
    pub from_reg: bool,
    /// Adjust cursor after put
    pub adjust_cursor: bool,
    /// In visual mode
    pub visual: bool,
    /// Silent (no messages)
    pub silent: bool,
    /// Escape special characters
    pub escape: bool,
}

impl PutFlags {
    /// Create with default values.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            from_reg: true,
            adjust_cursor: true,
            visual: false,
            silent: false,
            escape: false,
        }
    }

    /// Create flags from raw bits.
    #[must_use]
    pub const fn from_raw(bits: c_int) -> Self {
        Self {
            from_reg: (bits & 0x01) != 0,
            adjust_cursor: (bits & 0x02) != 0,
            visual: (bits & 0x04) != 0,
            silent: (bits & 0x08) != 0,
            escape: (bits & 0x10) != 0,
        }
    }

    /// Convert to raw bits.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        let mut bits = 0;
        if self.from_reg {
            bits |= 0x01;
        }
        if self.adjust_cursor {
            bits |= 0x02;
        }
        if self.visual {
            bits |= 0x04;
        }
        if self.silent {
            bits |= 0x08;
        }
        if self.escape {
            bits |= 0x10;
        }
        bits
    }
}

// =============================================================================
// Put Position Calculations
// =============================================================================

/// Calculate cursor position adjustment for linewise put.
///
/// For linewise put, the cursor moves to the first non-blank of the put line.
///
/// # Arguments
/// * `dir` - Put direction (before/after)
/// * `cur_lnum` - Current line number
/// * `line_count` - Number of lines being put
///
/// # Returns
/// New line number for cursor
#[must_use]
#[inline]
pub const fn calc_linewise_put_lnum(
    dir: PutDirection,
    cur_lnum: c_int,
    _line_count: c_int,
) -> c_int {
    match dir {
        PutDirection::Before => cur_lnum,
        PutDirection::After => cur_lnum + 1,
    }
}

/// Calculate the target line for linewise put with count.
///
/// # Arguments
/// * `dir` - Put direction
/// * `cur_lnum` - Current line number
/// * `count` - Repeat count
/// * `line_count` - Lines in register
///
/// # Returns
/// Starting line number for put
#[must_use]
#[inline]
pub const fn calc_put_target_line(
    dir: PutDirection,
    cur_lnum: c_int,
    count: c_int,
    _line_count: c_int,
) -> c_int {
    if dir.is_after() {
        cur_lnum + 1
    } else if count > 0 {
        // Before current line
        cur_lnum
    } else {
        cur_lnum
    }
}

/// Calculate cursor column for charwise put.
///
/// # Arguments
/// * `dir` - Put direction
/// * `cur_col` - Current column
/// * `at_eol` - Whether cursor is at end of line
///
/// # Returns
/// Column for put start
#[must_use]
#[inline]
pub const fn calc_charwise_put_col(dir: PutDirection, cur_col: c_int, at_eol: bool) -> c_int {
    if dir.is_after() && !at_eol {
        cur_col + 1
    } else {
        cur_col
    }
}

/// Check if put should replace visual selection.
///
/// # Arguments
/// * `is_visual` - Whether in visual mode
/// * `reg_type` - Register motion type
///
/// # Returns
/// true if visual selection should be deleted first
#[must_use]
#[inline]
pub const fn should_replace_visual(is_visual: bool, _reg_type: MotionType) -> bool {
    is_visual
}

/// Calculate number of lines affected by put with count.
///
/// # Arguments
/// * `reg_lines` - Lines in register
/// * `count` - Repeat count
///
/// # Returns
/// Total lines to be inserted
#[must_use]
#[inline]
pub const fn calc_put_line_count(reg_lines: c_int, count: c_int) -> c_int {
    if count <= 0 {
        reg_lines
    } else {
        reg_lines.saturating_mul(count)
    }
}

/// Check if cursor needs adjustment after put.
///
/// For linewise put, cursor goes to first non-blank.
/// For charwise put, cursor goes to end of put text (or start if 'p' option set).
///
/// # Arguments
/// * `reg_type` - Register motion type
/// * `flags` - Put flags
///
/// # Returns
/// true if cursor position needs adjustment
#[must_use]
#[inline]
pub const fn needs_cursor_adjust(reg_type: MotionType, flags: &PutFlags) -> bool {
    flags.adjust_cursor && !matches!(reg_type, MotionType::BlockWise)
}

/// Calculate indentation handling for put.
///
/// # Arguments
/// * `autoindent` - Whether autoindent is on
/// * `reg_type` - Register motion type
///
/// # Returns
/// true if indentation should be adjusted
#[must_use]
#[inline]
pub const fn should_adjust_indent(autoindent: bool, reg_type: MotionType) -> bool {
    autoindent && matches!(reg_type, MotionType::LineWise)
}

// =============================================================================
// Block Put Helpers
// =============================================================================

/// Calculate block width for visual block put.
///
/// # Arguments
/// * `start_vcol` - Start virtual column of block
/// * `end_vcol` - End virtual column of block
///
/// # Returns
/// Block width
#[must_use]
#[inline]
pub const fn calc_block_put_width(start_vcol: c_int, end_vcol: c_int) -> c_int {
    let width = end_vcol - start_vcol;
    if width < 0 {
        0
    } else {
        width
    }
}

/// Check if block put needs padding.
///
/// When putting a block past end of line, spaces are added.
///
/// # Arguments
/// * `cur_col` - Current column
/// * `line_len` - Length of current line
///
/// # Returns
/// Number of spaces to add for padding
#[must_use]
#[inline]
pub const fn calc_block_padding(cur_col: c_int, line_len: c_int) -> c_int {
    if cur_col > line_len {
        cur_col - line_len
    } else {
        0
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get put direction from raw value.
#[no_mangle]
pub extern "C" fn rs_put_direction_from_raw(value: c_int) -> c_int {
    PutDirection::from_raw(value).to_raw()
}

/// FFI: Check if put direction is after.
#[no_mangle]
pub extern "C" fn rs_put_is_after(dir: c_int) -> c_int {
    c_int::from(PutDirection::from_raw(dir).is_after())
}

/// FFI: Calculate linewise put line number.
#[no_mangle]
pub extern "C" fn rs_calc_linewise_put_lnum(
    dir: c_int,
    cur_lnum: c_int,
    line_count: c_int,
) -> c_int {
    calc_linewise_put_lnum(PutDirection::from_raw(dir), cur_lnum, line_count)
}

/// FFI: Calculate charwise put column.
#[no_mangle]
pub extern "C" fn rs_calc_charwise_put_col(dir: c_int, cur_col: c_int, at_eol: c_int) -> c_int {
    calc_charwise_put_col(PutDirection::from_raw(dir), cur_col, at_eol != 0)
}

/// FFI: Check if should replace visual selection.
#[no_mangle]
pub extern "C" fn rs_should_replace_visual(is_visual: c_int, reg_type: c_int) -> c_int {
    c_int::from(should_replace_visual(
        is_visual != 0,
        MotionType::from_raw(reg_type),
    ))
}

/// FFI: Calculate put line count with repeat.
#[no_mangle]
pub extern "C" fn rs_calc_put_line_count(reg_lines: c_int, count: c_int) -> c_int {
    calc_put_line_count(reg_lines, count)
}

/// FFI: Check if cursor needs adjustment.
#[no_mangle]
pub extern "C" fn rs_put_needs_cursor_adjust(reg_type: c_int, flags: c_int) -> c_int {
    let put_flags = PutFlags::from_raw(flags);
    c_int::from(needs_cursor_adjust(
        MotionType::from_raw(reg_type),
        &put_flags,
    ))
}

/// FFI: Check if should adjust indent.
#[no_mangle]
pub extern "C" fn rs_put_should_adjust_indent(autoindent: c_int, reg_type: c_int) -> c_int {
    c_int::from(should_adjust_indent(
        autoindent != 0,
        MotionType::from_raw(reg_type),
    ))
}

/// FFI: Calculate block padding.
#[no_mangle]
pub extern "C" fn rs_calc_block_padding(cur_col: c_int, line_len: c_int) -> c_int {
    calc_block_padding(cur_col, line_len)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_direction() {
        assert!(!PutDirection::Before.is_after());
        assert!(PutDirection::After.is_after());

        assert_eq!(PutDirection::from_raw(0), PutDirection::Before);
        assert_eq!(PutDirection::from_raw(1), PutDirection::After);
        assert_eq!(PutDirection::from_raw(99), PutDirection::Before);
    }

    #[test]
    fn test_put_flags() {
        let flags = PutFlags::new();
        assert!(flags.from_reg);
        assert!(flags.adjust_cursor);
        assert!(!flags.visual);

        let flags = PutFlags::from_raw(0x07);
        assert!(flags.from_reg);
        assert!(flags.adjust_cursor);
        assert!(flags.visual);
        assert!(!flags.silent);
    }

    #[test]
    fn test_calc_linewise_put_lnum() {
        // Put before - same line
        assert_eq!(calc_linewise_put_lnum(PutDirection::Before, 5, 3), 5);

        // Put after - next line
        assert_eq!(calc_linewise_put_lnum(PutDirection::After, 5, 3), 6);
    }

    #[test]
    fn test_calc_charwise_put_col() {
        // Put before - same column
        assert_eq!(calc_charwise_put_col(PutDirection::Before, 5, false), 5);

        // Put after, not at eol - next column
        assert_eq!(calc_charwise_put_col(PutDirection::After, 5, false), 6);

        // Put after, at eol - same column
        assert_eq!(calc_charwise_put_col(PutDirection::After, 5, true), 5);
    }

    #[test]
    fn test_calc_put_line_count() {
        assert_eq!(calc_put_line_count(3, 1), 3);
        assert_eq!(calc_put_line_count(3, 2), 6);
        assert_eq!(calc_put_line_count(3, 0), 3);
    }

    #[test]
    fn test_should_replace_visual() {
        assert!(should_replace_visual(true, MotionType::CharWise));
        assert!(!should_replace_visual(false, MotionType::CharWise));
    }

    #[test]
    fn test_should_adjust_indent() {
        assert!(should_adjust_indent(true, MotionType::LineWise));
        assert!(!should_adjust_indent(false, MotionType::LineWise));
        assert!(!should_adjust_indent(true, MotionType::CharWise));
    }

    #[test]
    fn test_calc_block_padding() {
        assert_eq!(calc_block_padding(5, 10), 0);
        assert_eq!(calc_block_padding(15, 10), 5);
        assert_eq!(calc_block_padding(10, 10), 0);
    }

    #[test]
    fn test_ffi_wrappers() {
        assert_eq!(rs_put_is_after(0), 0);
        assert_eq!(rs_put_is_after(1), 1);
        assert_eq!(rs_calc_linewise_put_lnum(1, 5, 3), 6);
        assert_eq!(rs_calc_charwise_put_col(1, 5, 0), 6);
        assert_eq!(rs_calc_put_line_count(3, 2), 6);
    }
}
