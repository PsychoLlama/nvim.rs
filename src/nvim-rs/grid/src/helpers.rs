//! Grid helper functions for Neovim
//!
//! This module provides additional Rust helper functions that complement
//! the main grid operations.

use std::ffi::c_int;

/// Type alias for screen character (matches C's `schar_T` which is `uint32_t`).
type ScharT = u32;

// =============================================================================
// schar_from_ascii - convert ASCII to schar_T
// =============================================================================

/// Put an ASCII character in a screen cell.
///
/// This is the Rust equivalent of C's `schar_from_ascii()` macro.
/// Handles endianness correctly.
///
/// # Arguments
/// * `c` - ASCII character code (0-127)
///
/// # Returns
/// `schar_T` representation of the character
#[inline]
const fn schar_from_ascii_impl(c: u8) -> ScharT {
    #[cfg(target_endian = "big")]
    {
        (c as ScharT) << 24
    }
    #[cfg(target_endian = "little")]
    {
        c as ScharT
    }
}

/// FFI wrapper for `schar_from_ascii`.
///
/// Convert an ASCII character to an `schar_T`.
#[no_mangle]
pub extern "C" fn rs_schar_from_ascii(c: c_int) -> ScharT {
    schar_from_ascii_impl(c as u8)
}

// =============================================================================
// Grid line flag helpers
// =============================================================================

/// SLF_RIGHTLEFT flag (0x01)
const SLF_RIGHTLEFT: c_int = 1;
/// SLF_WRAP flag (0x02)
const SLF_WRAP: c_int = 2;
/// SLF_INC_VCOL flag (0x04)
const SLF_INC_VCOL: c_int = 4;

/// Check if SLF_RIGHTLEFT flag is set.
#[no_mangle]
pub extern "C" fn rs_slf_is_rightleft(flags: c_int) -> c_int {
    c_int::from((flags & SLF_RIGHTLEFT) != 0)
}

/// Check if SLF_WRAP flag is set.
#[no_mangle]
pub extern "C" fn rs_slf_is_wrap(flags: c_int) -> c_int {
    c_int::from((flags & SLF_WRAP) != 0)
}

/// Check if SLF_INC_VCOL flag is set.
#[no_mangle]
pub extern "C" fn rs_slf_is_inc_vcol(flags: c_int) -> c_int {
    c_int::from((flags & SLF_INC_VCOL) != 0)
}

/// Get SLF_RIGHTLEFT constant value.
#[no_mangle]
pub extern "C" fn rs_slf_rightleft() -> c_int {
    SLF_RIGHTLEFT
}

/// Get SLF_WRAP constant value.
#[no_mangle]
pub extern "C" fn rs_slf_wrap() -> c_int {
    SLF_WRAP
}

/// Get SLF_INC_VCOL constant value.
#[no_mangle]
pub extern "C" fn rs_slf_inc_vcol() -> c_int {
    SLF_INC_VCOL
}

/// Combine multiple SLF flags into a single value.
#[no_mangle]
pub extern "C" fn rs_slf_combine(rightleft: c_int, wrap: c_int, inc_vcol: c_int) -> c_int {
    let mut flags = 0;
    if rightleft != 0 {
        flags |= SLF_RIGHTLEFT;
    }
    if wrap != 0 {
        flags |= SLF_WRAP;
    }
    if inc_vcol != 0 {
        flags |= SLF_INC_VCOL;
    }
    flags
}

// =============================================================================
// Grid dimension helpers
// =============================================================================

/// Calculate the number of cells needed for a grid.
///
/// Returns rows * columns as a size_t.
#[no_mangle]
pub extern "C" fn rs_grid_cell_count(rows: c_int, cols: c_int) -> usize {
    (rows as usize) * (cols as usize)
}

/// Calculate line offset for a row in a grid.
///
/// Returns row * cols as a size_t.
#[no_mangle]
pub extern "C" fn rs_grid_line_offset(row: c_int, cols: c_int) -> usize {
    (row as usize) * (cols as usize)
}

/// Check if a row is within grid bounds.
#[no_mangle]
pub extern "C" fn rs_grid_row_valid(row: c_int, rows: c_int) -> c_int {
    c_int::from(row >= 0 && row < rows)
}

/// Check if a column is within grid bounds.
#[no_mangle]
pub extern "C" fn rs_grid_col_valid(col: c_int, cols: c_int) -> c_int {
    c_int::from(col >= 0 && col < cols)
}

/// Check if a position (row, col) is within grid bounds.
#[no_mangle]
pub extern "C" fn rs_grid_pos_valid(row: c_int, col: c_int, rows: c_int, cols: c_int) -> c_int {
    c_int::from(row >= 0 && row < rows && col >= 0 && col < cols)
}

/// Clamp a column value to grid width.
#[no_mangle]
pub extern "C" fn rs_grid_clamp_col(col: c_int, cols: c_int) -> c_int {
    if col < 0 {
        0
    } else if col > cols {
        cols
    } else {
        col
    }
}

/// Clamp a row value to grid height.
#[no_mangle]
pub extern "C" fn rs_grid_clamp_row(row: c_int, rows: c_int) -> c_int {
    if row < 0 {
        0
    } else if row > rows {
        rows
    } else {
        row
    }
}

// =============================================================================
// schar_T helpers
// =============================================================================

/// Check if an schar represents a NUL character.
#[no_mangle]
pub extern "C" fn rs_schar_is_nul(sc: ScharT) -> c_int {
    c_int::from(sc == 0)
}

/// Check if an schar represents a space character.
#[no_mangle]
pub extern "C" fn rs_schar_is_space(sc: ScharT) -> c_int {
    c_int::from(sc == schar_from_ascii_impl(b' '))
}

/// Get space character as schar_T.
#[no_mangle]
pub extern "C" fn rs_schar_space() -> ScharT {
    schar_from_ascii_impl(b' ')
}

/// Get greater-than character as schar_T.
#[no_mangle]
pub extern "C" fn rs_schar_gt() -> ScharT {
    schar_from_ascii_impl(b'>')
}

/// Get less-than character as schar_T.
#[no_mangle]
pub extern "C" fn rs_schar_lt() -> ScharT {
    schar_from_ascii_impl(b'<')
}

/// Get tilde character as schar_T.
#[no_mangle]
pub extern "C" fn rs_schar_tilde() -> ScharT {
    schar_from_ascii_impl(b'~')
}

/// Get at-sign character as schar_T.
#[no_mangle]
pub extern "C" fn rs_schar_at() -> ScharT {
    schar_from_ascii_impl(b'@')
}

// =============================================================================
// Attribute helpers
// =============================================================================

/// Type alias for screen attribute (matches C's `sattr_T` which is `int32_t`).
type SattrT = i32;

/// Check if an attribute is invalid (negative).
#[no_mangle]
pub extern "C" fn rs_sattr_is_invalid(attr: SattrT) -> c_int {
    c_int::from(attr < 0)
}

/// Get the invalid attribute marker value (-1).
#[no_mangle]
pub extern "C" fn rs_sattr_invalid() -> SattrT {
    -1
}

/// Get the default attribute value (0).
#[no_mangle]
pub extern "C" fn rs_sattr_default() -> SattrT {
    0
}

// =============================================================================
// Column number helpers
// =============================================================================

/// Type alias for column number (matches C's `colnr_T` which is `int32_t`).
type ColnrT = i32;

/// Get the invalid column marker value (-1).
#[no_mangle]
pub extern "C" fn rs_colnr_invalid() -> ColnrT {
    -1
}

/// Check if a column number is invalid (negative).
#[no_mangle]
pub extern "C" fn rs_colnr_is_invalid(col: ColnrT) -> c_int {
    c_int::from(col < 0)
}

// =============================================================================
// Grid copy helpers
// =============================================================================

/// Calculate copy length when resizing grids.
///
/// Returns the minimum of old_cols and new_cols.
#[no_mangle]
pub extern "C" fn rs_grid_copy_len(old_cols: c_int, new_cols: c_int) -> c_int {
    old_cols.min(new_cols)
}

/// Check if grid data should be copied during resize.
///
/// Returns true if we have valid old data and the row exists in the old grid.
#[no_mangle]
pub extern "C" fn rs_grid_should_copy(
    new_row: c_int,
    old_rows: c_int,
    old_chars_not_null: c_int,
) -> c_int {
    c_int::from(new_row < old_rows && old_chars_not_null != 0)
}

// =============================================================================
// Border text helpers
// =============================================================================

/// Border text alignment - left (0)
const ALIGN_LEFT: c_int = 0;
/// Border text alignment - center (1)
const ALIGN_CENTER: c_int = 1;
/// Border text alignment - right (2)
const ALIGN_RIGHT: c_int = 2;

/// Check if alignment is left.
#[no_mangle]
pub extern "C" fn rs_grid_align_is_left(align: c_int) -> c_int {
    c_int::from(align == ALIGN_LEFT)
}

/// Check if alignment is center.
#[no_mangle]
pub extern "C" fn rs_grid_align_is_center(align: c_int) -> c_int {
    c_int::from(align == ALIGN_CENTER)
}

/// Check if alignment is right.
#[no_mangle]
pub extern "C" fn rs_grid_align_is_right(align: c_int) -> c_int {
    c_int::from(align == ALIGN_RIGHT)
}

/// Get ALIGN_LEFT constant.
#[no_mangle]
pub extern "C" fn rs_grid_align_left_val() -> c_int {
    ALIGN_LEFT
}

/// Get ALIGN_CENTER constant.
#[no_mangle]
pub extern "C" fn rs_grid_align_center_val() -> c_int {
    ALIGN_CENTER
}

/// Get ALIGN_RIGHT constant.
#[no_mangle]
pub extern "C" fn rs_grid_align_right_val() -> c_int {
    ALIGN_RIGHT
}

// =============================================================================
// Debug flag helpers
// =============================================================================

/// rdb_flags value for kOptRdbFlagNodelta (0x08)
const K_OPT_RDB_FLAG_NODELTA: u32 = 0x08;
/// rdb_flags value for kOptRdbFlagInvalid (0x04)
const K_OPT_RDB_FLAG_INVALID: u32 = 0x04;

/// Check if nodelta debug flag is set.
#[no_mangle]
pub extern "C" fn rs_rdb_is_nodelta(flags: u32) -> c_int {
    c_int::from((flags & K_OPT_RDB_FLAG_NODELTA) != 0)
}

/// Check if invalid debug flag is set.
#[no_mangle]
pub extern "C" fn rs_rdb_is_invalid(flags: u32) -> c_int {
    c_int::from((flags & K_OPT_RDB_FLAG_INVALID) != 0)
}

/// Get kOptRdbFlagNodelta constant.
#[no_mangle]
pub extern "C" fn rs_rdb_flag_nodelta() -> u32 {
    K_OPT_RDB_FLAG_NODELTA
}

/// Get kOptRdbFlagInvalid constant.
#[no_mangle]
pub extern "C" fn rs_rdb_flag_invalid() -> u32 {
    K_OPT_RDB_FLAG_INVALID
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schar_from_ascii() {
        // Test space character
        #[cfg(target_endian = "little")]
        {
            assert_eq!(schar_from_ascii_impl(b' '), 0x0000_0020);
            assert_eq!(schar_from_ascii_impl(b'A'), 0x0000_0041);
            assert_eq!(schar_from_ascii_impl(b'~'), 0x0000_007E);
        }
        #[cfg(target_endian = "big")]
        {
            assert_eq!(schar_from_ascii_impl(b' '), 0x2000_0000);
            assert_eq!(schar_from_ascii_impl(b'A'), 0x4100_0000);
            assert_eq!(schar_from_ascii_impl(b'~'), 0x7E00_0000);
        }
    }

    #[test]
    fn test_slf_flags() {
        assert_eq!(rs_slf_rightleft(), 1);
        assert_eq!(rs_slf_wrap(), 2);
        assert_eq!(rs_slf_inc_vcol(), 4);

        assert_eq!(rs_slf_is_rightleft(1), 1);
        assert_eq!(rs_slf_is_rightleft(0), 0);
        assert_eq!(rs_slf_is_rightleft(3), 1); // 1 | 2

        assert_eq!(rs_slf_combine(1, 1, 0), 3);
        assert_eq!(rs_slf_combine(1, 1, 1), 7);
    }

    #[test]
    fn test_grid_bounds() {
        assert_eq!(rs_grid_row_valid(0, 10), 1);
        assert_eq!(rs_grid_row_valid(9, 10), 1);
        assert_eq!(rs_grid_row_valid(10, 10), 0);
        assert_eq!(rs_grid_row_valid(-1, 10), 0);

        assert_eq!(rs_grid_pos_valid(5, 5, 10, 10), 1);
        assert_eq!(rs_grid_pos_valid(10, 5, 10, 10), 0);
    }

    #[test]
    fn test_grid_clamp() {
        assert_eq!(rs_grid_clamp_col(-5, 80), 0);
        assert_eq!(rs_grid_clamp_col(100, 80), 80);
        assert_eq!(rs_grid_clamp_col(40, 80), 40);
    }

    #[test]
    fn test_schar_helpers() {
        assert_eq!(rs_schar_is_nul(0), 1);
        assert_eq!(rs_schar_is_nul(1), 0);
        assert_eq!(rs_schar_is_space(rs_schar_space()), 1);
    }
}
