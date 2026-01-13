//! Grid Cell Operations
//!
//! This module provides cell-level operations for grid manipulation including
//! cell initialization, copying, clearing, and comparison operations.
//! Phase 169 of Rust migration.

use std::ffi::c_int;

/// Type alias for screen character (matches C's `schar_T` which is `uint32_t`).
type ScharT = u32;

/// Type alias for screen attribute (matches C's `sattr_T` which is `int32_t`).
type SattrT = i32;

/// Type alias for column number (matches C's `colnr_T` which is `int32_t`).
type ColnrT = i32;

/// Invalid attribute marker
const SATTR_INVALID: SattrT = -1;

/// Invalid column marker
const COLNR_INVALID: ColnrT = -1;

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    fn nvim_get_linebuf_char() -> *mut ScharT;
    fn nvim_get_linebuf_attr() -> *mut SattrT;
    fn nvim_get_linebuf_vcol() -> *mut ColnrT;
    fn nvim_get_linebuf_size() -> usize;
}

// =============================================================================
// Cell Value Helpers
// =============================================================================

/// Check if two schar values are equal.
#[no_mangle]
pub extern "C" fn rs_schar_equal(a: ScharT, b: ScharT) -> c_int {
    c_int::from(a == b)
}

/// Check if two sattr values are equal.
#[no_mangle]
pub extern "C" fn rs_sattr_equal(a: SattrT, b: SattrT) -> c_int {
    c_int::from(a == b)
}

/// Check if a cell is empty (NUL char and default attr).
#[no_mangle]
pub extern "C" fn rs_cell_is_empty(schar: ScharT, attr: SattrT) -> c_int {
    c_int::from(schar == 0 && attr == 0)
}

/// Check if a cell is a placeholder (continuation of wide char).
///
/// A placeholder cell has schar == 0 but may have an attribute.
#[no_mangle]
pub extern "C" fn rs_cell_is_placeholder(schar: ScharT) -> c_int {
    c_int::from(schar == 0)
}

// =============================================================================
// Cell Comparison
// =============================================================================

/// Compare two cells for equality.
///
/// Returns true if both the character and attribute match.
#[no_mangle]
pub extern "C" fn rs_cell_equal(
    schar1: ScharT,
    attr1: SattrT,
    schar2: ScharT,
    attr2: SattrT,
) -> c_int {
    c_int::from(schar1 == schar2 && attr1 == attr2)
}

/// Check if a cell needs redrawing compared to another cell.
///
/// A cell needs redraw if the character or attribute differs.
#[no_mangle]
pub extern "C" fn rs_cell_needs_redraw(
    new_schar: ScharT,
    new_attr: SattrT,
    old_schar: ScharT,
    old_attr: SattrT,
) -> c_int {
    c_int::from(new_schar != old_schar || new_attr != old_attr)
}

/// Check if a cell differs from the linebuf at a given column.
///
/// # Safety
/// `col` must be within bounds of the linebuf.
#[no_mangle]
pub unsafe extern "C" fn rs_cell_differs_from_linebuf(
    col: c_int,
    old_schar: ScharT,
    old_attr: SattrT,
) -> c_int {
    let col_idx = col as usize;
    let linebuf_char = nvim_get_linebuf_char();
    let linebuf_attr = nvim_get_linebuf_attr();

    if linebuf_char.is_null() || linebuf_attr.is_null() {
        return 0;
    }

    let new_schar = *linebuf_char.add(col_idx);
    let new_attr = *linebuf_attr.add(col_idx);

    c_int::from(new_schar != old_schar || new_attr != old_attr)
}

// =============================================================================
// Linebuf Operations
// =============================================================================

/// Get the linebuf size.
#[no_mangle]
pub unsafe extern "C" fn rs_get_linebuf_size() -> usize {
    nvim_get_linebuf_size()
}

/// Check if a column is within linebuf bounds.
#[no_mangle]
pub unsafe extern "C" fn rs_linebuf_col_valid(col: c_int) -> c_int {
    if col < 0 {
        return 0;
    }
    let size = nvim_get_linebuf_size();
    c_int::from((col as usize) < size)
}

/// Get character from linebuf at column.
///
/// # Safety
/// `col` must be within bounds.
#[no_mangle]
pub unsafe extern "C" fn rs_linebuf_get_char(col: c_int) -> ScharT {
    let linebuf_char = nvim_get_linebuf_char();
    if linebuf_char.is_null() {
        return 0;
    }
    *linebuf_char.add(col as usize)
}

/// Get attribute from linebuf at column.
///
/// # Safety
/// `col` must be within bounds.
#[no_mangle]
pub unsafe extern "C" fn rs_linebuf_get_attr(col: c_int) -> SattrT {
    let linebuf_attr = nvim_get_linebuf_attr();
    if linebuf_attr.is_null() {
        return SATTR_INVALID;
    }
    *linebuf_attr.add(col as usize)
}

/// Get vcol from linebuf at column.
///
/// # Safety
/// `col` must be within bounds.
#[no_mangle]
pub unsafe extern "C" fn rs_linebuf_get_vcol(col: c_int) -> ColnrT {
    let linebuf_vcol = nvim_get_linebuf_vcol();
    if linebuf_vcol.is_null() {
        return COLNR_INVALID;
    }
    *linebuf_vcol.add(col as usize)
}

/// Set character in linebuf at column.
///
/// # Safety
/// `col` must be within bounds.
#[no_mangle]
pub unsafe extern "C" fn rs_linebuf_set_char(col: c_int, schar: ScharT) {
    let linebuf_char = nvim_get_linebuf_char();
    if !linebuf_char.is_null() {
        *linebuf_char.add(col as usize) = schar;
    }
}

/// Set attribute in linebuf at column.
///
/// # Safety
/// `col` must be within bounds.
#[no_mangle]
pub unsafe extern "C" fn rs_linebuf_set_attr(col: c_int, attr: SattrT) {
    let linebuf_attr = nvim_get_linebuf_attr();
    if !linebuf_attr.is_null() {
        *linebuf_attr.add(col as usize) = attr;
    }
}

/// Set vcol in linebuf at column.
///
/// # Safety
/// `col` must be within bounds.
#[no_mangle]
pub unsafe extern "C" fn rs_linebuf_set_vcol(col: c_int, vcol: ColnrT) {
    let linebuf_vcol = nvim_get_linebuf_vcol();
    if !linebuf_vcol.is_null() {
        *linebuf_vcol.add(col as usize) = vcol;
    }
}

/// Set character and attribute in linebuf at column.
///
/// # Safety
/// `col` must be within bounds.
#[no_mangle]
pub unsafe extern "C" fn rs_linebuf_set_cell(col: c_int, schar: ScharT, attr: SattrT) {
    rs_linebuf_set_char(col, schar);
    rs_linebuf_set_attr(col, attr);
}

/// Set character, attribute, and vcol in linebuf at column.
///
/// # Safety
/// `col` must be within bounds.
#[no_mangle]
pub unsafe extern "C" fn rs_linebuf_set_cell_full(
    col: c_int,
    schar: ScharT,
    attr: SattrT,
    vcol: ColnrT,
) {
    rs_linebuf_set_char(col, schar);
    rs_linebuf_set_attr(col, attr);
    rs_linebuf_set_vcol(col, vcol);
}

// =============================================================================
// Cell Range Operations
// =============================================================================

/// Check if a range of columns is within bounds.
#[no_mangle]
pub extern "C" fn rs_col_range_valid(start_col: c_int, end_col: c_int, max_col: c_int) -> c_int {
    c_int::from(start_col >= 0 && end_col >= start_col && end_col <= max_col)
}

/// Calculate number of cells in a range.
#[no_mangle]
pub extern "C" fn rs_col_range_len(start_col: c_int, end_col: c_int) -> c_int {
    if end_col > start_col {
        end_col - start_col
    } else {
        0
    }
}

/// Clamp a column range to valid bounds.
///
/// Returns the clamped start column.
#[no_mangle]
pub extern "C" fn rs_clamp_col_range_start(start_col: c_int, _max_col: c_int) -> c_int {
    if start_col < 0 {
        0
    } else {
        start_col
    }
}

/// Clamp a column range end to valid bounds.
#[no_mangle]
pub extern "C" fn rs_clamp_col_range_end(end_col: c_int, max_col: c_int) -> c_int {
    if end_col > max_col {
        max_col
    } else if end_col < 0 {
        0
    } else {
        end_col
    }
}

// =============================================================================
// Wide Character Helpers
// =============================================================================

/// Check if a cell is the second half of a double-width character.
///
/// Double-width characters occupy two cells, with the second cell containing
/// schar == 0 (placeholder).
#[no_mangle]
pub extern "C" fn rs_is_wide_char_continuation(schar: ScharT, prev_schar: ScharT) -> c_int {
    // Second cell of wide char is a placeholder (0), first cell is non-zero
    c_int::from(schar == 0 && prev_schar != 0)
}

/// Calculate the number of cells needed for a character.
///
/// Returns 2 for double-width characters, 1 for normal characters.
/// This is a simple version - actual width calculation is in schar_cells().
#[no_mangle]
pub extern "C" fn rs_char_cells_simple(c: c_int) -> c_int {
    // Simplified check: CJK characters and some others are typically 2 cells
    // Full implementation is in rs_schar_cells
    if (0x1100..=0x115F).contains(&c)  // Hangul Jamo
        || (0x2E80..=0x9FFF).contains(&c)  // CJK Radicals through CJK Unified Ideographs
        || (0xAC00..=0xD7A3).contains(&c)  // Hangul Syllables
        || (0xF900..=0xFAFF).contains(&c)  // CJK Compatibility Ideographs
        || (0xFE10..=0xFE1F).contains(&c)  // Vertical forms
        || (0xFF00..=0xFF60).contains(&c)  // Fullwidth Forms
        || (0x20000..=0x2FFFF).contains(&c)
    // CJK Extension B and beyond
    {
        2
    } else {
        1
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schar_equal() {
        assert_eq!(rs_schar_equal(0, 0), 1);
        assert_eq!(rs_schar_equal(65, 65), 1);
        assert_eq!(rs_schar_equal(65, 66), 0);
    }

    #[test]
    fn test_sattr_equal() {
        assert_eq!(rs_sattr_equal(0, 0), 1);
        assert_eq!(rs_sattr_equal(1, 1), 1);
        assert_eq!(rs_sattr_equal(1, 2), 0);
    }

    #[test]
    fn test_cell_is_empty() {
        assert_eq!(rs_cell_is_empty(0, 0), 1);
        assert_eq!(rs_cell_is_empty(65, 0), 0);
        assert_eq!(rs_cell_is_empty(0, 1), 0);
    }

    #[test]
    fn test_cell_is_placeholder() {
        assert_eq!(rs_cell_is_placeholder(0), 1);
        assert_eq!(rs_cell_is_placeholder(65), 0);
    }

    #[test]
    fn test_cell_equal() {
        assert_eq!(rs_cell_equal(65, 1, 65, 1), 1);
        assert_eq!(rs_cell_equal(65, 1, 66, 1), 0);
        assert_eq!(rs_cell_equal(65, 1, 65, 2), 0);
    }

    #[test]
    fn test_cell_needs_redraw() {
        assert_eq!(rs_cell_needs_redraw(65, 1, 65, 1), 0);
        assert_eq!(rs_cell_needs_redraw(66, 1, 65, 1), 1);
        assert_eq!(rs_cell_needs_redraw(65, 2, 65, 1), 1);
    }

    #[test]
    fn test_col_range_valid() {
        assert_eq!(rs_col_range_valid(0, 10, 100), 1);
        assert_eq!(rs_col_range_valid(-1, 10, 100), 0);
        assert_eq!(rs_col_range_valid(10, 5, 100), 0);
        assert_eq!(rs_col_range_valid(0, 101, 100), 0);
    }

    #[test]
    fn test_col_range_len() {
        assert_eq!(rs_col_range_len(0, 10), 10);
        assert_eq!(rs_col_range_len(5, 15), 10);
        assert_eq!(rs_col_range_len(10, 5), 0);
    }

    #[test]
    fn test_char_cells_simple() {
        // ASCII characters are 1 cell
        assert_eq!(rs_char_cells_simple(65), 1); // 'A'
                                                 // CJK characters are 2 cells
        assert_eq!(rs_char_cells_simple(0x4E00), 2); // First CJK unified ideograph
    }

    #[test]
    fn test_is_wide_char_continuation() {
        assert_eq!(rs_is_wide_char_continuation(0, 65), 1); // placeholder after char
        assert_eq!(rs_is_wide_char_continuation(0, 0), 0); // both zero
        assert_eq!(rs_is_wide_char_continuation(65, 0), 0); // not a placeholder
    }
}
