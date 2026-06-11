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
// Phase O2: Additional Put Operation Helpers
// =============================================================================

/// Put flag constants (must match register_defs.h)
pub const PUT_FIXINDENT: c_int = 1; // make indent look nice
pub const PUT_CURSEND: c_int = 2; // leave cursor after end of new text
                                  // PUT_CURSLINE = 4 (not used here)
pub const PUT_LINE: c_int = 8; // put register as lines
pub const PUT_LINE_SPLIT: c_int = 16; // split line for linewise register
pub const PUT_LINE_FORWARD: c_int = 32; // put linewise register below Visual sel.
pub const PUT_BLOCK_INNER: c_int = 64; // in block mode, do not add trailing spaces

/// Forward direction constant
pub const FORWARD: c_int = 1;
/// Backward direction constant matching C `BACKWARD = -1` from `vim_defs.h`.
pub const BACKWARD: c_int = -1;

/// Check if put operation uses the '.' register (insert register).
///
/// # Arguments
/// * `regname` - Register name
///
/// # Returns
/// true if this is the insert register
#[must_use]
#[inline]
pub const fn is_insert_register_put(regname: c_int) -> bool {
    regname == b'.' as c_int
}

/// Check if put operation uses a special register that needs conversion.
///
/// Special registers like '%', '#', ':', '=' need special handling.
///
/// # Arguments
/// * `regname` - Register name
///
/// # Returns
/// true if this is a special register
#[must_use]
#[inline]
pub const fn is_special_put_register(regname: c_int) -> bool {
    matches!(
        regname,
        37  // '%' current file
            | 35 // '#' alternate file
            | 58 // ':' last command
            | 61 // '=' expression
    )
}

/// Get the insert command start character for '.' register put.
///
/// Returns 'c' for non-linewise visual, 'i' for PUT_LINE or before, 'a' for after.
///
/// # Arguments
/// * `non_linewise_vis` - Whether in non-linewise visual mode
/// * `flags` - Put flags
/// * `dir` - Put direction (FORWARD or BACKWARD)
///
/// # Returns
/// The command character ('c', 'i', or 'a')
#[must_use]
#[inline]
pub const fn get_insert_cmd_char(non_linewise_vis: bool, flags: c_int, dir: c_int) -> c_int {
    if non_linewise_vis {
        b'c' as c_int
    } else if (flags & PUT_LINE) != 0 {
        b'i' as c_int
    } else if dir == FORWARD {
        b'a' as c_int
    } else {
        b'i' as c_int
    }
}

/// Check if put needs to split lines (for visual linewise put).
///
/// # Arguments
/// * `flags` - Put flags
///
/// # Returns
/// true if lines need to be split
#[must_use]
#[inline]
pub const fn needs_put_line_split(flags: c_int) -> bool {
    (flags & PUT_LINE_SPLIT) != 0
}

/// Check if put should go to the forward position after visual block.
///
/// # Arguments
/// * `flags` - Put flags
///
/// # Returns
/// true if put should go forward
#[must_use]
#[inline]
pub const fn needs_put_line_forward(flags: c_int) -> bool {
    (flags & PUT_LINE_FORWARD) != 0
}

/// Check if put should be forced to linewise.
///
/// # Arguments
/// * `flags` - Put flags
///
/// # Returns
/// true if put should be linewise
#[must_use]
#[inline]
pub const fn is_forced_linewise_put(flags: c_int) -> bool {
    (flags & PUT_LINE) != 0
}

/// Check if put is in block inner mode (no trailing spaces).
///
/// # Arguments
/// * `flags` - Put flags
///
/// # Returns
/// true if block inner mode
#[must_use]
#[inline]
pub const fn is_block_inner_put(flags: c_int) -> bool {
    (flags & PUT_BLOCK_INNER) != 0
}

/// Check if cursor should be left at end of put text.
///
/// # Arguments
/// * `flags` - Put flags
///
/// # Returns
/// true if cursor should be at end
#[must_use]
#[inline]
pub const fn should_put_cursend(flags: c_int) -> bool {
    (flags & PUT_CURSEND) != 0
}

/// Check if put should fix indentation.
///
/// # Arguments
/// * `flags` - Put flags
///
/// # Returns
/// true if indent should be fixed
#[must_use]
#[inline]
pub const fn should_put_fixindent(flags: c_int) -> bool {
    (flags & PUT_FIXINDENT) != 0
}

/// Calculate the undo save range for block put.
///
/// # Arguments
/// * `cursor_lnum` - Current cursor line
/// * `y_size` - Number of lines in register
/// * `ml_line_count` - Total lines in buffer
///
/// # Returns
/// End line number for u_save
#[must_use]
#[inline]
pub const fn calc_block_put_undo_end(
    cursor_lnum: c_int,
    y_size: c_int,
    ml_line_count: c_int,
) -> c_int {
    let end = cursor_lnum + y_size + 1;
    if end > ml_line_count + 1 {
        ml_line_count + 1
    } else {
        end
    }
}

/// Calculate the line number for linewise put.
///
/// # Arguments
/// * `cursor_lnum` - Current cursor line
/// * `dir` - Put direction
///
/// # Returns
/// Line number for put
#[must_use]
#[inline]
pub const fn calc_linewise_put_line(cursor_lnum: c_int, dir: c_int) -> c_int {
    if dir == FORWARD {
        cursor_lnum + 1
    } else {
        cursor_lnum
    }
}

/// Calculate cursor line adjustment after linewise put.
///
/// # Arguments
/// * `lnum` - Line number where put happens
/// * `dir` - Put direction
///
/// # Returns
/// New cursor line number
#[must_use]
#[inline]
pub const fn calc_cursor_lnum_after_linewise_put(lnum: c_int, dir: c_int) -> c_int {
    if dir == FORWARD {
        lnum - 1
    } else {
        lnum
    }
}

/// Calculate start spaces for block put when line is short.
///
/// # Arguments
/// * `vcol` - Current virtual column
/// * `target_col` - Target column
///
/// # Returns
/// Number of spaces needed
#[must_use]
#[inline]
pub const fn calc_block_put_startspaces(vcol: c_int, target_col: c_int) -> c_int {
    if vcol < target_col {
        target_col - vcol
    } else {
        0
    }
}

/// Calculate end spaces for block put when inside a tab.
///
/// # Arguments
/// * `vcol` - Current virtual column
/// * `target_col` - Target column
///
/// # Returns
/// Number of end spaces
#[must_use]
#[inline]
pub const fn calc_block_put_endspaces(vcol: c_int, target_col: c_int) -> c_int {
    if vcol > target_col {
        vcol - target_col
    } else {
        0
    }
}

/// Calculate trailing spaces for block put line padding.
///
/// # Arguments
/// * `y_width` - Block width
/// * `text_width` - Actual text width
///
/// # Returns
/// Number of trailing spaces (0 if negative)
#[must_use]
#[inline]
pub const fn calc_block_put_trailing_spaces(y_width: c_int, text_width: c_int) -> c_int {
    let spaces = y_width + 1 - text_width;
    if spaces < 0 {
        0
    } else {
        spaces
    }
}

/// Check if block put needs a new line appended.
///
/// # Arguments
/// * `cursor_lnum` - Current cursor line
/// * `ml_line_count` - Total lines in buffer
///
/// # Returns
/// true if a new line needs to be appended
#[must_use]
#[inline]
pub const fn needs_block_put_append(cursor_lnum: c_int, ml_line_count: c_int) -> bool {
    cursor_lnum > ml_line_count
}

/// Calculate total length for a new line in put.
///
/// # Arguments
/// * `bd_textcol` - Block definition text column
/// * `bd_startspaces` - Start spaces
/// * `yanklen` - Length of yanked text
/// * `bd_endspaces` - End spaces
/// * `spaces` - Trailing spaces
/// * `oldlen` - Original line length
/// * `delcount` - Characters to delete
///
/// # Returns
/// Total length of new line
#[must_use]
#[inline]
pub const fn calc_put_newline_len(
    bd_textcol: c_int,
    bd_startspaces: c_int,
    yanklen: c_int,
    bd_endspaces: c_int,
    spaces: c_int,
    oldlen: c_int,
    delcount: c_int,
) -> c_int {
    bd_textcol + bd_startspaces + yanklen + bd_endspaces + spaces + oldlen - bd_textcol - delcount
}

/// Check if line is short for block put (needs padding).
///
/// # Arguments
/// * `vcol` - Current virtual column
/// * `target_col` - Target column
/// * `at_eol` - Whether at end of line
///
/// # Returns
/// true if line is short
#[must_use]
#[inline]
pub const fn is_shortline_for_put(vcol: c_int, target_col: c_int, at_eol: bool) -> bool {
    vcol < target_col || (vcol == target_col && at_eol)
}

// =============================================================================
// Phase O2 FFI Exports
// =============================================================================

/// FFI: Check if put uses insert register.
#[no_mangle]
pub extern "C" fn rs_is_insert_register_put(regname: c_int) -> c_int {
    c_int::from(is_insert_register_put(regname))
}

/// FFI: Check if put uses special register.
#[no_mangle]
pub extern "C" fn rs_is_special_put_register(regname: c_int) -> c_int {
    c_int::from(is_special_put_register(regname))
}

/// FFI: Get insert command character.
#[no_mangle]
pub extern "C" fn rs_get_insert_cmd_char(
    non_linewise_vis: c_int,
    flags: c_int,
    dir: c_int,
) -> c_int {
    get_insert_cmd_char(non_linewise_vis != 0, flags, dir)
}

/// FFI: Check if put needs line split.
#[no_mangle]
pub extern "C" fn rs_needs_put_line_split(flags: c_int) -> c_int {
    c_int::from(needs_put_line_split(flags))
}

/// FFI: Check if put needs line forward.
#[no_mangle]
pub extern "C" fn rs_needs_put_line_forward(flags: c_int) -> c_int {
    c_int::from(needs_put_line_forward(flags))
}

/// FFI: Check if put is forced linewise.
#[no_mangle]
pub extern "C" fn rs_is_forced_linewise_put(flags: c_int) -> c_int {
    c_int::from(is_forced_linewise_put(flags))
}

/// FFI: Check if block inner put.
#[no_mangle]
pub extern "C" fn rs_is_block_inner_put(flags: c_int) -> c_int {
    c_int::from(is_block_inner_put(flags))
}

/// FFI: Check if cursor should be at end.
#[no_mangle]
pub extern "C" fn rs_should_put_cursend(flags: c_int) -> c_int {
    c_int::from(should_put_cursend(flags))
}

/// FFI: Check if indent should be fixed.
#[no_mangle]
pub extern "C" fn rs_should_put_fixindent(flags: c_int) -> c_int {
    c_int::from(should_put_fixindent(flags))
}

/// FFI: Calculate block put undo end.
#[no_mangle]
pub extern "C" fn rs_calc_block_put_undo_end(
    cursor_lnum: c_int,
    y_size: c_int,
    ml_line_count: c_int,
) -> c_int {
    calc_block_put_undo_end(cursor_lnum, y_size, ml_line_count)
}

/// FFI: Calculate linewise put line.
#[no_mangle]
pub extern "C" fn rs_calc_linewise_put_line(cursor_lnum: c_int, dir: c_int) -> c_int {
    calc_linewise_put_line(cursor_lnum, dir)
}

/// FFI: Calculate cursor line after linewise put.
#[no_mangle]
pub extern "C" fn rs_calc_cursor_lnum_after_linewise_put(lnum: c_int, dir: c_int) -> c_int {
    calc_cursor_lnum_after_linewise_put(lnum, dir)
}

/// FFI: Calculate block put start spaces.
#[no_mangle]
pub extern "C" fn rs_calc_block_put_startspaces(vcol: c_int, target_col: c_int) -> c_int {
    calc_block_put_startspaces(vcol, target_col)
}

/// FFI: Calculate block put end spaces.
#[no_mangle]
pub extern "C" fn rs_calc_block_put_endspaces(vcol: c_int, target_col: c_int) -> c_int {
    calc_block_put_endspaces(vcol, target_col)
}

/// FFI: Calculate block put trailing spaces.
#[no_mangle]
pub extern "C" fn rs_calc_block_put_trailing_spaces(y_width: c_int, text_width: c_int) -> c_int {
    calc_block_put_trailing_spaces(y_width, text_width)
}

/// FFI: Check if block put needs append.
#[no_mangle]
pub extern "C" fn rs_needs_block_put_append(cursor_lnum: c_int, ml_line_count: c_int) -> c_int {
    c_int::from(needs_block_put_append(cursor_lnum, ml_line_count))
}

/// FFI: Calculate put new line length.
#[no_mangle]
pub extern "C" fn rs_calc_put_newline_len(
    bd_textcol: c_int,
    bd_startspaces: c_int,
    yanklen: c_int,
    bd_endspaces: c_int,
    spaces: c_int,
    oldlen: c_int,
    delcount: c_int,
) -> c_int {
    calc_put_newline_len(
        bd_textcol,
        bd_startspaces,
        yanklen,
        bd_endspaces,
        spaces,
        oldlen,
        delcount,
    )
}

/// FFI: Check if line is short for put.
#[no_mangle]
pub extern "C" fn rs_is_shortline_for_put(vcol: c_int, target_col: c_int, at_eol: c_int) -> c_int {
    c_int::from(is_shortline_for_put(vcol, target_col, at_eol != 0))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::cast_lossless)]
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

    // =========================================================================
    // Phase O2 Tests
    // =========================================================================

    #[test]
    fn test_is_insert_register_put() {
        assert!(is_insert_register_put(b'.' as c_int));
        assert!(!is_insert_register_put(b'a' as c_int));
        assert!(!is_insert_register_put(0));
    }

    #[test]
    fn test_is_special_put_register() {
        assert!(is_special_put_register(b'%' as c_int));
        assert!(is_special_put_register(b'#' as c_int));
        assert!(is_special_put_register(b':' as c_int));
        assert!(is_special_put_register(b'=' as c_int));
        assert!(!is_special_put_register(b'a' as c_int));
    }

    #[test]
    fn test_get_insert_cmd_char() {
        // Non-linewise visual: 'c'
        assert_eq!(get_insert_cmd_char(true, 0, FORWARD), b'c' as c_int);

        // PUT_LINE: 'i'
        assert_eq!(get_insert_cmd_char(false, PUT_LINE, FORWARD), b'i' as c_int);

        // Forward without PUT_LINE: 'a'
        assert_eq!(get_insert_cmd_char(false, 0, FORWARD), b'a' as c_int);

        // Backward: 'i'
        assert_eq!(get_insert_cmd_char(false, 0, BACKWARD), b'i' as c_int);
    }

    #[test]
    fn test_put_flag_checks() {
        assert!(needs_put_line_split(PUT_LINE_SPLIT));
        assert!(!needs_put_line_split(0));

        assert!(needs_put_line_forward(PUT_LINE_FORWARD));
        assert!(!needs_put_line_forward(0));

        assert!(is_forced_linewise_put(PUT_LINE));
        assert!(!is_forced_linewise_put(0));

        assert!(is_block_inner_put(PUT_BLOCK_INNER));
        assert!(!is_block_inner_put(0));

        assert!(should_put_cursend(PUT_CURSEND));
        assert!(!should_put_cursend(0));

        assert!(should_put_fixindent(PUT_FIXINDENT));
        assert!(!should_put_fixindent(0));
    }

    #[test]
    fn test_calc_block_put_undo_end() {
        // Normal case
        assert_eq!(calc_block_put_undo_end(5, 3, 100), 9);

        // Clamped to line count
        assert_eq!(calc_block_put_undo_end(98, 5, 100), 101);
    }

    #[test]
    fn test_calc_linewise_put_line() {
        assert_eq!(calc_linewise_put_line(10, FORWARD), 11);
        assert_eq!(calc_linewise_put_line(10, BACKWARD), 10);
    }

    #[test]
    fn test_calc_cursor_lnum_after_linewise_put() {
        assert_eq!(calc_cursor_lnum_after_linewise_put(11, FORWARD), 10);
        assert_eq!(calc_cursor_lnum_after_linewise_put(10, BACKWARD), 10);
    }

    #[test]
    fn test_calc_block_put_startspaces() {
        assert_eq!(calc_block_put_startspaces(5, 10), 5);
        assert_eq!(calc_block_put_startspaces(10, 10), 0);
        assert_eq!(calc_block_put_startspaces(15, 10), 0);
    }

    #[test]
    fn test_calc_block_put_endspaces() {
        assert_eq!(calc_block_put_endspaces(15, 10), 5);
        assert_eq!(calc_block_put_endspaces(10, 10), 0);
        assert_eq!(calc_block_put_endspaces(5, 10), 0);
    }

    #[test]
    fn test_calc_block_put_trailing_spaces() {
        assert_eq!(calc_block_put_trailing_spaces(10, 5), 6);
        assert_eq!(calc_block_put_trailing_spaces(10, 10), 1);
        assert_eq!(calc_block_put_trailing_spaces(5, 10), 0);
    }

    #[test]
    fn test_needs_block_put_append() {
        assert!(needs_block_put_append(101, 100));
        assert!(!needs_block_put_append(100, 100));
        assert!(!needs_block_put_append(50, 100));
    }

    #[test]
    fn test_calc_put_newline_len() {
        // bd_textcol=10, startspaces=2, yanklen=5, endspaces=1, spaces=3, oldlen=20, delcount=1
        // = 10 + 2 + 5 + 1 + 3 + 20 - 10 - 1 = 30
        assert_eq!(calc_put_newline_len(10, 2, 5, 1, 3, 20, 1), 30);
    }

    #[test]
    fn test_is_shortline_for_put() {
        assert!(is_shortline_for_put(5, 10, false));
        assert!(is_shortline_for_put(10, 10, true));
        assert!(!is_shortline_for_put(10, 10, false));
        assert!(!is_shortline_for_put(15, 10, false));
    }

    #[test]
    fn test_phase_o2_ffi_wrappers() {
        // rs_is_insert_register_put
        assert_eq!(rs_is_insert_register_put(b'.' as c_int), 1);
        assert_eq!(rs_is_insert_register_put(b'a' as c_int), 0);

        // rs_is_special_put_register
        assert_eq!(rs_is_special_put_register(b'%' as c_int), 1);

        // rs_get_insert_cmd_char
        assert_eq!(rs_get_insert_cmd_char(1, 0, FORWARD), b'c' as c_int);

        // rs_needs_put_line_split
        assert_eq!(rs_needs_put_line_split(PUT_LINE_SPLIT), 1);
        assert_eq!(rs_needs_put_line_split(0), 0);

        // rs_is_forced_linewise_put
        assert_eq!(rs_is_forced_linewise_put(PUT_LINE), 1);

        // rs_is_block_inner_put
        assert_eq!(rs_is_block_inner_put(PUT_BLOCK_INNER), 1);

        // rs_should_put_cursend
        assert_eq!(rs_should_put_cursend(PUT_CURSEND), 1);

        // rs_calc_block_put_undo_end
        assert_eq!(rs_calc_block_put_undo_end(5, 3, 100), 9);

        // rs_calc_linewise_put_line
        assert_eq!(rs_calc_linewise_put_line(10, FORWARD), 11);

        // rs_calc_cursor_lnum_after_linewise_put
        assert_eq!(rs_calc_cursor_lnum_after_linewise_put(11, FORWARD), 10);

        // rs_calc_block_put_startspaces
        assert_eq!(rs_calc_block_put_startspaces(5, 10), 5);

        // rs_calc_block_put_endspaces
        assert_eq!(rs_calc_block_put_endspaces(15, 10), 5);

        // rs_calc_block_put_trailing_spaces
        assert_eq!(rs_calc_block_put_trailing_spaces(10, 5), 6);

        // rs_needs_block_put_append
        assert_eq!(rs_needs_block_put_append(101, 100), 1);

        // rs_calc_put_newline_len
        assert_eq!(rs_calc_put_newline_len(10, 2, 5, 1, 3, 20, 1), 30);

        // rs_is_shortline_for_put
        assert_eq!(rs_is_shortline_for_put(5, 10, 0), 1);
    }
}
