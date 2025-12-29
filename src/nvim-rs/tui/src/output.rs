//! TUI output functions
//!
//! This module contains functions for terminal output, including attribute
//! comparison and ANSI escape sequence generation.

use nvim_highlight::hl_attr_flags::HL_UNDERLINE_MASK;
use nvim_highlight::HlAttrs;
use std::ffi::c_int;

// ============================================================================
// Attribute Comparison
// ============================================================================

/// Check if two attribute IDs have different visual attributes.
///
/// This function compares two highlight attribute entries to determine if they
/// would produce different visual output. It's used to optimize terminal output
/// by avoiding redundant attribute changes.
///
/// # Arguments
///
/// * `id1` - First attribute ID
/// * `id2` - Second attribute ID
/// * `rgb` - Whether we're in RGB (truecolor) mode
/// * `attrs` - Pointer to the HlAttrs array
/// * `attrs_size` - Size of the attrs array
///
/// # Safety
///
/// - `attrs` must be a valid pointer to an array of at least `attrs_size` HlAttrs
/// - The caller must ensure the array remains valid for the duration of the call
#[no_mangle]
pub unsafe extern "C" fn rs_attrs_differ(
    id1: c_int,
    id2: c_int,
    rgb: bool,
    attrs: *const HlAttrs,
    attrs_size: usize,
) -> bool {
    attrs_differ_impl(id1, id2, rgb, attrs, attrs_size)
}

/// Implementation of attrs_differ that can be tested
unsafe fn attrs_differ_impl(
    id1: c_int,
    id2: c_int,
    rgb: bool,
    attrs: *const HlAttrs,
    attrs_size: usize,
) -> bool {
    // Same ID means same attributes
    if id1 == id2 {
        return false;
    }

    // Negative IDs indicate special/missing attributes - always differ
    if id1 < 0 || id2 < 0 {
        return true;
    }

    let idx1 = id1 as usize;
    let idx2 = id2 as usize;

    // Bounds check
    if idx1 >= attrs_size || idx2 >= attrs_size {
        return true;
    }

    let a1 = *attrs.add(idx1);
    let a2 = *attrs.add(idx2);

    // URL always matters
    if a1.url != a2.url {
        return true;
    }

    if rgb {
        // RGB mode: compare RGB colors and attributes
        a1.rgb_fg_color != a2.rgb_fg_color
            || a1.rgb_bg_color != a2.rgb_bg_color
            || a1.rgb_ae_attr != a2.rgb_ae_attr
            || a1.rgb_sp_color != a2.rgb_sp_color
    } else {
        // cterm mode: compare cterm colors and attributes
        // Also check sp_color for underline styles
        a1.cterm_fg_color != a2.cterm_fg_color
            || a1.cterm_bg_color != a2.cterm_bg_color
            || a1.cterm_ae_attr != a2.cterm_ae_attr
            || (a1.cterm_ae_attr & HL_UNDERLINE_MASK != 0 && a1.rgb_sp_color != a2.rgb_sp_color)
    }
}

// ============================================================================
// Grid Cursor Position
// ============================================================================

/// Opaque handle to TUIData struct in C
#[repr(C)]
pub struct TuiHandle {
    _private: [u8; 0],
}

/// Opaque handle to UGrid struct in C
#[repr(C)]
pub struct UGridHandle {
    _private: [u8; 0],
}

extern "C" {
    fn nvim_tui_set_row(tui: *mut TuiHandle, row: c_int);
    fn nvim_tui_set_col(tui: *mut TuiHandle, col: c_int);
    fn nvim_tui_set_attrs(tui: *mut TuiHandle, idx: usize, attrs: HlAttrs);
    fn nvim_tui_set_clear_attrs(tui: *mut TuiHandle, attrs: HlAttrs);
    fn nvim_tui_set_print_attr_id(tui: *mut TuiHandle, id: c_int);
    fn nvim_tui_set_default_colors_flag(tui: *mut TuiHandle, value: bool);
    fn nvim_tui_get_grid_height(tui: *mut TuiHandle) -> c_int;
    fn nvim_tui_get_grid_width(tui: *mut TuiHandle) -> c_int;
    fn nvim_tui_invalidate(tui: *mut TuiHandle, top: c_int, bot: c_int, left: c_int, right: c_int);

    // Grid resize/clear accessors
    fn nvim_tui_get_is_starting(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_get_pending_resize_events(tui: *mut TuiHandle) -> c_int;
    fn nvim_tui_set_pending_resize_events(tui: *mut TuiHandle, val: c_int);
    fn nvim_tui_get_invalid_regions_size(tui: *mut TuiHandle) -> usize;
    fn nvim_tui_clear_invalid_regions(tui: *mut TuiHandle);
    fn nvim_tui_clip_invalid_region(
        tui: *mut TuiHandle,
        idx: usize,
        max_height: c_int,
        max_width: c_int,
    );
    fn nvim_tui_get_grid(tui: *mut TuiHandle) -> *mut UGridHandle;
    fn nvim_tui_invalidate_grid_cursor(tui: *mut TuiHandle);
    fn nvim_tui_get_width(tui: *mut TuiHandle) -> c_int;
    fn nvim_tui_get_height(tui: *mut TuiHandle) -> c_int;
    fn nvim_tui_out_resize(tui: *mut TuiHandle, height: c_int, width: c_int);
    fn nvim_tui_clear_region(
        tui: *mut TuiHandle,
        top: c_int,
        bot: c_int,
        left: c_int,
        right: c_int,
        attr_id: c_int,
    );

    // UGrid functions (already in Rust ugrid crate, called via C wrappers)
    fn ugrid_resize(grid: *mut UGridHandle, width: c_int, height: c_int);
    fn ugrid_clear(grid: *mut UGridHandle);

    // schar cache function
    fn schar_cache_clear_if_full() -> bool;

    // Busy state accessors
    fn nvim_tui_set_busy(tui: *mut TuiHandle, busy: bool);

    // Output function for bell
    fn nvim_tui_out(tui: *mut TuiHandle, str: *const u8, len: usize);

    // Mouse accessors
    fn nvim_tui_get_mouse_enabled(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_set_mouse_enabled(tui: *mut TuiHandle, enabled: bool);
    fn nvim_tui_get_mouse_move_enabled(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_set_term_mode(tui: *mut TuiHandle, mode: c_int, set: bool);
    fn nvim_tui_get_reset_scroll_region(tui: *mut TuiHandle) -> *const u8;
    fn nvim_tui_out_len(tui: *mut TuiHandle, str: *const u8);
    fn nvim_tui_get_screen_or_tmux(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_flush_buf(tui: *mut TuiHandle);
    fn nvim_tui_uv_sleep(ms: u64);
}

// Terminfo output infrastructure
#[allow(dead_code)]
extern "C" {
    fn nvim_tui_terminfo_out(tui: *mut TuiHandle, what: c_int);
    fn nvim_tui_terminfo_print_num1(tui: *mut TuiHandle, what: c_int, num1: c_int);
    fn nvim_tui_terminfo_print_num2(tui: *mut TuiHandle, what: c_int, num1: c_int, num2: c_int);
    fn nvim_tui_get_grid_row(tui: *mut TuiHandle) -> c_int;
    fn nvim_tui_get_grid_col(tui: *mut TuiHandle) -> c_int;
    fn nvim_tui_get_url(tui: *mut TuiHandle) -> c_int;
    fn nvim_tui_set_url(tui: *mut TuiHandle, url: c_int);
    fn nvim_tui_get_print_attr_id(tui: *mut TuiHandle) -> c_int;
    fn nvim_tui_get_immediate_wrap(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_cursor_goto(tui: *mut TuiHandle, row: c_int, col: c_int);
    fn nvim_tui_update_attrs(tui: *mut TuiHandle, attr_id: c_int);
    fn nvim_tui_get_can_clear_attr(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_get_can_erase_chars(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_get_set_default_colors(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_cheap_to_print(tui: *mut TuiHandle, row: c_int, col: c_int, next: c_int) -> bool;
    fn nvim_tui_get_default_attr(tui: *mut TuiHandle) -> bool;

    // UGrid goto (already in Rust ugrid crate, called via C wrapper)
    fn ugrid_goto(grid: *mut UGridHandle, row: c_int, col: c_int);

    // Grid col increment
    fn nvim_tui_inc_grid_col(tui: *mut TuiHandle);

    // Scroll capability accessors
    fn nvim_tui_get_can_scroll(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_get_can_change_scroll_region(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_get_has_lr_margin_mode(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_get_can_set_lr_margin(tui: *mut TuiHandle) -> bool;

    // Internal function wrappers for scroll
    fn nvim_tui_cursor_goto_internal(tui: *mut TuiHandle, row: c_int, col: c_int);
    fn nvim_tui_update_attrs_internal(tui: *mut TuiHandle, attr_id: c_int);
    fn nvim_tui_invalidate_region(
        tui: *mut TuiHandle,
        top: c_int,
        bot: c_int,
        left: c_int,
        right: c_int,
    );
    fn nvim_tui_ugrid_scroll(
        tui: *mut TuiHandle,
        top: c_int,
        bot: c_int,
        left: c_int,
        right: c_int,
        rows: c_int,
    );

    // Stopped and title accessors
    fn nvim_tui_get_stopped(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_get_can_set_title(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_get_title_enabled(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_set_title_enabled(tui: *mut TuiHandle, enabled: bool);
    fn nvim_tui_get_buf_space(tui: *mut TuiHandle) -> usize;

    // Extended underline accessors
    fn nvim_tui_set_can_set_underline_color(tui: *mut TuiHandle, val: bool);
    fn nvim_tui_terminfo_set_underline_style(tui: *mut TuiHandle);
}

/// Set cursor position for the grid.
///
/// This function stores the cursor row and column position in the TUIData struct.
/// The actual cursor movement happens during tui_flush.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_grid_cursor_goto(tui: *mut TuiHandle, row: i64, col: i64) {
    if tui.is_null() {
        return;
    }

    // cursor position is validated in tui_flush
    nvim_tui_set_row(tui, row as c_int);
    nvim_tui_set_col(tui, col as c_int);
}

// ============================================================================
// Highlight Attribute Definition
// ============================================================================

/// Store highlight attributes in the TUI attributes array.
///
/// This function merges RGB and cterm attributes and stores them at the
/// specified index in the TUI's highlight attribute array.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_hl_attr_define(
    tui: *mut TuiHandle,
    id: i64,
    mut attrs: HlAttrs,
    cterm_attrs: HlAttrs,
) {
    if tui.is_null() {
        return;
    }

    // Merge cterm attributes into the main attrs struct
    attrs.cterm_ae_attr = cterm_attrs.cterm_ae_attr;
    attrs.cterm_fg_color = cterm_attrs.cterm_fg_color;
    attrs.cterm_bg_color = cterm_attrs.cterm_bg_color;

    nvim_tui_set_attrs(tui, id as usize, attrs);
}

// ============================================================================
// Default Colors
// ============================================================================

/// Set default colors and invalidate the entire grid.
///
/// This function sets the clear_attrs used for background clearing,
/// resets print_attr_id to force attribute re-emission, and invalidates
/// the entire grid so it will be redrawn with the new colors.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_default_colors_set(
    tui: *mut TuiHandle,
    rgb_fg: i64,
    rgb_bg: i64,
    rgb_sp: i64,
    cterm_fg: i64,
    cterm_bg: i64,
) {
    if tui.is_null() {
        return;
    }

    // Build the clear_attrs struct
    let clear_attrs = HlAttrs {
        rgb_ae_attr: 0,
        cterm_ae_attr: 0,
        rgb_fg_color: rgb_fg as i32,
        rgb_bg_color: rgb_bg as i32,
        rgb_sp_color: rgb_sp as i32,
        cterm_fg_color: cterm_fg as i16,
        cterm_bg_color: cterm_bg as i16,
        hl_blend: -1,
        url: -1,
    };

    nvim_tui_set_clear_attrs(tui, clear_attrs);
    nvim_tui_set_print_attr_id(tui, -1);
    nvim_tui_set_default_colors_flag(tui, true);

    // Invalidate entire grid
    let height = nvim_tui_get_grid_height(tui);
    let width = nvim_tui_get_grid_width(tui);
    nvim_tui_invalidate(tui, 0, height, 0, width);
}

// ============================================================================
// Grid Resize
// ============================================================================

/// Resize the TUI grid.
///
/// This function resizes the internal grid and clips any invalid regions to the
/// new bounds. If this is not a startup resize and there are no pending resize
/// events, it sends an escape sequence to resize the host terminal.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_grid_resize(tui: *mut TuiHandle, _g: i64, width: i64, height: i64) {
    if tui.is_null() {
        return;
    }

    let grid = nvim_tui_get_grid(tui);
    ugrid_resize(grid, width as c_int, height as c_int);

    // Get new grid dimensions (ugrid_resize updates these)
    let grid_height = nvim_tui_get_grid_height(tui);
    let grid_width = nvim_tui_get_grid_width(tui);

    // resize might not always be followed by a clear before flush
    // so clip the invalid regions
    let num_regions = nvim_tui_get_invalid_regions_size(tui);
    for i in 0..num_regions {
        nvim_tui_clip_invalid_region(tui, i, grid_height, grid_width);
    }

    let pending = nvim_tui_get_pending_resize_events(tui);
    let is_starting = nvim_tui_get_is_starting(tui);

    if pending == 0 && !is_starting {
        // Resize the _host_ terminal
        nvim_tui_out_resize(tui, height as c_int, width as c_int);
    } else {
        // Already handled the resize; avoid double-resize
        let new_pending = if pending > 0 { pending - 1 } else { 0 };
        nvim_tui_set_pending_resize_events(tui, new_pending);
        nvim_tui_invalidate_grid_cursor(tui);
    }
}

// ============================================================================
// Grid Clear
// ============================================================================

/// Clear the TUI grid.
///
/// This function clears the internal grid data, clears the schar cache if full,
/// removes all invalid regions, and clears the entire screen region.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_grid_clear(tui: *mut TuiHandle, _g: i64) {
    if tui.is_null() {
        return;
    }

    let grid = nvim_tui_get_grid(tui);
    ugrid_clear(grid);

    // safe to clear cache at this point
    schar_cache_clear_if_full();

    nvim_tui_clear_invalid_regions(tui);

    let height = nvim_tui_get_height(tui);
    let width = nvim_tui_get_width(tui);
    nvim_tui_clear_region(tui, 0, height, 0, width, 0);
}

// ============================================================================
// Busy State
// ============================================================================

/// Mark TUI as busy (cursor hidden during output).
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_busy_start(tui: *mut TuiHandle) {
    if tui.is_null() {
        return;
    }
    nvim_tui_set_busy(tui, true);
}

/// Mark TUI as not busy (cursor can be shown).
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_busy_stop(tui: *mut TuiHandle) {
    if tui.is_null() {
        return;
    }
    nvim_tui_set_busy(tui, false);
}

// ============================================================================
// Bell
// ============================================================================

/// Ring the terminal bell (output '\a').
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_bell(tui: *mut TuiHandle) {
    if tui.is_null() {
        return;
    }
    // Bell character: '\a' = 0x07
    let bell: [u8; 1] = [0x07];
    nvim_tui_out(tui, bell.as_ptr(), 1);
}

// ============================================================================
// Icon (stub)
// ============================================================================

/// Set terminal icon (stub - not implemented).
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_set_icon(_tui: *mut TuiHandle) {
    // Icon setting is not implemented in TUI - intentionally empty
}

/// Update menu (stub - menus are for GUI only).
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_update_menu(_tui: *mut TuiHandle) {
    // Menus are for GUI only - intentionally empty
}

// ============================================================================
// Mouse Control
// ============================================================================

// Terminal mode constants
const TERM_MODE_MOUSE_BUTTON_EVENT: c_int = 1002;
const TERM_MODE_MOUSE_ANY_EVENT: c_int = 1003;
const TERM_MODE_MOUSE_SGR_EXT: c_int = 1006;

/// Enable mouse tracking.
///
/// This function enables mouse button and SGR extended mode escape sequences.
/// If mouse move tracking is also enabled, it enables mouse any event mode.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_mouse_on(tui: *mut TuiHandle) {
    if tui.is_null() {
        return;
    }

    if !nvim_tui_get_mouse_enabled(tui) {
        nvim_tui_set_term_mode(tui, TERM_MODE_MOUSE_BUTTON_EVENT, true);
        nvim_tui_set_term_mode(tui, TERM_MODE_MOUSE_SGR_EXT, true);
        if nvim_tui_get_mouse_move_enabled(tui) {
            nvim_tui_set_term_mode(tui, TERM_MODE_MOUSE_ANY_EVENT, true);
        }
        nvim_tui_set_mouse_enabled(tui, true);
    }
}

/// Disable mouse tracking.
///
/// This function disables mouse button and SGR extended mode escape sequences.
/// If mouse move tracking is enabled, it also disables mouse any event mode.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_mouse_off(tui: *mut TuiHandle) {
    if tui.is_null() {
        return;
    }

    if nvim_tui_get_mouse_enabled(tui) {
        if nvim_tui_get_mouse_move_enabled(tui) {
            nvim_tui_set_term_mode(tui, TERM_MODE_MOUSE_ANY_EVENT, false);
        }
        nvim_tui_set_term_mode(tui, TERM_MODE_MOUSE_BUTTON_EVENT, false);
        nvim_tui_set_term_mode(tui, TERM_MODE_MOUSE_SGR_EXT, false);
        nvim_tui_set_mouse_enabled(tui, false);
    }
}

// ============================================================================
// Scroll Region Functions
// ============================================================================

// Terminfo definition constants
const TERM_CHANGE_SCROLL_REGION: c_int = 1; // kTerm_change_scroll_region
const TERM_SET_LR_MARGIN: c_int = 42; // kTerm_set_lr_margin

// Terminal mode constants for scroll regions
const TERM_MODE_LEFT_RIGHT_MARGINS: c_int = 69; // kTermModeLeftAndRightMargins

/// Set the scroll region for the terminal.
///
/// This sets both the vertical scroll region (top/bot) and optionally
/// the horizontal margins (left/right) if they differ from full width.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_set_scroll_region(
    tui: *mut TuiHandle,
    top: c_int,
    bot: c_int,
    left: c_int,
    right: c_int,
) {
    if tui.is_null() {
        return;
    }

    // Set vertical scroll region
    nvim_tui_terminfo_print_num2(tui, TERM_CHANGE_SCROLL_REGION, top, bot);

    // Set horizontal margins if not full width
    let width = nvim_tui_get_width(tui);
    if left != 0 || right != width - 1 {
        nvim_tui_set_term_mode(tui, TERM_MODE_LEFT_RIGHT_MARGINS, true);
        nvim_tui_terminfo_print_num2(tui, TERM_SET_LR_MARGIN, left, right);
    }

    // Invalidate cursor position
    nvim_tui_invalidate_grid_cursor(tui);
}

/// Reset the scroll region to full screen.
///
/// This resets the scroll region after scrolling operations. If fullwidth is
/// false, it also resets the horizontal margins.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_reset_scroll_region(tui: *mut TuiHandle, fullwidth: bool) {
    if tui.is_null() {
        return;
    }

    // Check if we have a custom reset_scroll_region sequence
    let reset_seq = nvim_tui_get_reset_scroll_region(tui);
    if !reset_seq.is_null() {
        nvim_tui_out_len(tui, reset_seq);
    } else {
        // Use standard scroll region reset: top=0, bot=height-1
        let height = nvim_tui_get_height(tui);
        nvim_tui_terminfo_print_num2(tui, TERM_CHANGE_SCROLL_REGION, 0, height - 1);
    }

    // Reset horizontal margins if not fullwidth
    if !fullwidth {
        let width = nvim_tui_get_width(tui);
        nvim_tui_terminfo_print_num2(tui, TERM_SET_LR_MARGIN, 0, width - 1);
        nvim_tui_set_term_mode(tui, TERM_MODE_LEFT_RIGHT_MARGINS, false);
    }

    // Invalidate cursor position
    nvim_tui_invalidate_grid_cursor(tui);
}

// ============================================================================
// Final Column Wrap
// ============================================================================

/// Handle cursor wrapping at the final column.
///
/// When printing at the right margin, the cursor stays in place until the
/// next character is printed (in most terminals). This function handles the
/// wrap when we're at the final column.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_final_column_wrap(tui: *mut TuiHandle) {
    if tui.is_null() {
        return;
    }

    let row = nvim_tui_get_grid_row(tui);
    let col = nvim_tui_get_grid_col(tui);
    let width = nvim_tui_get_width(tui);

    if row != -1 && col == width {
        let grid = nvim_tui_get_grid(tui);
        let height = nvim_tui_get_height(tui);
        let grid_height = nvim_tui_get_grid_height(tui);
        let max_row = std::cmp::min(height, grid_height - 1);
        let new_row = if row < max_row { row + 1 } else { row };
        ugrid_goto(grid, new_row, 0);
    }
}

// ============================================================================
// Print Cell
// ============================================================================

/// Print a cell to the terminal output.
///
/// This function handles cursor wrapping at margins depending on the terminal's
/// immediate_wrap_after_last_column setting, updates attributes, outputs the
/// cell content, and advances the grid column.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
/// - `buf` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_print_cell(tui: *mut TuiHandle, buf: *const u8, attr: i16) {
    if tui.is_null() || buf.is_null() {
        return;
    }

    let immediate_wrap = nvim_tui_get_immediate_wrap(tui);

    if !immediate_wrap {
        // Printing the next character finally advances the cursor.
        rs_final_column_wrap(tui);
    }

    nvim_tui_update_attrs(tui, attr as c_int);

    // Calculate string length and output
    let len = libc::strlen(buf as *const libc::c_char);
    nvim_tui_out(tui, buf, len);

    // Advance grid column
    nvim_tui_inc_grid_col(tui);

    if immediate_wrap {
        // Printing at the right margin immediately advances the cursor.
        rs_final_column_wrap(tui);
    }
}

// ============================================================================
// Visual Bell
// ============================================================================

/// Trigger a visual bell effect.
///
/// For screen/tmux terminals, outputs the screen flash escape sequence.
/// For other terminals, temporarily inverts the video mode for 100ms.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_visual_bell(tui: *mut TuiHandle) {
    if tui.is_null() {
        return;
    }

    if nvim_tui_get_screen_or_tmux(tui) {
        // Screen/tmux: use the g (visual bell) sequence
        let seq = b"\x1bg";
        nvim_tui_out(tui, seq.as_ptr(), seq.len());
    } else {
        // Other terminals: invert video mode briefly
        let start_seq = b"\x1b[?5h";
        nvim_tui_out(tui, start_seq.as_ptr(), start_seq.len());

        nvim_tui_flush_buf(tui);
        nvim_tui_uv_sleep(100); // typically 100 or 200 in terminfo

        let end_seq = b"\x1b[?5l";
        nvim_tui_out(tui, end_seq.as_ptr(), end_seq.len());
    }

    nvim_tui_flush_buf(tui);
}

// ============================================================================
// Grid Scroll
// ============================================================================

// Terminfo definition constants for scroll
const TERM_DELETE_LINE: c_int = 13; // kTerm_delete_line
const TERM_INSERT_LINE: c_int = 24; // kTerm_insert_line
const TERM_PARM_DELETE_LINE: c_int = 27; // kTerm_parm_delete_line
const TERM_PARM_INSERT_LINE: c_int = 29; // kTerm_parm_insert_line

/// Scroll a region of the TUI grid.
///
/// This function scrolls a rectangular region of the grid by the specified
/// number of rows. Positive `rows` scrolls up (deletes lines), negative
/// scrolls down (inserts lines).
///
/// If the terminal supports scrolling, it uses efficient terminal scroll
/// sequences. Otherwise, it marks the affected region as invalid for
/// later redrawing.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_grid_scroll(
    tui: *mut TuiHandle,
    _g: i64,
    startrow: i64,
    endrow: i64,
    startcol: i64,
    endcol: i64,
    rows: i64,
    _cols: i64,
) {
    if tui.is_null() {
        return;
    }

    let top = startrow as c_int;
    let bot = (endrow - 1) as c_int;
    let left = startcol as c_int;
    let right = (endcol - 1) as c_int;

    let width = nvim_tui_get_width(tui);
    let height = nvim_tui_get_height(tui);

    let fullwidth = left == 0 && right == width - 1;
    let full_screen_scroll = fullwidth && top == 0 && bot == height - 1;

    // Scroll the grid data
    nvim_tui_ugrid_scroll(tui, top, bot, left, right, rows as c_int);

    // Check if we can use terminal scroll capabilities
    let has_lr_margins =
        nvim_tui_get_has_lr_margin_mode(tui) && nvim_tui_get_can_set_lr_margin(tui);
    let can_scroll = nvim_tui_get_can_scroll(tui)
        && (full_screen_scroll
            || (nvim_tui_get_can_change_scroll_region(tui)
                && ((left == 0 && right == width - 1) || has_lr_margins)));

    if can_scroll {
        // Change terminal scroll region and move cursor to the top
        if !full_screen_scroll {
            rs_set_scroll_region(tui, top, bot, left, right);
        }
        nvim_tui_cursor_goto_internal(tui, top, left);
        nvim_tui_update_attrs_internal(tui, 0);

        if rows > 0 {
            if rows == 1 {
                nvim_tui_terminfo_out(tui, TERM_DELETE_LINE);
            } else {
                nvim_tui_terminfo_print_num1(tui, TERM_PARM_DELETE_LINE, rows as c_int);
            }
        } else if rows == -1 {
            nvim_tui_terminfo_out(tui, TERM_INSERT_LINE);
        } else {
            nvim_tui_terminfo_print_num1(tui, TERM_PARM_INSERT_LINE, (-rows) as c_int);
        }

        // Restore terminal scroll region and cursor
        if !full_screen_scroll {
            rs_reset_scroll_region(tui, fullwidth);
        }
    } else {
        // Mark the moved region as invalid for redrawing later
        let (inv_startrow, inv_endrow) = if rows > 0 {
            (startrow as c_int, (endrow - rows) as c_int)
        } else {
            ((startrow - rows) as c_int, endrow as c_int)
        };
        nvim_tui_invalidate_region(
            tui,
            inv_startrow,
            inv_endrow,
            startcol as c_int,
            endcol as c_int,
        );
    }
}

// ============================================================================
// TUI State Queries
// ============================================================================

/// Check if TUI has been stopped.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_is_stopped(tui: *mut TuiHandle) -> bool {
    if tui.is_null() {
        return true; // Null TUI is considered stopped
    }
    nvim_tui_get_stopped(tui)
}

// ============================================================================
// Terminal Title
// ============================================================================

// Terminfo definition constants for title
const TERM_TO_STATUS_LINE: c_int = 55; // kTerm_to_status_line
const TERM_FROM_STATUS_LINE: c_int = 19; // kTerm_from_status_line

// TERMINFO_SEQ_LIMIT from tui.c
const TERMINFO_SEQ_LIMIT: usize = 128;

/// Set terminal title.
///
/// This function sets the terminal title to the given string. If the terminal
/// doesn't support title setting, this is a no-op. If the title is too long
/// (>4096 bytes), it's ignored.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
/// - `data` must be a valid pointer to `size` bytes (or null if size is 0)
#[no_mangle]
pub unsafe extern "C" fn rs_tui_set_title(tui: *mut TuiHandle, data: *const u8, size: usize) {
    if tui.is_null() {
        return;
    }

    // Check if terminal supports setting title
    if !nvim_tui_get_can_set_title(tui) {
        return;
    }

    // Check for too long title (> 4096 bytes)
    let too_long = size > 4096;
    if too_long {
        // In C this logs: ELOG("set_title: title string too long!");
        // We skip logging in Rust and just return
        return;
    }

    if size > 0 && !data.is_null() {
        // If title was not enabled, save current title to the "stack"
        if !nvim_tui_get_title_enabled(tui) {
            let save_seq = b"\x1b[22;0t";
            nvim_tui_out(tui, save_seq.as_ptr(), save_seq.len());
            nvim_tui_set_title_enabled(tui, true);
        }

        // Check if we need to flush buffer before writing title
        // Title sequence cannot be cut in half
        let buf_space = nvim_tui_get_buf_space(tui);
        if buf_space < size + 2 * TERMINFO_SEQ_LIMIT {
            nvim_tui_flush_buf(tui);
        }

        // Output: to_status_line + title + from_status_line
        nvim_tui_terminfo_out(tui, TERM_TO_STATUS_LINE);
        nvim_tui_out(tui, data, size);
        nvim_tui_terminfo_out(tui, TERM_FROM_STATUS_LINE);
    } else if nvim_tui_get_title_enabled(tui) {
        // Restore title from the "stack"
        let restore_seq = b"\x1b[23;0t";
        nvim_tui_out(tui, restore_seq.as_ptr(), restore_seq.len());
        nvim_tui_set_title_enabled(tui, false);
    }
}

// ============================================================================
// Extended Underline Support
// ============================================================================

/// Enable extended underline support.
///
/// Sets up the terminal to use extended underline styles (wavy, dotted, etc.)
/// and enables underline color support.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_enable_extended_underline(tui: *mut TuiHandle) {
    if tui.is_null() {
        return;
    }

    // Set the underline style terminfo capability if not already set
    nvim_tui_terminfo_set_underline_style(tui);

    // Enable underline color support
    nvim_tui_set_can_set_underline_color(tui, true);
}

/// Query the terminal background color using OSC 11 escape sequence.
///
/// This sends the query "\x1b]11;?\x07" to the terminal which requests
/// the current background color. The terminal will respond with the color.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_query_bg_color(tui: *mut TuiHandle) {
    if tui.is_null() {
        return;
    }

    // OSC 11 query: ESC ] 11 ; ? BEL
    // This asks the terminal for its current background color
    const QUERY_BG_COLOR: &[u8] = b"\x1b]11;?\x07";
    nvim_tui_out(tui, QUERY_BG_COLOR.as_ptr(), QUERY_BG_COLOR.len());
    nvim_tui_flush_buf(tui);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_attrs(count: usize) -> Vec<HlAttrs> {
        vec![HlAttrs::new(); count]
    }

    #[test]
    fn test_same_id() {
        let attrs = make_attrs(5);
        unsafe {
            assert!(!attrs_differ_impl(2, 2, true, attrs.as_ptr(), attrs.len()));
            assert!(!attrs_differ_impl(2, 2, false, attrs.as_ptr(), attrs.len()));
        }
    }

    #[test]
    fn test_negative_id() {
        let attrs = make_attrs(5);
        unsafe {
            assert!(attrs_differ_impl(-1, 2, true, attrs.as_ptr(), attrs.len()));
            assert!(attrs_differ_impl(2, -1, true, attrs.as_ptr(), attrs.len()));
            assert!(attrs_differ_impl(-1, -1, true, attrs.as_ptr(), attrs.len()));
        }
    }

    #[test]
    fn test_out_of_bounds() {
        let attrs = make_attrs(5);
        unsafe {
            assert!(attrs_differ_impl(10, 2, true, attrs.as_ptr(), attrs.len()));
            assert!(attrs_differ_impl(2, 10, true, attrs.as_ptr(), attrs.len()));
        }
    }

    #[test]
    fn test_identical_attrs() {
        let attrs = make_attrs(5);
        unsafe {
            // All default attrs are the same
            assert!(!attrs_differ_impl(0, 1, true, attrs.as_ptr(), attrs.len()));
            assert!(!attrs_differ_impl(0, 1, false, attrs.as_ptr(), attrs.len()));
        }
    }

    #[test]
    fn test_different_rgb_fg() {
        let mut attrs = make_attrs(5);
        attrs[0].rgb_fg_color = 0xFF0000; // red
        attrs[1].rgb_fg_color = 0x00FF00; // green
        unsafe {
            assert!(attrs_differ_impl(0, 1, true, attrs.as_ptr(), attrs.len()));
            // In cterm mode, RGB colors don't matter
            assert!(!attrs_differ_impl(0, 1, false, attrs.as_ptr(), attrs.len()));
        }
    }

    #[test]
    fn test_different_cterm_fg() {
        let mut attrs = make_attrs(5);
        attrs[0].cterm_fg_color = 1;
        attrs[1].cterm_fg_color = 2;
        unsafe {
            // In RGB mode, cterm colors don't matter
            assert!(!attrs_differ_impl(0, 1, true, attrs.as_ptr(), attrs.len()));
            assert!(attrs_differ_impl(0, 1, false, attrs.as_ptr(), attrs.len()));
        }
    }

    #[test]
    fn test_different_url() {
        let mut attrs = make_attrs(5);
        attrs[0].url = 0;
        attrs[1].url = 1;
        unsafe {
            // URL always matters regardless of RGB mode
            assert!(attrs_differ_impl(0, 1, true, attrs.as_ptr(), attrs.len()));
            assert!(attrs_differ_impl(0, 1, false, attrs.as_ptr(), attrs.len()));
        }
    }

    #[test]
    fn test_underline_sp_color() {
        let mut attrs = make_attrs(5);
        // Set underline attribute and different sp_color
        attrs[0].cterm_ae_attr = HL_UNDERLINE_MASK;
        attrs[0].rgb_sp_color = 0xFF0000;
        attrs[1].cterm_ae_attr = HL_UNDERLINE_MASK;
        attrs[1].rgb_sp_color = 0x00FF00;
        unsafe {
            // In cterm mode with underline, sp_color matters
            assert!(attrs_differ_impl(0, 1, false, attrs.as_ptr(), attrs.len()));
        }

        // Without underline, sp_color doesn't matter in cterm mode
        attrs[0].cterm_ae_attr = 0;
        attrs[1].cterm_ae_attr = 0;
        unsafe {
            assert!(!attrs_differ_impl(0, 1, false, attrs.as_ptr(), attrs.len()));
        }
    }
}
