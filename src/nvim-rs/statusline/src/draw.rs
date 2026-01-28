//! Main draw entry points for statusline, winbar, ruler, and tabline
//!
//! This module provides the high-level orchestration functions that coordinate
//! rendering of the various status components. These functions serve as the
//! main entry points called from C code.

use std::ffi::c_int;

use nvim_window::WinHandle;

use crate::highlight::{get_statusline_hl, get_winbar_hl};
use crate::ruler::{render_ruler, RulerContext, RulerOptions};

// C accessor functions for draw operations
extern "C" {
    // Window accessors
    fn nvim_win_is_curwin(wp: WinHandle) -> c_int;
    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> c_int;
    fn nvim_win_get_cursor_col(wp: WinHandle) -> c_int;
    fn nvim_win_get_virtcol(wp: WinHandle) -> c_int;
    fn nvim_win_buf_line_count(wp: WinHandle) -> c_int;

    // Global state accessors
    fn nvim_global_stl_height() -> c_int;
}

/// Draw context for status-related rendering operations.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DrawContext {
    /// Window handle (may be null for tabline)
    pub wp: WinHandle,
    /// Maximum width available
    pub max_width: c_int,
    /// Row to draw on
    pub row: c_int,
    /// Starting column
    pub col: c_int,
    /// Fill character
    pub fill_char: u32,
    /// Highlight attribute
    pub attr: c_int,
    /// Whether this is for current window
    pub is_curwin: bool,
}

impl DrawContext {
    /// Create a new draw context for the tabline.
    #[allow(clippy::cast_lossless)]
    pub const fn for_tabline(width: c_int) -> Self {
        Self {
            wp: WinHandle::null(),
            max_width: width,
            row: 0,
            col: 0,
            fill_char: b' ' as u32,
            attr: 0, // Will be set by caller
            is_curwin: false,
        }
    }

    /// Create a new draw context with explicit values.
    pub const fn new(
        wp: WinHandle,
        max_width: c_int,
        row: c_int,
        col: c_int,
        fill_char: u32,
        attr: c_int,
        is_curwin: bool,
    ) -> Self {
        Self {
            wp,
            max_width,
            row,
            col,
            fill_char,
            attr,
            is_curwin,
        }
    }
}

/// Result of a draw operation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DrawResult {
    /// Number of columns written
    pub width: c_int,
    /// Whether drawing was successful
    pub success: bool,
    /// Whether the content was truncated
    pub truncated: bool,
}

/// Get ruler context from explicit parameters.
///
/// Creates a ruler context from the provided cursor position info.
/// This allows C code to pass in values without requiring additional
/// accessor functions.
pub const fn make_ruler_context(
    lnum: c_int,
    line_count: c_int,
    col: c_int,
    virtcol: c_int,
    empty_line: bool,
) -> RulerContext {
    RulerContext {
        lnum,
        line_count,
        col,
        virtcol,
        empty_line,
    }
}

/// Render the ruler to a buffer using a pre-built context.
///
/// This is the main ruler rendering function that can be used
/// both for statusline rulers and command-line rulers.
#[allow(clippy::cast_sign_loss)]
pub fn render_ruler_with_context(buf: &mut [u8], ctx: &RulerContext, opts: &RulerOptions) -> c_int {
    render_ruler(buf, ctx, opts)
}

/// Calculate the ruler column position.
///
/// The ruler is positioned to the right of center, taking into
/// account the configured ruler column and window width.
pub const fn calc_ruler_col(ru_col: c_int, width: c_int) -> c_int {
    // Never use more than half the width
    let half = (width + 1) / 2;
    if ru_col > half {
        ru_col
    } else {
        half
    }
}

/// Check if global statusline is enabled.
pub fn is_global_statusline() -> bool {
    unsafe { nvim_global_stl_height() > 0 }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Get the highlight group for statusline.
///
/// Returns the appropriate highlight group ID based on whether this is
/// the current window.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub extern "C" fn rs_draw_statusline_hl(wp: WinHandle) -> c_int {
    let is_curwin = unsafe { nvim_win_is_curwin(wp) != 0 };
    get_statusline_hl(is_curwin)
}

/// FFI export: Get the highlight group for winbar.
///
/// Returns the appropriate highlight group ID based on whether this is
/// the current window.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub extern "C" fn rs_draw_winbar_hl(wp: WinHandle) -> c_int {
    let is_curwin = unsafe { nvim_win_is_curwin(wp) != 0 };
    get_winbar_hl(is_curwin)
}

/// FFI export: Check if global statusline is enabled.
#[no_mangle]
pub extern "C" fn rs_draw_is_global_stl() -> c_int {
    c_int::from(is_global_statusline())
}

/// FFI export: Calculate ruler column position.
#[no_mangle]
pub const extern "C" fn rs_draw_calc_ruler_col(ru_col: c_int, width: c_int) -> c_int {
    calc_ruler_col(ru_col, width)
}

/// FFI export: Render ruler to buffer with explicit context.
///
/// Renders the ruler string (line,col position) to the provided buffer.
/// Returns the number of bytes written.
///
/// # Safety
/// - `buf` must be null or a valid pointer to a buffer of at least `buflen` bytes.
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_draw_render_ruler_ex(
    buf: *mut u8,
    buflen: usize,
    lnum: c_int,
    line_count: c_int,
    col: c_int,
    virtcol: c_int,
    empty_line: c_int,
) -> c_int {
    if buf.is_null() || buflen == 0 {
        return 0;
    }

    let ctx = make_ruler_context(lnum, line_count, col, virtcol, empty_line != 0);
    let opts = RulerOptions::default();
    let slice = std::slice::from_raw_parts_mut(buf, buflen);
    render_ruler_with_context(slice, &ctx, &opts)
}

/// FFI export: Render ruler to buffer from window.
///
/// Convenience wrapper that extracts cursor position from window.
/// Returns the number of bytes written.
///
/// # Safety
/// - `buf` must be null or a valid pointer to a buffer of at least `buflen` bytes.
/// - `wp` must be a valid window handle.
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_draw_render_ruler(
    buf: *mut u8,
    buflen: usize,
    wp: WinHandle,
    empty_line: c_int,
) -> c_int {
    if buf.is_null() || buflen == 0 || wp.is_null() {
        return 0;
    }

    let lnum = nvim_win_get_cursor_lnum(wp);
    let col = nvim_win_get_cursor_col(wp) + 1; // 1-based
    let virtcol = nvim_win_get_virtcol(wp) + 1; // 1-based
    let line_count = nvim_win_buf_line_count(wp);

    let ctx = make_ruler_context(lnum, line_count, col, virtcol, empty_line != 0);
    let opts = RulerOptions::default();
    let slice = std::slice::from_raw_parts_mut(buf, buflen);
    render_ruler_with_context(slice, &ctx, &opts)
}

/// FFI export: Create a ruler context.
///
/// Returns a RulerContext structure initialized with the provided values.
#[no_mangle]
pub const extern "C" fn rs_draw_make_ruler_context(
    lnum: c_int,
    line_count: c_int,
    col: c_int,
    virtcol: c_int,
    empty_line: c_int,
) -> RulerContext {
    make_ruler_context(lnum, line_count, col, virtcol, empty_line != 0)
}

/// Tabline drawing state tracker.
#[repr(C)]
#[derive(Debug, Default)]
pub struct TablineDrawState {
    /// Current column position
    pub col: c_int,
    /// Total width available
    pub width: c_int,
    /// Number of tabs rendered
    pub tab_count: c_int,
    /// Current tab being rendered
    pub current_tab: c_int,
    /// Width per tab
    pub tab_width: c_int,
}

impl TablineDrawState {
    /// Create a new tabline draw state.
    pub const fn new(width: c_int, tab_count: c_int) -> Self {
        let tab_width = if tab_count > 0 {
            // Same formula as rs_tabwidth_calc
            let w = (width - 1 + tab_count / 2) / tab_count;
            if w < 6 {
                6
            } else {
                w
            }
        } else {
            0
        };

        Self {
            col: 0,
            width,
            tab_count,
            current_tab: 0,
            tab_width,
        }
    }

    /// Check if there's room for another tab.
    pub const fn has_room(&self) -> bool {
        self.col < self.width && self.current_tab < self.tab_count
    }

    /// Advance to the next tab.
    pub const fn advance_tab(&mut self) {
        self.current_tab += 1;
        self.col += self.tab_width;
    }

    /// Get remaining width.
    pub const fn remaining_width(&self) -> c_int {
        self.width - self.col
    }
}

/// FFI export: Create tabline draw state.
#[no_mangle]
pub const extern "C" fn rs_tabline_state_new(width: c_int, tab_count: c_int) -> TablineDrawState {
    TablineDrawState::new(width, tab_count)
}

/// FFI export: Check if tabline has room for more tabs.
#[no_mangle]
pub const extern "C" fn rs_tabline_has_room(state: &TablineDrawState) -> c_int {
    if state.has_room() {
        1
    } else {
        0
    }
}

/// FFI export: Get tab width for tabline.
#[no_mangle]
pub const extern "C" fn rs_tabline_get_tab_width(state: &TablineDrawState) -> c_int {
    state.tab_width
}

/// FFI export: Advance tabline state to next tab.
#[no_mangle]
pub const extern "C" fn rs_tabline_advance(state: &mut TablineDrawState) {
    state.advance_tab();
}

/// FFI export: Get remaining width in tabline.
#[no_mangle]
pub const extern "C" fn rs_tabline_remaining(state: &TablineDrawState) -> c_int {
    state.remaining_width()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_context_default() {
        let ctx = DrawContext::for_tabline(80);
        assert_eq!(ctx.max_width, 80);
        assert_eq!(ctx.row, 0);
        assert_eq!(ctx.col, 0);
        assert!(ctx.wp.is_null());
    }

    #[test]
    fn test_draw_context_new() {
        let ctx = DrawContext::new(WinHandle::null(), 100, 5, 10, u32::from(b'-'), 42, true);
        assert_eq!(ctx.max_width, 100);
        assert_eq!(ctx.row, 5);
        assert_eq!(ctx.col, 10);
        assert_eq!(ctx.fill_char, u32::from(b'-'));
        assert_eq!(ctx.attr, 42);
        assert!(ctx.is_curwin);
    }

    #[test]
    fn test_draw_result_default() {
        let result = DrawResult::default();
        assert_eq!(result.width, 0);
        assert!(!result.success);
        assert!(!result.truncated);
    }

    #[test]
    fn test_calc_ruler_col() {
        // Ruler column should never be less than half width
        // half = (80 + 1) / 2 = 40
        assert_eq!(calc_ruler_col(10, 80), 40); // 10 < 40, use 40
        assert_eq!(calc_ruler_col(50, 80), 50); // 50 > 40, use 50
        assert_eq!(calc_ruler_col(17, 80), 40); // 17 < 40, use 40

        // Edge cases
        assert_eq!(calc_ruler_col(0, 80), 40);
        assert_eq!(calc_ruler_col(100, 80), 100);
    }

    #[test]
    fn test_make_ruler_context() {
        let ctx = make_ruler_context(42, 100, 10, 15, false);
        assert_eq!(ctx.lnum, 42);
        assert_eq!(ctx.line_count, 100);
        assert_eq!(ctx.col, 10);
        assert_eq!(ctx.virtcol, 15);
        assert!(!ctx.empty_line);
    }

    #[test]
    fn test_render_ruler_with_context() {
        let ctx = make_ruler_context(42, 100, 10, 10, false);
        let opts = RulerOptions::default();
        let mut buf = [0u8; 64];
        let len = render_ruler_with_context(&mut buf, &ctx, &opts);

        assert!(len > 0);
        #[allow(clippy::cast_sign_loss)]
        let result = std::str::from_utf8(&buf[..len as usize]).unwrap();
        assert!(result.contains("42"));
        assert!(result.contains("10"));
    }

    #[test]
    fn test_tabline_draw_state_new() {
        let state = TablineDrawState::new(80, 5);
        assert_eq!(state.width, 80);
        assert_eq!(state.tab_count, 5);
        assert_eq!(state.current_tab, 0);
        assert_eq!(state.col, 0);
        // (80 - 1 + 2) / 5 = 81 / 5 = 16
        assert_eq!(state.tab_width, 16);
    }

    #[test]
    fn test_tabline_draw_state_min_width() {
        let state = TablineDrawState::new(20, 10);
        // Would be (20 - 1 + 5) / 10 = 24 / 10 = 2, but minimum is 6
        assert_eq!(state.tab_width, 6);
    }

    #[test]
    fn test_tabline_draw_state_has_room() {
        let mut state = TablineDrawState::new(80, 3);
        assert!(state.has_room());

        state.advance_tab();
        assert!(state.has_room());

        state.advance_tab();
        assert!(state.has_room());

        state.advance_tab();
        assert!(!state.has_room()); // All 3 tabs rendered
    }

    #[test]
    fn test_tabline_draw_state_remaining() {
        let mut state = TablineDrawState::new(80, 2);
        assert_eq!(state.remaining_width(), 80);

        state.advance_tab();
        // tab_width = (80 - 1 + 1) / 2 = 40
        assert_eq!(state.remaining_width(), 40);

        state.advance_tab();
        assert_eq!(state.remaining_width(), 0);
    }

    #[test]
    fn test_ruler_context_default() {
        let ctx = RulerContext::default();
        assert_eq!(ctx.lnum, 1);
        assert_eq!(ctx.line_count, 1);
        assert_eq!(ctx.col, 1);
        assert_eq!(ctx.virtcol, 1);
        assert!(!ctx.empty_line);
    }
}
