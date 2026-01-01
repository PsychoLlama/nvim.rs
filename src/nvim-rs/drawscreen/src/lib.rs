//! Screen drawing functions for Neovim
//!
//! This crate provides Rust implementations of screen drawing functions
//! from `src/nvim/drawscreen.c`, focusing on separator drawing between windows.

#![allow(unsafe_code)]
#![allow(dead_code)] // Some extern functions are pre-declared for future use
#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]

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
}
