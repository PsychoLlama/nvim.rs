//! Screen size and scroll helper functions.
//!
//! Rust ports of win_comp_scroll, win_new_screensize, win_new_screen_cols,
//! win_init_size, and snapshot_windows_scroll_size from window_shim.c.

use std::ffi::c_int;

use crate::{Frame, TabpageHandle, WinHandle};

// =============================================================================
// C Accessor Functions
// =============================================================================

extern "C" {
    // --- Globals ---
    fn nvim_get_Rows() -> c_int;
    fn nvim_get_Columns() -> c_int;
    fn nvim_get_rows_avail() -> c_int;
    fn nvim_get_firstwin() -> WinHandle;
    fn nvim_get_topframe() -> *mut Frame;
    fn nvim_get_curtab() -> TabpageHandle;

    // --- Option helpers ---
    fn nvim_get_p_window() -> i64;
    fn nvim_set_p_window(val: i64);
    fn nvim_option_was_set_window() -> c_int;

    // --- Window field setters ---
    fn nvim_win_set_field_height(wp: WinHandle, val: c_int);
    fn nvim_win_set_field_width(wp: WinHandle, val: c_int);
    fn nvim_win_get_w_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_width(wp: WinHandle) -> c_int;
    fn nvim_win_set_prev_height(wp: WinHandle, val: c_int);
    fn nvim_win_set_view_height(wp: WinHandle, val: c_int);
    fn nvim_win_set_view_width(wp: WinHandle, val: c_int);
    fn nvim_win_set_height_outer(wp: WinHandle, val: c_int);
    fn nvim_win_set_width_outer(wp: WinHandle, val: c_int);
    fn nvim_win_set_winrow_off(wp: WinHandle, val: c_int);
    fn nvim_win_get_winbar_height(wp: WinHandle) -> c_int;

    // --- Snapshot field setters ---
    fn nvim_win_set_last_topline(wp: WinHandle, val: i32);
    fn nvim_win_set_last_topfill(wp: WinHandle, val: c_int);
    fn nvim_win_set_last_leftcol(wp: WinHandle, val: c_int);
    fn nvim_win_set_last_skipcol(wp: WinHandle, val: c_int);
    fn nvim_win_set_last_width(wp: WinHandle, val: c_int);
    fn nvim_win_set_last_height(wp: WinHandle, val: c_int);

    // --- Snapshot field getters ---
    fn nvim_win_get_topline(wp: WinHandle) -> i32;
    fn nvim_win_get_topfill(wp: WinHandle) -> c_int;
    fn nvim_win_get_leftcol(wp: WinHandle) -> c_int;
    fn nvim_win_get_skipcol(wp: WinHandle) -> c_int;
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;
    fn nvim_tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle;

    // --- win_comp_scroll accessors ---
    fn nvim_win_get_p_scr(wp: WinHandle) -> i64;
    fn nvim_win_set_p_scr(wp: WinHandle, val: i64);
    fn nvim_win_set_script_ctx_scroll(wp: WinHandle);

    // --- Screen size helpers ---
    fn nvim_win_new_screen_rows_wrapper();
    fn nvim_win_reconfig_floats();

    // --- Rust resize functions used by screen size helpers ---
    fn rs_win_default_scroll(wp: WinHandle) -> i64;
    fn rs_frame_new_width(topfrp: *mut Frame, width: c_int, leftfirst: c_int, wfw: c_int);
    fn rs_frame_check_width(topfrp: *mut Frame, width: c_int) -> c_int;
    fn rs_win_comp_pos() -> c_int;

    // --- win_new_screen_rows dependencies ---
    fn rs_frame_minheight(topfrp: *const Frame, next_curwin: WinHandle) -> c_int;
    fn rs_frame_check_height(topfrp: *const Frame, height: c_int) -> c_int;
    fn rs_frame_new_height(
        topfrp: *mut Frame,
        height: c_int,
        topfirst: c_int,
        wfh: c_int,
        set_ch: c_int,
    );
    fn nvim_get_p_ch() -> i64;
    fn nvim_tabpage_set_ch_used(tp: TabpageHandle, val: i64);
    fn nvim_compute_cmdrow();
    fn nvim_get_skip_win_fix_scroll() -> c_int;
    fn rs_win_fix_scroll(resize: c_int);
}

// =============================================================================
// win_comp_scroll
// =============================================================================

/// Recompute w_p_scr (the scroll option) for a window.
///
/// Port of C `win_comp_scroll()`.
///
/// # Safety
/// Calls C accessor functions with a valid window handle.
unsafe fn win_comp_scroll_impl(wp: WinHandle) {
    let old_w_p_scr = nvim_win_get_p_scr(wp);
    let new_scroll = rs_win_default_scroll(wp);
    nvim_win_set_p_scr(wp, new_scroll);

    if new_scroll != old_w_p_scr {
        // Used by "verbose set scroll".
        nvim_win_set_script_ctx_scroll(wp);
    }
}

/// FFI export for `win_comp_scroll`.
///
/// # Safety
/// Calls C accessor functions with a valid window handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_comp_scroll(wp: WinHandle) {
    win_comp_scroll_impl(wp);
}

// =============================================================================
// win_new_screen_cols
// =============================================================================

/// Update window widths when Columns changes.
///
/// Port of C `win_new_screen_cols()`.
///
/// # Safety
/// Calls C accessor functions and Rust resize functions.
unsafe fn win_new_screen_cols_impl() {
    let firstwin = nvim_get_firstwin();
    if firstwin.is_null() {
        // not initialized yet
        return;
    }

    let topframe = nvim_get_topframe();
    let columns = nvim_get_Columns();

    // First try setting the widths of windows with 'winfixwidth'. If that
    // doesn't result in the right width, forget about that option.
    rs_frame_new_width(topframe, columns, 0, 1);
    if rs_frame_check_width(topframe, columns) == 0 {
        rs_frame_new_width(topframe, columns, 0, 0);
    }

    rs_win_comp_pos(); // recompute w_winrow and w_wincol
    nvim_win_reconfig_floats(); // The size of floats might change
}

/// FFI export for `win_new_screen_cols`.
///
/// # Safety
/// Calls C accessor functions and Rust resize functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_new_screen_cols() {
    win_new_screen_cols_impl();
}

// =============================================================================
// win_new_screensize
// =============================================================================

/// Static state for win_new_screensize (replaces static locals in C).
static OLD_ROWS: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
static OLD_COLUMNS: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// Handle screen size change -- update windows based on new Rows/Columns.
///
/// Port of C `win_new_screensize()`.
///
/// # Safety
/// Calls C accessor functions and Rust resize functions.
unsafe fn win_new_screensize_impl() {
    use std::sync::atomic::Ordering;

    let rows = nvim_get_Rows();
    let columns = nvim_get_Columns();
    let old_rows = OLD_ROWS.load(Ordering::Relaxed);
    let old_columns = OLD_COLUMNS.load(Ordering::Relaxed);

    if old_rows != rows {
        // If 'window' uses the whole screen, keep it using that.
        // Don't change it when set with "-w size" on the command line.
        let p_window = nvim_get_p_window();
        let window_unset = old_rows == 0 && nvim_option_was_set_window() == 0;
        if p_window == i64::from(old_rows) - 1 || window_unset {
            nvim_set_p_window(i64::from(rows) - 1);
        }
        OLD_ROWS.store(rows, Ordering::Relaxed);
        nvim_win_new_screen_rows_wrapper(); // update window sizes
    }
    if old_columns != columns {
        OLD_COLUMNS.store(columns, Ordering::Relaxed);
        win_new_screen_cols_impl(); // update window sizes
    }
}

/// FFI export for `win_new_screensize`.
///
/// # Safety
/// Calls C accessor functions and Rust resize functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_new_screensize() {
    win_new_screensize_impl();
}

// =============================================================================
// win_init_size
// =============================================================================

/// Initialize the window and frame size to the maximum screen dimensions.
///
/// Port of C `win_init_size()`.
///
/// # Safety
/// Calls C accessor functions. firstwin and topframe must be initialized.
unsafe fn win_init_size_impl() {
    let firstwin = nvim_get_firstwin();
    let topframe = nvim_get_topframe();
    let rows_avail = nvim_get_rows_avail();
    let columns = nvim_get_Columns();
    let winbar_height = nvim_win_get_winbar_height(firstwin);

    nvim_win_set_field_height(firstwin, rows_avail);
    nvim_win_set_prev_height(firstwin, rows_avail);
    nvim_win_set_view_height(firstwin, rows_avail - winbar_height);
    nvim_win_set_height_outer(firstwin, rows_avail);
    nvim_win_set_winrow_off(firstwin, winbar_height);
    if !topframe.is_null() {
        (*topframe).fr_height = rows_avail;
    }
    nvim_win_set_field_width(firstwin, columns);
    nvim_win_set_view_width(firstwin, columns);
    nvim_win_set_width_outer(firstwin, columns);
    if !topframe.is_null() {
        (*topframe).fr_width = columns;
    }
}

/// FFI export for `win_init_size`.
///
/// # Safety
/// Calls C accessor functions. firstwin and topframe must be initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_init_size() {
    win_init_size_impl();
}

// =============================================================================
// snapshot_windows_scroll_size
// =============================================================================

/// Make a snapshot of all the window scroll positions and sizes of the
/// current tab page.
///
/// Port of C `snapshot_windows_scroll_size()`.
///
/// # Safety
/// Calls C accessor functions.
unsafe fn snapshot_windows_scroll_size_impl() {
    let curtab = nvim_get_curtab();
    let mut wp = nvim_tabpage_get_firstwin(curtab);
    while !wp.is_null() {
        nvim_win_set_last_topline(wp, nvim_win_get_topline(wp));
        nvim_win_set_last_topfill(wp, nvim_win_get_topfill(wp));
        nvim_win_set_last_leftcol(wp, nvim_win_get_leftcol(wp));
        nvim_win_set_last_skipcol(wp, nvim_win_get_skipcol(wp));
        nvim_win_set_last_width(wp, nvim_win_get_w_width(wp));
        nvim_win_set_last_height(wp, nvim_win_get_w_height(wp));
        wp = nvim_win_get_next(wp);
    }
}

/// FFI export for `snapshot_windows_scroll_size`.
///
/// # Safety
/// Calls C accessor functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_snapshot_windows_scroll_size() {
    snapshot_windows_scroll_size_impl();
}

// =============================================================================
// win_new_screen_rows
// =============================================================================

/// Handle row count change: resize frame tree to new height, recompute positions,
/// reconfigure floats, update tp_ch_used, and fix scroll positions.
///
/// This is the Rust equivalent of `win_new_screen_rows()`.
///
/// # Safety
/// Calls C accessor functions.
unsafe fn win_new_screen_rows_impl() {
    let firstwin = nvim_get_firstwin();
    if firstwin.is_null() {
        return; // not initialized yet
    }

    let topframe = nvim_get_topframe();
    let rows_avail = nvim_get_rows_avail();
    let h = rows_avail.max(rs_frame_minheight(topframe, WinHandle::null()));

    // First try setting heights with 'winfixheight'. If that doesn't result
    // in the right height, forget about that option.
    rs_frame_new_height(topframe, h, 0, 1, 0);
    if rs_frame_check_height(topframe, h) == 0 {
        rs_frame_new_height(topframe, h, 0, 0, 0);
    }

    rs_win_comp_pos(); // recompute w_winrow and w_wincol
    nvim_win_reconfig_floats(); // the size of floats might change
    nvim_compute_cmdrow();

    let curtab = nvim_get_curtab();
    nvim_tabpage_set_ch_used(curtab, nvim_get_p_ch());

    if nvim_get_skip_win_fix_scroll() == 0 {
        rs_win_fix_scroll(1); // resize = true
    }
}

/// FFI export for `win_new_screen_rows`.
///
/// # Safety
/// Calls C accessor functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_new_screen_rows() {
    win_new_screen_rows_impl();
}
