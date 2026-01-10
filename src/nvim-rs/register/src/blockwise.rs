//! Block-wise register operations for Neovim.
//!
//! This module implements block-wise (visual block) register operations:
//! - Block text padding and alignment
//! - Block width calculations
//! - Block insertion positioning
//! - Block line iteration

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::c_int;

// =============================================================================
// Block Dimensions
// =============================================================================

/// Block dimensions for visual block operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BlockDimensions {
    /// Starting column (0-based)
    pub start_col: c_int,
    /// Ending column (exclusive, 0-based)
    pub end_col: c_int,
    /// Number of lines in the block
    pub line_count: c_int,
    /// Whether columns are virtual (past end of line)
    pub virtual_edit: bool,
}

impl BlockDimensions {
    /// Create new block dimensions.
    pub const fn new(start_col: c_int, end_col: c_int, line_count: c_int) -> Self {
        Self {
            start_col,
            end_col,
            line_count,
            virtual_edit: false,
        }
    }

    /// Get the width of the block.
    pub const fn width(&self) -> c_int {
        if self.end_col > self.start_col {
            self.end_col - self.start_col
        } else {
            0
        }
    }

    /// Check if block is empty.
    pub const fn is_empty(&self) -> bool {
        self.width() == 0 || self.line_count == 0
    }

    /// Check if column is within the block.
    pub const fn contains_col(&self, col: c_int) -> bool {
        col >= self.start_col && col < self.end_col
    }
}

/// FFI export: create block dimensions.
#[no_mangle]
pub extern "C" fn rs_block_dims_new(
    start_col: c_int,
    end_col: c_int,
    line_count: c_int,
) -> BlockDimensions {
    BlockDimensions::new(start_col, end_col, line_count)
}

/// FFI export: get block width.
#[no_mangle]
pub extern "C" fn rs_block_dims_width(dims: *const BlockDimensions) -> c_int {
    if dims.is_null() {
        return 0;
    }
    unsafe { (*dims).width() }
}

// =============================================================================
// Block Padding
// =============================================================================

/// Padding mode for block insertion.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockPadMode {
    /// No padding - use text as-is
    None = 0,
    /// Pad short lines with spaces
    PadSpaces = 1,
    /// Truncate long lines
    Truncate = 2,
    /// Both pad and truncate to exact width
    Exact = 3,
}

impl BlockPadMode {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::None),
            1 => Some(Self::PadSpaces),
            2 => Some(Self::Truncate),
            3 => Some(Self::Exact),
            _ => None,
        }
    }

    /// Check if padding is needed.
    pub const fn needs_padding(self) -> bool {
        matches!(self, Self::PadSpaces | Self::Exact)
    }

    /// Check if truncation is needed.
    pub const fn needs_truncation(self) -> bool {
        matches!(self, Self::Truncate | Self::Exact)
    }
}

/// Calculate how much padding is needed for a line.
///
/// Returns (left_padding, right_padding) based on the line length
/// and target width.
pub fn calculate_padding(
    line_len: c_int,
    target_width: c_int,
    mode: BlockPadMode,
) -> (c_int, c_int) {
    match mode {
        BlockPadMode::None => (0, 0),
        BlockPadMode::PadSpaces | BlockPadMode::Exact => {
            if line_len < target_width {
                (0, target_width - line_len)
            } else {
                (0, 0)
            }
        }
        BlockPadMode::Truncate => (0, 0),
    }
}

/// FFI export: calculate padding.
#[no_mangle]
pub extern "C" fn rs_block_calc_padding(
    line_len: c_int,
    target_width: c_int,
    mode: c_int,
    left_pad: *mut c_int,
    right_pad: *mut c_int,
) {
    let m = BlockPadMode::from_c_int(mode).unwrap_or(BlockPadMode::None);
    let (l, r) = calculate_padding(line_len, target_width, m);
    if !left_pad.is_null() {
        unsafe { *left_pad = l }
    }
    if !right_pad.is_null() {
        unsafe { *right_pad = r }
    }
}

// =============================================================================
// Block Insertion
// =============================================================================

/// Where to insert block text.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockInsertPos {
    /// Insert before column (I)
    Before = 0,
    /// Insert after column (A)
    After = 1,
    /// Replace existing text (R)
    Replace = 2,
}

impl BlockInsertPos {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Before),
            1 => Some(Self::After),
            2 => Some(Self::Replace),
            _ => None,
        }
    }

    /// Calculate actual insertion column.
    pub const fn actual_col(self, block_col: c_int) -> c_int {
        match self {
            Self::Before | Self::Replace => block_col,
            Self::After => block_col + 1,
        }
    }
}

/// Block insertion state.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BlockInsertState {
    /// Current line being processed (0-based)
    pub current_line: c_int,
    /// Total lines to process
    pub total_lines: c_int,
    /// Column for insertion
    pub insert_col: c_int,
    /// Whether we're past end of some lines
    pub needs_padding: bool,
    /// Whether operation is done
    pub done: bool,
}

impl BlockInsertState {
    /// Create new insertion state.
    pub const fn new(total_lines: c_int, insert_col: c_int) -> Self {
        Self {
            current_line: 0,
            total_lines,
            insert_col,
            needs_padding: false,
            done: total_lines == 0,
        }
    }

    /// Check if there are more lines.
    pub const fn has_more(&self) -> bool {
        !self.done && self.current_line < self.total_lines
    }

    /// Advance to next line.
    pub fn advance(&mut self) {
        self.current_line += 1;
        if self.current_line >= self.total_lines {
            self.done = true;
        }
    }
}

/// FFI export: create block insert state.
#[no_mangle]
pub extern "C" fn rs_block_insert_state_new(
    total_lines: c_int,
    insert_col: c_int,
) -> BlockInsertState {
    BlockInsertState::new(total_lines, insert_col)
}

/// FFI export: check if more lines.
#[no_mangle]
pub extern "C" fn rs_block_insert_has_more(state: *const BlockInsertState) -> bool {
    if state.is_null() {
        return false;
    }
    unsafe { (*state).has_more() }
}

/// FFI export: advance to next line.
#[no_mangle]
pub extern "C" fn rs_block_insert_advance(state: *mut BlockInsertState) {
    if !state.is_null() {
        unsafe { (*state).advance() }
    }
}

// =============================================================================
// Block Width Calculation
// =============================================================================

/// Calculate the display width of a block line.
///
/// This handles:
/// - Tab expansion at the given column
/// - Multi-byte character widths
/// - Control characters
pub fn block_line_width(line: &[u8], start_col: c_int, tabstop: c_int) -> c_int {
    let mut width: c_int = 0;
    let mut col = start_col;

    for &c in line {
        let char_width = match c {
            b'\t' => {
                if tabstop <= 0 {
                    1
                } else {
                    tabstop - (col % tabstop)
                }
            }
            0x00..=0x1F | 0x7F => 2, // Control chars ^X
            _ => 1,
        };
        width += char_width;
        col += char_width;
    }

    width
}

/// FFI export: calculate block line width.
#[no_mangle]
pub unsafe extern "C" fn rs_block_line_width(
    line: *const u8,
    len: c_int,
    start_col: c_int,
    tabstop: c_int,
) -> c_int {
    if line.is_null() || len < 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts(line, len as usize);
    block_line_width(slice, start_col, tabstop)
}

/// Calculate the maximum width among multiple lines.
pub fn max_block_width(widths: &[c_int]) -> c_int {
    widths.iter().copied().max().unwrap_or(0)
}

/// FFI export: get max width from array.
#[no_mangle]
pub unsafe extern "C" fn rs_max_block_width(widths: *const c_int, count: c_int) -> c_int {
    if widths.is_null() || count <= 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts(widths, count as usize);
    max_block_width(slice)
}

// =============================================================================
// Block Extraction
// =============================================================================

/// Result of extracting a block from a line.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BlockExtractResult {
    /// Starting byte offset in the line
    pub start_offset: c_int,
    /// Number of bytes to extract
    pub byte_count: c_int,
    /// Characters before block start that need to be replaced with spaces
    pub pre_spaces: c_int,
    /// Characters after block end that were cut (incomplete chars)
    pub post_cut: c_int,
    /// Whether the block extends past end of line
    pub past_eol: bool,
}

impl BlockExtractResult {
    /// Create empty result.
    pub const fn empty() -> Self {
        Self {
            start_offset: 0,
            byte_count: 0,
            pre_spaces: 0,
            post_cut: 0,
            past_eol: false,
        }
    }

    /// Create past-end-of-line result.
    pub const fn past_end() -> Self {
        Self {
            start_offset: 0,
            byte_count: 0,
            pre_spaces: 0,
            post_cut: 0,
            past_eol: true,
        }
    }

    /// Check if result is empty.
    pub const fn is_empty(&self) -> bool {
        self.byte_count == 0 && !self.past_eol
    }
}

/// FFI export: create empty extraction result.
#[no_mangle]
pub extern "C" fn rs_block_extract_empty() -> BlockExtractResult {
    BlockExtractResult::empty()
}

/// FFI export: create past-end extraction result.
#[no_mangle]
pub extern "C" fn rs_block_extract_past_end() -> BlockExtractResult {
    BlockExtractResult::past_end()
}

// =============================================================================
// Block Visual Position
// =============================================================================

/// Visual block position tracking.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BlockVisualPos {
    /// First line of the block (1-based)
    pub start_line: c_int,
    /// Last line of the block (1-based)
    pub end_line: c_int,
    /// Starting column (0-based visual)
    pub start_vcol: c_int,
    /// Ending column (0-based visual)
    pub end_vcol: c_int,
}

impl BlockVisualPos {
    /// Create new visual position.
    pub const fn new(
        start_line: c_int,
        end_line: c_int,
        start_vcol: c_int,
        end_vcol: c_int,
    ) -> Self {
        Self {
            start_line,
            end_line,
            start_vcol,
            end_vcol,
        }
    }

    /// Get number of lines.
    pub const fn line_count(&self) -> c_int {
        if self.end_line >= self.start_line {
            self.end_line - self.start_line + 1
        } else {
            0
        }
    }

    /// Get block width.
    pub const fn width(&self) -> c_int {
        if self.end_vcol >= self.start_vcol {
            self.end_vcol - self.start_vcol + 1
        } else {
            0
        }
    }

    /// Normalize so start <= end.
    pub const fn normalize(self) -> Self {
        let (sl, el) = if self.start_line <= self.end_line {
            (self.start_line, self.end_line)
        } else {
            (self.end_line, self.start_line)
        };
        let (sc, ec) = if self.start_vcol <= self.end_vcol {
            (self.start_vcol, self.end_vcol)
        } else {
            (self.end_vcol, self.start_vcol)
        };
        Self {
            start_line: sl,
            end_line: el,
            start_vcol: sc,
            end_vcol: ec,
        }
    }
}

/// FFI export: create visual position.
#[no_mangle]
pub extern "C" fn rs_block_visual_pos_new(
    start_line: c_int,
    end_line: c_int,
    start_vcol: c_int,
    end_vcol: c_int,
) -> BlockVisualPos {
    BlockVisualPos::new(start_line, end_line, start_vcol, end_vcol)
}

/// FFI export: get line count.
#[no_mangle]
pub extern "C" fn rs_block_visual_line_count(pos: *const BlockVisualPos) -> c_int {
    if pos.is_null() {
        return 0;
    }
    unsafe { (*pos).line_count() }
}

/// FFI export: normalize visual position.
#[no_mangle]
pub extern "C" fn rs_block_visual_normalize(pos: BlockVisualPos) -> BlockVisualPos {
    pos.normalize()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_dimensions() {
        let dims = BlockDimensions::new(5, 15, 10);
        assert_eq!(dims.width(), 10);
        assert!(!dims.is_empty());
        assert!(dims.contains_col(5));
        assert!(dims.contains_col(14));
        assert!(!dims.contains_col(4));
        assert!(!dims.contains_col(15));

        let empty = BlockDimensions::new(0, 0, 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_block_pad_mode() {
        assert_eq!(BlockPadMode::from_c_int(0), Some(BlockPadMode::None));
        assert_eq!(BlockPadMode::from_c_int(3), Some(BlockPadMode::Exact));
        assert_eq!(BlockPadMode::from_c_int(99), None);

        assert!(!BlockPadMode::None.needs_padding());
        assert!(BlockPadMode::PadSpaces.needs_padding());
        assert!(BlockPadMode::Exact.needs_padding());

        assert!(!BlockPadMode::None.needs_truncation());
        assert!(BlockPadMode::Truncate.needs_truncation());
        assert!(BlockPadMode::Exact.needs_truncation());
    }

    #[test]
    fn test_calculate_padding() {
        assert_eq!(calculate_padding(5, 10, BlockPadMode::None), (0, 0));
        assert_eq!(calculate_padding(5, 10, BlockPadMode::PadSpaces), (0, 5));
        assert_eq!(calculate_padding(15, 10, BlockPadMode::PadSpaces), (0, 0));
        assert_eq!(calculate_padding(5, 10, BlockPadMode::Exact), (0, 5));
    }

    #[test]
    fn test_block_insert_pos() {
        assert_eq!(BlockInsertPos::from_c_int(0), Some(BlockInsertPos::Before));
        assert_eq!(BlockInsertPos::from_c_int(1), Some(BlockInsertPos::After));

        assert_eq!(BlockInsertPos::Before.actual_col(5), 5);
        assert_eq!(BlockInsertPos::After.actual_col(5), 6);
        assert_eq!(BlockInsertPos::Replace.actual_col(5), 5);
    }

    #[test]
    fn test_block_insert_state() {
        let mut state = BlockInsertState::new(5, 10);
        assert!(state.has_more());
        assert_eq!(state.current_line, 0);

        state.advance();
        assert!(state.has_more());
        assert_eq!(state.current_line, 1);

        for _ in 0..4 {
            state.advance();
        }
        assert!(!state.has_more());
        assert!(state.done);
    }

    #[test]
    fn test_block_line_width() {
        // Simple text
        assert_eq!(block_line_width(b"hello", 0, 8), 5);

        // Tab expansion
        assert_eq!(block_line_width(b"\t", 0, 8), 8);
        assert_eq!(block_line_width(b"\t", 4, 8), 4);

        // Control chars
        assert_eq!(block_line_width(b"\x01", 0, 8), 2);
    }

    #[test]
    fn test_max_block_width() {
        assert_eq!(max_block_width(&[5, 10, 3, 8]), 10);
        assert_eq!(max_block_width(&[]), 0);
        assert_eq!(max_block_width(&[7]), 7);
    }

    #[test]
    fn test_block_extract_result() {
        let empty = BlockExtractResult::empty();
        assert!(empty.is_empty());

        let past = BlockExtractResult::past_end();
        assert!(!past.is_empty()); // past_eol is true
        assert!(past.past_eol);
    }

    #[test]
    fn test_block_visual_pos() {
        let pos = BlockVisualPos::new(5, 10, 3, 8);
        assert_eq!(pos.line_count(), 6);
        assert_eq!(pos.width(), 6);

        // Test normalization
        let reversed = BlockVisualPos::new(10, 5, 8, 3);
        let normalized = reversed.normalize();
        assert_eq!(normalized.start_line, 5);
        assert_eq!(normalized.end_line, 10);
        assert_eq!(normalized.start_vcol, 3);
        assert_eq!(normalized.end_vcol, 8);
    }
}
