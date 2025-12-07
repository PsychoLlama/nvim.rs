//! Window handling utilities for Neovim
//!
//! This crate provides Rust implementations of window-related functions
//! from `src/nvim/window.c`. It uses an opaque handle pattern where
//! `win_T*` pointers are treated as opaque handles, with field access
//! done through C accessor functions.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(dead_code)] // Some FFI declarations are pre-declared for future use

use std::ffi::c_int;

/// Opaque handle to a Neovim window (`win_T*`).
///
/// This is an opaque pointer type - Rust code should not attempt to
/// dereference or inspect the contents. All field access is done
/// through C accessor functions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WinHandle(*mut std::ffi::c_void);

impl WinHandle {
    /// Create a new window handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `win_T*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a Neovim tabpage (`tabpage_T*`).
///
/// This is an opaque pointer type - Rust code should not attempt to
/// dereference or inspect the contents. All field access is done
/// through C accessor functions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabpageHandle(*mut std::ffi::c_void);

impl TabpageHandle {
    /// Create a new tabpage handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `tabpage_T*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a Neovim frame (`frame_T*`).
///
/// Frames form a tree structure representing window layout.
/// A frame is either a leaf (containing a window) or a row/column
/// of child frames.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameHandle(*mut std::ffi::c_void);

impl FrameHandle {
    /// Create a new frame handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `frame_T*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Frame layout constants (matching C defines in `buffer_defs.h`).
pub const FR_LEAF: c_int = 0; // Frame is a leaf (contains a window)
pub const FR_ROW: c_int = 1; // Frame with a row of windows
pub const FR_COL: c_int = 2; // Frame with a column of windows

// C accessor functions for window fields.
// These are defined in window.c and provide safe access to win_T fields.
extern "C" {
    /// Get the `w_locked` field from a window.
    fn nvim_win_get_locked(win: WinHandle) -> c_int;

    /// Get the `w_floating` field from a window.
    fn nvim_win_get_floating(win: WinHandle) -> c_int;

    /// Get the `w_p_pvw` (preview window) field from a window.
    fn nvim_win_get_pvw(win: WinHandle) -> c_int;

    /// Get the `w_next` field from a window.
    fn nvim_win_get_next(win: WinHandle) -> WinHandle;

    /// Get the `w_prev` field from a window.
    fn nvim_win_get_prev(win: WinHandle) -> WinHandle;

    // Global state accessors
    /// Get the current window.
    fn nvim_get_curwin() -> WinHandle;

    /// Get the first window in the current tab.
    fn nvim_get_firstwin() -> WinHandle;

    /// Get the last window in the current tab.
    fn nvim_get_lastwin() -> WinHandle;

    /// Get the current tabpage.
    fn nvim_get_curtab() -> TabpageHandle;

    /// Get the `tp_firstwin` field from a tabpage.
    fn nvim_tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle;

    /// Get the `tp_next` field from a tabpage.
    fn nvim_tabpage_get_next(tp: TabpageHandle) -> TabpageHandle;

    /// Get the first tabpage (`first_tabpage` global).
    fn nvim_get_first_tabpage() -> TabpageHandle;

    // Frame accessors
    /// Get the `fr_layout` field from a frame (`FR_LEAF`, `FR_ROW`, or `FR_COL`).
    fn nvim_frame_get_layout(frp: FrameHandle) -> c_int;

    /// Get the `fr_win` field from a frame (window in a leaf frame).
    fn nvim_frame_get_win(frp: FrameHandle) -> WinHandle;

    /// Get the `fr_child` field from a frame (first child frame).
    fn nvim_frame_get_child(frp: FrameHandle) -> FrameHandle;

    /// Get the `fr_next` field from a frame (next sibling).
    fn nvim_frame_get_next(frp: FrameHandle) -> FrameHandle;

    /// Get the `fr_parent` field from a frame.
    fn nvim_frame_get_parent(frp: FrameHandle) -> FrameHandle;

    /// Get the `w_frame` field from a window.
    fn nvim_win_get_frame(wp: WinHandle) -> FrameHandle;

    /// Get the `w_p_wfh` (winfixheight) field from a window.
    fn nvim_win_get_wfh(wp: WinHandle) -> c_int;

    /// Get the `w_p_wfw` (winfixwidth) field from a window.
    fn nvim_win_get_wfw(wp: WinHandle) -> c_int;

    /// Get the `fr_height` field from a frame.
    fn nvim_frame_get_height(frp: FrameHandle) -> c_int;

    /// Get the `fr_width` field from a frame.
    fn nvim_frame_get_width(frp: FrameHandle) -> c_int;

    /// Get the `handle` field from a window.
    fn nvim_win_get_handle(wp: WinHandle) -> c_int;
}

/// Check if a window is locked (`w_locked` field).
///
/// A locked window cannot be closed by autocommands.
#[inline]
fn win_locked_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    // SAFETY: We check for null above, and nvim_win_get_locked
    // is a simple field accessor that handles the pointer safely.
    unsafe { nvim_win_get_locked(wp) != 0 }
}

/// FFI wrapper for `win_locked`.
///
/// Returns non-zero if the window is locked.
#[no_mangle]
pub extern "C" fn rs_win_locked(wp: WinHandle) -> c_int {
    c_int::from(win_locked_impl(wp))
}

/// Check if a window is floating (`w_floating` field).
///
/// A floating window is a popup window that appears above other windows.
#[inline]
fn win_floating_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    // SAFETY: We check for null above, and nvim_win_get_floating
    // is a simple field accessor that handles the pointer safely.
    unsafe { nvim_win_get_floating(wp) != 0 }
}

/// FFI wrapper for `win_floating`.
///
/// Returns non-zero if the window is floating.
#[no_mangle]
pub extern "C" fn rs_win_floating(wp: WinHandle) -> c_int {
    c_int::from(win_floating_impl(wp))
}

/// Check if a window is a preview window (`w_p_pvw` field).
///
/// A preview window is used for displaying preview information.
#[inline]
fn win_pvw_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    // SAFETY: We check for null above, and nvim_win_get_pvw
    // is a simple field accessor that handles the pointer safely.
    unsafe { nvim_win_get_pvw(wp) != 0 }
}

/// FFI wrapper for `win_pvw`.
///
/// Returns non-zero if the window is a preview window.
#[no_mangle]
pub extern "C" fn rs_win_pvw(wp: WinHandle) -> c_int {
    c_int::from(win_pvw_impl(wp))
}

// Window iteration helpers

/// Get the first window in a tabpage.
///
/// For the current tabpage, this returns `firstwin`. For other tabpages,
/// it returns `tp->tp_firstwin`.
#[inline]
fn get_tabpage_firstwin(tp: TabpageHandle) -> WinHandle {
    // SAFETY: nvim_get_curtab returns a valid tabpage handle (or the check would be invalid)
    // and nvim_get_firstwin/nvim_tabpage_get_firstwin are safe accessors.
    unsafe {
        if tp == nvim_get_curtab() {
            nvim_get_firstwin()
        } else {
            nvim_tabpage_get_firstwin(tp)
        }
    }
}

/// Check if "win" is a pointer to an existing window in tabpage "tp".
///
/// This is the Rust equivalent of `tabpage_win_valid()` in window.c.
#[inline]
fn tabpage_win_valid_impl(tp: TabpageHandle, win: WinHandle) -> bool {
    if win.is_null() {
        return false;
    }

    let mut wp = get_tabpage_firstwin(tp);
    while !wp.is_null() {
        if wp == win {
            return true;
        }
        // SAFETY: nvim_win_get_next is a safe field accessor
        wp = unsafe { nvim_win_get_next(wp) };
    }
    false
}

/// FFI wrapper for `tabpage_win_valid`.
///
/// Returns non-zero if the window is valid in the given tabpage.
#[no_mangle]
pub extern "C" fn rs_tabpage_win_valid(tp: TabpageHandle, win: WinHandle) -> c_int {
    c_int::from(tabpage_win_valid_impl(tp, win))
}

/// Check if "win" is a pointer to an existing window in the current tabpage.
///
/// This is the Rust equivalent of `win_valid()` in window.c.
#[inline]
fn win_valid_impl(win: WinHandle) -> bool {
    // SAFETY: nvim_get_curtab returns a valid tabpage handle
    tabpage_win_valid_impl(unsafe { nvim_get_curtab() }, win)
}

/// FFI wrapper for `win_valid`.
///
/// Returns non-zero if the window is valid in the current tabpage.
#[no_mangle]
pub extern "C" fn rs_win_valid(win: WinHandle) -> c_int {
    c_int::from(win_valid_impl(win))
}

/// Check if there is only one window in the current tabpage (excluding floating windows).
///
/// This is the Rust equivalent of the `ONE_WINDOW` macro, which checks `firstwin == lastwin`.
#[inline]
fn one_window_impl() -> bool {
    // SAFETY: nvim_get_firstwin and nvim_get_lastwin are safe accessors
    unsafe { nvim_get_firstwin() == nvim_get_lastwin() }
}

/// FFI wrapper for checking if there's only one window.
///
/// Returns non-zero if there is only one window in the current tabpage.
#[no_mangle]
pub extern "C" fn rs_one_window() -> c_int {
    c_int::from(one_window_impl())
}

/// Check if "win" is a pointer to an existing window in any tabpage.
///
/// This is the Rust equivalent of `win_valid_any_tab()` in window.c.
#[inline]
fn win_valid_any_tab_impl(win: WinHandle) -> bool {
    if win.is_null() {
        return false;
    }

    // Iterate over all tabpages using FOR_ALL_TABS pattern
    // SAFETY: nvim_get_first_tabpage and nvim_tabpage_get_next are safe accessors
    let mut tp = unsafe { nvim_get_first_tabpage() };
    while !tp.is_null() {
        if tabpage_win_valid_impl(tp, win) {
            return true;
        }
        tp = unsafe { nvim_tabpage_get_next(tp) };
    }
    false
}

/// FFI wrapper for `win_valid_any_tab`.
///
/// Returns non-zero if the window is valid in any tabpage.
#[no_mangle]
pub extern "C" fn rs_win_valid_any_tab(win: WinHandle) -> c_int {
    c_int::from(win_valid_any_tab_impl(win))
}

/// Check if "tpc" is a pointer to an existing tabpage.
///
/// This is the Rust equivalent of `valid_tabpage()` in window.c.
#[inline]
fn valid_tabpage_impl(tpc: TabpageHandle) -> bool {
    if tpc.is_null() {
        return false;
    }

    // Iterate over all tabpages using FOR_ALL_TABS pattern
    // SAFETY: nvim_get_first_tabpage and nvim_tabpage_get_next are safe accessors
    let mut tp = unsafe { nvim_get_first_tabpage() };
    while !tp.is_null() {
        if tp == tpc {
            return true;
        }
        tp = unsafe { nvim_tabpage_get_next(tp) };
    }
    false
}

/// FFI wrapper for `valid_tabpage`.
///
/// Returns non-zero if the tabpage is valid.
#[no_mangle]
pub extern "C" fn rs_valid_tabpage(tpc: TabpageHandle) -> c_int {
    c_int::from(valid_tabpage_impl(tpc))
}

/// Check if there is only one tabpage (i.e., `first_tabpage->tp_next == NULL`).
///
/// This is used by `last_window()` to check if there's only one tab.
#[inline]
fn one_tabpage_impl() -> bool {
    // SAFETY: nvim_get_first_tabpage and nvim_tabpage_get_next are safe accessors
    unsafe {
        let first = nvim_get_first_tabpage();
        nvim_tabpage_get_next(first).is_null()
    }
}

/// FFI wrapper for checking if there's only one tabpage.
///
/// Returns non-zero if there is only one tabpage.
#[no_mangle]
pub extern "C" fn rs_one_tabpage() -> c_int {
    c_int::from(one_tabpage_impl())
}

/// Check if "win" is the only non-floating window in a tabpage.
///
/// For `tp == NULL` (current tabpage), uses `firstwin`.
/// Otherwise uses `tp->tp_firstwin`.
///
/// This is the Rust equivalent of `one_window()` in window.c.
/// Note: The C version has an assert that `(!tp || tp != curtab) && !first->w_floating`,
/// meaning tp should not be curtab when non-NULL, and the first window should not be floating.
/// We don't check the assert here as the caller is responsible for ensuring this.
#[inline]
fn one_window_in_tab_impl(win: WinHandle, tp: TabpageHandle) -> bool {
    if win.is_null() {
        return false;
    }

    // Get the first window in the tabpage
    // SAFETY: All accessors are safe
    let first = if tp.is_null() {
        unsafe { nvim_get_firstwin() }
    } else {
        unsafe { nvim_tabpage_get_firstwin(tp) }
    };

    if first != win {
        return false;
    }

    // Check if win->w_next is NULL or floating
    // SAFETY: nvim_win_get_next and nvim_win_get_floating are safe accessors
    let next = unsafe { nvim_win_get_next(win) };
    next.is_null() || unsafe { nvim_win_get_floating(next) != 0 }
}

/// FFI wrapper for `one_window`.
///
/// Returns non-zero if the window is the only non-floating window in the tabpage.
#[no_mangle]
pub extern "C" fn rs_one_window_in_tab(win: WinHandle, tp: TabpageHandle) -> c_int {
    c_int::from(one_window_in_tab_impl(win, tp))
}

/// Check if "win" is the last non-floating window that exists.
///
/// This checks: `one_window(win, NULL) && first_tabpage->tp_next == NULL`.
///
/// This is the Rust equivalent of `last_window()` in window.c.
#[inline]
fn last_window_impl(win: WinHandle) -> bool {
    // Check if there's only one non-floating window in current tabpage
    // AND there's only one tabpage
    one_window_in_tab_impl(win, unsafe { TabpageHandle::from_ptr(std::ptr::null_mut()) })
        && one_tabpage_impl()
}

/// FFI wrapper for `last_window`.
///
/// Returns non-zero if the window is the last non-floating window.
#[no_mangle]
pub extern "C" fn rs_last_window(win: WinHandle) -> c_int {
    c_int::from(last_window_impl(win))
}

// Frame tree functions

/// Check if a frame tree contains a specific window.
///
/// This is the Rust equivalent of `frame_has_win()` in window.c.
/// Recursively searches the frame tree for the given window.
#[inline]
fn frame_has_win_impl(frp: FrameHandle, wp: WinHandle) -> bool {
    if frp.is_null() {
        return false;
    }

    // SAFETY: All accessors handle pointers safely
    unsafe {
        if nvim_frame_get_layout(frp) == FR_LEAF {
            // Leaf frame - check if it contains the window
            return nvim_frame_get_win(frp) == wp;
        }

        // Non-leaf frame - recursively check children
        let mut child = nvim_frame_get_child(frp);
        while !child.is_null() {
            if frame_has_win_impl(child, wp) {
                return true;
            }
            child = nvim_frame_get_next(child);
        }
    }
    false
}

/// FFI wrapper for `frame_has_win`.
///
/// Returns non-zero if the frame tree contains the window.
#[no_mangle]
pub extern "C" fn rs_frame_has_win(frp: FrameHandle, wp: WinHandle) -> c_int {
    c_int::from(frame_has_win_impl(frp, wp))
}

/// Check if a frame has fixed height (due to 'winfixheight').
///
/// This is the Rust equivalent of `frame_fixed_height()` in window.c.
/// - Leaf frame: fixed if window has 'winfixheight' set
/// - Row frame: fixed if ANY child is fixed
/// - Column frame: fixed if ALL children are fixed
#[inline]
fn frame_fixed_height_impl(frp: FrameHandle) -> bool {
    if frp.is_null() {
        return false;
    }

    // SAFETY: All accessors handle pointers safely
    unsafe {
        let layout = nvim_frame_get_layout(frp);

        if layout == FR_LEAF {
            // Leaf frame with a window - check w_p_wfh
            let win = nvim_frame_get_win(frp);
            return !win.is_null() && nvim_win_get_wfh(win) != 0;
        }

        if layout == FR_ROW {
            // Row: fixed if ONE of the frames in the row is fixed
            let mut child = nvim_frame_get_child(frp);
            while !child.is_null() {
                if frame_fixed_height_impl(child) {
                    return true;
                }
                child = nvim_frame_get_next(child);
            }
            return false;
        }

        // FR_COL: fixed if ALL frames in the column are fixed
        let mut child = nvim_frame_get_child(frp);
        while !child.is_null() {
            if !frame_fixed_height_impl(child) {
                return false;
            }
            child = nvim_frame_get_next(child);
        }
        // All children are fixed (or no children)
        !nvim_frame_get_child(frp).is_null()
    }
}

/// FFI wrapper for `frame_fixed_height`.
///
/// Returns non-zero if the frame has fixed height.
#[no_mangle]
pub extern "C" fn rs_frame_fixed_height(frp: FrameHandle) -> c_int {
    c_int::from(frame_fixed_height_impl(frp))
}

/// Check if a frame has fixed width (due to 'winfixwidth').
///
/// This is the Rust equivalent of `frame_fixed_width()` in window.c.
/// - Leaf frame: fixed if window has 'winfixwidth' set
/// - Column frame: fixed if ANY child is fixed
/// - Row frame: fixed if ALL children are fixed
#[inline]
fn frame_fixed_width_impl(frp: FrameHandle) -> bool {
    if frp.is_null() {
        return false;
    }

    // SAFETY: All accessors handle pointers safely
    unsafe {
        let layout = nvim_frame_get_layout(frp);

        if layout == FR_LEAF {
            // Leaf frame with a window - check w_p_wfw
            let win = nvim_frame_get_win(frp);
            return !win.is_null() && nvim_win_get_wfw(win) != 0;
        }

        if layout == FR_COL {
            // Column: fixed if ONE of the frames in the column is fixed
            let mut child = nvim_frame_get_child(frp);
            while !child.is_null() {
                if frame_fixed_width_impl(child) {
                    return true;
                }
                child = nvim_frame_get_next(child);
            }
            return false;
        }

        // FR_ROW: fixed if ALL frames in the row are fixed
        let mut child = nvim_frame_get_child(frp);
        while !child.is_null() {
            if !frame_fixed_width_impl(child) {
                return false;
            }
            child = nvim_frame_get_next(child);
        }
        // All children are fixed (or no children)
        !nvim_frame_get_child(frp).is_null()
    }
}

/// FFI wrapper for `frame_fixed_width`.
///
/// Returns non-zero if the frame has fixed width.
#[no_mangle]
pub extern "C" fn rs_frame_fixed_width(frp: FrameHandle) -> c_int {
    c_int::from(frame_fixed_width_impl(frp))
}

/// Check if window is at the bottom of its column.
///
/// This is the Rust equivalent of `is_bottom_win()` in window.c.
/// Returns true if there are no windows below the current window.
/// Traverses up the frame tree, checking if any parent is a column
/// layout with a sibling frame below.
#[inline]
fn is_bottom_win_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return true;
    }

    // Get the window's frame
    // SAFETY: wp is not null, and nvim_win_get_frame is a safe accessor
    let mut frp = unsafe { nvim_win_get_frame(wp) };

    // Traverse up the frame tree
    loop {
        // SAFETY: Safe accessor
        let parent = unsafe { nvim_frame_get_parent(frp) };
        if parent.is_null() {
            break;
        }

        // If parent is a column layout and there's a sibling below, not at bottom
        // SAFETY: Safe accessors
        let parent_layout = unsafe { nvim_frame_get_layout(parent) };
        let next_sibling = unsafe { nvim_frame_get_next(frp) };

        if parent_layout == FR_COL && !next_sibling.is_null() {
            return false;
        }

        frp = parent;
    }
    true
}

/// FFI wrapper for `is_bottom_win`.
///
/// Returns non-zero if the window is at the bottom.
#[no_mangle]
pub extern "C" fn rs_is_bottom_win(wp: WinHandle) -> c_int {
    c_int::from(is_bottom_win_impl(wp))
}

/// Check that "topfrp" and its children are at the right height.
///
/// This is the Rust equivalent of `frame_check_height()` in window.c.
/// If the frame is a FR_ROW layout, all children must have the same height.
#[inline]
fn frame_check_height_impl(topfrp: FrameHandle, height: c_int) -> bool {
    if topfrp.is_null() {
        return false;
    }

    // SAFETY: We check for null above.
    unsafe {
        if nvim_frame_get_height(topfrp) != height {
            return false;
        }
        // If it's a row layout, check all children have the same height
        if nvim_frame_get_layout(topfrp) == FR_ROW {
            let mut child = nvim_frame_get_child(topfrp);
            while !child.is_null() {
                if nvim_frame_get_height(child) != height {
                    return false;
                }
                child = nvim_frame_get_next(child);
            }
        }
    }
    true
}

/// FFI wrapper for `frame_check_height`.
///
/// Returns non-zero if all frames have the expected height.
#[no_mangle]
pub extern "C" fn rs_frame_check_height(topfrp: FrameHandle, height: c_int) -> c_int {
    c_int::from(frame_check_height_impl(topfrp, height))
}

/// Check that "topfrp" and its children are at the right width.
///
/// This is the Rust equivalent of `frame_check_width()` in window.c.
/// If the frame is a FR_COL layout, all children must have the same width.
#[inline]
fn frame_check_width_impl(topfrp: FrameHandle, width: c_int) -> bool {
    if topfrp.is_null() {
        return false;
    }

    // SAFETY: We check for null above.
    unsafe {
        if nvim_frame_get_width(topfrp) != width {
            return false;
        }
        // If it's a column layout, check all children have the same width
        if nvim_frame_get_layout(topfrp) == FR_COL {
            let mut child = nvim_frame_get_child(topfrp);
            while !child.is_null() {
                if nvim_frame_get_width(child) != width {
                    return false;
                }
                child = nvim_frame_get_next(child);
            }
        }
    }
    true
}

/// FFI wrapper for `frame_check_width`.
///
/// Returns non-zero if all frames have the expected width.
#[no_mangle]
pub extern "C" fn rs_frame_check_width(topfrp: FrameHandle, width: c_int) -> c_int {
    c_int::from(frame_check_width_impl(topfrp, width))
}

/// Find a window by its handle in the current tabpage.
///
/// This is the Rust equivalent of `win_find_by_handle()` in window.c.
/// Iterates through all windows in curtab, returning the one with the matching handle.
#[inline]
fn win_find_by_handle_impl(handle: c_int) -> WinHandle {
    // Get curtab to use FOR_ALL_WINDOWS_IN_TAB pattern
    // SAFETY: All accessors handle pointers safely
    let curtab = unsafe { nvim_get_curtab() };
    let mut wp = get_tabpage_firstwin(curtab);
    while !wp.is_null() {
        // SAFETY: nvim_win_get_handle is a safe accessor
        if unsafe { nvim_win_get_handle(wp) } == handle {
            return wp;
        }
        wp = unsafe { nvim_win_get_next(wp) };
    }
    // Return null if not found
    unsafe { WinHandle::from_ptr(std::ptr::null_mut()) }
}

/// FFI wrapper for `win_find_by_handle`.
///
/// Returns the window handle or NULL if not found.
#[no_mangle]
pub extern "C" fn rs_win_find_by_handle(handle: c_int) -> WinHandle {
    win_find_by_handle_impl(handle)
}

/// Find the tabpage that contains a given window.
///
/// This is the Rust equivalent of `win_find_tabpage()` in window.c.
/// Iterates through all tabpages and windows using FOR_ALL_TAB_WINDOWS pattern.
#[inline]
fn win_find_tabpage_impl(win: WinHandle) -> TabpageHandle {
    if win.is_null() {
        return unsafe { TabpageHandle::from_ptr(std::ptr::null_mut()) };
    }

    // FOR_ALL_TAB_WINDOWS pattern: iterate through all tabpages and their windows
    // SAFETY: All accessors handle pointers safely
    let mut tp = unsafe { nvim_get_first_tabpage() };
    while !tp.is_null() {
        // Iterate through windows in this tabpage
        let mut wp = get_tabpage_firstwin(tp);
        while !wp.is_null() {
            if wp == win {
                return tp;
            }
            wp = unsafe { nvim_win_get_next(wp) };
        }
        tp = unsafe { nvim_tabpage_get_next(tp) };
    }
    // Return null if not found
    unsafe { TabpageHandle::from_ptr(std::ptr::null_mut()) }
}

/// FFI wrapper for `win_find_tabpage`.
///
/// Returns the tabpage that contains the window or NULL if not found.
#[no_mangle]
pub extern "C" fn rs_win_find_tabpage(win: WinHandle) -> TabpageHandle {
    win_find_tabpage_impl(win)
}

/// Count the number of windows in the current tabpage.
///
/// This is the Rust equivalent of `win_count()` in window.c.
/// Iterates through all windows in the current tab (firstwin -> `w_next`).
#[inline]
fn win_count_impl() -> c_int {
    // SAFETY: nvim_get_firstwin and nvim_win_get_next are safe accessors
    let mut count: c_int = 0;
    let mut wp = unsafe { nvim_get_firstwin() };
    while !wp.is_null() {
        count += 1;
        wp = unsafe { nvim_win_get_next(wp) };
    }
    count
}

/// FFI wrapper for `win_count`.
#[no_mangle]
pub extern "C" fn rs_win_count() -> c_int {
    win_count_impl()
}

/// Get the 1-based index of a tabpage.
///
/// This is the Rust equivalent of `tabpage_index()` in window.c.
/// Iterates through tabpages from `first_tabpage` to find the index.
#[inline]
fn tabpage_index_impl(ftp: TabpageHandle) -> c_int {
    // SAFETY: nvim_get_first_tabpage and nvim_tabpage_get_next are safe accessors
    let mut i: c_int = 1;
    let mut tp = unsafe { nvim_get_first_tabpage() };
    while !tp.is_null() && tp != ftp {
        i += 1;
        tp = unsafe { nvim_tabpage_get_next(tp) };
    }
    i
}

/// FFI wrapper for `tabpage_index`.
#[no_mangle]
pub extern "C" fn rs_tabpage_index(ftp: TabpageHandle) -> c_int {
    tabpage_index_impl(ftp)
}

/// Check if a tabpage has any valid window.
///
/// This is the Rust equivalent of `valid_tabpage_win()` in window.c.
/// Iterates through all tabpages to find `tpc`, then checks if any window
/// in that tabpage is valid (using `win_valid_any_tab`).
#[inline]
fn valid_tabpage_win_impl(tpc: TabpageHandle) -> bool {
    if tpc.is_null() {
        return false;
    }

    // Find the tabpage in the list
    // SAFETY: All accessors handle pointers safely
    let mut tp = unsafe { nvim_get_first_tabpage() };
    while !tp.is_null() {
        if tp == tpc {
            // Found the tabpage - check if any window is valid
            let mut wp = get_tabpage_firstwin(tp);
            while !wp.is_null() {
                if win_valid_any_tab_impl(wp) {
                    return true;
                }
                wp = unsafe { nvim_win_get_next(wp) };
            }
            return false;
        }
        tp = unsafe { nvim_tabpage_get_next(tp) };
    }
    // Tabpage not found - shouldn't happen
    false
}

/// FFI wrapper for `valid_tabpage_win`.
///
/// Returns non-zero if the tabpage has at least one valid window.
#[no_mangle]
pub extern "C" fn rs_valid_tabpage_win(tpc: TabpageHandle) -> c_int {
    c_int::from(valid_tabpage_win_impl(tpc))
}

/// Find tab page by 1-based number.
///
/// This is the Rust equivalent of `find_tabpage()` in window.c.
/// Iterates through tabpages from `first_tabpage` counting to n.
/// Returns NULL when not found.
#[inline]
fn find_tabpage_impl(n: c_int) -> TabpageHandle {
    // SAFETY: nvim_get_first_tabpage and nvim_tabpage_get_next are safe accessors
    let mut i: c_int = 1;
    let mut tp = unsafe { nvim_get_first_tabpage() };
    while !tp.is_null() && i != n {
        i += 1;
        tp = unsafe { nvim_tabpage_get_next(tp) };
    }
    tp
}

/// FFI wrapper for `find_tabpage`.
///
/// Returns the tabpage at position n (1-based) or NULL if not found.
#[no_mangle]
pub extern "C" fn rs_find_tabpage(n: c_int) -> TabpageHandle {
    find_tabpage_impl(n)
}

// C accessor for last_win_id global
extern "C" {
    /// Get the `last_win_id` global.
    fn nvim_get_last_win_id() -> c_int;
}

/// Get the last window ID assigned.
///
/// This is the Rust equivalent of `get_last_winid()` in window.c.
/// Returns the global last_win_id value.
#[inline]
fn get_last_winid_impl() -> c_int {
    // SAFETY: nvim_get_last_win_id is a safe accessor
    unsafe { nvim_get_last_win_id() }
}

/// FFI wrapper for `get_last_winid`.
#[no_mangle]
pub extern "C" fn rs_get_last_winid() -> c_int {
    get_last_winid_impl()
}

/// Find the last non-floating window.
///
/// This is the Rust equivalent of `lastwin_nofloating()` in window.c.
/// Iterates backwards from `lastwin` to find the first non-floating window.
#[inline]
fn lastwin_nofloating_impl() -> WinHandle {
    // SAFETY: nvim_get_lastwin, nvim_win_get_prev, nvim_win_get_floating are safe accessors
    let mut res = unsafe { nvim_get_lastwin() };
    while !res.is_null() && unsafe { nvim_win_get_floating(res) } != 0 {
        res = unsafe { nvim_win_get_prev(res) };
    }
    res
}

/// FFI wrapper for `lastwin_nofloating`.
#[no_mangle]
pub extern "C" fn rs_lastwin_nofloating() -> WinHandle {
    lastwin_nofloating_impl()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_win_handle_null() {
        let handle = unsafe { WinHandle::from_ptr(std::ptr::null_mut()) };
        assert!(handle.is_null());
        assert!(!win_locked_impl(handle));
        assert!(!win_floating_impl(handle));
        assert!(!win_pvw_impl(handle));
    }

    #[test]
    fn test_win_handle_non_null() {
        // Create a fake non-null pointer for testing
        let fake_ptr = 0x1000 as *mut std::ffi::c_void;
        let handle = unsafe { WinHandle::from_ptr(fake_ptr) };
        assert!(!handle.is_null());
        assert_eq!(handle.as_ptr(), fake_ptr);
    }

    #[test]
    fn test_frame_handle_null() {
        let handle = unsafe { FrameHandle::from_ptr(std::ptr::null_mut()) };
        assert!(handle.is_null());
        // Null frame returns false for all checks
        assert!(!frame_has_win_impl(handle, unsafe {
            WinHandle::from_ptr(std::ptr::null_mut())
        }));
        assert!(!frame_fixed_height_impl(handle));
        assert!(!frame_fixed_width_impl(handle));
    }

    #[test]
    fn test_frame_constants() {
        assert_eq!(FR_LEAF, 0);
        assert_eq!(FR_ROW, 1);
        assert_eq!(FR_COL, 2);
    }

    #[test]
    fn test_tabpage_handle_null() {
        let handle = unsafe { TabpageHandle::from_ptr(std::ptr::null_mut()) };
        assert!(handle.is_null());
        assert!(!valid_tabpage_impl(handle));
        assert!(!valid_tabpage_win_impl(handle));
    }
}
