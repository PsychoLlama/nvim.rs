//! Screen Update Orchestration for Neovim
//!
//! This crate provides Rust implementations of screen update functions
//! from `src/nvim/drawscreen.c`. It handles:
//!
//! - **Separator Drawing**: Window separators (horizontal and vertical)
//! - **Redraw Management**: Tracking which windows need redrawing
//! - **Scroll Optimization**: Efficient scrolling within windows
//! - **Screen Invalidation**: Marking regions as needing updates
//!
//! # FFI Exports
//!
//! The crate exports 43 functions prefixed with `rs_` for C interop:
//! - Window redraw state (`rs_win_redr_*`)
//! - Scroll region calculation (`rs_calc_scroll_region`)
//! - Global screen state (`rs_must_redraw`, `rs_set_must_redraw`)

#![allow(unsafe_code)]
#![allow(dead_code)] // Some extern functions are pre-declared for future use
#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_safety_doc)] // FFI functions follow standard C calling conventions

use std::ffi::{c_int, c_void};

use nvim_window::{rs_frame2win, Frame, WinHandle, FR_COL, FR_LEAF, FR_ROW};

/// Opaque handle to C's buf_T.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BufHandle(*mut c_void);

impl BufHandle {
    /// Check if the handle is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// schar_T is stored as a u32 in Rust (matches grid crate).
type ScharT = u32;

/// Opaque GridView handle.
type GridViewHandle = *mut c_void;

/// Window corner enumeration (matching C WindowCorner enum).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowCorner {
    TopLeft = 0,
    TopRight = 1,
    BottomLeft = 2,
    BottomRight = 3,
}

/// Highlight group for WinSeparator (HLF_C in C).
pub const HLF_C: c_int = 21;

// C accessor functions for window fields
extern "C" {
    fn nvim_win_get_winrow(wp: WinHandle) -> c_int;
    fn nvim_win_get_wincol(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_width(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_hsep_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_vsep_width(wp: WinHandle) -> c_int;
    fn nvim_win_get_endrow(wp: WinHandle) -> c_int;
    fn nvim_win_get_endcol(wp: WinHandle) -> c_int;
    fn nvim_win_get_frame(wp: WinHandle) -> *mut Frame;
    fn nvim_win_get_fcs_vert(wp: WinHandle) -> ScharT;
    fn nvim_win_get_fcs_horiz(wp: WinHandle) -> ScharT;
    fn nvim_win_get_fcs_vertleft(wp: WinHandle) -> ScharT;
    fn nvim_win_get_fcs_vertright(wp: WinHandle) -> ScharT;
    fn nvim_win_get_fcs_verthoriz(wp: WinHandle) -> ScharT;
    fn nvim_win_get_fcs_horizup(wp: WinHandle) -> ScharT;
    fn nvim_win_get_fcs_horizdown(wp: WinHandle) -> ScharT;
    fn nvim_win_hl_attr(wp: WinHandle, hlf: c_int) -> c_int;

    // Window iteration accessors
    fn nvim_get_firstwin() -> WinHandle;
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;
    fn nvim_win_get_status_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_winbar_height(wp: WinHandle) -> c_int;
    fn nvim_win_set_redr_status(wp: WinHandle, val: c_int);
    fn nvim_win_get_redr_status(wp: WinHandle) -> c_int;
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;
    fn redraw_later(wp: WinHandle, redraw_type: c_int);

    // Buffer accessors
    fn nvim_get_curbuf() -> BufHandle;

    // Global functions
    fn global_stl_height() -> c_int;
    fn nvim_get_p_ru() -> c_int;
    fn nvim_set_redraw_cmdline(val: bool);
    fn nvim_get_default_gridview() -> GridViewHandle;

    // Grid functions (already in Rust, called via FFI)
    fn rs_grid_line_start(view: GridViewHandle, row: c_int);
    fn rs_grid_line_put_schar(col: c_int, schar: ScharT, attr: c_int);
    fn rs_grid_line_fill(start_col: c_int, end_col: c_int, sc: ScharT, attr: c_int) -> c_int;
    fn rs_grid_line_flush();
}

/// Check if horizontal separator of window at specified corner is connected
/// to the horizontal separator of another window.
///
/// This is the Rust equivalent of `hsep_connected()` in drawscreen.c.
/// Assumes global statusline is enabled.
fn hsep_connected_impl(wp: WinHandle, corner: WindowCorner) -> bool {
    if wp.is_null() {
        return false;
    }

    let before = matches!(corner, WindowCorner::TopLeft | WindowCorner::BottomLeft);
    let sep_row = unsafe {
        if matches!(corner, WindowCorner::TopLeft | WindowCorner::TopRight) {
            nvim_win_get_winrow(wp) - 1
        } else {
            nvim_win_get_endrow(wp)
        }
    };

    // Get the window's frame
    let mut fr = unsafe { nvim_win_get_frame(wp) };
    if fr.is_null() {
        return false;
    }

    // Walk up the frame tree looking for a row layout with a sibling
    unsafe {
        while !(*fr).fr_parent.is_null() {
            let parent = (*fr).fr_parent;
            if (*parent).fr_layout == FR_ROW {
                let sibling = if before { (*fr).fr_prev } else { (*fr).fr_next };
                if !sibling.is_null() {
                    fr = sibling;
                    break;
                }
            }
            fr = parent;
        }

        if (*fr).fr_parent.is_null() {
            return false;
        }

        // Walk down to find the leaf frame
        while (*fr).fr_layout != FR_LEAF {
            fr = (*fr).fr_child;
            if (*(*fr).fr_parent).fr_layout == FR_ROW && before {
                // Go to the last sibling
                while !(*fr).fr_next.is_null() {
                    fr = (*fr).fr_next;
                }
            } else {
                // Go to the sibling that contains sep_row
                // Use frame2win to get the window from the frame (handles non-leaf frames)
                while !(*fr).fr_next.is_null() {
                    let win = rs_frame2win(fr);
                    let win_row = nvim_win_get_winrow(win);
                    let fr_height = (*fr).fr_height;
                    if win_row + fr_height >= sep_row {
                        break;
                    }
                    fr = (*fr).fr_next;
                }
            }
        }

        let leaf_win = (*fr).fr_win;
        let win_row = nvim_win_get_winrow(leaf_win);
        let end_row = nvim_win_get_endrow(leaf_win);
        sep_row == win_row - 1 || sep_row == end_row
    }
}

/// FFI wrapper for `hsep_connected`.
#[no_mangle]
pub extern "C" fn rs_hsep_connected(wp: WinHandle, corner: WindowCorner) -> c_int {
    c_int::from(hsep_connected_impl(wp, corner))
}

/// Check if vertical separator of window at specified corner is connected
/// to the vertical separator of another window.
///
/// This is the Rust equivalent of `vsep_connected()` in drawscreen.c.
fn vsep_connected_impl(wp: WinHandle, corner: WindowCorner) -> bool {
    if wp.is_null() {
        return false;
    }

    let before = matches!(corner, WindowCorner::TopLeft | WindowCorner::TopRight);
    let sep_col = unsafe {
        if matches!(corner, WindowCorner::TopLeft | WindowCorner::BottomLeft) {
            nvim_win_get_wincol(wp) - 1
        } else {
            nvim_win_get_endcol(wp)
        }
    };

    // Get the window's frame
    let mut fr = unsafe { nvim_win_get_frame(wp) };
    if fr.is_null() {
        return false;
    }

    // Walk up the frame tree looking for a column layout with a sibling
    unsafe {
        while !(*fr).fr_parent.is_null() {
            let parent = (*fr).fr_parent;
            if (*parent).fr_layout == FR_COL {
                let sibling = if before { (*fr).fr_prev } else { (*fr).fr_next };
                if !sibling.is_null() {
                    fr = sibling;
                    break;
                }
            }
            fr = parent;
        }

        if (*fr).fr_parent.is_null() {
            return false;
        }

        // Walk down to find the leaf frame
        while (*fr).fr_layout != FR_LEAF {
            fr = (*fr).fr_child;
            if (*(*fr).fr_parent).fr_layout == FR_COL && before {
                // Go to the last sibling
                while !(*fr).fr_next.is_null() {
                    fr = (*fr).fr_next;
                }
            } else {
                // Go to the sibling that contains sep_col
                // Use frame2win to get the window from the frame (handles non-leaf frames)
                while !(*fr).fr_next.is_null() {
                    let win = rs_frame2win(fr);
                    let win_col = nvim_win_get_wincol(win);
                    let fr_width = (*fr).fr_width;
                    if win_col + fr_width >= sep_col {
                        break;
                    }
                    fr = (*fr).fr_next;
                }
            }
        }

        let leaf_win = (*fr).fr_win;
        let win_col = nvim_win_get_wincol(leaf_win);
        let end_col = nvim_win_get_endcol(leaf_win);
        sep_col == win_col - 1 || sep_col == end_col
    }
}

/// FFI wrapper for `vsep_connected`.
#[no_mangle]
pub extern "C" fn rs_vsep_connected(wp: WinHandle, corner: WindowCorner) -> c_int {
    c_int::from(vsep_connected_impl(wp, corner))
}

/// Draw the vertical separator right of window "wp".
///
/// This is the Rust equivalent of `draw_vsep_win()` in drawscreen.c.
fn draw_vsep_win_impl(wp: WinHandle) {
    if wp.is_null() {
        return;
    }

    unsafe {
        if nvim_win_get_vsep_width(wp) == 0 {
            return;
        }

        let winrow = nvim_win_get_winrow(wp);
        let endrow = nvim_win_get_endrow(wp);
        let endcol = nvim_win_get_endcol(wp);
        let vert_char = nvim_win_get_fcs_vert(wp);
        let hl_attr = nvim_win_hl_attr(wp, HLF_C);
        let gridview = nvim_get_default_gridview();

        // Draw the vertical separator right of this window
        for row in winrow..endrow {
            rs_grid_line_start(gridview, row);
            rs_grid_line_put_schar(endcol, vert_char, hl_attr);
            rs_grid_line_flush();
        }
    }
}

/// FFI wrapper for `draw_vsep_win`.
#[no_mangle]
pub extern "C" fn rs_draw_vsep_win(wp: WinHandle) {
    draw_vsep_win_impl(wp);
}

/// Draw the horizontal separator below window "wp".
///
/// This is the Rust equivalent of `draw_hsep_win()` in drawscreen.c.
fn draw_hsep_win_impl(wp: WinHandle) {
    if wp.is_null() {
        return;
    }

    unsafe {
        if nvim_win_get_hsep_height(wp) == 0 {
            return;
        }

        let wincol = nvim_win_get_wincol(wp);
        let endrow = nvim_win_get_endrow(wp);
        let endcol = nvim_win_get_endcol(wp);
        let horiz_char = nvim_win_get_fcs_horiz(wp);
        let hl_attr = nvim_win_hl_attr(wp, HLF_C);
        let gridview = nvim_get_default_gridview();

        // Draw the horizontal separator below this window
        rs_grid_line_start(gridview, endrow);
        rs_grid_line_fill(wincol, endcol, horiz_char, hl_attr);
        rs_grid_line_flush();
    }
}

/// FFI wrapper for `draw_hsep_win`.
#[no_mangle]
pub extern "C" fn rs_draw_hsep_win(wp: WinHandle) {
    draw_hsep_win_impl(wp);
}

/// Get the separator connector character for specified window corner.
///
/// This is the Rust equivalent of `get_corner_sep_connector()` in drawscreen.c.
fn get_corner_sep_connector_impl(wp: WinHandle, corner: WindowCorner) -> ScharT {
    unsafe {
        // It's impossible for windows to be connected neither vertically nor horizontally
        // So if they're not vertically connected, assume they're horizontally connected
        if vsep_connected_impl(wp, corner) {
            if hsep_connected_impl(wp, corner) {
                nvim_win_get_fcs_verthoriz(wp)
            } else if matches!(corner, WindowCorner::TopLeft | WindowCorner::BottomLeft) {
                nvim_win_get_fcs_vertright(wp)
            } else {
                nvim_win_get_fcs_vertleft(wp)
            }
        } else if matches!(corner, WindowCorner::TopLeft | WindowCorner::TopRight) {
            nvim_win_get_fcs_horizdown(wp)
        } else {
            nvim_win_get_fcs_horizup(wp)
        }
    }
}

/// FFI wrapper for `get_corner_sep_connector`.
#[no_mangle]
pub extern "C" fn rs_get_corner_sep_connector(wp: WinHandle, corner: WindowCorner) -> ScharT {
    get_corner_sep_connector_impl(wp, corner)
}

/// Draw separator connecting characters on the corners of window "wp".
///
/// This is the Rust equivalent of `draw_sep_connectors_win()` in drawscreen.c.
fn draw_sep_connectors_win_impl(wp: WinHandle) {
    if wp.is_null() {
        return;
    }

    unsafe {
        // Don't draw separator connectors unless global statusline is enabled and the window has
        // either a horizontal or vertical separator
        if global_stl_height() == 0 {
            return;
        }

        let hsep_height = nvim_win_get_hsep_height(wp);
        let vsep_width = nvim_win_get_vsep_width(wp);

        if !(hsep_height == 1 || vsep_width == 1) {
            return;
        }

        let hl = nvim_win_hl_attr(wp, HLF_C);

        // Determine which edges of the screen the window is located on
        let win_at_bottom = hsep_height == 0;
        let win_at_right = vsep_width == 0;

        // Check if window is at top edge
        let mut fr = nvim_win_get_frame(wp);
        while !(*fr).fr_parent.is_null() {
            let parent = (*fr).fr_parent;
            if (*parent).fr_layout == FR_COL && !(*fr).fr_prev.is_null() {
                break;
            }
            fr = parent;
        }
        let win_at_top = (*fr).fr_parent.is_null();

        // Check if window is at left edge
        fr = nvim_win_get_frame(wp);
        while !(*fr).fr_parent.is_null() {
            let parent = (*fr).fr_parent;
            if (*parent).fr_layout == FR_ROW && !(*fr).fr_prev.is_null() {
                break;
            }
            fr = parent;
        }
        let win_at_left = (*fr).fr_parent.is_null();

        // Draw the appropriate separator connector in every corner where drawing is necessary
        let top_left = !(win_at_top || win_at_left);
        let top_right = !(win_at_top || win_at_right);
        let bot_left = !(win_at_bottom || win_at_left);
        let bot_right = !(win_at_bottom || win_at_right);

        let winrow = nvim_win_get_winrow(wp);
        let wincol = nvim_win_get_wincol(wp);
        let endrow = nvim_win_get_endrow(wp);
        let endcol = nvim_win_get_endcol(wp);
        let gridview = nvim_get_default_gridview();

        if top_left {
            rs_grid_line_start(gridview, winrow - 1);
            rs_grid_line_put_schar(
                wincol - 1,
                get_corner_sep_connector_impl(wp, WindowCorner::TopLeft),
                hl,
            );
            rs_grid_line_flush();
        }
        if top_right {
            rs_grid_line_start(gridview, winrow - 1);
            rs_grid_line_put_schar(
                endcol,
                get_corner_sep_connector_impl(wp, WindowCorner::TopRight),
                hl,
            );
            rs_grid_line_flush();
        }
        if bot_left {
            rs_grid_line_start(gridview, endrow);
            rs_grid_line_put_schar(
                wincol - 1,
                get_corner_sep_connector_impl(wp, WindowCorner::BottomLeft),
                hl,
            );
            rs_grid_line_flush();
        }
        if bot_right {
            rs_grid_line_start(gridview, endrow);
            rs_grid_line_put_schar(
                endcol,
                get_corner_sep_connector_impl(wp, WindowCorner::BottomRight),
                hl,
            );
            rs_grid_line_flush();
        }
    }
}

/// FFI wrapper for `draw_sep_connectors_win`.
#[no_mangle]
pub extern "C" fn rs_draw_sep_connectors_win(wp: WinHandle) {
    draw_sep_connectors_win_impl(wp);
}

// =============================================================================
// Status Line Redraw Functions
// =============================================================================

/// Mark all status lines and window bars for redraw.
///
/// This is the Rust equivalent of `status_redraw_all()` in drawscreen.c.
/// Used after first :cd or when global statusline configuration changes.
///
/// Iterates through all windows in the current tab and marks them for
/// status line redraw if:
/// - The window has a local statusline (!is_stl_global && has status height), OR
/// - The window is the current window, OR
/// - The window has a winbar
fn status_redraw_all_impl() {
    unsafe {
        let is_stl_global = global_stl_height() != 0;
        let curwin = nvim_get_curwin();

        // FOR_ALL_WINDOWS_IN_TAB(wp, curtab) - iterate windows in current tab
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            let status_h = nvim_win_get_status_height(wp);
            let winbar_h = nvim_win_get_winbar_height(wp);

            // Mark for redraw if:
            // 1. Local statusline (not global) and window has status height, OR
            // 2. This is the current window (for global statusline), OR
            // 3. Window has a winbar
            if (!is_stl_global && status_h > 0) || wp == curwin || winbar_h > 0 {
                nvim_win_set_redr_status(wp, 1);
                redraw_later(wp, UPD_VALID);
            }
            wp = nvim_win_get_next(wp);
        }
    }
}

/// FFI wrapper for `status_redraw_all`.
///
/// Marks all status lines and window bars in the current tab for redraw.
#[no_mangle]
pub extern "C" fn rs_status_redraw_all() {
    status_redraw_all_impl();
}

/// Mark status lines and window bars for a specific buffer.
///
/// This is the Rust equivalent of `status_redraw_buf()` in drawscreen.c.
/// Marks windows that display the given buffer for status line redraw.
///
/// Also handles ruler redraw if:
/// - The ruler option is enabled
/// - Current window has no status height
/// - Current window wasn't already marked for redraw
fn status_redraw_buf_impl(buf: BufHandle) {
    unsafe {
        let is_stl_global = global_stl_height() != 0;
        let curwin = nvim_get_curwin();

        // FOR_ALL_WINDOWS_IN_TAB(wp, curtab)
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            let win_buf = nvim_win_get_buffer(wp);

            // Only process windows showing this buffer
            if win_buf == buf {
                let status_h = nvim_win_get_status_height(wp);
                let winbar_h = nvim_win_get_winbar_height(wp);

                // Mark for redraw if:
                // 1. Local statusline (not global) and window has status height, OR
                // 2. Global statusline and this is the current window, OR
                // 3. Window has a winbar
                if (!is_stl_global && status_h > 0)
                    || (is_stl_global && wp == curwin)
                    || winbar_h > 0
                {
                    nvim_win_set_redr_status(wp, 1);
                    redraw_later(wp, UPD_VALID);
                }
            }
            wp = nvim_win_get_next(wp);
        }

        // Redraw the ruler if it is in the command line and was not marked for redraw above
        let curwin_status_h = nvim_win_get_status_height(curwin);
        let curwin_redr_status = nvim_win_get_redr_status(curwin);
        if nvim_get_p_ru() != 0 && curwin_status_h == 0 && curwin_redr_status == 0 {
            nvim_set_redraw_cmdline(true);
            redraw_later(curwin, UPD_VALID);
        }
    }
}

/// FFI wrapper for `status_redraw_buf`.
///
/// Marks status lines and window bars of the given buffer for redraw.
#[no_mangle]
pub extern "C" fn rs_status_redraw_buf(buf: BufHandle) {
    status_redraw_buf_impl(buf);
}

/// Mark status lines and window bars for the current buffer.
///
/// This is the Rust equivalent of `status_redraw_curbuf()` in drawscreen.c.
/// Simply calls `status_redraw_buf` with the current buffer.
fn status_redraw_curbuf_impl() {
    unsafe {
        let curbuf = nvim_get_curbuf();
        status_redraw_buf_impl(curbuf);
    }
}

/// FFI wrapper for `status_redraw_curbuf`.
///
/// Marks status lines and window bars of the current buffer for redraw.
#[no_mangle]
pub extern "C" fn rs_status_redraw_curbuf() {
    status_redraw_curbuf_impl();
}

// =============================================================================
// Phase 6: Screen Update Logic - Update Strategies and Redraw Management
// =============================================================================

/// Line number type (matches C's linenr_T which is typically int32_t).
type LinenrT = i32;

// Redraw type constants from drawscreen.h
// The higher the value, the higher the priority.
pub const UPD_VALID: c_int = 10;
pub const UPD_INVERTED: c_int = 20;
pub const UPD_INVERTED_ALL: c_int = 25;
pub const UPD_REDRAW_TOP: c_int = 30;
pub const UPD_SOME_VALID: c_int = 35;
pub const UPD_NOT_VALID: c_int = 40;
pub const UPD_CLEAR: c_int = 50;

// Additional C accessor functions for window redraw state
extern "C" {
    fn nvim_win_get_redr_type(wp: WinHandle) -> c_int;
    fn nvim_win_set_redr_type(wp: WinHandle, val: c_int);
    fn nvim_win_get_lines_valid(wp: WinHandle) -> c_int;
    fn nvim_win_set_lines_valid(wp: WinHandle, val: c_int);
    fn nvim_win_get_redraw_top(wp: WinHandle) -> LinenrT;
    fn nvim_win_set_redraw_top(wp: WinHandle, val: LinenrT);
    fn nvim_win_get_redraw_bot(wp: WinHandle) -> LinenrT;
    fn nvim_win_set_redraw_bot(wp: WinHandle, val: LinenrT);
    fn nvim_win_get_topline(wp: WinHandle) -> LinenrT;
    fn nvim_win_get_botline(wp: WinHandle) -> LinenrT;
    fn nvim_get_must_redraw() -> c_int;
    fn nvim_set_must_redraw(val: c_int);
    fn nvim_get_redraw_not_allowed() -> c_int;
    fn nvim_redraw_later(wp: WinHandle, redraw_type: c_int);
}

/// Get the redraw type for a window.
#[no_mangle]
pub unsafe extern "C" fn rs_win_get_redr_type(wp: WinHandle) -> c_int {
    nvim_win_get_redr_type(wp)
}

/// Set the redraw type for a window.
#[no_mangle]
pub unsafe extern "C" fn rs_win_set_redr_type(wp: WinHandle, val: c_int) {
    nvim_win_set_redr_type(wp, val);
}

/// Get the number of valid lines in a window.
#[no_mangle]
pub unsafe extern "C" fn rs_win_get_lines_valid(wp: WinHandle) -> c_int {
    nvim_win_get_lines_valid(wp)
}

/// Set the number of valid lines in a window.
#[no_mangle]
pub unsafe extern "C" fn rs_win_set_lines_valid(wp: WinHandle, val: c_int) {
    nvim_win_set_lines_valid(wp, val);
}

/// Get the top line for redraw range.
#[no_mangle]
pub unsafe extern "C" fn rs_win_get_redraw_top(wp: WinHandle) -> LinenrT {
    nvim_win_get_redraw_top(wp)
}

/// Set the top line for redraw range.
#[no_mangle]
pub unsafe extern "C" fn rs_win_set_redraw_top(wp: WinHandle, val: LinenrT) {
    nvim_win_set_redraw_top(wp, val);
}

/// Get the bottom line for redraw range.
#[no_mangle]
pub unsafe extern "C" fn rs_win_get_redraw_bot(wp: WinHandle) -> LinenrT {
    nvim_win_get_redraw_bot(wp)
}

/// Set the bottom line for redraw range.
#[no_mangle]
pub unsafe extern "C" fn rs_win_set_redraw_bot(wp: WinHandle, val: LinenrT) {
    nvim_win_set_redraw_bot(wp, val);
}

/// Get the top line of the window viewport.
#[no_mangle]
pub unsafe extern "C" fn rs_win_get_topline(wp: WinHandle) -> LinenrT {
    nvim_win_get_topline(wp)
}

/// Get the bottom line of the window viewport.
#[no_mangle]
pub unsafe extern "C" fn rs_win_get_botline(wp: WinHandle) -> LinenrT {
    nvim_win_get_botline(wp)
}

/// Get the global must_redraw flag.
#[no_mangle]
pub extern "C" fn rs_get_must_redraw() -> c_int {
    unsafe { nvim_get_must_redraw() }
}

/// Set the global must_redraw flag.
#[no_mangle]
pub extern "C" fn rs_set_must_redraw(val: c_int) {
    unsafe { nvim_set_must_redraw(val) };
}

/// Check if redraw is currently allowed.
#[no_mangle]
pub extern "C" fn rs_redraw_allowed() -> c_int {
    unsafe { c_int::from(nvim_get_redraw_not_allowed() == 0) }
}

/// Set the must_redraw global only if type is higher.
#[no_mangle]
pub extern "C" fn rs_set_must_redraw_max(redraw_type: c_int) {
    unsafe {
        if nvim_get_redraw_not_allowed() == 0 {
            let current = nvim_get_must_redraw();
            if redraw_type > current {
                nvim_set_must_redraw(redraw_type);
            }
        }
    }
}

/// Check if a redraw type indicates the window needs full redraw.
#[no_mangle]
pub extern "C" fn rs_redraw_type_is_full(redraw_type: c_int) -> c_int {
    c_int::from(redraw_type >= UPD_NOT_VALID)
}

/// Check if a redraw type indicates clear and redraw.
#[no_mangle]
pub extern "C" fn rs_redraw_type_is_clear(redraw_type: c_int) -> c_int {
    c_int::from(redraw_type >= UPD_CLEAR)
}

/// Check if a line is within the window's visible range.
#[no_mangle]
pub unsafe extern "C" fn rs_line_in_window(wp: WinHandle, lnum: LinenrT) -> c_int {
    let topline = nvim_win_get_topline(wp);
    let botline = nvim_win_get_botline(wp);
    c_int::from(lnum >= topline && lnum < botline)
}

/// Check if a line range overlaps with the window's visible range.
#[no_mangle]
pub unsafe extern "C" fn rs_range_in_window(wp: WinHandle, first: LinenrT, last: LinenrT) -> c_int {
    let topline = nvim_win_get_topline(wp);
    let botline = nvim_win_get_botline(wp);
    c_int::from(last >= topline && first < botline)
}

/// Update the window's redraw range to include a specific line.
///
/// Similar to redrawWinline but just updates the range.
#[no_mangle]
pub unsafe extern "C" fn rs_update_redraw_line(wp: WinHandle, lnum: LinenrT) {
    rs_update_redraw_range(wp, lnum, lnum);
}

/// Update the window's redraw range to include a range of lines.
///
/// Only updates if the range is visible in the window.
#[no_mangle]
pub unsafe extern "C" fn rs_update_redraw_range(wp: WinHandle, first: LinenrT, last: LinenrT) {
    let topline = nvim_win_get_topline(wp);
    let botline = nvim_win_get_botline(wp);

    // Only update if range is visible
    if last >= topline && first < botline {
        let current_top = nvim_win_get_redraw_top(wp);
        let current_bot = nvim_win_get_redraw_bot(wp);

        // Update top of redraw range
        if current_top == 0 || first < current_top {
            nvim_win_set_redraw_top(wp, first);
        }

        // Update bottom of redraw range
        if current_bot == 0 || last > current_bot {
            nvim_win_set_redraw_bot(wp, last);
        }

        // Mark window for redraw
        nvim_redraw_later(wp, UPD_VALID);
    }
}

/// Reset the window's redraw range.
#[no_mangle]
pub unsafe extern "C" fn rs_reset_redraw_range(wp: WinHandle) {
    nvim_win_set_redraw_top(wp, 0);
    nvim_win_set_redraw_bot(wp, 0);
}

/// Check if window has a pending redraw range.
#[no_mangle]
pub unsafe extern "C" fn rs_has_redraw_range(wp: WinHandle) -> c_int {
    let top = nvim_win_get_redraw_top(wp);
    let bot = nvim_win_get_redraw_bot(wp);
    c_int::from(top != 0 || bot != 0)
}

/// Get the effective redraw range (clamped to visible area).
///
/// Returns the start line via out_first and end line via out_last.
/// Returns 1 if there is a valid range, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_get_effective_redraw_range(
    wp: WinHandle,
    out_first: *mut LinenrT,
    out_last: *mut LinenrT,
) -> c_int {
    let top = nvim_win_get_redraw_top(wp);
    let bot = nvim_win_get_redraw_bot(wp);
    let topline = nvim_win_get_topline(wp);
    let botline = nvim_win_get_botline(wp);

    if top == 0 && bot == 0 {
        return 0;
    }

    // Clamp to visible range
    let first = if top == 0 { topline } else { top.max(topline) };
    let last = if bot == 0 {
        botline - 1
    } else {
        bot.min(botline - 1)
    };

    if first <= last {
        if !out_first.is_null() {
            *out_first = first;
        }
        if !out_last.is_null() {
            *out_last = last;
        }
        1
    } else {
        0
    }
}

/// Calculate the number of lines in the redraw range.
#[no_mangle]
pub unsafe extern "C" fn rs_redraw_range_lines(wp: WinHandle) -> c_int {
    let mut first: LinenrT = 0;
    let mut last: LinenrT = 0;

    if rs_get_effective_redraw_range(wp, &raw mut first, &raw mut last) != 0 {
        (last - first + 1) as c_int
    } else {
        0
    }
}

/// Invalidate the window's line validity.
///
/// This is called when something changes that requires redrawing lines
/// even if they were previously marked as valid.
#[no_mangle]
pub unsafe extern "C" fn rs_invalidate_lines_valid(wp: WinHandle) {
    nvim_win_set_lines_valid(wp, 0);
}

/// Check if window needs any redrawing.
#[no_mangle]
pub unsafe extern "C" fn rs_win_needs_redraw(wp: WinHandle) -> c_int {
    let redr_type = nvim_win_get_redr_type(wp);
    c_int::from(redr_type != 0)
}

/// Check if window needs full redraw (not just partial).
#[no_mangle]
pub unsafe extern "C" fn rs_win_needs_full_redraw(wp: WinHandle) -> c_int {
    let redr_type = nvim_win_get_redr_type(wp);
    c_int::from(redr_type >= UPD_NOT_VALID)
}

/// Compare two redraw types and return the higher one.
#[no_mangle]
pub extern "C" fn rs_max_redraw_type(type1: c_int, type2: c_int) -> c_int {
    type1.max(type2)
}

/// Check if redraw type1 subsumes type2.
///
/// Returns true if satisfying type1 would also satisfy type2.
#[no_mangle]
pub extern "C" fn rs_redraw_type_subsumes(type1: c_int, type2: c_int) -> c_int {
    c_int::from(type1 >= type2)
}

// =============================================================================
// Scroll Region Helpers
// =============================================================================

/// Calculate the number of lines that can be scrolled.
///
/// Returns the number of valid lines that can be reused after scrolling.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // const extern "C" fn is unstable
pub extern "C" fn rs_scroll_reusable_lines(
    _wp: WinHandle,
    old_topline: LinenrT,
    new_topline: LinenrT,
    win_height: c_int,
) -> c_int {
    let scroll_amount = new_topline - old_topline;

    if scroll_amount == 0 {
        // No scroll
        return win_height;
    }

    let abs_scroll = scroll_amount.abs() as c_int;

    if abs_scroll >= win_height {
        // Scrolled more than window height - no reusable lines
        0
    } else {
        // Can reuse (win_height - abs_scroll) lines
        win_height - abs_scroll
    }
}

/// Check if a scroll operation would be beneficial.
///
/// Returns true if scrolling would preserve more lines than it invalidates.
#[no_mangle]
pub extern "C" fn rs_scroll_is_beneficial(scroll_amount: c_int, win_height: c_int) -> c_int {
    // Scrolling is beneficial if we preserve more than half the lines
    let abs_scroll = scroll_amount.abs();
    c_int::from(abs_scroll > 0 && abs_scroll < win_height / 2)
}

/// Calculate source row for a scroll copy operation.
///
/// Given a destination row and scroll delta, returns the source row.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // const extern "C" fn is unstable
pub extern "C" fn rs_scroll_source_row(dest_row: c_int, scroll_delta: c_int) -> c_int {
    dest_row + scroll_delta
}

/// Check if a row is valid for scroll copy from source.
#[no_mangle]
pub extern "C" fn rs_scroll_row_valid(
    source_row: c_int,
    first_row: c_int,
    last_row: c_int,
) -> c_int {
    c_int::from(source_row >= first_row && source_row < last_row)
}

/// Calculate the scroll region bounds.
///
/// Returns start_row and end_row for the scroll region.
///
/// # Safety
/// Caller must ensure out_start and out_end are valid pointers (or null).
#[no_mangle]
pub unsafe extern "C" fn rs_calc_scroll_region(
    win_row: c_int,
    win_height: c_int,
    scroll_delta: c_int,
    out_start: *mut c_int,
    out_end: *mut c_int,
) {
    if scroll_delta > 0 {
        // Scrolling up - copy from below
        if !out_start.is_null() {
            *out_start = win_row + scroll_delta;
        }
        if !out_end.is_null() {
            *out_end = win_row + win_height;
        }
    } else {
        // Scrolling down - copy from above
        if !out_start.is_null() {
            *out_start = win_row;
        }
        if !out_end.is_null() {
            *out_end = win_row + win_height + scroll_delta;
        }
    }
}

// =============================================================================
// Phase D1: Redraw Management Functions
// =============================================================================

// Additional C accessor functions for window redraw management
extern "C" {
    // Window w_redr_type accessor
    fn nvim_win_get_w_redr_type(wp: WinHandle) -> c_int;
    fn nvim_win_set_w_redr_type(wp: WinHandle, val: c_int);

    // Window w_lines_valid accessor
    fn nvim_win_get_w_lines_valid(wp: WinHandle) -> c_int;
    fn nvim_win_set_w_lines_valid(wp: WinHandle, val: c_int);

    // Grid invalidation
    fn nvim_win_grid_alloc_valid(wp: WinHandle) -> c_int;
    fn nvim_win_grid_alloc_set_valid(wp: WinHandle, val: c_int);

    // Buffer comparison
    fn nvim_win_buffer_eq(wp: WinHandle, buf: BufHandle) -> c_int;

    // Floating window check
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;
}

/// Mark a window for later redraw with the specified type.
///
/// This is the Rust equivalent of `redraw_later()` in drawscreen.c.
/// Sets `wp->w_redr_type` to `type` if current value is lower.
/// Also updates `must_redraw` global to ensure screen update happens.
///
/// # Arguments
/// * `wp` - Window to mark for redraw
/// * `redraw_type` - The redraw type (UPD_VALID, UPD_NOT_VALID, etc.)
fn redraw_later_impl(wp: WinHandle, redraw_type: c_int) {
    if wp.is_null() {
        return;
    }

    unsafe {
        // Check if redraw is allowed and type is higher than current
        if nvim_get_redraw_not_allowed() != 0 {
            return;
        }

        let current_type = nvim_win_get_w_redr_type(wp);
        if current_type >= redraw_type {
            return;
        }

        // Set the new redraw type
        nvim_win_set_w_redr_type(wp, redraw_type);

        // If type >= UPD_NOT_VALID, invalidate line cache
        if redraw_type >= UPD_NOT_VALID {
            nvim_win_set_w_lines_valid(wp, 0);
        }

        // Update must_redraw global
        let must_redraw = nvim_get_must_redraw();
        if redraw_type > must_redraw {
            nvim_set_must_redraw(redraw_type);
        }
    }
}

/// FFI wrapper for `redraw_later`.
///
/// Marks a window for later redraw with the specified type.
#[no_mangle]
pub extern "C" fn rs_redraw_later(wp: WinHandle, redraw_type: c_int) {
    redraw_later_impl(wp, redraw_type);
}

/// Mark all windows in the current tab for later redraw.
///
/// This is the Rust equivalent of `redraw_all_later()` in drawscreen.c.
fn redraw_all_later_impl(redraw_type: c_int) {
    unsafe {
        // Iterate all windows in current tab
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            redraw_later_impl(wp, redraw_type);
            wp = nvim_win_get_next(wp);
        }

        // Also update must_redraw directly
        if nvim_get_redraw_not_allowed() == 0 {
            let must_redraw = nvim_get_must_redraw();
            if redraw_type > must_redraw {
                nvim_set_must_redraw(redraw_type);
            }
        }
    }
}

/// FFI wrapper for `redraw_all_later`.
///
/// Marks all windows in the current tab for later redraw.
#[no_mangle]
pub extern "C" fn rs_redraw_all_later(redraw_type: c_int) {
    redraw_all_later_impl(redraw_type);
}

/// Mark all windows showing a specific buffer for redraw.
///
/// This is the Rust equivalent of `redraw_buf_later()` in drawscreen.c.
fn redraw_buf_later_impl(buf: BufHandle, redraw_type: c_int) {
    unsafe {
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            if nvim_win_buffer_eq(wp, buf) != 0 {
                redraw_later_impl(wp, redraw_type);
            }
            wp = nvim_win_get_next(wp);
        }
    }
}

/// FFI wrapper for `redraw_buf_later`.
///
/// Marks all windows displaying the given buffer for redraw.
#[no_mangle]
pub extern "C" fn rs_redraw_buf_later(buf: BufHandle, redraw_type: c_int) {
    redraw_buf_later_impl(buf, redraw_type);
}

/// Mark all windows showing the current buffer for redraw.
///
/// This is the Rust equivalent of `redraw_curbuf_later()` in drawscreen.c.
fn redraw_curbuf_later_impl(redraw_type: c_int) {
    unsafe {
        let curbuf = nvim_get_curbuf();
        redraw_buf_later_impl(curbuf, redraw_type);
    }
}

/// FFI wrapper for `redraw_curbuf_later`.
///
/// Marks all windows displaying the current buffer for redraw.
#[no_mangle]
pub extern "C" fn rs_redraw_curbuf_later(redraw_type: c_int) {
    redraw_curbuf_later_impl(redraw_type);
}

/// Invalidate highlights for all windows in the current tab.
///
/// This is the Rust equivalent of `screen_invalidate_highlights()` in drawscreen.c.
fn screen_invalidate_highlights_impl() {
    unsafe {
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            redraw_later_impl(wp, UPD_NOT_VALID);
            nvim_win_grid_alloc_set_valid(wp, 0);
            wp = nvim_win_get_next(wp);
        }
    }
}

/// FFI wrapper for `screen_invalidate_highlights`.
///
/// Invalidates highlights for all windows, forcing full redraw.
#[no_mangle]
pub extern "C" fn rs_screen_invalidate_highlights() {
    screen_invalidate_highlights_impl();
}

/// Mark a line range in a window for redraw.
///
/// This is the Rust equivalent of `redraw_win_range_later()` in drawscreen.c.
/// Only updates if the range is visible in the window.
fn redraw_win_range_later_impl(wp: WinHandle, first: LinenrT, last: LinenrT) {
    unsafe {
        let topline = nvim_win_get_topline(wp);
        let botline = nvim_win_get_botline(wp);

        // Only update if range is visible
        if last >= topline && first < botline {
            let current_top = nvim_win_get_redraw_top(wp);
            let current_bot = nvim_win_get_redraw_bot(wp);

            // Update top of redraw range
            if current_top == 0 || first < current_top {
                nvim_win_set_redraw_top(wp, first);
            }

            // Update bottom of redraw range
            if current_bot == 0 || last > current_bot {
                nvim_win_set_redraw_bot(wp, last);
            }

            redraw_later_impl(wp, UPD_VALID);
        }
    }
}

/// FFI wrapper for `redraw_win_range_later`.
///
/// Marks a range of lines in a window for redraw.
#[no_mangle]
pub extern "C" fn rs_redraw_win_range_later(wp: WinHandle, first: LinenrT, last: LinenrT) {
    redraw_win_range_later_impl(wp, first, last);
}

/// Mark a single line in a window for redraw.
///
/// This is the Rust equivalent of `redrawWinline()` in drawscreen.c.
#[no_mangle]
pub extern "C" fn rs_redrawWinline(wp: WinHandle, lnum: LinenrT) {
    redraw_win_range_later_impl(wp, lnum, lnum);
}

/// Mark a line range in all windows showing a buffer.
///
/// This is the Rust equivalent of `redraw_buf_range_later()` in drawscreen.c.
fn redraw_buf_range_later_impl(buf: BufHandle, first: LinenrT, last: LinenrT) {
    unsafe {
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            if nvim_win_buffer_eq(wp, buf) != 0 {
                redraw_win_range_later_impl(wp, first, last);
            }
            wp = nvim_win_get_next(wp);
        }
    }
}

/// FFI wrapper for `redraw_buf_range_later`.
///
/// Marks a range of lines in all windows displaying the buffer.
#[no_mangle]
pub extern "C" fn rs_redraw_buf_range_later(buf: BufHandle, first: LinenrT, last: LinenrT) {
    redraw_buf_range_later_impl(buf, first, last);
}

/// Check if window needs status line redraw.
///
/// Helper for redraw_buf_status_later.
fn win_needs_status_redraw(wp: WinHandle) -> bool {
    unsafe {
        let status_h = nvim_win_get_status_height(wp);
        let winbar_h = nvim_win_get_winbar_height(wp);
        let is_stl_global = global_stl_height() != 0;
        let is_curwin = wp == nvim_get_curwin();

        status_h > 0 || (is_curwin && is_stl_global) || winbar_h > 0
    }
}

/// Mark status lines for buffer for redraw.
///
/// This is the Rust equivalent of `redraw_buf_status_later()` in drawscreen.c.
fn redraw_buf_status_later_impl(buf: BufHandle) {
    unsafe {
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            if nvim_win_buffer_eq(wp, buf) != 0 && win_needs_status_redraw(wp) {
                nvim_win_set_redr_status(wp, 1);

                // Set must_redraw to at least UPD_VALID
                if nvim_get_redraw_not_allowed() == 0 {
                    let must_redraw = nvim_get_must_redraw();
                    if UPD_VALID > must_redraw {
                        nvim_set_must_redraw(UPD_VALID);
                    }
                }
            }
            wp = nvim_win_get_next(wp);
        }
    }
}

/// FFI wrapper for `redraw_buf_status_later`.
///
/// Marks status lines of windows displaying the buffer for redraw.
#[no_mangle]
pub extern "C" fn rs_redraw_buf_status_later(buf: BufHandle) {
    redraw_buf_status_later_impl(buf);
}

// =============================================================================
// Phase D1: Window Update State Management
// =============================================================================

/// Window update state tracking structure.
///
/// This struct tracks the state needed during a win_update() call.
/// It corresponds to the local variables in win_update() in drawscreen.c.
#[repr(C)]
pub struct WinUpdateState {
    /// First row below top area that needs updating
    pub top_end: c_int,
    /// First row of mid area that needs updating
    pub mid_start: c_int,
    /// First row below mid area that needs updating
    pub mid_end: c_int,
    /// First row of bot area that needs updating
    pub bot_start: c_int,
    /// First row that needs redraw due to scrolling
    pub bot_scroll_start: c_int,
    /// True when scrolled down when w_topline got smaller
    pub scrolled_down: c_int,
    /// Redraw above mod_top
    pub top_to_mod: c_int,
}

impl Default for WinUpdateState {
    fn default() -> Self {
        Self {
            top_end: 0,
            mid_start: 999,
            mid_end: 0,
            bot_start: 999,
            bot_scroll_start: 999,
            scrolled_down: 0,
            top_to_mod: 0,
        }
    }
}

/// Initialize window update state.
///
/// Returns the initial state for a win_update() call.
#[no_mangle]
pub extern "C" fn rs_win_update_state_init() -> WinUpdateState {
    WinUpdateState::default()
}

/// Check if window should skip update (zero-height or zero-width).
///
/// Returns 1 if window should skip update, 0 otherwise.
/// Also draws separators for zero-dimension windows.
#[no_mangle]
pub extern "C" fn rs_win_should_skip_update(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 1;
    }

    unsafe {
        let view_height = nvim_win_get_view_height(wp);
        let view_width = nvim_win_get_view_width(wp);

        if view_height == 0 {
            // Draw separators and return skip
            rs_draw_hsep_win(wp);
            rs_draw_sep_connectors_win(wp);
            nvim_win_set_w_redr_type(wp, 0);
            return 1;
        }

        if view_width == 0 {
            // Draw separators and return skip
            rs_draw_vsep_win(wp);
            rs_draw_sep_connectors_win(wp);
            nvim_win_set_w_redr_type(wp, 0);
            return 1;
        }

        0
    }
}

// Additional accessor for view dimensions
extern "C" {
    fn nvim_win_get_view_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_view_width(wp: WinHandle) -> c_int;
}

/// Reset window redraw type after update.
#[no_mangle]
pub extern "C" fn rs_win_update_reset_redr_type(wp: WinHandle) {
    if !wp.is_null() {
        unsafe {
            nvim_win_set_w_redr_type(wp, 0);
        }
    }
}

/// Check if redraw type requires full window redraw.
#[no_mangle]
pub extern "C" fn rs_win_needs_full_update(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let redr_type = nvim_win_get_w_redr_type(wp);
        c_int::from(redr_type >= UPD_NOT_VALID)
    }
}

/// Get the effective redraw type for a window, clamped to valid range.
#[no_mangle]
pub extern "C" fn rs_win_get_effective_redr_type(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe { nvim_win_get_w_redr_type(wp).clamp(0, UPD_CLEAR) }
}

// =============================================================================
// Phase D1.4: Screen Update Loop Helpers
// =============================================================================

// Additional C function declarations for screen update
extern "C" {
    // Global flags
    fn nvim_get_updating_screen() -> c_int;
    fn nvim_set_updating_screen(val: c_int);
    fn nvim_get_redrawing_disabled() -> c_int;

    // Window clear state
    fn nvim_win_get_redr_border(wp: WinHandle) -> c_int;
    fn nvim_win_set_redr_border(wp: WinHandle, val: c_int);

    // Buffer mod state
    fn nvim_buf_get_mod_set(buf: BufHandle) -> c_int;
    fn nvim_buf_set_mod_set(buf: BufHandle, val: c_int);
}

/// Check if screen updating is currently in progress.
///
/// Returns 1 if updating_screen is set, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_is_updating_screen() -> c_int {
    unsafe { nvim_get_updating_screen() }
}

/// Set the updating_screen flag.
#[no_mangle]
pub extern "C" fn rs_set_updating_screen(val: c_int) {
    unsafe {
        nvim_set_updating_screen(val);
    }
}

/// Check if redrawing is currently disabled.
///
/// Returns 1 if RedrawingDisabled is set, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_is_redrawing_disabled() -> c_int {
    unsafe { nvim_get_redrawing_disabled() }
}

/// Check if a window needs its border redrawn.
#[no_mangle]
pub extern "C" fn rs_win_needs_border_redraw(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let redr_border = nvim_win_get_redr_border(wp);
        let redr_type = nvim_win_get_w_redr_type(wp);
        c_int::from(redr_border != 0 || redr_type >= UPD_NOT_VALID)
    }
}

/// Reset window's border redraw flag.
#[no_mangle]
pub extern "C" fn rs_win_reset_redr_border(wp: WinHandle) {
    if !wp.is_null() {
        unsafe {
            nvim_win_set_redr_border(wp, 0);
        }
    }
}

/// Reset buffer's mod_set flag for a window.
#[no_mangle]
pub extern "C" fn rs_win_reset_buf_mod_set(wp: WinHandle) {
    if !wp.is_null() {
        unsafe {
            let buf = nvim_win_get_buffer(wp);
            if !buf.is_null() {
                nvim_buf_set_mod_set(buf, 0);
            }
        }
    }
}

/// Check if buffer has modifications that need to be displayed.
#[no_mangle]
pub extern "C" fn rs_win_buf_has_mod_set(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let buf = nvim_win_get_buffer(wp);
        if buf.is_null() {
            return 0;
        }
        nvim_buf_get_mod_set(buf)
    }
}

/// Update state for a window after win_update().
///
/// This resets relevant state that should be cleared after a window update.
#[no_mangle]
pub extern "C" fn rs_win_post_update(wp: WinHandle) {
    if wp.is_null() {
        return;
    }

    unsafe {
        // Reset redraw type
        nvim_win_set_w_redr_type(wp, 0);
        // Reset border redraw flag
        nvim_win_set_redr_border(wp, 0);
    }
}

// =============================================================================
// Phase D1.4: Visual Mode Update Helpers
// =============================================================================

// Additional C function declarations for visual mode
extern "C" {
    fn nvim_get_visual_active() -> c_int;
    fn nvim_get_visual_mode() -> c_int;
    fn nvim_win_get_old_visual_mode(wp: WinHandle) -> c_int;
    fn nvim_win_set_old_visual_mode(wp: WinHandle, val: c_int);
    fn nvim_win_get_old_cursor_lnum(wp: WinHandle) -> LinenrT;
    fn nvim_win_set_old_cursor_lnum(wp: WinHandle, val: LinenrT);
    fn nvim_win_get_old_visual_lnum(wp: WinHandle) -> LinenrT;
    fn nvim_win_set_old_visual_lnum(wp: WinHandle, val: LinenrT);
    fn nvim_win_get_old_visual_col(wp: WinHandle) -> ColnrT;
    fn nvim_win_set_old_visual_col(wp: WinHandle, val: ColnrT);
}

/// Column number type (matches `colnr_T` in Neovim).
type ColnrT = i32;

/// Check if visual selection changed and needs redraw.
///
/// Returns 1 if visual mode state changed in a way that requires redraw.
#[no_mangle]
pub extern "C" fn rs_visual_selection_changed(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let visual_active = nvim_get_visual_active() != 0;
        let old_visual_mode = nvim_win_get_old_visual_mode(wp);

        if !visual_active && old_visual_mode == 0 {
            return 0;
        }

        if visual_active {
            let current_mode = nvim_get_visual_mode();
            if current_mode != old_visual_mode {
                return 1;
            }
        }

        1
    }
}

/// Update visual mode tracking state after window update.
#[no_mangle]
pub extern "C" fn rs_update_visual_state(
    wp: WinHandle,
    cursor_lnum: LinenrT,
    visual_lnum: LinenrT,
    visual_col: ColnrT,
) {
    if wp.is_null() {
        return;
    }

    unsafe {
        let visual_active = nvim_get_visual_active() != 0;

        if visual_active {
            let visual_mode = nvim_get_visual_mode();
            nvim_win_set_old_visual_mode(wp, visual_mode);
            nvim_win_set_old_cursor_lnum(wp, cursor_lnum);
            nvim_win_set_old_visual_lnum(wp, visual_lnum);
            nvim_win_set_old_visual_col(wp, visual_col);
        } else {
            nvim_win_set_old_visual_mode(wp, 0);
            nvim_win_set_old_cursor_lnum(wp, 0);
            nvim_win_set_old_visual_lnum(wp, 0);
            nvim_win_set_old_visual_col(wp, 0);
        }
    }
}

/// Clear visual mode state for a window.
#[no_mangle]
pub extern "C" fn rs_clear_visual_state(wp: WinHandle) {
    if wp.is_null() {
        return;
    }

    unsafe {
        nvim_win_set_old_visual_mode(wp, 0);
        nvim_win_set_old_cursor_lnum(wp, 0);
        nvim_win_set_old_visual_lnum(wp, 0);
        nvim_win_set_old_visual_col(wp, 0);
    }
}

// =============================================================================
// Phase D4: Enhanced Scroll Optimization
// =============================================================================

/// Calculate the optimal scroll amount for smooth scrolling.
///
/// Given the current topline, target topline, and window height,
/// returns the recommended scroll step for smooth scrolling.
#[no_mangle]
pub extern "C" fn rs_smooth_scroll_step(
    current_topline: LinenrT,
    target_topline: LinenrT,
    win_height: c_int,
) -> c_int {
    let delta = target_topline - current_topline;
    if delta == 0 {
        return 0;
    }

    let abs_delta = delta.abs();

    // For small scrolls, move one line at a time
    if abs_delta <= 3 {
        return if delta > 0 { 1 } else { -1 };
    }

    // For larger scrolls, use proportional step (max half window)
    let max_step = (win_height / 2).max(1);
    let step = ((abs_delta as c_int) / 4).clamp(1, max_step);

    if delta > 0 {
        step
    } else {
        -step
    }
}

/// Check if scrolling would be more efficient than full redraw.
///
/// Returns 1 if scroll optimization should be used, 0 if full redraw is better.
#[no_mangle]
pub extern "C" fn rs_scroll_vs_redraw(
    scroll_lines: c_int,
    win_height: c_int,
    changed_lines: c_int,
) -> c_int {
    let abs_scroll = scroll_lines.abs();

    // If scrolling more than window height, redraw is always needed
    if abs_scroll >= win_height {
        return 0;
    }

    // Calculate lines that would be preserved by scrolling
    let preserved = win_height - abs_scroll;

    // Calculate lines that would need redraw anyway due to changes
    let effective_preserved = preserved - changed_lines.min(preserved);

    // Scrolling is worth it if we preserve at least 1/3 of the window
    c_int::from(effective_preserved > win_height / 3)
}

/// Calculate the first line that needs to be redrawn after scrolling.
///
/// Returns the first line number (0-based from window top) that needs drawing.
#[no_mangle]
pub extern "C" fn rs_scroll_first_redraw_line(scroll_lines: c_int, win_height: c_int) -> c_int {
    use std::cmp::Ordering;
    match scroll_lines.cmp(&0) {
        Ordering::Greater => 0, // Scrolled up - need to redraw from top
        Ordering::Less => (win_height + scroll_lines).max(0), // Scrolled down
        Ordering::Equal => win_height, // No scroll
    }
}

/// Calculate the last line that needs to be redrawn after scrolling.
///
/// Returns the last line number (0-based from window top, exclusive) that needs drawing.
#[no_mangle]
pub extern "C" fn rs_scroll_last_redraw_line(scroll_lines: c_int, win_height: c_int) -> c_int {
    use std::cmp::Ordering;
    match scroll_lines.cmp(&0) {
        Ordering::Greater => scroll_lines.min(win_height), // Scrolled up
        Ordering::Less => win_height,                      // Scrolled down
        Ordering::Equal => 0,                              // No scroll
    }
}

/// Calculate the number of filler lines visible at the top of window.
///
/// Returns the number of filler lines that should be shown above the first text line.
#[no_mangle]
pub extern "C" fn rs_visible_filler_lines(topfill: c_int, view_height: c_int) -> c_int {
    topfill.min(view_height).max(0)
}

/// Check if a line is in the visible portion of the window.
///
/// Returns 1 if the line is visible, 0 otherwise.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_line_visible_in_window(
    lnum: LinenrT,
    topline: LinenrT,
    botline: LinenrT,
) -> c_int {
    c_int::from(lnum >= topline && lnum <= botline)
}

/// Calculate the scroll direction from topline change.
///
/// Returns positive for scroll up (content moves up, lower lines become visible),
/// negative for scroll down, 0 for no change.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_scroll_direction(old_topline: LinenrT, new_topline: LinenrT) -> c_int {
    use std::cmp::Ordering;
    match new_topline.cmp(&old_topline) {
        Ordering::Greater => 1, // Scrolled up (moved down in buffer)
        Ordering::Less => -1,   // Scrolled down (moved up in buffer)
        Ordering::Equal => 0,   // No scroll
    }
}

/// Check if cursor is above the visible window area.
///
/// Returns 1 if cursor is above window, 0 otherwise.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_cursor_above_window(cursor_lnum: LinenrT, topline: LinenrT) -> c_int {
    c_int::from(cursor_lnum < topline)
}

/// Check if cursor is below the visible window area.
///
/// Returns 1 if cursor is below window, 0 otherwise.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_cursor_below_window(cursor_lnum: LinenrT, botline: LinenrT) -> c_int {
    c_int::from(cursor_lnum > botline)
}

/// Calculate the minimum scroll needed to make cursor visible.
///
/// Returns the number of lines to scroll (positive = up, negative = down).
/// Returns 0 if cursor is already visible.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_scroll_to_cursor(
    cursor_lnum: LinenrT,
    topline: LinenrT,
    botline: LinenrT,
    scrolloff: c_int,
) -> c_int {
    let effective_top = topline + scrolloff as LinenrT;
    let effective_bot = botline - scrolloff as LinenrT;

    if cursor_lnum < effective_top {
        // Need to scroll down (decrease topline)
        -((effective_top - cursor_lnum) as c_int)
    } else if cursor_lnum > effective_bot && effective_bot >= effective_top {
        // Need to scroll up (increase topline)
        (cursor_lnum - effective_bot) as c_int
    } else {
        0 // Cursor is visible with scrolloff
    }
}

/// Calculate rows consumed by a range of buffer lines.
///
/// This is a simple estimate that assumes one row per line.
/// For accurate counting with folds/wraps, use plines functions.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_line_count_to_rows(start_lnum: LinenrT, end_lnum: LinenrT) -> c_int {
    if end_lnum >= start_lnum {
        (end_lnum - start_lnum + 1) as c_int
    } else {
        0
    }
}

/// Check if screen line cache is valid for given topline.
///
/// Returns 1 if the cached line at index 0 matches the expected topline.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_wlines_cache_valid(
    cached_topline: LinenrT,
    expected_topline: LinenrT,
) -> c_int {
    c_int::from(cached_topline == expected_topline)
}

/// Calculate the invalidation range for a text change.
///
/// Given start and end of a change, and the scroll offset,
/// returns the first row that needs to be invalidated.
#[no_mangle]
pub extern "C" fn rs_change_invalidation_start(
    change_start_lnum: LinenrT,
    topline: LinenrT,
    topfill: c_int,
) -> c_int {
    if change_start_lnum < topline {
        // Change starts above window
        0
    } else {
        // Change starts within or below window
        ((change_start_lnum - topline) as c_int + topfill).max(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_corner_values() {
        assert_eq!(WindowCorner::TopLeft as c_int, 0);
        assert_eq!(WindowCorner::TopRight as c_int, 1);
        assert_eq!(WindowCorner::BottomLeft as c_int, 2);
        assert_eq!(WindowCorner::BottomRight as c_int, 3);
    }

    #[test]
    fn test_null_window_returns_false() {
        let null_win = unsafe { WinHandle::from_ptr(std::ptr::null_mut()) };
        assert!(!hsep_connected_impl(null_win, WindowCorner::TopLeft));
        assert!(!vsep_connected_impl(null_win, WindowCorner::TopLeft));
    }

    #[test]
    fn test_hlf_c_constant() {
        // HLF_C should be 21 (WinSeparator highlight group)
        let hlf_c = HLF_C;
        assert_eq!(hlf_c, 21);
    }

    #[test]
    fn test_upd_valid_constant() {
        // UPD_VALID should be 20
        let upd_valid = UPD_VALID;
        assert_eq!(upd_valid, 20);
    }

    #[test]
    fn test_window_corner_distinct() {
        // All corner values should be distinct
        let corners = [
            WindowCorner::TopLeft as c_int,
            WindowCorner::TopRight as c_int,
            WindowCorner::BottomLeft as c_int,
            WindowCorner::BottomRight as c_int,
        ];
        for i in 0..corners.len() {
            for j in (i + 1)..corners.len() {
                assert_ne!(corners[i], corners[j]);
            }
        }
    }

    #[test]
    fn test_window_corner_sequential() {
        // Corner values should be sequential 0-3
        assert_eq!(WindowCorner::TopLeft as c_int, 0);
        assert_eq!(
            WindowCorner::TopRight as c_int,
            WindowCorner::TopLeft as c_int + 1
        );
        assert_eq!(
            WindowCorner::BottomLeft as c_int,
            WindowCorner::TopRight as c_int + 1
        );
        assert_eq!(
            WindowCorner::BottomRight as c_int,
            WindowCorner::BottomLeft as c_int + 1
        );
    }

    #[test]
    fn test_window_corner_size() {
        // WindowCorner enum should be c_int sized
        assert_eq!(
            std::mem::size_of::<WindowCorner>(),
            std::mem::size_of::<c_int>()
        );
    }

    // Phase 6: Screen Update Logic tests

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_redraw_type_constants() {
        // Verify redraw type constants have correct ordering
        assert!(UPD_REDRAW_TOP < UPD_INVERTED_ALL);
        assert!(UPD_INVERTED_ALL < UPD_INVERTED);
        assert!(UPD_INVERTED < UPD_VALID);
        assert!(UPD_VALID < UPD_SOME_VALID);
        assert!(UPD_SOME_VALID < UPD_NOT_VALID);
        assert!(UPD_NOT_VALID < UPD_CLEAR);
    }

    #[test]
    fn test_redraw_type_is_full() {
        assert_eq!(rs_redraw_type_is_full(UPD_VALID), 0);
        assert_eq!(rs_redraw_type_is_full(UPD_SOME_VALID), 0);
        assert_eq!(rs_redraw_type_is_full(UPD_NOT_VALID), 1);
        assert_eq!(rs_redraw_type_is_full(UPD_CLEAR), 1);
    }

    #[test]
    fn test_redraw_type_is_clear() {
        assert_eq!(rs_redraw_type_is_clear(UPD_NOT_VALID), 0);
        assert_eq!(rs_redraw_type_is_clear(UPD_CLEAR), 1);
        assert_eq!(rs_redraw_type_is_clear(UPD_CLEAR + 1), 1);
    }

    #[test]
    fn test_max_redraw_type() {
        assert_eq!(rs_max_redraw_type(UPD_VALID, UPD_NOT_VALID), UPD_NOT_VALID);
        assert_eq!(rs_max_redraw_type(UPD_NOT_VALID, UPD_VALID), UPD_NOT_VALID);
        assert_eq!(rs_max_redraw_type(UPD_CLEAR, UPD_NOT_VALID), UPD_CLEAR);
    }

    #[test]
    fn test_redraw_type_subsumes() {
        assert_eq!(rs_redraw_type_subsumes(UPD_CLEAR, UPD_NOT_VALID), 1);
        assert_eq!(rs_redraw_type_subsumes(UPD_NOT_VALID, UPD_VALID), 1);
        assert_eq!(rs_redraw_type_subsumes(UPD_VALID, UPD_NOT_VALID), 0);
    }

    #[test]
    fn test_scroll_is_beneficial() {
        // Scrolling 1 line in 24 line window is beneficial
        assert_eq!(rs_scroll_is_beneficial(1, 24), 1);
        // Scrolling 10 lines in 24 line window is beneficial
        assert_eq!(rs_scroll_is_beneficial(10, 24), 1);
        // Scrolling 12+ lines in 24 line window is not beneficial
        assert_eq!(rs_scroll_is_beneficial(12, 24), 0);
        // Scrolling 0 lines is not beneficial
        assert_eq!(rs_scroll_is_beneficial(0, 24), 0);
        // Negative scroll also works
        assert_eq!(rs_scroll_is_beneficial(-5, 24), 1);
    }

    #[test]
    fn test_scroll_source_row() {
        assert_eq!(rs_scroll_source_row(10, 5), 15);
        assert_eq!(rs_scroll_source_row(10, -5), 5);
        assert_eq!(rs_scroll_source_row(0, 3), 3);
    }

    #[test]
    fn test_scroll_row_valid() {
        assert_eq!(rs_scroll_row_valid(5, 0, 10), 1);
        assert_eq!(rs_scroll_row_valid(0, 0, 10), 1);
        assert_eq!(rs_scroll_row_valid(9, 0, 10), 1);
        assert_eq!(rs_scroll_row_valid(10, 0, 10), 0); // at boundary
        assert_eq!(rs_scroll_row_valid(-1, 0, 10), 0); // below range
    }

    #[test]
    fn test_calc_scroll_region() {
        let mut start: c_int = 0;
        let mut end: c_int = 0;

        unsafe {
            // Scrolling up (positive delta)
            rs_calc_scroll_region(5, 20, 3, &raw mut start, &raw mut end);
            assert_eq!(start, 8); // win_row + scroll_delta
            assert_eq!(end, 25); // win_row + win_height

            // Scrolling down (negative delta)
            rs_calc_scroll_region(5, 20, -3, &raw mut start, &raw mut end);
            assert_eq!(start, 5); // win_row
            assert_eq!(end, 22); // win_row + win_height + scroll_delta
        }
    }

    // =============================================================================
    // Phase D1: Redraw Management Tests
    // =============================================================================

    #[test]
    fn test_win_update_state_default() {
        let state = WinUpdateState::default();
        assert_eq!(state.top_end, 0);
        assert_eq!(state.mid_start, 999);
        assert_eq!(state.mid_end, 0);
        assert_eq!(state.bot_start, 999);
        assert_eq!(state.bot_scroll_start, 999);
        assert_eq!(state.scrolled_down, 0);
        assert_eq!(state.top_to_mod, 0);
    }

    #[test]
    fn test_win_update_state_init() {
        let state = rs_win_update_state_init();
        assert_eq!(state.top_end, 0);
        assert_eq!(state.mid_start, 999);
        assert_eq!(state.mid_end, 0);
        assert_eq!(state.bot_start, 999);
        assert_eq!(state.bot_scroll_start, 999);
        assert_eq!(state.scrolled_down, 0);
        assert_eq!(state.top_to_mod, 0);
    }

    #[test]
    fn test_win_update_state_size() {
        // WinUpdateState should be properly sized for FFI
        assert_eq!(
            std::mem::size_of::<WinUpdateState>(),
            std::mem::size_of::<c_int>() * 7
        );
    }

    #[test]
    fn test_null_window_skip_update() {
        let null_win = unsafe { WinHandle::from_ptr(std::ptr::null_mut()) };
        // Null window should return skip
        assert_eq!(rs_win_should_skip_update(null_win), 1);
    }

    #[test]
    fn test_null_window_needs_full_update() {
        let null_win = unsafe { WinHandle::from_ptr(std::ptr::null_mut()) };
        // Null window should not need full update
        assert_eq!(rs_win_needs_full_update(null_win), 0);
    }

    #[test]
    fn test_null_window_effective_redr_type() {
        let null_win = unsafe { WinHandle::from_ptr(std::ptr::null_mut()) };
        // Null window should return 0
        assert_eq!(rs_win_get_effective_redr_type(null_win), 0);
    }

    #[test]
    fn test_buf_handle_null() {
        let null_buf = BufHandle(std::ptr::null_mut());
        assert!(null_buf.is_null());
    }

    // =============================================================================
    // Phase D4: Scroll Optimization Tests
    // =============================================================================

    #[test]
    fn test_smooth_scroll_step() {
        // No scroll needed
        assert_eq!(rs_smooth_scroll_step(10, 10, 24), 0);

        // Small scroll (1-3 lines) - move one at a time
        assert_eq!(rs_smooth_scroll_step(10, 11, 24), 1);
        assert_eq!(rs_smooth_scroll_step(10, 12, 24), 1);
        assert_eq!(rs_smooth_scroll_step(10, 13, 24), 1);
        assert_eq!(rs_smooth_scroll_step(10, 9, 24), -1);

        // Larger scroll - proportional step
        let step = rs_smooth_scroll_step(10, 30, 24);
        assert!(step > 1 && step <= 12); // max half window
    }

    #[test]
    fn test_scroll_vs_redraw() {
        // Small scroll with no changes - use scroll
        assert_eq!(rs_scroll_vs_redraw(3, 24, 0), 1);

        // Scroll entire window - use redraw
        assert_eq!(rs_scroll_vs_redraw(24, 24, 0), 0);

        // Scroll more than window - use redraw
        assert_eq!(rs_scroll_vs_redraw(30, 24, 0), 0);

        // Small scroll but many changed lines - might still redraw
        assert_eq!(rs_scroll_vs_redraw(5, 24, 20), 0);
    }

    #[test]
    fn test_scroll_first_redraw_line() {
        // Scroll up - redraw from top
        assert_eq!(rs_scroll_first_redraw_line(5, 24), 0);

        // Scroll down - redraw from bottom portion
        assert_eq!(rs_scroll_first_redraw_line(-5, 24), 19);

        // No scroll
        assert_eq!(rs_scroll_first_redraw_line(0, 24), 24);
    }

    #[test]
    fn test_scroll_last_redraw_line() {
        // Scroll up - redraw first N lines
        assert_eq!(rs_scroll_last_redraw_line(5, 24), 5);

        // Scroll down - redraw to end
        assert_eq!(rs_scroll_last_redraw_line(-5, 24), 24);

        // No scroll
        assert_eq!(rs_scroll_last_redraw_line(0, 24), 0);
    }

    #[test]
    fn test_visible_filler_lines() {
        assert_eq!(rs_visible_filler_lines(3, 24), 3);
        assert_eq!(rs_visible_filler_lines(30, 24), 24); // clamped to view_height
        assert_eq!(rs_visible_filler_lines(-5, 24), 0); // no negative
        assert_eq!(rs_visible_filler_lines(0, 24), 0);
    }

    #[test]
    fn test_line_visible_in_window() {
        // Line in visible range
        assert_eq!(rs_line_visible_in_window(15, 10, 30), 1);

        // Line at boundaries
        assert_eq!(rs_line_visible_in_window(10, 10, 30), 1);
        assert_eq!(rs_line_visible_in_window(30, 10, 30), 1);

        // Line outside range
        assert_eq!(rs_line_visible_in_window(5, 10, 30), 0);
        assert_eq!(rs_line_visible_in_window(35, 10, 30), 0);
    }

    #[test]
    fn test_scroll_direction() {
        // Scrolled up (higher topline)
        assert_eq!(rs_scroll_direction(10, 15), 1);

        // Scrolled down (lower topline)
        assert_eq!(rs_scroll_direction(15, 10), -1);

        // No change
        assert_eq!(rs_scroll_direction(10, 10), 0);
    }

    #[test]
    fn test_cursor_position_checks() {
        // Cursor above window
        assert_eq!(rs_cursor_above_window(5, 10), 1);
        assert_eq!(rs_cursor_above_window(10, 10), 0);
        assert_eq!(rs_cursor_above_window(15, 10), 0);

        // Cursor below window
        assert_eq!(rs_cursor_below_window(35, 30), 1);
        assert_eq!(rs_cursor_below_window(30, 30), 0);
        assert_eq!(rs_cursor_below_window(25, 30), 0);
    }

    #[test]
    fn test_scroll_to_cursor() {
        // Cursor visible - no scroll needed
        assert_eq!(rs_scroll_to_cursor(15, 10, 30, 3), 0);

        // Cursor above visible area - scroll down
        assert_eq!(rs_scroll_to_cursor(10, 15, 35, 3), -8); // 10 < (15+3), need -8

        // Cursor below visible area - scroll up
        assert_eq!(rs_scroll_to_cursor(35, 10, 30, 3), 8); // 35 > (30-3), need +8

        // Cursor at scrolloff boundary - just visible
        assert_eq!(rs_scroll_to_cursor(13, 10, 30, 3), 0); // 13 >= (10+3)
    }

    #[test]
    fn test_line_count_to_rows() {
        assert_eq!(rs_line_count_to_rows(10, 20), 11); // 10-20 inclusive
        assert_eq!(rs_line_count_to_rows(1, 1), 1);
        assert_eq!(rs_line_count_to_rows(20, 10), 0); // invalid range
    }

    #[test]
    fn test_wlines_cache_valid() {
        assert_eq!(rs_wlines_cache_valid(10, 10), 1);
        assert_eq!(rs_wlines_cache_valid(10, 15), 0);
    }

    #[test]
    fn test_change_invalidation_start() {
        // Change starts above window
        assert_eq!(rs_change_invalidation_start(5, 10, 0), 0);

        // Change starts at topline
        assert_eq!(rs_change_invalidation_start(10, 10, 0), 0);

        // Change starts within window
        assert_eq!(rs_change_invalidation_start(15, 10, 0), 5);

        // With topfill
        assert_eq!(rs_change_invalidation_start(10, 10, 3), 3);
    }

    // =============================================================================
    // Phase D5: Integration Tests
    // =============================================================================

    /// Test a complete scroll scenario combining multiple functions.
    #[test]
    fn test_integration_scroll_scenario() {
        // Simulate a window with 24 visible lines
        let win_height = 24;
        let old_topline: LinenrT = 100;
        let new_topline: LinenrT = 105; // Scrolled down 5 lines

        // 1. Calculate scroll direction
        let direction = rs_scroll_direction(old_topline, new_topline);
        assert_eq!(direction, 1); // Scrolled up (content moved up)

        // 2. Check if scroll is beneficial
        let scroll_lines = (new_topline - old_topline) as c_int;
        let is_beneficial = rs_scroll_is_beneficial(scroll_lines, win_height);
        assert_eq!(is_beneficial, 1); // 5 lines scroll in 24-line window is good

        // 3. Calculate reusable lines
        let reusable = rs_scroll_reusable_lines(
            unsafe { WinHandle::from_ptr(std::ptr::null_mut()) },
            old_topline,
            new_topline,
            win_height,
        );
        assert_eq!(reusable, 19); // 24 - 5 = 19 lines can be reused

        // 4. Calculate redraw bounds
        let first_redraw = rs_scroll_first_redraw_line(scroll_lines, win_height);
        let last_redraw = rs_scroll_last_redraw_line(scroll_lines, win_height);
        assert_eq!(first_redraw, 0); // Start redrawing from top
        assert_eq!(last_redraw, 5); // Redraw first 5 lines
    }

    /// Test a complete cursor visibility scenario.
    #[test]
    fn test_integration_cursor_visibility() {
        let topline: LinenrT = 100;
        let botline: LinenrT = 123; // 24 visible lines
        let scrolloff = 3;

        // Test cursor in visible area
        let cursor_visible: LinenrT = 110;
        assert_eq!(
            rs_line_visible_in_window(cursor_visible, topline, botline),
            1
        );
        assert_eq!(rs_cursor_above_window(cursor_visible, topline), 0);
        assert_eq!(rs_cursor_below_window(cursor_visible, botline), 0);
        assert_eq!(
            rs_scroll_to_cursor(cursor_visible, topline, botline, scrolloff),
            0
        );

        // Test cursor above visible area
        let cursor_above: LinenrT = 95;
        assert_eq!(rs_line_visible_in_window(cursor_above, topline, botline), 0);
        assert_eq!(rs_cursor_above_window(cursor_above, topline), 1);
        // Should scroll down to show cursor
        let scroll_needed = rs_scroll_to_cursor(cursor_above, topline, botline, scrolloff);
        assert!(scroll_needed < 0); // Negative means scroll down

        // Test cursor below visible area
        let cursor_below: LinenrT = 130;
        assert_eq!(rs_line_visible_in_window(cursor_below, topline, botline), 0);
        assert_eq!(rs_cursor_below_window(cursor_below, botline), 1);
        // Should scroll up to show cursor
        let scroll_needed = rs_scroll_to_cursor(cursor_below, topline, botline, scrolloff);
        assert!(scroll_needed > 0); // Positive means scroll up
    }

    /// Test smooth scroll step calculation for various scenarios.
    #[test]
    fn test_integration_smooth_scroll() {
        let win_height = 40;

        // Test progressive scroll steps
        let target_offsets = [1, 2, 5, 10, 20, 50, 100];
        let mut prev_abs_step = 0;

        for &offset in &target_offsets {
            let step = rs_smooth_scroll_step(100, 100 + offset, win_height);
            let abs_step = step.abs();

            // Step should be positive for positive offset
            assert!(step > 0);
            // Step should not exceed half window
            assert!(abs_step <= win_height / 2);
            // Larger offsets should generally produce larger steps
            // (but this isn't strictly monotonic due to clamping)
            if offset > 3 {
                assert!(abs_step >= prev_abs_step);
            }
            prev_abs_step = abs_step;
        }
    }

    /// Test full redraw vs scroll optimization decision.
    #[test]
    fn test_integration_redraw_decision() {
        let win_height = 30;

        // Case 1: Small scroll, no changes -> scroll
        assert_eq!(rs_scroll_vs_redraw(3, win_height, 0), 1);

        // Case 2: Small scroll, few changes -> scroll
        assert_eq!(rs_scroll_vs_redraw(3, win_height, 2), 1);

        // Case 3: Small scroll, many changes -> might redraw
        // With 3 scroll + 20 changes, only 7 lines preserved (< 10 = win_height/3)
        assert_eq!(rs_scroll_vs_redraw(3, win_height, 20), 0);

        // Case 4: Large scroll -> redraw
        assert_eq!(rs_scroll_vs_redraw(25, win_height, 0), 0);

        // Case 5: Moderate scroll, moderate changes
        // 10 scroll preserves 20, minus 5 changes = 15 preserved (> 10)
        assert_eq!(rs_scroll_vs_redraw(10, win_height, 5), 1);
    }

    /// Test change invalidation range calculation.
    #[test]
    fn test_integration_change_invalidation() {
        let topline: LinenrT = 100;
        let topfill = 2;

        // Change entirely above window
        let start = rs_change_invalidation_start(50, topline, topfill);
        assert_eq!(start, 0); // Start from row 0

        // Change at topline
        let start = rs_change_invalidation_start(100, topline, topfill);
        assert_eq!(start, topfill); // Offset by topfill

        // Change 5 lines below topline
        let start = rs_change_invalidation_start(105, topline, topfill);
        assert_eq!(start, 5 + topfill); // 5 lines + topfill
    }

    /// Test WinUpdateState initialization and field access.
    #[test]
    fn test_win_update_state_fields() {
        let state = rs_win_update_state_init();

        // Verify all fields have expected default values
        assert_eq!(state.top_end, 0);
        assert_eq!(state.mid_start, 999);
        assert_eq!(state.mid_end, 0);
        assert_eq!(state.bot_start, 999);
        assert_eq!(state.bot_scroll_start, 999);
        assert_eq!(state.scrolled_down, 0);
        assert_eq!(state.top_to_mod, 0);

        // Verify struct is properly aligned for FFI
        assert_eq!(
            std::mem::align_of::<WinUpdateState>(),
            std::mem::align_of::<c_int>()
        );
    }

    /// Test redraw type comparison functions work correctly together.
    #[test]
    fn test_integration_redraw_types() {
        // Test that all our redraw type helpers are consistent
        for base in [UPD_VALID, UPD_NOT_VALID, UPD_CLEAR] {
            // Max with self should return self
            assert_eq!(rs_max_redraw_type(base, base), base);

            // Self should subsume self
            assert_eq!(rs_redraw_type_subsumes(base, base), 1);
        }

        // Test transitivity: if A subsumes B and B subsumes C, then A subsumes C
        let types = [UPD_VALID, UPD_SOME_VALID, UPD_NOT_VALID, UPD_CLEAR];
        for i in 0..types.len() {
            for j in i..types.len() {
                // Higher index types should subsume lower index types
                assert_eq!(rs_redraw_type_subsumes(types[j], types[i]), 1);
            }
        }
    }
}
