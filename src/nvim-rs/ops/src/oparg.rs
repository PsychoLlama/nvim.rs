//! Operator arguments wrapper
//!
//! This module provides a safe Rust wrapper around the C `oparg_T` struct,
//! using FFI accessor functions to read and write fields.

use std::ffi::c_int;

use crate::types::{MotionType, OpType, Pos};
use nvim_normal::types::OpargT;

/// Typed handle to C's oparg_T struct
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct OpArgHandle(*mut OpargT);

impl OpArgHandle {
    /// Create from raw pointer
    ///
    /// # Safety
    /// The pointer must be a valid oparg_T pointer or null.
    #[inline]
    #[must_use]
    pub const unsafe fn from_raw(ptr: *mut OpargT) -> Self {
        Self(ptr)
    }

    /// Get raw pointer
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *mut OpargT {
        self.0
    }

    /// Check if handle is null
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Safe wrapper for reading oparg_T fields
///
/// This provides read-only access to the operator arguments.
pub struct OpArgRef {
    handle: OpArgHandle,
}

impl OpArgRef {
    /// Create a new OpArgRef from a handle
    ///
    /// # Safety
    /// The handle must be valid and not null.
    #[inline]
    #[must_use]
    pub const unsafe fn from_handle(handle: OpArgHandle) -> Self {
        Self { handle }
    }

    /// Get the operator type
    #[inline]
    #[must_use]
    pub fn op_type(&self) -> OpType {
        let raw = unsafe { (*self.handle.as_ptr()).op_type };
        OpType::from_raw(raw).unwrap_or(OpType::Nop)
    }

    /// Get the register name
    #[inline]
    #[must_use]
    pub fn regname(&self) -> c_int {
        unsafe { (*self.handle.as_ptr()).regname }
    }

    /// Get the motion type
    #[inline]
    #[must_use]
    pub fn motion_type(&self) -> MotionType {
        let raw = unsafe { (*self.handle.as_ptr()).motion_type };
        MotionType::from_raw(raw)
    }

    /// Get the motion force character ('v', 'V', or CTRL-V)
    #[inline]
    #[must_use]
    pub fn motion_force(&self) -> c_int {
        unsafe { (*self.handle.as_ptr()).motion_force }
    }

    /// Check if the motion is inclusive
    #[inline]
    #[must_use]
    pub fn inclusive(&self) -> bool {
        unsafe { (*self.handle.as_ptr()).inclusive }
    }

    /// Check if delete should use reg 1 even when not linewise
    #[inline]
    #[must_use]
    pub fn use_reg_one(&self) -> bool {
        unsafe { (*self.handle.as_ptr()).use_reg_one }
    }

    /// Get the line count
    #[inline]
    #[must_use]
    pub fn line_count(&self) -> c_int {
        unsafe { (*self.handle.as_ptr()).line_count }
    }

    /// Check if op_start and op_end are the same
    #[inline]
    #[must_use]
    pub fn empty(&self) -> bool {
        unsafe { (*self.handle.as_ptr()).empty }
    }

    /// Check if operator is on Visual area
    #[inline]
    #[must_use]
    pub fn is_visual(&self) -> bool {
        unsafe { (*self.handle.as_ptr()).is_visual }
    }

    /// Check if trailing whitespace should be excluded for block yank
    #[inline]
    #[must_use]
    pub fn excl_tr_ws(&self) -> bool {
        unsafe { (*self.handle.as_ptr()).excl_tr_ws }
    }

    /// Get the start position
    #[inline]
    #[must_use]
    pub fn start(&self) -> Pos {
        Pos {
            lnum: unsafe { (*self.handle.as_ptr()).start.lnum },
            col: unsafe { (*self.handle.as_ptr()).start.col },
            coladd: unsafe { (*self.handle.as_ptr()).start.coladd },
        }
    }

    /// Get the end position
    #[inline]
    #[must_use]
    pub fn end(&self) -> Pos {
        Pos {
            lnum: unsafe { (*self.handle.as_ptr()).end.lnum },
            col: unsafe { (*self.handle.as_ptr()).end.col },
            coladd: unsafe { (*self.handle.as_ptr()).end.coladd },
        }
    }

    /// Get the start virtual column (for block mode)
    #[inline]
    #[must_use]
    pub fn start_vcol(&self) -> c_int {
        unsafe { (*self.handle.as_ptr()).start_vcol }
    }

    /// Get the end virtual column (for block mode)
    #[inline]
    #[must_use]
    pub fn end_vcol(&self) -> c_int {
        unsafe { (*self.handle.as_ptr()).end_vcol }
    }

    /// Check if this is a block-wise operation
    #[inline]
    #[must_use]
    pub fn is_block(&self) -> bool {
        self.motion_type() == MotionType::BlockWise
    }

    /// Check if this is a line-wise operation
    #[inline]
    #[must_use]
    pub fn is_linewise(&self) -> bool {
        self.motion_type() == MotionType::LineWise
    }

    /// Check if this is a character-wise operation
    #[inline]
    #[must_use]
    pub fn is_charwise(&self) -> bool {
        self.motion_type() == MotionType::CharWise
    }
}

/// Safe wrapper for reading and writing oparg_T fields
///
/// This provides mutable access to operator arguments.
pub struct OpArgMut {
    handle: OpArgHandle,
}

impl OpArgMut {
    /// Create a new OpArgMut from a handle
    ///
    /// # Safety
    /// The handle must be valid and not null.
    #[inline]
    #[must_use]
    pub const unsafe fn from_handle(handle: OpArgHandle) -> Self {
        Self { handle }
    }

    /// Get read-only access
    #[inline]
    #[must_use]
    pub fn as_ref(&self) -> OpArgRef {
        OpArgRef {
            handle: self.handle,
        }
    }

    /// Set the operator type
    #[inline]
    pub fn set_op_type(&mut self, op_type: OpType) {
        unsafe { (*self.handle.as_ptr()).op_type = op_type.as_raw() }
    }

    /// Set the motion type
    #[inline]
    pub fn set_motion_type(&mut self, motion_type: MotionType) {
        unsafe { (*self.handle.as_ptr()).motion_type = motion_type.as_raw() }
    }

    /// Set inclusive flag
    #[inline]
    pub fn set_inclusive(&mut self, inclusive: bool) {
        unsafe { (*self.handle.as_ptr()).inclusive = inclusive }
    }

    /// Set line count
    #[inline]
    pub fn set_line_count(&mut self, line_count: c_int) {
        unsafe { (*self.handle.as_ptr()).line_count = line_count }
    }

    /// Set empty flag
    #[inline]
    pub fn set_empty(&mut self, empty: bool) {
        unsafe { (*self.handle.as_ptr()).empty = empty }
    }
}

// Forward common getters from OpArgRef
impl OpArgMut {
    /// Get the operator type
    #[inline]
    #[must_use]
    pub fn op_type(&self) -> OpType {
        self.as_ref().op_type()
    }

    /// Get the register name
    #[inline]
    #[must_use]
    pub fn regname(&self) -> c_int {
        self.as_ref().regname()
    }

    /// Get the motion type
    #[inline]
    #[must_use]
    pub fn motion_type(&self) -> MotionType {
        self.as_ref().motion_type()
    }

    /// Get the motion force character
    #[inline]
    #[must_use]
    pub fn motion_force(&self) -> c_int {
        self.as_ref().motion_force()
    }

    /// Check if the motion is inclusive
    #[inline]
    #[must_use]
    pub fn inclusive(&self) -> bool {
        self.as_ref().inclusive()
    }

    /// Get the line count
    #[inline]
    #[must_use]
    pub fn line_count(&self) -> c_int {
        self.as_ref().line_count()
    }

    /// Check if op_start and op_end are the same
    #[inline]
    #[must_use]
    pub fn empty(&self) -> bool {
        self.as_ref().empty()
    }

    /// Check if operator is on Visual area
    #[inline]
    #[must_use]
    pub fn is_visual(&self) -> bool {
        self.as_ref().is_visual()
    }

    /// Get the start position
    #[inline]
    #[must_use]
    pub fn start(&self) -> Pos {
        self.as_ref().start()
    }

    /// Get the end position
    #[inline]
    #[must_use]
    pub fn end(&self) -> Pos {
        self.as_ref().end()
    }

    /// Get the start virtual column
    #[inline]
    #[must_use]
    pub fn start_vcol(&self) -> c_int {
        self.as_ref().start_vcol()
    }

    /// Get the end virtual column
    #[inline]
    #[must_use]
    pub fn end_vcol(&self) -> c_int {
        self.as_ref().end_vcol()
    }

    /// Check if this is a block-wise operation
    #[inline]
    #[must_use]
    pub fn is_block(&self) -> bool {
        self.as_ref().is_block()
    }

    /// Check if this is a line-wise operation
    #[inline]
    #[must_use]
    pub fn is_linewise(&self) -> bool {
        self.as_ref().is_linewise()
    }

    /// Check if this is a character-wise operation
    #[inline]
    #[must_use]
    pub fn is_charwise(&self) -> bool {
        self.as_ref().is_charwise()
    }
}

// =============================================================================
// Phase O5 Block Operations Helpers
// =============================================================================

/// Calculate block width from start and end virtual columns.
#[must_use]
#[inline]
pub const fn calc_block_width(start_vcol: c_int, end_vcol: c_int) -> c_int {
    if end_vcol >= start_vcol {
        end_vcol - start_vcol + 1
    } else {
        0
    }
}

/// Check if block is a single column.
#[must_use]
#[inline]
pub const fn is_single_col_block(start_vcol: c_int, end_vcol: c_int) -> bool {
    start_vcol == end_vcol
}

/// Calculate number of lines in block operation.
#[must_use]
#[inline]
pub const fn calc_block_line_count(start_lnum: c_int, end_lnum: c_int) -> c_int {
    if end_lnum >= start_lnum {
        end_lnum - start_lnum + 1
    } else {
        0
    }
}

/// Calculate block start column from virtual column.
#[must_use]
#[inline]
pub const fn calc_block_start_col(start_vcol: c_int, coladd: c_int) -> c_int {
    start_vcol + coladd
}

/// Check if block operation spans multiple lines.
#[must_use]
#[inline]
pub const fn is_multiline_block(start_lnum: c_int, end_lnum: c_int) -> bool {
    end_lnum > start_lnum
}

/// Calculate the textcol for block operations (where text starts).
#[must_use]
#[inline]
pub const fn calc_block_textcol(start_col: c_int, startspaces: c_int) -> c_int {
    start_col + startspaces
}

/// Calculate the number of replacement characters for block mode.
#[must_use]
#[inline]
pub const fn calc_block_replacement_chars(block_width: c_int, char_width: c_int) -> c_int {
    if char_width <= 0 {
        0
    } else {
        block_width / char_width
    }
}

/// Check if block extends past end of line.
#[must_use]
#[inline]
pub const fn is_block_past_eol(end_vcol: c_int, line_len_vcol: c_int) -> bool {
    end_vcol > line_len_vcol
}

/// Calculate how much block extends past end of line.
#[must_use]
#[inline]
pub const fn calc_block_past_eol_amount(end_vcol: c_int, line_len_vcol: c_int) -> c_int {
    if end_vcol > line_len_vcol {
        end_vcol - line_len_vcol
    } else {
        0
    }
}

/// Check if block operation should process virtual spaces.
#[must_use]
#[inline]
pub const fn should_process_virtual_spaces(virtual_edit: bool, has_coladd: bool) -> bool {
    virtual_edit && has_coladd
}

/// Calculate effective block end column.
#[must_use]
#[inline]
pub const fn calc_effective_block_end(end_vcol: c_int, line_len_vcol: c_int) -> c_int {
    if end_vcol > line_len_vcol {
        line_len_vcol
    } else {
        end_vcol
    }
}

/// Calculate block operation's virtual column span.
#[must_use]
#[inline]
pub const fn calc_block_vcol_span(start_vcol: c_int, end_vcol: c_int, inclusive: bool) -> c_int {
    let inclusive_adj = if inclusive { 0 } else { 1 };
    let span = end_vcol - start_vcol + 1 - inclusive_adj;
    if span < 0 {
        0
    } else {
        span
    }
}

/// Check if line is short for block operation.
#[must_use]
#[inline]
pub const fn is_line_short_for_block(line_len: c_int, block_start: c_int) -> bool {
    line_len < block_start
}

/// Calculate number of spaces needed to fill block on short line.
#[must_use]
#[inline]
pub const fn calc_short_line_fill_spaces(
    block_start: c_int,
    block_end: c_int,
    line_len: c_int,
) -> c_int {
    if line_len >= block_end {
        0
    } else if line_len <= block_start {
        block_end - block_start
    } else {
        block_end - line_len
    }
}

/// Calculate cursor column after block operation.
#[must_use]
#[inline]
pub const fn calc_block_cursor_col(textcol: c_int, startspaces: c_int) -> c_int {
    textcol + startspaces
}

/// Check if block operation has startspaces.
#[must_use]
#[inline]
pub const fn has_block_startspaces(startspaces: c_int) -> bool {
    startspaces > 0
}

/// Check if block operation has endspaces.
#[must_use]
#[inline]
pub const fn has_block_endspaces(endspaces: c_int) -> bool {
    endspaces > 0
}

/// Calculate total spaces in block operation.
#[must_use]
#[inline]
pub const fn calc_block_total_spaces(startspaces: c_int, endspaces: c_int) -> c_int {
    startspaces + endspaces
}

/// Check if block includes partial character at start.
#[must_use]
#[inline]
pub const fn has_partial_start_char(startspaces: c_int) -> bool {
    startspaces > 0
}

/// Check if block includes partial character at end.
#[must_use]
#[inline]
pub const fn has_partial_end_char(endspaces: c_int) -> bool {
    endspaces > 0
}

/// Calculate adjusted block width accounting for partial chars.
#[must_use]
#[inline]
pub const fn calc_adjusted_block_width(
    textlen: c_int,
    startspaces: c_int,
    endspaces: c_int,
) -> c_int {
    textlen + startspaces + endspaces
}

/// Calculate number of characters in block (not including spaces).
#[must_use]
#[inline]
pub const fn calc_block_char_count(textlen: c_int) -> c_int {
    if textlen < 0 {
        0
    } else {
        textlen
    }
}

// =============================================================================
// Phase O5 FFI Wrappers
// =============================================================================

/// FFI: Calculate block width.
#[no_mangle]
pub extern "C" fn rs_calc_block_width(start_vcol: c_int, end_vcol: c_int) -> c_int {
    calc_block_width(start_vcol, end_vcol)
}

/// FFI: Check if single column block.
#[no_mangle]
pub extern "C" fn rs_is_single_col_block(start_vcol: c_int, end_vcol: c_int) -> c_int {
    c_int::from(is_single_col_block(start_vcol, end_vcol))
}

/// FFI: Calculate block line count.
#[no_mangle]
pub extern "C" fn rs_calc_block_line_count(start_lnum: c_int, end_lnum: c_int) -> c_int {
    calc_block_line_count(start_lnum, end_lnum)
}

/// FFI: Calculate block start column.
#[no_mangle]
pub extern "C" fn rs_calc_block_start_col(start_vcol: c_int, coladd: c_int) -> c_int {
    calc_block_start_col(start_vcol, coladd)
}

/// FFI: Check if multiline block.
#[no_mangle]
pub extern "C" fn rs_is_multiline_block(start_lnum: c_int, end_lnum: c_int) -> c_int {
    c_int::from(is_multiline_block(start_lnum, end_lnum))
}

/// FFI: Calculate block textcol.
#[no_mangle]
pub extern "C" fn rs_calc_block_textcol(start_col: c_int, startspaces: c_int) -> c_int {
    calc_block_textcol(start_col, startspaces)
}

/// FFI: Calculate block replacement chars.
#[no_mangle]
pub extern "C" fn rs_calc_block_replacement_chars(block_width: c_int, char_width: c_int) -> c_int {
    calc_block_replacement_chars(block_width, char_width)
}

/// FFI: Check if block past EOL.
#[no_mangle]
pub extern "C" fn rs_is_block_past_eol(end_vcol: c_int, line_len_vcol: c_int) -> c_int {
    c_int::from(is_block_past_eol(end_vcol, line_len_vcol))
}

/// FFI: Calculate amount past EOL.
#[no_mangle]
pub extern "C" fn rs_calc_block_past_eol_amount(end_vcol: c_int, line_len_vcol: c_int) -> c_int {
    calc_block_past_eol_amount(end_vcol, line_len_vcol)
}

/// FFI: Check if should process virtual spaces.
#[no_mangle]
pub extern "C" fn rs_should_process_virtual_spaces(
    virtual_edit: c_int,
    has_coladd: c_int,
) -> c_int {
    c_int::from(should_process_virtual_spaces(
        virtual_edit != 0,
        has_coladd != 0,
    ))
}

/// FFI: Calculate effective block end.
#[no_mangle]
pub extern "C" fn rs_calc_effective_block_end(end_vcol: c_int, line_len_vcol: c_int) -> c_int {
    calc_effective_block_end(end_vcol, line_len_vcol)
}

/// FFI: Calculate block vcol span.
#[no_mangle]
pub extern "C" fn rs_calc_block_vcol_span(
    start_vcol: c_int,
    end_vcol: c_int,
    inclusive: c_int,
) -> c_int {
    calc_block_vcol_span(start_vcol, end_vcol, inclusive != 0)
}

/// FFI: Check if line short for block.
#[no_mangle]
pub extern "C" fn rs_is_line_short_for_block(line_len: c_int, block_start: c_int) -> c_int {
    c_int::from(is_line_short_for_block(line_len, block_start))
}

/// FFI: Calculate short line fill spaces.
#[no_mangle]
pub extern "C" fn rs_calc_short_line_fill_spaces(
    block_start: c_int,
    block_end: c_int,
    line_len: c_int,
) -> c_int {
    calc_short_line_fill_spaces(block_start, block_end, line_len)
}

/// FFI: Calculate block cursor col.
#[no_mangle]
pub extern "C" fn rs_calc_block_cursor_col(textcol: c_int, startspaces: c_int) -> c_int {
    calc_block_cursor_col(textcol, startspaces)
}

/// FFI: Check if has block startspaces.
#[no_mangle]
pub extern "C" fn rs_has_block_startspaces(startspaces: c_int) -> c_int {
    c_int::from(has_block_startspaces(startspaces))
}

/// FFI: Check if has block endspaces.
#[no_mangle]
pub extern "C" fn rs_has_block_endspaces(endspaces: c_int) -> c_int {
    c_int::from(has_block_endspaces(endspaces))
}

/// FFI: Calculate block total spaces.
#[no_mangle]
pub extern "C" fn rs_calc_block_total_spaces(startspaces: c_int, endspaces: c_int) -> c_int {
    calc_block_total_spaces(startspaces, endspaces)
}

/// FFI: Check has partial start char.
#[no_mangle]
pub extern "C" fn rs_has_partial_start_char(startspaces: c_int) -> c_int {
    c_int::from(has_partial_start_char(startspaces))
}

/// FFI: Check has partial end char.
#[no_mangle]
pub extern "C" fn rs_has_partial_end_char(endspaces: c_int) -> c_int {
    c_int::from(has_partial_end_char(endspaces))
}

/// FFI: Calculate adjusted block width.
#[no_mangle]
pub extern "C" fn rs_calc_adjusted_block_width(
    textlen: c_int,
    startspaces: c_int,
    endspaces: c_int,
) -> c_int {
    calc_adjusted_block_width(textlen, startspaces, endspaces)
}

/// FFI: Calculate block char count.
#[no_mangle]
pub extern "C" fn rs_calc_block_char_count(textlen: c_int) -> c_int {
    calc_block_char_count(textlen)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_block_width() {
        assert_eq!(calc_block_width(0, 9), 10);
        assert_eq!(calc_block_width(5, 5), 1);
        assert_eq!(calc_block_width(10, 5), 0);
    }

    #[test]
    fn test_is_single_col_block() {
        assert!(is_single_col_block(5, 5));
        assert!(!is_single_col_block(5, 10));
    }

    #[test]
    fn test_calc_block_line_count() {
        assert_eq!(calc_block_line_count(1, 10), 10);
        assert_eq!(calc_block_line_count(5, 5), 1);
        assert_eq!(calc_block_line_count(10, 5), 0);
    }

    #[test]
    fn test_calc_block_start_col() {
        assert_eq!(calc_block_start_col(10, 5), 15);
        assert_eq!(calc_block_start_col(10, 0), 10);
    }

    #[test]
    fn test_is_multiline_block() {
        assert!(is_multiline_block(1, 10));
        assert!(!is_multiline_block(5, 5));
    }

    #[test]
    fn test_calc_block_textcol() {
        assert_eq!(calc_block_textcol(10, 5), 15);
        assert_eq!(calc_block_textcol(10, 0), 10);
    }

    #[test]
    fn test_calc_block_replacement_chars() {
        assert_eq!(calc_block_replacement_chars(10, 2), 5);
        assert_eq!(calc_block_replacement_chars(10, 1), 10);
        assert_eq!(calc_block_replacement_chars(10, 0), 0);
    }

    #[test]
    fn test_is_block_past_eol() {
        assert!(is_block_past_eol(100, 80));
        assert!(!is_block_past_eol(50, 80));
        assert!(!is_block_past_eol(80, 80));
    }

    #[test]
    fn test_calc_block_past_eol_amount() {
        assert_eq!(calc_block_past_eol_amount(100, 80), 20);
        assert_eq!(calc_block_past_eol_amount(50, 80), 0);
    }

    #[test]
    fn test_should_process_virtual_spaces() {
        assert!(should_process_virtual_spaces(true, true));
        assert!(!should_process_virtual_spaces(true, false));
        assert!(!should_process_virtual_spaces(false, true));
    }

    #[test]
    fn test_calc_effective_block_end() {
        assert_eq!(calc_effective_block_end(100, 80), 80);
        assert_eq!(calc_effective_block_end(50, 80), 50);
    }

    #[test]
    fn test_calc_block_vcol_span() {
        assert_eq!(calc_block_vcol_span(0, 9, true), 10);
        assert_eq!(calc_block_vcol_span(0, 9, false), 9);
        assert_eq!(calc_block_vcol_span(5, 5, true), 1);
    }

    #[test]
    fn test_is_line_short_for_block() {
        assert!(is_line_short_for_block(50, 80));
        assert!(!is_line_short_for_block(100, 80));
    }

    #[test]
    fn test_calc_short_line_fill_spaces() {
        // Line shorter than block start
        assert_eq!(calc_short_line_fill_spaces(10, 20, 5), 10);
        // Line within block
        assert_eq!(calc_short_line_fill_spaces(10, 20, 15), 5);
        // Line longer than block
        assert_eq!(calc_short_line_fill_spaces(10, 20, 30), 0);
    }

    #[test]
    fn test_calc_block_cursor_col() {
        assert_eq!(calc_block_cursor_col(10, 5), 15);
    }

    #[test]
    fn test_has_block_spaces() {
        assert!(has_block_startspaces(5));
        assert!(!has_block_startspaces(0));
        assert!(has_block_endspaces(5));
        assert!(!has_block_endspaces(0));
    }

    #[test]
    fn test_calc_block_total_spaces() {
        assert_eq!(calc_block_total_spaces(5, 3), 8);
    }

    #[test]
    fn test_partial_char_checks() {
        assert!(has_partial_start_char(5));
        assert!(!has_partial_start_char(0));
        assert!(has_partial_end_char(5));
        assert!(!has_partial_end_char(0));
    }

    #[test]
    fn test_calc_adjusted_block_width() {
        assert_eq!(calc_adjusted_block_width(10, 2, 3), 15);
        assert_eq!(calc_adjusted_block_width(10, 0, 0), 10);
    }

    #[test]
    fn test_calc_block_char_count() {
        assert_eq!(calc_block_char_count(10), 10);
        assert_eq!(calc_block_char_count(-5), 0);
    }

    #[test]
    fn test_ffi_wrappers() {
        assert_eq!(rs_calc_block_width(0, 9), 10);
        assert_eq!(rs_is_single_col_block(5, 5), 1);
        assert_eq!(rs_calc_block_line_count(1, 10), 10);
        assert_eq!(rs_is_multiline_block(1, 10), 1);
        assert_eq!(rs_calc_block_textcol(10, 5), 15);
        assert_eq!(rs_calc_block_replacement_chars(10, 2), 5);
        assert_eq!(rs_is_block_past_eol(100, 80), 1);
        assert_eq!(rs_calc_block_past_eol_amount(100, 80), 20);
        assert_eq!(rs_should_process_virtual_spaces(1, 1), 1);
        assert_eq!(rs_calc_effective_block_end(100, 80), 80);
        assert_eq!(rs_calc_block_vcol_span(0, 9, 1), 10);
        assert_eq!(rs_is_line_short_for_block(50, 80), 1);
        assert_eq!(rs_calc_short_line_fill_spaces(10, 20, 5), 10);
        assert_eq!(rs_calc_block_cursor_col(10, 5), 15);
        assert_eq!(rs_has_block_startspaces(5), 1);
        assert_eq!(rs_has_block_endspaces(0), 0);
        assert_eq!(rs_calc_block_total_spaces(5, 3), 8);
        assert_eq!(rs_has_partial_start_char(5), 1);
        assert_eq!(rs_has_partial_end_char(0), 0);
        assert_eq!(rs_calc_adjusted_block_width(10, 2, 3), 15);
        assert_eq!(rs_calc_block_char_count(10), 10);
    }
}
