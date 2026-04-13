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
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> c_int;
    fn nvim_win_get_cursor_col(wp: WinHandle) -> c_int;
    fn nvim_win_get_virtcol(wp: WinHandle) -> c_int;
    fn nvim_win_buf_line_count(wp: WinHandle) -> c_int;

    // Global state accessors
    #[link_name = "rs_global_stl_height"]
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
    let is_curwin = unsafe { nvim_get_curwin() == wp };
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
    let is_curwin = unsafe { nvim_get_curwin() == wp };
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

// =============================================================================
// Tabline Drawing Decisions
// =============================================================================

/// Represents the decision of what to do for tabline draw.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TablineDrawAction {
    /// Don't draw - no grid or no height
    None = 0,
    /// Use external UI tabline
    UseExtUi = 1,
    /// Use custom 'tabline' option
    UseCustom = 2,
    /// Draw built-in tabline
    DrawBuiltin = 3,
}

/// Context for deciding tabline draw action.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TablineDrawContext {
    /// default_grid.chars != NULL
    pub has_grid: c_int,
    /// ui_has(kUITabline)
    pub ui_has_tabline: c_int,
    /// tabline_height() > 0
    pub has_tabline_height: c_int,
    /// *p_tal != NUL (tabline option is set)
    pub has_tabline_option: c_int,
}

/// Determine what tabline draw action to take.
pub const fn decide_tabline_action(ctx: &TablineDrawContext) -> TablineDrawAction {
    // Early exits
    if ctx.has_grid == 0 {
        return TablineDrawAction::None;
    }

    if ctx.ui_has_tabline != 0 {
        return TablineDrawAction::UseExtUi;
    }

    if ctx.has_tabline_height == 0 {
        return TablineDrawAction::None;
    }

    // Check for custom tabline
    if ctx.has_tabline_option != 0 {
        return TablineDrawAction::UseCustom;
    }

    TablineDrawAction::DrawBuiltin
}

/// FFI export: Decide tabline draw action.
#[no_mangle]
pub const extern "C" fn rs_tabline_decide_action(ctx: &TablineDrawContext) -> c_int {
    decide_tabline_action(ctx) as c_int
}

/// Tab rendering info for a single tab.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TabRenderInfo {
    /// Tab number (1-based)
    pub tabnr: c_int,
    /// Number of (focusable, non-hidden) windows in this tab
    pub win_count: c_int,
    /// Whether any buffer in this tab is modified
    pub modified: c_int,
    /// Whether this is the current tab
    pub is_current: c_int,
    /// Starting column for this tab
    pub start_col: c_int,
    /// Ending column for this tab (exclusive)
    pub end_col: c_int,
}

impl TabRenderInfo {
    /// Create a new tab render info.
    pub const fn new(tabnr: c_int, is_current: bool) -> Self {
        Self {
            tabnr,
            win_count: 0,
            modified: 0,
            is_current: if is_current { 1 } else { 0 },
            start_col: 0,
            end_col: 0,
        }
    }
}

/// FFI export: Create tab render info.
#[no_mangle]
pub const extern "C" fn rs_tab_render_info_new(tabnr: c_int, is_current: c_int) -> TabRenderInfo {
    TabRenderInfo::new(tabnr, is_current != 0)
}

/// Calculate how much room is needed for tab prefix (count + modified indicator).
///
/// Returns the number of characters for the prefix (e.g., "2+ " or " ").
pub const fn calc_tab_prefix_width(win_count: c_int, modified: bool) -> c_int {
    let mut width = 0;

    // Window count
    if win_count > 1 {
        width += count_digits(win_count);
    }

    // Modified indicator
    if modified {
        width += 1; // '+'
    }

    // Space after prefix if there's any prefix
    if width > 0 {
        width += 1;
    }

    // Leading space
    width += 1;

    width
}

/// Count digits in a number.
const fn count_digits(n: c_int) -> c_int {
    if n <= 0 {
        return 1;
    }
    let mut count = 0;
    let mut val = n;
    while val > 0 {
        val /= 10;
        count += 1;
    }
    count
}

/// FFI export: Calculate tab prefix width.
#[no_mangle]
pub const extern "C" fn rs_tab_prefix_width(win_count: c_int, modified: c_int) -> c_int {
    calc_tab_prefix_width(win_count, modified != 0)
}

/// Check if there's room for another tab at the given column.
///
/// Returns true if there's at least 4 columns remaining (minimum for a useful tab).
pub const fn tabline_has_room_for_tab(col: c_int, columns: c_int) -> bool {
    col < columns - 4
}

/// FFI export: Check if tabline has room for another tab.
#[no_mangle]
pub const extern "C" fn rs_tabline_has_room_at(col: c_int, columns: c_int) -> c_int {
    if tabline_has_room_for_tab(col, columns) {
        1
    } else {
        0
    }
}

/// Calculate the column range for the close button (X).
///
/// Returns (start_col, end_col) where the X should be drawn.
/// The X is in the last column when there are multiple tabs.
pub const fn tabline_close_button_range(columns: c_int, has_multiple_tabs: bool) -> (c_int, c_int) {
    if has_multiple_tabs {
        (columns - 1, columns)
    } else {
        (-1, -1) // No close button
    }
}

/// FFI export: Get close button start column.
#[no_mangle]
pub const extern "C" fn rs_tabline_close_col(columns: c_int, tab_count: c_int) -> c_int {
    let (start, _) = tabline_close_button_range(columns, tab_count > 1);
    start
}

/// Calculate the available width for showcmd in tabline.
///
/// showcmd is displayed when 'showcmdloc' == "tabline".
pub const fn tabline_showcmd_width(columns: c_int, col: c_int, has_multiple_tabs: bool) -> c_int {
    let available = columns - col - if has_multiple_tabs { 3 } else { 0 };
    // Max showcmd width is 10
    if available > 10 {
        10
    } else if available > 0 {
        available
    } else {
        0
    }
}

/// FFI export: Calculate showcmd width in tabline.
#[no_mangle]
pub const extern "C" fn rs_tabline_showcmd_width(
    columns: c_int,
    col: c_int,
    tab_count: c_int,
) -> c_int {
    tabline_showcmd_width(columns, col, tab_count > 1)
}

/// Calculate the column where showcmd should start in tabline.
pub const fn tabline_showcmd_col(
    columns: c_int,
    showcmd_width: c_int,
    has_multiple_tabs: bool,
) -> c_int {
    columns - showcmd_width - if has_multiple_tabs { 2 } else { 0 }
}

/// FFI export: Calculate showcmd start column in tabline.
#[no_mangle]
pub const extern "C" fn rs_tabline_showcmd_col(
    columns: c_int,
    showcmd_width: c_int,
    tab_count: c_int,
) -> c_int {
    tabline_showcmd_col(columns, showcmd_width, tab_count > 1)
}

// =============================================================================
// Custom Window Redraw Decision Helpers
// =============================================================================

/// The mode for win_redr_custom.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomRedrawMode {
    /// Tabline (wp == NULL)
    Tabline = 0,
    /// Winbar (draw_winbar == true)
    Winbar = 1,
    /// Statusline (normal statusline)
    Statusline = 2,
    /// Rulerformat (draw_ruler == true)
    Ruler = 3,
}

/// Context for custom window redraw setup.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CustomRedrawContext {
    /// The mode of redraw
    pub mode: c_int,
    /// Row to draw on
    pub row: c_int,
    /// Starting column
    pub col: c_int,
    /// Maximum width for content
    pub maxwidth: c_int,
    /// Fill character (as schar_T/u32)
    pub fillchar: u32,
    /// Highlight attribute
    pub attr: c_int,
    /// Highlight group (hlf_T value)
    pub group: c_int,
    /// Option index (kOptTabline, kOptWinbar, etc.)
    pub opt_idx: c_int,
    /// Option scope (OPT_LOCAL or 0)
    pub opt_scope: c_int,
    /// Whether using external UI event
    pub use_ui_event: c_int,
}

/// Determine the custom redraw mode from the arguments.
pub const fn determine_custom_redraw_mode(
    wp_is_null: bool,
    draw_winbar: bool,
    draw_ruler: bool,
) -> CustomRedrawMode {
    if wp_is_null {
        CustomRedrawMode::Tabline
    } else if draw_winbar {
        CustomRedrawMode::Winbar
    } else if draw_ruler {
        CustomRedrawMode::Ruler
    } else {
        CustomRedrawMode::Statusline
    }
}

/// FFI export: Determine custom redraw mode.
#[no_mangle]
pub const extern "C" fn rs_custom_redraw_mode(
    wp_is_null: c_int,
    draw_winbar: c_int,
    draw_ruler: c_int,
) -> c_int {
    determine_custom_redraw_mode(wp_is_null != 0, draw_winbar != 0, draw_ruler != 0) as c_int
}

/// Calculate ruler column offset.
///
/// The ruler format may have a leading width spec that determines the column.
pub const fn calc_ruler_col_offset(ru_col: c_int, columns: c_int, maxwidth: c_int) -> c_int {
    let offset = ru_col - (columns - maxwidth);
    let half = (maxwidth + 1) / 2;
    if offset > half {
        offset
    } else {
        half
    }
}

/// FFI export: Calculate ruler column offset.
#[no_mangle]
pub const extern "C" fn rs_ruler_col_offset(
    ru_col: c_int,
    columns: c_int,
    maxwidth: c_int,
) -> c_int {
    calc_ruler_col_offset(ru_col, columns, maxwidth)
}

/// Calculate the effective maxwidth after ruler offset.
pub const fn calc_ruler_maxwidth(original_maxwidth: c_int, col_offset: c_int) -> c_int {
    original_maxwidth - col_offset
}

/// FFI export: Calculate ruler effective maxwidth.
#[no_mangle]
pub const extern "C" fn rs_ruler_maxwidth(original_maxwidth: c_int, col_offset: c_int) -> c_int {
    calc_ruler_maxwidth(original_maxwidth, col_offset)
}

/// Check if the custom redraw should be skipped.
///
/// Returns true if maxwidth <= 0 (nothing to draw).
pub const fn should_skip_custom_redraw(maxwidth: c_int) -> bool {
    maxwidth <= 0
}

/// FFI export: Check if custom redraw should be skipped.
#[no_mangle]
pub const extern "C" fn rs_should_skip_custom_redraw(maxwidth: c_int) -> c_int {
    if should_skip_custom_redraw(maxwidth) {
        1
    } else {
        0
    }
}

/// Result of processing highlight records for custom redraw.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct HlRecordResult {
    /// Current highlight attribute to use
    pub attr: c_int,
    /// Current highlight group
    pub group: c_int,
    /// Whether to use combined attribute
    pub use_combined: c_int,
}

/// Process a userhl value to determine the appropriate attribute.
///
/// Returns (attr, group, use_combined).
/// - userhl == 0: use default attr/group
/// - userhl < 0: use syn_id2attr(-userhl), group = -userhl
/// - userhl > 0: use highlight_user[userhl-1], group from User{N}
pub const fn process_userhl(
    userhl: c_int,
    default_attr: c_int,
    default_group: c_int,
) -> HlRecordResult {
    if userhl == 0 {
        HlRecordResult {
            attr: default_attr,
            group: default_group,
            use_combined: 0,
        }
    } else if userhl < 0 {
        // Will need to call syn_id2attr(-userhl) from C
        HlRecordResult {
            attr: -userhl, // Placeholder - C needs to call syn_id2attr
            group: -userhl,
            use_combined: 1,
        }
    } else {
        // Will need to lookup from highlight_user or highlight_stlnc
        HlRecordResult {
            attr: userhl,  // Placeholder - C needs to lookup from array
            group: userhl, // Placeholder - C needs to call syn_name2id for "User{N}"
            use_combined: 1,
        }
    }
}

/// FFI export: Process userhl value (returns interpretation flags).
///
/// Returns:
/// - 0 if userhl == 0 (use default)
/// - 1 if userhl < 0 (use syn_id2attr)
/// - 2 if userhl > 0 (use highlight_user lookup)
#[no_mangle]
pub const extern "C" fn rs_userhl_type(userhl: c_int) -> c_int {
    if userhl == 0 {
        0
    } else if userhl < 0 {
        1
    } else {
        2
    }
}

/// Get the syn_id value from a negative userhl.
#[no_mangle]
pub const extern "C" fn rs_userhl_syn_id(userhl: c_int) -> c_int {
    -userhl
}

/// Get the User highlight index (1-9) from a positive userhl.
#[no_mangle]
pub const extern "C" fn rs_userhl_index(userhl: c_int) -> c_int {
    userhl - 1
}

// =============================================================================
// Phase 5: Public API Entry Points
// =============================================================================

/// Result of win_redr_status decision.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusRedrawAction {
    /// Skip - busy/recursing or wildmenu showing
    Skip = 0,
    /// No status line - set redraw_cmdline
    NoCmdline = 1,
    /// Defer - not redrawing right now
    Defer = 2,
    /// Use custom statusline
    UseCustom = 3,
    /// Draw default statusline (not implemented in current Neovim)
    DrawDefault = 4,
}

/// Context for win_redr_status decision.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct StatusRedrawContext {
    /// Static busy flag (recursion guard)
    pub busy: c_int,
    /// wild_menu_showing != 0
    pub wild_menu_showing: c_int,
    /// ui_has(kUIWildmenu)
    pub ui_has_wildmenu: c_int,
    /// wp->w_status_height == 0
    pub no_status_height: c_int,
    /// global_stl_height() > 0
    pub is_stl_global: c_int,
    /// wp == curwin
    pub is_curwin: c_int,
    /// redrawing()
    pub is_redrawing: c_int,
    /// *wp->w_p_stl != NUL (window-local statusline set)
    pub has_local_stl: c_int,
    /// *p_stl != NUL (global statusline set)
    pub has_global_stl: c_int,
    /// wp->w_floating
    pub is_floating: c_int,
}

/// Determine the status redraw action.
pub const fn decide_status_redraw(ctx: &StatusRedrawContext) -> StatusRedrawAction {
    let is_stl_global = ctx.is_stl_global != 0;
    let is_curwin = ctx.is_curwin != 0;

    // Check recursion/busy and wildmenu
    if ctx.busy != 0 || (ctx.wild_menu_showing != 0 && ctx.ui_has_wildmenu == 0) {
        return StatusRedrawAction::Skip;
    }

    // No status line
    let no_status = ctx.no_status_height != 0;
    if no_status && !(is_stl_global && is_curwin) {
        return StatusRedrawAction::NoCmdline;
    }

    // Not redrawing right now
    if ctx.is_redrawing == 0 {
        return StatusRedrawAction::Defer;
    }

    // Check for custom statusline
    let has_local_stl = ctx.has_local_stl != 0;
    let has_global_stl = ctx.has_global_stl != 0;
    let is_floating = ctx.is_floating != 0;

    if has_local_stl || (has_global_stl && (!is_floating || (is_stl_global && is_curwin))) {
        return StatusRedrawAction::UseCustom;
    }

    // Default statusline (not used in current Neovim but kept for completeness)
    StatusRedrawAction::DrawDefault
}

/// FFI export: Decide status redraw action.
#[no_mangle]
pub const extern "C" fn rs_status_redraw_action(ctx: &StatusRedrawContext) -> c_int {
    decide_status_redraw(ctx) as c_int
}

/// Result of win_redr_winbar decision.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinbarRedrawAction {
    /// Skip - recursing, no height, or not redrawing
    Skip = 0,
    /// Use custom winbar
    UseCustom = 1,
}

/// Context for win_redr_winbar decision.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct WinbarRedrawContext {
    /// Static entered flag (recursion guard)
    pub entered: c_int,
    /// wp->w_winbar_height == 0
    pub no_winbar_height: c_int,
    /// redrawing()
    pub is_redrawing: c_int,
    /// *p_wbr != NUL (global winbar set)
    pub has_global_wbr: c_int,
    /// *wp->w_p_wbr != NUL (local winbar set)
    pub has_local_wbr: c_int,
}

/// Determine the winbar redraw action.
pub const fn decide_winbar_redraw(ctx: &WinbarRedrawContext) -> WinbarRedrawAction {
    // Check recursion
    if ctx.entered != 0 {
        return WinbarRedrawAction::Skip;
    }

    // No winbar height or not redrawing
    if ctx.no_winbar_height != 0 || ctx.is_redrawing == 0 {
        return WinbarRedrawAction::Skip;
    }

    // Check for custom winbar
    if ctx.has_global_wbr != 0 || ctx.has_local_wbr != 0 {
        return WinbarRedrawAction::UseCustom;
    }

    WinbarRedrawAction::Skip
}

/// FFI export: Decide winbar redraw action.
#[no_mangle]
pub const extern "C" fn rs_winbar_redraw_action(ctx: &WinbarRedrawContext) -> c_int {
    decide_winbar_redraw(ctx) as c_int
}

/// Result of redraw_custom_statusline decision.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomStatusRedrawAction {
    /// Skip - recursing
    Skip = 0,
    /// Redraw custom statusline
    Redraw = 1,
}

/// FFI export: Decide custom statusline redraw action.
///
/// Simple recursion check - returns Skip if entered, Redraw otherwise.
#[no_mangle]
pub const extern "C" fn rs_custom_status_redraw_action(entered: c_int) -> c_int {
    if entered != 0 {
        CustomStatusRedrawAction::Skip as c_int
    } else {
        CustomStatusRedrawAction::Redraw as c_int
    }
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

    #[test]
    fn test_tabline_draw_action_none() {
        let ctx = TablineDrawContext {
            has_grid: 0,
            ..Default::default()
        };
        assert_eq!(decide_tabline_action(&ctx), TablineDrawAction::None);
    }

    #[test]
    fn test_tabline_draw_action_use_ext_ui() {
        let ctx = TablineDrawContext {
            has_grid: 1,
            ui_has_tabline: 1,
            has_tabline_height: 1,
            has_tabline_option: 0,
        };
        assert_eq!(decide_tabline_action(&ctx), TablineDrawAction::UseExtUi);
    }

    #[test]
    fn test_tabline_draw_action_use_custom() {
        let ctx = TablineDrawContext {
            has_grid: 1,
            ui_has_tabline: 0,
            has_tabline_height: 1,
            has_tabline_option: 1,
        };
        assert_eq!(decide_tabline_action(&ctx), TablineDrawAction::UseCustom);
    }

    #[test]
    fn test_tabline_draw_action_draw_builtin() {
        let ctx = TablineDrawContext {
            has_grid: 1,
            ui_has_tabline: 0,
            has_tabline_height: 1,
            has_tabline_option: 0,
        };
        assert_eq!(decide_tabline_action(&ctx), TablineDrawAction::DrawBuiltin);
    }

    #[test]
    fn test_tab_prefix_width() {
        // Just leading space
        assert_eq!(calc_tab_prefix_width(1, false), 1);
        // Window count + trailing space + leading space
        assert_eq!(calc_tab_prefix_width(2, false), 3); // "2 " + leading " "
                                                        // Window count (2 digits) + trailing space + leading space
        assert_eq!(calc_tab_prefix_width(10, false), 4); // "10 " + " "
                                                         // Modified only + trailing space + leading space
        assert_eq!(calc_tab_prefix_width(1, true), 3); // "+ " + " "
                                                       // Window count + modified + trailing space + leading space
        assert_eq!(calc_tab_prefix_width(2, true), 4); // "2+ " + " "
    }

    #[test]
    fn test_tabline_has_room() {
        assert!(tabline_has_room_for_tab(0, 80));
        assert!(tabline_has_room_for_tab(75, 80));
        assert!(!tabline_has_room_for_tab(76, 80));
        assert!(!tabline_has_room_for_tab(80, 80));
    }

    #[test]
    fn test_tabline_close_button() {
        let (start, end) = tabline_close_button_range(80, true);
        assert_eq!(start, 79);
        assert_eq!(end, 80);

        let (start, end) = tabline_close_button_range(80, false);
        assert_eq!(start, -1);
        assert_eq!(end, -1);
    }

    #[test]
    fn test_tabline_showcmd_width() {
        // Plenty of room
        assert_eq!(tabline_showcmd_width(80, 0, false), 10);
        assert_eq!(tabline_showcmd_width(80, 0, true), 10); // 80 - 0 - 3 = 77 > 10

        // Limited room
        assert_eq!(tabline_showcmd_width(20, 15, true), 2); // 20 - 15 - 3 = 2

        // No room
        assert_eq!(tabline_showcmd_width(20, 20, false), 0);
    }

    #[test]
    fn test_tabline_showcmd_col() {
        assert_eq!(tabline_showcmd_col(80, 10, false), 70);
        assert_eq!(tabline_showcmd_col(80, 10, true), 68); // 80 - 10 - 2
    }

    #[test]
    fn test_tab_render_info() {
        let info = TabRenderInfo::new(1, true);
        assert_eq!(info.tabnr, 1);
        assert_eq!(info.is_current, 1);
        assert_eq!(info.win_count, 0);
        assert_eq!(info.modified, 0);

        let info = TabRenderInfo::new(2, false);
        assert_eq!(info.tabnr, 2);
        assert_eq!(info.is_current, 0);
    }

    // Tests for Phase 4: Custom Window Redraw

    #[test]
    fn test_custom_redraw_mode() {
        assert_eq!(
            determine_custom_redraw_mode(true, false, false),
            CustomRedrawMode::Tabline
        );
        assert_eq!(
            determine_custom_redraw_mode(false, true, false),
            CustomRedrawMode::Winbar
        );
        assert_eq!(
            determine_custom_redraw_mode(false, false, true),
            CustomRedrawMode::Ruler
        );
        assert_eq!(
            determine_custom_redraw_mode(false, false, false),
            CustomRedrawMode::Statusline
        );
    }

    #[test]
    fn test_calc_ruler_col_offset() {
        // ru_col = 17, Columns = 80, maxwidth = 80
        // offset = 17 - (80 - 80) = 17
        // half = (80 + 1) / 2 = 40
        // result = max(17, 40) = 40
        assert_eq!(calc_ruler_col_offset(17, 80, 80), 40);

        // ru_col = 50, Columns = 80, maxwidth = 80
        // offset = 50 - (80 - 80) = 50
        // half = 40
        // result = max(50, 40) = 50
        assert_eq!(calc_ruler_col_offset(50, 80, 80), 50);
    }

    #[test]
    fn test_calc_ruler_maxwidth() {
        assert_eq!(calc_ruler_maxwidth(80, 40), 40);
        assert_eq!(calc_ruler_maxwidth(100, 30), 70);
    }

    #[test]
    fn test_should_skip_custom_redraw() {
        assert!(should_skip_custom_redraw(0));
        assert!(should_skip_custom_redraw(-1));
        assert!(!should_skip_custom_redraw(1));
        assert!(!should_skip_custom_redraw(80));
    }

    #[test]
    fn test_userhl_type() {
        // Default highlight
        assert_eq!(rs_userhl_type(0), 0);
        // syn_id2attr highlight
        assert_eq!(rs_userhl_type(-5), 1);
        // User highlight
        assert_eq!(rs_userhl_type(3), 2);
    }

    #[test]
    fn test_userhl_syn_id() {
        assert_eq!(rs_userhl_syn_id(-5), 5);
        assert_eq!(rs_userhl_syn_id(-1), 1);
    }

    #[test]
    fn test_userhl_index() {
        assert_eq!(rs_userhl_index(1), 0);
        assert_eq!(rs_userhl_index(9), 8);
    }

    // Tests for Phase 5: Public API Entry Points

    #[test]
    fn test_status_redraw_action_skip_busy() {
        let ctx = StatusRedrawContext {
            busy: 1,
            ..Default::default()
        };
        assert_eq!(decide_status_redraw(&ctx), StatusRedrawAction::Skip);
    }

    #[test]
    fn test_status_redraw_action_skip_wildmenu() {
        let ctx = StatusRedrawContext {
            wild_menu_showing: 1,
            ui_has_wildmenu: 0,
            ..Default::default()
        };
        assert_eq!(decide_status_redraw(&ctx), StatusRedrawAction::Skip);
    }

    #[test]
    fn test_status_redraw_action_no_cmdline() {
        let ctx = StatusRedrawContext {
            no_status_height: 1,
            is_stl_global: 0,
            is_curwin: 0,
            ..Default::default()
        };
        assert_eq!(decide_status_redraw(&ctx), StatusRedrawAction::NoCmdline);
    }

    #[test]
    fn test_status_redraw_action_defer() {
        let ctx = StatusRedrawContext {
            no_status_height: 0,
            is_redrawing: 0,
            ..Default::default()
        };
        assert_eq!(decide_status_redraw(&ctx), StatusRedrawAction::Defer);
    }

    #[test]
    fn test_status_redraw_action_use_custom() {
        let ctx = StatusRedrawContext {
            no_status_height: 0,
            is_redrawing: 1,
            has_local_stl: 1,
            ..Default::default()
        };
        assert_eq!(decide_status_redraw(&ctx), StatusRedrawAction::UseCustom);
    }

    #[test]
    fn test_winbar_redraw_action_skip_entered() {
        let ctx = WinbarRedrawContext {
            entered: 1,
            ..Default::default()
        };
        assert_eq!(decide_winbar_redraw(&ctx), WinbarRedrawAction::Skip);
    }

    #[test]
    fn test_winbar_redraw_action_skip_no_height() {
        let ctx = WinbarRedrawContext {
            no_winbar_height: 1,
            is_redrawing: 1,
            ..Default::default()
        };
        assert_eq!(decide_winbar_redraw(&ctx), WinbarRedrawAction::Skip);
    }

    #[test]
    fn test_winbar_redraw_action_use_custom() {
        let ctx = WinbarRedrawContext {
            no_winbar_height: 0,
            is_redrawing: 1,
            has_global_wbr: 1,
            ..Default::default()
        };
        assert_eq!(decide_winbar_redraw(&ctx), WinbarRedrawAction::UseCustom);
    }

    #[test]
    fn test_custom_status_redraw_action() {
        assert_eq!(
            rs_custom_status_redraw_action(1),
            CustomStatusRedrawAction::Skip as c_int
        );
        assert_eq!(
            rs_custom_status_redraw_action(0),
            CustomStatusRedrawAction::Redraw as c_int
        );
    }
}
