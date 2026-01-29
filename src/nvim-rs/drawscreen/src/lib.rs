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
pub const HLF_C: c_int = 39;

/// UPD_VALID constant from screen.h - redraw when scrolled or text changed
const UPD_VALID: c_int = 20;

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

// Redraw type constants from screen.h
/// UPD_NOT_VALID - redraw all windows for buffer changes
pub const UPD_NOT_VALID: c_int = 30;
/// UPD_SOME_VALID - redraw some windows (changed highlighting)
pub const UPD_SOME_VALID: c_int = 25;
/// UPD_INVERTED - redraw inverted area (selection)
pub const UPD_INVERTED: c_int = 15;
/// UPD_INVERTED_ALL - redraw all inverted areas
pub const UPD_INVERTED_ALL: c_int = 10;
/// UPD_REDRAW_TOP - redraw top of window
pub const UPD_REDRAW_TOP: c_int = 5;
/// UPD_CLEAR - clear screen and redraw all
pub const UPD_CLEAR: c_int = 40;

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
        // HLF_C should be 39 (WinSeparator highlight group)
        let hlf_c = HLF_C;
        assert_eq!(hlf_c, 39);
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
}
