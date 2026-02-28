//! Snapshot lifecycle functions: allocation, deallocation, and traversal.
//!
//! This module provides Rust implementations of snapshot management functions
//! from `src/nvim/window.c`: clear_snapshot, clear_snapshot_rec,
//! make_snapshot, make_snapshot_rec, get_snapshot_curwin, get_snapshot_curwin_rec,
//! check_snapshot_rec, and restore_snapshot_rec.

use std::ffi::{c_int, c_void};

use crate::{Frame, TabpageHandle, WinHandle, FR_LEAF};

// =============================================================================
// Constants
// =============================================================================

const OK: c_int = 1;
const FAIL: c_int = 0;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn xcalloc(count: usize, size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn nvim_get_curtab() -> TabpageHandle;
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_get_topframe() -> *mut Frame;
    fn nvim_tabpage_get_snapshot(tp: TabpageHandle, idx: c_int) -> *mut Frame;
    fn nvim_tabpage_set_snapshot(tp: TabpageHandle, idx: c_int, val: *mut Frame);
    fn rs_win_valid(win: WinHandle) -> c_int;
    fn rs_frame_new_height(
        topfrp: *mut Frame,
        height: c_int,
        topfirst: c_int,
        wfh: c_int,
        set_ch: c_int,
    );
    fn rs_frame_new_width(topfrp: *mut Frame, width: c_int, leftfirst: c_int, wfw: c_int);

    // --- restore_snapshot dependencies ---
    fn rs_win_comp_pos() -> c_int;
    fn rs_win_goto(wp: WinHandle);
    fn nvim_redraw_all_later(type_: c_int);
}

// UPD_NOT_VALID = 40 (matches UPD_NOT_VALID in drawscreen.h)
const UPD_NOT_VALID: c_int = 40;

// =============================================================================
// Implementations
// =============================================================================

/// Recursively free a snapshot frame tree.
///
/// Equivalent to C `clear_snapshot_rec()` (window.c L7788).
fn clear_snapshot_rec_impl(fr: *mut Frame) {
    if fr.is_null() {
        return;
    }
    unsafe {
        clear_snapshot_rec_impl((*fr).fr_next);
        clear_snapshot_rec_impl((*fr).fr_child);
        xfree(fr.cast::<c_void>());
    }
}

/// Remove any existing snapshot.
///
/// Equivalent to C `clear_snapshot()` (window.c L7782).
fn clear_snapshot_impl(tp: TabpageHandle, idx: c_int) {
    if tp.is_null() {
        return;
    }
    unsafe {
        let snapshot = nvim_tabpage_get_snapshot(tp, idx);
        clear_snapshot_rec_impl(snapshot);
        nvim_tabpage_set_snapshot(tp, idx, std::ptr::null_mut());
    }
}

/// Recursively build a snapshot of the frame tree.
///
/// Equivalent to C `make_snapshot_rec()` (window.c L7764).
///
/// # Safety
/// `fr` must be a valid, non-null frame pointer.
unsafe fn make_snapshot_rec_impl(fr: *const Frame, frp: *mut *mut Frame) {
    let new_frame = xcalloc(1, std::mem::size_of::<Frame>()).cast::<Frame>();
    *frp = new_frame;

    (*new_frame).fr_layout = (*fr).fr_layout;
    (*new_frame).fr_width = (*fr).fr_width;
    (*new_frame).fr_height = (*fr).fr_height;

    if !(*fr).fr_next.is_null() {
        make_snapshot_rec_impl((*fr).fr_next, &raw mut (*new_frame).fr_next);
    }
    if !(*fr).fr_child.is_null() {
        make_snapshot_rec_impl((*fr).fr_child, &raw mut (*new_frame).fr_child);
    }
    if (*fr).fr_layout == FR_LEAF && (*fr).fr_win == nvim_get_curwin() {
        (*new_frame).fr_win = nvim_get_curwin();
    }
}

/// Create a snapshot of the current frame sizes.
///
/// Equivalent to C `make_snapshot()` (window.c L7758).
fn make_snapshot_impl(idx: c_int) {
    unsafe {
        let curtab = nvim_get_curtab();
        clear_snapshot_impl(curtab, idx);
        let topframe = nvim_get_topframe();
        if !topframe.is_null() {
            let mut snapshot: *mut Frame = std::ptr::null_mut();
            make_snapshot_rec_impl(topframe, &raw mut snapshot);
            nvim_tabpage_set_snapshot(curtab, idx, snapshot);
        }
    }
}

/// Traverse a snapshot to find the previous curwin.
///
/// Equivalent to C `get_snapshot_curwin_rec()` (window.c L7799).
fn get_snapshot_curwin_rec_impl(ft: *const Frame) -> WinHandle {
    if ft.is_null() {
        return WinHandle::null();
    }

    unsafe {
        if !(*ft).fr_next.is_null() {
            let wp = get_snapshot_curwin_rec_impl((*ft).fr_next);
            if !wp.is_null() {
                return wp;
            }
        }
        if !(*ft).fr_child.is_null() {
            let wp = get_snapshot_curwin_rec_impl((*ft).fr_child);
            if !wp.is_null() {
                return wp;
            }
        }

        (*ft).fr_win
    }
}

/// Return the current window stored in the snapshot or NULL.
///
/// Equivalent to C `get_snapshot_curwin()` (window.c L7818).
fn get_snapshot_curwin_impl(idx: c_int) -> WinHandle {
    unsafe {
        let curtab = nvim_get_curtab();
        let snapshot = nvim_tabpage_get_snapshot(curtab, idx);
        if snapshot.is_null() {
            return WinHandle::null();
        }
        get_snapshot_curwin_rec_impl(snapshot)
    }
}

/// Check if frames "sn" and "fr" have the same layout, same following frames
/// and same children. And the window pointer is valid.
///
/// Equivalent to C `check_snapshot_rec()` (window.c L7850).
fn check_snapshot_rec_impl(sn: *const Frame, fr: *const Frame) -> c_int {
    if sn.is_null() || fr.is_null() {
        return FAIL;
    }

    unsafe {
        if (*sn).fr_layout != (*fr).fr_layout
            || (*sn).fr_next.is_null() != (*fr).fr_next.is_null()
            || (*sn).fr_child.is_null() != (*fr).fr_child.is_null()
            || (!(*sn).fr_next.is_null()
                && check_snapshot_rec_impl((*sn).fr_next, (*fr).fr_next) == FAIL)
            || (!(*sn).fr_child.is_null()
                && check_snapshot_rec_impl((*sn).fr_child, (*fr).fr_child) == FAIL)
            || (!(*sn).fr_win.is_null() && rs_win_valid((*sn).fr_win) == 0)
        {
            return FAIL;
        }
    }
    OK
}

/// Copy the size of snapshot frame "sn" to frame "fr". Do the same for all
/// following frames and children.
/// Returns a pointer to the old current window, or NULL.
///
/// Equivalent to C `restore_snapshot_rec()` (window.c L7868).
fn restore_snapshot_rec_impl(sn: *const Frame, fr: *mut Frame) -> WinHandle {
    if sn.is_null() || fr.is_null() {
        return WinHandle::null();
    }

    let mut wp = WinHandle::null();

    unsafe {
        (*fr).fr_height = (*sn).fr_height;
        (*fr).fr_width = (*sn).fr_width;

        if (*fr).fr_layout == FR_LEAF {
            rs_frame_new_height(fr, (*fr).fr_height, 0, 0, 0);
            rs_frame_new_width(fr, (*fr).fr_width, 0, 0);
            wp = (*sn).fr_win;
        }

        if !(*sn).fr_next.is_null() {
            let wp2 = restore_snapshot_rec_impl((*sn).fr_next, (*fr).fr_next);
            if !wp2.is_null() {
                wp = wp2;
            }
        }

        if !(*sn).fr_child.is_null() {
            let wp2 = restore_snapshot_rec_impl((*sn).fr_child, (*fr).fr_child);
            if !wp2.is_null() {
                wp = wp2;
            }
        }
    }

    wp
}

/// Restore a previously created snapshot, if there is any.
///
/// Only restores if the screen size matches and the window layout is the same.
///
/// Equivalent to C `restore_snapshot()` (window.c).
///
/// # Safety
/// Calls C accessor functions.
unsafe fn restore_snapshot_impl(idx: c_int, close_curwin: bool) {
    let curtab = nvim_get_curtab();
    let topframe = nvim_get_topframe();
    let snapshot = nvim_tabpage_get_snapshot(curtab, idx);

    if !snapshot.is_null()
        && !topframe.is_null()
        && (*snapshot).fr_width == (*topframe).fr_width
        && (*snapshot).fr_height == (*topframe).fr_height
        && check_snapshot_rec_impl(snapshot, topframe) == OK
    {
        let wp = restore_snapshot_rec_impl(snapshot, topframe);
        rs_win_comp_pos();
        if !wp.is_null() && close_curwin {
            rs_win_goto(wp);
        }
        nvim_redraw_all_later(UPD_NOT_VALID);
    }
    clear_snapshot_impl(curtab, idx);
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Recursively free a snapshot frame tree.
#[unsafe(no_mangle)]
pub extern "C" fn rs_clear_snapshot_rec(fr: *mut Frame) {
    clear_snapshot_rec_impl(fr);
}

/// FFI: Remove any existing snapshot.
#[unsafe(no_mangle)]
pub extern "C" fn rs_clear_snapshot(tp: TabpageHandle, idx: c_int) {
    clear_snapshot_impl(tp, idx);
}

/// FFI: Create a snapshot of the current frame sizes.
#[unsafe(no_mangle)]
pub extern "C" fn rs_make_snapshot(idx: c_int) {
    make_snapshot_impl(idx);
}

/// FFI: Get the curwin stored in the snapshot, or NULL.
#[unsafe(no_mangle)]
pub extern "C" fn rs_get_snapshot_curwin(idx: c_int) -> WinHandle {
    get_snapshot_curwin_impl(idx)
}

/// FFI: Check if snapshot and frame tree have matching structure.
#[unsafe(no_mangle)]
pub extern "C" fn rs_check_snapshot_rec(sn: *const Frame, fr: *const Frame) -> c_int {
    check_snapshot_rec_impl(sn, fr)
}

/// FFI: Restore sizes from snapshot to live frame tree.
/// Returns the old curwin or NULL.
#[unsafe(no_mangle)]
pub extern "C" fn rs_restore_snapshot_rec(sn: *const Frame, fr: *mut Frame) -> WinHandle {
    restore_snapshot_rec_impl(sn, fr)
}

/// FFI: Restore a previously created snapshot.
///
/// # Safety
/// Calls C accessor functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_restore_snapshot(idx: c_int, close_curwin: c_int) {
    restore_snapshot_impl(idx, close_curwin != 0);
}

/// C export: `restore_snapshot` — eliminates the C thin wrapper.
///
/// # Safety
/// Calls C accessor functions.
#[unsafe(export_name = "restore_snapshot")]
pub unsafe extern "C" fn restore_snapshot(idx: c_int, close_curwin: c_int) {
    restore_snapshot_impl(idx, close_curwin != 0);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear_snapshot_rec_null() {
        // Should not panic
        clear_snapshot_rec_impl(std::ptr::null_mut());
    }

    #[test]
    fn test_clear_snapshot_null_tp() {
        // Should not panic
        clear_snapshot_impl(TabpageHandle::null(), 0);
    }

    #[test]
    fn test_get_snapshot_curwin_rec_null() {
        assert!(get_snapshot_curwin_rec_impl(std::ptr::null()).is_null());
    }

    #[test]
    fn test_check_snapshot_rec_null() {
        assert_eq!(
            check_snapshot_rec_impl(std::ptr::null(), std::ptr::null()),
            FAIL
        );
    }

    #[test]
    fn test_restore_snapshot_rec_null() {
        assert!(restore_snapshot_rec_impl(std::ptr::null(), std::ptr::null_mut()).is_null());
    }

    #[test]
    fn test_ok_fail_constants() {
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
    }
}
