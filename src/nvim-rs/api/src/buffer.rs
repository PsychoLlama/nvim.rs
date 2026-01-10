//! Buffer API helper functions
//!
//! This module provides Rust implementations of buffer manipulation utilities.

use std::ffi::{c_char, c_int};
use std::ptr;

/// linenr_T type (line number)
pub type LineNr = c_int;

/// colnr_T type (column number)
pub type ColNr = c_int;

/// bcount_t type (byte count)
pub type BCount = i64;

/// MAXLNUM constant
const MAXLNUM: LineNr = 0x7fffffff;

/// Check if start <= end for range validation
#[no_mangle]
pub extern "C" fn rs_range_ordered(start: i64, end: i64) -> bool {
    start <= end
}

/// Check if start row/col <= end row/col
#[no_mangle]
pub extern "C" fn rs_range_2d_ordered(
    start_row: i64,
    start_col: i64,
    end_row: i64,
    end_col: i64,
) -> bool {
    start_row < end_row || (start_row == end_row && start_col <= end_col)
}

/// Calculate number of lines to delete when replacing range
#[no_mangle]
pub extern "C" fn rs_calc_lines_to_delete(old_len: usize, new_len: usize) -> usize {
    old_len.saturating_sub(new_len)
}

/// Calculate number of lines to replace when changing range
#[no_mangle]
pub extern "C" fn rs_calc_lines_to_replace(old_len: usize, new_len: usize) -> usize {
    if old_len < new_len {
        old_len
    } else {
        new_len
    }
}

/// Calculate extra lines (positive = added, negative = deleted)
#[no_mangle]
pub extern "C" fn rs_calc_extra_lines(old_len: usize, new_len: usize) -> isize {
    new_len as isize - old_len as isize
}

/// Get the adjustment value for mark_adjust (MAXLNUM if deleting, 0 if inserting)
#[no_mangle]
pub extern "C" fn rs_mark_adjust_value(start: i64, end: i64) -> LineNr {
    if end > start {
        MAXLNUM
    } else {
        0
    }
}

/// Calculate column extent for extmark_splice
/// Returns end_col - start_col if on same row, just end_col otherwise
#[no_mangle]
pub extern "C" fn rs_col_extent(
    start_row: i64,
    end_row: i64,
    start_col: i64,
    end_col: i64,
) -> ColNr {
    if end_row == start_row {
        (end_col - start_col) as ColNr
    } else {
        end_col as ColNr
    }
}

/// Check if cursor is in the given range (inclusive)
#[no_mangle]
pub extern "C" fn rs_cursor_in_range_check(
    cursor_lnum: LineNr,
    start_row: LineNr,
    end_row: LineNr,
) -> bool {
    cursor_lnum >= start_row && cursor_lnum <= end_row
}

/// Calculate byte size of old region for set_text - single line case
#[no_mangle]
pub extern "C" fn rs_calc_old_byte_single_line(start_col: ColNr, end_col: ColNr) -> BCount {
    (end_col - start_col) as BCount
}

/// Calculate first line portion of old byte count
#[no_mangle]
pub extern "C" fn rs_calc_old_byte_first_line(len_at_start: ColNr, start_col: ColNr) -> BCount {
    (len_at_start - start_col) as BCount
}

/// Calculate last line portion of old byte count (including newline)
#[no_mangle]
pub extern "C" fn rs_calc_old_byte_last_line(end_col: ColNr) -> BCount {
    (end_col as BCount) + 1
}

/// Add line byte count to total (including newline)
#[no_mangle]
pub extern "C" fn rs_add_line_byte_count(byte_count: BCount, line_len: ColNr) -> BCount {
    byte_count + (line_len as BCount) + 1
}

/// Calculate first line length for set_text
#[no_mangle]
pub extern "C" fn rs_calc_first_line_len(
    start_col: ColNr,
    first_item_size: usize,
    replacement_size: usize,
    last_part_len: usize,
) -> usize {
    let mut len = (start_col as usize) + first_item_size;
    if replacement_size == 1 {
        len += last_part_len;
    }
    len
}

/// Check if we need a separate last line (replacement has > 1 line)
#[no_mangle]
pub extern "C" fn rs_needs_last_line(replacement_size: usize) -> bool {
    replacement_size > 1
}

/// Calculate new byte count from first item
#[no_mangle]
pub extern "C" fn rs_new_byte_from_first(first_item_size: usize) -> BCount {
    first_item_size as BCount
}

/// Add middle line to new byte count
#[no_mangle]
pub extern "C" fn rs_add_new_byte_middle(byte_count: BCount, line_size: usize) -> BCount {
    byte_count + (line_size as BCount) + 1
}

/// Add last line to new byte count
#[no_mangle]
pub extern "C" fn rs_add_new_byte_last(byte_count: BCount, last_item_size: usize) -> BCount {
    byte_count + (last_item_size as BCount) + 1
}

/// Check if index is valid (< MAXLNUM)
#[no_mangle]
pub extern "C" fn rs_index_valid(index: i64) -> bool {
    index < (MAXLNUM as i64)
}

/// Calculate inserted bytes for a line (len + 1 for newline)
#[no_mangle]
pub extern "C" fn rs_inserted_bytes_for_line(line_len: usize) -> BCount {
    (line_len as BCount) + 1
}

/// Check if buffer name is NULL (no name)
///
/// # Safety
/// The pointer may be NULL (that's what we're checking).
#[no_mangle]
pub unsafe extern "C" fn rs_buf_name_is_null(name: *const c_char) -> bool {
    name.is_null()
}

/// Create empty String
#[no_mangle]
pub extern "C" fn rs_api_string_init() -> super::NvimString {
    super::NvimString {
        data: ptr::null_mut(),
        size: 0,
    }
}

/// Check if mark name is single character
#[no_mangle]
pub extern "C" fn rs_mark_name_valid_len(size: usize) -> bool {
    size == 1
}

/// Check if mark lnum is non-zero (mark is set)
#[no_mangle]
pub extern "C" fn rs_mark_is_set(lnum: LineNr) -> bool {
    lnum != 0
}

/// Check if mark fnum matches buffer handle
#[no_mangle]
pub extern "C" fn rs_mark_in_buffer(fnum: c_int, handle: c_int) -> bool {
    fnum == handle
}

/// Create position tuple data (row)
#[no_mangle]
pub extern "C" fn rs_pos_to_row(lnum: LineNr) -> i64 {
    lnum as i64
}

/// Create position tuple data (col)
#[no_mangle]
pub extern "C" fn rs_pos_to_col(col: ColNr) -> i64 {
    col as i64
}

/// Check if size needs middle line collection (size > 2)
#[no_mangle]
pub extern "C" fn rs_needs_middle_lines(size: usize) -> bool {
    size > 2
}

/// Calculate middle lines count
#[no_mangle]
pub extern "C" fn rs_middle_lines_count(size: usize) -> usize {
    size.saturating_sub(2)
}

/// Calculate start index for middle lines
#[no_mangle]
pub extern "C" fn rs_middle_lines_start_idx(_size: usize) -> c_int {
    1
}

/// Calculate last line index
#[no_mangle]
pub extern "C" fn rs_last_line_idx(size: usize) -> c_int {
    (size - 1) as c_int
}

/// Get MAXCOL - 1 for end column
#[no_mangle]
pub extern "C" fn rs_maxcol_minus_one() -> i64 {
    // MAXCOL is typically INT_MAX or similar large value
    // In Neovim it's defined as MAXCOL = INT_MAX
    (i32::MAX - 1) as i64
}

/// Check if error indicates single line in get_text
#[no_mangle]
pub extern "C" fn rs_is_single_line_range(start_row: i64, end_row: i64) -> bool {
    start_row == end_row
}

/// Validate column is in range [0, len]
#[no_mangle]
pub extern "C" fn rs_col_in_range(col: i64, len: ColNr) -> bool {
    col >= 0 && col <= (len as i64)
}

/// Convert negative column to positive
#[no_mangle]
pub extern "C" fn rs_normalize_col(col: i64, len: ColNr) -> i64 {
    if col < 0 {
        (len as i64) + col + 1
    } else {
        col
    }
}

/// Check if DOBUF result is FAIL
#[no_mangle]
pub extern "C" fn rs_dobuf_failed(result: c_int) -> bool {
    result == 0 // FAIL = 0 in Neovim
}

/// uhp extmark size check (for stats)
///
/// # Safety
/// The pointer may be NULL (that's what we're checking).
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_has_extmarks(uhp: *const std::ffi::c_void) -> bool {
    !uhp.is_null()
}

/// Calculate last part length for set_text
#[no_mangle]
pub extern "C" fn rs_calc_last_part_len(len_at_end: ColNr, end_col: ColNr) -> usize {
    (len_at_end - end_col) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_ordered() {
        assert!(rs_range_ordered(0, 0));
        assert!(rs_range_ordered(0, 5));
        assert!(rs_range_ordered(-1, 0));
        assert!(!rs_range_ordered(5, 0));
    }

    #[test]
    fn test_range_2d_ordered() {
        // Same row
        assert!(rs_range_2d_ordered(1, 0, 1, 5));
        assert!(rs_range_2d_ordered(1, 5, 1, 5));
        assert!(!rs_range_2d_ordered(1, 5, 1, 0));

        // Different rows
        assert!(rs_range_2d_ordered(1, 10, 2, 0));
        assert!(!rs_range_2d_ordered(2, 0, 1, 10));
    }

    #[test]
    fn test_calc_lines_to_delete() {
        assert_eq!(rs_calc_lines_to_delete(5, 3), 2);
        assert_eq!(rs_calc_lines_to_delete(3, 5), 0);
        assert_eq!(rs_calc_lines_to_delete(5, 5), 0);
    }

    #[test]
    fn test_calc_lines_to_replace() {
        assert_eq!(rs_calc_lines_to_replace(5, 3), 3);
        assert_eq!(rs_calc_lines_to_replace(3, 5), 3);
        assert_eq!(rs_calc_lines_to_replace(5, 5), 5);
    }

    #[test]
    fn test_calc_extra_lines() {
        assert_eq!(rs_calc_extra_lines(3, 5), 2);
        assert_eq!(rs_calc_extra_lines(5, 3), -2);
        assert_eq!(rs_calc_extra_lines(5, 5), 0);
    }

    #[test]
    fn test_col_extent() {
        // Same row: end_col - start_col
        assert_eq!(rs_col_extent(1, 1, 5, 10), 5);
        // Different rows: just end_col
        assert_eq!(rs_col_extent(1, 2, 5, 10), 10);
    }

    #[test]
    fn test_normalize_col() {
        assert_eq!(rs_normalize_col(5, 10), 5);
        assert_eq!(rs_normalize_col(-1, 10), 10); // -1 -> len
        assert_eq!(rs_normalize_col(-2, 10), 9); // -2 -> len - 1
        assert_eq!(rs_normalize_col(0, 10), 0);
    }

    #[test]
    fn test_col_in_range() {
        assert!(rs_col_in_range(0, 10));
        assert!(rs_col_in_range(10, 10));
        assert!(rs_col_in_range(5, 10));
        assert!(!rs_col_in_range(-1, 10));
        assert!(!rs_col_in_range(11, 10));
    }

    #[test]
    fn test_calc_first_line_len() {
        // start_col=5, first_item=10, replacement_size=1, last_part=3
        // Should be: 5 + 10 + 3 = 18
        assert_eq!(rs_calc_first_line_len(5, 10, 1, 3), 18);

        // start_col=5, first_item=10, replacement_size=2, last_part=3
        // Should be: 5 + 10 = 15 (last_part not added for multi-line)
        assert_eq!(rs_calc_first_line_len(5, 10, 2, 3), 15);
    }

    #[test]
    fn test_middle_lines() {
        assert!(!rs_needs_middle_lines(1));
        assert!(!rs_needs_middle_lines(2));
        assert!(rs_needs_middle_lines(3));
        assert!(rs_needs_middle_lines(5));

        assert_eq!(rs_middle_lines_count(1), 0);
        assert_eq!(rs_middle_lines_count(2), 0);
        assert_eq!(rs_middle_lines_count(3), 1);
        assert_eq!(rs_middle_lines_count(5), 3);
    }

    #[test]
    fn test_mark_validation() {
        assert!(rs_mark_name_valid_len(1));
        assert!(!rs_mark_name_valid_len(0));
        assert!(!rs_mark_name_valid_len(2));

        assert!(rs_mark_is_set(1));
        assert!(!rs_mark_is_set(0));

        assert!(rs_mark_in_buffer(5, 5));
        assert!(!rs_mark_in_buffer(5, 6));
    }
}
