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
    // Negative IDs indicate special/missing attributes - always differ
    // (even if both are the same negative value)
    if id1 < 0 || id2 < 0 {
        return true;
    }

    // Same ID means same attributes (only for non-negative IDs)
    if id1 == id2 {
        return false;
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
    fn nvim_tui_update_attrs(tui: *mut TuiHandle, attr_id: c_int);
    fn nvim_tui_get_can_clear_attr(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_get_can_erase_chars(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_get_set_default_colors(tui: *mut TuiHandle) -> bool;
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

// Terminfo definition constants (0-based indices into tui->ti.defs[])
// Values computed as: file_line_number - 6 from terminfo_enum_defs.h
const TERM_CHANGE_SCROLL_REGION: c_int = 1; // kTerm_change_scroll_region
const TERM_SET_LR_MARGIN: c_int = 36; // kTerm_set_lr_margin (line 42 in enum file)

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
#[unsafe(export_name = "tui_is_stopped")]
pub unsafe extern "C" fn rs_tui_is_stopped(tui: *mut TuiHandle) -> bool {
    if tui.is_null() {
        return true; // Null TUI is considered stopped
    }
    nvim_tui_get_stopped(tui)
}

// ============================================================================
// Terminal Title
// ============================================================================

// Terminfo definition constants for title (0-based indices, value = line - 6)
const TERM_TO_STATUS_LINE: c_int = 37; // kTerm_to_status_line (line 43)
const TERM_FROM_STATUS_LINE: c_int = 23; // kTerm_from_status_line (line 29)

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

// ============================================================================
// Phase 2: Output Infrastructure
// ============================================================================

// Use TpVar from crate root (lib.rs)
use crate::TpVar;

// Additional C accessors needed for output infrastructure
extern "C" {
    fn nvim_tui_get_bufpos(tui: *mut TuiHandle) -> usize;
    fn nvim_tui_set_bufpos(tui: *mut TuiHandle, pos: usize);
    fn nvim_tui_get_buf_ptr(tui: *mut TuiHandle) -> *mut u8;
    fn nvim_tui_get_buf_capacity() -> usize;
    fn nvim_tui_set_buf_to_flush(tui: *mut TuiHandle, ptr: *mut u8);
    fn nvim_tui_get_ti_def(tui: *mut TuiHandle, idx: c_int) -> *const u8;
}

/// Write bytes to the TUI output buffer.
///
/// If the bytes don't fit, the current buffer is flushed first.
/// For very large writes (> buffer capacity), writes directly to TTY.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
/// - `str_ptr` must be a valid pointer to `len` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_out(tui: *mut TuiHandle, str_ptr: *const u8, len: usize) {
    if tui.is_null() || str_ptr.is_null() {
        return;
    }

    let bufpos = nvim_tui_get_bufpos(tui);
    let capacity = nvim_tui_get_buf_capacity();
    let available = capacity - bufpos;

    if len > available {
        rs_flush_buf(tui);
        let capacity = nvim_tui_get_buf_capacity();
        if len > capacity {
            // String too large for buffer: write directly
            nvim_tui_set_buf_to_flush(tui, str_ptr as *mut u8);
            nvim_tui_set_bufpos(tui, len);
            rs_flush_buf(tui);
            return;
        }
    }

    // Copy into buffer
    let bufpos = nvim_tui_get_bufpos(tui);
    let buf_ptr = nvim_tui_get_buf_ptr(tui);
    std::ptr::copy_nonoverlapping(str_ptr, buf_ptr.add(bufpos), len);
    nvim_tui_set_bufpos(tui, bufpos + len);
}

/// Write a null-terminated C string to the output buffer.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
/// - `str_ptr` must be a valid null-terminated C string (or null, in which case it's a no-op)
#[no_mangle]
pub unsafe extern "C" fn rs_out_len(tui: *mut TuiHandle, str_ptr: *const u8) {
    if tui.is_null() || str_ptr.is_null() {
        return;
    }
    let len = libc::strlen(str_ptr as *const libc::c_char);
    rs_out(tui, str_ptr, len);
}

/// Flush the output buffer to the TTY.
///
/// This writes pre-flush sequences (cursor hiding/sync), the buffered output,
/// and post-flush sequences (cursor showing/sync). If screenshot mode is active,
/// writes to the screenshot file instead.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_flush_buf(tui: *mut TuiHandle) {
    if tui.is_null() {
        return;
    }
    nvim_tui_flush_buf(tui);
}

/// Emit a terminfo sequence with no parameters.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_terminfo_out(tui: *mut TuiHandle, what: c_int) {
    rs_terminfo_print_impl(tui, what, 0, 0, 0);
}

/// Emit a terminfo sequence with 1-3 numeric parameters.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_terminfo_print_num(
    tui: *mut TuiHandle,
    what: c_int,
    num1: c_int,
    num2: c_int,
    num3: c_int,
) {
    rs_terminfo_print_impl(tui, what, num1, num2, num3);
}

unsafe fn rs_terminfo_print_impl(
    tui: *mut TuiHandle,
    what: c_int,
    num1: c_int,
    num2: c_int,
    num3: c_int,
) {
    extern "C" {
        fn rs_terminfo_fmt(
            buf_start: *mut libc::c_char,
            buf_end: *const libc::c_char,
            str_ptr: *const libc::c_char,
            params: *mut TpVar,
        ) -> usize;
    }

    let str_ptr = nvim_tui_get_ti_def(tui, what);
    if str_ptr.is_null() {
        return;
    }
    // Check for empty string
    if *str_ptr == 0 {
        return;
    }

    let mut params = [TpVar {
        num: 0,
        string: std::ptr::null_mut(),
    }; 9];
    params[0].num = num1 as libc::c_long;
    params[1].num = num2 as libc::c_long;
    params[2].num = num3 as libc::c_long;

    let bufpos = nvim_tui_get_bufpos(tui);
    let capacity = nvim_tui_get_buf_capacity();
    let buf_ptr = nvim_tui_get_buf_ptr(tui);

    if capacity - bufpos > TERMINFO_SEQ_LIMIT {
        let mut copy_params = [TpVar {
            num: 0,
            string: std::ptr::null_mut(),
        }; 9];
        for i in 0..9 {
            copy_params[i].num = params[i].num;
            copy_params[i].string = params[i].string;
        }
        let len = rs_terminfo_fmt(
            buf_ptr.add(bufpos) as *mut libc::c_char,
            buf_ptr.add(capacity) as *const libc::c_char,
            str_ptr as *const libc::c_char,
            copy_params.as_mut_ptr(),
        );
        if len > 0 {
            nvim_tui_set_bufpos(tui, bufpos + len);
            return;
        }
    }

    // Try again with fresh buffer
    rs_flush_buf(tui);
    let bufpos = nvim_tui_get_bufpos(tui);
    let buf_ptr = nvim_tui_get_buf_ptr(tui);
    let capacity = nvim_tui_get_buf_capacity();
    let len = rs_terminfo_fmt(
        buf_ptr.add(bufpos) as *mut libc::c_char,
        buf_ptr.add(capacity) as *const libc::c_char,
        str_ptr as *const libc::c_char,
        params.as_mut_ptr(),
    );
    if len > 0 {
        nvim_tui_set_bufpos(tui, bufpos + len);
    }
}

/// Write spaces to the output, filling up to `width` columns.
///
/// This writes to the output buffer directly for efficiency, flushing if needed.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_print_spaces(tui: *mut TuiHandle, width: c_int) {
    if tui.is_null() || width <= 0 {
        return;
    }

    let mut left = width as usize;
    let capacity = nvim_tui_get_buf_capacity();

    loop {
        let bufpos = nvim_tui_get_bufpos(tui);
        let buf_ptr = nvim_tui_get_buf_ptr(tui);
        let buf_fit = left.min(capacity - bufpos);
        std::ptr::write_bytes(buf_ptr.add(bufpos), b' ', buf_fit);
        nvim_tui_set_bufpos(tui, bufpos + buf_fit);
        left -= buf_fit;

        if left == 0 {
            break;
        }
        rs_flush_buf(tui);
    }

    // Advance grid column (done by caller in C or by the Rust implementation)
    // Note: grid->col += width and final_column_wrap are handled by caller
}

// ============================================================================
// Phase 3: Core Rendering Functions
// ============================================================================

use nvim_highlight::hl_attr_flags::*;

// Phase 3 C accessors (new, not yet declared above)
extern "C" {
    fn nvim_tui_get_rgb(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_get_bce(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_get_can_set_underline_color(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_get_clear_attrs(tui: *mut TuiHandle) -> HlAttrs;
    fn nvim_tui_set_default_attr(tui: *mut TuiHandle, val: bool);
    fn nvim_tui_set_can_clear_attr(tui: *mut TuiHandle, val: bool);
    fn nvim_tui_get_attrs_entry(tui: *mut TuiHandle, idx: usize) -> HlAttrs;
    fn nvim_tui_get_attrs_size(tui: *mut TuiHandle) -> usize;
    fn nvim_tui_get_attrs_ptr(tui: *mut TuiHandle) -> *const HlAttrs;
    fn nvim_tui_ti_has_def(tui: *mut TuiHandle, idx: c_int) -> bool;
    fn nvim_tui_get_cell_data(tui: *mut TuiHandle, row: c_int, col: c_int) -> u32;
    fn nvim_tui_get_cell_attr(tui: *mut TuiHandle, row: c_int, col: c_int) -> i32;
    fn nvim_tui_get_url_key(idx: c_int) -> *const libc::c_char;
    fn nvim_tui_urlbuf_reset(tui: *mut TuiHandle);
    fn nvim_tui_urlbuf_append_fmt(tui: *mut TuiHandle, id: u64, url: *const libc::c_char);
    fn nvim_tui_urlbuf_ptr(tui: *mut TuiHandle) -> *const u8;
    fn nvim_tui_urlbuf_size(tui: *mut TuiHandle) -> usize;
    fn nvim_tui_terminfo_print_num3(
        tui: *mut TuiHandle,
        what: c_int,
        n1: c_int,
        n2: c_int,
        n3: c_int,
    );
    fn nvim_tui_out_underline_color(tui: *mut TuiHandle, r: c_int, g: c_int, b: c_int);
    fn nvim_tui_get_invalid_region(
        tui: *mut TuiHandle,
        idx: usize,
        top: *mut c_int,
        bot: *mut c_int,
        left: *mut c_int,
        right: *mut c_int,
    );
    fn nvim_tui_set_invalid_region(
        tui: *mut TuiHandle,
        idx: usize,
        top: c_int,
        bot: c_int,
        left: c_int,
        right: c_int,
    );
    fn nvim_tui_push_invalid_region(
        tui: *mut TuiHandle,
        top: c_int,
        bot: c_int,
        left: c_int,
        right: c_int,
    );
    fn nvim_tui_set_grid_col(tui: *mut TuiHandle, col: c_int);
    fn nvim_tui_terminfo_print_attrs(
        tui: *mut TuiHandle,
        standout: c_int,
        underline: c_int,
        reverse: c_int,
        blink: c_int,
        dim: c_int,
        bold: c_int,
        blank: c_int,
        protect: c_int,
        acs: c_int,
    );
    fn nvim_tui_out_altfont(tui: *mut TuiHandle);
}

// Terminfo enum constants (from terminfo_enum_defs.h)
const KTERM_CARRIAGE_RETURN: c_int = 0;
const KTERM_CLEAR_SCREEN: c_int = 2;
const KTERM_CLR_EOL: c_int = 3;
const KTERM_CLR_EOS: c_int = 4;
const KTERM_CURSOR_DOWN: c_int = 6;
const KTERM_CURSOR_LEFT: c_int = 8;
const KTERM_CURSOR_HOME: c_int = 9;
const KTERM_CURSOR_UP: c_int = 11;
const KTERM_CURSOR_RIGHT: c_int = 12;
const KTERM_ENTER_BOLD_MODE: c_int = 14;
const KTERM_ENTER_ITALICS_MODE: c_int = 16;
const KTERM_ENTER_REVERSE_MODE: c_int = 17;
const KTERM_ENTER_STANDOUT_MODE: c_int = 18;
const KTERM_ENTER_UNDERLINE_MODE: c_int = 19;
const KTERM_ERASE_CHARS: c_int = 20;
const KTERM_EXIT_ATTRIBUTE_MODE: c_int = 21;
const KTERM_SET_A_BACKGROUND: c_int = 33;
const KTERM_SET_A_FOREGROUND: c_int = 34;
const KTERM_SET_ATTRIBUTES: c_int = 35;
const KTERM_ENTER_STRIKETHROUGH_MODE: c_int = 40;
const KTERM_SET_RGB_FOREGROUND: c_int = 41;
const KTERM_SET_RGB_BACKGROUND: c_int = 42;
const KTERM_SET_UNDERLINE_STYLE: c_int = 45;
const KTERM_PARM_LEFT_CURSOR: c_int = 30;
const KTERM_PARM_RIGHT_CURSOR: c_int = 31;
const KTERM_PARM_DOWN_CURSOR: c_int = 28;
const KTERM_PARM_UP_CURSOR: c_int = 32;
const KTERM_CURSOR_ADDRESS: c_int = 5;

/// Mark a region of the grid as needing redraw.
///
/// If the region intersects an existing invalid region, they are merged.
/// Otherwise, a new entry is added.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_invalidate(
    tui: *mut TuiHandle,
    top: c_int,
    bot: c_int,
    left: c_int,
    right: c_int,
) {
    let n = nvim_tui_get_invalid_regions_size(tui);
    let mut intersect_idx: Option<usize> = None;

    for i in 0..n {
        let (mut r_top, mut r_bot, mut r_left, mut r_right) = (0i32, 0i32, 0i32, 0i32);
        nvim_tui_get_invalid_region(tui, i, &mut r_top, &mut r_bot, &mut r_left, &mut r_right);
        // Adjacent regions are treated as overlapping
        let overlaps_vert = !(top > r_bot || bot < r_top);
        let overlaps_horiz = !(left > r_right || right < r_left);
        if overlaps_vert && overlaps_horiz {
            intersect_idx = Some(i);
            break;
        }
    }

    if let Some(idx) = intersect_idx {
        let (mut r_top, mut r_bot, mut r_left, mut r_right) = (0i32, 0i32, 0i32, 0i32);
        nvim_tui_get_invalid_region(tui, idx, &mut r_top, &mut r_bot, &mut r_left, &mut r_right);
        nvim_tui_set_invalid_region(
            tui,
            idx,
            top.min(r_top),
            bot.max(r_bot),
            left.min(r_left),
            right.max(r_right),
        );
    } else {
        nvim_tui_push_invalid_region(tui, top, bot, left, right);
    }
}

/// Check if printing from `col` to `col+next` cells on `row` is cheap.
///
/// Cheap means no attribute changes are required (or only default attrs needed)
/// and all characters are ASCII.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_cheap_to_print(
    tui: *mut TuiHandle,
    row: c_int,
    col: c_int,
    mut next: c_int,
) -> bool {
    let attrs_ptr = nvim_tui_get_attrs_ptr(tui);
    let attrs_size = nvim_tui_get_attrs_size(tui);
    let print_attr_id = nvim_tui_get_print_attr_id(tui);
    let rgb = nvim_tui_get_rgb(tui);
    let default_attr = nvim_tui_get_default_attr(tui);

    let mut c = col;
    while next > 0 {
        next -= 1;
        let cell_attr = nvim_tui_get_cell_attr(tui, row, c);
        let cell_data = nvim_tui_get_cell_data(tui, row, c);
        c += 1;

        if attrs_differ_impl(cell_attr, print_attr_id, rgb, attrs_ptr, attrs_size) {
            if default_attr {
                return false;
            }
        }
        // schar_get_ascii returns 0 for non-ASCII characters
        extern "C" {
            fn schar_get_ascii(sc: u32) -> i8;
        }
        if schar_get_ascii(cell_data) == 0 {
            return false;
        }
    }
    true
}

/// Update terminal attributes to match the given highlight ID.
///
/// Emits terminfo sequences for bold, italic, underline, colors, etc.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_update_attrs(tui: *mut TuiHandle, attr_id: c_int) {
    let attrs_ptr = nvim_tui_get_attrs_ptr(tui);
    let attrs_size = nvim_tui_get_attrs_size(tui);
    let print_attr_id = nvim_tui_get_print_attr_id(tui);
    let rgb = nvim_tui_get_rgb(tui);

    if !attrs_differ_impl(attr_id, print_attr_id, rgb, attrs_ptr, attrs_size) {
        nvim_tui_set_print_attr_id(tui, attr_id);
        return;
    }
    nvim_tui_set_print_attr_id(tui, attr_id);

    let attrs = nvim_tui_get_attrs_entry(tui, attr_id as usize);
    let attr: i16 = if rgb {
        attrs.rgb_ae_attr
    } else {
        attrs.cterm_ae_attr
    };

    let bold = (attr & HL_BOLD) != 0;
    let italic = (attr & HL_ITALIC) != 0;
    let reverse = (attr & HL_INVERSE) != 0;
    let standout = (attr & HL_STANDOUT) != 0;
    let strikethrough = (attr & HL_STRIKETHROUGH) != 0;
    let altfont = (attr & HL_ALTFONT) != 0;

    let has_underline_style = nvim_tui_ti_has_def(tui, KTERM_SET_UNDERLINE_STYLE);
    let (underline, undercurl, underdouble, underdotted, underdashed) = if has_underline_style {
        let ul = attr & HL_UNDERLINE_MASK;
        (
            ul == HL_UNDERLINE,
            ul == HL_UNDERCURL,
            ul == HL_UNDERDOUBLE,
            ul == HL_UNDERDOTTED,
            ul == HL_UNDERDASHED,
        )
    } else {
        ((attr & HL_UNDERLINE_MASK) != 0, false, false, false, false)
    };

    let has_any_underline = undercurl || underline || underdouble || underdotted || underdashed;

    if nvim_tui_ti_has_def(tui, KTERM_SET_ATTRIBUTES) {
        if bold || reverse || underline || standout {
            // terminfo set_attributes: standout, underline, reverse, blink, dim, bold, blank, protect, acs
            nvim_tui_terminfo_print_attrs(
                tui,
                standout as c_int,
                underline as c_int,
                reverse as c_int,
                0,
                0,
                bold as c_int,
                0,
                0,
                0,
            );
        } else if !nvim_tui_get_default_attr(tui) {
            nvim_tui_terminfo_out(tui, KTERM_EXIT_ATTRIBUTE_MODE);
        }
    } else {
        if !nvim_tui_get_default_attr(tui) {
            nvim_tui_terminfo_out(tui, KTERM_EXIT_ATTRIBUTE_MODE);
        }
        if bold {
            nvim_tui_terminfo_out(tui, KTERM_ENTER_BOLD_MODE);
        }
        if underline {
            nvim_tui_terminfo_out(tui, KTERM_ENTER_UNDERLINE_MODE);
        }
        if standout {
            nvim_tui_terminfo_out(tui, KTERM_ENTER_STANDOUT_MODE);
        }
        if reverse {
            nvim_tui_terminfo_out(tui, KTERM_ENTER_REVERSE_MODE);
        }
    }

    if italic {
        nvim_tui_terminfo_out(tui, KTERM_ENTER_ITALICS_MODE);
    }
    if altfont {
        nvim_tui_out_altfont(tui);
    }
    if strikethrough {
        nvim_tui_terminfo_out(tui, KTERM_ENTER_STRIKETHROUGH_MODE);
    }

    if has_underline_style {
        if undercurl {
            nvim_tui_terminfo_print_num1(tui, KTERM_SET_UNDERLINE_STYLE, 3);
        }
        if underdouble {
            nvim_tui_terminfo_print_num1(tui, KTERM_SET_UNDERLINE_STYLE, 2);
        }
        if underdotted {
            nvim_tui_terminfo_print_num1(tui, KTERM_SET_UNDERLINE_STYLE, 4);
        }
        if underdashed {
            nvim_tui_terminfo_print_num1(tui, KTERM_SET_UNDERLINE_STYLE, 5);
        }
    }

    if has_any_underline && nvim_tui_get_can_set_underline_color(tui) {
        let color = attrs.rgb_sp_color;
        if color != -1 {
            let r = (color >> 16) & 0xff;
            let g = (color >> 8) & 0xff;
            let b = color & 0xff;
            nvim_tui_out_underline_color(tui, r, g, b);
        }
    }

    let clear_attrs = nvim_tui_get_clear_attrs(tui);

    // Foreground color
    let fg: i32 = if rgb && (attr & HL_FG_INDEXED) == 0 {
        let c = if attrs.rgb_fg_color != -1 {
            attrs.rgb_fg_color
        } else {
            clear_attrs.rgb_fg_color
        };
        if c != -1 {
            nvim_tui_terminfo_print_num3(
                tui,
                KTERM_SET_RGB_FOREGROUND,
                (c >> 16) & 0xff,
                (c >> 8) & 0xff,
                c & 0xff,
            );
        }
        c
    } else {
        let c = if attrs.cterm_fg_color != 0 {
            attrs.cterm_fg_color as i32 - 1
        } else {
            clear_attrs.cterm_fg_color as i32 - 1
        };
        if c != -1 {
            nvim_tui_terminfo_print_num1(tui, KTERM_SET_A_FOREGROUND, c);
        }
        c
    };

    // Background color
    let bg: i32 = if rgb && (attr & HL_BG_INDEXED) == 0 {
        let c = if attrs.rgb_bg_color != -1 {
            attrs.rgb_bg_color
        } else {
            clear_attrs.rgb_bg_color
        };
        if c != -1 {
            nvim_tui_terminfo_print_num3(
                tui,
                KTERM_SET_RGB_BACKGROUND,
                (c >> 16) & 0xff,
                (c >> 8) & 0xff,
                c & 0xff,
            );
        }
        c
    } else {
        let c = if attrs.cterm_bg_color != 0 {
            attrs.cterm_bg_color as i32 - 1
        } else {
            clear_attrs.cterm_bg_color as i32 - 1
        };
        if c != -1 {
            nvim_tui_terminfo_print_num1(tui, KTERM_SET_A_BACKGROUND, c);
        }
        c
    };

    // URL handling
    let url = nvim_tui_get_url(tui);
    if url != attrs.url {
        if attrs.url >= 0 {
            let url_str = nvim_tui_get_url_key(attrs.url);
            let id = 0xE1EA0000u64 + attrs.url as u64;
            nvim_tui_urlbuf_reset(tui);
            nvim_tui_urlbuf_append_fmt(tui, id, url_str);
            let ptr = nvim_tui_urlbuf_ptr(tui);
            let size = nvim_tui_urlbuf_size(tui);
            nvim_tui_out(tui, ptr, size);
        } else {
            nvim_tui_out(tui, b"\x1b]8;;\x1b\\".as_ptr(), 6);
        }
        nvim_tui_set_url(tui, attrs.url);
    }

    nvim_tui_set_default_attr(
        tui,
        fg == -1
            && bg == -1
            && !bold
            && !italic
            && !has_any_underline
            && !reverse
            && !standout
            && !strikethrough,
    );

    let bce = nvim_tui_get_bce(tui);
    nvim_tui_set_can_clear_attr(
        tui,
        !reverse && !standout && !has_any_underline && !strikethrough && (bce || bg == -1),
    );
}

/// Optimized cursor positioning with relative motion.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_goto(tui: *mut TuiHandle, row: c_int, col: c_int) {
    let grid_row = nvim_tui_get_grid_row(tui);
    let grid_col = nvim_tui_get_grid_col(tui);

    if row == grid_row && col == grid_col {
        return;
    }

    // If an OSC 8 sequence is active, terminate it before moving the cursor
    let url = nvim_tui_get_url(tui);
    if url >= 0 {
        nvim_tui_out(tui, b"\x1b]8;;\x1b\\".as_ptr(), 6);
        nvim_tui_set_url(tui, -1);
        nvim_tui_set_print_attr_id(tui, -1);
    }

    if row == 0 && col == 0 {
        nvim_tui_terminfo_out(tui, KTERM_CURSOR_HOME);
        let grid = nvim_tui_get_grid(tui);
        ugrid_goto(grid, row, col);
        return;
    }

    let grid = nvim_tui_get_grid(tui);
    if grid_row == -1 {
        // Unknown cursor position — use absolute positioning
        rs_cursor_goto_safe(tui, row, col);
        return;
    }

    // Try carriage return optimization: motion to left margin
    let needs_cr = if col == 0 {
        col != grid_col
    } else if row != grid_row {
        false
    } else if col == 1 {
        2 < grid_col && rs_cheap_to_print(tui, grid_row, 0, col)
    } else if col == 2 {
        5 < grid_col && rs_cheap_to_print(tui, grid_row, 0, col)
    } else {
        false
    };

    if needs_cr {
        nvim_tui_terminfo_out(tui, KTERM_CARRIAGE_RETURN);
        ugrid_goto(grid, grid_row, 0);
    }

    let grid_row = nvim_tui_get_grid_row(tui);
    let grid_col = nvim_tui_get_grid_col(tui);

    if row == grid_row {
        if col < grid_col
            && (nvim_tui_get_immediate_wrap(tui) || grid_col < nvim_tui_get_width(tui))
        {
            let n = grid_col - col;
            if n <= 4 {
                for _ in 0..n {
                    nvim_tui_terminfo_out(tui, KTERM_CURSOR_LEFT);
                }
            } else {
                nvim_tui_terminfo_print_num1(tui, KTERM_PARM_LEFT_CURSOR, n);
            }
            ugrid_goto(grid, row, col);
            return;
        } else if col > grid_col {
            let n = col - grid_col;
            if n <= 2 {
                for _ in 0..n {
                    nvim_tui_terminfo_out(tui, KTERM_CURSOR_RIGHT);
                }
            } else {
                nvim_tui_terminfo_print_num1(tui, KTERM_PARM_RIGHT_CURSOR, n);
            }
            ugrid_goto(grid, row, col);
            return;
        }
    }
    if col == grid_col {
        if row > grid_row {
            let n = row - grid_row;
            if n <= 4 {
                for _ in 0..n {
                    nvim_tui_terminfo_out(tui, KTERM_CURSOR_DOWN);
                }
            } else {
                nvim_tui_terminfo_print_num1(tui, KTERM_PARM_DOWN_CURSOR, n);
            }
            ugrid_goto(grid, row, col);
            return;
        } else if row < grid_row {
            let n = grid_row - row;
            if n <= 2 {
                for _ in 0..n {
                    nvim_tui_terminfo_out(tui, KTERM_CURSOR_UP);
                }
            } else {
                nvim_tui_terminfo_print_num1(tui, KTERM_PARM_UP_CURSOR, n);
            }
            ugrid_goto(grid, row, col);
            return;
        }
    }

    rs_cursor_goto_safe(tui, row, col);
}

/// Emit a cursor_address (absolute position) sequence.
unsafe fn rs_cursor_goto_safe(tui: *mut TuiHandle, row: c_int, col: c_int) {
    nvim_tui_terminfo_print_num2(tui, KTERM_CURSOR_ADDRESS, row, col);
    let grid = nvim_tui_get_grid(tui);
    ugrid_goto(grid, row, col);
}

/// Clear a rectangular region of the screen.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_clear_region(
    tui: *mut TuiHandle,
    top: c_int,
    bot: c_int,
    left: c_int,
    right: c_int,
    attr_id: c_int,
) {
    let grid = nvim_tui_get_grid(tui);
    if nvim_tui_get_set_default_colors(tui) {
        rs_update_attrs(tui, attr_id);
    } else {
        nvim_tui_terminfo_out(tui, KTERM_EXIT_ATTRIBUTE_MODE);
    }

    let width_val = nvim_tui_get_width(tui);
    let height_val = nvim_tui_get_height(tui);

    if nvim_tui_get_can_clear_attr(tui) && left == 0 && right == width_val && bot == height_val {
        if top == 0 {
            nvim_tui_terminfo_out(tui, KTERM_CLEAR_SCREEN);
            ugrid_goto(grid, top, left);
        } else {
            rs_cursor_goto(tui, top, 0);
            nvim_tui_terminfo_out(tui, KTERM_CLR_EOS);
        }
    } else {
        let width = right - left;
        for row in top..bot {
            rs_cursor_goto(tui, row, left);
            if nvim_tui_get_can_clear_attr(tui) && right == width_val {
                nvim_tui_terminfo_out(tui, KTERM_CLR_EOL);
            } else if nvim_tui_get_can_erase_chars(tui)
                && nvim_tui_get_can_clear_attr(tui)
                && width >= 5
            {
                nvim_tui_terminfo_print_num1(tui, KTERM_ERASE_CHARS, width);
            } else {
                rs_print_spaces(tui, width);
                // Advance grid column manually after printing spaces
                let cur_col = nvim_tui_get_grid_col(tui);
                nvim_tui_set_grid_col(tui, cur_col + width);
                if nvim_tui_get_immediate_wrap(tui) {
                    rs_final_column_wrap(tui);
                }
            }
        }
    }
}

// Phase 5 C accessors
extern "C" {
    fn nvim_tui_ui_client_set_size(tui: *mut TuiHandle, width: c_int, height: c_int);
    fn nvim_tui_inc_pending_resize_events(tui: *mut TuiHandle);
    fn nvim_tui_set_width(tui: *mut TuiHandle, width: c_int);
    fn nvim_tui_set_height(tui: *mut TuiHandle, height: c_int);
}

/// Set terminal size and notify UI client.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_set_size(tui: *mut TuiHandle, width: c_int, height: c_int) {
    if tui.is_null() {
        return;
    }
    nvim_tui_inc_pending_resize_events(tui);
    nvim_tui_set_width(tui, width);
    nvim_tui_set_height(tui, height);
    nvim_tui_ui_client_set_size(tui, width, height);
}

// Phase 4 C accessors
extern "C" {
    fn nvim_tui_pop_invalid_region(
        tui: *mut TuiHandle,
        top: *mut c_int,
        bot: *mut c_int,
        left: *mut c_int,
        right: *mut c_int,
    ) -> bool;
    fn nvim_tui_loop_flooded(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_loop_purge(tui: *mut TuiHandle);
    fn nvim_tui_find_clear_col(
        tui: *mut TuiHandle,
        row: c_int,
        left: c_int,
        right: c_int,
        clear_attr: i32,
    ) -> c_int;
    fn nvim_tui_next_cell_is_nul(tui: *mut TuiHandle, row: c_int, col: c_int) -> bool;
    fn nvim_tui_ugrid_clear_chunk(
        tui: *mut TuiHandle,
        row: c_int,
        col: c_int,
        endcol: c_int,
        attr: i32,
    );
    fn nvim_tui_set_grid_cell(tui: *mut TuiHandle, row: c_int, col: c_int, data: u32, attr: i32);
    // UTF/schar functions for print_cell_at_pos
    fn schar_get(buf_out: *mut libc::c_char, sc: u32) -> usize;
    fn utf_ptr2char(p: *const libc::c_char) -> c_int;
    fn utf_ambiguous_width(p: *const libc::c_char) -> bool;
    fn utf_char2cells(c: c_int) -> c_int;
    fn nvim_tui_get_row(tui: *mut TuiHandle) -> c_int;
    fn nvim_tui_get_col(tui: *mut TuiHandle) -> c_int;
}

// schar_T NUL value (schar_from_ascii(0) = 0 on little-endian)
const SCHAR_NUL: u32 = 0;
// kLineFlagWrap = 1
const LINE_FLAG_WRAP: i64 = 1;
// MAX_SCHAR_SIZE = 32
const MAX_SCHAR_SIZE: usize = 32;

/// Print a cell at position (row, col). Rust implementation.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
pub unsafe fn rs_print_cell_at_pos_impl(
    tui: *mut TuiHandle,
    row: c_int,
    col: c_int,
    is_doublewidth: bool,
) {
    let cell_data = nvim_tui_get_cell_data(tui, row, col);
    let cell_attr = nvim_tui_get_cell_attr(tui, row, col);

    // If grid cursor is unknown and cell is NUL, skip (nothing to print)
    let grid_row = nvim_tui_get_grid_row(tui);
    if grid_row == -1 && cell_data == SCHAR_NUL {
        return;
    }

    rs_cursor_goto(tui, row, col);

    let mut buf = [0i8; MAX_SCHAR_SIZE];
    schar_get(buf.as_mut_ptr(), cell_data);

    let c = utf_ptr2char(buf.as_ptr());
    let is_ambiwidth = utf_ambiguous_width(buf.as_ptr());
    let is_ambiwidth = if is_doublewidth && (is_ambiwidth || utf_char2cells(c) == 1) {
        // Clear the two screen cells
        rs_update_attrs(tui, cell_attr);
        rs_print_spaces(tui, 2);
        let cur_col = nvim_tui_get_grid_col(tui);
        nvim_tui_set_grid_col(tui, cur_col + 2);
        rs_cursor_goto(tui, row, col);
        true
    } else {
        is_ambiwidth
    };

    rs_print_cell(tui, buf.as_ptr().cast::<u8>(), cell_attr as i16);

    if is_ambiwidth {
        // Force repositioning cursor after ambiguous-width char
        nvim_tui_invalidate_grid_cursor(tui);
    }
}

/// Flush the TUI: process all invalid regions and write to terminal.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_flush(tui: *mut TuiHandle) {
    if tui.is_null() {
        return;
    }

    // Back-pressure: if event queue is flooded, purge it
    if nvim_tui_loop_flooded(tui) {
        nvim_tui_loop_purge(tui);
        rs_tui_busy_stop(tui);
    }

    let grid_height = nvim_tui_get_grid_height(tui);
    let grid_width = nvim_tui_get_grid_width(tui);

    loop {
        let (mut top, mut bot, mut left, mut right) = (0i32, 0i32, 0i32, 0i32);
        if !nvim_tui_pop_invalid_region(tui, &mut top, &mut bot, &mut left, &mut right) {
            break;
        }

        // Clip to grid bounds
        if bot > grid_height {
            bot = grid_height;
        }
        if right > grid_width {
            right = grid_width;
        }

        for row in top..bot {
            // Find clear_col: where trailing spaces with clear_attr begin
            let clear_attr = nvim_tui_get_cell_attr(tui, row, right - 1);
            let clear_col = nvim_tui_find_clear_col(tui, row, left, right, clear_attr);

            // Print each non-trailing cell
            let mut col = left;
            while col < clear_col {
                let is_dw = col < clear_col - 1 && nvim_tui_next_cell_is_nul(tui, row, col);
                rs_print_cell_at_pos_impl(tui, row, col, is_dw);
                col += 1;
            }

            // Clear trailing space region
            if clear_col < right {
                rs_clear_region(tui, row, row + 1, clear_col, right, clear_attr);
            }
        }
    }

    // Position cursor at the target position
    let target_row = nvim_tui_get_row(tui);
    let target_col = nvim_tui_get_col(tui);
    rs_cursor_goto(tui, target_row, target_col);

    rs_flush_buf(tui);
}

/// Render a raw line directly to the terminal.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_raw_line(
    tui: *mut TuiHandle,
    _g: i64,
    linerow: i64,
    startcol: i64,
    endcol: i64,
    clearcol: i64,
    clearattr: i64,
    flags: i64,
    chunk: *const u32,
    attrs: *const i32,
) {
    if tui.is_null() {
        return;
    }

    let row = linerow as c_int;

    // Update grid cells from chunk/attrs arrays
    for c in startcol..endcol {
        let offset = (c - startcol) as isize;
        let data = *chunk.offset(offset);
        let attr = *attrs.offset(offset);
        nvim_tui_set_grid_cell(tui, row, c as c_int, data, attr);
    }

    // Print each cell
    let mut col = startcol as c_int;
    while col < endcol as c_int {
        let is_dw = col < endcol as c_int - 1 && nvim_tui_next_cell_is_nul(tui, row, col);
        rs_print_cell_at_pos_impl(tui, row, col, is_dw);
        col += 1;
    }

    // Clear beyond endcol if needed
    if clearcol > endcol {
        nvim_tui_ugrid_clear_chunk(
            tui,
            row,
            endcol as c_int,
            clearcol as c_int,
            clearattr as i32,
        );
        rs_clear_region(
            tui,
            row,
            row + 1,
            endcol as c_int,
            clearcol as c_int,
            clearattr as c_int,
        );
    }

    // Handle line wrap at right margin
    let grid_width = nvim_tui_get_grid_width(tui);
    let tui_width = nvim_tui_get_width(tui);
    let grid_height = nvim_tui_get_grid_height(tui);
    if flags & LINE_FLAG_WRAP != 0 && tui_width == grid_width && row + 1 < grid_height {
        if endcol as c_int != grid_width {
            // Print the last char of the row if not already done
            let last = grid_width - 1;
            let last_data = nvim_tui_get_cell_data(tui, row, last);
            let size = if last_data == SCHAR_NUL { 2 } else { 1 };
            rs_print_cell_at_pos_impl(tui, row, grid_width - size, size == 2);
        }
        rs_final_column_wrap(tui);
    }
}

// Phase 5b: cursor mode and size detection accessors
extern "C" {
    fn nvim_tui_cursor_style_enabled() -> bool;
    fn nvim_tui_get_cursor_shape_id(tui: *mut TuiHandle, mode: c_int) -> c_int;
    fn nvim_tui_get_cursor_shape_shape(tui: *mut TuiHandle, mode: c_int) -> c_int;
    fn nvim_tui_get_cursor_shape_blinkon(tui: *mut TuiHandle, mode: c_int) -> c_int;
    fn nvim_tui_get_cursor_shape_blinkoff(tui: *mut TuiHandle, mode: c_int) -> c_int;
    fn nvim_tui_get_want_invisible(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_set_want_invisible(tui: *mut TuiHandle, val: bool);
    fn nvim_tui_get_cursor_has_color(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_set_cursor_has_color(tui: *mut TuiHandle, val: bool);
    fn nvim_tui_get_set_cursor_color_as_str(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_terminfo_print_str(tui: *mut TuiHandle, what: c_int, str: *const u8);
    fn nvim_tui_get_out_isatty(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_uv_tty_get_winsize(
        tui: *mut TuiHandle,
        width: *mut c_int,
        height: *mut c_int,
    ) -> bool;
    fn nvim_tui_get_ti_lines(tui: *mut TuiHandle) -> c_int;
    fn nvim_tui_get_ti_columns(tui: *mut TuiHandle) -> c_int;
    fn os_getenv_noalloc(name: *const u8) -> *const u8;
    fn nvim_tui_stdin_isatty() -> bool;
    fn nvim_tui_tty_reset_mode_hack(tui: *mut TuiHandle);
    fn nvim_tui_show_verbose_terminfo(tui: *mut TuiHandle);
    fn nvim_tui_set_showing_mode(tui: *mut TuiHandle, mode: c_int);
    fn nvim_tui_set_is_starting(tui: *mut TuiHandle, val: bool);
    fn nvim_tui_get_verbose(tui: *mut TuiHandle) -> i64;
    fn nvim_tui_get_mode_theme_updates(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_get_mode_resize_events(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_get_mode_grapheme_clusters(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_get_disable_focus_reporting(tui: *mut TuiHandle) -> *const u8;
    fn nvim_tui_reset_title(tui: *mut TuiHandle);
    fn nvim_tui_reset_key_encoding(tui: *mut TuiHandle);
}

// kTerm enum values for cursor mode (0-based C enum values from terminfo_enum_defs.h)
// Note: #define on line 44 occupies a line but not an enum slot, so post-#define
// entries use formula: value = line - 7
const TERM_SET_CURSOR_STYLE: c_int = 39; // kTerm_set_cursor_style (line 46 - 7)
const TERM_SET_CURSOR_COLOR: c_int = 43; // kTerm_set_cursor_color (line 50 - 7)
const TERM_RESET_CURSOR_COLOR: c_int = 44; // kTerm_reset_cursor_color (line 51 - 7)

// HL_INVERSE flag
const HL_INVERSE: i16 = 0x01;

// Default terminal size
const DFLT_COLS: c_int = 80;
const DFLT_ROWS: c_int = 24;

/// Set cursor mode based on mode index.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_set_mode(tui: *mut TuiHandle, mode: c_int) {
    if tui.is_null() {
        return;
    }
    if !nvim_tui_cursor_style_enabled() {
        return;
    }

    let c_id = nvim_tui_get_cursor_shape_id(tui, mode);
    let attrs_size = nvim_tui_get_attrs_size(tui);

    if c_id != 0 && (c_id as usize) < attrs_size && nvim_tui_get_rgb(tui) {
        let aep = nvim_tui_get_attrs_entry(tui, c_id as usize);
        nvim_tui_set_want_invisible(tui, aep.hl_blend == 100);
        if !nvim_tui_get_want_invisible(tui) && (aep.rgb_ae_attr & HL_INVERSE) != 0 {
            // Interpret "inverse" as "default" -- no termcode for inverse cursor color.
            nvim_tui_terminfo_out(tui, TERM_RESET_CURSOR_COLOR);
        } else if !nvim_tui_get_want_invisible(tui) && aep.rgb_bg_color >= 0 {
            if nvim_tui_get_set_cursor_color_as_str(tui) {
                let hex = format!("#{:06x}\0", aep.rgb_bg_color);
                nvim_tui_terminfo_print_str(tui, TERM_SET_CURSOR_COLOR, hex.as_ptr());
            } else {
                nvim_tui_terminfo_print_num1(tui, TERM_SET_CURSOR_COLOR, aep.rgb_bg_color);
            }
            nvim_tui_set_cursor_has_color(tui, true);
        }
    } else if c_id == 0 && (nvim_tui_get_want_invisible(tui) || nvim_tui_get_cursor_has_color(tui))
    {
        // No cursor color for this mode; reset to default.
        nvim_tui_set_want_invisible(tui, false);
        nvim_tui_set_cursor_has_color(tui, false);
        nvim_tui_terminfo_out(tui, TERM_RESET_CURSOR_COLOR);
    }

    // Shape: SHAPE_BLOCK=0 -> 1, SHAPE_HOR=1 -> 3, SHAPE_VER=2 -> 5
    let c_shape = nvim_tui_get_cursor_shape_shape(tui, mode);
    let shape = match c_shape {
        0 => 1, // SHAPE_BLOCK
        1 => 3, // SHAPE_HOR
        _ => 5, // SHAPE_VER
    };
    let blink_on = nvim_tui_get_cursor_shape_blinkon(tui, mode);
    let blink_off = nvim_tui_get_cursor_shape_blinkoff(tui, mode);
    let blink_stop = if blink_on == 0 || blink_off == 0 {
        1
    } else {
        0
    };
    nvim_tui_terminfo_print_num1(tui, TERM_SET_CURSOR_STYLE, shape + blink_stop);
}

/// Detect and set terminal dimensions.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_guess_size(tui: *mut TuiHandle) {
    if tui.is_null() {
        return;
    }

    let mut width: c_int = 0;
    let mut height: c_int = 0;

    // 1 - try from a system call (ioctl/TIOCGWINSZ on unix)
    if nvim_tui_get_out_isatty(tui) && nvim_tui_uv_tty_get_winsize(tui, &mut width, &mut height) {
        // success
    } else {
        // 2 - use $LINES/$COLUMNS if available
        let parse_env_int = |name: &[u8]| -> Option<c_int> {
            let ptr = os_getenv_noalloc(name.as_ptr());
            if ptr.is_null() {
                return None;
            }
            let s = core::ffi::CStr::from_ptr(ptr as *const core::ffi::c_char);
            s.to_str().ok()?.trim().parse::<c_int>().ok()
        };
        let env_lines = parse_env_int(b"LINES\0");
        let env_cols = parse_env_int(b"COLUMNS\0");
        if let (Some(h), Some(w)) = (env_lines, env_cols) {
            if h > 0 && w > 0 {
                height = h;
                width = w;
            } else {
                // 3 - read from terminfo if available
                height = nvim_tui_get_ti_lines(tui);
                width = nvim_tui_get_ti_columns(tui);
            }
        } else {
            // 3 - read from terminfo if available
            height = nvim_tui_get_ti_lines(tui);
            width = nvim_tui_get_ti_columns(tui);
        }
    }

    if width <= 0 || height <= 0 {
        width = DFLT_COLS;
        height = DFLT_ROWS;
    }

    rs_tui_set_size(tui, width, height);
}

/// Handle mode change: TTY reset hack, set cursor mode, verbose info, update state.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_mode_change(tui: *mut TuiHandle, mode_idx: i64) {
    if tui.is_null() {
        return;
    }

    // On UNIX: If stdin is not a TTY, reset TTY modes to handle piped input
    // correctly (#13073).
    if nvim_tui_get_is_starting(tui) && !nvim_tui_stdin_isatty() {
        nvim_tui_tty_reset_mode_hack(tui);
    }

    rs_tui_set_mode(tui, mode_idx as c_int);

    if nvim_tui_get_is_starting(tui) && nvim_tui_get_verbose(tui) >= 3 {
        nvim_tui_show_verbose_terminfo(tui);
    }

    nvim_tui_set_is_starting(tui, false);
    nvim_tui_set_showing_mode(tui, mode_idx as c_int);
}

// kTerm enum values for terminfo_disable (0-based, value = line - 6 for pre-#define entries)
const KTERM_CURSOR_NORMAL: c_int = 10; // kTerm_cursor_normal (line 16)
const KTERM_RESET_CURSOR_STYLE: c_int = 38; // kTerm_reset_cursor_style (line 45 - 7)
const KTERM_KEYPAD_LOCAL: c_int = 25; // kTerm_keypad_local (line 31)

// TermMode enum values for terminfo_disable (explicit values from tui_defs.h)
const TERM_MODE_THEME_UPDATES: c_int = 2031; // kTermModeThemeUpdates
const TERM_MODE_RESIZE_EVENTS: c_int = 2048; // kTermModeResizeEvents
const TERM_MODE_GRAPHEME_CLUSTERS: c_int = 2027; // kTermModeGraphemeClusters
const TERM_MODE_BRACKETED_PASTE: c_int = 2004; // kTermModeBracketedPaste

/// Disable terminal modes and flush output before TUI exit or suspend.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_terminfo_disable(tui: *mut TuiHandle) {
    if tui.is_null() {
        return;
    }

    // Disable theme update notifications first to avoid spurious notifications.
    if nvim_tui_get_mode_theme_updates(tui) {
        nvim_tui_set_term_mode(tui, TERM_MODE_THEME_UPDATES, false);
    }

    // Reset cursor mode to normal (mode index 0 = SHAPE_IDX_N)
    rs_tui_mode_change(tui, 0);
    rs_tui_mouse_off(tui);
    nvim_tui_terminfo_out(tui, KTERM_EXIT_ATTRIBUTE_MODE);
    // Reset cursor to normal before exiting alternate screen.
    nvim_tui_terminfo_out(tui, KTERM_CURSOR_NORMAL);
    nvim_tui_terminfo_out(tui, KTERM_RESET_CURSOR_STYLE);
    nvim_tui_terminfo_out(tui, KTERM_KEYPAD_LOCAL);

    // Reset the key encoding
    nvim_tui_reset_key_encoding(tui);

    // Disable terminal modes that we enabled
    if nvim_tui_get_mode_resize_events(tui) {
        nvim_tui_set_term_mode(tui, TERM_MODE_RESIZE_EVENTS, false);
    }

    if nvim_tui_get_mode_grapheme_clusters(tui) {
        nvim_tui_set_term_mode(tui, TERM_MODE_GRAPHEME_CLUSTERS, false);
    }

    // May restore old title before exiting alternate screen.
    nvim_tui_reset_title(tui);
    if nvim_tui_get_cursor_has_color(tui) {
        nvim_tui_terminfo_out(tui, TERM_RESET_CURSOR_COLOR);
    }
    // Disable bracketed paste
    nvim_tui_set_term_mode(tui, TERM_MODE_BRACKETED_PASTE, false);
    // Disable focus reporting
    let disable_focus = nvim_tui_get_disable_focus_reporting(tui);
    nvim_tui_out_len(tui, disable_focus);

    // Send a DA1 request. When the terminal responds we know that it has
    // processed all of our requests and won't be emitting anymore sequences.
    nvim_tui_out(tui, b"\x1b[c".as_ptr(), 3);

    // Immediately flush the buffer and wait for the DA1 response.
    nvim_tui_flush_buf(tui);
}

// ============================================================================
// Phase 1: Terminal mode / key encoding cluster
// ============================================================================

// Additional TermMode enum values (from tui_defs.h) not yet defined above.
// TERM_MODE_LEFT_RIGHT_MARGINS, TERM_MODE_GRAPHEME_CLUSTERS, TERM_MODE_THEME_UPDATES,
// and TERM_MODE_RESIZE_EVENTS are already defined in the terminfo_disable section above.
const TERM_MODE_SYNCHRONIZED_OUTPUT: c_int = 2026; // kTermModeSynchronizedOutput

// KeyEncoding enum values (from input.h)
// KEY_ENCODING_LEGACY = 0 is the default (falls through to `_ => {}`)
const KEY_ENCODING_KITTY: c_int = 1; // kKeyEncodingKitty
const KEY_ENCODING_XTERM: c_int = 2; // kKeyEncodingXterm

extern "C" {
    fn nvim_tui_set_modes_grapheme_clusters(tui: *mut TuiHandle, val: bool);
    fn nvim_tui_set_modes_theme_updates(tui: *mut TuiHandle, val: bool);
    fn nvim_tui_set_modes_resize_events(tui: *mut TuiHandle, val: bool);
    fn nvim_tui_set_has_lr_margin_mode(tui: *mut TuiHandle, val: bool);
    fn nvim_tui_set_resize_events_enabled(tui: *mut TuiHandle, val: bool);
    fn nvim_tui_set_has_sync_mode(tui: *mut TuiHandle, val: bool);
    fn nvim_tui_set_primary_device_attr_cb(
        tui: *mut TuiHandle,
        cb: unsafe extern "C" fn(*mut TuiHandle),
    );
    fn nvim_tui_input_get_key_encoding(tui: *mut TuiHandle) -> c_int;
    // nvim_tui_set_print_attr_id already declared in the earlier extern "C" block
}

/// Request the terminal's mode (DECRQM). Emits \x1b[?<mode>$p.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_request_term_mode(tui: *mut TuiHandle, mode: c_int) {
    if tui.is_null() {
        return;
    }
    let mut buf = [0u8; 12];
    let s = format!("\x1b[?{}$p", mode);
    let bytes = s.as_bytes();
    let len = bytes.len().min(buf.len());
    buf[..len].copy_from_slice(&bytes[..len]);
    nvim_tui_out(tui, buf.as_ptr(), len);
}

/// Set (DECSET) or reset (DECRST) a terminal private mode.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_set_term_mode_impl(tui: *mut TuiHandle, mode: c_int, set: bool) {
    if tui.is_null() {
        return;
    }
    let ch = if set { b'h' } else { b'l' };
    let s = format!("\x1b[?{}{}", mode, ch as char);
    let bytes = s.as_bytes();
    nvim_tui_out(tui, bytes.as_ptr(), bytes.len());
}

/// Handle a mode report (DECRPM) from the terminal.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_handle_term_mode(tui: *mut TuiHandle, mode: c_int, state: c_int) {
    if tui.is_null() {
        return;
    }

    // kTermModeState values from tui_defs.h:
    // kTermModeNotRecognized = 0, kTermModeSet = 1, kTermModeReset = 2,
    // kTermModePermanentlySet = 3, kTermModePermanentlyReset = 4
    if state == 0 || state == 4 {
        // kTermModeNotRecognized or kTermModePermanentlyReset — nothing to do
        return;
    }

    // kTermModeSet (1), kTermModeReset (2), kTermModePermanentlySet (3) reach here
    // "is currently set" means the terminal already has the mode active
    let is_set = state == 1 || state == 3; // kTermModeSet or kTermModePermanentlySet

    match mode {
        TERM_MODE_SYNCHRONIZED_OUTPUT => {
            nvim_tui_set_has_sync_mode(tui, true);
        }
        TERM_MODE_GRAPHEME_CLUSTERS => {
            if !is_set {
                nvim_tui_set_term_mode(tui, TERM_MODE_GRAPHEME_CLUSTERS, true);
                nvim_tui_set_modes_grapheme_clusters(tui, true);
            }
        }
        TERM_MODE_THEME_UPDATES => {
            if !is_set {
                nvim_tui_set_term_mode(tui, TERM_MODE_THEME_UPDATES, true);
                nvim_tui_set_modes_theme_updates(tui, true);
            }
        }
        TERM_MODE_RESIZE_EVENTS => {
            if !is_set {
                nvim_tui_set_term_mode(tui, TERM_MODE_RESIZE_EVENTS, true);
                nvim_tui_set_modes_resize_events(tui, true);
            }
            // Track whether resize events are enabled regardless of who enabled it
            nvim_tui_set_resize_events_enabled(tui, true);
        }
        TERM_MODE_LEFT_RIGHT_MARGINS => {
            nvim_tui_set_has_lr_margin_mode(tui, true);
        }
        _ => {}
    }
}

/// Query terminal for extended underline support (DECRQSS SGR probe).
///
/// Emits reset + SGR undercurl + DECRQSS, and resets print_attr_id.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_query_extended_underline(tui: *mut TuiHandle) {
    if tui.is_null() {
        return;
    }
    let seq = b"\x1b[0m\x1b[4:3m\x1bP$qm\x1b\\";
    nvim_tui_out(tui, seq.as_ptr(), seq.len());
    nvim_tui_set_print_attr_id(tui, -1);
}

/// Query the terminal for Kitty keyboard protocol support.
///
/// Emits CSI ? u CSI c and registers tui_set_key_encoding as the DA1 callback.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_query_kitty_keyboard(tui: *mut TuiHandle) {
    if tui.is_null() {
        return;
    }
    nvim_tui_set_primary_device_attr_cb(tui, rs_tui_set_key_encoding_cb);
    let seq = b"\x1b[?u\x1b[c";
    nvim_tui_out(tui, seq.as_ptr(), seq.len());
}

/// Called when DA1 response is received; sets the appropriate key encoding.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_set_key_encoding_cb(tui: *mut TuiHandle) {
    if tui.is_null() {
        return;
    }
    let encoding = nvim_tui_input_get_key_encoding(tui);
    match encoding {
        e if e == KEY_ENCODING_KITTY => {
            // Progressive enhancement flags:
            //   0b01   (1) Disambiguate escape codes
            //   0b10   (2) Report event types
            nvim_tui_out(tui, b"\x1b[>3u".as_ptr(), 5);
        }
        e if e == KEY_ENCODING_XTERM => {
            nvim_tui_out(tui, b"\x1b[>4;2m".as_ptr(), 7);
        }
        _ => {} // kKeyEncodingLegacy: no action
    }
}

/// Reset the key encoding to legacy.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_reset_key_encoding_impl(tui: *mut TuiHandle) {
    if tui.is_null() {
        return;
    }
    let encoding = nvim_tui_input_get_key_encoding(tui);
    match encoding {
        e if e == KEY_ENCODING_KITTY => {
            nvim_tui_out(tui, b"\x1b[<u".as_ptr(), 4);
        }
        e if e == KEY_ENCODING_XTERM => {
            nvim_tui_out(tui, b"\x1b[>4;0m".as_ptr(), 7);
        }
        _ => {} // kKeyEncodingLegacy: no action
    }
}

// ============================================================================
// Phase 2: chdir, option_set, screenshot
// ============================================================================

extern "C" {
    fn nvim_tui_set_mouse_move_enabled(tui: *mut TuiHandle, val: bool);
    fn nvim_tui_set_rgb(tui: *mut TuiHandle, val: bool);
    fn nvim_tui_set_verbose(tui: *mut TuiHandle, val: i64);
    fn nvim_tui_set_sync_output(tui: *mut TuiHandle, val: bool);
    fn nvim_tui_set_input_ttimeout(tui: *mut TuiHandle, val: bool);
    fn nvim_tui_set_input_ttimeoutlen(tui: *mut TuiHandle, val: i64);
    fn nvim_tui_set_screenshot(tui: *mut TuiHandle, f: *mut libc::FILE);
    fn nvim_tui_set_grid_row_val(tui: *mut TuiHandle, row: c_int);
    fn nvim_tui_set_grid_col_val(tui: *mut TuiHandle, col: c_int);
    fn nvim_tui_rpc_send_termguicolors(value: bool);
    // schar_get already declared above (as *mut libc::c_char)
    // nvim_tui_cursor_goto_internal already declared above
}

extern "C" {
    fn uv_chdir(dir: *const libc::c_char) -> c_int;
}

/// Change the process working directory. Logs on failure.
///
/// # Safety
///
/// - `path` must be a valid NUL-terminated C string pointer
#[no_mangle]
pub unsafe extern "C" fn rs_tui_chdir(path: *const libc::c_char, _path_len: usize) {
    if path.is_null() {
        return;
    }
    // Use uv_chdir (matches the original C tui_chdir behavior)
    uv_chdir(path);
}

// ObjectType values matching C enum (kObjectTypeBoolean=1)
// kObjectTypeInteger=2 is handled by reading int_val directly
const OBJECT_TYPE_BOOLEAN: c_int = 1;

/// Handle a TUI option change dispatched from the server.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
/// - `name` must be a valid pointer to `name_len` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_tui_option_set(
    tui: *mut TuiHandle,
    name: *const u8,
    name_len: usize,
    obj_type: c_int,
    int_val: i64,
    bool_val: bool,
) {
    if tui.is_null() || name.is_null() {
        return;
    }
    let name_bytes = std::slice::from_raw_parts(name, name_len);

    if name_bytes == b"mousemoveevent" {
        let cur = nvim_tui_get_mouse_move_enabled(tui);
        let new_val = obj_type == OBJECT_TYPE_BOOLEAN && bool_val;
        if cur != new_val {
            if nvim_tui_get_mouse_enabled(tui) {
                rs_tui_mouse_off(tui);
                nvim_tui_set_mouse_move_enabled(tui, new_val);
                rs_tui_mouse_on(tui);
            } else {
                nvim_tui_set_mouse_move_enabled(tui, new_val);
            }
        }
    } else if name_bytes == b"termguicolors" {
        let new_val = obj_type == OBJECT_TYPE_BOOLEAN && bool_val;
        nvim_tui_set_rgb(tui, new_val);
        nvim_tui_set_print_attr_id(tui, -1);
        let height = nvim_tui_get_grid_height(tui);
        let width = nvim_tui_get_grid_width(tui);
        rs_invalidate(tui, 0, height, 0, width);
        nvim_tui_rpc_send_termguicolors(new_val);
    } else if name_bytes == b"ttimeout" {
        let new_val = obj_type == OBJECT_TYPE_BOOLEAN && bool_val;
        nvim_tui_set_input_ttimeout(tui, new_val);
    } else if name_bytes == b"ttimeoutlen" {
        nvim_tui_set_input_ttimeoutlen(tui, int_val);
    } else if name_bytes == b"verbose" {
        nvim_tui_set_verbose(tui, int_val);
    } else if name_bytes == b"termsync" {
        let new_val = obj_type == OBJECT_TYPE_BOOLEAN && bool_val;
        nvim_tui_set_sync_output(tui, new_val);
    }
}

// KTERM_CLEAR_SCREEN = 2 is already defined above in the terminfo_disable section

/// Capture a screenshot of the TUI grid to a file.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
/// - `path` must be a valid pointer to a NUL-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_tui_screenshot(
    tui: *mut TuiHandle,
    path: *const libc::c_char,
    _path_len: usize,
) {
    if tui.is_null() || path.is_null() {
        return;
    }

    let mode = b"w\0";
    let f = libc::fopen(path, mode.as_ptr() as *const libc::c_char);
    if f.is_null() {
        return;
    }

    let height = nvim_tui_get_grid_height(tui);
    let width = nvim_tui_get_grid_width(tui);

    nvim_tui_flush_buf(tui);
    nvim_tui_set_grid_row_val(tui, 0);
    nvim_tui_set_grid_col_val(tui, 0);

    nvim_tui_set_screenshot(tui, f);

    // Print header: "height,width\n"
    let header = format!("{},{}\n", height, width);
    libc::fwrite(header.as_ptr() as *const libc::c_void, 1, header.len(), f);

    // Clear screen
    rs_terminfo_out(tui, KTERM_CLEAR_SCREEN);

    // Walk grid
    for i in 0..height {
        nvim_tui_cursor_goto_internal(tui, i, 0);
        for j in 0..width {
            let data = nvim_tui_get_cell_data(tui, i, j);
            let attr = nvim_tui_get_cell_attr(tui, i, j);
            const MAX_SCHAR_SIZE: usize = 28; // from nvim/grid.h
            let mut buf = [0u8; MAX_SCHAR_SIZE];
            schar_get(buf.as_mut_ptr() as *mut libc::c_char, data);
            rs_print_cell(tui, buf.as_ptr(), attr as i16);
        }
    }

    nvim_tui_flush_buf(tui);
    nvim_tui_set_screenshot(tui, std::ptr::null_mut());
    libc::fclose(f);
}

// ============================================================================
// Phase 3: sigwinch, after_startup, terminal_after_startup callbacks
// ============================================================================

/// Opaque handle for C SignalWatcher (used for sigwinch callback)
#[repr(C)]
pub struct SignalWatcherHandle {
    _private: [u8; 0],
}

extern "C" {
    fn nvim_tui_get_enable_focus_reporting(tui: *mut TuiHandle) -> *const u8;
    fn nvim_tui_get_resize_events_enabled(tui: *mut TuiHandle) -> bool;
}

/// SIGWINCH callback. Calls rs_tui_guess_size unless stopped or resize_events_enabled.
///
/// # Safety
///
/// Callback from libuv/signal.c — cbdata is a TUIData pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_tui_sigwinch_cb(
    _watcher: *mut SignalWatcherHandle,
    _signum: c_int,
    cbdata: *mut libc::c_void,
) {
    let tui = cbdata as *mut TuiHandle;
    if tui.is_null() {
        return;
    }
    // Call rs_tui_is_stopped directly (same module) instead of the tui_is_stopped extern
    // to avoid calling ourselves (rs_tui_is_stopped is exported as "tui_is_stopped")
    if rs_tui_is_stopped(tui) || nvim_tui_get_resize_events_enabled(tui) {
        return;
    }
    rs_tui_guess_size(tui);
}

/// Emit focus reporting and flush. Called after TUI startup (working around tmux 2.3 bug).
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_terminal_after_startup(tui: *mut TuiHandle) {
    if tui.is_null() {
        return;
    }
    // Emit this after Nvim startup, not during.  #7649
    let focus_str = nvim_tui_get_enable_focus_reporting(tui);
    nvim_tui_out_len(tui, focus_str);
    nvim_tui_flush_buf(tui);
}

// rs_tui_after_startup_cb is implemented as a thin C wrapper in tui.c
// (kept there because extracting handle->data requires knowledge of uv_timer_t layout)

// ============================================================================
// Phase 5: flush_buf_start and flush_buf_end
// ============================================================================

extern "C" {
    fn nvim_tui_get_sync_output(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_get_has_sync_mode(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_get_is_invisible(tui: *mut TuiHandle) -> bool;
    fn nvim_tui_set_is_invisible(tui: *mut TuiHandle, val: bool);
    fn nvim_tui_get_busy(tui: *mut TuiHandle) -> bool;
    // nvim_tui_get_want_invisible already declared above
}

extern "C" {
    fn rs_terminfo_fmt(
        buf_start: *mut libc::c_char,
        buf_end: *const libc::c_char,
        str_ptr: *const libc::c_char,
        params: *mut TpVar,
    ) -> usize;
}

// kTerm_cursor_invisible = 7 (0-indexed in terminfo_enum_defs.h)
const KTERM_CURSOR_INVISIBLE: c_int = 7;
// kTerm_cursor_normal = 10 — already defined above as KTERM_CURSOR_NORMAL

/// Compute flush prefix: sync-start marker or cursor-hide sequence.
///
/// Returns the number of bytes written to `buf`.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
/// - `buf` must point to a writable buffer of at least `len` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_flush_buf_start(
    tui: *mut TuiHandle,
    buf: *mut libc::c_char,
    len: usize,
) -> usize {
    if tui.is_null() || buf.is_null() {
        return 0;
    }

    if nvim_tui_get_sync_output(tui) && nvim_tui_get_has_sync_mode(tui) {
        // Emit DEC private mode set for synchronized output
        let seq = b"\x1b[?2026h";
        let n = seq.len().min(len);
        std::ptr::copy_nonoverlapping(seq.as_ptr(), buf as *mut u8, n);
        return n;
    }

    if !nvim_tui_get_is_invisible(tui) {
        nvim_tui_set_is_invisible(tui, true);
        let str_ptr = nvim_tui_get_ti_def(tui, KTERM_CURSOR_INVISIBLE);
        if !str_ptr.is_null() && *str_ptr != 0 {
            let mut null_params = [TpVar {
                num: 0,
                string: std::ptr::null_mut(),
            }; 9];
            return rs_terminfo_fmt(
                buf,
                buf.add(len) as *const libc::c_char,
                str_ptr as *const libc::c_char,
                null_params.as_mut_ptr(),
            );
        }
    }
    0
}

/// Compute flush suffix: sync-end marker and cursor show/hide sequence.
///
/// Returns the number of bytes written to `buf`.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
/// - `buf` must point to a writable buffer of at least `len` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_flush_buf_end(
    tui: *mut TuiHandle,
    buf: *mut libc::c_char,
    len: usize,
) -> usize {
    if tui.is_null() || buf.is_null() {
        return 0;
    }

    let mut offset = 0usize;

    if nvim_tui_get_sync_output(tui) && nvim_tui_get_has_sync_mode(tui) {
        // Emit DEC private mode reset for synchronized output
        let seq = b"\x1b[?2026l";
        let n = seq.len().min(len - offset);
        std::ptr::copy_nonoverlapping(seq.as_ptr(), buf.add(offset) as *mut u8, n);
        offset += n;
    }

    let is_invisible = nvim_tui_get_is_invisible(tui);
    let should_invisible = nvim_tui_get_busy(tui) || nvim_tui_get_want_invisible(tui);

    let str_ptr: *const u8 = if is_invisible && !should_invisible {
        nvim_tui_set_is_invisible(tui, false);
        nvim_tui_get_ti_def(tui, KTERM_CURSOR_NORMAL)
    } else if !is_invisible && should_invisible {
        nvim_tui_set_is_invisible(tui, true);
        nvim_tui_get_ti_def(tui, KTERM_CURSOR_INVISIBLE)
    } else {
        std::ptr::null()
    };

    if !str_ptr.is_null() && *str_ptr != 0 && offset < len {
        let mut null_params = [TpVar {
            num: 0,
            string: std::ptr::null_mut(),
        }; 9];
        offset += rs_terminfo_fmt(
            buf.add(offset),
            buf.add(len) as *const libc::c_char,
            str_ptr as *const libc::c_char,
            null_params.as_mut_ptr(),
        );
    }
    offset
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
