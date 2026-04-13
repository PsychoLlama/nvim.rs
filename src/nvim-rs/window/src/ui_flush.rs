//! Window UI flush functions.
//!
//! This module provides Rust implementations of window UI flush functions:
//! - `tabpage_check_windows`: Update UI visibility when switching tabpages.
//! - `win_ui_flush`: Flush pending position/viewport updates to UI.

#![allow(clippy::cast_possible_wrap)]

use std::ffi::c_int;

use crate::{win_struct::win_ref, TabpageHandle, WinHandle};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    // Tabpage iteration
    fn nvim_get_first_tabpage() -> TabpageHandle;
    fn nvim_tabpage_get_next(tp: TabpageHandle) -> TabpageHandle;
    fn nvim_get_curtab() -> TabpageHandle;

    // Window iteration
    fn nvim_get_firstwin() -> WinHandle;
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;
    fn nvim_tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle;

    // Window state accessors
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_external_int(wp: WinHandle) -> c_int;
    fn nvim_win_set_pos_changed(wp: WinHandle, val: c_int);

    // Grid/UI accessors
    fn nvim_win_get_grid_pending_comp(wp: WinHandle) -> c_int;
    fn nvim_win_set_grid_pending_comp(wp: WinHandle, val: c_int);
    fn nvim_win_get_grid_chars_valid(wp: WinHandle) -> c_int;
    fn nvim_win_get_grid_alloc_handle(wp: WinHandle) -> c_int;

    // UI operations
    fn nvim_win_config_float(wp: WinHandle);
    fn nvim_ui_comp_remove_grid_win(wp: WinHandle);
    fn nvim_ui_call_win_hide_win(wp: WinHandle);

    // rs_win_remove / rs_win_append / rs_lastwin_nofloating
    fn rs_win_remove(wp: WinHandle, tp: TabpageHandle);
    fn rs_win_append(after: WinHandle, wp: WinHandle, tp: TabpageHandle);
    fn rs_lastwin_nofloating() -> WinHandle;

    // ui_ext_win_position (stays in C; wrapper delegates to rs_ui_ext_win_position)
    fn ui_ext_win_position(wp: WinHandle, validate: bool);

    // rs_ui_ext_win_viewport (just migrated to Rust)
    fn rs_ui_ext_win_viewport(wp: WinHandle);

    // pum and msg flush
    fn nvim_pum_ui_flush_wrapper();
    fn nvim_msg_ui_flush_wrapper();
}

// =============================================================================
// tabpage_check_windows
// =============================================================================

/// Tell external UI that windows in `old_curtab` are invisible and floats in
/// the current tab are now visible.
///
/// Called during tab page switches. Port of C `tabpage_check_windows()`.
///
/// # Safety
/// Accesses global Neovim state via C accessor functions.
unsafe fn tabpage_check_windows_impl(old_curtab: TabpageHandle) {
    // Handle windows in the old tabpage: hide external floats, remove internal
    // float grids from compositor, mark all windows as pos_changed.
    let mut wp = nvim_tabpage_get_firstwin(old_curtab);
    while !wp.is_null() {
        let next_wp = nvim_win_get_next(wp);

        if nvim_win_get_floating(wp) != 0 {
            if nvim_win_get_config_external_int(wp) != 0 {
                rs_win_remove(wp, old_curtab);
                rs_win_append(rs_lastwin_nofloating(), wp, TabpageHandle::null());
            } else {
                nvim_ui_comp_remove_grid_win(wp);
            }
        }
        nvim_win_set_pos_changed(wp, 1);

        wp = next_wp;
    }

    // Handle windows in the new (current) tabpage: configure internal floats
    // and mark all windows as pos_changed.
    let mut wp = nvim_get_firstwin();
    while !wp.is_null() {
        if nvim_win_get_floating(wp) != 0 && nvim_win_get_config_external_int(wp) == 0 {
            nvim_win_config_float(wp);
        }
        nvim_win_set_pos_changed(wp, 1);
        wp = nvim_win_get_next(wp);
    }
}

// =============================================================================
// win_ui_flush
// =============================================================================

/// Get the first window in a tabpage (current tab uses firstwin, others use tp_firstwin).
unsafe fn get_tabpage_firstwin(tp: TabpageHandle) -> WinHandle {
    let curtab = nvim_get_curtab();
    if tp == curtab {
        nvim_get_firstwin()
    } else {
        nvim_tabpage_get_firstwin(tp)
    }
}

/// Flush pending window position/viewport updates to UI.
///
/// Called from ui.c and drawscreen.c. Port of C `win_ui_flush()`.
///
/// # Safety
/// Accesses global Neovim state via C accessor functions.
unsafe fn win_ui_flush_impl(validate: c_int) {
    let curtab = nvim_get_curtab();

    // FOR_ALL_TAB_WINDOWS(tp, wp)
    let mut tp = nvim_get_first_tabpage();
    while !tp.is_null() {
        let mut wp = get_tabpage_firstwin(tp);
        while !wp.is_null() {
            if (win_ref(wp).w_pos_changed || nvim_win_get_grid_pending_comp(wp) != 0)
                && nvim_win_get_grid_chars_valid(wp) != 0
            {
                if tp == curtab {
                    ui_ext_win_position(wp, validate != 0);
                } else {
                    nvim_ui_call_win_hide_win(wp);
                    nvim_win_set_pos_changed(wp, 0);
                }
                nvim_win_set_grid_pending_comp(wp, 0);
            }
            if tp == curtab {
                rs_ui_ext_win_viewport(wp);
            }

            wp = nvim_win_get_next(wp);
        }
        tp = nvim_tabpage_get_next(tp);
    }

    // The popupmenu could also have moved or changed its comp_index
    nvim_pum_ui_flush_wrapper();

    // And the message
    nvim_msg_ui_flush_wrapper();
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Tell UI that old tab's windows are invisible, new tab's floats are visible.
///
/// Replaces C `tabpage_check_windows()`.
///
/// # Safety
/// Accesses global Neovim state via C accessor functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_tabpage_check_windows(old_curtab: TabpageHandle) {
    tabpage_check_windows_impl(old_curtab);
}

/// FFI: Flush pending window position/viewport updates to UI.
///
/// Replaces C `win_ui_flush()`.
///
/// # Safety
/// Accesses global Neovim state via C accessor functions.
#[unsafe(export_name = "win_ui_flush")]
pub unsafe extern "C" fn rs_win_ui_flush(validate: c_int) {
    win_ui_flush_impl(validate);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // ui_flush functions require live Neovim state
    }
}
