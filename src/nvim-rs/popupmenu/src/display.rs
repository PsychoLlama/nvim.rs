//! Popup menu display orchestration.
//!
//! This module provides helper functions for showing, hiding, and managing
//! the popup menu display state.

use std::ffi::{c_char, c_int, c_void};

use crate::PUM_STATE;

// ---- Minimal API types for nvim_pum_ext_show ----

/// Matches C `String` / `NvimString`.
#[repr(C)]
#[derive(Clone, Copy)]
struct ApiString {
    data: *mut c_char,
    size: usize,
}

/// Object type constants.
const K_OBJ_STRING: c_int = 4;
const K_OBJ_ARRAY: c_int = 5;

/// Object data union (sized to 8 bytes to cover all C variants).
#[repr(C)]
#[derive(Clone, Copy)]
union ObjData {
    string: ApiString,
    array: ApiArray,
    _integer: i64,
}

/// Matches C `Object`.
#[repr(C)]
#[derive(Clone, Copy)]
struct ApiObject {
    obj_type: c_int,
    data: ObjData,
}

impl ApiObject {
    const fn string(s: ApiString) -> Self {
        Self {
            obj_type: K_OBJ_STRING,
            data: ObjData { string: s },
        }
    }
    const fn array(a: ApiArray) -> Self {
        Self {
            obj_type: K_OBJ_ARRAY,
            data: ObjData { array: a },
        }
    }
}

/// Matches C `Array` kvec.
#[repr(C)]
#[derive(Clone, Copy)]
struct ApiArray {
    size: usize,
    capacity: usize,
    items: *mut ApiObject,
}

impl ApiArray {
    /// Push an item.
    ///
    /// # Safety
    /// Must have been allocated with sufficient capacity.
    unsafe fn push(&mut self, obj: ApiObject) {
        debug_assert!(self.size < self.capacity);
        *self.items.add(self.size) = obj;
        self.size += 1;
    }
}

/// Matches C `Arena` from `memory_defs.h`.
#[repr(C)]
struct ExtArena {
    cur_blk: *mut c_char,
    pos: usize,
    size: usize,
}

impl ExtArena {
    const fn empty() -> Self {
        Self {
            cur_blk: std::ptr::null_mut(),
            pos: 0,
            size: 0,
        }
    }
}

extern "C" {
    /// Allocate an Arena-backed Array with given capacity.
    fn arena_array(arena: *mut ExtArena, max_size: usize) -> ApiArray;
    /// Create a string alias (no copy) from a C string.
    fn cstr_as_string(s: *const c_char) -> ApiString;
    /// Send the `popupmenu_show` UI event.
    fn ui_call_popupmenu_show(items: ApiArray, selected: i64, row: i64, col: i64, grid: i64);
    /// Finish arena, returning memory block.
    fn arena_finish(arena: *mut ExtArena) -> *mut c_void;
    /// Free arena memory block.
    fn arena_mem_free(mem: *mut c_void);
}

// External functions needed (not PumState fields)
extern "C" {
    static Columns: c_int;
    static mut State: c_int;
    /// Set the `must_redraw_pum` global variable.
    fn nvim_set_must_redraw_pum(val: c_int);
}

// External UI functions.
extern "C" {
    /// Check if UI has a capability.
    fn ui_has(what: c_int) -> bool;
}

// Direct grid function declarations.
extern "C" {
    /// Compose grid (calls `ui_comp_compose_grid`).
    fn ui_comp_compose_grid(grid: *mut crate::ScreenGrid);
    /// Call `ui_call_win_float_pos` with pum parameters.
    fn nvim_pum_ui_call_win_float_pos(
        handle: c_int,
        anchor: *const std::ffi::c_char,
        anchor_grid: c_int,
        row: c_int,
        col: c_int,
        zindex: c_int,
        comp_index: c_int,
        comp_row: c_int,
        comp_col: c_int,
    );
}

/// UI capability for popup menu (kUIPopupmenu = 1).
const K_UI_POPUPMENU: c_int = 1;
/// UI capability for wildmenu (kUIWildmenu = 3).
const K_UI_WILDMENU: c_int = 3;
/// Mode flag for command line.
const MODE_CMDLINE: c_int = 0x08;

/// Result of display mode determination.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumDisplayMode {
    /// Whether to use external popup menu.
    pub external: c_int,
    /// Whether in right-to-left mode.
    pub rl: c_int,
}

/// Determine the display mode for the popup menu.
///
/// # Arguments
/// * `is_visible` - Whether the popup is currently visible
/// * `curwin_rl` - Whether the current window is right-to-left
///
/// # Safety
/// Calls C functions to check UI capabilities and state.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_determine_display_mode(
    is_visible: c_int,
    curwin_rl: c_int,
) -> PumDisplayMode {
    let state = State;
    let is_cmdline = (state & MODE_CMDLINE) != 0;

    // Only change draw mode when popup is not visible
    let external = if is_visible == 0 {
        let has_popupmenu = ui_has(K_UI_POPUPMENU);
        let has_wildmenu = ui_has(K_UI_WILDMENU);
        c_int::from(has_popupmenu || (is_cmdline && has_wildmenu))
    } else {
        PUM_STATE.external
    };

    // RL only applies in non-cmdline mode
    let rl = if is_cmdline { 0 } else { curwin_rl };

    PumDisplayMode { external, rl }
}

/// Mark the popup menu as visible (before any position calculations).
///
/// This should be called early in `pum_display` to prevent `must_redraw`
/// from being set when 'cursorcolumn' is on.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_mark_visible() {
    PUM_STATE.is_visible = 1;
    PUM_STATE.is_drawn = 1;
}

/// Set the external mode and RL flags for display.
///
/// # Arguments
/// * `external` - Whether to use external popup menu
/// * `rl` - Whether to use right-to-left mode
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_set_display_mode(external: c_int, rl: c_int) {
    PUM_STATE.external = external;
    PUM_STATE.rl = rl;
}

/// Undisplay the popup menu.
///
/// This marks the popup as not visible and clears the array pointer.
/// If `immediate` is true, also triggers clearing immediately.
///
/// # Arguments
/// * `immediate` - Whether to immediately clear the popup display
///
/// # Safety
/// Calls C accessor functions.
#[export_name = "pum_undisplay"]
pub unsafe extern "C" fn rs_pum_undisplay(immediate: bool) {
    PUM_STATE.is_visible = 0;
    PUM_STATE.array = std::ptr::null_mut();
    nvim_set_must_redraw_pum(0);

    if immediate {
        rs_pum_check_clear();
    }
}

/// Check if the popup menu should be cleared from display.
///
/// Returns 1 if the popup should be cleared (not visible but drawn), 0 otherwise.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_should_clear() -> c_int {
    c_int::from(PUM_STATE.is_visible == 0 && PUM_STATE.is_drawn != 0)
}

/// Mark the popup as cleared from display.
///
/// Call this after successfully clearing the popup display.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_mark_cleared() {
    PUM_STATE.is_drawn = 0;
    PUM_STATE.external = 0;
}

/// Clear the popup menu scroll position.
///
/// Resets `pum_first` to 0.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_clear_scroll() {
    PUM_STATE.first = 0;
}

/// Mark the popup menu as invalid (needs redraw).
///
/// Called when the screen was cleared.
///
/// # Safety
/// Calls C accessor function.
#[export_name = "pum_invalidate"]
pub unsafe extern "C" fn rs_pum_invalidate() {
    PUM_STATE.invalid = 1;
}

/// Check if the popup menu is marked invalid.
///
/// Returns 1 if invalid, 0 otherwise.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_is_invalid() -> c_int {
    PUM_STATE.invalid
}

/// Clear the invalid flag.
///
/// Call this after redrawing the popup.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_clear_invalid() {
    PUM_STATE.invalid = 0;
}

/// Result for external UI item selection.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumExtSelectResult {
    /// Whether the selection is valid.
    pub valid: c_int,
    /// The item index to select.
    pub item: c_int,
    /// Whether to insert the item.
    pub insert: c_int,
    /// Whether to finish completion.
    pub finish: c_int,
}

/// Validate and prepare external UI item selection.
///
/// # Arguments
/// * `item` - Item index to select (-1 for no selection)
/// * `insert` - Whether to insert the item
/// * `finish` - Whether to finish completion
///
/// Returns validation result with potentially adjusted values.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_validate_ext_select(
    item: c_int,
    insert: c_int,
    finish: c_int,
) -> PumExtSelectResult {
    let pum_size = PUM_STATE.size;
    let is_visible = PUM_STATE.is_visible != 0;

    // Check if selection is valid
    if !is_visible || item < -1 || item >= pum_size {
        return PumExtSelectResult {
            valid: 0,
            item,
            insert,
            finish,
        };
    }

    PumExtSelectResult {
        valid: 1,
        item,
        insert,
        finish,
    }
}

/// Check if the popup menu is using external display.
///
/// Returns 1 if external, 0 otherwise.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_is_external() -> c_int {
    PUM_STATE.external
}

/// Check if the popup needs a scrollbar.
///
/// Returns 1 if scrollbar is needed, 0 otherwise.
///
/// # Arguments
/// * `height` - Visible height
/// * `size` - Total number of items
#[no_mangle]
pub const extern "C" fn rs_pum_display_needs_scrollbar(height: c_int, size: c_int) -> c_int {
    (height < size) as c_int
}

/// Check if display should return early for external mode.
///
/// In external mode, after sending the items to the UI, we should return
/// early without doing internal rendering.
///
/// Returns 1 if should return early, 0 otherwise.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_should_return_external() -> c_int {
    PUM_STATE.external
}

/// Check if there is enough room to display the popup.
///
/// Returns 1 if there is enough room, 0 otherwise.
///
/// # Arguments
/// * `height` - Computed height
/// * `size` - Total number of items
/// * `border_width` - Border width (0 if no border)
#[no_mangle]
pub const extern "C" fn rs_pum_has_room(height: c_int, size: c_int, border_width: c_int) -> c_int {
    // Don't display when we only have room for one line
    if border_width == 0 && (height < 1 || (height == 1 && size > 1)) {
        return 0;
    }
    1
}

/// UI capability for multigrid mode (kUIMultigrid = 6).
const K_UI_MULTIGRID: c_int = 6;

// Direct C function declarations for check_clear.
extern "C" {
    /// Call `ui_call_popupmenu_hide()`.
    fn ui_call_popupmenu_hide();
    /// Remove grid from compositor.
    fn ui_comp_remove_grid(grid: *mut crate::ScreenGrid);
    /// Notify UI of window close.
    fn ui_call_win_close(grid: i64);
    /// Notify UI of grid destruction.
    fn ui_call_grid_destroy(grid: i64);
    /// Free a `ScreenGrid`.
    fn grid_free(grid: *mut crate::ScreenGrid);
    /// Find the floating preview window (returns NULL if none).
    fn win_float_find_preview() -> *mut WinHandle;
    /// Close a window.
    fn win_close(wp: *mut WinHandle, free_buf: bool, force: bool) -> c_int;
}

// Display orchestrator C accessor functions.
extern "C" {
    /// Validate cursor column in the given window.
    fn validate_cursor_col(wp: *mut WinHandle);
    // nvim_pum_ext_show is now implemented in Rust below.
    /// Send external popupmenu select event.
    fn ui_call_popupmenu_select(selected: i64);
    /// Get `w_p_rl` for a window.
    #[link_name = "nvim_win_get_p_rl"]
    fn nvim_win_get_w_p_rl(wp: *mut WinHandle) -> c_int;
    /// Set selected item (Rust function via extern "C").
    fn rs_pum_set_selected(n: c_int, repeat: c_int) -> c_int;
    /// Get border width from Rust.
    fn rs_pum_border_width() -> c_int;
    /// Compute item widths and write to `PUM_STATE` (Rust function via extern "C").
    fn rs_pum_compute_size(array: *const crate::item::PumItemArray);
    /// Return target window context after calling `validate_cheight`.
    fn nvim_pum_get_target_win_context(wp: *mut WinHandle) -> PumTargetWinContext;
    /// Return target window geometry fields.
    fn nvim_pum_get_target_win_geometry(wp: *mut WinHandle) -> PumTargetWinGeometry;
    /// Find the preview window and return above/below row adjustments.
    fn nvim_pum_find_pvwin() -> PumPvwinRows;
}

/// Context fields for the target window (for vertical placement).
#[repr(C)]
struct PumTargetWinContext {
    wrow: c_int,
    cline_row: c_int,
    cline_height: c_int,
}

/// Geometry fields for the target window.
#[repr(C)]
struct PumTargetWinGeometry {
    row_offset: c_int,
    col_offset: c_int,
    wrow: c_int,
    wcol: c_int,
    p_rl: c_int,
    view_width: c_int,
    view_height: c_int,
    winrow: c_int,
    wincol: c_int,
    grid_target_handle: c_int,
    grid_target_is_default: c_int,
    cmdline_offset: c_int,
}

/// Preview window row info.
#[repr(C)]
struct PumPvwinRows {
    above_row: c_int,
    below_row: c_int,
}

// C globals used by display.
extern "C" {
    /// C global: `curwin` (current window pointer).
    static mut curwin: *mut WinHandle;
    /// C global: `pum_grid` (the popup menu grid).
    static mut pum_grid: crate::ScreenGrid;
    /// C global: `cmdline_win` (window used for `ext_cmdline`, or `NULL`).
    static mut cmdline_win: *mut WinHandle;
    /// C global: `cmdline_row` (row of command line).
    static cmdline_row: c_int;
}

/// Result of display geometry computation.
struct PumDisplayGeometry {
    pum_win_row: c_int,
    cursor_col: c_int,
    anchor_grid: c_int,
    win_row_offset: c_int,
    win_col_offset: c_int,
    above_row: c_int,
    below_row: c_int,
}

/// Opaque handle to a `buf_T`.
#[repr(C)]
pub struct BufHandle {
    _private: [u8; 0],
}

/// Opaque handle to a `win_T`.
#[repr(C)]
pub struct WinHandle {
    _private: [u8; 0],
}

/// Recompose the popup menu grid.
///
/// Calls `ui_comp_compose_grid` on the `pum_grid` to recompose the area
/// under the popup menu. Needed when options affecting composition change
/// (e.g. 'pumblend').
///
/// # Safety
/// Calls C accessor functions.
#[export_name = "pum_recompose"]
pub unsafe extern "C" fn rs_pum_recompose() {
    ui_comp_compose_grid(&raw mut pum_grid);
}

/// Check and clear the popup menu display if needed.
///
/// If the popup is not visible but still drawn, tears down the grid and
/// closes the floating preview window. Handles both external and internal
/// popup display modes.
///
/// # Safety
/// Calls C accessor and UI functions.
#[export_name = "pum_check_clear"]
pub unsafe extern "C" fn rs_pum_check_clear() {
    let is_visible = PUM_STATE.is_visible != 0;
    let is_drawn = PUM_STATE.is_drawn != 0;

    if !is_visible && is_drawn {
        let is_external = PUM_STATE.external != 0;
        if is_external {
            ui_call_popupmenu_hide();
        } else {
            ui_comp_remove_grid(&raw mut pum_grid);
            if ui_has(K_UI_MULTIGRID) {
                ui_call_win_close(pum_grid.handle as i64);
                ui_call_grid_destroy(pum_grid.handle as i64);
            }
            grid_free(&raw mut pum_grid);
        }
        PUM_STATE.is_drawn = 0;
        PUM_STATE.external = 0;

        let wp = win_float_find_preview();
        if !wp.is_null() {
            win_close(wp, false, false);
        }
    }
}

/// Flush the popup menu UI position in multigrid mode.
///
/// Updates the floating window position for the popup menu grid when
/// there is a pending compositor index update. Only applies in multigrid
/// mode when the popup is drawn internally (not external).
///
/// # Safety
/// Calls C accessor functions and UI call wrappers.
#[export_name = "pum_ui_flush"]
pub unsafe extern "C" fn rs_pum_ui_flush() {
    let has_multigrid = ui_has(K_UI_MULTIGRID);
    let is_drawn = PUM_STATE.is_drawn != 0;
    let is_external = PUM_STATE.external != 0;
    let handle = pum_grid.handle;
    let pending = pum_grid.pending_comp_index_update != 0;

    if has_multigrid && is_drawn && !is_external && handle != 0 && pending {
        let pum_above = PUM_STATE.above != 0;
        let pum_height = PUM_STATE.height;
        let anchor = if pum_above {
            c"SW".as_ptr()
        } else {
            c"NW".as_ptr()
        };
        let row_off = if pum_above { -pum_height } else { 0 };
        let pum_row = PUM_STATE.row;
        let pum_left_col = PUM_STATE.left_col;
        let win_row_offset = PUM_STATE.win_row_offset;
        let win_col_offset = PUM_STATE.win_col_offset;
        let anchor_grid = PUM_STATE.anchor_grid;
        let zindex = pum_grid.zindex;
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let comp_index = pum_grid.comp_index as c_int;
        let comp_row = pum_grid.comp_row;
        let comp_col = pum_grid.comp_col;

        nvim_pum_ui_call_win_float_pos(
            handle,
            anchor,
            anchor_grid,
            pum_row - row_off - win_row_offset,
            pum_left_col - win_col_offset,
            zindex,
            comp_index,
            comp_row,
            comp_col,
        );
        pum_grid.pending_comp_index_update = 0;
    }
}

// rs_pum_preview_set_text: moved to preview.rs
// rs_pum_adjust_info_position: moved to preview.rs
// rs_pum_set_info: moved to preview.rs
// rs_pum_set_selected: moved to selection.rs

/// UI capability for cmdline mode (kUICmdline = 0).
const K_UI_CMDLINE: c_int = 0;
/// Default grid handle.
const DEFAULT_GRID_HANDLE: c_int = 1;

/// Compute display geometry (replaces `nvim_pum_compute_geometry`).
///
/// Returns `pum_win_row`, `cursor_col`, `anchor_grid`, `win_row/col` offsets, `above_row`,
/// `below_row`.
///
/// # Safety
/// Calls C accessor functions for `cmdline_win`, `curwin`, and state.
unsafe fn pum_compute_geometry(cmd_startcol: c_int) -> PumDisplayGeometry {
    let mut geom = PumDisplayGeometry {
        pum_win_row: 0,
        cursor_col: 0,
        anchor_grid: DEFAULT_GRID_HANDLE,
        win_row_offset: 0,
        win_col_offset: 0,
        above_row: 0,
        below_row: 0,
    };

    let state = State;
    let is_cmdline = (state & MODE_CMDLINE) != 0;

    // Compute below_row: max of cmdline_row and curwin bottom
    let cw = nvim_pum_get_target_win_geometry(curwin);
    let curwin_bottom = cw.winrow + cw.view_height;
    geom.below_row = if cmdline_row > curwin_bottom {
        cmdline_row
    } else {
        curwin_bottom
    };

    if is_cmdline {
        geom.below_row = cmdline_row;
    }

    let target_win = if is_cmdline { cmdline_win } else { curwin };

    if is_cmdline {
        if !cmdline_win.is_null() {
            let cw_geom = nvim_pum_get_target_win_geometry(cmdline_win);
            geom.pum_win_row = cw_geom.wrow;
            geom.cursor_col = cw_geom.cmdline_offset + cmd_startcol;
            geom.cursor_col %= cw_geom.view_width;
        } else if ui_has(K_UI_CMDLINE) {
            geom.pum_win_row = 0;
            geom.cursor_col = cmd_startcol % Columns;
            geom.anchor_grid = -1;
        } else {
            geom.pum_win_row = cmdline_row;
            geom.cursor_col = cmd_startcol % Columns;
        }
    } else {
        let cw_geom = nvim_pum_get_target_win_geometry(curwin);
        geom.pum_win_row = cw_geom.wrow;
        if PUM_STATE.rl != 0 {
            geom.cursor_col = cw_geom.view_width - cw_geom.wcol - 1;
        } else {
            geom.cursor_col = cw_geom.wcol;
        }
    }

    if !target_win.is_null() {
        let tw = nvim_pum_get_target_win_geometry(target_win);
        geom.anchor_grid = tw.grid_target_handle;
        geom.pum_win_row += tw.row_offset;
        geom.cursor_col += tw.col_offset;
        if tw.grid_target_is_default == 0 {
            geom.pum_win_row += tw.winrow;
            geom.cursor_col += tw.wincol;
            if ui_has(K_UI_MULTIGRID) {
                geom.win_row_offset = tw.winrow;
                geom.win_col_offset = tw.wincol;
            } else {
                geom.anchor_grid = DEFAULT_GRID_HANDLE;
            }
        }
    }

    geom
}

/// Compute vertical placement (replaces `nvim_pum_compute_vp`).
///
/// Writes `PUM_STATE.row`, `.height`, `.above`.
///
/// # Safety
/// Calls C accessor functions.
unsafe fn pum_compute_vp(
    size: c_int,
    pum_win_row: c_int,
    above_row: c_int,
    below_row: c_int,
    border_width: c_int,
) {
    let state = State;
    let is_cmdline = (state & MODE_CMDLINE) != 0;
    let target_win = if is_cmdline { cmdline_win } else { curwin };
    let has_target_win = !target_win.is_null();
    let (context_above, context_below) = if has_target_win {
        let ctx = nvim_pum_get_target_win_context(target_win);
        let above = ctx.wrow - ctx.cline_row;
        let below = ctx.cline_row + ctx.cline_height - ctx.wrow;
        (above, below)
    } else {
        (0, 0)
    };

    let result = crate::placement::rs_pum_compute_vertical(
        size,
        pum_win_row,
        above_row,
        below_row,
        border_width,
        cmdline_row,
        c_int::from(is_cmdline),
        c_int::from(has_target_win),
        context_above,
        context_below,
    );
    PUM_STATE.row = result.row;
    PUM_STATE.height = result.height;
    PUM_STATE.above = result.above;
}

/// Compute horizontal placement (replaces `nvim_pum_compute_hp`).
///
/// Writes `PUM_STATE.col`, `.width`.
///
/// # Safety
/// Calls C accessor functions.
unsafe fn pum_compute_hp(cursor_col: c_int) {
    let state = State;
    let is_cmdline = (state & MODE_CMDLINE) != 0;
    let target_win = if is_cmdline { cmdline_win } else { curwin };
    let max_col = if target_win.is_null() {
        Columns
    } else {
        let tw = nvim_pum_get_target_win_geometry(target_win);
        let win_right = tw.wincol + tw.view_width;
        if Columns > win_right {
            Columns
        } else {
            win_right
        }
    };

    let result = crate::placement::rs_pum_compute_horizontal(
        cursor_col,
        max_col,
        PUM_STATE.rl,
        PUM_STATE.scrollbar,
        PUM_STATE.base_width,
        PUM_STATE.kind_width,
        PUM_STATE.extra_width,
    );
    PUM_STATE.col = result.col;
    PUM_STATE.width = result.width;
}

/// Send the external popupmenu show event.
///
/// Builds an Arena-allocated nested Array from the popup items and
/// calls `ui_call_popupmenu_show`.
///
/// # Safety
/// `array` must point to at least `size` valid `PumItemArray` elements.
#[allow(clippy::cast_sign_loss, clippy::too_many_arguments)]
unsafe fn pum_ext_show(
    array: *mut crate::item::PumItemArray,
    size: c_int,
    selected: c_int,
    pum_win_row: c_int,
    cursor_col: c_int,
    anchor_grid: c_int,
    win_row_offset: c_int,
    win_col_offset: c_int,
) {
    let mut arena = ExtArena::empty();
    let mut arr = arena_array(&raw mut arena, size as usize);
    for i in 0..size as usize {
        let item = &*array.add(i);
        let mut entry = arena_array(&raw mut arena, 4);
        entry.push(ApiObject::string(cstr_as_string(item.pum_text)));
        entry.push(ApiObject::string(cstr_as_string(item.pum_kind)));
        entry.push(ApiObject::string(cstr_as_string(item.pum_extra)));
        entry.push(ApiObject::string(cstr_as_string(item.pum_info)));
        arr.push(ApiObject::array(entry));
    }
    ui_call_popupmenu_show(
        arr,
        i64::from(selected),
        i64::from(pum_win_row - win_row_offset),
        i64::from(cursor_col - win_col_offset),
        i64::from(anchor_grid),
    );
    arena_mem_free(arena_finish(&raw mut arena));
}

/// Display the popup menu.
///
/// Shows the popup menu with the given items array. Handles:
/// - Display mode determination (external vs internal)
/// - Geometry computation (position, size, anchor)
/// - External UI events (`popupmenu_show`/`select`)
/// - Internal rendering (vertical/horizontal placement, redraw)
/// - Preview window row adjustments
///
/// # Safety
/// `array` must be a valid `pumitem_T` array pointer with at least `size` elements.
#[export_name = "pum_display"]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_pum_display(
    array: *mut crate::item::PumItemArray,
    size: c_int,
    selected: c_int,
    array_changed: bool,
    cmd_startcol: c_int,
) {
    let mut redo_count: c_int = 0;

    // Determine display mode (external/rl) only when not already visible
    let is_visible = PUM_STATE.is_visible;
    let state = State;
    let is_cmdline = (state & MODE_CMDLINE) != 0;

    if is_visible == 0 {
        let has_popupmenu = ui_has(K_UI_POPUPMENU);
        let has_wildmenu = ui_has(K_UI_WILDMENU);
        PUM_STATE.external = c_int::from(has_popupmenu || (is_cmdline && has_wildmenu));
    }

    let curwin_rl = nvim_win_get_w_p_rl(curwin);
    PUM_STATE.rl = if is_cmdline { 0 } else { curwin_rl };

    let border_width = rs_pum_border_width();

    loop {
        // Mark as visible early to avoid must_redraw when 'cursorcolumn' is on
        PUM_STATE.is_visible = 1;
        PUM_STATE.is_drawn = 1;
        validate_cursor_col(curwin);

        // Compute geometry in Rust (handles target_win, cmdline_win, grid offsets)
        let geom = pum_compute_geometry(cmd_startcol);
        PUM_STATE.win_row_offset = geom.win_row_offset;
        PUM_STATE.win_col_offset = geom.win_col_offset;
        PUM_STATE.anchor_grid = geom.anchor_grid;

        let pum_win_row = geom.pum_win_row;
        let cursor_col = geom.cursor_col;

        if PUM_STATE.external != 0 {
            if array_changed {
                pum_ext_show(
                    array,
                    size,
                    selected,
                    pum_win_row,
                    cursor_col,
                    geom.anchor_grid,
                    geom.win_row_offset,
                    geom.win_col_offset,
                );
            } else {
                ui_call_popupmenu_select(i64::from(selected));
                return;
            }
        }

        // Find preview window and adjust above/below rows
        let mut above_row = geom.above_row;
        let mut below_row = geom.below_row;
        let pvwin = nvim_pum_find_pvwin();
        if pvwin.above_row > 0 {
            above_row = pvwin.above_row;
        }
        if pvwin.below_row > 0 {
            below_row = pvwin.below_row;
        }

        // Compute vertical placement in Rust (writes PUM_STATE.row, .height, .above)
        pum_compute_vp(size, pum_win_row, above_row, below_row, border_width);

        // Don't display when we only have room for one line
        let pum_height = PUM_STATE.height;
        if border_width == 0 && (pum_height < 1 || (pum_height == 1 && size > 1)) {
            return;
        }

        // Set array and size
        PUM_STATE.array = array;
        PUM_STATE.size = size;

        if PUM_STATE.external != 0 {
            return;
        }

        // Compute item widths (writes PUM_STATE.base_width, .kind_width, .extra_width)
        rs_pum_compute_size(PUM_STATE.array);

        // If there are more items than room we need a scrollbar
        let pum_height = PUM_STATE.height;
        PUM_STATE.scrollbar = c_int::from(pum_height < size);

        // Compute horizontal placement in Rust (writes PUM_STATE.col, .width)
        pum_compute_hp(cursor_col);

        // Adjust for border overflow
        let pum_col = PUM_STATE.col;
        let pum_width = PUM_STATE.width;
        let columns = Columns;
        if pum_col + border_width + pum_width > columns {
            PUM_STATE.col = pum_col - border_width;
        }

        // Set selected item and redraw. If the window size changed need to
        // redo the positioning. Limit to two times.
        let resized = rs_pum_set_selected(selected, redo_count) != 0;
        redo_count += 1;
        if !resized || redo_count > 2 {
            break;
        }
    }

    // kZIndexCmdlinePopupMenu = 250, kZIndexPopupMenu = 100
    pum_grid.zindex = if (State & MODE_CMDLINE) != 0 {
        250
    } else {
        100
    };
    crate::redraw::rs_pum_redraw();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_room_no_border() {
        // Not enough room
        assert_eq!(rs_pum_has_room(0, 5, 0), 0);
        assert_eq!(rs_pum_has_room(1, 5, 0), 0);
        // Enough room
        assert_eq!(rs_pum_has_room(1, 1, 0), 1);
        assert_eq!(rs_pum_has_room(5, 10, 0), 1);
    }

    #[test]
    fn test_has_room_with_border() {
        // With border, different rules apply
        assert_eq!(rs_pum_has_room(0, 5, 1), 1);
        assert_eq!(rs_pum_has_room(1, 5, 1), 1);
    }

    #[test]
    fn test_needs_scrollbar() {
        assert_eq!(rs_pum_display_needs_scrollbar(5, 10), 1);
        assert_eq!(rs_pum_display_needs_scrollbar(10, 10), 0);
        assert_eq!(rs_pum_display_needs_scrollbar(10, 5), 0);
    }
}
