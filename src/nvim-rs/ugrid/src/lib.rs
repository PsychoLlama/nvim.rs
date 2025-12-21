//! Unicode grid (`UGrid`) management for Neovim TUI
//!
//! This crate provides Rust implementations of the `UGrid` functions from
//! `src/nvim/ugrid.c`. The `UGrid` is used by the TUI to manage screen cells.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::cast_sign_loss)] // Grid indices are always non-negative
#![allow(clippy::cast_possible_truncation)] // Grid dimensions fit in c_int
#![allow(clippy::similar_names)] // start/stop/step are intentionally similar

use std::ffi::c_int;
use std::ptr;

// Re-export memory functions
use nvim_memory::{xcalloc, xfree, xmalloc};

// =============================================================================
// Type Definitions (matching C layout)
// =============================================================================

/// Type alias for screen character (matches C's `schar_T` which is `uint32_t`).
pub type ScharT = u32;

/// Type alias for screen attribute (matches C's `sattr_T` which is `int32_t`).
pub type SattrT = i32;

/// A single cell in the unicode grid.
///
/// Matches the C definition:
/// ```c
/// typedef struct {
///   schar_T data;
///   sattr_T attr;
/// } UCell;
/// ```
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct UCell {
    pub data: ScharT,
    pub attr: SattrT,
}

/// The unicode grid structure.
///
/// Matches the C definition:
/// ```c
/// typedef struct {
///   int row, col;
///   int width, height;
///   UCell **cells;
/// } UGrid;
/// ```
#[repr(C)]
pub struct UGrid {
    pub row: c_int,
    pub col: c_int,
    pub width: c_int,
    pub height: c_int,
    pub cells: *mut *mut UCell,
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Convert an ASCII character to an `schar_T`.
///
/// This matches the C macro `schar_from_ascii` which encodes
/// a single ASCII byte as UTF-8 in native byte order.
#[inline]
const fn schar_from_ascii(c: u8) -> ScharT {
    // ASCII characters are single-byte UTF-8, stored in native endian
    // This matches: ScharT::from_ne_bytes([c, 0, 0, 0])
    #[cfg(target_endian = "little")]
    {
        c as ScharT
    }
    #[cfg(target_endian = "big")]
    {
        (c as ScharT) << 24
    }
}

/// Space character as `schar_T` (compile-time constant)
const SPACE_SCHAR: ScharT = schar_from_ascii(b' ');

/// Clear a rectangular region of the grid.
///
/// # Safety
/// - `grid` must be a valid pointer to an initialized `UGrid`
/// - The region must be within the grid bounds
unsafe fn clear_region(grid: *mut UGrid, top: c_int, bot: c_int, left: c_int, right: c_int, attr: SattrT) {
    let grid = &mut *grid;
    for row in top..=bot {
        let row_cells = *grid.cells.add(row as usize);
        for col in left..=right {
            let cell = row_cells.add(col as usize);
            (*cell).data = SPACE_SCHAR;
            (*cell).attr = attr;
        }
    }
}

/// Free all cells in the grid.
///
/// # Safety
/// - `grid` must be a valid pointer to an initialized `UGrid`
unsafe fn destroy_cells(grid: *mut UGrid) {
    let grid = &mut *grid;
    if !grid.cells.is_null() {
        for i in 0..grid.height {
            xfree((*grid.cells.add(i as usize)).cast());
        }
        xfree(grid.cells.cast());
        grid.cells = ptr::null_mut();
    }
}

// =============================================================================
// Public API (FFI exports)
// =============================================================================

/// Initialize a `UGrid` structure.
///
/// Sets the cells pointer to NULL.
///
/// # Safety
/// - `grid` must be a valid pointer to a `UGrid`
#[no_mangle]
pub unsafe extern "C" fn rs_ugrid_init(grid: *mut UGrid) {
    (*grid).cells = ptr::null_mut();
}

/// Free all resources associated with a `UGrid`.
///
/// # Safety
/// - `grid` must be a valid pointer to an initialized `UGrid`
#[no_mangle]
pub unsafe extern "C" fn rs_ugrid_free(grid: *mut UGrid) {
    destroy_cells(grid);
}

/// Resize the grid, allocating new cells.
///
/// Frees any existing cells and allocates a new grid of the specified dimensions.
///
/// # Safety
/// - `grid` must be a valid pointer to an initialized `UGrid`
/// - `width` and `height` must be positive
#[no_mangle]
pub unsafe extern "C" fn rs_ugrid_resize(grid: *mut UGrid, width: c_int, height: c_int) {
    let grid = &mut *grid;

    // Free existing cells
    destroy_cells(grid);

    // Allocate row pointers
    let row_ptr_size = (height as usize) * std::mem::size_of::<*mut UCell>();
    grid.cells = xmalloc(row_ptr_size).cast();

    // Allocate each row (zero-initialized)
    for i in 0..height {
        let row = xcalloc(width as usize, std::mem::size_of::<UCell>());
        *grid.cells.add(i as usize) = row.cast();
    }

    grid.width = width;
    grid.height = height;
}

/// Clear the entire grid.
///
/// Sets all cells to space with attribute 0.
///
/// # Safety
/// - `grid` must be a valid pointer to an initialized `UGrid` with allocated cells
#[no_mangle]
pub unsafe extern "C" fn rs_ugrid_clear(grid: *mut UGrid) {
    let g = &*grid;
    clear_region(grid, 0, g.height - 1, 0, g.width - 1, 0);
}

/// Clear a chunk of a row.
///
/// Clears cells from `col` to `endcol - 1` on the specified row.
///
/// # Safety
/// - `grid` must be a valid pointer to an initialized `UGrid` with allocated cells
/// - `row` must be within [0, height)
/// - `col` and `endcol` must be within [0, width]
#[no_mangle]
pub unsafe extern "C" fn rs_ugrid_clear_chunk(
    grid: *mut UGrid,
    row: c_int,
    col: c_int,
    endcol: c_int,
    attr: SattrT,
) {
    clear_region(grid, row, row, col, endcol - 1, attr);
}

/// Set the cursor position in the grid.
///
/// # Safety
/// - `grid` must be a valid pointer to a `UGrid`
#[no_mangle]
pub unsafe extern "C" fn rs_ugrid_goto(grid: *mut UGrid, row: c_int, col: c_int) {
    (*grid).row = row;
    (*grid).col = col;
}

/// Scroll a region of the grid.
///
/// Scrolls the rectangular region defined by `top`, `bot`, `left`, `right`
/// by `count` lines. Positive count scrolls up (moves lines down),
/// negative count scrolls down (moves lines up).
///
/// # Safety
/// - `grid` must be a valid pointer to an initialized `UGrid` with allocated cells
/// - The region must be within the grid bounds
/// - `count` must be less than `bot - top + 1` in absolute value
#[no_mangle]
pub unsafe extern "C" fn rs_ugrid_scroll(
    grid: *mut UGrid,
    top: c_int,
    bot: c_int,
    left: c_int,
    right: c_int,
    count: c_int,
) {
    let grid = &*grid;

    // Compute start/stop/step for the loop
    let (start, stop, step): (c_int, c_int, c_int) = if count > 0 {
        (top, bot - count + 1, 1)
    } else {
        (bot, top - count - 1, -1)
    };

    // Copy cell data
    let mut i = start;
    while i != stop {
        let target_row = (*grid.cells.add(i as usize)).add(left as usize);
        let source_row = (*grid.cells.add((i + count) as usize)).add(left as usize);

        debug_assert!(right >= left && left >= 0);
        ptr::copy_nonoverlapping(source_row, target_row, (right - left + 1) as usize);

        i += step;
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schar_from_ascii() {
        let space = schar_from_ascii(b' ');
        // On little-endian, space (0x20) should be stored as 0x00000020
        // On big-endian, space (0x20) should be stored as 0x20000000
        #[cfg(target_endian = "little")]
        assert_eq!(space, 0x20);
        #[cfg(target_endian = "big")]
        assert_eq!(space, 0x20000000);
    }

    #[test]
    fn test_space_schar_constant() {
        assert_eq!(SPACE_SCHAR, schar_from_ascii(b' '));
    }
}
