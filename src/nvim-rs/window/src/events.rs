//! Window events and UI update functions.
//!
//! This module provides Rust implementations of window event and UI
//! functions from `src/nvim/window.c`.
//!
//! These functions handle UI synchronization, buffer switching, and event triggers.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::{c_double, c_int, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::WinHandle;

/// Opaque handle to C's ScreenGrid (local definition for this module).
///
/// Note: The compositor crate has its own identical type. We redeclare locally
/// to avoid adding a crate dependency.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct ScreenGridHandle(*mut c_void);

impl ScreenGridHandle {
    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// ---------------------------------------------------------------------------
// Float anchor constants
// NOTE: These match buffer_defs.h exactly:
//   kFloatAnchorEast  = 1
//   kFloatAnchorSouth = 2
//
// WARNING: The winfloat crate's FloatAnchor bitflags have SWAPPED values
// (SOUTH=0b01=1, EAST=0b10=2). Do NOT use that crate's type here.
// ---------------------------------------------------------------------------
const K_FLOAT_ANCHOR_EAST: c_int = 1;
const K_FLOAT_ANCHOR_SOUTH: c_int = 2;

// Float relative constants (match buffer_defs.h and winfloat crate)
const K_FLOAT_RELATIVE_WINDOW: c_int = 1;
const K_FLOAT_RELATIVE_TABLINE: c_int = 4;
const K_FLOAT_RELATIVE_LASTSTATUS: c_int = 5;

// Z-index threshold for showing above 'cmdheight' area
const K_ZINDEX_MESSAGES: c_int = 200;

// UI extension enum value for kUIMultigrid
const K_UI_MULTIGRID: c_int = 6;

// Redraw type constant (matches UPD_NOT_VALID = 40)
const UPD_NOT_VALID: c_int = 40;

// ---------------------------------------------------------------------------
// FFI declarations
// ---------------------------------------------------------------------------

extern "C" {
    // Window field accessors
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;
    fn nvim_win_get_handle(wp: WinHandle) -> c_int;
    fn nvim_win_get_winrow(wp: WinHandle) -> c_int;
    fn nvim_win_get_wincol(wp: WinHandle) -> c_int;
    fn nvim_win_set_winrow(wp: WinHandle, val: c_int);
    fn nvim_win_set_wincol(wp: WinHandle, val: c_int);
    fn nvim_win_set_pos_changed(wp: WinHandle, val: c_int);
    fn nvim_win_get_pos_changed(wp: WinHandle) -> c_int;
    fn nvim_win_get_redr_type(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_width(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_height_outer(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_width_outer(wp: WinHandle) -> c_int;

    // WinConfig field accessors
    fn nvim_win_get_config_external(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_hide(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_relative(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_window(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_row(wp: WinHandle) -> c_double;
    fn nvim_win_get_config_col(wp: WinHandle) -> c_double;
    fn nvim_win_get_config_anchor(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_fixed(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_mouse_flag(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_zindex(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_bufpos_lnum(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_bufpos_col(wp: WinHandle) -> c_int;

    // win_T::w_grid_alloc accessors (via ScreenGridHandle)
    fn nvim_win_get_grid_alloc(wp: WinHandle) -> ScreenGridHandle;
    fn nvim_screengrid_get_handle(grid: ScreenGridHandle) -> c_int;
    fn nvim_screengrid_get_comp_row(grid: ScreenGridHandle) -> c_int;
    fn nvim_screengrid_get_comp_col(grid: ScreenGridHandle) -> c_int;
    fn nvim_screengrid_set_comp_row(grid: ScreenGridHandle, val: c_int);
    fn nvim_screengrid_set_comp_col(grid: ScreenGridHandle, val: c_int);
    fn nvim_screengrid_get_comp_index(grid: ScreenGridHandle) -> usize;
    fn nvim_screengrid_get_zindex(grid: ScreenGridHandle) -> c_int;
    fn nvim_screengrid_set_zindex(grid: ScreenGridHandle, val: c_int);
    fn nvim_screengrid_set_valid(grid: ScreenGridHandle, val: bool);
    fn nvim_screengrid_set_mouse_enabled(grid: ScreenGridHandle, val: bool);
    fn nvim_screengrid_get_chars(grid: ScreenGridHandle) -> *mut c_void;

    // win_T::w_grid accessor (for grid_adjust)
    fn nvim_win_get_w_grid(wp: WinHandle) -> *mut c_void;

    // grid_adjust: takes GridView*, returns ScreenGrid*, writes row/col offsets
    fn rs_grid_adjust(view: *mut c_void, row_off: *mut c_int, col_off: *mut c_int) -> *mut c_void;

    // textpos2screenpos
    fn rs_textpos2screenpos(
        wp: WinHandle,
        pos: *const PosT,
        rowp: *mut c_int,
        scolp: *mut c_int,
        ccolp: *mut c_int,
        ecolp: *mut c_int,
        local: c_int,
    );

    // Buffer line count (for clamping lnum)
    fn nvim_win_buf_line_count(wp: WinHandle) -> i32;

    // Window validity
    fn rs_win_valid(win: WinHandle) -> c_int;

    // Find window by integer handle
    fn nvim_handle_get_window(handle: c_int) -> WinHandle;

    // win_grid_alloc call
    fn nvim_win_call_win_grid_alloc(wp: WinHandle);

    // Compositor functions
    fn rs_ui_comp_put_grid(
        grid: ScreenGridHandle,
        row: c_int,
        col: c_int,
        height: c_int,
        width: c_int,
        valid: bool,
        on_top: bool,
    ) -> bool;
    fn rs_ui_comp_remove_grid(grid: ScreenGridHandle);
    fn rs_ui_comp_layers_adjust(layer_idx: usize, raise: bool);

    // UI extension check
    fn nvim_get_ui_ext(ext: c_int) -> c_int;

    // UI call wrappers
    fn nvim_win_ui_call_win_pos(
        grid: c_int,
        win: c_int,
        row: c_int,
        col: c_int,
        width: c_int,
        height: c_int,
    );
    fn nvim_win_ui_call_win_float_pos(
        grid_handle: c_int,
        win_handle: c_int,
        anchor: c_int,
        anchor_grid: c_int,
        row: c_double,
        col: c_double,
        mouse: c_int,
        zindex: c_int,
        comp_index: c_int,
        screen_row: c_int,
        screen_col: c_int,
    );
    fn nvim_win_ui_call_win_hide(grid_handle: c_int);
    fn nvim_win_ui_call_win_external_pos(grid_handle: c_int, win_handle: c_int);
    fn nvim_win_ui_check_cursor_grid(grid_handle: c_int);

    // Globals
    fn nvim_get_rows() -> c_int;
    fn nvim_get_columns() -> c_int;
    fn nvim_get_p_ch() -> i64;

    // redraw_later
    fn nvim_redraw_later_wrapper(wp: WinHandle, type_: c_int);

    // Recursive call back through the C wrapper (handles anchor chain correctly)
    fn ui_ext_win_position(wp: WinHandle, validate: bool);
}

/// Position type matching C's `pos_T` (lnum, col, coladd).
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct PosT {
    lnum: i32,
    col: i32,
    coladd: i32,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Get the default grid as a `ScreenGridHandle`.
unsafe fn get_default_grid() -> ScreenGridHandle {
    extern "C" {
        fn nvim_get_default_grid() -> ScreenGridHandle;
    }
    nvim_get_default_grid()
}

/// Wrapper for `rs_last_stl_height` (plines crate export).
unsafe fn last_stl_height(morewin: c_int) -> c_int {
    extern "C" {
        fn rs_last_stl_height(morewin: c_int) -> c_int;
    }
    rs_last_stl_height(morewin)
}

/// Wrapper for `rs_tabline_height` (plines crate export).
unsafe fn tabline_height() -> c_int {
    extern "C" {
        fn rs_tabline_height() -> c_int;
    }
    rs_tabline_height()
}

/// Resolve the row/col offset and anchor grid for a window-relative floating window.
///
/// Positions the anchor window first if needed, then calls `rs_grid_adjust`
/// to get the actual grid and position offsets. Also handles `bufpos` if set.
///
/// Returns `(anchor_grid, row, col)` updated with offsets.
unsafe fn resolve_window_relative(
    wp: WinHandle,
    validate: bool,
    mut row: f64,
    mut col: f64,
) -> (ScreenGridHandle, f64, f64) {
    let config_window = nvim_win_get_config_window(wp);
    let win = nvim_handle_get_window(config_window);
    if win.is_null() {
        return (get_default_grid(), row, col);
    }

    // When a floating window is anchored to another window,
    // update the position of its anchored window first.
    if nvim_win_get_pos_changed(win) != 0
        && !nvim_screengrid_get_chars(nvim_win_get_grid_alloc(win)).is_null()
        && rs_win_valid(win) != 0
    {
        ui_ext_win_position(win, validate);
    }

    let mut row_off: c_int = 0;
    let mut col_off: c_int = 0;
    nvim_win_call_win_grid_alloc(win);
    let win_grid_view = nvim_win_get_w_grid(win);
    let adjusted = rs_grid_adjust(
        win_grid_view,
        std::ptr::addr_of_mut!(row_off),
        std::ptr::addr_of_mut!(col_off),
    );
    let anchor_grid = ScreenGridHandle(adjusted);
    row += f64::from(row_off);
    col += f64::from(col_off);

    let bufpos_lnum = nvim_win_get_config_bufpos_lnum(wp);
    if bufpos_lnum >= 0 {
        let line_count = nvim_win_buf_line_count(win);
        let lnum = (bufpos_lnum + 1).min(line_count);
        let pos = PosT {
            lnum,
            col: nvim_win_get_config_bufpos_col(wp),
            coladd: 0,
        };
        let mut screen_row: c_int = 0;
        let mut start_col: c_int = 0;
        let mut cursor_col: c_int = 0;
        let mut end_col: c_int = 0;
        rs_textpos2screenpos(
            win,
            std::ptr::addr_of!(pos),
            std::ptr::addr_of_mut!(screen_row),
            std::ptr::addr_of_mut!(start_col),
            std::ptr::addr_of_mut!(cursor_col),
            std::ptr::addr_of_mut!(end_col),
            1,
        );
        row += f64::from(screen_row - 1);
        col += f64::from(start_col - 1);
    }

    (anchor_grid, row, col)
}

/// Handle the non-floating window case.
unsafe fn handle_non_floating(wp: WinHandle) {
    if nvim_get_ui_ext(K_UI_MULTIGRID) != 0 {
        // Windows on the default grid don't necessarily have comp_col/comp_row set,
        // but the rest of the calculations relies on it.
        let grid = nvim_win_get_grid_alloc(wp);
        nvim_screengrid_set_comp_col(grid, nvim_win_get_wincol(wp));
        nvim_screengrid_set_comp_row(grid, nvim_win_get_winrow(wp));
    }
    let grid_handle = nvim_screengrid_get_handle(nvim_win_get_grid_alloc(wp));
    nvim_win_ui_call_win_pos(
        grid_handle,
        nvim_win_get_handle(wp),
        nvim_win_get_winrow(wp),
        nvim_win_get_wincol(wp),
        nvim_win_get_w_width(wp),
        nvim_win_get_w_height(wp),
    );
}

/// Handle the external window case.
unsafe fn handle_external(wp: WinHandle) {
    let grid_alloc = nvim_win_get_grid_alloc(wp);
    let grid_handle = nvim_screengrid_get_handle(grid_alloc);
    let win_handle = nvim_win_get_handle(wp);
    nvim_win_ui_call_win_external_pos(grid_handle, win_handle);
}

/// Handle the internal floating window case.
///
/// Resolves position, updates compositor, and fires UI events.
#[allow(clippy::too_many_lines)]
unsafe fn handle_internal_float(wp: WinHandle, validate: bool) {
    let mut row = nvim_win_get_config_row(wp);
    let mut col = nvim_win_get_config_col(wp);
    let relative = nvim_win_get_config_relative(wp);

    let anchor_grid = if relative == K_FLOAT_RELATIVE_WINDOW {
        let (ag, r, c) = resolve_window_relative(wp, validate, row, col);
        row = r;
        col = c;
        ag
    } else {
        if relative == K_FLOAT_RELATIVE_LASTSTATUS {
            let rows = nvim_get_rows();
            let p_ch = nvim_get_p_ch() as c_int;
            row += f64::from(rows - p_ch - last_stl_height(0));
        } else if relative == K_FLOAT_RELATIVE_TABLINE {
            row += f64::from(tabline_height());
        }
        get_default_grid()
    };

    let grid_alloc = nvim_win_get_grid_alloc(wp);
    let config_zindex = nvim_win_get_config_zindex(wp);
    let grid_zindex = nvim_screengrid_get_zindex(grid_alloc);
    let comp_index = nvim_screengrid_get_comp_index(grid_alloc);

    let resort = comp_index != 0 && grid_zindex != config_zindex;
    let raise = resort && grid_zindex < config_zindex;
    nvim_screengrid_set_zindex(grid_alloc, config_zindex);
    if resort {
        rs_ui_comp_layers_adjust(comp_index, raise);
    }

    let redr_type = nvim_win_get_redr_type(wp);
    let valid = redr_type == 0 || nvim_get_ui_ext(K_UI_MULTIGRID) != 0;
    if !valid && !validate {
        nvim_win_set_pos_changed(wp, 1);
        return;
    }

    // Compute anchor offsets.
    // NOTE: Using raw C constants: East=1, South=2 (see WARNING above about
    // winfloat crate's swapped values).
    let anchor = nvim_win_get_config_anchor(wp);
    let east = (anchor & K_FLOAT_ANCHOR_EAST) != 0;
    let south = (anchor & K_FLOAT_ANCHOR_SOUTH) != 0;

    let height_outer = nvim_win_get_w_height_outer(wp);
    let width_outer = nvim_win_get_w_width_outer(wp);

    let mut comp_row = row as c_int - if south { height_outer } else { 0 };
    let mut comp_col = col as c_int - if east { width_outer } else { 0 };

    let p_ch = nvim_get_p_ch() as c_int;
    let above_ch = if config_zindex < K_ZINDEX_MESSAGES {
        p_ch
    } else {
        0
    };

    comp_row += nvim_screengrid_get_comp_row(anchor_grid);
    comp_col += nvim_screengrid_get_comp_col(anchor_grid);

    let rows = nvim_get_rows();
    let columns = nvim_get_columns();
    let max_row = rows - height_outer - above_ch;
    comp_row = comp_row.min(max_row).max(0);

    let fixed = nvim_win_get_config_fixed(wp);
    if fixed == 0 || east {
        let max_col = columns - width_outer;
        comp_col = comp_col.min(max_col).max(0);
    }

    nvim_win_set_winrow(wp, comp_row);
    nvim_win_set_wincol(wp, comp_col);

    let hide = nvim_win_get_config_hide(wp) != 0;
    if hide {
        if nvim_get_ui_ext(K_UI_MULTIGRID) != 0 {
            let grid_handle = nvim_screengrid_get_handle(grid_alloc);
            nvim_win_ui_call_win_hide(grid_handle);
        }
        rs_ui_comp_remove_grid(grid_alloc);
    } else {
        rs_ui_comp_put_grid(
            grid_alloc,
            comp_row,
            comp_col,
            height_outer,
            width_outer,
            valid,
            false,
        );
        if nvim_get_ui_ext(K_UI_MULTIGRID) != 0 {
            let anchor_grid_handle = nvim_screengrid_get_handle(anchor_grid);
            let grid_handle = nvim_screengrid_get_handle(grid_alloc);
            let win_handle = nvim_win_get_handle(wp);
            let comp_idx = nvim_screengrid_get_comp_index(grid_alloc) as c_int;
            let mouse_flag = nvim_win_get_config_mouse_flag(wp);
            nvim_win_ui_call_win_float_pos(
                grid_handle,
                win_handle,
                anchor,
                anchor_grid_handle,
                row,
                col,
                mouse_flag,
                config_zindex,
                comp_idx,
                comp_row,
                comp_col,
            );
        }
        let grid_handle = nvim_screengrid_get_handle(grid_alloc);
        nvim_win_ui_check_cursor_grid(grid_handle);
        let mouse_flag = nvim_win_get_config_mouse_flag(wp);
        nvim_screengrid_set_mouse_enabled(grid_alloc, mouse_flag != 0);
        if !valid {
            nvim_screengrid_set_valid(grid_alloc, false);
            nvim_redraw_later_wrapper(wp, UPD_NOT_VALID);
        }
    }
}

// ---------------------------------------------------------------------------
// do_autocmd_winclosed
// ---------------------------------------------------------------------------

/// Recursion guard for `do_autocmd_winclosed`.
///
/// Replaces the C `static bool recursive` local inside `do_autocmd_winclosed`.
static DO_AUTOCMD_WINCLOSED_RECURSIVE: AtomicBool = AtomicBool::new(false);

extern "C" {
    /// Check whether EVENT_WINCLOSED has any autocmds registered.
    fn nvim_has_event_winclosed() -> c_int;

    /// Apply WinClosed autocmds for window `win`.
    fn nvim_apply_autocmds_winclosed(win: WinHandle);
}

/// Fire WinClosed autocmd for window `win`, with recursion guard.
///
/// Port of C `do_autocmd_winclosed()`.
fn do_autocmd_winclosed_impl(win: WinHandle) {
    if DO_AUTOCMD_WINCLOSED_RECURSIVE.load(Ordering::SeqCst) {
        return;
    }
    // SAFETY: nvim_has_event_winclosed is a C accessor
    if unsafe { nvim_has_event_winclosed() } == 0 {
        return;
    }
    DO_AUTOCMD_WINCLOSED_RECURSIVE.store(true, Ordering::SeqCst);
    // SAFETY: nvim_apply_autocmds_winclosed is a C accessor
    unsafe { nvim_apply_autocmds_winclosed(win) };
    DO_AUTOCMD_WINCLOSED_RECURSIVE.store(false, Ordering::SeqCst);
}

/// FFI export for `do_autocmd_winclosed`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_do_autocmd_winclosed(win: WinHandle) {
    do_autocmd_winclosed_impl(win);
}

// ---------------------------------------------------------------------------
// FFI Export
// ---------------------------------------------------------------------------

/// Compute window position for UI layer.
///
/// Handles floating/non-floating/external modes, manages compositor grid
/// placement and z-index. Corresponds to C's `ui_ext_win_position`.
///
/// # Safety
/// Accesses global Neovim state via C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_ext_win_position(wp: WinHandle, validate: bool) {
    // Clear pos_changed flag
    nvim_win_set_pos_changed(wp, 0);

    if nvim_win_get_floating(wp) == 0 {
        handle_non_floating(wp);
        return;
    }

    if nvim_win_get_config_external(wp) == 0 {
        handle_internal_float(wp, validate);
    } else {
        handle_external(wp);
    }
}
