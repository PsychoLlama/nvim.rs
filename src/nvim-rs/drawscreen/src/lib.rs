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

use std::ffi::{c_char, c_int, c_void};
use std::ptr::{addr_of, addr_of_mut};

/// UIExtension value for kUIMessages (ui_defs.h)
const K_UI_MESSAGES: c_int = 4;

use nvim_window::{rs_frame2win, Frame, WinHandle, FR_COL, FR_LEAF, FR_ROW};

// Direct access to C globals (avoids thin C accessor functions).
extern "C" {
    static mut msg_silent: c_int;
    static mut State: c_int;
    /// Global must_redraw flag (int in C).
    static mut must_redraw: c_int;
    /// Global redraw_not_allowed flag (bool in C, mapped as c_int for atomics).
    static mut redraw_not_allowed: bool;
    /// Global updating_screen flag.
    static mut updating_screen: bool;
    /// Global RedrawingDisabled counter.
    static mut RedrawingDisabled: c_int;
    /// Global VIsual_mode character.
    static mut VIsual_mode: c_int;
    /// p_lz lazyredraw option.
    static p_lz: c_int;
    /// ru_wid ruler width.
    static ru_wid: c_int;
    /// sc_col show-command column.
    static mut sc_col: c_int;
    /// ru_col ruler column.
    static mut ru_col: c_int;
    /// p_sloc showcmdloc option string.
    static p_sloc: *const c_char;
    /// Rows global.
    static mut Rows: c_int;
    /// Columns global.
    static mut Columns: c_int;
    /// redraw_tabline flag.
    static mut redraw_tabline: bool;
    /// need_maketitle flag.
    static mut need_maketitle: bool;
    /// redraw_mode flag (show mode in status line).
    static mut redraw_mode: c_int;
    /// p_sc (showcmd) option.
    static p_sc: c_int;
    /// p_icon (icon) option.
    static p_icon: c_int;
    /// p_title (title) option.
    static p_title: c_int;
    /// stl_syntax flags.
    static stl_syntax: c_int;
    /// p_ru (ruler) option.
    static p_ru: c_int;
    /// reg_recording: current recording register (0 = not recording).
    static reg_recording: c_int;
    /// msg_col: current message column.
    static mut msg_col: c_int;
    /// msg_row: current message row.
    static mut msg_row: c_int;
}

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
    #[link_name = "rs_global_stl_height"]
    fn global_stl_height() -> c_int;
    fn nvim_set_redraw_cmdline(val: bool);
    fn nvim_get_default_gridview() -> GridViewHandle;

    // Grid functions (already in Rust, called via FFI)
    fn rs_grid_line_start(view: GridViewHandle, row: c_int);
    fn rs_grid_line_put_schar(col: c_int, schar: ScharT, attr: c_int);
    fn rs_grid_line_fill(start_col: c_int, end_col: c_int, sc: ScharT, attr: c_int) -> c_int;
    fn rs_grid_line_flush();

    static do_redraw: bool;
}

extern "C" {
    // Phase 1: Flag/guard function accessors
    fn nvim_char_avail() -> c_int;
    fn nvim_get_KeyTyped() -> bool;
    fn ui_has(ext: c_int) -> bool;
    fn nvim_cmdline_mouse_used() -> c_int;
    #[link_name = "rs_min_rows_for_all_tabpages"]
    fn nvim_min_rows_for_all_tabpages() -> c_int;
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

/// Marks all status lines and window bars in the current tab for redraw.
#[unsafe(export_name = "status_redraw_all")]
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
        if p_ru != 0 && curwin_status_h == 0 && curwin_redr_status == 0 {
            nvim_set_redraw_cmdline(true);
            redraw_later(curwin, UPD_VALID);
        }
    }
}

/// Marks status lines and window bars of the given buffer for redraw.
#[unsafe(export_name = "status_redraw_buf")]
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

/// Marks status lines and window bars of the current buffer for redraw.
#[unsafe(export_name = "status_redraw_curbuf")]
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
    unsafe { must_redraw }
}

/// Set the global must_redraw flag.
#[no_mangle]
pub extern "C" fn rs_set_must_redraw(val: c_int) {
    unsafe { must_redraw = val };
}

/// Check if redraw is currently allowed.
#[no_mangle]
pub extern "C" fn rs_redraw_allowed() -> c_int {
    unsafe { c_int::from(!redraw_not_allowed) }
}

/// Set the must_redraw global only if type is higher.
#[unsafe(export_name = "set_must_redraw")]
pub extern "C" fn rs_set_must_redraw_max(redraw_type: c_int) {
    unsafe {
        if !redraw_not_allowed {
            let current = must_redraw;
            if redraw_type > current {
                must_redraw = redraw_type;
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
        redraw_later_impl(wp, UPD_VALID);
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
    // Grid invalidation
    fn nvim_win_grid_alloc_valid(wp: WinHandle) -> c_int;
    fn nvim_win_grid_alloc_set_valid(wp: WinHandle, val: c_int);

    // Buffer comparison
    fn nvim_win_buffer_eq(wp: WinHandle, buf: BufHandle) -> c_int;

    // Buffer accessors
    fn nvim_buf_get_ml_line_count(buf: BufHandle) -> LinenrT;

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
        if redraw_not_allowed {
            return;
        }

        let current_type = nvim_win_get_redr_type(wp);
        if current_type >= redraw_type {
            return;
        }

        // Set the new redraw type
        nvim_win_set_redr_type(wp, redraw_type);

        // If type >= UPD_NOT_VALID, invalidate line cache
        if redraw_type >= UPD_NOT_VALID {
            nvim_win_set_lines_valid(wp, 0);
        }

        // Update must_redraw global
        if redraw_type > must_redraw {
            must_redraw = redraw_type;
        }
    }
}

/// Marks a window for later redraw with the specified type.
#[unsafe(export_name = "redraw_later")]
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
        if !redraw_not_allowed && redraw_type > must_redraw {
            must_redraw = redraw_type;
        }
    }
}

/// Marks all windows in the current tab for later redraw.
#[unsafe(export_name = "redraw_all_later")]
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

/// Marks all windows displaying the given buffer for redraw.
#[unsafe(export_name = "redraw_buf_later")]
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

/// Marks all windows displaying the current buffer for redraw.
#[unsafe(export_name = "redraw_curbuf_later")]
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

/// Invalidates highlights for all windows, forcing full redraw.
#[unsafe(export_name = "screen_invalidate_highlights")]
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

/// Marks a range of lines in a window for redraw.
#[unsafe(export_name = "redraw_win_range_later")]
pub extern "C" fn rs_redraw_win_range_later(wp: WinHandle, first: LinenrT, last: LinenrT) {
    redraw_win_range_later_impl(wp, first, last);
}

/// Mark a single line in a window for redraw.
///
/// This is the Rust equivalent of `redrawWinline()` in drawscreen.c.
#[unsafe(export_name = "redrawWinline")]
#[allow(non_snake_case)]
pub extern "C" fn rs_redrawWinline(wp: WinHandle, lnum: LinenrT) {
    redraw_win_range_later_impl(wp, lnum, lnum);
}

/// Mark a specific line in all windows showing a buffer for redraw.
///
/// This is the Rust equivalent of `redraw_buf_line_later()` in drawscreen.c.
#[unsafe(export_name = "redraw_buf_line_later")]
pub extern "C" fn rs_redraw_buf_line_later(buf: BufHandle, line: LinenrT, force: bool) {
    unsafe {
        let ml_line_count = nvim_buf_get_ml_line_count(buf);
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            if nvim_win_buffer_eq(wp, buf) != 0 {
                let clamped = line.min(ml_line_count);
                redraw_win_range_later_impl(wp, clamped, clamped);
                if force && line > ml_line_count {
                    nvim_win_set_redraw_bot(wp, line);
                }
            }
            wp = nvim_win_get_next(wp);
        }
    }
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

/// Marks a range of lines in all windows displaying the buffer.
#[unsafe(export_name = "redraw_buf_range_later")]
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
                if !redraw_not_allowed && UPD_VALID > must_redraw {
                    must_redraw = UPD_VALID;
                }
            }
            wp = nvim_win_get_next(wp);
        }
    }
}

/// Marks status lines of windows displaying the buffer for redraw.
#[unsafe(export_name = "redraw_buf_status_later")]
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
            nvim_win_set_redr_type(wp, 0);
            return 1;
        }

        if view_width == 0 {
            // Draw separators and return skip
            rs_draw_vsep_win(wp);
            rs_draw_sep_connectors_win(wp);
            nvim_win_set_redr_type(wp, 0);
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
            nvim_win_set_redr_type(wp, 0);
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
        let redr_type = nvim_win_get_redr_type(wp);
        c_int::from(redr_type >= UPD_NOT_VALID)
    }
}

/// Get the effective redraw type for a window, clamped to valid range.
#[no_mangle]
pub extern "C" fn rs_win_get_effective_redr_type(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe { nvim_win_get_redr_type(wp).clamp(0, UPD_CLEAR) }
}

// =============================================================================
// Phase D1.4: Screen Update Loop Helpers
// =============================================================================

// Additional C function declarations for screen update
extern "C" {
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
    unsafe { c_int::from(updating_screen) }
}

/// Set the updating_screen flag.
#[no_mangle]
pub extern "C" fn rs_set_updating_screen(val: c_int) {
    unsafe {
        updating_screen = val != 0;
    }
}

/// Check if redrawing is currently disabled.
///
/// Returns 1 if RedrawingDisabled is set, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_is_redrawing_disabled() -> c_int {
    unsafe { RedrawingDisabled }
}

/// Check if a window needs its border redrawn.
#[no_mangle]
pub extern "C" fn rs_win_needs_border_redraw(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let redr_border = nvim_win_get_redr_border(wp);
        let redr_type = nvim_win_get_redr_type(wp);
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
        nvim_win_set_redr_type(wp, 0);
        // Reset border redraw flag
        nvim_win_set_redr_border(wp, 0);
    }
}

// =============================================================================
// Phase D1.4: Visual Mode Update Helpers
// =============================================================================

// Additional C function declarations for visual mode
extern "C" {
    static mut VIsual_active: bool;
    fn nvim_win_get_old_visual_mode(wp: WinHandle) -> c_int;
    fn nvim_win_set_old_visual_mode(wp: WinHandle, val: c_int);
    fn nvim_win_set_old_cursor_lnum(wp: WinHandle, val: LinenrT);
    fn nvim_win_set_old_visual_lnum(wp: WinHandle, val: LinenrT);
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
        let visual_active = VIsual_active;
        let old_visual_mode = nvim_win_get_old_visual_mode(wp);

        if !visual_active && old_visual_mode == 0 {
            return 0;
        }

        if visual_active {
            let current_mode = VIsual_mode;
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
        let visual_active = VIsual_active;

        if visual_active {
            let visual_mode = VIsual_mode;
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

// =============================================================================
// Phase 1: Pure Flag/Guard Functions
// =============================================================================

/// MODE_CMDLINE from state_defs.h
const MODE_CMDLINE: c_int = 0x08;
/// MIN_COLUMNS from window.h
const MIN_COLUMNS: c_int = 12;

/// Return true if redrawing should currently be done.
///
/// Rust equivalent of `redrawing()` in drawscreen.c.
fn redrawing_impl() -> bool {
    unsafe {
        (do_redraw || nvim_get_KeyTyped() || nvim_char_avail() == 0 || p_lz == 0)
            && RedrawingDisabled == 0
    }
}

/// FFI export for `redrawing()`.
#[unsafe(export_name = "redrawing")]
pub extern "C" fn rs_redrawing() -> bool {
    redrawing_impl()
}

/// Check if the new Nvim application "screen" dimensions are valid.
/// Correct it if it's too small or way too big.
///
/// Rust equivalent of `check_screensize()` in drawscreen.c.
#[unsafe(export_name = "check_screensize")]
pub extern "C" fn rs_check_screensize() {
    unsafe {
        let min_rows = nvim_min_rows_for_all_tabpages();
        Rows = Rows.clamp(min_rows, 1000);
        Columns = Columns.clamp(MIN_COLUMNS, 10000);
    }
}

/// Unlike cmdline "one_key" prompts, the message part of the prompt is not stored
/// to be re-emitted: avoid clearing the prompt from the message grid.
/// This is static in C so we keep the rs_ prefix for now.
fn cmdline_number_prompt_impl() -> bool {
    unsafe {
        !ui_has(K_UI_MESSAGES) && (State & MODE_CMDLINE) != 0 && nvim_cmdline_mouse_used() != 0
    }
}

/// FFI export for `cmdline_number_prompt()`.
#[no_mangle]
pub extern "C" fn rs_cmdline_number_prompt() -> c_int {
    c_int::from(cmdline_number_prompt_impl())
}

// =============================================================================
// Phase 2: Column Computation
// =============================================================================

extern "C" {
    #[link_name = "rs_last_stl_height"]
    fn nvim_last_stl_height(morewin: c_int) -> c_int;
    fn nvim_set_vim_var_echospace(val: c_int);
}

/// COL_RULER from drawscreen.c — columns needed by standard ruler.
const COL_RULER: c_int = 17;
/// SHOWCMD_COLS from normal_defs.h — columns needed by shown command.
const SHOWCMD_COLS: c_int = 10;

/// Compute columns for ruler and shown command.
///
/// 'sc_col' is also used to decide what the maximum length of a message on
/// the status line can be. If there is a status line for the last window,
/// 'sc_col' is independent of 'ru_col'.
/// Rust equivalent of `comp_col()` in drawscreen.c.
#[unsafe(export_name = "comp_col")]
pub extern "C" fn rs_comp_col() {
    unsafe {
        let last_has_status = nvim_last_stl_height(0) > 0;
        let columns = Columns;

        let mut new_sc_col: c_int = 0;
        let mut new_ru_col: c_int = 0;

        if p_ru != 0 {
            new_ru_col = (if ru_wid != 0 { ru_wid } else { COL_RULER }) + 1;
            // no last status line, adjust sc_col
            if !last_has_status {
                new_sc_col = new_ru_col;
            }
        }
        #[allow(clippy::cast_possible_wrap)]
        let p_sloc_is_last = *p_sloc == b'l' as c_char;
        if p_sc != 0 && p_sloc_is_last {
            new_sc_col += SHOWCMD_COLS;
            if p_ru == 0 || last_has_status {
                // no need for separating space
                new_sc_col += 1;
            }
        }
        debug_assert!(new_sc_col >= 0 && c_int::MIN + new_sc_col <= columns);
        new_sc_col = columns - new_sc_col;
        debug_assert!(new_ru_col >= 0 && c_int::MIN + new_ru_col <= columns);
        new_ru_col = columns - new_ru_col;
        if new_sc_col <= 0 {
            // screen too narrow, will become a mess
            new_sc_col = 1;
        }
        if new_ru_col <= 0 {
            new_ru_col = 1;
        }
        sc_col = new_sc_col;
        ru_col = new_ru_col;
        nvim_set_vim_var_echospace(new_sc_col - 1);
    }
}

// =============================================================================
// Phase 3: Mode Display Helpers
// =============================================================================

extern "C" {
    fn nvim_get_global_busy() -> c_int;
}

extern "C" {
    fn msg_ext_ui_flush();
    fn msg_clr_eos();
    fn msg_ext_flush_showmode();
    fn msg_puts_hl(s: *const c_char, hl_id: c_int, hist: bool);
    fn shortmess(x: c_int) -> bool;
}

/// Check if mode display should be postponed.
///
/// Returns true when not redrawing or inside a mapping.
/// Rust equivalent of `skip_showmode()` in drawscreen.c.
#[unsafe(export_name = "skip_showmode")]
pub extern "C" fn rs_skip_showmode() -> bool {
    unsafe {
        // Call char_avail() only when we are going to show something, because it
        // takes a bit of time.  redrawing() may also call char_avail().
        if nvim_get_global_busy() != 0
            || msg_silent != 0
            || !redrawing_impl()
            || (nvim_char_avail() != 0 && !nvim_get_KeyTyped())
        {
            redraw_mode = 1; // show mode later
            return true;
        }
        false
    }
}

// HLF_CM: highlight group for mode message (e.g. "-- INSERT --").
// Value verified by _Static_assert in drawscreen.c.
const HLF_CM: c_int = 11;
// SHM_RECORDING: shortmess flag 'q' — suppress recording messages.
const SHM_RECORDING: c_int = b'q' as c_int;

/// Display recording message (equivalent of static `recording_mode()` in C).
unsafe fn recording_mode_impl(hl_id: c_int) {
    if shortmess(SHM_RECORDING) {
        return;
    }
    msg_puts_hl(c"recording".as_ptr(), hl_id, false);
    // " @X" where X is the register character (known to be ASCII 0-127)
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let s = [b' ', b'@', reg_recording as u8, 0u8];
    msg_puts_hl(s.as_ptr().cast::<c_char>(), hl_id, false);
}

/// Clear the mode message.
///
/// Rust equivalent of `clearmode()` in drawscreen.c.
#[unsafe(export_name = "clearmode")]
pub extern "C" fn rs_clearmode() {
    unsafe {
        let save_msg_row = msg_row;
        let save_msg_col = msg_col;

        msg_ext_ui_flush();
        // msg_pos_mode(): set msg_col=0, msg_row=Rows-1
        msg_col = 0;
        msg_row = Rows - 1;

        if reg_recording != 0 {
            recording_mode_impl(HLF_CM);
        }
        msg_clr_eos();
        msg_ext_flush_showmode();

        msg_col = save_msg_col;
        msg_row = save_msg_row;
    }
}

/// Delete mode message.
///
/// Used when ESC is typed which is expected to end Insert mode
/// (but Insert mode didn't end yet!).
/// Rust equivalent of `unshowmode()` in drawscreen.c.
#[unsafe(export_name = "unshowmode")]
pub extern "C" fn rs_unshowmode(force: bool) {
    unsafe {
        // Don't delete it right now, when not redrawing or inside a mapping.
        if !redrawing_impl() || (!force && nvim_char_avail() != 0 && !nvim_get_KeyTyped()) {
            nvim_set_redraw_cmdline(true); // delete mode later
        } else {
            rs_clearmode();
        }
    }
}

// =============================================================================
// Phase 4: Status/Title Redraw Iteration
// =============================================================================

extern "C" {
    fn nvim_win_check_ns_hl(wp: WinHandle) -> bool;
    fn nvim_win_redr_winbar(wp: WinHandle);
    fn nvim_win_redr_status(wp: WinHandle);
    fn nvim_draw_tabline();
    fn nvim_maketitle();
}

/// STL_IN_ICON from globals.h
const STL_IN_ICON: c_int = 1;
/// STL_IN_TITLE from globals.h
const STL_IN_TITLE: c_int = 2;

/// Redraw all status lines that need to be redrawn.
///
/// Rust equivalent of `redraw_statuslines()` in drawscreen.c.
#[unsafe(export_name = "redraw_statuslines")]
pub extern "C" fn rs_redraw_statuslines() {
    unsafe {
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            if nvim_win_get_redr_status(wp) != 0 {
                nvim_win_check_ns_hl(wp);
                nvim_win_redr_winbar(wp);
                nvim_win_redr_status(wp);
            }
            wp = nvim_win_get_next(wp);
        }

        nvim_win_check_ns_hl(WinHandle::null());
        if redraw_tabline {
            nvim_draw_tabline();
        }

        if need_maketitle {
            nvim_maketitle();
        }
    }
}

/// Mark the title and icon for redraw if using statusline format.
///
/// Returns 1 if either title or icon uses statusline format.
/// Rust equivalent of `redraw_custom_title_later()` in drawscreen.c.
#[unsafe(export_name = "redraw_custom_title_later")]
pub extern "C" fn rs_redraw_custom_title_later() -> c_int {
    unsafe {
        if (p_icon != 0 && (stl_syntax & STL_IN_ICON) != 0)
            || (p_title != 0 && (stl_syntax & STL_IN_TITLE) != 0)
        {
            need_maketitle = true;
            return 1;
        }
        0
    }
}

// =============================================================================
// Phase 5: Conceal Check and Cursorline Update
// =============================================================================

/// Rust-owned static for the `conceal_cursor_used` flag (previously file-static in drawscreen.c).
static CONCEAL_CURSOR_USED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Get conceal_cursor_used (Rust-owned static). Called by C code in drawscreen.c.
#[unsafe(export_name = "nvim_get_conceal_cursor_used")]
pub extern "C" fn rs_get_conceal_cursor_used() -> c_int {
    c_int::from(CONCEAL_CURSOR_USED.load(std::sync::atomic::Ordering::Relaxed))
}

/// Set conceal_cursor_used (Rust-owned static). Called by C code in drawscreen.c.
#[unsafe(export_name = "nvim_set_conceal_cursor_used")]
pub extern "C" fn rs_set_conceal_cursor_used(val: c_int) {
    CONCEAL_CURSOR_USED.store(val != 0, std::sync::atomic::Ordering::Relaxed);
}

/// Opaque handle to C's foldinfo_T.
type FoldinfoHandle = *mut c_void;

/// Local mirror of `foldinfo_T` / `FoldInfoResult` from the fold crate.
///
/// `repr(C)` matches fold crate's `FoldInfoResult`.
#[repr(C)]
#[allow(clippy::struct_field_names)]
struct FoldInfo {
    fi_lnum: LinenrT,
    fi_level: c_int,
    fi_low_level: c_int,
    fi_lines: LinenrT,
}

extern "C" {
    #[link_name = "nvim_win_get_p_cole"]
    fn nvim_win_get_w_p_cole(wp: WinHandle) -> c_int;
    #[link_name = "nvim_win_get_p_cul"]
    fn nvim_win_get_w_p_cul(wp: WinHandle) -> c_int;
    fn nvim_win_set_w_cursorline(wp: WinHandle, val: LinenrT);
    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> LinenrT;
    fn nvim_decor_conceal_line(wp: WinHandle, row: c_int, check_cursor: c_int) -> c_int;
    fn changed_window_setting(wp: WinHandle);
    #[link_name = "curs_columns"]
    fn nvim_curs_columns(wp: WinHandle, may_scroll: c_int);
    /// Direct Rust entry: foldinfo for a window+lnum.
    #[link_name = "rs_fold_info"]
    fn nvim_fold_info_rs(wp: WinHandle, lnum: LinenrT) -> FoldInfo;
    // Already-migrated functions in other crates (now exported with C names)
    #[link_name = "conceal_cursor_line"]
    fn rs_conceal_cursor_line(wp: WinHandle) -> bool;
    #[link_name = "win_cursorline_standout"]
    fn rs_win_cursorline_standout(wp: WinHandle) -> bool;
}

/// Check if the cursor line needs to be redrawn because of 'concealcursor'.
///
/// Rust equivalent of `conceal_check_cursor_line()` in drawscreen.c.
#[unsafe(export_name = "conceal_check_cursor_line")]
pub extern "C" fn rs_conceal_check_cursor_line() {
    unsafe {
        let curwin = nvim_get_curwin();
        let should_conceal = rs_conceal_cursor_line(curwin);
        if nvim_win_get_w_p_cole(curwin) <= 0
            || CONCEAL_CURSOR_USED.load(std::sync::atomic::Ordering::Relaxed) == should_conceal
        {
            return;
        }

        let cursor_lnum = nvim_win_get_cursor_lnum(curwin);
        rs_redrawWinline(curwin, cursor_lnum);

        // Concealed line visibility toggled.
        if nvim_decor_conceal_line(curwin, cursor_lnum - 1, 1) != 0 {
            changed_window_setting(curwin);
        }
        // Need to recompute cursor column, e.g., when starting Visual mode
        // without concealing.
        nvim_curs_columns(curwin, 1);
    }
}

/// Update w_cursorline, setting it to the start of a closed fold.
///
/// Rust equivalent of `win_update_cursorline()` in drawscreen.c.
#[unsafe(export_name = "win_update_cursorline")]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn rs_win_update_cursorline(wp: WinHandle, _foldinfo: FoldinfoHandle) {
    unsafe {
        let cursor_lnum = nvim_win_get_cursor_lnum(wp);
        let cursorline = if rs_win_cursorline_standout(wp) {
            cursor_lnum
        } else {
            0
        };
        nvim_win_set_w_cursorline(wp, cursorline);

        if nvim_win_get_w_p_cul(wp) != 0 {
            // Make sure that the cursorline on a closed fold is redrawn
            let fi = nvim_fold_info_rs(wp, cursor_lnum);
            if fi.fi_level != 0 && fi.fi_lines > 0 {
                nvim_win_set_w_cursorline(wp, fi.fi_lnum);
            }
        }
    }
}

// =============================================================================
// Phase 6: Cursor Positioning
// =============================================================================

extern "C" {
    #[link_name = "nvim_win_get_wrow"]
    fn nvim_win_get_w_wrow(wp: WinHandle) -> c_int;
    #[link_name = "nvim_win_get_wcol"]
    fn nvim_win_get_w_wcol(wp: WinHandle) -> c_int;
    #[link_name = "nvim_win_get_p_rl"]
    fn nvim_win_get_w_p_rl(wp: WinHandle) -> c_int;
    fn nvim_win_rl_cursor_col(wp: WinHandle) -> c_int;
    fn nvim_grid_adjust_cursor_goto(wp: WinHandle, row: c_int, col: c_int);
    fn nvim_validate_cursor_for_win(wp: WinHandle);
}

/// Set cursor to its position in the current window.
///
/// Rust equivalent of `setcursor()` in drawscreen.c.
#[unsafe(export_name = "setcursor")]
pub extern "C" fn rs_setcursor() {
    unsafe {
        rs_setcursor_mayforce(nvim_get_curwin(), false);
    }
}

/// Set cursor to its position in a window.
///
/// Rust equivalent of `setcursor_mayforce()` in drawscreen.c.
#[unsafe(export_name = "setcursor_mayforce")]
pub extern "C" fn rs_setcursor_mayforce(wp: WinHandle, force: bool) {
    unsafe {
        if force || redrawing_impl() {
            nvim_validate_cursor_for_win(wp);

            let row = nvim_win_get_w_wrow(wp);
            let col = if nvim_win_get_w_p_rl(wp) != 0 {
                // With 'rightleft' set and the cursor on a double-wide character,
                // position it on the leftmost column.
                nvim_win_rl_cursor_col(wp)
            } else {
                nvim_win_get_w_wcol(wp)
            };

            nvim_grid_adjust_cursor_goto(wp, row, col);
        }
    }
}

// =============================================================================
// Phase 7: Search Highlight Bookkeeping
// =============================================================================

extern "C" {
    fn nvim_get_p_hls() -> c_int;
    fn nvim_get_no_hlsearch() -> c_int;
    fn nvim_search_hl_has_regprog() -> c_int;
    fn nvim_search_hl_start();
    fn nvim_search_hl_end();
}

/// Prepare for 'hlsearch' highlighting.
///
/// Rust equivalent of `start_search_hl()` in drawscreen.c.
#[unsafe(export_name = "start_search_hl")]
pub extern "C" fn rs_start_search_hl() {
    unsafe {
        if nvim_get_p_hls() == 0 || nvim_get_no_hlsearch() != 0 {
            return;
        }

        rs_end_search_hl(); // just in case it wasn't called before
        nvim_search_hl_start();
    }
}

/// Clean up for 'hlsearch' highlighting.
///
/// Rust equivalent of `end_search_hl()` in drawscreen.c.
#[unsafe(export_name = "end_search_hl")]
pub extern "C" fn rs_end_search_hl() {
    unsafe {
        if nvim_search_hl_has_regprog() == 0 {
            return;
        }

        nvim_search_hl_end();
    }
}

// =============================================================================
// Phase 4 (win_draw_end): Empty line drawing
// =============================================================================

extern "C" {
    #[link_name = "nvim_win_get_view_width"]
    fn nvim_win_get_w_view_width(wp: WinHandle) -> c_int;
    #[link_name = "nvim_win_get_scwidth"]
    fn nvim_win_get_w_scwidth(wp: WinHandle) -> c_int;
    #[link_name = "nvim_win_get_p_nu"]
    fn nvim_win_get_w_p_nu(wp: WinHandle) -> c_int;
    #[link_name = "nvim_win_get_p_rnu"]
    fn nvim_win_get_w_p_rnu(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_grid(wp: WinHandle) -> GridViewHandle;
    fn win_bg_attr(wp: WinHandle) -> c_int;
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;
    #[link_name = "rs_grid_line_clear_end"]
    fn grid_line_clear_end(n: c_int, end: c_int, bg_attr: c_int, attr: c_int);
    #[link_name = "rs_grid_line_mirror"]
    fn grid_line_mirror(width: c_int);
    #[link_name = "compute_foldcolumn"]
    fn rs_compute_foldcolumn(wp: WinHandle, extra: c_int) -> c_int;
    #[link_name = "number_width"]
    fn rs_number_width(wp: WinHandle) -> c_int;
}

// p_cpo (cpoptions) global.
extern "C" {
    static p_cpo: *const c_char;
}

/// CPO_NUMCOL: 'number' column also used for text (== 'n').
const CPO_NUMCOL: c_int = b'n' as c_int;
/// HLF_FC: FoldColumn highlight.
const HLF_FC: c_int = 29;
/// HLF_SC: SignColumn highlight.
const HLF_SC: c_int = 35;
/// HLF_N: LineNr highlight.
const HLF_N: c_int = 12;
/// SIGN_WIDTH: display cells per sign.
const SIGN_WIDTH: c_int = 2;
/// Space schar (schar_from_ascii(' ') on little-endian).
const SCHAR_SPACE: ScharT = b' ' as ScharT;

/// Draw empty lines at the bottom of a window, marking unused lines with `c1`.
/// When `draw_margin` is true, draw sign/fold/number columns.
///
/// Rust equivalent of `win_draw_end()` in drawscreen.c.
#[unsafe(export_name = "win_draw_end")]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
pub extern "C" fn rs_win_draw_end(
    wp: WinHandle,
    c1: ScharT,
    draw_margin: bool,
    startrow: c_int,
    endrow: c_int,
    hl: c_int,
) {
    unsafe {
        let view_width = nvim_win_get_w_view_width(wp);
        let fdc = rs_compute_foldcolumn(wp, 0);
        let scwidth = nvim_win_get_w_scwidth(wp);
        let grid = nvim_win_get_w_grid(wp);

        for row in startrow..endrow {
            rs_grid_line_start(grid, row);

            let mut n: c_int = 0;
            if draw_margin {
                // draw the fold column
                if fdc > 0 {
                    n = rs_grid_line_fill(
                        n,
                        (view_width).min(n + fdc),
                        SCHAR_SPACE,
                        nvim_win_hl_attr(wp, HLF_FC),
                    );
                }

                // draw the sign column
                if scwidth > 0 {
                    n = rs_grid_line_fill(
                        n,
                        (view_width).min(n + scwidth * SIGN_WIDTH),
                        SCHAR_SPACE,
                        nvim_win_hl_attr(wp, HLF_SC),
                    );
                }

                // draw the number column
                if (nvim_win_get_w_p_nu(wp) != 0 || nvim_win_get_w_p_rnu(wp) != 0)
                    && vim_strchr(p_cpo, CPO_NUMCOL).is_null()
                {
                    let width = rs_number_width(wp) + 1;
                    n = rs_grid_line_fill(
                        n,
                        (view_width).min(n + width),
                        SCHAR_SPACE,
                        nvim_win_hl_attr(wp, HLF_N),
                    );
                }
            }

            let attr = nvim_win_hl_attr(wp, hl);

            if n < view_width {
                rs_grid_line_put_schar(n, c1, attr);
                n += 1;
            }

            grid_line_clear_end(n, view_width, win_bg_attr(wp), attr);

            if nvim_win_get_w_p_rl(wp) != 0 {
                grid_line_mirror(view_width);
            }
            rs_grid_line_flush();
        }
    }
}

// =============================================================================
// Phase 3: default_grid_alloc, screenclear, screen_resize
// =============================================================================

/// MODE_HITRETURN from state_defs.h (0x2000 | MODE_NORMAL).
const MODE_HITRETURN: c_int = 0x2001;
/// MODE_SETWSIZE from state_defs.h.
const MODE_SETWSIZE: c_int = 0x4000;
/// MODE_ASKMORE from state_defs.h.
const MODE_ASKMORE: c_int = 0x3000;
/// MODE_EXTERNCMD from state_defs.h.
const MODE_EXTERNCMD: c_int = 0x5000;

extern "C" {
    /// Check if default_grid needs reallocation (size mismatch or NULL).
    fn nvim_default_grid_needs_alloc() -> c_int;
    /// Perform the actual default_grid allocation and field setup.
    fn nvim_default_grid_do_alloc();
    /// Full screenclear implementation (C helper, preserves static state).
    fn nvim_screenclear_impl();

    /// resizing_screen: true while screen_resize is running.
    static mut resizing_screen: bool;
    /// exmode_active: true in Ex mode.
    static exmode_active: c_int;
    /// starting: startup phase.
    static starting: c_int;
    /// p_lines: option 'lines'.
    static mut p_lines: c_int;
    /// p_columns: option 'columns'.
    static mut p_columns: c_int;

    /// Resize the UI grid.
    fn ui_call_grid_resize(grid: c_int, width: c_int, height: c_int);
    /// Fit windows in new sized screen.
    fn win_new_screensize();
    /// Apply autocommands.
    fn apply_autocmds(
        event: c_int,
        fname: *const c_char,
        fname_io: *const c_char,
        force: bool,
        buf: *mut c_void,
    ) -> bool;
    /// Invalidate the bottom line of a window.
    fn invalidate_botline(wp: WinHandle);
    /// Mark that a line above cursor changed.
    fn changed_line_abv_curs();
    /// Get current buffer (curbuf).
    fn nvim_get_curbuf_ptr() -> *mut c_void;

    /// Clamp cmdheight for all tabs; set p_lines/p_columns.
    fn nvim_screen_resize_clamp_cmdheight();
    /// Handle post-resize redraw for starting != NO_SCREEN.
    fn nvim_screen_resize_post(state: c_int);
    /// Return true if msg_grid.chars is non-NULL.
    fn nvim_msg_grid_has_chars() -> bool;
}

/// EVENT_VIMRESIZED autocmd event number (from auevents_enum.generated.h).
const EVENT_VIMRESIZED: c_int = 132;
/// NO_SCREEN: startup not yet showing a screen.
const NO_SCREEN: c_int = 0;

/// Resize the default screen grid to Rows and Columns.
///
/// Includes the reentrancy guard (previously in the C `default_grid_alloc` wrapper).
/// Returns true if resizing was performed.
///
/// Exported as both `rs_default_grid_alloc` and `default_grid_alloc` for C callers.
///
/// Rust equivalent of `default_grid_alloc()` in drawscreen.c.
#[unsafe(export_name = "default_grid_alloc")]
pub unsafe extern "C" fn rs_default_grid_alloc() -> bool {
    static RESIZING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    use std::sync::atomic::Ordering;

    // Reentrancy guard (equivalent to C static bool resizing).
    if RESIZING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return false;
    }

    let result = if nvim_default_grid_needs_alloc() == 0 {
        false
    } else {
        nvim_default_grid_do_alloc();
        true
    };

    RESIZING.store(false, Ordering::SeqCst);
    result
}

/// Clear entire screen and reset state.
///
/// Rust equivalent of `screenclear()` in drawscreen.c.
#[unsafe(export_name = "screenclear")]
pub unsafe extern "C" fn rs_screenclear() {
    nvim_screenclear_impl();
}

/// Set dimensions of the Nvim application "screen".
///
/// Contains full orchestration logic; struct-heavy sections are delegated to
/// C behavioral helpers in drawscreen_shim.c.
///
/// Rust equivalent of `screen_resize()` in drawscreen.c.
#[unsafe(export_name = "screen_resize")]
pub unsafe extern "C" fn rs_drawscreen_screen_resize(width: c_int, height: c_int) {
    // Avoid recursiveness.
    if updating_screen || resizing_screen || rs_cmdline_number_prompt() != 0 {
        return;
    }

    if width < 0 || height < 0 {
        return;
    }

    if State == MODE_HITRETURN || State == MODE_SETWSIZE {
        State = MODE_SETWSIZE;
        return;
    }

    resizing_screen = true;

    Rows = height;
    Columns = width;
    // check_screensize is exported from Rust with export_name; call via FFI name
    rs_check_screensize();

    // Clamp cmdheight for all tabs; set p_lines/p_columns (struct-heavy, C helper).
    nvim_screen_resize_clamp_cmdheight();

    // Re-read Rows/Columns after check_screensize may have clamped them.
    let width = Columns;
    let height = Rows;
    ui_call_grid_resize(1, width, height);

    let mut retry_count: c_int = 0;
    resizing_autocmd = true;

    // Retry loop: autocommands may alter Rows or Columns.
    while rs_default_grid_alloc() {
        ui_comp_set_screen_valid(false);
        if nvim_msg_grid_has_chars() {
            msg_grid_invalid = true;
        }

        RedrawingDisabled += 1;
        win_new_screensize();
        rs_comp_col();
        RedrawingDisabled -= 1;

        retry_count += 1;
        if retry_count > 3 {
            break;
        }

        apply_autocmds(
            EVENT_VIMRESIZED,
            std::ptr::null(),
            std::ptr::null(),
            false,
            nvim_get_curbuf_ptr(),
        );
    }

    resizing_autocmd = false;
    redraw_all_later_impl(UPD_CLEAR);

    if State != MODE_ASKMORE && State != MODE_EXTERNCMD {
        rs_screenclear();
    }

    // Post-resize redraw (struct-heavy, C helper).
    nvim_screen_resize_post(State);

    resizing_screen = false;
}

// =============================================================================
// Phase 1: win_scroll_lines
// =============================================================================

extern "C" {
    /// C's grid_adjust: adjusts row/col offsets and returns target ScreenGrid pointer.
    fn grid_adjust(view: *mut c_void, row_off: *mut c_int, col_off: *mut c_int) -> *mut c_void;
    /// C's grid_ins_lines: insert lines into a grid.
    fn grid_ins_lines(
        grid: *mut c_void,
        row: c_int,
        line_count: c_int,
        end: c_int,
        col: c_int,
        width: c_int,
    );
    /// C's grid_del_lines: delete lines from a grid.
    fn grid_del_lines(
        grid: *mut c_void,
        row: c_int,
        line_count: c_int,
        end: c_int,
        col: c_int,
        width: c_int,
    );
    /// Get rows from a ScreenGrid pointer.
    fn nvim_screengrid_get_rows(grid: *mut c_void) -> c_int;
    /// Get cols from a ScreenGrid pointer.
    fn nvim_screengrid_get_cols(grid: *mut c_void) -> c_int;
}

/// Scroll lines in window `wp` grid by `line_count` (positive = scroll up, negative = scroll down).
///
/// Rust equivalent of `win_scroll_lines()` in drawscreen.c.
#[unsafe(export_name = "win_scroll_lines")]
pub unsafe extern "C" fn rs_win_scroll_lines(wp: WinHandle, row: c_int, line_count: c_int) {
    if !redrawing_impl() || line_count == 0 {
        return;
    }

    let mut col: c_int = 0;
    let mut row_off: c_int = 0;
    let grid_view = nvim_win_get_w_grid(wp);
    let grid = grid_adjust(grid_view, &raw mut row_off, &raw mut col);

    let view_width = nvim_win_get_view_width(wp);
    let view_height = nvim_win_get_view_height(wp);

    // Get actual grid dimensions via the adjusted grid pointer
    // We need grid->cols and grid->rows for the bounds check
    // Use the existing accessors that work on ScreenGrid pointers
    let grid_rows = nvim_screengrid_get_rows(grid);
    let grid_cols = nvim_screengrid_get_cols(grid);

    let checked_width = (grid_cols - col).min(view_width);
    let checked_height = (grid_rows - row_off).min(view_height);

    // No lines are being moved, just draw over the entire area
    if row + line_count.abs() >= checked_height {
        return;
    }

    if line_count < 0 {
        grid_del_lines(
            grid,
            row + row_off,
            -line_count,
            checked_height + row_off,
            col,
            checked_width,
        );
    } else {
        grid_ins_lines(
            grid,
            row + row_off,
            line_count,
            checked_height + row_off,
            col,
            checked_width,
        );
    }
}

// =============================================================================
// Phase 2: showmode
// =============================================================================

// Mode state bit flags (from state_defs.h)
const MODE_INSERT: c_int = 0x10;
const MODE_LANGMAP: c_int = 0x20;
const MODE_TERMINAL: c_int = 0x80;
const REPLACE_FLAG: c_int = 0x100;
const VREPLACE_FLAG: c_int = 0x200;

/// CTRL-V character value.
const CTRL_V: c_int = 0x16;
/// SHM_COMPLETIONMENU: shortmess('c').
const SHM_COMPLETIONMENU: c_int = b'c' as c_int;
/// HLF_COUNT: number of highlight groups (verified by static_assert in C).
const HLF_COUNT: c_int = 76;

extern "C" {
    // nvim_get_p_smd: inlined (Phase 39, use p_smd directly)
    #[link_name = "p_smd"]
    static p_smd: c_int;
    static mut restart_edit: c_int;
    fn nvim_VIsual_active() -> c_int;
    static mut p_ch: i64;
    // ui_has is already declared in the Phase 1 extern block above.
    static mut need_wait_return: bool;
    fn msg_check_for_delay(check_msg_scroll: c_int);
    fn nvim_get_clear_cmdline() -> bool;
    fn nvim_set_clear_cmdline(val: bool);
    static mut cmdline_row: c_int;
    fn msg_clr_cmdline();
    static mut msg_no_more: bool;
    fn nvim_set_msg_no_more(val: c_int);
    static mut lines_left: c_int;
    // nvim_drawscreen_edit_submode_*: inlined (Phase 38, use extern statics directly)
    #[link_name = "edit_submode"]
    static mut g_edit_submode: *mut c_char;
    #[link_name = "edit_submode_pre"]
    static mut g_edit_submode_pre: *mut c_char;
    // nvim_get_edit_submode_highl_attr: inlined below (Phase 36)
    #[link_name = "edit_submode_highl"]
    static mut g_edit_submode_highl: c_int;
    fn vim_strsize(s: *const c_char) -> c_int;
    fn nvim_get_p_ri() -> c_int;
    fn nvim_buf_get_terminal(buf: BufHandle) -> c_int;
    fn nvim_get_VIsual_select() -> bool;
    fn nvim_get_p_paste() -> c_int;
    fn get_keymap_str(wp: WinHandle, fmt: *const c_char, buf: *mut c_char, len: c_int) -> c_int;
    static mut NameBuff: [c_char; 4096];
    fn nvim_get_mode_displayed() -> bool;
    fn nvim_set_mode_displayed(val: bool);
    static mut msg_didout: bool;
    fn redraw_ruler();
    fn msg_grid_validate();
    /// edit_submode_extra global (from globals.h).
    #[link_name = "edit_submode_extra"]
    static mut g_edit_submode_extra: *mut c_char;
    /// edit_submode_extra pointer (from insexpand_shim.c).
    // nvim_get_edit_submode_extra_ptr: inlined (Phase 37, use g_edit_submode_extra directly)
    /// w_p_arab window option accessor (from window crate).
    fn nvim_win_get_p_arab(wp: WinHandle) -> c_int;
    /// Clear the showcmd area.
    fn rs_clear_showcmd();
}

/// Inline equivalent of C's static `msg_pos_mode()`: set msg_col=0, msg_row=Rows-1.
#[inline]
unsafe fn msg_pos_mode() {
    msg_col = 0;
    msg_row = Rows - 1;
}

/// Display the mode text (insert, replace, visual, etc.) into the command line.
///
/// Called from `rs_showmode` when `do_mode` is true. Returns whether to clear
/// the command line after displaying.
///
/// # Safety
/// Caller must be inside `rs_showmode` with all globals valid.
unsafe fn showmode_display_mode(hl_id: c_int, length: &mut c_int) {
    msg_puts_hl(c"--".as_ptr(), hl_id, false);
    // CTRL-X in Insert mode
    if !g_edit_submode.is_null() && !shortmess(SHM_COMPLETIONMENU) {
        // Messages can get long; avoid a wrap in a narrow window.
        if ui_has(K_UI_MESSAGES) {
            *length = c_int::MAX;
        } else {
            *length = (Rows - msg_row) * Columns - 3;
        }
        if !g_edit_submode_extra.is_null() {
            *length -= vim_strsize(g_edit_submode_extra);
        }
        if *length > 0 {
            if !g_edit_submode_pre.is_null() {
                *length -= vim_strsize(g_edit_submode_pre);
            }
            let submode = g_edit_submode;
            if *length - vim_strsize(submode) > 0 {
                if !g_edit_submode_pre.is_null() {
                    msg_puts_hl(g_edit_submode_pre, hl_id, false);
                }
                msg_puts_hl(submode, hl_id, false);
            }
            if !g_edit_submode_extra.is_null() {
                msg_puts_hl(c" ".as_ptr(), hl_id, false);
                // edit_submode_highl+1 if valid (< HLF_COUNT), else 0
                let highl_attr = if g_edit_submode_highl < HLF_COUNT {
                    g_edit_submode_highl + 1
                } else {
                    0
                };
                let sub_id = if highl_attr > 0 {
                    highl_attr - 1
                } else {
                    hl_id
                };
                msg_puts_hl(g_edit_submode_extra, sub_id, false);
            }
        }
    } else {
        let state = State;
        if (state & MODE_TERMINAL) != 0 {
            msg_puts_hl(c" TERMINAL".as_ptr(), hl_id, false);
        } else if (state & VREPLACE_FLAG) != 0 {
            msg_puts_hl(c" VREPLACE".as_ptr(), hl_id, false);
        } else if (state & REPLACE_FLAG) != 0 {
            msg_puts_hl(c" REPLACE".as_ptr(), hl_id, false);
        } else if (state & MODE_INSERT) != 0 {
            if nvim_get_p_ri() != 0 {
                msg_puts_hl(c" REVERSE".as_ptr(), hl_id, false);
            }
            msg_puts_hl(c" INSERT".as_ptr(), hl_id, false);
        } else {
            let re = restart_edit;
            if re == c_int::from(b'I')
                || re == c_int::from(b'i')
                || re == c_int::from(b'a')
                || re == c_int::from(b'A')
            {
                if nvim_buf_get_terminal(nvim_get_curbuf()) != 0 {
                    msg_puts_hl(c" (terminal)".as_ptr(), hl_id, false);
                } else {
                    msg_puts_hl(c" (insert)".as_ptr(), hl_id, false);
                }
            } else if re == c_int::from(b'R') {
                msg_puts_hl(c" (replace)".as_ptr(), hl_id, false);
            } else if re == c_int::from(b'V') {
                msg_puts_hl(c" (vreplace)".as_ptr(), hl_id, false);
            }
        }
        if (state & MODE_LANGMAP) != 0 {
            if nvim_win_get_p_arab(nvim_get_curwin()) != 0 {
                msg_puts_hl(c" Arabic".as_ptr(), hl_id, false);
            } else if get_keymap_str(
                nvim_get_curwin(),
                c" (%s)".as_ptr(),
                addr_of_mut!(NameBuff).cast::<c_char>(),
                4096,
            ) > 0
            {
                msg_puts_hl(addr_of!(NameBuff).cast::<c_char>(), hl_id, false);
            }
        }
        if (state & MODE_INSERT) != 0 && nvim_get_p_paste() != 0 {
            msg_puts_hl(c" (paste)".as_ptr(), hl_id, false);
        }
        if nvim_VIsual_active() != 0 {
            let vm = VIsual_mode;
            let sel = c_int::from(nvim_get_VIsual_select());
            let key = c_int::from(vm == CTRL_V) * 2 + c_int::from(vm == c_int::from(b'V'));
            let p = match sel * 4 + key {
                0 => c" VISUAL".as_ptr(),
                1 => c" VISUAL LINE".as_ptr(),
                2 => c" VISUAL BLOCK".as_ptr(),
                4 => c" SELECT".as_ptr(),
                5 => c" SELECT LINE".as_ptr(),
                _ => c" SELECT BLOCK".as_ptr(),
            };
            msg_puts_hl(p, hl_id, false);
        }
        msg_puts_hl(c" --".as_ptr(), hl_id, false);
    }
}

/// Show the current mode and ruler.
///
/// If clear_cmdline is true, clear the rest of the cmdline.
/// If clear_cmdline is false there may be a message there that needs to be
/// cleared only if a mode is shown.
/// If redraw_mode is true show or clear the mode.
/// Returns the length of the message (0 if no message).
///
/// Rust equivalent of `showmode()` in drawscreen.c.
#[unsafe(export_name = "showmode")]
pub unsafe extern "C" fn rs_showmode() -> c_int {
    let mut length: c_int = 0;

    msg_ext_ui_flush();
    msg_grid_validate();

    let state = State;
    let do_mode = (p_smd != 0 && msg_silent == 0)
        && ((state & MODE_TERMINAL) != 0
            || (state & MODE_INSERT) != 0
            || restart_edit != 0
            || nvim_VIsual_active() != 0);

    let can_show_mode = p_ch != 0 || ui_has(K_UI_MESSAGES);

    if (do_mode || reg_recording != 0) && can_show_mode {
        if rs_skip_showmode() {
            return 0;
        }

        let nwr_save = need_wait_return;
        msg_check_for_delay(0);

        let mut need_clear = nvim_get_clear_cmdline();
        if nvim_get_clear_cmdline() && cmdline_row < Rows - 1 {
            msg_clr_cmdline();
        }

        msg_pos_mode();
        let hl_id = HLF_CM;

        nvim_set_msg_no_more(1);
        let save_lines_left = lines_left;
        lines_left = 0;

        if do_mode {
            showmode_display_mode(hl_id, &mut length);
            need_clear = true;
        }

        if reg_recording != 0 && g_edit_submode.is_null() {
            recording_mode_impl(hl_id);
            need_clear = true;
        }

        nvim_set_mode_displayed(true);
        if need_clear || nvim_get_clear_cmdline() || redraw_mode != 0 {
            msg_clr_eos();
        }
        msg_didout = false;
        length = msg_col;
        msg_col = 0;
        nvim_set_msg_no_more(0);
        lines_left = save_lines_left;
        need_wait_return = nwr_save;
    } else if nvim_get_clear_cmdline() && msg_silent == 0 {
        msg_clr_cmdline();
    } else if redraw_mode != 0 {
        msg_pos_mode();
        msg_clr_eos();
    }

    msg_ext_flush_showmode();

    if nvim_VIsual_active() != 0 {
        rs_clear_showcmd();
    }

    redraw_ruler();
    nvim_set_redraw_cmdline(false);
    redraw_mode = 0;
    nvim_set_clear_cmdline(false);

    length
}

// =============================================================================
// Phase 6 (plan eec0896b): migrate nvim_win_visual_region_impl to Rust
// =============================================================================

/// VALID_BOTLINE flag: w_botline and w_empty_rows are valid.
const VALID_BOTLINE: c_int = 0x20;

/// MAXCOL: maximal column number.
const MAXCOL: c_int = 0x7fff_ffff;

extern "C" {
    /// Get wp->w_old_cursor_lnum.
    fn nvim_win_get_old_cursor_lnum(wp: WinHandle) -> LinenrT;
    /// Get wp->w_old_visual_lnum.
    fn nvim_win_get_old_visual_lnum(wp: WinHandle) -> LinenrT;
    /// Get wp->w_old_visual_col.
    fn nvim_win_get_old_visual_col(wp: WinHandle) -> ColnrT;
    /// Get wp->w_old_cursor_fcol.
    fn nvim_win_get_old_cursor_fcol(wp: WinHandle) -> ColnrT;
    /// Get wp->w_old_cursor_lcol.
    fn nvim_win_get_old_cursor_lcol(wp: WinHandle) -> ColnrT;
    /// Get wp->w_old_curswant.
    fn nvim_win_get_old_curswant(wp: WinHandle) -> ColnrT;
    /// Set wp->w_old_cursor_fcol.
    fn nvim_win_set_old_cursor_fcol(wp: WinHandle, val: ColnrT);
    /// Set wp->w_old_cursor_lcol.
    fn nvim_win_set_old_cursor_lcol(wp: WinHandle, val: ColnrT);
    /// Set wp->w_old_curswant.
    fn nvim_win_set_old_curswant(wp: WinHandle, val: ColnrT);

    /// Get wp->w_valid flags.
    fn nvim_win_get_valid(wp: WinHandle) -> c_int;
    /// Get wp->w_lines[idx].wl_size.
    fn nvim_win_wlines_get_size(wp: WinHandle, idx: c_int) -> c_int;
    /// Get wp->w_lines[idx].wl_valid.
    fn nvim_win_get_lines_wl_valid(wp: WinHandle, idx: c_int) -> bool;
    /// Get wp->w_lines[idx].wl_lnum.
    fn nvim_win_get_lines_wl_lnum(wp: WinHandle, idx: c_int) -> LinenrT;

    /// Get curwin->w_p_lbr.
    fn nvim_curwin_get_w_p_lbr() -> c_int;
    /// Get curwin->w_curswant.
    fn nvim_curwin_get_curswant() -> ColnrT;
    /// Get curwin->w_ve_flags.
    fn nvim_curwin_get_w_ve_flags() -> u32;
    /// Set curwin->w_ve_flags.
    fn nvim_curwin_set_w_ve_flags(val: u32);
    /// Get curwin->w_cursor.lnum.
    fn nvim_curwin_get_cursor_lnum() -> LinenrT;
    /// Return true if buf == curwin->w_buffer.
    fn nvim_buf_is_curwin_buf(buf: BufHandle) -> bool;

    /// Get VIsual.lnum.
    fn nvim_get_VIsual_lnum() -> c_int;
    /// Get VIsual.col.
    fn nvim_get_VIsual_col() -> c_int;

    /// Compute block-visual column range: getvcols(&VIsual, &curwin->w_cursor),
    /// handles MAXCOL curswant, returns fromc and toc (post-increment).
    fn nvim_win_visual_block_cols(wp: WinHandle, fromc: *mut ColnrT, toc: *mut ColnrT);
}

/// Visual mode region update section extracted from `win_update()`.
///
/// Computes which screen rows need redrawing due to Visual selection changes,
/// updates mid_start/mid_end, and saves old visual state into window fields.
///
/// Rust equivalent of `nvim_win_visual_region_impl()` in drawscreen.c.
///
/// # Safety
/// Must be called from within the win_update() context with valid wp/buf.
#[allow(clippy::too_many_lines)]
unsafe fn win_update_visual_region_impl(
    wp: WinHandle,
    buf: BufHandle,
    type_: c_int,
    top_end: c_int,
    scrolled_down: bool,
    mid_start: *mut c_int,
    mid_end: *mut c_int,
) {
    let visual_active = VIsual_active;
    let buf_is_curwin = nvim_buf_is_curwin_buf(buf);
    let old_cursor_lnum = nvim_win_get_old_cursor_lnum(wp);

    // check if we are updating or removing the inverted part
    if (visual_active && buf_is_curwin) || (old_cursor_lnum != 0 && type_ != UPD_NOT_VALID) {
        let old_visual_lnum = nvim_win_get_old_visual_lnum(wp);
        let old_visual_col = nvim_win_get_old_visual_col(wp);

        let (from, to) = if visual_active {
            let cursor_lnum = nvim_curwin_get_cursor_lnum();
            let visual_lnum = nvim_get_VIsual_lnum();
            let visual_mode = VIsual_mode;
            let old_visual_mode = nvim_win_get_old_visual_mode(wp);

            let (from, to) = if visual_mode != old_visual_mode || type_ == UPD_INVERTED_ALL {
                let (f, t) = if cursor_lnum < visual_lnum {
                    (cursor_lnum, visual_lnum)
                } else {
                    (visual_lnum, cursor_lnum)
                };
                let f = f.min(old_cursor_lnum).min(old_visual_lnum);
                let t = t.max(old_cursor_lnum).max(old_visual_lnum);
                (f, t)
            } else {
                let (mut f, mut t) = if cursor_lnum < old_cursor_lnum {
                    (cursor_lnum, old_cursor_lnum)
                } else {
                    let f2 = old_cursor_lnum;
                    let t2 = cursor_lnum;
                    let f2 = if f2 == 0 { t2 } else { f2 };
                    (f2, t2)
                };

                let visual_col = nvim_get_VIsual_col();
                if visual_lnum != old_visual_lnum || visual_col != old_visual_col {
                    if old_visual_lnum < f && old_visual_lnum != 0 {
                        f = old_visual_lnum;
                    }
                    t = t.max(old_visual_lnum).max(visual_lnum);
                    f = f.min(visual_lnum);
                }
                (f, t)
            };

            // Handle block visual column tracking
            let ctrl_v: c_int = 0x16; // Ctrl-V character
            let (from, to) = if visual_mode == ctrl_v {
                let mut fromc: ColnrT = 0;
                let mut toc: ColnrT = 0;
                nvim_win_visual_block_cols(wp, &raw mut fromc, &raw mut toc);

                let prev_first_col = nvim_win_get_old_cursor_fcol(wp);
                let prev_last_col = nvim_win_get_old_cursor_lcol(wp);
                let (f, t) = if fromc != prev_first_col || toc != prev_last_col {
                    let visual_lnum2 = nvim_get_VIsual_lnum();
                    (from.min(visual_lnum2), to.max(visual_lnum2))
                } else {
                    (from, to)
                };
                nvim_win_set_old_cursor_fcol(wp, fromc);
                nvim_win_set_old_cursor_lcol(wp, toc);
                (f, t)
            } else {
                (from, to)
            };

            (from, to)
        } else {
            // Visual mode no longer active: use old saved lnums
            let (f, t) = if old_cursor_lnum < old_visual_lnum {
                (old_cursor_lnum, old_visual_lnum)
            } else {
                (old_visual_lnum, old_cursor_lnum)
            };
            (f, t)
        };

        let topline = nvim_win_get_topline(wp);
        let from = from.max(topline);

        let (from, to) = if (nvim_win_get_valid(wp) & VALID_BOTLINE) != 0 {
            let botline = nvim_win_get_botline(wp);
            (from.min(botline - 1), to.min(botline - 1))
        } else {
            (from, to)
        };

        let ms = *mid_start;
        if ms > 0 {
            let mut lnum = nvim_win_get_topline(wp);
            let mut idx: c_int = 0;
            let mut srow: c_int = 0;
            let lines_valid = nvim_win_get_lines_valid(wp);

            if scrolled_down {
                *mid_start = top_end;
            } else {
                *mid_start = 0;
            }

            while lnum < from && idx < lines_valid {
                let wl_valid = nvim_win_get_lines_wl_valid(wp, idx);
                let wl_size = nvim_win_wlines_get_size(wp, idx);
                if wl_valid {
                    *mid_start += wl_size;
                } else if !scrolled_down {
                    srow += wl_size;
                }
                idx += 1;
                if idx < lines_valid && nvim_win_get_lines_wl_valid(wp, idx) {
                    lnum = nvim_win_get_lines_wl_lnum(wp, idx);
                } else {
                    lnum += 1;
                }
            }

            srow += *mid_start;
            *mid_end = nvim_win_get_view_height(wp);

            while idx < lines_valid {
                let wl_valid = nvim_win_get_lines_wl_valid(wp, idx);
                let wl_lnum = nvim_win_get_lines_wl_lnum(wp, idx);
                if wl_valid && wl_lnum > to {
                    *mid_end = srow;
                    break;
                }
                srow += nvim_win_wlines_get_size(wp, idx);
                idx += 1;
            }
        }
    }

    // Save visual state for next iteration
    if visual_active && nvim_buf_is_curwin_buf(buf) {
        let cursor_lnum = nvim_curwin_get_cursor_lnum();
        let visual_lnum = nvim_get_VIsual_lnum();
        let visual_col = nvim_get_VIsual_col();
        let curswant = nvim_curwin_get_curswant();
        nvim_win_set_old_visual_mode(wp, VIsual_mode);
        nvim_win_set_old_cursor_lnum(wp, cursor_lnum);
        nvim_win_set_old_visual_lnum(wp, visual_lnum);
        nvim_win_set_old_visual_col(wp, visual_col);
        nvim_win_set_old_curswant(wp, curswant);
    } else {
        nvim_win_set_old_visual_mode(wp, 0);
        nvim_win_set_old_cursor_lnum(wp, 0);
        nvim_win_set_old_visual_lnum(wp, 0);
        nvim_win_set_old_visual_col(wp, 0);
    }
}

/// Visual mode region update. Replaces C `nvim_win_visual_region_impl`.
///
/// # Safety
/// Must be called from within the win_update() context with valid wp/buf.
#[no_mangle]
pub unsafe extern "C" fn rs_win_update_visual_region(
    wp: WinHandle,
    buf: BufHandle,
    type_: c_int,
    top_end: c_int,
    scrolled_down: bool,
    mid_start: *mut c_int,
    mid_end: *mut c_int,
) {
    win_update_visual_region_impl(wp, buf, type_, top_end, scrolled_down, mid_start, mid_end);
}

// =============================================================================
// Phase 1 (plan eec0896b): win_loop migration
// =============================================================================

extern "C" {
    /// Non-static wrapper for win_update() in drawscreen.c.
    fn nvim_win_update(wp: WinHandle);

    /// update_window_hl: update highlight attributes for a window.
    fn update_window_hl(wp: WinHandle, invalid: bool);

    /// syntax_present: returns true if syntax highlighting is active for wp.
    fn syntax_present(wp: WinHandle) -> bool;

    /// syn_stack_apply_changes: apply syntax stack changes for buf.
    fn syn_stack_apply_changes(buf: BufHandle);

    /// decor_providers_invoke_buf: invoke decoration providers for buf.
    fn decor_providers_invoke_buf(buf: BufHandle);

    /// win_grid_alloc: allocate/resize the window grid.
    fn win_grid_alloc(wp: WinHandle);

    /// pum_drawn: returns true if popup menu is currently drawn.
    fn pum_drawn() -> bool;

    /// pum_redraw: redraw the popup menu.
    fn pum_redraw();

    /// nvim_win_draw_border: call grid_draw_border for wp's grid_alloc + config.
    fn nvim_win_draw_border(wp: WinHandle);

    /// nvim_win_grid_alloc_invalidate: call grid_invalidate(&wp->w_grid_alloc).
    fn nvim_win_grid_alloc_invalidate(wp: WinHandle);

    /// nvim_win_get_grid_alloc_chars: returns true if wp->w_grid_alloc.chars is non-NULL.
    fn nvim_win_get_grid_alloc_chars(wp: WinHandle) -> bool;

    /// nvim_buf_get_mod_tick_syn: get buf->b_mod_tick_syn.
    fn nvim_buf_get_mod_tick_syn(buf: BufHandle) -> u64;
    /// nvim_buf_set_mod_tick_syn: set buf->b_mod_tick_syn.
    fn nvim_buf_set_mod_tick_syn(buf: BufHandle, val: u64);
    /// nvim_buf_get_mod_tick_decor: get buf->b_mod_tick_decor.
    fn nvim_buf_get_mod_tick_decor(buf: BufHandle) -> u64;
    /// nvim_buf_set_mod_tick_decor: set buf->b_mod_tick_decor.
    fn nvim_buf_set_mod_tick_decor(buf: BufHandle, val: u64);

}

/// Handle the three window iteration loops of update_screen.
///
/// Rust equivalent of `nvim_update_screen_win_loop()` in drawscreen.c.
/// Called from `rs_update_screen()`.
///
/// # Safety
/// Must be called from within the update_screen() context with screen/windows valid.
unsafe fn update_screen_win_loop_impl(type_: c_int, hl_changed: c_int) {
    // display_tick is declared as u32 in Rust (existing binding); use u64 for comparison
    // with b_mod_tick_syn/decor (uint64_t in C). Values stay well within 32-bit range.
    let tick = u64::from(display_tick);

    // Pass 1: update highlights + syntax/decor
    let mut wp = nvim_get_firstwin();
    while !wp.is_null() {
        update_window_hl(wp, type_ >= UPD_NOT_VALID || hl_changed != 0);

        let buf = nvim_win_get_buffer(wp);
        if nvim_buf_get_mod_set(buf) != 0 {
            if nvim_buf_get_mod_tick_syn(buf) < tick && syntax_present(wp) {
                syn_stack_apply_changes(buf);
                nvim_buf_set_mod_tick_syn(buf, tick);
            }
            if nvim_buf_get_mod_tick_decor(buf) < tick {
                decor_providers_invoke_buf(buf);
                nvim_buf_set_mod_tick_decor(buf, tick);
            }
        }
        wp = nvim_win_get_next(wp);
    }

    // Clear any stale search hl state before the draw pass
    // (equivalent to screen_search_hl.rm.regprog = NULL in C).
    rs_end_search_hl();
    let mut did_one = false;

    // Pass 2: grid alloc + border + win_update + status
    let mut wp = nvim_get_firstwin();
    while !wp.is_null() {
        let redr_type = nvim_win_get_redr_type(wp);
        if redr_type == UPD_CLEAR
            && nvim_win_get_floating(wp) != 0
            && nvim_win_get_grid_alloc_chars(wp)
        {
            nvim_win_grid_alloc_invalidate(wp);
            nvim_win_set_redr_type(wp, UPD_NOT_VALID);
        }

        nvim_win_check_ns_hl(wp);
        win_grid_alloc(wp);

        let redr_type = nvim_win_get_redr_type(wp);
        let redr_border = nvim_win_get_redr_border(wp);
        if redr_border != 0 || redr_type >= UPD_NOT_VALID {
            nvim_win_draw_border(wp);
        }

        if redr_type != 0 {
            if !did_one {
                did_one = true;
                rs_start_search_hl();
            }
            nvim_win_update(wp);
        }

        if nvim_win_get_redr_status(wp) != 0 {
            nvim_win_redr_winbar(wp);
            nvim_win_redr_status(wp);
        }

        wp = nvim_win_get_next(wp);
    }

    rs_end_search_hl();

    if pum_drawn() && must_redraw_pum {
        nvim_win_check_ns_hl(nvim_get_curwin());
        pum_redraw();
    }

    nvim_win_check_ns_hl(WinHandle::null());

    // Pass 3: reset b_mod_set
    let mut wp = nvim_get_firstwin();
    while !wp.is_null() {
        let buf = nvim_win_get_buffer(wp);
        nvim_buf_set_mod_set(buf, 0);
        wp = nvim_win_get_next(wp);
    }
}

/// Handle the three window iteration loops of update_screen.
///
/// Exported as `rs_update_screen_win_loop` for future direct calls.
/// Called internally by `rs_update_screen`.
#[no_mangle]
pub unsafe extern "C" fn rs_update_screen_win_loop(type_: c_int, hl_changed: c_int) {
    update_screen_win_loop_impl(type_, hl_changed);
}

// =============================================================================
// Phase 5: update_screen
// =============================================================================

extern "C" {
    /// Check if default_grid.chars is non-NULL.
    fn nvim_default_grid_has_chars() -> bool;
    /// Check if default_grid.valid is set.
    fn nvim_default_grid_is_valid() -> bool;
    /// Set default_grid.valid.
    fn nvim_default_grid_set_valid(val: bool);
    /// Invalidate default_grid via grid_invalidate.
    fn nvim_default_grid_invalidate();

    /// Handle msg_scrolled / msg_grid_invalid block (lines 608-650 of nvim_update_screen_impl).
    fn nvim_update_screen_msg_scroll(type_: c_int, is_stl_global: c_int) -> c_int;
    /// Handle curwin nrwidth check and tabline redraw (lines 691-709).
    fn nvim_update_screen_nrwidth_check(type_: c_int);

    /// resizing_autocmd: true during screen resize autocmds.
    static mut resizing_autocmd: bool;
    /// redraw_popupmenu: true when the popupmenu needs redraw.
    static mut redraw_popupmenu: bool;
    /// msg_grid_invalid: true when msg_grid needs revalidation.
    static mut msg_grid_invalid: bool;

    /// display_tick: incremented each time the screen is updated.
    static mut display_tick: u32;
    /// need_diff_redraw: diff redraw pending.
    static mut need_diff_redraw: bool;
    /// msg_did_scroll: set when message caused scrolling.
    static mut msg_did_scroll: bool;
    /// msg_scrolled_at_flush: saved msg_scrolled at last flush.
    static mut msg_scrolled_at_flush: c_int;
    /// need_highlight_changed: highlight groups changed.
    static mut need_highlight_changed: bool;
    /// must_redraw_pum: popupmenu must be redrawn.
    static mut must_redraw_pum: bool;
    /// cmdline_was_last_drawn: cmdline was last thing drawn.
    static mut cmdline_was_last_drawn: bool;

    /// Clear schar cache if full; returns true if cleared.
    fn schar_cache_clear_if_full() -> bool;
    /// Show intro message or return false if conditions not met.
    fn may_show_intro() -> bool;
    /// Redraw diffs.
    fn diff_redraw(check: bool);
    /// Called after screenclear when cmdline is cleared.
    fn cmdline_screen_cleared();
    /// UI clear message area.
    fn ui_call_msg_clear();
    /// Start decoration providers.
    fn decor_providers_start();
    /// End decoration providers.
    fn decor_providers_invoke_end();
    /// Flush window UI.
    fn win_ui_flush(validate: bool);
    /// Recompute cmdrow.
    fn compute_cmdrow();
    /// Apply pending highlight changes.
    fn highlight_changed();
    /// Update curswant after visual mode change.
    fn update_curswant();
    /// Show intro message.
    fn intro_message(when_starting: bool);
    /// Repeat last message.
    fn repeat_message();
    /// Call maketitle.
    fn maketitle();
    /// Check if kUICmdline is active; returns redraw_cmdline value.
    fn nvim_get_redraw_cmdline() -> c_int;
    /// Set screen valid/invalid for compositing.
    fn ui_comp_set_screen_valid(valid: bool) -> bool;
    /// Clear a region of the default grid.
    fn grid_clear(
        view: GridViewHandle,
        start_row: c_int,
        end_row: c_int,
        start_col: c_int,
        end_col: c_int,
        attr: c_int,
    );
}

/// C OK/FAIL return values.
const OK: c_int = 0;
const FAIL: c_int = 1;

/// kUIMessages UI extension constant.
const K_UI_CMDLINE: c_int = 3;

/// Main screen update orchestrator.
///
/// Replaces `nvim_update_screen_impl()` which has been removed from drawscreen.c.
/// Contains full orchestration logic; struct-heavy sections are delegated to
/// C behavioral helpers in drawscreen_shim.c.
///
/// Rust equivalent of `update_screen()` in drawscreen.c.
#[unsafe(export_name = "update_screen")]
pub unsafe extern "C" fn rs_update_screen() -> c_int {
    // Static: tracks whether intro has been shown.
    static STILL_MAY_INTRO: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(true);

    use std::sync::atomic::Ordering;

    if STILL_MAY_INTRO.load(Ordering::Relaxed) && !may_show_intro() {
        redraw_later(nvim_get_firstwin(), UPD_NOT_VALID);
        STILL_MAY_INTRO.store(false, Ordering::Relaxed);
    }

    let is_stl_global = c_int::from(global_stl_height() != 0);

    // Don't do anything if the screen structures are (not yet) valid.
    if resizing_autocmd || !nvim_default_grid_has_chars() {
        return FAIL;
    }

    // May have postponed updating diffs.
    if need_diff_redraw {
        diff_redraw(true);
    }

    // Postpone redrawing when not needed or called recursively.
    if !redrawing_impl() || updating_screen || rs_cmdline_number_prompt() != 0 {
        return FAIL;
    }

    let mut type_ = must_redraw;
    must_redraw = 0;
    updating_screen = true;

    display_tick += 1;

    if schar_cache_clear_if_full() {
        type_ = type_.max(UPD_CLEAR);
    }

    if msg_did_scroll {
        msg_did_scroll = false;
        msg_scrolled_at_flush = 0;
    }

    if type_ >= UPD_CLEAR || !nvim_default_grid_is_valid() {
        ui_comp_set_screen_valid(false);
    }

    // Handle msg_scrolled / msg_grid_invalid block (struct-heavy, delegated to C).
    type_ = nvim_update_screen_msg_scroll(type_, is_stl_global);

    win_ui_flush(true);
    compute_cmdrow();

    let hl_changed: c_int = if need_highlight_changed {
        highlight_changed();
        1
    } else {
        0
    };

    if type_ == UPD_CLEAR {
        rs_screenclear();
        cmdline_screen_cleared();
        if ui_has(K_UI_MESSAGES) {
            ui_call_msg_clear();
        }
        type_ = UPD_NOT_VALID;
        must_redraw = 0;
    } else if !nvim_default_grid_is_valid() {
        nvim_default_grid_invalidate();
        nvim_default_grid_set_valid(true);
    }

    #[allow(clippy::cast_possible_truncation)]
    if type_ == UPD_NOT_VALID && nvim_get_clear_cmdline() && !ui_has(K_UI_MESSAGES) {
        grid_clear(
            nvim_get_default_gridview(),
            Rows - p_ch as c_int,
            Rows,
            0,
            Columns,
            0,
        );
    }

    ui_comp_set_screen_valid(true);

    decor_providers_start();

    if nvim_win_check_ns_hl(WinHandle::null()) {
        nvim_set_redraw_cmdline(true);
        redraw_tabline = true;
    }

    if nvim_get_clear_cmdline() {
        msg_check_for_delay(c_int::from(false));
    }

    // Handle nrwidth check and tabline (struct-heavy, delegated to C).
    nvim_update_screen_nrwidth_check(type_);

    // Handle the three window loops (now in Rust).
    update_screen_win_loop_impl(type_, hl_changed);

    updating_screen = false;

    if need_maketitle {
        maketitle();
    }

    if nvim_get_clear_cmdline() || nvim_get_redraw_cmdline() != 0 || redraw_mode != 0 {
        rs_showmode();
    }

    if STILL_MAY_INTRO.load(Ordering::Relaxed) {
        intro_message(false);
    }
    repeat_message();

    decor_providers_invoke_end();

    if !ui_has(K_UI_CMDLINE) {
        cmdline_was_last_drawn = false;
    }
    OK
}

// =============================================================================
// Phase 4: show_cursor_info_later
// =============================================================================

extern "C" {
    /// Validate virtcol for the current window.
    fn nvim_validate_virtcol_curwin();
    /// Return 1 if cursor line in curwin starts with NUL (empty, not insert mode).
    fn nvim_curwin_cursor_line_is_nul() -> c_int;
    /// Get VIsual position (lnum, col, coladd) as three i32 values.
    fn nvim_get_VIsual_pos_fields(lnum: *mut i32, col: *mut i32, coladd: *mut i32);
    /// Individual stl snapshot getters.
    fn nvim_curwin_get_stl_cursor_lnum() -> i32;
    fn nvim_curwin_get_stl_cursor_col() -> i32;
    fn nvim_curwin_get_stl_cursor_coladd() -> i32;
    fn nvim_curwin_get_stl_virtcol() -> i32;
    fn nvim_curwin_get_stl_topline() -> i32;
    fn nvim_curwin_get_stl_line_count() -> i32;
    fn nvim_curwin_get_stl_topfill() -> i32;
    fn nvim_curwin_get_stl_empty() -> i32;
    fn nvim_curwin_get_stl_recording() -> i32;
    fn nvim_curwin_get_stl_state() -> i32;
    fn nvim_curwin_get_stl_visual_mode() -> i32;
    fn nvim_curwin_get_stl_vis_lnum() -> i32;
    fn nvim_curwin_get_stl_vis_col() -> i32;
    fn nvim_curwin_get_stl_vis_coladd() -> i32;
    /// Write curwin's w_stl_* state from current window values.
    fn nvim_curwin_set_stl_from_cursor(
        state: i32,
        empty_line: i32,
        visual_active: i32,
        visual_mode: i32,
        vis_lnum: i32,
        vis_col: i32,
        vis_coladd: i32,
    );
    /// Get w_virtcol for the window.
    fn nvim_win_get_virtcol(wp: WinHandle) -> c_int;
    /// Get w_topfill for the window.
    fn nvim_win_get_topfill(wp: WinHandle) -> c_int;
    /// Get b_ml.ml_line_count for window's buffer.
    fn nvim_win_buf_line_count(wp: WinHandle) -> c_int;
    /// Get w_cursor.col for window.
    fn nvim_win_get_cursor_col(wp: WinHandle) -> c_int;
    /// Get w_cursor.coladd for window.
    fn nvim_win_get_cursor_coladd(wp: WinHandle) -> c_int;
    /// Return 1 if global p_wbr is empty/NUL.
    fn nvim_get_p_wbr_empty() -> c_int;
    /// Return 1 if window's w_p_wbr is empty/NUL.
    fn nvim_win_get_p_wbr_empty(wp: WinHandle) -> c_int;
    /// Get real state (MODE_* flags).
    #[link_name = "get_real_state"]
    fn nvim_get_real_state() -> c_int;
}

/// Check if cursor/ruler/statusline info changed and schedule redraw.
///
/// Rust equivalent of `show_cursor_info_later()` in drawscreen.c.
#[unsafe(export_name = "show_cursor_info_later")]
pub unsafe extern "C" fn rs_show_cursor_info_later(force: bool) {
    let curwin = nvim_get_curwin();
    let state = nvim_get_real_state();
    let empty_line = if (State & MODE_INSERT) == 0 {
        nvim_curwin_cursor_line_is_nul()
    } else {
        0
    };

    nvim_validate_virtcol_curwin();

    // Read the saved stl snapshot (14 individual fields):
    let stl_cursor_lnum = nvim_curwin_get_stl_cursor_lnum();
    let stl_cursor_col = nvim_curwin_get_stl_cursor_col();
    let stl_cursor_coladd = nvim_curwin_get_stl_cursor_coladd();
    let stl_virtcol = nvim_curwin_get_stl_virtcol();
    let stl_topline = nvim_curwin_get_stl_topline();
    let stl_line_count = nvim_curwin_get_stl_line_count();
    let stl_topfill = nvim_curwin_get_stl_topfill();
    let stl_empty = nvim_curwin_get_stl_empty();
    let stl_recording = nvim_curwin_get_stl_recording();
    let stl_state = nvim_curwin_get_stl_state();
    let stl_visual_mode = nvim_curwin_get_stl_visual_mode();
    let stl_vis_lnum = nvim_curwin_get_stl_vis_lnum();
    let stl_vis_col = nvim_curwin_get_stl_vis_col();
    let stl_vis_coladd = nvim_curwin_get_stl_vis_coladd();

    let vis_active = nvim_VIsual_active() != 0;
    let cur_lnum = nvim_win_get_cursor_lnum(curwin);
    let cur_col = nvim_win_get_cursor_col(curwin);
    let cur_coladd = nvim_win_get_cursor_coladd(curwin);
    let cur_virtcol = nvim_win_get_virtcol(curwin);
    let cur_topline = nvim_win_get_topline(curwin);
    let cur_line_count = nvim_win_buf_line_count(curwin);
    let cur_topfill = nvim_win_get_topfill(curwin);

    let mut vis_lnum = 0i32;
    let mut vis_col = 0i32;
    let mut vis_coladd = 0i32;
    if vis_active {
        nvim_get_VIsual_pos_fields(&raw mut vis_lnum, &raw mut vis_col, &raw mut vis_coladd);
    }

    let changed = force
        || cur_lnum != stl_cursor_lnum
        || cur_col != stl_cursor_col
        || cur_coladd != stl_cursor_coladd
        || cur_virtcol != stl_virtcol
        || cur_topline != stl_topline
        || cur_line_count != stl_line_count
        || cur_topfill != stl_topfill
        || empty_line != stl_empty
        || reg_recording != stl_recording
        || state != stl_state
        || (vis_active
            && (VIsual_mode != stl_visual_mode
                || vis_lnum != stl_vis_lnum
                || vis_col != stl_vis_col
                || vis_coladd != stl_vis_coladd));

    if changed {
        if nvim_win_get_status_height(curwin) != 0 || global_stl_height() != 0 {
            nvim_win_set_redr_status(curwin, 1);
        } else {
            nvim_set_redraw_cmdline(true);
        }
        if nvim_get_p_wbr_empty() == 0 || nvim_win_get_p_wbr_empty(curwin) == 0 {
            nvim_win_set_redr_status(curwin, 1);
        }
        rs_redraw_custom_title_later();
    }

    nvim_curwin_set_stl_from_cursor(
        state,
        empty_line,
        c_int::from(vis_active),
        VIsual_mode,
        vis_lnum,
        vis_col,
        vis_coladd,
    );
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

    // Note: test_null_window_returns_false was removed because hsep_connected_impl
    // and vsep_connected_impl reference FFI symbols that aren't available in `cargo test`.

    #[test]
    fn test_hlf_c_constant() {
        // HLF_C should be 21 (WinSeparator highlight group)
        let hlf_c = HLF_C;
        assert_eq!(hlf_c, 21);
    }

    #[test]
    fn test_upd_valid_constant() {
        // UPD_VALID should be 10 (matches drawscreen.h)
        let upd_valid = UPD_VALID;
        assert_eq!(upd_valid, 10);
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
