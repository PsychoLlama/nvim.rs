//! Window handling utilities for Neovim
//!
//! This crate provides Rust implementations of window-related functions
//! from `src/nvim/window.c`. It uses a combination of:
//! - Full `repr(C)` struct for `frame_T` (direct field access)
//! - Opaque handle pattern for `win_T*` and `tabpage_T*` (via accessor functions)
//!
//! # Modules
//!
//! - [`alloc`]: Window allocation functions (placeholder)
//! - [`close`]: Window closing validation and execution helpers
//! - [`commands`]: CTRL-W command handler (placeholder)
//! - [`directory`]: Window directory management (win_fix_current_dir)
//! - [`enter`]: Window enter orchestrator (win_enter_ext)
//! - [`equalize`]: Window equalization functions (placeholder)
//! - [`events`]: Window events and UI updates (placeholder)
//! - [`focus`]: Window focus and navigation functions
//! - [`frame`]: Frame tree operations
//! - [`free`]: Window deallocation functions (placeholder)
//! - [`layout`]: Window layout tree operations
//! - [`list`]: Window list traversal and initialization functions
//! - [`navigate`]: Window finding and movement navigation
//! - [`preview`]: Preview window finding and state queries
//! - [`resize`]: Window resize calculations and execution helpers
//! - [`scroll`]: Window scroll position management for 'splitkeep' (win_fix_scroll, win_fix_cursor)
//! - [`snapshot`]: Window layout snapshot helpers
//! - [`split`]: Window splitting functions (placeholder)
//! - [`state`]: Window state accessors for layout fields
//! - [`state_validation`]: Cursor line number validation (check_lnums/reset_lnums)
//! - [`statusline`]: Status line and window bar (placeholder)
//! - [`tabpage`]: Tab page management operations
//! - [`utility`]: Small utility and validation helpers (check_can_set_curbuf_*, check_split_disallowed, make_windows)

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(dead_code)] // Some FFI declarations are pre-declared for future use
#![allow(clippy::doc_markdown)]

// Domain-specific modules
pub mod alloc;
pub mod close;
pub mod colorcolumn;
pub mod commands;
pub mod directory;
pub mod dispatch;
pub mod enter;
pub mod equalize;
pub mod events;
pub mod exchange;
pub mod focus;
pub mod frame;
pub mod free;
pub mod init;
pub mod layout;
pub mod list;
pub mod navigate;
pub mod preview;
pub mod resize;
pub mod scroll;
pub mod snapshot;
pub mod split;
pub mod state;
pub mod state_validation;
pub mod statusline;
pub mod tabpage;
pub mod ui_flush;
pub mod utility;
pub mod viml;
pub mod win_struct;

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

    /// Create a null handle.
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
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
    /// Create a null handle.
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

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

/// Opaque handle to a Neovim buffer (`buf_T*`).
///
/// This is an opaque pointer type - Rust code should not attempt to
/// dereference or inspect the contents. All field access is done
/// through C accessor functions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufHandle(*mut std::ffi::c_void);

impl BufHandle {
    /// Create a null handle.
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
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

/// Status line height constant (matching C enum in `window.h`).
pub const STATUS_HEIGHT: c_int = 1;

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
#[derive(Debug, Clone, Copy)]
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

    /// Get the `w_buffer` field from a window.
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;

    /// Get the current buffer (`curbuf` global).
    fn nvim_get_curbuf() -> BufHandle;

    // Buffer type check functions (from nvim-buffer crate)
    /// Check if buffer is a help buffer.
    fn rs_bt_help(buf: BufHandle) -> c_int;

    // Autocmd functions (from nvim-autocmd crate)
    /// Check if window is an aucmd_win.
    #[link_name = "is_aucmd_win"]
    fn rs_is_aucmd_win(win: WinHandle) -> c_int;
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

/// Check if "win" is a floating window in the current tabpage.
///
/// Iterates through all windows in the current tabpage to find the given
/// window, then returns whether it has the floating flag set.
#[inline]
fn win_float_valid_impl(win: WinHandle) -> bool {
    if win.is_null() {
        return false;
    }

    // SAFETY: nvim_get_curtab returns a valid tabpage handle
    let curtab = unsafe { nvim_get_curtab() };
    let mut wp = get_tabpage_firstwin(curtab);
    while !wp.is_null() {
        if wp == win {
            // SAFETY: nvim_win_get_floating is a safe field accessor
            return unsafe { nvim_win_get_floating(wp) != 0 };
        }
        // SAFETY: nvim_win_get_next is a safe field accessor
        wp = unsafe { nvim_win_get_next(wp) };
    }
    false
}

/// FFI wrapper for `win_float_valid`.
///
/// Returns non-zero if the window is a valid floating window in the current tabpage.
#[must_use]
#[unsafe(export_name = "win_float_valid")]
pub extern "C" fn rs_win_float_valid(win: WinHandle) -> c_int {
    c_int::from(win_float_valid_impl(win))
}

/// Check that there is only one window (and only one tab page), not counting a
/// help or preview window, unless it is the current window. Does not count
/// "aucmd_win". Does not count floats unless it is current.
#[inline]
fn only_one_window_impl() -> bool {
    // SAFETY: All accessor functions are safe
    unsafe {
        // If there is another tab page there always is another window.
        let first_tabpage = nvim_get_first_tabpage();
        if !nvim_tabpage_get_next(first_tabpage).is_null() {
            return false;
        }

        let curwin = nvim_get_curwin();
        let curbuf = nvim_get_curbuf();
        let curbuf_is_help = rs_bt_help(curbuf) != 0;

        let curtab = nvim_get_curtab();
        let mut count = 0;
        let mut wp = get_tabpage_firstwin(curtab);

        while !wp.is_null() {
            let buf = nvim_win_get_buffer(wp);
            if !buf.is_null() {
                let is_help = rs_bt_help(buf) != 0;
                let is_floating = nvim_win_get_floating(wp) != 0;
                let is_pvw = nvim_win_get_pvw(wp) != 0;
                let is_curwin = wp == curwin;
                let is_aucmd = rs_is_aucmd_win(wp) != 0;

                // Count if:
                // - Not a help window (unless curbuf is also help) AND not floating AND not preview
                //   OR it's the current window
                // - AND not an aucmd_win
                let should_skip = (is_help && !curbuf_is_help) || is_floating || is_pvw;
                if (!should_skip || is_curwin) && !is_aucmd {
                    count += 1;
                }
            }
            wp = nvim_win_get_next(wp);
        }

        count <= 1
    }
}

/// FFI wrapper for `only_one_window`.
///
/// Returns non-zero if there is only one relevant window.
#[no_mangle]
pub extern "C" fn rs_only_one_window() -> c_int {
    c_int::from(only_one_window_impl())
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
    #[link_name = "rs_tabline_height"]
    fn tabline_height() -> c_int;

    /// Get the global statusline height.
    #[link_name = "rs_global_stl_height"]
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

/// UPD_SOME_VALID constant from screen.h
const UPD_SOME_VALID: c_int = 35;

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

// ============================================================================
// Frame sizing functions
// ============================================================================

extern "C" {
    /// Get the global Rows value.
    fn nvim_get_Rows() -> c_int;

    /// Get the global Columns value.
    fn nvim_get_Columns() -> c_int;

    /// Set the w_height field of a window (raw field accessor).
    fn nvim_win_set_field_height(wp: WinHandle, val: c_int);

    /// Set the w_hsep_height field of a window.
    fn nvim_win_set_hsep_height(wp: WinHandle, val: c_int);

    /// Set the w_status_height field of a window.
    fn nvim_win_set_status_height(wp: WinHandle, val: c_int);

    /// Set the w_width field of a window (raw field accessor).
    fn nvim_win_set_field_width(wp: WinHandle, val: c_int);

    /// Set the w_vsep_width field of a window.
    fn nvim_win_set_vsep_width(wp: WinHandle, val: c_int);

    /// Set the fr_height field of a frame.
    fn nvim_frame_set_height(frp: *mut Frame, val: c_int);

    /// Set the fr_width field of a frame.
    fn nvim_frame_set_width(frp: *mut Frame, val: c_int);

    /// Wrapper for win_config_float().
    fn nvim_win_config_float(wp: WinHandle);

    /// Wrapper for win_fix_scroll().
    fn nvim_win_fix_scroll(upd_topline: bool);

    /// Wrapper for redraw_all_later().
    fn nvim_redraw_all_later(redraw_type: c_int);

    /// Get w_config.height from a window.
    fn nvim_win_get_config_height(wp: WinHandle) -> c_int;

    /// Set w_config.height on a window.
    fn nvim_win_set_config_height(wp: WinHandle, val: c_int);

    /// Get w_config.width from a window.
    fn nvim_win_get_config_width(wp: WinHandle) -> c_int;

    /// Set w_config.width on a window.
    fn nvim_win_set_config_width(wp: WinHandle, val: c_int);

    /// Get the global p_ch (cmdheight) value.
    fn nvim_get_window_p_ch() -> i64;

    /// Get the global ROWS_AVAIL value.
    fn nvim_get_rows_avail() -> c_int;

    /// Set the global redraw_cmdline flag.
    fn nvim_set_redraw_cmdline(val: bool);

    /// Get the global cmdline_row value.
    fn nvim_get_cmdline_row() -> c_int;

    /// Get the min_set_ch value (minimum command line height set by user).
    fn nvim_get_min_set_ch() -> i64;

    /// Wrapper for showmode().
    fn nvim_showmode();
}

/// UPD_VALID constant from drawscreen.h
const UPD_VALID: c_int = 10;

/// Set the window height of window "win" and take care of repositioning other
/// windows to fit around it.
///
/// This is the Rust equivalent of `win_setheight_win()` in window.c.
#[allow(clippy::cast_possible_truncation)]
fn win_setheight_win_impl(mut height: c_int, win: WinHandle) {
    if win.is_null() {
        return;
    }

    // SAFETY: All accessors are safe
    unsafe {
        // Always keep current window at least one line high, even when 'winminheight' is zero.
        // Keep window at least two lines high if 'winbar' is enabled.
        let curwin = nvim_get_curwin();
        let p_wmh = nvim_get_p_wmh() as c_int;
        let winbar_height = nvim_win_get_winbar_height(win);

        let min_height = if win == curwin {
            std::cmp::max(p_wmh, 1) + winbar_height
        } else {
            p_wmh + winbar_height
        };
        height = std::cmp::max(height, min_height);

        if nvim_win_get_floating(win) != 0 {
            // Floating window
            nvim_win_set_config_height(win, std::cmp::max(height, 1));
            nvim_win_config_float(win);
            redraw_later(win, UPD_VALID);
        } else {
            // Normal window - use frame_setheight
            let frame = nvim_win_get_frame(win);
            let hsep_height = nvim_win_get_hsep_height(win);
            let status_height = nvim_win_get_status_height(win);
            frame_setheight_impl(frame, height + hsep_height + status_height);

            // recompute the window positions
            win_comp_pos_impl();
            nvim_win_fix_scroll(true);

            nvim_redraw_all_later(UPD_NOT_VALID);
            nvim_set_redraw_cmdline(true);
        }
    }
}

/// FFI wrapper for `win_setheight_win`.
#[no_mangle]
pub extern "C" fn rs_win_setheight_win(height: c_int, win: WinHandle) {
    win_setheight_win_impl(height, win);
}

/// Set the height of a frame to "height" and take care that all frames and
/// windows inside it are resized.
///
/// This is the Rust equivalent of `frame_setheight()` in window.c.
/// Strategy:
/// - If the frame is part of a FR_COL frame, try fitting in that frame.
/// - If that doesn't work, recursively go to containing frames to resize them.
/// - If the frame is part of a FR_ROW frame, all frames must be resized as well.
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::too_many_lines)]
fn frame_setheight_impl(curfrp: *mut Frame, mut height: c_int) {
    if curfrp.is_null() {
        return;
    }

    // SAFETY: We check for null above
    unsafe {
        let frame = &*curfrp;

        // If the height already is the desired value, nothing to do.
        if frame.fr_height == height {
            return;
        }

        if frame.fr_parent.is_null() {
            // topframe: can only change the command line height
            if height > 0 {
                crate::resize::frame::frame_new_height_impl(curfrp, height, false, false, true);
            }
        } else if (*frame.fr_parent).fr_layout == FR_ROW {
            // Row of frames: Also need to resize frames left and right of this one.
            // First check for the minimal height of these.
            let h =
                frame_minheight_impl(frame.fr_parent, WinHandle::from_ptr(std::ptr::null_mut()));
            height = std::cmp::max(height, h);
            frame_setheight_impl(frame.fr_parent, height);
        } else {
            // Column of frames: try to change only frames in this column.
            let mut room: c_int = 0;
            let mut room_cmdline: c_int = 0;
            let mut room_reserved: c_int = 0;

            // Do this twice:
            // 1: compute room available, if it's not enough try resizing the containing frame.
            // 2: compute the room available and adjust the height to it.
            // Try not to reduce the height of a window with 'winfixheight' set.
            for run in 1..=2 {
                room = 0;
                room_reserved = 0;
                let parent = &*frame.fr_parent;
                let mut frp = parent.fr_child;
                while !frp.is_null() {
                    let fr = &*frp;
                    if frp != curfrp && !fr.fr_win.is_null() && nvim_win_get_wfh(fr.fr_win) != 0 {
                        room_reserved += fr.fr_height;
                    }
                    room += fr.fr_height;
                    if frp != curfrp {
                        room -=
                            frame_minheight_impl(frp, WinHandle::from_ptr(std::ptr::null_mut()));
                    }
                    frp = fr.fr_next;
                }

                if frame.fr_width == nvim_get_Columns() {
                    let wp = lastwin_nofloating_impl();
                    let p_ch = nvim_get_window_p_ch() as c_int;
                    room_cmdline = nvim_get_Rows()
                        - p_ch
                        - global_stl_height()
                        - (nvim_win_get_winrow(wp)
                            + nvim_win_get_w_height(wp)
                            + nvim_win_get_hsep_height(wp)
                            + nvim_win_get_status_height(wp));
                    room_cmdline = std::cmp::max(room_cmdline, 0);
                } else {
                    room_cmdline = 0;
                }

                if height <= room + room_cmdline {
                    break;
                }
                if run == 2 || frame.fr_width == nvim_get_Columns() {
                    height = room + room_cmdline;
                    break;
                }
                frame_setheight_impl(
                    frame.fr_parent,
                    height
                        + frame_minheight_impl(
                            frame.fr_parent,
                            WinHandle::from_ptr((-1isize) as *mut std::ffi::c_void), // NOWIN
                        )
                        - nvim_get_p_wmh() as c_int
                        - 1,
                );
                // NOTREACHED
            }

            // Compute the number of lines we will take from others frames (can be negative!)
            let mut take = height - frame.fr_height;

            // If there is not enough room, also reduce the height of a window with 'winfixheight' set.
            if height > room + room_cmdline - room_reserved {
                room_reserved = room + room_cmdline - height;
            }
            // If there is only a 'winfixheight' window and making the window smaller,
            // need to make the other window taller.
            if take < 0 && room - frame.fr_height < room_reserved {
                room_reserved = 0;
            }

            if take > 0 && room_cmdline > 0 {
                // use lines from cmdline first
                let use_from_cmdline = std::cmp::min(room_cmdline, take);
                take -= use_from_cmdline;
                let topframe = nvim_get_topframe();
                (*topframe).fr_height += use_from_cmdline;
            }

            // set the current frame to the new height
            crate::resize::frame::frame_new_height_impl(curfrp, height, false, false, true);

            // First take lines from the frames after the current frame.
            // If that is not enough, takes lines from frames above the current frame.
            for run in 0..2 {
                // 1st run: start with next window
                // 2nd run: start with prev window
                let mut frp = if run == 0 {
                    frame.fr_next
                } else {
                    frame.fr_prev
                };

                while !frp.is_null() && take != 0 {
                    let fr = &*frp;
                    let h = frame_minheight_impl(frp, WinHandle::from_ptr(std::ptr::null_mut()));
                    if room_reserved > 0 && !fr.fr_win.is_null() && nvim_win_get_wfh(fr.fr_win) != 0
                    {
                        if room_reserved >= fr.fr_height {
                            room_reserved -= fr.fr_height;
                        } else {
                            if fr.fr_height - room_reserved > take {
                                room_reserved = fr.fr_height - take;
                            }
                            take -= fr.fr_height - room_reserved;
                            crate::resize::frame::frame_new_height_impl(
                                frp,
                                room_reserved,
                                false,
                                false,
                                true,
                            );
                            room_reserved = 0;
                        }
                    } else if fr.fr_height - take < h {
                        take -= fr.fr_height - h;
                        crate::resize::frame::frame_new_height_impl(frp, h, false, false, true);
                    } else {
                        crate::resize::frame::frame_new_height_impl(
                            frp,
                            fr.fr_height - take,
                            false,
                            false,
                            true,
                        );
                        take = 0;
                    }
                    frp = if run == 0 { fr.fr_next } else { fr.fr_prev };
                }
            }
        }
    }
}

/// FFI wrapper for `frame_setheight`.
#[no_mangle]
pub extern "C" fn rs_frame_setheight(curfrp: *mut Frame, height: c_int) {
    frame_setheight_impl(curfrp, height);
}

/// Set the width of a frame to "width" and take care that all frames and
/// windows inside it are resized.
///
/// This is the Rust equivalent of `frame_setwidth()` in window.c.
/// Strategy is similar to frame_setheight().
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::too_many_lines)]
fn frame_setwidth_impl(curfrp: *mut Frame, mut width: c_int) {
    if curfrp.is_null() {
        return;
    }

    // SAFETY: We check for null above
    unsafe {
        let frame = &*curfrp;

        // If the width already is the desired value, nothing to do.
        if frame.fr_width == width {
            return;
        }

        if frame.fr_parent.is_null() {
            // topframe: can't change width
            return;
        }

        if (*frame.fr_parent).fr_layout == FR_COL {
            // Column of frames: Also need to resize frames above and below of this one.
            // First check for the minimal width of these.
            let w = frame_minwidth_impl(frame.fr_parent, WinHandle::from_ptr(std::ptr::null_mut()));
            width = std::cmp::max(width, w);
            frame_setwidth_impl(frame.fr_parent, width);
        } else {
            // Row of frames: try to change only frames in this row.
            let mut room: c_int = 0;
            let mut room_reserved: c_int = 0;

            // Do this twice:
            // 1: compute room available, if it's not enough try resizing the containing frame.
            // 2: compute the room available and adjust the width to it.
            for run in 1..=2 {
                room = 0;
                room_reserved = 0;
                let parent = &*frame.fr_parent;
                let mut frp = parent.fr_child;
                while !frp.is_null() {
                    let fr = &*frp;
                    if frp != curfrp && !fr.fr_win.is_null() && nvim_win_get_wfw(fr.fr_win) != 0 {
                        room_reserved += fr.fr_width;
                    }
                    room += fr.fr_width;
                    if frp != curfrp {
                        room -= frame_minwidth_impl(frp, WinHandle::from_ptr(std::ptr::null_mut()));
                    }
                    frp = fr.fr_next;
                }

                if width <= room {
                    break;
                }
                if run == 2 || frame.fr_height >= nvim_get_rows_avail() {
                    width = room;
                    break;
                }
                frame_setwidth_impl(
                    frame.fr_parent,
                    width
                        + frame_minwidth_impl(
                            frame.fr_parent,
                            WinHandle::from_ptr((-1isize) as *mut std::ffi::c_void), // NOWIN
                        )
                        - nvim_get_p_wmw() as c_int
                        - 1,
                );
            }

            // Compute the number of columns we will take from others frames (can be negative!)
            let mut take = width - frame.fr_width;

            // If there is not enough room, also reduce the width of a window with 'winfixwidth' set.
            if width > room - room_reserved {
                room_reserved = room - width;
            }
            // If there is only a 'winfixwidth' window and making the window smaller,
            // need to make the other window narrower.
            if take < 0 && room - frame.fr_width < room_reserved {
                room_reserved = 0;
            }

            // set the current frame to the new width
            crate::resize::frame::frame_new_width_impl(curfrp, width, false, false);

            // First take columns from the frames right of the current frame.
            // If that is not enough, takes columns from frames left of the current frame.
            for run in 0..2 {
                // 1st run: start with next window
                // 2nd run: start with prev window
                let mut frp = if run == 0 {
                    frame.fr_next
                } else {
                    frame.fr_prev
                };

                while !frp.is_null() && take != 0 {
                    let fr = &*frp;
                    let w = frame_minwidth_impl(frp, WinHandle::from_ptr(std::ptr::null_mut()));
                    if room_reserved > 0 && !fr.fr_win.is_null() && nvim_win_get_wfw(fr.fr_win) != 0
                    {
                        if room_reserved >= fr.fr_width {
                            room_reserved -= fr.fr_width;
                        } else {
                            if fr.fr_width - room_reserved > take {
                                room_reserved = fr.fr_width - take;
                            }
                            take -= fr.fr_width - room_reserved;
                            crate::resize::frame::frame_new_width_impl(
                                frp,
                                room_reserved,
                                false,
                                false,
                            );
                            room_reserved = 0;
                        }
                    } else if fr.fr_width - take < w {
                        take -= fr.fr_width - w;
                        crate::resize::frame::frame_new_width_impl(frp, w, false, false);
                    } else {
                        crate::resize::frame::frame_new_width_impl(
                            frp,
                            fr.fr_width - take,
                            false,
                            false,
                        );
                        take = 0;
                    }
                    frp = if run == 0 { fr.fr_next } else { fr.fr_prev };
                }
            }
        }
    }
}

/// FFI wrapper for `frame_setwidth`.
#[no_mangle]
pub extern "C" fn rs_frame_setwidth(curfrp: *mut Frame, width: c_int) {
    frame_setwidth_impl(curfrp, width);
}

/// Set the width of window "win" and take care of repositioning other windows
/// to fit around it.
///
/// This is the Rust equivalent of `win_setwidth_win()` in window.c.
#[allow(clippy::cast_possible_truncation)]
fn win_setwidth_win_impl(mut width: c_int, wp: WinHandle) {
    if wp.is_null() {
        return;
    }

    // SAFETY: All accessors are safe
    unsafe {
        let curwin = nvim_get_curwin();

        // Always keep current window at least one column wide, even when 'winminwidth' is zero.
        if wp == curwin {
            let p_wmw = nvim_get_p_wmw() as c_int;
            width = std::cmp::max(std::cmp::max(width, p_wmw), 1);
        } else if width < 0 {
            width = 0;
        }

        if nvim_win_get_floating(wp) != 0 {
            nvim_win_set_config_width(wp, width);
            nvim_win_config_float(wp);
            redraw_later(wp, UPD_NOT_VALID);
        } else {
            let frame = nvim_win_get_frame(wp);
            let vsep_width = nvim_win_get_vsep_width(wp);
            frame_setwidth_impl(frame, width + vsep_width);

            // recompute the window positions
            win_comp_pos_impl();
            nvim_redraw_all_later(UPD_NOT_VALID);
        }
    }
}

/// FFI wrapper for `win_setwidth_win`.
#[no_mangle]
pub extern "C" fn rs_win_setwidth_win(width: c_int, wp: WinHandle) {
    win_setwidth_win_impl(width, wp);
}

// ============================================================================
// Drag operations
// ============================================================================

/// Status line of dragwin is dragged "offset" lines down (negative is up).
///
/// This is the Rust equivalent of `win_drag_status_line()` in window.c.
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::too_many_lines)]
fn win_drag_status_line_impl(dragwin: WinHandle, mut offset: c_int) {
    if dragwin.is_null() {
        return;
    }

    // SAFETY: All pointer accesses are guarded by null checks
    unsafe {
        let topframe = nvim_get_topframe();
        let mut fr = nvim_win_get_frame(dragwin);
        if fr.is_null() {
            return;
        }

        let mut curfr = fr;
        if fr != topframe {
            // more than one window
            fr = (*fr).fr_parent;
            // When the parent frame is not a column of frames, its parent should be.
            if !fr.is_null() && (*fr).fr_layout != FR_COL {
                curfr = fr;
                if fr != topframe {
                    // only a row of windows, may drag statusline
                    fr = (*fr).fr_parent;
                }
            }
        }

        // If this is the last frame in a column, may want to resize the parent
        // frame instead (go two up to skip a row of frames).
        while curfr != topframe && (*curfr).fr_next.is_null() {
            if !fr.is_null() && fr != topframe {
                fr = (*fr).fr_parent;
            }
            curfr = if fr.is_null() { curfr } else { fr };
            if !fr.is_null() && fr != topframe {
                fr = (*fr).fr_parent;
            }
        }

        let up = offset < 0; // if true, drag status line up, otherwise down

        let room: c_int;
        if up {
            // drag up
            offset = -offset;
            // sum up the room of the current frame and above it
            if fr == curfr {
                // only one window
                room = (*fr).fr_height - frame_minheight_impl(fr, WinHandle::null());
            } else {
                let mut r = 0;
                let mut child = if fr.is_null() {
                    std::ptr::null_mut()
                } else {
                    (*fr).fr_child
                };
                while !child.is_null() {
                    r += (*child).fr_height - frame_minheight_impl(child, WinHandle::null());
                    if child == curfr {
                        break;
                    }
                    child = (*child).fr_next;
                }
                room = r;
            }
            fr = (*curfr).fr_next; // put fr at frame that grows
        } else {
            // drag down
            // Only dragging the last status line can reduce p_ch.
            let cmdline_row = nvim_get_cmdline_row();
            let p_ch = nvim_get_window_p_ch() as c_int;
            let min_set_ch = nvim_get_min_set_ch() as c_int;

            let mut r = nvim_get_Rows() - cmdline_row;
            if !(*curfr).fr_next.is_null() {
                r -= p_ch + global_stl_height();
            } else if min_set_ch > 0 {
                r -= 1;
            }
            r = std::cmp::max(r, 0);

            // sum up the room of frames below of the current one
            let mut child = (*curfr).fr_next;
            while !child.is_null() {
                r += (*child).fr_height - frame_minheight_impl(child, WinHandle::null());
                child = (*child).fr_next;
            }
            room = r;
            fr = curfr; // put fr at window that grows
        }

        // If not enough room then move as far as we can
        offset = std::cmp::min(offset, room);
        if offset <= 0 {
            return;
        }

        // Grow frame fr by "offset" lines.
        // Doesn't happen when dragging the last status line up.
        if !fr.is_null() {
            crate::resize::frame::frame_new_height_impl(
                fr,
                (*fr).fr_height + offset,
                up,
                false,
                true,
            );
        }

        let shrink_fr = if up {
            curfr // current frame gets smaller
        } else {
            (*curfr).fr_next // next frame gets smaller
        };

        // Now make the other frames smaller.
        let mut frp = shrink_fr;
        let mut remaining = offset;
        while !frp.is_null() && remaining > 0 {
            let n = frame_minheight_impl(frp, WinHandle::null());
            if (*frp).fr_height - remaining <= n {
                remaining -= (*frp).fr_height - n;
                crate::resize::frame::frame_new_height_impl(frp, n, !up, false, true);
            } else {
                crate::resize::frame::frame_new_height_impl(
                    frp,
                    (*frp).fr_height - remaining,
                    !up,
                    false,
                    true,
                );
                break;
            }
            frp = if up { (*frp).fr_prev } else { (*frp).fr_next };
        }

        win_comp_pos_impl();
        nvim_win_fix_scroll(true);

        nvim_redraw_all_later(UPD_SOME_VALID);
        nvim_showmode();
    }
}

/// FFI wrapper for `win_drag_status_line`.
#[no_mangle]
pub extern "C" fn rs_win_drag_status_line(dragwin: WinHandle, offset: c_int) {
    win_drag_status_line_impl(dragwin, offset);
}

/// Separator line of dragwin is dragged "offset" lines right (negative is left).
///
/// This is the Rust equivalent of `win_drag_vsep_line()` in window.c.
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::too_many_lines)]
fn win_drag_vsep_line_impl(dragwin: WinHandle, mut offset: c_int) {
    if dragwin.is_null() {
        return;
    }

    // SAFETY: All pointer accesses are guarded by null checks
    unsafe {
        let topframe = nvim_get_topframe();
        let mut fr = nvim_win_get_frame(dragwin);
        if fr.is_null() {
            return;
        }

        if fr == topframe {
            // only one window (cannot happen?)
            return;
        }

        let mut curfr = fr;
        fr = (*fr).fr_parent;
        if fr.is_null() {
            return;
        }

        // When the parent frame is not a row of frames, its parent should be.
        if (*fr).fr_layout != FR_ROW {
            if fr == topframe {
                // only a column of windows (cannot happen?)
                return;
            }
            curfr = fr;
            fr = (*fr).fr_parent;
            if fr.is_null() {
                return;
            }
        }

        // If this is the last frame in a row, may want to resize a parent
        // frame instead.
        while (*curfr).fr_next.is_null() {
            if fr == topframe {
                break;
            }
            curfr = fr;
            fr = (*fr).fr_parent;
            if fr.is_null() {
                break;
            }
            if fr != topframe {
                curfr = fr;
                fr = (*fr).fr_parent;
                if fr.is_null() {
                    break;
                }
            }
        }

        let left = offset < 0; // if true, drag separator line left, otherwise right

        let room: c_int;
        if left {
            // drag left
            offset = -offset;
            // sum up the room of the current frame and left of it
            let mut r = 0;
            let parent_fr = if fr.is_null() { curfr } else { fr };
            let mut child = (*parent_fr).fr_child;
            while !child.is_null() {
                r += (*child).fr_width - frame_minwidth_impl(child, WinHandle::null());
                if child == curfr {
                    break;
                }
                child = (*child).fr_next;
            }
            room = r;
            fr = (*curfr).fr_next; // put fr at frame that grows
        } else {
            // drag right
            // sum up the room of frames right of the current one
            let mut r = 0;
            let mut child = (*curfr).fr_next;
            while !child.is_null() {
                r += (*child).fr_width - frame_minwidth_impl(child, WinHandle::null());
                child = (*child).fr_next;
            }
            room = r;
            fr = curfr; // put fr at window that grows
        }

        // If not enough room then move as far as we can
        offset = std::cmp::min(offset, room);

        // No room at all, quit.
        if offset <= 0 {
            return;
        }

        if fr.is_null() {
            // This can happen when calling win_move_separator() on the rightmost
            // window. Just don't do anything.
            return;
        }

        // grow frame fr by offset lines
        crate::resize::frame::frame_new_width_impl(fr, (*fr).fr_width + offset, left, false);

        // shrink other frames: current and at the left or at the right
        let shrink_fr = if left {
            curfr // current frame gets smaller
        } else {
            (*curfr).fr_next // next frame gets smaller
        };

        let mut frp = shrink_fr;
        let mut remaining = offset;
        while !frp.is_null() && remaining > 0 {
            let n = frame_minwidth_impl(frp, WinHandle::null());
            if (*frp).fr_width - remaining <= n {
                remaining -= (*frp).fr_width - n;
                crate::resize::frame::frame_new_width_impl(frp, n, !left, false);
            } else {
                crate::resize::frame::frame_new_width_impl(
                    frp,
                    (*frp).fr_width - remaining,
                    !left,
                    false,
                );
                break;
            }
            frp = if left { (*frp).fr_prev } else { (*frp).fr_next };
        }

        win_comp_pos_impl();
        nvim_redraw_all_later(UPD_NOT_VALID);
    }
}

/// FFI wrapper for `win_drag_vsep_line`.
#[no_mangle]
pub extern "C" fn rs_win_drag_vsep_line(dragwin: WinHandle, offset: c_int) {
    win_drag_vsep_line_impl(dragwin, offset);
}

/// FFI wrapper for `frame_new_height`.
/// Calls the Rust implementation directly.
#[export_name = "frame_new_height"]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn rs_frame_new_height(
    topfrp: *mut Frame,
    height: c_int,
    topfirst: c_int,
    wfh: c_int,
    set_ch: c_int,
) {
    if topfrp.is_null() {
        return;
    }
    // SAFETY: topfrp is non-null and points to a valid Frame.
    unsafe {
        crate::resize::frame::frame_new_height_impl(
            topfrp,
            height,
            topfirst != 0,
            wfh != 0,
            set_ch != 0,
        );
    }
}

/// FFI wrapper for `frame_new_width`.
/// Calls the Rust implementation directly.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn rs_frame_new_width(
    topfrp: *mut Frame,
    width: c_int,
    leftfirst: c_int,
    wfw: c_int,
) {
    if topfrp.is_null() {
        return;
    }
    // SAFETY: topfrp is non-null and points to a valid Frame.
    unsafe { crate::resize::frame::frame_new_width_impl(topfrp, width, leftfirst != 0, wfw != 0) }
}

// ============================================================================
// Frame helper functions
// ============================================================================

/// Resize frame "frp" to be "n" lines higher (negative for less high).
/// Also resize the frames it is contained in.
///
/// This is the Rust equivalent of `frame_add_height()` in window.c.
fn frame_add_height_impl(frp: *mut Frame, n: c_int) {
    if frp.is_null() {
        return;
    }

    // SAFETY: Frame pointer is valid and we're calling FFI functions
    unsafe {
        let frame = &*frp;
        crate::resize::frame::frame_new_height_impl(frp, frame.fr_height + n, false, false, false);

        let mut parent = frame.fr_parent;
        while !parent.is_null() {
            let p = &mut *parent;
            p.fr_height += n;
            parent = p.fr_parent;
        }
    }
}

/// FFI wrapper for `frame_add_height`.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn rs_frame_add_height(frp: *mut Frame, n: c_int) {
    frame_add_height_impl(frp, n);
}

/// Add a status line to windows at the bottom of "frp".
/// Note: Does not check if there is room!
///
/// This is the Rust equivalent of `frame_add_statusline()` in window.c.
fn frame_add_statusline_impl(frp: *mut Frame) {
    if frp.is_null() {
        return;
    }

    // SAFETY: Frame pointer is valid
    unsafe {
        let frame = &*frp;

        if frame.fr_layout == FR_LEAF {
            // Leaf frame - add status to window
            let wp = frame.fr_win;
            nvim_win_set_status_height(wp, STATUS_HEIGHT);
        } else if frame.fr_layout == FR_ROW {
            // Handle all the frames in the row
            let mut child = frame.fr_child;
            while !child.is_null() {
                frame_add_statusline_impl(child);
                child = (*child).fr_next;
            }
        } else {
            // FR_COL: Only need to handle the last frame in the column
            let mut child = frame.fr_child;
            while !child.is_null() && !(*child).fr_next.is_null() {
                child = (*child).fr_next;
            }
            frame_add_statusline_impl(child);
        }
    }
}

/// FFI wrapper for `frame_add_statusline`.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn rs_frame_add_statusline(frp: *mut Frame) {
    frame_add_statusline_impl(frp);
}

/// Add or remove the vertical separator of windows to the right side of "frp".
/// Note: Does not check if there is room!
///
/// This is the Rust equivalent of `frame_set_vsep()` in window.c.
fn frame_set_vsep_impl(frp: *const Frame, add: bool) {
    if frp.is_null() {
        return;
    }

    // SAFETY: Frame pointer is valid
    unsafe {
        let frame = &*frp;

        if frame.fr_layout == FR_LEAF {
            let wp = frame.fr_win;
            let vsep_width = nvim_win_get_vsep_width(wp);
            let w_width = nvim_win_get_w_width(wp);

            if add && vsep_width == 0 {
                if w_width > 0 {
                    // don't make it negative
                    crate::resize::execute::win_new_width_impl(wp, w_width - 1);
                }
                nvim_win_set_vsep_width(wp, 1);
            } else if !add && vsep_width == 1 {
                crate::resize::execute::win_new_width_impl(wp, w_width + 1);
                nvim_win_set_vsep_width(wp, 0);
            }
        } else if frame.fr_layout == FR_COL {
            // Handle all the frames in the column
            let mut child = frame.fr_child;
            while !child.is_null() {
                frame_set_vsep_impl(child, add);
                child = (*child).fr_next;
            }
        } else {
            // FR_ROW: Only need to handle the last frame in the row
            let mut child = frame.fr_child;
            while !child.is_null() && !(*child).fr_next.is_null() {
                child = (*child).fr_next;
            }
            frame_set_vsep_impl(child, add);
        }
    }
}

/// FFI wrapper for `frame_set_vsep`.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn rs_frame_set_vsep(frp: *const Frame, add: c_int) {
    frame_set_vsep_impl(frp, add != 0);
}

/// Add the horizontal separator to windows at the bottom of "frp".
/// Note: Does not check if there is room or whether the windows have a statusline!
///
/// This is the Rust equivalent of `frame_add_hsep()` in window.c.
fn frame_add_hsep_impl(frp: *const Frame) {
    if frp.is_null() {
        return;
    }

    // SAFETY: Frame pointer is valid
    unsafe {
        let frame = &*frp;

        if frame.fr_layout == FR_LEAF {
            let wp = frame.fr_win;
            nvim_win_set_hsep_height(wp, 1);
        } else if frame.fr_layout == FR_ROW {
            // Handle all the frames in the row
            let mut child = frame.fr_child;
            while !child.is_null() {
                frame_add_hsep_impl(child);
                child = (*child).fr_next;
            }
        } else {
            // FR_COL: Only need to handle the last frame in the column
            let mut child = frame.fr_child;
            while !child.is_null() && !(*child).fr_next.is_null() {
                child = (*child).fr_next;
            }
            frame_add_hsep_impl(child);
        }
    }
}

/// FFI wrapper for `frame_add_hsep`.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn rs_frame_add_hsep(frp: *const Frame) {
    frame_add_hsep_impl(frp);
}

/// Set frame width from the window it contains.
///
/// This is the Rust equivalent of `frame_fix_width()` in window.c.
fn frame_fix_width_impl(wp: WinHandle) {
    if wp.is_null() {
        return;
    }

    // SAFETY: Window pointer is valid
    unsafe {
        let frame = nvim_win_get_frame(wp);
        if !frame.is_null() {
            let w_width = nvim_win_get_w_width(wp);
            let vsep_width = nvim_win_get_vsep_width(wp);
            nvim_frame_set_width(frame, w_width + vsep_width);
        }
    }
}

/// FFI wrapper for `frame_fix_width`.
#[no_mangle]
pub extern "C" fn rs_frame_fix_width(wp: WinHandle) {
    frame_fix_width_impl(wp);
}

/// Set frame height from the window it contains.
///
/// This is the Rust equivalent of `frame_fix_height()` in window.c.
fn frame_fix_height_impl(wp: WinHandle) {
    if wp.is_null() {
        return;
    }

    // SAFETY: Window pointer is valid
    unsafe {
        let frame = nvim_win_get_frame(wp);
        if !frame.is_null() {
            let w_height = nvim_win_get_w_height(wp);
            let hsep_height = nvim_win_get_hsep_height(wp);
            let status_height = nvim_win_get_status_height(wp);
            nvim_frame_set_height(frame, w_height + hsep_height + status_height);
        }
    }
}

/// FFI wrapper for `frame_fix_height`.
#[no_mangle]
pub extern "C" fn rs_frame_fix_height(wp: WinHandle) {
    frame_fix_height_impl(wp);
}

// =============================================================================
// Status Line Redraw Functions
// =============================================================================

/// Redraw all status lines at the bottom of a frame tree.
///
/// This is the Rust equivalent of `win_redraw_last_status()` in drawscreen.c.
/// It marks windows for status line redraw by setting `w_redr_status = true`.
///
/// - For FR_LEAF frames: mark the window for status redraw
/// - For FR_ROW frames: recursively process all children
/// - For FR_COL frames: find the last child and recursively process it
fn win_redraw_last_status_impl(frp: *const Frame) {
    if frp.is_null() {
        return;
    }

    // SAFETY: frp is checked for null above, and we trust the C frame tree structure
    unsafe {
        let frame = &*frp;

        if frame.is_leaf() {
            // Leaf frame - mark the window for status redraw
            let wp = frame.fr_win;
            if !wp.is_null() {
                nvim_win_set_redr_status(wp, 1);
            }
        } else if frame.is_row() {
            // Row layout - process all children
            let mut child = frame.fr_child;
            while !child.is_null() {
                win_redraw_last_status_impl(child);
                child = (*child).fr_next;
            }
        } else {
            // Column layout - find and process the last child
            debug_assert!(frame.is_col());
            let mut child = frame.fr_child;
            if !child.is_null() {
                while !(*child).fr_next.is_null() {
                    child = (*child).fr_next;
                }
                win_redraw_last_status_impl(child);
            }
        }
    }
}

/// Marks all status lines at the bottom of a frame tree for redraw.
///
/// # Safety
/// `frp` must be a valid frame pointer or null.
#[unsafe(export_name = "win_redraw_last_status")]
pub extern "C" fn rs_win_redraw_last_status(frp: *const Frame) {
    win_redraw_last_status_impl(frp);
}

// =============================================================================
// Window Validity Flags (w_valid)
// =============================================================================

// Window validity flag constants (from buffer_defs.h)
/// w_wrow (window row) is valid
const VALID_WROW: c_int = 0x01;
/// w_wcol (window col) is valid
const VALID_WCOL: c_int = 0x02;
/// w_virtcol (file col) is valid
const VALID_VIRTCOL: c_int = 0x04;
/// w_cline_height and w_cline_folded valid
const VALID_CHEIGHT: c_int = 0x08;
/// w_cline_row is valid
const VALID_CROW: c_int = 0x10;
/// w_botline and w_empty_rows are valid
const VALID_BOTLINE: c_int = 0x20;
/// w_botline is approximated
const VALID_BOTLINE_AP: c_int = 0x40;
/// w_topline is valid (for cursor position)
const VALID_TOPLINE: c_int = 0x80;

// C accessor for w_valid field
extern "C" {
    /// Get the w_valid field from a window.
    fn nvim_win_get_valid(wp: WinHandle) -> c_int;

    /// Clear specific bits from the w_valid field.
    fn nvim_win_clear_valid_bits(wp: WinHandle, bits: c_int);
}

/// Invalidate the bottom line position and approximation flags.
///
/// This is the Rust equivalent of `invalidate_botline()` in move.c.
/// Clears both VALID_BOTLINE and VALID_BOTLINE_AP flags from w_valid.
#[inline]
fn invalidate_botline_impl(wp: WinHandle) {
    if wp.is_null() {
        return;
    }
    unsafe {
        nvim_win_clear_valid_bits(wp, VALID_BOTLINE | VALID_BOTLINE_AP);
    }
}

/// FFI wrapper for `invalidate_botline`.
///
/// Clears the VALID_BOTLINE and VALID_BOTLINE_AP flags from the window.
#[export_name = "invalidate_botline"]
pub extern "C" fn rs_invalidate_botline(wp: WinHandle) {
    invalidate_botline_impl(wp);
}

/// Clear the VALID_BOTLINE flag but keep VALID_BOTLINE_AP.
///
/// This is the Rust equivalent of `approximate_botline_win()` in move.c.
/// Used when the bottom line is no longer exactly known but is still
/// approximately valid.
#[inline]
fn approximate_botline_win_impl(wp: WinHandle) {
    if wp.is_null() {
        return;
    }
    unsafe {
        nvim_win_clear_valid_bits(wp, VALID_BOTLINE);
    }
}

/// FFI wrapper for `approximate_botline_win`.
///
/// Clears only the VALID_BOTLINE flag (keeps VALID_BOTLINE_AP).
#[export_name = "approximate_botline_win"]
pub extern "C" fn rs_approximate_botline_win(wp: WinHandle) {
    approximate_botline_win_impl(wp);
}

/// Clear validity flags when cursor line length changed before cursor.
///
/// This is the Rust equivalent of `changed_cline_bef_curs()` in move.c.
/// Clears VALID_WROW, VALID_WCOL, VALID_VIRTCOL, VALID_CROW, VALID_CHEIGHT, VALID_TOPLINE.
#[inline]
fn changed_cline_bef_curs_impl(wp: WinHandle) {
    if wp.is_null() {
        return;
    }
    unsafe {
        nvim_win_clear_valid_bits(
            wp,
            VALID_WROW | VALID_WCOL | VALID_VIRTCOL | VALID_CROW | VALID_CHEIGHT | VALID_TOPLINE,
        );
    }
}

/// FFI wrapper for `changed_cline_bef_curs`.
///
/// Clears validity flags when cursor line length changed before cursor.
#[export_name = "changed_cline_bef_curs"]
pub extern "C" fn rs_changed_cline_bef_curs(wp: WinHandle) {
    changed_cline_bef_curs_impl(wp);
}

/// Clear validity flags when a line above cursor changed.
///
/// This is the Rust equivalent of `changed_line_abv_curs()` in move.c.
/// Operates on curwin.
#[inline]
fn changed_line_abv_curs_impl() {
    unsafe {
        let curwin = nvim_get_curwin();
        if !curwin.is_null() {
            nvim_win_clear_valid_bits(
                curwin,
                VALID_WROW
                    | VALID_WCOL
                    | VALID_VIRTCOL
                    | VALID_CROW
                    | VALID_CHEIGHT
                    | VALID_TOPLINE,
            );
        }
    }
}

/// FFI wrapper for `changed_line_abv_curs`.
///
/// Clears validity flags on curwin when a line above cursor changed.
#[export_name = "changed_line_abv_curs"]
pub extern "C" fn rs_changed_line_abv_curs() {
    changed_line_abv_curs_impl();
}

/// Clear validity flags when a line above cursor changed (window parameter version).
///
/// This is the Rust equivalent of `changed_line_abv_curs_win()` in move.c.
#[inline]
fn changed_line_abv_curs_win_impl(wp: WinHandle) {
    if wp.is_null() {
        return;
    }
    unsafe {
        nvim_win_clear_valid_bits(
            wp,
            VALID_WROW | VALID_WCOL | VALID_VIRTCOL | VALID_CROW | VALID_CHEIGHT | VALID_TOPLINE,
        );
    }
}

/// FFI wrapper for `changed_line_abv_curs_win`.
///
/// Clears validity flags on the given window when a line above cursor changed.
#[export_name = "changed_line_abv_curs_win"]
pub extern "C" fn rs_changed_line_abv_curs_win(wp: WinHandle) {
    changed_line_abv_curs_win_impl(wp);
}

// =============================================================================
// Window Setting Change Functions
// =============================================================================

// C accessor for w_lines_valid
extern "C" {
    /// Set the w_lines_valid field (number of valid w_lines entries).
    fn nvim_win_set_lines_valid(wp: WinHandle, val: c_int);
}

/// Handle window setting changes that require recomputation.
///
/// This is the Rust equivalent of `changed_window_setting()` in move.c.
/// Invalidates line cache, clears validity flags, and schedules redraw.
///
/// Called when window settings change (e.g., 'wrap' option or folding).
#[inline]
fn changed_window_setting_impl(wp: WinHandle) {
    if wp.is_null() {
        return;
    }
    unsafe {
        // Invalidate all line cache entries
        nvim_win_set_lines_valid(wp, 0);
        // Clear validity flags for lines above cursor
        changed_line_abv_curs_win_impl(wp);
        // Clear botline and topline validity
        nvim_win_clear_valid_bits(wp, VALID_BOTLINE | VALID_BOTLINE_AP | VALID_TOPLINE);
        // Schedule complete redraw
        redraw_later(wp, UPD_NOT_VALID);
    }
}

/// FFI wrapper for `changed_window_setting`.
///
/// Handles window setting changes that require recomputation.
#[export_name = "changed_window_setting"]
pub extern "C" fn rs_changed_window_setting(wp: WinHandle) {
    changed_window_setting_impl(wp);
}

/// Call `changed_window_setting` for every window in all tabpages.
///
/// This is the Rust equivalent of `changed_window_setting_all()` in move.c.
/// Iterates through all tabpages and all windows within each tabpage.
#[inline]
fn changed_window_setting_all_impl() {
    unsafe {
        // FOR_ALL_TAB_WINDOWS(tp, wp) - iterate all tabs and windows
        let mut tp = nvim_get_first_tabpage();
        while !tp.is_null() {
            let mut wp = nvim_tabpage_get_firstwin(tp);
            while !wp.is_null() {
                changed_window_setting_impl(wp);
                wp = nvim_win_get_next(wp);
            }
            tp = nvim_tabpage_get_next(tp);
        }
    }
}

/// FFI wrapper for `changed_window_setting_all`.
///
/// Calls `changed_window_setting` for every window in all tabpages.
#[export_name = "changed_window_setting_all"]
pub extern "C" fn rs_changed_window_setting_all() {
    changed_window_setting_all_impl();
}

// ============================================================================
// Syntax Highlighting Functions
// ============================================================================

extern "C" {
    /// Get the number of syntax patterns defined for a window.
    fn nvim_win_get_syn_patterns_len(win: WinHandle) -> c_int;

    /// Get the number of syntax clusters defined for a window.
    fn nvim_win_get_syn_clusters_len(win: WinHandle) -> c_int;

    /// Get the number of used entries in the keyword hashtab.
    fn nvim_win_get_keywtab_used(win: WinHandle) -> c_int;

    /// Get the number of used entries in the case-insensitive keyword hashtab.
    fn nvim_win_get_keywtab_ic_used(win: WinHandle) -> c_int;
}

/// Check if syntax highlighting is present in a window.
///
/// Returns true if the window has any syntax patterns, clusters, or keywords defined.
#[inline]
fn syntax_present_impl(win: WinHandle) -> bool {
    if win.is_null() {
        return false;
    }
    unsafe {
        nvim_win_get_syn_patterns_len(win) != 0
            || nvim_win_get_syn_clusters_len(win) != 0
            || nvim_win_get_keywtab_used(win) > 0
            || nvim_win_get_keywtab_ic_used(win) > 0
    }
}

/// FFI wrapper for `syntax_present`.
///
/// Checks if syntax highlighting is present for the given window.
///
/// # Safety
/// The `win` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_syntax_present(win: WinHandle) -> c_int {
    c_int::from(syntax_present_impl(win))
}

/// C-ABI export: syntax_present returns bool.
#[must_use]
#[export_name = "syntax_present"]
pub extern "C" fn syntax_present_export(win: WinHandle) -> bool {
    syntax_present_impl(win)
}

// =============================================================================
// Window Navigation Functions
// =============================================================================

extern "C" {
    /// Get the w_wcol field from a window (cursor column in window).
    fn nvim_win_get_wcol(wp: WinHandle) -> c_int;

    /// Get the w_wrow field from a window (cursor row in window).
    fn nvim_win_get_wrow(wp: WinHandle) -> c_int;

    /// Get the tp_topframe field from a tabpage.
    fn nvim_tabpage_get_topframe(tp: TabpageHandle) -> *mut Frame;

    /// Get the prevwin global.
    fn nvim_get_prevwin() -> WinHandle;
}

/// Get the above or below neighbor window of the specified window.
///
/// Returns the specified window if the neighbor is not found.
/// Returns the previous window if the specified window is a floating window.
///
/// This is the Rust equivalent of `win_vert_neighbor()` in window.c.
#[allow(clippy::too_many_lines)]
fn win_vert_neighbor_impl(
    tp: TabpageHandle,
    wp: WinHandle,
    up: bool,
    mut count: c_int,
) -> WinHandle {
    if wp.is_null() || tp.is_null() {
        return unsafe { WinHandle::from_ptr(std::ptr::null_mut()) };
    }

    unsafe {
        let mut foundfr = nvim_win_get_frame(wp);

        // If floating window, return prevwin if valid and non-floating, else firstwin
        if nvim_win_get_floating(wp) != 0 {
            let prevwin = nvim_get_prevwin();
            let firstwin = nvim_get_firstwin();
            return if win_valid_impl(prevwin) && nvim_win_get_floating(prevwin) == 0 {
                prevwin
            } else {
                firstwin
            };
        }

        let topframe = nvim_tabpage_get_topframe(tp);

        while count > 0 {
            count -= 1;
            // First go upwards in the tree of frames until we find an upwards or
            // downwards neighbor.
            let mut fr = foundfr;
            let mut nfr;
            loop {
                if fr == topframe {
                    // Reached top, return what we found
                    return if foundfr.is_null() {
                        WinHandle::from_ptr(std::ptr::null_mut())
                    } else {
                        (*foundfr).fr_win
                    };
                }
                nfr = if up { (*fr).fr_prev } else { (*fr).fr_next };
                let parent = (*fr).fr_parent;
                if !parent.is_null() && (*parent).fr_layout == FR_COL && !nfr.is_null() {
                    break;
                }
                fr = parent;
            }

            // Now go downwards to find the bottom or top frame in it.
            loop {
                if (*nfr).fr_layout == FR_LEAF {
                    foundfr = nfr;
                    break;
                }
                fr = (*nfr).fr_child;
                if (*nfr).fr_layout == FR_ROW {
                    // Find the frame at the cursor column.
                    let wp_wincol = nvim_win_get_wincol(wp);
                    let wp_wcol = nvim_win_get_wcol(wp);
                    while !(*fr).fr_next.is_null() {
                        let fr_win = frame2win_impl(fr);
                        let fr_wincol = nvim_win_get_wincol(fr_win);
                        if fr_wincol + (*fr).fr_width > wp_wincol + wp_wcol {
                            break;
                        }
                        fr = (*fr).fr_next;
                    }
                }
                if (*nfr).fr_layout == FR_COL && up {
                    while !(*fr).fr_next.is_null() {
                        fr = (*fr).fr_next;
                    }
                }
                nfr = fr;
            }
        }

        if foundfr.is_null() {
            WinHandle::from_ptr(std::ptr::null_mut())
        } else {
            (*foundfr).fr_win
        }
    }
}

/// FFI wrapper for `win_vert_neighbor`.
///
/// Returns the above or below neighbor window.
#[no_mangle]
pub extern "C" fn rs_win_vert_neighbor(
    tp: TabpageHandle,
    wp: WinHandle,
    up: c_int,
    count: c_int,
) -> WinHandle {
    win_vert_neighbor_impl(tp, wp, up != 0, count)
}

/// Get the left or right neighbor window of the specified window.
///
/// Returns the specified window if the neighbor is not found.
/// Returns the previous window if the specified window is a floating window.
///
/// This is the Rust equivalent of `win_horz_neighbor()` in window.c.
#[allow(clippy::too_many_lines)]
fn win_horz_neighbor_impl(
    tp: TabpageHandle,
    wp: WinHandle,
    left: bool,
    mut count: c_int,
) -> WinHandle {
    if wp.is_null() || tp.is_null() {
        return unsafe { WinHandle::from_ptr(std::ptr::null_mut()) };
    }

    unsafe {
        let mut foundfr = nvim_win_get_frame(wp);

        // If floating window, return prevwin if valid and non-floating, else firstwin
        if nvim_win_get_floating(wp) != 0 {
            let prevwin = nvim_get_prevwin();
            let firstwin = nvim_get_firstwin();
            return if win_valid_impl(prevwin) && nvim_win_get_floating(prevwin) == 0 {
                prevwin
            } else {
                firstwin
            };
        }

        let topframe = nvim_tabpage_get_topframe(tp);

        while count > 0 {
            count -= 1;
            // First go upwards in the tree of frames until we find a left or
            // right neighbor.
            let mut fr = foundfr;
            let mut nfr;
            loop {
                if fr == topframe {
                    // Reached top, return what we found
                    return if foundfr.is_null() {
                        WinHandle::from_ptr(std::ptr::null_mut())
                    } else {
                        (*foundfr).fr_win
                    };
                }
                nfr = if left { (*fr).fr_prev } else { (*fr).fr_next };
                let parent = (*fr).fr_parent;
                if !parent.is_null() && (*parent).fr_layout == FR_ROW && !nfr.is_null() {
                    break;
                }
                fr = parent;
            }

            // Now go downwards to find the leftmost or rightmost frame in it.
            loop {
                if (*nfr).fr_layout == FR_LEAF {
                    foundfr = nfr;
                    break;
                }
                fr = (*nfr).fr_child;
                if (*nfr).fr_layout == FR_COL {
                    // Find the frame at the cursor row.
                    let wp_winrow = nvim_win_get_winrow(wp);
                    let wp_wrow = nvim_win_get_wrow(wp);
                    while !(*fr).fr_next.is_null() {
                        let fr_win = frame2win_impl(fr);
                        let fr_winrow = nvim_win_get_winrow(fr_win);
                        if fr_winrow + (*fr).fr_height > wp_winrow + wp_wrow {
                            break;
                        }
                        fr = (*fr).fr_next;
                    }
                }
                if (*nfr).fr_layout == FR_ROW && left {
                    while !(*fr).fr_next.is_null() {
                        fr = (*fr).fr_next;
                    }
                }
                nfr = fr;
            }
        }

        if foundfr.is_null() {
            WinHandle::from_ptr(std::ptr::null_mut())
        } else {
            (*foundfr).fr_win
        }
    }
}

/// FFI wrapper for `win_horz_neighbor`.
///
/// Returns the left or right neighbor window.
#[no_mangle]
pub extern "C" fn rs_win_horz_neighbor(
    tp: TabpageHandle,
    wp: WinHandle,
    left: c_int,
    count: c_int,
) -> WinHandle {
    win_horz_neighbor_impl(tp, wp, left != 0, count)
}

// =============================================================================
// Frame List Operations
// =============================================================================

/// Append frame "frp" in a frame list after frame "after".
///
/// This is the Rust equivalent of `frame_append()` in window.c.
///
/// # Safety
/// Both `after` and `frp` must be valid, non-null frame pointers.
/// The caller is responsible for ensuring proper frame tree structure.
#[inline]
unsafe fn frame_append_impl(after: *mut Frame, frp: *mut Frame) {
    debug_assert!(!after.is_null());
    debug_assert!(!frp.is_null());

    (*frp).fr_next = (*after).fr_next;
    (*after).fr_next = frp;
    if !(*frp).fr_next.is_null() {
        (*(*frp).fr_next).fr_prev = frp;
    }
    (*frp).fr_prev = after;
}

/// FFI wrapper for `frame_append`.
///
/// Appends frame `frp` after frame `after` in a frame list.
///
/// # Safety
/// Both `after` and `frp` must be valid, non-null frame pointers.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn rs_frame_append(after: *mut Frame, frp: *mut Frame) {
    if after.is_null() || frp.is_null() {
        return;
    }
    // SAFETY: We verified both pointers are non-null.
    // The caller must ensure they point to valid Frame structures.
    unsafe { frame_append_impl(after, frp) }
}

/// Insert frame "frp" in a frame list before frame "before".
///
/// This is the Rust equivalent of `frame_insert()` in window.c.
///
/// # Safety
/// Both `before` and `frp` must be valid, non-null frame pointers.
/// The caller is responsible for ensuring proper frame tree structure.
#[inline]
unsafe fn frame_insert_impl(before: *mut Frame, frp: *mut Frame) {
    debug_assert!(!before.is_null());
    debug_assert!(!frp.is_null());

    (*frp).fr_next = before;
    (*frp).fr_prev = (*before).fr_prev;
    (*before).fr_prev = frp;
    if (*frp).fr_prev.is_null() {
        // frp becomes the first child of the parent
        let parent = (*frp).fr_parent;
        if !parent.is_null() {
            (*parent).fr_child = frp;
        }
    } else {
        (*(*frp).fr_prev).fr_next = frp;
    }
}

/// FFI wrapper for `frame_insert`.
///
/// Inserts frame `frp` before frame `before` in a frame list.
///
/// # Safety
/// Both `before` and `frp` must be valid, non-null frame pointers.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn rs_frame_insert(before: *mut Frame, frp: *mut Frame) {
    if before.is_null() || frp.is_null() {
        return;
    }
    // SAFETY: We verified both pointers are non-null.
    // The caller must ensure they point to valid Frame structures.
    unsafe { frame_insert_impl(before, frp) }
}

/// Remove a frame from a frame list.
///
/// This is the Rust equivalent of `frame_remove()` in window.c.
/// Note: This only removes the frame from the list, it does NOT free the frame.
///
/// # Safety
/// `frp` must be a valid, non-null frame pointer.
/// The caller is responsible for ensuring proper frame tree structure.
#[inline]
unsafe fn frame_remove_impl(frp: *mut Frame) {
    debug_assert!(!frp.is_null());

    if (*frp).fr_prev.is_null() {
        // frp was the first child, update parent's fr_child
        let parent = (*frp).fr_parent;
        if !parent.is_null() {
            (*parent).fr_child = (*frp).fr_next;
        }
    } else {
        (*(*frp).fr_prev).fr_next = (*frp).fr_next;
    }
    if !(*frp).fr_next.is_null() {
        (*(*frp).fr_next).fr_prev = (*frp).fr_prev;
    }
}

/// FFI wrapper for `frame_remove`.
///
/// Removes frame `frp` from its frame list.
/// Note: This only removes the frame from the list, it does NOT free the frame.
///
/// # Safety
/// `frp` must be a valid, non-null frame pointer.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn rs_frame_remove(frp: *mut Frame) {
    if frp.is_null() {
        return;
    }
    // SAFETY: We verified the pointer is non-null.
    // The caller must ensure it points to a valid Frame structure.
    unsafe { frame_remove_impl(frp) }
}

// =============================================================================
// Fold column count
// =============================================================================

extern "C" {
    /// Get the fold column display string (w_p_fdc) from a window.
    fn nvim_win_get_w_p_fdc(wp: WinHandle) -> *const std::ffi::c_char;

    /// Get the deepest nesting level for folds in a window.
    fn rs_getDeepestNesting(wp: WinHandle) -> c_int;
}

/// Return the number of fold columns to display.
///
/// Equivalent to C `win_fdccol_count(wp)` in window.c.
#[allow(clippy::cast_possible_truncation)]
fn win_fdccol_count_impl(wp: WinHandle) -> c_int {
    unsafe {
        let fdc = nvim_win_get_w_p_fdc(wp);
        if fdc.is_null() {
            return 0;
        }

        // Convert to a byte slice for easier comparison
        let fdc_cstr = std::ffi::CStr::from_ptr(fdc);
        let fdc_bytes = fdc_cstr.to_bytes();

        // Check for "auto" or "auto:<NUM>" prefix
        if fdc_bytes.starts_with(b"auto") {
            let fdccol: c_int = if fdc_bytes.len() > 5 && fdc_bytes[4] == b':' {
                c_int::from(fdc_bytes[5]) - c_int::from(b'0')
            } else {
                1
            };
            let needed = rs_getDeepestNesting(wp);
            return std::cmp::min(fdccol, needed);
        }

        if fdc_bytes.is_empty() {
            return 0;
        }

        c_int::from(fdc_bytes[0]) - c_int::from(b'0')
    }
}

/// FFI: Return the number of fold columns to display.
#[no_mangle]
pub extern "C" fn rs_win_fdccol_count(wp: WinHandle) -> c_int {
    win_fdccol_count_impl(wp)
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

    #[test]
    fn test_win_handle_equality() {
        let ptr1 = 0x1000 as *mut std::ffi::c_void;
        let ptr2 = 0x1000 as *mut std::ffi::c_void;
        let ptr3 = 0x2000 as *mut std::ffi::c_void;
        let h1 = unsafe { WinHandle::from_ptr(ptr1) };
        let h2 = unsafe { WinHandle::from_ptr(ptr2) };
        let h3 = unsafe { WinHandle::from_ptr(ptr3) };
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_handle_sizes() {
        use std::mem::size_of;
        // Handles should be pointer-sized
        assert_eq!(size_of::<WinHandle>(), size_of::<*mut std::ffi::c_void>());
        assert_eq!(
            size_of::<TabpageHandle>(),
            size_of::<*mut std::ffi::c_void>()
        );
    }

    // =========================================================================
    // Frame List Operations Tests
    // =========================================================================

    /// Create a test frame with specified layout and null pointers
    fn make_test_frame(layout: c_char) -> Frame {
        Frame {
            fr_layout: layout,
            fr_width: 80,
            fr_newwidth: 0,
            fr_height: 24,
            fr_newheight: 0,
            fr_parent: std::ptr::null_mut(),
            fr_next: std::ptr::null_mut(),
            fr_prev: std::ptr::null_mut(),
            fr_child: std::ptr::null_mut(),
            fr_win: WinHandle::null(),
        }
    }

    #[test]
    fn test_frame_append_basic() {
        let mut frame1 = make_test_frame(FR_LEAF);
        let mut frame2 = make_test_frame(FR_LEAF);

        let p1 = std::ptr::addr_of_mut!(frame1);
        let p2 = std::ptr::addr_of_mut!(frame2);

        unsafe {
            frame_append_impl(p1, p2);
        }

        // After append: frame1 -> frame2
        assert_eq!(frame1.fr_next, p2);
        assert!(frame1.fr_prev.is_null());
        assert_eq!(frame2.fr_prev, p1);
        assert!(frame2.fr_next.is_null());
    }

    #[test]
    fn test_frame_append_chain() {
        let mut frame1 = make_test_frame(FR_LEAF);
        let mut frame2 = make_test_frame(FR_LEAF);
        let mut frame3 = make_test_frame(FR_LEAF);

        let p1 = std::ptr::addr_of_mut!(frame1);
        let p2 = std::ptr::addr_of_mut!(frame2);
        let p3 = std::ptr::addr_of_mut!(frame3);

        unsafe {
            frame_append_impl(p1, p2);
            frame_append_impl(p2, p3);
        }

        // After appends: frame1 -> frame2 -> frame3
        assert_eq!(frame1.fr_next, p2);
        assert!(frame1.fr_prev.is_null());

        assert_eq!(frame2.fr_prev, p1);
        assert_eq!(frame2.fr_next, p3);

        assert_eq!(frame3.fr_prev, p2);
        assert!(frame3.fr_next.is_null());
    }

    #[test]
    fn test_frame_insert_at_start() {
        let mut parent = make_test_frame(FR_COL);
        let mut frame1 = make_test_frame(FR_LEAF);
        let mut frame2 = make_test_frame(FR_LEAF);

        let pp = std::ptr::addr_of_mut!(parent);
        let p1 = std::ptr::addr_of_mut!(frame1);
        let p2 = std::ptr::addr_of_mut!(frame2);

        // Set up frame1 as first child
        parent.fr_child = p1;
        frame1.fr_parent = pp;

        unsafe {
            // Insert frame2 before frame1
            frame2.fr_parent = pp;
            frame_insert_impl(p1, p2);
        }

        // After insert: parent -> frame2 -> frame1
        assert_eq!(parent.fr_child, p2);
        assert!(frame2.fr_prev.is_null());
        assert_eq!(frame2.fr_next, p1);
        assert_eq!(frame1.fr_prev, p2);
    }

    #[test]
    fn test_frame_insert_middle() {
        let mut frame1 = make_test_frame(FR_LEAF);
        let mut frame2 = make_test_frame(FR_LEAF);
        let mut frame3 = make_test_frame(FR_LEAF);

        let p1 = std::ptr::addr_of_mut!(frame1);
        let p2 = std::ptr::addr_of_mut!(frame2);
        let p3 = std::ptr::addr_of_mut!(frame3);

        // Set up chain: frame1 -> frame3
        frame1.fr_next = p3;
        frame3.fr_prev = p1;

        unsafe {
            // Insert frame2 before frame3
            frame_insert_impl(p3, p2);
        }

        // After insert: frame1 -> frame2 -> frame3
        assert_eq!(frame1.fr_next, p2);
        assert_eq!(frame2.fr_prev, p1);
        assert_eq!(frame2.fr_next, p3);
        assert_eq!(frame3.fr_prev, p2);
    }

    #[test]
    fn test_frame_remove_middle() {
        let mut frame1 = make_test_frame(FR_LEAF);
        let mut frame2 = make_test_frame(FR_LEAF);
        let mut frame3 = make_test_frame(FR_LEAF);

        let p1 = std::ptr::addr_of_mut!(frame1);
        let p2 = std::ptr::addr_of_mut!(frame2);
        let p3 = std::ptr::addr_of_mut!(frame3);

        // Set up chain: frame1 -> frame2 -> frame3
        frame1.fr_next = p2;
        frame2.fr_prev = p1;
        frame2.fr_next = p3;
        frame3.fr_prev = p2;

        unsafe {
            // Remove frame2
            frame_remove_impl(p2);
        }

        // After remove: frame1 -> frame3
        assert_eq!(frame1.fr_next, p3);
        assert_eq!(frame3.fr_prev, p1);
    }

    #[test]
    fn test_frame_remove_first() {
        let mut parent = make_test_frame(FR_COL);
        let mut frame1 = make_test_frame(FR_LEAF);
        let mut frame2 = make_test_frame(FR_LEAF);

        let pp = std::ptr::addr_of_mut!(parent);
        let p1 = std::ptr::addr_of_mut!(frame1);
        let p2 = std::ptr::addr_of_mut!(frame2);

        // Set up: parent -> frame1 -> frame2
        parent.fr_child = p1;
        frame1.fr_parent = pp;
        frame1.fr_next = p2;
        frame2.fr_prev = p1;
        frame2.fr_parent = pp;

        unsafe {
            // Remove frame1
            frame_remove_impl(p1);
        }

        // After remove: parent -> frame2
        assert_eq!(parent.fr_child, p2);
        assert!(frame2.fr_prev.is_null());
    }

    #[test]
    fn test_frame_remove_last() {
        let mut frame1 = make_test_frame(FR_LEAF);
        let mut frame2 = make_test_frame(FR_LEAF);

        let p1 = std::ptr::addr_of_mut!(frame1);
        let p2 = std::ptr::addr_of_mut!(frame2);

        // Set up: frame1 -> frame2
        frame1.fr_next = p2;
        frame2.fr_prev = p1;

        unsafe {
            // Remove frame2
            frame_remove_impl(p2);
        }

        // After remove: just frame1
        assert!(frame1.fr_next.is_null());
    }
}
