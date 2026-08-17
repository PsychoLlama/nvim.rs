#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uint};

use crate::api::buffer::nvim_buf_set_lines;
use crate::api::private::helpers::{
    api_clear_error, api_free_array, arena_array, cstr_as_string, cstr_to_string,
};
use crate::api::win_config::parse_winborder;
use crate::autocmd::{block_autocmds, unblock_autocmds};
use crate::buffer::{bt_nofile, buf_clear};
use crate::charset::{ptr2cells, transstr, vim_strsize};
use crate::cmdexpand::{cmdline_compl_is_fuzzy, cmdline_compl_pattern};
use crate::drawscreen::{
    UPD_NOT_VALID, UPD_SOME_VALID, redraw_later, setcursor_mayforce, update_screen,
};
use crate::eval::typval::{tv_dict_add_bool, tv_dict_add_float, tv_dict_add_nr};
use crate::ex_cmds::{do_ecmd, prepare_tagpreview};
use crate::fuzzy::fuzzy_match_str_with_pos;
use crate::garray::ga_clear;
use crate::getchar::{vgetc, vungetc};
use crate::global_cell::GlobalCell;
use crate::grid::{
    get_win_by_grid_handle, grid_alloc, grid_assign_handle, grid_draw_border, grid_free,
    grid_invalidate, grid_line_fill, grid_line_flush, grid_line_put_schar, grid_line_puts,
    schar_from_ascii, schar_from_str, screengrid_line_start,
};
use crate::highlight::{hl_combine_attr, hl_get_ui_attr, win_hl_attr};
use crate::highlight_group::{
    HLF_PBR, HLF_PMNI, HLF_PMSI, HLF_PNI, HLF_PNK, HLF_PNX, HLF_PSB, HLF_PSI, HLF_PSK, HLF_PST,
    HLF_PSX, syn_check_group,
};
use crate::insexpand::{
    compl_match_curr_select, get_cot_flags, ins_compl_active, ins_compl_leader,
};
use crate::keycodes::{
    K_DOWN, K_LEFTDRAG, K_LEFTMOUSE, K_LEFTMOUSE_NM, K_MOUSEDOWN, K_MOUSEMOVE, K_MOUSEUP,
    K_RIGHTDRAG, K_RIGHTMOUSE, K_RIGHTRELEASE, K_UP,
};
use crate::main::{
    Columns, PumWant, RedrawingDisabled, Rows, State, cia_flags, cmdline_row, cmdline_win,
    cmdwin_type, curbuf, curtab, curwin, default_grid, e_menu_only_exists_in_another_mode,
    firstwin, g_do_tagpreview, hl_attr_active, linebuf_attr, linebuf_char, mouse_col, mouse_grid,
    mouse_row, must_redraw_pum, no_u_sync, p_mousemev, p_pb, p_ph, p_pmw, p_pumborder, p_pvh, p_pw,
    pum_grid, pum_want, textlock,
};
use crate::mbyte::{mb_string2cells, mb_strnicmp, utf_ptr2cells, utfc_ptr2len};
use crate::memory::{ARENA_EMPTY, arena_finish, arena_mem_free, strequal, xfree, xmalloc};
use crate::menu::{execute_menu, get_menu_mode_flag, menu_find};
use crate::message::emsg;
use crate::mouse::{MousePos, find_win_outer};
use crate::r#move::{update_topline, validate_cheight, validate_cursor, validate_cursor_col};
use crate::option::set_option_value_give_err;
use crate::options::{
    kOptBufhidden, kOptBuflisted, kOptBuftype, kOptCotFlagFuzzy, kOptCotFlagPopup,
    kOptCotFlagPreview, kOptDiff, kOptSwapfile, opt_winborder_values,
};
use crate::os::libc::{gettext, strchr, strlen};
use crate::plines::{plines_m_win, win_linetabsize};
use crate::state::MODE_CMDLINE;
use crate::strings::reverse_text;
use crate::types::ui::{kUICmdline, kUIMultigrid, kUIPopupmenu, kUIWildmenu};
use crate::types::{
    AlignTextPos, Array, Buffer, Error, Float, Integer, Object, OptInt, OptVal, OptValData,
    OptValType, String_0, VirtText, VirtTextChunk, WinConfig, WinSplit, WinStyle, Window, dict_T,
    exarg_T, float_T, handle_T, hlf_T, kBoolVarFalse, kBoolVarTrue, kObjectTypeArray,
    kObjectTypeBoolean, kObjectTypeString, linenr_T, lpos_T, object_data as C2Rust_Unnamed_12,
    pumitem_T, sattr_T, schar_T, size_t, tabpage_T, uint32_t, varnumber_T, vimmenu_T, win_T,
};
use crate::ui::{
    ui_call_grid_destroy, ui_call_grid_resize, ui_call_option_set, ui_call_popupmenu_hide,
    ui_call_popupmenu_select, ui_call_popupmenu_show, ui_call_win_close, ui_call_win_float_pos,
    ui_has, ui_pum_get_height, ui_pum_get_pos,
};
use crate::ui_compositor::{ui_comp_put_grid, ui_comp_remove_grid};
use crate::window::{
    goto_tabpage_tp, valid_tabpage, win_close, win_enter, win_setheight, win_valid,
};
use crate::winfloat::{win_config_float, win_float_create_preview, win_float_find_preview};

// The carve of the transpiled module; see each child's docs.
mod layout;
pub(crate) use self::layout::*;
mod draw;
pub use self::draw::*;
mod preview;
pub use self::preview::*;
mod menu;
pub use self::menu::*;

pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitLeft: WinSplit = 0;
pub const kZIndexCmdlinePopupMenu: c_uint = 250;
pub const kZIndexPopupMenu: c_uint = 100;
pub const kZIndexFloatDefault: c_uint = 50;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeBoolean: OptValType = 0;
pub const ECMD_ONE: c_int = 1;
pub const CPT_MENU: c_uint = 2;
pub const CPT_KIND: c_uint = 1;
pub const CPT_ABBR: c_uint = 0;
pub const OPT_LOCAL: c_uint = 2;
pub const ARRAY_DICT_INIT: Array = Array {
    size: 0,
    capacity: 0,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const OK: c_int = 1;
pub const NL: c_int = '\n' as c_int;
pub const CAR: c_int = '\r' as c_int;
pub const ESC: c_int = '\u{1b}' as c_int;
pub const DEFAULT_GRID_HANDLE: c_int = 1;

// The menu's state. Every one of these is written by one of the placement
// steps in `layout` and read by `draw`; they are separate cells rather than
// one struct because `pum_set_selected` re-enters the editor (autocommands,
// `update_screen`) between writes, and no borrow may span that.

/// The items being shown. Borrowed from the caller of [`pum_display`], or
/// owned by `pum_show_popupmenu`; null while the menu is down.
static pum_array: GlobalCell<*mut pumitem_T> =
    GlobalCell::new(::core::ptr::null_mut::<pumitem_T>());
/// Number of items in `pum_array`.
static pum_size: GlobalCell<c_int> = GlobalCell::new(0);
/// Index of the selected item, or -1.
static pum_selected: GlobalCell<c_int> = GlobalCell::new(0);
/// Index of the item on the menu's top row.
static pum_first: GlobalCell<c_int> = GlobalCell::new(0);

/// Number of rows the menu shows.
static pum_height: GlobalCell<c_int> = GlobalCell::new(0);
/// Cells the item text may use, excluding padding and the scrollbar.
static pum_width: GlobalCell<c_int> = GlobalCell::new(0);
/// Width of the widest `word`.
static pum_base_width: GlobalCell<c_int> = GlobalCell::new(0);
/// Width of the `kind` column, including its separating space.
static pum_kind_width: GlobalCell<c_int> = GlobalCell::new(0);
/// Width of the `menu` column, including its separating space.
static pum_extra_width: GlobalCell<c_int> = GlobalCell::new(0);
/// One when there is a scrollbar, zero when there is not.
static pum_scrollbar: GlobalCell<c_int> = GlobalCell::new(0);
/// The menu is drawn `'rightleft'`.
static pum_rl: GlobalCell<bool> = GlobalCell::new(false);

/// Handle of the grid the position below is relative to.
static pum_anchor_grid: GlobalCell<c_int> = GlobalCell::new(0);
/// Top row of the menu.
static pum_row: GlobalCell<c_int> = GlobalCell::new(0);
/// Left column of the menu, or its right column under `'rightleft'`.
static pum_col: GlobalCell<c_int> = GlobalCell::new(0);
/// Offsets from grid coordinates back to window-relative ones.
static pum_win_row_offset: GlobalCell<c_int> = GlobalCell::new(0);
static pum_win_col_offset: GlobalCell<c_int> = GlobalCell::new(0);
/// Left column before padding and the scrollbar, and the column after them.
static pum_left_col: GlobalCell<c_int> = GlobalCell::new(0);
static pum_right_col: GlobalCell<c_int> = GlobalCell::new(0);
/// The menu is above the cursor line rather than below it.
static pum_above: GlobalCell<bool> = GlobalCell::new(false);

/// The menu is wanted on screen.
static pum_is_visible: GlobalCell<bool> = GlobalCell::new(false);
/// The menu's grid (or the external UI's menu) is still up.
static pum_is_drawn: GlobalCell<bool> = GlobalCell::new(false);
/// The UI draws the menu itself; nothing here paints anything.
static pum_external: GlobalCell<bool> = GlobalCell::new(false);
/// The screen was cleared, so the whole grid has to be sent again.
static pum_invalid: GlobalCell<bool> = GlobalCell::new(false);

/// The items the menu is showing, empty while it is down.
///
/// `pum_array` is borrowed from whoever called [`pum_display`] and is only
/// read; the strings inside it are what the drawing code writes through (it
/// NUL-terminates an item in place to measure a prefix, then puts the byte
/// back), which is why a shared slice is honest here.
///
/// # Safety
/// The result must not be held across `pum_undisplay`, which drops the
/// caller's array.
#[inline]
unsafe fn pum_items() -> &'static [pumitem_T] {
    let array = pum_array.get();
    if array.is_null() {
        return &[];
    }
    // SAFETY: `pum_array` and `pum_size` are set together and the array
    // outlives the menu.
    unsafe { ::core::slice::from_raw_parts(array, pum_size.get() as usize) }
}

/// Cells `'pumborder'` costs on each side of the menu.
///
/// Zero for no border, one for the shadow style — which only darkens the
/// right and bottom edges — and two for any of the box styles.
///
/// # Safety
/// `'pumborder'` must be a live option string.
#[inline]
unsafe fn pum_border_width() -> c_int {
    // SAFETY: `p_pumborder` and the option's value table are editor-owned
    // NUL-terminated strings.
    unsafe {
        let border = p_pumborder.get();
        if *border == 0 || strequal(border, opt_winborder_values.get()[7]) {
            return 0;
        }
        if strequal(border, opt_winborder_values.get()[3]) {
            1
        } else {
            2
        }
    }
}

/// Where the menu is anchored: the position the placement is computed from.
struct PumAnchor {
    /// The window the menu belongs to. Null only for a cmdline menu with no
    /// cmdline window, which is why the placement code guards on it.
    target_win: *mut win_T,
    /// Grid row of the line the menu hangs off.
    win_row: c_int,
    /// Grid column the menu is aligned with.
    cursor_col: c_int,
    /// Rows the menu may not draw above/below.
    above_row: c_int,
    below_row: c_int,
}

/// Work out the anchor, and set `pum_anchor_grid` and the two offsets.
///
/// `cmd_startcol` is the column of the completed match, and only means
/// anything for a cmdline menu.
///
/// # Safety
/// `curwin` must be live and the cursor column validated.
unsafe fn pum_compute_anchor(cmd_startcol: c_int) -> PumAnchor {
    // SAFETY: `curwin`, `cmdline_win` and the window tree are the editor's.
    unsafe {
        let win = curwin.get();
        let cmdline = State.get() & MODE_CMDLINE != 0;
        let target_win = if cmdline { cmdline_win.get() } else { win };
        let mut above_row = 0;
        let mut below_row = if cmdline {
            cmdline_row.get()
        } else {
            cmdline_row
                .get()
                .max((*win).w_winrow + (*win).w_view_height)
        };

        pum_win_row_offset.set(0);
        pum_win_col_offset.set(0);

        let (mut win_row, mut cursor_col);
        if cmdline {
            // wildoptions=pum
            let cw = cmdline_win.get();
            win_row = if !cw.is_null() {
                (*cw).w_wrow
            } else if ui_has(kUICmdline) {
                0
            } else {
                cmdline_row.get()
            };
            cursor_col = if cw.is_null() {
                0
            } else {
                (*cw).w_config._cmdline_offset
            } + cmd_startcol;
            cursor_col %= if cw.is_null() {
                Columns.get()
            } else {
                (*cw).w_view_width
            };
            pum_anchor_grid.set(if ui_has(kUICmdline) {
                -1
            } else {
                DEFAULT_GRID_HANDLE
            });
        } else {
            // The start of the completed word.
            win_row = (*win).w_wrow;
            cursor_col = if pum_rl.get() {
                (*win).w_view_width - (*win).w_wcol - 1
            } else {
                (*win).w_wcol
            };
        }

        if !target_win.is_null() {
            pum_anchor_grid.set((*(*target_win).w_grid.target).handle as c_int);
            win_row += (*target_win).w_grid.row_offset;
            cursor_col += (*target_win).w_grid.col_offset;
            if (*target_win).w_grid.target != default_grid.ptr() {
                win_row += (*target_win).w_winrow;
                cursor_col += (*target_win).w_wincol;
                if ui_has(kUIMultigrid) {
                    pum_win_row_offset.set((*target_win).w_winrow);
                    pum_win_col_offset.set((*target_win).w_wincol);
                } else {
                    // ext_popupmenu always anchors to the default grid when
                    // multigrid is off.
                    pum_anchor_grid.set(DEFAULT_GRID_HANDLE);
                }
            }
        }

        // A preview window takes its space away from the menu.
        let mut pvwin = ::core::ptr::null_mut::<win_T>();
        let mut wp = firstwin.get();
        while !wp.is_null() {
            if (*wp).w_onebuf_opt.wo_pvw != 0 {
                pvwin = wp;
                break;
            }
            wp = (*wp).w_next;
        }
        if !pvwin.is_null() {
            if (*pvwin).w_winrow < (*win).w_winrow {
                above_row = (*pvwin).w_winrow + (*pvwin).w_height;
            } else if (*pvwin).w_winrow > (*win).w_winrow + (*win).w_height {
                below_row = (*pvwin).w_winrow;
            }
        }

        PumAnchor {
            target_win,
            win_row,
            cursor_col,
            above_row,
            below_row,
        }
    }
}

/// Hand the whole item list to a UI that draws the menu itself.
///
/// # Safety
/// `array` must hold `size` items whose strings are NUL-terminated.
unsafe fn pum_publish_external(
    array: *mut pumitem_T,
    size: c_int,
    selected: c_int,
    anchor: &PumAnchor,
) {
    // SAFETY: the arena owns the arrays until `arena_mem_free`, and
    // `cstr_as_string` borrows the item strings for the duration of the call.
    unsafe {
        let mut arena = ARENA_EMPTY;
        let mut arr = arena_array(&raw mut arena, size as size_t);
        for i in 0..size as isize {
            let src = &*array.offset(i);
            let mut item = arena_array(&raw mut arena, 4);
            for text in [src.pum_text, src.pum_kind, src.pum_extra, src.pum_info] {
                *item.items.add(item.size) = Object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_12 {
                        string: cstr_as_string(text),
                    },
                };
                item.size += 1;
            }
            *arr.items.add(arr.size) = Object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed_12 { array: item },
            };
            arr.size += 1;
        }
        ui_call_popupmenu_show(
            arr,
            selected as Integer,
            (anchor.win_row - pum_win_row_offset.get()) as Integer,
            (anchor.cursor_col - pum_win_col_offset.get()) as Integer,
            pum_anchor_grid.get() as Integer,
        );
        arena_mem_free(arena_finish(&raw mut arena));
    }
}

/// Show the popup menu with `array[size]`.
///
/// `array` must stay valid until [`pum_undisplay`]. `selected` is the
/// initially selected item, or -1 for none; `array_changed` says whether the
/// items differ from the last call, which is all an external UI needs to know
/// to redraw. `cmd_startcol` is the column of the completed match and only
/// applies in cmdline mode.
///
/// # Safety
/// `array` must hold `size` items with NUL-terminated strings and outlive the
/// menu. Autocommands run from here.
pub unsafe fn pum_display(
    array: *mut pumitem_T,
    size: c_int,
    selected: c_int,
    array_changed: bool,
    cmd_startcol: c_int,
) {
    // SAFETY: `array` is the caller's and outlives the menu.
    unsafe {
        if !pum_is_visible.get() {
            // The draw mode may only change while the menu is down, which
            // keeps everything below from having to handle both.
            pum_external.set(
                ui_has(kUIPopupmenu) || (State.get() & MODE_CMDLINE != 0 && ui_has(kUIWildmenu)),
            );
        }
        pum_rl.set(State.get() & MODE_CMDLINE == 0 && (*curwin.get()).w_onebuf_opt.wo_rl != 0);
        let border_width = pum_border_width();

        // Placing the menu can resize a window, which invalidates the
        // placement. Redo it at most twice: with little room the size keeps
        // changing.
        for redo_count in 0..=2 {
            // Mark the menu visible up front, so that `'cursorcolumn'` does
            // not set `must_redraw` under us.
            pum_is_visible.set(true);
            pum_is_drawn.set(true);
            validate_cursor_col(curwin.get());

            let anchor = pum_compute_anchor(cmd_startcol);

            if pum_external.get() {
                if !array_changed {
                    ui_call_popupmenu_select(selected as Integer);
                    return;
                }
                pum_publish_external(array, size, selected, &anchor);
            }

            pum_compute_vertical_placement(
                size,
                anchor.target_win,
                anchor.win_row,
                anchor.above_row,
                anchor.below_row,
                border_width,
            );

            // Do not display when there is only room for one line.
            if border_width == 0 && (pum_height.get() < 1 || (pum_height.get() == 1 && size > 1)) {
                return;
            }

            pum_array.set(array);
            // Set before returning, so `pum_set_event_info` sees the size.
            pum_size.set(size);
            if pum_external.get() {
                return;
            }

            pum_compute_size();
            // More items than room means a scrollbar.
            pum_scrollbar.set(c_int::from(pum_height.get() < size));
            pum_compute_horizontal_placement(anchor.target_win, anchor.cursor_col, border_width);

            if !pum_set_selected(selected, redo_count) {
                break;
            }
        }

        (*pum_grid.ptr()).zindex = if State.get() & MODE_CMDLINE != 0 {
            kZIndexCmdlinePopupMenu as c_int
        } else {
            kZIndexPopupMenu as c_int
        };
        pum_redraw();
    }
}

/// Take the menu down.
///
/// `immediate` tears the grid down now rather than at the next
/// [`pum_check_clear`], which is what a caller that is about to draw
/// something else in its place wants.
///
/// # Safety
/// Autocommands run when `immediate` closes an info window.
pub unsafe fn pum_undisplay(immediate: bool) {
    pum_is_visible.set(false);
    pum_array.set(::core::ptr::null_mut());
    must_redraw_pum.set(false);

    if immediate {
        // SAFETY: the caller accepts the window close.
        unsafe { pum_check_clear() };
    }
}

/// Free the menu's grid once it is no longer wanted.
///
/// Split from [`pum_undisplay`] because the menu is taken down in the middle
/// of a redraw, where the grid may not be freed yet.
///
/// # Safety
/// Closing the info window runs autocommands.
pub unsafe fn pum_check_clear() {
    // SAFETY: `pum_grid` is the editor's own grid.
    unsafe {
        if pum_is_visible.get() || !pum_is_drawn.get() {
            return;
        }
        if pum_external.get() {
            ui_call_popupmenu_hide();
        } else {
            ui_comp_remove_grid(pum_grid.ptr());
            if ui_has(kUIMultigrid) {
                ui_call_win_close((*pum_grid.ptr()).handle as Integer);
                ui_call_grid_destroy((*pum_grid.ptr()).handle as Integer);
            }
            // TODO(bfredl): consider keeping float grids allocated.
            grid_free(pum_grid.ptr());
        }
        pum_is_drawn.set(false);
        pum_external.set(false);

        let wp = win_float_find_preview();
        if !wp.is_null() {
            win_close(wp, false, false);
        }
    }
}

/// Scroll the menu back to the top. Nothing else is reset.
pub fn pum_clear() {
    pum_first.set(0);
}

/// Whether the menu is wanted on screen.
pub fn pum_visible() -> bool {
    pum_is_visible.get()
}

/// Whether the menu is on screen *and* drawn here rather than by the UI.
pub fn pum_drawn() -> bool {
    pum_visible() && !pum_external.get()
}

/// The screen was cleared: the whole grid has to be sent again.
pub fn pum_invalidate() {
    pum_invalid.set(true);
}

/// Ask for an item to be selected, from a UI that draws the menu itself.
///
/// The request is picked up by `insexpand` on the next key.
pub fn pum_ext_select_item(item: c_int, insert: bool, finish: bool) {
    if !pum_visible() || item < -1 || item >= pum_size.get() {
        return;
    }
    pum_want.set(PumWant {
        active: true,
        item,
        insert,
        finish,
    });
}

/// Rows the menu shows. Only meaningful while [`pum_visible`].
pub fn pum_get_height() -> c_int {
    if pum_external.get() {
        let ui_pum_height = ui_pum_get_height();
        if ui_pum_height != 0 {
            return ui_pum_height;
        }
    }
    pum_height.get()
}

/// Add the menu's geometry to `dict`, for `v:event` of `CompleteChanged`.
///
/// An external UI's own geometry wins when it has one.
///
/// # Safety
/// `dict` must be a live dictionary.
pub unsafe fn pum_set_event_info(dict: *mut dict_T) {
    // SAFETY: `dict` is live and the keys are static strings.
    unsafe {
        if !pum_visible() {
            return;
        }
        let (mut w, mut h, mut r, mut c) = (0.0, 0.0, 0.0, 0.0);
        if !ui_pum_get_pos(&raw mut w, &raw mut h, &raw mut r, &raw mut c) {
            w = f64::from(pum_width.get());
            h = f64::from(pum_height.get());
            r = f64::from(pum_row.get());
            c = f64::from(pum_col.get());
        }
        for (key, value) in [(c"height", h), (c"width", w), (c"row", r), (c"col", c)] {
            tv_dict_add_float(dict, key.as_ptr(), key.count_bytes(), value as float_T);
        }
        tv_dict_add_nr(dict, c"size".as_ptr(), 4, pum_size.get() as varnumber_T);
        tv_dict_add_bool(
            dict,
            c"scrollbar".as_ptr(),
            9,
            if pum_scrollbar.get() != 0 {
                kBoolVarTrue
            } else {
                kBoolVarFalse
            },
        );
    }
}

/// Tell a multigrid UI where the menu's grid sits.
///
/// The anchor is the corner the menu grows away from the anchor row: a menu
/// drawn above the cursor line is anchored by its bottom-left corner, so the
/// row reported is its last one.
///
/// # Safety
/// `pum_grid` must be allocated and its placement settled.
unsafe fn pum_send_float_pos() {
    // SAFETY: `pum_grid` is the editor's own grid.
    unsafe {
        let above = pum_above.get();
        let anchor = if above { c"SW" } else { c"NW" };
        let row_off = if above { -pum_height.get() } else { 0 };
        ui_call_win_float_pos(
            (*pum_grid.ptr()).handle as Integer,
            -1 as Window,
            cstr_as_string(anchor.as_ptr()),
            pum_anchor_grid.get() as Integer,
            (pum_row.get() - row_off - pum_win_row_offset.get()) as Float,
            (pum_left_col.get() - pum_win_col_offset.get()) as Float,
            false,
            (*pum_grid.ptr()).zindex as Integer,
            (*pum_grid.ptr()).comp_index as c_int as Integer,
            (*pum_grid.ptr()).comp_row as Integer,
            (*pum_grid.ptr()).comp_col as Integer,
        );
    }
}

/// Re-send the menu's float position if the compositor moved it since the
/// last redraw.
///
/// # Safety
/// Called from the UI flush, with the grid still allocated.
pub unsafe fn pum_ui_flush() {
    // SAFETY: `pum_grid` is the editor's own grid.
    unsafe {
        if ui_has(kUIMultigrid)
            && pum_is_drawn.get()
            && !pum_external.get()
            && (*pum_grid.ptr()).handle != 0
            && (*pum_grid.ptr()).pending_comp_index_update
        {
            pum_send_float_pos();
            (*pum_grid.ptr()).pending_comp_index_update = false;
        }
    }
}
