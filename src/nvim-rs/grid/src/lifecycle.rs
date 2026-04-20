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

/// Access `WinStruct` fields from a raw `win_T` pointer.
#[allow(clippy::missing_const_for_fn)]
#[inline]
unsafe fn win_ref_raw<'a>(wp: *mut std::ffi::c_void) -> &'a nvim_window::win_struct::WinStruct {
    nvim_window::win_struct::win_ref(nvim_window::WinHandle::from_ptr(wp))
}
/// Mutable access to `WinStruct` fields from a raw `win_T` pointer.
#[inline]
unsafe fn win_mut_raw<'a>(wp: *mut std::ffi::c_void) -> &'a mut nvim_window::win_struct::WinStruct {
    nvim_window::win_struct::win_mut(nvim_window::WinHandle::from_ptr(wp))
}

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

    // Array getters for other array types
    // sattr_T = i32, colnr_T = i32
    fn nvim_screengrid_get_attrs(grid: *mut std::ffi::c_void) -> *mut i32;
    fn nvim_screengrid_get_vcols(grid: *mut std::ffi::c_void) -> *mut i32;
    fn nvim_screengrid_get_line_offset(grid: *mut std::ffi::c_void) -> *mut usize;

    // Null-setters for grid arrays
    fn nvim_screengrid_set_chars_null(grid: *mut std::ffi::c_void);
    fn nvim_screengrid_set_attrs_null(grid: *mut std::ffi::c_void);
    fn nvim_screengrid_set_vcols_null(grid: *mut std::ffi::c_void);
    fn nvim_screengrid_set_line_offset_null(grid: *mut std::ffi::c_void);

    // Full pointer setters (used by rs_grid_alloc)
    fn nvim_screengrid_set_chars(grid: *mut std::ffi::c_void, val: *mut u32);
    fn nvim_screengrid_set_attrs(grid: *mut std::ffi::c_void, val: *mut i32);
    fn nvim_screengrid_set_vcols(grid: *mut std::ffi::c_void, val: *mut i32);
    fn nvim_screengrid_set_line_offset(grid: *mut std::ffi::c_void, val: *mut usize);
    fn nvim_screengrid_set_rows(grid: *mut std::ffi::c_void, val: c_int);
    fn nvim_screengrid_set_cols(grid: *mut std::ffi::c_void, val: c_int);

    /// Free memory (Neovim's xfree, safe to call with NULL).
    fn xfree(ptr: *mut std::ffi::c_void);

    /// Allocate memory (Neovim's xmalloc, aborts on OOM).
    fn xmalloc(size: usize) -> *mut std::ffi::c_void;

    /// grid_clear_line (already in Rust but need to call it here).
    fn grid_clear_line(grid: *mut std::ffi::c_void, off: usize, width: c_int, valid: bool);

    /// Update linebuf arrays if wider than current size (Phase 2 helper).
    fn nvim_grid_alloc_update_linebuf(columns: c_int);

    /// Find window in curtab by its grid_alloc handle.
    fn nvim_find_win_by_grid_handle(handle: HandleT) -> *mut std::ffi::c_void;

    // =================================================================
    // Phase 3: win_grid_alloc accessors (w_config.*: opaque, keep extern)
    // =================================================================

    /// Get wp->w_config.border.
    fn nvim_win_get_config_border_flag(wp: *mut std::ffi::c_void) -> c_int;
    /// Get wp->w_grid as GridView*.
    fn nvim_win_get_w_grid(wp: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    /// Get &wp->w_grid_alloc as ScreenGrid*.
    fn nvim_win_get_w_grid_alloc(wp: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    /// Get default_grid.
    fn nvim_get_default_grid() -> *mut std::ffi::c_void;
    /// Get resizing_screen global.
    fn nvim_get_resizing_screen() -> bool;
    /// Wrapper for ui_call_grid_resize.
    fn nvim_call_grid_resize(handle: HandleT, cols: c_int, rows: c_int);
    /// Wrapper for ui_check_cursor_grid.
    #[link_name = "ui_check_cursor_grid"]
    fn nvim_win_ui_check_cursor_grid(grid_handle: c_int);
    /// Allocate zeroed memory.
    #[link_name = "xcalloc"]
    fn nvim_xcalloc(count: usize, size: usize) -> *mut std::ffi::c_void;
    /// Get sizeof(wline_T).
    fn nvim_wline_T_size() -> usize;
    /// Set GridView target.
    fn nvim_gridview_set_target(view: *mut std::ffi::c_void, target: *mut std::ffi::c_void);
    /// Set GridView row_offset.
    fn nvim_gridview_set_row_offset(view: *mut std::ffi::c_void, val: c_int);
    /// Set GridView col_offset.
    fn nvim_gridview_set_col_offset(view: *mut std::ffi::c_void, val: c_int);
    /// Check ui_has(kUIMultigrid) -- exported from Rust window crate.
    fn nvim_ui_has_multigrid() -> c_int;
    /// grid_alloc (now in Rust but callable via C ABI).
    fn grid_alloc(grid: *mut std::ffi::c_void, rows: c_int, cols: c_int, copy: bool, valid: bool);
    /// grid_free (now in Rust but callable via C ABI).
    fn grid_free(grid: *mut std::ffi::c_void);
    /// grid_invalidate (already in Rust via export_name).
    fn grid_invalidate(grid: *mut std::ffi::c_void);
    /// Get ScreenGrid valid field.
    fn nvim_screengrid_get_valid(grid: *mut std::ffi::c_void) -> bool;
    /// Set ScreenGrid valid field.
    fn nvim_screengrid_set_valid(grid: *mut std::ffi::c_void, val: bool);
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
// Allocation Validation Helpers (Phase 1)
// =============================================================================

/// Check if allocation parameters are valid for grid_alloc.
///
/// Returns 1 if parameters are valid, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_grid_alloc_params_valid(rows: c_int, cols: c_int) -> c_int {
    // Both dimensions must be non-negative
    if rows < 0 || cols < 0 {
        return 0;
    }
    // At least one dimension must be non-zero for a valid grid
    // (though 0x0 is technically valid for clearing)
    1
}

/// Calculate the number of bytes needed for chars/attrs/vcols arrays.
///
/// Returns bytes needed, or 0 on overflow/invalid params.
#[no_mangle]
pub extern "C" fn rs_grid_array_bytes(rows: c_int, cols: c_int, elem_size: usize) -> usize {
    if rows < 0 || cols < 0 {
        return 0;
    }

    let Some(cells) = (rows as usize).checked_mul(cols as usize) else {
        return 0;
    };

    cells.checked_mul(elem_size).unwrap_or(0)
}

/// Calculate line_offset array entry for a row.
///
/// This is used during grid_alloc to initialize the line_offset array.
#[no_mangle]
pub extern "C" fn rs_grid_row_offset(row: c_int, cols: c_int) -> usize {
    if row < 0 || cols < 0 {
        return 0;
    }
    (row as usize) * (cols as usize)
}

/// Determine if content should be copied during resize.
///
/// Returns 1 if copy is requested and we have valid source data.
#[no_mangle]
pub extern "C" fn rs_grid_should_copy_on_resize(
    copy: c_int,
    new_row: c_int,
    old_rows: c_int,
    old_chars_not_null: c_int,
) -> c_int {
    c_int::from(copy != 0 && new_row < old_rows && old_chars_not_null != 0)
}

/// Calculate number of elements to copy per row during resize.
///
/// Returns the minimum of old_cols and new_cols.
#[no_mangle]
pub extern "C" fn rs_grid_copy_width(old_cols: c_int, new_cols: c_int) -> c_int {
    if old_cols <= 0 || new_cols <= 0 {
        return 0;
    }
    old_cols.min(new_cols)
}

// =============================================================================
// Grid State Queries (Phase 1)
// =============================================================================

/// Check if grid has valid dimensions (rows > 0 && cols > 0).
#[no_mangle]
pub unsafe extern "C" fn rs_grid_has_valid_dims(grid: *mut std::ffi::c_void) -> c_int {
    if grid.is_null() {
        return 0;
    }

    let rows = nvim_screengrid_get_rows(grid);
    let cols = nvim_screengrid_get_cols(grid);

    c_int::from(rows > 0 && cols > 0)
}

/// Check if grid needs allocation (no chars array).
#[no_mangle]
pub unsafe extern "C" fn rs_grid_needs_alloc(grid: *mut std::ffi::c_void) -> c_int {
    if grid.is_null() {
        return 1;
    }

    let chars = nvim_screengrid_get_chars(grid);
    c_int::from(chars.is_null())
}

/// Check if grid dimensions match requested size.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_dims_match(
    grid: *mut std::ffi::c_void,
    rows: c_int,
    cols: c_int,
) -> c_int {
    if grid.is_null() {
        return 0;
    }

    let grid_rows = nvim_screengrid_get_rows(grid);
    let grid_cols = nvim_screengrid_get_cols(grid);

    c_int::from(grid_rows == rows && grid_cols == cols)
}

// =============================================================================
// Phase 1: grid_free and get_win_by_grid_handle
// =============================================================================

/// Free all allocated arrays in a ScreenGrid and null out the pointers.
///
/// Matches C's `grid_free()`. Safe to call on an already-freed or zeroed grid.
///
/// # Safety
/// `grid` must be a valid `ScreenGrid*` or null.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_free(grid: *mut std::ffi::c_void) {
    if grid.is_null() {
        return;
    }
    let chars = nvim_screengrid_get_chars(grid);
    let attrs = nvim_screengrid_get_attrs(grid);
    let vcols = nvim_screengrid_get_vcols(grid);
    let line_offset = nvim_screengrid_get_line_offset(grid);

    xfree(chars.cast());
    xfree(attrs.cast());
    xfree(vcols.cast());
    xfree(line_offset.cast());

    nvim_screengrid_set_chars_null(grid);
    nvim_screengrid_set_attrs_null(grid);
    nvim_screengrid_set_vcols_null(grid);
    nvim_screengrid_set_line_offset_null(grid);
}

/// Find the window in the current tabpage whose `w_grid_alloc.handle` equals `handle`.
///
/// Returns a pointer to the `win_T` or null if not found.
/// Matches C's `get_win_by_grid_handle()`.
///
/// # Safety
/// Must be called from the main Neovim thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_win_by_grid_handle(handle: HandleT) -> *mut std::ffi::c_void {
    nvim_find_win_by_grid_handle(handle)
}

// =============================================================================
// Phase 2: grid_alloc
// =============================================================================

/// Allocate or resize a ScreenGrid's arrays.
///
/// Matches C's `grid_alloc()`. Allocates new char/attr/vcol/line_offset arrays,
/// optionally copies content from the old grid, frees the old arrays, and
/// updates the linebuf globals if this grid is wider than any previous grid.
///
/// # Safety
/// `grid` must be a valid `ScreenGrid*`. `rows` and `columns` must be >= 0.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_alloc(
    grid: *mut std::ffi::c_void,
    rows: c_int,
    columns: c_int,
    copy: bool,
    valid: bool,
) {
    debug_assert!(rows >= 0 && columns >= 0);

    let ncells = (rows as usize) * (columns as usize);

    // Save old pointers and dimensions before overwriting
    let old_chars = nvim_screengrid_get_chars(grid);
    let old_attrs = nvim_screengrid_get_attrs(grid);
    let old_vcols = nvim_screengrid_get_vcols(grid);
    let old_line_offset = nvim_screengrid_get_line_offset(grid);
    let old_rows = nvim_screengrid_get_rows(grid);
    let old_cols = nvim_screengrid_get_cols(grid);

    // Allocate new arrays
    let new_chars = xmalloc(ncells * 4).cast::<u32>(); // sizeof(schar_T) = 4
    let new_attrs = xmalloc(ncells * 4).cast::<i32>(); // sizeof(sattr_T) = 4
    let new_vcols = xmalloc(ncells * 4).cast::<i32>(); // sizeof(colnr_T) = 4
    std::ptr::write_bytes(new_vcols, 0xFF_u8, ncells); // init to -1; count is in T elements
    let new_line_offset = xmalloc((rows as usize) * std::mem::size_of::<usize>()).cast::<usize>();

    // Update grid dimensions and array pointers
    nvim_screengrid_set_rows(grid, rows);
    nvim_screengrid_set_cols(grid, columns);
    nvim_screengrid_set_chars(grid, new_chars);
    nvim_screengrid_set_attrs(grid, new_attrs);
    nvim_screengrid_set_vcols(grid, new_vcols);
    nvim_screengrid_set_line_offset(grid, new_line_offset);

    // Initialize each row
    for new_row in 0..rows {
        let offset = (new_row as usize) * (columns as usize);
        *new_line_offset.add(new_row as usize) = offset;

        grid_clear_line(grid, offset, columns, valid);

        if copy && new_row < old_rows && !old_chars.is_null() {
            let len = old_cols.min(columns) as usize;
            let old_row_offset = *old_line_offset.add(new_row as usize);
            std::ptr::copy(old_chars.add(old_row_offset), new_chars.add(offset), len);
            std::ptr::copy(old_attrs.add(old_row_offset), new_attrs.add(offset), len);
            std::ptr::copy(old_vcols.add(old_row_offset), new_vcols.add(offset), len);
        }
    }

    // Free old arrays (manually, to avoid using grid_free which would use new ptrs)
    xfree(old_chars.cast());
    xfree(old_attrs.cast());
    xfree(old_vcols.cast());
    xfree(old_line_offset.cast());

    // Update linebuf if this grid is wider than current linebuf
    nvim_grid_alloc_update_linebuf(columns);
}

// =============================================================================
// Phase 3: win_grid_alloc
// =============================================================================

/// Allocate or resize the per-window grid.
///
/// Matches C's `win_grid_alloc()`. Manages the ScreenGrid for multigrid mode
/// and updates the GridView offsets for both single-grid and multigrid layouts.
///
/// # Safety
/// `wp` must be a valid `win_T*`.
#[no_mangle]
pub unsafe extern "C" fn rs_win_grid_alloc(wp: *mut std::ffi::c_void) {
    let grid_view = nvim_win_get_w_grid(wp);
    let grid_alloc_ptr = nvim_win_get_w_grid_alloc(wp);

    let total_rows = win_ref_raw(wp).w_height_outer;
    let total_cols = win_ref_raw(wp).w_width_outer;

    let want_allocation = nvim_ui_has_multigrid() != 0 || win_ref_raw(wp).w_floating;
    let has_allocation = !nvim_screengrid_get_chars(grid_alloc_ptr).is_null();

    // Grow w_lines array if view_height exceeds current allocation
    let view_height = win_ref_raw(wp).w_view_height;
    let lines_size = win_ref_raw(wp).w_lines_size;
    if view_height > lines_size {
        win_mut_raw(wp).w_lines_valid = 0;
        let old_lines = win_ref_raw(wp).w_lines;
        xfree(old_lines);
        let wline_sz = nvim_wline_T_size();
        let new_lines = nvim_xcalloc((view_height as usize) + 1, wline_sz);
        win_mut_raw(wp).w_lines = new_lines;
        win_mut_raw(wp).w_lines_size = view_height;
    }

    let mut was_resized = false;
    let ga_rows = nvim_screengrid_get_rows(grid_alloc_ptr);
    let ga_cols = nvim_screengrid_get_cols(grid_alloc_ptr);
    let ga_valid = nvim_screengrid_get_valid(grid_alloc_ptr);

    if want_allocation && (!has_allocation || ga_rows != total_rows || ga_cols != total_cols) {
        grid_alloc(grid_alloc_ptr, total_rows, total_cols, ga_valid, false);
        nvim_screengrid_set_valid(grid_alloc_ptr, true);
        if win_ref_raw(wp).w_floating && nvim_win_get_config_border_flag(wp) != 0 {
            win_mut_raw(wp).w_redr_border = true;
        }
        was_resized = true;
    } else if !want_allocation && has_allocation {
        grid_free(grid_alloc_ptr);
        nvim_screengrid_set_valid(grid_alloc_ptr, false);
        was_resized = true;
    } else if want_allocation && has_allocation && !ga_valid {
        grid_invalidate(grid_alloc_ptr);
        nvim_screengrid_set_valid(grid_alloc_ptr, true);
    }

    let winrow = win_ref_raw(wp).w_winrow;
    let wincol = win_ref_raw(wp).w_wincol;
    let winrow_off = win_ref_raw(wp).w_winrow_off;
    let wincol_off = win_ref_raw(wp).w_wincol_off;

    if want_allocation {
        nvim_gridview_set_target(grid_view, grid_alloc_ptr);
        nvim_gridview_set_row_offset(grid_view, winrow_off);
        nvim_gridview_set_col_offset(grid_view, wincol_off);
    } else {
        let default_grid = nvim_get_default_grid();
        nvim_gridview_set_target(grid_view, default_grid);
        nvim_gridview_set_row_offset(grid_view, winrow + winrow_off);
        nvim_gridview_set_col_offset(grid_view, wincol + wincol_off);
    }

    if (nvim_get_resizing_screen() || was_resized) && want_allocation {
        let ga_handle = nvim_screengrid_get_handle(grid_alloc_ptr);
        let ga_cols_new = nvim_screengrid_get_cols(grid_alloc_ptr);
        let ga_rows_new = nvim_screengrid_get_rows(grid_alloc_ptr);
        nvim_call_grid_resize(ga_handle, ga_cols_new, ga_rows_new);
        nvim_win_ui_check_cursor_grid(ga_handle);
    }
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

    #[test]
    fn test_grid_alloc_params_valid() {
        assert_eq!(rs_grid_alloc_params_valid(10, 80), 1);
        assert_eq!(rs_grid_alloc_params_valid(0, 0), 1); // 0x0 is valid for clearing
        assert_eq!(rs_grid_alloc_params_valid(-1, 80), 0);
        assert_eq!(rs_grid_alloc_params_valid(10, -1), 0);
    }

    #[test]
    fn test_grid_array_bytes() {
        // 10 rows * 80 cols = 800 cells
        assert_eq!(rs_grid_array_bytes(10, 80, 4), 3200); // 800 * 4
        assert_eq!(rs_grid_array_bytes(10, 80, 8), 6400); // 800 * 8
        assert_eq!(rs_grid_array_bytes(0, 80, 4), 0);
        assert_eq!(rs_grid_array_bytes(-1, 80, 4), 0);
    }

    #[test]
    fn test_grid_row_offset() {
        assert_eq!(rs_grid_row_offset(0, 80), 0);
        assert_eq!(rs_grid_row_offset(1, 80), 80);
        assert_eq!(rs_grid_row_offset(5, 80), 400);
        assert_eq!(rs_grid_row_offset(-1, 80), 0);
    }

    #[test]
    fn test_grid_should_copy_on_resize() {
        // copy=true, row<old_rows, has chars
        assert_eq!(rs_grid_should_copy_on_resize(1, 5, 10, 1), 1);
        // copy=false
        assert_eq!(rs_grid_should_copy_on_resize(0, 5, 10, 1), 0);
        // row >= old_rows
        assert_eq!(rs_grid_should_copy_on_resize(1, 15, 10, 1), 0);
        // no old chars
        assert_eq!(rs_grid_should_copy_on_resize(1, 5, 10, 0), 0);
    }

    #[test]
    fn test_grid_copy_width() {
        assert_eq!(rs_grid_copy_width(80, 120), 80);
        assert_eq!(rs_grid_copy_width(120, 80), 80);
        assert_eq!(rs_grid_copy_width(0, 80), 0);
        assert_eq!(rs_grid_copy_width(-1, 80), 0);
    }
}
