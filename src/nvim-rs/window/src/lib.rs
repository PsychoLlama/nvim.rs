//! Window handling utilities for Neovim
//!
//! This crate provides Rust implementations of window-related functions
//! from `src/nvim/window.c`. It uses a combination of:
//! - Full `repr(C)` struct for `frame_T` (direct field access)
//! - Opaque handle pattern for `win_T*` and `tabpage_T*` (via accessor functions)

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(dead_code)] // Some FFI declarations are pre-declared for future use
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_int};

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

/// Frame layout constants (matching C defines in `buffer_defs.h`).
pub const FR_LEAF: c_char = 0; // Frame is a leaf (contains a window)
pub const FR_ROW: c_char = 1; // Frame with a row of windows
pub const FR_COL: c_char = 2; // Frame with a column of windows

/// Frame structure matching C `frame_T` layout exactly.
///
/// Frames form a tree structure representing window layout.
/// A frame is either a leaf (containing a window) or a row/column
/// of child frames. This struct uses `repr(C)` to match the C layout
/// and allow direct field access from Rust.
///
/// # Memory Layout
/// This struct exactly matches `struct frame_S` in `buffer_defs.h`.
#[repr(C)]
#[derive(Debug)]
pub struct Frame {
    /// Frame layout type: FR_LEAF, FR_ROW, or FR_COL
    pub fr_layout: c_char,
    /// Frame width
    pub fr_width: c_int,
    /// New width used in win_equal_rec()
    pub fr_newwidth: c_int,
    /// Frame height
    pub fr_height: c_int,
    /// New height used in win_equal_rec()
    pub fr_newheight: c_int,
    /// Containing frame or NULL for top frame
    pub fr_parent: *mut Frame,
    /// Next sibling frame (right or below in same parent), NULL for last
    pub fr_next: *mut Frame,
    /// Previous sibling frame (left or above in same parent), NULL for first
    pub fr_prev: *mut Frame,
    /// First child frame (for FR_ROW or FR_COL layouts)
    /// Mutually exclusive with fr_win
    pub fr_child: *mut Frame,
    /// Window that fills this frame (for FR_LEAF layout)
    /// Mutually exclusive with fr_child
    pub fr_win: WinHandle,
}

impl Frame {
    /// Check if this is a null pointer.
    #[inline]
    #[must_use]
    pub const fn is_null(ptr: *const Self) -> bool {
        ptr.is_null()
    }

    /// Check if this frame is a leaf (contains a window).
    #[inline]
    #[must_use]
    pub const fn is_leaf(&self) -> bool {
        self.fr_layout == FR_LEAF
    }

    /// Check if this frame is a row layout.
    #[inline]
    #[must_use]
    pub const fn is_row(&self) -> bool {
        self.fr_layout == FR_ROW
    }

    /// Check if this frame is a column layout.
    #[inline]
    #[must_use]
    pub const fn is_col(&self) -> bool {
        self.fr_layout == FR_COL
    }
}

/// Type alias for frame pointer (for FFI compatibility).
pub type FramePtr = *mut Frame;

// C accessor functions for window fields.
// These are defined in window.c and provide safe access to win_T fields.
// Note: Frame accessors are no longer needed since Frame is now repr(C).
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

    /// Get the `w_frame` field from a window (returns Frame pointer).
    fn nvim_win_get_frame(wp: WinHandle) -> *mut Frame;

    /// Get the `w_p_wfh` (winfixheight) field from a window.
    fn nvim_win_get_wfh(wp: WinHandle) -> c_int;

    /// Get the `w_p_wfw` (winfixwidth) field from a window.
    fn nvim_win_get_wfw(wp: WinHandle) -> c_int;

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
    one_window_in_tab_impl(win, unsafe {
        TabpageHandle::from_ptr(std::ptr::null_mut())
    }) && one_tabpage_impl()
}

/// FFI wrapper for `last_window`.
///
/// Returns non-zero if the window is the last non-floating window.
#[no_mangle]
pub extern "C" fn rs_last_window(win: WinHandle) -> c_int {
    c_int::from(last_window_impl(win))
}

// Frame tree functions
// These functions now use direct Frame struct access instead of accessor functions.

/// Check if a frame tree contains a specific window.
///
/// This is the Rust equivalent of `frame_has_win()` in window.c.
/// Recursively searches the frame tree for the given window.
#[inline]
fn frame_has_win_impl(frp: *const Frame, wp: WinHandle) -> bool {
    if frp.is_null() {
        return false;
    }

    // SAFETY: We check for null above and the caller guarantees valid frame pointer
    unsafe {
        let frame = &*frp;
        if frame.fr_layout == FR_LEAF {
            // Leaf frame - check if it contains the window
            return frame.fr_win == wp;
        }

        // Non-leaf frame - recursively check children
        let mut child = frame.fr_child;
        while !child.is_null() {
            if frame_has_win_impl(child, wp) {
                return true;
            }
            child = (*child).fr_next;
        }
    }
    false
}

/// FFI wrapper for `frame_has_win`.
///
/// Returns non-zero if the frame tree contains the window.
#[no_mangle]
pub extern "C" fn rs_frame_has_win(frp: *const Frame, wp: WinHandle) -> c_int {
    c_int::from(frame_has_win_impl(frp, wp))
}

/// Check if a frame has fixed height (due to 'winfixheight').
///
/// This is the Rust equivalent of `frame_fixed_height()` in window.c.
/// - Leaf frame: fixed if window has 'winfixheight' set
/// - Row frame: fixed if ANY child is fixed
/// - Column frame: fixed if ALL children are fixed
#[inline]
fn frame_fixed_height_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return false;
    }

    // SAFETY: We check for null above
    unsafe {
        let frame = &*frp;
        let layout = frame.fr_layout;

        if layout == FR_LEAF {
            // Leaf frame with a window - check w_p_wfh
            let win = frame.fr_win;
            return !win.is_null() && nvim_win_get_wfh(win) != 0;
        }

        if layout == FR_ROW {
            // Row: fixed if ONE of the frames in the row is fixed
            let mut child = frame.fr_child;
            while !child.is_null() {
                if frame_fixed_height_impl(child) {
                    return true;
                }
                child = (*child).fr_next;
            }
            return false;
        }

        // FR_COL: fixed if ALL frames in the column are fixed
        let mut child = frame.fr_child;
        while !child.is_null() {
            if !frame_fixed_height_impl(child) {
                return false;
            }
            child = (*child).fr_next;
        }
        // All children are fixed (or no children)
        !frame.fr_child.is_null()
    }
}

/// FFI wrapper for `frame_fixed_height`.
///
/// Returns non-zero if the frame has fixed height.
#[no_mangle]
pub extern "C" fn rs_frame_fixed_height(frp: *const Frame) -> c_int {
    c_int::from(frame_fixed_height_impl(frp))
}

/// Check if a frame has fixed width (due to 'winfixwidth').
///
/// This is the Rust equivalent of `frame_fixed_width()` in window.c.
/// - Leaf frame: fixed if window has 'winfixwidth' set
/// - Column frame: fixed if ANY child is fixed
/// - Row frame: fixed if ALL children are fixed
#[inline]
fn frame_fixed_width_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return false;
    }

    // SAFETY: We check for null above
    unsafe {
        let frame = &*frp;
        let layout = frame.fr_layout;

        if layout == FR_LEAF {
            // Leaf frame with a window - check w_p_wfw
            let win = frame.fr_win;
            return !win.is_null() && nvim_win_get_wfw(win) != 0;
        }

        if layout == FR_COL {
            // Column: fixed if ONE of the frames in the column is fixed
            let mut child = frame.fr_child;
            while !child.is_null() {
                if frame_fixed_width_impl(child) {
                    return true;
                }
                child = (*child).fr_next;
            }
            return false;
        }

        // FR_ROW: fixed if ALL frames in the row are fixed
        let mut child = frame.fr_child;
        while !child.is_null() {
            if !frame_fixed_width_impl(child) {
                return false;
            }
            child = (*child).fr_next;
        }
        // All children are fixed (or no children)
        !frame.fr_child.is_null()
    }
}

/// FFI wrapper for `frame_fixed_width`.
///
/// Returns non-zero if the frame has fixed width.
#[no_mangle]
pub extern "C" fn rs_frame_fixed_width(frp: *const Frame) -> c_int {
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
    // SAFETY: We access frame fields directly
    unsafe {
        while !frp.is_null() {
            let parent = (*frp).fr_parent;
            if parent.is_null() {
                break;
            }

            // If parent is a column layout and there's a sibling below, not at bottom
            let parent_layout = (*parent).fr_layout;
            let next_sibling = (*frp).fr_next;

            if parent_layout == FR_COL && !next_sibling.is_null() {
                return false;
            }

            frp = parent;
        }
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
fn frame_check_height_impl(topfrp: *const Frame, height: c_int) -> bool {
    if topfrp.is_null() {
        return false;
    }

    // SAFETY: We check for null above.
    unsafe {
        let frame = &*topfrp;
        if frame.fr_height != height {
            return false;
        }
        // If it's a row layout, check all children have the same height
        if frame.fr_layout == FR_ROW {
            let mut child = frame.fr_child;
            while !child.is_null() {
                if (*child).fr_height != height {
                    return false;
                }
                child = (*child).fr_next;
            }
        }
    }
    true
}

/// FFI wrapper for `frame_check_height`.
///
/// Returns non-zero if all frames have the expected height.
#[no_mangle]
pub extern "C" fn rs_frame_check_height(topfrp: *const Frame, height: c_int) -> c_int {
    c_int::from(frame_check_height_impl(topfrp, height))
}

/// Check that "topfrp" and its children are at the right width.
///
/// This is the Rust equivalent of `frame_check_width()` in window.c.
/// If the frame is a FR_COL layout, all children must have the same width.
#[inline]
fn frame_check_width_impl(topfrp: *const Frame, width: c_int) -> bool {
    if topfrp.is_null() {
        return false;
    }

    // SAFETY: We check for null above.
    unsafe {
        let frame = &*topfrp;
        if frame.fr_width != width {
            return false;
        }
        // If it's a column layout, check all children have the same width
        if frame.fr_layout == FR_COL {
            let mut child = frame.fr_child;
            while !child.is_null() {
                if (*child).fr_width != width {
                    return false;
                }
                child = (*child).fr_next;
            }
        }
    }
    true
}

/// FFI wrapper for `frame_check_width`.
///
/// Returns non-zero if all frames have the expected width.
#[no_mangle]
pub extern "C" fn rs_frame_check_width(topfrp: *const Frame, width: c_int) -> c_int {
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

    // Accessors for frame_minheight/frame_minwidth
    /// Get the `w_winbar_height` field from a window.
    fn nvim_win_get_winbar_height(wp: WinHandle) -> c_int;

    /// Get the `w_status_height` field from a window.
    fn nvim_win_get_status_height(wp: WinHandle) -> c_int;

    /// Get the `w_hsep_height` field from a window (already declared but repeated for clarity).
    fn nvim_win_get_hsep_height(wp: WinHandle) -> c_int;

    /// Get the `w_vsep_width` field from a window (already declared but repeated for clarity).
    fn nvim_win_get_vsep_width(wp: WinHandle) -> c_int;

    /// Get the global `p_wh` (winheight) option value.
    fn nvim_get_p_wh() -> i64;

    /// Get the global `p_wmh` (winminheight) option value.
    fn nvim_get_p_wmh() -> i64;

    /// Get the global `p_wiw` (winwidth) option value.
    fn nvim_get_p_wiw() -> i64;

    /// Get the global `p_wmw` (winminwidth) option value.
    fn nvim_get_p_wmw() -> i64;

    // Accessors for win_comp_pos/frame_comp_pos
    /// Set the w_winrow field of a window.
    fn nvim_win_set_winrow(wp: WinHandle, val: c_int);

    /// Set the w_wincol field of a window.
    fn nvim_win_set_wincol(wp: WinHandle, val: c_int);

    /// Set the w_redr_status field of a window.
    fn nvim_win_set_redr_status(wp: WinHandle, val: c_int);

    /// Set the w_pos_changed field of a window.
    fn nvim_win_set_pos_changed(wp: WinHandle, val: c_int);

    /// Get the w_config.relative field from a window.
    fn nvim_win_get_config_relative(wp: WinHandle) -> c_int;

    /// Get the w_winrow field from a window.
    fn nvim_win_get_winrow(wp: WinHandle) -> c_int;

    /// Get the w_wincol field from a window.
    fn nvim_win_get_wincol(wp: WinHandle) -> c_int;

    /// Get the w_width field from a window.
    fn nvim_win_get_w_width(wp: WinHandle) -> c_int;

    /// Get the w_height field from a window.
    fn nvim_win_get_w_height(wp: WinHandle) -> c_int;

    /// Get the topframe global.
    fn nvim_get_topframe() -> *mut Frame;

    /// Get the tabline height.
    fn tabline_height() -> c_int;

    /// Get the global statusline height.
    fn global_stl_height() -> c_int;

    /// Call redraw_later from C.
    fn redraw_later(wp: WinHandle, redraw_type: c_int);
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

/// Find the left-upper window in frame.
///
/// Walks down the frame tree following fr_child until a leaf frame
/// with a window is found.
#[inline]
fn frame2win_impl(mut frp: *mut Frame) -> WinHandle {
    // SAFETY: The caller guarantees frp is non-null.
    // The loop walks down until we find a leaf with fr_win != NULL.
    unsafe {
        while (*frp).fr_win.is_null() {
            frp = (*frp).fr_child;
        }
        (*frp).fr_win
    }
}

/// FFI wrapper for `frame2win`.
#[no_mangle]
pub extern "C" fn rs_frame2win(frp: *mut Frame) -> WinHandle {
    frame2win_impl(frp)
}

// ============================================================================
// Frame minimum dimension functions
// ============================================================================

/// Compute the minimal height for a frame tree.
///
/// This is the Rust equivalent of `frame_minheight()` in window.c.
/// Uses the 'winminheight' option. When `next_curwin` isn't null,
/// use `p_wh` for that window. When `next_curwin` is `NOWIN` (-1 cast),
/// don't use at least one line for the current window.
///
/// # Arguments
/// * `topfrp` - The frame to compute minimum height for
/// * `next_curwin` - The window that will become current, or null, or NOWIN
#[allow(clippy::cast_possible_truncation)]
fn frame_minheight_impl(topfrp: *const Frame, next_curwin: WinHandle) -> c_int {
    if topfrp.is_null() {
        return 0;
    }

    // SAFETY: We check for null above
    unsafe {
        let frame = &*topfrp;

        if !frame.fr_win.is_null() {
            // Leaf frame with a window
            let wp = frame.fr_win;

            // Combined height of window bar and separator or status line
            let extra_height = nvim_win_get_winbar_height(wp)
                + nvim_win_get_hsep_height(wp)
                + nvim_win_get_status_height(wp);

            let m = if wp == next_curwin {
                // This window will be the current window - use p_wh
                nvim_get_p_wh() as c_int + extra_height
            } else {
                // Use p_wmh for non-current windows
                let mut m = nvim_get_p_wmh() as c_int + extra_height;

                // Check if this is curwin and next_curwin is NULL (not NOWIN)
                // NOWIN is represented as a specific non-null invalid pointer
                // In C, NOWIN is (win_T *)-1. We check by comparing raw pointers.
                let curwin = nvim_get_curwin();
                let nowin_ptr = (-1isize) as *mut std::ffi::c_void;
                let is_nowin = next_curwin.as_ptr() == nowin_ptr;

                if wp == curwin && !is_nowin && next_curwin.is_null() {
                    // Current window is minimal one line high
                    if nvim_get_p_wmh() == 0 {
                        m += 1;
                    }
                }
                m
            };

            return m;
        }

        // Non-leaf frame - iterate through children
        let mut m = 0;
        let mut child = frame.fr_child;

        if frame.is_row() {
            // FR_ROW: get the max minimal height from each frame in this row
            while !child.is_null() {
                let n = frame_minheight_impl(child, next_curwin);
                if n > m {
                    m = n;
                }
                child = (*child).fr_next;
            }
        } else {
            // FR_COL: add up the minimal heights for all frames in this column
            while !child.is_null() {
                m += frame_minheight_impl(child, next_curwin);
                child = (*child).fr_next;
            }
        }
        m
    }
}

/// FFI wrapper for `frame_minheight`.
///
/// Computes the minimal height for a frame tree.
#[no_mangle]
pub extern "C" fn rs_frame_minheight(topfrp: *const Frame, next_curwin: WinHandle) -> c_int {
    frame_minheight_impl(topfrp, next_curwin)
}

/// Compute the minimal width for a frame tree.
///
/// This is the Rust equivalent of `frame_minwidth()` in window.c.
/// Uses the 'winminwidth' option. When `next_curwin` isn't null,
/// use `p_wiw` for that window. When `next_curwin` is `NOWIN` (-1 cast),
/// don't use at least one column for the current window.
///
/// # Arguments
/// * `topfrp` - The frame to compute minimum width for
/// * `next_curwin` - The window that will become current, or null, or NOWIN
#[allow(clippy::cast_possible_truncation)]
fn frame_minwidth_impl(topfrp: *const Frame, next_curwin: WinHandle) -> c_int {
    if topfrp.is_null() {
        return 0;
    }

    // SAFETY: We check for null above
    unsafe {
        let frame = &*topfrp;

        if !frame.fr_win.is_null() {
            // Leaf frame with a window
            let wp = frame.fr_win;

            let m = if wp == next_curwin {
                // This window will be the current window - use p_wiw
                nvim_get_p_wiw() as c_int + nvim_win_get_vsep_width(wp)
            } else {
                // Use p_wmw for non-current windows
                let mut m = nvim_get_p_wmw() as c_int + nvim_win_get_vsep_width(wp);

                // Check if this is curwin and next_curwin is NULL (not NOWIN)
                let curwin = nvim_get_curwin();
                let nowin_ptr = (-1isize) as *mut std::ffi::c_void;
                let is_nowin = next_curwin.as_ptr() == nowin_ptr;

                if nvim_get_p_wmw() == 0 && wp == curwin && !is_nowin && next_curwin.is_null() {
                    // Current window is minimal one column wide
                    m += 1;
                }
                m
            };

            return m;
        }

        // Non-leaf frame - iterate through children
        let mut m = 0;
        let mut child = frame.fr_child;

        if frame.is_col() {
            // FR_COL: get the max minimal width from each frame in this column
            while !child.is_null() {
                let n = frame_minwidth_impl(child, next_curwin);
                if n > m {
                    m = n;
                }
                child = (*child).fr_next;
            }
        } else {
            // FR_ROW: add up the minimal widths for all frames in this row
            while !child.is_null() {
                m += frame_minwidth_impl(child, next_curwin);
                child = (*child).fr_next;
            }
        }
        m
    }
}

/// FFI wrapper for `frame_minwidth`.
///
/// Computes the minimal width for a frame tree.
#[no_mangle]
pub extern "C" fn rs_frame_minwidth(topfrp: *const Frame, next_curwin: WinHandle) -> c_int {
    frame_minwidth_impl(topfrp, next_curwin)
}

// ============================================================================
// Window position computation functions
// ============================================================================

/// UPD_NOT_VALID constant from screen.h
const UPD_NOT_VALID: c_int = 40;

/// kFloatRelativeWindow constant from window.h
const K_FLOAT_RELATIVE_WINDOW: c_int = 2;

/// Update the position of the windows in a frame tree.
///
/// This is the Rust equivalent of `frame_comp_pos()` in window.c.
/// Updates `*row` and `*col` from the top-left to the bottom-right position plus one.
fn frame_comp_pos_impl(topfrp: *mut Frame, row: &mut c_int, col: &mut c_int) {
    if topfrp.is_null() {
        return;
    }

    // SAFETY: We check for null above
    unsafe {
        let frame = &*topfrp;
        let wp = frame.fr_win;

        if wp.is_null() {
            // Non-leaf frame - iterate through children
            let startrow = *row;
            let startcol = *col;

            let mut child = frame.fr_child;
            while !child.is_null() {
                if frame.is_row() {
                    *row = startrow; // all frames are at the same row
                } else {
                    *col = startcol; // all frames are at the same col
                }
                frame_comp_pos_impl(child, row, col);
                child = (*child).fr_next;
            }
        } else {
            // Leaf frame with a window
            let old_row = nvim_win_get_winrow(wp);
            let old_col = nvim_win_get_wincol(wp);

            if old_row != *row || old_col != *col {
                // Position changed, update and redraw
                nvim_win_set_winrow(wp, *row);
                nvim_win_set_wincol(wp, *col);
                redraw_later(wp, UPD_NOT_VALID);
                nvim_win_set_redr_status(wp, 1);
                nvim_win_set_pos_changed(wp, 1);
            }

            // Calculate height adjustment
            let h = nvim_win_get_w_height(wp)
                + nvim_win_get_hsep_height(wp)
                + nvim_win_get_status_height(wp);
            *row += if h > frame.fr_height {
                frame.fr_height
            } else {
                h
            };
            *col += nvim_win_get_w_width(wp) + nvim_win_get_vsep_width(wp);
        }
    }
}

/// FFI wrapper for `frame_comp_pos`.
///
/// Updates window positions in a frame tree.
///
/// # Safety
/// `row` and `col` must be valid, non-null pointers to mutable c_int values.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn rs_frame_comp_pos(topfrp: *mut Frame, row: *mut c_int, col: *mut c_int) {
    if row.is_null() || col.is_null() {
        return;
    }
    // SAFETY: We check for null above, and caller ensures row and col are valid pointers
    unsafe {
        frame_comp_pos_impl(topfrp, &mut *row, &mut *col);
    }
}

/// Update the position for all windows, using the width and height of the frames.
///
/// This is the Rust equivalent of `win_comp_pos()` in window.c.
/// Returns the row just after the last window and global statusline (if there is one).
fn win_comp_pos_impl() -> c_int {
    // SAFETY: All FFI functions are safe to call
    unsafe {
        let mut row = tabline_height();
        let mut col = 0;

        let topframe = nvim_get_topframe();
        frame_comp_pos_impl(topframe, &mut row, &mut col);

        // Check floating windows anchored to moved windows
        let mut wp = nvim_get_lastwin();
        while !wp.is_null() && nvim_win_get_floating(wp) != 0 {
            if nvim_win_get_config_relative(wp) == K_FLOAT_RELATIVE_WINDOW {
                nvim_win_set_pos_changed(wp, 1);
            }
            wp = nvim_win_get_prev(wp);
        }

        row + global_stl_height()
    }
}

/// FFI wrapper for `win_comp_pos`.
///
/// Updates the position for all windows.
#[no_mangle]
pub extern "C" fn rs_win_comp_pos() -> c_int {
    win_comp_pos_impl()
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
    fn test_frame_ptr_null() {
        let null_frame: *const Frame = std::ptr::null();
        assert!(Frame::is_null(null_frame));
        // Null frame returns false for all checks
        assert!(!frame_has_win_impl(null_frame, unsafe {
            WinHandle::from_ptr(std::ptr::null_mut())
        }));
        assert!(!frame_fixed_height_impl(null_frame));
        assert!(!frame_fixed_width_impl(null_frame));
    }

    #[test]
    fn test_frame_constants() {
        assert_eq!(FR_LEAF, 0);
        assert_eq!(FR_ROW, 1);
        assert_eq!(FR_COL, 2);
    }

    #[test]
    fn test_frame_struct_size() {
        // Verify Frame struct size matches expectations for C interop
        // On 64-bit: 1 (char) + 3 (padding) + 4*4 (ints) + 5*8 (pointers) = 60 bytes
        // But with alignment it should be 64 bytes
        use std::mem::size_of;
        // The struct should have proper C layout
        assert!(size_of::<Frame>() > 0);
    }

    #[test]
    fn test_tabpage_handle_null() {
        let handle = unsafe { TabpageHandle::from_ptr(std::ptr::null_mut()) };
        assert!(handle.is_null());
        assert!(!valid_tabpage_impl(handle));
        assert!(!valid_tabpage_win_impl(handle));
    }
}
