//! Grid Lifecycle Management
//!
//! This module provides grid allocation and lifecycle management functions.
//! Phase 170 of Rust migration.
//!
//! Note: Actual memory allocation is done in C using xmalloc/xfree since
//! Neovim has its own memory management infrastructure. This module provides
//! the supporting calculations and validation logic.

use std::ffi::c_int;

/// Type alias for grid handle (matches C's `handle_T`).
type HandleT = i32;

/// Minimum grid dimensions
const MIN_GRID_ROWS: c_int = 1;
const MIN_GRID_COLS: c_int = 1;

/// Maximum reasonable grid dimensions to prevent overflow
const MAX_GRID_ROWS: c_int = 10000;
const MAX_GRID_COLS: c_int = 10000;

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    fn nvim_screengrid_get_handle(grid: *mut std::ffi::c_void) -> HandleT;
    fn nvim_screengrid_get_rows(grid: *mut std::ffi::c_void) -> c_int;
    fn nvim_screengrid_get_cols(grid: *mut std::ffi::c_void) -> c_int;
    fn nvim_screengrid_get_chars(grid: *mut std::ffi::c_void) -> *mut u32;
}

// =============================================================================
// Grid Dimension Validation
// =============================================================================

/// Validate grid dimensions for allocation.
///
/// Returns 1 if dimensions are valid for allocation, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_grid_dims_valid(rows: c_int, cols: c_int) -> c_int {
    c_int::from(
        (MIN_GRID_ROWS..=MAX_GRID_ROWS).contains(&rows)
            && (MIN_GRID_COLS..=MAX_GRID_COLS).contains(&cols),
    )
}

/// Check if grid dimensions would overflow when calculating cell count.
///
/// Returns 1 if safe, 0 if would overflow.
#[no_mangle]
pub extern "C" fn rs_grid_dims_safe(rows: c_int, cols: c_int) -> c_int {
    if rows <= 0 || cols <= 0 {
        return 0;
    }
    // Check for multiplication overflow
    let rows_usize = rows as usize;
    let cols_usize = cols as usize;
    c_int::from(rows_usize.checked_mul(cols_usize).is_some())
}

/// Calculate total cells needed for a grid.
///
/// Returns rows * cols, or 0 if either dimension is non-positive.
#[no_mangle]
pub extern "C" fn rs_grid_total_cells(rows: c_int, cols: c_int) -> usize {
    if rows <= 0 || cols <= 0 {
        return 0;
    }
    (rows as usize) * (cols as usize)
}

/// Calculate memory size needed for grid chars array.
///
/// Returns bytes needed for schar_T array (4 bytes per cell).
#[no_mangle]
pub extern "C" fn rs_grid_chars_size(rows: c_int, cols: c_int) -> usize {
    rs_grid_total_cells(rows, cols) * 4 // sizeof(schar_T) = 4
}

/// Calculate memory size needed for grid attrs array.
///
/// Returns bytes needed for sattr_T array (4 bytes per cell).
#[no_mangle]
pub extern "C" fn rs_grid_attrs_size(rows: c_int, cols: c_int) -> usize {
    rs_grid_total_cells(rows, cols) * 4 // sizeof(sattr_T) = 4
}

/// Calculate memory size needed for grid vcols array.
///
/// Returns bytes needed for colnr_T array (4 bytes per cell).
#[no_mangle]
pub extern "C" fn rs_grid_vcols_size(rows: c_int, cols: c_int) -> usize {
    rs_grid_total_cells(rows, cols) * 4 // sizeof(colnr_T) = 4
}

/// Calculate memory size needed for grid line_offset array.
///
/// Returns bytes needed for size_t array (8 bytes per row on 64-bit).
#[no_mangle]
pub extern "C" fn rs_grid_line_offset_size(rows: c_int) -> usize {
    if rows <= 0 {
        return 0;
    }
    (rows as usize) * std::mem::size_of::<usize>()
}

/// Calculate total memory needed for a grid allocation.
///
/// Returns total bytes for all arrays: chars, attrs, vcols, line_offset.
#[no_mangle]
pub extern "C" fn rs_grid_total_memory(rows: c_int, cols: c_int) -> usize {
    rs_grid_chars_size(rows, cols)
        + rs_grid_attrs_size(rows, cols)
        + rs_grid_vcols_size(rows, cols)
        + rs_grid_line_offset_size(rows)
}

// =============================================================================
// Grid Resize Calculations
// =============================================================================

/// Check if a grid needs resizing.
///
/// Returns 1 if current dimensions don't match requested dimensions.
#[no_mangle]
pub extern "C" fn rs_grid_needs_resize(
    current_rows: c_int,
    current_cols: c_int,
    new_rows: c_int,
    new_cols: c_int,
) -> c_int {
    c_int::from(current_rows != new_rows || current_cols != new_cols)
}

/// Calculate how many rows can be copied when resizing.
///
/// Returns the minimum of old and new row counts.
#[no_mangle]
pub extern "C" fn rs_grid_copy_rows(old_rows: c_int, new_rows: c_int) -> c_int {
    if old_rows < new_rows {
        old_rows
    } else {
        new_rows
    }
}

/// Calculate how many columns can be copied when resizing.
///
/// Returns the minimum of old and new column counts.
#[no_mangle]
pub extern "C" fn rs_grid_copy_cols(old_cols: c_int, new_cols: c_int) -> c_int {
    if old_cols < new_cols {
        old_cols
    } else {
        new_cols
    }
}

/// Check if a resize involves growing the grid.
///
/// Returns 1 if new dimensions are larger in either direction.
#[no_mangle]
pub extern "C" fn rs_grid_is_growing(
    old_rows: c_int,
    old_cols: c_int,
    new_rows: c_int,
    new_cols: c_int,
) -> c_int {
    c_int::from(new_rows > old_rows || new_cols > old_cols)
}

/// Check if a resize involves shrinking the grid.
///
/// Returns 1 if new dimensions are smaller in either direction.
#[no_mangle]
pub extern "C" fn rs_grid_is_shrinking(
    old_rows: c_int,
    old_cols: c_int,
    new_rows: c_int,
    new_cols: c_int,
) -> c_int {
    c_int::from(new_rows < old_rows || new_cols < old_cols)
}

// =============================================================================
// Grid Handle Management
// =============================================================================

/// Generate next grid handle value.
///
/// Uses a simple incrementing counter. The counter starts at 1 since 0
/// typically indicates "no handle" or "default grid".
static NEXT_GRID_HANDLE: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(2); // Start at 2, 1 is usually default

/// Allocate a new grid handle.
///
/// Returns a unique handle value for grid identification.
#[no_mangle]
pub extern "C" fn rs_grid_handle_alloc() -> HandleT {
    NEXT_GRID_HANDLE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// Get the current handle counter value.
///
/// For debugging/testing purposes.
#[no_mangle]
pub extern "C" fn rs_grid_handle_current() -> HandleT {
    NEXT_GRID_HANDLE.load(std::sync::atomic::Ordering::Relaxed)
}

/// Check if a handle is valid (non-zero).
#[no_mangle]
pub extern "C" fn rs_grid_handle_valid(handle: HandleT) -> c_int {
    c_int::from(handle != 0)
}

/// Check if handle represents the default grid (handle == 1).
#[no_mangle]
pub extern "C" fn rs_grid_handle_is_default(handle: HandleT) -> c_int {
    c_int::from(handle == 1)
}

// =============================================================================
// Grid Allocation State
// =============================================================================

/// Check if a grid pointer is allocated (has chars).
///
/// # Safety
/// `grid` must be a valid ScreenGrid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_is_allocated(grid: *mut std::ffi::c_void) -> c_int {
    if grid.is_null() {
        return 0;
    }
    let chars = nvim_screengrid_get_chars(grid);
    c_int::from(!chars.is_null())
}

/// Check if grid dimensions are non-zero.
///
/// # Safety
/// `grid` must be a valid ScreenGrid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_has_size(grid: *mut std::ffi::c_void) -> c_int {
    if grid.is_null() {
        return 0;
    }
    let rows = nvim_screengrid_get_rows(grid);
    let cols = nvim_screengrid_get_cols(grid);
    c_int::from(rows > 0 && cols > 0)
}

/// Get effective grid handle (0 if null or unallocated).
///
/// # Safety
/// `grid` must be a valid ScreenGrid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_effective_handle(grid: *mut std::ffi::c_void) -> HandleT {
    if grid.is_null() {
        return 0;
    }
    nvim_screengrid_get_handle(grid)
}

// =============================================================================
// Line Offset Calculations
// =============================================================================

/// Calculate line offset for a given row.
///
/// Returns the offset into the chars/attrs arrays for the start of `row`.
#[no_mangle]
pub extern "C" fn rs_calc_line_offset(row: c_int, cols: c_int) -> usize {
    if row < 0 || cols <= 0 {
        return 0;
    }
    (row as usize) * (cols as usize)
}

/// Initialize line offset array for a grid.
///
/// Fills `offsets` with correct values for each row.
///
/// # Safety
/// `offsets` must point to an array of at least `rows` elements.
#[no_mangle]
pub unsafe extern "C" fn rs_init_line_offsets(offsets: *mut usize, rows: c_int, cols: c_int) {
    if offsets.is_null() || rows <= 0 || cols <= 0 {
        return;
    }
    for row in 0..rows {
        *offsets.add(row as usize) = (row as usize) * (cols as usize);
    }
}

// =============================================================================
// Grid Copy Helpers
// =============================================================================

/// Calculate bytes to copy for a row during resize.
///
/// Returns the number of bytes to copy (min of old and new column widths * cell size).
#[no_mangle]
pub extern "C" fn rs_row_copy_bytes(old_cols: c_int, new_cols: c_int, cell_size: usize) -> usize {
    let copy_cols = if old_cols < new_cols {
        old_cols
    } else {
        new_cols
    };
    if copy_cols <= 0 {
        return 0;
    }
    (copy_cols as usize) * cell_size
}

/// Check if row content should be preserved during resize.
///
/// Returns 1 if the row exists in both old and new grid and old grid has chars.
#[no_mangle]
pub extern "C" fn rs_should_preserve_row(
    row: c_int,
    old_rows: c_int,
    has_old_chars: c_int,
) -> c_int {
    c_int::from(row < old_rows && has_old_chars != 0)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_dims_valid() {
        assert_eq!(rs_grid_dims_valid(10, 80), 1);
        assert_eq!(rs_grid_dims_valid(0, 80), 0);
        assert_eq!(rs_grid_dims_valid(10, 0), 0);
        assert_eq!(rs_grid_dims_valid(-1, 80), 0);
        assert_eq!(rs_grid_dims_valid(10, -1), 0);
        assert_eq!(rs_grid_dims_valid(MAX_GRID_ROWS + 1, 80), 0);
        assert_eq!(rs_grid_dims_valid(10, MAX_GRID_COLS + 1), 0);
    }

    #[test]
    fn test_grid_dims_safe() {
        assert_eq!(rs_grid_dims_safe(100, 200), 1);
        assert_eq!(rs_grid_dims_safe(0, 200), 0);
        assert_eq!(rs_grid_dims_safe(100, 0), 0);
        assert_eq!(rs_grid_dims_safe(-1, 200), 0);
    }

    #[test]
    fn test_grid_total_cells() {
        assert_eq!(rs_grid_total_cells(10, 80), 800);
        assert_eq!(rs_grid_total_cells(0, 80), 0);
        assert_eq!(rs_grid_total_cells(10, 0), 0);
    }

    #[test]
    fn test_grid_memory_sizes() {
        // 10x80 = 800 cells
        assert_eq!(rs_grid_chars_size(10, 80), 3200); // 800 * 4
        assert_eq!(rs_grid_attrs_size(10, 80), 3200); // 800 * 4
        assert_eq!(rs_grid_vcols_size(10, 80), 3200); // 800 * 4
        assert_eq!(rs_grid_line_offset_size(10), 80); // 10 * 8 on 64-bit
    }

    #[test]
    fn test_grid_needs_resize() {
        assert_eq!(rs_grid_needs_resize(10, 80, 10, 80), 0);
        assert_eq!(rs_grid_needs_resize(10, 80, 20, 80), 1);
        assert_eq!(rs_grid_needs_resize(10, 80, 10, 120), 1);
    }

    #[test]
    fn test_grid_copy_dims() {
        assert_eq!(rs_grid_copy_rows(10, 20), 10);
        assert_eq!(rs_grid_copy_rows(20, 10), 10);
        assert_eq!(rs_grid_copy_cols(80, 120), 80);
        assert_eq!(rs_grid_copy_cols(120, 80), 80);
    }

    #[test]
    fn test_grid_growing_shrinking() {
        assert_eq!(rs_grid_is_growing(10, 80, 20, 80), 1);
        assert_eq!(rs_grid_is_growing(10, 80, 10, 120), 1);
        assert_eq!(rs_grid_is_growing(20, 120, 10, 80), 0);
        assert_eq!(rs_grid_is_shrinking(20, 120, 10, 80), 1);
        assert_eq!(rs_grid_is_shrinking(10, 80, 20, 120), 0);
    }

    #[test]
    fn test_grid_handle() {
        // Handle allocation returns incrementing values
        let h1 = rs_grid_handle_alloc();
        let h2 = rs_grid_handle_alloc();
        assert!(h2 > h1);

        assert_eq!(rs_grid_handle_valid(0), 0);
        assert_eq!(rs_grid_handle_valid(1), 1);
        assert_eq!(rs_grid_handle_is_default(1), 1);
        assert_eq!(rs_grid_handle_is_default(2), 0);
    }

    #[test]
    fn test_calc_line_offset() {
        assert_eq!(rs_calc_line_offset(0, 80), 0);
        assert_eq!(rs_calc_line_offset(1, 80), 80);
        assert_eq!(rs_calc_line_offset(5, 80), 400);
        assert_eq!(rs_calc_line_offset(-1, 80), 0);
        assert_eq!(rs_calc_line_offset(5, 0), 0);
    }

    #[test]
    fn test_row_copy_bytes() {
        assert_eq!(rs_row_copy_bytes(80, 120, 4), 320); // copy 80 cols * 4 bytes
        assert_eq!(rs_row_copy_bytes(120, 80, 4), 320); // copy 80 cols * 4 bytes
        assert_eq!(rs_row_copy_bytes(0, 80, 4), 0);
    }

    #[test]
    fn test_should_preserve_row() {
        assert_eq!(rs_should_preserve_row(5, 10, 1), 1);
        assert_eq!(rs_should_preserve_row(15, 10, 1), 0); // row >= old_rows
        assert_eq!(rs_should_preserve_row(5, 10, 0), 0); // no old chars
    }
}
