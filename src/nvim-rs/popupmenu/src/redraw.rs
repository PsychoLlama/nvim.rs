//! Popup menu redraw logic helpers.
//!
//! This module provides helper functions for drawing the popup menu,
//! including scrollbar calculations, row rendering, and grid management.

use std::ffi::c_int;

use crate::PUM_STATE;

/// Result of grid width calculation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumGridWidthResult {
    /// Total grid width.
    pub grid_width: c_int,
    /// Column offset for content.
    pub col_off: c_int,
    /// Whether extra space padding is added.
    pub extra_space: c_int,
}

/// Calculate the grid width and column offset for LTR mode.
///
/// # Arguments
/// * `pum_width` - Width of popup menu content
/// * `pum_col` - Column position
/// * `pum_scrollbar` - Whether scrollbar is present (0 or 1)
/// * `has_border` - Whether border is present (non-zero = yes)
///
/// Returns grid width, column offset, and extra space flag.
#[no_mangle]
pub const extern "C" fn rs_pum_grid_width_ltr(
    pum_width: c_int,
    pum_col: c_int,
    pum_scrollbar: c_int,
    has_border: c_int,
) -> PumGridWidthResult {
    let mut grid_width = pum_width;
    let min_col = 0;
    let extra_space = pum_col > min_col;
    let col_off = if extra_space { 1 } else { 0 };

    if extra_space {
        grid_width += 1;
    }

    // Add scrollbar width if present and no border
    if pum_scrollbar > 0 && has_border == 0 {
        grid_width += 1;
    }

    PumGridWidthResult {
        grid_width,
        col_off,
        extra_space: extra_space as c_int,
    }
}

/// Calculate the grid width and column offset for RTL mode.
///
/// # Arguments
/// * `pum_width` - Width of popup menu content
/// * `pum_col` - Column position
/// * `win_end_col` - End column of window
/// * `pum_scrollbar` - Whether scrollbar is present (0 or 1)
/// * `has_border` - Whether border is present (non-zero = yes)
///
/// Returns grid width, column offset, and extra space flag.
#[no_mangle]
pub const extern "C" fn rs_pum_grid_width_rtl(
    pum_width: c_int,
    pum_col: c_int,
    win_end_col: c_int,
    pum_scrollbar: c_int,
    has_border: c_int,
) -> PumGridWidthResult {
    let mut grid_width = pum_width;
    let mut col_off = pum_width - 1;
    let extra_space = pum_col < win_end_col - 1;

    if extra_space {
        grid_width += 1;
    }

    // Add scrollbar width if present and no border
    if pum_scrollbar > 0 && has_border == 0 {
        grid_width += 1;
        col_off += 1;
    }

    PumGridWidthResult {
        grid_width,
        col_off,
        extra_space: extra_space as c_int,
    }
}

/// Clamp `pum_first` to valid scroll range.
///
/// Ensures first visible item doesn't show empty space at bottom.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_clamp_first() {
    let pum_size = PUM_STATE.size;
    let pum_height = PUM_STATE.height;
    let pum_first = PUM_STATE.first;

    let scroll_range = pum_size - pum_height;
    if pum_first > scroll_range {
        PUM_STATE.first = scroll_range;
    }
}

/// Calculate the scroll range for the popup menu.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_scroll_range() -> c_int {
    PUM_STATE.size - PUM_STATE.height
}

/// Thumb position and height result.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumThumbInfo {
    /// Position of thumb (row index).
    pub pos: c_int,
    /// Height of thumb in rows.
    pub height: c_int,
}

/// Compute scrollbar thumb position and height.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_compute_thumb_from_state() -> PumThumbInfo {
    let pum_first = PUM_STATE.first;
    let pum_height = PUM_STATE.height;
    let pum_size = PUM_STATE.size;

    if pum_size <= pum_height {
        return PumThumbInfo {
            pos: 0,
            height: pum_height,
        };
    }

    let scroll_range = pum_size - pum_height;
    let mut thumb_height = pum_height * pum_height / pum_size;
    if thumb_height == 0 {
        thumb_height = 1;
    }

    let thumb_pos = (pum_first * (pum_height - thumb_height) + scroll_range / 2) / scroll_range;

    PumThumbInfo {
        pos: thumb_pos,
        height: thumb_height,
    }
}

/// Check if a row index is within the scrollbar thumb.
///
/// # Arguments
/// * `row` - Row index relative to popup start
/// * `thumb_pos` - Start position of thumb
/// * `thumb_height` - Height of thumb
///
/// Returns 1 if row is in thumb, 0 otherwise.
#[no_mangle]
pub const extern "C" fn rs_pum_row_in_thumb(
    row: c_int,
    thumb_pos: c_int,
    thumb_height: c_int,
) -> c_int {
    (row >= thumb_pos && row < thumb_pos + thumb_height) as c_int
}

/// Calculate the item index for a given row.
///
/// # Arguments
/// * `row` - Row index (0-based, relative to popup content start)
///
/// Returns the item index in the items array.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_row_to_item(row: c_int) -> c_int {
    row + PUM_STATE.first
}

/// Check if a given item index is selected.
///
/// # Arguments
/// * `item_idx` - Item index to check
///
/// Returns 1 if selected, 0 otherwise.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_item_is_selected(item_idx: c_int) -> c_int {
    (item_idx == PUM_STATE.selected) as c_int
}

/// Column widths for popup menu rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumColumnWidths {
    /// Width of text/abbr column.
    pub base_width: c_int,
    /// Width of kind column.
    pub kind_width: c_int,
    /// Width of extra/menu column.
    pub extra_width: c_int,
}

/// Get the current column widths for popup menu rendering.
///
/// # Safety
/// Calls C accessor functions for global state.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_get_column_widths() -> PumColumnWidths {
    PumColumnWidths {
        base_width: PUM_STATE.base_width,
        kind_width: PUM_STATE.kind_width,
        extra_width: PUM_STATE.extra_width,
    }
}

/// Row rendering state for one popup menu row.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumRowState {
    /// Item index in the items array.
    pub item_idx: c_int,
    /// Whether this item is selected.
    pub is_selected: c_int,
    /// Current grid column position.
    pub grid_col: c_int,
    /// Total width used so far.
    pub total_width: c_int,
    /// Whether truncation indicator is needed.
    pub needs_trunc: c_int,
}

/// Initialize row rendering state for a given row.
///
/// # Arguments
/// * `row` - Row index (0-based)
/// * `col_off` - Column offset for content
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_init_row_state(row: c_int, col_off: c_int) -> PumRowState {
    let item_idx = row + PUM_STATE.first;
    let is_selected = (item_idx == PUM_STATE.selected) as c_int;

    PumRowState {
        item_idx,
        is_selected,
        grid_col: col_off,
        total_width: 0,
        needs_trunc: 0,
    }
}

/// Calculate the fill position range for padding between columns.
///
/// # Arguments
/// * `col_off` - Column offset for content
/// * `grid_col` - Current grid column position
/// * `basic_width` - Width of first column
/// * `n` - Additional spacing needed
/// * `is_rl` - Whether right-to-left mode
///
/// Returns `(start_col, end_col)` for fill operation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumFillRange {
    /// Start column for fill.
    pub start: c_int,
    /// End column for fill (exclusive).
    pub end: c_int,
    /// New grid column position after fill.
    pub new_grid_col: c_int,
}

#[no_mangle]
pub const extern "C" fn rs_pum_fill_range(
    col_off: c_int,
    grid_col: c_int,
    basic_width: c_int,
    n: c_int,
    is_rl: c_int,
) -> PumFillRange {
    if is_rl != 0 {
        PumFillRange {
            start: col_off - basic_width - n + 1,
            end: grid_col + 1,
            new_grid_col: col_off - basic_width - n,
        }
    } else {
        PumFillRange {
            start: grid_col,
            end: col_off + basic_width + n,
            new_grid_col: col_off + basic_width + n,
        }
    }
}

/// Calculate the scrollbar column position.
///
/// # Arguments
/// * `col_off` - Column offset for content
/// * `pum_width` - Width of popup menu content
/// * `is_rl` - Whether right-to-left mode
///
/// Returns the column for scrollbar rendering.
#[no_mangle]
pub const extern "C" fn rs_pum_scrollbar_col(
    col_off: c_int,
    pum_width: c_int,
    is_rl: c_int,
) -> c_int {
    if is_rl != 0 {
        col_off - pum_width
    } else {
        col_off + pum_width
    }
}

/// Calculate spacing value 'n' for column padding.
///
/// This determines additional spacing between columns based on column
/// order and item types.
///
/// # Arguments
/// * `j` - Current column index (0, 1, or 2)
/// * `items_width_kind` - Width of kind column
/// * `last_is_abbr` - Whether last column is abbr type
/// * `order_j` - Type of column j in order
///
/// Returns spacing value.
#[no_mangle]
pub const extern "C" fn rs_pum_column_spacing(
    j: c_int,
    items_width_kind: c_int,
    last_is_abbr: c_int,
    order_j: c_int,
) -> c_int {
    // CPT_ABBR = 0
    const CPT_ABBR: c_int = 0;

    if j > 0 {
        items_width_kind + if last_is_abbr != 0 { 0 } else { 1 }
    } else if order_j == CPT_ABBR {
        1
    } else {
        0
    }
}

/// Check if we should stop rendering columns for this row.
///
/// # Arguments
/// * `j` - Current column index (0, 1, or 2)
/// * `next_is_empty` - Whether next column is empty
/// * `next_next_is_empty` - Whether column after next is empty
/// * `basic_width` - Width of first column
/// * `n` - Additional spacing
/// * `pum_width` - Total popup width
///
/// Returns 1 if should stop, 0 otherwise.
#[no_mangle]
pub const extern "C" fn rs_pum_should_stop_columns(
    j: c_int,
    next_is_empty: c_int,
    next_next_is_empty: c_int,
    basic_width: c_int,
    n: c_int,
    pum_width: c_int,
) -> c_int {
    let stop = (j == 2)
        || (next_is_empty != 0 && (j == 1 || (j == 0 && next_next_is_empty != 0)))
        || (basic_width + n >= pum_width);
    stop as c_int
}

/// Compute the truncation fill range at end of row.
///
/// # Arguments
/// * `col_off` - Column offset for content
/// * `grid_col` - Current grid column position
/// * `pum_width` - Total popup width
/// * `is_rl` - Whether right-to-left mode
///
/// Returns fill range for end of row.
#[no_mangle]
pub const extern "C" fn rs_pum_end_fill_range(
    col_off: c_int,
    grid_col: c_int,
    pum_width: c_int,
    is_rl: c_int,
) -> PumFillRange {
    if is_rl != 0 {
        let lcol = col_off - pum_width + 1;
        PumFillRange {
            start: lcol,
            end: grid_col + 1,
            new_grid_col: lcol,
        }
    } else {
        let rcol = col_off + pum_width;
        PumFillRange {
            start: grid_col,
            end: rcol,
            new_grid_col: rcol,
        }
    }
}

/// Get the column position for truncation indicator.
///
/// # Arguments
/// * `col_off` - Column offset for content
/// * `pum_width` - Total popup width
/// * `is_rl` - Whether right-to-left mode
///
/// Returns column for truncation indicator.
#[no_mangle]
pub const extern "C" fn rs_pum_trunc_col(col_off: c_int, pum_width: c_int, is_rl: c_int) -> c_int {
    if is_rl != 0 {
        col_off - pum_width + 1
    } else {
        col_off + pum_width - 1
    }
}

use std::ffi::{c_char, c_void};

use crate::item::{PumItemArray, CPT_ABBR};
use crate::render::hlf;

/// `schar_T` is `uint32_t`.
type ScharT = u32;

/// NUL character value.
const NUL: u8 = 0;
/// TAB character value.
const TAB: u8 = 0x09;

// C function declarations for redraw.
#[allow(dead_code)]
extern "C" {
    // Direct grid operations
    fn screengrid_line_start(grid: *mut crate::ScreenGrid, row: c_int, col: c_int);
    fn grid_line_fill(start: c_int, end: c_int, fillchar: ScharT, attr: c_int) -> c_int;
    fn grid_line_put_schar(col: c_int, sc: ScharT, attr: c_int);
    fn grid_line_flush();
    fn grid_assign_handle(grid: *mut crate::ScreenGrid);
    fn grid_alloc(grid: *mut crate::ScreenGrid, rows: c_int, cols: c_int, copy: bool, valid: bool);
    fn grid_invalidate(grid: *mut crate::ScreenGrid);
    fn ui_call_grid_resize(grid: i64, width: i64, height: i64);
    fn ui_comp_put_grid(
        grid: *mut crate::ScreenGrid,
        row: c_int,
        col: c_int,
        height: c_int,
        width: c_int,
        valid: bool,
        on_top: bool,
    ) -> bool;
    /// Wrapper: calls `ui_call_win_float_pos` for `pum_grid` using given params.
    fn nvim_pum_ui_call_win_float_pos(
        handle: c_int,
        anchor: *const c_char,
        anchor_grid: c_int,
        row: c_int,
        col: c_int,
        zindex: c_int,
        comp_index: c_int,
        comp_row: c_int,
        comp_col: c_int,
    );
    fn ui_has(what: c_int) -> bool;
    fn grid_line_puts(col: c_int, text: *const c_char, textlen: c_int, attr: c_int) -> c_int;

    // State accessors
    fn nvim_set_must_redraw_pum(val: c_int);

    // Text/string operations
    fn nvim_pum_curwin_end_col() -> c_int;
    fn nvim_pum_fcs_trunc(is_rl: c_int) -> ScharT;
    fn nvim_pum_schar_from_ascii(c: c_char) -> ScharT;
    fn transstr(s: *const c_char, untab: bool) -> *mut c_char;
    fn reverse_text(s: *mut c_char) -> *mut c_char;
    fn mb_string2cells(s: *const c_char) -> usize;
    fn ptr2cells(p: *const c_char) -> c_int;
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn xfree(ptr: *mut c_void);

    // Highlight operations
    fn nvim_win_hl_attr(wp: *mut crate::display::WinHandle, hlf: c_int) -> c_int;
    fn hl_combine_attr(char_attr: c_int, comb_attr: c_int) -> c_int;

    // Border operations
    fn nvim_pum_parse_border(has_scrollbar: c_int) -> *mut c_void;
    fn nvim_pum_border_cfg_has_border(cfg: *mut c_void) -> c_int;
    fn nvim_pum_border_cfg_is_shadow(cfg: *mut c_void) -> c_int;
    fn nvim_pum_border_cfg_has_border_chars(cfg: *mut c_void) -> c_int;
    fn nvim_pum_border_cfg_scrollbar_char(cfg: *mut c_void) -> ScharT;
    fn nvim_pum_border_cfg_scrollbar_attr(cfg: *mut c_void) -> c_int;
    fn nvim_pum_border_draw(cfg: *mut c_void);
    fn nvim_pum_border_cfg_free(cfg: *mut c_void);

    // These are Rust #[no_mangle] functions callable via C linkage
    fn rs_pum_get_item(array: *const PumItemArray, index: c_int, item_type: c_int)
        -> *const c_char;
    fn rs_pum_user_attr_combine(
        array: *const PumItemArray,
        idx: c_int,
        item_type: c_int,
        attr: c_int,
    ) -> c_int;
    fn rs_pum_compute_text_attrs(
        text: *mut c_char,
        hlf_id: c_int,
        user_hlattr: c_int,
    ) -> *mut c_int;
    fn rs_pum_grid_puts_with_attrs(
        col: c_int,
        cells: c_int,
        text: *const c_char,
        textlen: c_int,
        attrs: *const c_int,
    );
    fn rs_pum_border_width() -> c_int;
}

extern "C" {
    /// C global: `State` (current editor mode).
    static State: c_int;
    /// C global: `curwin` (current window pointer).
    static mut curwin: *mut crate::display::WinHandle;
    /// C global: `pum_grid`.
    static mut pum_grid: crate::ScreenGrid;
    /// C global: `linebuf_char`.
    static mut linebuf_char: *mut ScharT;
    /// C global: `linebuf_attr`.
    static mut linebuf_attr: *mut i32;
}

/// `MODE_CMDLINE` = 0x08.
const MODE_CMDLINE: c_int = 0x08;
/// `kZIndexCmdlinePopupMenu` = 250.
const K_Z_INDEX_CMDLINE_POPUP_MENU: c_int = 250;
/// `kUIMultigrid` = 6.
const K_UI_MULTIGRID: c_int = 6;

/// Redraw the popup menu using current `pum_first` and `pum_selected`.
///
/// This is the core rendering function that handles grid allocation,
/// border drawing, row-by-row text rendering with highlight attributes,
/// RTL support, scrollbar rendering, and truncation indicators.
///
/// # Safety
/// Calls numerous C accessor and grid functions.
#[export_name = "pum_redraw"]
#[allow(
    clippy::too_many_lines,
    clippy::cognitive_complexity,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr,
    clippy::if_then_some_else_none,
    clippy::bool_to_int_with_if,
    clippy::unnecessary_operation,
    clippy::collapsible_else_if
)]
pub unsafe extern "C" fn rs_pum_redraw() {
    let pum_rl = PUM_STATE.rl != 0;
    let pum_width = PUM_STATE.width;
    let pum_col = PUM_STATE.col;
    let pum_height = PUM_STATE.height;
    let pum_size = PUM_STATE.size;
    let pum_scrollbar = PUM_STATE.scrollbar;
    let pum_row = PUM_STATE.row;
    let pum_above = PUM_STATE.above != 0;
    let pum_selected = PUM_STATE.selected;

    let mut row = 0;
    let attr_scroll = nvim_win_hl_attr(curwin, hlf::HLF_PSB);
    let attr_thumb = nvim_win_hl_attr(curwin, hlf::HLF_PST);
    let fcs_trunc = nvim_pum_fcs_trunc(pum_rl as c_int);
    let fill_char = nvim_pum_schar_from_ascii(b' ' as c_char);

    //                         "word"   "kind"   "extra text"
    let hlfs_norm: [c_int; 3] = [hlf::HLF_PNI, hlf::HLF_PNK, hlf::HLF_PNX];
    let hlfs_sel: [c_int; 3] = [hlf::HLF_PSI, hlf::HLF_PSK, hlf::HLF_PSX];

    let border_width = rs_pum_border_width();

    // Calculate grid width and column offset
    let mut grid_width = pum_width;
    let (mut col_off, extra_space) = if pum_rl {
        let win_end_col = nvim_pum_curwin_end_col();
        (pum_width - 1, pum_col < win_end_col - 1)
    } else {
        let es = pum_col > 0;
        (if es { 1 } else { 0 }, es)
    };
    if extra_space {
        grid_width += 1;
    }

    // Parse border configuration (opaque handle)
    let border_cfg = nvim_pum_parse_border(pum_scrollbar);
    if border_cfg.is_null() {
        return;
    }
    let has_border = nvim_pum_border_cfg_has_border(border_cfg) != 0;
    let (border_char, border_attr) = if has_border && pum_scrollbar != 0 {
        (
            nvim_pum_border_cfg_scrollbar_char(border_cfg),
            nvim_pum_border_cfg_scrollbar_attr(border_cfg),
        )
    } else {
        (0, 0)
    };
    let has_border_chars = nvim_pum_border_cfg_has_border_chars(border_cfg) != 0;

    if pum_scrollbar > 0 && !has_border_chars {
        grid_width += 1;
        if pum_rl {
            col_off += 1;
        }
    }

    grid_assign_handle(&raw mut pum_grid);

    PUM_STATE.left_col = pum_col - col_off;
    let pum_left_col = pum_col - col_off;
    PUM_STATE.right_col = pum_left_col + grid_width;

    let moved = ui_comp_put_grid(
        &raw mut pum_grid,
        pum_row,
        pum_left_col,
        pum_height + border_width,
        grid_width + border_width,
        false,
        true,
    );
    let invalid_grid = moved || PUM_STATE.invalid != 0;
    PUM_STATE.invalid = 0;
    nvim_set_must_redraw_pum(0);

    if pum_grid.chars.is_null()
        || pum_grid.rows != pum_height + border_width
        || pum_grid.cols != grid_width + border_width
    {
        grid_alloc(
            &raw mut pum_grid,
            pum_height + border_width,
            grid_width + border_width,
            !invalid_grid,
            false,
        );
        #[allow(clippy::cast_lossless)]
        ui_call_grid_resize(
            pum_grid.handle as i64,
            pum_grid.cols as i64,
            pum_grid.rows as i64,
        );
    } else if invalid_grid {
        grid_invalidate(&raw mut pum_grid);
    }

    if ui_has(K_UI_MULTIGRID) {
        let anchor: &[u8] = if pum_above { b"SW\0" } else { b"NW\0" };
        let row_off = if pum_above { -pum_height } else { 0 };
        let anchor_grid = PUM_STATE.anchor_grid;
        let win_row_offset = PUM_STATE.win_row_offset;
        let win_col_offset = PUM_STATE.win_col_offset;
        #[allow(clippy::cast_possible_truncation)]
        nvim_pum_ui_call_win_float_pos(
            pum_grid.handle,
            anchor.as_ptr().cast(),
            anchor_grid,
            pum_row - row_off - win_row_offset,
            pum_left_col - win_col_offset,
            pum_grid.zindex,
            pum_grid.comp_index as c_int,
            pum_grid.comp_row,
            pum_grid.comp_col,
        );
    }

    let scroll_range = pum_size - pum_height;

    // Avoid border for mouse menu
    let mouse_menu = (State & MODE_CMDLINE) == 0 && pum_grid.zindex == K_Z_INDEX_CMDLINE_POPUP_MENU;
    if !mouse_menu && has_border_chars {
        nvim_pum_border_draw(border_cfg);
        if nvim_pum_border_cfg_is_shadow(border_cfg) == 0 {
            row += 1;
            col_off += 1;
        }
    }

    // Never display more than we have
    let pum_first = {
        let f = PUM_STATE.first;
        let clamped = if f > scroll_range { scroll_range } else { f };
        if clamped != f {
            PUM_STATE.first = clamped;
        }
        clamped
    };

    let mut thumb_pos = 0;
    let mut thumb_height = 1;
    if pum_scrollbar != 0 {
        thumb_height = pum_height * pum_height / pum_size;
        if thumb_height == 0 {
            thumb_height = 1;
        }
        thumb_pos = (pum_first * (pum_height - thumb_height) + scroll_range / 2) / scroll_range;
    }

    let pum_array = PUM_STATE.array.cast_const();

    // Main row rendering loop
    for i in 0..pum_height {
        let idx = i + pum_first;
        let selected = idx == pum_selected;
        let hlfs = if selected { &hlfs_sel } else { &hlfs_norm };
        let trunc_attr =
            nvim_win_hl_attr(curwin, if selected { hlf::HLF_PSI } else { hlf::HLF_PNI });
        let mut hlf = hlfs[0]; // start with "word" highlight
        let mut attr = nvim_win_hl_attr(curwin, hlf);
        attr = hl_combine_attr(nvim_win_hl_attr(curwin, hlf::HLF_PNI), attr);

        screengrid_line_start(&raw mut pum_grid, row, 0);

        // Prepend a space if there is room
        if extra_space {
            let space = b" \0";
            if pum_rl {
                grid_line_puts(col_off + 1, space.as_ptr().cast(), 1, attr);
            } else {
                grid_line_puts(col_off - 1, space.as_ptr().cast(), 1, attr);
            }
        }

        // Display each entry, use two spaces for a Tab.
        // Do this 3 times and order from p_cia
        let mut grid_col = col_off;
        let mut totwidth = 0;
        let mut need_fcs_trunc = false;

        let align_order = crate::item::rs_pum_get_current_align_order();
        let order = [align_order.first, align_order.second, align_order.third];
        let items_width_array = [
            PUM_STATE.base_width,
            PUM_STATE.kind_width,
            PUM_STATE.extra_width,
        ];
        let basic_width = items_width_array[order[0] as usize];
        let last_isabbr = order[2] == CPT_ABBR;
        let mut orig_attr: c_int = -1;

        for j in 0..3 {
            let item_type = order[j];
            hlf = hlfs[item_type as usize];
            attr = nvim_win_hl_attr(curwin, hlf);
            attr = hl_combine_attr(nvim_win_hl_attr(curwin, hlf::HLF_PNI), attr);
            orig_attr = attr;
            if item_type < 2 {
                // try combine attr with user custom
                attr = rs_pum_user_attr_combine(pum_array, idx, item_type, attr);
            }
            let mut width: c_int = 0;
            let mut s: *const c_char = std::ptr::null();
            let mut p: *mut c_char = rs_pum_get_item(pum_array, idx, item_type).cast_mut();

            let next_isempty =
                j + 1 >= 3 || rs_pum_get_item(pum_array, idx, order[j + 1]).is_null();

            if !p.is_null() {
                loop {
                    if s.is_null() {
                        s = p;
                    }
                    let w = ptr2cells(p);
                    if *p as u8 != NUL && *p as u8 != TAB && totwidth + w <= pum_width {
                        width += w;
                        let adv = utfc_ptr2len(p);
                        p = p.add(adv as usize);
                        continue;
                    }

                    // Display the text that fits or comes before a Tab.
                    let saved = *p;
                    if saved as u8 != NUL {
                        *p = 0;
                    }
                    let st = transstr(s, true);
                    if saved as u8 != NUL {
                        *p = saved;
                    }

                    let attrs: *mut c_int = if item_type == CPT_ABBR {
                        let item = &*pum_array.offset(idx as isize);
                        let user_hlattr = item.pum_user_abbr_hlattr;
                        rs_pum_compute_text_attrs(st, hlf, user_hlattr)
                    } else {
                        std::ptr::null_mut()
                    };

                    if pum_rl {
                        let rt = reverse_text(st);
                        let rt_start = rt;
                        let cells = mb_string2cells(rt) as c_int;
                        let mut rt_cur = rt;
                        let pad = if next_isempty { 0 } else { 2 };
                        if pum_width - totwidth < cells + pad {
                            need_fcs_trunc = true;
                        }

                        // Only draw the text that fits
                        let mut cur_cells = cells;
                        if grid_col - cur_cells < col_off - pum_width {
                            while grid_col - cur_cells < col_off - pum_width {
                                let c = ptr2cells(rt_cur);
                                cur_cells -= c;
                                let adv = utfc_ptr2len(rt_cur);
                                rt_cur = rt_cur.add(adv as usize);
                            }
                            if grid_col - cur_cells > col_off - pum_width {
                                // Most left character requires 2 cells but only 1 available.
                                rt_cur = rt_cur.sub(1);
                                *rt_cur = b'<' as c_char;
                                cur_cells += 1;
                            }
                        }

                        if attrs.is_null() {
                            grid_line_puts(grid_col - cur_cells + 1, rt_cur, -1, attr);
                        } else {
                            rs_pum_grid_puts_with_attrs(
                                grid_col - cur_cells + 1,
                                cur_cells,
                                rt_cur,
                                -1,
                                attrs,
                            );
                        }
                        xfree(rt_start.cast());
                        xfree(st.cast());
                        grid_col -= width;
                    } else {
                        let cells = mb_string2cells(st) as c_int;
                        let pad = if next_isempty { 0 } else { 2 };
                        if pum_width - totwidth < cells + pad {
                            need_fcs_trunc = true;
                        }

                        if attrs.is_null() {
                            grid_line_puts(grid_col, st, -1, attr);
                        } else {
                            rs_pum_grid_puts_with_attrs(grid_col, cells, st, -1, attrs);
                        }
                        xfree(st.cast());
                        grid_col += width;
                    }

                    if !attrs.is_null() {
                        xfree(attrs.cast());
                    }

                    if *p as u8 != TAB {
                        break;
                    }

                    // Display two spaces for a Tab.
                    let two_spaces = b"  \0";
                    if pum_rl {
                        grid_line_puts(grid_col - 1, two_spaces.as_ptr().cast(), 2, attr);
                        grid_col -= 2;
                    } else {
                        grid_line_puts(grid_col, two_spaces.as_ptr().cast(), 2, attr);
                        grid_col += 2;
                    }
                    totwidth += 2;
                    s = std::ptr::null(); // start text at next char
                    width = 0;

                    let adv = utfc_ptr2len(p);
                    p = p.add(adv as usize);
                }
            }

            // Calculate spacing
            let n = if j > 0 {
                items_width_array[order[1] as usize] + if last_isabbr { 0 } else { 1 }
            } else if order[j] == CPT_ABBR {
                1
            } else {
                0
            };

            // Stop when nothing more to display
            let next_next_isempty =
                j + 2 >= 3 || rs_pum_get_item(pum_array, idx, order[j + 2]).is_null();
            if j == 2
                || (next_isempty && (j == 1 || (j == 0 && next_next_isempty)))
                || (basic_width + n >= pum_width)
            {
                break;
            }

            // Fill space between columns
            let space_char = nvim_pum_schar_from_ascii(b' ' as c_char);
            if pum_rl {
                grid_line_fill(
                    col_off - basic_width - n + 1,
                    grid_col + 1,
                    space_char,
                    orig_attr,
                );
                grid_col = col_off - basic_width - n;
            } else {
                grid_line_fill(grid_col, col_off + basic_width + n, space_char, orig_attr);
                grid_col = col_off + basic_width + n;
            }
            totwidth = basic_width + n;
        }

        // Fill remaining space and handle truncation indicator
        let space_char = nvim_pum_schar_from_ascii(b' ' as c_char);
        if pum_rl {
            let lcol = col_off - pum_width + 1;
            grid_line_fill(lcol, grid_col + 1, space_char, orig_attr);
            if need_fcs_trunc {
                let trunc_char = if fcs_trunc == NUL as ScharT {
                    nvim_pum_schar_from_ascii(b'<' as c_char)
                } else {
                    fcs_trunc
                };
                *linebuf_char.add(lcol as usize) = trunc_char;
                *linebuf_attr.add(lcol as usize) = trunc_attr;
                if pum_width > 1 && *linebuf_char.add((lcol + 1) as usize) == NUL as ScharT {
                    *linebuf_char.add((lcol + 1) as usize) = space_char;
                }
            }
        } else {
            let rcol = col_off + pum_width;
            grid_line_fill(grid_col, rcol, space_char, orig_attr);
            if need_fcs_trunc {
                if pum_width > 1 && *linebuf_char.add((rcol - 1) as usize) == NUL as ScharT {
                    *linebuf_char.add((rcol - 2) as usize) = space_char;
                }
                let trunc_char = if fcs_trunc == NUL as ScharT {
                    nvim_pum_schar_from_ascii(b'>' as c_char)
                } else {
                    fcs_trunc
                };
                *linebuf_char.add((rcol - 1) as usize) = trunc_char;
                *linebuf_attr.add((rcol - 1) as usize) = trunc_attr;
            }
        }

        // Scrollbar
        if pum_scrollbar > 0 {
            let thumb = i >= thumb_pos && i < thumb_pos + thumb_height;
            let scrollbar_col = col_off + if pum_rl { -pum_width } else { pum_width };
            let sc = if has_border && !thumb {
                border_char
            } else {
                fill_char
            };
            let sc_attr = if thumb {
                attr_thumb
            } else if has_border {
                border_attr
            } else {
                attr_scroll
            };
            grid_line_put_schar(scrollbar_col, sc, sc_attr);
        }

        grid_line_flush();
        row += 1;
    }

    nvim_pum_border_cfg_free(border_cfg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_width_ltr_with_extra() {
        let result = rs_pum_grid_width_ltr(20, 5, 1, 0);
        assert_eq!(result.grid_width, 22); // 20 + 1 (extra) + 1 (scrollbar)
        assert_eq!(result.col_off, 1);
        assert_eq!(result.extra_space, 1);
    }

    #[test]
    fn test_grid_width_ltr_at_zero() {
        let result = rs_pum_grid_width_ltr(20, 0, 0, 0);
        assert_eq!(result.grid_width, 20);
        assert_eq!(result.col_off, 0);
        assert_eq!(result.extra_space, 0);
    }

    #[test]
    fn test_grid_width_rtl() {
        let result = rs_pum_grid_width_rtl(20, 30, 80, 1, 0);
        assert_eq!(result.grid_width, 22); // 20 + 1 (extra) + 1 (scrollbar)
        assert_eq!(result.col_off, 20); // 19 + 1 for scrollbar
        assert_eq!(result.extra_space, 1);
    }

    #[test]
    fn test_row_in_thumb() {
        assert_eq!(rs_pum_row_in_thumb(5, 4, 3), 1); // 5 is in [4, 7)
        assert_eq!(rs_pum_row_in_thumb(3, 4, 3), 0); // 3 is not in [4, 7)
        assert_eq!(rs_pum_row_in_thumb(7, 4, 3), 0); // 7 is not in [4, 7)
    }

    #[test]
    fn test_fill_range_ltr() {
        let result = rs_pum_fill_range(1, 10, 15, 2, 0);
        assert_eq!(result.start, 10);
        assert_eq!(result.end, 18); // 1 + 15 + 2
        assert_eq!(result.new_grid_col, 18);
    }

    #[test]
    fn test_fill_range_rtl() {
        let result = rs_pum_fill_range(25, 20, 15, 2, 1);
        assert_eq!(result.start, 9); // 25 - 15 - 2 + 1
        assert_eq!(result.end, 21);
        assert_eq!(result.new_grid_col, 8); // 25 - 15 - 2
    }

    #[test]
    fn test_scrollbar_col() {
        assert_eq!(rs_pum_scrollbar_col(1, 20, 0), 21); // LTR
        assert_eq!(rs_pum_scrollbar_col(25, 20, 1), 5); // RTL
    }

    #[test]
    fn test_column_spacing() {
        assert_eq!(rs_pum_column_spacing(0, 5, 0, 0), 1); // abbr column
        assert_eq!(rs_pum_column_spacing(0, 5, 0, 1), 0); // kind column
        assert_eq!(rs_pum_column_spacing(1, 5, 0, 0), 6); // j=1, not last_is_abbr
        assert_eq!(rs_pum_column_spacing(1, 5, 1, 0), 5); // j=1, last_is_abbr
    }

    #[test]
    fn test_should_stop_columns() {
        // j=2 always stops
        assert_eq!(rs_pum_should_stop_columns(2, 0, 0, 10, 2, 30), 1);
        // j=1 with next empty
        assert_eq!(rs_pum_should_stop_columns(1, 1, 0, 10, 2, 30), 1);
        // j=0 with next two empty
        assert_eq!(rs_pum_should_stop_columns(0, 1, 1, 10, 2, 30), 1);
        // width overflow
        assert_eq!(rs_pum_should_stop_columns(0, 0, 0, 20, 15, 30), 1);
        // normal continue
        assert_eq!(rs_pum_should_stop_columns(0, 0, 0, 10, 2, 30), 0);
    }

    #[test]
    fn test_trunc_col() {
        assert_eq!(rs_pum_trunc_col(1, 20, 0), 20); // LTR: col_off + pum_width - 1
        assert_eq!(rs_pum_trunc_col(25, 20, 1), 6); // RTL: col_off - pum_width + 1
    }
}
